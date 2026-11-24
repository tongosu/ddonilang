//! B1 exact-1 lexical profile via the existing registry/identity owner.
//! A profile is neither a builtin module export nor checked Bool/call authority.
//! Existing profile fields stay closed; empty particle means the adopted
//! explicit-named-pin surface only, never an omitted required argument.
use super::*;
use crate::stdlib::logic_callable_registration_r1::{CheckedLogicDefinition,LogicPinRole as P};

fn content(definitions:&[CheckedLogicDefinition])
    ->Result<(Vec<SuccessorCallableSurface>,Vec<u8>,String),DdlError>{
    if definitions.len()!=4{return Err(successor_error("E_LOGIC_PROFILE_PUBLIC_DENOMINATOR"));}
    let owner=definitions[0].owner();let mut names=std::collections::BTreeSet::new();
    let mut identities=std::collections::BTreeSet::new();let mut surfaces=Vec::new();let mut count=0;
    for definition in definitions {
        if definition.owner()!=owner||definition.version()!=1||!names.insert(definition.name())
            ||!identities.insert(*definition.definition()){
            return Err(successor_error("E_LOGIC_PROFILE_OWNER_OR_VERSION"));
        }
        let pins=[(P::Left,"左"),(P::Right,"右"),(P::Value,"値")].into_iter()
            .filter_map(|(role,surface)|definition.pin(role).map(|id|(id,surface)))
            .map(|(identity,surface)|{
                if !identities.insert(*identity){return Err(successor_error("E_LOGIC_PROFILE_PIN_OWNER"));}
                count+=1;
                Ok(SuccessorPinSurface{identity:identity.iter().map(|b|format!("{b:02x}")).collect(),surface:surface.into(),particle:String::new()})
            }).collect::<Result<Vec<_>,DdlError>>()?;
        surfaces.push(SuccessorCallableSurface{identity:definition.definition().iter().map(|b|format!("{b:02x}")).collect(),surface:definition.surface().into(),pins});
    }
    if names!=["그리고","또는","아님","아니다"].into_iter().collect()||count!=6{
        return Err(successor_error("E_LOGIC_PROFILE_PUBLIC_DENOMINATOR"));
    }
    surfaces.sort_by(|a,b|a.identity.cmp(&b.identity));
    let rows=surfaces.iter().map(|row|serde_json::json!({"definition":row.identity,"surface":row.surface,
        "pins":row.pins.iter().map(|pin|serde_json::json!({"identity":pin.identity,"surface":pin.surface,"particle":pin.particle})).collect::<Vec<_>>() })).collect::<Vec<_>>();
    // Same closed lexical-profile fields as the existing rational path. Bool
    // is the adopted B1 surface requirement; its actual proof stays tool-owned.
    let bytes=serde_json::to_vec(&serde_json::json!({"registration_revision":1,"canonical_name":"日本語",
        "canonical_header":"#!日本語","explicit_aliases":[],"definitions":rows,"type":"真偽","kind":"関数",
        "unicode":"UTF8-exact","syntax":"ASCII","fallback":false,"required_pins_only":true}))
        .map_err(|_|successor_error("E_LOGIC_PROFILE_ENCODING"))?;
    let digest=ddonirang_core::bytes::ByteSequence::new(bytes.clone()).content_sha256().iter().map(|b|format!("{b:02x}")).collect();
    Ok((surfaces,bytes,digest))
}
impl SuccessorRegistryOwner {
    /// The same opaque language-profile issuer/cache/collision owner is used;
    /// immutable builtin IDs are inputs and are never reissued or transformed.
    pub fn issue_logic(&mut self,definitions:&[CheckedLogicDefinition],
        entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>,
    )->Result<RegisteredSuccessorProfile,DdlError>{
        let (surfaces,bytes,digest)=content(definitions)?;
        self.issue_frozen(surfaces,bytes,digest,SuccessorProfileKind::LogicBuiltin,entropy)
    }
    /// Restore only the recorded identity and exact B1 contents. No issue,
    /// entropy, default/latest lookup, first-profile or v2 reinterpretation.
    pub fn restore_logic_recorded_exact(&mut self,identity:&str,version:&str,revision:u32,
        recorded:&str,digest:&str,definitions:&[CheckedLogicDefinition],
    )->Result<RegisteredSuccessorProfile,DdlError>{
        let id=identity.strip_prefix("language-profile:").ok_or_else(||successor_error("E_LOGIC_PROFILE_ID"))?;
        if id.len()!=32||!id.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b))
            ||id.bytes().all(|b|b==b'0')||identity==FIRST_ISSUED_PROFILE.identity||version!="1"||revision!=1||recorded.len()>65536{
            return Err(successor_error("E_LOGIC_PROFILE_EXACT_REQUIRED"));
        }
        let (surfaces,bytes,actual_digest)=content(definitions)?;
        if bytes.as_slice()!=recorded.as_bytes()||actual_digest!=digest{
            return Err(successor_error("E_LOGIC_PROFILE_CONTENT_MISMATCH"));
        }
        if let Some(old)=self.profiles.iter().find(|row|row.identity==identity||row.content==bytes){
            if old.kind==SuccessorProfileKind::LogicBuiltin&&old.identity==identity&&old.content==bytes&&old.sha256==actual_digest{return Ok(old.clone());}
            return Err(successor_error("E_LOGIC_PROFILE_IDENTITY_CONFLICT"));
        }
        if self.profiles.len()>=128{return Err(successor_error("E_LOGIC_PROFILE_LIMIT"));}
        let profile=RegisteredSuccessorProfile{identity:identity.into(),content:bytes,sha256:actual_digest,surfaces,kind:SuccessorProfileKind::LogicBuiltin,version:"1".into(),revision:1};
        self.profiles.push(profile.clone());Ok(profile)
    }
}
