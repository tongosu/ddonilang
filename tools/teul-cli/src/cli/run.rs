use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use blake3;
use serde_json::json;
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::cli::bogae::{
    default_bogae_out_dir, is_bogae_out_dir, resolve_bogae_out_dir, BogaeMode, OverlayConfig,
};
use crate::cli::bogae_console::{render_drawlist_ascii, ConsoleLive, ConsoleRenderConfig};
use crate::cli::bogae_playback::{write_manifest, write_viewer_assets, PlaybackFrameMeta};
use crate::cli::bogae_web::write_web_assets;
use crate::cli::input_tape::{
    mask_from_bytes, mask_to_bytes, parse_held_mask, read_input_tape, write_input_tape, InputRecord,
    InputTape, KEY_REGISTRY_KEYS,
};
use crate::cli::sam_live::{LiveInput, SamLiveMode};
use crate::core::bogae::{
    build_bogae_output, load_css4_pack, BogaeCodec, CmdPolicyConfig, CmdPolicyEvent,
    CmdPolicyMode, BogaeError, BogaeOutput, ColorNamePack,
};
use crate::core::fixed64::Fixed64;
use crate::core::geoul::{
    encode_input_snapshot, encode_state_for_geoul, AuditHeader, GeoulBundleWriter, GeoulFramePayload,
    InputSnapshotV1, NetEventV1, TraceTier, DEFAULT_CHECKPOINT_STRIDE,
};
use crate::core::hash;
use crate::core::state::Key;
use crate::core::unit::UnitDim;
use crate::core::value::{ListValue, PackValue, Quantity, Value};
use crate::core::{State, Trace};
use crate::lang::ast::{ContractKind, ContractMode, Program, Stmt};
use crate::lang::lexer::{LexError, Lexer};
use crate::lang::parser::{ParseError, Parser, ParseMode};
use crate::runtime::{
    ContractDiag, EvalOutput, Evaluator, OpenDiagConfig, OpenMode, OpenPolicy, OpenRuntime,
    RuntimeError,
};
use ddonirang_core::gogae3::{
    compute_w24_state_hash, compute_w25_state_hash, compute_w26_state_hash,
    compute_w27_state_hash, compute_w28_state_hash, compute_w29_state_hash,
    compute_w30_state_hash, compute_w31_state_hash, compute_w32_state_hash,
    compute_w33_state_hash, W24Params, W25Params, W26Params, W27Params, W28Params, W29Params,
    W30Params, W31Params, W32Params, W33Params,
};
pub enum RunError {
    Lex(LexError),
    Parse(ParseError),
    Runtime(RuntimeError),
    Bogae(BogaeError),
    Io { path: PathBuf, message: String },
}

impl RunError {
    pub fn code(&self) -> &'static str {
        match self {
            RunError::Lex(err) => err.code(),
            RunError::Parse(err) => err.code(),
            RunError::Runtime(err) => err.code(),
            RunError::Bogae(err) => err.code(),
            RunError::Io { .. } => "E_IO_WRITE",
        }
    }

    pub fn format(&self, file: &str) -> String {
        match self {
            RunError::Lex(err) => format!(
                "{} {}:{}:{} {}",
                err.code(),
                file,
                lex_line(err),
                lex_col(err),
                lex_message(err)
            ),
            RunError::Parse(err) => format!(
                "{} {}:{}:{} {}",
                err.code(),
                file,
                parse_line(err),
                parse_col(err),
                parse_message(err)
            ),
            RunError::Runtime(err) => format!(
                "{} {}:{}:{} {}",
                err.code(),
                file,
                runtime_line(err),
                runtime_col(err),
                runtime_message(err)
            ),
            RunError::Bogae(err) => format!(
                "{} {}:1:1 {}",
                err.code(),
                file,
                err.message()
            ),
            RunError::Io { path, message } => {
                format!("E_IO_WRITE {}:1:1 {}", path.display(), message)
            }
        }
    }
}

pub trait RunEmitSink {
    fn out(&mut self, line: &str);
    fn err(&mut self, line: &str);
}

pub struct StdoutRunEmitter;

impl RunEmitSink for StdoutRunEmitter {
    fn out(&mut self, line: &str) {
        println!("{}", line);
    }

    fn err(&mut self, line: &str) {
        eprintln!("{}", line);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MadiLimit {
    Finite(u64),
    Infinite,
}

#[derive(Clone, Debug)]
pub struct ArtifactPin {
    pub id: String,
    pub hash: String,
}

impl ArtifactPin {
    pub fn parse(value: &str) -> Result<Self, String> {
        let Some((id, hash)) = value.split_once('=') else {
            return Err("형식은 id=hash 이어야 합니다.".to_string());
        };
        let id = id.trim();
        let hash = hash.trim();
        if id.is_empty() || hash.is_empty() {
            return Err("id 또는 hash가 비었습니다.".to_string());
        }
        Ok(ArtifactPin {
            id: id.to_string(),
            hash: hash.to_string(),
        })
    }
}

pub fn parse_artifact_pins(values: &[String]) -> Result<Vec<ArtifactPin>, String> {
    let mut pins = Vec::new();
    for value in values {
        pins.push(ArtifactPin::parse(value)?);
    }
    Ok(pins)
}

pub struct RunOptions {
    pub diag_jsonl: Option<PathBuf>,
    pub repro_json: Option<PathBuf>,
    pub trace_json: Option<PathBuf>,
    pub geoul_out: Option<PathBuf>,
    pub geoul_record_out: Option<PathBuf>,
    pub run_command: Option<String>,
    pub init_state: Vec<String>,
    pub init_state_files: Vec<PathBuf>,
    pub trace_tier: TraceTier,
    pub age_target: Option<String>,
    pub lang_mode: Option<crate::cli::lang_mode::LangModeArg>,
    pub bogae_mode: Option<BogaeMode>,
    pub bogae_codec: BogaeCodec,
    pub bogae_out: Option<PathBuf>,
    pub cmd_policy: CmdPolicyConfig,
    pub overlay: OverlayConfig,
    pub bogae_cache_log: bool,
    pub bogae_live: bool,
    pub console_config: ConsoleRenderConfig,
    pub until_gameover: bool,
    pub gameover_key: String,
    pub sam_path: Option<PathBuf>,
    pub record_sam_path: Option<PathBuf>,
    pub sam_live: Option<SamLiveMode>,
    pub sam_live_host: String,
    pub sam_live_port: u16,
    pub madi_hz: Option<u32>,
    pub open_mode: Option<crate::runtime::OpenMode>,
    pub open_log: Option<PathBuf>,
    pub open_bundle: Option<PathBuf>,
    pub bogae_skin: Option<PathBuf>,
    pub no_open: bool,
    pub unsafe_open: bool,
    pub run_manifest: Option<PathBuf>,
    pub artifact_pins: Vec<ArtifactPin>,
}

#[derive(Clone, Debug)]
struct StateEntry {
    key: String,
    value: Value,
}

fn apply_init_state(state: &mut State, options: &RunOptions, source_path: &Path) -> Result<(), String> {
    let mut entries = Vec::new();
    let base_dir = source_path.parent().unwrap_or_else(|| Path::new("."));
    for item in &options.init_state {
        entries.push(parse_state_assignment(item, base_dir)?);
    }
    for path in &options.init_state_files {
        let mut file_entries = parse_state_file(path)?;
        entries.append(&mut file_entries);
    }
    for entry in entries {
        state.set(Key::new(entry.key), entry.value);
    }
    Ok(())
}

fn parse_state_file(path: &Path) -> Result<Vec<StateEntry>, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("E_INIT_STATE_READ {} {}", path.display(), e))?;
    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    if is_state_json_path(path) {
        return parse_state_json(&text, base_dir);
    }
    let mut entries = Vec::new();
    for (idx, raw_line) in text.lines().enumerate() {
        let mut line = raw_line.trim_start_matches('\u{feff}').trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        if let Some(pos) = line.find('#') {
            if line[..pos].trim().len() > 0 {
                line = line[..pos].trim().to_string();
            } else {
                continue;
            }
        }
        if line.is_empty() {
            continue;
        }
        let entry = parse_state_assignment_with_context(&line, base_dir)
            .map_err(|e| format!("E_INIT_STATE_LINE {}:{} {}", path.display(), idx + 1, e))?;
        entries.push(entry);
    }
    Ok(entries)
}

fn is_state_json_path(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("json") => true,
        Some(ext) if ext.eq_ignore_ascii_case("detjson") => true,
        _ => false,
    }
}

fn parse_state_json(text: &str, base_dir: &Path) -> Result<Vec<StateEntry>, String> {
    let root: JsonValue =
        serde_json::from_str(text).map_err(|e| format!("E_INIT_STATE_JSON_PARSE {}", e))?;
    let mut entries = Vec::new();
    let JsonValue::Object(map) = root else {
        return Err("E_INIT_STATE_JSON_ROOT state 객체가 필요합니다".to_string());
    };

    if let Some(state_value) = map.get("state") {
        let state_map = state_value
            .as_object()
            .ok_or_else(|| "E_INIT_STATE_JSON_STATE state 객체가 필요합니다".to_string())?;
        for (key, value) in state_map {
            let normalized = normalize_state_key(key);
            let value = json_to_value(value, base_dir)
                .map_err(|e| format!("E_INIT_STATE_JSON_VALUE {} {}", key, e))?;
            entries.push(StateEntry { key: normalized, value });
        }
        return Ok(entries);
    }

    let ignore_schema = map.get("schema").and_then(JsonValue::as_str).is_some();
    for (key, value) in map {
        if ignore_schema && key == "schema" {
            continue;
        }
        let normalized = normalize_state_key(&key);
        let value = json_to_value(&value, base_dir)
            .map_err(|e| format!("E_INIT_STATE_JSON_VALUE {} {}", key, e))?;
        entries.push(StateEntry { key: normalized, value });
    }
    Ok(entries)
}

fn json_to_value(value: &JsonValue, base_dir: &Path) -> Result<Value, String> {
    match value {
        JsonValue::Null => Ok(Value::None),
        JsonValue::Bool(flag) => Ok(Value::Bool(*flag)),
        JsonValue::Number(num) => {
            if let Some(i) = num.as_i64() {
                Ok(Value::Num(Quantity::new(Fixed64::from_int(i), UnitDim::zero())))
            } else if let Some(u) = num.as_u64() {
                let signed = if u > i64::MAX as u64 { i64::MAX } else { u as i64 };
                Ok(Value::Num(Quantity::new(Fixed64::from_int(signed), UnitDim::zero())))
            } else if let Some(f) = num.as_f64() {
                let text = format!("{}", f);
                let raw = Fixed64::parse_literal(&text)
                    .ok_or_else(|| format!("숫자 변환 실패: {}", text))?;
                Ok(Value::Num(Quantity::new(raw, UnitDim::zero())))
            } else {
                Err("숫자 변환 실패".to_string())
            }
        }
        JsonValue::String(text) => {
            if let Some(rest) = text.strip_prefix("@file:") {
                let path = rest.trim();
                if path.is_empty() {
                    return Err("파일 경로가 비었습니다".to_string());
                }
                let resolved = if Path::new(path).is_absolute() {
                    PathBuf::from(path)
                } else {
                    base_dir.join(path)
                };
                let content = fs::read_to_string(&resolved)
                    .map_err(|e| format!("E_INIT_STATE_FILE {} {}", resolved.display(), e))?;
                return Ok(Value::Str(content));
            }
            Ok(Value::Str(text.clone()))
        }
        JsonValue::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(json_to_value(item, base_dir)?);
            }
            Ok(Value::List(ListValue { items: out }))
        }
        JsonValue::Object(map) => {
            let mut fields = std::collections::BTreeMap::new();
            for (key, value) in map {
                let field_value = json_to_value(value, base_dir)?;
                fields.insert(key.clone(), field_value);
            }
            Ok(Value::Pack(PackValue { fields }))
        }
    }
}

fn parse_state_assignment(text: &str, base_dir: &Path) -> Result<StateEntry, String> {
    parse_state_assignment_with_context(text, base_dir)
}

fn parse_state_assignment_with_context(text: &str, base_dir: &Path) -> Result<StateEntry, String> {
    let Some((raw_key, raw_value)) = text.split_once('=') else {
        return Err("key=value 형식이 필요합니다".to_string());
    };
    let key = normalize_state_key(raw_key.trim());
    if key.is_empty() {
        return Err("키가 비었습니다".to_string());
    }
    let value = parse_state_value(raw_value.trim(), base_dir)?;
    Ok(StateEntry { key, value })
}

fn normalize_state_key(input: &str) -> String {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix("살림.") {
        return rest.to_string();
    }
    if let Some(rest) = trimmed.strip_prefix("바탕.") {
        return rest.to_string();
    }
    trimmed.to_string()
}

fn parse_state_value(text: &str, base_dir: &Path) -> Result<Value, String> {
    if text.is_empty() {
        return Ok(Value::Str(String::new()));
    }
    if let Some(rest) = text.strip_prefix("@file:") {
        let path = rest.trim();
        if path.is_empty() {
            return Err("파일 경로가 비었습니다".to_string());
        }
        let resolved = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            base_dir.join(path)
        };
        let content = fs::read_to_string(&resolved)
            .map_err(|e| format!("E_INIT_STATE_FILE {} {}", resolved.display(), e))?;
        return Ok(Value::Str(content));
    }
    let text = text.trim();
    if text == "참" {
        return Ok(Value::Bool(true));
    }
    if text == "거짓" {
        return Ok(Value::Bool(false));
    }
    if text == "없음" {
        return Ok(Value::None);
    }
    if let Some(stripped) = strip_quoted(text, '"') {
        let value = unescape_state_string(stripped)?;
        return Ok(Value::Str(value));
    }
    if let Some(stripped) = strip_quoted(text, '\'') {
        let value = unescape_state_string(stripped)?;
        return Ok(Value::Str(value));
    }
    if let Some(raw) = Fixed64::parse_literal(text) {
        return Ok(Value::Num(Quantity::new(raw, UnitDim::zero())));
    }
    Ok(Value::Str(text.to_string()))
}

fn strip_quoted<'a>(text: &'a str, quote: char) -> Option<&'a str> {
    if text.len() >= 2 && text.starts_with(quote) && text.ends_with(quote) {
        Some(&text[1..text.len() - 1])
    } else {
        None
    }
}

fn unescape_state_string(text: &str) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        let esc = chars.next().ok_or_else(|| "문자열 이스케이프가 끝나지 않았습니다".to_string())?;
        let mapped = match esc {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '\\' => '\\',
            '"' => '"',
            '\'' => '\'',
            other => {
                return Err(format!("지원하지 않는 이스케이프: \\{}", other));
            }
        };
        out.push(mapped);
    }
    Ok(out)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BogaeCacheKey {
    state_hash: String,
    codec: BogaeCodec,
    policy_mode: CmdPolicyMode,
    policy_cap: u32,
}

struct BogaeCache {
    last_key: Option<BogaeCacheKey>,
    last_output: Option<BogaeOutput>,
    last_event: Option<CmdPolicyEvent>,
}

impl BogaeCache {
    fn new() -> Self {
        Self {
            last_key: None,
            last_output: None,
            last_event: None,
        }
    }

    fn build(
        &mut self,
        state: &State,
        pack: Option<&ColorNamePack>,
        policy: CmdPolicyConfig,
        codec: BogaeCodec,
        cache_log: bool,
        madi: Option<u64>,
        emit: &mut dyn RunEmitSink,
    ) -> Result<(BogaeOutput, Option<CmdPolicyEvent>), RunError> {
        let state_hash = hash::state_hash(state);
        let key = BogaeCacheKey {
            state_hash: state_hash.clone(),
            codec,
            policy_mode: policy.mode,
            policy_cap: policy.cap,
        };
        if self.last_key.as_ref() == Some(&key) {
            if let Some(output) = self.last_output.clone() {
                log_bogae_cache(cache_log, true, madi, &state_hash, &output, emit);
                return Ok((output, self.last_event.clone()));
            }
        }
        let (output, event) =
            build_bogae_output(state, pack, policy, codec).map_err(RunError::Bogae)?;
        log_bogae_cache(cache_log, false, madi, &state_hash, &output, emit);
        self.last_key = Some(key);
        self.last_output = Some(output.clone());
        self.last_event = event.clone();
        Ok((output, event))
    }
}

fn log_bogae_cache(
    enabled: bool,
    hit: bool,
    madi: Option<u64>,
    state_hash: &str,
    output: &BogaeOutput,
    emit: &mut dyn RunEmitSink,
) {
    if !enabled {
        return;
    }
    let madi_text = match madi {
        Some(value) => value.to_string(),
        None => "final".to_string(),
    };
    let hit_text = if hit { "hit" } else { "miss" };
    emit.out(&format!(
        "bogae_cache={} madi={} state_hash={} cmd_count={} codec={}",
        hit_text,
        madi_text,
        state_hash,
        output.drawlist.cmds.len(),
        output.codec.tag()
    ));
}

struct TickSnapshot {
    madi: u64,
    state: State,
}

struct RunOutcome {
    output: EvalOutput,
    ticks: u64,
}

struct SamPlan {
    masks: Vec<u16>,
    last_mask: u16,
    record_out: Option<(PathBuf, InputTape)>,
    net_events: Option<Vec<NetEventDet>>,
}

struct SamPlanResult {
    plan: Option<SamPlan>,
    ticks: Option<u64>,
}

#[derive(Clone, Debug)]
struct NetEventDet {
    sender: String,
    seq: u64,
    order_key: String,
    payload: String,
}

struct LivePlaybackWriter {
    out_dir: PathBuf,
    frames_dir: PathBuf,
    frames: Vec<PlaybackFrameMeta>,
    start_madi: Option<u64>,
    codec: BogaeCodec,
}

impl LivePlaybackWriter {
    fn new(
        out_dir: &Path,
        skin_source: Option<&Path>,
        overlay: OverlayConfig,
        codec: BogaeCodec,
    ) -> Result<Self, RunError> {
        fs::create_dir_all(out_dir).map_err(|e| RunError::Io {
            path: out_dir.to_path_buf(),
            message: e.to_string(),
        })?;
        let frames_dir = out_dir.join("frames");
        fs::create_dir_all(&frames_dir).map_err(|e| RunError::Io {
            path: frames_dir.clone(),
            message: e.to_string(),
        })?;
        write_manifest(out_dir, 0, 0, &[], codec.tag()).map_err(|e| RunError::Io {
            path: out_dir.join("manifest.detjson"),
            message: e,
        })?;
        write_viewer_assets(out_dir, skin_source, overlay).map_err(|e| RunError::Io {
            path: out_dir.join("viewer/index.html"),
            message: e,
        })?;
        Ok(Self {
            out_dir: out_dir.to_path_buf(),
            frames_dir,
            frames: Vec::new(),
            start_madi: None,
            codec,
        })
    }

    fn push_frame(
        &mut self,
        madi: u64,
        state_hash: String,
        output: &BogaeOutput,
    ) -> Result<(), RunError> {
        let idx = self.frames.len();
        let file_name = format!("{:06}.{}.detbin", idx, self.codec.file_ext());
        let rel_file = format!("frames/{}", file_name);
        fs::write(self.frames_dir.join(&file_name), &output.detbin).map_err(|e| RunError::Io {
            path: self.frames_dir.join(&file_name),
            message: e.to_string(),
        })?;
        self.frames.push(PlaybackFrameMeta {
            madi,
            state_hash,
            hash: output.hash.clone(),
            cmd_count: output.drawlist.cmds.len() as u32,
            file: rel_file,
        });
        if self.start_madi.is_none() {
            self.start_madi = Some(madi);
        }
        let start_madi = self.start_madi.unwrap_or(0);
        let end_madi = start_madi + self.frames.len() as u64;
        write_manifest(&self.out_dir, start_madi, end_madi, &self.frames, self.codec.tag())
            .map_err(|e| RunError::Io {
            path: self.out_dir.join("manifest.detjson"),
            message: e,
        })?;
        Ok(())
    }
}

impl SamPlan {
    fn apply_tick(&mut self, state: &mut State, madi: u64) {
        let idx = madi as usize;
        let held = self.masks.get(idx).copied().unwrap_or(0);
        let pressed = (!self.last_mask) & held;
        let released = self.last_mask & !held;
        clear_sam_keys(state);
        apply_keyboard_mask(state, held, pressed, released);
        apply_net_events(state, self.net_events.as_deref(), madi);
        self.last_mask = held;
    }

    fn finish(self) -> Result<(), String> {
        if let Some((path, tape)) = self.record_out {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            write_input_tape(&path, &tape)?;
        }
        Ok(())
    }
}

fn build_sam_plan(
    ticks: Option<u64>,
    sam_path: Option<&Path>,
    record_path: Option<&Path>,
    madi_hz: Option<u32>,
) -> Result<SamPlanResult, String> {
    if sam_path.is_some() && record_path.is_some() {
        return Err("E_SAM_MODE_CONFLICT --sam과 --record-sam은 동시에 사용할 수 없습니다".to_string());
    }
    if sam_path.is_none() && record_path.is_none() {
        return Ok(SamPlanResult { plan: None, ticks });
    }

    if let Some(path) = sam_path {
        if is_detjson_path(path) {
            let net_events = read_net_events_detjson(path)?;
            let det_ticks = 1u64;
            if let Some(ticks) = ticks {
                if ticks != det_ticks {
                    return Err(format!(
                        "E_SAM_RECORD_COUNT_MISMATCH record_count={} ticks={}",
                        det_ticks, ticks
                    ));
                }
            }
            return Ok(SamPlanResult {
                plan: Some(SamPlan {
                    masks: vec![0],
                    last_mask: 0,
                    record_out: None,
                    net_events: Some(net_events),
                }),
                ticks: Some(det_ticks),
            });
        }
        let tape = read_input_tape(path)?;
        if let Some(expected_hz) = madi_hz {
            if tape.madi_hz != expected_hz {
                return Err(format!(
                    "E_SAM_MADI_HZ_MISMATCH 테이프:{}hz 요청:{}hz",
                    tape.madi_hz, expected_hz
                ));
            }
        }
        let tape_ticks = tape.records.len() as u64;
        if let Some(ticks) = ticks {
            if tape_ticks != ticks {
                return Err(format!(
                    "E_SAM_RECORD_COUNT_MISMATCH record_count={} ticks={}",
                    tape.records.len(),
                    ticks
                ));
            }
        }
        let mut masks = Vec::with_capacity(tape.records.len());
        for (idx, record) in tape.records.iter().enumerate() {
            if record.madi != idx as u32 {
                return Err(format!(
                    "E_SAM_RECORD_ORDER_MISMATCH record.madi={} idx={}",
                    record.madi, idx
                ));
            }
            let mask = mask_from_bytes(&record.held_mask)?;
            masks.push(mask);
        }
        return Ok(SamPlanResult {
            plan: Some(SamPlan {
                masks,
                last_mask: 0,
                record_out: None,
                net_events: None,
            }),
            ticks: Some(tape_ticks),
        });
    }

    let ticks = ticks.ok_or_else(|| "E_SAM_RECORD_MADI_REQUIRED --record-sam은 --madi가 필요합니다.".to_string())?;
    let hz = madi_hz.unwrap_or(60);
    if hz == 0 {
        return Err("E_SAM_BAD_MADI_HZ madi-hz는 0이 될 수 없습니다".to_string());
    }
    let masks = collect_recorded_masks(ticks)?;
    let records = masks
        .iter()
        .enumerate()
        .map(|(idx, mask)| InputRecord {
            madi: idx as u32,
            held_mask: mask_to_bytes(*mask),
        })
        .collect::<Vec<_>>();
    let tape = InputTape { madi_hz: hz, records };
    let record_out = record_path.map(|path| (path.to_path_buf(), tape));

    Ok(SamPlanResult {
        plan: Some(SamPlan {
            masks,
            last_mask: 0,
            record_out,
            net_events: None,
        }),
        ticks: Some(ticks),
    })
}

fn is_detjson_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("detjson"))
        .unwrap_or(false)
}

fn read_net_events_detjson(path: &Path) -> Result<Vec<NetEventDet>, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("E_SAM_DETJSON_READ {}", e.to_string()))?;
    let value: JsonValue =
        serde_json::from_str(&text).map_err(|e| format!("E_SAM_DETJSON_PARSE {e}"))?;
    let schema = value
        .get("schema")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());
    if let Some(schema) = schema.as_deref() {
        if schema != "ddn.input_snapshot.v1" {
            return Err(format!("E_SAM_DETJSON_SCHEMA {}", schema));
        }
    }
    let mut events = Vec::new();
    if let Some(net_events) = value.get("net_events").and_then(|v| v.as_array()) {
        for event in net_events {
            let sender = event
                .get("sender")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "E_SAM_DETJSON_FIELD sender".to_string())?
                .to_string();
            let seq = event
                .get("seq")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| "E_SAM_DETJSON_FIELD seq".to_string())?;
            let order_key = event
                .get("order_key")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let payload = event
                .get("payload")
                .ok_or_else(|| "E_SAM_DETJSON_FIELD payload".to_string())?;
            let payload = serde_json::to_string(payload)
                .map_err(|e| format!("E_SAM_DETJSON_PAYLOAD {e}"))?;
            events.push(NetEventDet {
                sender,
                seq,
                order_key,
                payload,
            });
        }
    }
    events.sort_by(|a, b| {
        (
            a.sender.as_str(),
            a.seq,
            a.order_key.as_str(),
            a.payload.as_str(),
        )
            .cmp(&(
                b.sender.as_str(),
                b.seq,
                b.order_key.as_str(),
                b.payload.as_str(),
            ))
    });
    Ok(events)
}

fn collect_recorded_masks(ticks: u64) -> Result<Vec<u16>, String> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut masks = Vec::with_capacity(ticks as usize);
    for madi in 0..ticks {
        let prompt = format!(
            "[sam] tick {} held keys ({}): ",
            madi,
            KEY_REGISTRY_KEYS.join(", ")
        );
        eprint!("{}", prompt);
        io::stderr().flush().ok();

        let mut line = String::new();
        let read = handle.read_line(&mut line).map_err(|e| e.to_string())?;
        if read == 0 {
            return Err("E_SAM_INPUT_EOF 입력이 예상보다 빨리 종료되었습니다".to_string());
        }
        let mask = parse_held_mask(&line)?;
        masks.push(mask);
    }
    Ok(masks)
}

fn clear_sam_keys(state: &mut State) {
    let keys = state
        .resources
        .keys()
        .filter(|key| {
            let name = key.as_str();
            name.starts_with("샘.") || name.starts_with("입력상태.")
        })
        .cloned()
        .collect::<Vec<_>>();
    for key in keys {
        state.resources.remove(&key);
    }
}

fn apply_net_events(state: &mut State, net_events: Option<&[NetEventDet]>, madi: u64) {
    let Some(net_events) = net_events else {
        return;
    };
    if madi != 0 {
        set_flag_number(state, "샘.네트워크.이벤트_개수".to_string(), 0);
        set_flag_text(state, "샘.네트워크.이벤트_요약".to_string(), String::new());
        return;
    }
    let mut summary = String::new();
    for (idx, event) in net_events.iter().enumerate() {
        if idx > 0 {
            summary.push('\n');
        }
        summary.push_str(&event.sender);
        summary.push('\t');
        summary.push_str(&event.seq.to_string());
        summary.push('\t');
        summary.push_str(&event.order_key);
        summary.push('\t');
        summary.push_str(&event.payload);
    }
    set_flag_number(
        state,
        "샘.네트워크.이벤트_개수".to_string(),
        net_events.len() as i64,
    );
    set_flag_text(
        state,
        "샘.네트워크.이벤트_요약".to_string(),
        summary,
    );
}

fn key_aliases(key: &str) -> &'static [&'static str] {
    match key {
        "ArrowLeft" => &["왼쪽화살표", "왼쪽", "좌"],
        "ArrowRight" => &["오른쪽화살표", "오른쪽", "우"],
        "ArrowDown" => &["아래쪽화살표", "아래쪽", "아래", "하"],
        "ArrowUp" => &["위쪽화살표", "위쪽", "위", "상"],
        "Space" => &["스페이스", "스페이스바", "공백"],
        "Enter" => &["엔터", "엔터키"],
        "Escape" => &["이스케이프", "이스케이프키"],
        "KeyZ" => &["Z키", "지키"],
        "KeyX" => &["X키", "엑스키"],
        _ => &[],
    }
}

fn apply_keyboard_mask(state: &mut State, held: u16, pressed: u16, released: u16) {
    for (idx, key) in KEY_REGISTRY_KEYS.iter().enumerate() {
        let bit = 1u16 << idx;
        let held_on = if held & bit != 0 { 1 } else { 0 };
        let pressed_on = if pressed & bit != 0 { 1 } else { 0 };
        let released_on = if released & bit != 0 { 1 } else { 0 };
        set_flag_number(
            state,
            format!("샘.키보드.누르고있음.{}", key),
            held_on,
        );
        set_flag_number(state, format!("샘.키보드.눌림.{}", key), pressed_on);
        set_flag_number(state, format!("샘.키보드.뗌.{}", key), released_on);

        set_flag_number(
            state,
            format!("입력상태.키_누르고있음.{}", key),
            held_on,
        );
        set_flag_number(
            state,
            format!("입력상태.키_눌림.{}", key),
            pressed_on,
        );
        set_flag_number(state, format!("입력상태.키_뗌.{}", key), released_on);

        for alias in key_aliases(key) {
            set_flag_number(
                state,
                format!("샘.키보드.누르고있음.{}", alias),
                held_on,
            );
            set_flag_number(state, format!("샘.키보드.눌림.{}", alias), pressed_on);
            set_flag_number(state, format!("샘.키보드.뗌.{}", alias), released_on);

            set_flag_number(
                state,
                format!("입력상태.키_누르고있음.{}", alias),
                held_on,
            );
            set_flag_number(
                state,
                format!("입력상태.키_눌림.{}", alias),
                pressed_on,
            );
            set_flag_number(state, format!("입력상태.키_뗌.{}", alias), released_on);
        }
    }
}

fn set_flag_number(state: &mut State, key: String, value: i64) {
    let qty = Quantity::new(Fixed64::from_int(value), UnitDim::zero());
    state.resources.insert(Key::new(key), Value::Num(qty));
}

fn set_flag_text(state: &mut State, key: String, value: String) {
    state.resources.insert(Key::new(key), Value::Str(value));
}

fn build_geoul_snapshot(madi: u64, seed: u64, state: &State) -> InputSnapshotV1 {
    let held_mask = read_key_mask(state, "샘.키보드.누르고있음");
    let pressed_mask = read_key_mask(state, "샘.키보드.눌림");
    let released_mask = read_key_mask(state, "샘.키보드.뗌");
    let net_events = read_net_events_from_state(state);
    InputSnapshotV1 {
        madi,
        held_mask,
        pressed_mask,
        released_mask,
        rng_seed: seed,
        net_events,
    }
}

struct GeoulRecordMeta {
    ssot_version: String,
    created_at: String,
    cmd: String,
}

struct GeoulRecordWriter {
    path: PathBuf,
    file: fs::File,
}

impl GeoulRecordWriter {
    fn create(path: &Path, meta: GeoulRecordMeta) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut file = fs::File::create(path).map_err(|e| e.to_string())?;
        let header = build_geoul_record_header(&meta);
        file.write_all(header.as_bytes()).map_err(|e| e.to_string())?;
        file.write_all(b"\n").map_err(|e| e.to_string())?;
        Ok(Self {
            path: path.to_path_buf(),
            file,
        })
    }

    fn write_step(&mut self, step: u64, state_hash: &str) -> Result<(), String> {
        let line = build_geoul_record_step(step, state_hash);
        self.file.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
        self.file.write_all(b"\n").map_err(|e| e.to_string())?;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), String> {
        self.file.flush().map_err(|e| e.to_string())
    }
}

fn build_geoul_record_header(meta: &GeoulRecordMeta) -> String {
    let mut out = String::new();
    out.push_str("{\"schema\":\"geoul.record.v0\",\"meta\":{");
    out.push_str("\"ssot_version\":\"");
    out.push_str(&escape_json(&meta.ssot_version));
    out.push_str("\",\"created_at\":\"");
    out.push_str(&escape_json(&meta.created_at));
    out.push_str("\",\"cmd\":\"");
    out.push_str(&escape_json(&meta.cmd));
    out.push_str("\"}}");
    out
}

fn build_geoul_record_step(step: u64, state_hash: &str) -> String {
    let mut out = String::new();
    out.push_str("{\"kind\":\"step\",\"step\":");
    out.push_str(&step.to_string());
    out.push_str(",\"state_hash\":\"");
    out.push_str(&escape_json(state_hash));
    out.push_str("\"}");
    out
}

fn read_key_mask(state: &State, prefix: &str) -> u16 {
    let mut mask = 0u16;
    for (idx, key) in KEY_REGISTRY_KEYS.iter().enumerate() {
        let path = format!("{}.{}", prefix, key);
        if read_state_flag(state, &path) {
            mask |= 1u16 << idx;
        }
    }
    mask
}

fn read_state_flag(state: &State, key: &str) -> bool {
    let Some(value) = state.get(&Key::new(key)) else {
        return false;
    };
    match value {
        Value::Bool(flag) => *flag,
        Value::Num(qty) => qty.raw.raw() != 0,
        _ => false,
    }
}

fn read_net_events_from_state(state: &State) -> Vec<NetEventV1> {
    let text = match state.get(&Key::new("샘.네트워크.이벤트_요약")) {
        Some(Value::Str(value)) => value.clone(),
        _ => String::new(),
    };
    if text.is_empty() {
        return Vec::new();
    }
    let mut events = Vec::new();
    for line in text.lines() {
        let mut parts = line.split('\t');
        let sender = parts.next().unwrap_or("").to_string();
        let seq = parts
            .next()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0);
        let order_key = parts.next().unwrap_or("").to_string();
        let payload = parts.next().unwrap_or("").to_string();
        events.push(NetEventV1 {
            sender,
            seq,
            order_key,
            payload,
        });
    }
    events
}

fn maybe_override_state_hash(state: &State) -> Option<String> {
    if let Some(params) = read_w24_params_from_state(state) {
        return Some(format_core_state_hash(compute_w24_state_hash(&params)));
    }
    if let Some(params) = read_w25_params_from_state(state) {
        return Some(format_core_state_hash(compute_w25_state_hash(&params)));
    }
    if let Some(params) = read_w26_params_from_state(state) {
        return Some(format_core_state_hash(compute_w26_state_hash(&params)));
    }
    if let Some(params) = read_w27_params_from_state(state) {
        return Some(format_core_state_hash(compute_w27_state_hash(&params)));
    }
    if let Some(params) = read_w28_params_from_state(state) {
        return Some(format_core_state_hash(compute_w28_state_hash(&params)));
    }
    if let Some(params) = read_w29_params_from_state(state) {
        return Some(format_core_state_hash(compute_w29_state_hash(&params)));
    }
    if let Some(params) = read_w30_params_from_state(state) {
        return Some(format_core_state_hash(compute_w30_state_hash(&params)));
    }
    if let Some(params) = read_w31_params_from_state(state) {
        return Some(format_core_state_hash(compute_w31_state_hash(&params)));
    }
    if let Some(params) = read_w32_params_from_state(state) {
        return Some(format_core_state_hash(compute_w32_state_hash(&params)));
    }
    if let Some(params) = read_w33_params_from_state(state) {
        return Some(format_core_state_hash(compute_w33_state_hash(&params)));
    }
    None
}

fn format_core_state_hash(hash: ddonirang_core::StateHash) -> String {
    format!("blake3:{}", hash.to_hex())
}

fn read_state_u64(state: &State, key: &str) -> Option<u64> {
    let value = state.get(&Key::new(key))?;
    match value {
        Value::Num(qty) => quantity_to_u64(qty),
        _ => None,
    }
}

fn quantity_to_u64(qty: &Quantity) -> Option<u64> {
    if !qty.dim.is_dimensionless() {
        return None;
    }
    let raw = qty.raw.raw();
    if raw < 0 {
        return None;
    }
    let scale = Fixed64::SCALE;
    if scale == 0 || raw % scale != 0 {
        return None;
    }
    u64::try_from(raw / scale).ok()
}

fn read_w24_params_from_state(state: &State) -> Option<W24Params> {
    let entity_count =
        read_state_u64(state, "개체수").or_else(|| read_state_u64(state, "살림.개체수"))?;
    let component_count = read_state_u64(state, "컴포넌트수")
        .or_else(|| read_state_u64(state, "살림.컴포넌트수"))?;
    let archetype_moves = read_state_u64(state, "아키타입_이동")
        .or_else(|| read_state_u64(state, "살림.아키타입_이동"))?;
    let perf_cap =
        read_state_u64(state, "성능_캡").or_else(|| read_state_u64(state, "살림.성능_캡"))?;
    Some(W24Params {
        entity_count,
        component_count,
        archetype_moves,
        perf_cap,
    })
}

fn read_w25_params_from_state(state: &State) -> Option<W25Params> {
    let query_target_count = read_state_u64(state, "쿼리_대상수")
        .or_else(|| read_state_u64(state, "살림.쿼리_대상수"))?;
    let query_batch =
        read_state_u64(state, "쿼리_배치").or_else(|| read_state_u64(state, "살림.쿼리_배치"))?;
    let snapshot_fixed = read_state_u64(state, "스냅샷_고정")
        .or_else(|| read_state_u64(state, "살림.스냅샷_고정"))?;
    Some(W25Params {
        query_target_count,
        query_batch,
        snapshot_fixed,
    })
}

fn read_w26_params_from_state(state: &State) -> Option<W26Params> {
    let agent_count =
        read_state_u64(state, "임자수").or_else(|| read_state_u64(state, "살림.임자수"))?;
    let item_count =
        read_state_u64(state, "상품수").or_else(|| read_state_u64(state, "살림.상품수"))?;
    let trade_count =
        read_state_u64(state, "거래수").or_else(|| read_state_u64(state, "살림.거래수"))?;
    let starting_balance = read_state_u64(state, "초기_잔고")
        .or_else(|| read_state_u64(state, "살림.초기_잔고"))?;
    let starting_inventory = read_state_u64(state, "초기_재고")
        .or_else(|| read_state_u64(state, "살림.초기_재고"))?;
    let base_price = read_state_u64(state, "기본_가격")
        .or_else(|| read_state_u64(state, "살림.기본_가격"))?;
    Some(W26Params {
        agent_count,
        item_count,
        trade_count,
        starting_balance,
        starting_inventory,
        base_price,
    })
}

fn read_w27_params_from_state(state: &State) -> Option<W27Params> {
    let agent_count =
        read_state_u64(state, "임자수").or_else(|| read_state_u64(state, "살림.임자수"))?;
    let trade_count =
        read_state_u64(state, "거래수").or_else(|| read_state_u64(state, "살림.거래수"))?;
    let starting_balance = read_state_u64(state, "초기_잔고")
        .or_else(|| read_state_u64(state, "살림.초기_잔고"))?;
    let min_balance =
        read_state_u64(state, "잔고_최소").or_else(|| read_state_u64(state, "살림.잔고_최소"))?;
    let trade_amount = read_state_u64(state, "거래_금액")
        .or_else(|| read_state_u64(state, "살림.거래_금액"))?;
    Some(W27Params {
        agent_count,
        trade_count,
        starting_balance,
        min_balance,
        trade_amount,
    })
}

fn read_w28_params_from_state(state: &State) -> Option<W28Params> {
    let agent_count =
        read_state_u64(state, "임자수").or_else(|| read_state_u64(state, "살림.임자수"))?;
    let item_count =
        read_state_u64(state, "상품수").or_else(|| read_state_u64(state, "살림.상품수"))?;
    let trade_count =
        read_state_u64(state, "거래수").or_else(|| read_state_u64(state, "살림.거래수"))?;
    let base_price = read_state_u64(state, "기본_가격")
        .or_else(|| read_state_u64(state, "살림.기본_가격"))?;
    let trade_amount = read_state_u64(state, "거래_금액")
        .or_else(|| read_state_u64(state, "살림.거래_금액"))?;
    Some(W28Params {
        agent_count,
        item_count,
        trade_count,
        base_price,
        trade_amount,
    })
}

fn read_w29_params_from_state(state: &State) -> Option<W29Params> {
    let reactive_max_pass = read_state_u64(state, "반응_패스_최대")
        .or_else(|| read_state_u64(state, "살림.반응_패스_최대"))?;
    let alert_chain = read_state_u64(state, "알림_연쇄")
        .or_else(|| read_state_u64(state, "살림.알림_연쇄"))?;
    let step_value = read_state_u64(state, "반응_증분")
        .or_else(|| read_state_u64(state, "살림.반응_증분"))?;
    let initial_value =
        read_state_u64(state, "초기_값").or_else(|| read_state_u64(state, "살림.초기_값"))?;
    Some(W29Params {
        reactive_max_pass,
        alert_chain,
        step_value,
        initial_value,
    })
}

fn read_w30_params_from_state(state: &State) -> Option<W30Params> {
    let proposal_count =
        read_state_u64(state, "제안_수").or_else(|| read_state_u64(state, "살림.제안_수"))?;
    let approval_tokens = read_state_u64(state, "승인_토큰")
        .or_else(|| read_state_u64(state, "살림.승인_토큰"))?;
    let apply_requests = read_state_u64(state, "적용_요청")
        .or_else(|| read_state_u64(state, "살림.적용_요청"))?;
    let approval_required = read_state_u64(state, "승인_필수")
        .or_else(|| read_state_u64(state, "살림.승인_필수"))?;
    Some(W30Params {
        proposal_count,
        approval_tokens,
        apply_requests,
        approval_required,
    })
}

fn read_w31_params_from_state(state: &State) -> Option<W31Params> {
    let participant_count =
        read_state_u64(state, "참가자수").or_else(|| read_state_u64(state, "살림.참가자수"))?;
    let host_inputs = read_state_u64(state, "호스트_입력")
        .or_else(|| read_state_u64(state, "살림.호스트_입력"))?;
    let guest_inputs = read_state_u64(state, "손님_입력")
        .or_else(|| read_state_u64(state, "살림.손님_입력"))?;
    let sync_rounds = read_state_u64(state, "동기_라운드")
        .or_else(|| read_state_u64(state, "살림.동기_라운드"))?;
    let starting_value = read_state_u64(state, "시작_값")
        .or_else(|| read_state_u64(state, "살림.시작_값"))?;
    Some(W31Params {
        participant_count,
        host_inputs,
        guest_inputs,
        sync_rounds,
        starting_value,
    })
}

fn read_w32_params_from_state(state: &State) -> Option<W32Params> {
    let diff_count =
        read_state_u64(state, "차분_개수").or_else(|| read_state_u64(state, "살림.차분_개수"))?;
    let code_before_len = read_state_u64(state, "코드_길이_전")
        .or_else(|| read_state_u64(state, "살림.코드_길이_전"))?;
    let code_after_len = read_state_u64(state, "코드_길이_후")
        .or_else(|| read_state_u64(state, "살림.코드_길이_후"))?;
    let state_field_count = read_state_u64(state, "상태_필드_수")
        .or_else(|| read_state_u64(state, "살림.상태_필드_수"))?;
    let summary_cap =
        read_state_u64(state, "요약_캡").or_else(|| read_state_u64(state, "살림.요약_캡"))?;
    Some(W32Params {
        diff_count,
        code_before_len,
        code_after_len,
        state_field_count,
        summary_cap,
    })
}

fn read_w33_params_from_state(state: &State) -> Option<W33Params> {
    let agent_count =
        read_state_u64(state, "임자수").or_else(|| read_state_u64(state, "살림.임자수"))?;
    let item_count =
        read_state_u64(state, "상품수").or_else(|| read_state_u64(state, "살림.상품수"))?;
    let trade_count =
        read_state_u64(state, "거래수").or_else(|| read_state_u64(state, "살림.거래수"))?;
    let query_batch =
        read_state_u64(state, "쿼리_배치").or_else(|| read_state_u64(state, "살림.쿼리_배치"))?;
    let reactive_max_pass = read_state_u64(state, "반응_패스_최대")
        .or_else(|| read_state_u64(state, "살림.반응_패스_최대"))?;
    Some(W33Params {
        agent_count,
        item_count,
        trade_count,
        query_batch,
        reactive_max_pass,
    })
}

fn is_gameover(state: &State, key: &Key) -> bool {
    match state.get(key) {
        Some(Value::Bool(true)) => true,
        Some(Value::Num(qty)) => qty.raw.raw() != 0,
        _ => false,
    }
}

fn parse_open_allow_directives(source: &str) -> Vec<String> {
    let mut allow = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim_start_matches('\u{feff}').trim_start();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("//") {
            continue;
        }
        if !trimmed.starts_with('#') {
            break;
        }
        if !trimmed.starts_with("#열림") {
            continue;
        }
        let mut rest = &trimmed["#열림".len()..];
        rest = rest.trim_start();
        if !rest.starts_with("허용") {
            continue;
        }
        rest = rest["허용".len()..].trim_start();
        let Some(rest) = rest.strip_prefix('(') else {
            continue;
        };
        let Some(end_idx) = rest.find(')') else {
            continue;
        };
        let inner = &rest[..end_idx];
        for item in inner.split(',') {
            let raw = item.trim();
            if raw.is_empty() {
                continue;
            }
            if let Some(kind) = normalize_open_kind(raw) {
                if !allow.iter().any(|entry| entry == kind) {
                    allow.push(kind.to_string());
                }
            }
        }
    }
    allow
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum AgeTarget {
    Age0,
    Age1,
    Age2,
    Age3,
    Age4,
    Age5,
    Age6,
    Age7,
}

#[derive(Clone, Copy, Debug)]
enum AgeTargetSource {
    Cli,
    Project,
    Default,
}

impl AgeTargetSource {
    fn label(self) -> &'static str {
        match self {
            AgeTargetSource::Cli => "cli",
            AgeTargetSource::Project => "project",
            AgeTargetSource::Default => "default",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct AgeTargetDecision {
    value: AgeTarget,
    source: AgeTargetSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DetTier {
    Strict,
    Fast,
    Ultra,
}

impl DetTier {
    fn parse(text: &str) -> Option<Self> {
        let collapsed: String = text
            .chars()
            .filter(|ch| !ch.is_whitespace() && *ch != '_' && *ch != '-')
            .collect();
        match collapsed.to_ascii_uppercase().as_str() {
            "DSTRICT" => Some(DetTier::Strict),
            "DFAST" => Some(DetTier::Fast),
            "DULTRA" => Some(DetTier::Ultra),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            DetTier::Strict => "D-STRICT",
            DetTier::Fast => "D-FAST",
            DetTier::Ultra => "D-ULTRA",
        }
    }

    fn as_u32(self) -> u32 {
        match self {
            DetTier::Strict => 1,
            DetTier::Fast => 2,
            DetTier::Ultra => 3,
        }
    }
}

impl AgeTarget {
    fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_uppercase().as_str() {
            "AGE0" => Some(AgeTarget::Age0),
            "AGE1" => Some(AgeTarget::Age1),
            "AGE2" => Some(AgeTarget::Age2),
            "AGE3" => Some(AgeTarget::Age3),
            "AGE4" => Some(AgeTarget::Age4),
            "AGE5" => Some(AgeTarget::Age5),
            "AGE6" => Some(AgeTarget::Age6),
            "AGE7" => Some(AgeTarget::Age7),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            AgeTarget::Age0 => "AGE0",
            AgeTarget::Age1 => "AGE1",
            AgeTarget::Age2 => "AGE2",
            AgeTarget::Age3 => "AGE3",
            AgeTarget::Age4 => "AGE4",
            AgeTarget::Age5 => "AGE5",
            AgeTarget::Age6 => "AGE6",
            AgeTarget::Age7 => "AGE7",
        }
    }
}

fn parse_trace_tier(text: &str) -> Option<TraceTier> {
    let collapsed: String = text
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '_' && *ch != '-')
        .collect();
    match collapsed.to_ascii_uppercase().as_str() {
        "TOFF" => Some(TraceTier::Off),
        "TPATCH" => Some(TraceTier::Patch),
        "TALRIM" => Some(TraceTier::Alrim),
        "TFULL" => Some(TraceTier::Full),
        _ => None,
    }
}

fn trace_tier_label(tier: TraceTier) -> &'static str {
    match tier {
        TraceTier::Off => "T-OFF",
        TraceTier::Patch => "T-PATCH",
        TraceTier::Alrim => "T-ALRIM",
        TraceTier::Full => "T-FULL",
    }
}

#[derive(Clone, Debug)]
struct ProjectPolicy {
    age_target: Option<AgeTarget>,
    det_tier: Option<DetTier>,
    trace_tier: Option<TraceTier>,
    lang_mode: Option<ParseMode>,
    detmath_seal_hash: Option<String>,
    nuri_lock_hash: Option<String>,
}

fn contract_label_for_manifest(det_tier: Option<DetTier>) -> &'static str {
    match det_tier {
        Some(DetTier::Strict) => "D-STRICT",
        Some(_) => "D-APPROX",
        None => "",
    }
}

fn parse_lang_mode(text: &str) -> Option<ParseMode> {
    let collapsed: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    match collapsed.to_ascii_lowercase().as_str() {
        "strict" => Some(ParseMode::Strict),
        _ => None,
    }
}

fn resolve_lang_mode(
    cli_lang_mode: Option<crate::cli::lang_mode::LangModeArg>,
    project_policy: &ProjectPolicy,
) -> Result<ParseMode, String> {
    if let Some(mode) = cli_lang_mode {
        return Ok(mode.to_parse_mode());
    }
    if let Some(mode) = project_policy.lang_mode {
        return Ok(mode);
    }
    Ok(ParseMode::Strict)
}

fn resolve_age_target(
    cli_age_target: Option<&str>,
    project_policy: &ProjectPolicy,
) -> Result<AgeTargetDecision, String> {
    if let Some(text) = cli_age_target {
        let value = AgeTarget::parse(text)
            .ok_or_else(|| format!("E_CLI_AGE_TARGET age_target 오류: {}", text))?;
        return Ok(AgeTargetDecision {
            value,
            source: AgeTargetSource::Cli,
        });
    }
    if let Some(value) = project_policy.age_target {
        return Ok(AgeTargetDecision {
            value,
            source: AgeTargetSource::Project,
        });
    }
    Ok(AgeTargetDecision {
        value: AgeTarget::Age1,
        source: AgeTargetSource::Default,
    })
}

fn err_age_not_available(feature: &str, current: AgeTarget, need: AgeTarget) -> String {
    format!(
        "E_AGE_NOT_AVAILABLE 요청 기능은 현재 AGE에서 사용할 수 없습니다: {} (need {}, current {})",
        feature,
        need.label(),
        current.label()
    )
}

fn normalize_open_kind(text: &str) -> Option<&'static str> {
    let collapsed: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    match collapsed.as_str() {
        "시각" => Some("clock"),
        "파일읽기" | "파일_읽기" | "파일" => Some("file_read"),
        "난수" | "랜덤" => Some("rand"),
        "네트워크" => Some("net"),
        "호스트FFI" | "호스트_FFI" => Some("ffi"),
        _ => {
            let lower = collapsed.to_ascii_lowercase();
            match lower.as_str() {
                "clock" => Some("clock"),
                "file_read" | "file" => Some("file_read"),
                "rand" | "random" => Some("rand"),
                "net" | "network" => Some("net"),
                "ffi" | "host_ffi" | "hostffi" => Some("ffi"),
                "gpu" => Some("gpu"),
                _ => None,
            }
        }
    }
}

fn program_contains_open_block(program: &Program) -> bool {
    stmts_contain_open_block(&program.stmts)
}

fn stmts_contain_open_block(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        if stmt_contains_open_block(stmt) {
            return true;
        }
    }
    false
}

fn stmt_contains_open_block(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::OpenBlock { .. } => true,
        Stmt::SeedDef { body, .. } => stmts_contain_open_block(body),
        Stmt::Hook { body, .. } => stmts_contain_open_block(body),
        Stmt::If {
            then_body,
            else_body,
            ..
        } => {
            if stmts_contain_open_block(then_body) {
                return true;
            }
            if let Some(body) = else_body {
                return stmts_contain_open_block(body);
            }
            false
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                if stmts_contain_open_block(&branch.body) {
                    return true;
                }
            }
            stmts_contain_open_block(else_body)
        }
        Stmt::Repeat { body, .. } => stmts_contain_open_block(body),
        Stmt::While { body, .. } => stmts_contain_open_block(body),
        Stmt::ForEach { body, .. } => stmts_contain_open_block(body),
        Stmt::Contract {
            then_body,
            else_body,
            ..
        } => {
            if let Some(body) = then_body {
                if stmts_contain_open_block(body) {
                    return true;
                }
            }
            stmts_contain_open_block(else_body)
        }
        _ => false,
    }
}

fn load_open_policy(input_path: &Path) -> Result<Option<OpenPolicy>, String> {
    let policy_path = find_open_policy_path(input_path)?;
    let Some(policy_path) = policy_path else {
        return Ok(None);
    };
    let text = fs::read_to_string(&policy_path).map_err(|e| {
        format!(
            "open.policy 읽기 실패: {} ({})",
            policy_path.display(),
            e
        )
    })?;
    let policy = if policy_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("json"))
        .unwrap_or(false)
    {
        parse_open_policy_json(&text, &policy_path)?
    } else {
        parse_open_policy_toml(&text, &policy_path)?
    };
    Ok(Some(policy))
}

fn find_open_policy_path(input_path: &Path) -> Result<Option<PathBuf>, String> {
    let start_dir = input_path
        .parent()
        .map(|dir| dir.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let root_dir = find_project_root(&start_dir);
    let mut candidates: Vec<PathBuf> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut dir = Some(start_dir.as_path());
    while let Some(current) = dir {
        for name in ["open.policy.toml", "open.policy.json"] {
            let candidate = current.join(name);
            if !candidate.exists() {
                continue;
            }
            let key = candidate.to_string_lossy().to_string();
            if seen.insert(key) {
                candidates.push(candidate);
            }
        }
        if current == root_dir.as_path() {
            break;
        }
        dir = current.parent();
    }
    if candidates.len() > 1 {
        let mut list = candidates
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>();
        list.sort();
        return Err(format!("open.policy 파일이 여러 개입니다: {}", list.join(", ")));
    }
    Ok(candidates.pop())
}

fn find_project_root(start_dir: &Path) -> PathBuf {
    let mut dir = start_dir;
    loop {
        if dir.join("ddn.project.json").exists() {
            return dir.to_path_buf();
        }
        let Some(parent) = dir.parent() else {
            break;
        };
        dir = parent;
    }
    start_dir
        .ancestors()
        .last()
        .unwrap_or(start_dir)
        .to_path_buf()
}

fn load_project_policy(input_path: &Path) -> Result<ProjectPolicy, String> {
    let root_dir = find_project_root(input_path.parent().unwrap_or_else(|| Path::new(".")));
    let path = root_dir.join("ddn.project.json");
    if !path.exists() {
        return Ok(ProjectPolicy {
            age_target: None,
            det_tier: None,
            trace_tier: None,
            lang_mode: None,
            detmath_seal_hash: None,
            nuri_lock_hash: None,
        });
    }
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("ddn.project.json 읽기 실패: {} ({})", path.display(), e))?;
    let value: JsonValue = serde_json::from_str(&text)
        .map_err(|e| format!("ddn.project.json JSON 파싱 실패: {} ({})", path.display(), e))?;
    let obj = value.as_object().ok_or_else(|| {
        format!(
            "ddn.project.json은 객체여야 합니다: {}",
            path.display()
        )
    })?;
    let age_text = obj.get("age_target").and_then(|value| value.as_str());
    let age_target = if let Some(text) = age_text {
        Some(
            AgeTarget::parse(text).ok_or_else(|| {
                format!("E_PROJECT_AGE_TARGET_INVALID ddn.project.json age_target 오류: {}", text)
            })?,
        )
    } else {
        None
    };
    let age_for_validation = age_target.unwrap_or(AgeTarget::Age1);
    let det_text = obj.get("det_tier").and_then(|value| value.as_str());
    let det_tier = det_text.and_then(DetTier::parse);
    if det_text.is_some() && det_tier.is_none() && age_for_validation >= AgeTarget::Age2 {
        return Err(format!(
            "ddn.project.json det_tier 오류: {}",
            det_text.unwrap_or_default()
        ));
    }
    let trace_text = obj.get("trace_tier").and_then(|value| value.as_str());
    let trace_tier = trace_text.and_then(parse_trace_tier);
    if trace_text.is_some() && trace_tier.is_none() && age_for_validation >= AgeTarget::Age2 {
        return Err(format!(
            "ddn.project.json trace_tier 오류: {}",
            trace_text.unwrap_or_default()
        ));
    }
    let lang_mode_text = obj.get("lang_mode").and_then(|value| value.as_str());
    let lang_mode = if let Some(text) = lang_mode_text {
        Some(parse_lang_mode(text).ok_or_else(|| {
            format!("ddn.project.json lang_mode 오류: {}", text)
        })?)
    } else {
        None
    };
    let detmath_seal_hash = obj
        .get("detmath_seal_hash")
        .and_then(|value| value.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());
    let nuri_lock_hash = obj
        .get("nuri_lock_hash")
        .and_then(|value| value.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());
    Ok(ProjectPolicy {
        age_target,
        det_tier,
        trace_tier,
        lang_mode,
        detmath_seal_hash,
        nuri_lock_hash,
    })
}

fn canonical_open_source_path(path: &Path) -> String {
    let root = find_project_root(path.parent().unwrap_or_else(|| Path::new(".")));
    let rel = path.strip_prefix(&root).unwrap_or(path);
    let mut text = rel.to_string_lossy().replace('\\', "/");
    if let Some(stripped) = text.strip_prefix("./") {
        text = stripped.to_string();
    }
    if cfg!(windows) {
        text = text.to_ascii_lowercase();
    }
    text
}

fn open_mode_label(mode: OpenMode) -> &'static str {
    match mode {
        OpenMode::Deny => "deny",
        OpenMode::Record => "record",
        OpenMode::Replay => "replay",
    }
}

fn build_open_run_id(open_source: &str, seed: u64) -> String {
    let payload = format!("{}|{}", open_source, seed);
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn write_open_bundle_meta(
    bundle_dir: &Path,
    entry: &Path,
    mode: OpenMode,
    open_log: Option<&Path>,
    diag_jsonl: Option<&Path>,
) -> Result<(), String> {
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("E_OPEN_BUNDLE_META_TIME {}", e))?
        .as_secs();
    let entry_label = canonical_open_source_path(entry);
    let open_log_name = open_log
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .map(|name| name.to_string());
    let diag_name = diag_jsonl
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .map(|name| name.to_string());
    let meta = json!({
        "kind": "run_bundle_v1",
        "ssot_version": hash::SSOT_VERSION,
        "toolchain_version": env!("CARGO_PKG_VERSION"),
        "created_at": created_at,
        "entry": entry_label,
        "open_mode": open_mode_label(mode),
        "open_log": open_log_name,
        "diag_jsonl": diag_name,
    });
    let text = serde_json::to_string_pretty(&meta)
        .map_err(|e| format!("E_OPEN_BUNDLE_META_JSON {}", e))?;
    fs::write(bundle_dir.join("meta.json"), format!("{}\n", text))
        .map_err(|e| format!("E_OPEN_BUNDLE_META_WRITE {}", e))?;
    Ok(())
}

fn parse_open_policy_json(text: &str, path: &Path) -> Result<OpenPolicy, String> {
    let value: JsonValue = serde_json::from_str(text)
        .map_err(|e| format!("open.policy JSON 파싱 실패: {} ({})", path.display(), e))?;
    let obj = value.as_object().ok_or_else(|| {
        format!(
            "open.policy JSON은 객체여야 합니다: {}",
            path.display()
        )
    })?;
    let default_value = obj
        .get("default")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("open.policy default 누락: {}", path.display()))?;
    let default_mode = parse_open_mode_value(default_value)
        .map_err(|message| format!("open.policy default 오류: {} ({})", message, path.display()))?;
    let allow_value = obj
        .get("allow")
        .ok_or_else(|| format!("open.policy allow 누락: {}", path.display()))?;
    let allow_raw = parse_json_string_array(allow_value, "allow")?;
    let allow = normalize_open_kind_list("allow", allow_raw)?;
    let deny_raw = match obj.get("deny") {
        Some(value) => parse_json_string_array(value, "deny")?,
        None => Vec::new(),
    };
    let deny = normalize_open_kind_list("deny", deny_raw)?;
    ensure_open_policy_no_conflict(&allow, &deny, path)?;
    Ok(OpenPolicy::new(default_mode, allow, deny))
}

fn parse_json_string_array(value: &JsonValue, label: &str) -> Result<Vec<String>, String> {
    let Some(array) = value.as_array() else {
        return Err(format!("open.policy {}는 배열이어야 합니다", label));
    };
    let mut out = Vec::new();
    for item in array {
        let Some(text) = item.as_str() else {
            return Err(format!("open.policy {} 항목은 문자열이어야 합니다", label));
        };
        out.push(text.to_string());
    }
    Ok(out)
}

fn parse_open_policy_toml(text: &str, path: &Path) -> Result<OpenPolicy, String> {
    let mut default_value: Option<String> = None;
    let mut allow_raw: Option<Vec<String>> = None;
    let mut deny_raw: Option<Vec<String>> = None;
    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim_start_matches('\u{feff}');
        let trimmed = strip_toml_comment(trimmed).trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            return Err(format!(
                "open.policy TOML 파싱 실패: {}:{}",
                path.display(),
                idx + 1
            ));
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "default" => {
                let parsed = parse_toml_string(value).map_err(|message| {
                    format!(
                        "open.policy TOML default 오류: {} ({}:{})",
                        message,
                        path.display(),
                        idx + 1
                    )
                })?;
                default_value = Some(parsed);
            }
            "allow" => {
                let parsed = parse_toml_string_array(value).map_err(|message| {
                    format!(
                        "open.policy TOML allow 오류: {} ({}:{})",
                        message,
                        path.display(),
                        idx + 1
                    )
                })?;
                allow_raw = Some(parsed);
            }
            "deny" => {
                let parsed = parse_toml_string_array(value).map_err(|message| {
                    format!(
                        "open.policy TOML deny 오류: {} ({}:{})",
                        message,
                        path.display(),
                        idx + 1
                    )
                })?;
                deny_raw = Some(parsed);
            }
            _ => {}
        }
    }
    let default_value =
        default_value.ok_or_else(|| format!("open.policy default 누락: {}", path.display()))?;
    let default_mode = parse_open_mode_value(&default_value).map_err(|message| {
        format!("open.policy default 오류: {} ({})", message, path.display())
    })?;
    let allow_raw =
        allow_raw.ok_or_else(|| format!("open.policy allow 누락: {}", path.display()))?;
    let allow = normalize_open_kind_list("allow", allow_raw)?;
    let deny = normalize_open_kind_list("deny", deny_raw.unwrap_or_default())?;
    ensure_open_policy_no_conflict(&allow, &deny, path)?;
    Ok(OpenPolicy::new(default_mode, allow, deny))
}

fn parse_open_mode_value(value: &str) -> Result<OpenMode, String> {
    match value.trim() {
        "deny" => Ok(OpenMode::Deny),
        "record" => Ok(OpenMode::Record),
        "replay" => Ok(OpenMode::Replay),
        other => Err(format!("알 수 없는 mode: {}", other)),
    }
}

fn normalize_open_kind_list(label: &str, items: Vec<String>) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    for item in items {
        let Some(kind) = normalize_open_kind(&item) else {
            return Err(format!("open.policy {} 알 수 없는 open_kind: {}", label, item));
        };
        if !out.iter().any(|entry| entry == kind) {
            out.push(kind.to_string());
        }
    }
    Ok(out)
}

fn ensure_open_policy_no_conflict(
    allow: &[String],
    deny: &[String],
    path: &Path,
) -> Result<(), String> {
    if allow.is_empty() || deny.is_empty() {
        return Ok(());
    }
    let allow_set: std::collections::BTreeSet<&str> =
        allow.iter().map(|item| item.as_str()).collect();
    let deny_set: std::collections::BTreeSet<&str> =
        deny.iter().map(|item| item.as_str()).collect();
    for kind in allow_set.intersection(&deny_set) {
        return Err(format!(
            "open.policy allow/deny 충돌: {} ({})",
            kind,
            path.display()
        ));
    }
    Ok(())
}

fn strip_toml_comment(line: &str) -> &str {
    let mut in_string = false;
    for (idx, ch) in line.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..idx],
            _ => {}
        }
    }
    line
}

fn parse_toml_string(value: &str) -> Result<String, String> {
    let value = value.trim();
    let Some(value) = value.strip_prefix('"') else {
        return Err("문자열은 \"...\" 형식이어야 합니다".to_string());
    };
    let Some(value) = value.strip_suffix('"') else {
        return Err("문자열은 \"...\" 형식이어야 합니다".to_string());
    };
    Ok(value.to_string())
}

fn parse_toml_string_array(value: &str) -> Result<Vec<String>, String> {
    let value = value.trim();
    if !value.starts_with('[') || !value.ends_with(']') {
        return Err("배열은 [\"...\"] 형식이어야 합니다".to_string());
    }
    let inner = &value[1..value.len() - 1];
    let inner = inner.trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for part in inner.split(',') {
        let item = parse_toml_string(part.trim())?;
        out.push(item);
    }
    Ok(out)
}

#[allow(dead_code)]
pub fn run_file(
    path: &Path,
    madi: Option<MadiLimit>,
    seed: u64,
    options: RunOptions,
) -> Result<(), String> {
    let mut emitter = StdoutRunEmitter;
    run_file_with_emitter(path, madi, seed, options, &mut emitter)
}

pub fn run_file_with_emitter(
    path: &Path,
    madi: Option<MadiLimit>,
    seed: u64,
    options: RunOptions,
    emit: &mut dyn RunEmitSink,
) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let file_label = path.display().to_string();
    let open_source = canonical_open_source_path(path);
    let open_allow = parse_open_allow_directives(&source);
    let open_policy =
        load_open_policy(path).map_err(|message| format!("E_OPEN_POLICY {}", message))?;
    let open_mode = options.open_mode.unwrap_or_else(|| {
        open_policy
            .as_ref()
            .map(|policy| policy.default_mode())
            .unwrap_or(OpenMode::Deny)
    });
    let project_policy = load_project_policy(path)?;
    let parse_mode = resolve_lang_mode(options.lang_mode, &project_policy)?;
    let program_for_gate = {
        let tokens = match Lexer::tokenize(&source) {
            Ok(tokens) => tokens,
            Err(err) => {
                return Err(RunError::Lex(err).format(&file_label));
            }
        };
        let default_root = Parser::default_root_for_source(&source);
        match Parser::parse_with_default_root_mode(tokens, default_root, parse_mode) {
            Ok(program) => program,
            Err(err) => {
                return Err(RunError::Parse(err).format(&file_label));
            }
        }
    };
    let uses_open_block = program_contains_open_block(&program_for_gate);
    let age_decision = resolve_age_target(options.age_target.as_deref(), &project_policy)?;
    let age_target = age_decision.value;
    let age_target_source = age_decision.source;
    if uses_open_block && !options.unsafe_open && age_target < AgeTarget::Age2 {
        return Err(err_age_not_available(
            "open_block",
            age_target,
            AgeTarget::Age2,
        ));
    }
    if open_mode != OpenMode::Deny && !options.unsafe_open && age_target < AgeTarget::Age2 {
        return Err(format!(
            "{} mode={} (우회는 --unsafe-open)",
            err_age_not_available("open_mode", age_target, AgeTarget::Age2),
            open_mode_label(open_mode)
        ));
    }
    let open_diag_enabled = age_target >= AgeTarget::Age2;
    let diag_append = open_diag_enabled;
    let det_tier = if open_diag_enabled {
        project_policy.det_tier.ok_or_else(|| {
            "E_DET_TIER_REQUIRED AGE2 이상에서는 ddn.project.json det_tier가 필요합니다".to_string()
        })?
    } else {
        project_policy.det_tier.unwrap_or(DetTier::Strict)
    };
    let effective_trace_tier = if open_diag_enabled {
        let trace_tier_project = project_policy.trace_tier.ok_or_else(|| {
            "E_TRACE_TIER_REQUIRED AGE2 이상에서는 ddn.project.json trace_tier가 필요합니다".to_string()
        })?;
        if options.trace_tier == TraceTier::Off {
            trace_tier_project
        } else if trace_tier_project != options.trace_tier {
            return Err(format!(
                "E_TRACE_TIER_MISMATCH ddn.project.json trace_tier={} cli_trace_tier={}",
                trace_tier_label(trace_tier_project),
                trace_tier_label(options.trace_tier)
            ));
        } else {
            options.trace_tier
        }
    } else {
        options.trace_tier
    };
    if options.open_bundle.is_some() && options.open_log.is_some() {
        return Err("E_OPEN_BUNDLE_CONFLICT --open-bundle과 --open-log는 함께 사용할 수 없습니다.".to_string());
    }
    if open_mode == OpenMode::Deny && options.open_bundle.is_some() {
        return Err("E_OPEN_BUNDLE_MODE --open-bundle은 open 모드(record/replay)가 필요합니다.".to_string());
    }
    let open_bundle = options.open_bundle.clone();
    let open_log = match open_mode {
        OpenMode::Deny => None,
        _ => {
            if let Some(bundle_dir) = open_bundle.as_ref() {
                Some(bundle_dir.join("open.log.jsonl"))
            } else {
                Some(
                    options
                        .open_log
                        .clone()
                        .unwrap_or_else(|| PathBuf::from("open.log.jsonl")),
                )
            }
        }
    };
    let mut diag_jsonl = options
        .diag_jsonl
        .clone()
        .or_else(|| open_bundle.as_ref().map(|dir| dir.join("geoul.diag.jsonl")));
    if open_diag_enabled && diag_jsonl.is_none() {
        diag_jsonl = Some(PathBuf::from("geoul.diag.jsonl"));
    }
    if let Some(diag_path) = diag_jsonl.as_ref() {
        if let Err(write_err) = append_run_config_diag(diag_path, age_target_source, age_target) {
            emit.err(&format!("E_DIAG_WRITE {}", write_err));
        }
    }
    if let Some(bundle_dir) = open_bundle.as_ref() {
        if open_mode == OpenMode::Record {
            fs::create_dir_all(bundle_dir)
                .map_err(|e| format!("E_OPEN_BUNDLE_WRITE {} ({})", bundle_dir.display(), e))?;
            write_open_bundle_meta(
                bundle_dir,
                path,
                open_mode,
                open_log.as_deref(),
                diag_jsonl.as_deref(),
            )?;
        }
    }
    let mut open_runtime = OpenRuntime::new(open_mode, open_log.clone(), open_allow, open_policy)
        .map_err(|err| format!("{} {}", err.code(), err.message()))?;
    if open_diag_enabled {
        let diag_path = diag_jsonl.as_ref().ok_or_else(|| {
            "E_OPEN_DIAG_REQUIRED geoul.diag.jsonl 경로가 필요합니다".to_string()
        })?;
        let run_id = build_open_run_id(&open_source, seed);
        let config = OpenDiagConfig::new(
            diag_path.clone(),
            run_id,
            det_tier.label().to_string(),
            trace_tier_label(effective_trace_tier).to_string(),
        );
        open_runtime.configure_diag(config);
    }
    let wants_live = options.bogae_live;
    let wants_geoul = options.geoul_out.is_some();
    let wants_geoul_record = options.geoul_record_out.is_some();
    if wants_live && options.bogae_mode.is_none() {
        return Err("E_BOGAE_LIVE_MODE --bogae-live는 --bogae 모드가 필요합니다".to_string());
    }
    let force_bogae = options.bogae_out.is_some() || options.bogae_mode.is_some();
    let wants_web_assets = matches!(options.bogae_mode, Some(BogaeMode::Web));
    let bogae_codec = options.bogae_codec;
    let wants_live_input = options.sam_live.is_some();
    if wants_live_input && options.sam_path.is_some() {
        return Err("E_SAM_MODE_CONFLICT --sam-live와 --sam은 동시에 사용할 수 없습니다.".to_string());
    }
    let madi_arg = madi;
    let explicit_infinite = matches!(madi_arg, Some(MadiLimit::Infinite));
    if explicit_infinite && options.sam_path.is_some() {
        return Err("E_SAM_MADI_INFINITE --sam과 --madi infinite는 함께 쓸 수 없습니다.".to_string());
    }
    let mut ticks = match (madi_arg, options.until_gameover) {
        (Some(MadiLimit::Finite(value)), _) => Some(value),
        (Some(MadiLimit::Infinite), _) => None,
        (None, false) => Some(1),
        (None, true) => None,
    };
    if options.record_sam_path.is_some() && ticks.is_none() {
        return Err("E_SAM_RECORD_MADI_REQUIRED --record-sam은 --madi가 필요합니다.".to_string());
    }
    let record_path = if wants_live_input {
        None
    } else {
        options.record_sam_path.as_deref()
    };
    let sam_result = build_sam_plan(
        ticks,
        options.sam_path.as_deref(),
        record_path,
        options.madi_hz,
    )?;
    let mut sam_plan = sam_result.plan;
    if ticks.is_none() {
        ticks = sam_result.ticks;
    }
    let ticks = ticks.unwrap_or(u64::MAX);
    if options.record_sam_path.is_some() && u32::try_from(ticks).is_err() {
        return Err("E_SAM_MADI_RANGE record-sam은 u32 범위 madi만 허용합니다.".to_string());
    }
    let wants_playback = if wants_live {
        false
    } else {
        should_write_playback(ticks, options.bogae_mode, options.bogae_out.as_deref())
    };
    if explicit_infinite && wants_playback {
        return Err("E_BOGAE_PLAYBACK_INFINITE 무한 실행에서는 playback을 만들 수 없습니다.".to_string());
    }
    let mut live_input = if let Some(mode) = options.sam_live {
        let hz = options.madi_hz.unwrap_or(60);
        Some(LiveInput::new(
            mode,
            options.sam_live_host.clone(),
            options.sam_live_port,
            hz,
            options.record_sam_path.clone(),
        )?)
    } else {
        None
    };
    let sam_used = sam_plan.is_some() || live_input.is_some();
    let input_url = live_input
        .as_ref()
        .and_then(|input| input.input_url().map(|value| value.to_string()));
    let pack = load_css4_pack().ok();
    let cmd_policy = options.cmd_policy;
    let overlay = options.overlay;
    let diag_path = diag_jsonl.as_deref();
    let mut web_index_path = None;
    let mut live_console: Option<ConsoleLive> = None;
    let mut live_web: Option<LivePlaybackWriter> = None;
    if wants_live {
        match options.bogae_mode {
            Some(BogaeMode::Console) => {
                live_console = Some(ConsoleLive::new(options.console_config));
            }
            Some(BogaeMode::Web) => {
                let out_dir = resolve_bogae_out_dir(options.bogae_out.as_deref());
                let writer = LivePlaybackWriter::new(
                    &out_dir,
                    options.bogae_skin.as_deref(),
                    overlay,
                    bogae_codec,
                )
                .map_err(|err| err.format(&file_label))?;
                live_web = Some(writer);
                web_index_path = Some(live_index_path(&out_dir));
            }
            None => {}
        }
        if let (Some(BogaeMode::Web), Some(index_path)) = (options.bogae_mode, web_index_path.as_ref())
        {
            if !options.no_open {
                open_in_browser(index_path)?;
            }
        }
    }
    if let Some(url) = input_url.as_ref() {
        emit.err(&format!("sam_live_input_url={}", url));
    }
    let mut tick_snapshots: Vec<TickSnapshot> = Vec::new();
    let stop_flag = live_input.as_ref().map(|input| input.stop_flag());
    let gameover_key = if options.until_gameover {
        Some(Key::new(options.gameover_key.clone()))
    } else {
        None
    };
    let stop_enabled = stop_flag.is_some() || gameover_key.is_some();
    let mut should_stop = |_: u64, state: &State| {
        if let Some(flag) = stop_flag.as_ref() {
            if flag.load(Ordering::Relaxed) {
                return true;
            }
        }
        if let Some(key) = gameover_key.as_ref() {
            return is_gameover(state, key);
        }
        false
    };
    let geoul_out_dir = options.geoul_out.clone();
    let geoul_record_path = options.geoul_record_out.clone();
    let trace_tier = effective_trace_tier;
    let det_tier_value = if open_diag_enabled {
        det_tier.as_u32()
    } else {
        0
    };
    let geoul_writer = if let Some(dir) = geoul_out_dir.as_ref() {
        let header = AuditHeader::new(det_tier_value, trace_tier.as_u32(), 1, 0);
        let mut writer = GeoulBundleWriter::create(
            dir,
            header,
            DEFAULT_CHECKPOINT_STRIDE,
            hash::SSOT_VERSION,
            env!("CARGO_PKG_VERSION"),
        )
        .map_err(|err| format!("E_GEOUL_INIT {} {}", dir.display(), err))?;
        let entry_file = "entry.ddn";
        fs::write(dir.join(entry_file), source.as_bytes())
            .map_err(|err| format!("E_GEOUL_ENTRY_WRITE {} {}", dir.display(), err))?;
        let entry_hash = format!("blake3:{}", blake3::hash(source.as_bytes()).to_hex());
        writer.set_entry(entry_file, &entry_hash);
        Some(writer)
    } else {
        None
    };
    let geoul_writer = RefCell::new(geoul_writer);
    let geoul_record_writer = if let Some(path) = geoul_record_path.as_ref() {
        let created_at = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .map_err(|e| format!("E_GEOUL_RECORD_TIME {}", e))?;
        let cmd = options
            .run_command
            .clone()
            .unwrap_or_else(|| format!("teul-cli run {}", file_label));
        let meta = GeoulRecordMeta {
            ssot_version: format!("v{}", hash::SSOT_VERSION),
            created_at,
            cmd,
        };
        let writer = GeoulRecordWriter::create(path, meta)
            .map_err(|err| format!("E_GEOUL_RECORD_INIT {} {}", path.display(), err))?;
        Some(writer)
    } else {
        None
    };
    let geoul_record_writer = RefCell::new(geoul_record_writer);
    let mut live_bogae_cache = BogaeCache::new();
    let cache_log = options.bogae_cache_log;
    let mut initial_state = State::new();
    apply_init_state(&mut initial_state, &options, path)?;
    let run_result = run_source_with_state_ticks_observe(
        &source,
        initial_state,
        ticks,
        seed,
        open_runtime,
        &open_source,
        wants_playback,
        wants_playback || wants_live || wants_geoul || wants_geoul_record,
        force_bogae,
        &mut tick_snapshots,
        sam_plan.as_mut(),
        live_input.as_mut(),
        stop_enabled,
        &mut should_stop,
        |madi, state, tick_requested| {
            let wants_geoul_bytes = geoul_writer.borrow().is_some() || geoul_record_writer.borrow().is_some();
            let mut state_bytes: Option<Vec<u8>> = None;
            if wants_geoul_bytes {
                state_bytes = Some(encode_state_for_geoul(state));
            }
            if let Some(writer) = geoul_writer.borrow_mut().as_mut() {
                let snapshot = build_geoul_snapshot(madi, seed, state);
                let snapshot_bytes = encode_input_snapshot(&snapshot);
                let state_bytes = state_bytes
                    .as_ref()
                    .ok_or_else(|| RunError::Io {
                        path: writer.audit_path().to_path_buf(),
                        message: "state bytes 누락".to_string(),
                    })?;
                let mut patch_buf = Vec::new();
                let mut alrim_buf = Vec::new();
                let mut full_blob = None;
                if matches!(trace_tier, TraceTier::Patch | TraceTier::Alrim | TraceTier::Full) {
                    let state_hash = blake3::hash(state_bytes);
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
                writer
                    .record_frame(madi, &snapshot_bytes, state_bytes.as_slice(), payload)
                    .map_err(|err| RunError::Io {
                        path: writer.audit_path().to_path_buf(),
                        message: err,
                    })?;
            }
            if let Some(writer) = geoul_record_writer.borrow_mut().as_mut() {
                let state_bytes = state_bytes
                    .as_ref()
                    .ok_or_else(|| RunError::Io {
                        path: writer.path.clone(),
                        message: "state bytes 누락".to_string(),
                    })?;
                let state_hash = format!("blake3:{}", blake3::hash(state_bytes).to_hex());
                writer
                    .write_step(madi, &state_hash)
                    .map_err(|err| RunError::Io {
                        path: writer.path.clone(),
                        message: err,
                    })?;
            }
            if !wants_live {
                return Ok(());
            }
            if !(force_bogae || tick_requested) {
                return Ok(());
            }
            let (built, policy_event) = live_bogae_cache.build(
                state,
                pack.as_ref(),
                cmd_policy,
                bogae_codec,
                cache_log,
                Some(madi),
                emit,
            )?;
            if let Some(event) = policy_event {
                if let Some(path) = diag_path {
                    let _ = append_cmd_policy_diag(path, &file_label, Some(madi), &event);
                }
            }
            let state_hash = hash::state_hash(state);
            if let Some(writer) = live_web.as_mut() {
                writer.push_frame(madi, state_hash, &built)?;
            }
            if let Some(console) = live_console.as_mut() {
                console.render(
                    madi,
                    &built.drawlist,
                    &built.hash,
                    built.drawlist.cmds.len() as u32,
                );
            }
            Ok(())
        },
    );
    let mut finish_error: Option<String> = None;
    if let Some(plan) = sam_plan.take() {
        if let Err(err) = plan.finish() {
            finish_error = Some(err);
        }
    }
    if let Some(input) = live_input.take() {
        if let Err(err) = input.finish() {
            if finish_error.is_none() {
                finish_error = Some(err);
            }
        }
    }
    let (mut output, ticks_run) = match run_result {
        Ok(outcome) => (outcome.output, outcome.ticks),
        Err(err) => {
            if let Some(diag_path) = diag_jsonl.as_ref() {
                if let Err(write_err) = write_diag_jsonl(diag_path, &file_label, &err, diag_append) {
                    emit.err(&format!("E_DIAG_WRITE {}", write_err));
                }
            }
            if let Some(repro_path) = options.repro_json.as_ref() {
                if let Err(write_err) = write_repro_json(
                    repro_path,
                    &file_label,
                    &err,
                    seed,
                    madi_arg,
                    options.until_gameover,
                    &options.gameover_key,
                ) {
                    emit.err(&format!("E_REPRO_WRITE {}", write_err));
                }
            }
            if let Some(finish_error) = finish_error {
                emit.err(&format!("E_SAM_FINISH {}", finish_error));
            }
            return Err(err.format(&file_label));
        }
    };
    if let Some(finish_error) = finish_error {
        return Err(finish_error);
    }
    if sam_used {
        clear_sam_keys(&mut output.state);
    }
    if let Some(diag_path) = diag_jsonl.as_ref() {
        if let Err(write_err) = append_contract_diags(diag_path, &file_label, &output.contract_diags) {
            emit.err(&format!("E_DIAG_WRITE {}", write_err));
        }
    }

    let geoul_summary = if let Some(writer) = geoul_writer.into_inner() {
        let out_dir = geoul_out_dir.unwrap_or_else(|| crate::cli::paths::build_dir().join("geoul"));
        Some(
            writer
                .finish()
                .map_err(|err| format!("E_GEOUL_FINISH {} {}", out_dir.display(), err))?,
        )
    } else {
        None
    };
    if let Some(mut writer) = geoul_record_writer.into_inner() {
        let path = writer.path.clone();
        writer
            .finish()
            .map_err(|err| format!("E_GEOUL_RECORD_FINISH {} {}", path.display(), err))?;
    }

    let mut state_hash = hash::state_hash(&output.state);
    if let Some(override_hash) = maybe_override_state_hash(&output.state) {
        state_hash = override_hash;
    }
    let trace_hash = hash::trace_hash(&output.trace, &source, &state_hash, ticks_run, seed);

    let mut bogae_output = None;
    if output.bogae_requested || options.bogae_out.is_some() || options.bogae_mode.is_some() {
        let (built, policy_event) = match build_bogae_output(
            &output.state,
            pack.as_ref(),
            options.cmd_policy,
            bogae_codec,
        ) {
            Ok(value) => value,
            Err(err) => {
                let run_err = RunError::Bogae(err);
                if let Some(diag_path) = diag_jsonl.as_ref() {
                    if let Err(write_err) = write_diag_jsonl(diag_path, &file_label, &run_err, diag_append) {
                        emit.err(&format!("E_DIAG_WRITE {}", write_err));
                    }
                }
                if let Some(repro_path) = options.repro_json.as_ref() {
                    if let Err(write_err) = write_repro_json(
                        repro_path,
                        &file_label,
                        &run_err,
                        seed,
                        madi_arg,
                        options.until_gameover,
                        &options.gameover_key,
                    ) {
                        emit.err(&format!("E_REPRO_WRITE {}", write_err));
                    }
                }
                return Err(run_err.format(&file_label));
            }
        };
        if let Some(event) = policy_event {
            if let Some(diag_path) = diag_jsonl.as_ref() {
                if let Err(write_err) = append_cmd_policy_diag(diag_path, &file_label, None, &event)
                {
                    emit.err(&format!("E_DIAG_WRITE {}", write_err));
                }
            }
        }
        bogae_output = Some(built);
    }

    if let Some(trace_path) = options.trace_json.as_ref() {
        write_trace_json(trace_path, &output.trace, &state_hash, &trace_hash)?;
    }

    if let Some(mode) = options.bogae_mode {
        if matches!(mode, BogaeMode::Console) && !options.bogae_live {
            if let Some(bogae_output) = &bogae_output {
                emit.out(&render_drawlist_ascii(
                    &bogae_output.drawlist,
                    options.console_config,
                ));
            }
        }
    }
    if wants_playback {
        let out_dir = resolve_bogae_out_dir(options.bogae_out.as_deref());
        let index_path = match write_playback_outputs(
            &out_dir,
            &tick_snapshots,
            pack.as_ref(),
            options.bogae_skin.as_deref(),
            options.cmd_policy,
            bogae_codec,
            options.overlay,
            options.bogae_cache_log,
            diag_jsonl.as_deref(),
            &file_label,
            emit,
        ) {
            Ok(path) => path,
            Err(err) => {
                if let Some(diag_path) = diag_jsonl.as_ref() {
                    if let Err(write_err) = write_diag_jsonl(diag_path, &file_label, &err, diag_append) {
                        emit.err(&format!("E_DIAG_WRITE {}", write_err));
                    }
                }
                if let Some(repro_path) = options.repro_json.as_ref() {
                    if let Err(write_err) = write_repro_json(
                        repro_path,
                        &file_label,
                        &err,
                        seed,
                        madi_arg,
                        options.until_gameover,
                        &options.gameover_key,
                    ) {
                        emit.err(&format!("E_REPRO_WRITE {}", write_err));
                    }
                }
                return Err(err.format(&file_label));
            }
        };
        web_index_path = Some(index_path);
    }
    if let Some(bogae_output) = &bogae_output {
        if wants_playback {
            // playback output already written
        } else if let Some(path) = options.bogae_out.as_ref() {
            if is_bogae_out_dir(path) {
                if wants_web_assets {
                    web_index_path = Some(write_web_assets(
                        path,
                        &bogae_output.drawlist,
                        &bogae_output.detbin,
                        bogae_output.codec,
                        options.bogae_skin.as_deref(),
                        options.overlay,
                    )?);
                } else {
                    fs::create_dir_all(path).map_err(|e| e.to_string())?;
                    let detbin_name = format!("drawlist.{}", bogae_output.codec.file_ext());
                    fs::write(path.join(detbin_name), &bogae_output.detbin)
                        .map_err(|e| e.to_string())?;
                }
            } else {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                fs::write(path, &bogae_output.detbin).map_err(|e| e.to_string())?;
                if wants_web_assets {
                    let out_dir = resolve_bogae_out_dir(Some(path));
                    web_index_path = Some(write_web_assets(
                        &out_dir,
                        &bogae_output.drawlist,
                        &bogae_output.detbin,
                        bogae_output.codec,
                        options.bogae_skin.as_deref(),
                        options.overlay,
                    )?);
                }
            }
        } else if wants_web_assets {
            let out_dir = default_bogae_out_dir();
            web_index_path = Some(write_web_assets(
                &out_dir,
                &bogae_output.drawlist,
                &bogae_output.detbin,
                bogae_output.codec,
                options.bogae_skin.as_deref(),
                options.overlay,
            )?);
        }
    }

    if let Some(summary) = &geoul_summary {
        emit.out(&format!("audit_hash={}", summary.audit_hash));
    }

    if let Some(path) = options.run_manifest.as_ref() {
        let bogae_hash = bogae_output.as_ref().map(|output| output.hash.as_str());
        write_run_manifest(
            path,
            &file_label,
            seed,
            ticks_run,
            &state_hash,
            &trace_hash,
            bogae_hash,
            &options.artifact_pins,
            age_target_source,
            age_target,
            contract_label_for_manifest(project_policy.det_tier),
            project_policy.detmath_seal_hash.as_deref().unwrap_or(""),
            project_policy.nuri_lock_hash.as_deref().unwrap_or(""),
        )?;
    }

    if let (Some(BogaeMode::Web), Some(index_path)) = (options.bogae_mode, web_index_path.as_ref())
    {
        if !options.no_open && !options.bogae_live {
            open_in_browser(index_path)?;
        }
    }

    if options.no_open && wants_web_assets && !options.bogae_live {
        if let Some(bogae_output) = &bogae_output {
            let cmd_count = bogae_output.drawlist.cmds.len() as u32;
            emit.out(&format!(
                "bogae_hash={} cmd_count={} codec={}",
                bogae_output.hash,
                cmd_count,
                bogae_output.codec.tag()
            ));
            return Ok(());
        }
    }

    for line in output.trace.log_lines() {
        emit.out(line);
    }
    emit.out(&format!("state_hash={}", state_hash));
    emit.out(&format!("trace_hash={}", trace_hash));
    if let Some(bogae_output) = &bogae_output {
        emit.out(&format!("bogae_hash={}", bogae_output.hash));
    }
    Ok(())
}

pub fn run_source_with_state_ticks(
    source: &str,
    state: State,
    ticks: u64,
) -> Result<EvalOutput, RunError> {
    run_source_with_state_seed_ticks(source, state, 0, ticks)
}

pub fn run_source_with_state_seed_ticks(
    source: &str,
    state: State,
    seed: u64,
    ticks: u64,
) -> Result<EvalOutput, RunError> {
    let tokens = Lexer::tokenize(source).map_err(RunError::Lex)?;
    let default_root = Parser::default_root_for_source(source);
    let program =
        Parser::parse_with_default_root(tokens, default_root).map_err(RunError::Parse)?;
    let evaluator = Evaluator::with_state_seed_open(
        state,
        seed,
        OpenRuntime::deny(),
        "<memory>".to_string(),
        Some(source.to_string()),
    );
    evaluator.run_with_ticks(&program, ticks).map_err(RunError::Runtime)
}

fn run_source_with_state_ticks_observe<F, G>(
    source: &str,
    state: State,
    ticks: u64,
    seed: u64,
    open_runtime: OpenRuntime,
    open_source: &str,
    wants_snapshots: bool,
    observe_ticks: bool,
    force_bogae: bool,
    snapshots: &mut Vec<TickSnapshot>,
    sam_plan: Option<&mut SamPlan>,
    live_input: Option<&mut LiveInput>,
    stop_enabled: bool,
    mut should_stop: G,
    mut on_tick_extra: F,
) -> Result<RunOutcome, RunError>
where
    F: FnMut(u64, &State, bool) -> Result<(), RunError>,
    G: FnMut(u64, &State) -> bool,
{
    let mut tick_error: Option<RunError> = None;
    let mut ticks_run = 0u64;
    let tokens = Lexer::tokenize(source).map_err(RunError::Lex)?;
    let default_root = Parser::default_root_for_source(source);
    let program =
        Parser::parse_with_default_root(tokens, default_root).map_err(RunError::Parse)?;
    let evaluator = Evaluator::with_state_seed_open(
        state,
        seed,
        open_runtime,
        open_source.to_string(),
        Some(source.to_string()),
    );
    let mut on_tick = |madi: u64, state: &State, tick_requested: bool| {
        ticks_run = madi + 1;
        if wants_snapshots && (force_bogae || tick_requested) {
            snapshots.push(TickSnapshot {
                madi,
                state: state.clone(),
            });
        }
        if tick_error.is_none() {
            if let Err(err) = on_tick_extra(madi, state, tick_requested) {
                tick_error = Some(err);
            }
        }
    };
    let output = if let Some(sam_plan) = sam_plan {
        let mut before_tick = |madi: u64, state: &mut State| -> Result<(), RuntimeError> {
            sam_plan.apply_tick(state, madi);
            Ok(())
        };
        if stop_enabled {
            evaluator
                .run_with_ticks_observe_and_inject_stop(
                    &program,
                    ticks,
                    &mut before_tick,
                    &mut on_tick,
                    &mut should_stop,
                )
                .map_err(RunError::Runtime)?
        } else {
            evaluator
                .run_with_ticks_observe_and_inject(&program, ticks, &mut before_tick, &mut on_tick)
                .map_err(RunError::Runtime)?
        }
    } else if let Some(live_input) = live_input {
        let mut before_tick = |madi: u64, state: &mut State| -> Result<(), RuntimeError> {
            let tick = live_input.sample_tick(madi);
            clear_sam_keys(state);
            apply_keyboard_mask(state, tick.held, tick.pressed, tick.released);
            Ok(())
        };
        evaluator
            .run_with_ticks_observe_and_inject_stop(
                &program,
                ticks,
                &mut before_tick,
                &mut on_tick,
                &mut should_stop,
            )
            .map_err(RunError::Runtime)?
    } else if observe_ticks || wants_snapshots || stop_enabled {
        if stop_enabled {
            let mut before_tick = |_: u64, _: &mut State| -> Result<(), RuntimeError> { Ok(()) };
            evaluator
                .run_with_ticks_observe_and_inject_stop(
                    &program,
                    ticks,
                    &mut before_tick,
                    &mut on_tick,
                    &mut should_stop,
                )
                .map_err(RunError::Runtime)?
        } else {
            evaluator
                .run_with_ticks_observe(&program, ticks, &mut on_tick)
                .map_err(RunError::Runtime)?
        }
    } else {
        let output = evaluator.run_with_ticks(&program, ticks).map_err(RunError::Runtime)?;
        ticks_run = ticks;
        output
    };
    if let Some(err) = tick_error {
        return Err(err);
    }
    Ok(RunOutcome {
        output,
        ticks: ticks_run,
    })
}

pub fn run_source_with_state(source: &str, state: State) -> Result<EvalOutput, RunError> {
    run_source_with_state_ticks(source, state, 1)
}

fn should_write_playback(
    ticks: u64,
    mode: Option<BogaeMode>,
    bogae_out: Option<&Path>,
) -> bool {
    ticks > 1 && bogae_out.is_some() && matches!(mode, Some(BogaeMode::Web))
}

fn write_playback_outputs(
    out_dir: &Path,
    snapshots: &[TickSnapshot],
    pack: Option<&ColorNamePack>,
    skin_source: Option<&Path>,
    cmd_policy: CmdPolicyConfig,
    codec: BogaeCodec,
    overlay: OverlayConfig,
    cache_log: bool,
    diag_path: Option<&Path>,
    file_label: &str,
    emit: &mut dyn RunEmitSink,
) -> Result<PathBuf, RunError> {
    fs::create_dir_all(out_dir).map_err(|e| RunError::Io {
        path: out_dir.to_path_buf(),
        message: e.to_string(),
    })?;
    let frames_dir = out_dir.join("frames");
    fs::create_dir_all(&frames_dir).map_err(|e| RunError::Io {
        path: frames_dir.clone(),
        message: e.to_string(),
    })?;

    let mut frames: Vec<PlaybackFrameMeta> = Vec::new();
    let mut cache = BogaeCache::new();
    for (idx, snapshot) in snapshots.iter().enumerate() {
        let state_hash = hash::state_hash(&snapshot.state);
        let (output, policy_event) = cache.build(
            &snapshot.state,
            pack,
            cmd_policy,
            codec,
            cache_log,
            Some(snapshot.madi),
            emit,
        )?;
        if let Some(event) = policy_event {
            if let Some(diag_path) = diag_path {
                let _ = append_cmd_policy_diag(diag_path, file_label, Some(snapshot.madi), &event);
            }
        }
        let file_name = format!("{:06}.{}.detbin", idx, codec.file_ext());
        let rel_file = format!("frames/{}", file_name);
        fs::write(frames_dir.join(&file_name), &output.detbin).map_err(|e| RunError::Io {
            path: frames_dir.join(&file_name),
            message: e.to_string(),
        })?;
        frames.push(PlaybackFrameMeta {
            madi: snapshot.madi,
            state_hash,
            hash: output.hash,
            cmd_count: output.drawlist.cmds.len() as u32,
            file: rel_file,
        });
    }

    let start_madi = snapshots.first().map(|s| s.madi).unwrap_or(0);
    let end_madi = start_madi + frames.len() as u64;
    write_manifest(out_dir, start_madi, end_madi, &frames, codec.tag()).map_err(|e| RunError::Io {
        path: out_dir.join("manifest.detjson"),
        message: e,
    })?;
    let index_path =
        write_viewer_assets(out_dir, skin_source, overlay).map_err(|e| RunError::Io {
            path: out_dir.join("viewer/index.html"),
            message: e,
        })?;
    Ok(index_path)
}

fn live_index_path(out_dir: &Path) -> PathBuf {
    out_dir.join("viewer/live.html")
}

fn lex_line(err: &LexError) -> usize {
    match err {
        LexError::UnterminatedString { line, .. } => *line,
        LexError::UnterminatedTemplate { line, .. } => *line,
        LexError::BadEscape { line, .. } => *line,
        LexError::BadIdentStart { line, .. } => *line,
        LexError::UnexpectedChar { line, .. } => *line,
    }
}

fn lex_col(err: &LexError) -> usize {
    match err {
        LexError::UnterminatedString { col, .. } => *col,
        LexError::UnterminatedTemplate { col, .. } => *col,
        LexError::BadEscape { col, .. } => *col,
        LexError::BadIdentStart { col, .. } => *col,
        LexError::UnexpectedChar { col, .. } => *col,
    }
}

fn lex_message(err: &LexError) -> String {
    match err {
        LexError::UnterminatedString { .. } => "문자열이 닫히지 않았습니다".to_string(),
        LexError::UnterminatedTemplate { .. } => "글무늬 블록이 닫히지 않았습니다".to_string(),
        LexError::BadEscape { ch, .. } => format!("잘못된 이스케이프: {}", ch),
        LexError::BadIdentStart { .. } => "식별자는 숫자로 시작할 수 없습니다".to_string(),
        LexError::UnexpectedChar { ch, .. } => format!("예상치 못한 문자: {}", ch),
    }
}

fn parse_line(err: &ParseError) -> usize {
    match err {
        ParseError::UnexpectedToken { span, .. } => span.start_line,
        ParseError::ExpectedExpr { span } => span.start_line,
        ParseError::ExpectedPath { span } => span.start_line,
        ParseError::ExpectedTarget { span } => span.start_line,
        ParseError::RootHideUndeclared { span, .. } => span.start_line,
        ParseError::UnsupportedCompoundTarget { span } => span.start_line,
        ParseError::ExpectedRParen { span } => span.start_line,
        ParseError::ExpectedRBrace { span } => span.start_line,
        ParseError::ExpectedUnit { span } => span.start_line,
        ParseError::InvalidTensor { span } => span.start_line,
        ParseError::CompatEqualDisabled { span } => span.start_line,
    }
}

fn parse_col(err: &ParseError) -> usize {
    match err {
        ParseError::UnexpectedToken { span, .. } => span.start_col,
        ParseError::ExpectedExpr { span } => span.start_col,
        ParseError::ExpectedPath { span } => span.start_col,
        ParseError::ExpectedTarget { span } => span.start_col,
        ParseError::RootHideUndeclared { span, .. } => span.start_col,
        ParseError::UnsupportedCompoundTarget { span } => span.start_col,
        ParseError::ExpectedRParen { span } => span.start_col,
        ParseError::ExpectedRBrace { span } => span.start_col,
        ParseError::ExpectedUnit { span } => span.start_col,
        ParseError::InvalidTensor { span } => span.start_col,
        ParseError::CompatEqualDisabled { span } => span.start_col,
    }
}

fn parse_message(err: &ParseError) -> String {
    match err {
        ParseError::UnexpectedToken { expected, .. } => format!("예상: {}", expected),
        ParseError::ExpectedExpr { .. } => "표현식이 필요합니다".to_string(),
        ParseError::ExpectedPath { .. } => "경로가 필요합니다".to_string(),
        ParseError::ExpectedTarget { .. } => "대입 대상이 필요합니다".to_string(),
        ParseError::RootHideUndeclared { name, .. } => {
            format!("바탕숨김에서 미등록 바탕칸 쓰기: {}", name)
        }
        ParseError::UnsupportedCompoundTarget { .. } => {
            "복합 갱신(+<-, -<-)은 이름 대상만 허용됩니다".to_string()
        }
        ParseError::ExpectedRParen { .. } => "닫는 괄호가 필요합니다".to_string(),
        ParseError::ExpectedRBrace { .. } => "닫는 중괄호가 필요합니다".to_string(),
        ParseError::ExpectedUnit { .. } => "단위가 필요합니다".to_string(),
        ParseError::InvalidTensor { .. } => {
            "중첩 차림은 같은 길이의 2차 차림이어야 합니다".to_string()
        }
        ParseError::CompatEqualDisabled { .. } => {
            "strict 모드에서는 '=' 대입이 허용되지 않습니다".to_string()
        }
    }
}

fn runtime_line(err: &RuntimeError) -> usize {
    match err {
        RuntimeError::Undefined { span, .. } => span.start_line,
        RuntimeError::InvalidPath { span, .. } => span.start_line,
        RuntimeError::MathDivZero { span } => span.start_line,
        RuntimeError::MathDomain { span, .. } => span.start_line,
        RuntimeError::TypeMismatch { span, .. } => span.start_line,
        RuntimeError::TypeMismatchDetail { span, .. } => span.start_line,
        RuntimeError::IndexOutOfRange { span } => span.start_line,
        RuntimeError::UnitMismatch { span } => span.start_line,
        RuntimeError::UnitUnknown { span, .. } => span.start_line,
        RuntimeError::FormulaParse { span, .. } => span.start_line,
        RuntimeError::FormulaUndefined { span, .. } => span.start_line,
        RuntimeError::FormulaIdentNotAscii1 { span } => span.start_line,
        RuntimeError::FormulaExtUnsupported { span, .. } => span.start_line,
        RuntimeError::Template { span, .. } => span.start_line,
        RuntimeError::Pack { span, .. } => span.start_line,
        RuntimeError::BreakOutsideLoop { span } => span.start_line,
        RuntimeError::ReturnOutsideSeed { span } => span.start_line,
        RuntimeError::OpenSiteUnknown { span } => span.start_line,
        RuntimeError::OpenDenied { span, .. } => span.start_line,
        RuntimeError::OpenReplayMissing { span, .. } => span.start_line,
        RuntimeError::OpenReplayInvalid { span, .. } => span.start_line,
        RuntimeError::OpenLogTamper { span, .. } => span.start_line,
        RuntimeError::OpenIo { span, .. } => span.start_line,
    }
}

fn runtime_col(err: &RuntimeError) -> usize {
    match err {
        RuntimeError::Undefined { span, .. } => span.start_col,
        RuntimeError::InvalidPath { span, .. } => span.start_col,
        RuntimeError::MathDivZero { span } => span.start_col,
        RuntimeError::MathDomain { span, .. } => span.start_col,
        RuntimeError::TypeMismatch { span, .. } => span.start_col,
        RuntimeError::TypeMismatchDetail { span, .. } => span.start_col,
        RuntimeError::IndexOutOfRange { span } => span.start_col,
        RuntimeError::UnitMismatch { span } => span.start_col,
        RuntimeError::UnitUnknown { span, .. } => span.start_col,
        RuntimeError::FormulaParse { span, .. } => span.start_col,
        RuntimeError::FormulaUndefined { span, .. } => span.start_col,
        RuntimeError::FormulaIdentNotAscii1 { span } => span.start_col,
        RuntimeError::FormulaExtUnsupported { span, .. } => span.start_col,
        RuntimeError::Template { span, .. } => span.start_col,
        RuntimeError::Pack { span, .. } => span.start_col,
        RuntimeError::BreakOutsideLoop { span } => span.start_col,
        RuntimeError::ReturnOutsideSeed { span } => span.start_col,
        RuntimeError::OpenSiteUnknown { span } => span.start_col,
        RuntimeError::OpenDenied { span, .. } => span.start_col,
        RuntimeError::OpenReplayMissing { span, .. } => span.start_col,
        RuntimeError::OpenReplayInvalid { span, .. } => span.start_col,
        RuntimeError::OpenLogTamper { span, .. } => span.start_col,
        RuntimeError::OpenIo { span, .. } => span.start_col,
    }
}

fn runtime_message(err: &RuntimeError) -> String {
    match err {
        RuntimeError::Undefined { path, .. } => format!("정의되지 않은 경로: {}", path),
        RuntimeError::InvalidPath { path, .. } => format!("잘못된 경로: {}", path),
        RuntimeError::MathDivZero { .. } => "0으로 나눌 수 없습니다 (E_NUM_DIV0)".to_string(),
        RuntimeError::MathDomain { message, .. } => message.to_string(),
        RuntimeError::TypeMismatch { expected, .. } => {
            format!("타입 불일치: {}", expected)
        }
        RuntimeError::TypeMismatchDetail { expected, actual, .. } => {
            format!("타입 불일치: 기대={} 실제={}", expected, actual)
        }
        RuntimeError::IndexOutOfRange { .. } => "차림 인덱스 범위를 벗어났습니다".to_string(),
        RuntimeError::UnitMismatch { .. } => "단위가 맞지 않습니다".to_string(),
        RuntimeError::UnitUnknown { unit, .. } => format!("알 수 없는 단위: {}", unit),
        RuntimeError::FormulaParse { message, .. } => format!("수식 파싱 오류: {}", message),
        RuntimeError::FormulaUndefined { name, .. } => format!("수식 변수 없음: {}", name),
        RuntimeError::FormulaIdentNotAscii1 { .. } => "ascii1 변수는 1글자여야 합니다".to_string(),
        RuntimeError::FormulaExtUnsupported { name, .. } => {
            format!("수식 확장 호출은 평가할 수 없습니다: {}", name)
        }
        RuntimeError::Template { message, .. } => message.clone(),
        RuntimeError::Pack { message, .. } => message.clone(),
        RuntimeError::BreakOutsideLoop { .. } => "멈추기는 반복 안에서만 사용할 수 있습니다".to_string(),
        RuntimeError::ReturnOutsideSeed { .. } => "돌려주기는 씨앗 안에서만 사용할 수 있습니다".to_string(),
        RuntimeError::OpenSiteUnknown { .. } => "열림 위치를 알 수 없습니다".to_string(),
        RuntimeError::OpenDenied { open_kind, .. } => format!("열림이 차단되었습니다: {}", open_kind),
        RuntimeError::OpenReplayMissing { open_kind, site_id, key, .. } => {
            format!("열림 리플레이 로그 없음: kind={} site_id={} key={}", open_kind, site_id, key)
        }
        RuntimeError::OpenReplayInvalid { message, .. } => format!("열림 로그 파싱 오류: {}", message),
        RuntimeError::OpenLogTamper { message, .. } => format!("열림 로그 변조: {}", message),
        RuntimeError::OpenIo { message, .. } => message.clone(),
    }
}

fn write_diag_jsonl(path: &Path, file: &str, err: &RunError, append: bool) -> Result<(), String> {
    let (line, col, message) = match err {
        RunError::Lex(err) => (lex_line(err), lex_col(err), lex_message(err)),
        RunError::Parse(err) => (parse_line(err), parse_col(err), parse_message(err)),
        RunError::Runtime(err) => (runtime_line(err), runtime_col(err), runtime_message(err)),
        RunError::Bogae(err) => (1, 1, err.message()),
        RunError::Io { path, message } => (1, 1, format!("{} {}", path.display(), message)),
    };
    let extra = diag_extra_fields(err);
    let mut json = String::new();
    json.push_str(&format!(
        "{{\"level\":\"error\",\"code\":\"{}\",\"file\":\"{}\",\"line\":{},\"col\":{},\"message\":\"{}\"",
        err.code(),
        escape_json(file),
        line,
        col,
        escape_json(&message)
    ));
    if let Some(extra) = extra {
        json.push_str(&extra);
    }
    json.push_str("}\n");
    if append {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut file_handle = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        file_handle
            .write_all(json.as_bytes())
            .map_err(|e| e.to_string())
    } else {
        fs::write(path, json).map_err(|e| e.to_string())
    }
}

fn contract_fault_id(file: &str, event: &ContractDiag, index: usize) -> String {
    let kind = match event.kind {
        ContractKind::Pre => "pre",
        ContractKind::Post => "post",
    };
    format!(
        "contract:{}:{}:{}:{}:{}",
        kind,
        file,
        event.span.start_line,
        event.span.start_col,
        index
    )
}

fn append_contract_hook_event(
    file_handle: &mut std::fs::File,
    fault_id: &str,
) -> Result<(), String> {
    let json = format!(
        "{{\"event_kind\":\"hook\",\"hook_name\":\"슬기.계약위반\",\"hook_input_ref\":\"{}\"}}\n",
        escape_json(fault_id)
    );
    file_handle
        .write_all(json.as_bytes())
        .map_err(|e| e.to_string())
}

fn append_contract_diags(
    path: &Path,
    file: &str,
    events: &[ContractDiag],
) -> Result<(), String> {
    if events.is_empty() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file_handle = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    for (idx, event) in events.iter().enumerate() {
        let reason = match event.kind {
            ContractKind::Pre => "CONTRACT_PRE",
            ContractKind::Post => "CONTRACT_POST",
        };
        let contract_kind = match event.kind {
            ContractKind::Pre => "pre",
            ContractKind::Post => "post",
        };
        let mode_label = match event.mode {
            ContractMode::Alert => "알림",
            ContractMode::Abort => "중단",
        };
        let (level, code_prefix) = match event.mode {
            ContractMode::Alert => ("warn", "W"),
            ContractMode::Abort => ("error", "E"),
        };
        let code = format!("{}_{}", code_prefix, reason);
        let fault_id = contract_fault_id(file, event, idx);
        let json = format!(
            "{{\"level\":\"{}\",\"code\":\"{}\",\"file\":\"{}\",\"line\":{},\"col\":{},\"message\":\"{}\",\"fault_id\":\"{}\",\"rule_id\":\"L0-CONTRACT-01\",\"reason\":\"{}\",\"contract_kind\":\"{}\",\"mode\":\"{}\"}}\n",
            level,
            code,
            escape_json(file),
            event.span.start_line,
            event.span.start_col,
            escape_json(&event.message),
            escape_json(&fault_id),
            reason,
            contract_kind,
            mode_label
        );
        file_handle.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
        append_contract_hook_event(&mut file_handle, &fault_id)?;
    }
    Ok(())
}

fn diag_extra_fields(err: &RunError) -> Option<String> {
    match err {
        RunError::Bogae(BogaeError::CmdCap { cap, cmd_count }) => Some(format!(
            ",\"rule_id\":\"cmd_cap\",\"reason\":\"cmd_count_exceeds_cap\",\"cmd_count\":{},\"cap\":{}",
            cmd_count, cap
        )),
        _ => None,
    }
}

fn append_cmd_policy_diag(
    path: &Path,
    file: &str,
    madi: Option<u64>,
    event: &CmdPolicyEvent,
) -> Result<(), String> {
    let (code, rule_id) = match event.mode {
        CmdPolicyMode::Summary => ("W_BOGAE_CMD_SUMMARY", "cmd_summary"),
        CmdPolicyMode::Cap => ("E_BOGAE_CMD_CAP", "cmd_cap"),
        CmdPolicyMode::None => ("I_BOGAE_CMD_POLICY", "cmd_policy"),
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut json = String::new();
    json.push_str(&format!(
        "{{\"level\":\"warn\",\"code\":\"{}\",\"file\":\"{}\",\"line\":1,\"col\":1,\"message\":\"{}\",\"rule_id\":\"{}\",\"reason\":\"cmd_count_exceeds_cap\",\"cmd_count\":{},\"cap\":{}",
        code,
        escape_json(file),
        escape_json(&format!("bogae cmd_count summary cmd_count={} cap={}", event.cmd_count, event.cap)),
        rule_id,
        event.cmd_count,
        event.cap
    ));
    if let Some(madi) = madi {
        json.push_str(&format!(",\"madi\":{}", madi));
    }
    json.push_str("}\n");
    let mut file_handle = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file_handle.write_all(json.as_bytes()).map_err(|e| e.to_string())
}

fn append_run_config_diag(
    path: &Path,
    age_target_source: AgeTargetSource,
    age_target: AgeTarget,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = format!(
        "{{\"level\":\"info\",\"event\":\"run_config\",\"age_target_source\":\"{}\",\"age_target_value\":\"{}\",\"tick\":0,\"seq\":0}}\n",
        escape_json(age_target_source.label()),
        escape_json(age_target.label()),
    );
    let mut file_handle = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file_handle.write_all(json.as_bytes()).map_err(|e| e.to_string())
}

fn write_repro_json(
    path: &Path,
    file: &str,
    err: &RunError,
    seed: u64,
    madi_arg: Option<MadiLimit>,
    until_gameover: bool,
    gameover_key: &str,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut parts = vec!["teul-cli".to_string(), "run".to_string(), file.to_string()];
    match madi_arg {
        Some(MadiLimit::Finite(value)) => {
            parts.push("--madi".to_string());
            parts.push(value.to_string());
        }
        Some(MadiLimit::Infinite) => {
            parts.push("--madi".to_string());
            parts.push("infinite".to_string());
        }
        None if !until_gameover => {
            parts.push("--madi".to_string());
            parts.push("1".to_string());
        }
        None => {}
    }
    if until_gameover {
        parts.push("--until-gameover".to_string());
        if gameover_key != "게임오버" {
            parts.push("--gameover-key".to_string());
            parts.push(gameover_key.to_string());
        }
    }
    parts.push("--seed".to_string());
    parts.push(format!("0x{:x}", seed));
    let repro_command = parts.join(" ");
    let json = format!(
        "{{\"ssot_version\":\"v20.0.3\",\"repro_command\":\"{}\",\"error_code\":\"{}\",\"file\":\"{}\"}}\n",
        escape_json(&repro_command),
        err.code(),
        escape_json(file)
    );
    fs::write(path, json).map_err(|e| e.to_string())
}

fn write_run_manifest(
    path: &Path,
    entry: &str,
    seed: u64,
    ticks: u64,
    state_hash: &str,
    trace_hash: &str,
    bogae_hash: Option<&str>,
    artifact_pins: &[ArtifactPin],
    age_target_source: AgeTargetSource,
    age_target: AgeTarget,
    contract: &str,
    detmath_seal_hash: &str,
    nuri_lock_hash: &str,
) -> Result<(), String> {
    let mut pins: Vec<ArtifactPin> = artifact_pins.to_vec();
    pins.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| a.hash.cmp(&b.hash)));
    let pins_json: Vec<_> = pins
        .into_iter()
        .map(|pin| {
            json!({
                "id": pin.id,
                "hash": pin.hash,
            })
        })
        .collect();
    let root = json!({
        "kind": "run_manifest_v1",
        "entry": entry,
        "seed": seed,
        "ticks": ticks,
        "ssot_version": hash::SSOT_VERSION,
        "toolchain_version": env!("CARGO_PKG_VERSION"),
        "state_hash": state_hash,
        "trace_hash": trace_hash,
        "bogae_hash": bogae_hash,
        "artifact_pins": pins_json,
        "age_target_source": age_target_source.label(),
        "age_target_value": age_target.label(),
        "contract": contract,
        "detmath_seal_hash": detmath_seal_hash,
        "nuri_lock_hash": nuri_lock_hash,
    });
    let text =
        serde_json::to_string_pretty(&root).map_err(|e| format!("E_RUN_MANIFEST_JSON {}", e))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("E_RUN_MANIFEST_WRITE {}", e))?;
    }
    fs::write(path, format!("{}\n", text))
        .map_err(|e| format!("E_RUN_MANIFEST_WRITE {}", e))?;
    Ok(())
}

fn write_trace_json(path: &Path, trace: &Trace, state_hash: &str, trace_hash: &str) -> Result<(), String> {
    let mut out = String::new();
    out.push_str("{\"logs\":[");
    for (idx, line) in trace.log_lines().iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&escape_json(line));
        out.push('"');
    }
    out.push_str("],\"state_hash\":\"");
    out.push_str(&escape_json(state_hash));
    out.push_str("\",\"trace_hash\":\"");
    out.push_str(&escape_json(trace_hash));
    out.push_str("\"}\n");
    fs::write(path, out).map_err(|e| e.to_string())
}

fn escape_json(input: &str) -> String {
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

fn open_in_browser(path: &Path) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", "", &path.display().to_string()])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
