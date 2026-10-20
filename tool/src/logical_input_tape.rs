//! Alpha2 logical input tape. One shared codec; no physical-key fallback.
//! Channel identities must be supplied by the admitted execution contract.
use std::collections::BTreeMap;

use ddonirang_core::input::{
    LogicalChannelKind, LogicalInputEvent, LogicalInputEventKind, LogicalInputQuery,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const LOGICAL_INPUT_PROFILE: &str = "v1-core-v25-logical-alpha2";
const MAGIC: &[u8] = b"DDN_LOGICAL_INPUT_TAPE_V1\n";
const DOMAIN: &[u8] = b"ddn.logical_input_tape.v1\0";
const SNAPSHOT_DOMAIN: &[u8] = b"ddn.logical_input_snapshot.v1\0";
const MAX_BYTES: usize = 67_108_864;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalInputBinding {
    pub source_profile: String,
    pub input_schema: String,
    pub channel_registry: String,
    pub source_revision: String,
    pub initial_world: String,
    pub initial_rng: u64,
    pub recording_policy: String,
    pub integrity_policy: String,
    pub madi_hz: u32,
    pub dt_raw: i64,
    pub channels: BTreeMap<String, ChannelKind>,
    pub initial_held: BTreeMap<String, bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelKind {
    Control,
    // Reserved in the schema; Alpha2 admission explicitly rejects it.
    Text,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", deny_unknown_fields)]
pub enum EventKind {
    Press,
    Repeat,
    Release,
    CommittedText(String),
    Paste(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalEvent {
    pub ordinal: u64,
    pub channel: String,
    pub event: EventKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalSnapshot {
    pub madi: u64,
    pub events: Vec<LogicalEvent>,
    pub held: BTreeMap<String, bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalFrame {
    pub input: LogicalSnapshot,
    pub input_digest: String,
    pub accepted_world_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Termination {
    Complete,
    Aborted,
    Incomplete,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalTape {
    pub binding: LogicalInputBinding,
    pub frames: Vec<LogicalFrame>,
    pub termination: Termination,
    pub accepted_madi_count: u64,
    pub accepted_event_count: u64,
    pub failure_reason: Option<String>,
}

fn error(code: &str, message: &str) -> String {
    format!("{code}: {message}")
}

fn digest(domain: &[u8], bytes: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update(bytes);
    hash.finalize().into()
}

/// Canonical bytes: serde struct field order; BTreeMap channel order; event
/// order unchanged; UTF-8 unchanged; integers only; absent reason is JSON null.
pub fn snapshot_digest(input: &LogicalSnapshot) -> Result<String, String> {
    let bytes = serde_json::to_vec(input)
        .map_err(|_| error("E_LOGICAL_INPUT_ENCODING", "입력 정본화에 실패했습니다"))?;
    Ok(hex::encode(digest(SNAPSHOT_DOMAIN, &bytes)))
}

pub fn query(
    binding: &LogicalInputBinding,
    input: &LogicalSnapshot,
) -> Result<LogicalInputQuery<String>, String> {
    if binding.source_profile != LOGICAL_INPUT_PROFILE
        || binding.madi_hz != crate::input_receipt::alpha2_host_madi_hz()
        || binding.dt_raw != crate::input_receipt::alpha_host_dt().raw_i64()
    {
        return Err(error(
            "E_LOGICAL_INPUT_PROFILE",
            "논리 입력 프로필·마디율·dt가 다릅니다",
        ));
    }
    if binding
        .channels
        .values()
        .any(|kind| *kind == ChannelKind::Text)
        || input.events.iter().any(|event| {
            matches!(
                event.event,
                EventKind::CommittedText(_) | EventKind::Paste(_)
            )
        })
    {
        return Err(error(
            "E_LOGICAL_INPUT_TEXT_UNSUPPORTED",
            "알파2는 글자 입력을 지원하지 않습니다",
        ));
    }
    let mut events = Vec::with_capacity(input.events.len());
    for (ordinal, event) in input.events.iter().enumerate() {
        if event.ordinal != ordinal as u64 {
            return Err(error(
                "E_LOGICAL_INPUT_ORDER",
                "마디 안 사건 순서가 연속되지 않습니다",
            ));
        }
        let kind = match event.event {
            EventKind::Press => LogicalInputEventKind::Press,
            EventKind::Repeat => LogicalInputEventKind::Repeat,
            EventKind::Release => LogicalInputEventKind::Release,
            _ => unreachable!("text rejected above"),
        };
        events.push(LogicalInputEvent {
            receive_ordinal: event.ordinal,
            channel: event.channel.clone(),
            kind,
        });
    }
    let channels = binding
        .channels
        .keys()
        .map(|channel| (channel.clone(), LogicalChannelKind::Control))
        .collect();
    LogicalInputQuery::from_projection(input.madi, channels, events, input.held.clone()).map_err(
        |_| {
            error(
                "E_LOGICAL_INPUT_SCHEMA",
                "논리 채널·사건·눌린 상태가 입력 계약과 다릅니다",
            )
        },
    )
}

pub(crate) fn validate(tape: &LogicalTape) -> Result<(), String> {
    let binding = &tape.binding;
    if [
        &binding.input_schema,
        &binding.channel_registry,
        &binding.source_revision,
        &binding.initial_world,
        &binding.recording_policy,
        &binding.integrity_policy,
    ]
    .iter()
    .any(|value| value.is_empty())
        || binding.channels.keys().any(|value| value.is_empty())
    {
        return Err(error(
            "E_LOGICAL_INPUT_BINDING",
            "실행·채널 계약 결속이 비어 있습니다",
        ));
    }
    // Validate the binding even for a zero-frame recording.
    let empty = LogicalSnapshot {
        madi: 0,
        events: vec![],
        held: binding.initial_held.clone(),
    };
    query(binding, &empty)?;
    let mut settled = binding.initial_held.clone();
    let mut count = 0u64;
    for (madi, frame) in tape.frames.iter().enumerate() {
        if frame.input.madi != madi as u64 {
            return Err(error(
                "E_LOGICAL_INPUT_ORDER",
                "기록의 마디가 연속되지 않습니다",
            ));
        }
        query(binding, &frame.input)?;
        for event in &frame.input.events {
            let held = settled
                .get_mut(&event.channel)
                .ok_or_else(|| error("E_LOGICAL_INPUT_SCHEMA", "모르는 논리 채널입니다"))?;
            match event.event {
                EventKind::Press => *held = true,
                EventKind::Release => *held = false,
                EventKind::Repeat if *held => {}
                EventKind::Repeat => {
                    return Err(error(
                        "E_LOGICAL_INPUT_STATE",
                        "눌리지 않은 채널에 반복 사건이 있습니다",
                    ))
                }
                _ => unreachable!("text rejected by query"),
            }
        }
        if settled != frame.input.held {
            return Err(error(
                "E_LOGICAL_INPUT_STATE",
                "사건 기록과 확정된 눌린 상태가 다릅니다",
            ));
        }
        if frame.input_digest != snapshot_digest(&frame.input)?
            || frame.accepted_world_digest.is_empty()
        {
            return Err(error(
                "E_LOGICAL_INPUT_DIGEST",
                "입력 결속 또는 세계 결과 증거가 올바르지 않습니다",
            ));
        }
        count = count
            .checked_add(frame.input.events.len() as u64)
            .ok_or_else(|| error("E_LOGICAL_INPUT_COUNT", "사건 수가 범위를 넘었습니다"))?;
    }
    if tape.accepted_madi_count != tape.frames.len() as u64
        || tape.accepted_event_count != count
        || match tape.termination {
            Termination::Complete => tape.failure_reason.is_some(),
            _ => tape
                .failure_reason
                .as_ref()
                .is_none_or(|reason| reason.is_empty()),
        }
    {
        return Err(error(
            "E_LOGICAL_INPUT_SEAL",
            "완결 상태·실패 정보·기록 수가 맞지 않습니다",
        ));
    }
    Ok(())
}

/// Wire: magic, LE u64 payload length, canonical UTF-8 payload, domain SHA256.
/// The checksum detects corruption; it does not grant provenance authority.
pub fn encode(tape: &LogicalTape) -> Result<Vec<u8>, String> {
    validate(tape)?;
    let payload = serde_json::to_vec(tape)
        .map_err(|_| error("E_LOGICAL_INPUT_ENCODING", "기록 정본화에 실패했습니다"))?;
    if payload.len() > MAX_BYTES {
        return Err(error(
            "E_LOGICAL_INPUT_SIZE",
            "봉인 입력 크기 한도를 넘었습니다",
        ));
    }
    let mut bytes = Vec::with_capacity(MAGIC.len() + 8 + payload.len() + 32);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&payload);
    let seal = digest(DOMAIN, &bytes);
    bytes.extend_from_slice(&seal);
    Ok(bytes)
}

pub fn decode(bytes: &[u8]) -> Result<LogicalTape, String> {
    let prefix = MAGIC.len() + 8;
    if !bytes.starts_with(MAGIC) || bytes.len() < prefix + 32 {
        return Err(error(
            "E_LOGICAL_INPUT_VERSION",
            "논리 입력 기록 버전이 아니거나 잘렸습니다",
        ));
    }
    let length = u64::from_le_bytes(bytes[MAGIC.len()..prefix].try_into().unwrap());
    if length > MAX_BYTES as u64 || length as usize != bytes.len() - prefix - 32 {
        return Err(error("E_LOGICAL_INPUT_SIZE", "기록 길이가 맞지 않습니다"));
    }
    let end = bytes.len() - 32;
    if digest(DOMAIN, &bytes[..end]).as_slice() != &bytes[end..] {
        return Err(error("E_LOGICAL_INPUT_SEAL", "입력 기록 봉인이 다릅니다"));
    }
    let tape: LogicalTape = serde_json::from_slice(&bytes[prefix..end]).map_err(|_| {
        error(
            "E_LOGICAL_INPUT_ENCODING",
            "입력 기록 필드가 올바르지 않습니다",
        )
    })?;
    validate(&tape)?;
    if encode(&tape)? != bytes {
        return Err(error(
            "E_LOGICAL_INPUT_NONCANONICAL",
            "입력 기록이 정본 바이트가 아닙니다",
        ));
    }
    Ok(tape)
}

/// Replay requires the entire admitted binding, not merely a matching profile.
pub fn admit_replay(bytes: &[u8], expected: &LogicalInputBinding) -> Result<LogicalTape, String> {
    let tape = decode(bytes)?;
    if tape.termination != Termination::Complete {
        return Err(error(
            "E_LOGICAL_INPUT_INCOMPLETE",
            "미완결·중단 기록은 정상 재생하지 않습니다",
        ));
    }
    if &tape.binding != expected {
        return Err(error(
            "E_LOGICAL_INPUT_BINDING_MISMATCH",
            "원문·초기 상태·입력 실행 계약이 다릅니다",
        ));
    }
    Ok(tape)
}

/// A host record request is not an alternate replay format: only the sealed
/// tape bytes enter replay. Channel contracts remain explicit, never sniffed.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalRecordRequest {
    pub binding: LogicalInputBinding,
    pub snapshots: Vec<LogicalSnapshot>,
}

/// One incremental execution path for live hosts, file recording and replay.
/// A rejected madi leaves the preceding accepted world intact and permanently
/// aborts this session; it cannot subsequently mint a complete success seal.
pub struct LogicalInputSession {
    result_emissions:crate::replay_evidence::ResultEmissionSinkV1,
    result_evidence_required:bool,
    result_original_start:Option<OriginalLogicalResultStartV1>,
    binding: LogicalInputBinding,
    runner: crate::ddn_runtime::DdnRunner,
    world: ddonirang_core::platform::DetNuri,
    settled: BTreeMap<String, bool>,
    frames: Vec<LogicalFrame>,
    // Kept only for successfully committed madi, by the execution producer.
    core_inputs: Vec<ddonirang_core::InputSnapshot>,
    receipts: Vec<serde_json::Value>,
    failure: Option<String>,
    last_bogae_projection: Option<serde_json::Value>,
    display_outputs: Vec<crate::output_display::OutputDisplay>,
    last_observation_table: Option<serde_json::Value>,
    observation_frames: Vec<serde_json::Value>,
}

// Issued at this source-admission constructor, never deserialized from a
// checkpoint. A saved reconstruction cannot backfill original result evidence.
struct OriginalLogicalResultStartV1 { binding:LogicalInputBinding }

pub(crate) struct CommittedLogicalResultBoundary<'a> {session:&'a LogicalInputSession}
impl CommittedLogicalResultBoundary<'_> {
    pub(crate) fn content_parts(&self)->Result<(Vec<u8>,Vec<u8>,Vec<u8>,u64),String> {
        use ddonirang_core::Nuri;
        let session=self.session;
        let original=session.result_original_start.as_ref().ok_or(
            "E_MISSING_RESULT_EVIDENCE: 원 시작의 결과 증거가 없습니다")?;
        if original.binding!=session.binding {
            return Err("E_RESULT_SOURCE_MISMATCH: 원 시작과 현재 입력 계약이 다릅니다".into());
        }
        let next=session.frames.len() as u64;
        let outputs=session.accepted_output_content()?;
        let rng=crate::unit_wire::resumable_complete_state::encode_logical_accepted_rng_content(session)
            .map_err(|_|"E_RESULT_RNG_MISMATCH: 실제 확정 입력의 난수 경계가 다릅니다")?;
        Ok((crate::runtime_surface::result_world_canonical_bytes(session.world.world()),outputs,rng,next))
    }
}

impl Drop for LogicalInputSession {
    fn drop(&mut self) {
        crate::unit_wire::clear_retained_json_option(&mut self.last_bogae_projection);
        crate::unit_wire::clear_retained_json_option(&mut self.last_observation_table);
        crate::unit_wire::clear_retained_json_rows(&mut self.receipts);
        crate::unit_wire::clear_retained_json_rows(&mut self.observation_frames);
    }
}

impl LogicalInputSession {
    pub(crate) fn accepted_prefix_v3(&self) -> (&LogicalInputBinding, &[LogicalFrame], Option<&str>) {
        (&self.binding, &self.frames, self.failure.as_deref())
    }
    pub(crate) fn replay_v3_frame(&mut self, frame: &LogicalFrame) -> Result<serde_json::Value, String> {
        self.step_expected(&frame.input, Some(frame))
    }
    /// Explicit fresh-run source binding, never a replay mismatch fallback.
    /// All caller-supplied contract fields except source_revision remain exact.
    pub fn bind_source_for_fresh_run(source: &str, mut binding: LogicalInputBinding)
        -> Result<LogicalInputBinding, String>
    {
        binding.source_revision = format!("sha256:{}", hex::encode(Sha256::digest(source.as_bytes())));
        // Same admission/ingress as execution; this creates no accepted madi.
        Self::new(source, binding.clone())?;
        Ok(binding)
    }

    pub fn new(source: &str, binding: LogicalInputBinding) -> Result<Self, String> {
        use ddonirang_core::Nuri;
        let empty = LogicalTape {
        binding: binding.clone(),
        frames: vec![],
        termination: Termination::Complete,
        accepted_madi_count: 0,
        accepted_event_count: 0,
        failure_reason: None,
        };
        validate(&empty)?;
        let source_digest = format!("sha256:{}", hex::encode(Sha256::digest(source.as_bytes())));
        let world = ddonirang_core::platform::DetNuri::new();
    if binding.source_revision != source_digest
        || binding.initial_world != crate::runtime_surface::current_state_hash(world.world())
    {
        return Err(error(
            "E_LOGICAL_INPUT_BINDING_MISMATCH",
            "원문 또는 초기 세계 결속이 다릅니다",
        ));
    }
    // Shared source ingress rejects BOM and retired source forms without JS lowering.
    let program = crate::ddn_runtime::DdnProgram::from_v25_source(source, "logical-input.ddn")?;
    crate::runtime_surface::prepare_supported_source_for_execution_with_program(
        source, "logical-input.ddn", &program,
    ).map_err(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))?;
        Ok(Self {
            result_original_start:Some(OriginalLogicalResultStartV1{binding:binding.clone()}),
            settled: binding.initial_held.clone(), binding,
            result_emissions:Default::default(),
            result_evidence_required:false,
            runner: crate::ddn_runtime::DdnRunner::new(program, "매마디").with_bogae_activation(),
            world, frames: Vec::new(), core_inputs: Vec::new(), receipts: Vec::new(), failure: None,
            last_bogae_projection: None, last_observation_table: None,
            display_outputs: Vec::new(),
            observation_frames: Vec::new(),
        })
    }

    pub fn step(&mut self, input: &LogicalSnapshot) -> Result<serde_json::Value, String> {
        self.step_expected(input, None)
    }

    /// Private reconstruction ingress: retain the saved-source runner and its
    /// explicit Start instead of reparsing linked text as a new single file.
    /// This currently reuses the existing empty-initial-world logical profile.
    pub(crate) fn from_saved_runner(
        owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
        binding: LogicalInputBinding, runner: crate::ddn_runtime::DdnRunner,
    ) -> Result<Self,String> {
        use ddonirang_core::Nuri;
        let source = &owned.sources().preparation().source;
        let running = runner.saved_program_sources().ok_or_else(||error(
            "E_LOGICAL_INPUT_SAVED_SOURCE", "저장된 프로젝트 실행기가 필요합니다"))?;
        let supplied = crate::unit_wire::resumable_complete_state::encode_source_capture_supply(owned.sources())
            .map_err(|_|error("E_LOGICAL_INPUT_SAVED_SOURCE", "저장 원문 공급을 확인할 수 없습니다"))?;
        crate::unit_wire::resumable_complete_state::check_source_capture_supply(&supplied,running)
            .map_err(|_|error("E_LOGICAL_INPUT_SAVED_SOURCE", "실행기의 저장 원문이 다릅니다"))?;
        if runner.configured_saved_start() != Some(owned.sources().entry_declaration().name())
            || runner.work_cost_receipt().counters != crate::work_cost::WorkCostCounters::default()
            || runner.internal_provider_registry_len() != 0 {
            return Err(error("E_LOGICAL_INPUT_REBUILD_BOUNDARY", "초기 저장 시작 실행기만 재구성할 수 있습니다"));
        }
        let world = ddonirang_core::platform::DetNuri::new();
        let empty = LogicalTape {binding:binding.clone(),frames:Vec::new(),termination:Termination::Complete,
            accepted_madi_count:0,accepted_event_count:0,failure_reason:None};
        validate(&empty)?;
        if binding.source_revision != format!("sha256:{}",hex::encode(Sha256::digest(source.as_bytes())))
            || binding.initial_world != crate::runtime_surface::current_state_hash(world.world()) {
            return Err(error("E_LOGICAL_INPUT_BINDING_MISMATCH", "저장 원문 또는 초기 세계 결속이 다릅니다"));
        }
        Ok(Self {settled:binding.initial_held.clone(),binding,runner:runner.with_bogae_activation(),world,
            result_emissions:Default::default(),
            result_evidence_required:false,
            result_original_start:None,
            frames:Vec::new(),core_inputs:Vec::new(),receipts:Vec::new(),failure:None,
            last_bogae_projection:None,display_outputs:Vec::new(),last_observation_table:None,observation_frames:Vec::new()})
    }

    /// Replay in this private staging session, using the same commit path and
    /// per-madi input/world comparisons as ordinary sealed tape replay.
    pub(crate) fn replay_sealed_prefix(&mut self, bytes:&[u8]) -> Result<(),String> {
        if !self.frames.is_empty() || self.failure.is_some() {
            return Err(error("E_LOGICAL_INPUT_REBUILD_BOUNDARY", "입력 이력은 초기 실행기에서 한 번만 재구성합니다"));
        }
        let tape = admit_replay(bytes,&self.binding)?;
        for frame in &tape.frames { self.step_expected(&frame.input,Some(frame))?; }
        Ok(())
    }

    pub(crate) fn reconstructed_work_cost(&self) -> crate::work_cost::WorkCostReceipt {
        self.runner.work_cost_receipt()
    }

    pub(crate) fn reject_input(&mut self, reason: String) {
        if self.failure.is_none() { self.failure = Some(reason); }
    }

    fn step_expected(&mut self, input: &LogicalSnapshot, expected: Option<&LogicalFrame>)
        -> Result<serde_json::Value, String>
    {
        if self.failure.is_some() {
            return Err(error("E_LOGICAL_INPUT_SESSION_ABORTED", "중단된 실행은 다시 시작해야 합니다"));
        }
        let result = self.prepare_and_commit(input, expected);
        if let Err(reason) = &result {
            self.failure = Some(reason.clone());
        }
        result
    }

    fn prepare_and_commit(&mut self, input: &LogicalSnapshot, expected: Option<&LogicalFrame>)
        -> Result<serde_json::Value, String>
    {
        use ddonirang_core::{Nuri, VecSignalSink};
        if input.madi != self.frames.len() as u64 {
            return Err(error(
                "E_LOGICAL_INPUT_ORDER",
                "마디가 처음부터 연속되어야 합니다",
            ));
        }
        query(&self.binding, input)?;
        let mut settled = self.settled.clone();
        for event in &input.events {
            let state = settled
                .get_mut(&event.channel)
                .ok_or_else(|| error("E_LOGICAL_INPUT_SCHEMA", "모르는 논리 채널입니다"))?;
            match event.event {
                EventKind::Press => *state = true,
                EventKind::Release => *state = false,
                EventKind::Repeat if *state => {}
                _ => {
                    return Err(error(
                        "E_LOGICAL_INPUT_STATE",
                        "사건과 눌린 상태가 다릅니다",
                    ))
                }
            }
        }
        if settled != input.held {
            return Err(error(
                "E_LOGICAL_INPUT_STATE",
                "사건과 눌린 상태가 다릅니다",
            ));
        }
        let (output, receipt, core_input) =
            self.runner.run_update_logical_retaining_input(self.world.world(), &self.binding, input, &std::collections::HashMap::new(), Some(&self.settled))?;
        self.world.world().validate_typed_resource_patch(&output.patch)
            .map_err(|reason|format!("E_RESULT_WORLD_PATCH_REJECTED: {reason}"))?;
        let pending_outputs=self.result_emissions.prepare_madi(output.result_emissions,input.madi)?;
        if self.result_evidence_required {self.result_emissions.require_exact(&pending_outputs)?;}
        // Patch publication is delayed until both the runtime and replay hash
        // comparison succeed. No external signal sink is called while staged.
        let mut candidate = ddonirang_core::platform::DetNuri::new();
        *candidate.world_mut() = self.world.world().clone();
        let mut sink = VecSignalSink::default();
        candidate.apply_patch(&output.patch, input.madi, &mut sink);
        let frame = LogicalFrame {
            input: input.clone(),
            input_digest: snapshot_digest(input)?,
            accepted_world_digest: crate::runtime_surface::current_state_hash(candidate.world()),
        };
        if expected.is_some_and(|expected| expected != &frame) {
            return Err(error(
                "E_LOGICAL_INPUT_WORLD_MISMATCH",
                "재생 입력 또는 세계 결과가 기록과 다릅니다",
            ));
        }
        // 표시 투영도 마디 전체가 성공했을 때만 게시한다. 기록 바이트와
        // 세계 해시에는 넣지 않으며 CLI/WASM이 같은 원문에서 다시 만든다.
        let run_id = format!("logical:{}", self.binding.source_revision);
        let observations = crate::deterministic_debug::public_observation_events_json(&output.public_observations)?;
        let observation_frame=serde_json::json!({"committed_madi":input.madi,"state_hash":frame.accepted_world_digest,
            "input_snapshot":input,"public_observations":observations});
        let trace = crate::deterministic_debug::build_public_observation_trace(&serde_json::json!({
            "run_id":run_id,"source_identity":self.binding.source_revision,
            "revision_identity":self.binding.source_revision,"run_status":"completed",
            "frames":[observation_frame]
        }))?;
        let table = crate::deterministic_debug::build_public_observation_table_projection(&trace["trace"], &run_id)?;
        let mut bogae_projection=output.bogae_projection;
        if let Some(projection)=bogae_projection.as_mut().filter(|projection|projection["schema"]=="ddn.bogae_projection.v2") {
            let mut frames=self.observation_frames.clone();frames.push(observation_frame.clone());
            crate::bogae_time::resolve_result_tables(projection,&frames,&self.binding.source_revision,&self.binding.source_revision)?;
        }
        let mut result = serde_json::json!({
            "frame": frame, "input_receipt": receipt,
            "resources": crate::runtime_surface::serialize_world_resources(candidate.world()),
            "reaction_support": self.runner.public_provider_source_ingress_count() > 0,
            "state_hash":frame.accepted_world_digest,
            "public_observation_table_projection":table,
        });
        crate::bogae_projection::attach(&mut result, bogae_projection.as_ref());
        self.result_emissions.commit_madi(pending_outputs)?;
        self.display_outputs.extend(output.display_outputs);
        crate::output_display::retain_current(&mut self.display_outputs, &result["resources"]);
        crate::output_display::attach(&mut result, &self.display_outputs);
        crate::unit_wire::clear_retained_json_option(&mut self.last_bogae_projection);
        self.last_bogae_projection = bogae_projection;
        crate::unit_wire::clear_retained_json_option(&mut self.last_observation_table);
        self.last_observation_table = Some(table);
        self.world = candidate;
        self.settled = settled;
        self.frames.push(frame);
        self.core_inputs.push(core_input);
        self.receipts.push(receipt);
        self.observation_frames.push(observation_frame);
        Ok(result)
    }

    /// Actual accepted output content only; not a seed, restore or end proof.
    pub(crate) fn accepted_output_content(&self)->Result<Vec<u8>,String> {
        if self.result_emissions.next_madi()!=self.frames.len() as u64 {
            return Err("E_RESULT_EMISSION_ORDER: 출력과 확정 입력 경계가 다릅니다".into());
        }
        self.result_emissions.content_bytes()
    }

    pub(crate) fn borrow_result_boundary(&self)->Result<CommittedLogicalResultBoundary<'_>,String> {
        if self.result_original_start.is_none() {
            return Err("E_MISSING_RESULT_EVIDENCE: 원 시작의 결과 증거가 없습니다".into());
        }
        Ok(CommittedLogicalResultBoundary{session:self})
    }

    pub(crate) fn recording_generation(&self)->crate::replay_evidence::ExecutionGenerationV1 {
        self.result_emissions.generation()
    }
    pub(crate) fn require_result_evidence_from_start(&mut self)->Result<(),String> {
        if !self.frames.is_empty() || self.failure.is_some() || self.result_original_start.is_none() {
            return Err("E_RESULT_FRESH_START_REQUIRED: 새 원 실행에서만 정확 출력 검증을 선택합니다".into());
        }
        self.result_evidence_required=true;Ok(())
    }
    pub(crate) fn committed_input_receipts(&self)->&[serde_json::Value] {&self.receipts}
    pub(crate) fn initial_execution_contract_supply(&self,owned:&crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1)
        ->Result<Vec<u8>,String> {
        if !self.frames.is_empty() || self.result_original_start.is_none()
            || self.resume_boundary_next_tick(owned.sources().entry_declaration().name())?!=0 {
            return Err("E_REPLAY_SEED_INITIAL_BOUNDARY: 실제 원 시작 경계가 아닙니다".into());
        }
        crate::unit_wire::resumable_complete_state::encode_logical_execution_contract_supply(owned,&self.runner)
            .map_err(|cause|format!("E_REPLAY_SEED_EXECUTION_CONTRACT: {cause:?}"))
    }

    pub(crate) fn bind_original_result_start(&mut self,
        _admission:crate::unit_wire::resumable_complete_state::FreshLogicalResultAdmissionV1)->Result<(),String> {
        self.bind_initial_result_content()
    }
    pub(crate) fn bind_readonly_seed_result_start(&mut self,
        _admission:crate::unit_wire::resumable_complete_state::ReadOnlyLogicalSeedAdmissionV1)->Result<(),String>{
        self.bind_initial_result_content()
    }
    fn bind_initial_result_content(&mut self)->Result<(),String>{
        use ddonirang_core::Nuri;
        if self.result_original_start.is_some() || !self.frames.is_empty() || !self.core_inputs.is_empty()
            || !self.receipts.is_empty() || self.failure.is_some() || self.result_emissions.next_madi()!=0
            || crate::runtime_surface::current_state_hash(self.world.world())!=self.binding.initial_world {
            return Err("E_RESULT_FRESH_START_REQUIRED: 실제 새 시작 경계에서만 원 결과 공급을 결속합니다".into());
        }
        self.result_original_start=Some(OriginalLogicalResultStartV1{binding:self.binding.clone()});
        Ok(())
    }
    #[cfg(windows)]
    pub(crate) fn bind_native_recording_progress(&mut self,
        proof:&crate::unit_wire::resumable_complete_state::CurrentRecordingProgressProofV1)->Result<(),String>{
        proof.check_session(self)?;
        if self.result_original_start.is_some() || self.result_evidence_required {
            return Err("E_NATIVE_RECORDING_PROGRESS_OWNER: 이미 결속된 실행입니다".into());
        }
        self.accepted_output_content()?;
        self.result_original_start=Some(OriginalLogicalResultStartV1{binding:self.binding.clone()});
        self.result_evidence_required=true;Ok(())
    }

    pub fn summary(&self) -> serde_json::Value {
        use ddonirang_core::Nuri;
        let mut summary = serde_json::json!({
        "input_profile": LOGICAL_INPUT_PROFILE,
        "madi_count": self.frames.len(),
        "state_hash": crate::runtime_surface::current_state_hash(self.world.world()),
        "input_receipts": self.receipts,
        "frames": self.frames,
        "resources": crate::runtime_surface::serialize_world_resources(self.world.world()),
        "reaction_support": self.runner.public_provider_source_ingress_count() > 0,
        });
        crate::bogae_projection::attach(&mut summary, self.last_bogae_projection.as_ref());
        crate::output_display::attach(&mut summary, &self.display_outputs);
        if let Some(table) = &self.last_observation_table { summary["public_observation_table_projection"] = table.clone(); }
        if let Some(failure) = &self.failure {
            summary["outcome"] = serde_json::json!("ABORTED");
            summary["failure"] = serde_json::json!(failure);
        }
        summary
    }

    pub(crate) fn committed_world(&self) -> &ddonirang_core::platform::NuriWorld {
        use ddonirang_core::Nuri;
        self.world.world()
    }

    /// The same existing runtime/resource carrier used by ordinary product
    /// exports. Continuation authority still comes from the complete-state
    /// prefix/checkpoint gates, not this display/JSON projection.
    pub(crate) fn saved_runtime_carrier(&self)->Result<(String,String),String> {
        let start=self.runner.configured_saved_start().ok_or("E_RESUME_EXECUTION_STATE_UNSUPPORTED")?;
        let next=self.resume_boundary_next_tick(start)?;
        let frames=self.observation_frames.iter().map(|row|
            crate::unit_wire::SnapshotJsonOwner::clone_from_value(row).into_parent_value()).collect();
        let mut snapshot=crate::runtime_surface::runtime_state_snapshot(&self.world,frames,next);
        snapshot["scene_history"]=self.runner.scene_history_snapshot();
        let resources=crate::unit_wire::SnapshotJsonOwner::clone_from_value(&snapshot["resources"]).compact_text();
        let mut runtime=crate::unit_wire::SnapshotJsonOwner::from_value(self.summary());
        runtime.value_mut()["schema"]="seamgrim.engine_response.v0".into();
        runtime.value_mut()["state"]=serde_json::json!({"snapshot_v2":snapshot});
        Ok((runtime.compact_text(),resources))
    }

    /// R2 source/prefix reconstruction must preserve continuation state and
    /// observations, not just the filtered world hash or latest display.
    /// Both sides are actual runtime owners; transport JSON is not authority.
    /// This comparison does not admit an RNG mode or a durable checkpoint.
    pub(crate) fn check_reconstructed_continuation(&self,other:&Self,start:&str)->Result<(),String> {
        let tick=self.resume_boundary_next_tick(start)?;
        let same_json=|left:&serde_json::Value,right:&serde_json::Value|
            crate::unit_wire::SnapshotJsonOwner::clone_from_value(left).equals_json(right);
        let same_rows=|left:&[serde_json::Value],right:&[serde_json::Value]|
            left.len()==right.len() && left.iter().zip(right).all(|(a,b)|same_json(a,b));
        let same_optional=|left:&Option<serde_json::Value>,right:&Option<serde_json::Value>|match (left,right) {
            (None,None)=>true,(Some(a),Some(b))=>same_json(a,b),_=>false,
        };
        if other.resume_boundary_next_tick(start)?!=tick
            || self.binding!=other.binding || self.settled!=other.settled || self.frames!=other.frames
            || self.runner.consumed_keys_pressed()!=other.runner.consumed_keys_pressed()
            || self.reconstructed_work_cost()!=other.reconstructed_work_cost()
            || self.committed_world().state_hash()!=other.committed_world().state_hash()
            || self.display_outputs!=other.display_outputs
            || !same_rows(&self.receipts,&other.receipts)
            || !same_rows(&self.observation_frames,&other.observation_frames)
            || !same_optional(&self.last_bogae_projection,&other.last_bogae_projection)
            || !same_optional(&self.last_observation_table,&other.last_observation_table)
            || !self.runner.bogae_history_for_resume().same_execution_history(other.runner.bogae_history_for_resume()) {
            return Err(error("E_RESUME_RECONSTRUCTED_STATE_MISMATCH","재구성한 이어 실행 상태와 원 실행의 입력·관찰·보기 이력이 다릅니다"));
        }
        Ok(())
    }

    /// The bounded complete-state profile resumes only after a committed madi.
    /// No active call, reaction registration/queue, or internal reaction world
    /// may be silently dropped by emitting a zero suspended-frame marker.
    pub(crate) fn resume_boundary_next_tick(&self,expected_start:&str)->Result<u64,String> {
        let inputs=self.retained_core_inputs()?;
        let tick=u64::try_from(inputs.len()).map_err(|_|error("E_RESUME_BOUNDARY_LIMIT","마디 수 한도를 넘었습니다"))?;
        let cost=self.runner.work_cost_receipt();
        let reaction=self.runner.internal_reaction_runtime();
        if self.runner.configured_saved_start()!=Some(expected_start)
            || cost.counters.call_depth_current!=0 || cost.counters.committed_madi!=tick
            || self.frames.len()!=inputs.len() || self.receipts.len()!=inputs.len()
            || self.runner.internal_provider_registry_len()!=0
            || reaction.pending_delivery_count()!=0 || reaction.waiting_delivery_count()!=0
            || reaction.revision()!=0 || !reaction.world().is_empty() {
            return Err(error("E_RESUME_EXECUTION_STATE_UNSUPPORTED","완료 마디 이외의 호출·반응·대기 상태는 이어 실행 저장을 지원하지 않습니다"));
        }
        Ok(tick)
    }

    pub fn seal(&self) -> Result<Vec<u8>, String> {
        if self.failure.is_some() {
            return Err(error("E_LOGICAL_INPUT_SESSION_ABORTED", "중단된 실행은 성공 기록을 저장할 수 없습니다"));
        }
        let count = self.frames.iter().try_fold(0u64, |count, frame| {
            count.checked_add(frame.input.events.len() as u64)
        }).ok_or_else(|| error("E_LOGICAL_INPUT_COUNT", "사건 수가 범위를 넘었습니다"))?;
        encode(&LogicalTape {
            binding: self.binding.clone(), frames: self.frames.clone(),
            termination: Termination::Complete, accepted_madi_count: self.frames.len() as u64,
            accepted_event_count: count, failure_reason: None,
        })
    }

    /// Read-only accepted content; never grants continuation after a failure.
    pub(crate) fn accepted_core_input_content(&self)->&[ddonirang_core::InputSnapshot] {&self.core_inputs}

    /// Supply an accepted prefix only; an aborted session cannot supply resume evidence.
    pub(crate) fn retained_core_inputs(&self) -> Result<&[ddonirang_core::InputSnapshot], String> {
        if self.failure.is_some() {
            return Err(error("E_LOGICAL_INPUT_SESSION_ABORTED", "중단된 실행은 이어 실행 입력을 제공할 수 없습니다"));
        }
        Ok(&self.core_inputs)
    }
}

pub fn record_source(
    source: &str,
    request: &LogicalRecordRequest,
) -> Result<(Vec<u8>, serde_json::Value), String> {
    let mut session = LogicalInputSession::new(source, request.binding.clone())?;
    for input in &request.snapshots { session.step(input)?; }
    Ok((session.seal()?, session.summary()))
}

pub fn replay_source(
    source: &str,
    bytes: &[u8],
    expected: &LogicalInputBinding,
) -> Result<serde_json::Value, String> {
    let tape = admit_replay(bytes, expected)?;
    let mut session = LogicalInputSession::new(source, expected.clone())?;
    for frame in &tape.frames {
        session.step_expected(&frame.input, Some(frame))?;
    }
    Ok(session.summary())
}

#[cfg(test)]
mod tests {
    #[test]
    fn retained_logical_json_is_released_on_bounded_stack() {
        // Runner is thread-local (not Send), so construct it on its own normal
        // stack. The 20,000-deep payload still exceeds recursive drop capacity.
        std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(|| {
            let source = "(매마디)마다 { 값 <- 1. }.";
            let mut session = super::LogicalInputSession::new(source, request(source, false).binding).unwrap();
            let deep = || {
                let mut value = serde_json::Value::Null;
                for _ in 0..20_000 { value = serde_json::Value::Array(vec![value]); }
                value
            };
            session.last_bogae_projection = Some(deep());
            session.last_observation_table = Some(deep());
            session.observation_frames.push(deep());
            session.receipts.push(deep());
            drop(session);
        }).unwrap().join().unwrap();
    }
    use super::*;

    #[test]
    fn b3_result_identity_is_not_a_state_hash_alias() {
        use crate::replay_evidence::ResultPreimageV1;
        let capture=|source:&str|{
            let input=request(source,false);
            let mut session=LogicalInputSession::new(source,input.binding).unwrap();
            session.step(&input.snapshots[0]).unwrap();
            let result=ResultPreimageV1::capture(session.borrow_result_boundary().unwrap()).unwrap();
            assert!(result.canonical_bytes().starts_with(b"DDN-WO23-RESULT-V1\0\0\x01\x04"));
            (crate::runtime_surface::current_state_hash(session.committed_world()),result.digest_label())
        };
        let integer=capture("(매마디)마다 { 1 보여주기. }.");
        let approximate=capture("(매마디)마다 { 1.0 보여주기. }.");
        assert_eq!(integer.0,approximate.0,"same legacy output/world content");
        assert_ne!(integer.1,approximate.1,"the original result types differ");
    }

    #[test]
    fn b3_result_capture_requires_original_start_and_actual_rng_prefix() {
        use crate::replay_evidence::ResultPreimageV1;
        let source="(매마디)마다 { 1 보여주기. }.";
        let input=request(source,false);
        let mut session=LogicalInputSession::new(source,input.binding).unwrap();
        session.step(&input.snapshots[0]).unwrap();
        let original=ResultPreimageV1::capture(session.borrow_result_boundary().unwrap()).unwrap();
        assert!(original.digest_label().starts_with("sha256:"));
        session.core_inputs[0].rng_seed^=1;
        assert!(ResultPreimageV1::capture(session.borrow_result_boundary().unwrap()).is_err());
        session.result_original_start=None;
        assert!(session.borrow_result_boundary().is_err(),"no backfill from reconstructed state");
    }

    #[test]
    fn b3_typed_outputs_commit_in_actual_madi_order() {
        let source="(매마디)마다 { 1 보여주기. 1.0 보여주기. }.";
        let input=request(source,false);
        let mut session=LogicalInputSession::new(source,input.binding).unwrap();
        session.step(&input.snapshots[0]).unwrap();
        let first=session.accepted_output_content().unwrap();
        assert_eq!(&first[..8],&2u64.to_be_bytes());
        let second=LogicalSnapshot{madi:1,events:Vec::new(),held:session.settled.clone()};
        session.step(&second).unwrap();
        let after=session.accepted_output_content().unwrap();
        assert_eq!(&after[..8],&4u64.to_be_bytes());
        assert_eq!(&after[8..first.len()],&first[8..]);
        assert_eq!(session.result_emissions.next_madi(),2);
    }

    #[test]
    fn b3_later_handler_failure_keeps_the_accepted_typed_prefix() {
        let source="채비 { 값:바른수 := 0. }. (매마디)마다 { 값 <- 값 + 1. 값 보여주기. (1 / (2 - 값)) 보여주기. }.";
        let input=request(source,false);
        let mut session=LogicalInputSession::new(source,input.binding).unwrap();
        session.step(&input.snapshots[0]).unwrap();
        let prefix=session.accepted_output_content().unwrap();
        let world=crate::runtime_surface::current_state_hash(session.committed_world());
        let second=LogicalSnapshot{madi:1,events:Vec::new(),held:session.settled.clone()};
        assert!(session.step(&second).is_err());
        assert_eq!(session.accepted_output_content().unwrap(),prefix);
        assert_eq!(crate::runtime_surface::current_state_hash(session.committed_world()),world);
        assert_eq!(session.frames.len(),1);
    }

    #[test]
    fn b3_replay_mismatch_cannot_publish_typed_output_candidates() {
        let source="(매마디)마다 { 1 보여주기. }.";
        let input=request(source,false);
        let mut session=LogicalInputSession::new(source,input.binding).unwrap();
        let frame=LogicalFrame{input:input.snapshots[0].clone(),
            input_digest:snapshot_digest(&input.snapshots[0]).unwrap(),
            accepted_world_digest:format!("blake3:{}","00".repeat(32))};
        let world=crate::runtime_surface::current_state_hash(session.committed_world());
        assert!(session.step_expected(&input.snapshots[0],Some(&frame)).is_err());
        assert_eq!(session.accepted_output_content().unwrap(),0u64.to_be_bytes());
        assert_eq!(crate::runtime_surface::current_state_hash(session.committed_world()),world);
        assert!(session.frames.is_empty());
    }

    #[test]
    fn saved_named_start_prefix_reconstruction_continues_actual_execution() {
        use crate::project_revision_envelope_supply::{collect_saved_source_supply,seal_saved_v25_sources,
            admit_saved_authored_project,CanonicalIdentityBytesV1,SavedSourceBlobV1};
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        use crate::unit_wire::resumable_complete_state as complete;
        let root="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=차림[\"app/main.ddn\",\"app/run.ddn\"])], 시작=(모듈=앱, 정의=실행)) 얽음.\n";
        let body="채비 { 값: 수 := 0. }.\n실행:움직씨 := { (매마디)마다 { 값 <- 값 + 1. 값 보여주기. }. }.\n";
        let blobs=[SavedSourceBlobV1{path:"root.ddn",role:"root",bytes:root.as_bytes()},
            SavedSourceBlobV1{path:"app/main.ddn",role:"ddn_source",bytes:b""},
            SavedSourceBlobV1{path:"app/run.ddn",role:"ddn_source",bytes:body.as_bytes()}];
        let sources=std::sync::Arc::new(seal_saved_v25_sources(collect_saved_source_supply(
            "root.ddn","app/main.ddn",&CanonicalIdentityBytesV1::from_owner_bytes(&[9;32]).unwrap(),&blobs,
        ).unwrap()).unwrap());
        let artifact=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot {
            root_path:Some("root.ddn".into()),entry_path:"app/main.ddn".into(),
            files:blobs.iter().map(|b|SourceFile{path:b.path.into(),text:std::str::from_utf8(b.bytes).unwrap().into()}).collect(),
        },String::new()).unwrap();
        let owned=admit_saved_authored_project(sources.clone(),artifact.save().unwrap().as_bytes()).unwrap();
        let issued=crate::project_snapshot_identity::current::issue_saved_local_project(&owned).unwrap();
        let fresh_runner=|| {
            let p=crate::ddn_runtime::DdnProgram::from_saved_v25_project_with_mode(
                sources.clone(),ddonirang_lang::ParseMode::Strict,ddonirang_lang::AgeTarget::Age3,
            ).unwrap();
            let mut r=crate::ddn_runtime::DdnRunner::new(p,"실행");r.configure_project_entry("실행").unwrap();r
        };
        let binding=request(&sources.preparation().source,false).binding;
        let mut original=LogicalInputSession::from_saved_runner(&owned,binding.clone(),fresh_runner()).unwrap();
        let snapshots=[LogicalSnapshot{madi:0,events:vec![],held:binding.initial_held.clone()},
            request(&sources.preparation().source,false).snapshots.remove(0)];
        original.step(&snapshots[0]).unwrap();
        let mut second=snapshots[1].clone();second.madi=1;
        original.step(&second).unwrap();
        assert_eq!(original.resume_boundary_next_tick("실행").unwrap(),2);
        assert!(original.resume_boundary_next_tick("다른시작").is_err());
        // A reaction-owned world is not represented by this bounded position
        // marker. Reject it rather than losing that state during restoration.
        let mut unsupported=LogicalInputSession::from_saved_runner(&owned,binding.clone(),fresh_runner()).unwrap();
        let runner=std::mem::replace(&mut unsupported.runner,fresh_runner());
        unsupported.runner=runner.with_internal_reaction_runtime(
            crate::ddn_runtime::InternalTypedReactionRuntime::with_world_and_queue_capacity(
                BTreeMap::from([("남은상태".into(),"보존필요".into())]),1,
            ),
        );
        let before=unsupported.summary();let before_tape=unsupported.seal().unwrap();
        assert!(unsupported.resume_boundary_next_tick("실행").unwrap_err()
            .starts_with("E_RESUME_EXECUTION_STATE_UNSUPPORTED"));
        assert_eq!(unsupported.summary(),before);assert_eq!(unsupported.seal().unwrap(),before_tape);
        let mut checkpointed=complete::CheckpointedLogicalRunV1::fresh(
            &owned,binding.clone(),ddonirang_lang::AgeTarget::Age3,
        ).unwrap();
        assert_eq!(checkpointed.checkpoint_predecessor_supply().unwrap(),vec![0]);
        let fresh_position=checkpointed.execution_position_supply().unwrap();
        complete::check_execution_position_supply(&fresh_position,0,owned.definition_identity().as_bytes(),
            owned.definition_identity().as_bytes()).unwrap();
        let fresh_descriptor=checkpointed.source_rebuild_descriptor_supply().unwrap();
        let fresh_candidate=complete::rebuild_source_descriptor_candidate(&fresh_descriptor,&owned).unwrap();
        assert_eq!(fresh_candidate.phase,0);assert_eq!(fresh_candidate.successful_madi_count,0);
        checkpointed.step(&snapshots[0]).unwrap();checkpointed.step(&second).unwrap();
        assert_eq!(checkpointed.session().summary(),original.summary());
        assert_eq!(checkpointed.session().seal().unwrap(),original.seal().unwrap());
        let live_position=checkpointed.execution_position_supply().unwrap();
        complete::check_execution_position_supply(&live_position,2,owned.definition_identity().as_bytes(),
            owned.definition_identity().as_bytes()).unwrap();
        assert_ne!(fresh_position,live_position);
        // N>0 accepted madi is still the first durable-save candidate. No
        // project revision/count/SQL lookup manufactures a predecessor.
        assert_eq!(checkpointed.checkpoint_predecessor_supply().unwrap(),vec![0]);
        let live_descriptor=checkpointed.source_rebuild_descriptor_supply().unwrap();
        let live_candidate=complete::rebuild_source_descriptor_candidate(&live_descriptor,&owned).unwrap();
        assert_eq!(live_candidate.phase,1);assert_eq!(live_candidate.successful_madi_count,2);
        assert!(live_candidate.prior_durable_receipts.is_empty());
        let mut bad=second.clone();bad.madi=99;
        assert!(checkpointed.step(&bad).is_err());
        assert!(checkpointed.checkpoint_predecessor_supply().is_err());
        assert!(checkpointed.source_rebuild_descriptor_supply().is_err());
        assert!(checkpointed.execution_position_supply().is_err());
        let mut foreign_binding=binding.clone();foreign_binding.source_revision="foreign".into();
        assert!(complete::CheckpointedLogicalRunV1::fresh(&owned,foreign_binding,
            ddonirang_lang::AgeTarget::Age3).is_err());
        let supplied=complete::encode_processed_input_supply(&original).unwrap();
        let empty=[0;8];
        let initial=crate::unit_wire::CompleteStateFrameV1 {pesi:issued.pesi().as_raw(),resources:&empty,
            parameters:&empty,next_tick:0,subjects:&empty,atom_evidence:&[],subject_custody:&[]}.encode_canonical().unwrap();
        // Consume the descriptor produced by the actual fresh execution owner,
        // rather than synthesizing a matching SourceRebuildCandidate in tests.
        let candidate=|| complete::rebuild_source_descriptor_candidate(&live_descriptor,&owned).unwrap();
        let mut rebuilt=complete::replay_processed_input_candidate(&supplied,candidate(),&owned,&binding).unwrap();
        assert_eq!(original.seal().unwrap(),rebuilt.seal().unwrap());
        assert_eq!(original.summary(),rebuilt.summary());
        assert_eq!(original.reconstructed_work_cost(),rebuilt.reconstructed_work_cost());
        original.check_reconstructed_continuation(&rebuilt,"실행").unwrap();
        // World and tape still match when this hidden keyboard edge state is
        // wrong. The independent continuation comparison must reject it.
        let consumed=rebuilt.runner.consumed_keys_pressed();
        rebuilt.runner.restore_consumed_keys_pressed(consumed^1);
        assert_eq!(original.committed_world().state_hash(),rebuilt.committed_world().state_hash());
        assert!(original.check_reconstructed_continuation(&rebuilt,"실행").is_err());
        rebuilt.runner.restore_consumed_keys_pressed(consumed);
        original.check_reconstructed_continuation(&rebuilt,"실행").unwrap();
        // An old observation is not covered by the current world's hash or
        // latest observation table; preserve the whole prefix, not just its end.
        let old=rebuilt.observation_frames[0].clone();
        rebuilt.observation_frames[0]["state_hash"]=serde_json::json!("foreign-history");
        assert!(original.check_reconstructed_continuation(&rebuilt,"실행").is_err());
        rebuilt.observation_frames[0]=old;
        original.check_reconstructed_continuation(&rebuilt,"실행").unwrap();
        let work=complete::encode_replayed_work_supply(&original).unwrap();
        complete::check_replayed_work_supply(&work,&rebuilt.reconstructed_work_cost()).unwrap();
        let mut reset_work=work.clone();reset_work[3..3+15*8].fill(0);
        assert!(complete::check_replayed_work_supply(&reset_work,&rebuilt.reconstructed_work_cost()).is_err());
        // Independently encode the known hand-computed counter value2, not a
        // copy of runtime output. Changed/missing state and next tick reject.
        let world_bytes=|value:i64,tick:u64| {
            use ddonirang_core::platform::{ResourceValue as V,ResourceMapEntry};
            let logs=V::List((0..2).map(|i| {
                let pairs=[("kind",V::String("output".into())),("line_no",V::Fixed64(ddonirang_core::Fixed64::from_i64(i+1))),
                    ("text",V::String((i+1).to_string())),("tick",V::Fixed64(ddonirang_core::Fixed64::from_i64(i)))];
                V::Map(pairs.into_iter().map(|(name,value)| {
                    let key=V::String(name.into());(key.canon_key(),ResourceMapEntry{key,value})
                }).collect())
            }).collect());
            let values=BTreeMap::from([("output_log",logs),("값",V::Fixed64(ddonirang_core::Fixed64::from_i64(value)))]);
            let mut rows=(values.len() as u64).to_be_bytes().to_vec();
            for (name,value) in values {
                let mut leaf=Vec::new();value.encode_canon(&mut leaf);
                rows.extend((name.len() as u32).to_be_bytes());rows.extend(name.as_bytes());
                rows.extend((leaf.len() as u64).to_be_bytes());rows.extend(leaf);
            }
            crate::unit_wire::CompleteStateFrameV1 {pesi:issued.pesi().as_raw(),resources:&rows,
                parameters:&empty,next_tick:tick,subjects:&empty,atom_evidence:&[],subject_custody:&[]}.encode_canonical().unwrap()
        };
        let saved_world=world_bytes(2,2);
        let check_world=|raw:&[u8]| complete::check_replayed_world_candidate(
            &crate::unit_wire::check_complete_state_frame_v1(raw,issued.pesi().as_raw()).unwrap(),&rebuilt,
        ).is_ok();
        assert!(check_world(&saved_world),"{}",rebuilt.summary());assert!(!check_world(&world_bytes(3,2)));
        assert!(!check_world(&world_bytes(2,3)));
        let nonempty_initial=world_bytes(2,0);
        assert!(complete::replay_processed_input_candidate(&supplied,complete::SourceRebuildCandidateV1 {
            initial_world:crate::unit_wire::check_complete_state_frame_v1(&nonempty_initial,issued.pesi().as_raw()).unwrap(),
            ..candidate()
        },&owned,&binding).is_err(),"unsupported initial state must not be cleared");
        let next=LogicalSnapshot{madi:2,events:vec![LogicalEvent{ordinal:0,channel:"더함".into(),event:EventKind::Release}],
            held:BTreeMap::from([("더함".into(),false),("곱함".into(),true)])};
        assert_eq!(original.step(&next).unwrap(),rebuilt.step(&next).unwrap());
        assert_eq!(original.reconstructed_work_cost(),rebuilt.reconstructed_work_cost());
        assert_eq!(original.summary(),rebuilt.summary());
        assert_eq!(original.display_outputs.last().unwrap().text,"3");
        let mut damaged=supplied.clone();*damaged.last_mut().unwrap()^=1;
        assert!(complete::replay_processed_input_candidate(&damaged,candidate(),&owned,&binding).is_err());
        let mut changed=binding.clone();changed.channel_registry.push_str("foreign");
        assert!(complete::replay_processed_input_candidate(&supplied,candidate(),&owned,&changed).is_err());
        let before=original.seal().unwrap();
        assert!(complete::replay_processed_input_candidate(&supplied,complete::SourceRebuildCandidateV1 {
            successful_madi_count:3,..candidate()
        },&owned,&binding).is_err());
        assert_eq!(original.seal().unwrap(),before,"reconstruction failure touched the outgoing execution");
    }

    #[test]
    fn saved_scene_prefix_reconstruction_preserves_past_views() {
        use crate::project_revision_envelope_supply::{collect_saved_source_supply,seal_saved_v25_sources,
            admit_saved_authored_project,CanonicalIdentityBytesV1,SavedSourceBlobV1};
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        use crate::unit_wire::resumable_complete_state as complete;
        let root="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=차림[\"app/main.ddn\"])], 시작=(모듈=앱, 정의=실행)) 얽음.\n";
        let body="채비 { 값:수 := 0. }. 실행:움직씨 := { (매마디)마다 { 값 <- 값 + 1. (값:값) 보임. }. }. 짜임 := (관계들=차림[(보기=(() 표준보기.결과표)) 표준보개.으뜸]) 표준보개.짜임.";
        let blobs=[SavedSourceBlobV1{path:"root.ddn",role:"root",bytes:root.as_bytes()},
            SavedSourceBlobV1{path:"app/main.ddn",role:"ddn_source",bytes:body.as_bytes()}];
        let sources=std::sync::Arc::new(seal_saved_v25_sources(collect_saved_source_supply(
            "root.ddn","app/main.ddn",&CanonicalIdentityBytesV1::from_owner_bytes(&[8;32]).unwrap(),&blobs,
        ).unwrap()).unwrap());
        let artifact=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot {
            root_path:Some("root.ddn".into()),entry_path:"app/main.ddn".into(),
            files:blobs.iter().map(|b|SourceFile{path:b.path.into(),text:std::str::from_utf8(b.bytes).unwrap().into()}).collect(),
        },String::new()).unwrap();
        let owned=admit_saved_authored_project(sources.clone(),artifact.save().unwrap().as_bytes()).unwrap();
        let binding=request(&sources.preparation().source,false).binding;
        let mut original=complete::CheckpointedLogicalRunV1::fresh(
            &owned,binding.clone(),ddonirang_lang::AgeTarget::Age3,
        ).unwrap();
        for madi in 0..2 {original.step(&LogicalSnapshot{madi,events:vec![],held:binding.initial_held.clone()}).unwrap();}
        let session=original.session();
        let saved=session.runner.scene_history_snapshot();
        assert_eq!(saved["frames"].as_array().unwrap().len(),2,"actual source must produce nonempty scene history");
        let prefix=complete::encode_processed_input_supply(session).unwrap();
        let descriptor=original.source_rebuild_descriptor_supply().unwrap();
        let candidate=complete::rebuild_source_descriptor_candidate(&descriptor,&owned).unwrap();
        let mut rebuilt=complete::replay_processed_input_candidate(&prefix,candidate,&owned,&binding).unwrap();
        session.check_reconstructed_continuation(&rebuilt,"실행").unwrap();
        let world=rebuilt.committed_world().state_hash();
        let latest=rebuilt.last_bogae_projection.clone();
        // A valid, same-sized earlier scene differs while the latest scene,
        // world, tape and observations stay intact. No bad-digest shortcut.
        let mut changed=saved.clone();
        changed["frames"][0]["projection"]["candidates"][0]["artifact"]["view"]["kind"]=serde_json::json!("result_tablX");
        let frames=&changed["frames"];
        changed["sha256"]=serde_json::json!(crate::runtime_surface::sha256_hex(
            format!("ddn.bogae_scene_history.v1\0{frames}").as_bytes()));
        let history=crate::bogae_time::SceneHistory::from_snapshot(Some(&changed),2).unwrap();
        rebuilt.runner.restore_scene_history(history);
        assert_eq!(rebuilt.committed_world().state_hash(),world);
        assert_eq!(rebuilt.last_bogae_projection,latest);
        assert!(session.check_reconstructed_continuation(&rebuilt,"실행").is_err());
        rebuilt.runner.restore_scene_history(crate::bogae_time::SceneHistory::from_snapshot(Some(&saved),2).unwrap());
        session.check_reconstructed_continuation(&rebuilt,"실행").unwrap();
        let next=LogicalSnapshot{madi:2,events:vec![],held:binding.initial_held.clone()};
        assert_eq!(original.step(&next).unwrap(),rebuilt.step(&next).unwrap());
        original.session().check_reconstructed_continuation(&rebuilt,"실행").unwrap();
    }

    #[test]
    fn retained_core_inputs_are_actual_committed_prefix_and_abort_cannot_supply() {
        let source = "(매마디)마다 { 값 <- 3. }.";
        let request = request(source, false);
        let mut session = LogicalInputSession::new(source, request.binding.clone()).unwrap();
        assert!(session.retained_core_inputs().unwrap().is_empty());
        let empty_supply = crate::unit_wire::resumable_complete_state::encode_processed_input_supply(&session).unwrap();
        crate::unit_wire::resumable_complete_state::check_processed_input_supply(&empty_supply, &session).unwrap();
        for madi in 0..2 {
            session.step(&LogicalSnapshot {
                madi, events: vec![], held: request.binding.initial_held.clone(),
            }).unwrap();
        }
        let inputs = session.retained_core_inputs().unwrap();
        assert_eq!(inputs.len(), 2);
        for (madi, input) in inputs.iter().enumerate() {
            assert_eq!(input.tick_id, madi as u64);
            assert_eq!(input.dt.raw_i64(), request.binding.dt_raw);
            assert_eq!(input.rng_seed, request.binding.initial_rng);
            assert_eq!(input.frame_source, ddonirang_core::InputSource::Person);
            assert_eq!(input.keys_pressed, 0);
            assert!(input.last_key_name.is_empty());
            assert_eq!((input.pointer_x_i32, input.pointer_y_i32), (0, 0));
            assert!(input.ai_injections.is_empty() && input.net_events.is_empty());
        }
        let bytes = session.seal().unwrap();
        let tape = decode(&bytes).unwrap();
        assert_eq!(tape.frames.len(), inputs.len());
        let supply = crate::unit_wire::resumable_complete_state::encode_processed_input_supply(&session).unwrap();
        let check = |bytes: &[u8]| crate::unit_wire::resumable_complete_state::check_processed_input_supply(bytes, &session);
        check(&supply).unwrap();
        assert!(check(&empty_supply).is_err(), "earlier prefix must not replace the current prefix");
        let mut corrupted = supply.clone();
        *corrupted.last_mut().unwrap() ^= 1;
        assert!(check(&corrupted).is_err());
        let mut suffix = supply.clone(); suffix.push(0);
        assert!(check(&suffix).is_err());
        assert!(check(&supply[..supply.len()-1]).is_err());
        let preceding_hash = session.summary()["state_hash"].clone();
        assert!(session.step(&LogicalSnapshot {
            madi: 3, events: vec![], held: request.binding.initial_held.clone(),
        }).unwrap_err().contains("E_LOGICAL_INPUT_ORDER"));
        assert_eq!(session.core_inputs.len(), 2);
        assert_eq!(session.summary()["state_hash"], preceding_hash);
        assert!(session.retained_core_inputs().is_err());
        assert!(session.seal().is_err());
        assert!(crate::unit_wire::resumable_complete_state::encode_processed_input_supply(&session).is_err());
    }

    #[test]
    fn a3_rd136_logical_session_preserves_historical_tables_in_record_and_replay() {
        // A3-TIME-P1: same projector; no new input tape or source lowering.
        let source=include_str!("../../solutions/seamgrim_ui_mvp/samples/alpha3_table_time.ddn");
        let mut request=request(source,false);
        request.snapshots=(0..3).map(|madi|LogicalSnapshot {madi,events:vec![],held:request.binding.initial_held.clone()}).collect();
        let (bytes,summary)=record_source(source,&request).unwrap();
        let relations=&summary["bogae_projection"]["candidates"][0]["artifact"]["relations"];
        assert_eq!(relations[1]["resolved_madi"],0);
        assert_eq!(relations[1]["view"]["table_projection"]["tables"][0]["rows"].as_array().unwrap().len(),1);
        assert_eq!(relations[1]["view"]["table_projection"]["tables"][0]["rows"][0]["cells"][0]["value_text"],"1");
        assert_eq!(replay_source(source,&bytes,&request.binding).unwrap(),summary);
        let mut live=LogicalInputSession::new(source,request.binding.clone()).unwrap();
        for input in &request.snapshots {
            live.step(input).unwrap();
        }
        assert_eq!(live.summary(),summary,"live historical tables differ from batch recording");
        assert_eq!(live.seal().unwrap(),bytes,"presentation history changed input tape bytes");
    }

    fn tape() -> LogicalTape {
        let binding = LogicalInputBinding {
            source_profile: LOGICAL_INPUT_PROFILE.into(),
            input_schema: "test.schema@1".into(),
            channel_registry: "test.channels@1".into(),
            source_revision: "test.source".into(),
            initial_world: "test.world".into(),
            initial_rng: 7,
            recording_policy: "test.recording@1".into(),
            integrity_policy: "test.integrity@1".into(),
            madi_hz: 60,
            dt_raw: crate::input_receipt::alpha_host_dt().raw_i64(),
            channels: BTreeMap::from([("왼쪽".into(), ChannelKind::Control)]),
            initial_held: BTreeMap::from([("왼쪽".into(), false)]),
        };
        let input = LogicalSnapshot {
            madi: 0,
            held: BTreeMap::from([("왼쪽".into(), false)]),
            events: vec![
                LogicalEvent {
                    ordinal: 0,
                    channel: "왼쪽".into(),
                    event: EventKind::Press,
                },
                LogicalEvent {
                    ordinal: 1,
                    channel: "왼쪽".into(),
                    event: EventKind::Release,
                },
            ],
        };
        LogicalTape {
            binding,
            frames: vec![LogicalFrame {
                input_digest: snapshot_digest(&input).unwrap(),
                input,
                accepted_world_digest: "test.accepted.world".into(),
            }],
            termination: Termination::Complete,
            accepted_madi_count: 1,
            accepted_event_count: 2,
            failure_reason: None,
        }
    }

    #[test]
    fn canonical_roundtrip_and_order_are_preserved() {
        let tape = tape();
        let bytes = encode(&tape).unwrap();
        assert_eq!(encode(&decode(&bytes).unwrap()).unwrap(), bytes);
        let decoded = admit_replay(&bytes, &tape.binding).unwrap();
        let query = query(&decoded.binding, &decoded.frames[0].input).unwrap();
        assert!(!query.is_held(&"왼쪽".into()).unwrap());
        assert_eq!(query.events()[0].kind, LogicalInputEventKind::Press);
        assert_eq!(query.events()[1].kind, LogicalInputEventKind::Release);
    }

    #[test]
    fn corruption_length_and_old_versions_are_rejected() {
        let bytes = encode(&tape()).unwrap();
        let mut corrupt = bytes.clone();
        corrupt[MAGIC.len() + 8] ^= 1;
        assert!(decode(&corrupt)
            .unwrap_err()
            .starts_with("E_LOGICAL_INPUT_SEAL"));
        assert!(decode(&bytes[..bytes.len() - 1]).is_err());
        let mut trailing = bytes;
        trailing.push(0);
        assert!(decode(&trailing).is_err());
        assert!(decode(b"DDN_INPUT_TAPE_V2\n")
            .unwrap_err()
            .starts_with("E_LOGICAL_INPUT_VERSION"));
    }

    #[test]
    fn text_slots_are_reserved_but_alpha2_rejects_them() {
        let mut tape = tape();
        tape.frames[0].input.events[0].event = EventKind::CommittedText("한글".into());
        assert!(encode(&tape)
            .unwrap_err()
            .starts_with("E_LOGICAL_INPUT_TEXT_UNSUPPORTED"));
        tape = self::tape();
        tape.binding.channels.insert("글".into(), ChannelKind::Text);
        assert!(encode(&tape)
            .unwrap_err()
            .starts_with("E_LOGICAL_INPUT_TEXT_UNSUPPORTED"));
    }

    #[test]
    fn incomplete_and_wrong_execution_never_admit_replay() {
        let mut tape = tape();
        let expected = tape.binding.clone();
        tape.termination = Termination::Aborted;
        tape.failure_reason = Some("중단".into());
        assert!(admit_replay(&encode(&tape).unwrap(), &expected)
            .unwrap_err()
            .starts_with("E_LOGICAL_INPUT_INCOMPLETE"));
        tape = self::tape();
        tape.binding.initial_rng += 1;
        assert!(admit_replay(&encode(&tape).unwrap(), &expected)
            .unwrap_err()
            .starts_with("E_LOGICAL_INPUT_BINDING_MISMATCH"));
    }

    #[test]
    fn missing_state_order_and_snapshot_digest_fail_closed() {
        let mut tape = tape();
        tape.frames[0].input.events[1].ordinal = 0;
        assert!(encode(&tape)
            .unwrap_err()
            .starts_with("E_LOGICAL_INPUT_ORDER"));
        tape = self::tape();
        tape.frames[0].input.held.clear();
        assert!(encode(&tape)
            .unwrap_err()
            .starts_with("E_LOGICAL_INPUT_SCHEMA"));
        tape = self::tape();
        tape.frames[0].input_digest = "wrong".into();
        assert!(encode(&tape)
            .unwrap_err()
            .starts_with("E_LOGICAL_INPUT_DIGEST"));
    }

    #[test]
    fn settled_state_and_repeat_are_not_invented() {
        let mut tape = tape();
        tape.frames[0].input.held.insert("왼쪽".into(), true);
        assert!(encode(&tape)
            .unwrap_err()
            .starts_with("E_LOGICAL_INPUT_STATE"));
        tape = self::tape();
        tape.frames[0].input.events[0].event = EventKind::Repeat;
        assert!(encode(&tape)
            .unwrap_err()
            .starts_with("E_LOGICAL_INPUT_STATE"));
    }

    #[test]
    fn noncanonical_bytes_fail_even_with_recomputed_checksum() {
        let tape = tape();
        let payload = serde_json::to_vec_pretty(&tape).unwrap();
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&payload);
        bytes.extend_from_slice(&digest(DOMAIN, &bytes));
        assert!(decode(&bytes)
            .unwrap_err()
            .starts_with("E_LOGICAL_INPUT_NONCANONICAL"));
    }

    fn request(source: &str, reverse: bool) -> LogicalRecordRequest {
        let channels: BTreeMap<String, ChannelKind> = BTreeMap::from([
            ("더함".into(), ChannelKind::Control),
            ("곱함".into(), ChannelKind::Control),
        ]);
        let mut binding = tape().binding;
        binding.source_revision =
            format!("sha256:{}", hex::encode(Sha256::digest(source.as_bytes())));
        binding.initial_world =
            crate::runtime_surface::current_state_hash(&ddonirang_core::platform::NuriWorld::new());
        binding.initial_held = channels
            .keys()
            .map(|channel| (channel.clone(), false))
            .collect();
        binding.channels = channels;
        let names = if reverse {
            ["곱함", "더함"]
        } else {
            ["더함", "곱함"]
        };
        let input = LogicalSnapshot {
            madi: 0,
            held: binding
                .channels
                .keys()
                .map(|channel| (channel.clone(), true))
                .collect(),
            events: names
                .iter()
                .enumerate()
                .map(|(ordinal, channel)| LogicalEvent {
                    ordinal: ordinal as u64,
                    channel: (*channel).into(),
                    event: EventKind::Press,
                })
                .collect(),
        };
        LogicalRecordRequest {
            binding,
            snapshots: vec![input],
        }
    }

    #[test]
    fn explicit_source_binding_changes_only_revision_and_never_executes_a_madi() {
        let source = "(매마디)마다 { 값 <- 3. }.";
        let original = request(source, false).binding;
        let changed = "(매마디)마다 { 값 <- 4. }.";
        assert!(LogicalInputSession::new(changed, original.clone()).is_err());
        let prepared = LogicalInputSession::bind_source_for_fresh_run(changed, original.clone()).unwrap();
        let mut expected = original;
        expected.source_revision = format!("sha256:{}", hex::encode(Sha256::digest(changed.as_bytes())));
        assert_eq!(prepared, expected);
        let session = LogicalInputSession::new(changed, prepared).unwrap();
        assert_eq!(session.summary()["madi_count"], 0);
        assert!(session.frames.is_empty());
    }

    #[test]
    fn explicit_source_binding_does_not_repair_invalid_contracts_or_ingress() {
        let source = "(매마디)마다 { 값 <- 3. }.";
        let valid = request(source, false).binding;
        for invalid in 0..4 {
            let mut binding = valid.clone();
            match invalid {
                0 => binding.dt_raw = 1,
                1 => binding.source_profile = "not-approved".into(),
                2 => binding.initial_world = "wrong-world".into(),
                _ => { binding.channels.insert("글자".into(), ChannelKind::Text); }
            }
            assert!(LogicalInputSession::bind_source_for_fresh_run(source, binding).is_err());
        }
        assert!(LogicalInputSession::bind_source_for_fresh_run("\u{feff}1 보여주기.", valid.clone()).is_err());
        assert!(LogicalInputSession::bind_source_for_fresh_run("만약 참 이라면 { 1 보여주기. }.", valid).is_err());
    }

    #[test]
    fn authored_query_executes_ordered_events_and_replays_real_bytes() {
        let source = r#"(매마디)마다 {
            계산값 <- 2.
            사건들 <- () 입력사상.사건들.
            사건들마다 {
                { 사건들.채널 == "더함" }이면 { 계산값 <- 계산값 + 1. }
                아니면 { 계산값 <- 계산값 * 2. }.
            }.
            같은사건들 <- () 입력사상.사건들.
            조작맵 <- () 입력사상.만들기.
            눌림값 <- (조작맵, "더함") 입력사상.동작.
        }."#;
        let normal = request(source, false);
        let (bytes, summary) = record_source(source, &normal).unwrap();
        assert_eq!(summary["resources"]["fixed64"]["계산값"], "6");
        assert_eq!(
            replay_source(source, &bytes, &normal.binding).unwrap(),
            summary
        );
        assert_eq!(
            summary["input_receipts"][0]["input_snapshot_sha256"],
            snapshot_digest(&normal.snapshots[0]).unwrap()
        );
        let (_, reversed) = record_source(source, &request(source, true)).unwrap();
        assert_eq!(reversed["resources"]["fixed64"]["계산값"], "5");
        assert_ne!(summary["state_hash"], reversed["state_hash"]);
        assert_ne!(
            summary["input_receipts"][0]["input_snapshot_sha256"],
            reversed["input_receipts"][0]["input_snapshot_sha256"]
        );
    }

    fn a3_p4_reaction_source() -> &'static str {
        r#"
            채비 { 계산값:수 := 2. }.
            (채널:글, 사건:글, 순서:수, 마디번호:수) 조작알림:알림씨 := { 없음. }.
            계산받는이:임자 := {
                (받은:조작알림) 계산반응:움직씨 {조작알림을 받으면} := {
                    { 받은.채널 == "더함" }이면 { 계산값 <- 계산값 + 1. }
                    아니면 { 계산값 <- 계산값 * 2. }.
                }.
            }.
            ("더함", "조작알림", "계산받는이") 입력사상.알림결속.
            ("곱함", "조작알림", "계산받는이") 입력사상.알림결속.
            (매마디)마다 { 마디결과 <- 계산값. }.
        "#
    }

    #[test]
    fn a3_p4_authored_reactions_follow_event_order_before_madi_and_replay() {
        // A3-P4: actual source frontdoor, existing logical tape, not harness lowering.
        let source = a3_p4_reaction_source();
        for (reverse, expected) in [(false, "6"), (true, "5")] {
            let input = request(source, reverse);
            let (bytes, summary) = record_source(source, &input).unwrap();
            assert_eq!(summary["resources"]["fixed64"]["계산값"], expected);
            assert_eq!(summary["resources"]["fixed64"]["마디결과"], expected);
            assert_eq!(replay_source(source, &bytes, &input.binding).unwrap(), summary);
        }
    }

    #[test]
    fn a3_p4_wrong_alert_pin_rejects_before_the_logical_madi() {
        // N1/R4 rejects a statically wrong declared pin before a runner or
        // accepted input prefix exists. Runtime rollback remains separately
        // exercised by a3_p4_second_reaction_failure_publishes_nothing_and_cannot_seal.
        let source = format!(r#"
(값:수) 오류알림:알림씨 := {{ 없음. }}.
오류받는이:임자 := {{
  (받은:오류알림) 오류반응:움직씨 {{오류알림을 받으면}} := {{ 게시값 <- 7. }}.
}}.
{}"#, a3_p4_reaction_source()).replace(
            "마디결과 <- 계산값.",
            r#"마디결과 <- 계산값. (철수)의 ((값="틀린씨") 오류알림) ~> 오류받는이."#);
        let input = request(&source, false);
        let error=LogicalInputSession::new(&source,input.binding).err()
            .expect("statically wrong payload must not create a logical session");
        assert!(error.contains("E_RUNTIME_TYPE_MISMATCH"),"{error}");
        assert!(error.contains("핀=값") && error.contains("셈수") && error.contains("글"),"{error}");
    }

    #[test]
    fn a3_p4_second_reaction_failure_publishes_nothing_and_cannot_seal() {
        let source = a3_p4_reaction_source().replace("계산값 * 2", "1 / 0");
        let input = request(&source, false);
        let mut session = LogicalInputSession::new(&source, input.binding).unwrap();
        let before = session.summary();
        assert!(session.step(&input.snapshots[0]).is_err());
        let after = session.summary();
        for key in ["state_hash", "resources", "frames", "input_receipts", "madi_count", "bogae_projection", "public_observation_table_projection"] {
            assert_eq!(before[key], after[key], "failed reaction published {key}");
        }
        assert_eq!(after["outcome"], "ABORTED");
        assert!(session.seal().unwrap_err().contains("SESSION_ABORTED"));
    }

    #[test]
    fn a3_p4_reaction_reads_event_prefix_state_without_consuming_events() {
        let source = a3_p4_reaction_source().replace("계산값 <- 계산값 + 1.", r#"
            계산값 <- 계산값 + 1.
            조작맵 <- () 입력사상.만들기.
            중간눌림 <- (조작맵, "곱함") 입력사상.동작.
            읽은사건들 <- () 입력사상.사건들.
        "#);
        // R1.1 changes only Bool storage/projection, not event-prefix semantics.
        for (reverse, held) in [(false, false), (true, true)] {
            let input = request(&source, reverse);
            let (bytes, summary) = record_source(&source, &input).unwrap();
            assert_eq!(summary["resources"]["value_json"]["중간눌림"], held);
            assert_eq!(summary["resources"]["value_json"]["읽은사건들"].as_array().unwrap().len(), 2);
            assert_eq!(replay_source(&source, &bytes, &input.binding).unwrap(), summary);
            assert_eq!(summary["frames"][0]["input"]["events"].as_array().unwrap().len(), 2);
        }
    }

    #[test]
    fn legacy_profile_and_physical_reads_are_not_silent_fallbacks() {
        use crate::ddn_runtime::{DdnProgram, DdnRunner};
        let source = "(매마디)마다 { 사건들 <- () 입력사상.사건들. }.";
        let program = DdnProgram::from_v25_source(source, "physical-profile.ddn").unwrap();
        let mut runner = DdnRunner::new(program, "매마디");
        let input = ddonirang_core::InputSnapshot {
            tick_id: 0,
            dt: crate::input_receipt::alpha_host_dt(),
            rng_seed: 0,
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: vec![],
            net_events: vec![],
            frame_source: ddonirang_core::InputSource::Person,
        };
        assert!(runner
            .run_update(
                &ddonirang_core::platform::NuriWorld::new(),
                &input,
                &Default::default()
            )
            .err()
            .unwrap()
            .contains("E_LOGICAL_INPUT_PROFILE_REQUIRED"));
        for source in [
            "(매마디)마다 { 입력키 보여주기. }.",
            "(매마디)마다 { (\"left\") 눌렸나 보여주기. }.",
        ] {
            assert!(record_source(source, &request(source, false))
                .unwrap_err()
                .contains("E_LOGICAL_INPUT_PHYSICAL_UNSUPPORTED"));
        }
    }

    #[test]
    fn replay_world_mismatch_and_mid_madi_failure_return_no_success_record() {
        let source = "(매마디)마다 { 사건들 <- () 입력사상.사건들. 계산값 <- 6. }.";
        let request = request(source, false);
        let (bytes, _) = record_source(source, &request).unwrap();
        let mut tape = decode(&bytes).unwrap();
        tape.frames[0].accepted_world_digest = "blake3:wrong".into();
        assert!(
            replay_source(source, &encode(&tape).unwrap(), &request.binding)
                .unwrap_err()
                .contains("E_LOGICAL_INPUT_WORLD_MISMATCH")
        );
        let failing =
            "(매마디)마다 { 사건들 <- () 입력사상.사건들. 6 보여주기. 계산값 <- 1 / 0. }.";
        assert!(record_source(failing, &self::request(failing, false)).is_err());
    }

    #[test]
    fn live_steps_and_batch_recording_share_exact_world_receipts_and_bytes() {
        let source = "(매마디)마다 { 사건들 <- () 입력사상.사건들. 조작맵 <- () 입력사상.만들기. 눌림값 <- (조작맵, \"더함\") 입력사상.동작. }.";
        let mut request = request(source, false);
        request.snapshots.push(LogicalSnapshot {
            madi: 1, events: vec![LogicalEvent {
                ordinal: 0, channel: "더함".into(), event: EventKind::Repeat,
            }], held: request.snapshots[0].held.clone(),
        });
        request.snapshots.push(LogicalSnapshot {
            madi: 2, events: vec![LogicalEvent {
                ordinal: 0, channel: "더함".into(), event: EventKind::Release,
            }], held: BTreeMap::from([("더함".into(), false), ("곱함".into(), true)]),
        });
        let mut live = LogicalInputSession::new(source, request.binding.clone()).unwrap();
        for input in &request.snapshots {
            let step = live.step(input).unwrap();
            assert_eq!(step["frame"]["input"]["madi"], input.madi);
        }
        let (bytes, summary) = record_source(source, &request).unwrap();
        assert_eq!(live.summary(), summary);
        assert_eq!(live.seal().unwrap(), bytes);
        assert_eq!(replay_source(source, &bytes, &request.binding).unwrap(), summary);
    }

    #[test]
    fn live_failed_madi_preserves_accepted_prefix_but_cannot_continue_or_seal() {
        // Keep both branches in the same explicit numeric family so the
        // intended second-madi division failure reaches the runtime owner.
        let source = "(매마디)마다 { 값 <- 6.0. 사건들 <- () 입력사상.사건들. 사건들마다 { { 사건들.채널 == \"곱함\" }이면 { 값 <- 1.0 / 0.0. }. }. }. 짜임 := (관계들=차림[(보기=(() 표준보기.결과표)) 표준보개.으뜸]) 표준보개.짜임. (값:값) 보임.";
        let mut request = request(source, false);
        request.snapshots[0].events.truncate(1);
        request.snapshots[0].held.insert("곱함".into(), false);
        let mut live = LogicalInputSession::new(source, request.binding.clone()).unwrap();
        live.step(&request.snapshots[0]).unwrap();
        let before = live.summary();
        let failing = LogicalSnapshot {
            madi: 1, events: vec![LogicalEvent {
                ordinal: 0, channel: "곱함".into(), event: EventKind::Press,
            }], held: BTreeMap::from([("더함".into(), true), ("곱함".into(), true)]),
        };
        assert!(live.step(&failing).is_err());
        let after = live.summary();
        assert_eq!(before["bogae_projection"]["status"], "ready");
        for key in ["state_hash", "resources", "frames", "input_receipts", "madi_count", "bogae_projection", "public_observation_table_projection"] {
            assert_eq!(after[key], before[key], "failed madi changed {key}");
        }
        assert_eq!(after["outcome"], "ABORTED");
        assert!(live.step(&failing).unwrap_err().contains("SESSION_ABORTED"));
        assert!(live.seal().unwrap_err().contains("SESSION_ABORTED"));
        let mut restarted = LogicalInputSession::new(source, request.binding).unwrap();
        restarted.step(&request.snapshots[0]).unwrap();
        assert!(restarted.seal().is_ok());
    }
}
