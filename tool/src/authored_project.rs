//! Alpha2 authored identity storage. This is admission, not composition activation.
//! Locators and source bytes never supply persistent identities.
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;
pub(crate) mod registration;
mod source_save;
pub(crate) use source_save::prepare_authoring_source;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub const SCHEMA: &str = "ddn.authored_project_artifact.v1";
pub const SCHEMA_V2: &str = "ddn.authored_project_artifact.v2";
const MAX_BYTES: usize = 67_108_864;

macro_rules! identity {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);
        impl $name {
            pub(crate) fn fresh() -> Result<Self, String> {
                let mut bytes = [0; 32];
                getrandom::fill(&mut bytes)
                    .map_err(|_| "E_AUTHORED_ID_RANDOM: 정체 생성에 실패했습니다".to_string())?;
                Ok(Self(hex::encode(bytes)))
            }
            fn valid(&self) -> bool {
                valid_id(&self.0)
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}
identity!(ProjectId);
identity!(ModuleId);
identity!(CompositionId);
identity!(BindingId);

fn valid_id(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFile {
    pub path: String,
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_path: Option<String>,
    pub entry_path: String,
    pub files: Vec<SourceFile>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredComposition {
    pub id: CompositionId,
    pub definition: String,
    pub binding_ids: Vec<BindingId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registration: Option<crate::authored_bogae::Registration>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredModule {
    pub id: ModuleId,
    pub name: String,
    pub head: String,
    pub sources: Vec<String>,
    pub compositions: Vec<AuthoredComposition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<crate::authored_bogae::BindingDefinition>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub module: ModuleId,
    pub composition: CompositionId,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredIdentity {
    pub schema: String,
    pub logical_dependency_id: ProjectId,
    pub modules: Vec<AuthoredModule>,
    pub entry: Option<Entry>,
    pub clone_origin: Option<ProjectId>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectArtifact {
    pub schema: String,
    pub source_profile: String,
    pub source_snapshot: SourceSnapshot,
    pub authored_identity: AuthoredIdentity,
    pub content_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateRequest {
    source_profile: String,
    source_snapshot: SourceSnapshot,
    module_name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateRequest {
    original_json: String,
    source_snapshot: SourceSnapshot,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegisterBogaeRequest {
    original_json: String,
    module_name: String,
    source_path: String,
    composition_definition: String,
    make_entry: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RenameRequest {
    original_json: String,
    owner_module_id: String,
    target_kind: String,
    target_id: String,
    new_name: String,
    source_snapshot: SourceSnapshot,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RelocateRequest {
    original_json: String,
    old_path: String,
    new_path: String,
    source_snapshot: SourceSnapshot,
}

fn fail(message: &str) -> String {
    format!("E_AUTHORED_PROJECT: {message}")
}
fn locator(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':', '\0'])
        && !path.starts_with('/')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

impl ProjectArtifact {
    fn validate(&self) -> Result<(), String> {
        if !matches!(
            (self.schema.as_str(), self.authored_identity.schema.as_str()),
            (SCHEMA, "ddn.authored_identity.v1") | (SCHEMA_V2, "ddn.authored_identity.v2")
        ) {
            return Err(fail("지원하지 않는 저장 버전입니다"));
        }
        if !matches!(
            self.source_profile.as_str(),
            "v1-core-v25" | "v1-core-v25-logical-alpha2"
        ) {
            return Err(fail("문법 판을 명시해야 합니다"));
        }
        let identity = &self.authored_identity;
        if !identity.logical_dependency_id.valid()
            || identity.clone_origin.as_ref().is_some_and(|id| !id.valid())
        {
            return Err(fail("작품 정체가 올바르지 않습니다"));
        }
        let mut files = BTreeSet::new();
        for file in &self.source_snapshot.files {
            if !locator(&file.path)
                || !files.insert(&file.path)
                || file.text.starts_with('\u{feff}')
            {
                return Err(fail("원문 경로 중복·이탈 또는 BOM이 있습니다"));
            }
        }
        if !files.contains(&self.source_snapshot.entry_path) {
            return Err(fail("실행 대상 원문이 없습니다"));
        }
        let mut owned = BTreeSet::new();
        let mut modules = BTreeSet::new();
        let mut names = BTreeSet::new();
        let mut compositions = BTreeSet::new();
        for module in &identity.modules {
            if !module.id.valid()
                || !modules.insert(&module.id)
                || module.name.is_empty()
                || !names.insert(&module.name)
            {
                return Err(fail("모듈 정체 또는 작성 이름이 중복·누락되었습니다"));
            }
            if module.sources.is_empty() || !module.sources.contains(&module.head) {
                return Err(fail("모듈 머리는 소유 원문이어야 합니다"));
            }
            for path in &module.sources {
                if !files.contains(path) || !owned.insert(path) {
                    return Err(fail("원문 소유가 중복되거나 파일이 없습니다"));
                }
            }
            let mut bindings = BTreeSet::new();
            let mut definitions = BTreeSet::new();
            for composition in &module.compositions {
                if !composition.id.valid()
                    || !compositions.insert((&module.id, &composition.id))
                    || composition.definition.is_empty()
                    || !definitions.insert(&composition.definition)
                {
                    return Err(fail("짜임 정체 또는 정의 참조가 중복·누락되었습니다"));
                }
                for binding in &composition.binding_ids {
                    if !binding.valid() || (self.schema == SCHEMA && !bindings.insert(binding)) {
                        return Err(fail("작성 결속 정체가 중복·누락되었습니다"));
                    }
                }
            }
            crate::authored_bogae::validate_module_storage(self, module)?;
        }
        if identity.modules.is_empty() {
            return Err(fail("명시 작품에는 모듈 정체가 필요합니다"));
        }
        let mut expected_owned = files.clone();
        if let Some(root_path) = &self.source_snapshot.root_path {
            let graph = snapshot_graph(&self.source_snapshot, &self.source_profile)?;
            if owned.contains(root_path) {
                return Err(fail("작품 뿌리를 모듈 원문으로 소유할 수 없습니다"));
            }
            expected_owned.remove(root_path);
            if graph.module_paths.len() != identity.modules.len() {
                return Err(fail("작성 정체와 얽음의 모듈 수가 다릅니다"));
            }
            for module in &identity.modules {
                if graph.module_paths.get(&module.name) != Some(&module.head)
                    || graph
                        .module_sources
                        .get(&module.name)
                        .map(|paths| paths.iter().collect::<BTreeSet<_>>())
                        != Some(module.sources.iter().collect())
                {
                    return Err(fail(
                        "작성 모듈 소유와 얽음 선언이 다릅니다. 정체를 추정하지 않습니다",
                    ));
                }
            }
        } else if identity.modules.len() != 1 {
            return Err(fail("여러 모듈에는 명시 얽음 뿌리가 필요합니다"));
        }
        if owned != expected_owned {
            return Err(fail("소유 모듈이 없는 원문이 있습니다"));
        }
        if let Some(entry) = &identity.entry {
            if !compositions.contains(&(&entry.module, &entry.composition)) {
                return Err(fail("보개 시작 정체를 해소할 수 없습니다"));
            }
        }
        Ok(())
    }

    fn checksum(&self) -> Result<String, String> {
        let mut value =
            serde_json::to_value(self).map_err(|_| fail("저장 값을 직렬화할 수 없습니다"))?;
        value.as_object_mut().unwrap().remove("content_sha256");
        let bytes = serde_json::to_vec(&value).map_err(|_| fail("정본 직렬화에 실패했습니다"))?;
        Ok(hex::encode(Sha256::digest(bytes)))
    }

    pub fn save(&self) -> Result<String, String> {
        self.validate()?;
        let mut sealed = self.clone();
        sealed.content_sha256 = sealed.checksum()?;
        let bytes =
            serde_json::to_vec(&serde_json::to_value(sealed).map_err(|_| fail("직렬화 실패"))?)
                .map_err(|_| fail("직렬화 실패"))?;
        if bytes.len() > MAX_BYTES {
            return Err(fail("저장 크기 한도를 넘었습니다"));
        }
        String::from_utf8(bytes).map_err(|_| fail("UTF-8 직렬화 실패"))
    }

    /// Explicit creation only; callers must not invoke this while running a bare source.
    pub fn create(
        source_profile: String,
        source_snapshot: SourceSnapshot,
        module_name: String,
    ) -> Result<Self, String> {
        let modules = if source_snapshot.root_path.is_some() {
            if !module_name.is_empty() {
                return Err(fail(
                    "여러 모듈의 이름은 얽음에 선언합니다. 별도 모듈 이름을 덮어쓸 수 없습니다",
                ));
            }
            let graph = snapshot_graph(&source_snapshot, &source_profile)?;
            graph
                .module_paths
                .into_iter()
                .map(|(name, head)| {
                    Ok(AuthoredModule {
                        id: ModuleId::fresh()?,
                        sources: graph.module_sources[&name].clone(),
                        name,
                        head,
                        compositions: vec![],
                        bindings: vec![],
                    })
                })
                .collect::<Result<Vec<_>, String>>()?
        } else {
            vec![AuthoredModule {
                id: ModuleId::fresh()?,
                name: module_name,
                head: source_snapshot.entry_path.clone(),
                sources: source_snapshot
                    .files
                    .iter()
                    .map(|f| f.path.clone())
                    .collect(),
                compositions: vec![],
                bindings: vec![],
            }]
        };
        let project = Self {
            schema: SCHEMA.into(),
            source_profile,
            authored_identity: AuthoredIdentity {
                schema: "ddn.authored_identity.v1".into(),
                logical_dependency_id: ProjectId::fresh()?,
                modules,
                entry: None,
                clone_origin: None,
            },
            source_snapshot,
            content_sha256: String::new(),
        };
        project.validate()?;
        Ok(project)
    }

    /// The explicit authoring operation is the only composition/binding ID supplier.
    pub fn register_bogae(
        &self,
        module_name: &str,
        source_path: &str,
        composition_definition: &str,
        make_entry: bool,
    ) -> Result<Self, String> {
        self.validate()?;
        crate::authored_bogae::register(
            self,
            module_name,
            source_path,
            composition_definition,
            make_entry,
        )
    }

    /// Actual linked source and graph start, separate from presentation entry.
    pub fn execution_source(&self) -> Result<(String, String, Option<String>), String> {
        self.validate()?;
        if let Some(prepared) = self.authored_preparation()? {
            Ok((
                prepared.source,
                prepared.target_path,
                prepared.start_definition,
            ))
        } else {
            let source = self
                .source_snapshot
                .files
                .iter()
                .find(|file| file.path == self.source_snapshot.entry_path)
                .ok_or_else(|| fail("실행 원문이 없습니다"))?;
            Ok((source.text.clone(), source.path.clone(), None))
        }
    }

    pub(crate) fn authored_preparation(
        &self,
    ) -> Result<Option<crate::local_project::AuthoredPreparation>, String> {
        if self.source_snapshot.root_path.is_none() {
            return Ok(None);
        }
        crate::local_project::prepare_authored_execution(snapshot_source(&self.source_snapshot)?)
            .map(Some)
            .map_err(|error| fail(&format!("{}: {}", error.code, error.message)))
    }

    fn finish_explicit_edit(mut self) -> Result<Self, String> {
        crate::authored_bogae::refresh_source_locks(&mut self)?;
        let reopened = open(self.save()?.as_bytes())?;
        let (source, path, _) = reopened.execution_source()?;
        if reopened.schema == SCHEMA_V2
            && reopened
                .authored_identity
                .modules
                .iter()
                .any(|m| !m.compositions.is_empty())
        {
            crate::authored_bogae::Activation::from_artifact(&reopened.save()?, &source, &path)?;
        }
        Ok(reopened)
    }

    /// Caller supplies the completed source edit and one explicit identity map.
    pub fn rename(
        &self,
        owner_module_id: &str,
        target_kind: &str,
        target_id: &str,
        new_name: &str,
        snapshot: SourceSnapshot,
    ) -> Result<Self, String> {
        self.validate()?;
        if self.schema != SCHEMA_V2 {
            return Err(fail("정의 이름 변경은 명시 v2 작성 결속이 필요합니다"));
        }
        let parsed = ddonirang_lang::parse_v25(&format!("{new_name} := 0."), "<rename>")
            .map_err(|_| fail("새 이름은 정확한 정의 이름 하나여야 합니다"))?;
        if !parsed.items.is_empty()
            || parsed.root_stmts.len() != 1
            || !matches!(&parsed.root_stmts[0],
            ddonirang_lang::Stmt::Definition{name,..} if name==new_name)
        {
            return Err(fail("새 이름은 정확한 정의 이름 하나여야 합니다"));
        }
        if snapshot.root_path != self.source_snapshot.root_path
            || snapshot.entry_path != self.source_snapshot.entry_path
            || snapshot
                .files
                .iter()
                .map(|f| &f.path)
                .collect::<BTreeSet<_>>()
                != self
                    .source_snapshot
                    .files
                    .iter()
                    .map(|f| &f.path)
                    .collect::<BTreeSet<_>>()
        {
            return Err(fail("이름 변경으로 파일 위치·모듈 소유를 바꿀 수 없습니다"));
        }
        let mut copy = self.clone();
        let module = copy
            .authored_identity
            .modules
            .iter_mut()
            .find(|m| m.id.as_str() == owner_module_id)
            .ok_or_else(|| fail("명시 소유 모듈 정체가 없습니다"))?;
        let (old_name, path) = match target_kind {
            "binding" => {
                let binding = module
                    .bindings
                    .iter_mut()
                    .find(|b| b.id.as_str() == target_id)
                    .ok_or_else(|| fail("소유 결속 정체가 없습니다"))?;
                let old = binding.definition.clone();
                binding.definition = new_name.into();
                (old, binding.source_path.clone())
            }
            "composition" => {
                let composition = module
                    .compositions
                    .iter_mut()
                    .find(|c| c.id.as_str() == target_id)
                    .ok_or_else(|| fail("소유 짜임 정체가 없습니다"))?;
                let old = composition.definition.clone();
                composition.definition = new_name.into();
                (
                    old,
                    composition
                        .registration
                        .as_ref()
                        .ok_or_else(|| fail("짜임 등록이 없습니다"))?
                        .source_path
                        .clone(),
                )
            }
            _ => {
                return Err(fail(
                    "이름 변경 대상은 binding 또는 composition이어야 합니다",
                ))
            }
        };
        if old_name == new_name {
            return Err(fail("새 이름이 원래 이름과 같습니다"));
        }
        copy.source_snapshot = snapshot;
        crate::authored_bogae::verify_explicit_rename(
            &copy,
            owner_module_id,
            &path,
            &old_name,
            new_name,
        )?;
        copy.finish_explicit_edit()
    }

    /// Explicit file mapping only; this never rewrites source text or ownership.
    pub fn relocate(
        &self,
        old_path: &str,
        new_path: &str,
        snapshot: SourceSnapshot,
    ) -> Result<Self, String> {
        self.validate()?;
        if old_path == new_path
            || !locator(old_path)
            || !locator(new_path)
            || !self
                .source_snapshot
                .files
                .iter()
                .any(|f| f.path == old_path)
            || self
                .source_snapshot
                .files
                .iter()
                .any(|f| f.path == new_path)
        {
            return Err(fail("명시 파일 이동 위치가 잘못되었거나 충돌합니다"));
        }
        let mapped = |path: &str| {
            if path == old_path {
                new_path.to_string()
            } else {
                path.to_string()
            }
        };
        if snapshot
            .files
            .iter()
            .map(|f| f.path.clone())
            .collect::<BTreeSet<_>>()
            != self
                .source_snapshot
                .files
                .iter()
                .map(|f| mapped(&f.path))
                .collect::<BTreeSet<_>>()
            || snapshot.entry_path != mapped(&self.source_snapshot.entry_path)
            || snapshot.root_path != self.source_snapshot.root_path.as_deref().map(mapped)
        {
            return Err(fail("파일 이동 요청과 완성 원문의 위치 mapping이 다릅니다"));
        }
        let mut copy = self.clone();
        copy.source_snapshot = snapshot;
        for module in &mut copy.authored_identity.modules {
            module.head = mapped(&module.head);
            for path in &mut module.sources {
                *path = mapped(path);
            }
            for binding in &mut module.bindings {
                binding.source_path = mapped(&binding.source_path);
            }
            for composition in &mut module.compositions {
                if let Some(registration) = &mut composition.registration {
                    registration.source_path = mapped(&registration.source_path);
                }
            }
        }
        copy.finish_explicit_edit()
    }

    /// Explicit clone is atomic: original is immutable even if entropy acquisition fails.
    pub fn duplicate(&self) -> Result<Self, String> {
        self.validate()?;
        let mut copy = self.clone();
        copy.authored_identity.clone_origin =
            Some(self.authored_identity.logical_dependency_id.clone());
        copy.authored_identity.logical_dependency_id = ProjectId::fresh()?;
        for module in &mut copy.authored_identity.modules {
            let old_module = module.id.clone();
            module.id = ModuleId::fresh()?;
            let mut binding_map = BTreeMap::new();
            for binding in &mut module.bindings {
                let old = binding.id.clone();
                let fresh = BindingId::fresh()?;
                binding_map.insert(old, fresh.clone());
                binding.id = fresh;
            }
            for composition in &mut module.compositions {
                let old_composition = composition.id.clone();
                composition.id = CompositionId::fresh()?;
                if let Some(registration) = &mut composition.registration {
                    registration.clone_origin = Some(old_composition.clone());
                }
                if let Some(entry) = &mut copy.authored_identity.entry {
                    if entry.module == old_module && entry.composition == old_composition {
                        entry.module = module.id.clone();
                        entry.composition = composition.id.clone();
                    }
                }
                for binding in &mut composition.binding_ids {
                    *binding = if copy.schema == SCHEMA_V2 {
                        binding_map
                            .get(binding)
                            .cloned()
                            .ok_or_else(|| fail("복제 결속을 해소할 수 없습니다"))?
                    } else {
                        BindingId::fresh()?
                    };
                }
            }
        }
        copy.content_sha256.clear();
        copy.validate()?;
        Ok(copy)
    }
}

fn snapshot_graph(
    snapshot: &SourceSnapshot,
    profile: &str,
) -> Result<crate::local_project::LocalProjectGraph, String> {
    let source = snapshot_source(snapshot)?;
    crate::local_project::project_source_profile(&source, Some(profile))
        .map_err(|error| fail(&format!("{}: {}", error.code, error.message)))?;
    crate::local_project::build_project_graph(source)
        .map_err(|error| fail(&format!("{}: {}", error.code, error.message)))
}

fn snapshot_source(
    snapshot: &SourceSnapshot,
) -> Result<crate::local_project::VirtualProjectSource, String> {
    use crate::local_project::{self, VirtualProjectFileSource, VirtualProjectSource};
    let root = snapshot
        .root_path
        .as_ref()
        .ok_or_else(|| fail("명시 작품 뿌리가 없습니다"))?;
    if !locator(root) {
        return Err(fail("작품 뿌리 경로가 올바르지 않습니다"));
    }
    let source = VirtualProjectSource {
        schema: local_project::VIRTUAL_PROJECT_SOURCE_SCHEMA.into(),
        kind: None,
        package_hash: None,
        root_path: root.clone(),
        entry_path: snapshot.entry_path.clone(),
        files: snapshot
            .files
            .iter()
            .map(|file| VirtualProjectFileSource {
                path: file.path.clone(),
                role: if &file.path == root {
                    "root"
                } else {
                    "ddn_source"
                }
                .into(),
                utf8_text: file.text.clone(),
            })
            .collect(),
    };
    Ok(source)
}

// serde_json::Value normally silently accepts duplicate keys. Admission must not.
struct StrictValue(Value);
impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = StrictValue;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("중복 없는 정수 JSON")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut out = vec![];
                while let Some(v) = a.next_element::<StrictValue>()? {
                    out.push(v.0);
                }
                Ok(StrictValue(out.into()))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut out = serde_json::Map::new();
                while let Some(k) = a.next_key::<String>()? {
                    if out.contains_key(&k) {
                        return Err(de::Error::custom("중복 JSON 키"));
                    }
                    out.insert(k, a.next_value::<StrictValue>()?.0);
                }
                Ok(StrictValue(out.into()))
            }
        }
        d.deserialize_any(V)
    }
}

pub fn open(bytes: &[u8]) -> Result<ProjectArtifact, String> {
    if bytes.len() > MAX_BYTES || bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(fail("저장 크기 초과 또는 BOM입니다"));
    }
    let value: StrictValue =
        serde_json::from_slice(bytes).map_err(|_| fail("UTF-8·중복 키·수 표기를 확인하세요"))?;
    let project: ProjectArtifact =
        serde_json::from_value(value.0).map_err(|_| fail("저장 필드가 올바르지 않습니다"))?;
    project.validate()?;
    if project.content_sha256 != project.checksum()? {
        return Err(fail("저장 내용 해시가 다릅니다"));
    }
    Ok(project)
}

/// Shared native/WASM entry point. Does not activate unvalidated authored views.
pub fn operate(operation: &str, text: &str) -> Result<String, String> {
    match operation {
        // Private Workbench transport operations, not DDN callables or new
        // CLI command names. Source candidates grant no execution authority.
        "source_prepare"=>source_save::prepare(text),
        "source_open"=>source_save::open(text),
        "source_probe"=>source_save::requirements(text),
        "create" => {
            if text.len() > MAX_BYTES || text.starts_with('\u{feff}') {
                return Err(fail("크기 초과 또는 BOM입니다"));
            }
            let value: StrictValue =
                serde_json::from_str(text).map_err(|_| fail("작성 요청 JSON을 확인하세요"))?;
            let request: CreateRequest =
                serde_json::from_value(value.0).map_err(|_| fail("작성 요청 필드를 확인하세요"))?;
            ProjectArtifact::create(
                request.source_profile,
                request.source_snapshot,
                request.module_name,
            )?
            .save()
        }
        "open" => open(text.as_bytes())?.save(),
        "clone" => open(text.as_bytes())?.duplicate()?.save(),
        "update" => {
            if text.len() > MAX_BYTES || text.starts_with('\u{feff}') {
                return Err(fail("크기 초과 또는 BOM입니다"));
            }
            let value: StrictValue =
                serde_json::from_str(text).map_err(|_| fail("갱신 요청 JSON을 확인하세요"))?;
            let request: UpdateRequest =
                serde_json::from_value(value.0).map_err(|_| fail("갱신 요청 필드를 확인하세요"))?;
            // Verify the original before any edit; do not bless a tampered
            // identity registry by simply computing a new checksum for it.
            let mut project = open(request.original_json.as_bytes())?;
            if project.source_snapshot.root_path != request.source_snapshot.root_path {
                return Err(fail("원문 갱신으로 작품 뿌리를 추가·제거·바꿀 수 없습니다"));
            }
            project.source_snapshot = request.source_snapshot;
            crate::authored_bogae::refresh_source_locks(&mut project)?;
            // Module ownership is retained. File moves/ownership changes must
            // use an explicit edit operation, never infer new IDs from paths.
            project.save()
        }
        "register_bogae" => {
            if text.len() > MAX_BYTES || text.starts_with('\u{feff}') {
                return Err(fail("크기 초과 또는 BOM입니다"));
            }
            let value: StrictValue =
                serde_json::from_str(text).map_err(|_| fail("보개 등록 요청 JSON을 확인하세요"))?;
            let request: RegisterBogaeRequest = serde_json::from_value(value.0)
                .map_err(|_| fail("보개 등록 요청 필드를 확인하세요"))?;
            open(request.original_json.as_bytes())?
                .register_bogae(
                    &request.module_name,
                    &request.source_path,
                    &request.composition_definition,
                    request.make_entry,
                )?
                .save()
        }
        "rename" => {
            if text.len() > MAX_BYTES || text.starts_with('\u{feff}') {
                return Err(fail("크기 초과 또는 BOM입니다"));
            }
            let value: StrictValue =
                serde_json::from_str(text).map_err(|_| fail("이름 변경 요청 JSON을 확인하세요"))?;
            let request: RenameRequest = serde_json::from_value(value.0)
                .map_err(|_| fail("이름 변경 요청 필드를 확인하세요"))?;
            open(request.original_json.as_bytes())?
                .rename(
                    &request.owner_module_id,
                    &request.target_kind,
                    &request.target_id,
                    &request.new_name,
                    request.source_snapshot,
                )?
                .save()
        }
        "relocate" => {
            if text.len() > MAX_BYTES || text.starts_with('\u{feff}') {
                return Err(fail("크기 초과 또는 BOM입니다"));
            }
            let value: StrictValue =
                serde_json::from_str(text).map_err(|_| fail("파일 이동 요청 JSON을 확인하세요"))?;
            let request: RelocateRequest = serde_json::from_value(value.0)
                .map_err(|_| fail("파일 이동 요청 필드를 확인하세요"))?;
            open(request.original_json.as_bytes())?
                .relocate(
                    &request.old_path,
                    &request.new_path,
                    request.source_snapshot,
                )?
                .save()
        }
        _ => Err(fail("지원하지 않는 작성 작업입니다")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> ProjectArtifact {
        ProjectArtifact::create(
            "v1-core-v25".into(),
            SourceSnapshot {
                root_path: None,
                entry_path: "main.ddn".into(),
                files: vec![SourceFile {
                    path: "main.ddn".into(),
                    text: "값 := 1.".into(),
                }],
            },
            "앱".into(),
        )
        .unwrap()
    }
    #[test]
    fn update_preserves_identity_and_original_bytes() {
        let original = fixture();
        let bytes = original.save().unwrap();
        let mut snapshot = original.source_snapshot.clone();
        snapshot.files[0].text = "값 := 2.".into();
        let request = serde_json::json!({"original_json": bytes, "source_snapshot": snapshot});
        let updated = open(operate("update", &request.to_string()).unwrap().as_bytes()).unwrap();
        assert_eq!(updated.authored_identity, original.authored_identity);
        assert_eq!(updated.source_profile, original.source_profile);
        assert_eq!(updated.source_snapshot, snapshot);
        assert_ne!(
            updated.content_sha256,
            open(bytes.as_bytes()).unwrap().content_sha256
        );
        assert_eq!(
            open(bytes.as_bytes()).unwrap().source_snapshot,
            original.source_snapshot
        );
    }
    #[test]
    fn multi_module_root_is_separate_and_ids_survive_edit() {
        let snapshot = SourceSnapshot {
            root_path: Some("work.ddn".into()), entry_path: "app/main.ddn".into(),
            files: vec![
                SourceFile {path:"work.ddn".into(), text:r#"(글판="v25", 모듬=차림[(이름=앱, 머리="app/main.ddn", 원문="app/main.ddn"),(이름=힘, 머리="lib/힘.ddn", 원문="lib/힘.ddn")], 시작=(모듈=앱, 정의=실행)) 얽음."#.into()},
                SourceFile {path:"app/main.ddn".into(),text:"실행:움직씨 := { 1 보여주기. }.".into()},
                SourceFile {path:"lib/힘.ddn".into(),text:"힘 := 2.".into()},
            ],
        };
        let original =
            ProjectArtifact::create("v1-core-v25".into(), snapshot.clone(), String::new()).unwrap();
        assert_eq!(original.authored_identity.modules.len(), 2);
        assert!(original
            .authored_identity
            .modules
            .iter()
            .all(|module| !module.sources.contains(&"work.ddn".to_string())));
        let bytes = original.save().unwrap();
        let mut edited = snapshot;
        edited.files[2].text = "힘 := 3.".into();
        let request = serde_json::json!({"original_json":bytes,"source_snapshot":edited});
        let updated = open(operate("update", &request.to_string()).unwrap().as_bytes()).unwrap();
        assert_eq!(updated.authored_identity, original.authored_identity);
        let mut bad = updated.clone();
        bad.source_snapshot.root_path = None;
        bad.source_snapshot
            .files
            .retain(|file| file.path != "work.ddn");
        assert!(bad.save().is_err());
        let request =
            serde_json::json!({"original_json":bytes,"source_snapshot":bad.source_snapshot});
        assert!(operate("update", &request.to_string()).is_err());
        let mut bad = updated.clone();
        bad.authored_identity.modules[0].head = "lib/힘.ddn".into();
        assert!(bad.save().is_err());
        let mut bad = updated;
        bad.source_snapshot.files.push(SourceFile {
            path: "extra.ddn".into(),
            text: "3 보여주기.".into(),
        });
        assert!(bad.save().is_err());
    }
    #[test]
    fn update_rejects_tampered_original_and_implicit_ownership_change() {
        let original = fixture();
        let bytes = original.save().unwrap();
        let tampered = bytes.replace("값 := 1.", "값 := 99.");
        let request = serde_json::json!({"original_json": tampered, "source_snapshot": original.source_snapshot});
        assert!(operate("update", &request.to_string()).is_err());
        let mut snapshot = original.source_snapshot.clone();
        snapshot.files.push(SourceFile {
            path: "unowned.ddn".into(),
            text: "1 보여주기.".into(),
        });
        let request = serde_json::json!({"original_json": bytes, "source_snapshot": snapshot});
        assert!(operate("update", &request.to_string()).is_err());
    }
    #[test]
    fn save_reopen_move_rename_preserve_ids() {
        let mut p = fixture();
        let ids = p.authored_identity.clone();
        assert_eq!(
            open(p.save().unwrap().as_bytes())
                .unwrap()
                .authored_identity,
            ids
        );
        p.source_snapshot.entry_path = "app/new.ddn".into();
        p.source_snapshot.files[0].path = "app/new.ddn".into();
        p.authored_identity.modules[0].head = "app/new.ddn".into();
        p.authored_identity.modules[0].sources = vec!["app/new.ddn".into()];
        p.authored_identity.modules[0].name = "새이름".into();
        let reopened = open(p.save().unwrap().as_bytes()).unwrap();
        assert_eq!(
            reopened.authored_identity.logical_dependency_id,
            ids.logical_dependency_id
        );
        assert_eq!(reopened.authored_identity.modules[0].id, ids.modules[0].id);
    }
    #[test]
    fn clone_references_and_origin_are_explicit() {
        let mut p = fixture();
        let c = CompositionId::fresh().unwrap();
        let m = p.authored_identity.modules[0].id.clone();
        p.authored_identity.modules[0]
            .compositions
            .push(AuthoredComposition {
                id: c.clone(),
                definition: "보기".into(),
                binding_ids: vec![BindingId::fresh().unwrap()],
                registration: None,
            });
        p.authored_identity.entry = Some(Entry {
            module: m,
            composition: c,
        });
        let original = p.clone();
        let copy = open(p.duplicate().unwrap().save().unwrap().as_bytes()).unwrap();
        assert_eq!(p, original);
        assert_ne!(
            p.authored_identity.modules[0].id,
            copy.authored_identity.modules[0].id
        );
        assert_ne!(
            p.authored_identity.modules[0].compositions[0].id,
            copy.authored_identity.modules[0].compositions[0].id
        );
        assert_ne!(
            p.authored_identity.modules[0].compositions[0].binding_ids,
            copy.authored_identity.modules[0].compositions[0].binding_ids
        );
        assert_ne!(
            p.authored_identity.logical_dependency_id,
            copy.authored_identity.logical_dependency_id
        );
        assert_eq!(
            copy.authored_identity.clone_origin,
            Some(p.authored_identity.logical_dependency_id.clone())
        );
        assert_eq!(
            copy.authored_identity.entry.as_ref().unwrap().module,
            copy.authored_identity.modules[0].id
        );
        assert_eq!(
            copy.authored_identity.entry.as_ref().unwrap().composition,
            copy.authored_identity.modules[0].compositions[0].id
        );
    }
    #[test]
    fn rejects_resealed_unowned_source() {
        let mut p = fixture();
        p.source_snapshot.files.push(SourceFile {
            path: "extra.ddn".into(),
            text: "값 := 2.".into(),
        });
        p.content_sha256 = p.checksum().unwrap();
        let bytes = serde_json::to_vec(&p).unwrap();
        assert!(open(&bytes).unwrap_err().contains("소유 모듈이 없는"));
        assert!(p.save().is_err());
    }
    #[test]
    fn rejects_corruption_duplicate_unknown_and_unowned_source() {
        let p = fixture();
        let sealed = p.save().unwrap();
        assert!(open(sealed.replace("값 := 1.", "값 := 2.").as_bytes()).is_err());
        assert!(open(format!("{{\"schema\":\"bad\",{}", &sealed[1..]).as_bytes()).is_err());
        assert!(open(format!("{{\"unknown\":1,{}", &sealed[1..]).as_bytes()).is_err());
        assert!(open(format!("{{\"unknown\":1.5,{}", &sealed[1..]).as_bytes()).is_err());
        let mut bad = p.clone();
        bad.authored_identity
            .modules
            .push(bad.authored_identity.modules[0].clone());
        assert!(bad.save().is_err());
        let mut bad = p;
        bad.authored_identity.modules[0].head = "../main.ddn".into();
        assert!(bad.save().is_err());
    }
}
