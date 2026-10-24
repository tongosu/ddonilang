//! Alpha4 PROVISIONAL typed manipulation ingress/codec. Not an SSOT surface.
//! V1 logical tape remains byte-identical and never silently falls back here.
use ddonirang_core::input::{
    LogicalChannelKind, LogicalInputEvent, LogicalInputEventKind, LogicalInputQuery,
    ValueEventPhase,
};
use ddonirang_core::Nuri;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};
pub const PROFILE: &str = "v1-core-v25-manipulation-alpha4-provisional";
const MAGIC: &[u8] = b"DDN_MANIPULATION_INPUT_TAPE_V2_PROVISIONAL\n";
const DOMAIN: &[u8] = b"ddn.manipulation_input_tape.v2.provisional\0";
const SNAPSHOT_DOMAIN: &[u8] = b"ddn.manipulation_input_snapshot.v2.provisional\0";
const MAX_BYTES: usize = 67_108_864;
const MAX_EVENTS_PER_MADI: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueDomain {
    pub unit: String,
    pub min_raw: String,
    pub max_raw: String,
    pub step_raw: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum ChannelContract {
    Control,
    Number {
        target: String,
        domain: ValueDomain,
    },
    Coordinate {
        target: String,
        axes: [ValueDomain; 2],
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManipulationBinding {
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
    pub channels: BTreeMap<String, ChannelContract>,
    pub initial_held: BTreeMap<String, bool>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Set,
    Begin,
    Move,
    End,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", deny_unknown_fields)]
pub enum EventKind {
    Press,
    Repeat,
    Release,
    Number { raw: String, phase: Phase },
    Coordinate { raw: [String; 2], phase: Phase },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub ordinal: u64,
    pub channel: String,
    pub event: EventKind,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManipulationSnapshot {
    pub madi: u64,
    pub events: Vec<Event>,
    pub held: BTreeMap<String, bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub input: ManipulationSnapshot,
    pub input_digest: String,
    pub accepted_world_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tape {
    pub binding: ManipulationBinding,
    pub frames: Vec<Frame>,
    pub termination: crate::logical_input_tape::Termination,
    pub accepted_madi_count: u64,
    pub accepted_event_count: u64,
    pub failure_reason: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordRequest {
    pub binding: ManipulationBinding,
    pub snapshots: Vec<ManipulationSnapshot>,
}
fn error(code: &str, text: &str) -> String {
    format!("{code}: {text}")
}
fn hash(domain: &[u8], bytes: &[u8]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(domain);
    digest.update(bytes);
    digest.finalize().into()
}
fn raw(text: &str) -> Result<i64, String> {
    let value = text.parse::<i64>().map_err(|_| {
        error(
            "E_MANIPULATION_VALUE",
            "정확한 Fixed64 raw 십진 정수가 필요합니다",
        )
    })?;
    if value.to_string() != text {
        return Err(error(
            "E_MANIPULATION_VALUE",
            "raw 값은 정본 십진 문자열이어야 합니다",
        ));
    }
    Ok(value)
}
fn domain(domain: &ValueDomain) -> Result<(i64, i64, i64), String> {
    // Unit tags only; no inferred units or conversion. Physical unit arithmetic remains separate.
    if !matches!(domain.unit.as_str(), "1" | "rad" | "m") {
        return Err(error(
            "E_MANIPULATION_UNIT_UNSUPPORTED",
            "지원하지 않는 단위 표지입니다",
        ));
    }
    let (min, max, step) = (
        raw(&domain.min_raw)?,
        raw(&domain.max_raw)?,
        raw(&domain.step_raw)?,
    );
    if min > max || step <= 0 {
        return Err(error(
            "E_MANIPULATION_DOMAIN",
            "최소·최대·간격 계약이 잘못됐습니다",
        ));
    }
    Ok((min, max, step))
}
fn value(domain_value: &ValueDomain, text: &str) -> Result<i64, String> {
    let (min, max, step) = domain(domain_value)?;
    let parsed = raw(text)?;
    if parsed < min
        || parsed > max
        || (i128::from(parsed) - i128::from(min)) % i128::from(step) != 0
    {
        return Err(error(
            "E_MANIPULATION_RANGE",
            "값이 선언된 범위·간격에 맞지 않습니다",
        ));
    }
    Ok(parsed)
}
fn phase(value: Phase) -> ValueEventPhase {
    match value {
        Phase::Set => ValueEventPhase::Set,
        Phase::Begin => ValueEventPhase::Begin,
        Phase::Move => ValueEventPhase::Move,
        Phase::End => ValueEventPhase::End,
    }
}
pub fn snapshot_digest(snapshot: &ManipulationSnapshot) -> Result<String, String> {
    let bytes = serde_json::to_vec(snapshot)
        .map_err(|_| error("E_MANIPULATION_ENCODING", "입력 정본화 실패"))?;
    Ok(hex::encode(hash(SNAPSHOT_DOMAIN, &bytes)))
}
fn validate_binding(binding: &ManipulationBinding) -> Result<(), String> {
    if binding.source_profile != PROFILE
        || binding.madi_hz != crate::input_receipt::alpha2_host_madi_hz()
        || binding.dt_raw != crate::input_receipt::alpha_host_dt().raw_i64()
    {
        return Err(error(
            "E_MANIPULATION_PROFILE",
            "조작 프로필·마디율·dt가 다릅니다",
        ));
    }
    if [
        &binding.input_schema,
        &binding.channel_registry,
        &binding.source_revision,
        &binding.initial_world,
        &binding.recording_policy,
        &binding.integrity_policy,
    ]
    .iter()
    .any(|text| text.is_empty())
        || binding.channels.is_empty()
    {
        return Err(error(
            "E_MANIPULATION_BINDING",
            "조작 실행 계약이 비어 있습니다",
        ));
    }
    for (channel, contract) in &binding.channels {
        if channel.is_empty() {
            return Err(error("E_MANIPULATION_BINDING", "채널 이름이 비었습니다"));
        }
        match contract {
            ChannelContract::Control => {}
            ChannelContract::Number {
                target,
                domain: range,
            } => {
                if target.is_empty() {
                    return Err(error("E_MANIPULATION_TARGET", "대상이 비었습니다"));
                }
                domain(range)?;
            }
            ChannelContract::Coordinate { target, axes } => {
                if target.is_empty() {
                    return Err(error("E_MANIPULATION_TARGET", "대상이 비었습니다"));
                }
                for axis in axes {
                    domain(axis)?;
                }
            }
        }
    }
    Ok(())
}
pub fn query(
    binding: &ManipulationBinding,
    snapshot: &ManipulationSnapshot,
) -> Result<LogicalInputQuery<String>, String> {
    validate_binding(binding)?;
    if snapshot.events.len() > MAX_EVENTS_PER_MADI {
        return Err(error(
            "E_MANIPULATION_OVERFLOW",
            "조작 사건 한도를 넘었습니다. 마디 전체를 거부합니다",
        ));
    }
    let channels = binding
        .channels
        .iter()
        .map(|(name, contract)| {
            (
                name.clone(),
                match contract {
                    ChannelContract::Control => LogicalChannelKind::Control,
                    ChannelContract::Number { .. } => LogicalChannelKind::Number,
                    ChannelContract::Coordinate { .. } => LogicalChannelKind::Coordinate,
                },
            )
        })
        .collect();
    let mut events = Vec::with_capacity(snapshot.events.len());
    for (ordinal, event) in snapshot.events.iter().enumerate() {
        if event.ordinal != ordinal as u64 {
            return Err(error(
                "E_MANIPULATION_ORDER",
                "마디 사건 순번이 연속되지 않습니다",
            ));
        }
        let contract = binding
            .channels
            .get(&event.channel)
            .ok_or_else(|| error("E_MANIPULATION_CHANNEL", "모르는 채널입니다"))?;
        let kind = match (&event.event, contract) {
            (EventKind::Press, ChannelContract::Control) => LogicalInputEventKind::Press,
            (EventKind::Repeat, ChannelContract::Control) => LogicalInputEventKind::Repeat,
            (EventKind::Release, ChannelContract::Control) => LogicalInputEventKind::Release,
            (EventKind::Number { raw, phase: p }, ChannelContract::Number { domain, .. }) => {
                LogicalInputEventKind::Number(value(domain, raw)?, phase(*p))
            }
            (EventKind::Coordinate { raw, phase: p }, ChannelContract::Coordinate { axes, .. }) => {
                LogicalInputEventKind::Coordinate(
                    [value(&axes[0], &raw[0])?, value(&axes[1], &raw[1])?],
                    phase(*p),
                )
            }
            _ => {
                return Err(error(
                    "E_MANIPULATION_TYPE",
                    "채널 선언과 사건 값 씨가 다릅니다",
                ))
            }
        };
        events.push(LogicalInputEvent {
            receive_ordinal: event.ordinal,
            channel: event.channel.clone(),
            kind,
        });
    }
    LogicalInputQuery::from_projection(snapshot.madi, channels, events, snapshot.held.clone())
        .map_err(|_| error("E_MANIPULATION_STATE", "채널·사건·눌린 상태가 다릅니다"))
}
fn transition(
    snapshot: &ManipulationSnapshot,
    previous: &BTreeMap<String, bool>,
    active: &BTreeSet<String>,
) -> Result<(BTreeMap<String, bool>, BTreeSet<String>), String> {
    let mut held = previous.clone();
    let mut active = active.clone();
    for event in &snapshot.events {
        match &event.event {
            EventKind::Press => {
                *held
                    .get_mut(&event.channel)
                    .ok_or_else(|| error("E_MANIPULATION_STATE", "조작 상태가 없습니다"))? = true
            }
            EventKind::Release => {
                *held
                    .get_mut(&event.channel)
                    .ok_or_else(|| error("E_MANIPULATION_STATE", "조작 상태가 없습니다"))? = false
            }
            EventKind::Repeat if held.get(&event.channel) == Some(&true) => {}
            EventKind::Repeat => {
                return Err(error(
                    "E_MANIPULATION_STATE",
                    "누르지 않은 채널을 반복했습니다",
                ))
            }
            EventKind::Number { phase, .. } | EventKind::Coordinate { phase, .. } => match phase {
                Phase::Set if !active.contains(&event.channel) => {}
                Phase::Begin if active.insert(event.channel.clone()) => {}
                Phase::Move if active.contains(&event.channel) => {}
                Phase::End if active.remove(&event.channel) => {}
                _ => {
                    return Err(error(
                        "E_MANIPULATION_PHASE",
                        "조작 시작·이동·끝 순서가 잘못됐습니다",
                    ))
                }
            },
        }
    }
    if held != snapshot.held {
        return Err(error("E_MANIPULATION_STATE", "사건 뒤 상태가 다릅니다"));
    }
    Ok((held, active))
}
pub(crate) fn validate(tape: &Tape) -> Result<(), String> {
    query(
        &tape.binding,
        &ManipulationSnapshot {
            madi: 0,
            events: vec![],
            held: tape.binding.initial_held.clone(),
        },
    )?;
    let mut held = tape.binding.initial_held.clone();
    let mut active = BTreeSet::new();
    let mut count = 0u64;
    for (madi, frame) in tape.frames.iter().enumerate() {
        if frame.input.madi != madi as u64 {
            return Err(error(
                "E_MANIPULATION_ORDER",
                "기록 마디가 연속되지 않습니다",
            ));
        }
        query(&tape.binding, &frame.input)?;
        (held, active) = transition(&frame.input, &held, &active)?;
        if frame.input_digest != snapshot_digest(&frame.input)?
            || frame.accepted_world_digest.is_empty()
        {
            return Err(error("E_MANIPULATION_DIGEST", "입력·세계 증거가 다릅니다"));
        }
        count = count
            .checked_add(frame.input.events.len() as u64)
            .ok_or_else(|| error("E_MANIPULATION_COUNT", "사건 수 초과"))?;
    }
    let complete = tape.termination == crate::logical_input_tape::Termination::Complete;
    if tape.accepted_madi_count != tape.frames.len() as u64
        || count != tape.accepted_event_count
        || complete && (tape.failure_reason.is_some() || !active.is_empty())
        || !complete
            && tape
                .failure_reason
                .as_ref()
                .is_none_or(|text| text.is_empty())
    {
        return Err(error(
            "E_MANIPULATION_SEAL",
            "완결·누락된 끝·기록 수가 다릅니다",
        ));
    }
    Ok(())
}
pub fn encode(tape: &Tape) -> Result<Vec<u8>, String> {
    validate(tape)?;
    let payload = serde_json::to_vec(tape)
        .map_err(|_| error("E_MANIPULATION_ENCODING", "기록 인코딩 실패"))?;
    if payload.len() > MAX_BYTES {
        return Err(error("E_MANIPULATION_SIZE", "입력 기록이 너무 큽니다"));
    }
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&payload);
    bytes.extend_from_slice(&hash(DOMAIN, &bytes));
    Ok(bytes)
}
pub fn decode(bytes: &[u8]) -> Result<Tape, String> {
    let prefix = MAGIC.len() + 8;
    if !bytes.starts_with(MAGIC) || bytes.len() < prefix + 32 {
        return Err(error(
            "E_MANIPULATION_VERSION",
            "새 조작 기록 버전이 아니거나 잘렸습니다",
        ));
    }
    let length = u64::from_le_bytes(bytes[MAGIC.len()..prefix].try_into().unwrap());
    let end = bytes.len() - 32;
    if length > MAX_BYTES as u64 || length != (end - prefix) as u64 {
        return Err(error("E_MANIPULATION_SIZE", "기록 길이가 다릅니다"));
    }
    if hash(DOMAIN, &bytes[..end]).as_slice() != &bytes[end..] {
        return Err(error("E_MANIPULATION_SEAL", "기록 봉인이 다릅니다"));
    }
    let tape: Tape = serde_json::from_slice(&bytes[prefix..end])
        .map_err(|_| error("E_MANIPULATION_ENCODING", "기록 필드가 다릅니다"))?;
    if encode(&tape)? != bytes {
        return Err(error(
            "E_MANIPULATION_NONCANONICAL",
            "정본 바이트가 아닙니다",
        ));
    }
    Ok(tape)
}

pub struct ManipulationSession {
    result_emissions:crate::replay_evidence::ResultEmissionSinkV1,
    binding: ManipulationBinding,
    runner: crate::ddn_runtime::DdnRunner,
    world: ddonirang_core::platform::DetNuri,
    held: BTreeMap<String, bool>,
    active: BTreeSet<String>,
    frames: Vec<Frame>,
    receipts: Vec<serde_json::Value>,
    failure: Option<String>,
    last_projection: Option<serde_json::Value>,
    display_outputs: Vec<crate::output_display::OutputDisplay>,
    observations: Vec<serde_json::Value>,
    last_table: Option<serde_json::Value>,
}
impl ManipulationSession {
    pub(crate) fn recording_generation(&self)->crate::replay_evidence::ExecutionGenerationV1 {
        self.result_emissions.generation()
    }
    pub(crate) fn check_live_recording_end(&self)->Result<(),String> {
        if self.failure.is_some(){return Err(error("E_MANIPULATION_ABORTED","실패한 기록은 정상 마칠 수 없습니다"));}
        if !self.active.is_empty(){return Err(error("E_INPUT_FINISH_NOT_READY","시작한 조작이 끝나지 않았습니다. 미처리 끝 사건을 자동 실행하지 않습니다"));}
        Ok(())
    }
    pub(crate) fn accepted_prefix_v3(&self) -> (&ManipulationBinding, &[Frame], Option<&str>) {
        (&self.binding, &self.frames, self.failure.as_deref())
    }
    pub(crate) fn replay_v3_frame(&mut self, frame: &Frame) -> Result<serde_json::Value, String> {
        self.step_expected(&frame.input, Some(frame))
    }
    pub fn bind_source_for_fresh_run(
        source: &str,
        mut binding: ManipulationBinding,
    ) -> Result<ManipulationBinding, String> {
        binding.source_revision =
            format!("sha256:{}", hex::encode(Sha256::digest(source.as_bytes())));
        Self::new(source, binding.clone())?;
        Ok(binding)
    }
    pub fn new(source: &str, binding: ManipulationBinding) -> Result<Self, String> {
        query(
            &binding,
            &ManipulationSnapshot {
                madi: 0,
                events: vec![],
                held: binding.initial_held.clone(),
            },
        )?;
        let world = ddonirang_core::platform::DetNuri::new();
        if binding.source_revision
            != format!("sha256:{}", hex::encode(Sha256::digest(source.as_bytes())))
            || binding.initial_world != crate::runtime_surface::current_state_hash(world.world())
        {
            return Err(error(
                "E_MANIPULATION_BINDING_MISMATCH",
                "원문·초기 세계가 다릅니다",
            ));
        }
        let program =
            crate::ddn_runtime::DdnProgram::from_v25_source(source, "manipulation-input.ddn")?;
        crate::runtime_surface::prepare_supported_source_for_execution_with_program(
            source,
            "manipulation-input.ddn",
            &program,
        )
        .map_err(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))?;
        Ok(Self {
            held: binding.initial_held.clone(),
            result_emissions:Default::default(),
            binding,
            runner: crate::ddn_runtime::DdnRunner::new(program, "매마디").with_bogae_activation(),
            world,
            active: BTreeSet::new(),
            frames: vec![],
            receipts: vec![],
            failure: None,
            last_projection: None,
            display_outputs: vec![],
            observations: vec![],
            last_table: None,
        })
    }
    pub fn reject_input(&mut self, reason: String) {
        if self.failure.is_none() {
            self.failure = Some(reason);
        }
    }
    pub fn committed_world(&self) -> &ddonirang_core::platform::NuriWorld {
        self.world.world()
    }
    pub fn step(&mut self, input: &ManipulationSnapshot) -> Result<serde_json::Value, String> {
        self.step_expected(input, None)
    }
    fn step_expected(
        &mut self,
        input: &ManipulationSnapshot,
        expected: Option<&Frame>,
    ) -> Result<serde_json::Value, String> {
        if self.failure.is_some() {
            return Err(error(
                "E_MANIPULATION_SESSION_ABORTED",
                "중단된 조작 실행입니다",
            ));
        }
        let result = self.prepare_and_commit(input, expected);
        if let Err(reason) = &result {
            self.failure = Some(reason.clone());
        }
        result
    }
    fn prepare_and_commit(
        &mut self,
        input: &ManipulationSnapshot,
        expected: Option<&Frame>,
    ) -> Result<serde_json::Value, String> {
        if input.madi != self.frames.len() as u64 {
            return Err(error(
                "E_MANIPULATION_ORDER",
                "실행 마디가 연속되지 않습니다",
            ));
        }
        query(&self.binding, input)?;
        let (held, active) = transition(input, &self.held, &self.active)?;
        let (output, receipt) = self.runner.run_update_manipulation_with_initial_held(
            self.world.world(),
            &self.binding,
            input,
            &HashMap::new(),
            Some(&self.held),
        )?;
        self.world.world().validate_typed_resource_patch(&output.patch)
            .map_err(|reason|format!("E_RESULT_WORLD_PATCH_REJECTED: {reason}"))?;
        let pending_outputs=self.result_emissions.prepare_madi(output.result_emissions,input.madi)?;
        let mut candidate = ddonirang_core::platform::DetNuri::new();
        *candidate.world_mut() = self.world.world().clone();
        candidate.apply_patch(
            &output.patch,
            input.madi,
            &mut ddonirang_core::VecSignalSink::default(),
        );
        let frame = Frame {
            input: input.clone(),
            input_digest: snapshot_digest(input)?,
            accepted_world_digest: crate::runtime_surface::current_state_hash(candidate.world()),
        };
        if expected.is_some_and(|value| value != &frame) {
            return Err(error(
                "E_MANIPULATION_WORLD_MISMATCH",
                "재생 입력·세계 해시가 다릅니다",
            ));
        }
        let observations = crate::deterministic_debug::public_observation_events_json(
            &output.public_observations,
        )?;
        let observation_frame = serde_json::json!({"committed_madi":input.madi,"state_hash":frame.accepted_world_digest,"input_snapshot":input,"public_observations":observations});
        let run_id = format!("manipulation:{}", self.binding.source_revision);
        let trace = crate::deterministic_debug::build_public_observation_trace(
            &serde_json::json!({"run_id":run_id,"source_identity":self.binding.source_revision,
            "revision_identity":self.binding.source_revision,"run_status":"completed","frames":[observation_frame]}),
        )?;
        let table = crate::deterministic_debug::build_public_observation_table_projection(
            &trace["trace"],
            &run_id,
        )?;
        let mut projection = output.bogae_projection;
        if let Some(projection) = projection
            .as_mut()
            .filter(|value| value["schema"] == "ddn.bogae_projection.v2")
        {
            let mut history = self.observations.clone();
            history.push(observation_frame.clone());
            crate::bogae_time::resolve_result_tables(
                projection,
                &history,
                &self.binding.source_revision,
                &self.binding.source_revision,
            )?;
        }
        let mut result = serde_json::json!({"frame":frame,"state_hash":frame.accepted_world_digest,"input_receipt":receipt,
            "resources":crate::runtime_surface::serialize_world_resources(candidate.world()),"public_observation_table_projection":table});
        crate::bogae_projection::attach(&mut result, projection.as_ref());
        self.result_emissions.commit_madi(pending_outputs)?;
        self.display_outputs.extend(output.display_outputs);
        crate::output_display::retain_current(&mut self.display_outputs, &result["resources"]);
        crate::output_display::attach(&mut result, &self.display_outputs);
        self.world = candidate;
        self.held = held;
        self.active = active;
        self.frames.push(frame);
        self.receipts.push(receipt);
        self.observations.push(observation_frame);
        self.last_table = Some(table);
        self.last_projection = projection;
        Ok(result)
    }
    pub fn summary(&self) -> serde_json::Value {
        let mut result = serde_json::json!({"input_profile":PROFILE,"provisional":true,"madi_count":self.frames.len(),"frames":self.frames,"input_receipts":self.receipts,
            "state_hash":crate::runtime_surface::current_state_hash(self.world.world()),"resources":crate::runtime_surface::serialize_world_resources(self.world.world())});
        crate::bogae_projection::attach(&mut result, self.last_projection.as_ref());
        crate::output_display::attach(&mut result, &self.display_outputs);
        if let Some(table) = &self.last_table {
            result["public_observation_table_projection"] = table.clone();
        }
        if let Some(failure) = &self.failure {
            result["outcome"] = serde_json::json!("ABORTED");
            result["failure"] = serde_json::json!(failure);
        }
        result
    }
    pub fn seal(&self) -> Result<Vec<u8>, String> {
        if self.failure.is_some() {
            return Err(error(
                "E_MANIPULATION_SESSION_ABORTED",
                "중단된 실행은 성공 기록을 저장하지 않습니다",
            ));
        }
        encode(&Tape {
            binding: self.binding.clone(),
            frames: self.frames.clone(),
            termination: crate::logical_input_tape::Termination::Complete,
            accepted_madi_count: self.frames.len() as u64,
            accepted_event_count: self
                .frames
                .iter()
                .map(|frame| frame.input.events.len() as u64)
                .sum(),
            failure_reason: None,
        })
    }
}
pub fn record_source(
    source: &str,
    request: &RecordRequest,
) -> Result<(Vec<u8>, serde_json::Value), String> {
    let mut session = ManipulationSession::new(source, request.binding.clone())?;
    for snapshot in &request.snapshots {
        session.step(snapshot)?;
    }
    Ok((session.seal()?, session.summary()))
}
pub fn replay_source(
    source: &str,
    bytes: &[u8],
    binding: &ManipulationBinding,
) -> Result<serde_json::Value, String> {
    let tape = decode(bytes)?;
    if tape.termination != crate::logical_input_tape::Termination::Complete {
        return Err(error(
            "E_MANIPULATION_INCOMPLETE",
            "미완결 기록은 재생하지 않습니다",
        ));
    }
    if &tape.binding != binding {
        return Err(error(
            "E_MANIPULATION_BINDING_MISMATCH",
            "입력 실행 계약이 다릅니다",
        ));
    }
    let mut session = ManipulationSession::new(source, binding.clone())?;
    for frame in &tape.frames {
        session.step_expected(&frame.input, Some(frame))?;
    }
    Ok(session.summary())
}

#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &str = "(매마디)마다 { 사건목록 <- () 입력사상.사건들. 사건목록마다 { { 사건목록.채널 == \"수\" }이면 { 값 <- 사건목록.수값. } 아니면 { 위치 <- 사건목록.좌표값. }. }. }.";
    fn range() -> ValueDomain {
        ValueDomain {
            unit: "1".into(),
            min_raw: "-42949672960".into(),
            max_raw: "42949672960".into(),
            step_raw: "1".into(),
        }
    }
    fn binding(source: &str) -> ManipulationBinding {
        let world = ddonirang_core::platform::DetNuri::new();
        ManipulationBinding {
            source_profile: PROFILE.into(),
            input_schema: "test.manipulation@2".into(),
            channel_registry: "test.channels@2".into(),
            source_revision: format!("sha256:{}", hex::encode(Sha256::digest(source.as_bytes()))),
            initial_world: crate::runtime_surface::current_state_hash(world.world()),
            initial_rng: 7,
            recording_policy: "explicit-values@2".into(),
            integrity_policy: "sealed-provisional@2".into(),
            madi_hz: 60,
            dt_raw: 4294967296,
            channels: BTreeMap::from([
                (
                    "수".into(),
                    ChannelContract::Number {
                        target: "parameter.e".into(),
                        domain: range(),
                    },
                ),
                (
                    "좌표".into(),
                    ChannelContract::Coordinate {
                        target: "object.point".into(),
                        axes: [range(), range()],
                    },
                ),
            ]),
            initial_held: BTreeMap::new(),
        }
    }
    fn event(channel: &str, event: EventKind, ordinal: u64) -> Event {
        Event {
            channel: channel.into(),
            event,
            ordinal,
        }
    }
    fn snapshot(madi: u64, events: Vec<Event>) -> ManipulationSnapshot {
        ManipulationSnapshot {
            madi,
            events,
            held: BTreeMap::new(),
        }
    }
    fn requests() -> RecordRequest {
        RecordRequest {
            binding: binding(SOURCE),
            snapshots: vec![
                snapshot(
                    0,
                    vec![
                        event(
                            "수",
                            EventKind::Number {
                                raw: "2147483648".into(),
                                phase: Phase::Set,
                            },
                            0,
                        ),
                        event(
                            "좌표",
                            EventKind::Coordinate {
                                raw: ["4294967296".into(), "-4294967296".into()],
                                phase: Phase::Begin,
                            },
                            1,
                        ),
                    ],
                ),
                snapshot(
                    1,
                    vec![event(
                        "좌표",
                        EventKind::Coordinate {
                            raw: ["8589934592".into(), "0".into()],
                            phase: Phase::Move,
                        },
                        0,
                    )],
                ),
                snapshot(
                    2,
                    vec![event(
                        "좌표",
                        EventKind::Coordinate {
                            raw: ["12884901888".into(), "4294967296".into()],
                            phase: Phase::End,
                        },
                        0,
                    )],
                ),
            ],
        }
    }
    #[test]
    fn beta1_phase_query_uses_admitted_original_order_without_consuming(){
        let source="(매마디)마다 { (순서=0) 조작단계 보여주기. (순서=0) 조작단계 보여주기. }.";
        let mut session=ManipulationSession::new(source,binding(source)).unwrap();
        for (madi,phase) in [Phase::Begin,Phase::Move,Phase::End].into_iter().enumerate(){
            session.step(&snapshot(madi as u64,vec![event("좌표",EventKind::Coordinate{raw:["0".into(),"0".into()],phase},0)])).unwrap();
        }
        session.step(&snapshot(3,vec![event("수",EventKind::Number{raw:"0".into(),phase:Phase::Set},0)])).unwrap();
        assert_eq!(session.summary()["output_display"]["texts"],serde_json::json!(["시작","시작","이동","이동","끝","끝","설정","설정"]));
        assert_eq!(session.summary()["frames"].as_array().unwrap().len(),4);
        assert!(session.seal().is_ok());
    }
    #[test]
    fn beta1_phase_query_uses_event_supplied_integer_ordinal() {
        let source="(매마디)마다 { 사건목록 <- () 입력사상.사건들. 사건목록마다 { (순서=사건목록.순서) 조작단계 보여주기. }. }.";
        let mut session=ManipulationSession::new(source,binding(source)).unwrap();
        session.step(&snapshot(0,vec![
            event("좌표",EventKind::Coordinate{raw:["0".into(),"0".into()],phase:Phase::Begin},0),
            event("수",EventKind::Number{raw:"0".into(),phase:Phase::Set},1),
        ])).unwrap();
        assert_eq!(session.summary()["output_display"]["texts"],serde_json::json!(["시작","설정"]));
    }
    #[test]
    fn beta1_phase_warning_keeps_comparison_output_and_commit() {
        let source="(매마디)마다 { ((순서=0) 조작단계 == \"중간\") 보여주기. }.";
        let mut session=ManipulationSession::new(source,binding(source)).unwrap();
        let baseline_source="(매마디)마다 { 거짓 보여주기. }.";
        let mut baseline=ManipulationSession::new(baseline_source,binding(baseline_source)).unwrap();
        let input=snapshot(0,vec![event("수",EventKind::Number{raw:"0".into(),phase:Phase::Set},0)]);
        session.step(&input).unwrap(); baseline.step(&input).unwrap();
        assert_eq!(session.summary()["output_display"]["texts"],serde_json::json!(["거짓"]));
        // Compare the same accepted madi, not an initial-world hash against
        // a world whose tick has already advanced.
        assert_eq!(crate::runtime_surface::current_state_hash(session.committed_world()),
            crate::runtime_surface::current_state_hash(baseline.committed_world()));
        assert_eq!(session.summary()["frames"].as_array().unwrap().len(),1);
    }
    #[test]
    fn beta1_phase_query_invalid_selection_keeps_whole_frame_unpublished(){
        for index in ["-1","4096","1"] {
            let source=format!("(매마디)마다 {{ 9 보여주기. (순서={index}) 조작단계 보여주기. }}.");
            let mut session=ManipulationSession::new(&source,binding(&source)).unwrap();
            let before=crate::runtime_surface::current_state_hash(session.committed_world());
            let error=session.step(&snapshot(0,vec![event("수",EventKind::Number{raw:"0".into(),phase:Phase::Set},0)])).unwrap_err();
            assert!(error.contains("E_LOGICAL_INPUT_ORDER"),"{error}");
            assert_eq!(crate::runtime_surface::current_state_hash(session.committed_world()),before);
            assert_eq!(session.summary()["frames"],serde_json::json!([]));
            assert!(session.summary().get("output_display").is_none(),"no successful output display may be published");
        }
        for source in ["(순서=0.5) 조작단계 보여주기.","() 조작단계 보여주기.","(순번=0) 조작단계 보여주기."]{
            assert!(ManipulationSession::new(source,binding(source)).is_err(),"{source}");
        }
    }
    #[test]
    fn alpha4_query_preserves_exact_numbers_coordinates_phase_and_order() {
        let request = requests();
        let projection = query(&request.binding, &request.snapshots[0]).unwrap();
        assert_eq!(
            projection.events()[0].kind,
            LogicalInputEventKind::Number(2147483648, ValueEventPhase::Set)
        );
        assert_eq!(
            projection.events()[1].kind,
            LogicalInputEventKind::Coordinate([4294967296, -4294967296], ValueEventPhase::Begin)
        );
        assert!(projection.is_held(&"수".to_string()).is_err());
        let mut reversed = request.snapshots[0].clone();
        reversed.events.swap(0, 1);
        assert!(query(&request.binding, &reversed).is_err());
    }
    #[test]
    fn alpha4_invalid_raw_range_step_unit_type_and_target_are_rejected() {
        let request = requests();
        for raw in [
            "-0",
            "+1",
            "1.0",
            "01",
            "42949672961",
            "9223372036854775808",
        ] {
            let snapshot = snapshot(
                0,
                vec![event(
                    "수",
                    EventKind::Number {
                        raw: raw.into(),
                        phase: Phase::Set,
                    },
                    0,
                )],
            );
            assert!(
                query(&request.binding, &snapshot).is_err(),
                "accepted {raw}"
            );
        }
        let mut binding = request.binding.clone();
        if let ChannelContract::Number { domain, .. } = binding.channels.get_mut("수").unwrap() {
            domain.step_raw = "4294967296".into();
        }
        assert!(query(&binding, &request.snapshots[0])
            .unwrap_err()
            .starts_with("E_MANIPULATION_RANGE"));
        if let ChannelContract::Number { domain, target } = binding.channels.get_mut("수").unwrap()
        {
            domain.unit = "unknown".into();
            target.clear();
        }
        assert!(query(&binding, &request.snapshots[0]).is_err());
        let wrong = snapshot(
            0,
            vec![event(
                "수",
                EventKind::Coordinate {
                    raw: ["0".into(), "0".into()],
                    phase: Phase::Set,
                },
                0,
            )],
        );
        assert!(query(&request.binding, &wrong)
            .unwrap_err()
            .starts_with("E_MANIPULATION_TYPE"));
    }
    #[test]
    fn alpha4_actual_source_reads_values_and_roundtrips_with_full_input_receipts() {
        let request = requests();
        let (bytes, summary) = record_source(SOURCE, &request).unwrap();
        assert_eq!(summary["resources"]["fixed64"]["값"], "0.5");
        assert_eq!(
            summary["resources"]["value_json"]["위치"],
            serde_json::json!([3.0, 1.0])
        );
        assert_eq!(
            summary["frames"][0]["input_digest"],
            snapshot_digest(&request.snapshots[0]).unwrap()
        );
        assert_eq!(
            summary["input_receipts"][0]["input_snapshot_sha256"],
            snapshot_digest(&request.snapshots[0]).unwrap()
        );
        assert_eq!(
            summary["input_receipts"][0]["input_snapshot"],
            serde_json::to_value(&request.snapshots[0]).unwrap()
        );
        assert_eq!(
            replay_source(SOURCE, &bytes, &request.binding).unwrap(),
            summary
        );
        assert_eq!(encode(&decode(&bytes).unwrap()).unwrap(), bytes);
        assert!(crate::logical_input_tape::decode(&bytes)
            .unwrap_err()
            .starts_with("E_LOGICAL_INPUT_VERSION"));
    }
    #[test]
    fn alpha4_missing_start_end_and_failed_madi_never_mint_success_seal() {
        let request = requests();
        let mut session = ManipulationSession::new(SOURCE, request.binding.clone()).unwrap();
        let before = session.summary();
        assert!(session.step(&request.snapshots[1]).is_err());
        assert_eq!(session.summary()["state_hash"], before["state_hash"]);
        assert_eq!(session.summary()["madi_count"], 0);
        assert!(session.seal().is_err());
        let mut session = ManipulationSession::new(SOURCE, request.binding.clone()).unwrap();
        session.step(&request.snapshots[0]).unwrap();
        assert!(session
            .seal()
            .unwrap_err()
            .starts_with("E_MANIPULATION_SEAL"));
        let before = session.summary();
        let invalid = snapshot(
            1,
            vec![event(
                "수",
                EventKind::Number {
                    raw: "999999999999".into(),
                    phase: Phase::Set,
                },
                0,
            )],
        );
        assert!(session.step(&invalid).is_err());
        assert_eq!(session.summary()["state_hash"], before["state_hash"]);
        assert_eq!(session.summary()["frames"], before["frames"]);
        assert_eq!(session.summary()["resources"], before["resources"]);
        assert!(session.seal().is_err());
    }
    #[test]
    fn alpha4_record_version_integrity_and_full_contract_mismatch_are_explicit() {
        let request = requests();
        let (bytes, _) = record_source(SOURCE, &request).unwrap();
        let mut bad = bytes.clone();
        let index = bad.len() - 1;
        bad[index] ^= 1;
        assert!(decode(&bad).unwrap_err().starts_with("E_MANIPULATION_SEAL"));
        assert!(decode(b"DDN_LOGICAL_INPUT_TAPE_V1\n")
            .unwrap_err()
            .starts_with("E_MANIPULATION_VERSION"));
        let mut changed = request.binding.clone();
        changed.initial_rng += 1;
        assert!(replay_source(SOURCE, &bytes, &changed)
            .unwrap_err()
            .starts_with("E_MANIPULATION_BINDING_MISMATCH"));
        assert!(
            replay_source(&(SOURCE.to_string() + "\n"), &bytes, &request.binding)
                .unwrap_err()
                .starts_with("E_MANIPULATION_BINDING_MISMATCH")
        );
        let mut tape = decode(&bytes).unwrap();
        tape.frames[1].accepted_world_digest = "wrong".into();
        let bytes = encode(&tape).unwrap();
        assert!(replay_source(SOURCE, &bytes, &request.binding)
            .unwrap_err()
            .starts_with("E_MANIPULATION_WORLD_MISMATCH"));
    }
    #[test]
    fn alpha4_same_channel_order_changes_world_and_overflow_is_whole_madi_failure() {
        let binding = binding(SOURCE);
        let inputs = vec![
            event(
                "수",
                EventKind::Number {
                    raw: "4294967296".into(),
                    phase: Phase::Set,
                },
                0,
            ),
            event(
                "수",
                EventKind::Number {
                    raw: "8589934592".into(),
                    phase: Phase::Set,
                },
                1,
            ),
        ];
        let mut reversed = inputs.clone();
        reversed.reverse();
        for (index, event) in reversed.iter_mut().enumerate() {
            event.ordinal = index as u64;
        }
        let (_, left) = record_source(
            SOURCE,
            &RecordRequest {
                binding: binding.clone(),
                snapshots: vec![snapshot(0, inputs)],
            },
        )
        .unwrap();
        let (_, right) = record_source(
            SOURCE,
            &RecordRequest {
                binding: binding.clone(),
                snapshots: vec![snapshot(0, reversed)],
            },
        )
        .unwrap();
        assert_ne!(left["state_hash"], right["state_hash"]);
        assert_ne!(
            left["frames"][0]["input_digest"],
            right["frames"][0]["input_digest"]
        );
        let excess = (0..=MAX_EVENTS_PER_MADI)
            .map(|ordinal| {
                event(
                    "수",
                    EventKind::Number {
                        raw: "0".into(),
                        phase: Phase::Set,
                    },
                    ordinal as u64,
                )
            })
            .collect();
        let mut session = ManipulationSession::new(SOURCE, binding).unwrap();
        let before = session.summary();
        assert!(session
            .step(&snapshot(0, excess))
            .unwrap_err()
            .starts_with("E_MANIPULATION_OVERFLOW"));
        assert_eq!(session.summary()["state_hash"], before["state_hash"]);
        assert_eq!(session.summary()["madi_count"], 0);
        assert!(session.seal().is_err());
    }
}
