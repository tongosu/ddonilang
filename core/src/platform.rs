use core::cmp::Ordering;
use std::collections::{BTreeMap, VecDeque};

use blake3::hash;
use xxhash_rust::xxh3::xxh3_64;

use crate::fixed64::Fixed64;
use crate::resource::ResourceHandle;
use crate::units::UnitValue;
use crate::signals::{DiagEvent, ExprTrace, FaultContext, Signal, SignalSink, SourceSpan, TickId};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceValue {
    None,
    Bool(bool),
    Fixed64(Fixed64),
    Unit(UnitValue),
    String(String),
    ResourceHandle(ResourceHandle),
    List(Vec<ResourceValue>),
    Set(BTreeMap<String, ResourceValue>),
    Map(BTreeMap<String, ResourceMapEntry>),
}

impl ResourceValue {
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
        match self {
            ResourceValue::None => "없음".to_string(),
            ResourceValue::Bool(true) => "참".to_string(),
            ResourceValue::Bool(false) => "거짓".to_string(),
            ResourceValue::Fixed64(value) => value.to_string(),
            ResourceValue::Unit(value) => {
                let suffix = value
                    .display_symbol()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| value.dim.format());
                format!("{}@{}", value.value, suffix)
            }
            ResourceValue::String(value) => {
                format!("\"{}\"", Self::escape_canon_string(value))
            }
            ResourceValue::ResourceHandle(handle) => format!("자원#{}", handle.to_hex()),
            ResourceValue::List(items) => {
                let mut out = String::from("차림[");
                let mut first = true;
                for item in items {
                    if !first {
                        out.push_str(", ");
                    }
                    first = false;
                    out.push_str(&item.canon_string());
                }
                out.push(']');
                out
            }
            ResourceValue::Set(items) => {
                let mut out = String::from("모음{");
                let mut first = true;
                for value in items.values() {
                    if !first {
                        out.push_str(", ");
                    }
                    first = false;
                    out.push_str(&value.canon_string());
                }
                out.push('}');
                out
            }
            ResourceValue::Map(entries) => {
                let mut out = String::from("짝맞춤{");
                let mut first = true;
                for entry in entries.values() {
                    if !first {
                        out.push_str(", ");
                    }
                    first = false;
                    out.push_str(&entry.key.canon_string());
                    out.push_str("=>");
                    out.push_str(&entry.value.canon_string());
                }
                out.push('}');
                out
            }
        }
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

    pub fn encode_canon(&self, out: &mut Vec<u8>) {
        match self {
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
                out.push(0x06);
                out.extend_from_slice(&(items.len() as u64).to_le_bytes());
                for item in items {
                    item.encode_canon(out);
                }
            }
            ResourceValue::Set(items) => {
                out.push(0x07);
                out.extend_from_slice(&(items.len() as u64).to_le_bytes());
                for value in items.values() {
                    value.encode_canon(out);
                }
            }
            ResourceValue::Map(entries) => {
                out.push(0x08);
                out.extend_from_slice(&(entries.len() as u64).to_le_bytes());
                for entry in entries.values() {
                    entry.key.encode_canon(out);
                    entry.value.encode_canon(out);
                }
            }
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

#[derive(Clone, Debug, Default)]
pub struct NuriWorld {
    // 결정성을 위해 HashMap 대신 BTreeMap 사용
    next_entity: u64,

    ecs: EcsStore,
    resources_json: BTreeMap<String, String>,               // tag -> json

    // ✅ Fixed64 전용 Resource(터살림씨) 저장
    resources_fixed64: BTreeMap<String, Fixed64>,           // tag -> Fixed64
    resources_handle: BTreeMap<String, ResourceHandle>,     // tag -> handle
    resources_value: BTreeMap<String, ResourceValue>,       // tag -> value
}

impl NuriWorld {
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

    fn encode_canonical_with_filter(&self, excluded_prefixes: &[&str]) -> Vec<u8> {
        let include_tag = |tag: &str| -> bool {
            !excluded_prefixes.iter().any(|prefix| tag.starts_with(prefix))
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

        out
    }
}

fn push_str(out: &mut Vec<u8>, s: &str) {
    let b = s.as_bytes();
    out.extend_from_slice(&(b.len() as u64).to_le_bytes());
    out.extend_from_slice(b);
}

// ---------- Seulgi Intent / Packet / InputSnapshot ----------

#[repr(u32)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeulgiIntent {
    None = 0,

    MoveTo {
        x: Fixed64,
        y: Fixed64,
    } = 1,

    Attack {
        target_id: u64,
    } = 2,

    Say {
        text: String,
    } = 3,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeulgiPacket {
    pub agent_id: u64,
    pub recv_seq: u64,
    pub accepted_madi: u64,
    pub target_madi: u64,
    pub intent: SeulgiIntent,
}

impl SeulgiPacket {
    pub fn stable_sort_key(&self) -> (u64, u32, u64, u64) {
        (
            self.agent_id,
            self.intent.kind_u32(),
            self.intent.stable_payload_hash(),
            self.recv_seq,
        )
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
    SetComponentJson { entity: EntityId, tag: ComponentTag, json: String },
    RemoveComponent { entity: EntityId, tag: ComponentTag },
    SetResourceJson { tag: String, json: String },

    // ✅ Fixed64 Resource 세팅
    SetResourceFixed64 { tag: String, value: Fixed64 },
    SetResourceHandle { tag: String, handle: ResourceHandle },
    SetResourceValue { tag: String, value: ResourceValue },

    // DivAssignResourceFixed64: emit arithmetic fault on div0, commit only on success.
    DivAssignResourceFixed64 {
        tag: String,
        rhs: Fixed64,
        tick_id: TickId,
        location: &'static str,
        source_span: Option<SourceSpan>,
        expr: Option<ExprTrace>,
    },
    EmitSignal { signal: Signal, targets: Vec<String> },
    GuardViolation { entity: EntityId, rule_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TickFrame {
    pub snapshot: InputSnapshot,
    pub patch: Patch,
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
            intent,
        });
    }
}

pub struct DetNuri {
    world: NuriWorld,
}

impl DetNuri {
    pub fn new() -> Self {
        Self { world: NuriWorld::new() }
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

        for op in &patch.ops {
            match op {
                PatchOp::SetComponentJson { entity, tag, json } => {
                    if skip_assignments {
                        continue;
                    }
                    self.world.set_component_json(*entity, tag.clone(), json.clone());
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
                    if skip_assignments {
                        continue;
                    }
                    self.world.set_resource_value(tag.clone(), value.clone());
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
                    let cur = self.world.get_resource_fixed64(tag).unwrap_or(Fixed64::ZERO);
                    let ctx = FaultContext {
                        tick_id: *op_tick_id,
                        location,
                        source_span: source_span.clone(),
                        expr: expr.clone(),
                    };
                    match cur.try_div(*rhs) {
                        Ok(next) => self.world.set_resource_fixed64(tag.clone(), next),
                        Err(kind) => {
                            sink.emit(Signal::ArithmeticFault { ctx, kind: kind.clone() });
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
                PatchOp::EmitSignal { signal, targets } => {
                    match signal {
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
                    }
                }
                PatchOp::GuardViolation { .. } => {}
            }
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
    use super::NuriWorld;
    use crate::Fixed64;

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
}
