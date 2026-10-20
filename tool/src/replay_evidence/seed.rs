//! SEED1 content only. It is never a FRAME2 restore token or Native authority.
const DOMAIN:&[u8]=b"DDN-WO23-REPLAY-SEED-V1\0";
const MAX_BYTES:usize=2_000_000;
const TAGS:[u8;17]=[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,18];
pub(crate) struct ReplaySeedV1 {bytes:Vec<u8>}
/// Structural borrow only; owner grammar/source/RNG/custody admission remains
/// mandatory. This cannot construct ReplaySeedV1 or a normal restore permit.
pub(crate) struct ReplaySeedCandidateV1<'a> {fields:[&'a [u8];17]}
impl<'a> ReplaySeedCandidateV1<'a> {
    pub(crate) fn parse(bytes:&'a [u8])->Result<Self,String> {
        if bytes.len()>MAX_BYTES || !bytes.starts_with(DOMAIN)
            || bytes.get(DOMAIN.len()..DOMAIN.len()+3)!=Some(&[0,1,17][..]) {
            return Err("E_REPLAY_SEED_FORMAT: 원 시작 정본 머리·한도가 다릅니다".into());
        }
        let mut offset=DOMAIN.len()+3;let mut fields=[&[][..];17];
        for (i,tag) in TAGS.into_iter().enumerate() {
            let header_end=offset.checked_add(9).ok_or("E_REPLAY_SEED_FORMAT: 길이 초과")?;
            let header=bytes.get(offset..header_end).ok_or("E_REPLAY_SEED_FORMAT: 필드 머리가 잘렸습니다")?;
            if header[0]!=tag{return Err("E_REPLAY_SEED_FORMAT: 원 시작 필드 순서가 다릅니다".into());}
            let length=usize::try_from(u64::from_be_bytes(header[1..].try_into().unwrap()))
                .map_err(|_|"E_REPLAY_SEED_FORMAT: 필드 길이 초과")?;
            let end=header_end.checked_add(length).ok_or("E_REPLAY_SEED_FORMAT: 필드 길이 초과")?;
            fields[i]=bytes.get(header_end..end).ok_or("E_REPLAY_SEED_FORMAT: 필드가 잘렸습니다")?;offset=end;
        }
        if offset!=bytes.len(){return Err("E_REPLAY_SEED_FORMAT: 뒤에 남는 자료가 있습니다".into());}
        Ok(Self{fields})
    }
    pub(crate) fn payload(&self,tag:u8)->Option<&'a [u8]> {
        TAGS.iter().position(|item|*item==tag).map(|i|self.fields[i])
    }
}
impl ReplaySeedV1 {
    #[cfg(windows)]
    pub(crate) fn retain_general_original(
        proof:&crate::runtime_surface::native_source_world::general_recording::GeneralOriginalSeedAdmissionV1,
        zero_candidate:&[u8])->Result<Self,String>{
        let bytes=proof.check_seed_candidate(zero_candidate)?;
        ReplaySeedCandidateV1::parse(bytes)?;
        let mut owned=Vec::new();owned.try_reserve_exact(bytes.len()).map_err(|_|"E_REPLAY_SEED_LIMIT")?;
        owned.extend_from_slice(bytes);Ok(Self{bytes:owned})
    }
    #[cfg(windows)]
    pub(crate) fn retain_native_progress(bytes:&[u8],proof:&crate::unit_wire::resumable_complete_state::CurrentRecordingProgressProofV1)->Result<Self,String>{
        proof.check_seed(bytes)?;ReplaySeedCandidateV1::parse(bytes)?;
        let mut owned=Vec::new();owned.try_reserve_exact(bytes.len()).map_err(|_|"E_REPLAY_SEED_LIMIT: 원 시작 자료 메모리 부족")?;
        owned.extend_from_slice(bytes);Ok(Self{bytes:owned})
    }
    pub(crate) fn capture(boundary:crate::unit_wire::resumable_complete_state::InitialLogicalReplaySeedBoundaryV1<'_>)
        ->Result<Self,String> {
        let payloads=boundary.actual_payloads()?;
        Self::encode_parts(payloads)
    }
    pub(crate) fn capture_general(boundary:crate::runtime_surface::general_result::InitialGeneralReplaySeedBoundaryV1<'_>)
        ->Result<Self,String>{
        Self::encode_parts(boundary.actual_payloads()?)
    }
    fn encode_parts(payloads:[Vec<u8>;17])->Result<Self,String>{
        let size=payloads.iter().try_fold(DOMAIN.len()+3,|n,p|
            n.checked_add(9).and_then(|n|n.checked_add(p.len())))
            .filter(|n|*n<=MAX_BYTES).ok_or("E_REPLAY_SEED_LIMIT: 원 시작 정본 한도 초과")?;
        let mut bytes=Vec::new();bytes.try_reserve_exact(size)
            .map_err(|_|"E_REPLAY_SEED_LIMIT: 원 시작 정본 메모리를 확보할 수 없습니다")?;
        bytes.extend_from_slice(DOMAIN);bytes.extend_from_slice(&1u16.to_be_bytes());bytes.push(17);
        for (tag,payload) in TAGS.into_iter().zip(payloads) {
            bytes.push(tag);bytes.extend_from_slice(&(payload.len() as u64).to_be_bytes());bytes.extend_from_slice(&payload);
        }
        debug_assert_eq!(size,bytes.len());
        Ok(Self{bytes})
    }
    pub(crate) fn canonical_bytes(&self)->&[u8] {&self.bytes}
}
