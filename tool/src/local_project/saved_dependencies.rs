//! Private equivalent-seal supply for the same admitted saved source request.
//! A legacy lock/schema or a caller-provided verified flag is never sufficient.
use super::*;
use crate::project_snapshot_identity::{self as identity, VerifiedDependencyNode, DependencyNodeKind};

#[derive(Debug)]
pub(crate) struct VerifiedSavedDependencySupply {
    source_graph_sha256: String,
    source_profile: String,
    definition_identity: Vec<u8>,
    requirements: Vec<(String, Vec<u8>)>,
    nodes: Vec<VerifiedDependencyNode>,
    verified_graph_identity: identity::Identity32,
}
impl VerifiedSavedDependencySupply {
    pub(crate) fn verify_source_binding(&self, owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1) -> Result<(),String> {
        if self.source_graph_sha256 != owned.sources().saved().manifest().project_graph_sha256
            || self.source_profile != owned.project().source_profile
            || self.definition_identity.as_slice() != owned.definition_identity().as_bytes() {
            return Err("E_SAVED_PROJECT_EXTERNAL_SUPPLY_REQUIRED: 의존 공급과 저장 원문 판본이 다릅니다".into());
        }
        Ok(())
    }
    pub(crate) fn requirements(&self) -> &[(String,Vec<u8>)] { &self.requirements }
    pub(crate) fn nodes(&self) -> &[VerifiedDependencyNode] { &self.nodes }
    pub(crate) fn verified_graph_identity(&self) -> identity::Identity32 { self.verified_graph_identity }
}
fn error(error: LocalProjectError) -> String { format!("{}: {}",error.code,error.message) }
fn missing(message: &str) -> String { format!("E_SAVED_PROJECT_EXTERNAL_SUPPLY_REQUIRED: {message}") }
fn exact(version: &str) -> bool {
    !version.is_empty() && !version.eq_ignore_ascii_case("latest")
        && !version.chars().any(|ch|ch.is_whitespace() || "*^~<>=|,".contains(ch))
}

pub(crate) fn verify_saved_dependency_supply(
    owned: &crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1,
) -> Result<VerifiedSavedDependencySupply,String> {
    if !cfg!(feature="v2-shared-modules") { return Err(missing("이 제품은 고정 helper 저장 공급을 지원하지 않습니다")); }
    let saved=owned.sources().saved();
    let source=saved.source();
    let profile=project_source_profile(source,Some(&owned.project().source_profile)).map_err(error)?;
    let graph=build_project_graph(source.clone()).map_err(error)?;
    if graph.gaji_aliases.is_empty() { return Err(missing("실제 외부 요구가 없는 입력을 의존 공급으로 바꾸지 않습니다")); }
    // Recompute from the exact in-memory closure, never disk/latest/network.
    let lock=build_project_gaji_lock(source.clone(),true).map_err(error)?;
    if lock.packages.is_empty() { return Err(missing("외부 요구의 고정 package 목록이 없습니다")); }
    let root=source.files.iter().find(|file|file.path==source.root_path).ok_or_else(||missing("뿌리 원문이 없습니다"))?;
    let metadata=v25_package_metadata_roles(&root.utf8_text).map_err(error)?;
    for file in &source.files {
        match file.role.as_str() {
            "root" | "ddn_source" => {},
            role if metadata.get(&file.path).is_some_and(|expected|expected==role) => {},
            _ => return Err(missing("검증된 package 밖의 자료 역할이 포함돼 있습니다")),
        }
    }
    let mut nodes_by_id=BTreeMap::new();
    for pin in &lock.packages {
        if !exact(&pin.version) { return Err("E_GAJI_DEP_UNPINNED: 고정 판본만 저장할 수 있습니다".into()); }
        let artifact=installed_artifact_from_source(source,&pin.id).map_err(error)?;
        // The existing creator emits the closed D-STRICT profile. It has no
        // conformance supplier for extra requirements or external aliases;
        // preserve the guard instead of treating unknown metadata as true.
        let manifest=artifact.files.iter().find(|file|file.path=="gaji.toml")
            .ok_or_else(||missing("package manifest가 없습니다"))?;
        if !artifact.dependencies.is_empty()
            || manifest.utf8_text!=local_gaji_artifact_manifest(&artifact.gaji_id,&artifact.version,&artifact.content_sha256) {
            return Err(missing("이 첫 공급은 검증된 closed D-STRICT helper만 지원합니다. 추가 요구·전이 외부 공급의 정확한 결속이 필요합니다"));
        }
        if artifact.gaji_id!=pin.id || artifact.version!=pin.version || artifact.dependencies!=pin.dependencies {
            return Err(missing("manifest·고정 artifact·lock의 요구나 판본이 다릅니다"));
        }
        let mut expected_files=BTreeSet::new();
        expected_files.insert(format!("{}/artifact.detjson",pin.path));
        for file in &artifact.files {
            let path=format!("{}/{}",pin.path,file.path);
            expected_files.insert(path.clone());
            let actual=source.files.iter().find(|candidate|candidate.path==path)
                .ok_or_else(||missing("package 원자료가 빠져 있습니다"))?;
            if actual.utf8_text!=file.utf8_text { return Err("E_GAJI_ARTIFACT_FILE_HASH: 고정 package 원자료가 다릅니다".into()); }
        }
        let actual_files: BTreeSet<_>=source.files.iter().filter(|file|file.path.starts_with(&format!("{}/",pin.path)))
            .map(|file|file.path.clone()).collect();
        if actual_files!=expected_files { return Err(missing("package 전체 원자료 집합이 다릅니다")); }
        // Reuse the parser/lexical module export owner; artifact display rows
        // cannot manufacture an export or an internal definition.
        let mut actual_public=BTreeSet::new();
        for head in graph.module_paths.values().filter(|head|head.starts_with(&format!("{}/ddn/",pin.path))) {
            let directives=resolve_work_project_module_directives(source,head).map_err(error)?;
            for export in directives.exports {
                actual_public.insert((head.clone(),export.external_name,export.internal_name));
            }
        }
        let mut declared_public=BTreeSet::new();
        for symbol in &artifact.public_symbols {
            if !declared_public.insert((format!("{}/ddn/{}",pin.path,symbol.module_path),symbol.external_name.clone(),symbol.internal_name.clone())) {
                return Err(missing("package 공개 선언이 중복됐습니다"));
            }
        }
        if actual_public!=declared_public { return Err(missing("package 공개 경계와 실제 원문 해소가 다릅니다")); }
        let node=serde_json::to_vec(&[pin.id.as_str(),pin.version.as_str(),profile])
            .map_err(|_|missing("고정 package 참조를 만들 수 없습니다"))?;
        let content=hex::decode(&pin.package_sha256).map_err(|_|missing("검증된 package 내용 정체가 잘못됐습니다"))?;
        if content.len()!=32 { return Err(missing("검증된 package 내용 정체 길이가 다릅니다")); }
        if nodes_by_id.insert(pin.id.clone(),(node,content)).is_some() { return Err(missing("package 소유 정체가 중복됐습니다")); }
    }
    let mut nodes=Vec::new();
    for pin in &lock.packages {
        let (node,content)=nodes_by_id.get(&pin.id).ok_or_else(||missing("package 정체가 없습니다"))?;
        let mut dependencies=Vec::new();
        for (id,version) in &pin.dependencies {
            if !exact(version) || lock.packages.iter().filter(|target|target.id==*id && target.version==*version).count()!=1 {
                return Err("E_GAJI_VERSION_PIN_MISMATCH: 전이 요구와 고정 package 판본이 다릅니다".into());
            }
            dependencies.push(nodes_by_id.get(id).ok_or_else(||missing("전이 package가 없습니다"))?.0.clone());
        }
        dependencies.sort();
        nodes.push(VerifiedDependencyNode {kind:DependencyNodeKind::DdnGaji,node_identity:node.clone(),content_identity:content.clone(),dependencies});
    }
    nodes.sort_by(|a,b|a.node_identity.cmp(&b.node_identity));
    let verified=identity::issue_verified_dependency_resolution_identity_v1(&identity::VerifiedDependencyResolutionInput {graph_verified:true,nodes:nodes.clone()})
        .map_err(|error|format!("E_SAVED_PROJECT_EXECUTION_IDENTITY: {error:?}"))?;
    let mut requirements=Vec::new();
    for (alias,path) in &graph.gaji_aliases {
        let package=lock.packages.iter().find(|pin|pin.path==*path && pin.direct).ok_or_else(||missing("직접 요구의 고정 package가 없습니다"))?;
        requirements.push((alias.clone(),nodes_by_id.get(&package.id).ok_or_else(||missing("직접 요구의 package 정체가 없습니다"))?.0.clone()));
    }
    requirements.sort();
    Ok(VerifiedSavedDependencySupply {source_graph_sha256:saved.manifest().project_graph_sha256.clone(),source_profile:profile.into(),definition_identity:owned.definition_identity().as_bytes().to_vec(),requirements,nodes,verified_graph_identity:verified.identity})
}
