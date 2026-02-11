use std::cell::RefCell;
use std::fs;
use std::path::Path;

use crate::cli::sam_snapshot::apply_snapshot;
use crate::core::geoul::{decode_input_snapshot, geoul_state_hash_bytes, GeoulBundleReader};
use crate::core::State;
use crate::lang::lexer::Lexer;
use crate::lang::parser::Parser;
use crate::runtime::{Evaluator, RuntimeError};

struct FrameData {
    snapshot: crate::core::geoul::InputSnapshotV1,
    expected_hash: [u8; 32],
    patch_blob: Option<Vec<u8>>,
}

struct ReplayMismatch {
    madi: u64,
    expected_hash: String,
    actual_hash: String,
    patch_hex: Option<String>,
}

pub fn run_replay_verify(
    geoul_dir: &Path,
    entry_override: Option<&Path>,
    until: Option<u64>,
    seek: Option<u64>,
) -> Result<(), String> {
    let entry_path = entry_override
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| geoul_dir.join("entry.ddn"));
    if !entry_path.exists() {
        return Err(format!(
            "E_REPLAY_ENTRY_MISSING {}",
            entry_path.display()
        ));
    }
    let source = fs::read_to_string(&entry_path)
        .map_err(|err| format!("E_REPLAY_ENTRY_READ {} {}", entry_path.display(), err))?;

    let mut reader = GeoulBundleReader::open(geoul_dir)?;
    let frame_count = reader.frame_count();
    if frame_count == 0 {
        return Err("E_REPLAY_EMPTY_LOG geoul 로그에 프레임이 없습니다".to_string());
    }
    let max_madi = frame_count - 1;
    let until_madi = until.unwrap_or(max_madi);
    if until_madi > max_madi {
        return Err(format!(
            "E_REPLAY_UNTIL_RANGE until={} max={}",
            until_madi, max_madi
        ));
    }
    let seek_madi = seek.unwrap_or(0);
    if seek_madi > until_madi {
        return Err(format!(
            "E_REPLAY_SEEK_RANGE seek={} until={}",
            seek_madi, until_madi
        ));
    }

    let mut frames = Vec::with_capacity((until_madi + 1) as usize);
    for madi in 0..=until_madi {
        let frame = reader.read_frame(madi)?;
        if frame.header.madi != madi {
            return Err(format!(
                "E_REPLAY_FRAME_MADI_MISMATCH expected={} got={}",
                madi, frame.header.madi
            ));
        }
        let snapshot = decode_input_snapshot(&frame.snapshot_detbin)?;
        if snapshot.madi != madi {
            return Err(format!(
                "E_REPLAY_SNAPSHOT_MADI_MISMATCH expected={} got={}",
                madi, snapshot.madi
            ));
        }
        let patch_blob = if frame.header.patch_bytes > 0 {
            Some(frame.patch_blob)
        } else {
            None
        };
        frames.push(FrameData {
            snapshot,
            expected_hash: frame.header.state_hash,
            patch_blob,
        });
    }

    let tokens = Lexer::tokenize(&source)
        .map_err(|err| format!("E_REPLAY_LEX {:?}", err))?;
    let default_root = Parser::default_root_for_source(&source);
    let program = Parser::parse_with_default_root(tokens, default_root)
        .map_err(|err| format!("E_REPLAY_PARSE {:?}", err))?;
    let evaluator = Evaluator::with_state(State::new());

    let mismatch = RefCell::new(None);
    let mut before_tick = |madi: u64, state: &mut State| -> Result<(), RuntimeError> {
        if let Some(frame) = frames.get(madi as usize) {
            apply_snapshot(state, &frame.snapshot);
        }
        Ok(())
    };
    let mut on_tick = |madi: u64, state: &State, _tick_requested: bool| {
        if madi < seek_madi {
            return;
        }
        if mismatch.borrow().is_some() {
            return;
        }
        let Some(frame) = frames.get(madi as usize) else {
            return;
        };
        let actual_bytes = geoul_state_hash_bytes(state);
        if actual_bytes != frame.expected_hash {
            let patch_hex = frame.patch_blob.as_deref().map(hex_bytes);
            mismatch.replace(Some(ReplayMismatch {
                madi,
                expected_hash: format!("blake3:{}", hex32(&frame.expected_hash)),
                actual_hash: format!("blake3:{}", hex32(&actual_bytes)),
                patch_hex,
            }));
        }
    };

    let ticks = until_madi + 1;
    evaluator
        .run_with_ticks_observe_and_inject(&program, ticks, &mut before_tick, &mut on_tick)
        .map_err(|err| format!("E_REPLAY_RUNTIME {:?}", err))?;

    if let Some(mismatch) = mismatch.into_inner() {
        println!("verify_ok=false");
        println!("first_diverge_madi={}", mismatch.madi);
        println!("expected_state_hash={}", mismatch.expected_hash);
        println!("actual_state_hash={}", mismatch.actual_hash);
        if let Some(patch_hex) = mismatch.patch_hex {
            println!("patch_hex={}", patch_hex);
        }
        return Err(format!(
            "E_REPLAY_MISMATCH madi={} expected={} actual={}",
            mismatch.madi, mismatch.expected_hash, mismatch.actual_hash
        ));
    }

    println!("verify_ok=true");
    println!("first_diverge_madi=null");
    Ok(())
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut out, "{:02x}", b);
    }
    out
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut out, "{:02x}", b);
    }
    out
}
