use crate::core::state::State;
use crate::core::trace::{Trace, TraceEvent};

pub struct TraceMeta {
    pub ssot_version: &'static str,
    pub seed: u64,
    pub madi: u64,
}

pub fn encode_state(state: &State) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"DDN_STATE_V1\n");

    for (key, value) in &state.resources {
        out.extend_from_slice(key.as_str().as_bytes());
        out.push(b'\t');
        out.extend_from_slice(value.canon().as_bytes());
        out.push(b'\n');
    }

    out
}

pub fn encode_trace_bundle(
    source: &str,
    trace: &Trace,
    state_hash: &str,
    meta: &TraceMeta,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"DDN_TRACE_V1\n");

    write_kv(&mut out, "ssot", meta.ssot_version);
    write_kv(&mut out, "seed", &format!("0x{:016x}", meta.seed));
    write_kv(&mut out, "madi", &meta.madi.to_string());
    write_kv(&mut out, "state_hash", state_hash);
    write_kv(&mut out, "source", &escape_field(source));

    for event in &trace.events {
        match event {
            TraceEvent::Log(text) => write_kv(&mut out, "out", &escape_field(text)),
        }
    }

    out
}

fn write_kv(out: &mut Vec<u8>, key: &str, value: &str) {
    out.extend_from_slice(key.as_bytes());
    out.push(b'\t');
    out.extend_from_slice(value.as_bytes());
    out.push(b'\n');
}

fn escape_field(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}
