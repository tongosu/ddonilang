use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use blake3::hash as blake3_hash;
use ddonirang_core::platform::{
    DetNuri, InputSnapshot, InputSource, Patch, PatchOp, ResourceValue,
};
use ddonirang_core::{
    ExternalBoundaryFailure, ExternalBoundaryRequest, ExternalBoundaryResult, Fixed64, Nuri,
    ProcessLocalCapabilityStore, ResourceHandle, SealedBoundaryProvider, SealedExecutionLedger,
    SeulgiIntent, SeulgiPacket,
};
use ddonirang_lang::runtime::Value;
use ddonirang_lang::{ExecutionModeDecision, ParseMode};
use serde_json::{json, Map, Value as JsonValue};
use wasm_bindgen::prelude::*;

use crate::canon;
use crate::completion;
use crate::corpus_csv;
use crate::ddn_runtime::{
    resolve_supported_source_profile,
    DdnParseWarning, DdnProgram, DdnRunner,
};
use crate::deterministic_debug::{
    append_public_observation_surface, build_public_observation_table_projection,
    build_public_observation_trace, deterministic_debug_operation, public_observation_events_json,
    DebugEvaluator,
};
use crate::host_resource;
use crate::local_project;
use crate::module_directive_analysis;
use crate::project_authoring;
use crate::runtime_surface;
use crate::source_analysis;
pub(crate) use crate::unit_wire::json_to_fixed64;
use crate::unit_wire::{RESOURCE_SNAPSHOT_SCHEMA_V2, UNIT_VALUE_WIRE_SCHEMA_V2};

const DEFAULT_UPDATE_NAME: &str = "매마디";

#[wasm_bindgen]
pub fn authored_project_operation(operation: &str, artifact_json: &str) -> Result<String, JsValue> {
    crate::authored_project::operate(operation, artifact_json).map_err(|error| JsValue::from_str(&error))
}

/// Recorded source readback only. No fresh registration/capture, VM, world,
/// runtime restoration, host first-use authority or publication capability.
#[wasm_bindgen]
pub fn wasm_restore_recorded_source_revision(
    payload_utf8:&str,envelope:&[u8],project:&[u8],parent:&[u8],revision:&[u8],
    integrity:&[u8],profile:&[u8],pesi:&[u8],
)->Result<String,JsValue> {
    let integrity:&[u8;32]=integrity.try_into().map_err(|_|JsValue::from_str("E_SOURCE_REVISION_INTEGRITY_LENGTH"))?;
    let pesi:&[u8;32]=pesi.try_into().map_err(|_|JsValue::from_str("E_SOURCE_REVISION_PESI_LENGTH"))?;
    crate::unit_wire::project_runtime_attachment::source_only::PreparedSourceOnlyRevision::restore(
        project,parent,revision,integrity,profile,pesi,payload_utf8.as_bytes(),envelope,
    ).and_then(|source|source.owned().project().save()).map_err(|e|JsValue::from_str(&e))
}
const ENGINE_RESPONSE_SCHEMA: &str = "seamgrim.engine_response.v0";
const WASM_STATE_SNAPSHOT_SCHEMA_V2: &str = "ddn.wasm_state_snapshot.v2";

/// Live logical input uses the same transactional session as file replay.
/// Device mapping remains outside this VM; no raw-key/default-input fallback.
#[wasm_bindgen]
pub struct DdnLogicalInputVm {
    execution: LogicalVmExecution,
    recording:crate::input_tape_v3::LiveRecording,
}

enum LogicalVmExecution {
    SingleFile(crate::logical_input_tape::LogicalInputSession),
    Authored(crate::unit_wire::resumable_complete_state::CheckpointedLogicalRunV1),
}

impl DdnLogicalInputVm {
    fn session(&self)->&crate::logical_input_tape::LogicalInputSession {
        match &self.execution {
            LogicalVmExecution::SingleFile(session)=>session,
            LogicalVmExecution::Authored(run)=>run.session(),
        }
    }

    fn reject_input(&mut self,reason:String) {
        match &mut self.execution {
            LogicalVmExecution::SingleFile(session)=>session.reject_input(reason),
            LogicalVmExecution::Authored(run)=>run.reject_input(reason),
        }
    }
}

#[wasm_bindgen]
impl DdnLogicalInputVm {
    pub fn bind_source_json(source: &str, binding_json: &str) -> Result<String, JsValue> {
        let binding = serde_json::from_str(binding_json)
            .map_err(|error| JsValue::from_str(&format!("E_LOGICAL_INPUT_BINDING_JSON: {error}")))?;
        let bound = crate::logical_input_tape::LogicalInputSession::bind_source_for_fresh_run(source, binding)
            .map_err(|error| JsValue::from_str(&error))?;
        serde_json::to_string(&bound).map_err(|error| JsValue::from_str(&error.to_string()))
    }

    #[wasm_bindgen(constructor)]
    pub fn new(source: &str, binding_json: &str) -> Result<DdnLogicalInputVm, JsValue> {
        let binding = serde_json::from_str(binding_json)
            .map_err(|_| JsValue::from_str("E_LOGICAL_INPUT_BINDING: 논리 입력 결속 필드가 올바르지 않습니다"))?;
        let session = crate::logical_input_tape::LogicalInputSession::new(source, binding)
            .map_err(|reason| JsValue::from_str(&reason))?;
        Ok(Self { execution:LogicalVmExecution::SingleFile(session),recording:Default::default() })
    }

    /// Explicit project admission preserves modules and the named Start.
    /// No linked-text fallback or durable browser receipt is manufactured.
    pub fn from_authored_project(authored: &str,binding_json: &str)->Result<DdnLogicalInputVm,JsValue> {
        let binding=serde_json::from_str(binding_json)
            .map_err(|_|JsValue::from_str("E_LOGICAL_INPUT_BINDING: 논리 입력 결속 필드가 올바르지 않습니다"))?;
        let run=runtime_surface::fresh_authored_logical_run(authored,binding)
            .map_err(|reason|JsValue::from_str(&reason))?;
        Ok(Self {execution:LogicalVmExecution::Authored(run),recording:Default::default()})
    }

    pub fn bind_authored_project_json(authored: &str,binding_json: &str)->Result<String,JsValue> {
        let mut binding:crate::logical_input_tape::LogicalInputBinding=serde_json::from_str(binding_json)
            .map_err(|_|JsValue::from_str("E_LOGICAL_INPUT_BINDING: 논리 입력 결속 필드가 올바르지 않습니다"))?;
        let owned=crate::project_revision_envelope_supply::admit_fresh_authored_project(authored.as_bytes())
            .map_err(|reason|JsValue::from_str(&reason))?;
        binding.source_revision=runtime_surface::sha256_identity(owned.sources().preparation().source.as_bytes());
        runtime_surface::fresh_authored_logical_run(authored,binding.clone())
            .map_err(|reason|JsValue::from_str(&reason))?;
        serde_json::to_string(&binding).map_err(|reason|JsValue::from_str(&reason.to_string()))
    }

    /// Existing source-bound carrier only; not a committed durable receipt.
    pub fn from_saved_project_checkpoint(
        artifact:&str,checkpoint:&[u8],parent:&[u8],revision:&[u8],binding_json:&str,
    )->Result<DdnLogicalInputVm,JsValue> {
        use crate::unit_wire::project_runtime_attachment as attachment;
        let binding=serde_json::from_str(binding_json)
            .map_err(|_|JsValue::from_str("E_LOGICAL_INPUT_BINDING: 논리 입력 결속 필드가 올바르지 않습니다"))?;
        let checked=attachment::check_bytes(artifact.as_bytes()).map_err(|reason|JsValue::from_str(&reason))?;
        let owned=attachment::restore_saved_runtime_payload(&checked).map_err(|reason|JsValue::from_str(&reason))?;
        let context=attachment::check_saved_revision_context(&checked,&owned,parent,revision)
            .map_err(|reason|JsValue::from_str(&reason))?;
        let run=crate::unit_wire::resumable_complete_state::CheckpointedLogicalRunV1::restore_local_checkpoint(
            checkpoint,&context,&binding).map_err(|reason|JsValue::from_str(
                &format!("E_PROJECT_RUNTIME_LOGICAL: 저장된 입력·원문·실행 상태가 일치하지 않습니다 ({reason:?})")))?;
        Ok(Self {execution:LogicalVmExecution::Authored(run),recording:Default::default()})
    }

    /// Existing complete-state request bytes, paired with the RD-686 carrier.
    /// IndexedDB confirmation and Native durable receipts remain separate.
    pub fn export_project_checkpoint(&self,parent:&[u8],revision:&[u8])->Result<Vec<u8>,JsValue> {
        use crate::unit_wire::project_runtime_attachment as attachment;
        let LogicalVmExecution::Authored(run)=&self.execution else {
            return Err(JsValue::from_str("E_PROJECT_RUNTIME_LOGICAL: 명시적 프로젝트와 시작 정의가 필요합니다"));
        };
        run.check_checkpoint_channels().map_err(|reason|JsValue::from_str(&reason))?;
        let artifact=run.export_saved_artifact(parent,revision).map_err(|reason|JsValue::from_str(&format!("E_PROJECT_RUNTIME_LOGICAL: {reason:?}")))?;
        let checked=attachment::check_bytes(artifact.as_bytes()).map_err(|reason|JsValue::from_str(&reason))?;
        let owned=attachment::restore_saved_runtime_payload(&checked).map_err(|reason|JsValue::from_str(&reason))?;
        let context=attachment::check_saved_revision_context(&checked,&owned,parent,revision).map_err(|reason|JsValue::from_str(&reason))?;
        let world=run.encode_world_frame().map_err(|reason|JsValue::from_str(&format!("E_PROJECT_RUNTIME_LOGICAL: {reason:?}")))?;
        run.encode_checkpoint_request(&world,&context,parent,revision)
            .map_err(|reason|JsValue::from_str(&format!("E_PROJECT_RUNTIME_LOGICAL: {reason:?}")))
    }

    /// Existing source-bound carrier only; not a committed durable receipt.
    pub fn export_saved_project(&self,parent: &[u8],revision: &[u8])->Result<String,JsValue> {
        match &self.execution {
            LogicalVmExecution::Authored(run)=>{
                run.check_checkpoint_channels().map_err(|reason|JsValue::from_str(&reason))?;
                run.export_saved_artifact(parent,revision)
                    .map_err(|reason|JsValue::from_str(&format!("E_PROJECT_RUNTIME_LOGICAL: 저장 결속 실패 ({reason:?})")))
            },
            LogicalVmExecution::SingleFile(_)=>Err(JsValue::from_str(
                "E_PROJECT_RUNTIME_LOGICAL: 명시적 프로젝트와 시작 정의가 필요합니다")),
        }
    }

    pub fn step_json(&mut self, snapshot_json: &str) -> Result<String, JsValue> {
        self.recording.ensure_running().map_err(|e|JsValue::from_str(&e))?;
        let snapshot = serde_json::from_str(snapshot_json).map_err(|_| {
            let reason = "E_LOGICAL_INPUT_SNAPSHOT: 논리 입력 필드가 올바르지 않습니다";
            self.reject_input(reason.to_string());
            JsValue::from_str(reason)
        })?;
        let result = match &mut self.execution {
            LogicalVmExecution::SingleFile(session)=>session.step(&snapshot),
            LogicalVmExecution::Authored(run)=>run.step(&snapshot),
        }
            .map_err(|reason| JsValue::from_str(&reason))?;
        serde_json::to_string(&result).map_err(|reason| JsValue::from_str(&reason.to_string()))
    }

    pub fn summary_json(&self) -> String {
        self.session().summary().to_string()
    }

    /// Display projection only. The shared accepted world is never mutated.
    pub fn view_json(&self) -> String {
        build_view_meta(self.session().committed_world()).to_string()
    }

    /// Host input loss is explicit failure, not a manufactured Release event.
    pub fn abort_input(&mut self, message: &str) {
        if self.recording.ensure_running().is_err(){return;}
        self.reject_input(format!("E_LOGICAL_INPUT_LOST: {message}"));
    }
    pub fn stop_recording_v3_json(&mut self)->Result<String,JsValue> {
        let session=match &self.execution {
            LogicalVmExecution::SingleFile(session)=>session,
            LogicalVmExecution::Authored(run)=>run.session(),
        };
        let (bytes,summary)=self.recording.stop_logical(session).map_err(|e|JsValue::from_str(&e))?;
        let outcome=if summary.get("outcome").and_then(serde_json::Value::as_str)==Some("ABORTED"){"ABORTED"}else{"INCOMPLETE"};
        Ok(json!({"outcome":outcome,"tape_bytes":bytes,"summary":summary}).to_string())
    }

    pub fn failed_prefix_v3_json(&self) -> Result<String,JsValue> {
        let (bytes,summary)=crate::input_tape_v3::failed_prefix_logical(self.session()).map_err(|e|JsValue::from_str(&e))?;
        Ok(json!({"outcome":"ABORTED","tape_bytes":bytes,"summary":summary}).to_string())
    }

    pub fn seal_json(&self) -> Result<String, JsValue> {
        self.recording.ensure_running().map_err(|e|JsValue::from_str(&e))?;
        Err(JsValue::from_str("E_INPUT_V3_FINITE_PLAN_REQUIRED: live 실행에는 완료 계획이 없습니다. 명시 중단으로 미완 기록을 저장하세요."))
    }
}

/// Host snapshot recording and replay use the same Rust executor as Native.
/// These entrypoints do not accept old tape versions or raw device inputs.
#[wasm_bindgen]
pub fn logical_input_record_v3(source:&str,request_json:&str)->Result<String,JsValue> {
    let request=serde_json::from_str(request_json).map_err(|_|JsValue::from_str("E_INPUT_TAPE_V3: 입력 요청 필드 오류"))?;
    let (bytes,summary)=crate::input_tape_v3::record_logical(source,&request).map_err(|e|JsValue::from_str(&e))?;
    Ok(json!({"tape_bytes":bytes,"summary":summary}).to_string())
}
#[wasm_bindgen]
pub fn authored_logical_input_record_v3(authored:&str,request_json:&str)->Result<String,JsValue> {
    let request=serde_json::from_str(request_json).map_err(|_|JsValue::from_str("E_INPUT_TAPE_V3: 작품 입력 요청 필드 오류"))?;
    let (bytes,summary)=crate::input_tape_v3::record_authored_logical(authored,&request).map_err(|e|JsValue::from_str(&e))?;
    Ok(json!({"tape_bytes":bytes,"summary":summary}).to_string())
}
#[wasm_bindgen]
pub fn authored_logical_input_replay_v3(authored:&str,tape:&[u8],binding_json:&str)->Result<String,JsValue> {
    let binding=serde_json::from_str(binding_json).map_err(|_|JsValue::from_str("E_INPUT_TAPE_V3: 작품 기대 입력 계약 오류"))?;
    crate::input_tape_v3::replay_authored_logical(authored,tape,&binding)
        .map(|summary|crate::unit_wire::SnapshotJsonOwner::from_value(summary).compact_text()).map_err(|e|JsValue::from_str(&e))
}
#[wasm_bindgen]
pub fn logical_input_replay_v3(source:&str,tape:&[u8],binding_json:&str)->Result<String,JsValue> {
    let binding=serde_json::from_str(binding_json).map_err(|_|JsValue::from_str("E_INPUT_TAPE_V3: 기대 입력 계약 오류"))?;
    crate::input_tape_v3::replay(source,tape,crate::input_tape_v3::ExpectedBinding::Logical(&binding))
        .map(|summary|crate::unit_wire::SnapshotJsonOwner::from_value(summary).compact_text()).map_err(|e|JsValue::from_str(&e))
}
#[wasm_bindgen]
pub fn manipulation_input_record_v3(source:&str,request_json:&str)->Result<String,JsValue> {
    let request=serde_json::from_str(request_json).map_err(|_|JsValue::from_str("E_INPUT_TAPE_V3: 조작 입력 요청 필드 오류"))?;
    let (bytes,summary)=crate::input_tape_v3::record_manipulation(source,&request).map_err(|e|JsValue::from_str(&e))?;
    Ok(json!({"tape_bytes":bytes,"summary":summary}).to_string())
}
#[wasm_bindgen]
pub fn manipulation_input_replay_v3(source:&str,tape:&[u8],binding_json:&str)->Result<String,JsValue> {
    let binding=serde_json::from_str(binding_json).map_err(|_|JsValue::from_str("E_INPUT_TAPE_V3: 기대 조작 입력 계약 오류"))?;
    crate::input_tape_v3::replay(source,tape,crate::input_tape_v3::ExpectedBinding::Manipulation(&binding))
        .map(|summary|crate::unit_wire::SnapshotJsonOwner::from_value(summary).compact_text()).map_err(|e|JsValue::from_str(&e))
}
#[wasm_bindgen]
pub fn logical_input_record(source: &str, request_json: &str) -> Result<String, JsValue> {
    let request = serde_json::from_str(request_json)
        .map_err(|_| JsValue::from_str("E_LOGICAL_INPUT_REQUEST: 입력 요청 필드가 올바르지 않습니다"))?;
    let (bytes, summary) = crate::input_tape_v3::record_logical(source, &request)
        .map_err(|error| JsValue::from_str(&error))?;
    Ok(json!({"tape_bytes": bytes, "summary": summary}).to_string())
}

#[wasm_bindgen]
pub fn logical_input_replay(source: &str, tape: &[u8], binding_json: &str) -> Result<String, JsValue> {
    let binding = serde_json::from_str(binding_json)
        .map_err(|_| JsValue::from_str("E_LOGICAL_INPUT_BINDING: 입력 계약 필드가 올바르지 않습니다"))?;
    crate::input_tape_v3::replay(source, tape, crate::input_tape_v3::ExpectedBinding::Logical(&binding))
        .map(|summary| crate::unit_wire::SnapshotJsonOwner::from_value(summary).compact_text()).map_err(|error| JsValue::from_str(&error))
}
const SPACE2D_SCHEMA: &str = "seamgrim.space2d.v0";
/// WO15 provisional typed value intake; the V1 logical VM is unchanged.
#[wasm_bindgen]
pub struct DdnManipulationVm { session: crate::manipulation_input::ManipulationSession,recording:crate::input_tape_v3::LiveRecording }
#[wasm_bindgen]
impl DdnManipulationVm {
    pub fn bind_source_json(source: &str, binding_json: &str) -> Result<String, JsValue> {
        let binding = serde_json::from_str(binding_json).map_err(|error| JsValue::from_str(&format!("E_MANIPULATION_BINDING_JSON: {error}")))?;
        let bound = crate::manipulation_input::ManipulationSession::bind_source_for_fresh_run(source,binding).map_err(|error| JsValue::from_str(&error))?;
        serde_json::to_string(&bound).map_err(|error| JsValue::from_str(&error.to_string()))
    }
    #[wasm_bindgen(constructor)]
    pub fn new(source: &str, binding_json: &str) -> Result<DdnManipulationVm, JsValue> {
        let binding = serde_json::from_str(binding_json).map_err(|error| JsValue::from_str(&format!("E_MANIPULATION_BINDING_JSON: {error}")))?;
        let session = crate::manipulation_input::ManipulationSession::new(source,binding).map_err(|error| JsValue::from_str(&error))?;
        Ok(Self {session,recording:Default::default()})
    }
    pub fn step_json(&mut self, snapshot_json: &str) -> Result<String, JsValue> {
        self.recording.ensure_running().map_err(|e|JsValue::from_str(&e))?;
        let snapshot = serde_json::from_str(snapshot_json).map_err(|error| {
            let reason=format!("E_MANIPULATION_SNAPSHOT_JSON: {error}");self.session.reject_input(reason.clone());JsValue::from_str(&reason)
        })?;
        let result=self.session.step(&snapshot).map_err(|error| JsValue::from_str(&error))?;
        serde_json::to_string(&result).map_err(|error| JsValue::from_str(&error.to_string()))
    }
    pub fn summary_json(&self) -> String {self.session.summary().to_string()}
    pub fn view_json(&self) -> String {build_view_meta(self.session.committed_world()).to_string()}
    pub fn abort_input(&mut self,message: &str) {if self.recording.ensure_running().is_ok(){self.session.reject_input(format!("E_MANIPULATION_INPUT_LOST: {message}"));}}
    pub fn stop_recording_v3_json(&mut self)->Result<String,JsValue> {
        let (bytes,summary)=self.recording.stop_manipulation(&self.session).map_err(|e|JsValue::from_str(&e))?;
        let outcome=if summary.get("outcome").and_then(serde_json::Value::as_str)==Some("ABORTED"){"ABORTED"}else{"INCOMPLETE"};
        Ok(json!({"outcome":outcome,"tape_bytes":bytes,"summary":summary}).to_string())
    }
    pub fn failed_prefix_v3_json(&self)->Result<String,JsValue> {
        let (bytes,summary)=crate::input_tape_v3::failed_prefix_manipulation(&self.session).map_err(|e|JsValue::from_str(&e))?;
        Ok(json!({"outcome":"ABORTED","tape_bytes":bytes,"summary":summary}).to_string())
    }
    pub fn seal_json(&self) -> Result<String, JsValue> {
        self.recording.ensure_running().map_err(|e|JsValue::from_str(&e))?;
        Err(JsValue::from_str("E_INPUT_V3_FINITE_PLAN_REQUIRED: live 실행에는 완료 계획이 없습니다. 명시 중단으로 미완 기록을 저장하세요."))
    }
}
#[wasm_bindgen]
pub fn manipulation_input_record(source: &str, request_json: &str) -> Result<String, JsValue> {
    let request=serde_json::from_str(request_json).map_err(|error| JsValue::from_str(&format!("E_MANIPULATION_REQUEST_JSON: {error}")))?;
    let (bytes,summary)=crate::input_tape_v3::record_manipulation(source,&request).map_err(|error| JsValue::from_str(&error))?;
    Ok(json!({"tape_bytes":bytes,"summary":summary}).to_string())
}
#[wasm_bindgen]
pub fn manipulation_input_replay(source: &str,tape: &[u8],binding_json: &str) -> Result<String, JsValue> {
    let binding=serde_json::from_str(binding_json).map_err(|error| JsValue::from_str(&format!("E_MANIPULATION_BINDING_JSON: {error}")))?;
    crate::input_tape_v3::replay(source,tape,crate::input_tape_v3::ExpectedBinding::Manipulation(&binding)).map(|summary| crate::unit_wire::SnapshotJsonOwner::from_value(summary).compact_text()).map_err(|error| JsValue::from_str(&error))
}
#[wasm_bindgen]
pub fn manipulation_value_raw(text: &str) -> Result<String, JsValue> {
    Fixed64::parse_decimal(text).map(|value|value.raw_i64().to_string())
        .ok_or_else(||JsValue::from_str("E_MANIPULATION_VALUE: 정확한 소수 표기가 필요합니다"))
}
const BOGAE_DRAWLIST_TAG: &str = "보개_그림판_목록";
const BOGAE_WIDTH_TAG: &str = "보개_그림판_가로";
const BOGAE_HEIGHT_TAG: &str = "보개_그림판_세로";

/// WASM adapter only: no bindgen export and no separate external-boundary
/// semantics. The public product surface remains unchanged.
pub(crate) fn wasm_execute_sealed_external_boundary(
    ledger: &mut SealedExecutionLedger,
    capability_store: &ProcessLocalCapabilityStore,
    provider: &dyn SealedBoundaryProvider,
    request: &ExternalBoundaryRequest,
) -> Result<ExternalBoundaryResult, ExternalBoundaryFailure> {
    runtime_surface::execute_sealed_external_boundary(ledger, capability_store, provider, request)
}
const BOGAE_SHOW_LINES_TAG: &str = "보개_출력_줄들";
const GRAPH_SCHEMA: &str = "seamgrim.graph.v0";
const WORKBENCH_RESULT_SET_SCHEMA: &str =
    "seamgrim.workbench.authoritative_result_set.v1_candidate";
const WORKBENCH_LINE_RESULT_SCHEMA: &str = "seamgrim.workbench.line_result.v1_candidate";
const WORKBENCH_PENDULUM_RESULT_SCHEMA: &str = "seamgrim.workbench.pendulum_result.v1_candidate";
const WORKBENCH_PROJECTILE_RESULT_SCHEMA: &str =
    "seamgrim.workbench.projectile_result.v1_candidate";
const WORKBENCH_GRID_RESULT_SCHEMA: &str = "seamgrim.workbench.grid_result.v1_candidate";
const DEFAULT_VIEW_PREFIXES: &[&str] = &["보개_", "__view_"];
const GRAPH_POINTS_TAGS: &[(&str, &str, &str)] = &[
    ("보개_그래프_점목록_f", "f", "f(x)"),
    ("그래프_점목록_f", "f", "f(x)"),
    ("그래프_점목록_df", "df", "f'(x)"),
    ("그래프_점목록_fi", "fi", "int(f)"),
    ("graph_points_f", "f", "f(x)"),
    ("graph_points_df", "df", "f'(x)"),
    ("graph_points_fi", "fi", "int(f)"),
];
fn parse_mode_from_str(mode: &str) -> Result<ParseMode, JsValue> {
    let normalized = mode.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "" => Ok(ParseMode::SuccessorDefault),
        "strict" | "successor-default" => Ok(ParseMode::SuccessorDefault),
        _ => Err(JsValue::from_str(&format!(
            "지원하지 않는 lang-mode: {mode} (strict|successor-default)"
        ))),
    }
}

fn require_wasm_source_path(source_path: &str) -> Result<&str, JsValue> {
    let trimmed = source_path.trim();
    if trimmed.is_empty() {
        return Err(JsValue::from_str(
            "E_WASM_SOURCE_PATH_EMPTY: 공개 관찰 source_ref에 사용할 DDN 경로가 필요합니다",
        ));
    }
    Ok(trimmed)
}

fn canonicalize_for_wasm(source: &str) -> Result<canon::CanonOutput, JsValue> {
    runtime_surface::canonicalize_shared_source(source)
        .map_err(|err| JsValue::from_str(&format!("WASM canon 실패: {err}")))
}

fn canonicalize_for_wasm_with_supported_profile(
    source: &str,
    source_profile_identity: &str,
) -> Result<canon::CanonOutput, JsValue> {
    runtime_surface::canonicalize_shared_source_with_supported_profile(
        source,
        Some(source_profile_identity),
    )
    .map_err(|err| JsValue::from_str(&format!("WASM canon 실패: {err}")))
}

fn canonicalize_flat_for_wasm(source: &str) -> Result<canon::CanonOutput, JsValue> {
    canon::canonicalize(source, false)
        .map_err(|err| JsValue::from_str(&format!("WASM canon 실패: {err}")))
}

fn preprocess_program_source_for_wasm(source: &str) -> Result<String, JsValue> {
    runtime_surface::prepare_source_for_execution(source)
        .map(|prepared| prepared.execution_source)
        .map_err(|err| JsValue::from_str(&format!("WASM preprocess 실패: {err}")))
}

fn parse_program_for_wasm(
    source: &str,
    source_path: &str,
    mode: ParseMode,
) -> Result<(DdnProgram, ExecutionModeDecision), JsValue> {
    let prepared = runtime_surface::prepare_source_for_execution(source)
        .map_err(|err| JsValue::from_str(&format!("WASM preprocess 실패: {err}")))?;
    let mut program = DdnProgram::from_source_with_mode(source, source_path, mode)
        .map_err(|err| JsValue::from_str(&err))?;
    source_analysis::validate_program_for_execution(program.canonical_program())
        .map_err(|err| JsValue::from_str(&err))?;
    program.prepare_temporary_source_execution().map_err(|error|JsValue::from_str(&error))?;
    Ok((program, prepared.execution_mode_decision))
}

#[wasm_bindgen]
pub fn wasm_build_info() -> String {
    format!(
        "pkg={} version={} state_schema={} state_snapshot={} resource_schema={} unit_wire={} update={}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        ENGINE_RESPONSE_SCHEMA,
        WASM_STATE_SNAPSHOT_SCHEMA_V2,
        RESOURCE_SNAPSHOT_SCHEMA_V2,
        UNIT_VALUE_WIRE_SCHEMA_V2,
        DEFAULT_UPDATE_NAME
    )
}

#[wasm_bindgen]
pub fn wasm_deterministic_debug_json(operation: &str, input_json: &str) -> Result<String, JsValue> {
    let input: JsonValue = serde_json::from_str(input_json)
        .map_err(|err| JsValue::from_str(&format!("E_DEBUG_JSON_INVALID: {err}")))?;
    deterministic_debug_operation(operation, &input)
        .map(|value| value.to_string())
        .map_err(|err| JsValue::from_str(&err))
}

#[wasm_bindgen]
pub fn wasm_preprocess_source(source: &str) -> Result<String, JsValue> {
    preprocess_program_source_for_wasm(source)
}

#[wasm_bindgen]
pub fn wasm_source_analysis_snapshot_json(request_json: &str) -> Result<String, JsValue> {
    source_analysis::source_analysis_snapshot_json(request_json)
        .map_err(|err| JsValue::from_str(&err))
}

/// Additive internal candidate carrier for lexer-backed RD-457 metadata.
/// It deliberately does not alter the established source-analysis v1 export.
#[wasm_bindgen]
pub fn wasm_module_directive_analysis_snapshot_json(request_json: &str) -> Result<String, JsValue> {
    module_directive_analysis::module_directive_analysis_snapshot_json(request_json)
        .map_err(|err| JsValue::from_str(&err))
}

#[wasm_bindgen]
pub fn wasm_historical_async_migration_evidence_json(
    source: &str,
    source_path: &str,
) -> Result<String, JsValue> {
    let source_path = require_wasm_source_path(source_path)?;
    runtime_surface::historical_async_migration_evidence_json(source, source_path)
        .map_err(|err| JsValue::from_str(&err))
}

#[wasm_bindgen]
pub fn wasm_execution_envelope_json(source: &str, requested_ticks: i32) -> Result<String, JsValue> {
    let requested = if requested_ticks < 0 {
        None
    } else {
        Some(requested_ticks as u32)
    };
    runtime_surface::execution_envelope_from_source(source, requested)
        .map(|envelope| envelope.to_string())
        .map_err(|err| JsValue::from_str(&format!("WASM execution envelope 실패: {err}")))
}

#[wasm_bindgen]
pub fn wasm_run_summary_json(
    source: &str,
    source_path: &str,
    requested_ticks: i32,
) -> Result<String, JsValue> {
    let requested = if requested_ticks < 0 {
        None
    } else {
        Some(requested_ticks as u32)
    };
    let source_path = require_wasm_source_path(source_path)?;
    runtime_surface::run_summary_from_source(source, source_path, requested)
        .map(|summary| crate::unit_wire::SnapshotJsonOwner::from_value(summary).compact_text())
        .map_err(|err| JsValue::from_str(&format!("WASM run summary 실패: {err}")))
}

#[wasm_bindgen]
pub fn wasm_run_summary_with_source_profile_json(
    source: &str,
    source_path: &str,
    requested_ticks: i32,
    source_profile_identity: &str,
) -> Result<String, JsValue> {
    let requested = if requested_ticks < 0 {
        None
    } else {
        Some(requested_ticks as u32)
    };
    let source_path = require_wasm_source_path(source_path)?;
    runtime_surface::run_summary_from_supported_source_profile(
        source,
        source_path,
        requested,
        Some(source_profile_identity),
    )
    .map(|summary| crate::unit_wire::SnapshotJsonOwner::from_value(summary).compact_text())
    .map_err(|err| JsValue::from_str(&err))
}

/// Typed host-resource request entrypoint for RuntimeBridge/Workbench.
#[wasm_bindgen]
pub fn wasm_run_summary_with_bogae_entry_json(
    source:&str, source_path:&str, requested_ticks:i32, source_profile:&str, entry:&str,
) -> Result<String,JsValue> {
    let requested=if requested_ticks<0 {None} else {Some(requested_ticks as u32)};
    runtime_surface::run_summary_with_bogae_entry(source,require_wasm_source_path(source_path)?,requested,source_profile,entry)
        .map(|summary| crate::unit_wire::SnapshotJsonOwner::from_value(summary).compact_text()).map_err(|err|JsValue::from_str(&err))
}

#[wasm_bindgen]
pub fn wasm_run_summary_with_authored_bogae_json(
    source: &str, source_path: &str, requested_ticks: i32, source_profile: &str,
    artifact_json: &str, receipts_json: Option<String>, param_overrides_json: Option<String>,
) -> Result<String, JsValue> {
    let receipts = receipts_json.as_deref().map(serde_json::from_str::<JsonValue>)
        .transpose().map_err(|error| JsValue::from_str(&format!("E_INPUT_RECEIPT_INVALID: {error}")))?;
    let params = param_overrides_json.as_deref().map(|text| {
        let json: JsonValue = serde_json::from_str(text)
            .map_err(|error| format!("E_WASM_STATE_PARAMS_INVALID: {error}"))?;
        parse_param_overrides(&json)
    }).transpose().map_err(|error| JsValue::from_str(&error))?;
    runtime_surface::run_summary_with_authored_bogae(
        source, require_wasm_source_path(source_path)?,
        if requested_ticks < 0 { None } else { Some(requested_ticks as u32) },
        source_profile, artifact_json, receipts.as_ref(), params.as_ref(),
    ).map(|summary| crate::unit_wire::SnapshotJsonOwner::from_value(summary).compact_text()).map_err(|error| JsValue::from_str(&error))
}

/// Typed host-resource request entrypoint for RuntimeBridge/Workbench.
#[wasm_bindgen]
pub fn wasm_run_summary_with_source_profile_input_receipts_json(
    source: &str, source_path: &str, requested_ticks: i32,
    source_profile_identity: &str, receipts_json: &str,
    param_overrides_json: Option<String>,
    bogae_entry: Option<String>,
) -> Result<String, JsValue> {
    let receipts: JsonValue = serde_json::from_str(receipts_json)
        .map_err(|error| JsValue::from_str(&format!("E_INPUT_RECEIPT_INVALID: {error}")))?;
    let rows = receipts.as_array()
        .ok_or_else(|| JsValue::from_str("E_INPUT_RECEIPT_INVALID: 입력 영수증 목록이 필요합니다"))?;
    for row in rows {
        let input = crate::input_receipt::parse_receipt_snapshot(row)
            .map_err(|error| JsValue::from_str(&error))?;
        crate::input_receipt::validate_alpha_host_dt(input.dt)
            .map_err(|error| JsValue::from_str(&error))?;
    }
    let path = require_wasm_source_path(source_path)?;
    let ticks = if requested_ticks < 0 { None } else { Some(requested_ticks as u32) };
    let params = param_overrides_json.as_deref().map(|text| {
        let json: JsonValue = serde_json::from_str(text)
            .map_err(|error| format!("E_WASM_STATE_PARAMS_INVALID: {error}"))?;
        parse_param_overrides(&json)
    }).transpose().map_err(|error| JsValue::from_str(&error))?;
    runtime_surface::run_summary_with_receipts_and_bogae_entry(
        source, path, ticks, Some(source_profile_identity), &receipts, params.as_ref(), bogae_entry.as_deref(),
    ).map(|summary| crate::unit_wire::SnapshotJsonOwner::from_value(summary).compact_text()).map_err(|error| JsValue::from_str(&error))
}

/// Existing Tape V1 bytes only; no full-input receipt file replay format.
#[wasm_bindgen]
pub fn wasm_export_input_tape_v1(receipts_json: &str, madi_hz: u32) -> Result<Vec<u8>, JsValue> {
    let receipts: JsonValue = serde_json::from_str(receipts_json)
        .map_err(|error| JsValue::from_str(&format!("E_INPUT_RECEIPT_INVALID: {error}")))?;
    crate::input_receipt::encode_tape_v1_from_receipts(&receipts, madi_hz)
        .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen]
pub fn wasm_run_summary_with_source_profile_tape_v1_json(
    source: &str, source_path: &str, source_profile_identity: &str, tape_bytes: &[u8],
) -> Result<String, JsValue> {
    let tape = crate::input_tape::decode_input_tape(tape_bytes)
        .map_err(|error| JsValue::from_str(&error))?;
    if tape.version != 1 {
        return Err(JsValue::from_str("E_SAM_TAPE_VERSION_UNSUPPORTED: 이 입구는 Tape V1만 받습니다"));
    }
    let path = require_wasm_source_path(source_path)?;
    runtime_surface::run_summary_from_supported_source_profile_with_input_tape(
        source, path, None, Some(source_profile_identity), &tape,
    ).map(|summary| crate::unit_wire::SnapshotJsonOwner::from_value(summary).compact_text()).map_err(|error| JsValue::from_str(&error))
}

/// Typed host-resource request entrypoint for RuntimeBridge/Workbench.
/// The bridge supplies sealed text payloads for record or a sealed transcript
/// for replay; it never interprets their semantic identity itself.
#[wasm_bindgen]
pub fn wasm_run_summary_with_source_profile_host_resources_json(
    source: &str,
    source_path: &str,
    requested_ticks: i32,
    source_profile_identity: &str,
    host_resource_request_json: &str,
) -> Result<String, JsValue> {
    let requested = if requested_ticks < 0 {
        None
    } else {
        Some(requested_ticks as u32)
    };
    let source_path = require_wasm_source_path(source_path)?;
    let request = host_resource::parse_request_json(host_resource_request_json)
        .map_err(|error| JsValue::from_str(&error))?;
    let prepared = host_resource::prepare_request(&request)
        .map_err(|error| JsValue::from_str(&error))?;
    runtime_surface::run_summary_from_supported_source_profile_with_host_resources(
        source,
        source_path,
        requested,
        Some(source_profile_identity),
        &prepared,
    )
    .map(|summary| crate::unit_wire::SnapshotJsonOwner::from_value(summary).compact_text())
    .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen]
pub fn wasm_model_prepared_state_json(source: &str, source_path: &str) -> Result<String, JsValue> {
    let source_path = require_wasm_source_path(source_path)?;
    runtime_surface::model_prepared_state_from_source(source, source_path)
        .map(|state| state.to_string())
        .map_err(|err| JsValue::from_str(&format!("WASM model prepared state 실패: {err}")))
}

/// Returns the bounded runtime-artifact primary-view decision for one run.
/// This is a candidate product ABI for the primary-only wave; callers must
/// render only a selected artifact and surface diagnostics for absent/ambiguous.
#[wasm_bindgen]
pub fn wasm_primary_view_decision_json(
    source: &str,
    requested_ticks: i32,
) -> Result<String, JsValue> {
    let requested = if requested_ticks < 0 {
        None
    } else {
        Some(requested_ticks as u32)
    };
    let summary =
        crate::unit_wire::SnapshotJsonOwner::from_value(runtime_surface::run_summary_from_source(source, "<wasm-primary-view>", requested)
            .map_err(|err| JsValue::from_str(&format!("WASM primary view 실패: {err}")))?);
    Ok(crate::unit_wire::SnapshotJsonOwner::clone_from_value(
        summary.get("primary_view_decision").unwrap_or(&JsonValue::Null)).compact_text())
}

#[wasm_bindgen]
pub fn wasm_normalize_project_source_json(source_json: &str) -> Result<String, JsValue> {
    local_project::normalize_virtual_project_source_json(source_json).map_err(|err| {
        JsValue::from_str(&format!(
            "{} {}",
            err.code,
            serde_json::to_string(&err.to_json_value()).unwrap_or_else(|_| err.message.clone())
        ))
    })
}

#[wasm_bindgen]
pub fn wasm_discover_project_root_json(request_json: &str) -> Result<String, JsValue> {
    local_project::discover_virtual_project_root_json(request_json).map_err(|err| {
        JsValue::from_str(&format!(
            "{} {}",
            err.code,
            serde_json::to_string(&err.to_json_value()).unwrap_or_else(|_| err.message.clone())
        ))
    })
}

#[wasm_bindgen]
pub fn wasm_project_graph_json(source_json: &str) -> Result<String, JsValue> {
    local_project::build_project_graph_json(source_json).map_err(|err| {
        JsValue::from_str(&format!(
            "{} {}",
            err.code,
            serde_json::to_string(&err.to_json_value()).unwrap_or_else(|_| err.message.clone())
        ))
    })
}

#[wasm_bindgen]
pub fn wasm_project_edit_plan_json(request_json: &str) -> Result<String, JsValue> {
    local_project::build_project_edit_plan_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_execution_recipe_json(request_json: &str) -> Result<String, JsValue> {
    local_project::build_project_execution_recipe_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_gaji_extraction_plan_json(request_json: &str) -> Result<String, JsValue> {
    local_project::build_gaji_extraction_plan_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_local_gaji_artifact_json(request_json: &str) -> Result<String, JsValue> {
    local_project::build_local_gaji_artifact_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_verify_local_gaji_artifact_json(artifact_json: &str) -> Result<String, JsValue> {
    local_project::verify_local_gaji_artifact_json(artifact_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_publish_local_gotgan_json(request_json: &str) -> Result<String, JsValue> {
    local_project::publish_local_gotgan_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_search_local_gotgan_json(request_json: &str) -> Result<String, JsValue> {
    local_project::search_local_gotgan_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_gaji_lifecycle_plan_json(request_json: &str) -> Result<String, JsValue> {
    local_project::build_gaji_lifecycle_plan_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_source_sha256_json(source_json: &str) -> Result<String, JsValue> {
    local_project::virtual_project_source_sha256_json(source_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_symbols_json(source_json: &str) -> Result<String, JsValue> {
    local_project::build_project_symbol_boundary_json(source_json).map_err(|err| {
        JsValue::from_str(&format!(
            "{} {}",
            err.code,
            serde_json::to_string(&err.to_json_value()).unwrap_or_else(|_| err.message.clone())
        ))
    })
}

#[wasm_bindgen]
pub fn wasm_project_authoring_snapshot_json(request_json: &str) -> Result<String, JsValue> {
    project_authoring::build_project_authoring_snapshot_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_authoring_definition_json(request_json: &str) -> Result<String, JsValue> {
    project_authoring::build_project_authoring_definition_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_authoring_references_json(request_json: &str) -> Result<String, JsValue> {
    project_authoring::build_project_authoring_references_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_authoring_hover_json(request_json: &str) -> Result<String, JsValue> {
    project_authoring::build_project_authoring_hover_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_authoring_completion_json(request_json: &str) -> Result<String, JsValue> {
    let request_schema = serde_json::from_str::<Map<String, JsonValue>>(request_json)
        .ok()
        .and_then(|request| request.get("schema").and_then(JsonValue::as_str).map(str::to_owned));
    if request_schema.as_deref() == Some(completion::SHARED_COMPLETION_SCHEMA) {
        return completion::build_structured_completion_json(request_json)
            .map_err(|err| JsValue::from_str(&err));
    }
    project_authoring::build_project_authoring_completion_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_project_authoring_rename_preview_json(request_json: &str) -> Result<String, JsValue> {
    project_authoring::build_project_authoring_rename_preview_json(request_json)
        .map_err(|err| JsValue::from_str(&err.to_json_value().to_string()))
}

#[wasm_bindgen]
pub fn wasm_corpus_csv_intake_json(request_json: &str) -> Result<String, JsValue> {
    corpus_csv::build_corpus_csv_intake_json(request_json).map_err(|err| {
        JsValue::from_str(&serde_json::to_string(&err).unwrap_or_else(|_| err.to_string()))
    })
}

#[wasm_bindgen]
pub fn wasm_corpus_columns_json(request_json: &str) -> Result<String, JsValue> {
    crate::corpus_authoring::build_corpus_columns_json(request_json).map_err(|err| {
        JsValue::from_str(&serde_json::to_string(&err).unwrap_or_else(|_| err.to_string()))
    })
}

#[wasm_bindgen]
pub fn wasm_corpus_sources_json(request_json: &str) -> Result<String, JsValue> {
    crate::corpus_authoring::build_corpus_sources_json(request_json).map_err(|err| {
        JsValue::from_str(&serde_json::to_string(&err).unwrap_or_else(|_| err.to_string()))
    })
}

#[wasm_bindgen]
pub fn wasm_corpus_evidence_json(request_json: &str) -> Result<String, JsValue> {
    crate::corpus_authoring::build_corpus_evidence_json(request_json).map_err(|err| {
        JsValue::from_str(&serde_json::to_string(&err).unwrap_or_else(|_| err.to_string()))
    })
}

#[wasm_bindgen]
pub fn wasm_corpus_review_json(request_json: &str) -> Result<String, JsValue> {
    crate::corpus_authoring::build_corpus_review_json(request_json).map_err(|err| {
        JsValue::from_str(&serde_json::to_string(&err).unwrap_or_else(|_| err.to_string()))
    })
}

#[wasm_bindgen]
pub fn wasm_corpus_seal_json(request_json: &str) -> Result<String, JsValue> {
    crate::corpus_authoring::build_corpus_seal_json(request_json).map_err(|err| {
        JsValue::from_str(&serde_json::to_string(&err).unwrap_or_else(|_| err.to_string()))
    })
}

#[wasm_bindgen]
pub fn wasm_project_gaji_lock_json(source_json: &str) -> Result<String, JsValue> {
    local_project::build_project_gaji_lock_json(source_json).map_err(|err| {
        JsValue::from_str(&format!(
            "{} {}",
            err.code,
            serde_json::to_string(&err.to_json_value()).unwrap_or_else(|_| err.message.clone())
        ))
    })
}

#[wasm_bindgen]
pub fn wasm_project_prepare_link_json(
    source_json: &str,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
) -> Result<String, JsValue> {
    local_project::build_project_prepare_link_json(source_json, target, inspect_graph, inspect_lock)
        .map_err(|err| {
            JsValue::from_str(
                &serde_json::to_string(&err.to_json_value())
                    .unwrap_or_else(|_| err.message.clone()),
            )
        })
}

#[wasm_bindgen]
pub fn wasm_project_run_summary_json(
    source_json: &str,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
) -> Result<String, JsValue> {
    local_project::build_project_run_summary_with_supported_profile_json(
        source_json,
        target,
        inspect_graph,
        inspect_lock,
    )
    .map_err(|err| {
            JsValue::from_str(
                &serde_json::to_string(&err.to_json_value())
                    .unwrap_or_else(|_| err.message.clone()),
            )
        })
}

/// RuntimeBridge/Workbench local-project variant of the same shared
/// host-resource invocation algorithm used by the native product frontdoor.
#[wasm_bindgen]
pub fn wasm_project_run_summary_with_host_resources_json(
    source_json: &str,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
    host_resource_request_json: &str,
) -> Result<String, JsValue> {
    local_project::build_project_run_summary_with_supported_profile_host_resources_json(
        source_json,
        target,
        inspect_graph,
        inspect_lock,
        host_resource_request_json,
    )
    .map_err(|err| {
        JsValue::from_str(
            &serde_json::to_string(&err.to_json_value())
                .unwrap_or_else(|_| err.message.clone()),
        )
    })
}

#[wasm_bindgen]
pub fn wasm_canon_flat_json(source: &str) -> Result<String, JsValue> {
    let output = canonicalize_flat_for_wasm(source)?;
    Ok(output.guseong_flat_json)
}

#[wasm_bindgen]
pub fn wasm_canon_ddn(source: &str) -> Result<String, JsValue> {
    let output = canonicalize_for_wasm(source)?;
    Ok(output.ddn)
}

/// 명시적 supported source profile용 내부 candidate canon entry다.
/// stable public WASM API 또는 V1 membership을 선언하지 않는다.
#[wasm_bindgen]
pub fn wasm_canon_ddn_with_source_profile(
    source: &str,
    source_profile_identity: &str,
) -> Result<String, JsValue> {
    let output =
        canonicalize_for_wasm_with_supported_profile(source, source_profile_identity)?;
    Ok(output.ddn)
}

/// 검사 전용 candidate projection이다. persistent public wire/ABI가 아니다.
#[wasm_bindgen]
pub fn wasm_model_ir_candidate_json(source: &str) -> Result<String, JsValue> {
    runtime_surface::model_ir_candidate_json_from_source(source)
        .map_err(|err| JsValue::from_str(&format!("WASM model IR 실패: {err}")))
}

/// Owner-approved bounded backend execution. The JSON wire remains a
/// versioned candidate and is not a public stable ABI.
#[wasm_bindgen]
pub fn wasm_model_run_candidate_json(source: &str) -> Result<String, JsValue> {
    crate::model_runtime::execute_source_pretty_json(source).map_err(|err| {
        JsValue::from_str(&format!("WASM model 실행 실패: {} {}", err, err.to_json()))
    })
}

/// Source labels are provenance only; they never participate in backend
/// selection. This candidate API lets project/CLI parity bind the same label.
#[wasm_bindgen]
pub fn wasm_model_run_candidate_json_with_source_file(
    source: &str,
    source_file: &str,
) -> Result<String, JsValue> {
    crate::model_runtime::execute_source_pretty_json_with_source_file(source, source_file).map_err(
        |err| JsValue::from_str(&format!("WASM model 실행 실패: {} {}", err, err.to_json())),
    )
}

/// Stored candidate execution evidence verifier. It never executes a fallback
/// backend and returns a structured fail-closed verdict for content tampering.
#[wasm_bindgen]
pub fn wasm_model_execution_verify_json(report_json: &str) -> Result<String, JsValue> {
    runtime_surface::model_execution_verify_json(report_json)
        .map_err(|err| JsValue::from_str(&format!("WASM model receipt 검증 실패: {err}")))
}

#[wasm_bindgen]
pub fn wasm_canon_maegim_plan(source: &str) -> Result<String, JsValue> {
    let output = canonicalize_flat_for_wasm(source)?;
    Ok(output.maegim_control_json)
}

#[wasm_bindgen]
pub fn wasm_canon_maegim_source_plan(source: &str, source_path: &str) -> Result<String, JsValue> {
    canon::maegim_control_plan_with_source(source, source_path)
        .map_err(|err| JsValue::from_str(&format!("WASM 채비 source plan 실패: {err}")))
}

#[wasm_bindgen]
pub fn wasm_apply_maegim_source_edit(
    source: &str,
    source_path: &str,
    control_name: &str,
    value_text: &str,
) -> Result<String, JsValue> {
    canon::apply_maegim_source_edit(source, source_path, control_name, value_text)
        .map_err(|err| JsValue::from_str(&format!("WASM 채비 원문 수정 실패: {err}")))
}

#[wasm_bindgen]
pub fn wasm_canon_alrim_plan(source: &str) -> Result<String, JsValue> {
    let output = canonicalize_flat_for_wasm(source)?;
    Ok(output.alrim_plan_json)
}

#[wasm_bindgen]
pub fn wasm_canon_exec_policy_map(source: &str) -> Result<String, JsValue> {
    if !canon::has_exec_policy_surface(source) {
        return Ok("{}\n".to_string());
    }
    let output = canonicalize_flat_for_wasm(source)?;
    Ok(output.exec_policy_map_json)
}

#[wasm_bindgen]
pub fn wasm_canon_block_editor_plan(source: &str) -> Result<String, JsValue> {
    let output = canonicalize_flat_for_wasm(source)?;
    Ok(output.block_editor_plan_json)
}

type PublicObservationSurface = crate::unit_wire::runtime_checkpoint::PublicObservationSurface;

fn build_wasm_state_snapshot_v2(
    resources: JsonValue,
    param_overrides: JsonValue,
    public_observation_frames: Vec<JsonValue>,
    next_tick_id: u64,
    scene_history: JsonValue,
) -> JsonValue {
    JsonValue::Object([
        ("schema".into(), WASM_STATE_SNAPSHOT_SCHEMA_V2.into()),
        ("resource_snapshot_schema".into(), RESOURCE_SNAPSHOT_SCHEMA_V2.into()),
        ("unit_wire_schema".into(), UNIT_VALUE_WIRE_SCHEMA_V2.into()),
        ("resources".into(), resources),
        ("param_overrides".into(), param_overrides),
        ("public_observation_frames".into(), JsonValue::Array(public_observation_frames)),
        ("next_tick_id".into(), next_tick_id.into()),
        ("scene_history".into(), scene_history),
    ].into_iter().collect())
}

fn last_committed_madi(frames: &[JsonValue]) -> JsonValue {
    frames
        .last()
        .and_then(|frame| frame.get("committed_madi"))
        .cloned()
        .unwrap_or(JsonValue::Null)
}

fn copy_wasm_json_rows(rows: &[JsonValue]) -> Vec<JsonValue> {
    rows.iter().map(|row| crate::unit_wire::SnapshotJsonOwner::clone_from_value(row)
        .into_parent_value()).collect()
}

/// Preserve the existing response keys; copy retained metadata only through
/// the common iterative owner, including optional top/state current values.
fn attach_wasm_observation_response(
    payload: &mut crate::unit_wire::SnapshotJsonOwner, vm: &DdnWasmVm,
) {
    for (key, value) in [
        ("public_observation_trace", &vm.public_observation_trace),
        ("public_observation_table_projection", &vm.public_observation_table_projection),
    ] {
        payload.value_mut()[key] = crate::unit_wire::SnapshotJsonOwner::clone_from_value(value).into_parent_value();
    }
    for (key, trace_key) in [("observation_columns", "columns"), ("output_rows", "samples")] {
        payload.value_mut()[key] = vm.public_observation_trace.get(trace_key)
            .map(|value| crate::unit_wire::SnapshotJsonOwner::clone_from_value(value).into_parent_value())
            .unwrap_or_else(|| JsonValue::Array(Vec::new()));
    }
    for in_state in [false, true] {
        if in_state && payload.get("state").is_none() { continue; }
        let target = if in_state { &mut payload.value_mut()["state"] } else { payload.value_mut() };
        if target.get("public_observations").is_some() {
            crate::unit_wire::replace_retained_json(&mut target["public_observations"],
                JsonValue::Array(copy_wasm_json_rows(&vm.last_public_observations)));
        }
        if target.get("debug_pause").is_some() {
            crate::unit_wire::replace_retained_json(&mut target["debug_pause"],
                crate::unit_wire::SnapshotJsonOwner::clone_from_value(&vm.last_debug_evidence).into_parent_value());
        }
    }
}

fn build_wasm_public_observation_surface(
    source_path: &str,
    source_bytes_sha256: &str,
    canonical_ddn_sha256: &str,
    frames: &[JsonValue],
) -> Result<PublicObservationSurface, String> {
    crate::unit_wire::runtime_checkpoint::build_public_observation_surface(
        source_path,source_bytes_sha256,canonical_ddn_sha256,frames)
}

fn append_wasm_public_observation_surface(
    vm: &DdnWasmVm,
) -> Result<PublicObservationSurface, String> {
    let frame = vm.public_observation_frames.last().ok_or_else(|| {
        "E_WASM_OBSERVATION_APPEND_FRAME_MISSING: 최신 frame이 없습니다".to_string()
    })?;
    let source_identity = format!("sha256:{}", vm.source_bytes_sha256);
    let revision_identity = format!("sha256:{}", vm.canonical_ddn_sha256);
    let run_id = format!(
        "run:{}",
        runtime_surface::sha256_hex(
            format!(
                "{}\0{}\0{}\0{}",
                vm.source_path,
                source_identity,
                revision_identity,
                vm.public_observation_frames.len()
            )
            .as_bytes()
        )
    );
    let delta_input = json!({
        "project_id": "",
        "run_id": run_id,
        "source_identity": source_identity,
        "revision_identity": revision_identity,
        "run_status": "completed",
        "frames": [frame],
    });
    let (trace, trace_hash, table_projection) = append_public_observation_surface(
        &vm.public_observation_trace,
        &vm.public_observation_table_projection,
        &delta_input,
    )?;
    Ok(PublicObservationSurface {
        trace:crate::unit_wire::SnapshotJsonOwner::from_value(trace),
        trace_hash,
        table_projection:crate::unit_wire::SnapshotJsonOwner::from_value(table_projection),
    })
}

struct NativeSourceBinding {authored:String,checkpoint_tick:u64,first_frame_pending:bool,first_input_receipt:Option<crate::unit_wire::SnapshotJsonOwner>,initial_reset_pending:bool}

// Fixed same-origin Native transport only. There is deliberately no public
// constructor accepting a caller-supplied frame, permit flag or HTTP URL.
fn native_source_exchange(request:&JsonValue,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>)->Result<crate::unit_wire::SnapshotJsonOwner,JsValue>{
    let xhr=web_sys::XmlHttpRequest::new().map_err(|_|JsValue::from_str("E_NATIVE_SOURCE_TRANSPORT_UNAVAILABLE"))?;
    xhr.open_with_async("POST","/__ddn/vc04/source",false)?;
    xhr.set_request_header("Content-Type","application/json")?;
    xhr.send_with_opt_str(Some(&request.to_string()))?;
    let text=xhr.response_text()?.ok_or_else(||JsValue::from_str("E_NATIVE_SOURCE_RESPONSE_EMPTY"))?;
    if text.len()>8*1024*1024{return Err(JsValue::from_str("E_NATIVE_SOURCE_RESPONSE_LIMIT"));}
    let parsed=crate::unit_wire::parse_snapshot_json_with_catalog(&text,catalog).map_err(|e|JsValue::from_str(&e))?;
    if xhr.status()?!=200{return Err(JsValue::from_str(parsed["error"].as_str().unwrap_or("E_NATIVE_SOURCE_REQUEST_REJECTED")));}
    Ok(parsed)
}

#[wasm_bindgen]
pub struct DdnWasmVm {
    native_source_binding:Option<NativeSourceBinding>,
    runner: DdnRunner,
    saved_project_owner: Option<Arc<crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1>>,
    world: DetNuri,
    defaults: HashMap<String, Value>,
    param_overrides: BTreeMap<String, Value>,
    tick_id: u64,
    rng_seed: u64,
    view_prefixes: Vec<String>,
    input_keys_pressed: u64,
    input_last_key_name: String,
    input_pointer_x_i32: i32,
    input_pointer_y_i32: i32,
    input_dt: Fixed64,
    last_patch: Option<Patch>,
    last_bogae_projection: Option<JsonValue>,
    display_outputs: Vec<crate::output_display::OutputDisplay>,
    pending_ai_injections: Vec<(String, String)>,
    lang_mode: ParseMode,
    parse_warnings: Vec<DdnParseWarning>,
    configured_madi: u64,
    execution_mode_decision: ExecutionModeDecision,
    source_path: String,
    source_bytes_sha256: String,
    canonical_ddn_sha256: String,
    debug_evaluator: Option<DebugEvaluator>,
    last_debug_evidence: JsonValue,
    last_public_observations: Vec<JsonValue>,
    public_observation_frames: Vec<JsonValue>,
    public_observation_trace: JsonValue,
    public_observation_trace_hash: String,
    public_observation_table_projection: JsonValue,
    project_asset_registry: Option<Arc<crate::gate0_registry::ProjectAssetRegistry>>,
}

impl DdnWasmVm {
    /// Same raw project, actual protected Native owner and the exact binary
    /// checkpoint. Standalone/Node source readback cannot take this route.
    fn from_native_project_connected(authored:&str)->Result<DdnWasmVm,JsValue>{
        let reply=native_source_exchange(&json!({"operation":"connect","authored_project_json_utf8":authored}),None)?;
        let text=|key:&str|reply[key].as_str().ok_or_else(||JsValue::from_str("E_NATIVE_SOURCE_RESPONSE_FIELDS"));
        let artifact=text("saved_artifact_json_utf8")?;
        let parent=decode_storage_lineage(text("parent_hex")?,false)?;
        let revision=decode_storage_lineage(text("revision_hex")?,false)?;
        let snapshot=hex::decode(text("snapshot_hex")?).map_err(|_|JsValue::from_str("E_NATIVE_SOURCE_FRAME"))?;
        let bytes=crate::unit_wire::project_runtime_attachment::check_bytes(artifact.as_bytes()).map_err(|e|JsValue::from_str(&e))?;
        let owned=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&bytes).map_err(|e|JsValue::from_str(&e))?;
        if owned.project().save().map_err(|e|JsValue::from_str(&e))?!=authored{return Err(JsValue::from_str("E_NATIVE_SOURCE_SOURCE_CHANGED"));}
        let pesi=crate::project_snapshot_identity::current::issue_saved_local_project(&owned).map_err(|e|JsValue::from_str(&e))?;
        let layout=crate::unit_wire::check_resumable_complete_state_layout_v2(&snapshot,pesi.pesi().as_raw())
            .map_err(|e|JsValue::from_str(&format!("E_NATIVE_SOURCE_FRAME: {e:?}")))?;
        let context=crate::unit_wire::project_runtime_attachment::check_saved_revision_context_with_frame(&bytes,&owned,&parent,&revision,layout.world)
            .map_err(|e|JsValue::from_str(&e))?;
        let descriptor=crate::unit_wire::resumable_complete_state::rebuild_general_complete_state_source_candidate(&layout,&context)
            .map_err(|e|JsValue::from_str(&format!("E_NATIVE_SOURCE_SUPPLY: {e:?}")))?;
        let bootstrap=crate::unit_wire::subject_execution::SubjectExecutionStage::from_recorded_initial(&owned,&descriptor.initial_world)
            .map_err(|e|JsValue::from_str(&e))?;
        let mut candidate=build_saved_authored_project_vm(Arc::new(owned.clone()))?;
        if let Some(stage)=bootstrap{candidate.runner.bind_subject_bootstrap(stage);}
        candidate.restore_state_with_complete_frame(context.runtime_state(),Some(layout.world))?;
        let first_frame_pending=reply["first_frame_pending"].as_bool().ok_or_else(||JsValue::from_str("E_NATIVE_SOURCE_RESPONSE_FIELDS"))?;
        let first_input_receipt=if first_frame_pending {
            let receipt=&reply["first_input_receipt"];
            let delivered=crate::input_receipt::parse_receipt_snapshot(receipt).map_err(|e|JsValue::from_str(&e))?;
            let inputs=crate::unit_wire::resumable_complete_state::decode_general_processed_inputs(layout.supplies[4],descriptor.initial_tick,descriptor.base_rng_seed)
                .map_err(|e|JsValue::from_str(&format!("E_NATIVE_SOURCE_INPUT_RECEIPT: {e:?}")))?;
            let actual=inputs.last().ok_or_else(||JsValue::from_str("E_NATIVE_SOURCE_INPUT_RECEIPT"))?;
            if delivered.tick_id.checked_add(1)!=Some(candidate.tick_id)||crate::input_receipt::snapshot_sha256(&delivered)!=crate::input_receipt::snapshot_sha256(&actual.input) {
                return Err(JsValue::from_str("E_NATIVE_SOURCE_INPUT_RECEIPT"));
            }
            Some(crate::unit_wire::SnapshotJsonOwner::clone_from_value(receipt))
        }else{None};
        candidate.native_source_binding=Some(NativeSourceBinding{authored:authored.into(),checkpoint_tick:candidate.tick_id,
            first_frame_pending,first_input_receipt,initial_reset_pending:true});
        Ok(candidate)
    }
    fn clear_retained_observation_json(&mut self) {
        crate::unit_wire::clear_retained_json_option(&mut self.last_bogae_projection);
        crate::unit_wire::clear_retained_json_rows(&mut self.last_public_observations);
        crate::unit_wire::clear_retained_json_rows(&mut self.public_observation_frames);
        crate::unit_wire::replace_retained_json(&mut self.public_observation_trace, JsonValue::Null);
        crate::unit_wire::replace_retained_json(&mut self.public_observation_table_projection, JsonValue::Null);
    }
}

impl Drop for DdnWasmVm {
    fn drop(&mut self) {
        self.clear_retained_observation_json();
        crate::unit_wire::replace_retained_json(&mut self.last_debug_evidence, JsonValue::Null);
    }
}

fn update_wasm_logic_with_supported_source_profile(
    vm: &mut DdnWasmVm,
    source: &str,
    source_profile_identity: &str,
) -> Result<(), JsValue> {
    let profile = resolve_supported_source_profile(Some(source_profile_identity))
        .map_err(|error| JsValue::from_str(&format!("{}: {}", error.code, error.message)))?;
    let mut program = DdnProgram::from_supported_source_profile_with_mode(
        source,
        &vm.source_path,
        vm.lang_mode,
        profile,
    )
    .map_err(|error| JsValue::from_str(&error))?;
    source_analysis::validate_program_for_execution(program.canonical_program())
        .map_err(|error| JsValue::from_str(&error))?;
    program.prepare_temporary_source_execution().map_err(|error|JsValue::from_str(&error))?;
    let canonical_ddn = program.canonical_ddn();
    vm.configured_madi = program.configured_madi().unwrap_or(0);
    vm.execution_mode_decision = program.execution_mode_decision();
    vm.parse_warnings = program.parse_warnings().to_vec();
    vm.runner = DdnRunner::new(program, DEFAULT_UPDATE_NAME).with_bogae_activation();
    vm.saved_project_owner = None;
    vm.runner
        .set_project_asset_registry(vm.project_asset_registry.clone());
    vm.source_bytes_sha256 = runtime_surface::sha256_hex(source.as_bytes());
    vm.canonical_ddn_sha256 = runtime_surface::sha256_hex(canonical_ddn.as_bytes());
    vm.debug_evaluator = None;
    crate::unit_wire::replace_retained_json(&mut vm.last_debug_evidence, json!({"triggered": false, "reasons": []}));
    reset_wasm_public_observation_surface(vm).map_err(|error| JsValue::from_str(&error))?;
    Ok(())
}

fn apply_currentline_result_to_vm(
    vm: &mut DdnWasmVm,
    result: ddonirang_lang::CurrentLineResult,
    source_profile_identity: Option<&str>,
) -> Result<JsValue, JsValue> {
    if let Some(source_profile_identity) = source_profile_identity {
        update_wasm_logic_with_supported_source_profile(
            vm,
            &result.project_source,
            source_profile_identity,
        )?;
    } else {
        vm.update_logic(&result.project_source)?;
    }
    // A rejected edit must not erase the state of the previous valid program.
    let _ = vm.reset(Some(false))?;
    let state = vm.run_ticks(1)?;
    let state_text = state
        .as_string()
        .ok_or_else(|| JsValue::from_str("apply_currentline_cell: state JSON 누락"))?;
    let mut payload: JsonValue = serde_json::from_str(&state_text).map_err(|error| {
        JsValue::from_str(&format!(
            "apply_currentline_cell: state JSON 파싱 실패: {error}"
        ))
    })?;
    let context: JsonValue = serde_json::from_str(&result.context_json).map_err(|error| {
        JsValue::from_str(&format!(
            "apply_currentline_cell: context JSON 생성 실패: {error}"
        ))
    })?;
    if let Some(map) = payload.as_object_mut() {
        map.insert("currentline_context".to_string(), context.clone());
        if let Some(state) = map.get_mut("state").and_then(JsonValue::as_object_mut) {
            state.insert("currentline_context".to_string(), context);
        }
    }
    Ok(JsValue::from_str(&payload.to_string()))
}

fn refresh_wasm_public_observation_surface(vm: &mut DdnWasmVm) -> Result<(), String> {
    let surface = if vm.public_observation_frames.is_empty() {
        build_wasm_public_observation_surface(
            &vm.source_path,
            &vm.source_bytes_sha256,
            &vm.canonical_ddn_sha256,
            &vm.public_observation_frames,
        )?
    } else {
        append_wasm_public_observation_surface(vm)?
    };
    crate::unit_wire::replace_retained_json(&mut vm.public_observation_trace, surface.trace.into_parent_value());
    vm.public_observation_trace_hash = surface.trace_hash;
    crate::unit_wire::replace_retained_json(&mut vm.public_observation_table_projection, surface.table_projection.into_parent_value());
    Ok(())
}

fn reset_wasm_public_observation_surface(vm: &mut DdnWasmVm) -> Result<(), String> {
    vm.clear_retained_observation_json();
    vm.display_outputs.clear();
    refresh_wasm_public_observation_surface(vm)
}

struct RestoredInputState {
    keys_pressed: u64,
    last_key_name: String,
    pointer_x_i32: i32,
    pointer_y_i32: i32,
    rng_seed: u64,
    dt: Fixed64,
}

type StagedWasmStateRestore = crate::unit_wire::runtime_checkpoint::StagedRuntimeState;

fn require_schema(
    object: &Map<String, JsonValue>,
    key: &str,
    expected: &str,
    code: &str,
) -> Result<(), String> {
    let actual = object.get(key).and_then(JsonValue::as_str);
    if actual != Some(expected) {
        return Err(format!(
            "{code}: expected {expected}, got {}",
            actual.unwrap_or("<missing>")
        ));
    }
    Ok(())
}

fn parse_restored_input(payload: Option<&JsonValue>) -> Result<RestoredInputState, String> {
    let Some(payload) = payload else {
        return Err("E_INPUT_RECORD_LEGACY_UNSUPPORTED: 입력 기록이 없어 정확한 dt를 확인할 수 없습니다".into());
    };
    let JsonValue::Object(obj) = payload else {
        return Err("E_WASM_STATE_INPUT_INVALID: state.input은 object여야 합니다".to_string());
    };
    let parse_u64 = |key: &str| -> Result<Option<u64>, String> {
        obj.get(key)
            .map(|value| {
                json_to_u64(value)
                    .ok_or_else(|| format!("E_WASM_STATE_INPUT_INVALID: state.input.{key}"))
            })
            .transpose()
    };
    let parse_i32 = |key: &str| -> Result<Option<i32>, String> {
        obj.get(key)
            .map(|value| {
                json_to_i32(value)
                    .ok_or_else(|| format!("E_WASM_STATE_INPUT_INVALID: state.input.{key}"))
            })
            .transpose()
    };
    let last_key_name = match obj.get("last_key_name") {
        Some(JsonValue::String(value)) => value.clone(),
        Some(_) => {
            return Err(
                "E_WASM_STATE_INPUT_INVALID: state.input.last_key_name은 string이어야 합니다"
                    .to_string(),
            )
        }
        None => String::new(),
    };
    let dt = crate::input_receipt::parse_recorded_dt(payload)?;
    Ok(RestoredInputState {
        keys_pressed: parse_u64("keys_pressed")?.unwrap_or(0),
        last_key_name,
        pointer_x_i32: parse_i32("pointer_x_i32")?.unwrap_or(0),
        pointer_y_i32: parse_i32("pointer_y_i32")?.unwrap_or(0),
        rng_seed: parse_u64("rng_base_seed")?
            .or(parse_u64("rng_seed")?)
            .unwrap_or(0),
        dt,
    })
}

fn stage_wasm_state_restore(
    payload: &JsonValue,
    source_path: &str,
    source_bytes_sha256: &str,
    canonical_ddn_sha256: &str,
) -> Result<StagedWasmStateRestore, String> {
    crate::unit_wire::runtime_checkpoint::stage_runtime_state(
        payload,source_path,source_bytes_sha256,canonical_ddn_sha256)
}

fn build_wasm_vm_from_program(
    mut program: DdnProgram,
    source: &str,
    source_path: &str,
    execution_mode_decision: ExecutionModeDecision,
) -> Result<DdnWasmVm, JsValue> {
        program.prepare_temporary_source_execution().map_err(|error|JsValue::from_str(&error))?;
        let configured_madi = program.configured_madi().unwrap_or(0);
        let parse_warnings = program.parse_warnings().to_vec();
        let prepared = if program.sealed_project_sources().is_some() {
            runtime_surface::prepare_supported_source_for_execution_with_program(source,source_path,&program)
                .map_err(|error|JsValue::from_str(&format!("{}: {}",error.code,error.message)))?
        } else {
            runtime_surface::prepare_source_for_execution(source)
                .map_err(|err|JsValue::from_str(&format!("WASM preprocess 실패: {err}")))?
        };
        let source_bytes_sha256 = runtime_surface::sha256_hex(source.as_bytes());
        let canonical_ddn_sha256 = runtime_surface::sha256_hex(prepared.canonical_ddn.as_bytes());
        let observation_surface = build_wasm_public_observation_surface(
            source_path,
            &source_bytes_sha256,
            &canonical_ddn_sha256,
            &[],
        )
        .map_err(|err| JsValue::from_str(&err))?;
        let mut defaults = HashMap::new();
        seed_bogae_defaults(&mut defaults);
        Ok(DdnWasmVm {
            runner: DdnRunner::new(program, DEFAULT_UPDATE_NAME).with_bogae_activation(),
            saved_project_owner: None,
            native_source_binding:None,
            world: DetNuri::new(),
            defaults,
            param_overrides: BTreeMap::new(),
            tick_id: 0,
            rng_seed: 0,
            view_prefixes: DEFAULT_VIEW_PREFIXES
                .iter()
                .map(|prefix| (*prefix).to_string())
                .collect(),
            input_keys_pressed: 0,
            input_last_key_name: String::new(),
            input_pointer_x_i32: 0,
            input_pointer_y_i32: 0,
            input_dt: crate::input_receipt::alpha_host_dt(),
            last_patch: None,
            last_bogae_projection: None,
            display_outputs: Vec::new(),
            pending_ai_injections: Vec::new(),
            lang_mode: ParseMode::Strict,
            parse_warnings,
            configured_madi,
            execution_mode_decision,
            source_path: source_path.to_string(),
            source_bytes_sha256,
            canonical_ddn_sha256,
            debug_evaluator: None,
            last_debug_evidence: json!({"triggered": false, "reasons": []}),
            last_public_observations: Vec::new(),
            public_observation_frames: Vec::new(),
            public_observation_trace: observation_surface.trace.into_parent_value(),
            public_observation_trace_hash: observation_surface.trace_hash,
            public_observation_table_projection: observation_surface.table_projection.into_parent_value(),
            project_asset_registry: None,
        })
}

/// Internal candidate restore: no public JSON admission or live-VM mutation.
/// The saved-source owner supplies the typed capture after revision admission.
pub(crate) fn restore_saved_v25_candidate(
    capture: crate::source_snapshot_admission::SourceCapture,
    source_path: &str,
    state_json: &str,
    resource_snapshot_json: &str,
) -> Result<DdnWasmVm, JsValue> {
    crate::unit_wire::project_runtime_attachment::check_runtime_resource_binding(
        state_json, resource_snapshot_json,
    ).map_err(|err| JsValue::from_str(&err))?;
    let source_path = require_wasm_source_path(source_path)?;
    let program = DdnProgram::from_sealed_supported_source_profile_with_mode(
        capture.clone(), source_path, ParseMode::Strict,
        crate::ddn_runtime::SupportedSourceProfile::V1CoreV25,
        crate::ddn_runtime::default_age_target(),
    ).map_err(|err| JsValue::from_str(&err))?;
    source_analysis::validate_program_for_execution(program.canonical_program())
        .map_err(|err| JsValue::from_str(&err))?;
    let prepared = runtime_surface::prepare_source_for_execution(capture.text())
        .map_err(|err| JsValue::from_str(&err))?;
    let mut candidate = build_wasm_vm_from_program(
        program, capture.text(), source_path, prepared.execution_mode_decision,
    )?;
    candidate.restore_state(state_json)?;
    crate::source_snapshot_admission::retain_verified_replay(&[&capture])
        .map_err(|error|JsValue::from_str(&format!("E_SOURCE_ADMISSION_CUSTODY: {error:?}")))?;
    Ok(candidate)
}

/// Project restore uses the linked execution unit and the root-resolved start.
/// The public artifact entrypoint calls this only after source, payload and
/// revision/PESI checks. No existing VM is mutated while building a candidate.
pub(crate) fn restore_saved_v25_project_candidate(
    context: &crate::unit_wire::project_runtime_attachment::SourceBoundSavedRuntimeAttachment<'_>,
) -> Result<DdnWasmVm, JsValue> {
    let owned=context.owned();
    let state_json=context.runtime_state();
    let resource_snapshot_json=context.resource_snapshot();
    let catalog=owned.registration().map(|registration|registration.saved_nominal_catalog(owned))
        .transpose().map_err(|err|JsValue::from_str(&err))?;
    crate::unit_wire::project_runtime_attachment::check_runtime_resource_binding_with_catalog(
        state_json,resource_snapshot_json,catalog.as_ref(),
    ).map_err(|err|JsValue::from_str(&err))?;
    let mut candidate=build_saved_authored_project_vm(Arc::new(owned.clone()))?;
    candidate.restore_state(state_json)?;
    let captures=owned.sources().captures().iter().map(|(_,capture)|capture).collect::<Vec<_>>();
    crate::source_snapshot_admission::retain_verified_replay(&captures)
        .map_err(|error|JsValue::from_str(&format!("E_SOURCE_ADMISSION_CUSTODY: {error:?}")))?;
    Ok(candidate)
}

fn build_saved_authored_project_vm(
    owned:Arc<crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1>,
) -> Result<DdnWasmVm,JsValue> {
    // Prove the supported local-only closure before publishing any candidate.
    crate::project_snapshot_identity::current::issue_saved_local_project(&owned)
        .map_err(|error|JsValue::from_str(&error))?;
    let saved=owned.sources();
    let mut program=DdnProgram::from_saved_v25_project_with_mode(
        saved.clone(),ParseMode::Strict,crate::ddn_runtime::default_age_target(),
    ).map_err(|err|JsValue::from_str(&err))?;
    program.bind_registered_project_context(&owned)
        .map_err(|err|JsValue::from_str(&err))?;
    source_analysis::validate_program_for_execution(program.canonical_program())
        .map_err(|err|JsValue::from_str(&err))?;
    let source=&saved.preparation().source;
    let source_path=require_wasm_source_path(&saved.preparation().target_path)?;
    let prepared=runtime_surface::prepare_supported_source_for_execution_with_program(source,source_path,&program)
        .map_err(|error|JsValue::from_str(&format!("{}: {}",error.code,error.message)))?;
    let mut candidate=build_wasm_vm_from_program(
        program,source,source_path,prepared.execution_mode_decision,
    )?;
    candidate.configure_project_entry(saved.entry_declaration().name())?;
    let project=owned.project();
    if project.schema==crate::authored_project::SCHEMA_V2
        || project.authored_identity.modules.iter().any(|module|!module.compositions.is_empty()) {
        let artifact=project.save().map_err(|err|JsValue::from_str(&err))?;
        candidate.configure_authored_bogae(&artifact)?;
    }
    candidate.saved_project_owner=Some(owned);
    Ok(candidate)
}

fn decode_storage_lineage(text:&str,allow_empty:bool) -> Result<Vec<u8>,JsValue> {
    if (text.is_empty()&&!allow_empty) || text.len()>8192 || text.len()%2!=0
        || !text.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)) {
        return Err(JsValue::from_str("E_PROJECT_RUNTIME_STORAGE_LINEAGE: 저장 이력 정체가 정본이 아닙니다"));
    }
    hex::decode(text).map_err(|_|JsValue::from_str("E_PROJECT_RUNTIME_STORAGE_LINEAGE"))
}

#[wasm_bindgen]
impl DdnWasmVm {
    pub fn from_native_project_json(authored:&str)->Result<DdnWasmVm,JsValue>{Self::from_native_project_connected(authored)}
    /// Explicit project execution retains exactly the captures that this VM
    /// executes. Saving does not reread source or allocate replacement ordinals.
    pub fn from_authored_project_json(artifact_json:&str) -> Result<DdnWasmVm,JsValue> {
        let owner=crate::project_revision_envelope_supply::admit_fresh_authored_project(artifact_json.as_bytes())
            .map_err(|error|JsValue::from_str(&error))?;
        build_saved_authored_project_vm(Arc::new(owner))
    }

    pub fn is_native_source_bound(&self)->bool{self.native_source_binding.is_some()}

    pub fn save_native_source_checkpoint(&self,authored:&str)->Result<String,JsValue>{
        let Some(binding)=self.native_source_binding.as_ref()else{return Ok("null".into())};
        if binding.authored!=authored||binding.first_frame_pending||!self.pending_ai_injections.is_empty(){
            return Err(JsValue::from_str("E_NATIVE_SOURCE_SAVE_CONTEXT: 현재 실행 원문과 처리 입력을 확인하세요"));
        }
        let catalog=self.saved_project_owner.as_ref().and_then(|owner|owner.registration().map(|registration|registration.saved_nominal_catalog(owner)))
            .transpose().map_err(|e|JsValue::from_str(&e))?;
        let result=native_source_exchange(&json!({"operation":"save_world","authored_project_json_utf8":binding.authored,
            "next_tick":self.tick_id.to_string(),"state_hash":format!("blake3:{}",self.world.state_hash().to_hex())}),catalog.as_ref())?;
        Ok(result.compact_text())
    }

    pub fn export_saved_project_artifact_json(
        &self,expected_parent_hex:&str,revision_hex:&str,
    ) -> Result<String,JsValue> {
        if self.runner.has_temporary_registration(){return Err(JsValue::from_str("E_VC04_TEMPORARY_SAVED_ESCAPE: 임시 실행을 저장 작품 상태로 승격할 수 없습니다"))}
        if self.native_source_binding.is_some(){return Err(JsValue::from_str("E_NATIVE_SOURCE_FOLLOWUP_SAVE_UNSUPPORTED: 현재 Native 실행의 후속 저장 연결은 아직 지원하지 않습니다"));}
        if !self.pending_ai_injections.is_empty() {
            return Err(JsValue::from_str("E_PROJECT_RUNTIME_PENDING_INPUT: 아직 처리하지 않은 입력이 있어 실행 상태를 저장할 수 없습니다."));
        }
        let owner=self.saved_project_owner.as_ref()
            .ok_or_else(||JsValue::from_str("E_PROJECT_RUNTIME_OWNER_REQUIRED: 명시 작품으로 실행한 상태만 저장할 수 있습니다"))?;
        let parent=decode_storage_lineage(expected_parent_hex,true)?;
        let revision=decode_storage_lineage(revision_hex,false)?;
        let runtime=self.get_state_json().as_string()
            .ok_or_else(||JsValue::from_str("E_PROJECT_RUNTIME_STATE_ENCODING"))?;
        let catalog=owner.registration().map(|registration|registration.saved_nominal_catalog(owner)).transpose()
            .map_err(|error|JsValue::from_str(&error))?;
        let parsed=crate::unit_wire::parse_snapshot_json_with_catalog(&runtime,catalog.as_ref())
            .map_err(|error|JsValue::from_str(&error))?;
        let resources=parsed.pointer("/state/snapshot_v2/resources")
            .ok_or_else(||JsValue::from_str("E_PROJECT_RUNTIME_RESOURCE_SCHEMA"))?;
        let resources=crate::unit_wire::serialize_resource_snapshot_json(resources);
        crate::unit_wire::project_runtime_attachment::encode_saved_runtime_artifact(
            owner,&runtime,&resources,&parent,&revision,
        ).map_err(|error|JsValue::from_str(&error))
    }

    /// Restores into a separate candidate. The host supplies the revision pair
    /// selected from its storage lineage; neither value is taken from the file.
    /// A failure leaves every existing VM untouched.
    pub fn from_saved_project_artifact_json(
        artifact_json: &str, expected_parent_hex: &str, revision_hex: &str,
    ) -> Result<DdnWasmVm,JsValue> {
        let parent=decode_storage_lineage(expected_parent_hex,true)?;
        let revision=decode_storage_lineage(revision_hex,false)?;
        let attachment=crate::unit_wire::project_runtime_attachment::check_bytes(artifact_json.as_bytes())
            .map_err(|error|JsValue::from_str(&error))?;
        let owned=crate::unit_wire::project_runtime_attachment::restore_saved_runtime_payload(&attachment)
            .map_err(|error|JsValue::from_str(&error))?;
        let context=crate::unit_wire::project_runtime_attachment::check_saved_revision_context(
            &attachment,&owned,&parent,&revision,
        ).map_err(|error|JsValue::from_str(&error))?;
        restore_saved_v25_project_candidate(&context)
    }

    #[wasm_bindgen(constructor)]
    pub fn new(source: &str) -> Result<DdnWasmVm, JsValue> {
        let source_path = "<wasm>";
        let (program, execution_mode_decision) =
            parse_program_for_wasm(source, source_path, ParseMode::Strict)?;
        build_wasm_vm_from_program(program, source, source_path, execution_mode_decision)
    }

    pub fn new_with_mode(source: &str, mode: &str) -> Result<DdnWasmVm, JsValue> {
        let source_path = "<wasm>";
        let mode = parse_mode_from_str(mode)?;
        let (program, execution_mode_decision) = parse_program_for_wasm(source, source_path, mode)?;
        let configured_madi = program.configured_madi().unwrap_or(0);
        let parse_warnings = program.parse_warnings().to_vec();
        let prepared = runtime_surface::prepare_source_for_execution(source)
            .map_err(|err| JsValue::from_str(&format!("WASM preprocess 실패: {err}")))?;
        let source_bytes_sha256 = runtime_surface::sha256_hex(source.as_bytes());
        let canonical_ddn_sha256 = runtime_surface::sha256_hex(prepared.canonical_ddn.as_bytes());
        let observation_surface = build_wasm_public_observation_surface(
            source_path,
            &source_bytes_sha256,
            &canonical_ddn_sha256,
            &[],
        )
        .map_err(|err| JsValue::from_str(&err))?;
        let mut defaults = HashMap::new();
        seed_bogae_defaults(&mut defaults);
        Ok(DdnWasmVm {
            runner: DdnRunner::new(program, DEFAULT_UPDATE_NAME).with_bogae_activation(),
            saved_project_owner: None,
            native_source_binding:None,
            world: DetNuri::new(),
            defaults,
            param_overrides: BTreeMap::new(),
            tick_id: 0,
            rng_seed: 0,
            view_prefixes: DEFAULT_VIEW_PREFIXES
                .iter()
                .map(|prefix| (*prefix).to_string())
                .collect(),
            input_keys_pressed: 0,
            input_last_key_name: String::new(),
            input_pointer_x_i32: 0,
            input_pointer_y_i32: 0,
            input_dt: crate::input_receipt::alpha_host_dt(),
            last_patch: None,
            last_bogae_projection: None,
            display_outputs: Vec::new(),
            pending_ai_injections: Vec::new(),
            lang_mode: mode,
            parse_warnings,
            configured_madi,
            execution_mode_decision,
            source_path: source_path.to_string(),
            source_bytes_sha256,
            canonical_ddn_sha256,
            debug_evaluator: None,
            last_debug_evidence: json!({"triggered": false, "reasons": []}),
            last_public_observations: Vec::new(),
            public_observation_frames: Vec::new(),
            public_observation_trace: observation_surface.trace.into_parent_value(),
            public_observation_trace_hash: observation_surface.trace_hash,
            public_observation_table_projection: observation_surface.table_projection.into_parent_value(),
            project_asset_registry: None,
        })
    }

    pub fn new_with_source_path(source: &str, source_path: &str) -> Result<DdnWasmVm, JsValue> {
        let source_path = require_wasm_source_path(source_path)?;
        let mut vm = DdnWasmVm::new(source)?;
        vm.update_logic_with_source_path(source, source_path)?;
        Ok(vm)
    }

    pub fn update_logic(&mut self, source: &str) -> Result<(), JsValue> {
        let source_path = self.source_path.clone();
        self.update_logic_with_source_path(source, &source_path)
    }

    pub fn update_logic_with_source_profile(
        &mut self,
        source: &str,
        source_profile_identity: &str,
    ) -> Result<(), JsValue> {
        update_wasm_logic_with_supported_source_profile(self, source, source_profile_identity)
    }

    pub fn update_logic_with_source_path(
        &mut self,
        source: &str,
        source_path: &str,
    ) -> Result<(), JsValue> {
        let source_path = require_wasm_source_path(source_path)?;
        let (program, execution_mode_decision) =
            parse_program_for_wasm(source, source_path, self.lang_mode)?;
        let prepared = runtime_surface::prepare_source_for_execution(source)
            .map_err(|err| JsValue::from_str(&format!("WASM preprocess 실패: {err}")))?;
        self.configured_madi = program.configured_madi().unwrap_or(0);
        self.execution_mode_decision = execution_mode_decision;
        self.parse_warnings = program.parse_warnings().to_vec();
        self.runner = DdnRunner::new(program, DEFAULT_UPDATE_NAME).with_bogae_activation();
        self.saved_project_owner = None;
        self.runner
            .set_project_asset_registry(self.project_asset_registry.clone());
        self.source_path = source_path.to_string();
        self.source_bytes_sha256 = runtime_surface::sha256_hex(source.as_bytes());
        self.canonical_ddn_sha256 = runtime_surface::sha256_hex(prepared.canonical_ddn.as_bytes());
        self.debug_evaluator = None;
        crate::unit_wire::replace_retained_json(&mut self.last_debug_evidence, json!({"triggered": false, "reasons": []}));
        reset_wasm_public_observation_surface(self).map_err(|err| JsValue::from_str(&err))?;
        Ok(())
    }

    pub fn get_configured_madi(&self) -> u64 {
        self.configured_madi
    }

    /// Bind the entry selected by the explicit work-root linker. Call again
    /// after installing a replacement program; no filename or name inference.
    pub fn configure_project_entry(&mut self, name: &str) -> Result<(), JsValue> {
        if let Some(owner) = &self.saved_project_owner {
            if name != owner.sources().entry_declaration().name() {
                return Err(JsValue::from_str("E_PROJECT_RUNTIME_ENTRY_CHANGED: 저장 원문의 시작 정의를 바꿀 수 없습니다."));
            }
        }
        self.runner.configure_project_entry(name).map_err(|err| JsValue::from_str(&err))
    }

    pub fn configure_bogae_entry(&mut self, name: &str) -> Result<(), JsValue> {
        self.runner.configure_bogae_entry(name).map_err(|err|JsValue::from_str(&err))
    }

    pub fn configure_authored_bogae(&mut self, artifact_json: &str) -> Result<(), JsValue> {
        if let Some(owner) = &self.saved_project_owner {
            let expected = owner.project().save().map_err(|err|JsValue::from_str(&err))?;
            if artifact_json != expected {
                return Err(JsValue::from_str("E_PROJECT_RUNTIME_OWNER_CHANGED: 실행 중인 저장 작품의 보개 정체를 다른 작품으로 바꿀 수 없습니다."));
            }
        }
        if self.tick_id != 0 {
            return Err(JsValue::from_str("E_AUTHORED_BOGAE_LIFECYCLE: 실행을 시작한 뒤 정체를 교체할 수 없습니다"));
        }
        self.runner.configure_authored_bogae(artifact_json).map_err(|err| JsValue::from_str(&err))
    }

    pub fn get_execution_mode_decision_json(&self) -> String {
        runtime_surface::execution_mode_decision_json(self.execution_mode_decision).to_string()
    }

    pub fn update_logic_with_mode(&mut self, source: &str, mode: &str) -> Result<(), JsValue> {
        let mode = parse_mode_from_str(mode)?;
        let (program, execution_mode_decision) =
            parse_program_for_wasm(source, &self.source_path, mode)?;
        let prepared = runtime_surface::prepare_source_for_execution(source)
            .map_err(|err| JsValue::from_str(&format!("WASM preprocess 실패: {err}")))?;
        self.configured_madi = program.configured_madi().unwrap_or(0);
        self.execution_mode_decision = execution_mode_decision;
        self.parse_warnings = program.parse_warnings().to_vec();
        self.runner = DdnRunner::new(program, DEFAULT_UPDATE_NAME).with_bogae_activation();
        self.saved_project_owner = None;
        self.runner
            .set_project_asset_registry(self.project_asset_registry.clone());
        self.lang_mode = mode;
        self.source_bytes_sha256 = runtime_surface::sha256_hex(source.as_bytes());
        self.canonical_ddn_sha256 = runtime_surface::sha256_hex(prepared.canonical_ddn.as_bytes());
        self.debug_evaluator = None;
        crate::unit_wire::replace_retained_json(&mut self.last_debug_evidence, json!({"triggered": false, "reasons": []}));
        reset_wasm_public_observation_surface(self).map_err(|err| JsValue::from_str(&err))?;
        Ok(())
    }

    pub fn configure_project_assets(
        &mut self,
        project_source_json: &str,
    ) -> Result<String, JsValue> {
        let registry = local_project::project_asset_registry_from_json(project_source_json)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        let identities = registry
            .as_deref()
            .map(crate::gate0_registry::ProjectAssetRegistry::sealed_dataset_identities)
            .transpose()
            .map_err(|err| JsValue::from_str(&err))?
            .unwrap_or_default();
        self.project_asset_registry = registry;
        self.runner
            .set_project_asset_registry(self.project_asset_registry.clone());
        Ok(json!({
            "schema": "ddn.wasm.project_asset_registry.v1",
            "sealed_dataset_assets": identities,
        })
        .to_string())
    }

    pub fn get_debug_source_identity_json(&self) -> String {
        json!({
            "source_bytes_sha256": self.source_bytes_sha256,
            "canonical_ddn_sha256": self.canonical_ddn_sha256,
        })
        .to_string()
    }

    pub fn configure_debug_plan(&mut self, plan_json: &str) -> Result<JsValue, JsValue> {
        let plan: JsonValue = serde_json::from_str(plan_json)
            .map_err(|err| JsValue::from_str(&format!("E_DEBUG_PLAN_JSON_INVALID: {err}")))?;
        let evaluator = DebugEvaluator::from_plan(&plan, Some(&self.source_bytes_sha256))
            .map_err(|err| JsValue::from_str(&err))?;
        let plan_hash = evaluator.plan_hash().to_string();
        self.debug_evaluator = Some(evaluator);
        crate::unit_wire::replace_retained_json(&mut self.last_debug_evidence,
            json!({"triggered": false, "reasons": [], "debug_plan_hash": plan_hash}));
        Ok(JsValue::from_str(
            &json!({"ok": true, "debug_plan_hash": plan_hash}).to_string(),
        ))
    }

    pub fn clear_debug_plan(&mut self) -> JsValue {
        self.debug_evaluator = None;
        crate::unit_wire::replace_retained_json(&mut self.last_debug_evidence, json!({"triggered": false, "reasons": []}));
        JsValue::from_str(&json!({"ok": true}).to_string())
    }

    pub fn get_build_info(&self) -> String {
        format!(
            "pkg={} version={} state_schema={} resource_schema={} unit_wire={} update={}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            ENGINE_RESPONSE_SCHEMA,
            RESOURCE_SNAPSHOT_SCHEMA_V2,
            UNIT_VALUE_WIRE_SCHEMA_V2,
            DEFAULT_UPDATE_NAME
        )
    }

    pub fn get_parse_warnings(&self) -> JsValue {
        let warnings: Vec<JsonValue> = self
            .parse_warnings
            .iter()
            .map(|warning| {
                json!({
                    "code": warning.code,
                    "message": warning.message,
                    "span": {
                        "start": warning.span_start,
                        "end": warning.span_end,
                    }
                })
            })
            .collect();
        JsValue::from_str(&json!({ "warnings": warnings }).to_string())
    }

    pub fn set_rng_seed(&mut self, seed: u64) {
        self.rng_seed = seed;
    }

    pub fn add_view_prefix(&mut self, prefix: &str) {
        let trimmed = prefix.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.view_prefixes.iter().any(|p| p == trimmed) {
            return;
        }
        self.view_prefixes.push(trimmed.to_string());
    }

    pub fn clear_view_prefixes(&mut self) {
        self.view_prefixes.clear();
    }

    pub fn set_input_fixed64_v1(
        &mut self,
        keys_pressed: u32,
        last_key_name: &str,
        pointer_x_i32: i32,
        pointer_y_i32: i32,
        dt_high_i32: i32,
        dt_low_u32: u32,
    ) -> Result<JsValue, JsValue> {
        crate::input_receipt::validate_alpha_host_dt(fixed64_from_q31_32_parts(dt_high_i32, dt_low_u32))
            .map_err(|message| JsValue::from_str(&message))?;
        let dt = self.set_input_fixed64_parts(
            keys_pressed,
            last_key_name,
            pointer_x_i32,
            pointer_y_i32,
            dt_high_i32,
            dt_low_u32,
        );
        Ok(JsValue::from_str(&json!({
            "ok": true,
            "diag": [],
            "input_dt": fixed64_q31_32_parts(dt),
        }).to_string()))
    }

    /// Canonical textual admission for a Workbench frame duration.  JavaScript
    /// carries only this decimal text and the returned Q31.32 pair; the
    /// numeric conversion itself remains owned by Fixed64.
    pub fn fixed64_from_decimal_v1(&self, decimal: &str) -> Result<JsValue, JsValue> {
        let value = Fixed64::parse_decimal_exact(decimal).ok_or_else(|| {
            wasm_error(
                "E_FIXED64_UNREPRESENTABLE_CONVERSION",
                "Fixed64 decimal input is not exactly representable",
                "provide an exact Q31.32-compatible decimal value",
            )
        })?;
        Ok(JsValue::from_str(
            &json!({"ok": true, "diag": [], "value": fixed64_q31_32_parts(value)}).to_string(),
        ))
    }

    pub fn set_keys_pressed(&mut self, keys_pressed: u32) {
        self.input_keys_pressed = u64::from(keys_pressed);
    }

    pub fn set_last_key_name(&mut self, last_key_name: &str) {
        self.input_last_key_name = last_key_name.to_string();
    }

    pub fn set_pointer(&mut self, pointer_x_i32: i32, pointer_y_i32: i32) {
        self.input_pointer_x_i32 = pointer_x_i32;
        self.input_pointer_y_i32 = pointer_y_i32;
    }

    pub fn set_dt_fixed64_v1(
        &mut self,
        dt_high_i32: i32,
        dt_low_u32: u32,
    ) -> Result<JsValue, JsValue> {
        crate::input_receipt::validate_alpha_host_dt(fixed64_from_q31_32_parts(dt_high_i32, dt_low_u32))
            .map_err(|message| JsValue::from_str(&message))?;
        let dt = self.set_dt_fixed64_parts(dt_high_i32, dt_low_u32);
        Ok(JsValue::from_str(&json!({
            "ok": true,
            "diag": [],
            "input_dt": fixed64_q31_32_parts(dt),
        }).to_string()))
    }

    fn set_input_fixed64_parts(
        &mut self,
        keys_pressed: u32,
        last_key_name: &str,
        pointer_x_i32: i32,
        pointer_y_i32: i32,
        dt_high_i32: i32,
        dt_low_u32: u32,
    ) -> Fixed64 {
        let dt = fixed64_from_q31_32_parts(dt_high_i32, dt_low_u32);
        self.input_keys_pressed = u64::from(keys_pressed);
        self.input_last_key_name = last_key_name.to_string();
        self.input_pointer_x_i32 = pointer_x_i32;
        self.input_pointer_y_i32 = pointer_y_i32;
        self.input_dt = dt;
        dt
    }

    fn set_dt_fixed64_parts(&mut self, dt_high_i32: i32, dt_low_u32: u32) -> Fixed64 {
        let dt = fixed64_from_q31_32_parts(dt_high_i32, dt_low_u32);
        self.input_dt = dt;
        dt
    }

    pub fn columns(&self) -> JsValue {
        JsValue::from_str(&self.columns_json_text())
    }

    fn columns_json_text(&self) -> String {
        let mut payload = crate::unit_wire::SnapshotJsonOwner::from_value(json!({
            "observation_columns": null,
            "output_rows": null,
            "public_observation_trace": null,
            "public_observation_trace_hash": self.public_observation_trace_hash,
            "public_observation_table_projection": null,
            "deprecated": true,
            "use_instead": "step_one() 또는 get_state_json()의 public_observation_trace",
        }));
        attach_wasm_observation_response(&mut payload, self);
        payload.compact_text()
    }

    pub fn set_param(&mut self, key: &str, value: JsValue) -> Result<JsValue, JsValue> {
        let key = key.trim();
        if key.is_empty() {
            return Err(wasm_error(
                "PARAM_KEY_EMPTY",
                "set_param: key가 비어 있습니다",
                "변수 이름을 입력해 주세요. 예: set_param(\"속도\", 10)",
            ));
        }
        let parsed = js_scalar_to_runtime_value(&value).ok_or_else(|| {
            wasm_error(
                "PARAM_VALUE_INVALID",
                "set_param: Fixed64 수는 raw setter만 허용하며 참거짓/글만 직접 허용됩니다",
                "Fixed64는 set_param_fixed64_str 또는 versioned raw carrier를 사용해 주세요.",
            )
        })?;
        apply_param_value(self.world.world_mut(), key, &parsed).map_err(|err| {
            wasm_error(
                "PARAM_APPLY_FAILED",
                &err,
                "대상 변수 이름과 타입을 확인해 주세요.",
            )
        })?;
        self.param_overrides.insert(key.to_string(), parsed);

        let payload = json!({
            "ok": true,
            "state_hash": current_state_hash(&self.world, &self.view_prefixes),
            "diag": [],
        });
        Ok(JsValue::from_str(&payload.to_string()))
    }

    pub fn set_param_fixed64(&mut self, key: &str, raw_i64: i64) -> Result<JsValue, JsValue> {
        let key = key.trim();
        if key.is_empty() {
            return Err(wasm_error(
                "PARAM_KEY_EMPTY",
                "set_param_fixed64: key가 비어 있습니다",
                "변수 이름을 입력해 주세요. 예: set_param_fixed64(\"속도\", 4294967296)",
            ));
        }
        let parsed = Value::Fixed64(Fixed64::from_raw_i64(raw_i64));
        apply_param_value(self.world.world_mut(), key, &parsed).map_err(|err| {
            wasm_error(
                "PARAM_APPLY_FAILED",
                &err,
                "대상 변수 이름과 타입을 확인해 주세요.",
            )
        })?;
        self.param_overrides.insert(key.to_string(), parsed);
        let payload = json!({
            "ok": true,
            "state_hash": current_state_hash(&self.world, &self.view_prefixes),
            "diag": [],
            "raw_i64": raw_i64,
        });
        Ok(JsValue::from_str(&payload.to_string()))
    }

    pub fn set_param_fixed64_str(&mut self, key: &str, raw_i64: &str) -> Result<JsValue, JsValue> {
        let key = key.trim();
        if key.is_empty() {
            return Err(wasm_error(
                "PARAM_KEY_EMPTY",
                "set_param_fixed64_str: key가 비어 있습니다",
                "변수 이름을 입력해 주세요. 예: set_param_fixed64_str(\"속도\", \"4294967296\")",
            ));
        }
        let trimmed = raw_i64.trim();
        if trimmed.is_empty() {
            return Err(wasm_error(
                "PARAM_RAW_I64_EMPTY",
                "set_param_fixed64_str: raw_i64가 비어 있습니다",
                "raw_i64 정수 문자열을 입력해 주세요. 예: \"4294967296\"",
            ));
        }
        let parsed = trimmed.parse::<i64>().map_err(|_| {
            wasm_error(
                "PARAM_RAW_I64_INVALID",
                "set_param_fixed64_str: raw_i64는 i64 정수 문자열이어야 합니다",
                "정수 문자열만 허용됩니다. 예: -123, 4294967296",
            )
        })?;
        let value = Value::Fixed64(Fixed64::from_raw_i64(parsed));
        apply_param_value(self.world.world_mut(), key, &value).map_err(|err| {
            wasm_error(
                "PARAM_APPLY_FAILED",
                &err,
                "대상 변수 이름과 타입을 확인해 주세요.",
            )
        })?;
        self.param_overrides.insert(key.to_string(), value);
        let payload = json!({
            "ok": true,
            "state_hash": current_state_hash(&self.world, &self.view_prefixes),
            "diag": [],
            "raw_i64": trimmed,
        });
        Ok(JsValue::from_str(&payload.to_string()))
    }

    pub fn reset(&mut self, keep_params: Option<bool>) -> Result<JsValue, JsValue> {
        if let Some(binding)=self.native_source_binding.as_mut(){
            if binding.initial_reset_pending{binding.initial_reset_pending=false;return Ok(self.get_state_json());}
            return Err(JsValue::from_str("E_NATIVE_SOURCE_RESTART_REQUIRES_REOPEN: 저장된 Native 작품을 다시 열어 실행하세요"));
        }
        let keep_params = keep_params.unwrap_or(false);
        self.world = DetNuri::new();
        self.tick_id = 0;
        self.rng_seed = 0;
        self.input_keys_pressed = 0;
        self.input_last_key_name.clear();
        self.input_pointer_x_i32 = 0;
        self.input_pointer_y_i32 = 0;
        self.input_dt = crate::input_receipt::alpha_host_dt();
        self.last_patch = None;
        self.pending_ai_injections.clear();
        self.runner.reset_transient_state();
        if let Some(evaluator) = self.debug_evaluator.as_mut() {
            evaluator.reset_history();
        }
        crate::unit_wire::replace_retained_json(&mut self.last_debug_evidence, json!({"triggered": false, "reasons": []}));
        reset_wasm_public_observation_surface(self).map_err(|err| JsValue::from_str(&err))?;

        if !keep_params {
            self.param_overrides.clear();
        } else {
            for (key, value) in &self.param_overrides {
                apply_param_value(self.world.world_mut(), key, value)
                    .map_err(|err| JsValue::from_str(&err))?;
            }
        }

        let (columns, row) = collect_columns_and_row(self.world.world());
        let mut payload = crate::unit_wire::SnapshotJsonOwner::from_value(json!({
            "ok": true,
            "tick": self.tick_id,
            "columns": columns,
            "row": row,
            "input": serialize_input_state(self),
            "observation_columns": null,
            "output_rows": null,
            "public_observation_trace": null,
            "public_observation_trace_hash": self.public_observation_trace_hash,
            "public_observation_table_projection": null,
            "state_hash": current_state_hash(&self.world, &self.view_prefixes),
            "diag": [],
        }));
        attach_wasm_observation_response(&mut payload, self);
        Ok(JsValue::from_str(&payload.compact_text()))
    }

    pub fn step_one_with_input_fixed64_v1(
        &mut self,
        keys_pressed: u32,
        last_key_name: &str,
        pointer_x_i32: i32,
        pointer_y_i32: i32,
        dt_high_i32: i32,
        dt_low_u32: u32,
    ) -> Result<JsValue, JsValue> {
        self.set_input_fixed64_v1(
            keys_pressed,
            last_key_name,
            pointer_x_i32,
            pointer_y_i32,
            dt_high_i32,
            dt_low_u32,
        )?;
        self.step_one()
    }

    pub fn step_one(&mut self) -> Result<JsValue, JsValue> {
        let native_catalog=if self.native_source_binding.is_some(){self.saved_project_owner.as_ref()
            .map(|owner|owner.registration().map(|registration|registration.saved_nominal_catalog(owner)).transpose())
            .transpose().map_err(|e|JsValue::from_str(&e))?.flatten()}else{None};
        let native_expected=if let Some(binding)=self.native_source_binding.as_mut(){
            if self.input_keys_pressed!=0||!self.input_last_key_name.is_empty()||self.input_pointer_x_i32!=0||self.input_pointer_y_i32!=0
                ||!self.param_overrides.is_empty()||!self.pending_ai_injections.is_empty()||self.input_dt!=crate::input_receipt::alpha_host_dt(){
                return Err(JsValue::from_str("E_NATIVE_SOURCE_INPUT_UNSUPPORTED: 이 Native 원문 실행은 기본 입력만 지원합니다"));
            }
            if binding.first_frame_pending{
                let receipt=binding.first_input_receipt.take().ok_or_else(||JsValue::from_str("E_NATIVE_SOURCE_INPUT_RECEIPT"))?;
                binding.first_frame_pending=false;
                let mut frame=self.current_state_json_owner();
                frame.value_mut()["tick_id"]=receipt["input_snapshot"]["madi"].clone();
                frame.value_mut()["input_receipt"]=receipt.into_parent_value();
                return Ok(JsValue::from_str(&frame.compact_text()));
            }
            let next=self.tick_id.checked_add(1)
                .ok_or_else(||JsValue::from_str("E_NATIVE_SOURCE_MADI_LIMIT"))?;
            Some(native_source_exchange(&json!({"operation":"run_to","authored_project_json_utf8":binding.authored,"next_tick":next.to_string()}),native_catalog.as_ref())?)
        }else{None};
        let native_runner_checkpoint=native_expected.as_ref().map(|_|(self.runner.bogae_history_for_resume().clone(),self.runner.consumed_keys_pressed()));
        let tick_seed = self.rng_seed ^ self.tick_id;
        let ai_injections = self
            .pending_ai_injections
            .drain(..)
            .enumerate()
            .map(|(idx, (key, value_json))| SeulgiPacket {
                agent_id: 0,
                recv_seq: idx as u64,
                accepted_madi: self.tick_id,
                target_madi: self.tick_id,
                source: InputSource::Seulgi,
                intent: SeulgiIntent::Say {
                    text: format!("{key}={value_json}"),
                },
            })
            .collect();
        let input = InputSnapshot {
            tick_id: self.tick_id,
            dt: self.input_dt,
            keys_pressed: self.input_keys_pressed,
            last_key_name: self.input_last_key_name.clone(),
            pointer_x_i32: self.input_pointer_x_i32,
            pointer_y_i32: self.input_pointer_y_i32,
            ai_injections,
            net_events: Vec::new(),
            frame_source: InputSource::Person,
            rng_seed: tick_seed,
        };

        let output = self
            .runner
            .run_update_diagnostic_with_control_edges(
                self.world.world(),
                &input,
                &self.defaults,
                None,
                Some(&self.param_overrides),
            )
            .map_err(|error| {
                let diagnostic = runtime_surface::execution_error_diagnostic(error, &self.source_path);
                JsValue::from_str(&diagnostic.to_json_value().to_string())
            })?;
        let public_observations = crate::unit_wire::SnapshotJsonOwner::from_value(JsonValue::Array(
            public_observation_events_json(&output.public_observations)
                .map_err(|err| JsValue::from_str(&err))?));

        let mut sink = ddonirang_core::signals::VecSignalSink::default();
        if let Some(expected)=native_expected.as_ref(){
            let mut candidate=DetNuri::new();*candidate.world_mut()=self.world.world().clone();candidate.apply_patch(&output.patch,input.tick_id,&mut sink);
            if expected["summary"]["state_hash"]!=current_state_hash(&candidate,&self.view_prefixes)
                ||expected["summary"]["next_tick_id"]!=self.tick_id+1 {
                if let Some((history,keys))=native_runner_checkpoint{self.runner.restore_scene_history(history);self.runner.restore_consumed_keys_pressed(keys);}
                return Err(JsValue::from_str("E_NATIVE_SOURCE_PARITY: Native·WASM 결과가 달라 게시하지 않았습니다"));
            }
            self.world=candidate;
        }else{self.world.apply_patch(&output.patch, input.tick_id, &mut sink);}

        self.last_patch = Some(output.patch);
        crate::unit_wire::clear_retained_json_option(&mut self.last_bogae_projection);
        self.last_bogae_projection = output.bogae_projection;
        self.display_outputs.extend(output.display_outputs);
        let state_hash = current_state_hash(&self.world, &self.view_prefixes);
        let resources = crate::unit_wire::SnapshotJsonOwner::from_value(serialize_world_resources(self.world.world()));
        crate::output_display::retain_current(&mut self.display_outputs, &resources);
        let (columns, row) = collect_columns_and_row(self.world.world());
        let streams = collect_streams(self.world.world());
        let view_meta = build_view_meta(self.world.world());
        let view_hash = hash_json_string(&view_meta);
        let primary_view_decision = crate::primary_view::decide(
            &view_meta,
            &self.source_bytes_sha256,
            &self.canonical_ddn_sha256,
            &state_hash,
            &view_hash,
        );
        let snapshot_resources = crate::unit_wire::SnapshotJsonOwner::from_value(serialize_world_resources_snapshot(self.world.world()));
        let snapshot_params = serialize_param_overrides(&self.param_overrides);
        // 한 프레임의 두 공개 경로는 같은 Patch를 담는다. 정본 변환은 한 번만
        // 수행하고 JSON 값만 복제하여 전체 연산 순서와 payload 모양을 보존한다.
        let patch_json = crate::unit_wire::SnapshotJsonOwner::from_value(self
            .last_patch
            .as_ref()
            .map(serialize_patch)
            .unwrap_or_else(|| JsonValue::Array(Vec::new())));

        let input_json = serialize_input_snapshot(&input, Some(self.rng_seed));
        let mut frame = crate::unit_wire::SnapshotJsonOwner::from_value(json!({
            "committed_madi": input.tick_id,
            "state_hash": state_hash,
            "input_snapshot": input_json,
            "public_observations": null,
            "input_snapshot_sha256": crate::input_receipt::snapshot_sha256(&input),
            "input_reads": output.input_reads,
            "tape_v1_recording_error": crate::input_receipt::validate_tape_v1_reads(&output.input_reads).err(),
        }));
        frame.value_mut()["public_observations"] = public_observations.clone().into_parent_value();
        self.public_observation_frames.push(frame.into_parent_value());
        if let Err(err) = refresh_wasm_public_observation_surface(self) {
            if let Some(frame) = self.public_observation_frames.pop() {
                drop(crate::unit_wire::SnapshotJsonOwner::from_value(frame));
            }
            return Err(JsValue::from_str(&err));
        }
        if let Some(projection)=self.last_bogae_projection.as_mut() {
            crate::bogae_time::resolve_result_tables(projection,&self.public_observation_frames,
                &format!("sha256:{}",self.source_bytes_sha256),&format!("sha256:{}",self.canonical_ddn_sha256))
                .map_err(|err|JsValue::from_str(&err))?;
        }
        self.tick_id = self.tick_id.saturating_add(1);
        crate::unit_wire::clear_retained_json_rows(&mut self.last_public_observations);
        self.last_public_observations = copy_wasm_json_rows(public_observations.as_array().expect("observation rows"));

        let debug_evidence = if let Some(evaluator) = self.debug_evaluator.as_mut() {
            let mut frame = crate::unit_wire::SnapshotJsonOwner::from_value(json!({
                "committed_madi": input.tick_id,
                "state_hash": state_hash,
                "input_snapshot": input_json,
                "public_observations": null,
            }));
            frame.value_mut()["public_observations"] = public_observations.clone().into_parent_value();
            evaluator
                .evaluate_frame(&frame)
                .map_err(|err| JsValue::from_str(&err))?
        } else {
            json!({
                "schema": crate::deterministic_debug::DEBUG_EVIDENCE_SCHEMA,
                "triggered": false,
                "committed_madi": input.tick_id,
                "state_hash": state_hash,
                "debug_plan_hash": "",
                "reasons": [],
            })
        };
        crate::unit_wire::replace_retained_json(&mut self.last_debug_evidence, debug_evidence);
        let snapshot = crate::unit_wire::SnapshotJsonOwner::from_value(build_wasm_state_snapshot_v2(
            snapshot_resources.into_parent_value(), snapshot_params, copy_wasm_json_rows(&self.public_observation_frames),
            self.tick_id, self.runner.scene_history_snapshot()));
        let payload = json!({
            "schema": ENGINE_RESPONSE_SCHEMA,
            "tick_id": input.tick_id,
            "committed_madi": input.tick_id,
            "input_receipt": crate::input_receipt::receipt(&input, &output.input_reads),
            "state_hash": state_hash,
            "input": input_json.clone(),
            "resources": null,
            "channels": columns.clone(),
            "row": row.clone(),
            "observation_columns": null,
            "output_rows": null,
            "public_observation_trace": null,
            "public_observation_trace_hash": self.public_observation_trace_hash.clone(),
            "public_observation_table_projection": null,
            "public_observations": null,
            "debug_pause": null,
            "streams": streams.clone(),
            "patch": null,
            "state": {
                "tick_id": input.tick_id,
                "committed_madi": input.tick_id,
                "input": input_json,
                "resources": null,
                "channels": columns,
                "row": row,
                "public_observations": null,
                "debug_pause": null,
                "patch": null,
                "streams": streams,
                "snapshot_v2": null
            },
            "view_meta": view_meta,
            "view_hash": view_hash,
            "primary_view_decision": primary_view_decision,
        });
        let mut payload = crate::unit_wire::SnapshotJsonOwner::from_value(payload);
        payload.value_mut()["resources"] = resources.clone().into_parent_value();
        payload.value_mut()["state"]["resources"] = resources.into_parent_value();
        payload.value_mut()["state"]["snapshot_v2"] = snapshot.into_parent_value();
        payload.value_mut()["patch"] = patch_json.clone().into_parent_value();
        payload.value_mut()["state"]["patch"] = patch_json.into_parent_value();
        attach_wasm_observation_response(&mut payload, self);
        crate::bogae_projection::attach(payload.value_mut(), self.last_bogae_projection.as_ref());
        crate::output_display::attach(payload.value_mut(), &self.display_outputs);
        // A temporary registration is a fresh invocation identity. A second
        // independent run-summary cannot certify this VM's committed world.
        // Bind the actual first frame, input and work meter inside Rust; never
        // replace its hash with a replay's hash or promote it to saved custody.
        if self.runner.has_temporary_registration() && self.tick_id == 1 {
            payload.value_mut()["temporary_committed_summary"] = json!({
                "schema": "ddn.ddonirang_tool.shared_run_summary.v1",
                "binding_scope": "temporary_same_vm_first_committed_frame",
                "source_profile_identity": crate::ddn_runtime::V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
                "execution_envelope": {"source_bytes_sha256": self.source_bytes_sha256},
                "state_hash": state_hash,
                "input_receipts": [crate::input_receipt::receipt(&input, &output.input_reads)],
                "input_record_sha256": crate::input_receipt::record_sha256(std::slice::from_ref(&input)),
                "work_cost_receipt": self.runner.work_cost_receipt(),
                "output_log_texts": self.display_outputs.iter().map(|row|row.text.clone()).collect::<Vec<_>>(),
            });
        }
        Ok(JsValue::from_str(&payload.compact_text()))
    }

    pub fn run_ticks(&mut self, count: u32) -> Result<JsValue, JsValue> {
        let count = count.max(1);
        let mut state = JsValue::NULL;
        for _ in 0..count {
            state = self.step_one()?;
            if self
                .last_debug_evidence
                .get("triggered")
                .and_then(JsonValue::as_bool)
                == Some(true)
            {
                break;
            }
        }

        if self
            .last_debug_evidence
            .get("triggered")
            .and_then(JsonValue::as_bool)
            == Some(true)
        {
            return Ok(state);
        }

        let input = InputSnapshot {
            tick_id: self.tick_id,
            dt: self.input_dt,
            keys_pressed: self.input_keys_pressed,
            last_key_name: self.input_last_key_name.clone(),
            pointer_x_i32: self.input_pointer_x_i32,
            pointer_y_i32: self.input_pointer_y_i32,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            frame_source: InputSource::Person,
            rng_seed: self.rng_seed ^ self.tick_id,
        };
        let output = self
            .runner
            .run_finish_with_param_overrides(
                self.world.world(),
                &input,
                &self.defaults,
                &self.param_overrides,
            )
            .map_err(|err| JsValue::from_str(&err))?;
        if !output.patch.ops.is_empty() {
            let mut sink = ddonirang_core::signals::VecSignalSink::default();
            self.world
                .apply_patch(&output.patch, input.tick_id, &mut sink);
            self.last_patch = Some(output.patch);
            self.display_outputs.extend(output.display_outputs);
            crate::output_display::retain_current(&mut self.display_outputs,
                &serialize_world_resources(self.world.world()));
            state = self.get_state_json();
        }
        Ok(state)
    }

    pub fn apply_currentline_cell(
        &mut self,
        source: &str,
        context_json: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let result = ddonirang_lang::apply_currentline_cell(source, context_json.as_deref())
            .map_err(|err| JsValue::from_str(&err))?;
        apply_currentline_result_to_vm(self, result, None)
    }

    /// 명시적 supported source profile을 소비하는 currentline 내부 candidate entry다.
    /// 기존 apply_currentline_cell의 legacy 기본 동작은 바꾸지 않는다.
    pub fn apply_currentline_cell_with_source_profile(
        &mut self,
        source: &str,
        source_profile_identity: &str,
        context_json: Option<String>,
    ) -> Result<JsValue, JsValue> {
        resolve_supported_source_profile(Some(source_profile_identity))
            .map_err(|error| JsValue::from_str(&format!("{}: {}", error.code, error.message)))?;
        let result = ddonirang_lang::apply_currentline_cell_v25(source, context_json.as_deref())
            .map_err(|err| JsValue::from_str(&err))?;
        apply_currentline_result_to_vm(self, result, Some(source_profile_identity))
    }

    pub fn get_state_hash(&self) -> String {
        current_state_hash(&self.world, &self.view_prefixes)
    }

    pub fn get_state_json(&self) -> JsValue {
        JsValue::from_str(&self.current_state_json_text())
    }

    // Keep the ordinary response bytes testable on Native without constructing
    // a wasm-bindgen JS value. This is the same product snapshot path.
    fn current_state_json_text(&self) -> String {
        crate::unit_wire::serialize_resource_snapshot_json(&self.current_state_json_owner())
    }
    fn current_state_json_owner(&self)->crate::unit_wire::SnapshotJsonOwner{
        let resources = crate::unit_wire::SnapshotJsonOwner::from_value(serialize_world_resources(self.world.world()));
        let (columns, row) = collect_columns_and_row(self.world.world());
        let streams = collect_streams(self.world.world());
        let view_meta = build_view_meta(self.world.world());
        let view_hash = hash_json_string(&view_meta);
        let state_hash = current_state_hash(&self.world, &self.view_prefixes);
        let primary_view_decision = crate::primary_view::decide(
            &view_meta,
            &self.source_bytes_sha256,
            &self.canonical_ddn_sha256,
            &state_hash,
            &view_hash,
        );
        let snapshot_resources = crate::unit_wire::SnapshotJsonOwner::from_value(serialize_world_resources_snapshot(self.world.world()));
        let snapshot_params = serialize_param_overrides(&self.param_overrides);
        let committed_madi = last_committed_madi(&self.public_observation_frames);
        let snapshot = crate::unit_wire::SnapshotJsonOwner::from_value(build_wasm_state_snapshot_v2(
            snapshot_resources.into_parent_value(), snapshot_params,
            self.public_observation_frames.iter().map(|frame|
                crate::unit_wire::SnapshotJsonOwner::clone_from_value(frame).into_parent_value()).collect(),
            self.tick_id, self.runner.scene_history_snapshot()));
        let payload = json!({
            "schema": ENGINE_RESPONSE_SCHEMA,
            "tick_id": self.tick_id,
            "committed_madi": committed_madi.clone(),
            "state_hash": state_hash,
            "input": serialize_input_state(self),
            "resources": null,
            "channels": columns.clone(),
            "row": row.clone(),
            "observation_columns": null,
            "output_rows": null,
            "public_observation_trace": null,
            "public_observation_trace_hash": self.public_observation_trace_hash.clone(),
            "public_observation_table_projection": null,
            "public_observations": null,
            "debug_pause": null,
            "streams": streams.clone(),
            "state": {
                "tick_id": self.tick_id,
                "committed_madi": committed_madi,
                "input": serialize_input_state(self),
                "resources": null,
                "channels": columns,
                "row": row,
                "public_observations": null,
                "debug_pause": null,
                "patch": JsonValue::Array(Vec::new()),
                "streams": streams,
                "snapshot_v2": null
            },
            "view_meta": view_meta,
            "view_hash": view_hash,
            "primary_view_decision": primary_view_decision,
        });
        let mut payload = crate::unit_wire::SnapshotJsonOwner::from_value(payload);
        payload.value_mut()["resources"] = resources.clone().into_parent_value();
        payload.value_mut()["state"]["resources"] = resources.into_parent_value();
        payload.value_mut()["state"]["snapshot_v2"] = snapshot.into_parent_value();
        attach_wasm_observation_response(&mut payload, self);
        crate::bogae_projection::attach(payload.value_mut(), self.last_bogae_projection.as_ref());
        crate::output_display::attach(payload.value_mut(), &self.display_outputs);
        // A3-P11: outside the closed resource/snapshot wire; full u64 text.
        payload.value_mut()["runner_input_history"] = json!({
            "schema": "ddn.runner_input_history.v1",
            "consumed_keys_decimal": self.runner.consumed_keys_pressed().to_string(),
        });
        payload
    }

    pub fn get_input_state_json(&self) -> JsValue {
        JsValue::from_str(&serialize_input_state(self).to_string())
    }

    pub fn restore_state(&mut self, state_json: &str) -> Result<JsValue, JsValue> {
        if self.native_source_binding.is_some(){return Err(JsValue::from_str("E_NATIVE_SOURCE_RESTORE_REQUIRES_REOPEN"));}
        self.restore_state_with_complete_frame(state_json,None)
    }
    fn restore_state_with_complete_frame(&mut self,state_json:&str,frame:Option<crate::unit_wire::CompleteStateFrameV1<'_>>)->Result<JsValue,JsValue>{
        let catalog=self.saved_project_owner.as_ref()
            .map(|owned|owned.registration().map(|registration|registration.saved_nominal_catalog(owned)).transpose())
            .transpose().map_err(|err|JsValue::from_str(&err))?.flatten();
        let payload = crate::unit_wire::parse_snapshot_json_with_catalog(state_json,catalog.as_ref())
            .map_err(|err| JsValue::from_str(&format!("restore_state: JSON 파싱 실패: {err}")))?;
        // Validate before publishing any staged world. Source-bound projects
        // cannot resume an older private artifact that omitted this history.
        let consumed_keys = match payload.get("runner_input_history") {
            Some(value) => {
                let invalid = || JsValue::from_str("E_PROJECT_RUNTIME_INPUT_HISTORY: 처리한 입력 이력이 올바르지 않습니다.");
                let object = value.as_object().ok_or_else(invalid)?;
                if object.len() != 2 || value["schema"] != "ddn.runner_input_history.v1" {
                    return Err(invalid());
                }
                let text = value["consumed_keys_decimal"].as_str().ok_or_else(invalid)?;
                let keys = text.parse::<u64>().map_err(|_| invalid())?;
                if keys.to_string() != text { return Err(invalid()); }
                keys
            }
            None if self.saved_project_owner.is_some() => return Err(JsValue::from_str(
                "E_PROJECT_RUNTIME_INPUT_HISTORY: 저장 작품에 처리한 입력 이력이 없습니다.")),
            None => 0,
        };
        let schema = payload
            .get("schema")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if schema != ENGINE_RESPONSE_SCHEMA {
            return Err(JsValue::from_str(&format!(
                "restore_state: 지원하지 않는 schema: {schema}"
            )));
        }

        let tuple_plan = self.runner.saved_tuple_restore_plan();
        let staged = crate::unit_wire::runtime_checkpoint::stage_runtime_state_with_complete_frame(
            &payload,
            &self.source_path,
            &self.source_bytes_sha256,
            &self.canonical_ddn_sha256,
            tuple_plan.as_ref(),
            catalog.as_ref(),
            frame,
        )
        .map_err(|err| JsValue::from_str(&format!("restore_state: {err}")))?;
        self.runner.validate_persisted_records(staged.world.world())
            .map_err(|err| JsValue::from_str(&format!("restore_state: {err}")))?;
        crate::input_receipt::validate_alpha_host_dt(staged.input.dt)
            .map_err(|message| JsValue::from_str(&message))?;
        let state_hash = current_state_hash(&staged.world, &self.view_prefixes);
        let view_hash = hash_json_string(&build_view_meta(staged.world.world()));
        let diag = restore_hash_diags(&payload, &state_hash, &view_hash);
        let display_outputs = crate::output_display::restore(&payload)
            .map_err(|err| JsValue::from_str(&err))?;
        if payload.get("output_display").is_some()
            && payload["output_display"]["world_state_hash"] != state_hash {
            return Err(JsValue::from_str("E_OUTPUT_DISPLAY_SNAPSHOT: 표시 정보가 복원한 세계와 다릅니다."));
        }

        self.world = staged.world;
        self.param_overrides = staged.param_overrides;
        self.tick_id = staged.tick_id;
        self.rng_seed = staged.input.rng_seed;
        self.input_keys_pressed = staged.input.keys_pressed;
        self.input_last_key_name = staged.input.last_key_name;
        self.input_pointer_x_i32 = staged.input.pointer_x_i32;
        self.input_pointer_y_i32 = staged.input.pointer_y_i32;
        self.input_dt = staged.input.dt;
        self.last_patch = None;
        self.pending_ai_injections.clear();
        self.runner.reset_transient_state();
        self.debug_evaluator = None;
        crate::unit_wire::replace_retained_json(&mut self.last_debug_evidence, json!({"triggered": false, "reasons": []}));
        self.clear_retained_observation_json();
        self.last_public_observations = match staged.last_public_observations.into_parent_value() {
            JsonValue::Array(values)=>values,_=>unreachable!("staged observations are an array"),
        };
        self.runner.restore_consumed_keys_pressed(consumed_keys);
        self.runner.restore_scene_history(staged.scene_history);
        self.last_bogae_projection=staged.bogae_projection.map(crate::unit_wire::SnapshotJsonOwner::into_parent_value);
        // Restore the explicitly stored non-world projection, never guess from text.
        self.display_outputs = display_outputs;
        self.public_observation_frames = match staged.public_observation_frames.into_parent_value() {
            JsonValue::Array(values)=>values,_=>unreachable!("staged frames are an array"),
        };
        self.public_observation_trace = staged.observation_surface.trace.into_parent_value();
        self.public_observation_trace_hash = staged.observation_surface.trace_hash;
        self.public_observation_table_projection = staged.observation_surface.table_projection.into_parent_value();

        let (columns, row) = collect_columns_and_row(self.world.world());
        let mut out = crate::unit_wire::SnapshotJsonOwner::from_value(json!({
            "ok": true,
            "tick": self.tick_id,
            "committed_madi": last_committed_madi(&self.public_observation_frames),
            "columns": columns,
            "row": row,
            "observation_columns": null,
            "output_rows": null,
            "public_observation_trace": null,
            "public_observation_trace_hash": self.public_observation_trace_hash,
            "public_observation_table_projection": null,
            "state_hash": state_hash,
            "view_hash": view_hash,
            "diag": diag,
        }));
        attach_wasm_observation_response(&mut out, self);
        Ok(JsValue::from_str(&out.compact_text()))
    }

    pub fn inject_ai_action(&mut self, key: &str, value_json: &str) {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return;
        }
        self.pending_ai_injections
            .push((trimmed.to_string(), value_json.to_string()));
    }

    pub fn clear_ai_injections(&mut self) {
        self.pending_ai_injections.clear();
    }
}

fn current_state_hash(world: &DetNuri, view_prefixes: &[String]) -> String {
    let refs: Vec<&str> = view_prefixes.iter().map(String::as_str).collect();
    format!(
        "blake3:{}",
        world
            .world()
            .state_hash_excluding_resource_prefixes(&refs)
            .to_hex()
    )
}

fn hash_json_string(value: &JsonValue) -> String {
    let canonical = value.to_string();
    let digest = blake3_hash(canonical.as_bytes());
    format!("blake3:{}", hex::encode(digest.as_bytes()))
}

fn restore_hash_diags(
    payload: &JsonValue,
    actual_state_hash: &str,
    actual_view_hash: &str,
) -> Vec<JsonValue> {
    let mut diags = Vec::new();

    if let Some(expected_state_hash) = payload.get("state_hash").and_then(JsonValue::as_str) {
        if expected_state_hash != actual_state_hash {
            diags.push(json!({
                "code": "STATE_HASH_MISMATCH",
                "expected": expected_state_hash,
                "actual": actual_state_hash,
                "detail": "복원 후 state_hash 불일치",
            }));
        }
    }

    if let Some(expected_view_hash) = payload.get("view_hash").and_then(JsonValue::as_str) {
        if expected_view_hash != actual_view_hash {
            diags.push(json!({
                "code": "VIEW_HASH_MISMATCH",
                "expected": expected_view_hash,
                "actual": actual_view_hash,
                "detail": "복원 후 view_hash 불일치 (view_meta 변경 가능)",
            }));
        }
    }

    diags
}

fn wasm_error(code: &str, detail: &str, hint: &str) -> JsValue {
    let payload = json!({
        "error": true,
        "code": code,
        "detail": detail,
        "hint": hint,
    });
    JsValue::from_str(&payload.to_string())
}

fn fixed64_from_q31_32_parts(high_i32: i32, low_u32: u32) -> Fixed64 {
    let raw = (i64::from(high_i32) << 32) | i64::from(low_u32);
    Fixed64::from_raw_i64(raw)
}

fn fixed64_q31_32_parts(value: Fixed64) -> JsonValue {
    let raw = value.raw_i64();
    json!({
        "high_i32": (raw >> 32) as i32,
        "low_u32": raw as u32,
    })
}

fn build_view_meta(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();

    if let Some(width) = world.get_resource_fixed64(BOGAE_WIDTH_TAG) {
        out.insert("canvas_width".to_string(), json!(fixed64_to_f64(width)));
    }
    if let Some(height) = world.get_resource_fixed64(BOGAE_HEIGHT_TAG) {
        out.insert("canvas_height".to_string(), json!(fixed64_to_f64(height)));
    }

    if let Some(draw_list_value) = world.get_resource_value(BOGAE_DRAWLIST_TAG) {
        if let ResourceValue::List(items) = &draw_list_value {
            let draw_items: Vec<JsonValue> = items
                .iter()
                .filter_map(resource_value_to_draw_item)
                .collect();
            out.insert("draw_list".to_string(), JsonValue::Array(draw_items));
            out.insert(
                "draw_list_meta".to_string(),
                json!({
                    "deprecated": true,
                    "use_instead": "view_meta.space2d",
                }),
            );
        }
    }

    let mut available_families = Vec::new();

    if let Some(grid2d_json) = derive_grid2d_from_bogae(world) {
        if let Ok(grid2d) = serde_json::from_str::<JsonValue>(&grid2d_json) {
            out.insert("grid2d".to_string(), grid2d);
            available_families.push("grid2d");
        }
    }
    // 격자 전용 투영이 일반 그림판 출력을 가리지 않게 한다.
    // 두 투영 모두 같은 게시된 drawlist에서만 만들어진다.
    if let Some(space2d_json) = derive_space2d_from_bogae(world) {
        if let Ok(space2d) = serde_json::from_str::<JsonValue>(&space2d_json) {
            out.insert("space2d".to_string(), space2d);
            available_families.push("space2d");
        }
    }
    if let Some(graph_json) = runtime_surface::derive_graph_from_points(world) {
        if let Ok(graph) = serde_json::from_str::<JsonValue>(&graph_json) {
            out.insert("graph".to_string(), graph);
            available_families.push("graph");
        }
    }
    if let Some(workbench) = build_workbench_authoritative_results(world, out.get("space2d")) {
        out.insert("workbench".to_string(), workbench);
    }

    let mut graph_hints = Vec::new();
    for (tag, id, label) in GRAPH_POINTS_TAGS.iter() {
        let Some(value) = world.get_resource_value(tag) else {
            continue;
        };
        let Some(points) = points_from_resource_value(&value) else {
            continue;
        };
        if points.is_empty() {
            continue;
        }
        graph_hints.push(json!({
            "series_id": *id,
            "source": *tag,
            "y_label": *label,
            "overlay": true,
        }));
    }
    out.insert("graph_hints".to_string(), JsonValue::Array(graph_hints));
    append_view_stack_meta(&mut out, &available_families);

    JsonValue::Object(out)
}

fn world_number(world: &ddonirang_core::platform::NuriWorld, tag: &str) -> Option<f64> {
    world
        .get_resource_fixed64(tag)
        .map(fixed64_to_f64)
        .or_else(|| {
            world
                .get_resource_value(tag)
                .and_then(|value| resource_value_to_number(&value))
        })
        .filter(|value| value.is_finite())
}

fn optional_world_number_json(world: &ddonirang_core::platform::NuriWorld, tag: &str) -> JsonValue {
    world_number(world, tag).map_or(JsonValue::Null, |value| json!(value))
}

fn workbench_result(schema: &str, family: &str, content: JsonValue) -> JsonValue {
    json!({
        "schema": schema,
        "family": family,
        "status": "available",
        "producer": "ddonirang_tool.wasm_runtime",
        "provenance": "native_authoritative_structured_result",
        "content": content,
    })
}

fn space2d_dimensions(space2d: &JsonValue) -> Option<JsonValue> {
    let camera = space2d.get("camera")?.as_object()?;
    let x_min = camera.get("x_min")?.as_f64()?;
    let x_max = camera.get("x_max")?.as_f64()?;
    let y_min = camera.get("y_min")?.as_f64()?;
    let y_max = camera.get("y_max")?.as_f64()?;
    let columns = (x_max - x_min).abs();
    let rows = (y_max - y_min).abs();
    if !columns.is_finite() || !rows.is_finite() || columns <= 0.0 || rows <= 0.0 {
        return None;
    }
    Some(json!({ "columns": columns, "rows": rows }))
}

fn build_workbench_authoritative_results(
    world: &ddonirang_core::platform::NuriWorld,
    space2d: Option<&JsonValue>,
) -> Option<JsonValue> {
    let mut results = Map::new();

    if let (Some(x), Some(y), Some(a), Some(b)) = (
        world_number(world, "x"),
        world_number(world, "y"),
        world_number(world, "a"),
        world_number(world, "b"),
    ) {
        results.insert(
            "line".to_string(),
            workbench_result(
                WORKBENCH_LINE_RESULT_SCHEMA,
                "line",
                json!({
                    "current": { "x": x, "y": y },
                    "equation": { "a": a, "b": b },
                }),
            ),
        );
    }

    if let (Some(theta), Some(omega), Some(length), Some(bob_x), Some(bob_y)) = (
        world_number(world, "theta"),
        world_number(world, "omega"),
        world_number(world, "L"),
        world_number(world, "bob_x"),
        world_number(world, "bob_y"),
    ) {
        if length > 0.0 {
            results.insert(
                "pendulum".to_string(),
                workbench_result(
                    WORKBENCH_PENDULUM_RESULT_SCHEMA,
                    "pendulum",
                    json!({
                        "current": {
                            "theta": theta,
                            "omega": omega,
                            "energy": optional_world_number_json(world, "energy"),
                            "kinetic_energy": optional_world_number_json(world, "kinetic_energy"),
                            "potential_energy": optional_world_number_json(world, "potential_energy"),
                            "length": length,
                            "physical_bob": { "x": bob_x, "y": bob_y },
                            "normalized_bob": { "x": bob_x / length, "y": -bob_y / length },
                        }
                    }),
                ),
            );
        }
    }

    if let (Some(t), Some(x), Some(y), Some(vx), Some(vy)) = (
        world_number(world, "t"),
        world_number(world, "x"),
        world_number(world, "y"),
        world_number(world, "vx"),
        world_number(world, "vy"),
    ) {
        results.insert(
            "projectile".to_string(),
            workbench_result(
                WORKBENCH_PROJECTILE_RESULT_SCHEMA,
                "projectile",
                json!({
                    "current": { "t": t, "x": x, "y": y, "vx": vx, "vy": vy },
                }),
            ),
        );
    }

    let grid_position = world
        .get_resource_value("위치")
        .and_then(|value| resource_value_to_point(&value));
    let grid_space2d = space2d.filter(|value| {
        value
            .get("drawlist")
            .and_then(JsonValue::as_array)
            .is_some_and(|items| !items.is_empty())
    });
    if let (Some((x, y)), Some(space2d)) = (grid_position, grid_space2d) {
        results.insert(
            "grid".to_string(),
            workbench_result(
                WORKBENCH_GRID_RESULT_SCHEMA,
                "grid",
                json!({
                    "position": { "x": x, "y": y },
                    "dimensions": space2d_dimensions(space2d),
                    "space2d": space2d.clone(),
                }),
            ),
        );
    }

    if results.is_empty() {
        return None;
    }
    Some(json!({
        "schema": WORKBENCH_RESULT_SET_SCHEMA,
        "producer": "ddonirang_tool.wasm_runtime",
        "provenance": "native_authoritative_structured_result",
        "results": results,
    }))
}

fn append_view_stack_meta(out: &mut Map<String, JsonValue>, available_families: &[&str]) {
    let Some((primary_family, secondary_families)) = available_families.split_first() else {
        return;
    };
    out.insert(
        "primary".to_string(),
        json!({
            "family": *primary_family,
            "role": "main",
        }),
    );
    let secondary = secondary_families
        .iter()
        .map(|family| {
            json!({
                "family": *family,
                "role": "secondary",
            })
        })
        .collect::<Vec<_>>();
    out.insert("secondary".to_string(), JsonValue::Array(secondary));
    out.insert("overlays".to_string(), JsonValue::Array(Vec::new()));
}

fn collect_streams(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();
    let sidecars = collect_stream_sidecars(world);
    for (tag, value) in world.resource_value_entries() {
        let meta = sidecars.get(&tag).copied();
        if !is_stream_resource(&tag, &value, meta.as_ref()) {
            continue;
        }
        let Some(stream) = extract_ring_buffer(&value, meta.as_ref()) else {
            continue;
        };
        out.insert(tag, stream);
    }
    JsonValue::Object(out)
}

#[derive(Clone, Copy, Default)]
struct StreamMeta {
    capacity: Option<u64>,
    head: Option<u64>,
    len: Option<u64>,
}

impl StreamMeta {
    fn has_any(self) -> bool {
        self.capacity.is_some() || self.head.is_some() || self.len.is_some()
    }

    fn set_from_kind(&mut self, kind: &str, value: Option<u64>) {
        if value.is_none() {
            return;
        }
        match kind {
            "capacity" => self.capacity = value,
            "head" => self.head = value,
            "len" => self.len = value,
            _ => {}
        }
    }
}

fn collect_stream_sidecars(
    world: &ddonirang_core::platform::NuriWorld,
) -> HashMap<String, StreamMeta> {
    let mut out: HashMap<String, StreamMeta> = HashMap::new();
    for (tag, value) in world.resource_fixed64_entries() {
        let Some((base, kind)) = parse_stream_sidecar_tag(&tag) else {
            continue;
        };
        let numeric = u64::try_from(value.raw_i64()).ok();
        out.entry(base).or_default().set_from_kind(kind, numeric);
    }
    for (tag, value) in world.resource_value_entries() {
        let Some((base, kind)) = parse_stream_sidecar_tag(&tag) else {
            continue;
        };
        let numeric = resource_value_to_u64(&value);
        out.entry(base).or_default().set_from_kind(kind, numeric);
    }
    out
}

fn parse_stream_sidecar_tag(tag: &str) -> Option<(String, &'static str)> {
    const PATTERNS: [(&str, &str); 16] = [
        ("_stream_capacity", "capacity"),
        (".stream_capacity", "capacity"),
        ("_capacity", "capacity"),
        (".capacity", "capacity"),
        ("_head", "head"),
        (".head", "head"),
        ("_length", "len"),
        (".length", "len"),
        ("_len", "len"),
        (".len", "len"),
        ("_용량", "capacity"),
        (".용량", "capacity"),
        ("_머리", "head"),
        (".머리", "head"),
        ("_길이", "len"),
        (".길이", "len"),
    ];
    let lower = tag.to_ascii_lowercase();
    for (suffix, kind) in PATTERNS {
        let is_match = if suffix.is_ascii() {
            lower.ends_with(suffix)
        } else {
            tag.ends_with(suffix)
        };
        if !is_match {
            continue;
        }
        // "foo_stream_capacity"는 base를 "foo_stream"으로 해석한다.
        let raw_base = if suffix == "_stream_capacity" {
            tag.strip_suffix("_capacity").unwrap_or_default()
        } else if suffix == ".stream_capacity" {
            tag.strip_suffix(".capacity").unwrap_or_default()
        } else {
            tag.strip_suffix(suffix).unwrap_or_default()
        };
        let base = raw_base
            .trim_matches(|c| c == '.' || c == '_' || c == '/')
            .to_string();
        if base.is_empty() {
            continue;
        }
        return Some((base, kind));
    }
    None
}

fn is_stream_resource(tag: &str, value: &ResourceValue, sidecar: Option<&StreamMeta>) -> bool {
    if sidecar.copied().unwrap_or_default().has_any() {
        return matches!(value, ResourceValue::List(_) | ResourceValue::Map(_));
    }
    let lower = tag.to_ascii_lowercase();
    if lower.contains("stream") || tag.contains("흐름") {
        return matches!(value, ResourceValue::List(_) | ResourceValue::Map(_));
    }
    let ResourceValue::Map(entries) = value else {
        return false;
    };
    let mut has_buffer = false;
    let mut has_meta = false;
    for entry in entries.values() {
        let key = resource_key_to_string(&entry.key);
        match key.as_str() {
            "buffer" | "버퍼" => has_buffer = true,
            "capacity" | "stream_capacity" | "용량" | "head" | "머리" | "len" | "length"
            | "길이" => has_meta = true,
            _ => {}
        }
    }
    has_buffer || has_meta
}

fn extract_ring_buffer(value: &ResourceValue, sidecar: Option<&StreamMeta>) -> Option<JsonValue> {
    match value {
        ResourceValue::List(items) => {
            let side = sidecar.copied().unwrap_or_default();
            let mut buffer = items.iter().map(resource_value_to_json).collect::<Vec<_>>();
            let capacity = side
                .capacity
                .unwrap_or(items.len() as u64)
                .max(items.len() as u64);
            if buffer.len() < capacity as usize {
                buffer.resize(capacity as usize, JsonValue::Null);
            }
            let len = side
                .len
                .unwrap_or(items.len() as u64)
                .min(buffer.len() as u64);
            let head = if buffer.is_empty() {
                0
            } else {
                side.head
                    .unwrap_or_else(|| len.saturating_sub(1))
                    .min(buffer.len().saturating_sub(1) as u64)
            };
            Some(json!({
                "capacity": capacity,
                "head": head,
                "len": len,
                "buffer": JsonValue::Array(buffer),
            }))
        }
        ResourceValue::Map(entries) => {
            let side = sidecar.copied().unwrap_or_default();
            let mut buffer_json: Option<Vec<JsonValue>> = None;
            let mut capacity: Option<u64> = None;
            let mut head: Option<u64> = None;
            let mut len: Option<u64> = None;

            for entry in entries.values() {
                let key = resource_key_to_string(&entry.key);
                match key.as_str() {
                    "buffer" | "버퍼" => {
                        buffer_json = match &entry.value {
                            ResourceValue::List(items) => {
                                Some(items.iter().map(resource_value_to_json).collect::<Vec<_>>())
                            }
                            other => Some(vec![resource_value_to_json(other)]),
                        };
                    }
                    "capacity" | "stream_capacity" | "용량" => {
                        capacity = resource_value_to_u64(&entry.value);
                    }
                    "head" | "머리" => {
                        head = resource_value_to_u64(&entry.value);
                    }
                    "len" | "length" | "길이" => {
                        len = resource_value_to_u64(&entry.value);
                    }
                    _ => {}
                }
            }
            if capacity.is_none() {
                capacity = side.capacity;
            }
            if head.is_none() {
                head = side.head;
            }
            if len.is_none() {
                len = side.len;
            }

            let mut buffer = buffer_json.unwrap_or_default();
            let cap = capacity.unwrap_or(buffer.len() as u64);
            let cap_clamped = cap.max(buffer.len() as u64);
            if buffer.len() < cap_clamped as usize {
                buffer.resize(cap_clamped as usize, JsonValue::Null);
            }
            let raw_len = len.unwrap_or(buffer.len() as u64);
            let safe_len = raw_len.min(buffer.len() as u64);
            let safe_head = head
                .unwrap_or_else(|| safe_len.saturating_sub(1))
                .min(buffer.len().saturating_sub(1) as u64);

            Some(json!({
                "capacity": cap_clamped,
                "head": if buffer.is_empty() { 0 } else { safe_head },
                "len": safe_len,
                "buffer": buffer,
            }))
        }
        _ => None,
    }
}

fn resource_value_to_u64(value: &ResourceValue) -> Option<u64> {
    match value {
        ResourceValue::Fixed64(v) => u64::try_from(v.raw_i64()).ok(),
        ResourceValue::Unit(v) => u64::try_from(v.value.raw_i64()).ok(),
        ResourceValue::Bool(v) => Some(if *v { 1 } else { 0 }),
        ResourceValue::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn wasm_run_summary_export_preserves_native_compact_response_bytes() {
        let source = "(매마디)마다 { 값 <- 차림[1, 2]. }.";
        let native = crate::runtime_surface::run_summary_from_supported_source_profile(
            source, "compact-parity.ddn", Some(1), Some("v1-core-v25")).unwrap();
        let exported = super::wasm_run_summary_with_source_profile_json(
            source, "compact-parity.ddn", 1, "v1-core-v25").unwrap();
        assert_eq!(exported, native.to_string());
        assert!(!exported.ends_with('\n'));
    }

    use super::*;
    #[test]
    fn wasm_current_state_response_copies_deep_history_without_recursion() {
        std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(|| {
            use crate::unit_wire::SnapshotJsonOwner as Owned;
            let mut vm = DdnWasmVm::new("1 보여주기.").unwrap();
            let shallow = crate::unit_wire::parse_snapshot_json(&vm.current_state_json_text()).unwrap();
            assert_eq!(shallow["public_observations"], shallow["state"]["public_observations"]);
            assert_eq!(shallow["debug_pause"], shallow["state"]["debug_pause"]);
            assert!(shallow["output_rows"].as_array().is_some());
            // Independent legacy shallow serialization oracle: exact bytes,
            // including the deprecated columns shape and absence of new keys.
            let expected_columns = json!({
                "observation_columns": vm.public_observation_trace.get("columns").cloned().unwrap_or_else(|| json!([])),
                "output_rows": vm.public_observation_trace.get("samples").cloned().unwrap_or_else(|| json!([])),
                "public_observation_trace": vm.public_observation_trace,
                "public_observation_trace_hash": vm.public_observation_trace_hash,
                "public_observation_table_projection": vm.public_observation_table_projection,
                "deprecated": true,
                "use_instead": "step_one() 또는 get_state_json()의 public_observation_trace",
            }).to_string();
            assert_eq!(vm.columns_json_text(), expected_columns);
            let mut value = JsonValue::Bool(true);
            for _ in 0..20_000 { value = JsonValue::Array(vec![value]); }
            let deep = Owned::from_value(value);
            vm.last_public_observations.push(deep.clone().into_parent_value());
            vm.public_observation_frames.push(JsonValue::Object([
                ("committed_madi".into(), 0.into()),
                ("checkpoint".into(), deep.clone().into_parent_value()),
            ].into_iter().collect()));
            crate::unit_wire::replace_retained_json(&mut vm.public_observation_trace,
                JsonValue::Object([("columns".into(), deep.clone().into_parent_value()),
                    ("samples".into(), deep.clone().into_parent_value())].into_iter().collect()));
            crate::unit_wire::replace_retained_json(&mut vm.public_observation_table_projection, deep.clone().into_parent_value());
            crate::unit_wire::replace_retained_json(&mut vm.last_debug_evidence, deep.clone().into_parent_value());
            let before = vm.get_state_hash();
            let encoded = vm.current_state_json_text();
            // The transitional restore depth gate remains enabled. Verify
            // export bytes without trying to admit this synthetic deep state.
            let exact_deep = format!("{}true{}", "[".repeat(20_000), "]".repeat(20_000));
            for key in ["observation_columns", "output_rows", "public_observation_table_projection", "debug_pause"] {
                assert!(encoded.contains(&format!("\"{key}\":{exact_deep}")), "{key}");
            }
            assert_eq!(encoded.matches(&format!("\"public_observations\":[{exact_deep}]")).count(), 2);
            assert!(encoded.contains(&format!("\"checkpoint\":{exact_deep}")));
            assert!(crate::unit_wire::parse_snapshot_json(&encoded).is_err());
            let columns = vm.columns_json_text();
            for key in ["observation_columns", "output_rows", "public_observation_table_projection"] {
                assert!(columns.contains(&format!("\"{key}\":{exact_deep}")));
            }
            assert!(!columns.contains("\"public_observations\":"));
            assert!(!columns.contains("\"debug_pause\":"));
            assert_eq!(vm.get_state_hash(), before);
            // Ownership/response safety only: these synthetic nested fields
            // are not an admission or replay claim for a new observation type.
        }).unwrap().join().unwrap();
    }

    #[test]
    fn wasm_retained_observation_json_replacement_reset_and_close_are_stack_safe() {
        // VM owns thread-local runtime state; do not move it between threads.
        std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(|| {
            let mut vm = DdnWasmVm::new("1 보여주기.").unwrap();
            let deep = || {
                let mut value = JsonValue::Null;
                for _ in 0..20_000 { value = JsonValue::Array(vec![value]); }
                value
            };
            vm.last_bogae_projection = Some(deep());
            vm.last_public_observations.push(deep());
            vm.public_observation_frames.push(deep());
            vm.public_observation_trace = deep();
            vm.public_observation_table_projection = deep();
            crate::unit_wire::replace_retained_json(&mut vm.public_observation_trace, deep());
            reset_wasm_public_observation_surface(&mut vm).unwrap();
            assert!(vm.last_bogae_projection.is_none());
            assert!(vm.public_observation_frames.is_empty());
            assert!(vm.last_public_observations.is_empty());
            vm.public_observation_frames.push(deep());
            vm.last_debug_evidence = deep();
            drop(vm);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn wasm_snapshot_builder_transfers_deep_resources_without_reencoding() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut value = JsonValue::Bool(true);
            for _ in 0..20_000 { value = JsonValue::Array(vec![value]); }
            let snapshot = crate::unit_wire::SnapshotJsonOwner::from_value(build_wasm_state_snapshot_v2(
                value, json!({}), vec![], 7, json!({})));
            assert_eq!(snapshot["next_tick_id"], json!(7));
            let copied = snapshot.clone();
            assert!(copied == snapshot);
            assert_eq!(copied.compact_text(), snapshot.compact_text());
        }).unwrap().join().unwrap();
        let resources = json!({"tree":[null,1.25,"한글"]});
        let expected = json!({"schema":WASM_STATE_SNAPSHOT_SCHEMA_V2,
            "resource_snapshot_schema":RESOURCE_SNAPSHOT_SCHEMA_V2,"unit_wire_schema":UNIT_VALUE_WIRE_SCHEMA_V2,
            "resources":resources.clone(),"param_overrides":{},"public_observation_frames":[],
            "next_tick_id":7,"scene_history":{}});
        let actual = build_wasm_state_snapshot_v2(resources, json!({}), vec![], 7, json!({}));
        assert_eq!(actual.to_string(), expected.to_string());
    }
    use ddonirang_core::{
        replay_sealed_external_execution, CanonicalExternalInput, CapabilityBinding, CapabilityRef,
        CapabilityScope, CapabilityValue, DependencyNodeKind, DeterministicSealedProvider,
        ExternalBoundaryRequest, ExternalDependencyNode,
    };
    use ddonirang_core::platform::ResourceMapEntry;
    use sha2::{Digest, Sha256};

    #[test]
    fn d9_wasm_vm_binds_named_project_entry_after_profile_update() {
        let source = "중력:수 := 9.8. 실행:움직씨 := { 중력 보여주기. }";
        let mut vm = DdnWasmVm::new("1 보여주기.").unwrap();
        vm.update_logic_with_source_profile(source, "v1-core-v25").unwrap();
        vm.configure_project_entry("실행").unwrap();
        let input = InputSnapshot {
            tick_id: 0, dt: Fixed64::from_i64(1), keys_pressed: 0,
            last_key_name: String::new(), pointer_x_i32: 0, pointer_y_i32: 0,
            ai_injections: Vec::new(), net_events: Vec::new(),
            frame_source: InputSource::Person, rng_seed: 0,
        };
        let output = vm.runner.run_update_with_param_overrides(vm.world.world(), &input,
            &vm.defaults, &vm.param_overrides).unwrap();
        assert!(output.resources.contains_key("중력"));
    }

    #[test]
    fn wasm_vm_profile_update_replaces_inert_bootstrap_with_v25_source() {
        let source = "매마디:움직씨 := {\n    값:수 := 1.\n    값 <- 2.\n    값 보여주기.\n}\n";
        let mut vm = DdnWasmVm::new("1 보여주기.").expect("inert bootstrap VM");
        vm.update_logic_with_source_profile(source, "v1-core-v25")
            .expect("V25 profile update");
        assert_eq!(vm.get_configured_madi(), 0);
        assert_eq!(vm.source_bytes_sha256, runtime_surface::sha256_hex(source.as_bytes()));
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn wo02_invalid_vm_update_preserves_installed_source() {
        let valid = "값 <- 1. 값 보여주기.";
        let invalid = "값 <- 1. 값 <- \"가\".";
        assert!(DdnWasmVm::new(invalid).is_err());
        let mut vm = DdnWasmVm::new(valid).expect("valid source");
        let before = vm.source_bytes_sha256.clone();
        assert!(vm.update_logic(invalid).is_err());
        assert_eq!(vm.source_bytes_sha256, before);
        assert!(vm.update_logic_with_mode(invalid, "strict").is_err());
        assert_eq!(vm.source_bytes_sha256, before);
        assert!(vm.update_logic_with_source_profile(invalid, "v1-core-v25").is_err());
        assert_eq!(vm.source_bytes_sha256, before);
    }

    #[test]
    fn wasm_fixed64_input_v1_reconstructs_q31_32_without_f64_conversion() {
        let mut vm = DdnWasmVm::new("1 보여주기.").expect("inert bootstrap VM");

        vm.set_input_fixed64_parts(7, "k", -3, 9, i32::MIN, 0);
        assert_eq!(vm.input_dt, Fixed64::MIN);
        assert_eq!(vm.input_keys_pressed, 7);
        assert_eq!(vm.input_last_key_name, "k");
        assert_eq!(vm.input_pointer_x_i32, -3);
        assert_eq!(vm.input_pointer_y_i32, 9);

        vm.set_dt_fixed64_parts(0, 1);
        assert_eq!(vm.input_dt, Fixed64::from_raw_i64(1));
        assert_eq!(
            fixed64_q31_32_parts(Fixed64::MIN),
            json!({"high_i32": i32::MIN, "low_u32": 0})
        );
        for value in [
            Fixed64::MIN,
            Fixed64::MAX,
            Fixed64::from_raw_i64(-1),
            Fixed64::from_raw_i64(1),
            Fixed64::from_raw_i64(0x0000_0000_FFFF_FFFF),
        ] {
            let parts = fixed64_q31_32_parts(value);
            let high = parts["high_i32"].as_i64().expect("high") as i32;
            let low = parts["low_u32"].as_u64().expect("low") as u32;
            assert_eq!(fixed64_from_q31_32_parts(high, low), value);
        }
        assert_eq!(
            Fixed64::parse_decimal_exact("0.5"),
            Some(fixed64_from_q31_32_parts(0, 0x8000_0000))
        );
    }


    fn sealed_external_test_request() -> (ProcessLocalCapabilityStore, ExternalBoundaryRequest) {
        let binding = CapabilityBinding::new("owner-a", "world-a", 7, 3);
        let mut store = ProcessLocalCapabilityStore::new(7);
        assert!(store.insert(CapabilityValue::new(
            CapabilityRef::from_process_local(41),
            binding.clone(),
            [CapabilityScope::ExternalBoundaryExecute { boundary_id: "boundary:one".to_string() }],
        )));
        let request = ExternalBoundaryRequest {
            dependency: ExternalDependencyNode {
                kind: DependencyNodeKind::ExternalArtifact,
                artifact_identity: "artifact:sealed-01".to_string(),
                provenance_identity: "provenance:sealed-01".to_string(),
                provider_owner: "provider:sealed".to_string(),
                callable_owner: "callable:transform".to_string(),
                invocation_identity: "invocation:one".to_string(),
                execution_contract: "contract:sealed-v1".to_string(),
                external_boundary_identity: "boundary:one".to_string(),
            },
            input: CanonicalExternalInput { canonical_text: "입력:42".to_string() },
            capability: Some(CapabilityRef::from_process_local(41)),
            capability_binding: binding,
            max_input_bytes: 64,
            max_output_bytes: 64,
            max_calls: 1,
            call_ordinal: 1,
        };
        (store, request)
    }

    #[test]
    fn external_boundary_native_and_wasm_adapters_share_one_typed_owner() {
        let provider = DeterministicSealedProvider::new("provider:sealed", "callable:transform");
        let (native_store, request) = sealed_external_test_request();
        let (wasm_store, wasm_request) = sealed_external_test_request();
        let mut native_ledger = SealedExecutionLedger::new();
        let mut wasm_ledger = SealedExecutionLedger::new();

        let native = runtime_surface::execute_sealed_external_boundary(
            &mut native_ledger, &native_store, &provider, &request,
        ).expect("native adapter");
        let wasm = wasm_execute_sealed_external_boundary(
            &mut wasm_ledger, &wasm_store, &provider, &wasm_request,
        ).expect("wasm adapter");

        assert_eq!(native, wasm);
        assert_eq!(native.dispatch_count, 1);
        assert_eq!(native.publication_count, 1);
        assert_eq!(
            replay_sealed_external_execution(
                &request.dependency, &request.input, &native.output, &native.receipt,
            ),
            Ok(())
        );
    }

    #[test]
    fn declared_host_resource_native_and_wasm_share_one_receipt_bound_payload() {
        let source = "매마디:움직씨 := {\n    값:수 := 1.\n    값 보여주기.\n}\n";
        let text = "고정 입력\n";
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let request = serde_json::json!({
            "schema": crate::host_resource::HOST_RESOURCE_REQUEST_SCHEMA,
            "mode": "record",
            "resources": [{
                "logical_resource_id": "caption",
                "sha256": format!("{:x}", hasher.finalize()),
                "byte_length": text.len(),
                "media_type": crate::host_resource::V1_TEXT_MEDIA_TYPE,
                "utf8_text": text,
            }]
        })
        .to_string();
        let parsed = crate::host_resource::parse_request_json(&request).expect("typed request");
        let prepared = crate::host_resource::prepare_request(&parsed).expect("record preflight");
        let native = runtime_surface::run_summary_from_supported_source_profile_with_host_resources(
            source,
            "host-resource.ddn",
            Some(1),
            Some("v1-core-v25"),
            &prepared,
        )
        .expect("native shared runtime");
        let wasm = wasm_run_summary_with_source_profile_host_resources_json(
            source,
            "host-resource.ddn",
            1,
            "v1-core-v25",
            &request,
        )
        .expect("WASM shared runtime");
        assert_eq!(wasm, native.to_string());
        assert_eq!(
            native["host_resource_receipt"]["receipt_identity"],
            prepared.receipt.receipt_identity
        );

        let replay = serde_json::json!({
            "schema": crate::host_resource::HOST_RESOURCE_REQUEST_SCHEMA,
            "mode": "replay",
            "transcript": prepared.transcript,
        })
        .to_string();
        assert_eq!(
            wasm_run_summary_with_source_profile_host_resources_json(
                source,
                "host-resource.ddn",
                1,
                "v1-core-v25",
                &replay,
            )
            .expect("WASM replay"),
            native.to_string()
        );
    }

    #[test]
    fn p1a_supported_profile_native_wasm_payloads_are_exactly_equal() {
        let cases = [
            (
                "p1a-parity.ddn",
                "매마디:움직씨 := {\n    값:수 := 1.\n    값 <- 2.\n    값 보여주기.\n}\n",
            ),
            (
                "vc12-collection-parity.ddn",
                "매마디:움직씨 := {\n    값들:차림 := (1, 2, 3) 차림.\n    값들 보여주기.\n}\n",
            ),
            (
                "pg-cli-result-flow-parity.ddn",
                "매마디:움직씨 := {\n    판:(수)차림 := 차림[0, 0].\n    1 -> 판[0].\n    판 보여주기.\n}\n",
            ),
        ];
        for (path, source) in cases {
            let native = runtime_surface::run_summary_from_supported_source_profile(
                source,
                path,
                Some(1),
                Some("v1-core-v25"),
            )
            .expect("native supported profile summary")
            .to_string();
            let wasm = wasm_run_summary_with_source_profile_json(
                source,
                path,
                1,
                "v1-core-v25",
            )
            .expect("WASM supported profile summary");
            assert_eq!(wasm, native, "path={path}");
        }

        let source = cases[0].1;

        let native_canon = runtime_surface::canonicalize_shared_source_with_supported_profile(
            source,
            Some("v1-core-v25"),
        )
        .expect("native supported profile canon")
        .ddn;
        let wasm_canon = wasm_canon_ddn_with_source_profile(source, "v1-core-v25")
            .expect("WASM supported profile canon");
        assert_eq!(wasm_canon, native_canon);

        let request = serde_json::json!({
            "source_text": source,
            "path": "p1a-parity.ddn",
            "analysis_generation": 201,
            "source_profile_identity": "v1-core-v25"
        })
        .to_string();
        assert_eq!(
            wasm_source_analysis_snapshot_json(&request).expect("WASM source analysis"),
            source_analysis::source_analysis_snapshot_json(&request)
                .expect("native source analysis")
        );

        let module_request = serde_json::json!({
            "schema": module_directive_analysis::MODULE_DIRECTIVE_ANALYSIS_SCHEMA,
            "source_text": "(운동=\"./lib/😀\") 쓰임.\n(속도=중력) 드러냄.\n계산:움직씨 := {\n  값:수 := 1.\n}\n",
            "path": "운동😀.ddn",
            "analysis_generation": 202,
            "source_profile_identity": "v1-core-v25",
            "module_registry": {
                "known_import_targets": ["./lib/😀"],
                "public_definition_targets": ["중력"]
            }
        })
        .to_string();
        assert_eq!(
            wasm_module_directive_analysis_snapshot_json(&module_request)
                .expect("WASM module directive analysis"),
            module_directive_analysis::module_directive_analysis_snapshot_json(&module_request)
                .expect("native module directive analysis")
        );
    }

    #[test]
    fn committed_madi_is_last_successfully_committed_frame_not_next_tick() {
        assert_eq!(last_committed_madi(&[]), JsonValue::Null);
        let frames = vec![
            json!({"committed_madi": 0, "state_hash": "a"}),
            json!({"committed_madi": 7, "state_hash": "b"}),
        ];
        assert_eq!(last_committed_madi(&frames), json!(7));
    }

    const SAMPLE_MAEGIM_SOURCE: &str = r#"
채비 {
  g:수 = (9.8) 매김 {
    범위: 1..20.
    간격: 0.1.
  }.
  theta0:수 <- (0.5) 매김 {
    범위: -1.2..1.2.
    분할수: 24.
  }.
}.
"#;

    const SAMPLE_GUSEONG_SOURCE: &str = r#"
짜임 {
  형식: 진자_틀.
  입력 {
    기준점: (수,수) <- (0.0, 0.0).
  }.
  출력 {
    끝점: (수,수) <- (0.0, 0.0).
  }.
}.

짜임 {
  형식: 물체_틀.
  입력 {
    붙는점: (수,수) <- (0.0, 0.0).
  }.
  출력 {
    꼭짓점: (수,수) <- (0.0, 0.0).
  }.
}.

a <- (L=1.0)인 진자_틀.
b <- (L=0.8)인 물체_틀.
a.기준점 <- b.꼭짓점.
"#;

    const SAMPLE_ALRIM_SOURCE: &str = r#"
"jump"라는 알림이 오면 {
  y <- 1.
}.

매틱:움직씨 = {
  "tick"라는 알림이 오면 {
    t <- t + 1.
  }.
}.

"reset"라는 알림이 오면 {
  y <- 0.
}.
"#;

    const SAMPLE_BLOCK_EDITOR_SOURCE: &str = r#"
채비 {
  g:수 = (9.8) 매김 {
    범위: 1..20.
    간격: 0.1.
  }.
  t:변수 <- 0.
  안내:글 = "초기".
}.

(시작)할때 {
  안내 보여주기.
}.

(매마디)마다 {
  t <- t + 1.
  t < 3 일때 {
    t 보여주기.
  } 아니면 {
    g 보여주기.
  }.
}.

(x) x목록에 대해 {
  고르기:
    { x < 1 }인것: {
      "low" 보여주기.
    }
    아니면: {
      "high" 보여주기.
    }.
}.

고르기:
  t < 10 인 경우 {
    "small" 보여주기.
  }
  모든 경우 다룸.

되풀이 {
  "tick" 보여주기.
}.

너머 {
  "side" 보여주기.
}.

{ t < 2 }인 동안 {
  t <- t + 1.
}.

{ t == 2 }인것 바탕으로 아니면 {
  "bad" 보여주기.
} 맞으면 {
  "ok" 보여주기.
}.

{ t >= 0 }인것 다짐하고(알림) 아니면 {
  "warn" 보여주기.
}.

?? {
  "prompt" 보여주기.
}

"answer" ??: {
  "after" 보여주기.
}

{ t >= 0 }인것 ??
?? {
  "cond" 보여주기.
}

??:
  { t < 1 }인것: {
    "choose-low" 보여주기.
  }
??: {
    "choose-else" 보여주기.
  }

(기준:수 = 1) 판정:움직씨 = {
  기준 되돌림.
}.

기상청:임자 = {
  기상특보를 받으면 {
    알림.이름 보여주기.
  }.

  (정보 정보.온도 > 40)인 기상특보를 받으면 {
    정보.온도 보여주기.
  }.

  (알림 알림.이름 == "경보")인 알림을 받으면 {
    알림.정보.온도 보여주기.
  }.
}.

"tick"라는 알림이 오면 {
  "evt" 보여주기.
}

보개로 그려.

(매마디)마다 {
  t 톺아보기.
  되풀이 {
    t <- t + 1.
    { t > 0 }인것 일때 {
      멈추기.
    }.
  }.
}

실행정책 {
  실행모드: 일반.
  효과정책: 허용.
}.

짜임 {
  형식: 점_틀.
  입력 {
    시작점: (수,수) <- (0.0, 0.0).
  }.
  출력 {
    끝점: (수,수) <- (0.0, 0.0).
  }.
}.

테스트:움직씨 = {
  보개마당 {
    #자막("정본 테스트").
  }.
}.

() 증명.
"#;

    const WAVE1_LESSONS: &[(&str, &str)] = &[
        (
            "synthetic/projectile-canon-surface.ddn",
            crate::self_contained_test_sources::PROJECTILE_CANON_SURFACE,
        ),
        (
            "synthetic/self-contained-multifile-entry.ddn",
            crate::self_contained_test_sources::MULTIFILE_ENTRY,
        ),
    ];

    fn wave1_lessons() -> Vec<(&'static str, String)> {
        WAVE1_LESSONS
            .iter()
            .map(|(path, source)| (*path, (*source).to_string()))
            .collect()
    }

    fn cli_canon_output(source: &str) -> crate::canon::CanonOutput {
        crate::canon::canonicalize(source, false).expect("cli canon")
    }

    fn cli_like_exec_policy_emit(source: &str) -> String {
        if !crate::canon::has_exec_policy_surface(source) {
            "{}\n".to_string()
        } else {
            cli_canon_output(source).exec_policy_map_json
        }
    }

    #[test]
    fn wasm_canon_flat_json_matches_cli_canon_surface() {
        let flat_json = wasm_canon_flat_json(SAMPLE_GUSEONG_SOURCE).expect("wasm flat json");
        assert!(flat_json.contains("\"schema\": \"ddn.guseong_flatten_plan.v1\""));
        assert!(flat_json.contains("\"name\": \"a\""));
        assert!(flat_json.contains("\"name\": \"b\""));
        assert!(flat_json.contains("\"dst_port\": \"기준점\""));
    }

    #[test]
    fn wasm_canon_flat_json_exact_parity_with_cli_output() {
        let wasm = wasm_canon_flat_json(SAMPLE_GUSEONG_SOURCE).expect("wasm flat json");
        let cli = cli_canon_output(SAMPLE_GUSEONG_SOURCE).guseong_flat_json;
        assert_eq!(wasm, cli);
    }

    #[test]
    fn wasm_model_frontdoor_matches_shared_rust_canon_and_semantic_hash() {
        let source = r#"
낙하운동 = 모형 {
  시간:수@s :: (축, 처음::고정=0@s).
  위치:수@m :: (이어감, 처음::고정=100@m, 처음받음, 내줌).
  속도:수@m/s :: (들임, 내줌).
  위치' =:= 속도.
}.
"#;
        let wasm_canon = wasm_canon_ddn(source).expect("wasm model canon");
        let shared_canon =
            runtime_surface::canonicalize_shared_ddn(source).expect("shared model canon");
        assert_eq!(wasm_canon, shared_canon);

        let wasm_ir: JsonValue =
            serde_json::from_str(&wasm_model_ir_candidate_json(source).expect("wasm model IR"))
                .expect("wasm model IR json");
        let shared_ir: JsonValue = serde_json::from_str(
            &runtime_surface::model_ir_candidate_json_from_source(source).expect("shared model IR"),
        )
        .expect("shared model IR json");
        assert_eq!(wasm_ir["semantic_hash"], shared_ir["semantic_hash"]);
        assert_eq!(wasm_ir, shared_ir);
    }

    #[test]
    fn wasm_canon_boim_surface_is_not_legacy() {
        let source = r#"
(매마디)마다 {
  n <- 1.
  (n: n) 보임.
}.
"#;
        assert!(!crate::canon::has_legacy_boim_surface(source));
        assert!(!crate::canon::has_legacy_boim_surface(
            SAMPLE_GUSEONG_SOURCE
        ));
    }

    #[test]
    fn wasm_run_summary_emits_ordered_successor_public_observation_trace() {
        let source = r#"
채비 {
  시간 <- 0.
  각도 <- 15.
  각속도 <- 2.
}.
(매마디)마다 {
  시간 <- 시간 + 1.
  각도 <- 각도 + 각속도.
  (시간: 시간 * 1@s, 각도: 각도 * 1@rad, 각속도: 각속도) 보임.
}.
"#;
        let text = wasm_run_summary_json(source, "pack/ddn_boim_open_value_pack_w1/main.ddn", 2)
            .expect("wasm run summary");
        let summary: JsonValue = serde_json::from_str(&text).expect("summary json");
        let trace = &summary["public_observation_trace"];
        assert_eq!(trace["schema"], "ddn.public_observation_trace.v1");
        let names = trace["columns"]
            .as_array()
            .expect("columns")
            .iter()
            .map(|column| column["identity"]["name"].as_str().unwrap_or_default())
            .collect::<Vec<_>>();
        assert_eq!(names, ["시간", "각도", "각속도"]);
        let samples = trace["samples"].as_array().expect("samples");
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0]["cells"][0]["value_text"], "1@s");
        assert_eq!(samples[0]["cells"][1]["value_text"], "17@rad");
        assert_eq!(samples[1]["cells"][2]["value_text"], "2");
        assert!(trace["columns"][0]["identity"]["source_ref"]
            .as_str()
            .unwrap_or_default()
            .starts_with("pack/ddn_boim_open_value_pack_w1/main.ddn#bytes="));
        let projection = &summary["public_observation_table_projection"];
        assert_eq!(
            projection["schema"],
            "ddn.public_observation_table_projection.v2"
        );
        assert_eq!(projection["presentation_state"], "ready");
        assert_eq!(
            projection["tables"][0]["columns"]
                .as_array()
                .expect("projection columns")
                .iter()
                .map(|column| column["identity"]["name"].as_str().unwrap_or_default())
                .collect::<Vec<_>>(),
            names
        );
        assert_eq!(projection["tables"].as_array().map(Vec::len), Some(1));
        assert_eq!(
            projection["tables"][0]["rows"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(
            projection["tables"][0]["rows"][0]["cells"][0]["value_text"],
            "1@s"
        );
        assert_eq!(
            projection["tables"][0]["rows"][0]["cells"][1]["value_text"],
            "17@rad"
        );
        assert_eq!(
            projection["tables"][0]["rows"][1]["cells"][2]["value_text"],
            "2"
        );
        assert_eq!(
            projection["source_trace_hash"],
            summary["public_observation_trace_hash"]
        );
        assert_eq!(
            &summary["engine_state"]["public_observation_table_projection"],
            projection
        );
        assert_eq!(
            projection["tables"][0]["rows"][1]["state_hash"],
            summary["state_hash"]
        );
        assert!(summary.get("observation_manifest").is_none());
        assert!(summary["engine_state"]
            .get("observation_manifest")
            .is_none());
    }

    #[test]
    fn wasm_run_summary_keeps_multiple_and_repeated_authored_observations_as_separate_tables() {
        let source = r#"
채비 {
  시간 <- 0.
  각도 <- 10.
  횟수 <- 0.
}.

(매마디)마다 {
  되풀이 {
    (값: 시간, 증가: 1) 보임.
    횟수 <- 횟수 + 1.
    { 횟수 >= 2 }인것 일때 {
      멈추기.
    }.
  }.
  (값: 각도, 다른: 시간) 보임.
}.
"#;
        let text =
            wasm_run_summary_json(source, "app/main.ddn", 1).expect("multiple observation summary");
        let summary: JsonValue = serde_json::from_str(&text).expect("summary json");
        let trace = &summary["public_observation_trace"];
        assert_eq!(trace["samples"].as_array().map(Vec::len), Some(3));
        assert!(trace["samples"]
            .as_array()
            .expect("samples")
            .iter()
            .all(|sample| sample["committed_madi"] == 0));

        let projection = &summary["public_observation_table_projection"];
        assert_eq!(
            projection["schema"],
            "ddn.public_observation_table_projection.v2"
        );
        assert_eq!(projection["tables"].as_array().map(Vec::len), Some(2));
        assert_eq!(
            projection["tables"][0]["columns"][0]["identity"]["name"],
            "값"
        );
        assert_eq!(
            projection["tables"][0]["columns"][1]["identity"]["name"],
            "증가"
        );
        assert_eq!(
            projection["tables"][0]["rows"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(
            projection["tables"][1]["columns"][0]["identity"]["name"],
            "값"
        );
        assert_eq!(
            projection["tables"][1]["columns"][1]["identity"]["name"],
            "다른"
        );
        assert_eq!(
            projection["tables"][1]["rows"].as_array().map(Vec::len),
            Some(1)
        );
        assert_ne!(
            projection["tables"][0]["columns"][0]["identity"]["source_ref"],
            projection["tables"][1]["columns"][0]["identity"]["source_ref"]
        );
        assert_eq!(projection["tables"][0]["rows"][0]["display_event_index"], 0);
        assert_eq!(projection["tables"][0]["rows"][1]["display_event_index"], 1);
        assert_eq!(projection["tables"][1]["rows"][0]["display_event_index"], 2);
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn wasm_vm_constructor_build_info_contains_pkg() {
        let vm = DdnWasmVm::new(SAMPLE_GUSEONG_SOURCE).expect("vm constructor");
        assert!(vm.get_build_info().contains("pkg="));
    }

    #[test]
    fn wasm_canon_maegim_plan_matches_cli_canon_surface() {
        let maegim_json = wasm_canon_maegim_plan(SAMPLE_MAEGIM_SOURCE).expect("wasm maegim plan");
        assert!(maegim_json.contains("\"schema\": \"ddn.maegim_control_plan.v1\""));
        assert!(maegim_json.contains("\"name\": \"g\""));
        assert!(maegim_json.contains("\"name\": \"theta0\""));
        assert!(maegim_json.contains("\"split_count_expr_canon\": \"24\""));
    }

    #[test]
    fn wasm_canon_maegim_plan_exact_parity_with_cli_output() {
        let wasm = wasm_canon_maegim_plan(SAMPLE_MAEGIM_SOURCE).expect("wasm maegim plan");
        let cli = cli_canon_output(SAMPLE_MAEGIM_SOURCE).maegim_control_json;
        assert_eq!(wasm, cli);
    }

    #[test]
    fn wasm_canon_alrim_plan_matches_cli_canon_surface() {
        let alrim_json = wasm_canon_alrim_plan(SAMPLE_ALRIM_SOURCE).expect("wasm alrim plan");
        assert!(alrim_json.contains("\"schema\": \"ddn.alrim_event_plan.v1\""));
        assert!(alrim_json.contains("\"kind\": \"jump\""));
        assert!(alrim_json.contains("\"kind\": \"tick\""));
        assert!(alrim_json.contains("\"scope\": \"root/seed:매틱\""));
    }

    #[test]
    fn wasm_canon_alrim_plan_exact_parity_with_cli_output() {
        let wasm = wasm_canon_alrim_plan(SAMPLE_ALRIM_SOURCE).expect("wasm alrim plan");
        let cli = cli_canon_output(SAMPLE_ALRIM_SOURCE).alrim_plan_json;
        assert_eq!(wasm, cli);
    }

    #[test]
    fn wasm_canon_exec_policy_map_uses_empty_fast_path_without_surface() {
        let source = r#"
채비 {
  x:수 <- 1.
}.
"#;
        let exec_json = wasm_canon_exec_policy_map(source).expect("wasm exec policy map");
        assert_eq!(exec_json.trim(), "{}");
    }

    #[test]
    fn wasm_canon_exec_policy_map_exact_parity_with_cli_emit_semantics_without_surface() {
        let source = r#"
채비 {
  x:수 <- 1.
}.
"#;
        let wasm = wasm_canon_exec_policy_map(source).expect("wasm exec policy map");
        let cli_like = cli_like_exec_policy_emit(source);
        assert_eq!(wasm, cli_like);
    }

    #[test]
    fn wasm_canon_exec_policy_map_emits_schema_with_surface() {
        let source = r#"
너머 {
  실행모드: 일반.
  효과정책: 허용.
}.
"#;
        let exec_json = wasm_canon_exec_policy_map(source).expect("wasm exec policy map");
        assert!(exec_json.contains("\"schema\": \"ddn.exec_policy_effect_map.v1\""));
        assert_ne!(exec_json.trim(), "{}");
    }

    #[test]
    fn wasm_canon_exec_policy_map_exact_parity_with_cli_emit_semantics_with_surface() {
        let source = r#"
너머 {
  실행모드: 일반.
  효과정책: 허용.
}.
"#;
        let wasm = wasm_canon_exec_policy_map(source).expect("wasm exec policy map");
        let cli_like = cli_like_exec_policy_emit(source);
        assert_eq!(wasm, cli_like);
    }

    #[test]
    fn wasm_canon_wave1_flat_json_exact_parity_with_cli_output() {
        for (lesson_path, source) in wave1_lessons() {
            let wasm = wasm_canon_flat_json(&source).expect("wasm flat json");
            let cli = cli_canon_output(&source).guseong_flat_json;
            assert_eq!(wasm, cli, "wave1 flat parity mismatch: {lesson_path}");
        }
    }

    #[test]
    fn wasm_canon_wave1_maegim_plan_exact_parity_with_cli_output() {
        for (lesson_path, source) in wave1_lessons() {
            let wasm = wasm_canon_maegim_plan(&source).expect("wasm maegim plan");
            let cli = cli_canon_output(&source).maegim_control_json;
            assert_eq!(wasm, cli, "wave1 maegim parity mismatch: {lesson_path}");
        }
    }

    #[test]
    fn wasm_canon_wave1_alrim_plan_exact_parity_with_cli_output() {
        for (lesson_path, source) in wave1_lessons() {
            let wasm = wasm_canon_alrim_plan(&source).expect("wasm alrim plan");
            let cli = cli_canon_output(&source).alrim_plan_json;
            assert_eq!(wasm, cli, "wave1 alrim parity mismatch: {lesson_path}");
        }
    }

    #[test]
    fn wasm_canon_wave1_exec_policy_map_exact_parity_with_cli_emit_semantics() {
        for (lesson_path, source) in wave1_lessons() {
            let wasm = wasm_canon_exec_policy_map(&source).expect("wasm exec policy map");
            let cli_like = cli_like_exec_policy_emit(&source);
            assert_eq!(wasm, cli_like, "wave1 exec parity mismatch: {lesson_path}");
        }
    }

    #[test]
    fn wasm_canon_block_editor_plan_matches_cli_canon_surface() {
        // Report the actual source diagnostic before constructing a JS error
        // on a Native test host (wasm-bindgen cannot do that off wasm32).
        let _ = cli_canon_output(SAMPLE_BLOCK_EDITOR_SOURCE);
        let block_json =
            wasm_canon_block_editor_plan(SAMPLE_BLOCK_EDITOR_SOURCE).expect("wasm block plan");
        assert!(block_json.contains("\"schema\": \"ddn.block_editor_plan.v1\""));
        assert!(block_json.contains("\"kind\": \"charim_block\""));
        assert!(block_json.contains("\"kind\": \"hook_start\""));
        assert!(block_json.contains("\"kind\": \"hook_tick\""));
        assert!(block_json.contains("\"kind\": \"if_else\""));
        assert!(block_json.contains("\"kind\": \"for_each\""));
        assert!(block_json.contains("\"kind\": \"choose_else\""));
        assert!(block_json.contains("\"kind\": \"choose_exhaustive\""));
        assert!(block_json.contains("\"exhaustive\": \"true\""));
        assert!(block_json.contains("\"kind\": \"repeat\""));
        assert!(block_json.contains("\"kind\": \"open_block\""));
        assert!(block_json.contains("\"kind\": \"while_block\""));
        assert!(block_json.contains("\"kind\": \"contract_guard\""));
        assert!(block_json.contains("\"kind\": \"prompt_block\""));
        assert!(block_json.contains("\"kind\": \"prompt_after\""));
        assert!(block_json.contains("\"kind\": \"prompt_condition\""));
        assert!(block_json.contains("\"kind\": \"prompt_choose\""));
        assert!(block_json.contains("\"kind\": \"seed_def\""));
        assert!(block_json.contains("\"kind\": \"receive_block\""));
        assert!(block_json.contains("\"kind\": \"event_react\""));
        // Typed rendezvous is not a legacy flat block-editor node. Its
        // admission guard is checked separately below, without a JS exception
        // on this Native host.
        let transfer = crate::canon::canonicalize(
            "(기상청)의 (온도:1) 경보 ~~> 제.", false).err().unwrap();
        assert_eq!(transfer.code(), "E_CANON_TYPED_TRANSFER_REQUIRES_SHARED_FRONTDOOR");
        assert!(block_json.contains("\"kind\": \"return_value\""));
        assert!(block_json.contains("\"kind\": \"inspect_value\""));
        assert!(block_json.contains("\"kind\": \"bogae_draw\""));
        assert!(block_json.contains("\"kind\": \"break_loop\""));
        assert!(block_json.contains("\"kind\": \"bogae_madang_block\""));
        assert!(block_json.contains("\"kind\": \"exec_policy_block\""));
        assert!(block_json.contains("\"kind\": \"jjaim_block\""));
        assert!(block_json.contains("\"kind\": \"expr_stmt\""));
        assert!(block_json.contains("\"exprs\": {"));
        assert!(block_json.contains("\"kind\": \"call\""));
        // Preserve binding-node coverage separately from typed transfer,
        // which the flat editor intentionally does not admit.
        let binding_plan = wasm_canon_block_editor_plan("(값:1) 보여주기.")
            .expect("named argument block plan");
        assert!(binding_plan.contains("\"kind\": \"binding\""));
    }

    #[test]
    fn wasm_canon_block_editor_plan_preserves_authoring_hooks() {
        let source = r#"
채비 {
  t:변수 <- 0.
}.

(시작)할때 {
  t 보여주기.
}.

(매마디)마다 {
  t <- t + 1.
}.
"#;
        let block_json = wasm_canon_block_editor_plan(source).expect("wasm block plan");
        assert!(block_json.contains("\"kind\": \"charim_block\""));
        assert!(block_json.contains("\"kind\": \"hook_start\""));
        assert!(block_json.contains("\"kind\": \"hook_tick\""));
        assert!(!block_json.contains("\"kind\": \"seed_def\""));
    }

    #[test]
    fn parse_stream_sidecar_tag_supports_ascii_and_ko_suffix() {
        assert_eq!(
            parse_stream_sidecar_tag("energy_stream_head"),
            Some(("energy_stream".to_string(), "head"))
        );
        assert_eq!(
            parse_stream_sidecar_tag("energy_stream_capacity"),
            Some(("energy_stream".to_string(), "capacity"))
        );
        assert_eq!(
            parse_stream_sidecar_tag("가격흐름_길이"),
            Some(("가격흐름".to_string(), "len"))
        );
        assert_eq!(parse_stream_sidecar_tag("plain_value"), None);
    }

    #[test]
    fn extract_ring_buffer_list_merges_sidecar_meta() {
        let value = ResourceValue::List(vec![
            ResourceValue::Fixed64(Fixed64::from_i64(10)),
            ResourceValue::Fixed64(Fixed64::from_i64(20)),
        ]);
        let meta = StreamMeta {
            capacity: Some(4),
            head: Some(1),
            len: Some(2),
        };
        let stream = extract_ring_buffer(&value, Some(&meta)).expect("stream json");
        let obj = stream.as_object().expect("object");
        assert_eq!(obj.get("capacity").and_then(JsonValue::as_u64), Some(4));
        assert_eq!(obj.get("head").and_then(JsonValue::as_u64), Some(1));
        assert_eq!(obj.get("len").and_then(JsonValue::as_u64), Some(2));
        let buffer = obj
            .get("buffer")
            .and_then(JsonValue::as_array)
            .expect("buffer array");
        assert_eq!(buffer.len(), 4);
        assert!(buffer[2].is_null());
        assert!(buffer[3].is_null());
    }

    #[test]
    fn state_hash_is_stable_when_only_view_resource_changes() {
        let mut world = DetNuri::new();
        world
            .world_mut()
            .set_resource_fixed64("경제.자본".to_string(), Fixed64::from_i64(1));
        world
            .world_mut()
            .set_resource_fixed64(BOGAE_WIDTH_TAG.to_string(), Fixed64::from_i64(640));

        let view_prefixes = DEFAULT_VIEW_PREFIXES
            .iter()
            .map(|prefix| (*prefix).to_string())
            .collect::<Vec<_>>();
        let state_hash_a = current_state_hash(&world, &view_prefixes);
        let view_hash_a = hash_json_string(&build_view_meta(world.world()));

        world
            .world_mut()
            .set_resource_fixed64(BOGAE_WIDTH_TAG.to_string(), Fixed64::from_i64(800));
        let state_hash_b = current_state_hash(&world, &view_prefixes);
        let view_hash_b = hash_json_string(&build_view_meta(world.world()));

        assert_eq!(state_hash_a, state_hash_b);
        assert_ne!(view_hash_a, view_hash_b);
    }

    fn point_resource(x: i64, y: i64) -> ResourceValue {
        ResourceValue::List(vec![
            ResourceValue::Fixed64(Fixed64::from_i64(x)),
            ResourceValue::Fixed64(Fixed64::from_i64(y)),
        ])
    }

    fn draw_item_resource(kind: &str) -> ResourceValue {
        ResourceValue::Map(BTreeMap::from([
            (
                "kind".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("kind".to_string()),
                    value: ResourceValue::String(kind.to_string()),
                },
            ),
            (
                "x".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("x".to_string()),
                    value: ResourceValue::Fixed64(Fixed64::from_i64(0)),
                },
            ),
            (
                "y".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("y".to_string()),
                    value: ResourceValue::Fixed64(Fixed64::from_i64(0)),
                },
            ),
            (
                "r".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("r".to_string()),
                    value: ResourceValue::Fixed64(Fixed64::from_i64(1)),
                },
            ),
        ]))
    }

    #[test]
    fn actual_logical_tetris_projects_canonical_rectangles() {
        // Match the existing Windows product stack budget in .cargo/config.toml;
        // do not raise the budget of the separate deep-owner regression tests.
        std::thread::Builder::new().stack_size(16 * 1024 * 1024).spawn(|| {
        use crate::logical_input_tape::{LogicalInputBinding, LogicalInputSession, LogicalSnapshot};
        // The retired game-specific lesson is not an Alpha2+ product entry.
        // Execute the actual shipped D7 source through its logical frontdoor;
        // do not rewrite that source in a harness or invoke retired stdlib.
        let source = include_str!("../../solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.ddn");
        let binding: LogicalInputBinding = serde_json::from_str(include_str!(
            "../../solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.binding.json")).unwrap();
        let binding = LogicalInputSession::bind_source_for_fresh_run(source, binding).unwrap();
        let input = LogicalSnapshot { madi: 0, events: vec![], held: binding.initial_held.clone() };
        let mut session = LogicalInputSession::new(source, binding).unwrap();
        session.step(&input).expect("actual logical tetris execution");
        let before = session.committed_world().state_hash();
        let meta = build_view_meta(session.committed_world());
        let rows = meta["space2d"]["drawlist"].as_array().unwrap();
        // Empty board: one background rectangle plus the first four-cell piece.
        assert_eq!(rows.len(), 1 + 4);
        for row in rows {
            assert_eq!(row["kind"], "rect");
            let x = row["x"].as_f64().unwrap();
            let y = row["y"].as_f64().unwrap();
            assert_eq!(row["x1"], x);
            assert_eq!(row["y1"], y);
            assert_eq!(row["x2"], x + row["w"].as_f64().unwrap());
            assert_eq!(row["y2"], y + row["h"].as_f64().unwrap());
        }
        assert_eq!(session.committed_world().state_hash(), before);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn grid_drawlist_keeps_grid_and_space_projections_without_mutating_world() {
        let mut world = DetNuri::new();
        let cells = (0..64).map(|index| {
            let mut cell = draw_item_resource("rect");
            let ResourceValue::Map(fields) = &mut cell else {
                unreachable!()
            };
            fields.insert("id".to_string(), ResourceMapEntry {
                key: ResourceValue::String("id".to_string()),
                value: ResourceValue::String(format!("격자게임셀_{index}")),
            });
            cell
        }).collect();
        world.world_mut().set_resource_value(BOGAE_DRAWLIST_TAG.to_string(), ResourceValue::List(cells));
        world.world_mut().set_resource_fixed64(BOGAE_WIDTH_TAG.to_string(), Fixed64::from_i64(32));
        world.world_mut().set_resource_fixed64(BOGAE_HEIGHT_TAG.to_string(), Fixed64::from_i64(32));
        let before = world.world().state_hash();
        let meta = build_view_meta(world.world());
        assert_eq!(meta["grid2d"]["schema"], "seamgrim.grid2d.v0");
        assert_eq!(meta["space2d"]["schema"], "seamgrim.space2d.v0");
        assert_eq!(meta["grid2d"]["drawlist"], meta["space2d"]["drawlist"]);
        assert_eq!(meta["space2d"]["drawlist"].as_array().unwrap().len(), 64);
        assert_eq!(meta["space2d"]["camera"]["x_max"], 32.0);
        assert_eq!(meta["primary"]["family"], "grid2d");
        assert_eq!(world.world().state_hash(), before);
    }

    #[test]
    fn build_view_meta_emits_primary_and_secondary_stack() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            BOGAE_DRAWLIST_TAG.to_string(),
            ResourceValue::List(vec![draw_item_resource("circle")]),
        );
        world.world_mut().set_resource_value(
            "그래프_점목록_f".to_string(),
            ResourceValue::List(vec![point_resource(0, 1), point_resource(1, 2)]),
        );

        let view_meta = build_view_meta(world.world());
        let obj = view_meta.as_object().expect("view_meta object");

        assert_eq!(
            obj.get("primary")
                .and_then(JsonValue::as_object)
                .and_then(|row| row.get("family"))
                .and_then(JsonValue::as_str),
            Some("space2d")
        );
        let secondary = obj
            .get("secondary")
            .and_then(JsonValue::as_array)
            .expect("secondary array");
        assert_eq!(secondary.len(), 1);
        assert_eq!(
            secondary[0]
                .as_object()
                .and_then(|row| row.get("family"))
                .and_then(JsonValue::as_str),
            Some("graph")
        );
        assert_eq!(
            obj.get("overlays")
                .and_then(JsonValue::as_array)
                .map(Vec::len),
            Some(0)
        );
    }

    #[test]
    fn build_view_meta_graph_only_sets_graph_primary() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            "그래프_점목록_f".to_string(),
            ResourceValue::List(vec![point_resource(0, 1), point_resource(1, 2)]),
        );

        let view_meta = build_view_meta(world.world());
        let obj = view_meta.as_object().expect("view_meta object");

        assert_eq!(
            obj.get("primary")
                .and_then(JsonValue::as_object)
                .and_then(|row| row.get("family"))
                .and_then(JsonValue::as_str),
            Some("graph")
        );
        assert_eq!(
            obj.get("secondary")
                .and_then(JsonValue::as_array)
                .map(Vec::len),
            Some(0)
        );
    }

    #[test]
    fn build_view_meta_emits_exact_workbench_authoritative_results() {
        let mut world = DetNuri::new();
        for (tag, value) in [
            ("t", 3),
            ("x", 4),
            ("y", 5),
            ("vx", 6),
            ("vy", 7),
            ("a", 2),
            ("b", 1),
            ("theta", 1),
            ("omega", -1),
            ("energy", 9),
            ("kinetic_energy", 4),
            ("potential_energy", 5),
            ("L", 2),
            ("bob_x", 1),
            ("bob_y", -1),
        ] {
            world
                .world_mut()
                .set_resource_fixed64(tag.to_string(), Fixed64::from_i64(value));
        }
        world.world_mut().set_resource_value(
            BOGAE_DRAWLIST_TAG.to_string(),
            ResourceValue::List(vec![draw_item_resource("circle")]),
        );
        world.world_mut().set_resource_value(
            "위치".to_string(),
            ResourceValue::Map(BTreeMap::from([
                (
                    "x".to_string(),
                    ResourceMapEntry {
                        key: ResourceValue::String("x".to_string()),
                        value: ResourceValue::Fixed64(Fixed64::from_i64(3)),
                    },
                ),
                (
                    "y".to_string(),
                    ResourceMapEntry {
                        key: ResourceValue::String("y".to_string()),
                        value: ResourceValue::Fixed64(Fixed64::from_i64(0)),
                    },
                ),
            ])),
        );

        let meta = build_view_meta(world.world());
        let workbench = meta
            .get("workbench")
            .and_then(JsonValue::as_object)
            .expect("workbench result set");
        assert_eq!(
            workbench.get("schema").and_then(JsonValue::as_str),
            Some(WORKBENCH_RESULT_SET_SCHEMA)
        );
        let results = workbench
            .get("results")
            .and_then(JsonValue::as_object)
            .expect("workbench results");
        assert_eq!(
            results.keys().cloned().collect::<Vec<_>>(),
            vec!["grid", "line", "pendulum", "projectile"]
        );
        assert_eq!(
            results["line"].get("schema").and_then(JsonValue::as_str),
            Some(WORKBENCH_LINE_RESULT_SCHEMA)
        );
        assert_eq!(
            results["projectile"]
                .pointer("/content/current/vy")
                .and_then(JsonValue::as_f64),
            Some(7.0)
        );
        assert_eq!(
            results["pendulum"]
                .pointer("/content/current/physical_bob/y")
                .and_then(JsonValue::as_f64),
            Some(-1.0)
        );
        assert_eq!(
            results["grid"]
                .pointer("/content/position/x")
                .and_then(JsonValue::as_f64),
            Some(3.0)
        );
    }

    #[test]
    fn workbench_authoritative_results_are_exact_and_partial_inputs_fail_closed() {
        let mut world = DetNuri::new();
        world
            .world_mut()
            .set_resource_fixed64("x".to_string(), Fixed64::from_i64(1));
        world
            .world_mut()
            .set_resource_fixed64("y".to_string(), Fixed64::from_i64(2));
        world
            .world_mut()
            .set_resource_fixed64("가로속도".to_string(), Fixed64::from_i64(3));
        world
            .world_mut()
            .set_resource_fixed64("세로속도".to_string(), Fixed64::from_i64(4));

        let meta = build_view_meta(world.world());
        assert!(meta.get("workbench").is_none());
    }

    #[test]
    fn serialize_world_resources_emits_value_json_alongside_value_keys() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            "묶음값".to_string(),
            ResourceValue::Map(BTreeMap::from([
                (
                    "x".to_string(),
                    ResourceMapEntry {
                        key: ResourceValue::String("x".to_string()),
                        value: ResourceValue::Fixed64(Fixed64::from_i64(1)),
                    },
                ),
                (
                    "ok".to_string(),
                    ResourceMapEntry {
                        key: ResourceValue::String("ok".to_string()),
                        value: ResourceValue::Bool(true),
                    },
                ),
            ])),
        );

        let resources = serialize_world_resources(world.world());
        let root = resources.as_object().expect("resources object");
        let value = root
            .get("value")
            .and_then(JsonValue::as_object)
            .expect("value map");
        let value_json = root
            .get("value_json")
            .and_then(JsonValue::as_object)
            .expect("value_json map");

        assert!(value.get("묶음값").and_then(JsonValue::as_str).is_some());
        let row = value_json
            .get("묶음값")
            .and_then(JsonValue::as_object)
            .expect("value_json row");
        assert_eq!(row.get("x").and_then(JsonValue::as_f64), Some(1.0));
        assert_eq!(row.get("ok").and_then(JsonValue::as_bool), Some(true));
    }

    #[test]
    fn serialize_patch_set_resource_value_emits_value_json_payload() {
        let mut patch = Patch::default();
        patch.ops.push(PatchOp::SetResourceValue {
            tag: "리스트".to_string(),
            value: ResourceValue::List(vec![
                ResourceValue::Fixed64(Fixed64::from_i64(1)),
                ResourceValue::String("둘".to_string()),
            ]),
        });

        let json = serialize_patch(&patch);
        let rows = json.as_array().expect("patch array");
        assert_eq!(rows.len(), 1);
        let row = rows[0].as_object().expect("patch row");
        assert_eq!(
            row.get("op").and_then(JsonValue::as_str),
            Some("set_resource_value")
        );
        assert_eq!(row.get("tag").and_then(JsonValue::as_str), Some("리스트"));
        let value_json = row
            .get("value_json")
            .and_then(JsonValue::as_array)
            .expect("value_json array");
        assert_eq!(value_json.len(), 2);
        assert_eq!(value_json[0].as_f64(), Some(1.0));
        assert_eq!(value_json[1].as_str(), Some("둘"));
    }

    #[test]
    fn wo09_shared_patch_projection_preserves_both_payload_paths() {
        let mut patch = Patch::default();
        for length in 0..4 {
            patch.ops.push(PatchOp::SetResourceValue {
                tag: "출력".into(),
                value: ResourceValue::List((0..length)
                    .map(|n| ResourceValue::String(format!("행{n}")))
                    .collect()),
            });
        }
        let original = json!({
            "patch": serialize_patch(&patch),
            "state": {"patch": serialize_patch(&patch)},
        });
        let projected = serialize_patch(&patch);
        let shared = json!({
            "patch": projected.clone(),
            "state": {"patch": projected},
        });
        assert_eq!(original.to_string(), shared.to_string());
        assert_eq!(shared["patch"].as_array().unwrap().len(), 4);
        assert_eq!(shared["state"]["patch"], shared["patch"]);
    }

    #[test]
    fn restore_hash_diags_detect_mismatch_codes() {
        let payload = json!({
            "state_hash": "blake3:expected-state",
            "view_hash": "blake3:expected-view",
        });
        let diags = restore_hash_diags(&payload, "blake3:actual-state", "blake3:actual-view");
        assert_eq!(diags.len(), 2);
        let codes = diags
            .iter()
            .filter_map(|diag| diag.get("code").and_then(JsonValue::as_str))
            .collect::<Vec<_>>();
        assert!(codes.contains(&"STATE_HASH_MISMATCH"));
        assert!(codes.contains(&"VIEW_HASH_MISMATCH"));
    }

    #[test]
    fn resource_value_to_draw_item_maps_layer_index_aliases() {
        let draw_item = ResourceValue::Map(BTreeMap::from([
            (
                "kind".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("kind".to_string()),
                    value: ResourceValue::String("polygon".to_string()),
                },
            ),
            (
                "레이어".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("레이어".to_string()),
                    value: ResourceValue::Fixed64(Fixed64::from_i64(3)),
                },
            ),
            (
                "fill".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("fill".to_string()),
                    value: ResourceValue::String("#f59e0b".to_string()),
                },
            ),
        ]));
        let json = resource_value_to_draw_item(&draw_item).expect("draw item json");
        let obj = json.as_object().expect("draw item object");
        assert_eq!(obj.get("kind").and_then(JsonValue::as_str), Some("polygon"));
        assert_eq!(
            obj.get("layer_index").and_then(JsonValue::as_f64),
            Some(3.0)
        );
        assert_eq!(obj.get("fill").and_then(JsonValue::as_str), Some("#f59e0b"));
    }

    #[test]
    fn resource_value_to_draw_item_maps_group_id_aliases() {
        let draw_item = ResourceValue::Map(BTreeMap::from([
            (
                "kind".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("kind".to_string()),
                    value: ResourceValue::String("polyline".to_string()),
                },
            ),
            (
                "그룹".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("그룹".to_string()),
                    value: ResourceValue::String("pendulum.path".to_string()),
                },
            ),
            (
                "stroke".to_string(),
                ResourceMapEntry {
                    key: ResourceValue::String("stroke".to_string()),
                    value: ResourceValue::String("#22c55e".to_string()),
                },
            ),
        ]));
        let json = resource_value_to_draw_item(&draw_item).expect("draw item json");
        let obj = json.as_object().expect("draw item object");
        assert_eq!(
            obj.get("kind").and_then(JsonValue::as_str),
            Some("polyline")
        );
        assert_eq!(
            obj.get("group_id").and_then(JsonValue::as_str),
            Some("pendulum.path")
        );
        assert_eq!(
            obj.get("stroke").and_then(JsonValue::as_str),
            Some("#22c55e")
        );
    }

    #[test]
    fn bogae_kind_mapping_requires_an_exact_existing_identity() {
        assert_eq!(map_bogae_kind("선"), "line");
        assert_eq!(map_bogae_kind("사각형"), "rect");
        assert_eq!(map_bogae_kind("다각형"), "polygon");
        assert_eq!(map_bogae_kind("화살표"), "arrow");
        assert_eq!(map_bogae_kind("#보개/2D.Rect"), "rect");

        for unsupported in [
            "prefix-line",
            "line-suffix",
            "LINE",
            "사각형처럼",
            "원형",
            "sprite",
            "#보개/2D.Rect.extra",
            "#보개/2d.rect",
            "",
        ] {
            assert_eq!(map_bogae_kind(unsupported), unsupported);
        }
    }

    #[test]
    fn unit_value_wire_v2_is_explicit_and_roundtrips_ninth_axis() {
        let value = ResourceValue::Unit(ddonirang_core::UnitValue::new(
            Fixed64::from_i64(2),
            ddonirang_core::Unit::Mole,
        ));
        let encoded = resource_value_to_detjson(&value);
        assert_eq!(encoded["schema"], UNIT_VALUE_WIRE_SCHEMA_V2);
        assert_eq!(
            encoded["dimension"]["schema"],
            ddonirang_core::UnitDim::WIRE_SCHEMA_V2
        );
        assert_eq!(
            encoded["dimension"]["axis_order"],
            json!(ddonirang_core::UnitDim::AXIS_ORDER_V2)
        );
        assert_eq!(
            encoded["dimension"]["exponents"],
            json!([0, 0, 0, 0, 0, 0, 0, 0, 1])
        );
        assert_eq!(
            resource_value_from_detjson(&encoded).expect("v2 roundtrip"),
            value
        );
    }

    #[test]
    fn wasm_snapshot_number_admission_matches_checked_textual_fixed64_boundary() {
        let three_halves_ulp: JsonValue =
            serde_json::from_str("0.000000000349245965480804443359375")
                .expect("three halves ULP JSON number");
        let overflow: JsonValue = serde_json::from_str("2147483648")
            .expect("out-of-range JSON number");

        assert_eq!(
            json_to_fixed64(&three_halves_ulp),
            None
        );
        assert_eq!(json_to_fixed64(&overflow), None);
        assert_eq!(
            json_to_fixed64(&json!("0.000000000349245965480804443359375")),
            Some(Fixed64::from_raw_i64(2))
        );
    }

    #[test]
    fn unit_value_wire_v2_rejects_v1_shape_unknown_version_and_axis_mismatch() {
        let legacy = json!({
            "type": "unit",
            "raw_i64": "2000000",
            "dimension": [0, 0, 0, 0, 0, 0, 0, 0],
        });
        assert!(resource_value_from_detjson(&legacy)
            .expect_err("unversioned v1 must fail")
            .contains("E_UNIT_WIRE_VERSION_MISMATCH"));

        let mut unknown_version = resource_value_to_detjson(&ResourceValue::Unit(
            ddonirang_core::UnitValue::new(Fixed64::from_i64(2), ddonirang_core::Unit::Mole),
        ));
        unknown_version["schema"] = json!("ddn.unit_value_wire.v3");
        assert!(resource_value_from_detjson(&unknown_version)
            .expect_err("unknown version must fail")
            .contains("E_UNIT_WIRE_VERSION_MISMATCH"));

        let mut reordered = resource_value_to_detjson(&ResourceValue::Unit(
            ddonirang_core::UnitValue::new(Fixed64::from_i64(2), ddonirang_core::Unit::Mole),
        ));
        reordered["dimension"]["axis_order"][0] = json!("time");
        reordered["dimension"]["axis_order"][1] = json!("length");
        assert!(resource_value_from_detjson(&reordered)
            .expect_err("axis reorder must fail")
            .contains("E_UNIT_DIMENSION_AXIS_ORDER_MISMATCH"));

        let mut missing_axis = resource_value_to_detjson(&ResourceValue::Unit(
            ddonirang_core::UnitValue::new(Fixed64::from_i64(2), ddonirang_core::Unit::Mole),
        ));
        missing_axis["dimension"]["exponents"]
            .as_array_mut()
            .expect("exponents")
            .pop();
        assert!(resource_value_from_detjson(&missing_axis)
            .expect_err("missing ninth axis must fail")
            .contains("E_UNIT_DIMENSION_AXIS_COUNT_MISMATCH"));
    }

    #[test]
    fn resource_snapshot_v2_rejects_checkpoint_unit_wire_mismatch() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            "물질량".to_string(),
            ResourceValue::Unit(ddonirang_core::UnitValue::new(
                Fixed64::from_i64(2),
                ddonirang_core::Unit::Mole,
            )),
        );
        let encoded = serialize_world_resources_snapshot(world.world());
        assert_eq!(encoded["schema"], RESOURCE_SNAPSHOT_SCHEMA_V2);
        assert_eq!(encoded["unit_wire_schema"], UNIT_VALUE_WIRE_SCHEMA_V2);

        let mut mismatch = encoded.clone();
        mismatch["unit_wire_schema"] = json!("ddn.unit_value_wire.v1");
        let mut restored = DetNuri::new();
        assert!(restore_world_resources(restored.world_mut(), &mismatch)
            .expect_err("checkpoint mismatch must fail")
            .contains("E_UNIT_WIRE_CHECKPOINT_MISMATCH"));
    }

    fn wasm_restore_payload(resources: JsonValue) -> JsonValue {
        json!({
            "schema": ENGINE_RESPONSE_SCHEMA,
            "state": {
                "input": {
                    "keys_pressed": 1,
                    "last_key_name": "m",
                    "pointer_x_i32": 2,
                    "pointer_y_i32": 3,
                    "rng_base_seed": 4,
                    // 입력 기록 identity 변경: 옛 raw/text 혼용이 아닌 정확 정본 기록.
                    // 이 fixture의 목적(restore 원자성/자원 보존)은 유지한다.
                    "input_encoding": crate::input_receipt::SNAPSHOT_ENCODING,
                    "dt": "1",
                },
                "snapshot_v2": build_wasm_state_snapshot_v2(
                    resources,
                    json!({"속도": {"raw_i64": "2"}}),
                    // This unit fixture models seven committed boundaries.
                    // It is not execution/replay evidence; an empty history
                    // cannot legitimately carry next_tick_id=7.
                    (0..7).map(|madi| json!({
                        "committed_madi": madi,
                        "state_hash": format!("unit-fixture-state-{madi}"),
                        "input_snapshot": null,
                        "public_observations": [],
                    })).collect(),
                    7,
                    crate::bogae_time::SceneHistory::default().snapshot(),
                ),
            },
        })
    }

    #[test]
    fn wasm_state_snapshot_v2_roundtrips_ninth_axis_unit_exactly() {
        let mut world = DetNuri::new();
        world.world_mut().set_resource_value(
            "물질량".to_string(),
            ResourceValue::Unit(ddonirang_core::UnitValue::new(
                Fixed64::from_i64(2),
                ddonirang_core::Unit::Mole,
            )),
        );
        let resources = serialize_world_resources_snapshot(world.world());
        let payload = wasm_restore_payload(resources.clone());
        let staged = stage_wasm_state_restore(&payload, "app/main.ddn", "source", "canon")
            .expect("v2 state snapshot");

        assert_eq!(
            serialize_world_resources_snapshot(staged.world.world()),
            resources
        );
        assert_eq!(staged.tick_id, 7);
        assert_eq!(
            serialize_world_resources_snapshot(staged.world.world())["value_det"]["물질량"]
                ["dimension"]["exponents"],
            json!([0, 0, 0, 0, 0, 0, 0, 0, 1])
        );
        assert_eq!(
            serialize_world_resources_snapshot(staged.world.world())["value_det"]["물질량"]
                ["raw_i64"],
            json!(Fixed64::from_i64(2).raw_i64().to_string())
        );
    }

    #[test]
    fn wasm_restore_fixture_never_accepts_unbacked_madi_cursor() {
        let resources = serialize_world_resources_snapshot(DetNuri::new().world());
        let mut payload = wasm_restore_payload(resources);
        payload["state"]["snapshot_v2"]["public_observation_frames"] = json!([]);
        assert!(stage_wasm_state_restore(&payload, "app/main.ddn", "source", "canon")
            .err().unwrap().starts_with("E_WASM_STATE_TICK_INVALID:"));
    }

    #[test]
    fn wasm_state_snapshot_v2_rejects_old_missing_and_mixed_versions() {
        let resources = serialize_world_resources_snapshot(DetNuri::new().world());
        let mut old = wasm_restore_payload(resources.clone());
        old["state"]["snapshot_v1"] = old["state"]["snapshot_v2"].take();
        assert!(
            stage_wasm_state_restore(&old, "app/main.ddn", "source", "canon")
                .err()
                .expect("v1 must fail")
                .contains("E_WASM_STATE_SNAPSHOT_MIXED_VERSION")
        );

        let mut missing = wasm_restore_payload(resources.clone());
        missing["state"]
            .as_object_mut()
            .expect("state")
            .remove("snapshot_v2");
        assert!(
            stage_wasm_state_restore(&missing, "app/main.ddn", "source", "canon")
                .err()
                .expect("missing snapshot must fail")
                .contains("E_WASM_STATE_SNAPSHOT_VERSION_MISMATCH")
        );

        let mut unknown = wasm_restore_payload(resources);
        unknown["state"]["snapshot_v2"]["schema"] = json!("ddn.wasm_state_snapshot.v3");
        assert!(
            stage_wasm_state_restore(&unknown, "app/main.ddn", "source", "canon")
                .err()
                .expect("unknown snapshot must fail")
                .contains("E_WASM_STATE_SNAPSHOT_VERSION_MISMATCH")
        );
    }

    #[test]
    fn failed_staged_restore_leaves_existing_world_unchanged() {
        let mut current = DetNuri::new();
        current.world_mut().set_resource_value(
            "기존".to_string(),
            ResourceValue::Fixed64(Fixed64::from_i64(9)),
        );
        let before = serialize_world_resources_snapshot(current.world());
        let before_hash = current
            .world()
            .state_hash_excluding_resource_prefixes(&[])
            .to_hex()
            .to_string();

        let mut corrupt = serialize_world_resources_snapshot(DetNuri::new().world());
        corrupt["fixed64"]["먼저_유효"] = json!({"raw_i64": "1000000"});
        corrupt["fixed64"]["뒤에_손상"] = json!({"raw_i64": "not-an-i64"});
        let payload = wasm_restore_payload(corrupt);
        assert!(
            stage_wasm_state_restore(&payload, "app/main.ddn", "source", "canon")
                .err()
                .expect("corrupt later entry must fail")
                .contains("fixed64[뒤에_손상]")
        );

        assert_eq!(serialize_world_resources_snapshot(current.world()), before);
        assert_eq!(
            current
                .world()
                .state_hash_excluding_resource_prefixes(&[])
                .to_hex()
                .to_string(),
            before_hash
        );
    }

    #[test]
    fn snapshot_param_overrides_use_the_shared_fixed64_raw_carrier() {
        let exact = Fixed64::from_raw_i64(2);
        let params = BTreeMap::from([
            ("고정".to_string(), Value::Fixed64(exact)),
            ("글".to_string(), Value::String("2".to_string())),
        ]);

        let serialized = serialize_param_overrides(&params);
        assert_eq!(serialized["고정"], json!({"raw_i64": "2"}));
        assert_eq!(serialized["글"], json!("2"));
        assert_eq!(parse_param_overrides(&serialized).expect("raw carrier restore"), params);

        let binary_number = parse_param_overrides(&json!({"고정": 2}));
        assert!(
            binary_number
                .err()
                .expect("binary JSON number must fail closed")
                .contains("E_WASM_STATE_PARAMS_INVALID")
        );
    }
}

fn serialize_param_overrides(params: &BTreeMap<String, Value>) -> JsonValue {
    let mut out = Map::new();
    for (key, value) in params {
        let raw = match value {
            Value::Fixed64(v) => json!({ "raw_i64": v.raw_i64().to_string() }),
            Value::Bool(v) => JsonValue::Bool(*v),
            Value::String(v) => JsonValue::String(v.clone()),
            _ => continue,
        };
        out.insert(key.clone(), raw);
    }
    JsonValue::Object(out)
}

fn parse_param_overrides(payload: &JsonValue) -> Result<BTreeMap<String, Value>, String> {
    crate::unit_wire::parse_param_overrides(payload)
}

fn serialize_world_resources_snapshot(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();

    out.insert(
        "schema".to_string(),
        JsonValue::String(RESOURCE_SNAPSHOT_SCHEMA_V2.to_string()),
    );
    out.insert(
        "unit_wire_schema".to_string(),
        JsonValue::String(UNIT_VALUE_WIRE_SCHEMA_V2.to_string()),
    );
    out.insert("next_entity".to_string(), json!(world.next_entity_cursor()));
    out.insert(
        "components".to_string(),
        JsonValue::Array(
            world
                .component_json_entries()
                .into_iter()
                .map(
                    |(entity, tag, value)| json!({"entity": entity.0, "tag": tag.0, "json": value}),
                )
                .collect(),
        ),
    );

    let mut json_map = Map::new();
    for (tag, value) in world.resource_json_entries() {
        json_map.insert(tag, JsonValue::String(value));
    }
    out.insert("json".to_string(), JsonValue::Object(json_map));

    let mut fixed_map = Map::new();
    for (tag, value) in world.resource_fixed64_entries() {
        fixed_map.insert(tag, json!({"raw_i64": value.raw_i64().to_string()}));
    }
    out.insert("fixed64".to_string(), JsonValue::Object(fixed_map));

    let mut handle_map = Map::new();
    for (tag, handle) in world.resource_handle_entries() {
        handle_map.insert(tag, JsonValue::String(handle_to_string(handle)));
    }
    out.insert("handle".to_string(), JsonValue::Object(handle_map));

    let mut value_det = Map::new();
    for (tag, value) in world.resource_value_entries() {
        value_det.insert(tag, resource_value_to_detjson(&value));
    }
    out.insert("value_det".to_string(), JsonValue::Object(value_det));

    JsonValue::Object(out)
}

fn restore_world_resources(
    world: &mut ddonirang_core::platform::NuriWorld,
    resources: &JsonValue,
) -> Result<(), String> {
    crate::unit_wire::restore_world_resources_snapshot(world,resources)
}

fn parse_handle_string(text: &str) -> Result<u64, String> {
    let trimmed = text.trim();
    let without_handle = trimmed.strip_prefix("handle:").unwrap_or(trimmed);
    let hex = without_handle
        .strip_prefix("자원#")
        .unwrap_or(without_handle);
    u64::from_str_radix(hex, 16).map_err(|_| format!("handle 파싱 실패: {text}"))
}

fn resource_value_to_detjson(value: &ResourceValue) -> JsonValue {
    crate::unit_wire::resource_value_to_detjson(value)
}

fn resource_value_from_detjson(payload: &JsonValue) -> Result<ResourceValue, String> {
    crate::unit_wire::resource_value_from_detjson(payload)
}

fn json_to_u64(value: &JsonValue) -> Option<u64> {
    match value {
        JsonValue::Number(n) => n.as_u64(),
        JsonValue::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}

fn json_to_i32(value: &JsonValue) -> Option<i32> {
    match value {
        JsonValue::Number(n) => n.as_i64().and_then(|v| i32::try_from(v).ok()),
        JsonValue::String(s) => s.parse::<i32>().ok(),
        _ => None,
    }
}

fn serialize_world_resources(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();
    let mut value_json_map = Map::new();

    let mut json_map = Map::new();
    for (tag, json) in world.resource_json_entries() {
        value_json_map.insert(tag.clone(), resource_json_text_to_json(&json));
        json_map.insert(tag, JsonValue::String(json));
    }
    out.insert("json".to_string(), JsonValue::Object(json_map));

    let mut fixed_map = Map::new();
    for (tag, value) in world.resource_fixed64_entries() {
        value_json_map.insert(tag.clone(), json!(fixed64_to_f64(value)));
        fixed_map.insert(tag, JsonValue::String(value.display_rounded_six()));
    }
    out.insert("fixed64".to_string(), JsonValue::Object(fixed_map));

    let mut handle_map = Map::new();
    for (tag, handle) in world.resource_handle_entries() {
        handle_map.insert(tag, JsonValue::String(handle_to_string(handle)));
    }
    out.insert("handle".to_string(), JsonValue::Object(handle_map));

    let mut value_map = Map::new();
    for (tag, value) in world.resource_value_entries() {
        value_map.insert(tag.clone(), JsonValue::String(value.canon_key()));
        value_json_map.insert(tag, resource_value_to_json(&value));
    }
    out.insert("value".to_string(), JsonValue::Object(value_map));
    out.insert("value_json".to_string(), JsonValue::Object(value_json_map));

    JsonValue::Object(out)
}

fn resource_json_text_to_json(text: &str) -> JsonValue {
    serde_json::from_str::<JsonValue>(text).unwrap_or_else(|_| JsonValue::String(text.to_string()))
}

fn input_source_json(source: InputSource) -> JsonValue {
    json!({
        "label": source.label(),
        "code": source.code_u8(),
    })
}

fn seulgi_intent_json(intent: &SeulgiIntent) -> JsonValue {
    match intent {
        SeulgiIntent::None => json!({
            "kind": "none",
            "kind_code": intent.kind_u32(),
        }),
        SeulgiIntent::MoveTo { x, y } => json!({
            "kind": "move_to",
            "kind_code": intent.kind_u32(),
            "x": x.to_string(),
            "y": y.to_string(),
            "x_raw_i64": x.raw_i64(),
            "y_raw_i64": y.raw_i64(),
        }),
        SeulgiIntent::Attack { target_id } => json!({
            "kind": "attack",
            "kind_code": intent.kind_u32(),
            "target_id": target_id,
        }),
        SeulgiIntent::Say { text } => json!({
            "kind": "say",
            "kind_code": intent.kind_u32(),
            "text": text.as_str(),
        }),
    }
}

fn serialize_input_snapshot(input: &InputSnapshot, rng_base_seed: Option<u64>) -> JsonValue {
    runtime_surface::serialize_input_snapshot(input, rng_base_seed)
}

fn serialize_input_state(vm: &DdnWasmVm) -> JsonValue {
    json!({
        "alpha2_host_madi_hz": crate::input_receipt::alpha2_host_madi_hz(),
        "alpha_host_dt": crate::input_receipt::exact_dt_text(crate::input_receipt::alpha_host_dt()),
        "keys_pressed": vm.input_keys_pressed,
        "last_key_name": vm.input_last_key_name,
        "pointer_x_i32": vm.input_pointer_x_i32,
        "pointer_y_i32": vm.input_pointer_y_i32,
        "input_encoding": crate::input_receipt::SNAPSHOT_ENCODING,
        "dt": crate::input_receipt::exact_dt_text(vm.input_dt),
        "frame_source": InputSource::Person.label(),
        "frame_source_code": InputSource::Person.code_u8(),
        "rng_seed": vm.rng_seed,
        "rng_base_seed": vm.rng_seed,
        "pending_ai_injection_count": vm.pending_ai_injections.len(),
    })
}

fn serialize_patch(patch: &Patch) -> JsonValue {
    let mut items = Vec::new();
    for op in &patch.ops {
        match op {
            PatchOp::SetResourceJson { tag, json } => {
                items.push(json!({ "op": "set_resource_json", "tag": tag, "value": json }));
            }
            PatchOp::SetResourceFixed64 { tag, value } => {
                items.push(
                    json!({ "op": "set_resource_fixed64", "tag": tag, "value": value.to_string() }),
                );
            }
            PatchOp::SetResourceHandle { tag, handle } => {
                items.push(json!({ "op": "set_resource_handle", "tag": tag, "value": handle_to_string(*handle) }));
            }
            PatchOp::SetResourceValue { tag, value } => {
                items.push(crate::runtime_surface::resource_value_patch_json(tag, value));
            }
            PatchOp::SetSubjectWorld { value } => {
                items.push(json!({"op":"set_subject_world",
                    "subject_values_sha256":runtime_surface::sha256_hex(&value.subject_values),
                    "subject_custody_sha256":runtime_surface::sha256_hex(&value.subject_custody)}));
            }
            PatchOp::DivAssignResourceFixed64 { tag, rhs, .. } => {
                items.push(json!({ "op": "div_assign_resource_fixed64", "tag": tag, "value": rhs.to_string() }));
            }
            PatchOp::SetComponentJson { entity, tag, json } => {
                items.push(json!({ "op": "set_component_json", "entity": entity.0, "tag": tag.0, "value": json }));
            }
            PatchOp::RemoveComponent { entity, tag } => {
                items.push(json!({ "op": "remove_component", "entity": entity.0, "tag": tag.0 }));
            }
            PatchOp::EmitSignal { signal, targets } => {
                items.push(
                    json!({ "op": "emit_signal", "signal": signal.name(), "targets": targets }),
                );
            }
            PatchOp::GuardViolation { entity, rule_id } => {
                items.push(
                    json!({ "op": "guard_violation", "entity": entity.0, "rule_id": rule_id }),
                );
            }
        }
    }
    JsonValue::Array(items)
}

fn collect_columns_and_row(
    world: &ddonirang_core::platform::NuriWorld,
) -> (Vec<JsonValue>, Vec<JsonValue>) {
    let mut table: BTreeMap<String, (String, JsonValue)> = BTreeMap::new();

    for (tag, value) in world.resource_fixed64_entries() {
        table.insert(tag, ("num".to_string(), json!(fixed64_to_f64(value))));
    }

    for (tag, value) in world.resource_json_entries() {
        if let Some((dtype, scalar)) = json_text_to_scalar(&value) {
            table.insert(tag, (dtype, scalar));
        }
    }

    for (tag, value) in world.resource_value_entries() {
        if let Some((dtype, scalar)) = resource_value_to_scalar(&value) {
            table.insert(tag, (dtype, scalar));
        }
    }

    let mut columns = Vec::with_capacity(table.len());
    let mut row = Vec::with_capacity(table.len());
    for (key, (dtype, value)) in table {
        columns.push(json!({
            "key": key,
            "dtype": dtype,
            "role": "state",
        }));
        row.push(value);
    }
    (columns, row)
}

fn json_text_to_scalar(text: &str) -> Option<(String, JsonValue)> {
    let trimmed = text.trim();
    if trimmed == "참" {
        return Some(("bool".to_string(), JsonValue::Bool(true)));
    }
    if trimmed == "거짓" {
        return Some(("bool".to_string(), JsonValue::Bool(false)));
    }
    if let Ok(parsed) = serde_json::from_str::<JsonValue>(trimmed) {
        return match parsed {
            JsonValue::Bool(v) => Some(("bool".to_string(), JsonValue::Bool(v))),
            JsonValue::Number(v) => Some(("num".to_string(), JsonValue::Number(v))),
            JsonValue::String(v) => Some(("str".to_string(), JsonValue::String(v))),
            _ => None,
        };
    }
    if let Ok(number) = trimmed.parse::<f64>() {
        if number.is_finite() {
            return Some(("num".to_string(), json!(number)));
        }
    }
    Some(("str".to_string(), JsonValue::String(text.to_string())))
}

fn resource_value_to_scalar(value: &ResourceValue) -> Option<(String, JsonValue)> {
    match value {
        ResourceValue::Bool(v) => Some(("bool".to_string(), JsonValue::Bool(*v))),
        ResourceValue::Fixed64(v) => Some(("num".to_string(), json!(fixed64_to_f64(*v)))),
        ResourceValue::Unit(v) => Some(("num".to_string(), json!(fixed64_to_f64(v.value)))),
        ResourceValue::String(v) => Some(("str".to_string(), JsonValue::String(v.clone()))),
        _ => None,
    }
}

fn js_scalar_to_runtime_value(value: &JsValue) -> Option<Value> {
    if value.as_f64().is_some() {
        return None;
    }
    if let Some(v) = value.as_bool() {
        return Some(Value::Bool(v));
    }
    value.as_string().map(Value::String)
}

fn apply_param_value(
    world: &mut ddonirang_core::platform::NuriWorld,
    key: &str,
    value: &Value,
) -> Result<(), String> {
    world.validate_typed_resource_lanes().map_err(str::to_string)?;
    let fixed=matches!(value,Value::Fixed64(_));
    if world.get_resource_handle(key).is_some()
        || (fixed && world.get_resource_value(key).is_some())
        || (!fixed && world.get_resource_fixed64(key).is_some()) {
        return Err("같은 자원 이름이 여러 저장 통로에 있습니다".into());
    }
    match value {
        Value::Fixed64(v) => {
            world.set_resource_fixed64(key.to_string(), *v);
            Ok(())
        }
        Value::Bool(v) => {
            world.set_resource_value(key.to_string(), ResourceValue::Bool(*v));
            Ok(())
        }
        Value::String(v) => {
            world.set_resource_value(key.to_string(), ResourceValue::String(v.clone()));
            Ok(())
        }
        _ => Err("set_param: 수/참거짓/글 스칼라만 허용됩니다".to_string()),
    }
}

fn handle_to_string(handle: ResourceHandle) -> String {
    format!("handle:{}", handle.to_hex())
}

fn seed_bogae_defaults(defaults: &mut HashMap<String, Value>) {
    defaults.insert(BOGAE_DRAWLIST_TAG.to_string(), Value::List(Vec::new()));
    defaults.insert(BOGAE_SHOW_LINES_TAG.to_string(), Value::List(Vec::new()));
    defaults.insert(
        BOGAE_WIDTH_TAG.to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert(
        BOGAE_HEIGHT_TAG.to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert("프레임수".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
    defaults.insert(
        "__wasm_start_once".to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert("회전_x".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
    defaults.insert("회전_y".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
}

fn derive_space2d_from_bogae(world: &ddonirang_core::platform::NuriWorld) -> Option<String> {
    let list = world.get_resource_value(BOGAE_DRAWLIST_TAG)?;
    let ResourceValue::List(items) = &list else {
        return None;
    };
    let mut draw_items = Vec::new();
    for item in items {
        if let Some(obj) = resource_value_to_draw_item(item) {
            draw_items.push(obj);
        }
    }
    if draw_items.is_empty() {
        return None;
    }
    let mut space2d = Map::new();
    space2d.insert(
        "schema".to_string(),
        JsonValue::String(SPACE2D_SCHEMA.to_string()),
    );
    space2d.insert("drawlist".to_string(), JsonValue::Array(draw_items));
    if let (Some(w), Some(h)) = (
        world.get_resource_fixed64(BOGAE_WIDTH_TAG),
        world.get_resource_fixed64(BOGAE_HEIGHT_TAG),
    ) {
        let w = fixed64_to_f64(w);
        let h = fixed64_to_f64(h);
        if w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 {
            space2d.insert(
                "camera".to_string(),
                json!({
                    "x_min": 0.0,
                    "x_max": w,
                    "y_min": 0.0,
                    "y_max": h,
                }),
            );
        }
    }
    Some(JsonValue::Object(space2d).to_string())
}

fn derive_grid2d_from_bogae(world: &ddonirang_core::platform::NuriWorld) -> Option<String> {
    let list = world.get_resource_value(BOGAE_DRAWLIST_TAG)?;
    let ResourceValue::List(items) = &list else {
        return None;
    };
    let draw_items: Vec<JsonValue> = items
        .iter()
        .filter_map(resource_value_to_draw_item)
        .collect();
    if draw_items.is_empty()
        || !draw_items.iter().all(|item| {
            item.get("id")
                .and_then(JsonValue::as_str)
                .is_some_and(|id| id.starts_with("격자게임셀_") || id.starts_with("cell."))
        })
    {
        return None;
    }
    Some(
        json!({
            "schema": "seamgrim.grid2d.v0",
            "drawlist": draw_items,
            "source": "std_grid_game_bogae_drawlist"
        })
        .to_string(),
    )
}

fn points_from_resource_value(value: &ResourceValue) -> Option<Vec<(f64, f64)>> {
    let ResourceValue::List(items) = value else {
        return None;
    };
    let mut out = Vec::new();
    for item in items {
        if let Some(point) = resource_value_to_point(item) {
            out.push(point);
        }
    }
    Some(out)
}

fn resource_value_to_point(value: &ResourceValue) -> Option<(f64, f64)> {
    match value {
        ResourceValue::Map(entries) => {
            let mut x = None;
            let mut y = None;
            for entry in entries.values() {
                let key = resource_key_to_string(&entry.key);
                if key == "x" {
                    x = resource_value_to_number(&entry.value);
                } else if key == "y" {
                    y = resource_value_to_number(&entry.value);
                }
            }
            match (x, y) {
                (Some(x), Some(y)) => Some((x, y)),
                _ => None,
            }
        }
        ResourceValue::List(items) => {
            if items.len() < 2 {
                return None;
            }
            Some((
                resource_value_to_number(&items[0])?,
                resource_value_to_number(&items[1])?,
            ))
        }
        _ => None,
    }
}

fn resource_value_to_number(value: &ResourceValue) -> Option<f64> {
    match value {
        ResourceValue::Fixed64(v) => Some(fixed64_to_f64(*v)),
        ResourceValue::Unit(v) => Some(fixed64_to_f64(v.value)),
        ResourceValue::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
        ResourceValue::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn resource_value_to_draw_item(value: &ResourceValue) -> Option<JsonValue> {
    let ResourceValue::Map(entries) = value else {
        return None;
    };
    let mut obj = Map::new();
    let mut kind_raw: Option<String> = None;
    for entry in entries.values() {
        let key = resource_key_to_string(&entry.key);
        let json_value = resource_value_to_json(&entry.value);
        match key.as_str() {
            "결" | "kind" | "shape" | "도형" | "형태" => {
                kind_raw = Some(resource_value_to_string(&entry.value));
            }
            "채움색" | "채움" => {
                obj.insert("fill".to_string(), json_value);
            }
            "색" | "선색" => {
                obj.insert("color".to_string(), json_value);
            }
            "굵기" => {
                obj.insert("width".to_string(), json_value);
            }
            "크기" => {
                obj.insert("size".to_string(), json_value);
            }
            "글" | "내용" => {
                obj.insert("text".to_string(), json_value);
            }
            "층" | "레이어" | "layer" | "layerIndex" | "z" | "z_index" => {
                obj.insert("layer_index".to_string(), json_value);
            }
            "그룹" | "묶음" | "group" | "groupId" => {
                obj.insert("group_id".to_string(), json_value);
            }
            _ => {
                obj.insert(key, json_value);
            }
        }
    }
    if let Some(raw) = kind_raw {
        obj.insert("kind".to_string(), JsonValue::String(map_bogae_kind(&raw)));
        // 기존 격자 생산자의 정확한 Rect 식별자만 표준 꼭짓점으로 투영한다.
        // 세계의 x/y/w/h 값은 변경하지 않고, 잘못된 값은 그대로 거부된다.
        if raw == "#보개/2D.Rect" {
            if let (Some(x), Some(y), Some(w), Some(h)) = (
                obj.get("x").and_then(JsonValue::as_f64),
                obj.get("y").and_then(JsonValue::as_f64),
                obj.get("w").and_then(JsonValue::as_f64),
                obj.get("h").and_then(JsonValue::as_f64),
            ) {
                if [x, y, w, h, x + w, y + h].iter().all(|n| n.is_finite()) {
                    obj.insert("x1".to_string(), json!(x));
                    obj.insert("y1".to_string(), json!(y));
                    obj.insert("x2".to_string(), json!(x + w));
                    obj.insert("y2".to_string(), json!(y + h));
                }
            }
        }
    }
    Some(JsonValue::Object(obj))
}

fn resource_key_to_string(value: &ResourceValue) -> String {
    match value {
        ResourceValue::String(s) => s.clone(),
        _ => value.canon_key(),
    }
}

fn resource_value_to_string(value: &ResourceValue) -> String {
    match value {
        ResourceValue::String(s) => s.clone(),
        _ => value.canon_key(),
    }
}

fn resource_value_to_json(value: &ResourceValue) -> JsonValue {
    crate::runtime_surface::resource_value_to_json(value)
}

fn map_bogae_kind(raw: &str) -> String {
    // A draw kind is an existing typed projection identity.  Do not infer it
    // from a display label, case-folded spelling, or a substring: unknown
    // values are deliberately preserved for the projection boundary to reject.
    match raw {
        "rect" | "사각형" | "#보개/2D.Rect" => "rect".to_string(),
        "text" | "글" => "text".to_string(),
        "circle" | "원" => "circle".to_string(),
        "polygon" | "다각형" => "polygon".to_string(),
        "polyline" | "선들" => "polyline".to_string(),
        "line" | "선" => "line".to_string(),
        "point" | "점" => "point".to_string(),
        "arrow" | "화살표" => "arrow".to_string(),
        _ => raw.to_string(),
    }
}

fn fixed64_to_f64(value: Fixed64) -> f64 {
    value.raw_i64() as f64 / Fixed64::ONE_RAW as f64
}
