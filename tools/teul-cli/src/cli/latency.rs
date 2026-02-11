use std::path::Path;

use ddonirang_core::seulgi::latency::{simulate, LatencyEvent, LatencyMode, LatencyPolicy};

use super::detjson::{sha256_hex, write_text};

pub fn run_simulate(
    l_madi: u64,
    mode: LatencyMode,
    count: u64,
    seed: u64,
    out: Option<&Path>,
) -> Result<(), String> {
    let policy = LatencyPolicy { l_madi, mode, seed };
    let events = simulate(policy, count);
    let detjson = build_detjson(policy, &events);
    let hash = sha256_hex(detjson.as_bytes());
    if let Some(path) = out {
        write_text(path, &detjson)?;
    } else {
        println!("{}", detjson);
    }
    println!("latency_hash=sha256:{}", hash);
    Ok(())
}

fn build_detjson(policy: LatencyPolicy, events: &[LatencyEvent]) -> String {
    let mode = match policy.mode {
        LatencyMode::Fixed => "fixed",
        LatencyMode::Jitter => "jitter",
    };
    let mut out = String::new();
    out.push_str("{\"schema\":\"latency.simulation.v1\",\"L\":");
    out.push_str(&policy.l_madi.to_string());
    out.push_str(",\"mode\":\"");
    out.push_str(mode);
    out.push_str("\",\"seed\":");
    out.push_str(&policy.seed.to_string());
    out.push_str(",\"schedule\":[");
    for (idx, ev) in events.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str("{\"madi\":");
        out.push_str(&ev.madi.to_string());
        out.push_str(",\"deliver_madi\":");
        out.push_str(&ev.deliver_madi.to_string());
        out.push('}');
    }
    out.push_str("]}");
    out
}