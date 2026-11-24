//! Private verified-dependency input adapter. No DDN source generation, parser,
//! public builtin, saved authority, or host-side list formatting.
use std::collections::BTreeMap;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use ddonirang_lang::runtime::Value;
use crate::local_project::{VirtualProjectSource, LocalGajiLock, build_project_gaji_lock};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    project: VirtualProjectSource,
    tool_source: String,
    tool_path: String,
    expected_project_graph_sha256: String,
    expected_lock_sha256: String,
}

/// No Deserialize or public constructor: only a fresh frozen validation makes it.
struct VerifiedDependencyInput { lock: LocalGajiLock }
impl VerifiedDependencyInput {
    fn verify(request: &Request) -> Result<Self, String> {
        let lock = build_project_gaji_lock(request.project.clone(), true)
            .map_err(|e| format!("{}: {}", e.code, e.message))?;
        if lock.project_graph_sha256 != request.expected_project_graph_sha256
            || lock.lock_sha256 != request.expected_lock_sha256 {
            return Err("E_DEPENDENCY_TOOL_STALE: 요청한 작품·의존 판과 현재 검증한 판이 다릅니다".into());
        }
        if lock.packages.len() > 1024 { return Err("E_DEPENDENCY_TOOL_BUDGET: 의존 입력 한도를 넘었습니다".into()); }
        Ok(Self { lock })
    }
    fn inputs(&self) -> BTreeMap<String, Value> {
        // Copy verified facts only. DDN owns iteration, formatting and output.
        let rows = self.lock.packages.iter().map(|p| Value::Pack(BTreeMap::from([
            ("정체".into(), Value::String(p.id.clone())),
            ("판".into(), Value::String(p.version.clone())),
            ("경로".into(), Value::String(p.path.clone())),
            ("내용해시".into(), Value::String(p.package_sha256.clone())),
            ("명세해시".into(), Value::String(p.manifest_sha256.clone())),
        ]))).collect();
        BTreeMap::from([("의존목록".into(), Value::List(rows))])
    }
}

pub(crate) fn operate(text: &str) -> Result<String, String> {
    if text.len() > 4 * 1024 * 1024 { return Err("E_DEPENDENCY_TOOL_BUDGET: 요청 크기 한도를 넘었습니다".into()); }
    let request: Request = serde_json::from_str(text)
        .map_err(|e| format!("E_DEPENDENCY_TOOL_REQUEST: {e}"))?;
    if request.tool_source.len() > 65536 || request.tool_path.is_empty() {
        return Err("E_DEPENDENCY_TOOL_REQUEST: 도구 원문 크기와 경로를 확인하세요".into());
    }
    let verified = VerifiedDependencyInput::verify(&request)?;
    let inputs = verified.inputs();
    // One actual host-delivered, deterministic idle tick; this is not a
    // replayed success record or a dependency authority token.
    let tick = ddonirang_core::InputSnapshot {
        tick_id: 0, dt: crate::input_receipt::alpha_host_dt(), keys_pressed: 0,
        last_key_name: String::new(), pointer_x_i32: 0, pointer_y_i32: 0,
        ai_injections: vec![], net_events: vec![],
        frame_source: ddonirang_core::InputSource::Person, rng_seed: 0,
    };
    let receipts = json!([crate::input_receipt::receipt(&tick, &Default::default())]);
    let mut summary = crate::runtime_surface::run_summary_from_supported_source_profile_with_receipts(
        &request.tool_source, &request.tool_path, Some(1), Some("v1-core-v25"), &receipts, Some(&inputs))?;
    summary["verified_dependency_input"] = json!({
        "project_graph_sha256": verified.lock.project_graph_sha256,
        "lock_sha256": verified.lock.lock_sha256,
        "tool_source_sha256": format!("{:x}", Sha256::digest(request.tool_source.as_bytes())),
        "tool_path": request.tool_path,
        "proof": false
    });
    serde_json::to_string(&summary).map_err(|e| e.to_string())
}
