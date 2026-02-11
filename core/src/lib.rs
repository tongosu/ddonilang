pub mod fixed64;
pub mod signals;
pub mod alrim;
pub mod platform;
pub mod engine;
pub mod units;
pub mod input;
pub mod resource;
pub mod gogae3;
pub mod seulgi;
pub mod nurigym;
pub mod sam;
pub mod realms;
pub mod warp;

pub use fixed64::Fixed64;
pub use signals::{
    ArithmeticFaultKind, ExprTrace, FaultContext, Signal, SignalSink, SourceSpan, VecSignalSink,
    TickId,
};
pub use alrim::{AlrimHandler, AlrimLogEntry, AlrimLogger, AlrimLoop, VecAlrimLogger, ALRIM_MAX_PASSES};
pub use platform::{
    Bogae, ComponentTag, DetSam, EntityId, Geoul, InMemoryGeoul, InputSnapshot, Iyagi,
    KEY_A, KEY_D, KEY_S, KEY_W, Nuri, NuriWorld, Patch, PatchOp, ResourceMapEntry, ResourceValue,
    Sam, Seulgi, SeulgiContext, SeulgiIntent, SeulgiPacket, StateHash, TickFrame
};
pub use seulgi::latency::{LatencyEvent, LatencyMode, LatencyPolicy};
pub use seulgi::safety::{SafetyDecision, SafetyMode, SafetyRule};
pub use seulgi::{goal, intent};
pub use nurigym::spec::{ActionSpec, ObservationSpec};
pub use engine::EngineLoop;
pub use units::{
    base_unit_symbol_for_dim, canonical_unit_symbol, is_known_unit, resource_tag_with_unit,
    set_unit_registry_symbols, unit_spec_from_symbol, Unit, UnitDim, UnitError, UnitSpec,
    UnitValue,
};
pub use input::{is_key_just_pressed, is_key_pressed, key_bit_from_name};
pub use resource::{asset_handle_from_bundle_path, ResourceHandle};
pub use realms::{mix64, MultiRealmManager, Realm, RealmStepInput, RealmStepOutput, ThreadMode};
pub use warp::{run_warp_bench, StepBatchSoA, WarpBackend, WarpBenchInput, WarpBenchOutput, WarpPolicy};

#[cfg(test)]
mod tests;
