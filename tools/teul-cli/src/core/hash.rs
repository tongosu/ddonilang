use blake3;

use crate::core::detbin::{encode_state, encode_trace_bundle, TraceMeta};
use crate::core::{State, Trace};

pub const SSOT_VERSION: &str = "20.6.6";

pub fn state_hash(state: &State) -> String {
    hash_bytes(&encode_state(state))
}

pub fn trace_hash(trace: &Trace, source: &str, state_hash: &str, madi: u64, seed: u64) -> String {
    let meta = TraceMeta {
        ssot_version: SSOT_VERSION,
        seed,
        madi,
    };
    let bytes = encode_trace_bundle(source, trace, state_hash, &meta);
    hash_bytes(&bytes)
}

fn hash_bytes(bytes: &[u8]) -> String {
    let digest = blake3::hash(bytes);
    format!("blake3:{}", digest.to_hex())
}
