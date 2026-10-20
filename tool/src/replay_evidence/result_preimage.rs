//! RESULT1 content, distinct from state_hash, a receipt or a replay verdict.
use sha2::{Digest,Sha256};
const DOMAIN:&[u8]=b"DDN-WO23-RESULT-V1\0";
const MAX_BYTES:usize=134_217_728;

pub(crate) struct ResultPreimageV1 {
    bytes:Vec<u8>,
    digest:[u8;32],
}

impl ResultPreimageV1 {
    /// No JSON/bytes/expected-success constructor exists at this boundary.
    /// The borrow can only be issued by the actual logical execution owner.
    pub(crate) fn capture(boundary:crate::logical_input_tape::CommittedLogicalResultBoundary<'_>)
        ->Result<Self,String> {
        let (world,outputs,rng,next)=boundary.content_parts()?;
        Self::encode_parts(world,outputs,rng,next)
    }
    pub(crate) fn capture_general(boundary:crate::runtime_surface::general_result::CommittedGeneralResultBoundaryV1<'_>)
        ->Result<Self,String>{
        let (world,outputs,rng,next)=boundary.content_parts()?;
        Self::encode_parts(world,outputs,rng,next)
    }
    pub(crate) fn capture_general_readonly(boundary:crate::unit_wire::resumable_complete_state::GeneralReadOnlyBoundaryV1<'_>)
        ->Result<Self,String>{
        let (world,outputs,rng,next)=boundary.content_parts()?;Self::encode_parts(world,outputs,rng,next)
    }
    fn encode_parts(world:Vec<u8>,outputs:Vec<u8>,rng:Vec<u8>,next:u64)->Result<Self,String>{
        let position=next.to_be_bytes();
        let fields:[&[u8];4]=[&world,&outputs,&rng,&position];
        let size=fields.iter().try_fold(DOMAIN.len()+3,|size,bytes|
            size.checked_add(9).and_then(|n|n.checked_add(bytes.len())))
            .filter(|size|*size<=MAX_BYTES)
            .ok_or("E_RESULT_EVIDENCE_LIMIT: 결과 정본 한도를 넘었습니다")?;
        let mut bytes=Vec::new();bytes.try_reserve(size)
            .map_err(|_|"E_RESULT_EVIDENCE_LIMIT: 결과 정본 메모리를 확보할 수 없습니다")?;
        bytes.extend_from_slice(DOMAIN);bytes.extend_from_slice(&1u16.to_be_bytes());bytes.push(4);
        for (i,field) in fields.into_iter().enumerate() {
            bytes.push((i+1) as u8);bytes.extend_from_slice(&(field.len() as u64).to_be_bytes());
            bytes.extend_from_slice(field);
        }
        debug_assert_eq!(bytes.len(),size);
        let digest=Sha256::digest(&bytes).into();
        Ok(Self{bytes,digest})
    }

    pub(crate) fn canonical_bytes(&self)->&[u8] {&self.bytes}
    pub(crate) fn digest_bytes(&self)->&[u8;32] {&self.digest}
    pub(crate) fn digest_label(&self)->String {format!("sha256:{}",hex::encode(self.digest))}
}
