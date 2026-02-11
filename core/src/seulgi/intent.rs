use crate::fixed64::Fixed64;
use crate::platform::{SeulgiIntent, SeulgiPacket};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntentRecord {
    pub agent_id: u64,
    pub recv_seq: u64,
    pub accepted_madi: u64,
    pub target_madi: u64,
    pub intent: SeulgiIntent,
}

impl IntentRecord {
    pub fn from_packet(packet: &SeulgiPacket) -> Self {
        Self {
            agent_id: packet.agent_id,
            recv_seq: packet.recv_seq,
            accepted_madi: packet.accepted_madi,
            target_madi: packet.target_madi,
            intent: packet.intent.clone(),
        }
    }
}

pub fn intent_kind(intent: &SeulgiIntent) -> &'static str {
    match intent {
        SeulgiIntent::None => "None",
        SeulgiIntent::MoveTo { .. } => "MoveTo",
        SeulgiIntent::Attack { .. } => "Attack",
        SeulgiIntent::Say { .. } => "Say",
    }
}

pub fn intent_to_detjson(intent: &SeulgiIntent) -> String {
    let mut out = String::new();
    out.push('{');
    push_kv_str(&mut out, "schema", "seulgi.intent.v1", true);
    push_kv_str(&mut out, "kind", intent_kind(intent), false);
    match intent {
        SeulgiIntent::None => {}
        SeulgiIntent::MoveTo { x, y } => {
            push_kv_fixed(&mut out, "x", *x, false);
            push_kv_fixed(&mut out, "y", *y, false);
        }
        SeulgiIntent::Attack { target_id } => {
            push_kv_num(&mut out, "target_id", *target_id as i64, false);
        }
        SeulgiIntent::Say { text } => {
            push_kv_str(&mut out, "text", text, false);
        }
    }
    out.push('}');
    out
}

pub fn intent_bundle_detjson(records: &[IntentRecord]) -> String {
    let mut items = records.to_vec();
    items.sort_by(|a, b| {
        (
            a.accepted_madi,
            a.agent_id,
            a.recv_seq,
            a.target_madi,
        )
            .cmp(&(b.accepted_madi, b.agent_id, b.recv_seq, b.target_madi))
    });

    let mut out = String::new();
    out.push('{');
    push_kv_str(&mut out, "schema", "seulgi.intent_bundle.v1", true);
    out.push_str(",\"items\":[");
    for (idx, item) in items.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push('{');
        push_kv_num(&mut out, "accepted_madi", item.accepted_madi as i64, true);
        push_kv_num(&mut out, "agent_id", item.agent_id as i64, false);
        push_kv_num(&mut out, "recv_seq", item.recv_seq as i64, false);
        push_kv_num(&mut out, "target_madi", item.target_madi as i64, false);
        out.push_str(",\"intent\":");
        out.push_str(&intent_to_detjson(&item.intent));
        out.push('}');
    }
    out.push_str("]}");
    out
}

fn push_kv_str(out: &mut String, key: &str, value: &str, first: bool) {
    if !first {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":\"");
    out.push_str(&escape_json(value));
    out.push('"');
}

fn push_kv_num(out: &mut String, key: &str, value: i64, first: bool) {
    if !first {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":");
    out.push_str(&value.to_string());
}

fn push_kv_fixed(out: &mut String, key: &str, value: Fixed64, first: bool) {
    if !first {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":");
    out.push_str(&value.to_string());
}

fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}
