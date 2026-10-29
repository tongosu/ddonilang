//! Common data checker, not a Native custody/Ready/normal-end issuer.
use super::*;

fn bounded_tape<'de,D:serde::Deserializer<'de>>(decoder:D)->Result<Vec<u8>,D::Error>{
    struct Bytes;
    impl<'de> serde::de::Visitor<'de> for Bytes{
        type Value=Vec<u8>;
        fn expecting(&self,formatter:&mut std::fmt::Formatter)->std::fmt::Result{formatter.write_str("bounded original Tape bytes")}
        fn visit_seq<A:serde::de::SeqAccess<'de>>(self,mut seq:A)->Result<Vec<u8>,A::Error>{
            let mut bytes=Vec::new();
            while let Some(byte)=seq.next_element::<u8>()?{
                if bytes.len()==MAX_BYTES{return Err(serde::de::Error::custom("원 Tape 길이 한도 초과"));}
                if bytes.len()==bytes.capacity(){
                    bytes.try_reserve(4096.min(MAX_BYTES-bytes.len())).map_err(|_|serde::de::Error::custom("원 Tape 메모리 한도"))?;
                }
                bytes.push(byte);
            }Ok(bytes)
        }
    }
    decoder.deserialize_seq(Bytes)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OriginalResponse {
    #[serde(deserialize_with="bounded_tape")]
    tape_bytes:Vec<u8>,summary:serde_json::Value,
}

/// Compare the authenticated original C01 with the full Native archive.
/// This does not replay, execute, repair, publish Finished or mint a receipt.
pub(crate) fn check_original_finish_response(c01:&[u8],contract:&[u8],evidence:&[u8],receipts:&[u8],
    binding:&logical::LogicalInputBinding,closed_count:u64,notice:&[u8])->Result<serde_json::Value,String>{
    if c01.len() as u64>crate::replay_evidence::finish_recovery::PAYLOAD_LIMIT{return Err(error("원 종료 응답 한도 초과"));}
    let response:OriginalResponse=serde_json::from_slice(c01).map_err(|_|error("원 종료 응답의 두 필드·중복·길이 구조가 다릅니다"))?;
    if contract!=logical_contract_bytes(binding)? || evidence.first()!=Some(&1) || evidence[1..]!=response.tape_bytes {
        return Err(error("원 종료 응답과 저장된 입력이 다릅니다"));
    }
    let tape=Tape::decode(&response.tape_bytes,&ExpectedBinding::Logical(binding))?;
    if tape.termination!=logical::Termination::Complete || tape.failure_reason.is_some()
        || tape.accepted_madi_count!=closed_count{return Err(error("원 종료 상태·마디 범위가 다릅니다"));}
    let summary=response.summary.as_object().ok_or_else(||error("원 종료 요약이 아닙니다"))?;
    let state=tape.frames.last().map(|frame|frame.accepted_world_digest.as_str()).unwrap_or(&binding.initial_world);
    if summary.get("madi_count").and_then(serde_json::Value::as_u64)!=Some(closed_count)
        || summary.get("state_hash").and_then(serde_json::Value::as_str)!=Some(state)
        || summary.contains_key("failure") || summary.contains_key("outcome"){
        return Err(error("원 종료 요약과 확정 입력 결과가 다릅니다"));
    }
    let frames=summary.get("frames").and_then(serde_json::Value::as_array).ok_or_else(||error("원 종료 입력 요약 누락"))?;
    if frames.len()!=tape.frames.len(){return Err(error("원 종료 입력 요약 범위가 다릅니다"));}
    for (actual,frame) in frames.iter().zip(&tape.frames){
        if *actual!=serde_json::to_value(frame.logical()?).map_err(|_|error("입력 요약 직렬화 오류"))?{
            return Err(error("원 종료 입력 요약 내용이 다릅니다"));
        }
    }
    let expected_receipts=crate::replay_evidence::decode_receipt_array(receipts,closed_count)?;
    if summary.get("input_receipts")!=Some(&serde_json::Value::Array(expected_receipts)){
        return Err(error("원 종료 입력 영수증이 다릅니다"));
    }
    let excluded=match notice{
        [0]=>0,
        raw if raw.len()==49 && raw[0]==1=>{
            let first=u64::from_be_bytes(raw[1..9].try_into().map_err(|_|error("원 안내 범위 오류"))?);
            let until=u64::from_be_bytes(raw[9..17].try_into().map_err(|_|error("원 안내 범위 오류"))?);
            if first!=tape.accepted_event_count{return Err(error("원 안내와 처리 사건 범위가 다릅니다"));}
            until.checked_sub(first).ok_or_else(||error("원 안내 범위가 거꾸로입니다"))?
        },
        _=>return Err(error("원 안내 종류나 결속이 누락됐습니다")),
    };
    Ok(serde_json::json!({"excluded_input_count":excluded.to_string()}))
}
