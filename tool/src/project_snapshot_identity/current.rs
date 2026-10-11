//! RD-689 exact forward binary capsule. Never construct trusted inputs from
//! adapter JSON, projection hex or the untrusted envelope being checked.
use super::{Identity32, SnapshotIdentityError as Error, validate_logical_path};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const PROFILES: [&str;4] = ["ddn.source_logical_tree_content_identity.v1","ddn.project_semantic_configuration_identity.v1","ddn.verified_dependency_resolution_identity.v1","ddn.declared_resource_content_closure_identity.v1"];
const MAX_PAYLOAD: usize = 2_000_000;

// Private fields/constructors restrict issuance to the shared owner. Current
// source/configuration/resolution/resource admission will supply these types;
// a structurally parsed request is intentionally unable to do so.
#[derive(Clone, Copy)]
pub(super) struct Source<'a> { path: &'a str, identity: &'a [u8] }
#[derive(Clone, Copy)]
pub(super) struct Requirement<'a> { name: &'a str, identity: &'a [u8] }
#[derive(Clone, Copy)]
pub(super) struct Dependency<'a> { kind: u8, identity: &'a [u8], digest: [u8;32] }
#[derive(Clone, Copy)]
pub(super) struct Edge<'a> { from: &'a [u8], to: &'a [u8] }
#[derive(Clone, Copy)]
pub(super) struct Resource<'a> { path: &'a str, identity: [u8;32], byte_length: u64, digest: [u8;32] }
#[derive(Clone, Copy)]
pub(super) struct AdmittedProjectSnapshotIdentityInput<'a> {
    sources: &'a [Source<'a>], epoch: &'a [u8], entry: &'a str, definition: &'a [u8], target_profile: &'a [u8],
    requirements: &'a [Requirement<'a>], resource_paths: &'a [&'a str],
    dependencies: &'a [Dependency<'a>], edges: &'a [Edge<'a>], resources: &'a [Resource<'a>],
}
pub(super) struct ExpectedProjectSnapshotIdentity<'a> { constituents: &'a [Identity32;4], pesi: Identity32 }
pub(super) struct IssuedProjectSnapshotIdentity {
    pub payloads: [Vec<u8>;4], pub preimages: [Vec<u8>;4], pub constituents: [Identity32;4], pub pesi_preimage: Vec<u8>, pub pesi: Identity32,
}
pub(super) struct VerifiedProjectSnapshotIdentity(IssuedProjectSnapshotIdentity);

/// Forward identity from the saved-source/compiler/authored owners, not the
/// untrusted revision under comparison. Runtime admission remains separate.
pub(crate) struct IssuedSavedLocalProjectIdentity(IssuedProjectSnapshotIdentity);
impl IssuedSavedLocalProjectIdentity {
    pub(crate) fn pesi(&self) -> Identity32 { self.0.pesi }
    pub(crate) fn constituents(&self) -> &[Identity32;4] { &self.0.constituents }
}

pub(crate) fn issue_saved_local_project(
    owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
) -> Result<IssuedSavedLocalProjectIdentity,String> {
    let saved=owned.sources();
    // Current D9 linker rejects unbound execution contracts/views/exports.
    // Its local module closure is source-owned, not an external dependency
    // registry. Never turn an unverified package/resource supply into zero.
    let graph=crate::local_project::build_project_graph(saved.saved().source().clone())
        .map_err(|e|format!("{}: {}",e.code,e.message))?;
    if !graph.gaji_aliases.is_empty() || saved.saved().manifest().lock_sha256.is_some()
        || saved.saved().source().files.iter().any(|file|!matches!(file.role.as_str(),"root"|"ddn_source")) {
        return Err("E_SAVED_PROJECT_EXTERNAL_SUPPLY_REQUIRED: 외부 가지·자료의 검증된 공급이 필요합니다".into());
    }
    let rows:Vec<_>=saved.captures().iter().map(|(path,capture)|Source {
        path:path.as_str(),identity:capture.identity().as_slice(),
    }).collect();
    let epoch=saved.entry_capture().epoch_owner_bytes();
    if saved.captures().iter().any(|(_,capture)|capture.epoch_owner_bytes()!=epoch) {
        return Err("E_SAVED_PROJECT_SOURCE_EPOCH: 저장 원문의 글판 정체가 다릅니다".into());
    }
    // A3-P9: physical projection of the actual admitted effective profile.
    // No fallback, inferred input mode or display-string hashing in adapters.
    let target:&[u8]=match owned.project().source_profile.as_str() {
        "v1-core-v25" => b"ExecutionProfileIdentity:v1-core-v25",
        "v1-core-v25-logical-alpha2" => b"ExecutionProfileIdentity:v1-core-v25-logical-alpha2",
        _ => return Err("E_SAVED_PROJECT_TARGET_PROFILE: 지원 실행 프로필이 아닙니다".into()),
    };
    let issued=issue_constituents_and_pesi(AdmittedProjectSnapshotIdentityInput {
        sources:&rows,epoch,entry:&saved.saved().manifest().entry_path,
        definition:owned.definition_identity().as_bytes(),target_profile:target,
        requirements:&[],resource_paths:&[],dependencies:&[],edges:&[],resources:&[],
    }).map_err(|e|format!("E_SAVED_PROJECT_EXECUTION_IDENTITY: {e:?}"))?;
    Ok(IssuedSavedLocalProjectIdentity(issued))
}

fn fail(name: &'static str) -> Error { Error::ConstituentMalformed(name) }
fn count(n: usize, limit: usize) -> Result<[u8;4],Error> {
    if n > limit { return Err(fail("count_limit")); } Ok(u32::try_from(n).map_err(|_| fail("count"))?.to_be_bytes())
}
fn append(out: &mut Vec<u8>, raw: &[u8]) -> Result<(),Error> {
    let total = out.len().checked_add(raw.len()).ok_or(fail("payload_length"))?;
    if total > MAX_PAYLOAD { return Err(fail("payload_limit")); }
    out.try_reserve_exact(raw.len()).map_err(|_| fail("allocation"))?; out.extend_from_slice(raw); Ok(())
}
fn blob(out: &mut Vec<u8>, raw: &[u8]) -> Result<(),Error> {
    append(out,&u32::try_from(raw.len()).map_err(|_| fail("field_length"))?.to_be_bytes())?; append(out,raw)
}
fn field(out: &mut Vec<u8>, tag: u8, raw: &[u8]) -> Result<(),Error> { append(out,&[tag])?; blob(out,raw) }
fn identity(raw: &[u8]) -> Result<(),Error> { if raw.is_empty() || raw.len()>4096 { Err(fail("owner_identity")) } else { Ok(()) } }
fn path(text: &str) -> Result<(),Error> {
    if text.len()>4096 { return Err(fail("path_limit")); } validate_logical_path(text)
}
fn increasing(previous: Option<&[u8]>, key: &[u8]) -> Result<(),Error> {
    if previous.is_some_and(|p| p >= key) { Err(fail("duplicate_or_order")) } else { Ok(()) }
}
fn preimage(profile: &str, payload: &[u8]) -> Result<Vec<u8>,Error> {
    if payload.len()>MAX_PAYLOAD { return Err(fail("payload_limit")); }
    let total=profile.len().checked_add(7).and_then(|n|n.checked_add(payload.len())).ok_or(fail("preimage_length"))?;
    let mut out=Vec::new(); out.try_reserve_exact(total).map_err(|_| fail("allocation"))?;
    out.extend_from_slice(profile.as_bytes()); out.extend_from_slice(&[0,0,1]); out.extend_from_slice(&(payload.len() as u32).to_be_bytes()); out.extend_from_slice(payload); Ok(out)
}
fn preflight(base: usize, sizes: impl Iterator<Item=Option<usize>>) -> Result<(),Error> {
    let mut total=base;
    for size in sizes { total=total.checked_add(size.ok_or(fail("payload_length"))?).ok_or(fail("payload_length"))?;
        if total>MAX_PAYLOAD { return Err(fail("payload_limit")); }
    } Ok(())
}
fn source_payload(rows: &[Source<'_>]) -> Result<Vec<u8>,Error> {
    count(rows.len(),64)?;
    preflight(10,rows.iter().map(|r|r.path.len().checked_add(r.identity.len()).and_then(|n|n.checked_add(8))))?;
    let mut previous=None; let mut folded=BTreeSet::new();
    let mut set=count(rows.len(),64)?.to_vec();
    for row in rows {
        path(row.path)?; identity(row.identity)?; increasing(previous,row.path.as_bytes())?;
        previous=Some(row.path.as_bytes());
        if !folded.insert(row.path.to_ascii_lowercase()) { return Err(fail("source_path_collision")); }
        blob(&mut set,row.path.as_bytes())?; blob(&mut set,row.identity)?;
    }
    let mut payload=vec![1]; field(&mut payload,1,&set)?; Ok(payload)
}

/// Current RD-689 source constituent, supplied by the actual saved-source
/// owner. Adapter hex/JSON is not a source identity constructor.
pub(crate) fn issue_saved_source_tree(
    saved: &crate::project_revision_envelope_supply::VerifiedSavedV25Sources,
) -> Result<super::IssuedIdentity,Error> {
    let rows:Vec<_>=saved.captures().iter().map(|(path,capture)|Source {
        path:path.as_str(),identity:capture.identity().as_slice(),
    }).collect();
    let canonical_payload=source_payload(&rows)?;
    let envelope=preimage(PROFILES[0],&canonical_payload)?;
    let identity=Identity32::from_raw(Sha256::digest(&envelope).into());
    Ok(super::IssuedIdentity {canonical_payload,envelope,identity})
}
pub(super) fn issue_constituents_and_pesi(input: AdmittedProjectSnapshotIdentityInput<'_>) -> Result<IssuedProjectSnapshotIdentity,Error> {
    // Cardinalities and all individual raw lengths precede payload allocation.
    count(input.sources.len(),64)?; count(input.requirements.len(),65536)?; count(input.resource_paths.len(),65536)?;
    count(input.dependencies.len(),65536)?; count(input.edges.len(),262144)?; count(input.resources.len(),65536)?;
    preflight(10,input.sources.iter().map(|r|r.path.len().checked_add(r.identity.len()).and_then(|n|n.checked_add(8))))?;
    preflight(39,[input.epoch.len(),input.entry.len(),input.definition.len(),input.target_profile.len()].into_iter().map(Some)
        .chain(input.requirements.iter().map(|r|r.name.len().checked_add(r.identity.len()).and_then(|n|n.checked_add(8))))
        .chain(input.resource_paths.iter().map(|p|p.len().checked_add(4))))?;
    preflight(19,input.dependencies.iter().map(|r|r.identity.len().checked_add(37))
        .chain(input.edges.iter().map(|r|r.from.len().checked_add(r.to.len()).and_then(|n|n.checked_add(8)))))?;
    preflight(10,input.resources.iter().map(|r|r.path.len().checked_add(76)))?;
    for raw in [input.epoch,input.definition,input.target_profile] { identity(raw)?; } path(input.entry)?;
    let mut previous = None; let mut folded = BTreeSet::new();
    for row in input.sources { path(row.path)?; identity(row.identity)?; increasing(previous,row.path.as_bytes())?; previous=Some(row.path.as_bytes());
        if !folded.insert(row.path.to_ascii_lowercase()) { return Err(fail("source_path_collision")); }
    }
    let mut prior_req: Option<(&str,&[u8])> = None;
    for row in input.requirements {
        if row.name.is_empty() || row.name.len()>4096 || row.name.contains('\0') || !row.name.chars().eq(unicode_normalization::UnicodeNormalization::nfc(row.name)) { return Err(fail("requirement_name")); }
        identity(row.identity)?;
        if prior_req.is_some_and(|p| p >= (row.name,row.identity)) { return Err(fail("requirement_order")); } prior_req=Some((row.name,row.identity));
    }
    previous=None;
    for row in input.resource_paths { path(row)?; increasing(previous,row.as_bytes())?; previous=Some(row.as_bytes()); }
    let mut prior_node: Option<(u8,&[u8])> = None; let mut node_ids=BTreeSet::new();
    for row in input.dependencies { identity(row.identity)?;
        if !matches!(row.kind,1|2) || prior_node.is_some_and(|p| p >= (row.kind,row.identity)) || !node_ids.insert(row.identity) { return Err(fail("dependency_node")); }
        prior_node=Some((row.kind,row.identity));
    }
    let mut prior_edge=None;
    for row in input.edges {
        identity(row.from)?; identity(row.to)?;
        if !node_ids.contains(row.from) || !node_ids.contains(row.to) || row.from==row.to || prior_edge.is_some_and(|p| p >= (row.from,row.to)) { return Err(fail("dependency_edge")); }
        prior_edge=Some((row.from,row.to));
    }
    // Verified graphs must still be acyclic; a supplied node/edge collection is
    // not an admission proof merely because its ordering is canonical.
    let mut indegrees: BTreeMap<&[u8],usize> = node_ids.iter().copied().map(|node|(node,0)).collect();
    let mut children: BTreeMap<&[u8],Vec<&[u8]>> = BTreeMap::new();
    for edge in input.edges { *indegrees.get_mut(edge.to).ok_or(fail("dependency_endpoint"))? += 1; children.entry(edge.from).or_default().push(edge.to); }
    let mut ready:Vec<&[u8]>=indegrees.iter().filter_map(|(node,n)|(*n==0).then_some(*node)).collect();
    let mut visited=0;
    while let Some(node)=ready.pop() { visited+=1;
        if let Some(next)=children.get(node) { for child in next { let n=indegrees.get_mut(child).ok_or(fail("dependency_endpoint"))?; *n-=1; if *n==0 { ready.push(child); } } }
    }
    if visited!=node_ids.len() { return Err(fail("dependency_cycle")); }
    // RD-689 permits up to 65,536 declared resources. Do not turn membership
    // and closure checking into two quadratic scans of that bounded set.
    // These sets are lookup-only: authored canonical order is still checked
    // above/below and is never repaired by sorting the supplied input.
    let declared_resources: BTreeSet<&str> = input.resource_paths.iter().copied().collect();
    let mut covered_resources = BTreeSet::new();
    let mut prior_resource=None;
    for row in input.resources { path(row.path)?;
        if prior_resource.is_some_and(|p| p >= (row.path,row.identity)) || !declared_resources.contains(row.path) { return Err(fail("resource_membership")); }
        covered_resources.insert(row.path);
        prior_resource=Some((row.path,row.identity));
    }
    if covered_resources.len() != declared_resources.len() { return Err(fail("resource_missing")); }
    let source=source_payload(input.sources)?;
    let mut requirement_set=count(input.requirements.len(),65536)?.to_vec();
    for row in input.requirements { blob(&mut requirement_set,row.name.as_bytes())?; blob(&mut requirement_set,row.identity)?; }
    let mut paths=count(input.resource_paths.len(),65536)?.to_vec(); for row in input.resource_paths { blob(&mut paths,row.as_bytes())?; }
    let mut config=vec![6]; for (tag,raw) in [(1,input.epoch),(2,input.entry.as_bytes()),(3,input.definition),(4,input.target_profile),(5,requirement_set.as_slice()),(6,paths.as_slice())] { field(&mut config,tag,raw)?; }
    let mut nodes=count(input.dependencies.len(),65536)?.to_vec();
    for row in input.dependencies { append(&mut nodes,&[row.kind])?; blob(&mut nodes,row.identity)?; append(&mut nodes,&row.digest)?; }
    let mut edges=count(input.edges.len(),262144)?.to_vec(); for row in input.edges { blob(&mut edges,row.from)?; blob(&mut edges,row.to)?; }
    let mut dependency=vec![2]; field(&mut dependency,1,&nodes)?; field(&mut dependency,2,&edges)?;
    let mut resources=count(input.resources.len(),65536)?.to_vec();
    for row in input.resources { blob(&mut resources,row.path.as_bytes())?; append(&mut resources,&row.identity)?; append(&mut resources,&row.byte_length.to_be_bytes())?; append(&mut resources,&row.digest)?; }
    let mut resource=vec![1]; field(&mut resource,1,&resources)?;
    let payloads=[source,config,dependency,resource];
    let preimages: [Vec<u8>;4] = PROFILES.iter().zip(&payloads).map(|(profile,payload)| preimage(profile,payload)).collect::<Result<Vec<_>,_>>()?.try_into().map_err(|_| fail("constituent_count"))?;
    let constituents=preimages.each_ref().map(|bytes| Identity32::from_raw(Sha256::digest(bytes).into()));
    let parent=super::issue_project_execution_snapshot_identity_v1(super::ProjectExecutionSnapshotConstituents {source_logical_tree:constituents[0],semantic_configuration:constituents[1],dependency_resolution:constituents[2],resource_content_closure:constituents[3]});
    if parent.envelope.len()!=194 { return Err(fail("pesi_length")); }
    Ok(IssuedProjectSnapshotIdentity {payloads,preimages,constituents,pesi_preimage:parent.envelope,pesi:parent.identity})
}
pub(super) fn verify_constituents_and_pesi(input: AdmittedProjectSnapshotIdentityInput<'_>, expected: ExpectedProjectSnapshotIdentity<'_>) -> Result<VerifiedProjectSnapshotIdentity,Error> {
    let issued=issue_constituents_and_pesi(input)?;
    if &issued.constituents!=expected.constituents || issued.pesi!=expected.pesi { return Err(Error::ConstituentMismatch("current_constituents_and_pesi")); }
    Ok(VerifiedProjectSnapshotIdentity(issued))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rd689_large_resource_closure_preserves_missing_and_foreign_rejection() {
        let owner=[1;32];
        let sources=[Source {path:"main.ddn",identity:&owner}];
        let names:Vec<String>=(0..4096).map(|n|format!("assets/{n:05}.bin")).collect();
        let paths:Vec<&str>=names.iter().map(String::as_str).collect();
        let resources:Vec<Resource<'_>>=paths.iter().map(|p|Resource {path:p,identity:owner,byte_length:1,digest:owner}).collect();
        let input=AdmittedProjectSnapshotIdentityInput {sources:&sources,epoch:&owner,entry:"main.ddn",definition:&owner,target_profile:&owner,requirements:&[],resource_paths:&paths,dependencies:&[],edges:&[],resources:&resources};
        let issued=issue_constituents_and_pesi(input).unwrap();
        verify_constituents_and_pesi(input,ExpectedProjectSnapshotIdentity {constituents:&issued.constituents,pesi:issued.pesi}).unwrap();
        assert!(matches!(issue_constituents_and_pesi(AdmittedProjectSnapshotIdentityInput {resources:&resources[..4095],..input}),Err(Error::ConstituentMalformed("resource_missing"))));
        let foreign=[Resource {path:"assets/foreign.bin",..resources[0]}];
        assert!(matches!(issue_constituents_and_pesi(AdmittedProjectSnapshotIdentityInput {resources:&foreign,..input}),Err(Error::ConstituentMalformed("resource_membership"))));
    }
    #[test]
    fn rd689_p01_exact_four_payloads_and_pesi() {
        let a1=[0xa1;32]; let a2=[0xa2;32]; let epoch=[4;32]; let definition=[6;32]; let target=[7;32];
        let sources=[Source {path:"main.ddn",identity:&a1},Source {path:"project.ddn",identity:&a2}];
        let input=AdmittedProjectSnapshotIdentityInput {sources:&sources,epoch:&epoch,entry:"main.ddn",definition:&definition,target_profile:&target,requirements:&[],resource_paths:&[],dependencies:&[],edges:&[],resources:&[]};
        let issued=issue_constituents_and_pesi(input).unwrap();
        let hashes=["f7739961fae5081172ce59aed6af54ebea44685f7ea7b977f9d6e8bc9e681d01","9744b9272e1f93ca815adf9695ee5768030c815729127bc2acd3f9d4fbe4fce8","506fd2ab48f71395fedfbbe43196ac50084730f15e0319c426f58a0198061abd","fb066f6471a9f45e81151f3a5c93faefa4f10e87915d3da4af0168d259190e01"];
        for (actual,expected) in issued.constituents.iter().zip(hashes) { assert_eq!(actual.to_hex(),expected); }
        assert_eq!(issued.payloads.each_ref().map(|p|p.len()),[109,143,19,10]);
        assert_eq!(issued.pesi_preimage.len(),194); assert_eq!(issued.pesi.to_hex(),"e76634efb031c623bea257c5f2223fe4e5a913a7ae8316faa123c0645a10326f");
        assert_eq!(hex::encode(&issued.payloads[2]),"02010000000400000000020000000400000000");
        verify_constituents_and_pesi(input,ExpectedProjectSnapshotIdentity {constituents:&issued.constituents,pesi:issued.pesi}).unwrap();
        assert!(verify_constituents_and_pesi(input,ExpectedProjectSnapshotIdentity {constituents:&issued.constituents,pesi:Identity32::from_raw([0;32])}).is_err());
        let reversed=[sources[1],sources[0]]; assert!(issue_constituents_and_pesi(AdmittedProjectSnapshotIdentityInput {sources:&reversed,..input}).is_err());
        let invalid=[Source {path:"../main.ddn",..sources[0]}]; assert!(issue_constituents_and_pesi(AdmittedProjectSnapshotIdentityInput {sources:&invalid,..input}).is_err());
    }
    #[test]
    fn rd689_p02_exact_nonempty_capsule_and_mutations() {
        let b1=[0xb1;32]; let b2=[0xb2;32]; let b3=[0xb3;32]; let epoch=[0x14;32]; let definition=[6;32]; let target=[0x17;32]; let required=[0x18;32]; let a=[0x31;32]; let b=[0x32;32];
        let sources=[Source {path:"lib/도움.ddn",identity:&b1},Source {path:"main.ddn",identity:&b2},Source {path:"project.ddn",identity:&b3}];
        let requirements=[Requirement {name:"example.dep",identity:&required}];
        let dependencies=[Dependency {kind:1,identity:&a,digest:Sha256::digest(b"p02-direct-dependency-content").into()},Dependency {kind:2,identity:&b,digest:Sha256::digest(b"p02-transitive-dependency-content").into()}];
        let edges=[Edge {from:&a,to:&b}];
        let paths=["assets/a.bin","assets/가.bin"];
        let parse_digest=|text:&str| {let mut raw=[0;32]; hex::decode_to_slice(text,&mut raw).unwrap(); raw};
        let resources=[Resource {path:paths[0],identity:[0x21;32],byte_length:14,digest:parse_digest("1afc54eb34dd61ea2848f5c7f7e8826322382a317bf9524ff8dc6824af75311b")},Resource {path:paths[1],identity:[0x22;32],byte_length:6,digest:parse_digest("c143fe9e8e3a2db7a2c30240b4ba1e9b6a0bcd1e4623b4832448cf32454c3861")}];
        let input=AdmittedProjectSnapshotIdentityInput {sources:&sources,epoch:&epoch,entry:"main.ddn",definition:&definition,target_profile:&target,requirements:&requirements,resource_paths:&paths,dependencies:&dependencies,edges:&edges,resources:&resources};
        let issued=issue_constituents_and_pesi(input).unwrap();
        for (actual,expected) in issued.constituents.iter().zip(["f7044a4e381693f35777be9a836daec34e673c26e9f01bbeaa6fb2168d74b2a2","ccaa30e5f9eb741726323f9966e01389bb733f7004ecddd01cf03f56bf44f605","1ae8d43704d8c85b7c60b3afb80143f490b3e17a7b13f907fe646d220fb968ac","ba214e813d5d2b5bee06fcd3b66b945f0e9ff0c06777ae32330bc99983c4396e"]) { assert_eq!(actual.to_hex(),expected); }
        assert_eq!(issued.pesi.to_hex(),"a12d631ef2582314ca61963bd68fa4281fbccad6eeb092670e9650b1afe3404c");
        verify_constituents_and_pesi(input,ExpectedProjectSnapshotIdentity {constituents:&issued.constituents,pesi:issued.pesi}).unwrap();
        let missing=[Edge {from:&a,to:&epoch}]; assert!(issue_constituents_and_pesi(AdmittedProjectSnapshotIdentityInput {edges:&missing,..input}).is_err());
        let cycle=[edges[0],Edge {from:&b,to:&a}]; assert!(issue_constituents_and_pesi(AdmittedProjectSnapshotIdentityInput {edges:&cycle,..input}).is_err());
        let reversed=[dependencies[1],dependencies[0]]; assert!(issue_constituents_and_pesi(AdmittedProjectSnapshotIdentityInput {dependencies:&reversed,..input}).is_err());
        let reversed=[resources[1],resources[0]]; assert!(issue_constituents_and_pesi(AdmittedProjectSnapshotIdentityInput {resources:&reversed,..input}).is_err());
        assert!(issue_constituents_and_pesi(AdmittedProjectSnapshotIdentityInput {resources:&[],..input}).is_err());
        let empty=vec![Source {path:"a.ddn",identity:&a};65]; assert!(issue_constituents_and_pesi(AdmittedProjectSnapshotIdentityInput {sources:&empty,..input}).is_err());
    }
    #[test]
    fn rd689_payload_bound_does_not_count_envelope_header() {
        let data=vec![0;MAX_PAYLOAD]; assert_eq!(preimage(PROFILES[0],&data).unwrap().len(),MAX_PAYLOAD+PROFILES[0].len()+7);
        assert!(preimage(PROFILES[0],&vec![0;MAX_PAYLOAD+1]).is_err());
        assert!(preflight(39,[Some(usize::MAX)].into_iter()).is_err());
    }
}
