//! Private BUNDLE1 structural bytes. Parsing never grants Native custody.
const DOMAIN:&[u8]=b"DDN-WO23-REPLAY-BUNDLE-V1\0";
const MAX:usize=134_217_728;
pub(super) fn encode_planned(fields:[&[u8];11])->Result<Vec<u8>,String> {
    let length=fields.iter().try_fold(DOMAIN.len()+3,|n,p|n.checked_add(9).and_then(|n|n.checked_add(p.len())))
        .filter(|n|*n<=MAX).ok_or("E_REPLAY_BUNDLE_LIMIT: 재생 묶음 한도 초과")?;
    let mut bytes=Vec::new();bytes.try_reserve_exact(length).map_err(|_|"E_REPLAY_BUNDLE_LIMIT: 재생 묶음 메모리 부족")?;
    bytes.extend_from_slice(DOMAIN);bytes.extend_from_slice(&[0,1,11]);
    for (i,field) in fields.into_iter().enumerate(){bytes.push(i as u8+1);bytes.extend_from_slice(&(field.len() as u64).to_be_bytes());bytes.extend_from_slice(field);}
    ReplayBundleCandidateV1::parse(&bytes)?;Ok(bytes)
}
pub(crate) struct ReplayBundleCandidateV1<'a> {fields:[&'a [u8];11]}
impl<'a> ReplayBundleCandidateV1<'a> {
    pub(crate) fn parse(bytes:&'a [u8])->Result<Self,String> {
        if bytes.len()>MAX || !bytes.starts_with(DOMAIN)
            || bytes.get(DOMAIN.len()..DOMAIN.len()+3)!=Some(&[0,1,11][..]) {
            return Err("E_REPLAY_BUNDLE_FORMAT: 재생 묶음 머리·한도가 다릅니다".into());
        }
        let mut fields=[&[][..];11];let mut offset=DOMAIN.len()+3;
        for (i,field) in fields.iter_mut().enumerate(){
            let header_end=offset.checked_add(9).ok_or("E_REPLAY_BUNDLE_FORMAT: 길이 초과")?;
            let header=bytes.get(offset..header_end).ok_or("E_REPLAY_BUNDLE_FORMAT: 필드가 잘렸습니다")?;
            if header[0]!=i as u8+1{return Err("E_REPLAY_BUNDLE_FORMAT: 필드 순서가 다릅니다".into());}
            let length=usize::try_from(u64::from_be_bytes(header[1..].try_into().unwrap())).map_err(|_|"E_REPLAY_BUNDLE_FORMAT: 필드 길이 초과")?;
            let end=header_end.checked_add(length).ok_or("E_REPLAY_BUNDLE_FORMAT: 필드 길이 초과")?;
            *field=bytes.get(header_end..end).ok_or("E_REPLAY_BUNDLE_FORMAT: 필드가 잘렸습니다")?;offset=end;
        }
        if offset!=bytes.len() || fields[4].len()!=16 || fields[4][..8]!=[0;8]
            || fields[6].len()!=32 || fields[10].len()!=113 {
            return Err("E_REPLAY_BUNDLE_FORMAT: 범위·결과·제어 참조가 다릅니다".into());
        }
        // Typed source/seed/input/receipt/normal-end and trust admission are
        // separate gates, not inferred from this structural candidate.
        Ok(Self{fields})
    }
    pub(crate) fn payload(&self,tag:u8)->Option<&'a [u8]> {tag.checked_sub(1).and_then(|i|self.fields.get(i as usize)).copied()}
}
