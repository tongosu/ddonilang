//! Exact offline static-PNG physical material. This is not a runtime AssetRef
//! admission, rights grant, source/run/view capability or storage format.
use serde::Deserialize;

const MATERIAL: &str = include_str!("static_png_assets_r1.json");
const MATERIAL_SHA256: &str = "af9411a4f761a4e3ebce9497533e8c1eb15ccb21cb8a3a9c58794ee374f9d8ed";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    material_version: u32,
    kind: String,
    owner: String,
    asset: String,
    asset_version: u64,
    byte_sha256: String,
    byte_size: u64,
    mime: String,
}

/// Private fields and no Deserialize/constructor prevent caller values from
/// becoming checked material. Public source admission is separately gated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedStaticPngMaterial {
    owner: [u8; 32],
    asset: [u8; 32],
    version: u64,
    byte_sha256: [u8; 32],
    byte_size: u64,
}
impl CheckedStaticPngMaterial {
    pub fn owner(&self) -> &[u8; 32] { &self.owner }
    pub fn asset(&self) -> &[u8; 32] { &self.asset }
    pub fn version(&self) -> u64 { self.version }
    pub fn byte_sha256(&self) -> &[u8;32] { &self.byte_sha256 }
    pub fn byte_size(&self) -> u64 { self.byte_size }
    pub fn mime(&self) -> &str { "image/png" }
    pub fn verify_bytes<'a>(&'a self, bytes: &'a [u8]) -> Result<CheckedStaticPngBytes<'a>, String> {
        if bytes.len() as u64 != self.byte_size
            || digest(bytes) != self.byte_sha256
            || !bytes.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]) {
            return Err(fail("등록된 PNG 원자료와 일치하지 않습니다"));
        }
        Ok(CheckedStaticPngBytes { material: self, bytes })
    }
}
pub struct CheckedStaticPngBytes<'a> {
    material: &'a CheckedStaticPngMaterial,
    bytes: &'a [u8],
}
impl<'a> CheckedStaticPngBytes<'a> {
    pub fn material(&self) -> &CheckedStaticPngMaterial { self.material }
    pub fn bytes(&self) -> &'a [u8] { self.bytes }
}
fn fail(detail: &str) -> String { format!("E_STATIC_PNG_MATERIAL: {detail}") }
fn digest(bytes: &[u8]) -> [u8; 32] {
    ddonirang_core::bytes::ByteSequence::new(bytes.to_vec()).content_sha256()
}
fn raw32(text: &str, identity: bool) -> Result<[u8; 32], String> {
    if text.len() != 64 || !text.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return Err(fail("소문자 raw32 표기가 필요합니다"));
    }
    let mut out = [0; 32];
    for (i, pair) in text.as_bytes().chunks_exact(2).enumerate() {
        let digit = |b: u8| if b <= b'9' { b - b'0' } else { b - b'a' + 10 };
        out[i] = digit(pair[0]) * 16 + digit(pair[1]);
    }
    if identity && out == [0; 32] { return Err(fail("영 정체는 허용하지 않습니다")); }
    Ok(out)
}
fn validate_shape(raw: &str, foreign_reserved: &[[u8; 32]]) -> Result<CheckedStaticPngMaterial, String> {
    let r: Record = serde_json::from_str(raw).map_err(|_| fail("등록 자료 구조가 다릅니다"))?;
    if r.material_version != 1 || r.asset_version != 1 || r.kind != "offline_static_png"
        || r.mime != "image/png" || r.byte_size == 0 || r.byte_size > 256 * 1024 {
        return Err(fail("정확 판본·종류·크기가 다릅니다"));
    }
    let owner = raw32(&r.owner, true)?;
    let asset = raw32(&r.asset, true)?;
    if owner == asset || foreign_reserved.contains(&owner) || foreign_reserved.contains(&asset) {
        return Err(fail("책임 owner와 자산 정체가 충돌합니다"));
    }
    Ok(CheckedStaticPngMaterial { owner, asset, version: r.asset_version,
        byte_sha256: raw32(&r.byte_sha256, false)?, byte_size: r.byte_size })
}
fn validate_fixed(raw: &str, hash: &str, foreign_reserved: &[[u8; 32]]) -> Result<CheckedStaticPngMaterial, String> {
    // A rehashed caller record cannot become a registration. Only immutable
    // exact source material is admitted here; no newest/default or migration.
    if raw != MATERIAL || hash != MATERIAL_SHA256 || digest(raw.as_bytes()) != raw32(MATERIAL_SHA256, false)? {
        return Err(fail("고정 등록 원문과 정확 해시가 필요합니다"));
    }
    validate_shape(raw, foreign_reserved)
}
pub fn material(version: u64) -> Option<(&'static str, &'static str)> {
    match version { 1 => Some((MATERIAL, MATERIAL_SHA256)), _ => None }
}
pub fn lookup(owner: &[u8; 32], asset: &[u8; 32], version: u64,
    material_sha256: &str, foreign_reserved: &[[u8; 32]]) -> Result<CheckedStaticPngMaterial, String> {
    let (raw, hash) = material(version).ok_or_else(|| fail("정확 자산 판본이 없습니다"))?;
    if hash != material_sha256 { return Err(fail("등록 해시가 다릅니다")); }
    let checked = validate_fixed(raw, hash, foreign_reserved)?;
    if checked.owner() != owner || checked.asset() != asset { return Err(fail("책임 owner·자산이 다릅니다")); }
    Ok(checked)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> &'static [u8] { include_bytes!("../../tests/fixtures/static_png_asset_2x2.png") }
    fn value() -> serde_json::Value { serde_json::from_str(MATERIAL).unwrap() }
    fn checked() -> CheckedStaticPngMaterial { validate_fixed(MATERIAL, MATERIAL_SHA256, &[]).unwrap() }
    fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b|format!("{b:02x}")).collect() }
    #[test]
    fn static_png_physical_exact_immutable_lookup_and_borrowed_original() {
        let d = checked(); let before = fixture().to_vec();
        let a = lookup(d.owner(), d.asset(), 1, MATERIAL_SHA256, &[]).unwrap();
        let b = lookup(d.owner(), d.asset(), 1, MATERIAL_SHA256, &[]).unwrap();
        assert_eq!(a.asset(), b.asset()); assert_eq!(a.owner(), b.owner()); assert_eq!(a.version(), 1);
        let bytes = a.verify_bytes(fixture()).unwrap();
        assert_eq!(bytes.bytes(), before); assert_eq!(bytes.material().asset(), d.asset());
        assert_eq!(bytes.bytes().as_ptr(), fixture().as_ptr());
        for version in [0, 2, u64::MAX] { assert!(lookup(d.owner(), d.asset(), version, MATERIAL_SHA256, &[]).is_err()); }
        assert!(lookup(&[1;32], d.asset(), 1, MATERIAL_SHA256, &[]).is_err());
        assert!(lookup(d.owner(), &[2;32], 1, MATERIAL_SHA256, &[]).is_err());
        assert!(lookup(d.owner(), d.asset(), 1, "wrong", &[]).is_err());
    }
    #[test]
    fn static_png_physical_zero_collision_closed_shape_and_rehashed_forgery_refuse() {
        let d = checked();
        for id in [*d.owner(), *d.asset()] { assert!(lookup(d.owner(), d.asset(), 1, MATERIAL_SHA256, &[id]).is_err()); }
        for (key, change) in [("owner",serde_json::json!("0".repeat(64))),
            ("asset",serde_json::json!("0".repeat(64))), ("asset",value()["owner"].clone()),
            ("mime",serde_json::json!("image/jpeg")), ("asset_version",serde_json::json!(2)),
            ("material_version",serde_json::json!(2)), ("kind",serde_json::json!("dataset")),
            ("byte_size",serde_json::json!(0)), ("byte_size",serde_json::json!(262145)),
            ("owner",serde_json::json!("a".repeat(63))), ("asset",serde_json::json!("Z".repeat(64))),
            ("unknown",serde_json::json!(true))] {
            let mut v=value(); v[key]=change; let raw=v.to_string();
            assert!(validate_shape(&raw,&[]).is_err(),"{key}");
            assert!(validate_fixed(&raw,&hex(&digest(raw.as_bytes())),&[]).is_err());
        }
        let mut v=value(); v["owner"]=serde_json::json!("12".repeat(32));
        let raw=v.to_string(); assert!(validate_shape(&raw,&[]).is_ok());
        assert!(validate_fixed(&raw,&hex(&digest(raw.as_bytes())),&[]).is_err());
        let duplicate=MATERIAL.replacen('{',"{\"asset_version\":1,",1);
        assert!(validate_shape(&duplicate,&[]).is_err());
        assert!(validate_fixed(&(MATERIAL.to_owned()+" "),MATERIAL_SHA256,&[]).is_err());
    }
    #[test]
    fn static_png_physical_wrong_missing_extra_or_changed_bytes_refuse() {
        let d=checked(); let mut changed=fixture().to_vec(); changed[10]^=1;
        assert!(d.verify_bytes(&changed).is_err()); assert!(d.verify_bytes(&[]).is_err());
        assert!(d.verify_bytes(&fixture()[..fixture().len()-1]).is_err());
        let mut extra=fixture().to_vec();extra.push(0);assert!(d.verify_bytes(&extra).is_err());
        assert_eq!(fixture(),include_bytes!("../../tests/fixtures/static_png_asset_2x2.png"));
    }
}
