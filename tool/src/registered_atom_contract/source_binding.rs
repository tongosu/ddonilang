//! VC04 R4 descriptor parsing only. A parsed descriptor is NOT source,
//! registration, project, or execution admission. Those owners must bind it
//! to their recorded inputs before it can supply a runtime atom.
use super::ContractFailure;
pub(crate) mod nominal;
type Result<T> = std::result::Result<T, ContractFailure>;
const HEADER: &[u8] = b"ddn.vc04.atom-source-binding.v1\0\0\x01";

#[derive(Debug)]
pub(crate) struct Source<'a> {
    pub path: &'a str,
    pub snapshot: &'a [u8],
    pub length: u64,
    pub sha: &'a [u8],
}
#[derive(Debug)]
pub(crate) struct Entry<'a> { pub identity: &'a [u8], pub label: &'a str }
#[derive(Debug)]
pub(crate) struct Registry<'a> {
    pub identity: &'a [u8], pub origin: u8,
    pub contract_length: u64, pub contract_sha: &'a [u8], pub snapshot: &'a [u8],
    pub entries: Vec<Entry<'a>>,
}
#[derive(Debug)]
pub(crate) struct Binding {
    pub source: usize, pub start: u64, pub end: u64,
    pub registry: usize, pub entry: usize,
}
#[derive(Debug)]
pub(crate) struct Candidate<'a> {
    pub version:u16,
    pub sources: Vec<Source<'a>>, pub registries: Vec<Registry<'a>>, pub bindings: Vec<Binding>,
    pub nominal: Option<nominal::Tail<'a>>,
}

struct Reader<'a> { rest: &'a [u8] }
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if n > self.rest.len() { return Err(ContractFailure::SchemaInvalid); }
        let (value, rest) = self.rest.split_at(n); self.rest = rest; Ok(value)
    }
    fn u32(&mut self) -> Result<u32> { Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap())) }
    fn u64(&mut self) -> Result<u64> { Ok(u64::from_be_bytes(self.take(8)?.try_into().unwrap())) }
    fn blob(&mut self) -> Result<&'a [u8]> {
        let n = usize::try_from(self.u32()?).map_err(|_|ContractFailure::SchemaInvalid)?;
        if n == 0 || n > 4096 { return Err(ContractFailure::SchemaInvalid); }
        self.take(n)
    }
    fn text(&mut self) -> Result<&'a str> {
        std::str::from_utf8(self.blob()?).map_err(|_|ContractFailure::SchemaInvalid)
    }
    fn count(&mut self, limit: usize, minimum_row_bytes: usize) -> Result<usize> {
        let n = usize::try_from(self.u32()?).map_err(|_|ContractFailure::SchemaInvalid)?;
        if n > limit || n > self.rest.len() / minimum_row_bytes { return Err(ContractFailure::SchemaInvalid); }
        Ok(n)
    }
}

struct Writer(Vec<u8>);
impl Writer {
    fn raw(&mut self, bytes: &[u8]) -> Result<()> {
        let size = self.0.len().checked_add(bytes.len()).ok_or(ContractFailure::SchemaInvalid)?;
        if size > 1_000_000 { return Err(ContractFailure::SchemaInvalid); }
        self.0.try_reserve(bytes.len()).map_err(|_|ContractFailure::SchemaInvalid)?;
        self.0.extend_from_slice(bytes); Ok(())
    }
    fn count(&mut self, n: usize) -> Result<()> {
        self.raw(&u32::try_from(n).map_err(|_|ContractFailure::SchemaInvalid)?.to_be_bytes())
    }
    fn blob(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.is_empty() || bytes.len() > 4096 { return Err(ContractFailure::SchemaInvalid); }
        self.count(bytes.len())?; self.raw(bytes)
    }
    fn digest(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() != 32 { return Err(ContractFailure::SchemaInvalid); }
        self.raw(bytes)
    }
}

fn exact_label(label: &str) -> bool {
    use ddonirang_lang::{Lexer, TokenKind};
    // A leading `#` is the lexer's historical line-directive position.
    // Use an expression token position, not that unrelated line grammar.
    let spelling = format!("(#{label})");
    let Ok(tokens) = Lexer::new(&spelling).tokenize() else { return false; };
    matches!(tokens.as_slice(), [open, atom, close, eof]
        if matches!(&atom.kind, TokenKind::Atom(parsed) if parsed == label)
        && matches!(open.kind, TokenKind::LParen) && matches!(close.kind, TokenKind::RParen)
        && atom.span.start == 1 && atom.span.end == spelling.len() - 1
        && matches!(eof.kind, TokenKind::Eof))
}

impl<'a> Candidate<'a> {
    pub(crate) fn verify_resolved_occurrences(&self, saved: &crate::project_revision_envelope_supply::VerifiedSavedV25Sources) -> Result<()> {
        self.verify_sources(saved.captures())?;
        let actual = saved.atom_value_occurrences();
        if self.bindings.len() != actual.len() { return Err(ContractFailure::NotAdmitted); }
        for (binding, expected) in self.bindings.iter().zip(actual) {
            let source = &self.sources[binding.source];
            let entry = &self.registries[binding.registry].entries[binding.entry];
            if source.path != expected.path() || entry.label != expected.occurrence().label
                || binding.start != expected.occurrence().span.start as u64
                || binding.end != expected.occurrence().span.end as u64 {
                return Err(ContractFailure::NotAdmitted);
            }
        }
        Ok(())
    }
    pub(crate) fn encode(&self) -> Result<Vec<u8>> {
        if !matches!((self.version,self.nominal.is_some()),(1,false)|(2|3,true)){return Err(ContractFailure::SchemaInvalid);}
        let mut out = Writer(Vec::new()); out.raw(&HEADER[..HEADER.len()-2])?;
        out.raw(&self.version.to_be_bytes())?;
        out.count(self.sources.len())?;
        for source in &self.sources {
            out.blob(source.path.as_bytes())?; out.blob(source.snapshot)?;
            out.raw(&source.length.to_be_bytes())?; out.digest(source.sha)?;
        }
        out.count(self.registries.len())?;
        for registry in &self.registries {
            out.blob(registry.identity)?; out.raw(&[registry.origin])?;
            out.raw(&registry.contract_length.to_be_bytes())?;
            out.digest(registry.contract_sha)?; out.digest(registry.snapshot)?;
            out.count(registry.entries.len())?;
            for entry in &registry.entries { out.blob(entry.identity)?; out.blob(entry.label.as_bytes())?; }
        }
        out.count(self.bindings.len())?;
        for binding in &self.bindings {
            out.count(binding.source)?; out.raw(&binding.start.to_be_bytes())?; out.raw(&binding.end.to_be_bytes())?;
            out.count(binding.registry)?; out.count(binding.entry)?;
        }
        if let Some(nominal) = &self.nominal { nominal.encode(&mut out)?; }
        // Share the decoder's closed ordering/count/range checks. This does
        // not turn a newly encoded candidate into saved project admission.
        Candidate::parse(&out.0)?;
        Ok(out.0)
    }
    /// Compare against captures retained by the source owner, not paths or a
    /// descriptor's self-asserted hash. Does not issue execution authority.
    pub(crate) fn verify_sources(&self, captures: &[(String, crate::source_snapshot_admission::SourceCapture)]) -> Result<()> {
        if captures.len() != self.sources.len() { return Err(ContractFailure::NotAdmitted); }
        for (source, (path, capture)) in self.sources.iter().zip(captures) {
            if source.path != path || source.snapshot != capture.identity()
                || source.length != capture.bytes().len() as u64 || source.sha != capture.content_digest() {
                return Err(ContractFailure::NotAdmitted);
            }
        }
        Ok(())
    }

    /// Entries include retained-but-no-longer-used labels. Membership must
    /// match the *whole* recorded contract, not just this run's occurrences.
    pub(crate) fn verify_contracts(&self, contracts: &[super::RecordedAtomContract]) -> Result<()> {
        super::verify_contract_set(contracts)?;
        if contracts.len() != self.registries.len() { return Err(ContractFailure::NotAdmitted); }
        for registry in &self.registries {
            let contract = contracts.iter().find(|contract|contract.owner == registry.identity)
                .ok_or(ContractFailure::NotAdmitted)?;
            if registry.contract_length != contract.custody.byte_length
                || registry.contract_sha != contract.custody.sha256
                || registry.snapshot != contract.custody.snapshot_identity
                || registry.entries.len() != contract.atoms.len()
                || registry.entries.iter().zip(&contract.atoms).any(|(entry,atom)|entry.identity != atom) {
                return Err(ContractFailure::NotAdmitted);
            }
        }
        Ok(())
    }

    pub(crate) fn parse(raw: &'a [u8]) -> Result<Self> {
        if raw.len() > 1_000_000 { return Err(ContractFailure::SchemaInvalid); }
        let mut input = Reader {rest: raw};
        if input.take(HEADER.len()-2)? != &HEADER[..HEADER.len()-2] { return Err(ContractFailure::SchemaInvalid); }
        let version=u16::from_be_bytes(input.take(2)?.try_into().unwrap());
        if !matches!(version,1|2|3) { return Err(ContractFailure::SchemaInvalid); }
        let n = input.count(64, 50)?;
        let mut sources: Vec<Source<'a>> = Vec::new();
        sources.try_reserve_exact(n).map_err(|_|ContractFailure::SchemaInvalid)?;
        for _ in 0..n {
            let path = input.text()?;
            if sources.last().is_some_and(|prior| prior.path.as_bytes() >= path.as_bytes()) {
                return Err(ContractFailure::SchemaInvalid);
            }
            // This checks canonical logical path shape, not filesystem custody.
            if path.starts_with('/') || path.contains('\\') || path.contains(':')
                || path.split('/').any(|part| matches!(part, ""|"."|"..")) {
                return Err(ContractFailure::SchemaInvalid);
            }
            sources.push(Source {path, snapshot:input.blob()?, length:input.u64()?, sha:input.take(32)?});
        }
        let n = input.count(256, 82)?;
        let mut registries: Vec<Registry<'a>> = Vec::new();
        registries.try_reserve_exact(n).map_err(|_|ContractFailure::SchemaInvalid)?;
        let mut total_entries = 0usize;
        for _ in 0..n {
            let identity = input.blob()?;
            let origin = input.take(1)?[0];
            if !matches!(origin, 1|2) || origin == 1 && identity.len() != 32
                || registries.last().is_some_and(|prior|prior.identity >= identity) {
                return Err(ContractFailure::SchemaInvalid);
            }
            let contract_length = input.u64()?;
            if contract_length > 1_000_000 { return Err(ContractFailure::SchemaInvalid); }
            let contract_sha = input.take(32)?; let snapshot = input.take(32)?;
            let count = input.count(65536 - total_entries, 10)?;
            total_entries += count;
            let mut entries: Vec<Entry<'a>> = Vec::new();
            entries.try_reserve_exact(count).map_err(|_|ContractFailure::SchemaInvalid)?;
            let mut labels = std::collections::BTreeSet::new();
            for _ in 0..count {
                let atom = input.blob()?; let label = input.text()?;
                if origin == 1 && atom.len() != 32
                    || entries.last().is_some_and(|prior|prior.identity >= atom)
                    || !exact_label(label) || !labels.insert(label) { return Err(ContractFailure::SchemaInvalid); }
                entries.push(Entry {identity:atom, label});
            }
            registries.push(Registry {identity, origin, contract_length, contract_sha, snapshot, entries});
        }
        let n = input.count(usize::MAX, 28)?;
        let mut bindings: Vec<Binding> = Vec::new();
        bindings.try_reserve_exact(n).map_err(|_|ContractFailure::SchemaInvalid)?;
        for _ in 0..n {
            let source = input.u32()? as usize;
            let start = input.u64()?; let end = input.u64()?;
            let registry = input.u32()? as usize; let entry = input.u32()? as usize;
            let selected_source = sources.get(source).ok_or(ContractFailure::SchemaInvalid)?;
            let selected_registry = registries.get(registry).ok_or(ContractFailure::SchemaInvalid)?;
            if entry >= selected_registry.entries.len() || start >= end || end > selected_source.length
                || bindings.last().is_some_and(|prior|
                    (prior.source,prior.start,prior.end) >= (source,start,end)
                    || prior.source == source && prior.end > start) {
                return Err(ContractFailure::SchemaInvalid);
            }
            bindings.push(Binding {source,start,end,registry,entry});
        }
        let nominal=if version>=2 {Some(nominal::Tail::parse(&mut input,&sources,version)?)}else{None};
        if !input.rest.is_empty() { return Err(ContractFailure::SchemaInvalid); }
        Ok(Self {version,sources,registries,bindings,nominal})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const VECTOR: &str = "64646e2e766330342e61746f6d2d736f757263652d62696e64696e672e763100000100000001000000086d61696e2e64646e000000202ad676d6b8245049a6461309a5bd057f5a5678b3ad927c6770ba899f07a9ed6b0000000000000053f39f96ff060c64315e3c43399d7e78520cd5d3189736257b5c53861ab1f506240000000100000020000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f01000000000000011c31d6b824d72493ff3a662eee89c64aebcb7805a4540a6ae0c68fbd1836c780dd64b4c969b23dbd6ca5d09c1cb9d93a2ca6dbaed9dd42f62b04df035a9e24523b0000000100000020202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f00000006ebb9a8eab09500000002000000000000000000000027000000000000002e000000000000000000000000000000000000003d00000000000000440000000000000000";
    #[test]
    fn vc04_source_binding_exact_r4_vector_is_only_a_candidate() {
        use sha2::{Digest,Sha256};
        let raw = hex::decode(VECTOR).unwrap();
        assert_eq!(hex::encode(Sha256::digest(&raw)), "3fd371ad0c2e7cf101ca3e0e9c1187762f51a160e9c437b9a17846654d462470");
        let decoded = Candidate::parse(&raw).unwrap();
        assert_eq!(decoded.encode().unwrap(), raw);
        assert_eq!(decoded.sources[0].path, "main.ddn");
        assert_eq!(decoded.registries[0].entries[0].label, "빨강");
        assert_eq!(decoded.bindings.len(), 2);
        let (_, contract) = super::super::RecordedAtomContract::record(
            decoded.registries[0].identity, &[decoded.registries[0].entries[0].identity.to_vec()]).unwrap();
        decoded.verify_contracts(&[contract.clone()]).unwrap();
        assert!(decoded.verify_contracts(&[]).is_err());
        let (_, extended) = super::super::RecordedAtomContract::record(
            decoded.registries[0].identity, &[decoded.registries[0].entries[0].identity.to_vec(), vec![255;32]]).unwrap();
        assert!(decoded.verify_contracts(&[extended]).is_err());
        assert!(decoded.verify_sources(&[]).is_err());
        assert_eq!((decoded.bindings[1].start,decoded.bindings[1].end), (61,68));
        for n in 0..raw.len() { assert!(Candidate::parse(&raw[..n]).is_err(), "prefix {n}"); }
        let mut trailer = raw.clone(); trailer.push(0); assert!(Candidate::parse(&trailer).is_err());
        let mut huge_count = raw; huge_count[HEADER.len()..HEADER.len()+4].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(Candidate::parse(&huge_count).is_err());
    }

    #[test]
    fn vc04_source_binding_checks_captured_identity_not_just_source_content() {
        let source = crate::source_snapshot_admission::capture_declared_v25("1 보여주기.".as_bytes()).unwrap();
        let candidate = Candidate { version:1,sources: vec![Source {path:"main.ddn", snapshot:source.identity(),
            length:source.bytes().len() as u64, sha:source.content_digest()}], registries:vec![], bindings:vec![], nominal:None };
        let raw = candidate.encode().unwrap();
        let decoded = Candidate::parse(&raw).unwrap();
        decoded.verify_sources(&[("main.ddn".into(),source.clone())]).unwrap();
        let recaptured = crate::source_snapshot_admission::capture_declared_v25(source.bytes()).unwrap();
        assert_eq!(source.content_digest(), recaptured.content_digest());
        assert_ne!(source.identity(), recaptured.identity());
        assert!(decoded.verify_sources(&[("main.ddn".into(),recaptured)]).is_err());
        assert!(decoded.verify_sources(&[("other.ddn".into(),source)]).is_err());
    }

    #[test]
    fn vc04_source_binding_rejects_overlaps_foreign_rows_and_unbounded_counts() {
        let raw = hex::decode(VECTOR).unwrap();
        let n = raw.len();
        let mut overlap = raw.clone();
        let first = overlap[n-56..n-28].to_vec();
        overlap[n-28..].copy_from_slice(&first);
        assert!(Candidate::parse(&overlap).is_err());
        let mut foreign = raw.clone(); foreign[n-28..n-24].copy_from_slice(&1u32.to_be_bytes());
        assert!(Candidate::parse(&foreign).is_err());
        let mut count = raw; count[n-60..n-56].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(Candidate::parse(&count).is_err());
        assert!(Candidate::parse(&vec![0;1_000_001]).is_err());
    }

    #[test]
    fn vc04_source_binding_covers_saved_linker_occurrences_and_reopened_captures() {
        use crate::project_revision_envelope_supply as supply;
        let root = "(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n";
        let entry = "실행:움직씨 := { 색 <- #빨강. 복사 <- #빨강. \"#가짜\" 보여주기. }.\n";
        let blobs = [supply::SavedSourceBlobV1 {path:"root.ddn",role:"root",bytes:root.as_bytes()},
            supply::SavedSourceBlobV1 {path:"main.ddn",role:"ddn_source",bytes:entry.as_bytes()}];
        let definition = supply::CanonicalIdentityBytesV1::from_owner_bytes(&[4;32]).unwrap();
        let fresh = || supply::collect_saved_source_supply("root.ddn", "main.ddn", &definition, &blobs);
        // Use the same existing owner used for sealed project execution;
        // opaque test IDs below are fixtures, never an issuance implementation.
        let saved = supply::seal_saved_v25_sources(fresh().unwrap()).unwrap();
        let owner = [1;32]; let atom = [2;32];
        let (_, contract) = super::super::RecordedAtomContract::record(&owner, &[atom.to_vec()]).unwrap();
        let sources = saved.captures().iter().map(|(path,capture)|Source {path,snapshot:capture.identity(),
            length:capture.bytes().len() as u64,sha:capture.content_digest()}).collect();
        let registries = vec![Registry {identity:&owner,origin:1,contract_length:contract.custody.byte_length,
            contract_sha:&contract.custody.sha256,snapshot:&contract.custody.snapshot_identity,
            entries:vec![Entry {identity:&atom,label:"빨강"}]}];
        let bindings = saved.atom_value_occurrences().iter().map(|occurrence|Binding {
            source:saved.captures().iter().position(|(path,_)|path == occurrence.path()).unwrap(),
            start:occurrence.occurrence().span.start as u64,end:occurrence.occurrence().span.end as u64,
            registry:0,entry:0 }).collect();
        let mut candidate = Candidate {version:1,sources,registries,bindings,nominal:None};
        assert_eq!(candidate.bindings.len(),2);
        let raw = candidate.encode().unwrap();
        let parsed = Candidate::parse(&raw).unwrap();
        parsed.verify_resolved_occurrences(&saved).unwrap();
        parsed.verify_contracts(&[contract.clone()]).unwrap();
        let catalog = saved.recorded_capture_catalog_json().unwrap();
        let reopened = supply::restore_saved_v25_sources(fresh().unwrap(), &catalog).unwrap();
        parsed.verify_resolved_occurrences(&reopened).unwrap();
        candidate.bindings.pop();
        assert!(candidate.verify_resolved_occurrences(&saved).is_err());
        candidate.registries[0].entries[0].label = "파랑";
        assert!(candidate.verify_resolved_occurrences(&saved).is_err());
    }
}
