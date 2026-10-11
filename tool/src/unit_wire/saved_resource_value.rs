//! Shared decoder of existing core ResourceValue canonical bytes. Table BE
//! framing does not change this owner's LE scalars/counts or Tensor framing.
use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1 as Failure;
use ddonirang_core::{Fixed64, ResourceHandle, UnitDim, UnitValue};
use ddonirang_core::platform::{ResourceMapEntry,ResourceValue};
use std::collections::BTreeMap;

const MAX_BYTES: usize = 2_000_000;
// A3-P13 provisional safety profile, not a language nesting restriction.
const MAX_DEPTH: usize = 128;
const MAX_MATERIALIZED: usize = 16_000_000;

pub(super) struct ValueDecodeBudget { remaining:usize }
impl ValueDecodeBudget {
    pub(super) fn new()->Self {Self{remaining:MAX_MATERIALIZED}}
    fn charge(&mut self,n:usize)->Result<(),Failure> {
        self.remaining=self.remaining.checked_sub(n).ok_or(Failure::LimitExceeded)?;Ok(())
    }
    fn slots(&mut self,n:usize)->Result<(),Failure> {
        self.charge(n.checked_mul(std::mem::size_of::<ResourceValue>()).ok_or(Failure::LimitExceeded)?)
    }
}

struct Reader<'a> { bytes: &'a [u8], at: usize }
impl<'a> Reader<'a> {
    fn take(&mut self,n:usize)->Result<&'a [u8],Failure> {
        let end=self.at.checked_add(n).ok_or(Failure::LimitExceeded)?;
        let bytes=self.bytes.get(self.at..end).ok_or(Failure::InvalidInput)?;
        self.at=end;Ok(bytes)
    }
    fn le64(&mut self)->Result<u64,Failure> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().map_err(|_|Failure::InvalidInput)?))
    }
    fn text(&mut self)->Result<&'a str,Failure> {
        let length=usize::try_from(self.le64()?).map_err(|_|Failure::LimitExceeded)?;
        std::str::from_utf8(self.take(length)?).map_err(|_|Failure::InvalidInput)
    }
    fn count(&mut self)->Result<usize,Failure> {
        let n=usize::try_from(self.le64()?).map_err(|_|Failure::LimitExceeded)?;
        if n>self.bytes.len().saturating_sub(self.at) {return Err(Failure::InvalidInput);}
        Ok(n)
    }
    fn value(&mut self,depth:usize,budget:&mut ValueDecodeBudget)->Result<ResourceValue,Failure> {
        if depth>MAX_DEPTH {return Err(Failure::LimitExceeded);}
        budget.slots(1)?;
        let start=self.at;
        Ok(match self.take(1)?[0] {
            0=>ResourceValue::None,
            1=>ResourceValue::Bool(match self.take(1)?[0] {0=>false,1=>true,_=>return Err(Failure::InvalidInput)}),
            2=>ResourceValue::Fixed64(Fixed64::from_raw_i64(self.le64()? as i64)),
            3=>{
                let raw=self.le64()? as i64;
                let dim=dimension(self.text()?)?;
                ResourceValue::Unit(UnitValue{value:Fixed64::from_raw_i64(raw),dim})
            },
            4=>{let text=self.text()?;budget.charge(text.len())?;ResourceValue::String(text.to_owned())},
            5=>ResourceValue::ResourceHandle(ResourceHandle::from_raw(self.le64()?)),
            6=>{
                let n=self.count()?;let mut values=Vec::new();
                budget.slots(n)?;
                values.try_reserve_exact(n).map_err(|_|Failure::LimitExceeded)?;
                for _ in 0..n {values.push(self.value(depth+1,budget)?);}
                ResourceValue::List(values)
            },
            7=>{
                let n=self.count()?;let mut values=BTreeMap::new();let mut previous:Option<String>=None;
                for _ in 0..n {
                    let value=self.value(depth+1,budget)?;let key=value.canon_key();
                    budget.charge(key.len().checked_mul(2).and_then(|n|n.checked_add(64)).ok_or(Failure::LimitExceeded)?)?;
                    if previous.as_ref().is_some_and(|p|p>=&key) {return Err(Failure::InvalidInput);}
                    previous=Some(key.clone());values.insert(key,value);
                }
                ResourceValue::Set(values)
            },
            8=>{
                let n=self.count()?;
                if n>self.bytes.len().saturating_sub(self.at)/2 {return Err(Failure::InvalidInput);}
                let mut entries=BTreeMap::new();let mut previous:Option<String>=None;
                for _ in 0..n {
                    let key=self.value(depth+1,budget)?;let index=key.canon_key();
                    budget.charge(index.len().checked_mul(2).and_then(|n|n.checked_add(64)).ok_or(Failure::LimitExceeded)?)?;
                    if previous.as_ref().is_some_and(|p|p>=&index) {return Err(Failure::InvalidInput);}
                    let value=self.value(depth+1,budget)?;
                    previous=Some(index.clone());entries.insert(index,ResourceMapEntry{key,value});
                }
                ResourceValue::Map(entries)
            },
            9=>{
                let capacity=usize::try_from(self.le64()?).map_err(|_|Failure::LimitExceeded)?;
                let n=self.count()?;
                if n>capacity {return Err(Failure::InvalidInput);}
                // A3-P13 bounds materialized empty ring slots too, not just wire size.
                if capacity>MAX_BYTES/std::mem::size_of::<ResourceValue>() {return Err(Failure::LimitExceeded);}
                budget.slots(capacity)?;
                let mut buffer=Vec::new();buffer.try_reserve_exact(capacity).map_err(|_|Failure::LimitExceeded)?;
                for _ in 0..n {buffer.push(self.value(depth+1,budget)?);}
                buffer.resize(capacity,ResourceValue::None);
                ResourceValue::stream_v1_from_ring_buffer(capacity as u64,n.saturating_sub(1) as u64,n as u64,buffer)
            },
            0x0e=>{
                // Determine exactly seven FIELD32 members, then delegate every
                // profile/type/shape/element check to the existing Tensor owner.
                const PROFILE:&[u8]=b"DDN-BASIC-TENSOR-VALUE-V1\0";
                if self.take(PROFILE.len())?!=PROFILE || self.take(3)?!=[0,1,7] {return Err(Failure::InvalidInput);}
                for tag in 1..=7 {
                    if self.take(1)?!=[tag] {return Err(Failure::InvalidInput);}
                    let n=u32::from_be_bytes(self.take(4)?.try_into().map_err(|_|Failure::InvalidInput)?) as usize;
                    self.take(n)?;
                }
                let carrier=&self.bytes[start+1..self.at];
                budget.charge(carrier.len().checked_mul(4).ok_or(Failure::LimitExceeded)?)?;
                super::tensor_from_carrier(carrier).map_err(|_|Failure::InvalidInput)?;
                ResourceValue::BasicTensorV1(carrier.to_vec())
            },
            // Record/reference/Bytes/atom require their exact typed owners.
            // Never guess them from a Map/String or grant custody here.
            _=>return Err(Failure::InvalidInput),
        })
    }
}

fn dimension(text:&str)->Result<UnitDim,Failure> {
    let mut exponents=[0i8;UnitDim::AXIS_COUNT];
    let mut halves=text.split('/');let numerator=halves.next().ok_or(Failure::InvalidInput)?;
    let denominator=halves.next();if halves.next().is_some(){return Err(Failure::InvalidInput);}
    for (part,sign) in [(Some(numerator),1i16),(denominator,-1i16)] {
        let Some(part)=part else{continue};
        if part=="1"&&sign==1 {continue;}
        for factor in part.split('*') {
            let mut pieces=factor.split('^');let symbol=pieces.next().ok_or(Failure::InvalidInput)?;
            let power=match pieces.next(){None=>1,Some(n)=>n.parse::<i16>().map_err(|_|Failure::InvalidInput)?};
            if pieces.next().is_some()||!(1..=128).contains(&power){return Err(Failure::InvalidInput);}
            let axis=match symbol{"m"=>0,"s"=>1,"kg"=>2,"rad"=>3,"px"=>4,"K"=>5,"KRW"=>6,"USD"=>7,"mol"=>8,_=>return Err(Failure::InvalidInput)};
            if exponents[axis]!=0{return Err(Failure::InvalidInput);}
            exponents[axis]=i8::try_from(power*sign).map_err(|_|Failure::InvalidInput)?;
        }
    }
    let dim=UnitDim::from_exponents(exponents);
    if dim.format()!=text{return Err(Failure::InvalidInput);}
    Ok(dim)
}

pub(crate) fn decode_saved_resource_value(bytes:&[u8])->Result<ResourceValue,Failure> {
    decode_saved_resource_value_with_budget(bytes,&mut ValueDecodeBudget::new())
}

pub(super) fn decode_saved_resource_value_with_budget(bytes:&[u8],budget:&mut ValueDecodeBudget)->Result<ResourceValue,Failure> {
    if bytes.len()>MAX_BYTES{return Err(Failure::LimitExceeded);}
    let mut reader=Reader{bytes,at:0};let value=reader.value(0,budget)?;
    if reader.at!=bytes.len(){return Err(Failure::InvalidInput);}
    // Re-encode through the existing authority; no alternate canonical writer.
    let mut canonical=Vec::new();value.encode_canon(&mut canonical);
    if canonical!=bytes{return Err(Failure::InvalidInput);}
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rd670_unit_minimum_exponents_roundtrip_without_panic_or_wrapping() {
        for axis in 0..UnitDim::AXIS_COUNT {
            for exponent in [i8::MIN,i8::MAX] {
                let mut exponents=[0;UnitDim::AXIS_COUNT];exponents[axis]=exponent;
                let dim=UnitDim::from_exponents(exponents);
                let value=ResourceValue::Unit(UnitValue{value:Fixed64::from_raw_i64(i64::MIN),dim});
                let mut bytes=Vec::new();value.encode_canon(&mut bytes);
                assert_eq!(decode_saved_resource_value(&bytes).unwrap(),value);
            }
        }
        assert!(dimension("1/m^129").is_err());
        assert!(dimension("m^128").is_err());
    }
    #[test]
    fn rd670_actual_resource_values_roundtrip_without_scalar_endian_drift() {
        let unit=ResourceValue::Unit(UnitValue{value:Fixed64::from_raw_i64(-97),dim:UnitDim::from_exponents([1,-2,1,0,0,0,0,0,0])});
        let tensor=super::super::tensor_frame::encode_typed_tensor(&ddonirang_lang::ast::TypeRef::Named("셈수".into()),&[2],&[Fixed64::ZERO,Fixed64::ONE]).unwrap().into_runtime().unwrap();
        let values=vec![ResourceValue::None,ResourceValue::Bool(true),ResourceValue::Bool(false),
            ResourceValue::Fixed64(Fixed64::from_raw_i64(i64::MIN)),unit,
            ResourceValue::String("값\0\n한글".into()),ResourceValue::ResourceHandle(ResourceHandle::from_raw(u64::MAX)),
            ResourceValue::set_from_values(vec![ResourceValue::String("나".into()),ResourceValue::String("가".into())]),
            ResourceValue::map_from_entries(vec![ResourceMapEntry{key:ResourceValue::String("키".into()),value:ResourceValue::Bool(true)}]),
            ResourceValue::stream_v1_from_ring_buffer(3,2,2,vec![ResourceValue::None,ResourceValue::Bool(true),ResourceValue::Bool(false)]),
            super::super::tensor_to_resource(&tensor).unwrap()];
        for value in values {
            let mut bytes=Vec::new();value.encode_canon(&mut bytes);
            let decoded=decode_saved_resource_value(&bytes).unwrap();let mut again=Vec::new();decoded.encode_canon(&mut again);assert_eq!(again,bytes);
            for n in 0..bytes.len(){assert!(decode_saved_resource_value(&bytes[..n]).is_err());}
            bytes.push(0);assert!(decode_saved_resource_value(&bytes).is_err());
        }
    }
    #[test]
    fn rd670_nested_values_and_noncanonical_bodies_fail_closed() {
        let nested=ResourceValue::List(vec![ResourceValue::List(vec![ResourceValue::Fixed64(Fixed64::from_raw_i64(1))])]);
        let mut bytes=Vec::new();nested.encode_canon(&mut bytes);assert_eq!(decode_saved_resource_value(&bytes).unwrap(),nested);
        for bytes in [vec![1,2],vec![0xff],vec![0x0a,1],vec![4,1,0,0,0,0,0,0,0,255],vec![6,255,255,255,255,255,255,255,255]]{assert!(decode_saved_resource_value(&bytes).is_err());}
        for text in ["m^1","s*m","m/m","m^128","m//s","m^01"]{assert!(dimension(text).is_err());}
        let mut deep=vec![];for _ in 0..130{deep.push(6);deep.extend_from_slice(&1u64.to_le_bytes());}deep.push(0);
        assert!(matches!(decode_saved_resource_value(&deep),Err(Failure::LimitExceeded)));
    }
    #[test]
    fn rd670_decode_budget_is_shared_across_tiny_stream_amplification() {
        let mut bytes=vec![9];bytes.extend_from_slice(&10_000u64.to_le_bytes());bytes.extend_from_slice(&0u64.to_le_bytes());
        let mut budget=ValueDecodeBudget::new();let mut admitted=0;
        while decode_saved_resource_value_with_budget(&bytes,&mut budget).is_ok(){admitted+=1;}
        assert!(admitted>0&&admitted<100);
        assert!(matches!(decode_saved_resource_value_with_budget(&bytes,&mut budget),Err(Failure::LimitExceeded)));
    }
}
