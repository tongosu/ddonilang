use std::collections::VecDeque;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal;

use crate::input_tape::{
    key_index, mask_to_bytes, write_input_tape, InputEvent, InputEventKind, InputRecord, InputTape,
    KEY_REGISTRY_KEYS,
};

const LIVE_QUEUE_CAPACITY: usize = 1024;

#[derive(Clone, Copy, Debug)]
struct LiveKeyEvent {
    sequence: u64,
    key_bit: u16,
    kind: InputEventKind,
}

trait RawModeBackend {
    fn enable(&mut self) -> Result<(), String>;
    fn disable(&mut self) -> Result<(), String>;
}

struct ConsoleRawMode;
impl RawModeBackend for ConsoleRawMode {
    fn enable(&mut self) -> Result<(), String> {
        terminal::enable_raw_mode().map_err(|error| error.to_string())
    }
    fn disable(&mut self) -> Result<(), String> {
        terminal::disable_raw_mode().map_err(|error| error.to_string())
    }
}

struct RawModeGuard<B: RawModeBackend> {
    active: bool,
    backend: B,
}
impl<B: RawModeBackend> RawModeGuard<B> {
    fn acquire(mut backend: B) -> Result<Self, String> {
        backend.enable().map_err(|error| format!("E_SAM_LIVE_RAW 터미널 입력 모드 획득 실패: {error}"))?;
        Ok(Self { active: true, backend })
    }
    fn restore(&mut self) -> Result<(), String> {
        if self.active {
            self.backend.disable()
                .map_err(|error| format!("E_SAM_LIVE_RAW_RESTORE 터미널 입력 모드 복원 실패: {error}"))?;
            self.active = false;
        }
        Ok(())
    }
}
impl<B: RawModeBackend> Drop for RawModeGuard<B> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

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
    pub events: Vec<InputEvent>,
}

impl LiveInputTick {
    pub fn into_record(self, madi: u64) -> Result<InputRecord, String> {
        Ok(InputRecord {
            madi: u32::try_from(madi).map_err(|_| "E_SAM_TAPE_MADI_OVERFLOW 기록 가능한 마디 범위를 넘었습니다".to_string())?,
            held_mask: mask_to_bytes(self.held),
            events: self.events,
        })
    }
}

pub struct LiveInput {
    held: u16,
    queue: Arc<Mutex<VecDeque<LiveKeyEvent>>>,
    sequence: Arc<AtomicU64>,
    last_sequence: Option<u64>,
    observed_held: Arc<std::sync::atomic::AtomicU16>,
    failure: Arc<Mutex<Option<String>>>,
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
        let queue = Arc::new(Mutex::new(VecDeque::with_capacity(LIVE_QUEUE_CAPACITY)));
        let sequence = Arc::new(AtomicU64::new(0));
        let observed_held = Arc::new(std::sync::atomic::AtomicU16::new(0));
        let failure = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        let records = record_path.as_ref().map(|_| Vec::new());
        let mut input = Self {
            held: 0,
            queue: queue.clone(),
            sequence: sequence.clone(),
            last_sequence: None,
            observed_held: observed_held.clone(),
            failure: failure.clone(),
            record_path,
            records,
            madi_hz,
            ticker,
            stop: stop.clone(),
            thread: None,
            input_url: None,
        };
        let handle = match mode {
            LiveInputMode::Console => start_console_input(queue, sequence, failure, stop)?,
            LiveInputMode::Web => {
                input.input_url = Some(format!("http://{}:{}/input", host, port));
                start_web_input_server(host, port, queue, sequence, observed_held, failure, stop)?
            }
        };
        input.thread = Some(handle);
        Ok(input)
    }

    pub fn sample_tick(&mut self, madi: u64) -> Result<LiveInputTick, String> {
        self.ticker.sleep_until_tick(madi);
        if self.records.is_some() && u32::try_from(madi).is_err() {
            return Err("E_SAM_TAPE_MADI_OVERFLOW 기록 가능한 마디 범위를 넘었습니다".to_string());
        }
        if let Some(error) = recorded_failure(&self.failure)? {
            return Err(error);
        }
        if self.stop.load(Ordering::Acquire) {
            return Err("사용자가 실시간 입력을 종료했습니다. 성공 요약과 입력 테이프는 게시하지 않습니다".into());
        }
        let mut events = Vec::new();
        {
            let mut queue = self
                .queue
                .lock()
                .map_err(|_| "E_SAM_LIVE_QUEUE_LOCK".to_string())?;
            if let Some(error) = recorded_failure(&self.failure)? {
                return Err(error);
            }
            if self.stop.load(Ordering::Acquire) {
                return Err("사용자가 실시간 입력을 종료했습니다. 성공 요약과 입력 테이프는 게시하지 않습니다".into());
            }
            let mut previous = self.last_sequence;
            for event in queue.iter() {
                if previous.is_some_and(|sequence| event.sequence <= sequence) {
                    return Err("E_SAM_LIVE_INPUT_READ 기록된 사건 순번이 중복되거나 역전되었습니다".into());
                }
                previous = Some(event.sequence);
            }
            while let Some(event) = queue.pop_front() {
                events.push(event);
            }
        }
        let tick = apply_events(&mut self.held, &events);
        self.last_sequence = events.last().map(|event| event.sequence).or(self.last_sequence);
        self.observed_held.store(tick.held, Ordering::Release);
        if let Some(records) = self.records.as_mut() {
            records.push(InputRecord {
                madi: madi as u32,
                held_mask: mask_to_bytes(tick.held),
                events: tick.events.clone(),
            });
        }
        Ok(tick)
    }

    pub fn finish(mut self) -> Result<(), String> {
        self.stop_and_restore()?;
        if let (Some(path), Some(records)) = (self.record_path.take(), self.records.take()) {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            write_input_tape(
                &path,
                &InputTape {
                    version: 2,
                    madi_hz: self.madi_hz,
                    records,
                },
            )?;
        }
        Ok(())
    }

    pub fn abort(mut self) -> Result<(), String> {
        self.stop_and_restore()
    }

    /// Hold the candidate result until terminal restoration and tape finalization succeed.
    pub fn complete<T>(self, result: Result<T, String>) -> Result<T, String> {
        match result {
            Ok(value) => complete_after_restore(Ok(value), self.finish()),
            Err(primary) => complete_after_restore(Err(primary), self.abort()),
        }
    }

    /// Return sealed candidates for the caller's existing multi-artifact atomic publisher.
    /// No file or successful summary is published here.
    pub fn seal_result<T>(mut self, result: Result<T, String>) -> Result<(T, Option<(PathBuf, Vec<u8>)>), String> {
        let value = complete_after_restore(result, self.stop_and_restore())?;
        let tape = match (self.record_path.take(), self.records.take()) {
            (Some(path), Some(records)) => Some((path, crate::input_tape::encode_input_tape(&InputTape {
                version: 2, madi_hz: self.madi_hz, records,
            })?)),
            _ => None,
        };
        Ok((value, tape))
    }

    fn stop_and_restore(&mut self) -> Result<(), String> {
        if let Some(handle) = self.thread.take() {
            self.stop.store(true, Ordering::Relaxed);
            if handle.join().is_err() {
                record_failure(&self.failure, "E_SAM_LIVE_INPUT_READ 입력 스레드가 비정상 종료되었습니다".to_string());
            }
        }
        if let Some(error) = recorded_failure(&self.failure)? {
            return Err(error);
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

impl Drop for LiveInput {
    fn drop(&mut self) {
        let _ = self.stop_and_restore();
    }
}

fn complete_after_restore<T>(result: Result<T, String>, restore: Result<(), String>) -> Result<T, String> {
    match (result, restore) {
        (Ok(value), Ok(())) => Ok(value),
        (Ok(_), Err(error)) => Err(error),
        (Err(primary), Ok(())) => Err(primary),
        (Err(primary), Err(secondary)) => Err(format!("{primary}; 복원/종료 추가 오류: {secondary}")),
    }
}

fn recorded_failure(failure: &Mutex<Option<String>>) -> Result<Option<String>, String> {
    failure
        .lock()
        .map_err(|_| "E_SAM_LIVE_QUEUE_LOCK".to_string())
        .map(|failure| failure.clone())
}

fn record_failure(failure: &Mutex<Option<String>>, primary: String) {
    if let Ok(mut failure) = failure.lock() {
        if failure.is_none() {
            *failure = Some(primary);
        }
    }
}

fn record_restore_failure(failure: &Mutex<Option<String>>, restore: String) {
    if let Ok(mut failure) = failure.lock() {
        *failure = Some(match failure.take() {
            Some(primary) => format!("{primary}; 복원 추가 오류: {restore}"),
            None => restore,
        });
    }
}

fn apply_events(held: &mut u16, events: &[LiveKeyEvent]) -> LiveInputTick {
    let mut pressed = 0u16;
    let mut released = 0u16;
    for event in events {
        match event.kind {
            InputEventKind::Press => {
                *held |= event.key_bit;
                pressed |= event.key_bit;
            }
            InputEventKind::Release => {
                *held &= !event.key_bit;
                released |= event.key_bit;
            }
        }
    }
    LiveInputTick {
        held: *held,
        pressed,
        released,
        events: events.iter().map(|event| InputEvent {
            sequence: event.sequence, key_bit: event.key_bit, kind: event.kind,
        }).collect(),
    }
}

struct LiveTicker {
    start: Instant,
    step: Duration,
}

impl LiveTicker {
    fn new(madi_hz: u32) -> Result<Self, String> {
        if madi_hz == 0 {
            return Err("E_SAM_BAD_MADI_HZ --초당-마디(--madi-hz)는 0이 될 수 없습니다".to_string());
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
    queue: Arc<Mutex<VecDeque<LiveKeyEvent>>>,
    sequence: Arc<AtomicU64>,
    failure: Arc<Mutex<Option<String>>>,
    stop: Arc<AtomicBool>,
) -> Result<JoinHandle<()>, String> {
    require_terminal(io::stdin().is_terminal())?;
    let (ready_sender, ready_receiver) = std::sync::mpsc::sync_channel(1);
    let handle = thread::Builder::new()
        .name("ddn-sam-live-input".to_string())
        .spawn(move || {
            let mut guard = match RawModeGuard::acquire(ConsoleRawMode) {
                Ok(guard) => guard,
                Err(error) => {
                    let _ = ready_sender.send(Err(error));
                    return;
                }
            };
            if ready_sender.send(Ok(())).is_err() {
                return;
            }
            while !stop.load(Ordering::Relaxed) {
                match event::poll(Duration::from_millis(10)) {
                    Ok(true) => match event::read() {
                        Ok(Event::Key(key)) => {
                            if is_stop_key(&key) {
                                stop.store(true, Ordering::Relaxed);
                                break;
                            }
                            queue_key_event(key, &queue, &sequence, &failure);
                        }
                        Ok(_) => {}
                        Err(error) => {
                            record_failure(
                                &failure,
                                format!("E_SAM_LIVE_INPUT_READ 입력을 읽을 수 없습니다: {error}"),
                            );
                            break;
                        }
                    },
                    Ok(false) => {}
                    Err(error) => {
                        record_failure(
                            &failure,
                            format!("E_SAM_LIVE_INPUT_POLL 입력 대기 중 오류: {error}"),
                        );
                        break;
                    }
                }
            }
            if let Err(error) = guard.restore() {
                record_restore_failure(&failure, error);
            }
        })
        .map_err(|error| format!("E_SAM_LIVE_THREAD_SPAWN {error}"))?;
    match ready_receiver.recv() {
        Ok(Ok(())) => Ok(handle),
        Ok(Err(error)) => {
            let _ = handle.join();
            Err(error)
        }
        Err(error) => {
            let _ = handle.join();
            Err(format!("E_SAM_LIVE_RAW 입력 준비를 확인할 수 없습니다: {error}"))
        }
    }
}

fn queue_key_event(
    event: KeyEvent,
    queue: &Mutex<VecDeque<LiveKeyEvent>>,
    sequence: &AtomicU64,
    failure: &Mutex<Option<String>>,
) {
    if is_stop_key(&event) {
        return;
    }
    let Some(bit) = key_bit_from_code(event.code) else {
        return;
    };
    let kind = match event.kind {
        KeyEventKind::Press => InputEventKind::Press,
        KeyEventKind::Release => InputEventKind::Release,
        KeyEventKind::Repeat => return,
    };
    enqueue_event(queue, sequence, failure, bit, kind);
}

fn enqueue_event(
    queue: &Mutex<VecDeque<LiveKeyEvent>>,
    sequence: &AtomicU64,
    failure: &Mutex<Option<String>>,
    key_bit: u16,
    kind: InputEventKind,
) {
    let Ok(mut queue) = queue.lock() else {
        record_failure(failure, "E_SAM_LIVE_INPUT_READ 입력 대기열을 읽을 수 없습니다".to_string());
        return;
    };
    if recorded_failure(failure).ok().flatten().is_some() {
        return;
    }
    if queue.len() >= LIVE_QUEUE_CAPACITY {
        record_failure(
            failure,
            "E_SAM_LIVE_QUEUE_OVERFLOW 입력 대기열이 가득 찼습니다".to_string(),
        );
        return;
    }
    let sequence = match sequence.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
        value.checked_add(1)
    }) {
        Ok(value) => value,
        Err(_) => {
            record_failure(
                failure,
                "E_SAM_LIVE_SEQUENCE_OVERFLOW 입력 순번이 한계를 넘었습니다".to_string(),
            );
            return;
        }
    };
    queue.push_back(LiveKeyEvent {
        sequence,
        key_bit,
        kind,
    });
}

fn require_terminal(is_terminal: bool) -> Result<(), String> {
    if is_terminal {
        Ok(())
    } else {
        Err("E_SAM_LIVE_NOT_TTY 실시간 입력은 터미널 표준입력이 필요합니다".to_string())
    }
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

fn start_web_input_server(
    host: String,
    port: u16,
    queue: Arc<Mutex<VecDeque<LiveKeyEvent>>>,
    sequence: Arc<AtomicU64>,
    observed_held: Arc<std::sync::atomic::AtomicU16>,
    failure: Arc<Mutex<Option<String>>>,
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
                    let _ = handle_web_connection(
                        &mut stream,
                        &queue,
                        &sequence,
                        &observed_held,
                        &failure,
                    );
                }
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10))
                }
                Err(_) => break,
            }
        }
    }))
}

fn handle_web_connection(
    stream: &mut TcpStream,
    queue: &Mutex<VecDeque<LiveKeyEvent>>,
    sequence: &AtomicU64,
    observed_held: &std::sync::atomic::AtomicU16,
    failure: &Mutex<Option<String>>,
) -> Result<(), String> {
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
        let mut held = observed_held.load(Ordering::Acquire);
        {
            let pending = queue.lock().map_err(|_| "E_SAM_LIVE_INPUT_READ 입력 대기열을 읽을 수 없습니다".to_string())?;
            for event in pending.iter() {
                match event.kind {
                    InputEventKind::Press => held |= event.key_bit,
                    InputEventKind::Release => held &= !event.key_bit,
                }
            }
        }
        for index in 0..KEY_REGISTRY_KEYS.len() {
            let key_bit = 1u16 << index;
            if index != 6 && held & key_bit != 0 {
                enqueue_event(queue, sequence, failure, key_bit, InputEventKind::Release);
            }
        }
    } else if (kind.eq_ignore_ascii_case("down") || kind.eq_ignore_ascii_case("up"))
        && code.as_deref().and_then(key_bit_from_token).is_some()
    {
        enqueue_event(
            queue,
            sequence,
            failure,
            key_bit_from_token(code.as_deref().unwrap()).expect("checked key bit"),
            if kind.eq_ignore_ascii_case("down") {
                InputEventKind::Press
            } else {
                InputEventKind::Release
            },
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
    key_index(code).filter(|idx| *idx != 6).map(|idx| 1u16 << idx)
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

    struct FakeRawMode {
        calls: Arc<Mutex<Vec<&'static str>>>,
        acquire_failure: bool,
        restore_failure: bool,
    }

    fn scripted_input() -> LiveInput {
        LiveInput {
            held: 0,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            sequence: Arc::new(AtomicU64::new(0)),
            last_sequence: None,
            observed_held: Arc::new(std::sync::atomic::AtomicU16::new(0)),
            failure: Arc::new(Mutex::new(None)),
            record_path: Some(PathBuf::from("candidate-only.ddntape")),
            records: Some(Vec::new()),
            madi_hz: 60,
            ticker: LiveTicker { start: Instant::now(), step: Duration::ZERO },
            stop: Arc::new(AtomicBool::new(false)),
            thread: None,
            input_url: None,
        }
    }

    #[test]
    fn live_queue_runs_real_ddn_and_sealed_tape_replays_identically() {
        let source = include_str!("../../tools/teul-cli/tests/fixtures/cli_control_input_r1.ddn");
        let mut input = scripted_input();
        let result = crate::runtime_surface::run_summary_from_source_with_live_input(
            source, "control-input.ddn", Some(3), 0, Some("v1-core-v25"), 60,
            |madi| {
                queue_key_event(KeyEvent::new_with_kind(KeyCode::Right, KeyModifiers::NONE,
                    if madi == 1 { KeyEventKind::Release } else { KeyEventKind::Press }),
                    &input.queue, &input.sequence, &input.failure);
                input.sample_tick(madi)?.into_record(madi)
            },
        );
        let (live, (_, bytes)) = input.seal_result(result).map(|(result, tape)| (result, tape.unwrap())).expect("sealed candidates");
        let tape = crate::input_tape::decode_input_tape(&bytes).unwrap();
        assert_eq!(tape.version, 2);
        assert_eq!(tape.records.iter().flat_map(|record| record.events.iter()).map(|event| event.sequence).collect::<Vec<_>>(), [0, 1, 2]);
        let replay = crate::runtime_surface::run_summary_from_supported_source_profile_with_input_tape(
            source, "control-input.ddn", Some(3), Some("v1-core-v25"), &tape,
        ).unwrap();
        assert_eq!(live["output_log_texts"], serde_json::json!(["참", "거짓", "참"]));
        assert_eq!(live, replay);
    }

    #[test]
    fn latched_overflow_and_host_stop_publish_no_madi_or_tape_candidate() {
        let mut input = scripted_input();
        for _ in 0..=LIVE_QUEUE_CAPACITY {
            enqueue_event(&input.queue, &input.sequence, &input.failure, 1, InputEventKind::Press);
        }
        assert!(input.sample_tick(0).err().unwrap().contains("QUEUE_OVERFLOW"));
        assert_eq!(input.held, 0);
        assert!(input.records.as_ref().unwrap().is_empty());
        assert_eq!(input.queue.lock().unwrap().len(), LIVE_QUEUE_CAPACITY);
        assert!(input.seal_result(Ok("unpublished summary")).is_err());

        let mut input = scripted_input();
        input.stop.store(true, Ordering::Release);
        let result = input.sample_tick(0).map(|_| "unpublished summary");
        assert!(result.is_err());
        assert!(input.records.as_ref().unwrap().is_empty());
        assert!(input.seal_result(result).is_err());
    }
    impl RawModeBackend for FakeRawMode {
        fn enable(&mut self) -> Result<(), String> {
            self.calls.lock().unwrap().push("enable");
            if self.acquire_failure { Err("획득 시험".into()) } else { Ok(()) }
        }
        fn disable(&mut self) -> Result<(), String> {
            self.calls.lock().unwrap().push("disable");
            if self.restore_failure { Err("복원 시험".into()) } else { Ok(()) }
        }
    }

    #[test]
    fn raw_lifecycle_acquire_restore_and_unwind_are_owned() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let backend = || FakeRawMode { calls: calls.clone(), acquire_failure: false, restore_failure: false };
        {
            let mut guard = RawModeGuard::acquire(backend()).unwrap();
            guard.restore().unwrap();
        }
        assert_eq!(*calls.lock().unwrap(), ["enable", "disable"]);
        calls.lock().unwrap().clear();
        let _ = std::panic::catch_unwind(|| {
            let _guard = RawModeGuard::acquire(backend()).unwrap();
            panic!("입력 처리 unwind 시험");
        });
        assert_eq!(*calls.lock().unwrap(), ["enable", "disable"]);
        calls.lock().unwrap().clear();
        let error = RawModeGuard::acquire(FakeRawMode { acquire_failure: true, ..backend() }).err().unwrap();
        assert!(error.starts_with("E_SAM_LIVE_RAW "));
        assert_eq!(*calls.lock().unwrap(), ["enable"]);
    }

    #[test]
    fn raw_restore_failure_suppresses_success_and_preserves_primary_failure() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut guard = RawModeGuard::acquire(FakeRawMode {
            calls, acquire_failure: false, restore_failure: true,
        }).unwrap();
        let error = complete_after_restore(Ok("success summary"), guard.restore()).unwrap_err();
        assert!(error.starts_with("E_SAM_LIVE_RAW_RESTORE "));
        assert!(!error.contains("success summary"));
        let error = complete_after_restore::<()>(Err("E_PRIMARY 원래 실행 실패".into()), guard.restore()).unwrap_err();
        assert!(error.starts_with("E_PRIMARY "));
        assert!(error.contains("E_SAM_LIVE_RAW_RESTORE"));
    }

    #[test]
    fn host_stop_and_repeat_never_enter_world_queue() {
        let queue = Mutex::new(VecDeque::new());
        let sequence = AtomicU64::new(0);
        let failure = Mutex::new(None);
        for event in [
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            KeyEvent::new_with_kind(KeyCode::Left, KeyModifiers::NONE, KeyEventKind::Repeat),
        ] {
            queue_key_event(event, &queue, &sequence, &failure);
        }
        assert!(queue.lock().unwrap().is_empty());
        assert_eq!(sequence.load(Ordering::SeqCst), 0);
        assert!(recorded_failure(&failure).unwrap().is_none());
    }
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
    fn same_madi_fifo_preserves_transient_masks_and_final_held_state() {
        let press_then_release = [
            LiveKeyEvent {
                sequence: 0,
                key_bit: 1,
                kind: InputEventKind::Press,
            },
            LiveKeyEvent {
                sequence: 1,
                key_bit: 1,
                kind: InputEventKind::Release,
            },
        ];
        let mut held = 0;
        let tick = apply_events(&mut held, &press_then_release);
        assert_eq!((tick.held, tick.pressed, tick.released), (0, 1, 1));

        let release_then_press = [
            LiveKeyEvent {
                sequence: 2,
                key_bit: 1,
                kind: InputEventKind::Release,
            },
            LiveKeyEvent {
                sequence: 3,
                key_bit: 1,
                kind: InputEventKind::Press,
            },
        ];
        let tick = apply_events(&mut held, &release_then_press);
        assert_eq!((tick.held, tick.pressed, tick.released), (1, 1, 1));
    }

    #[test]
    fn queue_capacity_and_sequence_overflow_fail_before_any_event_is_applied() {
        let queue = Mutex::new(VecDeque::new());
        let sequence = AtomicU64::new(0);
        let failure = Mutex::new(None);
        for _ in 0..LIVE_QUEUE_CAPACITY {
            enqueue_event(&queue, &sequence, &failure, 1, InputEventKind::Press);
        }
        assert_eq!(queue.lock().expect("queue").len(), LIVE_QUEUE_CAPACITY);
        enqueue_event(&queue, &sequence, &failure, 1, InputEventKind::Release);
        assert!(recorded_failure(&failure)
            .expect("failure")
            .expect("overflow")
            .contains("QUEUE_OVERFLOW"));
        assert_eq!(queue.lock().expect("queue").len(), LIVE_QUEUE_CAPACITY);

        let queue = Mutex::new(VecDeque::new());
        let sequence = AtomicU64::new(u64::MAX);
        let failure = Mutex::new(None);
        enqueue_event(&queue, &sequence, &failure, 1, InputEventKind::Press);
        assert!(recorded_failure(&failure)
            .expect("failure")
            .expect("sequence overflow")
            .contains("SEQUENCE_OVERFLOW"));
        assert!(queue.lock().expect("queue").is_empty());
    }

    #[test]
    fn non_tty_live_console_is_rejected_before_raw_mode_acquisition() {
        assert!(require_terminal(false)
            .expect_err("non tty")
            .contains("NOT_TTY"));
        assert!(require_terminal(true).is_ok());
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
        let tick = input.sample_tick(0).expect("sample");
        assert_eq!((tick.held, tick.pressed, tick.released), (1, 1, 0));
        let up = send_request(
            port,
            "GET /input?code=ArrowLeft&kind=up HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        );
        assert!(up.starts_with("HTTP/1.1 200 OK"));
        let tick = input.sample_tick(0).expect("sample");
        assert_eq!((tick.held, tick.pressed, tick.released), (0, 0, 1));
        let down = send_request(
            port,
            "GET /input?code=ArrowLeft&kind=down HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        );
        assert!(down.starts_with("HTTP/1.1 200 OK"));
        assert_eq!(input.sample_tick(0).expect("sample").held, 1);
        let clear = send_request(
            port,
            "GET /input?kind=clear HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        );
        assert!(clear.starts_with("HTTP/1.1 200 OK"));
        let tick = input.sample_tick(0).expect("sample");
        assert_eq!((tick.held, tick.released), (0, 1));
        send_request(port, "GET /input?code=ArrowLeft&kind=down HTTP/1.1\r\nHost: localhost\r\n\r\n");
        send_request(port, "GET /input?kind=clear HTTP/1.1\r\nHost: localhost\r\n\r\n");
        let tick = input.sample_tick(0).expect("pending down then clear");
        assert_eq!((tick.held, tick.pressed, tick.released), (0, 1, 1));
        let options = send_request(port, "OPTIONS /input HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
        assert!(options.starts_with("HTTP/1.1 204 No Content"));
        assert!(options.contains("Access-Control-Allow-Methods: GET, OPTIONS"));
        input.finish().expect("finish live input");
    }
}
