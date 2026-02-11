use serde_json::{json, Value as JsonValue};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::detjson::{sha256_hex, write_text};
use crate::core::hash::SSOT_VERSION;

pub struct LoadSimOptions {
    pub clients: u64,
    pub ticks: u64,
    pub seed: u64,
    pub realms: u64,
    pub tick_hz: u64,
    pub threads: usize,
    pub out: Option<PathBuf>,
}

pub struct ServeOptions {
    pub world: PathBuf,
    pub threads: usize,
    pub input: Option<PathBuf>,
    pub out: Option<PathBuf>,
    pub realms: Option<u64>,
    pub input_format: InputFormat,
    pub listen_addr: Option<String>,
    pub listen_proto: ListenProtocol,
    pub listen_max_events: Option<u64>,
    pub listen_timeout_ms: Option<u64>,
    pub send_path: Option<PathBuf>,
    pub send_format: InputFormat,
}

#[derive(Clone, Debug)]
struct GatewayNetEvent {
    sender: String,
    seq: u64,
    order_key: String,
    payload: String,
    realm_id: u64,
}

#[derive(Clone, Copy, Debug)]
pub enum InputFormat {
    Auto,
    DetJson,
    Jsonl,
    Sam,
}

#[derive(Clone, Copy, Debug)]
pub enum ListenProtocol {
    Tcp,
    Udp,
}

pub fn run_serve(opts: ServeOptions) -> Result<(), String> {
    let world = &opts.world;
    if !world.exists() {
        return Err(format!("E_GATEWAY_WORLD_NOT_FOUND {}", world.display()));
    }
    let bytes = fs::read(world).map_err(|e| format!("E_GATEWAY_WORLD_READ {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hasher.finalize();
    println!("gateway_mode=serve");
    println!("gateway_threads={}", opts.threads);
    println!("gateway_world_hash=sha256:{}", hex::encode(digest));
    if opts.input.is_some() && opts.listen_addr.is_some() {
        return Err("E_GATEWAY_INPUT_CONFLICT input과 listen은 동시에 지정할 수 없습니다.".to_string());
    }
    if opts.send_path.is_some() && opts.listen_addr.is_none() {
        return Err("E_GATEWAY_SEND_REQUIRES_LISTEN listen 없이 send를 사용할 수 없습니다.".to_string());
    }
    if opts.send_path.is_some() && opts.listen_max_events.is_none() {
        return Err("E_GATEWAY_SEND_REQUIRES_MAX listen_max_events가 필요합니다.".to_string());
    }
    if opts.input.is_some() || opts.listen_addr.is_some() {
        let report = build_serve_report(&opts)?;
        let text = serde_json::to_string_pretty(&report)
            .map_err(|e| format!("E_GATEWAY_REPORT_JSON {}", e))?
            + "\n";
        let hash = sha256_hex(text.as_bytes());
        if let Some(out) = opts.out.as_deref() {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("E_GATEWAY_OUT_DIR {}", e))?;
            }
            write_text(out, &text)?;
            println!("gateway_report_written: {}", out.display());
        } else {
            println!("{}", text.trim_end());
        }
        println!("gateway_report_hash=sha256:{}", hash);
    }
    Ok(())
}

pub fn run_load_sim(opts: LoadSimOptions) -> Result<(), String> {
    if opts.clients == 0 || opts.ticks == 0 || opts.realms == 0 || opts.tick_hz == 0 {
        return Err("E_GATEWAY_INVALID_PARAMS clients/ticks/realms/tick_hz는 1 이상이어야 합니다.".to_string());
    }
    let width = digits(opts.clients.saturating_sub(1)).max(2);
    let events_total = opts
        .clients
        .checked_mul(opts.ticks)
        .ok_or_else(|| "E_GATEWAY_OVERFLOW events_total overflow".to_string())?;
    let throughput_events_per_sec = opts
        .clients
        .checked_mul(opts.tick_hz)
        .ok_or_else(|| "E_GATEWAY_OVERFLOW throughput overflow".to_string())?;

    let mut hashers = vec![Sha256::new(); opts.realms as usize];
    for tick in 0..opts.ticks {
        for client in 0..opts.clients {
            let sender = format!("c{:0width$}", client, width = width as usize);
            let seq = tick;
            let realm_id = (client % opts.realms) as usize;
            let payload = mix_payload(opts.seed, client, tick);
            let line = format!(
                "sender={sender}|seq={seq}|realm={realm_id}|payload={payload}\n"
            );
            hashers[realm_id].update(line.as_bytes());
        }
    }

    let mut final_state_hashes = Vec::new();
    for (realm_id, hasher) in hashers.into_iter().enumerate() {
        let digest = hasher.finalize();
        final_state_hashes.push(json!({
            "realm_id": realm_id,
            "state_hash": format!("sha256:{}", hex::encode(digest)),
        }));
    }

    let report = json!({
        "schema": "gateway.load_report.v1",
        "ssot_version": SSOT_VERSION,
        "clients": opts.clients,
        "ticks": opts.ticks,
        "seed": opts.seed,
        "events_total": events_total,
        "throughput_events_per_sec": throughput_events_per_sec,
        "order_rule": "sender_seq",
        "drop_duplicates": true,
        "thread_mode": format!("threads={}", opts.threads),
        "tick_hz": opts.tick_hz,
        "final_state_hashes": final_state_hashes,
    });

    let text = serde_json::to_string_pretty(&report)
        .map_err(|e| format!("E_GATEWAY_REPORT_JSON {}", e))?
        + "\n";
    let hash = sha256_hex(text.as_bytes());
    if let Some(out) = opts.out.as_deref() {
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("E_GATEWAY_OUT_DIR {}", e))?;
        }
        write_text(out, &text)?;
        println!("gateway_report_written: {}", out.display());
    } else {
        println!("{}", text.trim_end());
    }
    println!("gateway_report_hash=sha256:{}", hash);
    Ok(())
}

fn digits(mut value: u64) -> u64 {
    if value == 0 {
        return 1;
    }
    let mut count = 0;
    while value > 0 {
        count += 1;
        value /= 10;
    }
    count
}

fn mix_payload(seed: u64, client: u64, tick: u64) -> u64 {
    let mut x = seed ^ (client.wrapping_mul(0x9e3779b97f4a7c15)) ^ tick;
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ceb9fe1a85ec53);
    x ^= x >> 33;
    x
}

fn read_gateway_events(path: &Path, format: InputFormat) -> Result<Vec<GatewayNetEvent>, String> {
    match format {
        InputFormat::DetJson => read_detjson_events(path),
        InputFormat::Jsonl => read_jsonl_events(path),
        InputFormat::Sam => read_sam_input_events(path),
        InputFormat::Auto => read_auto_events(path),
    }
}

fn read_auto_events(path: &Path) -> Result<Vec<GatewayNetEvent>, String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext.eq_ignore_ascii_case("jsonl") {
        return read_jsonl_events(path);
    }
    let text = fs::read_to_string(path).map_err(|e| format!("E_GATEWAY_INPUT_READ {}", e))?;
    let value: JsonValue =
        serde_json::from_str(&text).map_err(|e| format!("E_GATEWAY_INPUT_PARSE {e}"))?;
    if let Some(schema) = value.get("schema").and_then(|v| v.as_str()) {
        if schema == "sam.input.v0" {
            return read_sam_input_events(path);
        }
    }
    read_detjson_events(path)
}

fn read_detjson_events(path: &Path) -> Result<Vec<GatewayNetEvent>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("E_GATEWAY_INPUT_READ {}", e))?;
    let value: JsonValue =
        serde_json::from_str(&text).map_err(|e| format!("E_GATEWAY_INPUT_PARSE {e}"))?;
    if let Some(schema) = value.get("schema").and_then(|v| v.as_str()) {
        if schema != "ddn.input_snapshot.v1" && schema != "sam.input.v0" {
            return Err(format!("E_GATEWAY_INPUT_SCHEMA {}", schema));
        }
    }
    extract_net_events_from_value(&value)
}

fn read_jsonl_events(path: &Path) -> Result<Vec<GatewayNetEvent>, String> {
    let file = fs::File::open(path).map_err(|e| format!("E_GATEWAY_INPUT_READ {}", e))?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("E_GATEWAY_INPUT_READ {}", e))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: JsonValue = serde_json::from_str(trimmed)
            .map_err(|e| format!("E_GATEWAY_INPUT_PARSE line {}: {e}", idx + 1))?;
        if value.get("net_events").is_some() {
            let nested = extract_net_events_from_value(&value)?;
            events.extend(nested);
        } else {
            events.push(parse_event_from_value(&value)?);
        }
    }
    Ok(events)
}

fn read_sam_input_events(path: &Path) -> Result<Vec<GatewayNetEvent>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("E_GATEWAY_INPUT_READ {}", e))?;
    let value: JsonValue =
        serde_json::from_str(&text).map_err(|e| format!("E_GATEWAY_INPUT_PARSE {e}"))?;
    if let Some(schema) = value.get("schema").and_then(|v| v.as_str()) {
        if schema != "sam.input.v0" {
            return Err(format!("E_GATEWAY_INPUT_SCHEMA {}", schema));
        }
    }
    let mut events = Vec::new();
    let Some(items) = value.get("events").and_then(|v| v.as_array()) else {
        return Ok(events);
    };
    for (idx, item) in items.iter().enumerate() {
        let sender = item
            .get("sender")
            .and_then(|v| v.as_str())
            .unwrap_or("sam")
            .to_string();
        let seq = item
            .get("t")
            .and_then(|v| v.as_u64())
            .unwrap_or(idx as u64);
        let order_key = item
            .get("order_key")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let realm_id = item
            .get("realm_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let payload = serde_json::to_string(item)
            .map_err(|e| format!("E_GATEWAY_INPUT_PAYLOAD {e}"))?;
        events.push(GatewayNetEvent {
            sender,
            seq,
            order_key,
            payload,
            realm_id,
        });
    }
    Ok(events)
}

fn extract_net_events_from_value(value: &JsonValue) -> Result<Vec<GatewayNetEvent>, String> {
    let mut events = Vec::new();
    let Some(net_events) = value.get("net_events").and_then(|v| v.as_array()) else {
        return Ok(events);
    };
    for event in net_events {
        events.push(parse_event_from_value(event)?);
    }
    Ok(events)
}

fn parse_event_from_value(value: &JsonValue) -> Result<GatewayNetEvent, String> {
    let sender = value
        .get("sender")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "E_GATEWAY_INPUT_FIELD sender".to_string())?
        .to_string();
    let seq = value
        .get("seq")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "E_GATEWAY_INPUT_FIELD seq".to_string())?;
    let order_key = value
        .get("order_key")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let realm_id = value
        .get("realm_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let payload = value
        .get("payload")
        .ok_or_else(|| "E_GATEWAY_INPUT_FIELD payload".to_string())?;
    let payload = serde_json::to_string(payload)
        .map_err(|e| format!("E_GATEWAY_INPUT_PAYLOAD {e}"))?;
    Ok(GatewayNetEvent {
        sender,
        seq,
        order_key,
        payload,
        realm_id,
    })
}

fn build_serve_report(opts: &ServeOptions) -> Result<JsonValue, String> {
    let events = if let Some(addr) = opts.listen_addr.as_deref() {
        let send_events = if let Some(send_path) = opts.send_path.as_deref() {
            Some(read_gateway_events(send_path, opts.send_format)?)
        } else {
            None
        };
        read_events_from_socket(
            addr,
            opts.listen_proto,
            opts.listen_max_events,
            opts.listen_timeout_ms,
            send_events,
        )?
    } else if let Some(input) = opts.input.as_deref() {
        read_gateway_events(input, opts.input_format)?
    } else {
        Vec::new()
    };
    let total = events.len() as u64;
    let (ordered, dropped) = order_and_dedupe_events(events);
    let realm_count = resolve_realm_count(&ordered, opts.realms)?;
    let final_state_hashes = compute_realm_hashes(&ordered, realm_count);
    let mut report = json!({
        "schema": "gateway.serve_report.v1",
        "ssot_version": SSOT_VERSION,
        "events_total": total,
        "events_ordered": ordered.len() as u64,
        "events_deduped": dropped,
        "order_rule": "sender_seq",
        "drop_duplicates": true,
        "realms": realm_count,
        "final_state_hashes": final_state_hashes,
    });
    if let Some(input) = opts.input.as_deref() {
        report["input_path"] = JsonValue::String(input.to_string_lossy().to_string());
        report["input_format"] = JsonValue::String(format!("{:?}", opts.input_format).to_lowercase());
    }
    if let Some(addr) = opts.listen_addr.as_deref() {
        report["listen_addr"] = JsonValue::String(addr.to_string());
        report["listen_proto"] = JsonValue::String(match opts.listen_proto {
            ListenProtocol::Tcp => "tcp",
            ListenProtocol::Udp => "udp",
        }.to_string());
        if let Some(max_events) = opts.listen_max_events {
            report["listen_max_events"] = JsonValue::Number(max_events.into());
        }
        if let Some(timeout_ms) = opts.listen_timeout_ms {
            report["listen_timeout_ms"] = JsonValue::Number(timeout_ms.into());
        }
    }
    Ok(report)
}

fn read_events_from_socket(
    addr: &str,
    proto: ListenProtocol,
    max_events: Option<u64>,
    timeout_ms: Option<u64>,
    send_events: Option<Vec<GatewayNetEvent>>,
) -> Result<Vec<GatewayNetEvent>, String> {
    match proto {
        ListenProtocol::Tcp => read_events_from_tcp(addr, max_events, timeout_ms, send_events),
        ListenProtocol::Udp => read_events_from_udp(addr, max_events, timeout_ms, send_events),
    }
}

fn read_events_from_tcp(
    addr: &str,
    max_events: Option<u64>,
    timeout_ms: Option<u64>,
    send_events: Option<Vec<GatewayNetEvent>>,
) -> Result<Vec<GatewayNetEvent>, String> {
    let listener = TcpListener::bind(addr).map_err(|e| format!("E_GATEWAY_LISTEN {}", e))?;
    let local_addr = listener
        .local_addr()
        .map_err(|e| format!("E_GATEWAY_LISTEN {}", e))?;
    let sender_handle = if let Some(events) = send_events {
        Some(std::thread::spawn(move || {
            if let Ok(mut stream) = TcpStream::connect(local_addr) {
                let _ = send_events_over_tcp(&mut stream, &events);
            }
        }))
    } else {
        None
    };
    let (stream, _) = listener.accept().map_err(|e| format!("E_GATEWAY_ACCEPT {}", e))?;
    if let Some(ms) = timeout_ms {
        stream
            .set_read_timeout(Some(Duration::from_millis(ms)))
            .map_err(|e| format!("E_GATEWAY_TIMEOUT {}", e))?;
    }
    let events = read_events_from_stream(stream, max_events)?;
    if let Some(handle) = sender_handle {
        let _ = handle.join();
    }
    Ok(events)
}

fn read_events_from_stream(mut stream: TcpStream, max_events: Option<u64>) -> Result<Vec<GatewayNetEvent>, String> {
    let mut events = Vec::new();
    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let value: JsonValue = serde_json::from_str(trimmed)
                    .map_err(|e| format!("E_GATEWAY_INPUT_PARSE {e}"))?;
                events.push(parse_event_from_value(&value)?);
                if let Some(limit) = max_events {
                    if events.len() as u64 >= limit {
                        break;
                    }
                }
            }
            Err(err) => {
                if err.kind() == std::io::ErrorKind::WouldBlock
                    || err.kind() == std::io::ErrorKind::TimedOut
                {
                    break;
                }
                return Err(format!("E_GATEWAY_INPUT_READ {}", err));
            }
        }
    }
    Ok(events)
}

fn read_events_from_udp(
    addr: &str,
    max_events: Option<u64>,
    timeout_ms: Option<u64>,
    send_events: Option<Vec<GatewayNetEvent>>,
) -> Result<Vec<GatewayNetEvent>, String> {
    let socket = UdpSocket::bind(addr).map_err(|e| format!("E_GATEWAY_LISTEN {}", e))?;
    let local_addr = socket
        .local_addr()
        .map_err(|e| format!("E_GATEWAY_LISTEN {}", e))?;
    let sender_handle = if let Some(events) = send_events {
        Some(std::thread::spawn(move || {
            if let Ok(sock) = UdpSocket::bind("127.0.0.1:0") {
                let _ = send_events_over_udp(&sock, local_addr, &events);
            }
        }))
    } else {
        None
    };
    if let Some(ms) = timeout_ms {
        socket
            .set_read_timeout(Some(Duration::from_millis(ms)))
            .map_err(|e| format!("E_GATEWAY_TIMEOUT {}", e))?;
    }
    let mut events = Vec::new();
    let mut buf = [0u8; 4096];
    let mut done = false;
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, _)) => {
                let text = String::from_utf8_lossy(&buf[..size]);
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let value: JsonValue = serde_json::from_str(trimmed)
                        .map_err(|e| format!("E_GATEWAY_INPUT_PARSE {e}"))?;
                    events.push(parse_event_from_value(&value)?);
                    if let Some(limit) = max_events {
                        if events.len() as u64 >= limit {
                            done = true;
                            break;
                        }
                    }
                }
                if done {
                    break;
                }
            }
            Err(err) => {
                if err.kind() == std::io::ErrorKind::WouldBlock
                    || err.kind() == std::io::ErrorKind::TimedOut
                {
                    break;
                }
                return Err(format!("E_GATEWAY_INPUT_READ {}", err));
            }
        }
    }
    if let Some(handle) = sender_handle {
        let _ = handle.join();
    }
    Ok(events)
}

fn send_events_over_tcp(stream: &mut TcpStream, events: &[GatewayNetEvent]) -> Result<(), String> {
    for event in events {
        let line = serialize_event_line(event)?;
        stream
            .write_all(line.as_bytes())
            .map_err(|e| format!("E_GATEWAY_SEND {}", e))?;
    }
    Ok(())
}

fn send_events_over_udp(
    socket: &UdpSocket,
    target: std::net::SocketAddr,
    events: &[GatewayNetEvent],
) -> Result<(), String> {
    for event in events {
        let line = serialize_event_line(event)?;
        socket
            .send_to(line.as_bytes(), target)
            .map_err(|e| format!("E_GATEWAY_SEND {}", e))?;
    }
    Ok(())
}

fn serialize_event_line(event: &GatewayNetEvent) -> Result<String, String> {
    let payload: JsonValue = serde_json::from_str(&event.payload)
        .map_err(|e| format!("E_GATEWAY_INPUT_PAYLOAD {e}"))?;
    let value = json!({
        "sender": event.sender,
        "seq": event.seq,
        "order_key": event.order_key,
        "realm_id": event.realm_id,
        "payload": payload,
    });
    let mut line = serde_json::to_string(&value)
        .map_err(|e| format!("E_GATEWAY_INPUT_PAYLOAD {e}"))?;
    line.push('\n');
    Ok(line)
}

fn order_and_dedupe_events(mut events: Vec<GatewayNetEvent>) -> (Vec<GatewayNetEvent>, u64) {
    events.sort_by(|a, b| {
        (
            a.sender.as_str(),
            a.seq,
            a.order_key.as_str(),
            a.payload.as_str(),
        )
            .cmp(&(
                b.sender.as_str(),
                b.seq,
                b.order_key.as_str(),
                b.payload.as_str(),
            ))
    });
    let mut ordered = Vec::with_capacity(events.len());
    let mut dropped = 0u64;
    let mut last_sender: Option<String> = None;
    let mut last_seq: Option<u64> = None;
    for event in events {
        let dup = if let (Some(prev_sender), Some(prev_seq)) = (&last_sender, last_seq) {
            prev_sender == &event.sender && prev_seq == event.seq
        } else {
            false
        };
        if dup {
            dropped += 1;
            continue;
        }
        last_sender = Some(event.sender.clone());
        last_seq = Some(event.seq);
        ordered.push(event);
    }
    (ordered, dropped)
}

fn resolve_realm_count(events: &[GatewayNetEvent], realms: Option<u64>) -> Result<u64, String> {
    let inferred = events.iter().map(|e| e.realm_id).max().unwrap_or(0) + 1;
    if let Some(expected) = realms {
        if expected == 0 {
            return Err("E_GATEWAY_REALMS realms는 1 이상이어야 합니다.".to_string());
        }
        if inferred > expected {
            return Err("E_GATEWAY_REALMS realm_id가 realms 범위를 초과했습니다.".to_string());
        }
        return Ok(expected);
    }
    Ok(inferred.max(1))
}

fn compute_realm_hashes(events: &[GatewayNetEvent], realm_count: u64) -> Vec<JsonValue> {
    let mut hashers = vec![Sha256::new(); realm_count as usize];
    for event in events {
        let realm_id = (event.realm_id % realm_count) as usize;
        let line = format!(
            "sender={}|seq={}|order_key={}|payload={}\n",
            event.sender, event.seq, event.order_key, event.payload
        );
        hashers[realm_id].update(line.as_bytes());
    }
    let mut final_state_hashes = Vec::new();
    for (realm_id, hasher) in hashers.into_iter().enumerate() {
        let digest = hasher.finalize();
        final_state_hashes.push(json!({
            "realm_id": realm_id,
            "state_hash": format!("sha256:{}", hex::encode(digest)),
        }));
    }
    final_state_hashes
}
