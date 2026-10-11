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

fn validate(tape: &LogicalTape) -> Result<(), String> {
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
    binding: LogicalInputBinding,
    runner: crate::ddn_runtime::DdnRunner,
    world: ddonirang_core::platform::DetNuri,
    settled: BTreeMap<String, bool>,
    frames: Vec<LogicalFrame>,
    receipts: Vec<serde_json::Value>,
    failure: Option<String>,
    last_bogae_projection: Option<serde_json::Value>,
    last_observation_table: Option<serde_json::Value>,
}

impl LogicalInputSession {
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
            settled: binding.initial_held.clone(), binding,
            runner: crate::ddn_runtime::DdnRunner::new(program, "매마디").with_bogae_activation(),
            world, frames: Vec::new(), receipts: Vec::new(), failure: None,
            last_bogae_projection: None, last_observation_table: None,
        })
    }

    pub fn step(&mut self, input: &LogicalSnapshot) -> Result<serde_json::Value, String> {
        self.step_expected(input, None)
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
        let (output, receipt) =
            self.runner.run_update_logical(self.world.world(), &self.binding, input, &std::collections::HashMap::new())?;
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
        let trace = crate::deterministic_debug::build_public_observation_trace(&serde_json::json!({
            "run_id":run_id,"source_identity":self.binding.source_revision,
            "revision_identity":self.binding.source_revision,"run_status":"completed",
            "frames":[{"committed_madi":input.madi,"state_hash":frame.accepted_world_digest,
                "input_snapshot":input,"public_observations":observations}]
        }))?;
        let table = crate::deterministic_debug::build_public_observation_table_projection(&trace["trace"], &run_id)?;
        let mut result = serde_json::json!({
            "frame": frame, "input_receipt": receipt,
            "resources": crate::runtime_surface::serialize_world_resources(candidate.world()),
            "reaction_support": false,
            "state_hash":frame.accepted_world_digest,
            "public_observation_table_projection":table,
        });
        crate::bogae_projection::attach(&mut result, output.bogae_projection.as_ref());
        self.last_bogae_projection = output.bogae_projection;
        self.last_observation_table = Some(table);
        self.world = candidate;
        self.settled = settled;
        self.frames.push(frame);
        self.receipts.push(receipt);
        Ok(result)
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
        "reaction_support": false,
        });
        crate::bogae_projection::attach(&mut summary, self.last_bogae_projection.as_ref());
        if let Some(table) = &self.last_observation_table { summary["public_observation_table_projection"] = table.clone(); }
        if let Some(failure) = &self.failure {
            summary["outcome"] = serde_json::json!("ABORTED");
            summary["failure"] = serde_json::json!(failure);
        }
        summary
    }

    #[cfg(feature = "wasm")]
    pub(crate) fn committed_world(&self) -> &ddonirang_core::platform::NuriWorld {
        use ddonirang_core::Nuri;
        self.world.world()
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
    use super::*;

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
        let source = "(매마디)마다 { 값 <- 6. 사건들 <- () 입력사상.사건들. 사건들마다 { { 사건들.채널 == \"곱함\" }이면 { 값 <- 1 / 0. }. }. }. 짜임 := (관계들=차림[(보기=(() 표준보기.결과표)) 표준보개.으뜸]) 표준보개.짜임. (값:값) 보임.";
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
