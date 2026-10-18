use core::cmp::Ordering;
use std::collections::{BTreeMap, VecDeque};

use blake3::hash;
use serde::{Deserialize, Serialize};
use xxhash_rust::xxh3::xxh3_64;

use crate::atomic_commit::{AtomicCommitInputRecord, AtomicCommitReceipt};
use crate::fixed64::Fixed64;
use crate::resource::ResourceHandle;
use crate::signals::{DiagEvent, ExprTrace, FaultContext, Signal, SignalSink, SourceSpan, TickId};
use crate::units::UnitValue;

// A counting sink uses the SAME canonical writer as the byte sink. This is
// allocation preflight, not a second value grammar or a restore authority.
trait CanonOutput {
    fn push(&mut self, byte:u8);
    fn extend_from_slice(&mut self, bytes:&[u8]);
    fn bytes_value(&mut self, value:&crate::bytes::ByteSequence);
    fn rational_value(&mut self,value:&crate::saved_rational::NormalizedRationalValueV1);
    fn available(&self)->bool {true}
    fn children(&mut self, _count:usize)->bool {true}
}
impl CanonOutput for Vec<u8> {
    fn push(&mut self,byte:u8){Vec::push(self,byte);}
    fn extend_from_slice(&mut self,bytes:&[u8]){Vec::extend_from_slice(self,bytes);}
    fn bytes_value(&mut self,value:&crate::bytes::ByteSequence){self.extend_from_slice(&value.storage_frame());}
    fn rational_value(&mut self,value:&crate::saved_rational::NormalizedRationalValueV1){value.write_canonical_projection(|part|self.extend_from_slice(part));}
}
struct CanonByteCount {length:usize,limit:usize,remaining_nodes:usize,failed:bool}
impl CanonByteCount {
    fn add(&mut self,n:usize){
        match self.length.checked_add(n) {
            Some(next) if next<=self.limit=>self.length=next,
            _=>self.failed=true,
        }
    }
}
impl CanonOutput for CanonByteCount {
    fn push(&mut self,_byte:u8){self.add(1);}
    fn extend_from_slice(&mut self,bytes:&[u8]){self.add(bytes.len());}
    fn bytes_value(&mut self,value:&crate::bytes::ByteSequence){self.add(value.storage_frame_byte_length());}
    fn rational_value(&mut self,value:&crate::saved_rational::NormalizedRationalValueV1){
        if let Some(length)=value.canonical_projection_length(){self.add(length);}else{self.failed=true;}
    }
    fn available(&self)->bool {!self.failed}
    fn children(&mut self,count:usize)->bool {
        match self.remaining_nodes.checked_sub(count) {
            Some(next)=>self.remaining_nodes=next,
            None=>self.failed=true,
        }
        !self.failed
    }
}

#[cfg(test)]
#[test]
fn bounded_canonical_count_uses_the_existing_value_writer() {
    let cases=[ResourceValue::None,ResourceValue::Bool(true),ResourceValue::Fixed64(Fixed64::ONE),
        ResourceValue::String("한글🙂".into()),ResourceValue::Bytes(crate::bytes::ByteSequence::new(vec![0,255])),
        ResourceValue::List(vec![ResourceValue::Bool(false),ResourceValue::String("점".into())]),
        ResourceValue::set_from_values(vec![ResourceValue::Fixed64(Fixed64::ONE)]),
        ResourceValue::map_from_entries(vec![ResourceMapEntry{key:ResourceValue::String("x".into()),value:ResourceValue::None}]),
        ResourceValue::Record {field_types:BTreeMap::from([("x".into(),RecordFieldType::Named{name:"셈수".into()})]),
            fields:BTreeMap::from([("x".into(),ResourceValue::Fixed64(Fixed64::ONE))])},
        ResourceValue::stream_v1_from_ring_buffer(2,0,1,vec![ResourceValue::Bool(true),ResourceValue::None]),
    ];
    for value in cases {
        let mut bytes=Vec::new();value.encode_canon(&mut bytes);
        assert_eq!(value.canonical_byte_length(bytes.len(),100),Some(bytes.len()));
        assert_eq!(value.canonical_byte_length(bytes.len()-1,100),None);
        assert_eq!(value.canonical_byte_length(bytes.len(),0),None);
    }
    let wide=ResourceValue::List(vec![ResourceValue::None;100]);
    assert_eq!(wide.canonical_byte_length(1000,100),None);
    assert_eq!(wide.canonical_byte_length(1000,101),Some(109));
}

/// RD-660/RD-662/RD-664: semantic owner/atom identities and the separate
/// recorded-contract provenance. This carrier does not grant admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredAtomValueV1 {
    pub profile_version: u16,
    pub owner_registry_identity: Vec<u8>,
    pub registered_atom_identity: Vec<u8>,
    pub contract_profile: String,
    pub contract_version: u16,
    pub contract_snapshot_identity: Vec<u8>,
    pub contract_sha256: [u8; 32],
}

// ---------- ECS 기본 타입 (최소) ----------

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentTag(pub String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ArchetypeKey(Vec<ComponentTag>);

impl ArchetypeKey {
    fn empty() -> Self {
        Self(Vec::new())
    }

    fn from_components(components: &BTreeMap<ComponentTag, String>) -> Self {
        let tags = components.keys().cloned().collect::<Vec<_>>();
        Self(tags)
    }

    fn contains_tag(&self, tag: &ComponentTag) -> bool {
        self.0.binary_search(tag).is_ok()
    }
}

#[derive(Clone, Debug)]
struct Archetype {
    tags: Vec<ComponentTag>,
    entities: Vec<EntityId>,
    columns: BTreeMap<ComponentTag, Vec<String>>,
}

impl Archetype {
    fn new(tags: Vec<ComponentTag>) -> Self {
        let mut columns = BTreeMap::new();
        for tag in &tags {
            columns.insert(tag.clone(), Vec::new());
        }
        Self {
            tags,
            entities: Vec::new(),
            columns,
        }
    }
}

#[derive(Clone, Debug)]
struct EntityLocation {
    key: ArchetypeKey,
    index: usize,
}

#[derive(Clone, Debug, Default)]
struct EcsStore {
    archetypes: BTreeMap<ArchetypeKey, Archetype>,
    locations: BTreeMap<EntityId, EntityLocation>,
}

impl EcsStore {
    fn ensure_entity(&mut self, entity: EntityId) {
        if self.locations.contains_key(&entity) {
            return;
        }
        let components = BTreeMap::new();
        self.insert_entity(entity, ArchetypeKey::empty(), &components);
    }

    fn get_component_json(&self, entity: EntityId, tag: &ComponentTag) -> Option<String> {
        let location = self.locations.get(&entity)?;
        let archetype = self.archetypes.get(&location.key)?;
        let column = archetype.columns.get(tag)?;
        column.get(location.index).cloned()
    }

    fn set_component_json(&mut self, entity: EntityId, tag: ComponentTag, json: String) {
        if let Some(location) = self.locations.get(&entity) {
            if let Some(archetype) = self.archetypes.get_mut(&location.key) {
                if let Some(column) = archetype.columns.get_mut(&tag) {
                    if let Some(slot) = column.get_mut(location.index) {
                        *slot = json;
                        return;
                    }
                }
            }
        }

        let mut components = self.collect_components(entity);
        components.insert(tag, json);
        self.relocate_entity(entity, components);
    }

    fn remove_component(&mut self, entity: EntityId, tag: &ComponentTag) {
        let location = match self.locations.get(&entity) {
            Some(loc) => loc,
            None => return,
        };
        if !location.key.contains_tag(tag) {
            return;
        }
        let mut components = self.collect_components(entity);
        components.remove(tag);
        self.relocate_entity(entity, components);
    }

    fn component_count(&self) -> u64 {
        let mut count = 0u64;
        for location in self.locations.values() {
            if let Some(archetype) = self.archetypes.get(&location.key) {
                count += archetype.tags.len() as u64;
            }
        }
        count
    }

    fn query_entities_with_all_tags(&self, tags: &[ComponentTag]) -> Vec<EntityId> {
        let mut out = Vec::new();
        for (entity, location) in &self.locations {
            let Some(archetype) = self.archetypes.get(&location.key) else {
                continue;
            };
            if tags.iter().all(|tag| archetype.columns.contains_key(tag)) {
                out.push(*entity);
            }
        }
        out
    }

    fn for_each_component_sorted<F>(&self, mut visit: F)
    where
        F: FnMut(EntityId, &ComponentTag, &str),
    {
        for (entity, location) in &self.locations {
            let Some(archetype) = self.archetypes.get(&location.key) else {
                continue;
            };
            for tag in &archetype.tags {
                let Some(column) = archetype.columns.get(tag) else {
                    continue;
                };
                let Some(value) = column.get(location.index) else {
                    continue;
                };
                visit(*entity, tag, value);
            }
        }
    }

    fn collect_components(&self, entity: EntityId) -> BTreeMap<ComponentTag, String> {
        let mut components = BTreeMap::new();
        let Some(location) = self.locations.get(&entity) else {
            return components;
        };
        let Some(archetype) = self.archetypes.get(&location.key) else {
            return components;
        };
        for tag in &archetype.tags {
            if let Some(column) = archetype.columns.get(tag) {
                if let Some(value) = column.get(location.index) {
                    components.insert(tag.clone(), value.clone());
                }
            }
        }
        components
    }

    fn relocate_entity(&mut self, entity: EntityId, components: BTreeMap<ComponentTag, String>) {
        self.remove_entity(entity);
        let key = ArchetypeKey::from_components(&components);
        self.insert_entity(entity, key, &components);
    }

    fn remove_entity(&mut self, entity: EntityId) {
        let location = match self.locations.remove(&entity) {
            Some(location) => location,
            None => return,
        };
        let key = location.key.clone();
        if let Some(archetype) = self.archetypes.get_mut(&key) {
            if location.index < archetype.entities.len() {
                archetype.entities.remove(location.index);
                for column in archetype.columns.values_mut() {
                    if location.index < column.len() {
                        column.remove(location.index);
                    }
                }
            }
        }
        self.update_locations_from(&key, location.index);
    }

    fn insert_entity(
        &mut self,
        entity: EntityId,
        key: ArchetypeKey,
        components: &BTreeMap<ComponentTag, String>,
    ) {
        let tags = key.0.clone();
        let insert_idx = {
            let archetype = self
                .archetypes
                .entry(key.clone())
                .or_insert_with(|| Archetype::new(tags));
            let insert_idx = match archetype.entities.binary_search(&entity) {
                Ok(idx) => idx,
                Err(idx) => idx,
            };
            archetype.entities.insert(insert_idx, entity);
            for tag in &archetype.tags {
                let value = components.get(tag).cloned().unwrap_or_default();
                if let Some(column) = archetype.columns.get_mut(tag) {
                    column.insert(insert_idx, value);
                }
            }
            insert_idx
        };
        self.update_locations_from(&key, insert_idx);
    }

    fn update_locations_from(&mut self, key: &ArchetypeKey, start: usize) {
        let Some(archetype) = self.archetypes.get(key) else {
            return;
        };
        for (offset, entity) in archetype.entities.iter().skip(start).enumerate() {
            let idx = start + offset;
            self.locations.insert(
                *entity,
                EntityLocation {
                    key: key.clone(),
                    index: idx,
                },
            );
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceMapEntry {
    pub key: ResourceValue,
    pub value: ResourceValue,
}

/// RD-660/RD-662: captured committed Subject tuple. A carrier is not a
/// dereference capability; the shared Subject owner admits every use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityReferenceValueV1 {
    pub profile_version:u16,
    pub target_type_identity:Vec<u8>,
    pub owner_space_identity:Vec<u8>,
    pub target_identity:Vec<u8>,
    pub generation:u64,
}

#[derive(Debug)]
pub enum ResourceValue {
    NominalValueV1(std::sync::Arc<crate::nominal::NominalSlotTableV1>),
    SubjectHandleV1(Box<crate::nominal::SubjectHandleV1>),
    /// Authoritative Subject table payload; ordinary value admission rejects it.
    SubjectPayloadV1(std::sync::Arc<crate::nominal::NominalSlotTableV1>),
    /// Genuine immutable numeric owner. Raw Map/JSON shapes cannot create it.
    RationalV1(std::sync::Arc<crate::saved_rational::NormalizedRationalValueV1>),
    /// VC13 approved saved carrier. Types are supplied by the source owner,
    /// never inferred from scalar value shapes by the persistence adapter.
    Tuple { element_types: Vec<RecordFieldType>, items: Vec<ResourceValue> },
    IdentityReference(Box<IdentityReferenceValueV1>),
    RegisteredAtom(Box<RegisteredAtomValueV1>),
    /// RD-674 exact carrier, admitted by shared unit_wire; not a List/Map alias.
    BasicTensorV1(Vec<u8>),
    None,
    Bool(bool),
    Fixed64(Fixed64),
    Unit(UnitValue),
    String(String),
    Bytes(crate::bytes::ByteSequence),
    ResourceHandle(ResourceHandle),
    List(Vec<ResourceValue>),
    Set(BTreeMap<String, ResourceValue>),
    Map(BTreeMap<String, ResourceMapEntry>),
    /// Explicit structural record; never inferred from a Map's keys.
    Record {
        field_types: BTreeMap<String, RecordFieldType>,
        fields: BTreeMap<String, ResourceValue>,
    },
}

/// The structural type identity of an existing record field. Declaration names
/// of structural records are resolved before persistence (RD-350).
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum RecordFieldType {
    /// R5 resolved nominal leaf. Data only: schema/category admission is owned
    /// by the recorded catalog; a spelling or value shape never issues this ID.
    ResolvedNominalV1 { category:u8, name_type_identity:[u8;32] },
    Tuple { items: Vec<RecordFieldType> },
    Named { name: String },
    Applied { name: String, args: Vec<RecordFieldType> },
    Record { fields: BTreeMap<String, RecordFieldType> },
}

impl Clone for RecordFieldType {
    fn clone(&self) -> Self {
        enum Job<'a> { Type(&'a RecordFieldType), Applied(&'a str, usize), Tuple(usize), Record(&'a BTreeMap<String, RecordFieldType>) }
        let mut jobs = vec![Job::Type(self)];
        let mut built = Vec::new();
        while let Some(job) = jobs.pop() {
            match job {
                Job::Type(Self::Tuple { items }) => {
                    jobs.push(Job::Tuple(items.len()));
                    for item in items.iter().rev() { jobs.push(Job::Type(item)); }
                }
                Job::Tuple(n) => {
                    let items = built.split_off(built.len() - n);
                    built.push(Self::Tuple { items });
                }
                Job::Type(Self::Named { name }) => built.push(Self::Named { name: name.clone() }),
                Job::Type(Self::ResolvedNominalV1 {category,name_type_identity})=>built.push(Self::ResolvedNominalV1{category:*category,name_type_identity:*name_type_identity}),
                Job::Type(Self::Applied { name, args }) => {
                    jobs.push(Job::Applied(name, args.len()));
                    for arg in args.iter().rev() { jobs.push(Job::Type(arg)); }
                }
                Job::Type(Self::Record { fields }) => {
                    jobs.push(Job::Record(fields));
                    for kind in fields.values().rev() { jobs.push(Job::Type(kind)); }
                }
                Job::Applied(name, n) => {
                    let args = built.split_off(built.len() - n);
                    built.push(Self::Applied { name: name.to_owned(), args });
                }
                Job::Record(fields) => {
                    let children = built.split_off(built.len() - fields.len());
                    let fields = fields.keys().cloned().zip(children).collect();
                    built.push(Self::Record { fields });
                }
            }
        }
        built.pop().expect("one cloned type root")
    }
}

impl PartialEq for RecordFieldType {
    fn eq(&self, other: &Self) -> bool {
        let mut pending = vec![(self, other)];
        while let Some((left, right)) = pending.pop() {
            match (left, right) {
                (Self::Tuple { items: x }, Self::Tuple { items: y }) if x.len() == y.len() => pending.extend(x.iter().zip(y)),
                (Self::Named { name: a }, Self::Named { name: b }) if a == b => {}
                (Self::ResolvedNominalV1{category:a,name_type_identity:x},Self::ResolvedNominalV1{category:b,name_type_identity:y}) if a==b&&x==y=>{},
                (Self::Applied { name: a, args: x }, Self::Applied { name: b, args: y }) if a == b && x.len() == y.len() => pending.extend(x.iter().zip(y)),
                (Self::Record { fields: x }, Self::Record { fields: y }) if x.keys().eq(y.keys()) => pending.extend(x.values().zip(y.values())),
                _ => return false,
            }
        }
        true
    }
}
impl Eq for RecordFieldType {}

#[cfg(test)]
#[test]
fn vc04_resolved_nominal_type_clone_drop_and_canonical_identity_are_stack_safe(){
    std::thread::Builder::new().stack_size(128*1024).spawn(||{
        let mut kind=RecordFieldType::ResolvedNominalV1{category:2,name_type_identity:[0x31;32]};
        for _ in 0..20_000{kind=RecordFieldType::Tuple{items:vec![kind]};}
        let copy=kind.clone();assert_eq!(kind,copy);
        let mut bytes=Vec::new();copy.encode_canon(&mut bytes);
        assert_eq!(&bytes[bytes.len()-36..bytes.len()-32],&[4,1,0,2]);
        drop(copy);drop(kind);
    }).unwrap().join().unwrap();
}

impl Drop for RecordFieldType {
    fn drop(&mut self) {
        fn detach(kind: &mut RecordFieldType, pending: &mut Vec<RecordFieldType>) {
            match kind {
                RecordFieldType::Tuple { items } => pending.extend(std::mem::take(items)),
                RecordFieldType::Applied { args, .. } => pending.extend(std::mem::take(args)),
                RecordFieldType::Record { fields } => pending.extend(std::mem::take(fields).into_values()),
                RecordFieldType::Named { .. } | RecordFieldType::ResolvedNominalV1 {..} => {}
            }
        }
        let mut pending = Vec::new();
        detach(self, &mut pending);
        while let Some(mut kind) = pending.pop() {
            detach(&mut kind, &mut pending);
            // This child's Drop now sees empty containers, so call depth is
            // constant rather than proportional to the original type tree.
        }
    }
}

#[cfg(test)]
#[test]
fn saved_record_type_automatic_drop_is_stack_safe() {
    std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
        let mut kind = RecordFieldType::Named { name: "셈수".into() };
        for index in 0..20_000 {
            kind = if index % 2 == 0 {
                RecordFieldType::Applied { name: "차림".into(), args: vec![kind] }
            } else {
                RecordFieldType::Record { fields: BTreeMap::from([("a".into(), kind),
                    ("b".into(), RecordFieldType::Named { name: "글".into() })]) }
            };
        }
        let copy = kind.clone();
        let same = kind == copy;
        // Ordinary Rust Drop, not the explicit staging helper.
        drop(copy);
        drop(kind);
        assert!(same);
    }).unwrap().join().unwrap();
}

impl Clone for ResourceValue {
    fn clone(&self) -> Self {
        enum Job<'a> {
            Tuple(&'a [RecordFieldType], usize),
            Value(&'a ResourceValue), List(usize), Set(&'a BTreeMap<String, ResourceValue>),
            Map(&'a BTreeMap<String, ResourceMapEntry>),
            Record(&'a BTreeMap<String, RecordFieldType>, &'a BTreeMap<String, ResourceValue>),
        }
        let mut jobs = vec![Job::Value(self)];
        let mut built = Vec::new();
        while let Some(job) = jobs.pop() {
            match job {
                Job::Value(value) => match value {
                    Self::Tuple { element_types, items } => {
                        jobs.push(Job::Tuple(element_types, items.len()));
                        for item in items.iter().rev() { jobs.push(Job::Value(item)); }
                    }
                    Self::List(items) => {
                        jobs.push(Job::List(items.len()));
                        for value in items.iter().rev() { jobs.push(Job::Value(value)); }
                    }
                    Self::Set(items) => {
                        jobs.push(Job::Set(items));
                        for value in items.values().rev() { jobs.push(Job::Value(value)); }
                    }
                    Self::Map(items) => {
                        jobs.push(Job::Map(items));
                        for entry in items.values().rev() {
                            jobs.push(Job::Value(&entry.value)); jobs.push(Job::Value(&entry.key));
                        }
                    }
                    Self::Record { field_types, fields } => {
                        jobs.push(Job::Record(field_types, fields));
                        for value in fields.values().rev() { jobs.push(Job::Value(value)); }
                    }
                    Self::IdentityReference(value) => built.push(Self::IdentityReference(value.clone())),
                    Self::NominalValueV1(value) => built.push(Self::NominalValueV1(value.clone())),
                    Self::SubjectHandleV1(value) => built.push(Self::SubjectHandleV1(value.clone())),
                    Self::SubjectPayloadV1(value) => built.push(Self::SubjectPayloadV1(value.clone())),
                    Self::RegisteredAtom(value) => built.push(Self::RegisteredAtom(value.clone())),
                    Self::BasicTensorV1(value) => built.push(Self::BasicTensorV1(value.clone())),
                    Self::RationalV1(value) => built.push(Self::RationalV1(value.clone())),
                    Self::None => built.push(Self::None), Self::Bool(value) => built.push(Self::Bool(*value)),
                    Self::Fixed64(value) => built.push(Self::Fixed64(*value)), Self::Unit(value) => built.push(Self::Unit(*value)),
                    Self::String(value) => built.push(Self::String(value.clone())), Self::Bytes(value) => built.push(Self::Bytes(value.clone())),
                    Self::ResourceHandle(value) => built.push(Self::ResourceHandle(*value)),
                },
                Job::List(n) => {
                    let children = built.split_off(built.len() - n); built.push(Self::List(children));
                }
                Job::Tuple(types, n) => {
                    let items = built.split_off(built.len() - n);
                    built.push(Self::Tuple { element_types: types.to_vec(), items });
                }
                Job::Set(items) => {
                    let children = built.split_off(built.len() - items.len());
                    built.push(Self::Set(items.keys().cloned().zip(children).collect()));
                }
                Job::Map(items) => {
                    let children = built.split_off(built.len() - items.len() * 2);
                    let mut children = children.into_iter(); let mut entries = BTreeMap::new();
                    for index in items.keys() {
                        entries.insert(index.clone(), ResourceMapEntry { key: children.next().expect("map key"), value: children.next().expect("map value") });
                    }
                    built.push(Self::Map(entries));
                }
                Job::Record(types, fields) => {
                    let children = built.split_off(built.len() - fields.len());
                    built.push(Self::Record { field_types: types.clone(), fields: fields.keys().cloned().zip(children).collect() });
                }
            }
        }
        built.pop().expect("one cloned resource root")
    }
}

impl PartialEq for ResourceValue {
    fn eq(&self, other: &Self) -> bool {
        let mut pending = vec![(self, other)];
        while let Some((left, right)) = pending.pop() {
            match (left, right) {
                (Self::Tuple { element_types: a, items: x }, Self::Tuple { element_types: b, items: y }) if a == b && x.len() == y.len() => pending.extend(x.iter().zip(y)),
                (Self::None, Self::None) => {}
                (Self::NominalValueV1(a), Self::NominalValueV1(b)) if a == b => {}
                (Self::SubjectHandleV1(a), Self::SubjectHandleV1(b)) if a == b => {}
                (Self::SubjectPayloadV1(a), Self::SubjectPayloadV1(b)) if a == b => {}
                (Self::Bool(a), Self::Bool(b)) if a == b => {}
                (Self::Fixed64(a), Self::Fixed64(b)) if a == b => {}
                (Self::Unit(a), Self::Unit(b)) if a == b => {}
                (Self::String(a), Self::String(b)) if a == b => {}
                (Self::Bytes(a), Self::Bytes(b)) if a == b => {}
                (Self::ResourceHandle(a), Self::ResourceHandle(b)) if a == b => {}
                (Self::BasicTensorV1(a), Self::BasicTensorV1(b)) if a == b => {}
                (Self::RationalV1(a),Self::RationalV1(b)) if a==b => {}
                (Self::RegisteredAtom(a), Self::RegisteredAtom(b)) if a == b => {}
                (Self::IdentityReference(a), Self::IdentityReference(b)) if a == b => {}
                (Self::List(a), Self::List(b)) if a.len() == b.len() => pending.extend(a.iter().zip(b)),
                (Self::Set(a), Self::Set(b)) if a.keys().eq(b.keys()) => pending.extend(a.values().zip(b.values())),
                (Self::Map(a), Self::Map(b)) if a.keys().eq(b.keys()) => {
                    for (a, b) in a.values().zip(b.values()) {
                        pending.push((&a.key, &b.key)); pending.push((&a.value, &b.value));
                    }
                }
                (Self::Record { field_types: a, fields: x }, Self::Record { field_types: b, fields: y }) if a == b && x.keys().eq(y.keys()) => pending.extend(x.values().zip(y.values())),
                _ => return false,
            }
        }
        true
    }
}
impl Eq for ResourceValue {}

impl Drop for ResourceValue {
    fn drop(&mut self) {
        fn detach(value: &mut ResourceValue, pending: &mut Vec<ResourceValue>) {
            match value {
                ResourceValue::Tuple { element_types, items } => {
                    element_types.clear();
                    pending.extend(std::mem::take(items));
                }
                ResourceValue::List(items) => pending.extend(std::mem::take(items)),
                ResourceValue::Set(items) => pending.extend(std::mem::take(items).into_values()),
                ResourceValue::Map(items) => for entry in std::mem::take(items).into_values() {
                    pending.push(entry.key); pending.push(entry.value);
                },
                ResourceValue::Record { field_types, fields } => {
                    // RecordFieldType has its own stack-safe automatic Drop.
                    field_types.clear();
                    pending.extend(std::mem::take(fields).into_values());
                }
                _ => {}
            }
        }
        let mut pending = Vec::new();
        detach(self, &mut pending);
        while let Some(mut value) = pending.pop() { detach(&mut value, &mut pending); }
    }
}

#[cfg(test)]
#[test]
fn saved_resource_automatic_drop_covers_world_replacement_and_unwind() {
    fn deep_value() -> ResourceValue {
        let mut key = ResourceValue::None;
        let mut value = ResourceValue::Bool(true);
        let mut kind = RecordFieldType::Named { name: "참거짓".into() };
        for _ in 0..20_000 {
            key = ResourceValue::List(vec![key]);
            value = ResourceValue::List(vec![value]);
            kind = RecordFieldType::Applied { name: "차림".into(), args: vec![kind] };
        }
        ResourceValue::Map(BTreeMap::from([("index".into(), ResourceMapEntry { key,
            value: ResourceValue::Record {
                field_types: BTreeMap::from([("a".into(), kind)]),
                fields: BTreeMap::from([("a".into(), ResourceValue::Set(BTreeMap::from([("stored".into(), value)])))]),
            },
        })]))
    }
    std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
        let value = deep_value();
        let copy = value.clone();
        let same = value == copy;
        drop(copy);
        let mut world = NuriWorld::new();
        world.set_resource_value("deep".into(), value);
        world.set_resource_value("deep".into(), ResourceValue::None);
        world.set_resource_value("deep".into(), deep_value());
        drop(world);
        // Also cover automatic cleanup during failure unwinding; the panic is
        // deliberate test input, not a product error or a swallowed abort.
        let unwound = std::panic::catch_unwind(|| {
            let _owned = deep_value();
            panic!("saved resource cleanup probe");
        });
        assert!(same && unwound.is_err());
    }).unwrap().join().unwrap();
}

impl RecordFieldType {
    pub fn canonical_measure(&self, limit:usize, node_budget:usize)->Option<(usize,usize)> {
        let mut count=CanonByteCount {length:0,limit,remaining_nodes:node_budget.checked_sub(1)?,failed:false};
        self.encode_into(&mut count);
        (!count.failed).then_some((count.length,node_budget-count.remaining_nodes))
    }
    /// Release an owned saved type without recursively dropping its children.
    /// Saved-codec staging uses this on both accepted and rejected trees.
    pub fn drop_stack_safe(self) {
        drop(self);
    }

    pub fn encode_canon(&self, out: &mut Vec<u8>) {self.encode_into(out);}

    fn encode_into(&self, out:&mut impl CanonOutput) {
        enum Job<'a> { Type(&'a RecordFieldType), Name(&'a str) }
        let mut jobs = vec![Job::Type(self)];
        while let Some(job) = jobs.pop() {
            if !out.available(){return;}
            let kind = match job {
                Job::Name(name) => { push_str(out, name); continue; }
                Job::Type(kind) => kind,
            };
            match kind {
            Self::Tuple { items } => {
                if !out.children(items.len()) { return; }
                out.push(3);
                out.extend_from_slice(&(items.len() as u64).to_le_bytes());
                for item in items.iter().rev() { jobs.push(Job::Type(item)); }
            }
            Self::Named { name } => { out.push(0); push_str(out, name); }
            Self::ResolvedNominalV1{category,name_type_identity}=>{
                out.push(4);out.extend_from_slice(&1u16.to_le_bytes());out.push(*category);out.extend_from_slice(name_type_identity);
            }
            Self::Applied { name, args } => {
                if !out.children(args.len()){return;}
                out.push(1); push_str(out, name);
                out.extend_from_slice(&(args.len() as u64).to_le_bytes());
                for arg in args.iter().rev() { jobs.push(Job::Type(arg)); }
            }
            Self::Record { fields } => {
                if !out.children(fields.len()){return;}
                out.push(2);
                out.extend_from_slice(&(fields.len() as u64).to_le_bytes());
                for (name, kind) in fields.iter().rev() {
                    jobs.push(Job::Type(kind)); jobs.push(Job::Name(name));
                }
            }
            }
        }
    }
}

impl ResourceValue {
    /// Saved staging cleanup; also handles both Map keys and values and the
    /// separate type trees in Record. Automatic Drop uses the same owner.
    pub fn drop_stack_safe(self) {
        drop(self);
    }

    pub fn stream_v1_from_ring_buffer(
        capacity: u64,
        head: u64,
        len: u64,
        buffer: Vec<ResourceValue>,
    ) -> Self {
        ResourceValue::map_from_entries(vec![
            ResourceMapEntry {
                key: ResourceValue::String("__schema".to_string()),
                value: ResourceValue::String("ddn.stream.v1".to_string()),
            },
            ResourceMapEntry {
                key: ResourceValue::String("capacity".to_string()),
                value: ResourceValue::Fixed64(Fixed64::from_i64(
                    capacity.min(i64::MAX as u64) as i64
                )),
            },
            ResourceMapEntry {
                key: ResourceValue::String("head".to_string()),
                value: ResourceValue::Fixed64(Fixed64::from_i64(head.min(i64::MAX as u64) as i64)),
            },
            ResourceMapEntry {
                key: ResourceValue::String("len".to_string()),
                value: ResourceValue::Fixed64(Fixed64::from_i64(len.min(i64::MAX as u64) as i64)),
            },
            ResourceMapEntry {
                key: ResourceValue::String("buffer".to_string()),
                value: ResourceValue::List(buffer),
            },
        ])
    }

    pub fn set_from_values(values: Vec<ResourceValue>) -> Self {
        let mut items = BTreeMap::new();
        for value in values {
            let key = value.canon_key();
            items.entry(key).or_insert(value);
        }
        ResourceValue::Set(items)
    }

    pub fn map_from_entries(entries: Vec<ResourceMapEntry>) -> Self {
        let mut items = BTreeMap::new();
        for entry in entries {
            let key = entry.key.canon_key();
            items.entry(key).or_insert(entry);
        }
        ResourceValue::Map(items)
    }

    pub fn canon_key(&self) -> String {
        self.canon_string()
    }

    fn canon_string(&self) -> String {
        enum Job<'a> { Value(&'a ResourceValue), Text(&'static str) }
        let mut jobs = vec![Job::Value(self)];
        let mut out = String::new();
        while let Some(job) = jobs.pop() {
            let value = match job {
                Job::Text(text) => { out.push_str(text); continue; }
                Job::Value(value) => value,
            };
            let piece = match value {
            ResourceValue::NominalValueV1(_) | ResourceValue::SubjectHandleV1(_) | ResourceValue::SubjectPayloadV1(_) => {
                let mut bytes = Vec::new(); value.encode_canon(&mut bytes);
                format!("nominal-carrier-v1:{}", crate::commit_canon::carrier_base64url(&bytes))
            }
            ResourceValue::Tuple { .. } => {
                let mut bytes = Vec::new();
                value.encode_canon(&mut bytes);
                format!("tuple-v1:{}", crate::commit_canon::carrier_base64url(&bytes))
            }
            ResourceValue::RegisteredAtom(value) => format!("registered-atom-v1:{}:{}", hex_identity(&value.owner_registry_identity), hex_identity(&value.registered_atom_identity)),
            ResourceValue::IdentityReference(value) => format!("identity-reference-v1:{}:{}:{}:{}",hex_identity(&value.target_type_identity),hex_identity(&value.owner_space_identity),hex_identity(&value.target_identity),value.generation),
            ResourceValue::Bytes(value) => value.canonical_literal(),
            ResourceValue::Record { .. } => {
                let mut bytes = Vec::new();
                self.encode_canon(&mut bytes);
                let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
                format!("record-v1:{hex}")
            }
            ResourceValue::BasicTensorV1(carrier) => format!("basic-tensor-v1:{}", crate::commit_canon::carrier_base64url(carrier)),
            ResourceValue::RationalV1(_) => {
                let mut bytes=Vec::new();value.encode_canon(&mut bytes);
                format!("rational-v1:{}",crate::commit_canon::carrier_base64url(&bytes))
            }
            ResourceValue::None => "없음".to_string(),
            ResourceValue::Bool(true) => "참".to_string(),
            ResourceValue::Bool(false) => "거짓".to_string(),
            ResourceValue::Fixed64(value) => value.canonical_decimal_text(),
            ResourceValue::Unit(value) => {
                let suffix = value
                    .display_symbol()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| value.dim.format());
                format!("{}@{}", value.value.canonical_decimal_text(), suffix)
            }
            ResourceValue::String(value) => {
                format!("\"{}\"", Self::escape_canon_string(value))
            }
            ResourceValue::ResourceHandle(handle) => format!("자원#{}", handle.to_hex()),
            ResourceValue::List(items) => {
                out.push_str("차림["); jobs.push(Job::Text("]"));
                for (i, item) in items.iter().enumerate().rev() {
                    jobs.push(Job::Value(item));
                    if i > 0 { jobs.push(Job::Text(", ")); }
                }
                continue;
            }
            ResourceValue::Set(items) => {
                out.push_str("모음{"); jobs.push(Job::Text("}"));
                for (i, value) in items.values().enumerate().rev() {
                    jobs.push(Job::Value(value));
                    if i > 0 { jobs.push(Job::Text(", ")); }
                }
                continue;
            }
            ResourceValue::Map(entries) => {
                out.push_str("짝맞춤{"); jobs.push(Job::Text("}"));
                for (i, entry) in entries.values().enumerate().rev() {
                    jobs.push(Job::Value(&entry.value));
                    jobs.push(Job::Text("=>"));
                    jobs.push(Job::Value(&entry.key));
                    if i > 0 { jobs.push(Job::Text(", ")); }
                }
                continue;
            }
            };
            out.push_str(&piece);
        }
        out
    }

    fn escape_canon_string(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        for ch in input.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                '\r' => out.push_str("\\r"),
                _ => out.push(ch),
            }
        }
        out
    }

    pub fn encode_canon(&self, out: &mut Vec<u8>) {self.encode_into(out);}

    /// Product allocation preflight. The caller selects byte/node budgets;
    /// existing canonical bytes and the unbounded encoder stay unchanged.
    pub fn canonical_byte_length(&self, limit:usize, node_budget:usize)->Option<usize> {
        self.canonical_measure(limit,node_budget).map(|measure|measure.0)
    }

    pub fn canonical_measure(&self, limit:usize, node_budget:usize)->Option<(usize,usize)> {
        let mut count=CanonByteCount {length:0,limit,remaining_nodes:node_budget.checked_sub(1)?,failed:false};
        self.encode_into(&mut count);
        (!count.failed).then_some((count.length,node_budget-count.remaining_nodes))
    }

    fn encode_into(&self, out:&mut impl CanonOutput) {
        // RD-694: traversal uses borrowed heap jobs; neither encoding nor
        // recognizing a stream clones a recursively nested saved tree.
        enum Job<'a> { Value(&'a ResourceValue), Name(&'a str), Count(usize), Type(&'a RecordFieldType) }
        let mut jobs = vec![Job::Value(self)];
        while let Some(job) = jobs.pop() {
            if !out.available(){return;}
            let value = match job {
                Job::Name(name) => { push_str(out, name); continue; }
                Job::Count(n) => { out.extend_from_slice(&(n as u64).to_le_bytes()); continue; }
                Job::Type(kind) => { kind.encode_into(out); continue; }
                Job::Value(value) => value,
            };
            match value {
            ResourceValue::Tuple { element_types, items } => {
                if !out.children(items.len().saturating_add(element_types.len())) { return; }
                out.push(0x0f); out.push(1); out.push(3);
                out.extend_from_slice(&(element_types.len() as u64).to_le_bytes());
                for item in items.iter().rev() { jobs.push(Job::Value(item)); }
                jobs.push(Job::Count(items.len()));
                for kind in element_types.iter().rev() { jobs.push(Job::Type(kind)); }
            }
            ResourceValue::NominalValueV1(value) => {
                if !out.children(value.node_count()-1) { return; }
                out.push(0x11); out.extend_from_slice(value.canonical_body());
            }
            ResourceValue::SubjectPayloadV1(value) => {
                if !out.children(value.node_count()-1) { return; }
                out.push(0x13); out.extend_from_slice(value.canonical_body());
            }
            ResourceValue::SubjectHandleV1(value) => {
                out.push(0x12); out.extend_from_slice(&1u16.to_le_bytes());
                out.extend_from_slice(&value.name_type_identity);
                out.extend_from_slice(&value.world_identity);
                out.extend_from_slice(&value.subject_identity);
                out.extend_from_slice(&0u64.to_le_bytes());
            }
            ResourceValue::IdentityReference(value) => {
                out.push(0x0b);
                out.extend_from_slice(&value.profile_version.to_le_bytes());
                push_bytes(out,&value.target_type_identity);
                push_bytes(out,&value.owner_space_identity);
                push_bytes(out,&value.target_identity);
                out.extend_from_slice(&value.generation.to_le_bytes());
            },
            ResourceValue::RegisteredAtom(value) => {
                out.push(0x0d);
                out.extend_from_slice(&value.profile_version.to_le_bytes());
                push_bytes(out, &value.owner_registry_identity);
                push_bytes(out, &value.registered_atom_identity);
                push_str(out, &value.contract_profile);
                out.extend_from_slice(&value.contract_version.to_le_bytes());
                push_bytes(out, &value.contract_snapshot_identity);
                out.extend_from_slice(&value.contract_sha256);
            }
            ResourceValue::Bytes(value) => { out.push(0x0c); out.bytes_value(value); }
            ResourceValue::BasicTensorV1(carrier) => { out.push(0x0e); out.extend_from_slice(carrier); }
            ResourceValue::RationalV1(value) => out.rational_value(value),
            ResourceValue::Record { field_types, fields } => {
                if !out.children(fields.len().saturating_add(field_types.len())){return;}
                out.push(0x0a);
                out.push(1); // record payload version, not a change to old tags
                // Emit the same Record type header without a temporary clone.
                out.push(2);
                out.extend_from_slice(&(field_types.len() as u64).to_le_bytes());
                for (name, value) in fields.iter().rev() {
                    jobs.push(Job::Value(value)); jobs.push(Job::Name(name));
                }
                jobs.push(Job::Count(fields.len()));
                for (name, kind) in field_types.iter().rev() {
                    jobs.push(Job::Type(kind)); jobs.push(Job::Name(name));
                }
            }
            ResourceValue::None => {
                out.push(0x00);
            }
            ResourceValue::Bool(value) => {
                out.push(0x01);
                out.push(if *value { 1 } else { 0 });
            }
            ResourceValue::Fixed64(value) => {
                out.push(0x02);
                out.extend_from_slice(&value.raw_i64().to_le_bytes());
            }
            ResourceValue::Unit(value) => {
                out.push(0x03);
                out.extend_from_slice(&value.value.raw_i64().to_le_bytes());
                push_str(out, &value.dim.format());
            }
            ResourceValue::String(value) => {
                out.push(0x04);
                push_str(out, value);
            }
            ResourceValue::ResourceHandle(handle) => {
                out.push(0x05);
                out.extend_from_slice(&handle.raw().to_le_bytes());
            }
            ResourceValue::List(items) => {
                if !out.children(items.len()){return;}
                out.push(0x06);
                out.extend_from_slice(&(items.len() as u64).to_le_bytes());
                for item in items.iter().rev() { jobs.push(Job::Value(item)); }
            }
            ResourceValue::Set(items) => {
                if !out.children(items.len()){return;}
                out.push(0x07);
                out.extend_from_slice(&(items.len() as u64).to_le_bytes());
                for value in items.values().rev() { jobs.push(Job::Value(value)); }
            }
            ResourceValue::Map(entries) => {
                if let Some((capacity, len, ordered)) = Self::stream_v1_oldest_to_newest(entries) {
                    if !out.children(ordered.len()){return;}
                    out.push(0x09);
                    out.extend_from_slice(&capacity.to_le_bytes());
                    out.extend_from_slice(&len.to_le_bytes());
                    for value in ordered.into_iter().rev() { jobs.push(Job::Value(value)); }
                    continue;
                }
                if !out.children(entries.len().saturating_mul(2)){return;}
                out.push(0x08);
                out.extend_from_slice(&(entries.len() as u64).to_le_bytes());
                for entry in entries.values().rev() {
                    jobs.push(Job::Value(&entry.value));
                    jobs.push(Job::Value(&entry.key));
                }
            }
            }
        }
    }

    fn stream_v1_oldest_to_newest(
        entries: &BTreeMap<String, ResourceMapEntry>,
    ) -> Option<(u64, u64, Vec<&ResourceValue>)> {
        let mut schema: Option<&str> = None;
        let mut capacity: Option<u64> = None;
        let mut head: Option<u64> = None;
        let mut len: Option<u64> = None;
        let mut buffer: Option<&[ResourceValue]> = None;

        for entry in entries.values() {
            let key = match &entry.key {
                ResourceValue::String(key) => key.as_str(),
                _ => return None,
            };
            match key {
                "__schema" => {
                    let ResourceValue::String(value) = &entry.value else {
                        return None;
                    };
                    schema = Some(value.as_str());
                }
                "capacity" => {
                    capacity = Self::stream_v1_parse_u64(&entry.value);
                }
                "head" => {
                    head = Self::stream_v1_parse_u64(&entry.value);
                }
                "len" => {
                    len = Self::stream_v1_parse_u64(&entry.value);
                }
                "buffer" => {
                    let ResourceValue::List(items) = &entry.value else {
                        return None;
                    };
                    buffer = Some(items);
                }
                _ => return None,
            }
        }

        if schema.as_deref() != Some("ddn.stream.v1") {
            return None;
        }

        let buffer = buffer?;
        let mut capacity = capacity?;
        let mut len = len?;
        capacity = capacity.max(buffer.len() as u64);
        len = len.min(capacity);

        if capacity == 0 || len == 0 {
            return Some((capacity, 0, Vec::new()));
        }

        let mut head = head.unwrap_or_else(|| len.saturating_sub(1));
        head = head.min(capacity.saturating_sub(1));

        let oldest = (head + capacity + 1 - len) % capacity;
        let mut ordered = Vec::with_capacity(len as usize);
        for offset in 0..len {
            let idx = ((oldest + offset) % capacity) as usize;
            static EMPTY_STREAM_SLOT: ResourceValue = ResourceValue::None;
            ordered.push(buffer.get(idx).unwrap_or(&EMPTY_STREAM_SLOT));
        }

        Some((capacity, len, ordered))
    }

    fn stream_v1_parse_u64(value: &ResourceValue) -> Option<u64> {
        match value {
            ResourceValue::Fixed64(v) => {
                if v.frac_part() != 0 {
                    return None;
                }
                u64::try_from(v.int_part()).ok()
            }
            ResourceValue::Unit(v) => {
                if v.value.frac_part() != 0 {
                    return None;
                }
                u64::try_from(v.value.int_part()).ok()
            }
            ResourceValue::Bool(v) => Some(if *v { 1 } else { 0 }),
            ResourceValue::String(v) => v.parse::<u64>().ok(),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StateHash([u8; 32]);

impl StateHash {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        let mut out = String::with_capacity(64);
        for b in self.0 {
            use std::fmt::Write;
            let _ = write!(&mut out, "{:02x}", b);
        }
        out
    }
}

/// Data-only projection of the existing FRAME2 Subject table and section-B.
/// This is neither a registry nor restore/creation authority. The common
/// runtime owner validates both tables before putting them in a patch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubjectWorldStateV1 {
    pub subject_values: Vec<u8>,
    pub subject_custody: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct NuriWorld {
    // 결정성을 위해 HashMap 대신 BTreeMap 사용
    next_entity: u64,

    ecs: EcsStore,
    resources_json: BTreeMap<String, String>, // tag -> json

    // ✅ Fixed64 전용 Resource(터살림씨) 저장
    resources_fixed64: BTreeMap<String, Fixed64>, // tag -> Fixed64
    resources_handle: BTreeMap<String, ResourceHandle>, // tag -> handle
    resources_value: BTreeMap<String, ResourceValue>, // tag -> value
    subject_world: Option<SubjectWorldStateV1>,
}

impl NuriWorld {
    pub fn subject_world(&self) -> Option<&SubjectWorldStateV1> { self.subject_world.as_ref() }
    pub fn set_subject_world(&mut self, value: SubjectWorldStateV1) { self.subject_world=Some(value); }
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self) -> EntityId {
        let id = self.next_entity;
        self.next_entity = self.next_entity.wrapping_add(1);
        let entity = EntityId(id);
        self.ecs.ensure_entity(entity);
        entity
    }

    pub fn set_component_json(&mut self, entity: EntityId, tag: ComponentTag, json: String) {
        if entity.0 >= self.next_entity {
            self.next_entity = entity.0.wrapping_add(1);
        }
        self.ecs.set_component_json(entity, tag, json);
    }

    pub fn remove_component(&mut self, entity: EntityId, tag: &ComponentTag) {
        self.ecs.remove_component(entity, tag);
    }

    pub fn get_component_json(&self, entity: EntityId, tag: &ComponentTag) -> Option<String> {
        self.ecs.get_component_json(entity, tag)
    }

    pub fn query_entities_with_all_tags(&self, tags: &[ComponentTag]) -> Vec<EntityId> {
        self.ecs.query_entities_with_all_tags(tags)
    }

    pub fn set_resource_json(&mut self, tag: String, json: String) {
        self.resources_json.insert(tag, json);
    }

    pub fn get_resource_json(&self, tag: &str) -> Option<String> {
        self.resources_json.get(tag).cloned()
    }

    // ✅ Fixed64 Resource 접근자
    pub fn set_resource_fixed64(&mut self, tag: String, value: Fixed64) {
        self.resources_fixed64.insert(tag, value);
    }

    pub fn get_resource_fixed64(&self, tag: &str) -> Option<Fixed64> {
        self.resources_fixed64.get(tag).copied()
    }

    pub fn set_resource_handle(&mut self, tag: String, handle: ResourceHandle) {
        self.resources_handle.insert(tag, handle);
    }

    pub fn get_resource_handle(&self, tag: &str) -> Option<ResourceHandle> {
        self.resources_handle.get(tag).copied()
    }

    pub fn set_resource_value(&mut self, tag: String, value: ResourceValue) {
        self.resources_value.insert(tag, value);
    }

    pub fn get_resource_value(&self, tag: &str) -> Option<ResourceValue> {
        self.resources_value.get(tag).cloned()
    }

    pub fn resource_json_entries(&self) -> Vec<(String, String)> {
        self.resources_json
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Beta clean-break admission. The old lane remains readable only for
    /// historical impact evidence; it is not an executable resource producer.
    pub fn validate_typed_resource_lanes(&self) -> Result<(), &'static str> {
        if !self.resources_json.is_empty() {
            return Err("legacy resource_json 자원은 지원하지 않습니다");
        }
        if self.resources_fixed64.keys().any(|name|
            self.resources_handle.contains_key(name) || self.resources_value.contains_key(name))
            || self.resources_handle.keys().any(|name|self.resources_value.contains_key(name)) {
            return Err("같은 자원 이름이 여러 저장 통로에 있습니다");
        }
        Ok(())
    }

    pub fn validate_typed_resource_patch(&self, patch: &Patch) -> Result<(), &'static str> {
        self.validate_typed_resource_lanes()?;
        let mut lanes=BTreeMap::new();
        for name in self.resources_fixed64.keys(){lanes.insert(name.as_str(),0u8);}
        for name in self.resources_handle.keys(){lanes.insert(name.as_str(),1u8);}
        for name in self.resources_value.keys(){lanes.insert(name.as_str(),2u8);}
        for op in &patch.ops {
            let (name,lane)=match op {
                PatchOp::SetResourceJson {..}=>return Err("legacy resource_json patch는 지원하지 않습니다"),
                PatchOp::SetResourceFixed64 {tag,..}=>(tag.as_str(),0),
                // Faulting division emits its existing consequence, not a write.
                PatchOp::DivAssignResourceFixed64 {rhs,..} if rhs.raw_i64()==0=>continue,
                PatchOp::DivAssignResourceFixed64 {tag,..}=>(tag.as_str(),0),
                PatchOp::SetResourceHandle {tag,..}=>(tag.as_str(),1),
                PatchOp::SetResourceValue {tag,..}=>(tag.as_str(),2),
                _=>continue,
            };
            if lanes.insert(name,lane).is_some_and(|old|old!=lane) {
                return Err("같은 자원 이름이 여러 저장 통로에 있습니다");
            }
        }
        Ok(())
    }

    pub fn resource_fixed64_entries(&self) -> Vec<(String, Fixed64)> {
        self.resources_fixed64
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    pub fn resource_handle_entries(&self) -> Vec<(String, ResourceHandle)> {
        self.resources_handle
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    pub fn resource_value_entries(&self) -> Vec<(String, ResourceValue)> {
        self.resources_value
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn next_entity_cursor(&self) -> u64 {
        self.next_entity
    }

    pub fn restore_next_entity_cursor(&mut self, next_entity: u64) {
        self.next_entity = self.next_entity.max(next_entity);
    }

    pub fn component_json_entries(&self) -> Vec<(EntityId, ComponentTag, String)> {
        let mut entries = Vec::new();
        self.ecs.for_each_component_sorted(|entity, tag, json| {
            entries.push((entity, tag.clone(), json.to_string()));
        });
        entries
    }

    /// SSOT: state_hash는 BLAKE3(DetBin) 기반
    pub fn state_hash(&self) -> StateHash {
        let bytes = self.encode_canonical();
        let digest = hash(&bytes);
        StateHash::from_bytes(*digest.as_bytes())
    }

    /// state_hash 계산 시 리소스 키 접두어를 제외한다.
    /// 예: ["보개_"]를 넘기면 보개(view-only) 리소스는 해시에 포함되지 않는다.
    pub fn state_hash_excluding_resource_prefixes(&self, excluded_prefixes: &[&str]) -> StateHash {
        let bytes = self.encode_canonical_with_filter(excluded_prefixes);
        let digest = hash(&bytes);
        StateHash::from_bytes(*digest.as_bytes())
    }

    fn encode_canonical(&self) -> Vec<u8> {
        self.encode_canonical_with_filter(&[])
    }

    /// Existing state-hash preimage, exposed for exact-byte impact diagnostics.
    /// This is not admission or a legacy restore/migration authority.
    pub fn encode_canonical_with_filter(&self, excluded_prefixes: &[&str]) -> Vec<u8> {
        let include_tag = |tag: &str| -> bool {
            !excluded_prefixes
                .iter()
                .any(|prefix| tag.starts_with(prefix))
        };

        let mut out = Vec::<u8>::new();

        out.extend_from_slice(&self.next_entity.to_le_bytes());

        // resources_json (tag, json)
        let json_len = self
            .resources_json
            .iter()
            .filter(|(k, _)| include_tag(k))
            .count() as u64;
        out.extend_from_slice(&json_len.to_le_bytes());
        for (k, v) in &self.resources_json {
            if !include_tag(k) {
                continue;
            }
            push_str(&mut out, k);
            push_str(&mut out, v);
        }

        // ✅ resources_fixed64 (tag, raw_i64)
        let fixed_len = self
            .resources_fixed64
            .iter()
            .filter(|(k, _)| include_tag(k))
            .count() as u64;
        out.extend_from_slice(&fixed_len.to_le_bytes());
        for (k, v) in &self.resources_fixed64 {
            if !include_tag(k) {
                continue;
            }
            push_str(&mut out, k);
            out.extend_from_slice(&v.raw_i64().to_le_bytes());
        }

        // ✅ resources_handle (tag, u64)
        let handle_len = self
            .resources_handle
            .iter()
            .filter(|(k, _)| include_tag(k))
            .count() as u64;
        out.extend_from_slice(&handle_len.to_le_bytes());
        for (k, v) in &self.resources_handle {
            if !include_tag(k) {
                continue;
            }
            push_str(&mut out, k);
            out.extend_from_slice(&v.raw().to_le_bytes());
        }

        let value_len = self
            .resources_value
            .iter()
            .filter(|(k, _)| include_tag(k))
            .count() as u64;
        if value_len > 0 {
            out.extend_from_slice(&value_len.to_le_bytes());
            for (k, v) in &self.resources_value {
                if !include_tag(k) {
                    continue;
                }
                push_str(&mut out, k);
                v.encode_canon(&mut out);
            }
        }

        // components ((entity, tag), json)
        out.extend_from_slice(&self.ecs.component_count().to_le_bytes());
        self.ecs.for_each_component_sorted(|entity, tag, json| {
            out.extend_from_slice(&entity.0.to_le_bytes());
            push_str(&mut out, &tag.0);
            push_str(&mut out, json);
        });

        // Commit the actual, jointly checked Subject values and custody, not
        // just handles in resources. Existing worlds without this lane retain
        // their exact preimage. No hidden resource key or hash domain is used.
        if let Some(subjects)=&self.subject_world {
            push_bytes(&mut out,&subjects.subject_values);
            push_bytes(&mut out,&subjects.subject_custody);
        }

        out
    }
}

fn hex_identity(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn push_bytes(out: &mut impl CanonOutput, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(bytes);
}

fn push_str(out: &mut impl CanonOutput, s: &str) {
    let b = s.as_bytes();
    out.extend_from_slice(&(b.len() as u64).to_le_bytes());
    out.extend_from_slice(b);
}

// ---------- Seulgi Intent / Packet / InputSnapshot ----------

#[repr(u32)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeulgiIntent {
    None = 0,

    MoveTo { x: Fixed64, y: Fixed64 } = 1,

    Attack { target_id: u64 } = 2,

    Say { text: String } = 3,
}

impl SeulgiIntent {
    pub fn kind_u32(&self) -> u32 {
        match self {
            SeulgiIntent::None => 0,
            SeulgiIntent::MoveTo { .. } => 1,
            SeulgiIntent::Attack { .. } => 2,
            SeulgiIntent::Say { .. } => 3,
        }
    }

    pub fn stable_payload_hash(&self) -> u64 {
        let mut bytes = Vec::<u8>::new();
        bytes.extend_from_slice(&self.kind_u32().to_le_bytes());

        match self {
            SeulgiIntent::None => {}
            SeulgiIntent::MoveTo { x, y } => {
                bytes.extend_from_slice(&x.raw_i64().to_le_bytes());
                bytes.extend_from_slice(&y.raw_i64().to_le_bytes());
            }
            SeulgiIntent::Attack { target_id } => {
                bytes.extend_from_slice(&target_id.to_le_bytes());
            }
            SeulgiIntent::Say { text } => {
                push_str(&mut bytes, text);
            }
        }

        xxh3_64(&bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputSource {
    Person,       // 사람
    Seulgi,       // 슬기
    ExternalTask, // 밖일
    Schedule,     // 일정
    Relay,        // 이어전달
    #[allow(dead_code)]
    ScenarioExec, // 펼침실행: 예약, 미배선
}

impl InputSource {
    pub fn code_u8(self) -> u8 {
        match self {
            InputSource::Person => 0,
            InputSource::Seulgi => 1,
            InputSource::ExternalTask => 2,
            InputSource::Schedule => 3,
            InputSource::Relay => 4,
            InputSource::ScenarioExec => 5,
        }
    }

    pub fn from_code_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(InputSource::Person),
            1 => Some(InputSource::Seulgi),
            2 => Some(InputSource::ExternalTask),
            3 => Some(InputSource::Schedule),
            4 => Some(InputSource::Relay),
            5 => Some(InputSource::ScenarioExec),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            InputSource::Person => "사람",
            InputSource::Seulgi => "슬기",
            InputSource::ExternalTask => "밖일",
            InputSource::Schedule => "일정",
            InputSource::Relay => "이어전달",
            InputSource::ScenarioExec => "펼침실행",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeulgiPacket {
    pub agent_id: u64,
    pub recv_seq: u64,
    pub accepted_madi: u64,
    pub target_madi: u64,
    pub source: InputSource,
    pub intent: SeulgiIntent,
}

impl SeulgiPacket {
    pub fn stable_sort_key(&self) -> (u64, u64) {
        // Execution ordering hard-cut: (agent_id, recv_seq).
        // accepted_madi/target_madi are scheduling metadata and are not ordering keys.
        (self.agent_id, self.recv_seq)
    }
}

impl Ord for SeulgiPacket {
    fn cmp(&self, other: &Self) -> Ordering {
        self.stable_sort_key().cmp(&other.stable_sort_key())
    }
}
impl PartialOrd for SeulgiPacket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetEvent {
    pub sender: String,
    pub seq: u64,
    pub order_key: String,
    pub payload_detjson: String,
    pub source: InputSource,
}

impl NetEvent {
    pub fn stable_sort_key(&self) -> (&str, u64, &str, &str) {
        (
            self.sender.as_str(),
            self.seq,
            self.order_key.as_str(),
            self.payload_detjson.as_str(),
        )
    }
}

impl Ord for NetEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.stable_sort_key().cmp(&other.stable_sort_key())
    }
}

impl PartialOrd for NetEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputSnapshot {
    pub tick_id: TickId,
    pub dt: Fixed64,
    pub keys_pressed: u64,
    pub last_key_name: String,
    pub pointer_x_i32: i32,
    pub pointer_y_i32: i32,
    pub ai_injections: Vec<SeulgiPacket>,
    pub net_events: Vec<NetEvent>,
    pub frame_source: InputSource,
    pub rng_seed: u64,
}

pub const KEY_W: u64 = 1 << 0;
pub const KEY_A: u64 = 1 << 1;
pub const KEY_S: u64 = 1 << 2;
pub const KEY_D: u64 = 1 << 3;

impl InputSnapshot {
    pub fn is_key_pressed(&self, name: &str) -> bool {
        crate::input::is_key_pressed(self.keys_pressed, name)
    }
}

// ---------- Patch / TickFrame ----------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Patch {
    pub ops: Vec<PatchOp>,
    pub origin: Origin,
}

impl Default for Patch {
    fn default() -> Self {
        Self {
            ops: Vec::new(),
            origin: Origin::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Origin {
    Entity(EntityId),
    System(&'static str),
}

impl Origin {
    pub fn system(name: &'static str) -> Self {
        Self::System(name)
    }

    pub fn label(&self) -> String {
        match self {
            Origin::Entity(entity) => format!("entity:{}", entity.0),
            Origin::System(name) => format!("#system:{}", name),
        }
    }

    pub fn is_entity(&self, entity: EntityId) -> bool {
        matches!(self, Origin::Entity(e) if *e == entity)
    }
}

impl Default for Origin {
    fn default() -> Self {
        Origin::System("unknown")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatchOp {
    SetSubjectWorld { value: SubjectWorldStateV1 },
    SetComponentJson {
        entity: EntityId,
        tag: ComponentTag,
        json: String,
    },
    RemoveComponent {
        entity: EntityId,
        tag: ComponentTag,
    },
    SetResourceJson {
        tag: String,
        json: String,
    },

    // ✅ Fixed64 Resource 세팅
    SetResourceFixed64 {
        tag: String,
        value: Fixed64,
    },
    SetResourceHandle {
        tag: String,
        handle: ResourceHandle,
    },
    SetResourceValue {
        tag: String,
        value: ResourceValue,
    },

    // DivAssignResourceFixed64: emit arithmetic fault on div0, commit only on success.
    DivAssignResourceFixed64 {
        tag: String,
        rhs: Fixed64,
        tick_id: TickId,
        location: &'static str,
        source_span: Option<SourceSpan>,
        expr: Option<ExprTrace>,
    },
    EmitSignal {
        signal: Signal,
        targets: Vec<String>,
    },
    GuardViolation {
        entity: EntityId,
        rule_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TickFrame {
    pub snapshot: InputSnapshot,
    pub patch: Patch,
    pub commit_input: AtomicCommitInputRecord,
    pub commit_receipt: AtomicCommitReceipt,
    pub state_hash: StateHash,
}

// ---------- 6원소 Traits ----------

pub trait Sam {
    fn begin_tick(&mut self, tick_id: TickId) -> InputSnapshot;
    fn push_async_ai(
        &mut self,
        agent_id: u64,
        recv_seq: u64,
        accepted_madi: u64,
        target_madi: u64,
        intent: SeulgiIntent,
    );
}

pub trait Nuri {
    fn world(&self) -> &NuriWorld;
    fn world_mut(&mut self) -> &mut NuriWorld;

    fn apply_patch(&mut self, patch: &Patch, tick_id: TickId, sink: &mut dyn SignalSink);
}

pub trait Iyagi {
    fn run_startup(&mut self, world: &NuriWorld) -> Patch;
    fn run_update(&mut self, world: &NuriWorld, input: &InputSnapshot) -> Patch;
}

pub trait Bogae {
    fn render(&mut self, world: &NuriWorld, tick_id: TickId);
}

pub trait Geoul {
    fn record(&mut self, frame: &TickFrame);
    fn replay_next(&mut self) -> Option<TickFrame>;
}

pub struct SeulgiContext {
    pub tick_id: TickId,
    pub state_hash: StateHash,
}

pub trait Seulgi {
    fn observe(&self, world: &NuriWorld) -> SeulgiContext;
    fn think_async(&self, ctx: SeulgiContext);
}

// ---------- 기본 구현체들 ----------

pub struct DetSam {
    pub dt: Fixed64,
    pub keys_pressed: u64,
    pub last_key_name: String,
    pub pointer_x_i32: i32,
    pub pointer_y_i32: i32,
    pub rng_seed: u64,

    ai_queue: Vec<SeulgiPacket>,
    net_queue: Vec<NetEvent>,
}

impl DetSam {
    pub fn new(dt: Fixed64) -> Self {
        Self {
            dt,
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            rng_seed: 0,
            ai_queue: Vec::new(),
            net_queue: Vec::new(),
        }
    }

    pub fn push_net_event(
        &mut self,
        sender: impl Into<String>,
        seq: u64,
        order_key: impl Into<String>,
        payload_detjson: impl Into<String>,
    ) {
        self.net_queue.push(NetEvent {
            sender: sender.into(),
            seq,
            order_key: order_key.into(),
            payload_detjson: payload_detjson.into(),
            source: InputSource::ExternalTask,
        });
    }
}

impl Sam for DetSam {
    fn begin_tick(&mut self, tick_id: TickId) -> InputSnapshot {
        self.ai_queue.sort();
        let ai = core::mem::take(&mut self.ai_queue);
        self.net_queue.sort();
        let net_events = core::mem::take(&mut self.net_queue);

        InputSnapshot {
            tick_id,
            dt: self.dt,
            keys_pressed: self.keys_pressed,
            last_key_name: self.last_key_name.clone(),
            pointer_x_i32: self.pointer_x_i32,
            pointer_y_i32: self.pointer_y_i32,
            ai_injections: ai,
            net_events,
            frame_source: InputSource::Person,
            rng_seed: self.rng_seed,
        }
    }

    fn push_async_ai(
        &mut self,
        agent_id: u64,
        recv_seq: u64,
        accepted_madi: u64,
        target_madi: u64,
        intent: SeulgiIntent,
    ) {
        self.ai_queue.push(SeulgiPacket {
            agent_id,
            recv_seq,
            accepted_madi,
            target_madi,
            source: InputSource::Seulgi,
            intent,
        });
    }
}

pub struct DetNuri {
    world: NuriWorld,
}

impl DetNuri {
    pub fn new() -> Self {
        Self {
            world: NuriWorld::new(),
        }
    }

    pub fn state_hash(&self) -> StateHash {
        self.world.state_hash()
    }
}

impl Nuri for DetNuri {
    fn world(&self) -> &NuriWorld {
        &self.world
    }

    fn world_mut(&mut self) -> &mut NuriWorld {
        &mut self.world
    }

    fn apply_patch(&mut self, patch: &Patch, tick_id: TickId, sink: &mut dyn SignalSink) {
        if let Err(reason)=self.world.validate_typed_resource_patch(patch) {
            sink.emit(Signal::Diag {event:DiagEvent {
                madi:tick_id,seq:0,fault_id:"PATCH_SCHEMA".into(),rule_id:String::new(),
                reason:"PATCH_SCHEMA".into(),sub_reason:None,mode:None,contract_kind:None,
                origin:patch.origin.label(),targets:Vec::new(),
                sam_hash:None,source_span:None,expr:None,message:Some(reason.into()),
            }});
            return;
        }
        let mut diag_seq = 0u64;
        let mut guard_entities = Vec::new();
        for op in &patch.ops {
            if let PatchOp::GuardViolation { entity, rule_id } = op {
                guard_entities.push((*entity, rule_id.clone()));
                let event = DiagEvent {
                    madi: tick_id,
                    seq: diag_seq,
                    fault_id: "GUARD_VIOLATION".to_string(),
                    rule_id: rule_id.clone(),
                    reason: "GUARD_VIOLATION".to_string(),
                    sub_reason: None,
                    mode: None,
                    contract_kind: None,
                    origin: Origin::Entity(*entity).label(),
                    targets: vec![format!("entity:{}", entity.0)],
                    sam_hash: None,
                    source_span: None,
                    expr: None,
                    message: None,
                };
                sink.emit(Signal::Diag { event });
                diag_seq += 1;
            }
        }

        if !guard_entities.is_empty() {
            guard_entities.sort_by_key(|(entity, _)| entity.0);
            guard_entities.dedup_by_key(|(entity, _)| entity.0);
            for (entity, _) in &guard_entities {
                self.world.set_component_json(
                    *entity,
                    ComponentTag("#규칙위반".to_string()),
                    "참".to_string(),
                );
                self.world.set_component_json(
                    *entity,
                    ComponentTag("#휴면".to_string()),
                    "참".to_string(),
                );
            }
        }

        let skip_assignments = guard_entities
            .iter()
            .any(|(entity, _)| patch.origin.is_entity(*entity));

        // 이 commit의 다른 op는 resources_value를 읽지 않는다. 공개 Patch는
        // 그대로 순회하되 즉시 덮일 payload의 중간 deep clone만 생략한다.
        let mut last_value_writes = BTreeMap::new();
        if !skip_assignments {
            for (index, op) in patch.ops.iter().enumerate() {
                if let PatchOp::SetResourceValue { tag, .. } = op {
                    last_value_writes.insert(tag.as_str(), index);
                }
            }
        }
        for (index, op) in patch.ops.iter().enumerate() {
            match op {
                PatchOp::SetComponentJson { entity, tag, json } => {
                    if skip_assignments {
                        continue;
                    }
                    self.world
                        .set_component_json(*entity, tag.clone(), json.clone());
                }
                PatchOp::RemoveComponent { entity, tag } => {
                    if skip_assignments {
                        continue;
                    }
                    self.world.remove_component(*entity, tag);
                }
                PatchOp::SetResourceJson { tag, json } => {
                    if skip_assignments {
                        continue;
                    }
                    self.world.set_resource_json(tag.clone(), json.clone());
                }

                PatchOp::SetResourceFixed64 { tag, value } => {
                    if skip_assignments {
                        continue;
                    }
                    self.world.set_resource_fixed64(tag.clone(), *value);
                }
                PatchOp::SetResourceHandle { tag, handle } => {
                    if skip_assignments {
                        continue;
                    }
                    self.world.set_resource_handle(tag.clone(), *handle);
                }
                PatchOp::SetResourceValue { tag, value } => {
                    if skip_assignments || last_value_writes.get(tag.as_str()) != Some(&index) {
                        continue;
                    }
                    self.world.set_resource_value(tag.clone(), value.clone());
                }
                PatchOp::SetSubjectWorld { value } => {
                    if !skip_assignments { self.world.set_subject_world(value.clone()); }
                }

                PatchOp::DivAssignResourceFixed64 {
                    tag,
                    rhs,
                    tick_id: op_tick_id,
                    location,
                    source_span,
                    expr,
                } => {
                    if skip_assignments {
                        continue;
                    }
                    // Missing resource defaults to zero; commit only if division succeeds.
                    let cur = self
                        .world
                        .get_resource_fixed64(tag)
                        .unwrap_or(Fixed64::ZERO);
                    let ctx = FaultContext {
                        tick_id: *op_tick_id,
                        location,
                        source_span: source_span.clone(),
                        expr: expr.clone(),
                    };
                    match cur.try_div(*rhs) {
                        Ok(next) => self.world.set_resource_fixed64(tag.clone(), next),
                        Err(kind) => {
                            sink.emit(Signal::ArithmeticFault {
                                ctx,
                                kind: kind.clone(),
                            });
                            let reason = match kind {
                                crate::signals::ArithmeticFaultKind::DimensionMismatch {
                                    ..
                                } => "UNIT_MISMATCH",
                                _ => "ARITH_FAULT",
                            };
                            let sub_reason = match kind {
                                crate::signals::ArithmeticFaultKind::DimensionMismatch {
                                    ..
                                } => Some("DIM_MISMATCH".to_string()),
                                _ => Some("DIV0".to_string()),
                            };
                            let event = DiagEvent {
                                madi: tick_id,
                                seq: diag_seq,
                                fault_id: reason.to_string(),
                                rule_id: String::new(),
                                reason: reason.to_string(),
                                sub_reason,
                                mode: None,
                                contract_kind: None,
                                origin: patch.origin.label(),
                                targets: vec![format!("resource:{}", tag)],
                                sam_hash: None,
                                source_span: source_span.clone(),
                                expr: expr.clone(),
                                message: None,
                            };
                            sink.emit(Signal::Diag { event });
                            diag_seq += 1;
                        }
                    }
                }
                PatchOp::EmitSignal { signal, targets } => match signal {
                    Signal::Diag { event } => {
                        let mut event = event.clone();
                        event.madi = tick_id;
                        event.seq = diag_seq;
                        if event.targets.is_empty() {
                            event.targets = if targets.is_empty() {
                                vec!["unknown".to_string()]
                            } else {
                                targets.clone()
                            };
                        }
                        sink.emit(Signal::Diag { event });
                        diag_seq += 1;
                    }
                    Signal::ArithmeticFault { ctx, kind } => {
                        sink.emit(signal.clone());
                        let reason = match kind {
                            crate::signals::ArithmeticFaultKind::DimensionMismatch { .. } => {
                                "UNIT_MISMATCH"
                            }
                            _ => "ARITH_FAULT",
                        };
                        let sub_reason = match kind {
                            crate::signals::ArithmeticFaultKind::DimensionMismatch { .. } => {
                                Some("DIM_MISMATCH".to_string())
                            }
                            _ => Some("DIV0".to_string()),
                        };
                        let diag_targets = if targets.is_empty() {
                            vec!["unknown".to_string()]
                        } else {
                            targets.clone()
                        };
                        let event = DiagEvent {
                            madi: tick_id,
                            seq: diag_seq,
                            fault_id: reason.to_string(),
                            rule_id: String::new(),
                            reason: reason.to_string(),
                            sub_reason,
                            mode: None,
                            contract_kind: None,
                            origin: patch.origin.label(),
                            targets: diag_targets,
                            sam_hash: None,
                            source_span: ctx.source_span.clone(),
                            expr: ctx.expr.clone(),
                            message: None,
                        };
                        sink.emit(Signal::Diag { event });
                        diag_seq += 1;
                    }
                    _ => sink.emit(signal.clone()),
                },
                PatchOp::GuardViolation { .. } => {}
            }
        }
    }
}

// 최적화 전 순차 적용 본문을 테스트 전용 참조로 보존한다.
#[cfg(test)]
fn apply_patch_sequential_reference(
    nuri: &mut DetNuri,
    patch: &Patch,
    tick_id: TickId,
    sink: &mut dyn SignalSink,
) {
    let mut diag_seq = 0u64;
    let mut guard_entities = Vec::new();
    for op in &patch.ops {
        if let PatchOp::GuardViolation { entity, rule_id } = op {
            guard_entities.push((*entity, rule_id.clone()));
            let event = DiagEvent {
                madi: tick_id,
                seq: diag_seq,
                fault_id: "GUARD_VIOLATION".to_string(),
                rule_id: rule_id.clone(),
                reason: "GUARD_VIOLATION".to_string(),
                sub_reason: None,
                mode: None,
                contract_kind: None,
                origin: Origin::Entity(*entity).label(),
                targets: vec![format!("entity:{}", entity.0)],
                sam_hash: None,
                source_span: None,
                expr: None,
                message: None,
            };
            sink.emit(Signal::Diag { event });
            diag_seq += 1;
        }
    }

    if !guard_entities.is_empty() {
        guard_entities.sort_by_key(|(entity, _)| entity.0);
        guard_entities.dedup_by_key(|(entity, _)| entity.0);
        for (entity, _) in &guard_entities {
            nuri.world.set_component_json(
                *entity,
                ComponentTag("#규칙위반".to_string()),
                "참".to_string(),
            );
            nuri.world.set_component_json(
                *entity,
                ComponentTag("#휴면".to_string()),
                "참".to_string(),
            );
        }
    }

    let skip_assignments = guard_entities
        .iter()
        .any(|(entity, _)| patch.origin.is_entity(*entity));

    for op in &patch.ops {
        match op {
            PatchOp::SetComponentJson { entity, tag, json } => {
                if skip_assignments {
                    continue;
                }
                nuri.world
                    .set_component_json(*entity, tag.clone(), json.clone());
            }
            PatchOp::RemoveComponent { entity, tag } => {
                if skip_assignments {
                    continue;
                }
                nuri.world.remove_component(*entity, tag);
            }
            PatchOp::SetResourceJson { tag, json } => {
                if skip_assignments {
                    continue;
                }
                nuri.world.set_resource_json(tag.clone(), json.clone());
            }

            PatchOp::SetResourceFixed64 { tag, value } => {
                if skip_assignments {
                    continue;
                }
                nuri.world.set_resource_fixed64(tag.clone(), *value);
            }
            PatchOp::SetResourceHandle { tag, handle } => {
                if skip_assignments {
                    continue;
                }
                nuri.world.set_resource_handle(tag.clone(), *handle);
            }
            PatchOp::SetResourceValue { tag, value } => {
                if skip_assignments {
                    continue;
                }
                nuri.world.set_resource_value(tag.clone(), value.clone());
            }

            PatchOp::SetSubjectWorld { value } => {
                if !skip_assignments { nuri.world.set_subject_world(value.clone()); }
            }

            PatchOp::DivAssignResourceFixed64 {
                tag,
                rhs,
                tick_id: op_tick_id,
                location,
                source_span,
                expr,
            } => {
                if skip_assignments {
                    continue;
                }
                // Missing resource defaults to zero; commit only if division succeeds.
                let cur = nuri
                    .world
                    .get_resource_fixed64(tag)
                    .unwrap_or(Fixed64::ZERO);
                let ctx = FaultContext {
                    tick_id: *op_tick_id,
                    location,
                    source_span: source_span.clone(),
                    expr: expr.clone(),
                };
                match cur.try_div(*rhs) {
                    Ok(next) => nuri.world.set_resource_fixed64(tag.clone(), next),
                    Err(kind) => {
                        sink.emit(Signal::ArithmeticFault {
                            ctx,
                            kind: kind.clone(),
                        });
                        let reason = match kind {
                            crate::signals::ArithmeticFaultKind::DimensionMismatch { .. } => {
                                "UNIT_MISMATCH"
                            }
                            _ => "ARITH_FAULT",
                        };
                        let sub_reason = match kind {
                            crate::signals::ArithmeticFaultKind::DimensionMismatch { .. } => {
                                Some("DIM_MISMATCH".to_string())
                            }
                            _ => Some("DIV0".to_string()),
                        };
                        let event = DiagEvent {
                            madi: tick_id,
                            seq: diag_seq,
                            fault_id: reason.to_string(),
                            rule_id: String::new(),
                            reason: reason.to_string(),
                            sub_reason,
                            mode: None,
                            contract_kind: None,
                            origin: patch.origin.label(),
                            targets: vec![format!("resource:{}", tag)],
                            sam_hash: None,
                            source_span: source_span.clone(),
                            expr: expr.clone(),
                            message: None,
                        };
                        sink.emit(Signal::Diag { event });
                        diag_seq += 1;
                    }
                }
            }
            PatchOp::EmitSignal { signal, targets } => match signal {
                Signal::Diag { event } => {
                    let mut event = event.clone();
                    event.madi = tick_id;
                    event.seq = diag_seq;
                    if event.targets.is_empty() {
                        event.targets = if targets.is_empty() {
                            vec!["unknown".to_string()]
                        } else {
                            targets.clone()
                        };
                    }
                    sink.emit(Signal::Diag { event });
                    diag_seq += 1;
                }
                Signal::ArithmeticFault { ctx, kind } => {
                    sink.emit(signal.clone());
                    let reason = match kind {
                        crate::signals::ArithmeticFaultKind::DimensionMismatch { .. } => {
                            "UNIT_MISMATCH"
                        }
                        _ => "ARITH_FAULT",
                    };
                    let sub_reason = match kind {
                        crate::signals::ArithmeticFaultKind::DimensionMismatch { .. } => {
                            Some("DIM_MISMATCH".to_string())
                        }
                        _ => Some("DIV0".to_string()),
                    };
                    let diag_targets = if targets.is_empty() {
                        vec!["unknown".to_string()]
                    } else {
                        targets.clone()
                    };
                    let event = DiagEvent {
                        madi: tick_id,
                        seq: diag_seq,
                        fault_id: reason.to_string(),
                        rule_id: String::new(),
                        reason: reason.to_string(),
                        sub_reason,
                        mode: None,
                        contract_kind: None,
                        origin: patch.origin.label(),
                        targets: diag_targets,
                        sam_hash: None,
                        source_span: ctx.source_span.clone(),
                        expr: ctx.expr.clone(),
                        message: None,
                    };
                    sink.emit(Signal::Diag { event });
                    diag_seq += 1;
                }
                _ => sink.emit(signal.clone()),
            },
            PatchOp::GuardViolation { .. } => {}
        }
    }
}

pub struct InMemoryGeoul {
    q: VecDeque<TickFrame>,
}

impl InMemoryGeoul {
    pub fn new() -> Self {
        Self { q: VecDeque::new() }
    }

    pub fn len(&self) -> usize {
        self.q.len()
    }
}

impl Geoul for InMemoryGeoul {
    fn record(&mut self, frame: &TickFrame) {
        self.q.push_back(frame.clone());
    }

    fn replay_next(&mut self) -> Option<TickFrame> {
        self.q.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::{NuriWorld, ResourceMapEntry, ResourceValue};
    use crate::{Fixed64, Unit, UnitValue};

    #[test]
    fn saved_tree_encoder_preserves_record_wire_order() {
        use super::RecordFieldType;
        let value = ResourceValue::Record {
            field_types: [("a".into(), RecordFieldType::Applied {
                name: "L".into(), args: vec![RecordFieldType::Named { name: "N".into() }],
            })].into(),
            fields: [("a".into(), ResourceValue::List(vec![ResourceValue::Bool(true), ResourceValue::None]))].into(),
        };
        // Independently assembled legacy wire: record/version/type/count,
        // named field, Applied/N type, then the value table and List children.
        let mut expected = vec![0x0a, 1, 2];
        expected.extend_from_slice(&1u64.to_le_bytes());
        for bytes in [b"a".as_slice(), b"L".as_slice()] {
            if bytes == b"L" { expected.push(1); }
            expected.extend_from_slice(&1u64.to_le_bytes()); expected.extend_from_slice(bytes);
        }
        expected.extend_from_slice(&1u64.to_le_bytes()); expected.push(0);
        expected.extend_from_slice(&1u64.to_le_bytes()); expected.push(b'N');
        expected.extend_from_slice(&1u64.to_le_bytes());
        expected.extend_from_slice(&1u64.to_le_bytes()); expected.push(b'a');
        expected.push(6); expected.extend_from_slice(&2u64.to_le_bytes());
        expected.extend_from_slice(&[1, 1, 0]);
        let mut actual = Vec::new(); value.encode_canon(&mut actual);
        assert_eq!(actual, expected);
    }

    #[test]
    fn saved_tree_encoder_deep_list_and_record_type_use_heap_jobs() {
        use super::RecordFieldType;
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            const DEPTH: usize = 20_000;
            let mut value = ResourceValue::None;
            let mut kind = RecordFieldType::Named { name: "N".into() };
            for _ in 0..DEPTH {
                value = ResourceValue::List(vec![value]);
                kind = RecordFieldType::Applied { name: "L".into(), args: vec![kind] };
            }
            let mut bytes = Vec::new(); value.encode_canon(&mut bytes);
            let list_bytes = bytes;
            let list_key = value.canon_key();
            let stream = ResourceValue::stream_v1_from_ring_buffer(1, 0, 1, vec![value]);
            let mut stream_bytes = Vec::new(); stream.encode_canon(&mut stream_bytes);
            let record = ResourceValue::Record {
                field_types: [("a".into(), kind)].into(), fields: [("a".into(), ResourceValue::None)].into(),
            };
            let mut bytes = Vec::new(); record.encode_canon(&mut bytes);
            // Exercise ordinary ownership cleanup after encoding, not a
            // test-only manual walk that bypasses the product Drop owner.
            drop(stream);
            drop(record);
            assert_eq!(list_bytes.len(), DEPTH * 9 + 1);
            for node in list_bytes[..DEPTH * 9].chunks_exact(9) {
                assert_eq!(node, &[6, 1, 0, 0, 0, 0, 0, 0, 0]);
            }
            assert_eq!(bytes.len(), DEPTH * 18 + 48);
            assert_eq!(stream_bytes.len(), list_bytes.len() + 17);
            assert_eq!(&stream_bytes[17..], list_bytes);
            assert_eq!(list_key, format!("{}없음{}", "차림[".repeat(DEPTH), "]".repeat(DEPTH)));
        }).unwrap().join().unwrap();
    }

    #[test]
    fn saved_tree_encoder_stream_padding_preserves_wire() {
        let value = ResourceValue::stream_v1_from_ring_buffer(
            3, 2, 3, vec![ResourceValue::Bool(true)],
        );
        let mut expected = vec![9];
        expected.extend_from_slice(&3u64.to_le_bytes());
        expected.extend_from_slice(&3u64.to_le_bytes());
        expected.extend_from_slice(&[1, 1, 0, 0]);
        let mut actual = Vec::new(); value.encode_canon(&mut actual);
        assert_eq!(actual, expected);
    }

    #[test]
    fn saved_tree_canonical_keys_preserve_collection_text_and_order() {
        let value = ResourceValue::map_from_entries(vec![
            ResourceMapEntry { key: ResourceValue::String("b".into()), value: ResourceValue::List(vec![ResourceValue::None, ResourceValue::Bool(false)]) },
            ResourceMapEntry { key: ResourceValue::String("a".into()), value: ResourceValue::set_from_values(vec![ResourceValue::Bool(true), ResourceValue::String("x\n".into())]) },
        ]);
        assert_eq!(value.canon_key(), "짝맞춤{\"a\"=>모음{\"x\\n\", 참}, \"b\"=>차림[없음, 거짓]}");
    }

    #[test]
    fn saved_tree_clone_equality_and_explicit_cleanup_are_stack_safe() {
        use super::RecordFieldType;
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut value = ResourceValue::None;
            let mut kind = RecordFieldType::Named { name: "N".into() };
            for _ in 0..20_000 {
                value = ResourceValue::List(vec![value]);
                kind = RecordFieldType::Applied { name: "L".into(), args: vec![kind] };
            }
            // Both Map key/value and Record type/value children participate.
            // Clone must preserve the storage index too, not reconstruct it.
            let tree = ResourceValue::Map([("stored-index".into(), ResourceMapEntry {
                key: value,
                value: ResourceValue::Record {
                    field_types: [("a".into(), kind)].into(),
                    fields: [("a".into(), ResourceValue::Bool(true))].into(),
                },
            })].into());
            let mut copy = tree.clone();
            let same = tree == copy;
            let mut original_bytes = Vec::new(); tree.encode_canon(&mut original_bytes);
            let mut copied_bytes = Vec::new(); copy.encode_canon(&mut copied_bytes);
            let ResourceValue::Map(entries) = &mut copy else { unreachable!() };
            let ResourceValue::Record { field_types, .. } = &mut entries.get_mut("stored-index").unwrap().value else { unreachable!() };
            let mut cursor = field_types.get_mut("a").unwrap();
            loop {
                match cursor {
                    RecordFieldType::Applied { args, .. } => cursor = &mut args[0],
                    RecordFieldType::Named { name } => { *name = "different".into(); break; }
                    _ => unreachable!(),
                }
            }
            let different = tree != copy;
            tree.drop_stack_safe(); copy.drop_stack_safe();
            assert!(same); assert!(different); assert_eq!(original_bytes, copied_bytes);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn saved_tree_clone_keeps_every_leaf_and_storage_key_without_admission() {
        use super::{IdentityReferenceValueV1, RecordFieldType, RegisteredAtomValueV1};
        let leaves = vec![
            ResourceValue::None, ResourceValue::Bool(true), ResourceValue::Fixed64(Fixed64::from_raw_i64(-1)),
            ResourceValue::String("a\0\n".into()), ResourceValue::Bytes(crate::bytes::ByteSequence::new(vec![0, 255])),
            ResourceValue::ResourceHandle(crate::ResourceHandle::from_raw(9)),
            ResourceValue::Unit(UnitValue::new(Fixed64::ONE, Unit::Second)),
            ResourceValue::BasicTensorV1(vec![255]), // trait cloning is not Tensor admission
            ResourceValue::IdentityReference(Box::new(IdentityReferenceValueV1 { profile_version: 9, target_type_identity: vec![2], owner_space_identity: vec![3], target_identity: vec![4], generation: 8 })),
            ResourceValue::RegisteredAtom(Box::new(RegisteredAtomValueV1 { profile_version: 9, owner_registry_identity: vec![1], registered_atom_identity: vec![2], contract_profile: "unadmitted".into(), contract_version: 7, contract_snapshot_identity: vec![3], contract_sha256: [4; 32] })),
        ];
        for leaf in leaves {
            let tree = ResourceValue::Record {
                field_types: [("a".into(), RecordFieldType::Named { name: "unadmitted".into() })].into(),
                fields: [("a".into(), ResourceValue::Set([("exact-index".into(), leaf)].into()))].into(),
            };
            let copy = tree.clone();
            assert_eq!(tree, copy);
            let mut a = Vec::new(); let mut b = Vec::new(); tree.encode_canon(&mut a); copy.encode_canon(&mut b);
            assert_eq!(a, b);
        }
    }

    fn assert_value_commit_matches_sequential(patch: &super::Patch) {
        use super::{DetNuri, Nuri};
        use crate::signals::VecSignalSink;
        let mut reference = DetNuri::new();
        reference
            .world
            .set_resource_value("shared".into(), ResourceValue::String("이전".into()));
        reference
            .world
            .set_resource_fixed64("prior_fixed".into(), Fixed64::from_i64(12));
        let saved = reference.world.clone();
        let mut optimized = DetNuri {
            world: saved.clone(),
        };
        let original_patch = patch.clone();
        let original_patch_bytes = format!("{patch:?}").into_bytes();
        let mut expected_sink = VecSignalSink::default();
        let mut actual_sink = VecSignalSink::default();
        // R1.1 intentionally retires the cross-lane/legacy candidate. Keep
        // its rejection test; do not compare it with the old accepting oracle.
        if saved.validate_typed_resource_patch(patch).is_err() {
            optimized.apply_patch(patch,17,&mut actual_sink);
            assert_eq!(optimized.world.encode_canonical(),saved.encode_canonical());
            assert!(actual_sink.diag_events.iter().any(|event|event.fault_id=="PATCH_SCHEMA"));
            assert_eq!(patch,&original_patch);
            return;
        }
        super::apply_patch_sequential_reference(&mut reference, patch, 17, &mut expected_sink);
        optimized.apply_patch(patch, 17, &mut actual_sink);
        assert_eq!(
            optimized.world.encode_canonical(),
            reference.world.encode_canonical()
        );
        assert_eq!(optimized.state_hash(), reference.state_hash());
        assert_eq!(actual_sink.signals, expected_sink.signals);
        assert_eq!(actual_sink.diag_events, expected_sink.diag_events);
        assert_eq!(patch, &original_patch, "공개 Patch의 전체 op와 순서 보존");
        assert_eq!(format!("{patch:?}").into_bytes(), original_patch_bytes);
        // 기존 소유 snapshot 복원 후 같은 Patch를 적용하는 결과도 보존한다.
        reference.world = saved.clone();
        optimized.world = saved;
        let mut expected_sink = VecSignalSink::default();
        let mut actual_sink = VecSignalSink::default();
        super::apply_patch_sequential_reference(&mut reference, patch, 18, &mut expected_sink);
        optimized.apply_patch(patch, 18, &mut actual_sink);
        assert_eq!(
            optimized.world.encode_canonical(),
            reference.world.encode_canonical()
        );
        assert_eq!(actual_sink.signals, expected_sink.signals);
        assert_eq!(actual_sink.diag_events, expected_sink.diag_events);
    }

    #[test]
    fn value_commit_last_write_preserves_repeated_payloads_and_other_tags() {
        use super::{Origin, Patch, PatchOp};
        let mut ops = Vec::new();
        for count in 0..128 {
            ops.push(PatchOp::SetResourceValue {
                tag: "output_log".into(),
                value: ResourceValue::List(
                    (0..count)
                        .map(|index| ResourceValue::String(format!("원 관찰 {index}")))
                        .collect(),
                ),
            });
            ops.push(PatchOp::SetResourceValue {
                tag: "shared".into(),
                value: ResourceValue::Fixed64(Fixed64::from_i64(count)),
            });
        }
        assert_value_commit_matches_sequential(&Patch {
            ops,
            origin: Origin::system("ddn"),
        });
    }

    #[test]
    fn value_commit_last_write_keeps_other_resource_maps_and_signal_order() {
        use super::{Origin, Patch, PatchOp};
        use crate::resource::ResourceHandle;
        use crate::signals::{ArithmeticFaultKind, FaultContext, Signal};
        let patch = Patch {
            origin: Origin::system("ddn"),
            ops: vec![
                PatchOp::SetResourceValue {
                    tag: "shared".into(),
                    value: ResourceValue::String("초기".into()),
                },
                PatchOp::SetResourceJson {
                    tag: "shared".into(),
                    json: "독립 글".into(),
                },
                PatchOp::SetResourceFixed64 {
                    tag: "shared".into(),
                    value: Fixed64::from_i64(10),
                },
                PatchOp::EmitSignal {
                    signal: Signal::Alrim { name: "first" },
                    targets: vec![],
                },
                PatchOp::DivAssignResourceFixed64 {
                    tag: "shared".into(),
                    rhs: Fixed64::from_i64(2),
                    tick_id: 17,
                    location: "test",
                    source_span: None,
                    expr: None,
                },
                PatchOp::SetResourceValue {
                    tag: "shared".into(),
                    value: ResourceValue::Bool(true),
                },
                PatchOp::SetResourceHandle {
                    tag: "shared".into(),
                    handle: ResourceHandle::from_raw(7),
                },
                PatchOp::DivAssignResourceFixed64 {
                    tag: "shared".into(),
                    rhs: Fixed64::ZERO,
                    tick_id: 17,
                    location: "test",
                    source_span: None,
                    expr: None,
                },
                PatchOp::EmitSignal {
                    signal: Signal::ArithmeticFault {
                        ctx: FaultContext {
                            tick_id: 17,
                            location: "test",
                            source_span: None,
                            expr: None,
                        },
                        kind: ArithmeticFaultKind::DivByZero,
                    },
                    targets: vec!["shared".into()],
                },
                PatchOp::SetResourceValue {
                    tag: "shared".into(),
                    value: ResourceValue::None,
                },
                PatchOp::EmitSignal {
                    signal: Signal::Alrim { name: "last" },
                    targets: vec![],
                },
            ],
        };
        assert_value_commit_matches_sequential(&patch);
    }

    #[test]
    fn value_commit_last_write_preserves_guard_origin_and_component_effects() {
        use super::{ComponentTag, EntityId, Origin, Patch, PatchOp};
        use crate::signals::Signal;
        for origin in [
            Origin::system("ddn"),
            Origin::Entity(EntityId(3)),
            Origin::Entity(EntityId(4)),
        ] {
            let patch = Patch {
                origin,
                ops: vec![
                    PatchOp::SetResourceValue {
                        tag: "shared".into(),
                        value: ResourceValue::Bool(false),
                    },
                    PatchOp::SetComponentJson {
                        entity: EntityId(3),
                        tag: ComponentTag("기존".into()),
                        json: "1".into(),
                    },
                    PatchOp::GuardViolation {
                        entity: EntityId(3),
                        rule_id: "원 규칙".into(),
                    },
                    PatchOp::EmitSignal {
                        signal: Signal::Alrim { name: "guarded" },
                        targets: vec![],
                    },
                    PatchOp::SetResourceValue {
                        tag: "shared".into(),
                        value: ResourceValue::Bool(true),
                    },
                    PatchOp::GuardViolation {
                        entity: EntityId(3),
                        rule_id: "중복 규칙".into(),
                    },
                    PatchOp::RemoveComponent {
                        entity: EntityId(3),
                        tag: ComponentTag("기존".into()),
                    },
                ],
            };
            assert_value_commit_matches_sequential(&patch);
        }
        assert_value_commit_matches_sequential(&Patch::default());
    }

    #[test]
    fn state_hash_filter_excludes_bogae_prefix() {
        let mut world = NuriWorld::new();
        world.set_resource_fixed64("x".to_string(), Fixed64::from_i64(1));
        world.set_resource_json("보개_색".to_string(), "#111111".to_string());

        let full_a = world.state_hash();
        let filtered_a = world.state_hash_excluding_resource_prefixes(&["보개_"]);

        world.set_resource_json("보개_색".to_string(), "#222222".to_string());
        let full_b = world.state_hash();
        let filtered_b = world.state_hash_excluding_resource_prefixes(&["보개_"]);

        assert_ne!(full_a.to_hex(), full_b.to_hex());
        assert_eq!(filtered_a.to_hex(), filtered_b.to_hex());

        world.set_resource_fixed64("x".to_string(), Fixed64::from_i64(2));
        let filtered_c = world.state_hash_excluding_resource_prefixes(&["보개_"]);
        assert_ne!(filtered_b.to_hex(), filtered_c.to_hex());
    }

    #[test]
    fn state_hash_filter_empty_prefix_is_identity() {
        let mut world = NuriWorld::new();
        world.set_resource_fixed64("a".to_string(), Fixed64::from_i64(10));
        world.set_resource_json("보개_디버그".to_string(), "on".to_string());
        assert_eq!(
            world.state_hash().to_hex(),
            world.state_hash_excluding_resource_prefixes(&[]).to_hex()
        );
    }

    #[test]
    fn typed_state_hash_uses_existing_unit_serialization_for_mol() {
        let mut amount_world = NuriWorld::new();
        amount_world.set_resource_value(
            "양".to_string(),
            ResourceValue::Unit(UnitValue::new(Fixed64::from_i64(2), Unit::Mole)),
        );
        let mut same_amount_world = NuriWorld::new();
        same_amount_world.set_resource_value(
            "양".to_string(),
            ResourceValue::Unit(UnitValue::new(Fixed64::from_i64(2), Unit::Mole)),
        );
        let mut mass_world = NuriWorld::new();
        mass_world.set_resource_value(
            "양".to_string(),
            ResourceValue::Unit(UnitValue::new(Fixed64::from_i64(2), Unit::Kilogram)),
        );

        assert_eq!(
            amount_world.state_hash().to_hex(),
            same_amount_world.state_hash().to_hex()
        );
        assert_ne!(
            amount_world.state_hash().to_hex(),
            mass_world.state_hash().to_hex()
        );
    }

    #[test]
    fn stream_v1_hash_uses_oldest_to_newest_order() {
        let x = ResourceValue::String("x".to_string());
        let y = ResourceValue::String("y".to_string());
        let z = ResourceValue::String("z".to_string());
        let none = ResourceValue::None;

        let stream_a = ResourceValue::stream_v1_from_ring_buffer(
            4,
            2,
            3,
            vec![x.clone(), y.clone(), z.clone(), none.clone()],
        );
        let stream_b = ResourceValue::stream_v1_from_ring_buffer(
            4,
            0,
            3,
            vec![z.clone(), none.clone(), x.clone(), y.clone()],
        );

        let mut world_a = NuriWorld::new();
        world_a.set_resource_value("흐름".to_string(), stream_a);
        let mut world_b = NuriWorld::new();
        world_b.set_resource_value("흐름".to_string(), stream_b);

        assert_eq!(world_a.state_hash().to_hex(), world_b.state_hash().to_hex());
    }

    #[test]
    fn stream_v1_hash_changes_when_capacity_changes() {
        let stream_a = ResourceValue::stream_v1_from_ring_buffer(
            4,
            2,
            3,
            vec![
                ResourceValue::String("x".to_string()),
                ResourceValue::String("y".to_string()),
                ResourceValue::String("z".to_string()),
                ResourceValue::None,
            ],
        );
        let stream_b = ResourceValue::stream_v1_from_ring_buffer(
            5,
            2,
            3,
            vec![
                ResourceValue::String("x".to_string()),
                ResourceValue::String("y".to_string()),
                ResourceValue::String("z".to_string()),
                ResourceValue::None,
                ResourceValue::None,
            ],
        );

        let mut world_a = NuriWorld::new();
        world_a.set_resource_value("흐름".to_string(), stream_a);
        let mut world_b = NuriWorld::new();
        world_b.set_resource_value("흐름".to_string(), stream_b);

        assert_ne!(world_a.state_hash().to_hex(), world_b.state_hash().to_hex());
    }

    #[test]
    fn plain_map_without_stream_schema_keeps_map_hash_behavior() {
        let map_a = ResourceValue::map_from_entries(vec![
            ResourceMapEntry {
                key: ResourceValue::String("capacity".to_string()),
                value: ResourceValue::Fixed64(Fixed64::from_i64(4)),
            },
            ResourceMapEntry {
                key: ResourceValue::String("head".to_string()),
                value: ResourceValue::Fixed64(Fixed64::from_i64(2)),
            },
            ResourceMapEntry {
                key: ResourceValue::String("len".to_string()),
                value: ResourceValue::Fixed64(Fixed64::from_i64(3)),
            },
            ResourceMapEntry {
                key: ResourceValue::String("buffer".to_string()),
                value: ResourceValue::List(vec![
                    ResourceValue::String("x".to_string()),
                    ResourceValue::String("y".to_string()),
                    ResourceValue::String("z".to_string()),
                    ResourceValue::None,
                ]),
            },
        ]);
        let map_b = ResourceValue::map_from_entries(vec![
            ResourceMapEntry {
                key: ResourceValue::String("capacity".to_string()),
                value: ResourceValue::Fixed64(Fixed64::from_i64(4)),
            },
            ResourceMapEntry {
                key: ResourceValue::String("head".to_string()),
                value: ResourceValue::Fixed64(Fixed64::from_i64(0)),
            },
            ResourceMapEntry {
                key: ResourceValue::String("len".to_string()),
                value: ResourceValue::Fixed64(Fixed64::from_i64(3)),
            },
            ResourceMapEntry {
                key: ResourceValue::String("buffer".to_string()),
                value: ResourceValue::List(vec![
                    ResourceValue::String("z".to_string()),
                    ResourceValue::None,
                    ResourceValue::String("x".to_string()),
                    ResourceValue::String("y".to_string()),
                ]),
            },
        ]);

        let mut world_a = NuriWorld::new();
        world_a.set_resource_value("흐름".to_string(), map_a);
        let mut world_b = NuriWorld::new();
        world_b.set_resource_value("흐름".to_string(), map_b);

        assert_ne!(world_a.state_hash().to_hex(), world_b.state_hash().to_hex());
    }
}
