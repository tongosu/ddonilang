//! Shared decoder of existing core ResourceValue canonical bytes. Table BE
//! framing does not change this owner's LE scalars/counts or Tensor framing.
use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1 as Failure;
use ddonirang_core::{Fixed64, ResourceHandle, UnitDim, UnitValue};
use ddonirang_core::platform::{RecordFieldType,ResourceMapEntry,ResourceValue};
use std::collections::BTreeMap;

const MAX_BYTES: usize = 2_000_000;
// RD-694: host stack depth is not saved-value authority. Heap frames and the
// existing byte/materialization budgets govern this binary consumer.
const MAX_MATERIALIZED: usize = 16_000_000;

pub(super) struct ValueDecodeBudget {
    remaining:usize,
    targets:Vec<(Vec<u8>,[u8;32])>,
    rational:std::pin::Pin<Box<ddonirang_core::saved_rational::SavedRationalValidationContextV1>>,
}
impl ValueDecodeBudget {
    pub(super) fn new()->Self {Self{remaining:MAX_MATERIALIZED,targets:Vec::new(),
        rational:ddonirang_core::saved_rational::SavedRationalValidationContextV1::new_root()}}
    pub(super) fn charge(&mut self,n:usize)->Result<(),Failure> {
        self.remaining=self.remaining.checked_sub(n).ok_or(Failure::LimitExceeded)?;Ok(())
    }
    fn slots(&mut self,n:usize)->Result<(),Failure> {
        self.charge(n.checked_mul(std::mem::size_of::<ResourceValue>()).ok_or(Failure::LimitExceeded)?)
    }
    fn target(&mut self,kind:&[u8],identity:[u8;32])->Result<(),Failure>{
        self.charge(kind.len().checked_add(128).ok_or(Failure::LimitExceeded)?)?;
        self.targets.try_reserve(1).map_err(|_|Failure::LimitExceeded)?;
        self.targets.push((kind.to_vec(),identity));Ok(())
    }
    pub(super) fn check_targets(&self,subjects:&BTreeMap<[u8;32],super::complete_state_tables::SubjectValueCandidateV1>)->Result<(),Failure>{
        for (kind,id) in &self.targets {
            if !subjects.get(id).is_some_and(|row|row.target_type==*kind){return Err(Failure::SubjectCustodyInvalid);}
        }
        Ok(())
    }

    /// One save-side root is reused for every rational leaf and table row.
    /// Intrinsic hash projection is not a substitute for this admission.
    pub(super) fn validate_rational_save(&mut self,root:&ResourceValue)->Result<(),Failure> {
        use ddonirang_core::saved_rational::{encode,SavedRationalError};
        let mut frames:Vec<Box<dyn Iterator<Item=&ResourceValue>+'_>>=vec![Box::new(std::iter::once(root))];
        while let Some(frame)=frames.last_mut() {
            let Some(value)=frame.next() else {frames.pop();continue;};
            match value {
                ResourceValue::RationalV1(value)=>{
                    let remaining=&mut self.remaining;
                    encode(value,self.rational.as_mut(),&mut |n|{
                        *remaining=remaining.checked_sub(n).ok_or(SavedRationalError::LimitExceeded)?;Ok(())
                    }).map_err(|error|match error {SavedRationalError::LimitExceeded=>Failure::LimitExceeded,_=>Failure::InvalidInput})?;
                }
                ResourceValue::List(items)|ResourceValue::Tuple{items,..}=>frames.push(Box::new(items.iter())),
                ResourceValue::Set(items)=>frames.push(Box::new(items.values())),
                ResourceValue::Record{fields,..}=>frames.push(Box::new(fields.values())),
                ResourceValue::Map(items)=>frames.push(Box::new(items.values().flat_map(|entry|[&entry.key,&entry.value]))),
                _=>{},
            }
        }
        Ok(())
    }
}

struct Reader<'a> { bytes: &'a [u8], at: usize, contracts: &'a [crate::registered_atom_contract::RecordedAtomContract], references:Option<&'a crate::subject_reference_owner::SubjectReferenceContextV1<'a>> }

// Each frame owns the already parsed siblings. Its cleanup is iterative even
// when a later name, count or child is rejected. No semantic depth cap.
struct TypeFrame {
    tuple: bool,
    name: Option<String>,
    args: Vec<RecordFieldType>,
    fields: BTreeMap<String, RecordFieldType>,
    next_field: Option<String>,
    remaining: usize,
}

impl TypeFrame {
    fn finish(&mut self) -> RecordFieldType {
        if self.tuple { return RecordFieldType::Tuple { items: std::mem::take(&mut self.args) }; }
        match self.name.take() {
            Some(name) => RecordFieldType::Applied { name, args: std::mem::take(&mut self.args) },
            None => RecordFieldType::Record { fields: std::mem::take(&mut self.fields) },
        }
    }
}

impl Drop for TypeFrame {
    fn drop(&mut self) {
        for kind in self.args.drain(..) { kind.drop_stack_safe(); }
        for kind in std::mem::take(&mut self.fields).into_values() { kind.drop_stack_safe(); }
    }
}

struct StagedValue(ResourceValue);
impl StagedValue {
    fn take(&mut self) -> ResourceValue { std::mem::replace(&mut self.0, ResourceValue::None) }
}
impl Drop for StagedValue {
    fn drop(&mut self) { self.take().drop_stack_safe(); }
}

struct ValueFrame {
    value: StagedValue,
    remaining: usize,
    capacity: Option<usize>,
    key: Option<StagedValue>,
    index: Option<String>,
    next_field: Option<String>,
    nominal:Option<NominalFrame>,
}
struct NominalFrame {
    tag:u8,identity:[u8;32],start:usize,outer_limit:usize,child_end:usize,
    slots:Vec<([u8;32],RecordFieldType,ResourceValue)>,
    next:Option<([u8;32],RecordFieldType)>,
}
impl ValueFrame {
    fn accept(&mut self, value: &mut StagedValue, budget: &mut ValueDecodeBudget) -> Result<(), Failure> {
        if let Some(nominal)=&mut self.nominal {
            let (id,kind)=nominal.next.take().ok_or(Failure::InvalidInput)?;
            check_saved_type(&value.0,&kind)?;
            nominal.slots.push((id,kind,value.take()));
            self.remaining-=1;return Ok(());
        }
        match &mut self.value.0 {
            ResourceValue::List(items) | ResourceValue::Tuple { items, .. } => items.push(value.take()),
            ResourceValue::Set(items) => {
                let key = value.0.canon_key();
                budget.charge(key.len().checked_mul(2).and_then(|n| n.checked_add(64)).ok_or(Failure::LimitExceeded)?)?;
                if items.last_key_value().is_some_and(|(previous, _)| previous >= &key) { return Err(Failure::InvalidInput); }
                items.insert(key, value.take());
            }
            ResourceValue::Map(entries) => {
                if let Some(mut key) = self.key.take() {
                    entries.insert(self.index.take().expect("parsed index"), ResourceMapEntry { key: key.take(), value: value.take() });
                } else {
                    let index = value.0.canon_key();
                    budget.charge(index.len().checked_mul(2).and_then(|n| n.checked_add(64)).ok_or(Failure::LimitExceeded)?)?;
                    if entries.last_key_value().is_some_and(|(previous, _)| previous >= &index) { return Err(Failure::InvalidInput); }
                    self.index = Some(index);
                    self.key = Some(StagedValue(value.take()));
                }
            }
            ResourceValue::Record { fields, .. } => {
                fields.insert(self.next_field.take().expect("parsed field"), value.take());
            }
            _ => unreachable!("container frame"),
        }
        self.remaining -= 1;
        Ok(())
    }
    fn finish(&mut self) -> Result<ResourceValue, Failure> {
        if let Some(nominal)=self.nominal.take(){
            let table=ddonirang_core::nominal::NominalSlotTableV1::from_slots(nominal.identity,&nominal.slots,MAX_BYTES-1,MAX_MATERIALIZED)
                .map_err(|_|Failure::InvalidInput)?;
            let table=std::sync::Arc::new(table);
            return Ok(if nominal.tag==0x11 {ResourceValue::NominalValueV1(table)}else{ResourceValue::SubjectPayloadV1(table)});
        }
        if let Some(capacity) = self.capacity {
            let ResourceValue::List(buffer) = &mut self.value.0 else { unreachable!("stream buffer") };
            let n = buffer.len();
            buffer.resize(capacity, ResourceValue::None);
            return Ok(ResourceValue::stream_v1_from_ring_buffer(capacity as u64, n.saturating_sub(1) as u64, n as u64, std::mem::take(buffer)));
        }
        if let ResourceValue::Record{field_types,fields}=&self.value.0 {
            if !field_types.keys().eq(fields.keys()){return Err(Failure::InvalidInput);}
            let mut pending=field_types.values().collect::<Vec<_>>();let mut resolved_nominal=false;
            while let Some(kind)=pending.pop(){match kind {
                RecordFieldType::ResolvedNominalV1{..}=>{resolved_nominal=true;break;},
                RecordFieldType::Tuple{items}=>pending.extend(items),
                RecordFieldType::Applied{args,..}=>pending.extend(args),
                RecordFieldType::Record{fields}=>pending.extend(fields.values()),_=>{},
            }}
            if resolved_nominal {
                for (kind,value) in field_types.values().zip(fields.values()){check_saved_type(value,kind)?;}
            }else{
                let restored=crate::ddn_runtime::value_from_resource_value(&self.value.0);
                let admitted=ddonirang_lang::runtime::validate_record_value(&restored);
                restored.drop_saved_tree_stack_safe();admitted.map_err(|_|Failure::InvalidInput)?;
            }
        }
        Ok(self.value.take())
    }
}

impl<'a> Reader<'a> {
    fn next_nominal_slot(&mut self,frame:&mut ValueFrame,catalog:&crate::authored_project::registration::SavedNominalCatalogV1,budget:&mut ValueDecodeBudget)->Result<(),Failure>{
        let nominal=frame.nominal.as_mut().ok_or(Failure::InvalidInput)?;
        let schema=catalog.schema(&nominal.identity).ok_or(Failure::CustodyConflict)?;
        let slot=schema.slots.get(nominal.slots.len()).ok_or(Failure::InvalidInput)?;
        if self.take(32)?!=slot.identity{return Err(Failure::InvalidInput);}
        let type_len=usize::try_from(self.le64()?).map_err(|_|Failure::LimitExceeded)?;
        let type_raw=self.take(type_len)?;
        let (expected_len,nodes)=slot.kind.canonical_measure(4096,4096).ok_or(Failure::LimitExceeded)?;
        if type_len!=expected_len{return Err(Failure::InvalidInput);}
        budget.charge(nodes.checked_mul(std::mem::size_of::<RecordFieldType>()).and_then(|n|n.checked_add(type_len)).ok_or(Failure::LimitExceeded)?)?;
        let mut expected=Vec::new();slot.kind.encode_canon(&mut expected);
        if type_raw!=expected{return Err(Failure::InvalidInput);}
        let child_len=usize::try_from(self.le64()?).map_err(|_|Failure::LimitExceeded)?;
        if child_len==0{return Err(Failure::InvalidInput);}
        let end=self.at.checked_add(child_len).filter(|end|*end<=self.bytes.len()).ok_or(Failure::InvalidInput)?;
        nominal.child_end=end;nominal.next=Some((slot.identity,slot.kind.clone()));
        self.bytes=&self.bytes[..end];Ok(())
    }
    fn take(&mut self,n:usize)->Result<&'a [u8],Failure> {
        let end=self.at.checked_add(n).ok_or(Failure::LimitExceeded)?;
        let bytes=self.bytes.get(self.at..end).ok_or(Failure::InvalidInput)?;
        self.at=end;Ok(bytes)
    }
    fn le64(&mut self)->Result<u64,Failure> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().map_err(|_|Failure::InvalidInput)?))
    }
    fn identity(&mut self,budget:&mut ValueDecodeBudget,max:usize)->Result<Vec<u8>,Failure> {
        let n=usize::try_from(self.le64()?).map_err(|_|Failure::LimitExceeded)?;
        if n==0||n>max {return Err(Failure::LimitExceeded);}
        let bytes=self.take(n)?;budget.charge(n)?;Ok(bytes.to_vec())
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
    fn next_type_field(&mut self, frame: &mut TypeFrame, budget: &mut ValueDecodeBudget) -> Result<(), Failure> {
        let name = self.text()?;
        if name.is_empty() || frame.fields.last_key_value().is_some_and(|(previous, _)| previous.as_str() >= name) {
            return Err(Failure::InvalidInput);
        }
        budget.charge(name.len().checked_mul(2).and_then(|n| n.checked_add(64)).ok_or(Failure::LimitExceeded)?)?;
        frame.next_field = Some(name.to_owned());
        Ok(())
    }

    fn record_type(&mut self,_depth:usize,budget:&mut ValueDecodeBudget)->Result<RecordFieldType,Failure> {
        self.record_type_context(budget,false)
    }
    fn record_type_context(&mut self,budget:&mut ValueDecodeBudget,output_atoms:bool)->Result<RecordFieldType,Failure>{
        let mut frames: Vec<TypeFrame> = Vec::new();
        loop {
            budget.charge(std::mem::size_of::<RecordFieldType>())?;
            let mut kind = match self.take(1)?[0] {
                5 if output_atoms=>{
                    if self.take(2)?!=[1,0]{return Err(Failure::InvalidInput);}
                    RecordFieldType::RegisteredAtomV1
                },
                4=>{
                    if self.take(2)?!=[1,0]{return Err(Failure::InvalidInput);}
                    let category=self.take(1)?[0];
                    if !matches!(category,1|2){return Err(Failure::InvalidInput);}
                    let name_type_identity=self.take(32)?.try_into().map_err(|_|Failure::InvalidInput)?;
                    RecordFieldType::ResolvedNominalV1{category,name_type_identity}
                },
                0 => {
                    let name = self.text()?; budget.charge(name.len())?;
                    RecordFieldType::Named { name: name.to_owned() }
                }
                tag @ (1 | 2 | 3) => {
                    let name = if tag == 1 {
                        let name = self.text()?; budget.charge(name.len())?;
                        Some(name.to_owned())
                    } else { None };
                    let n = self.count()?;
                    let mut frame = TypeFrame { tuple: tag == 3, name, args: Vec::new(), fields: BTreeMap::new(), next_field: None, remaining: n };
                    if tag != 2 {
                        budget.charge(n.checked_mul(std::mem::size_of::<RecordFieldType>()).ok_or(Failure::LimitExceeded)?)?;
                        frame.args.try_reserve_exact(n).map_err(|_| Failure::LimitExceeded)?;
                    }
                    if n == 0 { frame.finish() } else {
                        if tag == 2 { self.next_type_field(&mut frame, budget)?; }
                        frames.try_reserve(1).map_err(|_| Failure::LimitExceeded)?;
                        frames.push(frame);
                        continue;
                    }
                }
                _ => return Err(Failure::InvalidInput),
            };
            loop {
                let Some(frame) = frames.last_mut() else { return Ok(kind); };
                if frame.name.is_some() || frame.tuple {
                    frame.args.push(kind);
                } else {
                    frame.fields.insert(frame.next_field.take().expect("parsed field name"), kind);
                }
                frame.remaining -= 1;
                if frame.remaining != 0 {
                    if frame.name.is_none() && !frame.tuple { self.next_type_field(frame, budget)?; }
                    break;
                }
                kind = frames.pop().expect("completed frame").finish();
            }
        }
    }
    fn next_value_field(&mut self, frame: &mut ValueFrame, budget: &mut ValueDecodeBudget) -> Result<(), Failure> {
        let ResourceValue::Record { field_types, fields } = &frame.value.0 else { unreachable!("record frame") };
        let name = self.text()?;
        if name.is_empty() || !field_types.contains_key(name) || fields.last_key_value().is_some_and(|(previous, _)| previous.as_str() >= name) {
            return Err(Failure::InvalidInput);
        }
        budget.charge(name.len().checked_mul(2).and_then(|n| n.checked_add(64)).ok_or(Failure::LimitExceeded)?)?;
        frame.next_field = Some(name.to_owned());
        Ok(())
    }
    fn value(&mut self,_depth:usize,budget:&mut ValueDecodeBudget,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,subject_payload:bool)->Result<ResourceValue,Failure> {
        let full_bytes=self.bytes;
        let mut frames: Vec<ValueFrame> = Vec::new();
        loop {
        budget.slots(1)?;
        let start=self.at;
        let mut value = StagedValue(match self.take(1)?[0] {
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
            tag @ (0x11|0x13)=>{
                if tag==0x13&&(!subject_payload||!frames.is_empty()||start!=0){return Err(Failure::SubjectCustodyInvalid);}
                let catalog=catalog.ok_or(Failure::CustodyConflict)?;
                if self.take(2)?!=[1,0]{return Err(Failure::InvalidInput);}
                let identity=self.take(32)?.try_into().map_err(|_|Failure::InvalidInput)?;
                let schema=catalog.schema(&identity).ok_or(Failure::CustodyConflict)?;
                if schema.category!=if tag==0x11{2}else{1}{return Err(Failure::CustodyConflict);}
                let count=usize::try_from(self.le64()?).map_err(|_|Failure::LimitExceeded)?;
                if count!=schema.slots.len(){return Err(Failure::InvalidInput);}
                budget.charge(count.checked_mul(std::mem::size_of::<([u8;32],RecordFieldType,ResourceValue)>()).ok_or(Failure::LimitExceeded)?)?;
                let mut slots=Vec::new();slots.try_reserve_exact(count).map_err(|_|Failure::LimitExceeded)?;
                let mut frame=ValueFrame{value:StagedValue(ResourceValue::None),remaining:count,capacity:None,key:None,index:None,next_field:None,
                    nominal:Some(NominalFrame{tag,identity,start,outer_limit:self.bytes.len(),child_end:0,slots,next:None})};
                if count==0 {
                    budget.charge((self.at-start).checked_mul(2).ok_or(Failure::LimitExceeded)?)?;
                    frame.finish()?
                }else{
                    self.next_nominal_slot(&mut frame,catalog,budget)?;
                    budget.charge(std::mem::size_of::<ValueFrame>())?;
                    frames.try_reserve(1).map_err(|_|Failure::LimitExceeded)?;
                    frames.push(frame);continue;
                }
            },
            0x12=>{
                let catalog=catalog.ok_or(Failure::CustodyConflict)?;
                if self.take(2)?!=[1,0]{return Err(Failure::InvalidInput);}
                let name_type_identity=self.take(32)?.try_into().map_err(|_|Failure::InvalidInput)?;
                if !catalog.schema(&name_type_identity).is_some_and(|schema|schema.category==1){return Err(Failure::CustodyConflict);}
                let world_identity=self.take(32)?.try_into().map_err(|_|Failure::InvalidInput)?;
                let subject_identity=self.take(32)?.try_into().map_err(|_|Failure::InvalidInput)?;
                if self.le64()?!=0{return Err(Failure::SubjectCustodyInvalid);}
                let handle=ddonirang_core::nominal::SubjectHandleV1{name_type_identity,world_identity,subject_identity};
                self.references.ok_or(Failure::SubjectCustodyInvalid)?.verify_handle(&handle)?;
                budget.charge(std::mem::size_of::<ddonirang_core::nominal::SubjectHandleV1>())?;
                budget.target(&handle.name_type_identity,handle.subject_identity)?;
                ResourceValue::SubjectHandleV1(Box::new(handle))
            },
            0x10=>{
                use ddonirang_core::saved_rational::{decode_prefix,SavedRationalError};
                let remaining=&mut budget.remaining;
                let candidate=decode_prefix(&self.bytes[start..],budget.rational.as_mut(),&mut |n| {
                    *remaining=remaining.checked_sub(n).ok_or(SavedRationalError::LimitExceeded)?;Ok(())
                }).map_err(|error|match error {
                    SavedRationalError::LimitExceeded=>Failure::LimitExceeded,_=>Failure::InvalidInput,
                })?;
                self.at=start.checked_add(candidate.consumed()).ok_or(Failure::LimitExceeded)?;
                ResourceValue::RationalV1(candidate.value().clone())
            },
            tag @ (6..=0x0a | 0x0f)=>{
                let mut frame = ValueFrame { value: StagedValue(ResourceValue::None), remaining: 0,
                    capacity: None, key: None, index: None, next_field: None,nominal:None };
                if tag == 9 {
                    frame.capacity = Some(usize::try_from(self.le64()?).map_err(|_|Failure::LimitExceeded)?);
                }
                if tag == 0x0a {
                    if self.take(1)? != [1] { return Err(Failure::InvalidInput); }
                    let mut kind = self.record_type(0, budget)?;
                    let RecordFieldType::Record { fields: field_types } = &mut kind else {
                        kind.drop_stack_safe(); return Err(Failure::InvalidInput);
                    };
                    frame.value.0 = ResourceValue::Record { field_types: std::mem::take(field_types), fields: BTreeMap::new() };
                }
                if tag == 0x0f {
                    if self.take(1)? != [1] { return Err(Failure::InvalidInput); }
                    let mut kind = self.record_type(0, budget)?;
                    let RecordFieldType::Tuple { items } = &mut kind else { return Err(Failure::InvalidInput); };
                    frame.value.0 = ResourceValue::Tuple { element_types: std::mem::take(items), items: Vec::new() };
                }
                let n = self.count()?;
                frame.remaining = n;
                match tag {
                    6 | 9 => {
                        let capacity = frame.capacity.unwrap_or(n);
                        if tag == 9 {
                            if n > capacity { return Err(Failure::InvalidInput); }
                            if capacity > MAX_BYTES / std::mem::size_of::<ResourceValue>() { return Err(Failure::LimitExceeded); }
                        }
                        budget.slots(capacity)?;
                        let mut items = Vec::new();
                        items.try_reserve_exact(capacity).map_err(|_|Failure::LimitExceeded)?;
                        frame.value.0 = ResourceValue::List(items);
                    }
                    7 => frame.value.0 = ResourceValue::Set(BTreeMap::new()),
                    8 => {
                        if n > self.bytes.len().saturating_sub(self.at) / 2 { return Err(Failure::InvalidInput); }
                        frame.remaining = n.checked_mul(2).ok_or(Failure::LimitExceeded)?;
                        frame.value.0 = ResourceValue::Map(BTreeMap::new());
                    }
                    0x0a => {
                        let ResourceValue::Record { field_types, .. } = &frame.value.0 else { unreachable!() };
                        if n != field_types.len() { return Err(Failure::InvalidInput); }
                        if n != 0 { self.next_value_field(&mut frame, budget)?; }
                    }
                    0x0f => {
                        let ResourceValue::Tuple { element_types, items } = &mut frame.value.0 else { unreachable!() };
                        if n != element_types.len() { return Err(Failure::InvalidInput); }
                        budget.slots(n)?;
                        items.try_reserve_exact(n).map_err(|_| Failure::LimitExceeded)?;
                    }
                    _ => unreachable!(),
                }
                if frame.remaining == 0 { frame.finish()? } else {
                    budget.charge(std::mem::size_of::<ValueFrame>())?;
                    frames.try_reserve(1).map_err(|_|Failure::LimitExceeded)?;
                    frames.push(frame);
                    continue;
                }
            },
            0x0b=>{
                if self.take(2)?!=[1,0] {return Err(Failure::SubjectCustodyInvalid);}
                let target_type_identity=self.identity(budget,4096)?;
                let owner_space_identity=self.identity(budget,32)?;
                let target_identity=self.identity(budget,32)?;
                let generation=self.le64()?;
                let value=ddonirang_core::platform::IdentityReferenceValueV1 {
                    profile_version:1,target_type_identity,owner_space_identity,target_identity,generation,
                };
                self.references.ok_or(Failure::SubjectCustodyInvalid)?.verify(&value)?;
                budget.target(&value.target_type_identity,value.target_identity.as_slice().try_into().map_err(|_|Failure::SubjectCustodyInvalid)?)?;
                budget.charge(std::mem::size_of::<ddonirang_core::platform::IdentityReferenceValueV1>())?;
                ResourceValue::IdentityReference(value.into())
            },
            0x0c=>{
                // The existing Bytes owner retains its BE header inside the
                // ResourceValue frame. Charge the payload before allocation.
                let header=self.take(20)?;
                let n=usize::try_from(u64::from_be_bytes(header[12..20].try_into()
                    .map_err(|_|Failure::InvalidInput)?)).map_err(|_|Failure::LimitExceeded)?;
                self.take(n)?;
                budget.charge(n)?;
                let frame=&self.bytes[start+1..self.at];
                ResourceValue::Bytes(ddonirang_core::bytes::ByteSequence::from_storage_frame(frame)
                    .map_err(|_|Failure::InvalidInput)?)
            },
            0x0d=>{
                if self.take(2)?!=[1,0] {return Err(Failure::InvalidInput);}
                let owner_registry_identity=self.identity(budget,4096)?;
                let registered_atom_identity=self.identity(budget,4096)?;
                let profile=self.identity(budget,64)?;
                if profile!=b"ddn.registered-atom-contract.v1"||self.take(2)?!=[1,0] {return Err(Failure::InvalidInput);}
                let contract_snapshot_identity=self.identity(budget,32)?;
                if contract_snapshot_identity.len()!=32 {return Err(Failure::InvalidInput);}
                let contract_sha256=self.take(32)?.try_into().map_err(|_|Failure::InvalidInput)?;
                let atom=ddonirang_core::platform::RegisteredAtomValueV1 {
                    profile_version:1,owner_registry_identity,registered_atom_identity,
                    contract_profile:"ddn.registered-atom-contract.v1".into(),contract_version:1,
                    contract_snapshot_identity,contract_sha256,
                };
                // Provenance admission precedes Set/Map key materialization.
                crate::registered_atom_contract::verify_recorded_atom(&atom,self.contracts).map_err(|_|Failure::CustodyConflict)?;
                budget.charge(std::mem::size_of::<ddonirang_core::platform::RegisteredAtomValueV1>())?;
                ResourceValue::RegisteredAtom(atom.into())
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
            // Reference/atom require their exact typed owners.
            // Never guess them from a Map/String or grant custody here.
            _=>return Err(Failure::InvalidInput),
        });
        loop {
            let Some(frame) = frames.last_mut() else { return Ok(value.take()); };
            if let Some(nominal)=&frame.nominal {
                if self.at!=nominal.child_end{return Err(Failure::InvalidInput);}
                self.bytes=&full_bytes[..nominal.outer_limit];
            }
            frame.accept(&mut value, budget)?;
            if frame.remaining != 0 {
                if frame.nominal.is_some(){self.next_nominal_slot(frame,catalog.ok_or(Failure::CustodyConflict)?,budget)?;}
                else if matches!(&frame.value.0, ResourceValue::Record { .. }) { self.next_value_field(frame, budget)?; }
                break;
            }
            if let Some(nominal)=&frame.nominal {
                budget.charge((self.at-nominal.start).checked_mul(2).ok_or(Failure::LimitExceeded)?)?;
            }
            value.0 = frames.pop().expect("completed value frame").finish()?;
        }
        }
    }
}

/// Saved bytes prove only category/type consistency, not source provenance.
/// Source-bound admission remains the caller's separate publication gate.
pub(crate) fn check_saved_type(value: &ResourceValue, kind: &RecordFieldType) -> Result<(), Failure> {
    use ddonirang_lang::{ast::TypeRef, runtime::{check_value_type, canonical_type_name}};
    let mut pending = vec![(value, kind)];
    while let Some((value, kind)) = pending.pop() {
        match kind {
            RecordFieldType::RegisteredAtomV1=>return Err(Failure::InvalidInput),
            RecordFieldType::ResolvedNominalV1{category,name_type_identity}=>{
                // Type equality only. This helper does not admit a catalog,
                // saved handle, Subject payload or a registration context.
                if !match (*category,value) {
                    (2,ResourceValue::NominalValueV1(value))=>value.name_type_identity()==name_type_identity,
                    (1,ResourceValue::SubjectHandleV1(value))=>&value.name_type_identity==name_type_identity,
                    _=>false,
                } {return Err(Failure::InvalidInput);}
            },
            RecordFieldType::Tuple { items: expected } => {
                let ResourceValue::Tuple { element_types, items } = value else { return Err(Failure::InvalidInput); };
                if expected != element_types || expected.len() != items.len() { return Err(Failure::InvalidInput); }
                pending.extend(items.iter().zip(expected).rev());
            }
            RecordFieldType::Record { fields: expected } => {
                let ResourceValue::Record { field_types, fields } = value else { return Err(Failure::InvalidInput); };
                if expected != field_types || !expected.keys().eq(fields.keys()) { return Err(Failure::InvalidInput); }
                pending.extend(fields.values().zip(expected.values()).rev());
            }
            RecordFieldType::Applied { name, args } => {
                let canonical = canonical_type_name(name);
                if canonical != *name { return Err(Failure::InvalidInput); }
                match (canonical.as_str(), value) {
                    ("차림", ResourceValue::List(items)) if args.len() == 1 => pending.extend(items.iter().rev().map(|item| (item, &args[0]))),
                    ("모음", ResourceValue::Set(items)) if args.len() == 1 => pending.extend(items.values().rev().map(|item| (item, &args[0]))),
                    ("짝맞춤", ResourceValue::Map(entries)) if args.len() == 2 => {
                        for entry in entries.values().rev() { pending.push((&entry.value, &args[1])); pending.push((&entry.key, &args[0])); }
                    }
                    // Existing applied numeric-unit types are still admitted
                    // by the ordinary language type checker, not a new rule.
                    _ if args.iter().all(|arg| matches!(arg, RecordFieldType::Named { .. })) => {
                        let expected = TypeRef::Applied { name: name.clone(), args: args.iter().map(|arg| {
                            let RecordFieldType::Named { name } = arg else { unreachable!() };
                            TypeRef::Named(name.clone())
                        }).collect() };
                        let runtime = crate::ddn_runtime::value_from_resource_value(value);
                        let checked = check_value_type(&runtime, &expected);
                        runtime.drop_saved_tree_stack_safe();
                        checked.map_err(|_| Failure::InvalidInput)?;
                    }
                    _ => return Err(Failure::InvalidInput),
                }
            }
            RecordFieldType::Named { name } => {
                // Argument-level integer widening is not saved type authority.
                if name == "나눔수" || matches!(value, ResourceValue::RationalV1(_)) {
                    if name != "나눔수" || !matches!(value, ResourceValue::RationalV1(_)) { return Err(Failure::InvalidInput); }
                    continue;
                }
                if super::tuple_value::contains_tuple(value) { return Err(Failure::InvalidInput); }
                // Preserve canonical alias admission; never use this check to
                // infer which declared numeric type supplied the descriptor.
                if canonical_type_name(name) != *name { return Err(Failure::InvalidInput); }
                let runtime = crate::ddn_runtime::value_from_resource_value(value);
                let checked = check_value_type(&runtime, &TypeRef::Named(name.clone()));
                runtime.drop_saved_tree_stack_safe();
                checked.map_err(|_| Failure::InvalidInput)?;
            }
        }
    }
    Ok(())
}

fn check_tuple_trees(root: &ResourceValue) -> Result<(), Failure> {
    let mut pending = vec![root];
    while let Some(value) = pending.pop() {
        match value {
            ResourceValue::Tuple { element_types, items } => {
                if element_types.len() != items.len() { return Err(Failure::InvalidInput); }
                for (kind, item) in element_types.iter().zip(items) { check_saved_type(item, kind)?; }
            }
            ResourceValue::List(items) => pending.extend(items.iter().rev()),
            ResourceValue::Set(items) => pending.extend(items.values().rev()),
            ResourceValue::Record { fields, .. } => pending.extend(fields.values().rev()),
            ResourceValue::Map(entries) => for entry in entries.values().rev() { pending.push(&entry.value); pending.push(&entry.key); },
            _ => {},
        }
    }
    Ok(())
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

/// Captures actual resolved slots once; later snapshot access never consults
/// the changing live table or re-dereferences nested reference leaves.
pub(super) fn capture_subject_view(
    table:std::sync::Arc<ddonirang_core::nominal::NominalSlotTableV1>,
    catalog:&crate::authored_project::registration::SavedNominalCatalogV1,
    references:&crate::subject_reference_owner::SubjectReferenceContextV1<'_>,
    contracts:&[crate::registered_atom_contract::RecordedAtomContract],
)->Result<ddonirang_lang::runtime::FrozenSubjectViewV1,Failure>{
    let slots=decode_nominal_slots(&table,catalog,Some(references),contracts,1)?;
    Ok(ddonirang_lang::runtime::FrozenSubjectViewV1::from_resolved_slots(table,slots))
}
pub(super) fn decode_nominal_slots(
    table:&std::sync::Arc<ddonirang_core::nominal::NominalSlotTableV1>,
    catalog:&crate::authored_project::registration::SavedNominalCatalogV1,
    references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,
    contracts:&[crate::registered_atom_contract::RecordedAtomContract],category:u8,
)->Result<BTreeMap<[u8;32],ResourceValue>,Failure>{
    let schema=catalog.schema(table.name_type_identity()).filter(|schema|schema.category==category).ok_or(Failure::CustodyConflict)?;
    let mut reader=Reader{bytes:table.canonical_body(),at:0,contracts,references};
    if reader.take(2)?!=1u16.to_le_bytes()||reader.take(32)?!=table.name_type_identity()
        ||reader.le64()?!=schema.slots.len() as u64{return Err(Failure::InvalidInput);}
    let mut budget=ValueDecodeBudget::new();
    budget.charge(schema.slots.len().checked_mul(96+std::mem::size_of::<ResourceValue>()).ok_or(Failure::LimitExceeded)?)?;
    let mut slots=BTreeMap::new();
    for slot in &schema.slots {
        if reader.take(32)?!=slot.identity{return Err(Failure::InvalidInput);}
        let n=usize::try_from(reader.le64()?).map_err(|_|Failure::LimitExceeded)?;
        let raw=reader.take(n)?;let mut expected=Vec::new();slot.kind.encode_canon(&mut expected);
        if raw!=expected{return Err(Failure::InvalidInput);}
        let n=usize::try_from(reader.le64()?).map_err(|_|Failure::LimitExceeded)?;
        let raw=reader.take(n)?;
        let value=decode_saved_resource_value_with_catalog(raw,&mut budget,contracts,references,Some(catalog),false)?;
        check_saved_type(&value,&slot.kind)?;slots.insert(slot.identity,value);
    }
    if reader.at!=reader.bytes.len(){return Err(Failure::InvalidInput);}
    Ok(slots)
}

pub(crate) fn decode_saved_resource_value(bytes:&[u8])->Result<ResourceValue,Failure> {
    decode_saved_resource_value_with_budget(bytes,&mut ValueDecodeBudget::new())
}

/// Shared canonical type decoder for the VC04 descriptor's existing-type
/// field. It issues no value, nominal identity, or schema admission.
pub(crate) fn decode_recorded_field_type(bytes:&[u8])->Result<RecordFieldType,Failure> {
    decode_field_type_context(bytes,false)
}
/// Output-evidence data only. This parser issues no source/registration proof
/// and is deliberately not used by existing saved-slot/value admissions.
pub(crate) fn decode_result_output_field_type(bytes:&[u8])->Result<RecordFieldType,Failure>{
    decode_field_type_context(bytes,true)
}
fn decode_field_type_context(bytes:&[u8],output_atoms:bool)->Result<RecordFieldType,Failure>{
    if bytes.is_empty() || bytes.len()>4096 {return Err(Failure::LimitExceeded);}
    let mut reader=Reader{bytes,at:0,contracts:&[],references:None};
    let value=reader.record_type_context(&mut ValueDecodeBudget::new(),output_atoms)?;
    let mut canonical=Vec::new();value.encode_canon(&mut canonical);
    if reader.at!=bytes.len() || canonical!=bytes {
        value.drop_stack_safe();return Err(Failure::InvalidInput);
    }
    Ok(value)
}

pub(super) fn decode_saved_resource_value_with_budget(bytes:&[u8],budget:&mut ValueDecodeBudget)->Result<ResourceValue,Failure> {
    decode_saved_resource_value_with_contracts(bytes,budget,&[])
}

pub(super) fn decode_saved_resource_value_with_contracts(bytes:&[u8],budget:&mut ValueDecodeBudget,contracts:&[crate::registered_atom_contract::RecordedAtomContract])->Result<ResourceValue,Failure> {
    decode_saved_resource_value_with_custody(bytes,budget,contracts,None)
}

pub(super) fn decode_saved_resource_value_with_custody(bytes:&[u8],budget:&mut ValueDecodeBudget,contracts:&[crate::registered_atom_contract::RecordedAtomContract],references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>)->Result<ResourceValue,Failure> {
    decode_saved_resource_value_with_catalog(bytes,budget,contracts,references,None,false)
}

/// Catalog/category/type/custody admission of a value candidate. Complete
/// FRAME2 still owns the joint LIVE payload bijection and target existence.
pub(super) fn decode_saved_resource_value_with_catalog(bytes:&[u8],budget:&mut ValueDecodeBudget,contracts:&[crate::registered_atom_contract::RecordedAtomContract],references:Option<&crate::subject_reference_owner::SubjectReferenceContextV1<'_>>,catalog:Option<&crate::authored_project::registration::SavedNominalCatalogV1>,subject_payload:bool)->Result<ResourceValue,Failure> {
    if bytes.len()>MAX_BYTES{return Err(Failure::LimitExceeded);}
    crate::registered_atom_contract::verify_contract_set(contracts).map_err(|_|Failure::CustodyConflict)?;
    let mut reader=Reader{bytes,at:0,contracts,references};let value=reader.value(0,budget,catalog,subject_payload)?;
    if reader.at!=bytes.len(){value.drop_stack_safe();return Err(Failure::InvalidInput);}
    check_tuple_trees(&value)?;
    // Re-encode through the existing authority; no alternate canonical writer.
    let mut canonical=Vec::new();value.encode_canon(&mut canonical);
    if canonical!=bytes{value.drop_stack_safe();return Err(Failure::InvalidInput);}
    Ok(value)
}

/// Tests consume the actual protected Native source owner's catalog and raw
/// bytes printed by the unmodified shared VM; no source lowering or ID issuer.
/// The Subject portion is a data-owner vector, NOT a source birth/retire PASS.
#[cfg(test)]
pub(crate) fn check_actual_r5_nominal_catalog_vectors(catalog:&crate::authored_project::registration::SavedNominalCatalogV1,encoded:&[String],pesi:&[u8;32]) {
    use super::complete_state_frame::CompleteStateFrameV1;
    use crate::subject_reference_owner::{LiveSubjectOwnerV1,SubjectReferenceContextV1};
    use ddonirang_core::nominal::{NominalSlotTableV1,SubjectHandleV1};
    use std::sync::Arc;
    let transported=encoded.iter().map(|text|super::tensor_frame::decode_base64url(text).unwrap()).collect::<Vec<_>>();
    let raw=transported.as_slice();
    let decode=|bytes:&[u8]|decode_saved_resource_value_with_catalog(bytes,&mut ValueDecodeBudget::new(),&[],None,Some(catalog),false);
    assert_eq!(raw.len(),3);
    let decoded=raw.iter().map(|bytes|decode(bytes).unwrap()).collect::<Vec<_>>();
    assert_ne!(decoded[0],decoded[1]);
    let dto=super::resource_value_to_detjson(&decoded[0]);
    assert!(super::resource_value_from_detjson_with_contracts(&dto,&[]).is_err(),"JSON never supplies its own catalog");
    assert_eq!(super::resource_value_candidate_from_detjson_with_catalog(&dto,&[],None,&mut ValueDecodeBudget::new(),Some(catalog)).unwrap(),decoded[0]);
    let mut claimed=dto.clone();claimed.as_object_mut().unwrap().insert("admitted".into(),serde_json::Value::Bool(true));
    assert!(super::resource_value_candidate_from_detjson_with_catalog(&claimed,&[],None,&mut ValueDecodeBudget::new(),Some(catalog)).is_err());
    for bytes in raw {
        assert!(decode_saved_resource_value(bytes).is_err(),"no catalog cannot admit a nominal value");
        let value=decode(bytes).unwrap();let mut encoded=Vec::new();value.encode_canon(&mut encoded);assert_eq!(&encoded,bytes);
        for n in 0..bytes.len(){assert!(decode(&bytes[..n]).is_err(),"truncation {n}");}
    }
    let mut wrong=raw[0].clone();wrong[43]^=1;assert!(decode(&wrong).is_err(),"wrong SlotID");
    let mut wrong=raw[0].clone();wrong[3]^=1;assert!(decode(&wrong).is_err(),"unrecorded NameTypeID");
    let mut suffix=raw[2].clone();suffix.push(0);assert!(decode(&suffix).is_err());
    let mut limited=ValueDecodeBudget::new();limited.remaining=1;
    assert!(matches!(decode_saved_resource_value_with_catalog(&raw[0],&mut limited,&[],None,Some(catalog),false),Err(Failure::LimitExceeded)));
    let subject=catalog.schemas().find(|row|row.category==1).unwrap();
    let mut owner=LiveSubjectOwnerV1::fresh(*pesi).unwrap();
    let allocated=owner.commit_allocation(owner.prepare_allocation(&subject.name_type).unwrap()).unwrap();
    let handle=ResourceValue::SubjectHandleV1(Box::new(SubjectHandleV1{name_type_identity:subject.name_type,
        world_identity:owner.custody().world,subject_identity:allocated.target_identity.as_slice().try_into().unwrap()}));
    let subject_id:[u8;32]=allocated.target_identity.as_slice().try_into().unwrap();
    let references=SubjectReferenceContextV1::new(owner.custody()).unwrap();
    let subject_slots=subject.slots.iter().map(|slot|(slot.identity,slot.kind.clone(),decoded[0].clone())).collect::<Vec<_>>();
    let payload=ResourceValue::SubjectPayloadV1(Arc::new(NominalSlotTableV1::from_slots(subject.name_type,&subject_slots,MAX_BYTES-1,1_000_000).unwrap()));
    let holder=catalog.schemas().find(|row|row.category==2&&row.slots.iter().any(|slot|matches!(&slot.kind,RecordFieldType::ResolvedNominalV1{category:1,..}))).unwrap();
    let holder_slots=holder.slots.iter().map(|slot|(slot.identity,slot.kind.clone(),handle.clone())).collect::<Vec<_>>();
    let holding=ResourceValue::NominalValueV1(Arc::new(NominalSlotTableV1::from_slots(holder.name_type,&holder_slots,MAX_BYTES-1,1_000_000).unwrap()));
    fn canonical(value:&ResourceValue)->Vec<u8>{let mut out=Vec::new();value.encode_canon(&mut out);out}
    let payload_bytes=canonical(&payload);let handle_bytes=canonical(&handle);
    assert!(decode_saved_resource_value_with_catalog(&payload_bytes,&mut ValueDecodeBudget::new(),&[],Some(&references),Some(catalog),false).is_err());
    let mut nested=vec![6];nested.extend_from_slice(&1u64.to_le_bytes());nested.extend_from_slice(&payload_bytes);
    assert!(decode_saved_resource_value_with_catalog(&nested,&mut ValueDecodeBudget::new(),&[],Some(&references),Some(catalog),true).is_err());
    assert_eq!(decode_saved_resource_value_with_catalog(&handle_bytes,&mut ValueDecodeBudget::new(),&[],Some(&references),Some(catalog),false).unwrap(),handle);
    let mut foreign=handle_bytes.clone();foreign[35]^=1;
    assert!(decode_saved_resource_value_with_catalog(&foreign,&mut ValueDecodeBudget::new(),&[],Some(&references),Some(catalog),false).is_err());
    let mut resources=1u64.to_be_bytes().to_vec();resources.extend_from_slice(&1u32.to_be_bytes());resources.push(b'r');
    let holding_bytes=canonical(&holding);resources.extend_from_slice(&(holding_bytes.len() as u64).to_be_bytes());resources.extend_from_slice(&holding_bytes);
    let mut subjects=1u64.to_be_bytes().to_vec();subjects.extend_from_slice(&subject_id);
    subjects.extend_from_slice(&(payload_bytes.len() as u64).to_be_bytes());subjects.extend_from_slice(&payload_bytes);
    let custody=super::complete_state_tables::encode_saved_world_custody(owner.custody()).unwrap();let empty=0u64.to_be_bytes();
    let frame=CompleteStateFrameV1{pesi,resources:&resources,parameters:&empty,next_tick:0,subjects:&subjects,atom_evidence:&empty,subject_custody:&custody};
    assert!(frame.decode_value_tables().is_err());
    let candidate=frame.decode_value_tables_with_catalog(Some(catalog)).unwrap();
    assert_eq!(candidate.resources["r"],holding);assert_eq!(candidate.subjects[&subject_id].value,payload);
    assert!(!frame.encode_canonical_with_catalog(Some(catalog)).unwrap().is_empty());
    let missing=CompleteStateFrameV1{subjects:&empty,..frame};assert!(missing.decode_value_tables_with_catalog(Some(catalog)).is_err());
    println!("VC04_R5_ACTUAL_CATALOG_VALUE_AND_JOINT_FRAME_PASS; SOURCE_SUBJECT_LIFECYCLE_NOT_RUN");
}

#[cfg(test)]
mod tests {
    #[test]
    fn n1_rational_approved_25_carrier_vectors() {
        use sha2::Digest;
        // Fixture bytes are retained verbatim, including historical proposal
        // labels. The later Owner packet assigns tag10 and the 8M profile.
        let raw=include_str!("../../tests/fixtures/n1_rational_approved_carrier_vectors.json");
        assert_eq!(hex::encode(sha2::Sha256::digest(raw.as_bytes())),"9605b22d92d9f6eddbfd8f7a7fb6dcdf4c28c119d17156d373475b441e594124");
        let fixture:serde_json::Value=serde_json::from_str(raw).unwrap();
        let mut counts=[0;2];
        for row in fixture["rows"].as_array().unwrap() {
            let id=row["id"].as_str().unwrap();let bytes=hex::decode(row["hex"].as_str().unwrap()).unwrap();
            assert_eq!(bytes.len() as u64,row["byte_length"].as_u64().unwrap(),"{id}");
            assert_eq!(hex::encode(sha2::Sha256::digest(&bytes)),row["sha256"],"{id}");
            let decoded=decode_saved_resource_value(&bytes);
            match row["expect"].as_str().unwrap() {
                "ACCEPT_SPEC_ONLY"=>{
                    let value=decoded.unwrap_or_else(|error|panic!("{id}: {error:?}"));
                    let mut actual=Vec::new();value.encode_canon(&mut actual);assert_eq!(actual,bytes,"{id}");
                    if !matches!(&value,ResourceValue::Tuple{..}) {check_saved_type(&value,&RecordFieldType::Named{name:"나눔수".into()}).unwrap();}
                    counts[0]+=1;
                }
                "REJECT_TYPE_MISMATCH"=>{
                    assert!(check_saved_type(&decoded.unwrap(),&RecordFieldType::Named{name:row["source_type"].as_str().unwrap().into()}).is_err(),"{id}");counts[1]+=1;
                }
                _=>{assert!(decoded.is_err(),"{id}");counts[1]+=1;}
            }
        }
        assert_eq!(counts,[7,18]);
    }

    #[test]
    fn n1_rational_genuine_codec_kind_and_shared_root_budget() {
        use sha2::Digest;
        use ddonirang_core::saved_rational::NormalizedRationalValueV1;
        use num_bigint::BigInt;
        let small=ResourceValue::RationalV1(NormalizedRationalValueV1::from_parts(1.into(),3.into()).unwrap());
        let mut bytes=Vec::new();small.encode_canon(&mut bytes);
        assert_eq!(small.canonical_byte_length(2_000_000,100),Some(bytes.len()));
        assert_eq!(bytes.len(),44);assert_eq!(bytes[0],0x10);
        assert_eq!(hex::encode(sha2::Sha256::digest(&bytes)),"7022e00436f7ed8aa070fbcc7c8f1dfd32a02a96a02a6b152d1a371d2e3d7fa6");
        let restored=decode_saved_resource_value(&bytes).unwrap();assert_eq!(restored,small);
        let named=|name:&str|RecordFieldType::Named{name:name.into()};
        check_saved_type(&restored,&named("나눔수")).unwrap();
        assert!(check_saved_type(&restored,&named("셈수")).is_err());
        assert!(check_saved_type(&ResourceValue::Fixed64(Fixed64::ONE),&named("나눔수")).is_err());
        let integer=ResourceValue::RationalV1(NormalizedRationalValueV1::from_parts(2.into(),1.into()).unwrap());
        let mut integer_bytes=Vec::new();integer.encode_canon(&mut integer_bytes);
        assert!(matches!(decode_saved_resource_value(&integer_bytes).unwrap(),ResourceValue::RationalV1(_)),"denominator one keeps its kind");
        let (mut a,mut b)=(BigInt::from(0),BigInt::from(1));
        for _ in 0..4097 {let next=&a+&b;a=b;b=next;}
        let large=ResourceValue::RationalV1(NormalizedRationalValueV1::from_parts(a,b).unwrap());
        let mut raw=Vec::new();large.encode_canon(&mut raw);
        let mut shared=ValueDecodeBudget::new();
        for _ in 0..2 {decode_saved_resource_value_with_budget(&raw,&mut shared).unwrap();}
        assert!(matches!(decode_saved_resource_value_with_budget(&raw,&mut shared),Err(Failure::LimitExceeded)));
        assert!(decode_saved_resource_value(&raw).is_ok(),"same third leaf fits only a different root");
        let mut save=ValueDecodeBudget::new();
        for _ in 0..2 {save.validate_rational_save(&large).unwrap();}
        assert!(matches!(save.validate_rational_save(&large),Err(Failure::LimitExceeded)));
        let leaf=super::super::resource_value_to_detjson(&large);
        let two=serde_json::json!({"value_det":{"a":leaf.clone(),"b":leaf.clone()}}).to_string();
        assert!(super::super::parse_snapshot_json(&two).is_ok());
        let three=serde_json::json!({"value_det":{"a":leaf.clone(),"b":leaf.clone(),"c":leaf}}).to_string();
        assert!(three.len()<2_000_000);
        assert!(super::super::parse_snapshot_json(&three).is_err(),"the JSON parser must not reset work per leaf");
        let rows=[("a".into(),large.clone()),("b".into(),large.clone()),("c".into(),large)].into_iter().collect();
        assert!(matches!(super::super::complete_state_tables::encode_resource_table(&rows),Err(Failure::LimitExceeded)),"save must not reset per row");
    }
    use super::*;

    #[test]
    fn vc13_tuple_approved_binary_vectors_and_value_type_rejection() {
        let empty = hex::decode("0f010300000000000000000000000000000000").unwrap();
        let value = decode_saved_resource_value(&empty).unwrap();
        assert!(matches!(&value, ResourceValue::Tuple { element_types, items } if element_types.is_empty() && items.is_empty()));
        assert_ne!(value, ResourceValue::List(Vec::new()));
        assert_eq!(super::super::resource_value_to_detjson(&value).to_string(),
            "{\"canonical_base64url\":\"DwEDAAAAAAAAAAAAAAAAAAAAAA\",\"canonical_byte_length\":19,\"canonical_sha256\":\"13514904a0016fa6868d482de2785eedbe6de6a9493e3fbe51ae751c1f609d28\",\"resource_value_tag\":\"0f\",\"type\":\"tuple_v1\",\"version\":1}");
        let singleton = hex::decode("0f01030100000000000000000600000000000000ec8588ec88980100000000000000020700000000000000").unwrap();
        let value = decode_saved_resource_value(&singleton).unwrap();
        let copy = value.clone();
        let mut encoded = Vec::new(); copy.encode_canon(&mut encoded);
        assert_eq!(encoded, singleton);
        let mut wrong_version = empty.clone(); wrong_version[1] = 2;
        let mut wrong_kind = empty.clone(); wrong_kind[2] = 2;
        let mut wrong_arity = empty.clone(); wrong_arity[11] = 1;
        let mut wrong_type = singleton.clone();
        // Same UTF-8 byte length, wrong existing declared type; never infer
        // a numeric descriptor from the Fixed64 payload to repair this.
        wrong_type[20..26].copy_from_slice("없음".as_bytes());
        for bytes in [wrong_version, wrong_kind, wrong_arity, wrong_type] {
            assert!(decode_saved_resource_value(&bytes).is_err());
        }
    }

    #[test]
    fn vc04_r5_resolved_nominal_type_exact_leaf_and_nested_codec(){
        let ty=RecordFieldType::ResolvedNominalV1{category:2,name_type_identity:[0x31;32]};
        let mut raw=Vec::new();ty.encode_canon(&mut raw);
        assert_eq!(raw.len(),36);assert_eq!(&raw[..4],&[4,1,0,2]);assert_eq!(&raw[4..],&[0x31;32]);
        assert_eq!(decode_recorded_field_type(&raw).unwrap(),ty);
        for n in 0..raw.len(){assert!(decode_recorded_field_type(&raw[..n]).is_err());}
        for (index,byte) in [(0,5),(1,2),(2,1),(3,0),(3,3)]{
            let mut bad=raw.clone();bad[index]=byte;assert!(decode_recorded_field_type(&bad).is_err());
        }
        let mut trailing=raw.clone();trailing.push(0);assert!(decode_recorded_field_type(&trailing).is_err());
        let nested=RecordFieldType::Tuple{items:vec![RecordFieldType::Named{name:"나눔수".into()},ty]};
        let mut bytes=Vec::new();nested.encode_canon(&mut bytes);assert_eq!(decode_recorded_field_type(&bytes).unwrap(),nested);
    }

    #[test]
    fn vc13_tuple_type_clone_drop_and_encoding_are_stack_safe() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut kind = RecordFieldType::Named { name: "글".into() };
            for _ in 0..20_000 { kind = RecordFieldType::Tuple { items: vec![kind] }; }
            let copy = kind.clone();
            assert!(kind == copy);
            let mut bytes = Vec::new(); kind.encode_canon(&mut bytes);
            assert_eq!(bytes.len(), 20_000 * 9 + 12);
            drop(kind); drop(copy);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn rd694_value_frames_parse_and_reject_partial_siblings_on_small_stack() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut nested = ResourceValue::None;
            for _ in 0..20_000 { nested = ResourceValue::List(vec![nested]); }
            let mut body = Vec::new(); nested.encode_canon(&mut body);
            nested.drop_stack_safe();
            let mut accepted = vec![6]; accepted.extend_from_slice(&1u64.to_le_bytes());
            accepted.extend_from_slice(&body);
            let decoded = decode_saved_resource_value(&accepted).unwrap();
            let mut again = Vec::new(); decoded.encode_canon(&mut again);
            drop(decoded);
            assert_eq!(again, accepted);
            // First completed child / pending Map key must survive a later
            // invalid tag without recursive destructor cleanup.
            for (tag, count) in [(6, 2u64), (7, 2), (8, 1)] {
                let mut damaged = vec![tag]; damaged.extend_from_slice(&count.to_le_bytes());
                damaged.extend_from_slice(&body); damaged.push(255);
                assert!(matches!(decode_saved_resource_value(&damaged), Err(Failure::InvalidInput)));
            }
            let mut limited = vec![6]; limited.extend_from_slice(&2u64.to_le_bytes());
            limited.extend_from_slice(&body); limited.push(0);
            let mut budget = ValueDecodeBudget::new();
            budget.remaining = 258 * std::mem::size_of::<ResourceValue>();
            assert!(matches!(decode_saved_resource_value_with_budget(&limited, &mut budget), Err(Failure::LimitExceeded)));
        }).unwrap().join().unwrap();
    }
    use super::*;
    #[test]
    fn saved_record_type_parser_uses_frames_without_host_depth_authority() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
        fn applied(depth: usize) -> Vec<u8> {
            let mut bytes = Vec::new();
            for _ in 0..depth {
                bytes.push(1); bytes.extend_from_slice(&1u64.to_le_bytes()); bytes.push(b'L');
                bytes.extend_from_slice(&1u64.to_le_bytes());
            }
            bytes.push(0); bytes.extend_from_slice(&1u64.to_le_bytes()); bytes.push(b'N');
            bytes
        }
        for depth in [0, 1, 127, 128, 129, 20_000] {
            let bytes = applied(depth);
            let mut reader = Reader { bytes: &bytes, at: 0, contracts: &[], references: None };
            let kind = reader.record_type(0, &mut ValueDecodeBudget::new()).unwrap();
            let mut encoded = Vec::new(); kind.encode_canon(&mut encoded);
            kind.drop_stack_safe();
            assert_eq!(encoded, bytes); assert_eq!(reader.at, bytes.len());
        }
        let bytes = applied(20_000);
        let mut reader = Reader { bytes: &bytes, at: 0, contracts: &[], references: None };
        let mut budget = ValueDecodeBudget::new();
        budget.remaining = std::mem::size_of::<RecordFieldType>();
        assert!(matches!(reader.record_type(0, &mut budget), Err(Failure::LimitExceeded)));
        }).unwrap().join().unwrap();
    }

    #[test]
    fn saved_record_type_partial_siblings_are_cleaned_on_later_failure() {
        let mut child = Vec::new();
        for _ in 0..120 {
            child.push(1); child.extend_from_slice(&1u64.to_le_bytes()); child.push(b'L');
            child.extend_from_slice(&1u64.to_le_bytes());
        }
        child.push(0); child.extend_from_slice(&1u64.to_le_bytes()); child.push(b'N');
        let mut prefix = vec![2]; prefix.extend_from_slice(&2u64.to_le_bytes());
        prefix.extend_from_slice(&1u64.to_le_bytes()); prefix.push(b'a'); prefix.extend_from_slice(&child);
        for suffix in [vec![], vec![0; 8], { let mut v = 1u64.to_le_bytes().to_vec(); v.push(b'a'); v }] {
            let mut bytes = prefix.clone(); bytes.extend_from_slice(&suffix);
            let mut reader = Reader { bytes: &bytes, at: 0, contracts: &[], references: None };
            assert!(matches!(reader.record_type(0, &mut ValueDecodeBudget::new()), Err(Failure::InvalidInput)));
        }
        let bytes = prefix;
        let mut reader = Reader { bytes: &bytes, at: 0, contracts: &[], references: None };
        let mut budget = ValueDecodeBudget::new();
        budget.remaining = 1;
        assert!(matches!(reader.record_type(0, &mut budget), Err(Failure::LimitExceeded)));
    }
    #[test]
    fn beta1_reference_binary_uses_one_recorded_world_and_checks_nested_leaves(){
        use crate::subject_reference_owner::{SavedSubjectRow,SavedWorldIdentityCustodyV1,SubjectReferenceContextV1};
        let identity=[2;32];let world=[3;32];
        let reference=ddonirang_core::platform::IdentityReferenceValueV1{profile_version:1,target_type_identity:vec![0x11,0],owner_space_identity:world.to_vec(),target_identity:identity.to_vec(),generation:0};
        let saved=ResourceValue::IdentityReference(reference.clone().into());
        let mut custody=SavedWorldIdentityCustodyV1{nonce:[1;32],world,pesi:[4;32],next_subject_ordinal:1,
            rows:vec![SavedSubjectRow{identity,ordinal:0,live:true,target_type:vec![0x11,0]}]};
        let context=SubjectReferenceContextV1::new(&custody).unwrap();
        let mut bytes=Vec::new();saved.encode_canon(&mut bytes);
        assert_eq!(&bytes[..3],&[0x0b,1,0]);
        assert!(decode_saved_resource_value(&bytes).is_err(),"context-free ingress cannot grant reference custody");
        assert_eq!(decode_saved_resource_value_with_custody(&bytes,&mut ValueDecodeBudget::new(),&[],Some(&context)).unwrap(),saved);
        for n in 0..bytes.len(){assert!(decode_saved_resource_value_with_custody(&bytes[..n],&mut ValueDecodeBudget::new(),&[],Some(&context)).is_err(),"truncation {n}");}
        for mutation in 0..5 {
            let mut wrong=reference.clone();match mutation {
                0=>wrong.profile_version=2,1=>wrong.owner_space_identity[0]^=1,
                2=>wrong.target_identity[0]^=1,3=>wrong.generation=1,_=>wrong.target_type_identity[0]^=1,
            }
            let nested=ResourceValue::List(vec![saved.clone(),ResourceValue::IdentityReference(wrong.into())]);
            let mut raw=Vec::new();nested.encode_canon(&mut raw);
            assert!(decode_saved_resource_value_with_custody(&raw,&mut ValueDecodeBudget::new(),&[],Some(&context)).is_err());
        }
        let runtime=crate::ddn_runtime::value_from_resource_value(&saved);
        assert!(matches!(runtime,ddonirang_lang::runtime::Value::IdentityReference(ref value) if value.as_ref()==&reference));
        assert_eq!(crate::unit_wire::resource_value_to_detjson(&saved)["generation_u64"],"0");
        drop(context);custody.rows[0].live=false;
        let dead=SubjectReferenceContextV1::new(&custody).unwrap();
        assert!(decode_saved_resource_value_with_custody(&bytes,&mut ValueDecodeBudget::new(),&[],Some(&dead)).is_err());
        let mut suffix=bytes.clone();suffix.push(0);
        assert!(decode_saved_resource_value_with_custody(&suffix,&mut ValueDecodeBudget::new(),&[],Some(&dead)).is_err());
    }
    #[test]
    fn beta1_atom_binary_admission_precedes_keys_and_rejects_every_truncation() {
        let (_,contract)=crate::registered_atom_contract::RecordedAtomContract::record(b"owner",&[vec![1]]).unwrap();
        let atom=ResourceValue::RegisteredAtom(contract.admitted_value(b"owner",&[1]).unwrap().into());
        let mut bytes=Vec::new();atom.encode_canon(&mut bytes);
        assert_eq!(&bytes[..3],&[0x0d,1,0]);
        assert!(decode_saved_resource_value(&bytes).is_err());
        assert_eq!(decode_saved_resource_value_with_contracts(&bytes,&mut ValueDecodeBudget::new(),&[contract.clone()]).unwrap(),atom);
        for n in 0..bytes.len() {
            assert!(decode_saved_resource_value_with_contracts(&bytes[..n],&mut ValueDecodeBudget::new(),&[contract.clone()]).is_err(),"truncated {n}");
        }
        for value in [ResourceValue::List(vec![atom.clone()]),ResourceValue::set_from_values(vec![atom.clone()])] {
            let mut nested=Vec::new();value.encode_canon(&mut nested);
            assert_eq!(decode_saved_resource_value_with_contracts(&nested,&mut ValueDecodeBudget::new(),&[contract.clone()]).unwrap(),value);
            *nested.last_mut().unwrap()^=1;
            assert!(decode_saved_resource_value_with_contracts(&nested,&mut ValueDecodeBudget::new(),&[contract.clone()]).is_err());
        }
        let mut oversized=bytes.clone();oversized[3..11].copy_from_slice(&4097u64.to_le_bytes());
        assert!(matches!(decode_saved_resource_value_with_contracts(&oversized,&mut ValueDecodeBudget::new(),&[contract]),Err(Failure::LimitExceeded)));
    }
    #[test]
    fn rd670_record_bytes_use_exact_typed_owner_and_reject_corruption() {
        // Independent literal VC07 wire: record/version, Record type, one
        // Named field, then one Fixed64 field value. LE counts are unchanged.
        let mut bytes=vec![0x0a,1,2];
        bytes.extend_from_slice(&1u64.to_le_bytes());
        bytes.extend_from_slice(&3u64.to_le_bytes());bytes.extend_from_slice("값".as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&6u64.to_le_bytes());bytes.extend_from_slice("셈수".as_bytes());
        bytes.extend_from_slice(&1u64.to_le_bytes());
        bytes.extend_from_slice(&3u64.to_le_bytes());bytes.extend_from_slice("값".as_bytes());
        bytes.push(2);bytes.extend_from_slice(&(-7i64).to_le_bytes());
        let value=ResourceValue::Record{
            field_types:BTreeMap::from([("값".into(),RecordFieldType::Named{name:"셈수".into()})]),
            fields:BTreeMap::from([("값".into(),ResourceValue::Fixed64(Fixed64::from_raw_i64(-7)))])
        };
        assert_eq!(decode_saved_resource_value(&bytes).unwrap(),value);
        let mut encoded=Vec::new();value.encode_canon(&mut encoded);assert_eq!(encoded,bytes);
        let nested=ResourceValue::Record{
            field_types:BTreeMap::from([("점".into(),RecordFieldType::Record{fields:match &value {
                ResourceValue::Record{field_types,..}=>field_types.clone(),_=>unreachable!()
            }})]),
            fields:BTreeMap::from([("점".into(),value.clone())])
        };
        let mut encoded=Vec::new();nested.encode_canon(&mut encoded);
        assert_eq!(decode_saved_resource_value(&encoded).unwrap(),nested);
        for n in 0..bytes.len() {assert!(decode_saved_resource_value(&bytes[..n]).is_err());}
        let mut wrong=bytes.clone();wrong[1]=2;assert!(decode_saved_resource_value(&wrong).is_err());
        let ResourceValue::Record{field_types,..}=&value else {unreachable!()};
        let wrong=ResourceValue::Record{field_types:field_types.clone(),fields:BTreeMap::from([("값".into(),ResourceValue::Bool(true))])};
        let mut encoded=Vec::new();wrong.encode_canon(&mut encoded);
        assert!(decode_saved_resource_value(&encoded).is_err());
        let mut trailing=bytes;trailing.push(0);assert!(decode_saved_resource_value(&trailing).is_err());
    }
    #[test]
    fn rd694_record_projection_uses_actual_saved_decoder_and_preserves_bytes() {
        use ddonirang_core::platform::RecordFieldType as R;
        for leaf in [ResourceValue::Fixed64(Fixed64::ZERO), ResourceValue::Bool(true)] {
            let valid = matches!(&leaf, ResourceValue::Fixed64(_));
            let mut kind = R::Named { name: "셈수".into() };
            let mut value = leaf;
            // Cross the retired 128-depth gate in both value and type trees.
            for _ in 0..512 {
                kind = R::Applied { name: "차림".into(), args: vec![kind] };
                value = ResourceValue::List(vec![value]);
            }
            let record = ResourceValue::Record {
                field_types: [("a".into(), kind)].into(),
                fields: [("a".into(), value)].into(),
            };
            let mut bytes = Vec::new(); record.encode_canon(&mut bytes);
            let decoded = decode_saved_resource_value(&bytes);
            record.drop_stack_safe();
            if valid {
                let decoded = decoded.unwrap();
                let mut again = Vec::new(); decoded.encode_canon(&mut again);
                decoded.drop_stack_safe();
                assert_eq!(again, bytes);
            } else {
                assert!(decoded.is_err());
            }
        }
    }
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
    fn rd670_bytes_use_the_existing_owner_and_reject_truncation() {
        use ddonirang_core::bytes::ByteSequence;
        for payload in [vec![], vec![0,255], "가".as_bytes().to_vec()] {
            let value=ResourceValue::Bytes(ByteSequence::new(payload));
            let mut frame=Vec::new();value.encode_canon(&mut frame);
            assert_eq!(decode_saved_resource_value(&frame).unwrap(),value);
            assert_eq!(frame[0],0x0c);
            for n in 0..frame.len() {assert!(decode_saved_resource_value(&frame[..n]).is_err());}
            let mut bad=frame.clone();bad[12]=2;
            assert!(decode_saved_resource_value(&bad).is_err());
            let mut bad=frame;bad.push(0);
            assert!(decode_saved_resource_value(&bad).is_err());
        }
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
        // RD-694 supersedes the provisional nesting gate, not canonical bytes.
        let decoded = decode_saved_resource_value(&deep).unwrap();
        let mut again = Vec::new(); decoded.encode_canon(&mut again);
        assert_eq!(again, deep);
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
