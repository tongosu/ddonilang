//! R4.1 tail with R4.2 active/inactive correction. Codec/graph validation is
//! distinct from trusted project/module/catalog admission and ID issuance.
use super::{Reader,Writer,Source,Result,ContractFailure};
use std::collections::BTreeSet;

#[derive(Debug)]
pub(crate) struct Slot<'a> {
    pub identity:&'a [u8], pub label:&'a str, pub kind:u8, pub resolved_type:&'a [u8],
}
#[derive(Debug)]
pub(crate) struct Schema<'a> {
    pub module:&'a [u8], pub definition:&'a [u8], pub name_type:&'a [u8],
    pub category:u8, pub label:&'a str, pub slots:Vec<Slot<'a>>,
}
#[derive(Debug)]
pub(crate) struct Declaration {pub source:usize,pub start:u64,pub end:u64,pub schema:usize}
#[derive(Debug)]
pub(crate) struct Tail<'a> {pub schemas:Vec<Schema<'a>>,pub declarations:Vec<Declaration>}

fn identifier(text:&str)->bool {
    use ddonirang_lang::{Lexer,TokenKind};
    let Ok(tokens)=Lexer::new(text).tokenize() else{return false};
    matches!(tokens.as_slice(),[token,eof]
        if matches!(token.kind,TokenKind::Ident(_)) && token.span.start==0
        && token.span.end==text.len() && matches!(eof.kind,TokenKind::Eof))
}
fn resolved_type_dependencies(raw:&[u8],version:u16)->Option<Vec<(u8,[u8;32])>> {
    use ddonirang_core::platform::RecordFieldType as T;
    let Ok(value)=crate::unit_wire::decode_recorded_field_type(raw) else{return None};
    let mut jobs=vec![&value];let mut valid=true;let mut dependencies=Vec::new();
    while let Some(value)=jobs.pop() {
        match value {
            T::RegisteredAtomV1=>{valid=false;break;},
            T::ResolvedNominalV1{category,name_type_identity}=>dependencies.push((*category,*name_type_identity)),
            T::Named{name}=>if !matches!(name.as_str(),"바른수"|"셈수"|"참거짓"|"글")&&!(version==3&&name=="나눔수") {valid=false;break;},
            T::Tuple{items}=>jobs.extend(items),
            T::Record{fields}=>jobs.extend(fields.values()),
            T::Applied{name,args}=>{
                if !matches!((name.as_str(),args.len()),("차림"|"모음",1)|("짝맞춤",2)) {valid=false;break;}
                jobs.extend(args);
            }
        }
    }
    drop(jobs);value.drop_stack_safe();valid.then_some(dependencies)
}

impl<'a> Tail<'a> {
    // F5-GA[REG_NOMINAL_TAKE]: The shared Reader rejects short take(1) before category/kind index0; the returned slice is exactly one byte.
    pub(super) fn parse(input:&mut Reader<'a>,sources:&[Source<'a>],version:u16)->Result<Self> {
        let count=input.count(65536,106)?;
        let mut schemas:Vec<Schema<'a>>=Vec::new();schemas.try_reserve_exact(count).map_err(|_|ContractFailure::SchemaInvalid)?;
        let mut identities=BTreeSet::new();let mut labels=BTreeSet::new();let mut slot_total=0;
        for _ in 0..count {
            let module=input.take(32)?;let definition=input.take(32)?;let name_type=input.take(32)?;
            let category=input.take(1)?[0];let label=input.text()?;
            if !matches!(category,1|2) || !identifier(label) || !labels.insert((module,label))
                || !identities.insert(definition) || !identities.insert(name_type)
                || schemas.last().is_some_and(|old|(old.module,old.definition)>=(module,definition)) {
                return Err(ContractFailure::SchemaInvalid);
            }
            let count=input.count(65536-slot_total,43)?;slot_total+=count;
            let mut slots:Vec<Slot<'a>>=Vec::new();slots.try_reserve_exact(count).map_err(|_|ContractFailure::SchemaInvalid)?;
            let mut labels=BTreeSet::new();
            for _ in 0..count {
                let identity=input.take(32)?;let label=input.text()?;
                let kind=input.take(1)?[0];let resolved_type=input.blob()?;
                if !identities.insert(identity) || !identifier(label) || !labels.insert(label)
                    || slots.last().is_some_and(|old|old.identity>=identity)
                    || !match kind {
                        0=>resolved_type_dependencies(resolved_type,version).is_some_and(|ids|ids.is_empty()),
                        1=>resolved_type.len()==32,
                        2=>version==3&&resolved_type.first()!=Some(&4)&&resolved_type_dependencies(resolved_type,version).is_some_and(|ids|!ids.is_empty()),
                        _=>false} {
                    return Err(ContractFailure::SchemaInvalid);
                }
                slots.push(Slot{identity,label,kind,resolved_type});
            }
            schemas.push(Schema{module,definition,name_type,category,label,slots});
        }
        let count=input.count(65536,24)?;
        let mut declarations:Vec<Declaration>=Vec::new();declarations.try_reserve_exact(count).map_err(|_|ContractFailure::SchemaInvalid)?;
        let mut active=BTreeSet::new();
        for _ in 0..count {
            let source=input.u32()? as usize;let start=input.u64()?;let end=input.u64()?;let schema=input.u32()? as usize;
            // F5-GA[REG_NOMINAL_SOURCE]: Left-to-right short-circuit rejects source>=len before sources[source]; the borrowed slice is not resized.
            if source>=sources.len() || schema>=schemas.len() || !active.insert(schema)
                || start>=end || end>sources[source].length
                || declarations.last().is_some_and(|old|(old.source,old.start,old.end)>=(source,start,end)
                    || old.source==source && old.end>start) {return Err(ContractFailure::SchemaInvalid);}
            declarations.push(Declaration{source,start,end,schema});
        }
        for (index,schema) in schemas.iter().enumerate() {
            for slot in &schema.slots {
                if slot.kind==1 {
                    let Some(target)=schemas.iter().position(|row|row.name_type==slot.resolved_type) else{return Err(ContractFailure::SchemaInvalid)};
                    // R4.2: an inactive retained row does not require its
                    // historical dependency to be currently declared.
                    if active.contains(&index) && !active.contains(&target) {return Err(ContractFailure::NotAdmitted);}
                }else if slot.kind==2 {
                    let dependencies=resolved_type_dependencies(slot.resolved_type,version).ok_or(ContractFailure::SchemaInvalid)?;
                    for (category,identity) in dependencies {
                        let Some(target)=schemas.iter().position(|row|row.name_type==identity&&row.category==category) else{return Err(ContractFailure::SchemaInvalid)};
                        if active.contains(&index)&&!active.contains(&target){return Err(ContractFailure::NotAdmitted);}
                    }
                }
            }
        }
        Ok(Self{schemas,declarations})
    }
    pub(super) fn encode(&self,out:&mut Writer)->Result<()> {
        out.count(self.schemas.len())?;
        for schema in &self.schemas {
            out.digest(schema.module)?;out.digest(schema.definition)?;out.digest(schema.name_type)?;
            out.raw(&[schema.category])?;out.blob(schema.label.as_bytes())?;out.count(schema.slots.len())?;
            for slot in &schema.slots {out.digest(slot.identity)?;out.blob(slot.label.as_bytes())?;
                out.raw(&[slot.kind])?;out.blob(slot.resolved_type)?;}
        }
        out.count(self.declarations.len())?;
        for declaration in &self.declarations {out.count(declaration.source)?;out.raw(&declaration.start.to_be_bytes())?;
            out.raw(&declaration.end.to_be_bytes())?;out.count(declaration.schema)?;}
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registered_atom_contract::source_binding::Candidate;
    const VECTOR:&str="64646e2e766330342e61746f6d2d736f757263652d62696e64696e672e763100000200000001000000086d61696e2e64646e00000020024cccbd51f513458d2ffa01bfb4b184ff4102582ac3e56b0a21e2cee687a3d50000000000000075b3611d454d211f7186242fabe0af59f6eed7832581e783d1a262cb9c6e981a800000000000000000000000031111111111111111111111111111111111111111111111111111111111111111212121212121212121212121212121212121212121212121212121212121212122222222222222222222222222222222222222222222222222222222222222220200000003eab09100000001232323232323232323232323232323232323232323232323232323232323232300000006ec9db4eba684000000000c000300000000000000eab8801111111111111111111111111111111111111111111111111111111111111111313131313131313131313131313131313131313131313131313131313131313132323232323232323232323232323232323232323232323232323232323232320200000003ec9d8400000001333333333333333333333333333333333333333333333333333333333333333300000006ec9db4eba684000000000c000300000000000000eab8801111111111111111111111111111111111111111111111111111111111111111414141414141414141414141414141414141414141414141414141414141414142424242424242424242424242424242424242424242424242424242424242420100000006ebb391ec82ac00000002434343434343434343434343434343434343434343434343434343434343434300000006ec99bcecaabd01000000202222222222222222222222222222222222222222222222222222222222222222444444444444444444444444444444444444444444444444444444444444444400000009ec98a4eba5b8ecaabd0100000020222222222222222222222222222222222222222222222222222222222222222200000003000000000000000000000000000000000000002100000000000000000000000000000022000000000000004300000001000000000000000000000044000000000000007400000002";
    fn valid_label_codec_fixture()->Vec<u8> {
        // Explicit auxiliary codec fixture, NOT the reviewed positive vector
        // or a rewritten executable source. The supplied `을` is a reserved
        // particle and remains a reported material counterexample.
        let original=hex::decode(VECTOR).unwrap();
        assert!(Candidate::parse(&original).is_err());
        let pattern=hex::decode("0200000003ec9d8400000001").unwrap();
        let offset=original.windows(pattern.len()).position(|part|part==pattern).unwrap();
        let mut result=original[..offset].to_vec();
        result.extend(hex::decode("0200000006ec9d84eab09200000001").unwrap());
        result.extend_from_slice(&original[offset+pattern.len()..]);result
    }
    #[test]
    fn vc04_nominal_v2_codec_roundtrip_preserves_distinct_type_and_slot_ids() {
        let raw=valid_label_codec_fixture();assert_eq!(raw.len(),823);
        let candidate=Candidate::parse(&raw).unwrap();assert_eq!(candidate.encode().unwrap(),raw);
        let tail=candidate.nominal.as_ref().unwrap();
        assert_ne!(tail.schemas[0].name_type,tail.schemas[1].name_type);
        assert_ne!(tail.schemas[2].slots[0].identity,tail.schemas[2].slots[1].identity);
        assert_eq!(tail.schemas[2].slots[0].resolved_type,tail.schemas[2].slots[1].resolved_type);
        for n in 0..raw.len() {assert!(Candidate::parse(&raw[..n]).is_err(),"prefix {n}");}
        let mut duplicate=Candidate::parse(&raw).unwrap();
        let tail=duplicate.nominal.as_mut().unwrap();tail.schemas[1].name_type=tail.schemas[0].name_type;
        assert!(duplicate.encode().is_err());
    }
    #[test]
    fn vc04_nominal_r42_graph_keeps_inactive_dependencies_without_activating_them() {
        // Real current source/capture/binding ranges change for each case.
        // This tests the codec graph, not trusted prior-catalog admission.
        let raw=valid_label_codec_fixture();
        for (source,accepted) in [
            ("을값:이름씨 := { 이름:글. }.",true),
            ("을값:이름씨 := { 이름:글. }. 병사:임자 := { 왼쪽:갑. 오른쪽:갑. }.",false),
            ("갑:이름씨 := { 이름:글. }. 을값:이름씨 := { 이름:글. }. 병사:임자 := { 왼쪽:갑. 오른쪽:갑. }.",true),
        ] {
            let capture=crate::source_snapshot_admission::capture_declared_v25(source.as_bytes()).unwrap();
            let parsed=ddonirang_lang::parse_v25_with_mode(source,"main.ddn",ddonirang_lang::ParseMode::Strict).unwrap();
            let mut candidate=Candidate::parse(&raw).unwrap();
            candidate.sources=vec![Source{path:"main.ddn",snapshot:capture.identity(),length:source.len() as u64,sha:capture.content_digest()}];
            let tail=candidate.nominal.as_mut().unwrap();tail.declarations.clear();
            for ddonirang_lang::TopLevelItem::SeedDef(seed) in &parsed.items {
                let schema=tail.schemas.iter().position(|row|row.label==seed.canonical_name).unwrap();
                tail.declarations.push(Declaration{source:0,start:seed.span.start as u64,end:seed.span.end as u64,schema});
            }
            assert_eq!(candidate.encode().is_ok(),accepted,"{source}");
        }
    }
}
