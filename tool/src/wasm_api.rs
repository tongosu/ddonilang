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
        let mut session = crate::logical_input_tape::LogicalInputSession::new(source, binding)
            .map_err(|reason| JsValue::from_str(&reason))?;
        let recording=crate::input_tape_v3::LiveRecording::for_logical(&mut session).map_err(|reason|JsValue::from_str(&reason))?;
        Ok(Self { execution:LogicalVmExecution::SingleFile(session),recording })
    }

    /// Explicit project admission preserves modules and the named Start.
    /// No linked-text fallback or durable browser receipt is manufactured.
    pub fn from_authored_project(authored: &str,binding_json: &str)->Result<DdnLogicalInputVm,JsValue> {
        let binding=serde_json::from_str(binding_json)
            .map_err(|_|JsValue::from_str("E_LOGICAL_INPUT_BINDING: 논리 입력 결속 필드가 올바르지 않습니다"))?;
        let mut run=runtime_surface::fresh_authored_logical_run(authored,binding)
            .map_err(|reason|JsValue::from_str(&reason))?;
        let recording=crate::input_tape_v3::LiveRecording::for_logical(run.session_mut_for_recording())
            .map_err(|reason|JsValue::from_str(&reason))?;
        Ok(Self {execution:LogicalVmExecution::Authored(run),recording})
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

    pub fn register_input_producer_json(&mut self)->Result<String,JsValue> {
        let session=match &mut self.execution {
            LogicalVmExecution::SingleFile(session)=>session,
            LogicalVmExecution::Authored(run)=>run.session_mut_for_recording(),
        };
        self.recording.register_frontend_logical(session).map(|value|value.to_string()).map_err(|e|JsValue::from_str(&e))
    }
    pub fn preview_finish_pending_json(&mut self,packet:&str)->Result<String,JsValue> {
        let session=match &mut self.execution {
            LogicalVmExecution::SingleFile(session)=>session,
            LogicalVmExecution::Authored(run)=>run.session_mut_for_recording(),
        };
        let (bytes,summary)=self.recording.preview_logical_finish(session,packet).map_err(|e|JsValue::from_str(&e))?;
        Ok(json!({"tape_bytes":bytes,"summary":summary}).to_string())
    }
    pub fn result_identity_json(&self)->Result<String,JsValue>{
        let result=self.session().borrow_result_boundary()
            .and_then(crate::replay_evidence::ResultPreimageV1::capture).map_err(|e|JsValue::from_str(&e))?;
        Ok(json!({"result_identity":result.digest_label()}).to_string())
    }
    pub fn finish_pending_json(&mut self,packet:&str)->Result<String,JsValue> {
        let session=match &mut self.execution {
            LogicalVmExecution::SingleFile(session)=>session,
            LogicalVmExecution::Authored(run)=>run.session_mut_for_recording(),
        };
        let (bytes,summary)=self.recording.finish_logical(session,Some(packet)).map_err(|e|JsValue::from_str(&e))?;
        Ok(json!({"tape_bytes":bytes,"summary":summary}).to_string())
    }
    pub fn finish_notice_json(&self)->Result<String,JsValue> {
        self.recording.finish_notice().map(|value|value.to_string()).map_err(|e|JsValue::from_str(&e))
    }
    pub fn seal_json(&mut self) -> Result<String, JsValue> {
        let session=match &mut self.execution {
            LogicalVmExecution::SingleFile(session)=>session,
            LogicalVmExecution::Authored(run)=>run.session_mut_for_recording(),
        };
        let (bytes,summary)=self.recording.finish_logical(session,None).map_err(|e|JsValue::from_str(&e))?;
        Ok(json!({"tape_bytes":bytes,"summary":summary}).to_string())
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
fn input_replay_mode(allow_partial: Option<bool>) -> crate::input_tape_v3::ReplayMode {
    if allow_partial == Some(true) { crate::input_tape_v3::ReplayMode::RecordedPrefix }
    else { crate::input_tape_v3::ReplayMode::CompleteOnly }
}
#[wasm_bindgen]
pub fn authored_logical_input_replay_v3(authored:&str,tape:&[u8],binding_json:&str,allow_partial:Option<bool>)->Result<String,JsValue> {
    let binding=serde_json::from_str(binding_json).map_err(|_|JsValue::from_str("E_INPUT_TAPE_V3: 작품 기대 입력 계약 오류"))?;
    crate::input_tape_v3::replay_authored_logical_with_mode(authored,tape,&binding,input_replay_mode(allow_partial))
        .map(|summary|crate::unit_wire::SnapshotJsonOwner::from_value(summary).compact_text()).map_err(|e|JsValue::from_str(&e))
}
#[wasm_bindgen]
pub fn logical_input_replay_v3(source:&str,tape:&[u8],binding_json:&str,allow_partial:Option<bool>)->Result<String,JsValue> {
    let binding=serde_json::from_str(binding_json).map_err(|_|JsValue::from_str("E_INPUT_TAPE_V3: 기대 입력 계약 오류"))?;
    crate::input_tape_v3::replay_with_mode(source,tape,crate::input_tape_v3::ExpectedBinding::Logical(&binding),input_replay_mode(allow_partial))
        .map(|summary|crate::unit_wire::SnapshotJsonOwner::from_value(summary).compact_text()).map_err(|e|JsValue::from_str(&e))
}
#[wasm_bindgen]
pub fn manipulation_input_record_v3(source:&str,request_json:&str)->Result<String,JsValue> {
    let request=serde_json::from_str(request_json).map_err(|_|JsValue::from_str("E_INPUT_TAPE_V3: 조작 입력 요청 필드 오류"))?;
    let (bytes,summary)=crate::input_tape_v3::record_manipulation(source,&request).map_err(|e|JsValue::from_str(&e))?;
    Ok(json!({"tape_bytes":bytes,"summary":summary}).to_string())
}
#[wasm_bindgen]
pub fn manipulation_input_replay_v3(source:&str,tape:&[u8],binding_json:&str,allow_partial:Option<bool>)->Result<String,JsValue> {
    let binding=serde_json::from_str(binding_json).map_err(|_|JsValue::from_str("E_INPUT_TAPE_V3: 기대 조작 입력 계약 오류"))?;
    crate::input_tape_v3::replay_with_mode(source,tape,crate::input_tape_v3::ExpectedBinding::Manipulation(&binding),input_replay_mode(allow_partial))
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
        let mut session = crate::manipulation_input::ManipulationSession::new(source,binding).map_err(|error| JsValue::from_str(&error))?;
        let recording=crate::input_tape_v3::LiveRecording::for_manipulation(&mut session).map_err(|error|JsValue::from_str(&error))?;
        Ok(Self {session,recording})
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
    pub fn register_input_producer_json(&mut self)->Result<String,JsValue> {
        self.recording.register_frontend_manipulation(&mut self.session)
            .map(|value|value.to_string()).map_err(|e|JsValue::from_str(&e))
    }
    pub fn finish_pending_json(&mut self,packet:&str)->Result<String,JsValue> {
        let (bytes,summary)=self.recording.finish_manipulation(&mut self.session,Some(packet)).map_err(|e|JsValue::from_str(&e))?;
        Ok(json!({"tape_bytes":bytes,"summary":summary}).to_string())
    }
    pub fn finish_notice_json(&self)->Result<String,JsValue> {
        self.recording.finish_notice().map(|value|value.to_string()).map_err(|e|JsValue::from_str(&e))
    }
    pub fn seal_json(&mut self) -> Result<String, JsValue> {
        let (bytes,summary)=self.recording.finish_manipulation(&mut self.session,None).map_err(|e|JsValue::from_str(&e))?;
        Ok(json!({"tape_bytes":bytes,"summary":summary}).to_string())
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

/// Product-private original ordinary recording comparison. Native custody
/// must be checked separately; no VM, End, restore, or ReplayVerified leaks.
#[wasm_bindgen]
pub fn wasm_compare_general_original_recording_json(bundle_bytes:&[u8])->Result<String,JsValue>{
    crate::unit_wire::resumable_complete_state::ReadOnlyGeneralReplayV1::check_running(bundle_bytes)
        .map(|checked|checked.report_json()).map_err(|cause|JsValue::from_str(&cause))
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

struct NativeSourceBinding {authored:String,checkpoint_tick:u64,first_frame_pending:bool,first_input_receipt:Option<crate::unit_wire::SnapshotJsonOwner>,initial_reset_pending:bool,authoring_selection:String}

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
        Self::from_native_project_reply(authored,reply)
    }
    fn from_native_project_reply(authored:&str,reply:crate::unit_wire::SnapshotJsonOwner)->Result<DdnWasmVm,JsValue>{
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
            first_frame_pending,first_input_receipt,initial_reset_pending:true,authoring_selection:reply["authoring_selection"].to_string()});
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
    /// Only an explicit Run action requests consumption. Selection JSON is
    /// untrusted locator data; the Native protected owner issues the permit.
    pub fn from_native_project_json_for_new_run(authored:&str,selection_json:&str)->Result<DdnWasmVm,JsValue>{
        let selection:crate::runtime_surface::NativeAuthoringSelection=serde_json::from_str(selection_json)
            .map_err(|_|JsValue::from_str("E_NATIVE_SOURCE_SELECTION_FIELDS"))?;
        let reply=native_source_exchange(&json!({"operation":"new_run","authored_project_json_utf8":authored,
            "authoring_selection":selection,"seed":"0"}),None)?;
        Self::from_native_project_reply(authored,reply)
    }
    pub fn native_source_selection_json(&self)->String{
        self.native_source_binding.as_ref().map(|binding|binding.authoring_selection.clone()).unwrap_or_else(||"null".into())
    }
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
        if staged.tick_id>0 {
            self.runner.validate_restored_derived_world(staged.world.world())
                .map_err(|err|JsValue::from_str(&format!("restore_state: {err}")))?;
        }
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

mod observation_results;
use observation_results::*;

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

mod stream_projection;
use stream_projection::*;

#[cfg(test)]
mod tests;

mod resource_projection;
use resource_projection::*;

