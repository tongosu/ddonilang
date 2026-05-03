use std::path::Path;

use ddonirang_core::seulgi::latency::{
    build_replay_header, simulate, LatencyEvent, LatencyMode, LatencyPolicy, LatencySchedule,
    ScheduledPacket,
};

use super::detjson::{sha256_hex, write_text};

pub fn run_simulate(
    l_madi: u64,
    mode: LatencyMode,
    count: u64,
    seed: u64,
    current_madi: u64,
    out: Option<&Path>,
) -> Result<(), String> {
    let policy = LatencyPolicy { l_madi, mode, seed };
    let events = simulate(policy, count);
    let detjson = build_detjson(policy, &events, current_madi);
    let hash = sha256_hex(detjson.as_bytes());
    if let Some(path) = out {
        write_text(path, &detjson)?;
    } else {
        println!("{}", detjson);
    }
    println!("latency_hash=sha256:{}", hash);
    Ok(())
}

fn build_detjson(policy: LatencyPolicy, events: &[LatencyEvent], current_madi: u64) -> String {
    let mode = match policy.mode {
        LatencyMode::Fixed => "fixed",
        LatencyMode::Jitter => "jitter",
    };
    let replay_header = build_replay_header(policy.l_madi);
    let mut out = String::new();
    out.push_str("{\"schema\":\"latency.simulation.v1\",\"L\":");
    out.push_str(&policy.l_madi.to_string());
    out.push_str(",\"mode\":\"");
    out.push_str(mode);
    out.push_str("\",\"seed\":");
    out.push_str(&policy.seed.to_string());
    out.push_str(",\"current_madi\":");
    out.push_str(&current_madi.to_string());
    out.push_str(",\"replay_header\":{\"seulgi_latency_madi\":");
    out.push_str(&replay_header.seulgi_latency_madi.to_string());
    out.push_str(",\"seulgi_latency_drop_policy\":\"");
    out.push_str(&replay_header.seulgi_latency_drop_policy);
    out.push_str("\"");
    out.push_str(",\"created_at\":\"");
    out.push_str(&replay_header.created_at);
    out.push_str("\"}");
    out.push_str(",\"schedule\":[");
    for (idx, ev) in events.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        let schedule = LatencySchedule::new(ev.deliver_madi.saturating_sub(ev.madi), ev.madi);
        let packet = ScheduledPacket::from_schedule(schedule, current_madi);
        out.push_str("{\"madi\":");
        out.push_str(&ev.madi.to_string());
        out.push_str(",\"deliver_madi\":");
        out.push_str(&ev.deliver_madi.to_string());
        out.push_str(",\"accepted_madi\":");
        out.push_str(&packet.accept_madi.to_string());
        out.push_str(",\"target_madi\":");
        out.push_str(&packet.target_madi.to_string());
        out.push_str(",\"late\":");
        out.push_str(if packet.late { "true" } else { "false" });
        out.push_str(",\"dropped\":");
        out.push_str(if packet.dropped { "true" } else { "false" });
        out.push('}');
    }
    out.push_str("]}");
    out
}
