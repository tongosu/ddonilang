//! Ordinary accepted input/result data. Not a normal-end or Native permission.
use super::ResultPreimageV1;
pub(crate) struct GeneralProgressEvidenceV1{
    pub(super) input_contract:Vec<u8>,pub(super) input_evidence:Vec<u8>,
    pub(super) accepted_count:u64,pub(super) actual_receipts:Vec<u8>,pub(super) result:ResultPreimageV1,
}
impl GeneralProgressEvidenceV1{
    pub(crate) fn accepted_count(&self)->u64{self.accepted_count}
    pub(crate) fn input_contract(&self)->&[u8]{&self.input_contract}
    pub(crate) fn input_evidence(&self)->&[u8]{&self.input_evidence}
    pub(crate) fn actual_receipts(&self)->&[u8]{&self.actual_receipts}
    pub(crate) fn result_canonical_bytes(&self)->&[u8]{self.result.canonical_bytes()}
    pub(crate) fn capture(boundary:crate::runtime_surface::general_result::CommittedGeneralResultBoundaryV1<'_>)->Result<Self,String>{
        let (input_contract,input_evidence,accepted_count,actual_receipts)=boundary.transport_parts()?;
        let result=ResultPreimageV1::capture_general(boundary)?;
        Ok(Self{input_contract,input_evidence,accepted_count,actual_receipts,result})
    }
}
