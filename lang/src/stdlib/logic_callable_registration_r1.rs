//! B1 physical registration owner, not executable admission or Bool proof.
//! Fixed CSPRNG material is restored exactly; lookup never issues identities.
//! Tool admission must additionally bind the existing checked Bool type owner,
//! actual builtin implementation and source-local argument/role bindings.
use std::collections::BTreeSet;

const MATERIAL: &str = include_str!("logic_callable_registration_r1.json");
const DIGEST: &str = "b99bf0ac333e94f918eddde28f49e27e07063d51bb0e8e7895cfc668f4016d08";
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum LogicEvaluationPolicy { AndShortCircuit, OrShortCircuit, UnaryEager }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum LogicPinRole { Left, Right, Value }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum LogicTypeRequirement { ExistingCheckedBool }
#[derive(Debug,Clone)]
struct Pin { identity:[u8;32], role:LogicPinRole }
/// Private fields prevent caller-selected IDs/policies or deserialized authority.
/// This proves immutable material ownership only, never source/type admission.
#[derive(Debug,Clone)]
pub struct CheckedLogicDefinition {
    owner:[u8;32], definition:[u8;32], pins:Vec<Pin>, policy:LogicEvaluationPolicy,
    name:&'static str, surface:&'static str,
}
impl CheckedLogicDefinition {
    pub fn owner(&self)->&[u8;32]{&self.owner}
    pub fn definition(&self)->&[u8;32]{&self.definition}
    pub fn version(&self)->u32{1}
    pub fn name(&self)->&'static str{self.name}
    pub fn surface(&self)->&'static str{self.surface}
    pub fn evaluation_policy(&self)->LogicEvaluationPolicy{self.policy}
    pub fn type_requirement(&self)->LogicTypeRequirement{LogicTypeRequirement::ExistingCheckedBool}
    pub fn pin(&self,role:LogicPinRole)->Option<&[u8;32]>{self.pins.iter().find(|p|p.role==role).map(|p|&p.identity)}
    /// Declaration-role -> actual source argument index; spelling/order is not authority.
    pub fn argument_slots(&self,definition:&[u8;32],pins:&[[u8;32]])->Result<Vec<usize>,String>{
        if definition!=&self.definition||pins.len()!=self.pins.len(){return Err("E_LOGIC_BINDING_ARITY_OR_OWNER".into());}
        let mut slots=vec![None;self.pins.len()];
        for (actual,pin) in pins.iter().enumerate(){
            let slot=self.pins.iter().position(|p|&p.identity==pin).ok_or("E_LOGIC_PIN_OWNER")?;
            if slots[slot].replace(actual).is_some(){return Err("E_LOGIC_DUPLICATE_PIN".into());}
        }
        slots.into_iter().map(|slot|slot.ok_or_else(||"E_LOGIC_REQUIRED_PIN".into())).collect()
    }
}
pub fn material(version:u32)->Option<(&'static str,&'static str)>{(version==1).then_some((MATERIAL,DIGEST))}
fn raw_id(text:&str)->Result<[u8;32],String>{
    if text.len()!=64||!text.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)){return Err("E_LOGIC_ID_ENCODING".into());}
    let mut out=[0;32];for (i,pair) in text.as_bytes().chunks_exact(2).enumerate(){
        let d=|b:u8|if b<=b'9'{b-b'0'}else{b-b'a'+10};out[i]=16*d(pair[0])+d(pair[1]);
    }
    if out==[0;32]{return Err("E_LOGIC_ZERO_ID".into());}Ok(out)
}
fn hash(raw:&str)->String{ddonirang_core::bytes::ByteSequence::new(raw.as_bytes().to_vec()).content_sha256().iter().map(|b|format!("{b:02x}")).collect()}
fn string<'a>(v:&'a serde_json::Value,key:&str)->Result<&'a str,String>{v.get(key).and_then(|x|x.as_str()).ok_or_else(||"E_LOGIC_MATERIAL_FIELD".into())}
fn object(v:&serde_json::Value,len:usize)->Result<(),String>{if v.as_object().is_some_and(|o|o.len()==len){Ok(())}else{Err("E_LOGIC_MATERIAL_FIELDS".into())}}
fn validate(raw:&str,digest:&str,reserved:&[[u8;32]])->Result<Vec<CheckedLogicDefinition>,String>{
    if raw.len()>16384||hash(raw)!=digest{return Err("E_LOGIC_MATERIAL_HASH".into());}
    let value:serde_json::Value=serde_json::from_str(raw).map_err(|_|"E_LOGIC_MATERIAL_JSON")?;
    // Fixed canonical encoding rejects duplicate keys, alternate encodings and nulls.
    if serde_json::to_string(&value).map_err(|_|"E_LOGIC_MATERIAL_JSON")?!=raw{return Err("E_LOGIC_MATERIAL_CANON".into());}
    object(&value,4)?;
    if value["version"]!=1||string(&value,"library")?!="logic"{return Err("E_LOGIC_MATERIAL_VERSION".into());}
    let owner=raw_id(string(&value,"owner")?)?;let mut used=BTreeSet::new();used.insert(owner);
    let rows=value["definitions"].as_array().ok_or("E_LOGIC_PUBLIC_DENOMINATOR")?;
    if rows.len()!=4{return Err("E_LOGIC_PUBLIC_DENOMINATOR".into());}
    let specs=[("그리고","かつ",LogicEvaluationPolicy::AndShortCircuit,"and-short-circuit"),
        ("또는","または",LogicEvaluationPolicy::OrShortCircuit,"or-short-circuit"),
        ("아님","否定する",LogicEvaluationPolicy::UnaryEager,"unary-eager"),
        ("아니다","否定である",LogicEvaluationPolicy::UnaryEager,"unary-eager")];
    let signatures=super::logic_function_sigs();let mut out=Vec::new();
    if signatures.len()!=4{return Err("E_LOGIC_SOURCE_SIGNATURE_STALE".into());}
    for (index,(row,(name,surface,policy,policy_name))) in rows.iter().zip(specs).enumerate(){
        object(row,6)?;
        if string(row,"name")?!=name||string(row,"surface")?!=surface||string(row,"evaluation")?!=policy_name
            ||string(row,"result_requirement")?!="existing-bool"{return Err("E_LOGIC_IMPLEMENTATION_POLICY".into());}
        let definition=raw_id(string(row,"definition")?)?;if !used.insert(definition){return Err("E_LOGIC_ID_COLLISION".into());}
        let pin_rows=row["pins"].as_array().ok_or("E_LOGIC_PINS")?;
        let expected:Vec<_>=if index<2{vec![("왼쪽","左","left",LogicPinRole::Left),("오른쪽","右","right",LogicPinRole::Right)]}else{vec![("값","値","value",LogicPinRole::Value)]};
        if pin_rows.len()!=expected.len()||signatures[index].name!=name||signatures[index].params!=expected.iter().map(|(k,_,_,_)|*k).collect::<Vec<_>>().as_slice()
            ||signatures[index].ret!="참거짓"{return Err("E_LOGIC_SOURCE_SIGNATURE_STALE".into());}
        let mut pins=Vec::new();
        for (pin,(name,surface,role,kind)) in pin_rows.iter().zip(expected){
            object(pin,5)?;
            if string(pin,"name")?!=name||string(pin,"surface")?!=surface||string(pin,"role")?!=role||string(pin,"type_requirement")?!="existing-bool"{return Err("E_LOGIC_PIN_POLICY".into());}
            let identity=raw_id(string(pin,"identity")?)?;if !used.insert(identity){return Err("E_LOGIC_ID_COLLISION".into());}pins.push(Pin{identity,role:kind});
        }
        out.push(CheckedLogicDefinition{owner,definition,pins,policy,name,surface});
    }
    if used.iter().any(|id|reserved.contains(id)){return Err("E_LOGIC_RESERVED_ID_COLLISION".into());}
    Ok(out)
}
/// Exact version/hash/owner/Definition lookup. No entropy or name-authorized dispatch.
pub fn lookup(owner:&[u8;32],definition:&[u8;32],version:u32,digest:&str,reserved:&[[u8;32]])->Result<CheckedLogicDefinition,String>{
    let (raw,fixed)=material(version).ok_or("E_LOGIC_VERSION")?;if digest!=fixed{return Err("E_LOGIC_MATERIAL_HASH".into());}
    validate(raw,fixed,reserved)?.into_iter().find(|row|row.owner()==owner&&row.definition()==definition).ok_or_else(||"E_LOGIC_OWNER_OR_DEFINITION".into())
}
#[cfg(test)]
mod r31_logic_material_tests {
    use super::*;
    #[test]fn r31_logic_fixed_full_public_and_definition_scoped_pins(){
        let rows=validate(MATERIAL,DIGEST,&[]).unwrap();assert_eq!(rows.len(),4);
        assert_ne!(rows[0].pin(LogicPinRole::Left),rows[1].pin(LogicPinRole::Left));
        let d=&rows[0];let left=*d.pin(LogicPinRole::Left).unwrap();let right=*d.pin(LogicPinRole::Right).unwrap();
        assert_eq!(d.argument_slots(d.definition(),&[right,left]).unwrap(),vec![1,0]);
        assert!(d.argument_slots(d.definition(),&[left,left]).is_err());
        assert!(d.argument_slots(d.definition(),&[left]).is_err());
        assert!(d.argument_slots(d.definition(),&[left,*rows[1].pin(LogicPinRole::Right).unwrap()]).is_err());
        assert!(lookup(d.owner(),d.definition(),2,DIGEST,&[]).is_err());
        assert!(lookup(d.owner(),d.definition(),1,"latest",&[]).is_err());
        assert!(lookup(d.owner(),d.definition(),1,DIGEST,&[left]).is_err());
    }
    #[test]fn r31_logic_zero_duplicate_partial_unknown_and_policy_tampering_refused(){
        for edit in 0..6{
            let mut value:serde_json::Value=serde_json::from_str(MATERIAL).unwrap();
            match edit{0=>value["owner"]="0".repeat(64).into(),1=>value["definitions"][1]["definition"]=value["definitions"][0]["definition"].clone(),
                2=>{value["definitions"].as_array_mut().unwrap().pop();},3=>value["extra"]=true.into(),
                4=>value["definitions"][0]["evaluation"]="eager".into(),_=>value["definitions"][0]["pins"][0]["type_requirement"]="new-bool".into()}
            let raw=serde_json::to_string(&value).unwrap();assert!(validate(&raw,&hash(&raw),&[]).is_err(),"{edit}");
        }
        let raw=MATERIAL.replacen("{","{\"version\":1,",1);assert!(validate(&raw,&hash(&raw),&[]).is_err());
    }
}
