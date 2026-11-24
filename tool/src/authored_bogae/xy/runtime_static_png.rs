//! Compiled assignment for the one issued offline asset. Locators look up
//! explicit opaque material; neither paths nor byte hashes issue identities.
use super::{binary_assets::PreparedBinaryAssets, static_png_assets};
use crate::source_snapshot_admission::SourceCapture;
use ddonirang_core::ResourceHandle;
use ddonirang_lang::runtime::Value;
use std::{collections::BTreeSet,sync::OnceLock};

struct Assignment {
    bundle: &'static str,
    path: &'static str,
    owner: &'static str,
    asset: &'static str,
    version: u64,
    material_hash: &'static str,
}
// Reuses R39 ISSUED_MATERIAL; this is not an arbitrary PNG import registry.
const ASSIGNMENTS: &[Assignment] = &[Assignment {
    bundle: "binary-input-ui-fixture", path: "images/pixel.png",
    owner: "32314d8bfb096d4d31f5dc17330b5aff508b38ca0f06d879396528c94ddc4a6e",
    asset: "3dd33cc47a9ecea11261d9bb3be6f316f7da41c5db20cca0ffc5f7778b0d7da9",
    version: 1,
    material_hash: "af9411a4f761a4e3ebce9497533e8c1eb15ccb21cb8a3a9c58794ee374f9d8ed",
}];
fn fail(message:impl std::fmt::Display)->String {format!("E_AUTHORED_BOGAE: {message}")}
fn raw_id(text:&str)->Result<[u8;32],String> {
    hex::decode(text).map_err(fail)?.try_into().map_err(|_|fail("고정 등록 정체 크기가 다릅니다"))
}
fn known_reserved()->Result<&'static [[u8;32]],String> {
    static IDS:OnceLock<Result<Vec<[u8;32]>,String>>=OnceLock::new();
    IDS.get_or_init(|| {
        let bytes=crate::authored_project::registration::resolved_supply::builtin_manifest_bytes()?;
        if bytes.len()!=10*32 {return Err(fail("기존 builtin 정체 공급 크기가 다릅니다"));}
        let mut ids:BTreeSet<[u8;32]>=bytes.chunks_exact(32).map(|b|b.try_into().expect("exact chunk")).collect();
        fn collect(value:&serde_json::Value,ids:&mut BTreeSet<[u8;32]>)->Result<(),String> {
            match value {
                serde_json::Value::Object(fields)=>for (key,value) in fields {
                    if matches!(key.as_str(),"owner"|"definition"|"identity") {
                        ids.insert(raw_id(value.as_str().ok_or_else(||fail("고정 역할 정체가 문자열이 아닙니다"))?)?);
                    } else {collect(value,ids)?;}
                },
                serde_json::Value::Array(values)=>for value in values {collect(value,ids)?;},
                _=>{},
            }
            Ok(())
        }
        for raw in [include_str!("../../../../lang/src/stdlib/result_table_registration_r1.json"),
            include_str!("../../../../lang/src/stdlib/xy_registration_r1.json"),
            include_str!("../../../../lang/src/stdlib/logic_callable_registration_r1.json")] {
            collect(&serde_json::from_str(raw).map_err(fail)?,&mut ids)?;
        }
        if ids.len()!=31 {return Err(fail("고정 builtin/D1/XY/logic 역할 공급이 달라졌습니다"));}
        Ok(ids.into_iter().collect())
    }).as_ref().map(Vec::as_slice).map_err(Clone::clone)
}

pub(crate) fn construct_candidate(assets:&PreparedBinaryAssets,capture:&SourceCapture,
    handle:ResourceHandle,horizontal:&Value,vertical:&Value)->Result<Value,String> {
    let read=assets.registry().read_verified_asset_bytes(handle,assets.registry().manifest_sha256())?;
    let mut matches=ASSIGNMENTS.iter().filter(|a|a.bundle==read.bundle_id && a.path==read.path);
    let assignment=matches.next().ok_or_else(||fail("해당 자원의 고정 소유자·자산 판본 공급이 없습니다"))?;
    if matches.next().is_some() {return Err(fail("해당 자원에 고정 자산 공급이 중복됐습니다"));}
    let material=static_png_assets::lookup(&raw_id(assignment.owner)?,&raw_id(assignment.asset)?,
        assignment.version,assignment.material_hash,known_reserved()?)?;
    let bound=assets.bind_static_png(&material,handle,assets.source_path(),capture)?;
    crate::scene_value::construct_static_png_image(&bound,assets.source_path(),capture,horizontal,vertical)
}

/// No fixture/test bypass: actual public calls retain this gate until the
/// exact R39 material and R42 code migration have an actual review result.
pub(crate) fn require_independent_physical_review()->Result<(),String> {
    // Author-independent R39 physical review + exact R42 migration only.
    // Review target43fe6ce7/materialaf9411a4; not rights/product/storage approval.
    let (_,hash)=static_png_assets::material(1).ok_or_else(||fail("검토한 정확 물리 판본이 없습니다"))?;
    let code=include_bytes!("../../../../lang/src/runtime/static_png_assets.rs");
    if hash!="af9411a4f761a4e3ebce9497533e8c1eb15ccb21cb8a3a9c58794ee374f9d8ed" || hex::encode(ddonirang_core::bytes::ByteSequence::new(code.to_vec()).content_sha256())!="00666ad71d8697ddc1c268da39ed7852fe2805eabee4a8804820d6f00402a7c0" {
        return Err(fail("정적 그림의 실제 물리자료·소비 코드가 검토 대상과 다릅니다"));
    }
    Ok(())
}
