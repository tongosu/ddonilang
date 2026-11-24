//! Immutable XY physical material only. Not source/axis admission or a proof
//! of joint samples. No runtime issuance or deserializable capability.
use std::collections::BTreeSet;

const MATERIAL: &str = include_str!("xy_registration_r1.json");
const DIGEST: &str = "d5bc59976160cad7e6d51a0d132d0441247adf00c439eda0de9ad83d3d4ae827";
const PARENT: &str = include_str!("result_table_registration_r1.json");
const PARENT_DIGEST: &str = "0168cdc420b86e65ba4cfb62ad2e0cb185410b6b870143d5287bf8dbc19c2673";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XyMode { Scatter, Polyline }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxisRole { X, Y }
#[derive(Clone, Debug)]
pub struct CheckedXyDescriptor {
    owner: [u8;32], definition: [u8;32], pins: [[u8;32];2], mode: XyMode,
}
impl CheckedXyDescriptor {
    pub fn owner(&self) -> &[u8;32] { &self.owner }
    pub fn definition(&self) -> &[u8;32] { &self.definition }
    pub fn version(&self) -> u32 { 1 }
    pub fn mode(&self) -> XyMode { self.mode }
    pub fn pin(&self, role:AxisRole) -> &[u8;32] {
        &self.pins[match role { AxisRole::X=>0, AxisRole::Y=>1 }]
    }
    pub fn check_owned_pin(&self, definition:&[u8;32], pin:&[u8;32], role:AxisRole)->bool {
        definition==self.definition() && pin==self.pin(role)
    }
    /// Required singular roles. A single observation may explicitly serve both
    /// axes; the pin roles remain distinct. No result-table dedup rule here.
    pub fn argument_slots(&self, roles:&[AxisRole])->Result<[usize;2],String> {
        let mut slots=[None,None];
        for (i,role) in roles.iter().enumerate() {
            let slot=match role { AxisRole::X=>0, AxisRole::Y=>1 };
            if slots[slot].replace(i).is_some() {return Err("같은 축 받는자리를 두 번 지정했습니다".into());}
        }
        match slots { [Some(x),Some(y)]=>Ok([x,y]), _=>Err("가로와 세로 축이 각각 한 번 필요합니다".into()) }
    }
}
fn digest(raw:&str)->String {
    ddonirang_core::bytes::ByteSequence::new(raw.as_bytes().to_vec()).content_sha256()
        .iter().map(|b|format!("{b:02x}")).collect()
}
fn raw_id(text:&str)->Result<[u8;32],String> {
    if text.len()!=64 || !text.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)) {
        return Err("등록 정체는 소문자 raw32여야 합니다".into());
    }
    let mut id=[0;32];
    for (i,pair) in text.as_bytes().chunks_exact(2).enumerate() {
        let nibble=|b:u8|if b<=b'9'{b-b'0'}else{b-b'a'+10};
        id[i]=nibble(pair[0])*16+nibble(pair[1]);
    }
    if id==[0;32] {return Err("영 정체는 사용할 수 없습니다".into());}
    Ok(id)
}
fn validate(raw:&str, hash:&str, reserved:&[[u8;32]])->Result<Vec<CheckedXyDescriptor>,String> {
    let fail=||"XY 등록의 소유·판본·핀·무결성이 맞지 않습니다".to_string();
    if digest(raw)!=hash || digest(PARENT)!=PARENT_DIGEST {return Err(fail());}
    let parent:serde_json::Value=serde_json::from_str(PARENT).map_err(|_|fail())?;
    let owner=raw_id(parent["owner"].as_str().ok_or_else(fail)?)?;
    let value:serde_json::Value=serde_json::from_str(raw).map_err(|_|fail())?;
    if value.as_object().ok_or_else(fail)?.len()!=3 || value["version"]!=1
        || value["owner"]!=parent["owner"] {return Err(fail());}
    let definitions=value["definitions"].as_array().ok_or_else(fail)?;
    if definitions.len()!=2 {return Err(fail());}
    // Reusing the exact standard-view owner is intentional. New definition/pin
    // identities must not reuse any old role, each other or external reserves.
    let mut used=BTreeSet::from([owner]);
    used.insert(raw_id(parent["definition"].as_str().ok_or_else(fail)?)?);
    for pin in parent["pins"].as_array().ok_or_else(fail)? {
        used.insert(raw_id(pin["identity"].as_str().ok_or_else(fail)?)?);
    }
    let mut checked=Vec::new();
    for (row,(name,mode,kind)) in definitions.iter().zip([
        ("표준보기.흩어진점","scatter",XyMode::Scatter),
        ("표준보기.좌표선","polyline",XyMode::Polyline),
    ]) {
        if row.as_object().ok_or_else(fail)?.len()!=5 || row["callable"]!=name
            || row["mode"]!=mode || row["result"]!="existing_view_value" {return Err(fail());}
        let definition=raw_id(row["definition"].as_str().ok_or_else(fail)?)?;
        if reserved.contains(&definition)||!used.insert(definition) {return Err(fail());}
        let pins=row["pins"].as_array().ok_or_else(fail)?;
        if pins.len()!=2 {return Err(fail());}
        let mut owned=[[0;32];2];
        for (i,(pin,(role,spelling))) in pins.iter().zip([("x","가로"),("y","세로")]).enumerate() {
            if pin.as_object().ok_or_else(fail)?.len()!=6 || pin["definition"]!=row["definition"]
                || pin["role"]!=role || pin["spelling"]!=spelling || pin["optional"]!=false
                || pin["expectation"]!="existing_checked_direct_root_numeric_observation" {return Err(fail());}
            owned[i]=raw_id(pin["identity"].as_str().ok_or_else(fail)?)?;
            if reserved.contains(&owned[i])||!used.insert(owned[i]) {return Err(fail());}
        }
        checked.push(CheckedXyDescriptor {owner,definition,pins:owned,mode:kind});
    }
    Ok(checked)
}
/// Inspection bytes are not a capability. Exact v1 only; no latest/fallback.
pub fn material(version:u32)->Option<(&'static str,&'static str)> {
    match version {1=>Some((MATERIAL,DIGEST)),_=>None}
}
pub fn lookup(owner:&[u8;32], definition:&[u8;32], version:u32,
    hash:&str, reserved:&[[u8;32]])->Result<CheckedXyDescriptor,String> {
    let (raw,fixed)=material(version).ok_or("XY 등록 판본이 없습니다")?;
    if hash!=fixed {return Err("정확한 XY 등록 해시가 필요합니다".into());}
    validate(raw,fixed,reserved)?.into_iter().find(|d|d.owner()==owner && d.definition()==definition)
        .ok_or_else(||"XY 등록 owner/정의가 다릅니다".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_material_owner_pins_and_version() {
        let all=validate(MATERIAL,DIGEST,&[]).unwrap();
        assert_eq!(all.len(),2);
        for (i,d) in all.iter().enumerate() {
            assert_eq!(d.mode(),if i==0{XyMode::Scatter}else{XyMode::Polyline});
            assert_eq!(d.version(),1);
            assert!(lookup(d.owner(),d.definition(),1,DIGEST,&[]).is_ok());
            assert!(d.check_owned_pin(d.definition(),d.pin(AxisRole::X),AxisRole::X));
            assert!(!d.check_owned_pin(d.definition(),d.pin(AxisRole::X),AxisRole::Y));
            assert!(!d.check_owned_pin(all[1-i].definition(),d.pin(AxisRole::X),AxisRole::X));
            assert!(lookup(d.owner(),d.definition(),2,DIGEST,&[]).is_err());
            assert!(lookup(&[1;32],d.definition(),1,DIGEST,&[]).is_err());
            assert!(lookup(d.owner(),d.definition(),1,"wrong",&[]).is_err());
            assert!(lookup(d.owner(),d.definition(),1,DIGEST,&[*d.pin(AxisRole::X)]).is_err());
        }
    }
    #[test]
    fn reject_zero_duplicates_owner_scope_and_changed_contract() {
        let original:serde_json::Value=serde_json::from_str(MATERIAL).unwrap();
        let mut changes=Vec::new();
        let mut v=original.clone();v["definitions"][0]["definition"]=serde_json::json!("0".repeat(64));changes.push(v);
        let mut v=original.clone();v["definitions"][1]["definition"]=v["definitions"][0]["definition"].clone();changes.push(v);
        let mut v=original.clone();v["definitions"][1]["pins"][0]["identity"]=v["definitions"][0]["pins"][0]["identity"].clone();changes.push(v);
        let mut v=original.clone();v["definitions"][0]["pins"][0]["definition"]=v["definitions"][1]["definition"].clone();changes.push(v);
        let mut v=original.clone();v["definitions"][0]["pins"][0]["optional"]=serde_json::json!(true);changes.push(v);
        let mut v=original.clone();v["definitions"][0]["pins"][0]["role"]=serde_json::json!("time");changes.push(v);
        let mut v=original.clone();v["owner"]=serde_json::json!("12".repeat(32));changes.push(v);
        let mut v=original.clone();v["extra"]=serde_json::json!(true);changes.push(v);
        for v in changes {let raw=v.to_string();assert!(validate(&raw,&digest(&raw),&[]).is_err());}
        assert!(validate(MATERIAL,"wrong",&[]).is_err());
    }
    #[test]
    fn required_singular_axes_do_not_forbid_explicit_same_observation() {
        let d=validate(MATERIAL,DIGEST,&[]).unwrap().remove(0);
        assert_eq!(d.argument_slots(&[AxisRole::Y,AxisRole::X]).unwrap(),[1,0]);
        assert!(d.argument_slots(&[AxisRole::X]).is_err());
        assert!(d.argument_slots(&[AxisRole::X,AxisRole::X,AxisRole::Y]).is_err());
        assert!(d.argument_slots(&[]).is_err());
    }
}

/// Cross-crate checked consumer of the existing immutable physical pack.
/// No raw constructor, source admission or new identity is supplied here.
pub fn checked_descriptor(mode:XyMode)->Result<CheckedXyDescriptor,String> {
    validate(MATERIAL,DIGEST,&[])?.into_iter().find(|row|row.mode()==mode)
        .ok_or_else(||"XY 등록 정의가 없습니다".into())
}
