//! RD-643 shared source admission mechanism. No filesystem/editor reread.
//! Epoch identity MUST come from its semantic owner, never from source hashes
//! or URI/version. This module does not invent or admit an epoch identity.
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

const MAX_SOURCE_BYTES: usize = 2_000_000;
const MAX_EPOCH_BYTES: usize = 4096;
const DOMAIN: &[u8] = b"DDN-SOURCE-SNAPSHOT-IDENTITY-V1\0";

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum AdmissionError { EmptyEpoch, Limit, Bom, Utf8, Exhausted, Poisoned, ForeignReplay }

/// Physical supply is not an upstream semantic admission proof. Keep issuance
/// crate-internal until the actual SourceEpoch owner is connected.
pub(crate) struct EpochSupply(Vec<u8>);
impl EpochSupply {
    pub(crate) fn from_owner_bytes(raw: &[u8]) -> Result<Self, AdmissionError> {
        if raw.is_empty() { return Err(AdmissionError::EmptyEpoch); }
        if raw.len() > MAX_EPOCH_BYTES { return Err(AdmissionError::Limit); }
        Ok(Self(raw.to_vec()))
    }

    /// Product physical projection of the explicitly selected, existing
    /// SourceEpoch enum. Never infer an epoch from source contents or paths.
    pub(crate) fn for_declared_epoch(epoch: ddonirang_lang::SourceEpoch) -> Self {
        let bytes: &[u8] = match epoch {
            ddonirang_lang::SourceEpoch::PreV25 => b"SourceEpochIdentity:pre-v25",
            ddonirang_lang::SourceEpoch::V25 => b"SourceEpochIdentity:v25",
        };
        Self(bytes.to_vec())
    }
}

/// Opaque immutable capture. Its tuple cannot be selected by an adapter.
#[derive(Debug, Clone)]
pub(crate) struct SourceCapture {
    epoch: Vec<u8>, ordinal: u64, bytes: Vec<u8>, content: [u8;32], identity: [u8;32],
}
impl SourceCapture {
    pub(crate) fn epoch_owner_bytes(&self) -> &[u8] { &self.epoch }
    pub(crate) fn bytes(&self) -> &[u8] { &self.bytes }
    pub(crate) fn text(&self) -> &str {
        // Strict UTF-8 was checked before capture publication.
        std::str::from_utf8(&self.bytes).expect("sealed source is UTF-8")
    }
    pub(crate) fn identity(&self) -> &[u8;32] { &self.identity }
    pub(crate) fn content_digest(&self) -> &[u8;32] { &self.content }
    pub(crate) fn ordinal(&self) -> u64 { self.ordinal }
    pub(crate) fn recorded_json(&self) -> Result<String,String> {
        serde_json::to_string(&RecordedSourceCaptureV1 {
            schema:"ddn.recorded_source_capture.v1".into(),
            epoch_identity_hex:hex::encode(&self.epoch),
            admission_ordinal_decimal:self.ordinal.to_string(),
            content_sha256:hex::encode(self.content),
            snapshot_identity_sha256:hex::encode(self.identity),
        }).map_err(|_|"E_SOURCE_CAPTURE_RECORD_ENCODING: 원문 봉인을 저장할 수 없습니다".into())
    }
}

// A3-P10 storage sidecar, not a language surface or an input tape. Ordinal is
// decimal text so JS transport cannot truncate u64 to its 53-bit integer range.
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedSourceCaptureV1 {
    schema:String,
    epoch_identity_hex:String,
    admission_ordinal_decimal:String,
    content_sha256:String,
    snapshot_identity_sha256:String,
}

fn exact_hex(text:&str,length:usize) -> Result<Vec<u8>,String> {
    if text.len()!=length*2 || !text.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)) {
        return Err("E_SOURCE_CAPTURE_RECORD_IDENTITY: 원문 봉인 바이트 표기가 정본이 아닙니다".into());
    }
    hex::decode(text).map_err(|_|"E_SOURCE_CAPTURE_RECORD_IDENTITY: 원문 봉인 바이트를 읽을 수 없습니다".into())
}

/// Decode a recorded tuple against already supplied source bytes and its
/// expected capture identity. No admission counter access or live source I/O.
/// The complete revision/PESI owner must still bind this identity catalog;
/// a valid self-consistent record alone is not durable provenance admission.
pub(crate) fn restore_recorded_v25_capture(
    record_json:&str, raw_source:&[u8], expected_identity:&[u8;32],
) -> Result<SourceCapture,String> {
    if record_json.len()>12_000 || raw_source.len()>MAX_SOURCE_BYTES {
        return Err("E_SOURCE_CAPTURE_RECORD_LIMIT: 저장 원문 봉인 한도를 넘었습니다".into());
    }
    let strict=crate::unit_wire::parse_snapshot_json(record_json)?;
    let record=<RecordedSourceCaptureV1 as serde::Deserialize>::deserialize(&*strict)
        .map_err(|_|"E_SOURCE_CAPTURE_RECORD_FIELDS: 저장 원문 봉인 필드가 다릅니다")?;
    if record.schema!="ddn.recorded_source_capture.v1"
        || serde_json::to_string(&record).map_err(|_|"E_SOURCE_CAPTURE_RECORD_ENCODING")?!=record_json {
        return Err("E_SOURCE_CAPTURE_RECORD_CANONICAL: 저장 원문 봉인이 정본이 아닙니다".into());
    }
    let epoch=EpochSupply::for_declared_epoch(ddonirang_lang::SourceEpoch::V25);
    if exact_hex(&record.epoch_identity_hex,epoch.0.len())?!=epoch.0 {
        return Err("E_SOURCE_CAPTURE_RECORD_EPOCH: 저장 원문의 글판 정체가 다릅니다".into());
    }
    let ordinal=record.admission_ordinal_decimal.parse::<u64>()
        .map_err(|_|"E_SOURCE_CAPTURE_RECORD_ORDINAL: 원문 입장 순번이 u64가 아닙니다")?;
    if ordinal.to_string()!=record.admission_ordinal_decimal {
        return Err("E_SOURCE_CAPTURE_RECORD_ORDINAL: 원문 입장 순번이 정본이 아닙니다".into());
    }
    if raw_source.starts_with(&[0xef,0xbb,0xbf]) || std::str::from_utf8(raw_source).is_err() {
        return Err("E_SOURCE_CAPTURE_RECORD_SOURCE: 저장 원문은 BOM 없는 UTF-8이어야 합니다".into());
    }
    let content:[u8;32]=Sha256::digest(raw_source).into();
    if exact_hex(&record.content_sha256,32)?!=content {
        return Err("E_SOURCE_CAPTURE_RECORD_CONTENT: 저장 원문 바이트와 봉인이 다릅니다".into());
    }
    let identity=snapshot_digest(&epoch.0,ordinal,&content);
    if exact_hex(&record.snapshot_identity_sha256,32)?!=identity || &identity!=expected_identity {
        return Err("E_SOURCE_CAPTURE_RECORD_IDENTITY: 저장 원문 정체가 다릅니다".into());
    }
    let source=SourceCapture {epoch:epoch.0.clone(),ordinal,bytes:raw_source.to_vec(),content,identity};
    replay(&epoch,&source).map_err(|_|"E_SOURCE_CAPTURE_RECORD_CUSTODY: 저장 원문 봉인을 검증할 수 없습니다")?;
    Ok(source)
}

// A3-P7 product-owned byte framing: domain/NUL, version U16BE, epoch
// BYTES32, ordinal U64BE, raw SHA256. Not a new public record format.
fn snapshot_digest(epoch: &[u8], ordinal: u64, content: &[u8;32]) -> [u8;32] {
    let mut hash=Sha256::new(); hash.update(DOMAIN); hash.update(1u16.to_be_bytes());
    hash.update((epoch.len() as u32).to_be_bytes()); hash.update(epoch);
    hash.update(ordinal.to_be_bytes()); hash.update(content); hash.finalize().into()
}

#[derive(Default)]
struct AdmissionOwner { next: BTreeMap<Vec<u8>,Option<u64>> }
impl AdmissionOwner {
    /// Reopening verified saved captures must not let later fresh admissions
    /// reuse their ordinals. This restores a high-water mark; it does not admit
    /// the replayed source again or consume a new ordinal.
    fn retain_verified_replay(&mut self, captures: &[&SourceCapture]) -> Result<(),AdmissionError> {
        // Preflight the whole closure before touching any allocator state.
        for source in captures {
            replay(&EpochSupply::from_owner_bytes(&source.epoch)?,source)?;
        }
        for source in captures {
            let retained=source.ordinal.checked_add(1);
            let previous=self.next.get(&source.epoch).copied().unwrap_or(Some(0));
            let next=match (previous,retained) {
                (Some(a),Some(b))=>Some(a.max(b)),
                _=>None, // Exhaustion is permanent, never wrap or resurrect.
            };
            self.next.insert(source.epoch.clone(),next);
        }
        Ok(())
    }
    fn capture(&mut self, epoch: &EpochSupply, bytes: &[u8]) -> Result<SourceCapture,AdmissionError> {
        // Invalid source must not consume the epoch's ordinal.
        if bytes.len()>MAX_SOURCE_BYTES { return Err(AdmissionError::Limit); }
        if bytes.starts_with(&[0xef,0xbb,0xbf]) { return Err(AdmissionError::Bom); }
        std::str::from_utf8(bytes).map_err(|_|AdmissionError::Utf8)?;
        let ordinal=self.next.get(&epoch.0).copied().unwrap_or(Some(0)).ok_or(AdmissionError::Exhausted)?;
        let content=Sha256::digest(bytes).into();
        let identity=snapshot_digest(&epoch.0,ordinal,&content);
        let capture=SourceCapture {epoch:epoch.0.clone(),ordinal,bytes:bytes.to_vec(),content,identity};
        self.next.insert(epoch.0.clone(),ordinal.checked_add(1));
        Ok(capture)
    }
}

static SHARED_OWNER: OnceLock<Mutex<AdmissionOwner>> = OnceLock::new();
pub(crate) fn capture(epoch: &EpochSupply, bytes: &[u8]) -> Result<SourceCapture,AdmissionError> {
    SHARED_OWNER.get_or_init(||Mutex::new(AdmissionOwner::default()))
        .lock().map_err(|_|AdmissionError::Poisoned)?.capture(epoch,bytes)
}

/// Only a fully restored project candidate may retain its recorded admission
/// floor. Parsing an untrusted capture catalog must never call this function.
/// This covers verified reopen within this producer; it is not a durable
/// cross-producer ordinal service or an RD-669 commit receipt.
pub(crate) fn retain_verified_replay(captures:&[&SourceCapture]) -> Result<(),AdmissionError> {
    SHARED_OWNER.get_or_init(||Mutex::new(AdmissionOwner::default()))
        .lock().map_err(|_|AdmissionError::Poisoned)?.retain_verified_replay(captures)
}

/// Replay consumes an existing sealed capture. No allocate-next, live source
/// read or caller-specified ordinal. Durable decoding requires its separate
/// verified provenance owner; untrusted JSON cannot construct this type.
pub(crate) fn replay<'a>(epoch: &EpochSupply, source: &'a SourceCapture) -> Result<&'a SourceCapture,AdmissionError> {
    if epoch.0 != source.epoch || source.content != <[u8;32]>::from(Sha256::digest(&source.bytes))
        || source.identity != snapshot_digest(&source.epoch,source.ordinal,&source.content) {
        return Err(AdmissionError::ForeignReplay);
    }
    Ok(source)
}

pub(crate) fn capture_declared_v25(bytes: &[u8]) -> Result<SourceCapture,String> {
    capture(&EpochSupply::for_declared_epoch(ddonirang_lang::SourceEpoch::V25),bytes)
        .map_err(|error|match error {
            AdmissionError::Bom => "E_SOURCE_UTF8_BOM_FORBIDDEN: 원문은 BOM 없는 UTF-8이어야 합니다".to_string(),
            AdmissionError::Utf8 => "E_SOURCE_UTF8_INVALID: 원문은 올바른 UTF-8이어야 합니다".to_string(),
            AdmissionError::Limit => "E_SOURCE_SNAPSHOT_LIMIT: 원문 봉인 크기 한도를 넘었습니다".to_string(),
            AdmissionError::Exhausted => "E_SOURCE_ADMISSION_ORDINAL_EXHAUSTED: 원문 입장 순번을 더 만들 수 없습니다".to_string(),
            _ => "E_SOURCE_ADMISSION_CUSTODY: 원문 봉인을 확인할 수 없습니다".to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rd643_reopened_capture_advances_fresh_admission_without_readmitting_replay() {
        let epoch=EpochSupply::for_declared_epoch(ddonirang_lang::SourceEpoch::V25);
        let mut previous=AdmissionOwner::default();
        previous.next.insert(epoch.0.clone(),Some(41));
        let sealed=previous.capture(&epoch,b"value <- 1.").unwrap();
        let mut fresh=AdmissionOwner::default();
        fresh.retain_verified_replay(&[&sealed]).unwrap();
        assert_eq!(fresh.next.get(&epoch.0),Some(&Some(42)));
        // Repeated replay is idempotent, and replay bytes/identity are retained.
        fresh.retain_verified_replay(&[&sealed]).unwrap();
        assert_eq!(fresh.next.get(&epoch.0),Some(&Some(42)));
        let newly_admitted=fresh.capture(&epoch,sealed.bytes()).unwrap();
        assert_eq!(newly_admitted.ordinal(),42);
        assert_ne!(newly_admitted.identity(),sealed.identity());
    }
    #[test]
    fn rd643_reopen_high_water_is_atomic_and_exhaustion_cannot_be_reset() {
        let epoch=EpochSupply::for_declared_epoch(ddonirang_lang::SourceEpoch::V25);
        let mut previous=AdmissionOwner::default();
        previous.next.insert(epoch.0.clone(),Some(u64::MAX));
        let sealed=previous.capture(&epoch,b"value <- 1.").unwrap();
        let mut damaged=sealed.clone();damaged.bytes.push(b' ');
        let mut fresh=AdmissionOwner::default();
        let before=fresh.next.clone();
        assert_eq!(fresh.retain_verified_replay(&[&sealed,&damaged]),Err(AdmissionError::ForeignReplay));
        assert_eq!(fresh.next,before);
        fresh.retain_verified_replay(&[&sealed]).unwrap();
        assert_eq!(fresh.capture(&epoch,b"value <- 2.").unwrap_err(),AdmissionError::Exhausted);
        let mut early=AdmissionOwner::default();
        let early=early.capture(&epoch,b"value <- 3.").unwrap();
        fresh.retain_verified_replay(&[&early]).unwrap();
        assert_eq!(fresh.capture(&epoch,b"value <- 4.").unwrap_err(),AdmissionError::Exhausted);
    }
    #[test]
    fn rd643_recorded_capture_preserves_full_u64_without_readmission() {
        // A3-P10: storage transport must not pass ordinals through JS numbers.
        let epoch=EpochSupply::for_declared_epoch(ddonirang_lang::SourceEpoch::V25);
        let mut owner=AdmissionOwner::default();
        owner.next.insert(epoch.0.clone(),Some(u64::MAX));
        let source=owner.capture(&epoch,"값 <- 1.\r\n".as_bytes()).unwrap();
        let before=owner.next.clone();
        let record=source.recorded_json().unwrap();
        assert!(record.contains("\"18446744073709551615\""));
        let restored=restore_recorded_v25_capture(&record,source.bytes(),source.identity()).unwrap();
        assert_eq!(restored.ordinal(),u64::MAX);
        assert_eq!(restored.identity(),source.identity());
        assert_eq!(restored.bytes(),source.bytes());
        assert_eq!(restored.recorded_json().unwrap(),record);
        assert_eq!(owner.next,before);
    }
    #[test]
    fn rd643_recorded_capture_rejects_changed_bytes_and_noncanonical_records() {
        let epoch=EpochSupply::for_declared_epoch(ddonirang_lang::SourceEpoch::V25);
        let mut owner=AdmissionOwner::default();
        let source=owner.capture(&epoch,b"value <- 1.").unwrap();
        let record=source.recorded_json().unwrap();
        let before=owner.next.clone();
        for bad in [
            format!(" {record}"),
            record.replace("\"admission_ordinal_decimal\":\"0\"","\"admission_ordinal_decimal\":\"00\""),
            record.replacen("{","{\"schema\":\"duplicate\",",1),
            record.replacen("{","{\"unknown\":true,",1),
            record.replace(&hex::encode(source.content),&"00".repeat(32)),
        ] {
            assert!(restore_recorded_v25_capture(&bad,source.bytes(),source.identity()).is_err());
        }
        for bad_source in [b"value <- 2.".as_slice(),&[0xff],&[0xef,0xbb,0xbf]] {
            assert!(restore_recorded_v25_capture(&record,bad_source,source.identity()).is_err());
        }
        assert!(restore_recorded_v25_capture(&record,source.bytes(),&[0;32]).is_err());
        assert_eq!(owner.next,before);
    }
    #[test]
    fn rd643_real_v25_program_keeps_raw_capture_separate_from_canonical_source() {
        let raw="값 <- 1.\n(값) 보여주기.\n";
        let first=crate::ddn_runtime::DdnProgram::from_v25_source(raw,"capture-a.ddn").unwrap();
        let cloned=first.clone();
        let second=crate::ddn_runtime::DdnProgram::from_v25_source(raw,"capture-b.ddn").unwrap();
        let a=first.sealed_source_capture().unwrap(); let b=second.sealed_source_capture().unwrap();
        assert_eq!(a.bytes(),raw.as_bytes());
        assert_eq!(cloned.sealed_source_capture().unwrap().identity(),a.identity());
        assert_ne!(a.identity(),b.identity());
        assert_eq!(first.canonical_ddn(),second.canonical_ddn());
        replay(&EpochSupply::for_declared_epoch(ddonirang_lang::SourceEpoch::V25),a).unwrap();
        let restored=crate::ddn_runtime::DdnProgram::from_sealed_v25_source_with_mode(
            a.clone(),"saved-entry.ddn",ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target(),
        ).unwrap();
        assert_eq!(restored.sealed_source_capture().unwrap().identity(),a.identity());
        assert_eq!(restored.sealed_source_capture().unwrap().ordinal(),a.ordinal());
        assert_eq!(restored.sealed_source_capture().unwrap().bytes(),raw.as_bytes());
        assert_eq!(restored.canonical_ddn(),first.canonical_ddn());
        assert!(crate::ddn_runtime::DdnProgram::from_v25_source("\u{feff}값 <- 1.","bom.ddn").err().unwrap().contains("E_SOURCE_UTF8_BOM_FORBIDDEN"));
    }
    #[test]
    fn rd643_saved_program_rejects_foreign_epoch_and_historical_mode() {
        let v25=EpochSupply::for_declared_epoch(ddonirang_lang::SourceEpoch::V25);
        let foreign=EpochSupply::for_declared_epoch(ddonirang_lang::SourceEpoch::PreV25);
        let mut owner=AdmissionOwner::default();
        let raw="값 <- 1.".as_bytes();
        let rejected=crate::ddn_runtime::DdnProgram::from_sealed_v25_source_with_mode(
            owner.capture(&foreign,raw).unwrap(),"saved.ddn",ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target(),
        ).err().unwrap();
        assert!(rejected.starts_with("E_SOURCE_ADMISSION_CUSTODY:"));
        let rejected=crate::ddn_runtime::DdnProgram::from_sealed_v25_source_with_mode(
            owner.capture(&v25,raw).unwrap(),"saved.ddn",ddonirang_lang::ParseMode::HistoricalMigrationEvidence,crate::ddn_runtime::default_age_target(),
        ).err().unwrap();
        assert!(rejected.starts_with("E_HISTORICAL_ASYNC_MIGRATION_EXECUTION_FORBIDDEN:"));
    }
    #[test]
    fn rd643_saved_supported_profile_keeps_execution_admission_rules() {
        use crate::ddn_runtime::{DdnProgram, SupportedSourceProfile, default_age_target};
        use ddonirang_lang::ParseMode;
        let epoch=EpochSupply::for_declared_epoch(ddonirang_lang::SourceEpoch::V25);
        let mut owner=AdmissionOwner::default();
        for raw in [
            include_str!("../../pack/v1_core_supported_ingress_w1/fixtures/direct_recursion.ddn"),
            include_str!("../../pack/v1_core_supported_ingress_w1/fixtures/mutual_recursion.ddn"),
        ] {
            let live=DdnProgram::from_supported_source_profile_with_mode(
                raw,"saved.ddn",ParseMode::Strict,SupportedSourceProfile::V1CoreV25,
            ).err().expect("live profile rejects recursion");
            let capture=owner.capture(&epoch,raw.as_bytes()).unwrap();
            let before=owner.next.clone();
            let saved=DdnProgram::from_sealed_supported_source_profile_with_mode(
                capture,"saved.ddn",ParseMode::Strict,SupportedSourceProfile::V1CoreV25,default_age_target(),
            ).err().expect("saved profile rejects recursion before candidate VM creation");
            assert_eq!(saved,live);
            assert!(saved.starts_with(crate::ddn_runtime::E_V1_CORE_RECURSION_UNSUPPORTED));
            assert_eq!(owner.next,before,"replay rejection never allocates an ordinal");
        }
        let raw="값 <- 1.";
        let capture=owner.capture(&epoch,raw.as_bytes()).unwrap();
        let saved=DdnProgram::from_sealed_supported_source_profile_with_mode(
            capture.clone(),"saved.ddn",ParseMode::Strict,SupportedSourceProfile::V1CoreV25,default_age_target(),
        ).unwrap();
        let live=DdnProgram::from_supported_source_profile_with_mode(
            raw,"saved.ddn",ParseMode::Strict,SupportedSourceProfile::V1CoreV25,
        ).unwrap();
        assert_eq!(saved.canonical_ddn(),live.canonical_ddn());
        assert_eq!(saved.sealed_source_capture().unwrap().identity(),capture.identity());
    }
    #[test]
    fn rd643_capture_is_ordinal_distinct_and_replay_does_not_allocate() {
        // A3-P7. Epoch bytes here are test supply, not a semantic owner claim.
        let epoch=EpochSupply::from_owner_bytes(b"test-epoch").unwrap();
        let mut owner=AdmissionOwner::default();
        let first=owner.capture(&epoch,"값 <- 1.".as_bytes()).unwrap();
        let second=owner.capture(&epoch,first.bytes()).unwrap();
        assert_ne!(first.identity(),second.identity()); assert_eq!(first.content,second.content);
        assert_eq!(first.text(),"값 <- 1.");
        let next=owner.next.clone(); assert_eq!(replay(&epoch,&first).unwrap().ordinal(),0);
        assert_eq!(owner.next,next);
        assert!(matches!(replay(&EpochSupply::from_owner_bytes(b"foreign").unwrap(),&first),Err(AdmissionError::ForeignReplay)));
    }
    #[test]
    fn rd643_failure_is_atomic_and_maximum_ordinal_never_wraps() {
        let epoch=EpochSupply::from_owner_bytes(b"exhaustion").unwrap();
        let mut owner=AdmissionOwner::default();
        for bytes in [&[0xef,0xbb,0xbf][..],&[0xff][..]] {
            assert!(owner.capture(&epoch,bytes).is_err()); assert!(owner.next.is_empty());
        }
        owner.next.insert(epoch.0.clone(),Some(u64::MAX));
        assert_eq!(owner.capture(&epoch,b"ok").unwrap().ordinal(),u64::MAX);
        let before=owner.next.clone();
        assert!(matches!(owner.capture(&epoch,b"ok"),Err(AdmissionError::Exhausted)));
        assert_eq!(owner.next,before);
    }
    #[test]
    fn rd643_shared_owner_interleaves_callers_in_one_epoch() {
        let epoch=EpochSupply::from_owner_bytes(b"shared-test-isolated-epoch").unwrap();
        let first=capture(&epoch,b"first").unwrap(); let second=capture(&epoch,b"second").unwrap();
        assert_eq!(second.ordinal(),first.ordinal()+1);
        replay(&epoch,&first).unwrap();
        assert_eq!(capture(&epoch,b"third").unwrap().ordinal(),second.ordinal()+1);
    }
    #[test]
    fn rd643_parallel_host_calls_share_one_ordinal_owner() {
        let threads:Vec<_>=(0..8).map(|_|std::thread::spawn(|| {
            let epoch=EpochSupply::from_owner_bytes(b"parallel-host-test-epoch").unwrap();
            capture(&epoch,b"same bytes").unwrap()
        })).collect();
        let mut captures:Vec<_>=threads.into_iter().map(|t|t.join().unwrap()).collect();
        captures.sort_by_key(SourceCapture::ordinal);
        for pair in captures.windows(2) {
            assert_eq!(pair[1].ordinal(),pair[0].ordinal()+1);
            assert_ne!(pair[1].identity(),pair[0].identity());
            assert_eq!(pair[1].content,pair[0].content);
        }
    }
}
