//! Structural fixtures only. No source, Native owner, receipt, or permit is
//! fabricated here; full issuer/SQLite/cold recovery tests are separate gates.
use super::*;

fn lifecycle(phase:u8,ordinal:u64,row:&[u8])->Vec<u8>{
    let zero=[0;32];let generation=1u64.to_be_bytes();let epoch=0u64.to_be_bytes();
    let ordinal=ordinal.to_be_bytes();let count=3u64.to_be_bytes();let row_hash=Sha256::digest(row);
    let fields:[&[u8];17]=[&zero,&zero,&generation,&zero,&zero,&epoch,&ordinal,&[phase],&[1],
        &zero,&zero,&row_hash,&zero,&zero,&count,&zero,&[1]];
    let mut raw=b"DDN-WO23-REPLAY-LIFECYCLE-V1\0".to_vec();raw.extend_from_slice(&[0,1,17]);
    for (index,field) in fields.into_iter().enumerate(){
        raw.push(index as u8+1);raw.extend_from_slice(&(field.len() as u64).to_be_bytes());raw.extend_from_slice(field);
    }raw
}
fn intent(payload:&[u8])->Vec<Vec<u8>>{
    let zero=vec![0;32];let row=b"structural-only-row-not-native-admitted";
    let mut notice=vec![1];notice.extend_from_slice(&10u64.to_be_bytes());
    notice.extend_from_slice(&11u64.to_be_bytes());notice.extend_from_slice(&[0;32]);
    vec![zero.clone(),zero.clone(),zero.clone(),1u64.to_be_bytes().to_vec(),zero.clone(),
        0u64.to_be_bytes().to_vec(),2u64.to_be_bytes().to_vec(),zero.clone(),
        lifecycle(1,1,row),lifecycle(0,2,row),lifecycle(1,2,row),row.to_vec(),zero.clone(),zero,
        3u64.to_be_bytes().to_vec(),vec![1],notice,Sha256::digest(payload).to_vec(),
        (payload.len() as u64).to_be_bytes().to_vec(),1u32.to_be_bytes().to_vec()]
}
fn header(kind:RecordKindV1,fields:&[Vec<u8>])->Result<Vec<u8>,String>{
    encode_header(kind,&fields.iter().map(Vec::as_slice).collect::<Vec<_>>())
}
#[test]
fn wo25_finish_structural_header_and_stream_hash_preserve_bytes(){
    let payload=b"{\"summary\":{},\"tape_bytes\":[]}";
    let raw=header(RecordKindV1::Intent,&intent(payload)).unwrap();
    let checked=RecordHeaderV1::parse(&raw).unwrap();
    assert_eq!(checked.kind(),RecordKindV1::Intent);assert_eq!(checked.chunk_count(),1);
    assert_eq!(checked.payload_length(),payload.len() as u64);
    assert_eq!(checked.field(17).unwrap().len(),49,"notice uses event order, not the madi count3");
    let actual=checked.check_payload([payload.as_slice()]).unwrap();
    let mut expected=Sha256::new();expected.update(&raw);expected.update(payload);
    assert_eq!(actual,<[u8;32]>::from(expected.finalize()));
    assert_eq!(checked.field(0),None);assert_eq!(checked.field(21),None);
}
#[test]
fn wo25_finish_structural_rejects_tag_payload_and_oversize_before_allocation(){
    let payload=b"prepared payload, not a C01 admission";
    let fields=intent(payload);let raw=header(RecordKindV1::Intent,&fields).unwrap();
    let mut wrong_tag=raw.clone();wrong_tag[DOMAIN.len()+4]=2;
    assert!(RecordHeaderV1::parse(&wrong_tag).is_err());
    let mut wrong_payload=payload.to_vec();wrong_payload[0]^=1;
    assert!(RecordHeaderV1::parse(&raw).unwrap().check_payload([wrong_payload.as_slice()]).is_err());
    let mut oversized=fields;oversized[18]=u64::MAX.to_be_bytes().to_vec();
    assert!(header(RecordKindV1::Intent,&oversized).is_err());
}
#[test]
fn wo25_finish_failure_has_actual_native_code_and_original_utf8_shape(){
    use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1;
    let message="내구 확인 실패: 원 오류와 rollback 표현 보존".as_bytes();
    let mut fields=vec![vec![9;32],vec![9],vec![1],
        (SnapshotV2DurabilityFailureV1::SyncFailed as u16).to_be_bytes().to_vec(),
        Sha256::digest(message).to_vec(),(message.len() as u64).to_be_bytes().to_vec(),1u32.to_be_bytes().to_vec()];
    let raw=header(RecordKindV1::ActualFailure,&fields).unwrap();
    let checked=RecordHeaderV1::parse(&raw).unwrap();
    assert_eq!(checked.kind(),RecordKindV1::ActualFailure);
    assert!(checked.check_payload([message]).is_ok());
    fields[3]=u16::MAX.to_be_bytes().to_vec();
    assert!(header(RecordKindV1::ActualFailure,&fields).is_err());
}
