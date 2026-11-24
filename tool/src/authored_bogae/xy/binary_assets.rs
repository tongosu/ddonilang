//! Transient PNG input preparation, not an AssetRef/rights/run/view capability.
//! Reuses the existing source normalizer and strict v0 registry; no text wire
//! changes, entropy, global registry, I/O, source execution or storage writes.
use std::{collections::HashMap, sync::Arc};
use crate::source_snapshot_admission::SourceCapture;
use crate::{gate0_registry::{build_project_asset_registry_from_manifest, ProjectAssetRegistry}, local_project::{self, VirtualProjectSource}};

pub(crate) struct PreparedBinaryAssets {
    registry: Arc<ProjectAssetRegistry>,
    source_path: String,
    source_sha256: String,
    source_snapshot_identity: [u8;32],
    project_manifest_sha256: Option<String>,
}
impl PreparedBinaryAssets {
    pub(crate) fn registry(&self) -> &Arc<ProjectAssetRegistry> { &self.registry }
    pub(crate) fn source_path(&self) -> &str { &self.source_path }
    pub(crate) fn source_sha256(&self) -> &str { &self.source_sha256 }
    // This is the existing source owner value, never a stable AssetRef ID.
    pub(crate) fn source_snapshot_identity(&self) -> &[u8;32] { &self.source_snapshot_identity }
    pub(crate) fn matches_source(&self, path: &str, capture: &SourceCapture) -> bool {
        self.source_path == path && self.source_snapshot_identity == *capture.identity()
    }
    // A native source/registry association has no virtual-source bundle seal.
    // Absence is preserved rather than borrowing an asset-manifest hash.
    pub(crate) fn project_manifest_sha256(&self) -> Option<&str> { self.project_manifest_sha256.as_deref() }
    pub(crate) fn from_native_execution(registry: Arc<ProjectAssetRegistry>,
        source_path: &str, capture: &SourceCapture) -> Self {
        // The execution owner passes its actual sealed capture. This neither
        // verifies a permissive registry nor creates an AssetRef/rights cap.
        Self { registry, source_path: source_path.to_string(),
            source_sha256: hex::encode(capture.content_digest()),
            source_snapshot_identity: *capture.identity(), project_manifest_sha256: None }
    }
}
/// A private source-bound byte capability, not public image admission.
/// The checked material is selected explicitly by its real owner/asset/version;
/// matching bytes or a handle alone never selects or issues an asset identity.
pub(crate) struct SourceBoundStaticPng<'a> {
    checked: super::static_png_assets::CheckedStaticPngBytes<'a>,
    source: &'a PreparedBinaryAssets,
    asset_path: &'a str,
    handle: ddonirang_core::ResourceHandle,
}
impl<'a> SourceBoundStaticPng<'a> {
    pub(crate) fn bytes(&self) -> &'a [u8] { self.checked.bytes() }
    pub(crate) fn material(&self) -> &super::static_png_assets::CheckedStaticPngMaterial { self.checked.material() }
    pub(crate) fn asset_path(&self) -> &str { self.asset_path }
    pub(crate) fn image_selection(&self) -> Result<ddonirang_lang::runtime::StaticPngSelection,String> {
        let sha:[u8;32]=hex::decode(self.source.source_sha256()).map_err(fail)?
            .try_into().map_err(|_|fail("원문 내용 해시 크기가 다릅니다"))?;
        Ok(ddonirang_lang::runtime::StaticPngSelection::from_checked_source_bytes(&self.checked,self.handle,sha))
    }

    pub(crate) fn source_snapshot_identity(&self) -> &[u8;32] { self.source.source_snapshot_identity() }
    pub(crate) fn matches_current_source(&self, path:&str, capture:&SourceCapture) -> bool {
        self.source.matches_source(path, capture)
    }
}
impl PreparedBinaryAssets {
    pub(crate) fn bind_static_png<'a>(&'a self,
        material: &'a super::static_png_assets::CheckedStaticPngMaterial,
        handle: ddonirang_core::ResourceHandle,
        path: &str, capture: &SourceCapture) -> Result<SourceBoundStaticPng<'a>, String> {
        if !self.matches_source(path, capture) { return Err(fail("그림과 실제 실행 원문 고정본이 다릅니다")); }
        let entry = self.registry.read_verified_asset_bytes(handle, self.registry.manifest_sha256()).map_err(fail)?;
        let checked = material.verify_bytes(entry.bytes)?;
        Ok(SourceBoundStaticPng { checked, source:self, asset_path:entry.path, handle })
    }
}

fn fail(detail: impl std::fmt::Display) -> String { format!("E_PROJECT_BINARY_ASSETS: {detail}") }

pub(crate) fn prepare_binary_assets(
    project_source_json: &str,
    files: &HashMap<String, Vec<u8>>,
    expected_source_path: &str,
    source_capture: &SourceCapture,
) -> Result<PreparedBinaryAssets, String> {
    if project_source_json.len() > local_project::MAX_SERIALIZED_ARTIFACT_BYTES
        || files.is_empty() || files.len() > local_project::MAX_PROJECT_FILES {
        return Err(fail("원문 또는 바이너리 입력 개수 한도를 확인하세요"));
    }
    // Reuse existing project byte ceilings as technical input limits, without
    // pretending binary input is part of the text source serialization.
    let mut total = 0usize;
    for (path, bytes) in files {
        if local_project::normalize_project_path(path).map_err(fail)? != *path
            || bytes.len() > local_project::MAX_FILE_BYTES {
            return Err(fail("정규 자산 경로와 파일 크기 한도를 확인하세요"));
        }
        total = total.checked_add(bytes.len()).ok_or_else(|| fail("입력 크기 초과"))?;
        if total > local_project::MAX_PROJECT_TOTAL_UTF8_BYTES { return Err(fail("바이너리 합계 크기 초과")); }
    }
    let source: VirtualProjectSource = serde_json::from_str(project_source_json).map_err(fail)?;
    if source.files.len() > local_project::MAX_PROJECT_FILES { return Err(fail("원문 파일 개수 초과")); }
    let manifests: Vec<_> = source.files.iter().filter(|f| f.path == "ddn.asset.json").collect();
    if manifests.len() != 1 { return Err(fail("정확한 ddn.asset.json 하나가 필요합니다")); }
    let manifest = manifests[0];
    if manifest.role != "asset_manifest" || manifest.utf8_text.len() > local_project::MAX_FILE_BYTES {
        return Err(fail("자산 manifest 역할 또는 크기를 확인하세요"));
    }
    let raw: serde_json::Value = serde_json::from_str(&manifest.utf8_text).map_err(fail)?;
    let entries = raw.get("entries").and_then(|v| v.as_array()).ok_or_else(|| fail("자산 entries가 필요합니다"))?;
    if entries.len() != files.len() || entries.iter().any(|e| e.get("sealed_dataset").is_some_and(|v| !v.is_null())) {
        return Err(fail("선언 자산과 바이너리 집합이 다르거나 자료셋 연결입니다"));
    }
    // Only this explicit display manifest is removed for text normalization;
    // other evidence/dataset roles keep their existing feature gates.
    let mut text_source = source.clone();text_source.files.retain(|f| f.path != "ddn.asset.json");
    for file in &text_source.files {
        if files.contains_key(&file.path) { return Err(fail("자산과 원문 경로가 겹칩니다")); }
    }
    let normalized = local_project::normalize_virtual_project_source(text_source).map_err(fail)?;
    let path = local_project::normalize_project_path(expected_source_path).map_err(fail)?;
    if path != expected_source_path || normalized.entry_path != path { return Err(fail("실행 원문 경로가 entry와 다릅니다")); }
    let entry = source.files.iter().find(|f| f.path == path).ok_or_else(|| fail("실행 원문이 없습니다"))?;
    if entry.utf8_text.as_bytes() != source_capture.bytes() { return Err(fail("실제 실행 원문 판본이 다릅니다")); }
    let source_sha256 = hex::encode(source_capture.content_digest());
    let registry = build_project_asset_registry_from_manifest(manifest.utf8_text.as_bytes(), files, true).map_err(fail)?;
    for (path, expected) in files {
        let handle = registry.resolve_handle(path).map_err(fail)?;
        let asset = registry.read_verified_asset_bytes(handle, registry.manifest_sha256()).map_err(fail)?;
        if asset.mime != Some("image/png") || asset.bytes != expected
            || !asset.bytes.starts_with(&[137,80,78,71,13,10,26,10]) {
            return Err(fail("이 입력은 원시 PNG 표시 자산만 받습니다"));
        }
    }
    // This does not bind a semantic asset owner/version, a run or view.
    // The caller must not use input preparation as public image admission.
    Ok(PreparedBinaryAssets { registry: Arc::new(registry), source_path: path,
        source_sha256, source_snapshot_identity: *source_capture.identity(),
        project_manifest_sha256: Some(normalized.canonical_manifest_sha256) })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn capture_for(code:&str) -> SourceCapture {
        let program=crate::ddn_runtime::DdnProgram::from_v25_source_with_mode(code,"main.ddn",ddonirang_lang::ParseMode::Strict).unwrap();
        let runner=crate::ddn_runtime::DdnRunner::new(program,"매마디");
        runner.sealed_execution_source_capture().unwrap().clone()
    }
    fn fixture() -> (serde_json::Value, HashMap<String, Vec<u8>>, SourceCapture) {
        let path = "images/pixel.png";
        let bytes = include_bytes!("../../../tests/bogae_xy/fixtures/배치방향_2x2.png").to_vec();
        let handle = ddonirang_core::asset_handle_from_bundle_path("binary-input-ui-fixture", path);
        let manifest = serde_json::json!({"version":"v0","bundle_id":"binary-input-ui-fixture","hash_algo":"blake3","entries":[{
            "path":path,"handle":handle.to_hex(),"hash":blake3::hash(&bytes).to_hex().to_string(),"size":bytes.len(),"mime":"image/png"}]});
        let code = "1 보여주기.\n";
        let source = serde_json::json!({"schema":local_project::VIRTUAL_PROJECT_SOURCE_SCHEMA,"root_path":"main.ddn","entry_path":"main.ddn","files":[
            {"path":"main.ddn","role":"ddn_source","utf8_text":code},
            {"path":"ddn.asset.json","role":"asset_manifest","utf8_text":manifest.to_string()}]});
        (source, HashMap::from([(path.into(), bytes)]), capture_for(code))
    }
    fn prepare(source:&serde_json::Value, files:&HashMap<String,Vec<u8>>, capture:&SourceCapture) -> Result<PreparedBinaryAssets,String> {
        prepare_binary_assets(&source.to_string(),files,"main.ddn",capture)
    }
    #[test]
    fn binary_input_raw_png_and_exact_source_reuse_strict_registry_without_asset_issuance() {
        let (source,mut files,digest)=fixture();let before=source.clone();let original=files["images/pixel.png"].clone();
        let prepared=prepare(&source,&files,&digest).unwrap();files.get_mut("images/pixel.png").unwrap()[0]=0;
        assert_eq!(prepared.source_sha256(),hex::encode(digest.content_digest()));assert_eq!(prepared.source_path(),"main.ddn");
        assert!(prepared.project_manifest_sha256().is_some_and(|hash|!hash.is_empty()));assert_eq!(source,before);
        let registry=prepared.registry();let handle=registry.resolve_handle("images/pixel.png").unwrap();
        assert_eq!(registry.read_verified_asset_bytes(handle,registry.manifest_sha256()).unwrap().bytes,original);
        assert!(registry.sealed_dataset_identities().unwrap().is_empty());
    }
    #[test]
    fn binary_input_missing_extra_tampered_paths_and_source_mismatch_refuse() {
        let (source,files,digest)=fixture();
        assert!(prepare(&source,&HashMap::new(),&digest).is_err());
        let mut extra=files.clone();extra.insert("extra.png".into(),vec![0]);assert!(prepare(&source,&extra,&digest).is_err());
        let mut changed=files.clone();changed.get_mut("images/pixel.png").unwrap()[10]^=1;assert!(prepare(&source,&changed,&digest).is_err());
        assert!(prepare(&source,&files,&capture_for("2 보여주기.\n")).is_err());
    }
    #[test]
    fn binary_input_does_not_open_dataset_or_text_frontdoors() {
        let (source,files,digest)=fixture();let mut manifest:serde_json::Value=serde_json::from_str(source["files"][1]["utf8_text"].as_str().unwrap()).unwrap();
        for modification in [serde_json::json!({"mime":"text/plain"}),serde_json::json!({"sealed_dataset":{}})] {
            let mut altered=manifest.clone();for (key,value) in modification.as_object().unwrap(){altered["entries"][0][key]=value.clone();}
            let mut changed=source.clone();changed["files"][1]["utf8_text"]=serde_json::json!(altered.to_string());assert!(prepare(&changed,&files,&digest).is_err());
        }
        manifest["entries"][0]["path"]=serde_json::json!("../pixel.png");
        let mut changed=source.clone();changed["files"][1]["utf8_text"]=serde_json::json!(manifest.to_string());assert!(prepare(&changed,&files,&digest).is_err());
        #[cfg(not(feature="beta-project-features"))]
        assert!(local_project::project_asset_registry_from_json(&source.to_string()).is_err());
    }
    #[test]
    fn binary_input_actual_capture_distinguishes_equal_content_admissions_and_paths() {
        let (source,files,a)=fixture();let code=source["files"][0]["utf8_text"].as_str().unwrap();let b=capture_for(code);
        assert_eq!(a.content_digest(),b.content_digest());assert_ne!(a.identity(),b.identity());
        let first=prepare(&source,&files,&a).unwrap();let same=prepare(&source,&files,&a).unwrap();
        assert_eq!(first.source_snapshot_identity(),a.identity());
        assert_eq!(first.source_snapshot_identity(),same.source_snapshot_identity());
        assert!(first.matches_source("main.ddn",&a));assert!(!first.matches_source("main.ddn",&b));
        assert!(!first.matches_source("different.ddn",&a));
        let fresh=prepare(&source,&files,&b).unwrap();assert!(fresh.matches_source("main.ddn",&b));
        assert_ne!(first.source_snapshot_identity(),fresh.source_snapshot_identity());
        assert_eq!(first.registry().manifest_sha256(),fresh.registry().manifest_sha256());
    }
    fn static_material() -> super::super::static_png_assets::CheckedStaticPngMaterial {
        use super::super::static_png_assets;
        let (raw, hash)=static_png_assets::material(1).unwrap();
        let v:serde_json::Value=serde_json::from_str(raw).unwrap();
        let owner:[u8;32]=hex::decode(v["owner"].as_str().unwrap()).unwrap().try_into().unwrap();
        let asset:[u8;32]=hex::decode(v["asset"].as_str().unwrap()).unwrap().try_into().unwrap();
        static_png_assets::lookup(&owner,&asset,1,hash,&[]).unwrap()
    }
    #[test]
    fn binary_input_static_png_actual_material_capture_and_bytes_bound_without_new_identity() {
        let (source,files,capture)=fixture();let prepared=prepare(&source,&files,&capture).unwrap();let material=static_material();
        let handle=prepared.registry().resolve_handle("images/pixel.png").unwrap();
        let bound=prepared.bind_static_png(&material,handle,"main.ddn",&capture).unwrap();
        assert_eq!(bound.bytes(),files["images/pixel.png"]);assert_eq!(bound.asset_path(),"images/pixel.png");
        assert_eq!(bound.material().asset(),material.asset());assert_eq!(bound.material().owner(),material.owner());
        assert_eq!(bound.material().version(),1);assert_eq!(bound.source_snapshot_identity(),capture.identity());
        assert!(bound.matches_current_source("main.ddn",&capture.clone()));
        let registry_bytes=prepared.registry().read_verified_asset_bytes(handle,prepared.registry().manifest_sha256()).unwrap();
        assert_eq!(bound.bytes().as_ptr(),registry_bytes.bytes.as_ptr());
    }
    #[test]
    fn binary_input_static_png_equal_content_new_capture_path_and_foreign_handle_refuse() {
        let (source,files,capture)=fixture();let prepared=prepare(&source,&files,&capture).unwrap();let material=static_material();
        let handle=prepared.registry().resolve_handle("images/pixel.png").unwrap();
        let bound=prepared.bind_static_png(&material,handle,"main.ddn",&capture).unwrap();
        let fresh=capture_for(source["files"][0]["utf8_text"].as_str().unwrap());
        assert_eq!(fresh.content_digest(),capture.content_digest());assert!(!bound.matches_current_source("main.ddn",&fresh));
        assert!(prepared.bind_static_png(&material,handle,"main.ddn",&fresh).is_err());
        assert!(prepared.bind_static_png(&material,handle,"other.ddn",&capture).is_err());
        let foreign=ddonirang_core::asset_handle_from_bundle_path("actual-other-registry","images/pixel.png");
        assert!(prepared.bind_static_png(&material,foreign,"main.ddn",&capture).is_err());
        let fresh_prepared=prepare(&source,&files,&fresh).unwrap();
        let fresh_bound=fresh_prepared.bind_static_png(&material,handle,"main.ddn",&fresh).unwrap();
        assert!(fresh_bound.matches_current_source("main.ddn",&fresh));assert!(!fresh_bound.matches_current_source("main.ddn",&capture));
    }
    #[test]
    fn binary_input_static_png_different_integrity_verified_bytes_do_not_select_asset_by_hash() {
        let (mut source,mut files,capture)=fixture();let material=static_material();
        files.get_mut("images/pixel.png").unwrap()[40]^=1;
        let mut manifest:serde_json::Value=serde_json::from_str(source["files"][1]["utf8_text"].as_str().unwrap()).unwrap();
        manifest["entries"][0]["hash"]=serde_json::json!(blake3::hash(&files["images/pixel.png"]).to_hex().to_string());
        source["files"][1]["utf8_text"]=serde_json::json!(manifest.to_string());
        let prepared=prepare(&source,&files,&capture).unwrap();let handle=prepared.registry().resolve_handle("images/pixel.png").unwrap();
        assert!(prepared.bind_static_png(&material,handle,"main.ddn",&capture).is_err());
        // Intentional byte mutation tests ownership/content mismatch. It does
        // not claim a valid new PNG, a new asset version or decoder support.
    }

    fn image_range(a:i64,b:i64) -> ddonirang_lang::runtime::Value {
        use ddonirang_lang::runtime::Value;
        Value::List(vec![Value::Fixed64(ddonirang_core::Fixed64::from_i64(a)),Value::Fixed64(ddonirang_core::Fixed64::from_i64(b))])
    }
    #[test]
    fn static_png_scene_actual_bound_material_and_exact_rectangle_project_losslessly() {
        use ddonirang_lang::runtime::{Value,PlaneScene,ViewValue};
        let (source,files,capture)=fixture();let prepared=prepare(&source,&files,&capture).unwrap();let material=static_material();
        let handle=prepared.registry().resolve_handle("images/pixel.png").unwrap();
        let bound=prepared.bind_static_png(&material,handle,"main.ddn",&capture).unwrap();
        let image=crate::scene_value::construct_static_png_image(&bound,"main.ddn",&capture,&image_range(-2,2),&image_range(0,1)).unwrap();
        assert_eq!(image,image.clone());
        let Value::SceneElement(element)=&image else {panic!("typed image element expected")};
        let projected=crate::scene_value::project_element(element);
        assert_eq!(projected["kind"],"image");assert_eq!(projected["horizontal"],serde_json::json!(["-8589934592","8589934592"]));
        assert_eq!(projected["vertical"],serde_json::json!(["0","4294967296"]));
        assert_eq!(projected["asset"]["owner"],hex::encode(material.owner()));assert_eq!(projected["asset"]["identity"],hex::encode(material.asset()));
        assert_eq!(projected["asset"]["version"],1);assert_eq!(projected["asset"]["byte_size"],files["images/pixel.png"].len());
        assert_eq!(projected["asset"]["byte_sha256"],hex::encode(material.byte_sha256()));assert_eq!(projected["asset"]["resource_handle"],handle.to_hex());
        assert_eq!(projected["source_content_sha256"],hex::encode(capture.content_digest()));
        let f=ddonirang_core::Fixed64::from_i64;
        let view=ViewValue::Scene(PlaneScene {elements:vec![(*element).clone()],horizontal:[f(-2),f(2)],vertical:[f(0),f(1)]});
        let output=crate::scene_value::project_view(&view);assert_eq!(output["elements"][0],projected);
        assert_eq!(bound.bytes(),files["images/pixel.png"]);
    }
    #[test]
    fn static_png_scene_zero_reverse_wrong_coordinate_and_stale_source_refuse() {
        use ddonirang_lang::runtime::Value;
        let (source,files,capture)=fixture();let prepared=prepare(&source,&files,&capture).unwrap();let material=static_material();
        let handle=prepared.registry().resolve_handle("images/pixel.png").unwrap();let bound=prepared.bind_static_png(&material,handle,"main.ddn",&capture).unwrap();
        for (h,v) in [(image_range(0,0),image_range(0,1)),(image_range(2,1),image_range(0,1)),(image_range(0,1),image_range(0,0)),(Value::String("0,1".into()),image_range(0,1))] {
            assert!(crate::scene_value::construct_static_png_image(&bound,"main.ddn",&capture,&h,&v).is_err());
        }
        let fresh=capture_for(source["files"][0]["utf8_text"].as_str().unwrap());
        assert!(crate::scene_value::construct_static_png_image(&bound,"main.ddn",&fresh,&image_range(0,1),&image_range(0,1)).is_err());
        assert!(crate::scene_value::construct_static_png_image(&bound,"other.ddn",&capture,&image_range(0,1),&image_range(0,1)).is_err());
    }

}
