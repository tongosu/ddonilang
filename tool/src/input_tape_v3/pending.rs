//! Runtime-only frontend queue handshake. These fields never enter V3 Tape,
//! RESULT1, an input receipt, or a saved semantic identity.
use super::*;

pub(super) struct PendingSupplyOwnerV1 {token:String}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingAtFinishV1 {
    producer_generation:String,
    first_unprocessed:String,
    observed_until:String,
    #[serde(deserialize_with="bounded_events")]
    events:Vec<Event>,
}
pub(super) struct CheckedPendingAtFinishV1 {
    first:u64,
    until:u64,
    digest:[u8;32],
}
impl CheckedPendingAtFinishV1 {
    pub(super) fn count(&self)->u64 {self.until-self.first}
    /// Read-only R2 export from the checked, actual frontend queue owner.
    /// An empty observed range still has tag01; it is not a direct owner.
    pub(super) fn recovery_payload(&self)->Vec<u8> {
        let mut raw=Vec::with_capacity(49);raw.push(1);
        raw.extend_from_slice(&self.first.to_be_bytes());
        raw.extend_from_slice(&self.until.to_be_bytes());
        raw.extend_from_slice(&self.digest);raw
    }
    pub(super) fn matches(&self,other:&Self)->bool {
        self.first==other.first && self.until==other.until && self.digest==other.digest
    }
}
fn counter(text:&str)->Result<u64,String> {
    let value=text.parse::<u64>().map_err(|_|error("MissingPendingSupply: 순서 토큰이 아닙니다"))?;
    if value.to_string()!=text {return Err(error("MissingPendingSupply: 정본 순서 토큰이 아닙니다"));}
    Ok(value)
}
impl PendingSupplyOwnerV1 {
    pub(super) fn issue()->Result<Self,String> {
        // A counter would restart when a worker/process is replaced. This
        // ephemeral callback capability never enters world/RNG/record bytes,
        // storage identities or replay. Read-only replay never registers it.
        let mut nonce=[0u8;32];getrandom::fill(&mut nonce)
            .map_err(|_|error("MissingPendingSupply: 실제 입력 생산자의 새 세대를 만들 수 없습니다"))?;
        Ok(Self{token:format!("wo23-runtime-producer-{}",hex::encode(nonce))})
    }
    pub(super) fn registration(&self,accepted_events:u64)->serde_json::Value {
        serde_json::json!({"producer_generation":self.token,"accepted_event_count":accepted_events.to_string()})
    }
    pub(super) fn check(&self,packet:&str,accepted_events:u64,expected:&ExpectedBinding<'_>)
        ->Result<CheckedPendingAtFinishV1,String> {
        if packet.len()>MAX_BYTES{return Err(error("MissingPendingSupply: queue 운반 한도 초과"));}
        let pending:PendingAtFinishV1=serde_json::from_str(packet)
            .map_err(|_|error("MissingPendingSupply: 실제 입력 queue 공급이 필요합니다"))?;
        if pending.producer_generation!=self.token {
            return Err(error("MissingPendingSupply: 지난 입력 생산자입니다"));
        }
        let first=counter(&pending.first_unprocessed)?;let until=counter(&pending.observed_until)?;
        if first!=accepted_events || first.checked_add(pending.events.len() as u64)!=Some(until) {
            return Err(error("MissingPendingSupply: 처리 입력과 미처리 queue 범위가 겹치거나 빠졌습니다"));
        }
        let binding=expected.binding();
        for (i,event) in pending.events.iter().enumerate() {
            if event.ordinal!=i as u64{return Err(error("MissingPendingSupply: queue 순서가 끊겼습니다"));}
            let kind=binding.channels.get(&event.channel).ok_or_else(||error("MissingPendingSupply: 선언 없는 채널"))?;
            let compatible=matches!((kind,&event.event),
                (Channel::Control,EventKind::Press|EventKind::Repeat|EventKind::Release)
                |(Channel::Text,EventKind::CommittedText(_)|EventKind::Paste(_))
                |(Channel::Number{..},EventKind::Number{..})
                |(Channel::Coordinate{..},EventKind::Coordinate{..}));
            if !compatible{return Err(error("MissingPendingSupply: queue 사건과 채널 종류가 다릅니다"));}
        }
        // The ordinal of each item is first+i. The exact submitted items are
        // retained only as private handshake content, never executed here.
        let bytes=canonical(&pending.events)?;
        Ok(CheckedPendingAtFinishV1{first,until,digest:digest(b"ddn.wo23.pending-finish.v1\0",&bytes)})
    }
}
