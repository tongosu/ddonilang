//! Exact child custody bytes; not the existing RootGrant/token profile.
pub(crate) const DOMAIN:&[u8]=b"DDN-WO23-REPLAY-CUSTODY-V1\0";
const LENGTHS:[usize;9]=[32,32,32,32,32,32,32,1,8];
pub(crate) struct ReplayCustodyPlainV1 {bytes:Vec<u8>}
impl ReplayCustodyPlainV1 {
    #[cfg(windows)]
    pub(super) fn from_plan(plan:&super::ReplayCheckpointPlanDataV1,
        adapter:&crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1)->Result<Self,String> {
        let (project,nonce,fingerprint,generation)=plan.scope.identity_fields(adapter)
            .map_err(|e|format!("E_REPLAY_CUSTODY_ROOT: {e:?}"))?;
        let generation=generation.to_be_bytes();let end_kind=[plan.end_owner_kind];
        let fields:[&[u8];9]=[&project,&nonce,&fingerprint,&plan.checkpoint_tx,&plan.bundle_digest,
            &plan.binding.pesi,&plan.result_digest,&end_kind,&generation];
        let mut bytes=Vec::new();bytes.extend_from_slice(DOMAIN);bytes.extend_from_slice(&[0,1,9]);
        for (i,field) in fields.into_iter().enumerate(){bytes.push(i as u8+1);bytes.extend_from_slice(&(field.len() as u64).to_be_bytes());bytes.extend_from_slice(field);}
        ReplayCustodyCandidateV1::parse(&bytes)?;Ok(Self{bytes})
    }
    pub(crate) fn canonical_bytes(&self)->&[u8] {&self.bytes}
}
pub(crate) struct ReplayCustodyCandidateV1<'a>{fields:[&'a [u8];9]}
impl<'a> ReplayCustodyCandidateV1<'a> {
    pub(crate) fn parse(bytes:&'a [u8])->Result<Self,String> {
        let exact=DOMAIN.len()+3+9*9+LENGTHS.iter().sum::<usize>();
        if bytes.len()!=exact || !bytes.starts_with(DOMAIN)
            || bytes.get(DOMAIN.len()..DOMAIN.len()+3)!=Some(&[0,1,9][..]) {
            return Err("E_REPLAY_CUSTODY_FORMAT: 보호 자료의 머리·길이가 다릅니다".into());
        }
        let mut fields=[&[][..];9];let mut offset=DOMAIN.len()+3;
        for (i,field) in fields.iter_mut().enumerate(){
            let header=bytes.get(offset..offset+9).ok_or("E_REPLAY_CUSTODY_FORMAT")?;
            if header[0]!=i as u8+1 || u64::from_be_bytes(header[1..].try_into().unwrap())!=LENGTHS[i] as u64 {
                return Err("E_REPLAY_CUSTODY_FORMAT: 보호 필드 순서·길이가 다릅니다".into());
            }
            offset+=9;*field=&bytes[offset..offset+LENGTHS[i]];offset+=LENGTHS[i];
        }
        if fields[7][0]>2{return Err("E_REPLAY_CUSTODY_FORMAT: 종료 권한 종류가 다릅니다".into());}
        Ok(Self{fields})
    }
    pub(crate) fn matches_root(&self,project:&[u8;32],nonce:&[u8;32],fingerprint:&[u8;32],generation:u64)->bool {
        self.fields[0]==project && self.fields[1]==nonce && self.fields[2]==fingerprint && self.fields[8]==generation.to_be_bytes()
    }
    pub(crate) fn payload(&self,tag:u8)->Option<&'a [u8]> {tag.checked_sub(1).and_then(|i|self.fields.get(i as usize)).copied()}
}
