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

use clap::ValueEnum;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal;

use crate::cli::input_tape::{
    key_index, mask_to_bytes, write_input_tape, InputRecord, InputTape,
};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum SamLiveMode {
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
        mode: SamLiveMode,
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
            SamLiveMode::Console => start_console_input(held, stop)?,
            SamLiveMode::Web => {
                let url = format!("http://{}:{}/input", host, port);
                input.input_url = Some(url);
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
            let tape = InputTape {
                madi_hz: self.madi_hz,
                records,
            };
            write_input_tape(&path, &tape)?;
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
        let step_ns = 1_000_000_000u64 / madi_hz as u64;
        let step = Duration::from_nanos(step_ns.max(1));
        Ok(Self {
            start: Instant::now(),
            step,
        })
    }

    fn sleep_until_tick(&self, tick: u64) {
        if tick == 0 {
            return;
        }
        let step_ns = self.step.as_nanos();
        let total_ns = step_ns.saturating_mul(tick as u128);
        let target = self
            .start
            .checked_add(Duration::from_nanos(total_ns.min(u64::MAX as u128) as u64));
        if let Some(deadline) = target {
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
    let handle = thread::spawn(move || {
        let poll_wait = Duration::from_millis(10);
        while !stop.load(Ordering::Relaxed) {
            match event::poll(poll_wait) {
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
    });
    Ok(handle)
}

fn update_held_from_key_event(event: KeyEvent, held: &AtomicU16) {
    let Some(bit) = key_bit_from_code(event.code) else {
        return;
    };
    match event.kind {
        KeyEventKind::Press | KeyEventKind::Repeat => update_held_mask(held, bit, true),
        KeyEventKind::Release => update_held_mask(held, bit, false),
    }
}

fn is_stop_key(event: &KeyEvent) -> bool {
    if matches!(event.code, KeyCode::Esc) {
        return true;
    }
    matches!(event.code, KeyCode::Char('c') | KeyCode::Char('C'))
        && event.modifiers.contains(KeyModifiers::CONTROL)
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
    let listener = TcpListener::bind(&addr)
        .map_err(|e| format!("E_SAM_LIVE_BIND {} {}", addr, e))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("E_SAM_LIVE_NONBLOCK {}", e))?;
    let handle = thread::spawn(move || {
        let poll_wait = Duration::from_millis(10);
        while !stop.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let _ = handle_web_connection(&mut stream, &held);
                }
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(poll_wait);
                }
                Err(_) => break,
            }
        }
    });
    Ok(handle)
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
    let mut lines = request.lines();
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");
    if method.eq_ignore_ascii_case("OPTIONS") {
        write_response(stream, "204 No Content", "")?;
        return Ok(());
    }
    if !method.eq_ignore_ascii_case("GET") {
        write_response(stream, "405 Method Not Allowed", "")?;
        return Ok(());
    }
    let (route, query) = split_query(path);
    if route != "/input" {
        write_response(stream, "404 Not Found", "")?;
        return Ok(());
    }
    let (code, kind) = parse_query(query);
    let kind = kind.unwrap_or_default();
    if kind.eq_ignore_ascii_case("clear") {
        held.store(0, Ordering::Relaxed);
    } else if kind.eq_ignore_ascii_case("down") || kind.eq_ignore_ascii_case("up") {
        if let Some(code) = code {
            if let Some(bit) = key_bit_from_token(&code) {
                update_held_mask(held, bit, kind.eq_ignore_ascii_case("down"));
            }
        }
    }
    write_response(stream, "200 OK", "ok")?;
    Ok(())
}

fn split_query(path: &str) -> (&str, &str) {
    if let Some((route, query)) = path.split_once('?') {
        (route, query)
    } else {
        (path, "")
    }
}

fn parse_query(query: &str) -> (Option<String>, Option<String>) {
    let mut code = None;
    let mut kind = None;
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");
        let value = decode_query_value(value);
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
    let mut idx = 0usize;
    while idx < bytes.len() {
        match bytes[idx] {
            b'+' => {
                out.push(' ');
                idx += 1;
            }
            b'%' if idx + 2 < bytes.len() => {
                let hi = bytes[idx + 1];
                let lo = bytes[idx + 2];
                if let (Some(hi), Some(lo)) = (from_hex(hi), from_hex(lo)) {
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
        b'a'..=b'f' => Some(10 + (byte - b'a')),
        b'A'..=b'F' => Some(10 + (byte - b'A')),
        _ => None,
    }
}

fn key_bit_from_token(code: &str) -> Option<u16> {
    key_index(code).map(|idx| 1u16 << idx)
}

fn write_response(stream: &mut TcpStream, status: &str, body: &str) -> Result<(), String> {
    let response = format!(
        "HTTP/1.1 {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        body.as_bytes().len(),
        body
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|e| e.to_string())
}
