//! Accepted output content, never a source/restore/normal-end permission.
//! The execution owner prepares a batch, finishes all fallible checks, then
//! consumes its private token immediately before publishing the whole madi.
use std::rc::Rc;
use super::typed_output::{StagedOutputV1,StagedResultEmissionsV1,MAX_STAGED_BYTES};

pub(crate) struct PreparedOutputMadiV1 {
    owner:Rc<()>,
    madi:u64,
    candidate:Result<(Vec<StagedOutputV1>,usize),String>,
}

/// A process-local capability, never a semantic identity or serialized field.
#[derive(Clone)]
pub(crate) struct ExecutionGenerationV1(Rc<()>);
impl ExecutionGenerationV1 {
    pub(crate) fn matches(&self,other:&Self)->bool {Rc::ptr_eq(&self.0,&other.0)}
}

pub(crate) struct ResultEmissionSinkV1 {
    owner:Rc<()>,
    next_madi:u64,
    byte_length:usize,
    rows:Vec<StagedOutputV1>,
    unavailable:Option<String>,
}

impl Default for ResultEmissionSinkV1 {
    fn default()->Self {Self{owner:Rc::new(()),next_madi:0,byte_length:8,rows:Vec::new(),unavailable:None}}
}

impl ResultEmissionSinkV1 {
    pub(crate) fn generation(&self)->ExecutionGenerationV1 {ExecutionGenerationV1(Rc::clone(&self.owner))}
    pub(crate) fn prepare_madi(&mut self,staged:StagedResultEmissionsV1,madi:u64)
        ->Result<PreparedOutputMadiV1,String> {
        if madi!=self.next_madi || madi==u64::MAX {
            return Err("E_RESULT_EMISSION_ORDER: 확정 출력 마디가 연속되지 않습니다".into());
        }
        let candidate=(|| {
            let rows=staged.into_checked_madi(madi)?;
            let mut length=self.byte_length;
            for row in &rows {
                length=length.checked_add(34).and_then(|n|n.checked_add(row.type_bytes.len()))
                    .and_then(|n|n.checked_add(row.value_bytes.len()))
                    .filter(|n|*n<=MAX_STAGED_BYTES)
                    .ok_or_else(||"E_UNSUPPORTED_RESULT_EVIDENCE: 출력 증거 한도를 넘었습니다".to_owned())?;
            }
            self.rows.try_reserve(rows.len()).map_err(|_|
                "E_UNSUPPORTED_RESULT_EVIDENCE: 출력 증거를 보존할 메모리가 없습니다".to_owned())?;
            Ok((rows,length))
        })();
        Ok(PreparedOutputMadiV1{owner:Rc::clone(&self.owner),madi,candidate})
    }

    /// New evidence-required execution calls this before world publication.
    /// Legacy execution may retain an explicit unavailable marker instead;
    /// that marker can never produce result bytes or a successful receipt.
    pub(crate) fn require_exact(&self,prepared:&PreparedOutputMadiV1)->Result<(),String> {
        self.check_token(prepared)?;
        if let Some(reason)=&self.unavailable {return Err(reason.clone());}
        prepared.candidate.as_ref().map(|_|()).map_err(Clone::clone)
    }

    fn check_token(&self,prepared:&PreparedOutputMadiV1)->Result<(),String> {
        if !Rc::ptr_eq(&self.owner,&prepared.owner) || self.next_madi!=prepared.madi {
            return Err("E_RESULT_EMISSION_STALE_OWNER: 다른 실행 또는 지난 출력 후보입니다".into());
        }
        Ok(())
    }

    pub(crate) fn commit_madi(&mut self,prepared:PreparedOutputMadiV1)->Result<(),String> {
        self.check_token(&prepared)?;
        match prepared.candidate {
            Ok((rows,length)) if self.unavailable.is_none()=>{
                self.rows.extend(rows);self.byte_length=length;
            },
            Ok(_)=>{},
            Err(reason)=>{if self.unavailable.is_none(){self.unavailable=Some(reason);}},
        }
        self.next_madi+=1;
        Ok(())
    }

    pub(crate) fn next_madi(&self)->u64 {self.next_madi}

    /// Content exporter only. The result owner must separately hold the
    /// original seed and a real committed-boundary borrow before hashing.
    pub(crate) fn content_bytes(&self)->Result<Vec<u8>,String> {
        if let Some(reason)=&self.unavailable {return Err(reason.clone());}
        let mut out=Vec::new();out.try_reserve(self.byte_length).map_err(|_|
            "E_UNSUPPORTED_RESULT_EVIDENCE: 출력 정본 메모리 한도를 확인할 수 없습니다".to_owned())?;
        out.extend_from_slice(&(self.rows.len() as u64).to_be_bytes());
        let mut last_madi=None;let mut ordinal=0u64;
        for row in &self.rows {
            if last_madi!=Some(row.madi){last_madi=Some(row.madi);ordinal=0;}
            out.extend_from_slice(&row.madi.to_be_bytes());out.extend_from_slice(&ordinal.to_be_bytes());
            for (tag,bytes) in [(1u8,&row.type_bytes),(2u8,&row.value_bytes)] {
                out.push(tag);out.extend_from_slice(&(bytes.len() as u64).to_be_bytes());out.extend_from_slice(bytes);
            }
            ordinal=ordinal.checked_add(1).ok_or("E_RESULT_EMISSION_ORDER: 출력 순번 초과")?;
        }
        debug_assert_eq!(out.len(),self.byte_length);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn foreign_and_stale_batches_cannot_publish() {
        let mut a=ResultEmissionSinkV1::default();let mut b=ResultEmissionSinkV1::default();
        let batch=a.prepare_madi(Default::default(),0).unwrap();
        assert!(b.commit_madi(batch).unwrap_err().starts_with("E_RESULT_EMISSION_STALE_OWNER"));
        assert_eq!(b.next_madi(),0);assert_eq!(b.content_bytes().unwrap(),0u64.to_be_bytes());
        let stale=a.prepare_madi(Default::default(),0).unwrap();
        let actual=a.prepare_madi(Default::default(),0).unwrap();a.commit_madi(actual).unwrap();
        assert!(a.commit_madi(stale).is_err());assert_eq!(a.next_madi(),1);
    }

    #[test]
    fn unsupported_type_cannot_become_result_content() {
        let mut staged=StagedResultEmissionsV1::default();
        staged.stage(0,None,&Default::default(),&ddonirang_lang::runtime::Value::Bool(true),None,&[]);
        let mut sink=ResultEmissionSinkV1::default();let prepared=sink.prepare_madi(staged,0).unwrap();
        assert!(sink.require_exact(&prepared).is_err());assert_eq!(sink.next_madi(),0);
        sink.commit_madi(prepared).unwrap();assert!(sink.content_bytes().is_err());
    }
}
