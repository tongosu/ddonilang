use std::cell::RefCell;
use std::fs;
use std::path::Path;

use blake3;

use crate::cli::input_tape::{mask_from_bytes, read_input_tape};
use crate::cli::sam_snapshot::{apply_snapshot, snapshot_from_held_mask};
use crate::core::geoul::{
    decode_input_snapshot, encode_input_snapshot, encode_state_for_geoul, AuditHeader,
    GeoulBundleReader, GeoulBundleWriter, GeoulFramePayload, InputSnapshotV1, TraceTier,
    DEFAULT_CHECKPOINT_STRIDE,
};
use crate::core::hash;
use crate::core::State;
use crate::lang::lexer::Lexer;
use crate::lang::parser::Parser;
use crate::runtime::{Evaluator, RuntimeError};

struct BaseFrame {
    snapshot: InputSnapshotV1,
    expected_hash: [u8; 32],
}

struct BranchMismatch {
    madi: u64,
    expected_hash: String,
    actual_hash: String,
}

pub fn run_replay_branch(
    geoul_dir: &Path,
    at: u64,
    inject_sam: &Path,
    out_dir: &Path,
    entry_override: Option<&Path>,
) -> Result<(), String> {
    let entry_path = entry_override
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| geoul_dir.join("entry.ddn"));
    if !entry_path.exists() {
        return Err(format!("E_REPLAY_ENTRY_MISSING {}", entry_path.display()));
    }
    let source = fs::read_to_string(&entry_path)
        .map_err(|err| format!("E_REPLAY_ENTRY_READ {} {}", entry_path.display(), err))?;

    let mut reader = GeoulBundleReader::open(geoul_dir)?;
    let base_frame_count = reader.frame_count();
    if base_frame_count == 0 {
        return Err("E_REPLAY_EMPTY_LOG geoul 로그에 프레임이 없습니다".to_string());
    }
    if at >= base_frame_count {
        return Err(format!(
            "E_REPLAY_BRANCH_AT_RANGE at={} max={}",
            at,
            base_frame_count.saturating_sub(1)
        ));
    }

    let mut base_frames = Vec::with_capacity(base_frame_count as usize);
    for madi in 0..base_frame_count {
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
        base_frames.push(BaseFrame {
            snapshot,
            expected_hash: frame.header.state_hash,
        });
    }

    let tape = read_input_tape(inject_sam)?;
    let mut inject_masks = Vec::with_capacity(tape.records.len());
    for (idx, record) in tape.records.iter().enumerate() {
        if record.madi != idx as u32 {
            return Err(format!(
                "E_REPLAY_BRANCH_RECORD_ORDER_MISMATCH record.madi={} idx={}",
                record.madi, idx
            ));
        }
        let mask = mask_from_bytes(&record.held_mask)?;
        inject_masks.push(mask);
    }

    let inject_len = inject_masks.len() as u64;
    let branch_ticks = at + 1 + inject_len;
    if branch_ticks == 0 {
        return Err("E_REPLAY_BRANCH_EMPTY branch ticks가 0입니다".to_string());
    }

    let base_seed = base_frames
        .first()
        .map(|frame| frame.snapshot.rng_seed)
        .unwrap_or(0);
    let mut branch_snapshots = Vec::with_capacity(branch_ticks as usize);
    for madi in 0..=at {
        branch_snapshots.push(base_frames[madi as usize].snapshot.clone());
    }
    let mut last_mask = base_frames[at as usize].snapshot.held_mask;
    for (idx, held_mask) in inject_masks.iter().enumerate() {
        let madi = at + 1 + idx as u64;
        let snapshot = snapshot_from_held_mask(madi, base_seed, *held_mask, last_mask);
        last_mask = snapshot.held_mask;
        branch_snapshots.push(snapshot);
    }

    let tokens = Lexer::tokenize(&source).map_err(|err| format!("E_REPLAY_LEX {:?}", err))?;
    let default_root = Parser::default_root_for_source(&source);
    let program = Parser::parse_with_default_root(tokens, default_root)
        .map_err(|err| format!("E_REPLAY_PARSE {:?}", err))?;
    let evaluator = Evaluator::with_state(State::new());

    let base_header: AuditHeader = reader.header().clone();
    let trace_tier = TraceTier::from_u32(base_header.trace_tier).unwrap_or(TraceTier::Off);
    let mut writer = GeoulBundleWriter::create(
        out_dir,
        base_header,
        DEFAULT_CHECKPOINT_STRIDE,
        hash::SSOT_VERSION,
        env!("CARGO_PKG_VERSION"),
    )
    .map_err(|err| format!("E_GEOUL_INIT {} {}", out_dir.display(), err))?;
    let entry_file = "entry.ddn";
    fs::write(out_dir.join(entry_file), source.as_bytes())
        .map_err(|err| format!("E_GEOUL_ENTRY_WRITE {} {}", out_dir.display(), err))?;
    let entry_hash = format!("blake3:{}", blake3::hash(source.as_bytes()).to_hex());
    writer.set_entry(entry_file, &entry_hash);
    let writer = RefCell::new(writer);

    let mismatch = RefCell::new(None);
    let first_diverge = RefCell::new(None);
    let last_hash = RefCell::new(None);
    let record_error = RefCell::new(None);

    let mut before_tick = |madi: u64, state: &mut State| -> Result<(), RuntimeError> {
        if let Some(snapshot) = branch_snapshots.get(madi as usize) {
            apply_snapshot(state, snapshot);
        }
        Ok(())
    };
    let mut on_tick = |madi: u64, state: &State, _tick_requested: bool| {
        if record_error.borrow().is_some() {
            return;
        }
        let Some(snapshot) = branch_snapshots.get(madi as usize) else {
            return;
        };
        let state_bytes = encode_state_for_geoul(state);
        let actual_hash = *blake3::hash(&state_bytes).as_bytes();
        if madi <= at {
            let expected = base_frames[madi as usize].expected_hash;
            if actual_hash != expected && mismatch.borrow().is_none() {
                mismatch.replace(Some(BranchMismatch {
                    madi,
                    expected_hash: format!("blake3:{}", hex32(&expected)),
                    actual_hash: format!("blake3:{}", hex32(&actual_hash)),
                }));
            }
        } else if (madi as usize) < base_frames.len() && first_diverge.borrow().is_none() {
            let expected = base_frames[madi as usize].expected_hash;
            if actual_hash != expected {
                first_diverge.replace(Some(madi));
            }
        }

        let snapshot_bytes = encode_input_snapshot(snapshot);
        let mut patch_buf = Vec::new();
        let mut alrim_buf = Vec::new();
        let mut full_blob = None;
        if matches!(
            trace_tier,
            TraceTier::Patch | TraceTier::Alrim | TraceTier::Full
        ) {
            let state_hash = blake3::hash(&state_bytes);
            patch_buf.extend_from_slice(state_hash.as_bytes());
            if matches!(trace_tier, TraceTier::Alrim | TraceTier::Full) {
                let snapshot_hash = blake3::hash(&snapshot_bytes);
                alrim_buf.extend_from_slice(snapshot_hash.as_bytes());
                alrim_buf.extend_from_slice(state_hash.as_bytes());
            }
            if trace_tier == TraceTier::Full {
                full_blob = Some(state_bytes.as_slice());
            }
        }
        let payload = GeoulFramePayload {
            patch: if patch_buf.is_empty() {
                None
            } else {
                Some(patch_buf.as_slice())
            },
            alrim: if alrim_buf.is_empty() {
                None
            } else {
                Some(alrim_buf.as_slice())
            },
            full: full_blob,
        };
        if let Err(err) =
            writer
                .borrow_mut()
                .record_frame(madi, &snapshot_bytes, &state_bytes, payload)
        {
            record_error.replace(Some(err));
            return;
        }
        last_hash.replace(Some(actual_hash));
    };

    let result = evaluator.run_with_ticks_observe_and_inject(
        &program,
        branch_ticks,
        &mut before_tick,
        &mut on_tick,
    );

    if let Some(err) = record_error.into_inner() {
        return Err(format!("E_GEOUL_RECORD {} {}", out_dir.display(), err));
    }

    result.map_err(|err| format!("E_REPLAY_RUNTIME {:?}", err))?;

    if let Some(mismatch) = mismatch.into_inner() {
        return Err(format!(
            "E_REPLAY_BRANCH_MISMATCH madi={} expected={} actual={}",
            mismatch.madi, mismatch.expected_hash, mismatch.actual_hash
        ));
    }

    let summary = writer
        .into_inner()
        .finish()
        .map_err(|err| format!("E_GEOUL_FINISH {} {}", out_dir.display(), err))?;

    let last_hash = last_hash
        .into_inner()
        .map(|hash| format!("blake3:{}", hex32(&hash)))
        .unwrap_or_else(|| "null".to_string());
    let first_diverge = match first_diverge.into_inner() {
        Some(madi) => madi.to_string(),
        None => "null".to_string(),
    };

    println!("verify_base_ok=true");
    println!("first_diverge_madi={}", first_diverge);
    println!("branch_last_state_hash={}", last_hash);
    println!("branch_audit_hash={}", summary.audit_hash);
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
