//! Approved transport candidate. The binding retains its admitted execution
//! profile; this checksum never issues input or replay authority.
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::{logical_input_tape as logical, manipulation_input as manipulation};
mod recording;
pub(crate) use recording::finished_output_value;
mod pending;
#[cfg(windows)]
mod native_finish;
#[cfg(windows)]
pub(crate) use native_finish::PreparedNativeLogicalFinishV1;

const MAGIC: &[u8] = b"DDN_INPUT_TAPE_V3\n";
const DOMAIN: &[u8] = b"ddn.input_tape.v3\0";
const SNAPSHOT_DOMAIN: &[u8] = b"ddn.input_snapshot.v3\0";
const MAX_BYTES: usize = 67_108_864;
const MAX_EVENTS: usize = 4096;
fn error(text: &str) -> String { format!("E_INPUT_TAPE_V3: {text}") }
fn digest(domain: &[u8], bytes: &[u8]) -> [u8; 32] {
    let mut hasher=Sha256::new(); hasher.update(domain); hasher.update(bytes); hasher.finalize().into()
}
fn canonical<T: Serialize>(value: &T) -> Result<Vec<u8>,String> {
    struct Bounded(Vec<u8>);
    impl std::io::Write for Bounded {
        fn write(&mut self,bytes:&[u8])->std::io::Result<usize> {
            if self.0.len().checked_add(bytes.len()).is_none_or(|size|size>MAX_BYTES-MAGIC.len()-8-32) {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,"입력 payload 한도 초과"));
            }
            self.0.extend_from_slice(bytes);Ok(bytes.len())
        }
        fn flush(&mut self)->std::io::Result<()> {Ok(())}
    }
    let mut output=Bounded(Vec::new());
    serde_json::to_writer(&mut output,value).map_err(|_|error("정본 입력 직렬화 실패·한도 초과"))?;
    Ok(output.0)
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(tag="kind",deny_unknown_fields)]
enum Channel {
    Control, Text,
    Number { target:String, domain:manipulation::ValueDomain },
    Coordinate { target:String, axes:[manipulation::ValueDomain;2] },
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    source_profile:String,input_schema:String,channel_registry:String,source_revision:String,
    initial_world:String,initial_rng:u64,recording_policy:String,integrity_policy:String,
    madi_hz:u32,dt_raw:i64,channels:BTreeMap<String,Channel>,initial_held:BTreeMap<String,bool>,
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(tag="kind",content="payload",deny_unknown_fields)]
enum EventKind {
    Press, Repeat, Release, CommittedText(String), Paste(String),
    Number { raw:String,phase:manipulation::Phase },
    Coordinate { raw:[String;2],phase:manipulation::Phase },
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Event {ordinal:u64,channel:String,event:EventKind}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    madi:u64,
    #[serde(deserialize_with="bounded_events")]
    events:Vec<Event>,held:BTreeMap<String,bool>,
}
fn bounded_events<'de,D:serde::Deserializer<'de>>(deserializer:D)->Result<Vec<Event>,D::Error> {
    struct Events;
    impl<'de> serde::de::Visitor<'de> for Events {
        type Value=Vec<Event>;
        fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result {write!(f,"최대4096 입력 사건")}
        fn visit_seq<A:serde::de::SeqAccess<'de>>(self,mut seq:A)->Result<Self::Value,A::Error> {
            let mut events=Vec::new();
            while events.len()<MAX_EVENTS {
                let Some(event)=seq.next_element()? else {return Ok(events)};
                events.push(event);
            }
            // Do not deserialize or allocate a 4097th event payload.
            if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(serde::de::Error::custom("마디당 사건 한도4096 초과"));
            }
            Ok(events)
        }
    }
    deserializer.deserialize_seq(Events)
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Frame {input:Snapshot,input_digest:String,accepted_world_digest:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Tape {
    binding:Binding,frames:Vec<Frame>,termination:logical::Termination,
    accepted_madi_count:u64,accepted_event_count:u64,failure_reason:Option<String>,
}

/// Expected context comes from the caller's admitted owner, not the tape.
pub enum ExpectedBinding<'a> {Logical(&'a logical::LogicalInputBinding),Manipulation(&'a manipulation::ManipulationBinding)}
/// An explicit replay request, never a completion proof or Tape conversion.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum ReplayMode {CompleteOnly,RecordedPrefix}

fn replay_is_partial(tape:&Tape,mode:ReplayMode)->Result<bool,String>{
    match tape.termination {
        logical::Termination::Complete=>Ok(false),
        logical::Termination::Incomplete if mode==ReplayMode::RecordedPrefix=>Ok(true),
        logical::Termination::Incomplete=>Err(error("미완 기록은 정상 재생하지 않습니다. 중단된 기록 끝까지만 재생하는 옵션을 명시하세요")),
        logical::Termination::Aborted=>Err(error("실패 기록은 정상·부분 재생하지 않습니다. 원래 실패 원인을 확인하세요")),
    }
}
fn replay_summary(mut summary:serde_json::Value,tape:&Tape,partial:bool)->Result<serde_json::Value,String>{
    if partial {
        let object=summary.as_object_mut().ok_or_else(||error("재생 요약 구조가 다릅니다"))?;
        object.insert("replay_status".into(),"incomplete_prefix".into());
        object.insert("source_record_termination".into(),"Incomplete".into());
        object.insert("partial_replay".into(),true.into());
        object.insert("stopped_after_madi_count".into(),tape.accepted_madi_count.into());
        object.insert("last_recorded_madi".into(),tape.frames.last().map(|frame|serde_json::Value::from(frame.input.madi)).unwrap_or(serde_json::Value::Null));
        object.insert("replay_notice".into(),format!("중단된 기록, {}마디까지",tape.accepted_madi_count).into());
    }
    Ok(summary)
}
impl ExpectedBinding<'_> {
    fn binding(&self)->Binding {
        macro_rules! binding {
            ($b:expr,$channels:expr)=>{{let b=$b; Binding {
                source_profile:b.source_profile.clone(),input_schema:b.input_schema.clone(),
                channel_registry:b.channel_registry.clone(),source_revision:b.source_revision.clone(),
                initial_world:b.initial_world.clone(),initial_rng:b.initial_rng,
                recording_policy:b.recording_policy.clone(),integrity_policy:b.integrity_policy.clone(),
                madi_hz:b.madi_hz,dt_raw:b.dt_raw,channels:$channels,initial_held:b.initial_held.clone(),
            }}};
        }
        match self {
            Self::Logical(b)=>binding!(b,b.channels.iter().map(|(name,kind)|(name.clone(),match kind {
                logical::ChannelKind::Control=>Channel::Control,logical::ChannelKind::Text=>Channel::Text,
            })).collect()),
            Self::Manipulation(b)=>binding!(b,b.channels.iter().map(|(name,kind)|(name.clone(),match kind {
                manipulation::ChannelContract::Control=>Channel::Control,
                manipulation::ChannelContract::Number{target,domain}=>Channel::Number{target:target.clone(),domain:domain.clone()},
                manipulation::ChannelContract::Coordinate{target,axes}=>Channel::Coordinate{target:target.clone(),axes:axes.clone()},
            })).collect()),
        }
    }
}
impl From<&logical::LogicalSnapshot> for Snapshot {
    fn from(input:&logical::LogicalSnapshot)->Self {Self {madi:input.madi,held:input.held.clone(),events:input.events.iter().map(|event|Event {
        ordinal:event.ordinal,channel:event.channel.clone(),event:match &event.event {
            logical::EventKind::Press=>EventKind::Press,logical::EventKind::Repeat=>EventKind::Repeat,
            logical::EventKind::Release=>EventKind::Release,logical::EventKind::CommittedText(s)=>EventKind::CommittedText(s.clone()),
            logical::EventKind::Paste(s)=>EventKind::Paste(s.clone()),
        },
    }).collect()}}
}
impl From<&manipulation::ManipulationSnapshot> for Snapshot {
    fn from(input:&manipulation::ManipulationSnapshot)->Self {Self {madi:input.madi,held:input.held.clone(),events:input.events.iter().map(|event|Event {
        ordinal:event.ordinal,channel:event.channel.clone(),event:match &event.event {
            manipulation::EventKind::Press=>EventKind::Press,manipulation::EventKind::Repeat=>EventKind::Repeat,
            manipulation::EventKind::Release=>EventKind::Release,
            manipulation::EventKind::Number{raw,phase}=>EventKind::Number{raw:raw.clone(),phase:*phase},
            manipulation::EventKind::Coordinate{raw,phase}=>EventKind::Coordinate{raw:raw.clone(),phase:*phase},
        },
    }).collect()}}
}
impl Snapshot {
    fn logical(&self)->Result<logical::LogicalSnapshot,String> {Ok(logical::LogicalSnapshot {
        madi:self.madi,held:self.held.clone(),events:self.events.iter().map(|event|Ok(logical::LogicalEvent {
            ordinal:event.ordinal,channel:event.channel.clone(),event:match &event.event {
                EventKind::Press=>logical::EventKind::Press,EventKind::Repeat=>logical::EventKind::Repeat,
                EventKind::Release=>logical::EventKind::Release,EventKind::CommittedText(s)=>logical::EventKind::CommittedText(s.clone()),
                EventKind::Paste(s)=>logical::EventKind::Paste(s.clone()),
                _=>return Err(error("논리 입력 owner는 수·좌표를 받지 않습니다")),
            },
        })).collect::<Result<_,String>>()?,
    })}
    fn manipulation(&self)->Result<manipulation::ManipulationSnapshot,String> {Ok(manipulation::ManipulationSnapshot {
        madi:self.madi,held:self.held.clone(),events:self.events.iter().map(|event|Ok(manipulation::Event {
            ordinal:event.ordinal,channel:event.channel.clone(),event:match &event.event {
                EventKind::Press=>manipulation::EventKind::Press,EventKind::Repeat=>manipulation::EventKind::Repeat,
                EventKind::Release=>manipulation::EventKind::Release,
                EventKind::Number{raw,phase}=>manipulation::EventKind::Number{raw:raw.clone(),phase:*phase},
                EventKind::Coordinate{raw,phase}=>manipulation::EventKind::Coordinate{raw:raw.clone(),phase:*phase},
                _=>return Err(error("이 입력 owner는 글 입력을 지원하지 않습니다")),
            },
        })).collect::<Result<_,String>>()?,
    })}
    fn digest(&self)->Result<String,String> {Ok(hex::encode(digest(SNAPSHOT_DOMAIN,&canonical(self)?)))}
}
impl Frame {
    fn logical(&self)->Result<logical::LogicalFrame,String> {
        let input=self.input.logical()?;
        Ok(logical::LogicalFrame {input_digest:logical::snapshot_digest(&input)?,input,accepted_world_digest:self.accepted_world_digest.clone()})
    }
    fn manipulation(&self)->Result<manipulation::Frame,String> {
        let input=self.input.manipulation()?;
        Ok(manipulation::Frame {input_digest:manipulation::snapshot_digest(&input)?,input,accepted_world_digest:self.accepted_world_digest.clone()})
    }
    fn from_input(input:Snapshot,world:&str)->Result<Self,String> {
        Ok(Self {input_digest:input.digest()?,input,accepted_world_digest:world.into()})
    }
}
impl Tape {
    fn validate(&self,expected:&ExpectedBinding<'_>)->Result<(),String> {
        if self.binding!=expected.binding() {return Err(error("기대 입력 계약과 기록이 다릅니다"));}
        for frame in &self.frames {
            if frame.input.events.len()>MAX_EVENTS || frame.input_digest!=frame.input.digest()? {
                return Err(error("입력 운반 digest 또는 사건 한도가 다릅니다"));
            }
        }
        // Existing typed owners alone validate profiles, held/active transitions,
        // raw ranges and termination. Their semantic digests remain unchanged.
        match expected {
            ExpectedBinding::Logical(binding)=>logical::validate(&logical::LogicalTape {
                binding:(*binding).clone(),frames:self.frames.iter().map(Frame::logical).collect::<Result<_,_>>()?,
                termination:self.termination,accepted_madi_count:self.accepted_madi_count,
                accepted_event_count:self.accepted_event_count,failure_reason:self.failure_reason.clone(),
            }),
            ExpectedBinding::Manipulation(binding)=>manipulation::validate(&manipulation::Tape {
                binding:(*binding).clone(),frames:self.frames.iter().map(Frame::manipulation).collect::<Result<_,_>>()?,
                termination:self.termination,accepted_madi_count:self.accepted_madi_count,
                accepted_event_count:self.accepted_event_count,failure_reason:self.failure_reason.clone(),
            }),
        }
    }
    fn encode(&self,expected:&ExpectedBinding<'_>)->Result<Vec<u8>,String> {
        self.validate(expected)?;
        let payload=canonical(self)?;
        let size=MAGIC.len().checked_add(8).and_then(|v|v.checked_add(payload.len())).and_then(|v|v.checked_add(32))
            .filter(|size|*size<=MAX_BYTES).ok_or_else(||error("입력 파일 한도 초과"))?;
        let mut bytes=Vec::with_capacity(size);bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());bytes.extend_from_slice(&payload);
        let footer=digest(DOMAIN,&bytes);bytes.extend_from_slice(&footer);Ok(bytes)
    }
    fn decode(bytes:&[u8],expected:&ExpectedBinding<'_>)->Result<Self,String> {
        let prefix=MAGIC.len()+8;
        if bytes.len()>MAX_BYTES || bytes.len()<prefix+32 || !bytes.starts_with(MAGIC) {
            return Err(error("V3 머리표·길이가 다릅니다. 옛 기록은 명시 변환이 필요합니다"));
        }
        let count=u64::from_le_bytes(bytes[MAGIC.len()..prefix].try_into().unwrap());
        let count=usize::try_from(count).map_err(|_|error("payload 길이 초과"))?;
        let end=prefix.checked_add(count).filter(|end|end.checked_add(32)==Some(bytes.len())).ok_or_else(||error("payload 길이 불일치"))?;
        if digest(DOMAIN,&bytes[..end]).as_slice()!=&bytes[end..] {return Err(error("입력 봉인 불일치"));}
        let tape:Self=serde_json::from_slice(&bytes[prefix..end]).map_err(|_|error("정본 입력 구조·중복·한도 오류"))?;
        if canonical(&tape)?!=bytes[prefix..end] {return Err(error("정본 UTF-8 JSON bytes가 아닙니다"));}
        tape.validate(expected)?;Ok(tape)
    }
}

fn finite_tape(binding:Binding,frames:Vec<Frame>,planned:usize,failure:Option<&str>)->Result<Tape,String> {
    if failure.is_some() || frames.len()!=planned {return Err(error("실제 finite 계획이 끝나지 않았습니다"));}
    let count=frames.iter().try_fold(0u64,|count,frame|count.checked_add(frame.input.events.len() as u64)).ok_or_else(||error("사건 합계 초과"))?;
    Ok(Tape {binding,accepted_madi_count:frames.len() as u64,accepted_event_count:count,frames,
        termination:logical::Termination::Complete,failure_reason:None})
}
/// Explicit private archive-progress decoder. It does not weaken Complete-only
/// public replay admission, terminate an owner, or create a normal-end proof.
pub(crate) fn checked_running_logical_archive_prefix(contract:&[u8],evidence:&[u8],
    binding:&logical::LogicalInputBinding)->Result<Vec<logical::LogicalFrame>,String>{
    checked_logical_archive_prefix(contract,evidence,binding,logical::Termination::Incomplete)
}
pub(crate) fn checked_completed_logical_archive_prefix(contract:&[u8],evidence:&[u8],
    binding:&logical::LogicalInputBinding)->Result<Vec<logical::LogicalFrame>,String>{
    checked_logical_archive_prefix(contract,evidence,binding,logical::Termination::Complete)
}
mod recovery_response;
pub(crate) use recovery_response::check_original_finish_response;
fn decode_logical_archive_tape(contract:&[u8],evidence:&[u8],binding:&logical::LogicalInputBinding)->Result<Tape,String>{
    let expected=ExpectedBinding::Logical(binding);
    let wanted=logical_contract_bytes(binding)?;
    if contract!=wanted || evidence.first()!=Some(&1){return Err(error("기록 보관 입력 계약이 다릅니다"));}
    Tape::decode(&evidence[1..],&expected)
}
const LIVE_STOP_REASON:&str="사용자가 기록을 명시적으로 중단했습니다";
pub(crate) fn checked_terminal_logical_archive_prefix(contract:&[u8],evidence:&[u8],binding:&logical::LogicalInputBinding,
    state:u8)->Result<Vec<logical::LogicalFrame>,String>{
    let tape=decode_logical_archive_tape(contract,evidence,binding)?;
    let valid=match state{
        1=>tape.termination==logical::Termination::Complete&&tape.failure_reason.is_none(),
        2=>tape.termination==logical::Termination::Incomplete&&tape.failure_reason.as_deref()==Some(LIVE_STOP_REASON),
        3=>tape.termination==logical::Termination::Aborted&&tape.failure_reason.as_ref().is_some_and(|v|!v.is_empty()),
        _=>false,
    };
    if !valid{return Err(error("실제 종료 기록과 원 입력의 종료 종류가 다릅니다"));}
    tape.frames.iter().map(Frame::logical).collect()
}
fn checked_logical_archive_prefix(contract:&[u8],evidence:&[u8],binding:&logical::LogicalInputBinding,
    required:logical::Termination)->Result<Vec<logical::LogicalFrame>,String>{
    let tape=decode_logical_archive_tape(contract,evidence,binding)?;
    let correct_reason=match required {
        logical::Termination::Incomplete=>tape.failure_reason.as_deref()==Some("진행 중인 기록의 저장 경계"),
        logical::Termination::Complete=>tape.failure_reason.is_none(),_=>false,
    };
    if tape.termination!=required || !correct_reason {
        return Err(error("요구한 기록 종료 상태와 원 저장 경계가 다릅니다"));
    }
    tape.frames.iter().map(Frame::logical).collect()
}
fn aborted_tape(binding:Binding,frames:Vec<Frame>,failure:Option<&str>)->Result<Tape,String> {
    let reason=failure.filter(|reason|!reason.is_empty()).ok_or_else(||error("실제 실패가 없는 실행을 실패 기록으로 만들 수 없습니다"))?;
    let count=frames.iter().try_fold(0u64,|count,frame|count.checked_add(frame.input.events.len() as u64)).ok_or_else(||error("사건 합계 초과"))?;
    Ok(Tape {binding,accepted_madi_count:frames.len() as u64,accepted_event_count:count,frames,
        termination:logical::Termination::Aborted,failure_reason:Some(reason.into())})
}
/// One recording controller belongs to one VM. A live user stop is not a
/// finite-plan completion proof, and cannot be supplied as a host Boolean.
#[derive(Default)]
pub(crate) struct LiveRecording { stopped:bool,completion:Option<recording::RecordingControllerV1> }
pub(crate) struct RunningLogicalRecordingBorrowV1<'a> {
    _owner:&'a LiveRecording,session:&'a logical::LogicalInputSession,
}
pub(crate) fn logical_contract_bytes(binding:&logical::LogicalInputBinding)->Result<Vec<u8>,String>{
    let expected=ExpectedBinding::Logical(binding);
    let mut bytes=vec![1];bytes.extend(canonical(&expected.binding())?);Ok(bytes)
}
impl<'a> RunningLogicalRecordingBorrowV1<'a> {
    pub(crate) fn transport_parts(&self)->Result<(Vec<u8>,Vec<u8>,u64,&'a [serde_json::Value]),String> {
        let (binding,accepted,_)=self.session.accepted_prefix_v3();
        let expected=ExpectedBinding::Logical(binding);
        let frames=accepted.iter().map(|f|Frame::from_input(Snapshot::from(&f.input),&f.accepted_world_digest)).collect::<Result<Vec<_>,_>>()?;
        let count=frames.iter().try_fold(0u64,|n,f|n.checked_add(f.input.events.len() as u64)).ok_or_else(||error("사건 합계 초과"))?;
        let tape=Tape{binding:expected.binding(),accepted_madi_count:frames.len() as u64,accepted_event_count:count,frames,
            termination:logical::Termination::Incomplete,failure_reason:Some("진행 중인 기록의 저장 경계".into())};
        let contract=logical_contract_bytes(binding)?;
        let mut evidence=vec![1];evidence.extend(tape.encode(&expected)?);
        let receipts=self.session.committed_input_receipts();
        if receipts.len()!=accepted.len(){return Err(error("실제 입력 영수증 수가 확정 마디와 다릅니다"));}
        Ok((contract,evidence,accepted.len() as u64,receipts))
    }
    pub(crate) fn result_boundary(&self)->Result<crate::logical_input_tape::CommittedLogicalResultBoundary<'_>,String> {
        self.session.borrow_result_boundary()
    }
}
impl LiveRecording {
    #[cfg(windows)]
    pub(crate) fn from_native_running_progress(session:&mut logical::LogicalInputSession,
        proof:&crate::unit_wire::resumable_complete_state::CurrentRecordingProgressProofV1)->Result<Self,String>{
        Ok(Self{stopped:false,completion:Some(recording::RecordingControllerV1::for_native_running_progress(session,proof)?)})
    }
    pub(crate) fn for_logical(session:&mut logical::LogicalInputSession)->Result<Self,String> {
        Ok(Self{stopped:false,completion:Some(recording::RecordingControllerV1::for_fresh_logical(session)?)})
    }
    pub(crate) fn for_manipulation(session:&mut manipulation::ManipulationSession)->Result<Self,String> {
        Ok(Self{stopped:false,completion:Some(recording::RecordingControllerV1::for_fresh_manipulation(session)?)})
    }
    pub(crate) fn ensure_running(&self)->Result<(),String> {
        if self.stopped {return Err(error("기록을 중단했습니다. 다시 시작하거나 저장된 작품을 다시 여세요"));}
        match &self.completion {Some(controller)=>controller.ensure_running(),None=>Ok(())}
    }
    pub(crate) fn borrow_running_logical<'a>(&'a self,session:&'a logical::LogicalInputSession)
        ->Result<RunningLogicalRecordingBorrowV1<'a>,String> {
        self.ensure_running()?;
        self.completion.as_ref().ok_or_else(||error("MissingReplayProgress: 원 기록 생산자가 없습니다"))?.check_owner(session)?;
        if session.accepted_prefix_v3().2.is_some(){return Err(error("실패를 진행 중 기록으로 저장할 수 없습니다"));}
        Ok(RunningLogicalRecordingBorrowV1{_owner:self,session})
    }
    pub(crate) fn register_frontend_logical(&mut self,session:&mut logical::LogicalInputSession)
        ->Result<serde_json::Value,String> {
        self.ensure_running()?;
        self.completion.as_mut().ok_or_else(||error("MissingReplayProgress: 원 기록의 진행 증거가 없습니다"))?
            .register_frontend(session)
    }
    pub(crate) fn check_live_logical_owner(&self,session:&logical::LogicalInputSession)->Result<(),String>{
        self.ensure_running()?;
        let controller=self.completion.as_ref().ok_or_else(||error("MissingReplayProgress: 원 기록 생산자가 없습니다"))?;
        controller.ensure_running()?;controller.check_owner(session)
    }
    pub(crate) fn preview_logical_finish(&self,session:&mut logical::LogicalInputSession,packet:&str)
        ->Result<(Vec<u8>,serde_json::Value),String>{
        self.ensure_running()?;
        let controller=self.completion.as_ref().ok_or_else(||error("MissingReplayProgress: 원 기록의 진행 증거가 없습니다"))?;
        let candidate=controller.prepare_frontend_finish(session,packet)?;
        let (bytes,summary)=candidate.planned_artifact_parts();
        Ok((bytes.to_vec(),summary.clone()))
    }
    pub(crate) fn finish_logical(&mut self,session:&mut logical::LogicalInputSession,pending:Option<&str>)
        ->Result<(Vec<u8>,serde_json::Value),String> {
        if self.stopped{return Err(error("중단 기록을 정상 마치기로 바꿀 수 없습니다"));}
        let controller=self.completion.as_mut().ok_or_else(||error("MissingReplayProgress: 원 기록의 진행 증거가 없습니다"))?;
        controller.check_owner(session)?;
        if let Ok(original)=controller.finished(){return Ok(original);}
        let candidate=match pending {
            Some(packet)=>controller.prepare_frontend_finish(session,packet)?,
            None=>controller.prepare_direct_finish(session)?,
        };
        controller.commit_direct_finish(session,candidate)?;
        controller.finished()
    }
    pub(crate) fn finish_notice(&self)->Result<serde_json::Value,String> {
        let count=self.completion.as_ref().ok_or_else(||error("정상 종료 증거가 없습니다"))?.finish_excluded_count()?;
        Ok(serde_json::json!({"excluded_input_count":count.to_string()}))
    }
    pub(crate) fn register_frontend_manipulation(&mut self,session:&mut manipulation::ManipulationSession)
        ->Result<serde_json::Value,String> {
        self.ensure_running()?;
        self.completion.as_mut().ok_or_else(||error("MissingReplayProgress: 원 조작 기록의 증거가 없습니다"))?
            .register_manipulation_frontend(session)
    }
    pub(crate) fn finish_manipulation(&mut self,session:&mut manipulation::ManipulationSession,pending:Option<&str>)
        ->Result<(Vec<u8>,serde_json::Value),String> {
        if self.stopped{return Err(error("중단 기록을 정상 마치기로 바꿀 수 없습니다"));}
        let controller=self.completion.as_mut().ok_or_else(||error("MissingReplayProgress: 원 조작 기록의 증거가 없습니다"))?;
        controller.check_manipulation_owner(session)?;
        if let Ok(original)=controller.finished(){return Ok(original);}
        let candidate=controller.prepare_manipulation_finish(session,pending)?;
        controller.commit_manipulation_finish(session,candidate)?;
        controller.finished()
    }
    fn stop(&mut self,expected:ExpectedBinding<'_>,frames:Vec<Frame>,failure:Option<&str>)->Result<Vec<u8>,String> {
        self.ensure_running()?;
        let tape=if failure.is_some() {aborted_tape(expected.binding(),frames,failure)?}else{
            let count=frames.iter().try_fold(0u64,|count,frame|count.checked_add(frame.input.events.len() as u64)).ok_or_else(||error("사건 합계 초과"))?;
            Tape {binding:expected.binding(),accepted_madi_count:frames.len() as u64,accepted_event_count:count,frames,
                termination:logical::Termination::Incomplete,failure_reason:Some(LIVE_STOP_REASON.into())}
        };
        let bytes=tape.encode(&expected)?;
        // Encoding/admission failure never publishes a stopped success state.
        self.stopped=true;Ok(bytes)
    }
    pub(crate) fn stop_logical(&mut self,session:&logical::LogicalInputSession)->Result<(Vec<u8>,serde_json::Value),String> {
        let (binding,frames,failure)=session.accepted_prefix_v3();
        let frames=frames.iter().map(|frame|Frame::from_input(Snapshot::from(&frame.input),&frame.accepted_world_digest)).collect::<Result<_,_>>()?;
        Ok((self.stop(ExpectedBinding::Logical(binding),frames,failure)?,session.summary()))
    }
    pub(crate) fn stop_manipulation(&mut self,session:&manipulation::ManipulationSession)->Result<(Vec<u8>,serde_json::Value),String> {
        let (binding,frames,failure)=session.accepted_prefix_v3();
        let frames=frames.iter().map(|frame|Frame::from_input(Snapshot::from(&frame.input),&frame.accepted_world_digest)).collect::<Result<_,_>>()?;
        Ok((self.stop(ExpectedBinding::Manipulation(binding),frames,failure)?,session.summary()))
    }
}
/// Failure evidence is borrowed from the actual owner, never reconstructed
/// from presentation JSON. It is explicitly not a successful replay artifact.
pub fn failed_prefix_logical(session:&logical::LogicalInputSession)->Result<(Vec<u8>,serde_json::Value),String> {
    let (binding,frames,failure)=session.accepted_prefix_v3();
    let expected=ExpectedBinding::Logical(binding);
    let tape=aborted_tape(expected.binding(),frames.iter().map(|frame|Frame::from_input(Snapshot::from(&frame.input),&frame.accepted_world_digest)).collect::<Result<_,_>>()?,failure)?;
    Ok((tape.encode(&expected)?,session.summary()))
}
pub fn failed_prefix_manipulation(session:&manipulation::ManipulationSession)->Result<(Vec<u8>,serde_json::Value),String> {
    let (binding,frames,failure)=session.accepted_prefix_v3();
    let expected=ExpectedBinding::Manipulation(binding);
    let tape=aborted_tape(expected.binding(),frames.iter().map(|frame|Frame::from_input(Snapshot::from(&frame.input),&frame.accepted_world_digest)).collect::<Result<_,_>>()?,failure)?;
    Ok((tape.encode(&expected)?,session.summary()))
}
/// Finite controller owns the original request, not a host 'complete' Boolean.
pub fn record_logical(source:&str,request:&logical::LogicalRecordRequest)->Result<(Vec<u8>,serde_json::Value),String> {
    let mut session=logical::LogicalInputSession::new(source,request.binding.clone())?;
    for input in &request.snapshots {session.step(input)?;}
    encode_finite_logical(&session,request.snapshots.len())
}
fn encode_finite_logical(session:&logical::LogicalInputSession,planned:usize)->Result<(Vec<u8>,serde_json::Value),String> {
    let (binding,frames,failure)=session.accepted_prefix_v3();
    let tape=finite_tape(ExpectedBinding::Logical(binding).binding(),frames.iter().map(|frame|Frame::from_input(Snapshot::from(&frame.input),&frame.accepted_world_digest)).collect::<Result<_,_>>()?,planned,failure)?;
    Ok((tape.encode(&ExpectedBinding::Logical(binding))?,session.summary()))
}
/// Reuse the real project source/Start/capture admission exactly once. Never
/// feed linked test text into the single-file constructor as a replacement.
pub fn record_authored_logical(authored:&str,request:&logical::LogicalRecordRequest)->Result<(Vec<u8>,serde_json::Value),String> {
    let mut run=crate::runtime_surface::fresh_authored_logical_run(authored,request.binding.clone())?;
    for input in &request.snapshots {run.step(input)?;}
    encode_finite_logical(run.session(),request.snapshots.len())
}
/// Private CLI finish frontdoor: the command owns the fresh live controller,
/// accepts actual input frames and explicitly closes that same producer. It
/// issues recording content only, never a Native storage/receipt permission.
pub fn finish_live_logical(source:&str,request:&logical::LogicalRecordRequest,authored:bool)
    ->Result<(Vec<u8>,serde_json::Value),String>{
    if authored {
        let mut run=crate::runtime_surface::fresh_authored_logical_run(source,request.binding.clone())?;
        let mut recording=LiveRecording::for_logical(run.session_mut_for_recording())?;
        for input in &request.snapshots {run.step(input)?;}
        recording.finish_logical(run.session_mut_for_recording(),None)
    }else{
        let mut session=logical::LogicalInputSession::new(source,request.binding.clone())?;
        let mut recording=LiveRecording::for_logical(&mut session)?;
        for input in &request.snapshots {session.step(input)?;}
        recording.finish_logical(&mut session,None)
    }
}
pub fn replay_authored_logical(authored:&str,bytes:&[u8],binding:&logical::LogicalInputBinding)->Result<serde_json::Value,String> {
    replay_authored_logical_with_mode(authored,bytes,binding,ReplayMode::CompleteOnly)
}
pub fn replay_authored_logical_with_mode(authored:&str,bytes:&[u8],binding:&logical::LogicalInputBinding,mode:ReplayMode)->Result<serde_json::Value,String> {
    let tape=Tape::decode(bytes,&ExpectedBinding::Logical(binding))?;
    let partial=replay_is_partial(&tape,mode)?;
    let mut run=crate::runtime_surface::fresh_authored_logical_run(authored,binding.clone())?;
    for frame in &tape.frames {run.replay_v3_frame(&frame.logical()?)?;}
    replay_summary(run.session().summary(),&tape,partial)
}
pub fn record_manipulation(source:&str,request:&manipulation::RecordRequest)->Result<(Vec<u8>,serde_json::Value),String> {
    let mut session=manipulation::ManipulationSession::new(source,request.binding.clone())?;
    for input in &request.snapshots {session.step(input)?;}
    let (binding,frames,failure)=session.accepted_prefix_v3();
    let tape=finite_tape(ExpectedBinding::Manipulation(binding).binding(),frames.iter().map(|frame|Frame::from_input(Snapshot::from(&frame.input),&frame.accepted_world_digest)).collect::<Result<_,_>>()?,request.snapshots.len(),failure)?;
    Ok((tape.encode(&ExpectedBinding::Manipulation(&request.binding))?,session.summary()))
}
pub fn replay(source:&str,bytes:&[u8],expected:ExpectedBinding<'_>)->Result<serde_json::Value,String> {
    replay_with_mode(source,bytes,expected,ReplayMode::CompleteOnly)
}
pub fn replay_with_mode(source:&str,bytes:&[u8],expected:ExpectedBinding<'_>,mode:ReplayMode)->Result<serde_json::Value,String> {
    let tape=Tape::decode(bytes,&expected)?;
    let partial=replay_is_partial(&tape,mode)?;
    let summary=match expected {
        ExpectedBinding::Logical(binding)=>{
            let mut session=logical::LogicalInputSession::new(source,binding.clone())?;
            for frame in &tape.frames {session.replay_v3_frame(&frame.logical()?)?;}
            session.summary()
        }
        ExpectedBinding::Manipulation(binding)=>{
            let mut session=manipulation::ManipulationSession::new(source,binding.clone())?;
            for frame in &tape.frames {session.replay_v3_frame(&frame.manipulation()?)?;}
            session.summary()
        }
    };
    replay_summary(summary,&tape,partial)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn b2_logical_prefix_is_explicit_and_does_not_reseal_the_record(){
        let source="(매마디)마다 { 1 보여주기. }.";let request=logical_request(source);
        let mut session=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();
        session.step(&request.snapshots[0]).unwrap();let expected_hash=session.summary()["state_hash"].clone();
        let mut controller=LiveRecording::default();let (bytes,_)=controller.stop_logical(&session).unwrap();let original=bytes.clone();
        assert!(replay(source,&bytes,ExpectedBinding::Logical(&request.binding)).unwrap_err().contains("옵션"));
        let result=replay_with_mode(source,&bytes,ExpectedBinding::Logical(&request.binding),ReplayMode::RecordedPrefix).unwrap();
        assert_eq!(result["state_hash"],expected_hash);assert_eq!(result["partial_replay"],true);
        assert_eq!(result["stopped_after_madi_count"],1);assert_eq!(result["last_recorded_madi"],0);
        assert_eq!(result["replay_notice"],"중단된 기록, 1마디까지");assert_eq!(bytes,original);
        assert_eq!(Tape::decode(&bytes,&ExpectedBinding::Logical(&request.binding)).unwrap().termination,logical::Termination::Incomplete);
    }
    #[test]
    fn b2_manipulation_prefix_preserves_coordinates_and_order(){
        let source="(매마디)마다 { (순서=0) 조작단계 보여주기. }.";let mut request=manipulation_request(source);request.snapshots[0].events.truncate(1);
        let mut session=manipulation::ManipulationSession::new(source,request.binding.clone()).unwrap();session.step(&request.snapshots[0]).unwrap();
        let before=session.summary();let mut controller=LiveRecording::default();let (bytes,_)=controller.stop_manipulation(&session).unwrap();
        let result=replay_with_mode(source,&bytes,ExpectedBinding::Manipulation(&request.binding),ReplayMode::RecordedPrefix).unwrap();
        assert_eq!(result["state_hash"],before["state_hash"]);assert_eq!(result["source_record_termination"],"Incomplete");
        assert!(replay(source,&bytes,ExpectedBinding::Manipulation(&request.binding)).is_err());
    }
    #[test]
    fn b2_prefix_never_admits_aborted_or_tampered_worlds(){
        let source="(매마디)마다 { 1 보여주기. }.";let request=logical_request(source);
        let mut session=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();session.step(&request.snapshots[0]).unwrap();
        let mut invalid=request.snapshots[0].clone();invalid.madi=2;assert!(session.step(&invalid).is_err());
        let (aborted,_)=failed_prefix_logical(&session).unwrap();
        assert!(replay_with_mode(source,&aborted,ExpectedBinding::Logical(&request.binding),ReplayMode::RecordedPrefix).is_err());
        let mut clean=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();clean.step(&request.snapshots[0]).unwrap();
        let mut controller=LiveRecording::default();let (bytes,_)=controller.stop_logical(&clean).unwrap();
        let mut corrupt=bytes.clone();let end=corrupt.len()-1;corrupt[end]^=1;
        assert!(replay_with_mode(source,&corrupt,ExpectedBinding::Logical(&request.binding),ReplayMode::RecordedPrefix).is_err());
        let mut wrong=request.binding.clone();wrong.initial_rng+=1;
        assert!(replay_with_mode(source,&bytes,ExpectedBinding::Logical(&wrong),ReplayMode::RecordedPrefix).is_err());
    }
    #[test]
    fn b2_complete_replay_output_stays_unchanged_with_prefix_option(){
        let source="(매마디)마다 { 1 보여주기. }.";let request=logical_request(source);let (bytes,_)=record_logical(source,&request).unwrap();
        let normal=replay(source,&bytes,ExpectedBinding::Logical(&request.binding)).unwrap();
        let opted=replay_with_mode(source,&bytes,ExpectedBinding::Logical(&request.binding),ReplayMode::RecordedPrefix).unwrap();
        assert_eq!(normal,opted);assert!(normal.get("partial_replay").is_none());
    }
    fn logical_request(source:&str)->logical::LogicalRecordRequest {
        use ddonirang_core::Nuri;
        logical::LogicalRecordRequest {
            binding:logical::LogicalInputBinding {
                source_profile:logical::LOGICAL_INPUT_PROFILE.into(),input_schema:"test.input@1".into(),channel_registry:"test.channels@1".into(),
                source_revision:format!("sha256:{}",hex::encode(Sha256::digest(source.as_bytes()))),
                initial_world:crate::runtime_surface::current_state_hash(ddonirang_core::platform::DetNuri::new().world()),
                initial_rng:7,recording_policy:"test.record@1".into(),integrity_policy:"test.seal@1".into(),madi_hz:60,dt_raw:4294967296,
                channels:BTreeMap::from([("왼쪽".into(),logical::ChannelKind::Control)]),initial_held:BTreeMap::from([("왼쪽".into(),false)]),
            },
            snapshots:vec![logical::LogicalSnapshot {madi:0,events:vec![logical::LogicalEvent {
                ordinal:0,channel:"왼쪽".into(),event:logical::EventKind::Press,
            }],held:BTreeMap::from([("왼쪽".into(),true)])}],
        }
    }
    #[test]
    fn b3_cli_live_finish_uses_the_original_owner_and_preserves_held_input(){
        let source="(매마디)마다 { 1 보여주기. }.";let mut request=logical_request(source);
        let finite=record_logical(source,&request).unwrap();
        let live=finish_live_logical(source,&request,false).unwrap();
        assert_eq!(live,finite,"live finish must preserve the whole artifact, not just the world hash");
        let tape=Tape::decode(&live.0,&ExpectedBinding::Logical(&request.binding)).unwrap();
        assert_eq!(tape.frames[0].input.held.get("왼쪽"),Some(&true));
        request.snapshots.push(request.snapshots[0].clone());
        assert!(finish_live_logical(source,&request,false).is_err(),"failed input cannot publish Complete");
    }
    #[cfg(windows)]
    #[test]
    fn b3_native_finish_prepares_without_finished_publication_and_rejects_an_interleaved_step(){
        let source="(매마디)마다 { 1 보여주기. }.";let request=logical_request(source);
        let mut session=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();
        let mut recording=LiveRecording::for_logical(&mut session).unwrap();
        session.step(&request.snapshots[0]).unwrap();
        let candidate=recording.prepare_native_logical_finish(&mut session,None).unwrap();
        assert_eq!(candidate.closed_count(),1);
        let expected=record_logical(source,&request).unwrap();
        assert_eq!(candidate.tape_bytes(),expected.0);assert_eq!(candidate.summary(),&expected.1);
        assert!(recording.completion.as_ref().unwrap().finished().is_err());
        assert!(recording.finish_notice().is_err());
        let progress=recording.borrow_running_logical(&session).unwrap().transport_parts().unwrap();
        assert!(replay(source,&progress.1[1..],ExpectedBinding::Logical(&request.binding)).is_err());
        session.step(&logical::LogicalSnapshot{madi:1,events:vec![],held:request.snapshots[0].held.clone()}).unwrap();
        assert!(recording.validate_native_logical_finish(&mut session,&candidate).is_err());
        assert!(recording.completion.as_ref().unwrap().finished().is_err());
    }
    #[test]
    fn b3_direct_finish_matches_original_batch_and_caches_exact_two_key_result() {
        use recording::RecordingControllerV1;
        let source="(매마디)마다 { 1 보여주기. }.";
        let request=logical_request(source);
        let mut session=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();
        let mut controller=RecordingControllerV1::for_fresh_logical(&mut session).unwrap();
        session.step(&request.snapshots[0]).unwrap();let before=session.summary();
        let candidate=controller.prepare_direct_finish(&mut session).unwrap();
        controller.commit_direct_finish(&mut session,candidate).unwrap();
        let first=controller.finished().unwrap();let batch=record_logical(source,&request).unwrap();
        assert_eq!(first,batch);assert_eq!(controller.finished().unwrap(),first);
        assert_eq!(session.summary(),before);assert!(controller.ensure_running().is_err());
        let response=serde_json::json!({"tape_bytes":first.0,"summary":first.1});
        assert_eq!(response.as_object().unwrap().len(),2);
        // A held key is valid in the original control codec. Do not invent
        // a release event or impose a new all-held-false completion rule.
        let tape=Tape::decode(&batch.0,&ExpectedBinding::Logical(&request.binding)).unwrap();
        assert_eq!(tape.frames[0].input.held.get("왼쪽"),Some(&true));
    }
    #[test]
    fn b3_direct_finish_rejects_foreign_generation_and_interleaved_step() {
        use recording::RecordingControllerV1;
        let source="(매마디)마다 { 1 보여주기. }.";let request=logical_request(source);
        let mut a=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();
        let mut b=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();
        let mut controller=RecordingControllerV1::for_fresh_logical(&mut a).unwrap();
        assert!(controller.prepare_direct_finish(&mut b).is_err());
        let old=controller.prepare_direct_finish(&mut a).unwrap();
        a.step(&request.snapshots[0]).unwrap();let before=a.summary();
        assert!(controller.commit_direct_finish(&mut a,old).is_err());
        controller.ensure_running().unwrap();assert_eq!(a.summary(),before);
        assert!(controller.finished().is_err());
    }
    #[test]
    fn b3_direct_finish_never_promotes_stop_or_failed_owner() {
        use recording::RecordingControllerV1;
        let source="(매마디)마다 { 1 보여주기. }.";let request=logical_request(source);
        for failed in [false,true] {
            let mut session=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();
            let mut controller=RecordingControllerV1::for_fresh_logical(&mut session).unwrap();
            session.step(&request.snapshots[0]).unwrap();
            if failed {session.reject_input("E_TEST: 실제 owner 입력 실패".into());
                assert!(controller.prepare_direct_finish(&mut session).is_err());}
            let before=session.summary();let (bytes,_)=controller.stop_logical(&mut session).unwrap();
            let tape=Tape::decode(&bytes,&ExpectedBinding::Logical(&request.binding)).unwrap();
            assert_eq!(tape.termination,if failed{logical::Termination::Aborted}else{logical::Termination::Incomplete});
            assert_eq!(session.summary(),before);assert!(controller.prepare_direct_finish(&mut session).is_err());
            assert!(controller.finished().is_err());
        }
    }
    #[test]
    fn b3_frontend_finish_excludes_pending_release_without_tape_or_summary_change() {
        use recording::RecordingControllerV1;
        let source="(매마디)마다 { 1 보여주기. }.";let request=logical_request(source);
        let mut session=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();
        let mut controller=RecordingControllerV1::for_fresh_logical(&mut session).unwrap();
        let registration=controller.register_frontend(&mut session).unwrap();
        session.step(&request.snapshots[0]).unwrap();let before=session.summary();
        assert!(controller.prepare_direct_finish(&mut session).err().unwrap().contains("MissingPendingSupply"));
        controller.ensure_running().unwrap();assert_eq!(session.summary(),before);
        let packet=serde_json::json!({"producer_generation":registration["producer_generation"],
            "first_unprocessed":"1","observed_until":"2",
            "events":[{"ordinal":0,"channel":"왼쪽","event":{"kind":"Release"}}]}).to_string();
        let original=packet.clone();let candidate=controller.prepare_frontend_finish(&mut session,&packet).unwrap();
        assert!(controller.finish_excluded_count().is_err());
        controller.commit_direct_finish(&mut session,candidate).unwrap();
        assert_eq!(controller.finished().unwrap(),record_logical(source,&request).unwrap());
        assert_eq!(controller.finish_excluded_count().unwrap(),1);assert_eq!(packet,original);
        assert_eq!(session.summary(),before);assert_eq!(controller.finish_excluded_count().unwrap(),1);
    }
    #[test]
    fn wo25_finish_export_preserves_actual_response_and_distinguishes_empty_frontend_notice() {
        use recording::RecordingControllerV1;
        let source="(매마디)마다 { 1 보여주기. }.";let request=logical_request(source);
        let mut session=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();
        let mut controller=RecordingControllerV1::for_fresh_logical(&mut session).unwrap();
        session.step(&request.snapshots[0]).unwrap();let before=session.summary();
        let direct=controller.prepare_direct_finish(&mut session).unwrap();
        assert_eq!(direct.recovery_notice_payload(),vec![0]);
        let (tape,summary)=direct.planned_artifact_parts();
        let expected=serde_json::json!({"tape_bytes":tape,"summary":summary}).to_string().into_bytes();
        assert_eq!(direct.recovery_payload_bytes().unwrap(),expected);

        let registration=controller.register_frontend(&mut session).unwrap();
        assert!(controller.prepare_direct_finish(&mut session).is_err(),"missing supply must not become direct-owner zero notice");
        let packet=serde_json::json!({"producer_generation":registration["producer_generation"],
            "first_unprocessed":"1","observed_until":"1","events":[]}).to_string();
        let frontend=controller.prepare_frontend_finish(&mut session,&packet).unwrap();
        let notice=frontend.recovery_notice_payload();
        assert_eq!(notice.len(),49);assert_eq!(notice[0],1);
        assert_eq!(&notice[1..9],&1u64.to_be_bytes());assert_eq!(&notice[9..17],&1u64.to_be_bytes());
        assert_eq!(frontend.recovery_payload_bytes().unwrap(),expected,"notice is never inserted into C01/world/summary");
        controller.ensure_running().unwrap();assert!(controller.finished().is_err());
        assert_eq!(session.summary(),before,"exporting cannot advance or publish");
    }
    #[test]
    fn b3_frontend_finish_rejects_stale_or_overlapping_queue_without_publication() {
        use recording::RecordingControllerV1;
        let source="(매마디)마다 { 1 보여주기. }.";let request=logical_request(source);
        let mut session=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();
        let mut controller=RecordingControllerV1::for_fresh_logical(&mut session).unwrap();
        let registration=controller.register_frontend(&mut session).unwrap();
        session.step(&request.snapshots[0]).unwrap();let before=session.summary();
        let valid=serde_json::json!({"producer_generation":registration["producer_generation"],
            "first_unprocessed":"1","observed_until":"2",
            "events":[{"ordinal":0,"channel":"왼쪽","event":{"kind":"Release"}}]});
        for key in ["producer_generation","first_unprocessed","observed_until"] {
            let mut wrong=valid.clone();wrong[key]="0".into();
            assert!(controller.prepare_frontend_finish(&mut session,&wrong.to_string()).is_err());
            controller.ensure_running().unwrap();assert!(controller.finished().is_err());
            assert_eq!(session.summary(),before);
        }
        let mut wrong=valid.clone();wrong["first_unprocessed"]="01".into();
        assert!(controller.prepare_frontend_finish(&mut session,&wrong.to_string()).is_err());
        assert!(controller.register_frontend(&mut session).is_err());
        let candidate=controller.prepare_frontend_finish(&mut session,&valid.to_string()).unwrap();
        controller.commit_direct_finish(&mut session,candidate).unwrap();
    }
    #[test]
    fn b3_live_frontdoor_binds_actual_prefix_and_caches_only_for_same_owner() {
        let source="(매마디)마다 { 1 보여주기. }.";let request=logical_request(source);
        let mut session=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();
        let mut recording=LiveRecording::for_logical(&mut session).unwrap();
        session.step(&request.snapshots[0]).unwrap();
        // The UI may attach after the source-start step, without issuing a new
        // recording generation or pretending the accepted prefix was empty.
        let registered=recording.register_frontend_logical(&mut session).unwrap();
        assert_eq!(registered["accepted_event_count"],"1");
        let packet=serde_json::json!({"producer_generation":registered["producer_generation"],
            "first_unprocessed":"1","observed_until":"1","events":[]}).to_string();
        let result=recording.finish_logical(&mut session,Some(&packet)).unwrap();
        assert_eq!(result,record_logical(source,&request).unwrap());
        assert_eq!(recording.finish_logical(&mut session,None).unwrap(),result);
        assert_eq!(recording.finish_notice().unwrap()["excluded_input_count"],"0");
        assert!(recording.ensure_running().is_err());
        let mut foreign=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();
        assert!(recording.finish_logical(&mut foreign,None).is_err());
        let mut old=LiveRecording::default();
        assert!(old.finish_logical(&mut session,None).err().unwrap().contains("MissingReplayProgress"));
    }
    #[test]
    fn b3_original_c01_raw_source_has_exact_native_live_batch_results_for_all_rows() {
        let script=include_str!("../../tests/beta1_determinism_native_wasm_runner.mjs");
        assert_eq!(hex::encode(Sha256::digest(script.as_bytes())),
            "17385629d7b4112d859e8babe03cb72831e08db84c3a6d2a7c11202a2e3c0ea9");
        // Extract unchanged DDN bytes, with no test-only source conversion.
        let source=script.split_once("const source=`").unwrap().1.split_once("`;").unwrap().0;
        for reverse in [false,true] {for seed in [7,8] {
            let mut binding=logical_request(source).binding;binding.initial_rng=seed;
            binding.channels=BTreeMap::from([("더함".into(),logical::ChannelKind::Control),("곱함".into(),logical::ChannelKind::Control)]);
            binding.initial_held=BTreeMap::from([("더함".into(),false),("곱함".into(),false)]);
            let names=if reverse{["곱함","더함"]}else{["더함","곱함"]};
            let snapshots=(0..4).map(|madi|logical::LogicalSnapshot {madi,
                held:BTreeMap::from([("더함".into(),madi%2==0),("곱함".into(),madi%2==0)]),
                events:names.iter().enumerate().map(|(ordinal,channel)|logical::LogicalEvent{
                    ordinal:ordinal as u64,channel:(*channel).into(),event:if madi%2==0{logical::EventKind::Press}else{logical::EventKind::Release}}).collect(),
            }).collect::<Vec<_>>();
            let request=logical::LogicalRecordRequest{binding:binding.clone(),snapshots};
            let batch=record_logical(source,&request).unwrap();
            let mut session=logical::LogicalInputSession::new(source,binding).unwrap();
            let mut live=LiveRecording::for_logical(&mut session).unwrap();
            for input in &request.snapshots {live.ensure_running().unwrap();session.step(input).unwrap();}
            let result=live.finish_logical(&mut session,None).unwrap();
            assert_eq!(result,batch);assert_eq!(result.1["resources"]["fixed64"]["계산값"],if reverse{"5"}else{"6"});
        }}
        // This is the Native common-owner counterpart, not a claim that the
        // original CLI3/WASM runner or actual Chrome has been executed.
    }
    fn manipulation_request(source:&str)->manipulation::RecordRequest {
        let old=logical_request(source).binding;
        let domain=manipulation::ValueDomain {unit:"1".into(),min_raw:"-4294967296".into(),max_raw:"4294967296".into(),step_raw:"1".into()};
        manipulation::RecordRequest {binding:manipulation::ManipulationBinding {
            source_profile:manipulation::PROFILE.into(),input_schema:old.input_schema,channel_registry:old.channel_registry,
            source_revision:old.source_revision,initial_world:old.initial_world,initial_rng:old.initial_rng,
            recording_policy:old.recording_policy,integrity_policy:old.integrity_policy,madi_hz:old.madi_hz,dt_raw:old.dt_raw,
            channels:BTreeMap::from([
                ("좌표".into(),manipulation::ChannelContract::Coordinate {target:"test.point".into(),axes:[domain.clone(),domain.clone()]}),
                ("수".into(),manipulation::ChannelContract::Number {target:"test.radius".into(),domain}),
            ]),initial_held:BTreeMap::new(),
        },snapshots:vec![manipulation::ManipulationSnapshot {madi:0,held:BTreeMap::new(),events:[
            manipulation::EventKind::Coordinate{raw:["0".into(),"0".into()],phase:manipulation::Phase::Begin},
            manipulation::EventKind::Coordinate{raw:["1".into(),"0".into()],phase:manipulation::Phase::Move},
            manipulation::EventKind::Coordinate{raw:["2".into(),"0".into()],phase:manipulation::Phase::End},
            manipulation::EventKind::Number{raw:"3".into(),phase:manipulation::Phase::Set},
        ].into_iter().enumerate().map(|(ordinal,event)|manipulation::Event {ordinal:ordinal as u64,channel:if ordinal<3 {"좌표"}else{"수"}.into(),event}).collect()}]}
    }
    #[test]
    fn b3_manipulation_finish_matches_batch_and_keeps_pending_outside_result() {
        let source="(매마디)마다 { 사건목록 <- () 입력사상.사건들. 사건목록마다 { (순서=사건목록.순서) 조작단계 보여주기. }. }.";
        let request=manipulation_request(source);
        let mut session=manipulation::ManipulationSession::new(source,request.binding.clone()).unwrap();
        let mut recording=LiveRecording::for_manipulation(&mut session).unwrap();
        session.step(&request.snapshots[0]).unwrap();let before=session.summary();
        let registered=recording.register_frontend_manipulation(&mut session).unwrap();
        assert_eq!(registered["accepted_event_count"],"4");
        let packet=serde_json::json!({"producer_generation":registered["producer_generation"],
            "first_unprocessed":"4","observed_until":"5",
            "events":[{"ordinal":0,"channel":"수","event":{"kind":"Number","payload":{"raw":"4","phase":"Set"}}}]}).to_string();
        let live=recording.finish_manipulation(&mut session,Some(&packet)).unwrap();
        assert_eq!(live,record_manipulation(source,&request).unwrap());
        assert_eq!(session.summary(),before);assert_eq!(recording.finish_notice().unwrap()["excluded_input_count"],"1");
        assert_eq!(recording.finish_manipulation(&mut session,None).unwrap(),live);
        let mut foreign=manipulation::ManipulationSession::new(source,request.binding.clone()).unwrap();
        assert!(recording.finish_manipulation(&mut foreign,None).is_err());
    }
    #[test]
    fn b3_manipulation_active_end_is_not_synthesized_from_pending_queue() {
        let source="(매마디)마다 { (순서=0) 조작단계 보여주기. }.";
        let mut request=manipulation_request(source);request.snapshots[0].events.truncate(1);
        let mut session=manipulation::ManipulationSession::new(source,request.binding.clone()).unwrap();
        let mut recording=LiveRecording::for_manipulation(&mut session).unwrap();
        let registered=recording.register_frontend_manipulation(&mut session).unwrap();
        session.step(&request.snapshots[0]).unwrap();let before=session.summary();
        let packet=serde_json::json!({"producer_generation":registered["producer_generation"],
            "first_unprocessed":"1","observed_until":"2",
            "events":[{"ordinal":0,"channel":"좌표","event":{"kind":"Coordinate","payload":{"raw":["0","0"],"phase":"End"}}}]}).to_string();
        assert!(recording.finish_manipulation(&mut session,Some(&packet)).err().unwrap().contains("E_INPUT_FINISH_NOT_READY"));
        recording.ensure_running().unwrap();assert_eq!(session.summary(),before);assert!(recording.finish_notice().is_err());
        let end=manipulation::ManipulationSnapshot{madi:1,held:BTreeMap::new(),events:vec![manipulation::Event{
            ordinal:0,channel:"좌표".into(),event:manipulation::EventKind::Coordinate{raw:["0".into(),"0".into()],phase:manipulation::Phase::End}}]};
        session.step(&end).unwrap();request.snapshots.push(end);
        let empty=serde_json::json!({"producer_generation":registered["producer_generation"],
            "first_unprocessed":"2","observed_until":"2","events":[]}).to_string();
        assert_eq!(recording.finish_manipulation(&mut session,Some(&empty)).unwrap(),record_manipulation(source,&request).unwrap());
    }
    #[test]
    fn v3_actual_typed_owners_record_and_replay_without_changing_semantic_receipts() {
        let source="(매마디)마다 { 사건목록 <- () 입력사상.사건들. 사건목록마다 { 사건목록.종류 보여주기. }. }.";
        let request=logical_request(source);
        let (bytes,result)=record_logical(source,&request).unwrap();
        assert!(bytes.starts_with(MAGIC));
        let (_,old)=logical::record_source(source,&request).unwrap();
        assert_eq!(result,old);
        assert_eq!(replay(source,&bytes,ExpectedBinding::Logical(&request.binding)).unwrap(),old);
        let source="(매마디)마다 { 사건목록 <- () 입력사상.사건들. 사건목록마다 { (순서=사건목록.순서) 조작단계 보여주기. }. }.";
        let request=manipulation_request(source);
        let (bytes,result)=record_manipulation(source,&request).unwrap();
        let (_,old)=manipulation::record_source(source,&request).unwrap();
        assert_eq!(result,old);
        assert_eq!(replay(source,&bytes,ExpectedBinding::Manipulation(&request.binding)).unwrap(),old);
        assert_eq!(result["output_display"]["texts"],serde_json::json!(["시작","이동","끝","설정"]));
    }
    #[test]
    fn v3_same_authored_sample_and_original_input_use_real_project_owner() {
        let authored=include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/authored-project.json");
        let mut request:logical::LogicalRecordRequest=serde_json::from_str(include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/initial-input.json")).unwrap();
        let next:logical::LogicalRecordRequest=serde_json::from_str(include_str!("../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/next-input.json")).unwrap();
        assert_eq!(request.binding,next.binding);request.snapshots.extend(next.snapshots);
        let (bytes,result)=record_authored_logical(authored,&request).unwrap();
        assert_eq!(result["state_hash"],"blake3:c7c9359477b58adc903fe72ad9b1c9105b5da8158db169842ef5a36d4468202d");
        assert_eq!(replay_authored_logical(authored,&bytes,&request.binding).unwrap(),result);
    }
    #[test]
    fn v3_rejects_legacy_and_independently_wrong_binding() {
        let source="(매마디)마다 { 1 보여주기. }.";
        let request=logical_request(source);
        let (old,_)=logical::record_source(source,&request).unwrap();
        assert!(replay(source,&old,ExpectedBinding::Logical(&request.binding)).is_err());
        let (bytes,_)=record_logical(source,&request).unwrap();
        let mut wrong=request.binding.clone();wrong.initial_rng+=1;
        assert!(replay(source,&bytes,ExpectedBinding::Logical(&wrong)).is_err());
    }
    #[test]
    fn v3_unchanged_point_sample_has_actual_scalar_coordinate_type_supply() {
        let source=include_str!("../../solutions/seamgrim_ui_mvp/samples/alpha4_manipulation_point.ddn");
        let mut binding:manipulation::ManipulationBinding=serde_json::from_str(include_str!("../../solutions/seamgrim_ui_mvp/samples/alpha4_manipulation_point.binding.json")).unwrap();
        binding.source_revision=format!("sha256:{}",hex::encode(Sha256::digest(source.as_bytes())));
        let request=manipulation::RecordRequest {binding,snapshots:vec![manipulation::ManipulationSnapshot {
            madi:0,held:BTreeMap::new(),events:vec![
                manipulation::Event {ordinal:0,channel:"반지름".into(),event:manipulation::EventKind::Number{raw:"4294967296".into(),phase:manipulation::Phase::Set}},
                manipulation::Event {ordinal:1,channel:"위치".into(),event:manipulation::EventKind::Coordinate{raw:["2147483648".into(),"0".into()],phase:manipulation::Phase::Set}},
            ],
        }]};
        let (bytes,result)=record_manipulation(source,&request).unwrap();
        assert_eq!(replay(source,&bytes,ExpectedBinding::Manipulation(&request.binding)).unwrap(),result);
    }
    #[test]
    fn v3_rejects_resealed_noncanonical_and_tampered_world_before_commit() {
        let source="(매마디)마다 { 1 보여주기. }.";
        let request=logical_request(source);let expected=ExpectedBinding::Logical(&request.binding);
        let (bytes,_)=record_logical(source,&request).unwrap();
        let mut tape=Tape::decode(&bytes,&expected).unwrap();
        tape.frames[0].accepted_world_digest="blake3:wrong".into();
        let changed=tape.encode(&expected).unwrap();
        let mut owner=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();
        let before=owner.summary();
        assert!(owner.replay_v3_frame(&tape.frames[0].logical().unwrap()).is_err());
        assert_eq!(owner.summary()["state_hash"],before["state_hash"]);
        assert_eq!(owner.summary()["frames"],serde_json::json!([]));
        assert!(replay(source,&changed,expected).is_err());
        let expected=ExpectedBinding::Logical(&request.binding);
        let prefix=MAGIC.len()+8;let mut payload=bytes[prefix..bytes.len()-32].to_vec();payload.insert(0,b' ');
        let mut changed=MAGIC.to_vec();changed.extend_from_slice(&(payload.len() as u64).to_le_bytes());changed.extend_from_slice(&payload);
        changed.extend_from_slice(&digest(DOMAIN,&changed));
        assert!(Tape::decode(&changed,&expected).is_err());
    }
    #[test]
    fn v3_rejects_resealed_raw_and_event_overflow_without_publishing() {
        let source="(매마디)마다 { 1 보여주기. }.";
        let request=manipulation_request(source);let expected=ExpectedBinding::Manipulation(&request.binding);
        let (bytes,_)=record_manipulation(source,&request).unwrap();
        let mut tape=Tape::decode(&bytes,&expected).unwrap();
        if let EventKind::Number{raw,..}=&mut tape.frames[0].input.events[3].event {*raw="03".into();}else{panic!()}
        tape.frames[0].input_digest=tape.frames[0].input.digest().unwrap();
        assert!(tape.encode(&expected).is_err(),"rewriting the input checksum cannot admit noncanonical raw");
        let mut tape=Tape::decode(&bytes,&expected).unwrap();
        let first=tape.frames[0].input.events[0].clone();
        tape.frames[0].input.events=vec![first;MAX_EVENTS+1];
        // Deliberately produce a sealed, syntactically valid over-limit wire
        // without using the product encoder, to exercise decoder allocation.
        let payload=canonical(&tape).unwrap();let mut oversized=MAGIC.to_vec();
        oversized.extend_from_slice(&(payload.len() as u64).to_le_bytes());oversized.extend_from_slice(&payload);
        oversized.extend_from_slice(&digest(DOMAIN,&oversized));
        assert!(Tape::decode(&oversized,&expected).is_err());
    }
    #[test]
    fn v3_failed_prefix_comes_from_owner_and_never_admits_as_success() {
        let source="(매마디)마다 { 1 보여주기. }.";
        let request=logical_request(source);
        let mut session=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();
        assert!(failed_prefix_logical(&session).is_err());
        session.step(&request.snapshots[0]).unwrap();
        let before=session.summary();let mut invalid=request.snapshots[0].clone();invalid.madi=2;
        assert!(session.step(&invalid).is_err());
        assert_eq!(session.summary()["state_hash"],before["state_hash"]);
        let (bytes,summary)=failed_prefix_logical(&session).unwrap();
        assert_eq!(summary["outcome"],"ABORTED");
        let expected=ExpectedBinding::Logical(&request.binding);let tape=Tape::decode(&bytes,&expected).unwrap();
        assert_eq!(tape.termination,logical::Termination::Aborted);assert_eq!(tape.frames.len(),1);
        assert_eq!(tape.frames[0].input.madi,0);assert!(tape.failure_reason.is_some());
        assert!(replay(source,&bytes,expected).is_err());assert!(session.seal().is_err());
        let mut controller=LiveRecording::default();let (stopped,_)=controller.stop_logical(&session).unwrap();
        assert_eq!(Tape::decode(&stopped,&ExpectedBinding::Logical(&request.binding)).unwrap().termination,logical::Termination::Aborted);
        assert!(controller.ensure_running().is_err());
        let source="(매마디)마다 { (순서=0) 조작단계 보여주기. }.";
        let mut request=manipulation_request(source);request.snapshots[0].events.truncate(1);
        let mut session=manipulation::ManipulationSession::new(source,request.binding.clone()).unwrap();
        session.step(&request.snapshots[0]).unwrap();
        let mut invalid=request.snapshots[0].clone();invalid.madi=2;
        assert!(session.step(&invalid).is_err());
        let (bytes,summary)=failed_prefix_manipulation(&session).unwrap();
        assert_eq!(summary["outcome"],"ABORTED");
        let expected=ExpectedBinding::Manipulation(&request.binding);let tape=Tape::decode(&bytes,&expected).unwrap();
        assert_eq!(tape.frames.len(),1);assert_eq!(tape.accepted_event_count,1);
        assert!(matches!(&tape.frames[0].input.events[0].event,EventKind::Coordinate{phase:manipulation::Phase::Begin,..}));
        assert!(replay(source,&bytes,expected).is_err());assert!(session.seal().is_err());
    }
    #[test]
    fn v3_live_explicit_stop_preserves_prefix_and_world_but_is_not_complete() {
        let source="(매마디)마다 { 1 보여주기. }.";
        let request=logical_request(source);
        let mut session=logical::LogicalInputSession::new(source,request.binding.clone()).unwrap();
        let mut controller=LiveRecording::default();controller.ensure_running().unwrap();
        session.step(&request.snapshots[0]).unwrap();let before=session.summary();
        let (bytes,summary)=controller.stop_logical(&session).unwrap();
        assert_eq!(summary,before);assert_eq!(session.summary(),before);
        let expected=ExpectedBinding::Logical(&request.binding);let tape=Tape::decode(&bytes,&expected).unwrap();
        assert_eq!(tape.termination,logical::Termination::Incomplete);assert_eq!(tape.frames.len(),1);
        assert!(controller.ensure_running().is_err());assert!(controller.stop_logical(&session).is_err());
        assert!(replay(source,&bytes,expected).is_err());
        let source="(매마디)마다 { (순서=0) 조작단계 보여주기. }.";
        let mut request=manipulation_request(source);request.snapshots[0].events.truncate(1);
        let mut session=manipulation::ManipulationSession::new(source,request.binding.clone()).unwrap();
        session.step(&request.snapshots[0]).unwrap();let before=session.summary();
        let mut controller=LiveRecording::default();let (bytes,_)=controller.stop_manipulation(&session).unwrap();
        assert_eq!(session.summary(),before);
        let tape=Tape::decode(&bytes,&ExpectedBinding::Manipulation(&request.binding)).unwrap();
        assert_eq!(tape.termination,logical::Termination::Incomplete);assert_eq!(tape.accepted_event_count,1);
        assert!(matches!(&tape.frames[0].input.events[0].event,EventKind::Coordinate{phase:manipulation::Phase::Begin,..}));
    }
}
