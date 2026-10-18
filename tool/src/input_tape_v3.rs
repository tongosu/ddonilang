//! Approved transport candidate. The binding retains its admitted execution
//! profile; this checksum never issues input or replay authority.
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::{logical_input_tape as logical, manipulation_input as manipulation};

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
fn aborted_tape(binding:Binding,frames:Vec<Frame>,failure:Option<&str>)->Result<Tape,String> {
    let reason=failure.filter(|reason|!reason.is_empty()).ok_or_else(||error("실제 실패가 없는 실행을 실패 기록으로 만들 수 없습니다"))?;
    let count=frames.iter().try_fold(0u64,|count,frame|count.checked_add(frame.input.events.len() as u64)).ok_or_else(||error("사건 합계 초과"))?;
    Ok(Tape {binding,accepted_madi_count:frames.len() as u64,accepted_event_count:count,frames,
        termination:logical::Termination::Aborted,failure_reason:Some(reason.into())})
}
/// One recording controller belongs to one VM. A live user stop is not a
/// finite-plan completion proof, and cannot be supplied as a host Boolean.
#[derive(Default)]
pub(crate) struct LiveRecording { stopped:bool }
impl LiveRecording {
    pub(crate) fn ensure_running(&self)->Result<(),String> {
        if self.stopped {Err(error("기록을 중단했습니다. 다시 시작하거나 저장된 작품을 다시 여세요"))}else{Ok(())}
    }
    fn stop(&mut self,expected:ExpectedBinding<'_>,frames:Vec<Frame>,failure:Option<&str>)->Result<Vec<u8>,String> {
        self.ensure_running()?;
        let tape=if failure.is_some() {aborted_tape(expected.binding(),frames,failure)?}else{
            let count=frames.iter().try_fold(0u64,|count,frame|count.checked_add(frame.input.events.len() as u64)).ok_or_else(||error("사건 합계 초과"))?;
            Tape {binding:expected.binding(),accepted_madi_count:frames.len() as u64,accepted_event_count:count,frames,
                termination:logical::Termination::Incomplete,failure_reason:Some("사용자가 기록을 명시적으로 중단했습니다".into())}
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
pub fn replay_authored_logical(authored:&str,bytes:&[u8],binding:&logical::LogicalInputBinding)->Result<serde_json::Value,String> {
    let tape=Tape::decode(bytes,&ExpectedBinding::Logical(binding))?;
    if tape.termination!=logical::Termination::Complete {return Err(error("미완·실패 기록을 정상 재생하지 않습니다"));}
    let mut run=crate::runtime_surface::fresh_authored_logical_run(authored,binding.clone())?;
    for frame in &tape.frames {run.replay_v3_frame(&frame.logical()?)?;}
    Ok(run.session().summary())
}
pub fn record_manipulation(source:&str,request:&manipulation::RecordRequest)->Result<(Vec<u8>,serde_json::Value),String> {
    let mut session=manipulation::ManipulationSession::new(source,request.binding.clone())?;
    for input in &request.snapshots {session.step(input)?;}
    let (binding,frames,failure)=session.accepted_prefix_v3();
    let tape=finite_tape(ExpectedBinding::Manipulation(binding).binding(),frames.iter().map(|frame|Frame::from_input(Snapshot::from(&frame.input),&frame.accepted_world_digest)).collect::<Result<_,_>>()?,request.snapshots.len(),failure)?;
    Ok((tape.encode(&ExpectedBinding::Manipulation(&request.binding))?,session.summary()))
}
pub fn replay(source:&str,bytes:&[u8],expected:ExpectedBinding<'_>)->Result<serde_json::Value,String> {
    let tape=Tape::decode(bytes,&expected)?;
    if tape.termination!=logical::Termination::Complete {return Err(error("미완·실패 기록을 정상 재생하지 않습니다"));}
    match expected {
        ExpectedBinding::Logical(binding)=>{
            let mut session=logical::LogicalInputSession::new(source,binding.clone())?;
            for frame in &tape.frames {session.replay_v3_frame(&frame.logical()?)?;}
            Ok(session.summary())
        }
        ExpectedBinding::Manipulation(binding)=>{
            let mut session=manipulation::ManipulationSession::new(source,binding.clone())?;
            for frame in &tape.frames {session.replay_v3_frame(&frame.manipulation()?)?;}
            Ok(session.summary())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
