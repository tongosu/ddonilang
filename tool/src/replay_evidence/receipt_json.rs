//! Private canonical receipt transport. Never redefines snapshot SHA bytes.
use serde_json::Value;
const MAX:usize=134_217_728;
fn append(out:&mut Vec<u8>,bytes:&[u8])->Result<(),String> {
    if out.len().checked_add(bytes.len()).is_none_or(|n|n>MAX){return Err("E_REPLAY_RECEIPT_LIMIT: 영수증 운반 한도 초과".into());}
    out.try_reserve(bytes.len()).map_err(|_|"E_REPLAY_RECEIPT_LIMIT: 영수증 메모리 부족")?;
    out.extend_from_slice(bytes);Ok(())
}
fn value(out:&mut Vec<u8>,input:&Value,depth:usize)->Result<(),String> {
    if depth>128{return Err("E_REPLAY_RECEIPT_LIMIT: 영수증 중첩 한도 초과".into());}
    match input {
        Value::Null=>append(out,b"null"),Value::Bool(v)=>append(out,if *v{b"true"}else{b"false"}),
        Value::Number(v)=>{
            if v.as_i64().is_none() && v.as_u64().is_none(){return Err("E_REPLAY_RECEIPT_FLOAT: 영수증 운반은 소수를 재해석하지 않습니다".into());}
            append(out,v.to_string().as_bytes())
        },
        Value::String(v)=>append(out,serde_json::to_string(v).map_err(|_|"E_REPLAY_RECEIPT_ENCODING")?.as_bytes()),
        Value::Array(items)=>{
            append(out,b"[")?;
            for (i,item) in items.iter().enumerate(){if i!=0{append(out,b",")?;}value(out,item,depth+1)?;}
            append(out,b"]")
        },
        Value::Object(items)=>{
            let mut ordered=Vec::new();ordered.try_reserve_exact(items.len()).map_err(|_|"E_REPLAY_RECEIPT_LIMIT")?;
            ordered.extend(items.iter());ordered.sort_by(|a,b|a.0.as_bytes().cmp(b.0.as_bytes()));
            append(out,b"{")?;
            for (i,(key,item)) in ordered.into_iter().enumerate(){
                if i!=0{append(out,b",")?;}
                append(out,serde_json::to_string(key).map_err(|_|"E_REPLAY_RECEIPT_ENCODING")?.as_bytes())?;
                append(out,b":")?;value(out,item,depth+1)?;
            }
            append(out,b"}")
        },
    }
}
/// Canonical transport only, never a receipt/admission issuer.
pub(crate) fn canonical_transport_json(input:&Value)->Result<Vec<u8>,String>{
    let mut bytes=Vec::new();value(&mut bytes,input,0)?;Ok(bytes)
}
pub(crate) fn actual_receipt_array(receipts:&[Value])->Result<Vec<u8>,String> {
    let mut out=Vec::new();append(&mut out,&(receipts.len() as u64).to_be_bytes())?;
    for receipt in receipts {
        let mut bytes=Vec::new();value(&mut bytes,receipt,0)?;
        append(&mut out,&[1])?;append(&mut out,&(bytes.len() as u64).to_be_bytes())?;append(&mut out,&bytes)?;
    }
    Ok(out)
}
/// Structural decoding only. Typed replay must still reproduce all receipts.
pub(crate) fn decode_receipt_array(bytes:&[u8],max_count:u64)->Result<Vec<Value>,String>{
    if bytes.len()>MAX||bytes.len()<8{return Err("E_REPLAY_RECEIPT_FORMAT".into());}
    // F5-GA[RECEIPT_COUNT]: 위 최소8바이트 검사 뒤 정확히 앞8바이트만 정수로 읽는다.
    let count=u64::from_be_bytes(bytes[..8].try_into().unwrap());
    if count>max_count||count as u128>(bytes.len() as u128-8)/9{return Err("E_REPLAY_RECEIPT_LIMIT".into());}
    let count=usize::try_from(count).map_err(|_|"E_REPLAY_RECEIPT_LIMIT")?;
    let mut receipts=Vec::new();receipts.try_reserve_exact(count).map_err(|_|"E_REPLAY_RECEIPT_LIMIT")?;
    let mut cursor=8usize;
    for _ in 0..count{
        let end=cursor.checked_add(9).ok_or("E_REPLAY_RECEIPT_FORMAT")?;
        let header=bytes.get(cursor..end).ok_or("E_REPLAY_RECEIPT_FORMAT")?;
        // F5-GA[RECEIPT_HEADER]: checked_add(9)와 get 뒤 header는 정확히9바이트라 0번과 나머지8바이트 정수 변환이 안전하다.
        if header[0]!=1{return Err("E_REPLAY_RECEIPT_FORMAT".into());}
        let length=usize::try_from(u64::from_be_bytes(header[1..].try_into().unwrap())).map_err(|_|"E_REPLAY_RECEIPT_LIMIT")?;
        cursor=end;let end=cursor.checked_add(length).ok_or("E_REPLAY_RECEIPT_FORMAT")?;
        let raw=bytes.get(cursor..end).ok_or("E_REPLAY_RECEIPT_FORMAT")?;
        let value:Value=serde_json::from_slice(raw).map_err(|_|"E_REPLAY_RECEIPT_FORMAT")?;
        if canonical_transport_json(&value)?!=raw{return Err("E_REPLAY_RECEIPT_NONCANONICAL".into());}
        receipts.push(value);cursor=end;
    }
    if cursor!=bytes.len(){return Err("E_REPLAY_RECEIPT_FORMAT".into());}Ok(receipts)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn b3_receipt_transport_preserves_large_integer_and_unicode_without_float_coercion() {
        let input=serde_json::json!({"한글":"e\u{301}","z":18446744073709551615u64,"a":"\n\\\""});
        let mut bytes=Vec::new();value(&mut bytes,&input,0).unwrap();
        assert_eq!(std::str::from_utf8(&bytes).unwrap(),"{\"a\":\"\\n\\\\\\\"\",\"z\":18446744073709551615,\"한글\":\"e\u{301}\"}");
        assert!(value(&mut Vec::new(),&serde_json::json!(1.0),0).is_err());
    }
}
