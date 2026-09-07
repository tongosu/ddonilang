use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicU16, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal;

use crate::input_tape::{key_index, mask_to_bytes, write_input_tape, InputRecord, InputTape};

/// Shared product input transport modes. CLI spelling is intentionally owned by
/// the retained adapter, not this semantic owner.
#[derive(Clone, Copy, Debug)]
pub enum LiveInputMode {
    Console,
    Web,
}

pub struct LiveInputTick {
    pub held: u16,
    pub pressed: u16,
    pub released: u16,
}

pub struct LiveInput {
    held: Arc<AtomicU16>,
    last_mask: u16,
    record_path: Option<PathBuf>,
    records: Option<Vec<InputRecord>>,
    madi_hz: u32,
    ticker: LiveTicker,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    input_url: Option<String>,
}

impl LiveInput {
    pub fn new(
        mode: LiveInputMode,
        host: String,
        port: u16,
        madi_hz: u32,
        record_path: Option<PathBuf>,
    ) -> Result<Self, String> {
        let ticker = LiveTicker::new(madi_hz)?;
        let held = Arc::new(AtomicU16::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let records = record_path.as_ref().map(|_| Vec::new());
        let mut input = Self {
            held: held.clone(),
            last_mask: 0,
            record_path,
            records,
            madi_hz,
            ticker,
            stop: stop.clone(),
            thread: None,
            input_url: None,
        };
        let handle = match mode {
            LiveInputMode::Console => start_console_input(held, stop)?,
            LiveInputMode::Web => {
                input.input_url = Some(format!("http://{}:{}/input", host, port));
                start_web_input_server(host, port, held, stop)?
            }
        };
        input.thread = Some(handle);
        Ok(input)
    }

    pub fn sample_tick(&mut self, madi: u64) -> LiveInputTick {
        self.ticker.sleep_until_tick(madi);
        let held = self.held.load(Ordering::Relaxed);
        let pressed = (!self.last_mask) & held;
        let released = self.last_mask & !held;
        self.last_mask = held;
        if let Some(records) = self.records.as_mut() {
            if let Ok(madi_u32) = u32::try_from(madi) {
                records.push(InputRecord {
                    madi: madi_u32,
                    held_mask: mask_to_bytes(held),
                });
            }
        }
        LiveInputTick {
            held,
            pressed,
            released,
        }
    }

    pub fn finish(mut self) -> Result<(), String> {
        if let Some(handle) = self.thread.take() {
            self.stop.store(true, Ordering::Relaxed);
            let _ = handle.join();
        }
        if let (Some(path), Some(records)) = (self.record_path, self.records.take()) {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            write_input_tape(
                &path,
                &InputTape {
                    madi_hz: self.madi_hz,
                    records,
                },
            )?;
        }
        Ok(())
    }

    pub fn input_url(&self) -> Option<&str> {
        self.input_url.as_deref()
    }
    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        self.stop.clone()
    }
}

struct LiveTicker {
    start: Instant,
    step: Duration,
}

impl LiveTicker {
    fn new(madi_hz: u32) -> Result<Self, String> {
        if madi_hz == 0 {
            return Err("E_SAM_BAD_MADI_HZ madi-hz는 0이 될 수 없습니다".to_string());
        }
        Ok(Self {
            start: Instant::now(),
            step: Duration::from_nanos((1_000_000_000u64 / madi_hz as u64).max(1)),
        })
    }
    fn sleep_until_tick(&self, tick: u64) {
        if tick == 0 {
            return;
        }
        let total_ns = self.step.as_nanos().saturating_mul(tick as u128);
        if let Some(deadline) = self
            .start
            .checked_add(Duration::from_nanos(total_ns.min(u64::MAX as u128) as u64))
        {
            if let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
                thread::sleep(remaining);
            }
        }
    }
}

fn start_console_input(
    held: Arc<AtomicU16>,
    stop: Arc<AtomicBool>,
) -> Result<JoinHandle<()>, String> {
    terminal::enable_raw_mode().map_err(|e| format!("E_SAM_LIVE_RAW {}", e))?;
    Ok(thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            match event::poll(Duration::from_millis(10)) {
                Ok(true) => {
                    if let Ok(Event::Key(key)) = event::read() {
                        if is_stop_key(&key) {
                            stop.store(true, Ordering::Relaxed);
                            break;
                        }
                        update_held_from_key_event(key, &held);
                    }
                }
                Ok(false) => {}
                Err(_) => break,
            }
        }
        let _ = terminal::disable_raw_mode();
    }))
}

fn update_held_from_key_event(event: KeyEvent, held: &AtomicU16) {
    let Some(bit) = key_bit_from_code(event.code) else {
        return;
    };
    update_held_mask(
        held,
        bit,
        matches!(event.kind, KeyEventKind::Press | KeyEventKind::Repeat),
    );
}

fn is_stop_key(event: &KeyEvent) -> bool {
    matches!(event.code, KeyCode::Esc)
        || (matches!(event.code, KeyCode::Char('c') | KeyCode::Char('C'))
            && event.modifiers.contains(KeyModifiers::CONTROL))
}

fn key_bit_from_code(code: KeyCode) -> Option<u16> {
    let idx = match code {
        KeyCode::Left => 0,
        KeyCode::Right => 1,
        KeyCode::Down => 2,
        KeyCode::Up => 3,
        KeyCode::Char(' ') => 4,
        KeyCode::Enter => 5,
        KeyCode::Esc => 6,
        KeyCode::Char('z') | KeyCode::Char('Z') => 7,
        KeyCode::Char('x') | KeyCode::Char('X') => 8,
        _ => return None,
    };
    Some(1u16 << idx)
}

fn update_held_mask(held: &AtomicU16, bit: u16, pressed: bool) {
    loop {
        let current = held.load(Ordering::Relaxed);
        let next = if pressed {
            current | bit
        } else {
            current & !bit
        };
        if held
            .compare_exchange(current, next, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            break;
        }
    }
}

fn start_web_input_server(
    host: String,
    port: u16,
    held: Arc<AtomicU16>,
    stop: Arc<AtomicBool>,
) -> Result<JoinHandle<()>, String> {
    let addr = format!("{}:{}", host, port);
    let listener =
        TcpListener::bind(&addr).map_err(|e| format!("E_SAM_LIVE_BIND {} {}", addr, e))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("E_SAM_LIVE_NONBLOCK {}", e))?;
    Ok(thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let _ = handle_web_connection(&mut stream, &held);
                }
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10))
                }
                Err(_) => break,
            }
        }
    }))
}

fn handle_web_connection(stream: &mut TcpStream, held: &AtomicU16) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_millis(200)))
        .map_err(|e| e.to_string())?;
    let mut buf = [0u8; 2048];
    let size = match stream.read(&mut buf) {
        Ok(0) => return Ok(()),
        Ok(n) => n,
        Err(err) => return Err(err.to_string()),
    };
    let request = String::from_utf8_lossy(&buf[..size]);
    let request_line = request.lines().next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");
    if method.eq_ignore_ascii_case("OPTIONS") {
        return write_response(stream, "204 No Content", "");
    }
    if !method.eq_ignore_ascii_case("GET") {
        return write_response(stream, "405 Method Not Allowed", "");
    }
    let (route, query) = path.split_once('?').unwrap_or((path, ""));
    if route != "/input" {
        return write_response(stream, "404 Not Found", "");
    }
    let (code, kind) = parse_query(query);
    let kind = kind.unwrap_or_default();
    if kind.eq_ignore_ascii_case("clear") {
        held.store(0, Ordering::Relaxed);
    } else if (kind.eq_ignore_ascii_case("down") || kind.eq_ignore_ascii_case("up"))
        && code.as_deref().and_then(key_bit_from_token).is_some()
    {
        update_held_mask(
            held,
            key_bit_from_token(code.as_deref().unwrap()).expect("checked key bit"),
            kind.eq_ignore_ascii_case("down"),
        );
    }
    write_response(stream, "200 OK", "ok")
}

fn parse_query(query: &str) -> (Option<String>, Option<String>) {
    let mut code = None;
    let mut kind = None;
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = decode_query_value(parts.next().unwrap_or(""));
        match key {
            "code" => code = Some(value),
            "kind" => kind = Some(value),
            _ => {}
        }
    }
    (code, kind)
}

fn decode_query_value(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = String::with_capacity(bytes.len());
    let mut idx = 0;
    while idx < bytes.len() {
        match bytes[idx] {
            b'+' => {
                out.push(' ');
                idx += 1;
            }
            b'%' if idx + 2 < bytes.len() => {
                if let (Some(hi), Some(lo)) = (from_hex(bytes[idx + 1]), from_hex(bytes[idx + 2])) {
                    out.push((hi << 4 | lo) as char);
                    idx += 3;
                } else {
                    out.push('%');
                    idx += 1;
                }
            }
            ch => {
                out.push(ch as char);
                idx += 1;
            }
        }
    }
    out
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(10 + byte - b'a'),
        b'A'..=b'F' => Some(10 + byte - b'A'),
        _ => None,
    }
}
fn key_bit_from_token(code: &str) -> Option<u16> {
    key_index(code).map(|idx| 1u16 << idx)
}

fn write_response(stream: &mut TcpStream, status: &str, body: &str) -> Result<(), String> {
    let response = format!("HTTP/1.1 {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", status, body.as_bytes().len(), body);
    stream
        .write_all(response.as_bytes())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    fn free_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind free port");
        listener.local_addr().expect("local addr").port()
    }
    fn send_request(port: u16, request: &str) -> String {
        let mut stream =
            TcpStream::connect(("127.0.0.1", port)).expect("connect live input server");
        stream.write_all(request.as_bytes()).expect("write request");
        let mut response = String::new();
        stream.read_to_string(&mut response).expect("read response");
        response
    }
    #[test]
    fn std_grid_game_bogae_live_web_input_server_routes_update_mask() {
        let port = free_port();
        let mut input = LiveInput::new(
            LiveInputMode::Web,
            "127.0.0.1".to_string(),
            port,
            1000,
            None,
        )
        .expect("live input");
        assert_eq!(
            input.input_url(),
            Some(format!("http://127.0.0.1:{port}/input").as_str())
        );
        let down = send_request(
            port,
            "GET /input?code=ArrowLeft&kind=down HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        );
        assert!(down.starts_with("HTTP/1.1 200 OK"));
        assert!(down.contains("Access-Control-Allow-Origin: *"));
        let tick = input.sample_tick(0);
        assert_eq!((tick.held, tick.pressed, tick.released), (1, 1, 0));
        let up = send_request(
            port,
            "GET /input?code=ArrowLeft&kind=up HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        );
        assert!(up.starts_with("HTTP/1.1 200 OK"));
        let tick = input.sample_tick(0);
        assert_eq!((tick.held, tick.pressed, tick.released), (0, 0, 1));
        let down = send_request(
            port,
            "GET /input?code=ArrowLeft&kind=down HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        );
        assert!(down.starts_with("HTTP/1.1 200 OK"));
        assert_eq!(input.sample_tick(0).held, 1);
        let clear = send_request(
            port,
            "GET /input?kind=clear HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        );
        assert!(clear.starts_with("HTTP/1.1 200 OK"));
        let tick = input.sample_tick(0);
        assert_eq!((tick.held, tick.released), (0, 1));
        let options = send_request(port, "OPTIONS /input HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
        assert!(options.starts_with("HTTP/1.1 204 No Content"));
        assert!(options.contains("Access-Control-Allow-Methods: GET, OPTIONS"));
        input.finish().expect("finish live input");
    }
}
