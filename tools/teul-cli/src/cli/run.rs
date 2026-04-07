use std::cell::RefCell;
use std::collections::{BTreeSet, VecDeque};
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

use crate::canon;
use crate::cli::bogae::{
    default_bogae_out_dir, is_bogae_out_dir, resolve_bogae_out_dir, BogaeMode, OverlayConfig,
};
use crate::cli::frontdoor_parse::{
    parse_program_for_runtime, parse_program_for_runtime_with_mode, FrontdoorParseFailure,
};
use crate::cli::bogae_console::{render_drawlist_ascii, ConsoleLive, ConsoleRenderConfig};
use crate::cli::bogae_playback::{write_manifest, write_viewer_assets, PlaybackFrameMeta};
use crate::cli::bogae_web::write_web_assets;
use crate::cli::cert;
use crate::cli::input_tape::{
    mask_from_bytes, mask_to_bytes, parse_held_mask, read_input_tape, write_input_tape,
    InputRecord, InputTape, KEY_REGISTRY_KEYS,
};
use crate::cli::sam_live::{LiveInput, SamLiveMode};
use crate::core::bogae::{
    build_bogae_output, build_bogae_output_with_trace, load_css4_pack, BogaeCodec, BogaeError,
    BogaeOutput, CmdPolicyConfig, CmdPolicyEvent, CmdPolicyMode, ColorNamePack,
};
use crate::core::fixed64::Fixed64;
use crate::core::geoul::{
    encode_input_snapshot, encode_state_for_geoul, AuditHeader, GeoulBundleWriter,
    GeoulFramePayload, InputSnapshotV1, NetEventV1, TraceTier, DEFAULT_CHECKPOINT_STRIDE,
};
use crate::core::hash;
use crate::core::state::Key;
use crate::core::unit::UnitDim;
use crate::core::value::{ListValue, PackValue, Quantity, Value};
use crate::core::{State, Trace};
use crate::lang::ast::{
    ArgBinding, BinaryOp, Binding, ContractKind, ContractMode, Expr, Literal, Path as AstPath,
    Program, QuantifierKind, SeedKind, Stmt, UnaryOp,
};
use crate::lang::lexer::LexError;
use crate::lang::parser::{ParseError, ParseMode};
use crate::runtime::{
    ContractDiag, DiagnosticFailure, DiagnosticRecord, EvalFailure, EvalOutput, Evaluator,
    OpenDiagConfig, OpenInputFrame, OpenMode, OpenPolicy, OpenRuntime, ProofRuntimeEvent,
    RuntimeError,
};
use ddonirang_core::gogae3::{
    compute_w24_state_hash, compute_w25_state_hash, compute_w26_state_hash, compute_w27_state_hash,
    compute_w28_state_hash, compute_w29_state_hash, compute_w30_state_hash, compute_w31_state_hash,
    compute_w32_state_hash, compute_w33_state_hash, W24Params, W25Params, W26Params, W27Params,
    W28Params, W29Params, W30Params, W31Params, W32Params, W33Params,
};
use ddonirang_core::seulgi::latency::LATENCY_DROP_POLICY_LATE_DROP;
use ddonirang_lang::{age_not_available_error, AgeTarget};
pub enum RunError {
    Frontdoor { message: String },
    Lex(LexError),
    Parse(ParseError),
    Runtime(RuntimeError),
    Bogae(BogaeError),
    Io { path: PathBuf, message: String },
}

impl RunError {
    pub fn code(&self) -> &'static str {
        match self {
            RunError::Frontdoor { message } => frontdoor_code(message),
            RunError::Lex(err) => err.code(),
            RunError::Parse(err) => err.code(),
            RunError::Runtime(err) => err.code(),
            RunError::Bogae(err) => err.code(),
            RunError::Io { .. } => "E_IO_WRITE",
        }
    }

    pub fn format(&self, file: &str) -> String {
        match self {
            RunError::Frontdoor { message } => {
                let (code, detail) = frontdoor_code_and_detail(message);
                if detail.is_empty() {
                    format!("{} {}:1:1", code, file)
                } else {
                    format!("{} {}:1:1 {}", code, file, detail)
                }
            }
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
            RunError::Bogae(err) => format!("{} {}:1:1 {}", err.code(), file, err.message()),
            RunError::Io { path, message } => {
                format!("E_IO_WRITE {}:1:1 {}", path.display(), message)
            }
        }
    }
}

fn frontdoor_code(message: &str) -> &'static str {
    frontdoor_code_and_detail(message).0
}

fn frontdoor_code_and_detail(message: &str) -> (&'static str, &str) {
    if message.starts_with("E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN") {
        (
            "E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN",
            message
                .trim_start_matches("E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN")
                .trim_start(),
        )
    } else if message.starts_with("E_CANON_LEGACY_BOIM_FORBIDDEN") {
        (
            "E_CANON_LEGACY_BOIM_FORBIDDEN",
            message
                .trim_start_matches("E_CANON_LEGACY_BOIM_FORBIDDEN")
                .trim_start(),
        )
    } else if message.starts_with("E_FRONTDOOR_LANG_PARSER_GAP") {
        (
            "E_FRONTDOOR_LANG_PARSER_GAP",
            message
                .trim_start_matches("E_FRONTDOOR_LANG_PARSER_GAP")
                .trim_start(),
        )
    } else {
        ("E_FRONTDOOR", message)
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
    pub diag_report_out: Option<PathBuf>,
    pub repro_json: Option<PathBuf>,
    pub trace_json: Option<PathBuf>,
    pub proof_out: Option<PathBuf>,
    pub proof_cert_key: Option<PathBuf>,
    pub geoul_out: Option<PathBuf>,
    pub geoul_record_out: Option<PathBuf>,
    pub latency_madi: u64,
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

fn apply_init_state(
    state: &mut State,
    options: &RunOptions,
    source_path: &Path,
) -> Result<(), String> {
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
            entries.push(StateEntry {
                key: normalized,
                value,
            });
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
        entries.push(StateEntry {
            key: normalized,
            value,
        });
    }
    Ok(entries)
}

fn json_to_value(value: &JsonValue, base_dir: &Path) -> Result<Value, String> {
    match value {
        JsonValue::Null => Ok(Value::None),
        JsonValue::Bool(flag) => Ok(Value::Bool(*flag)),
        JsonValue::Number(num) => {
            if let Some(i) = num.as_i64() {
                Ok(Value::Num(Quantity::new(
                    Fixed64::from_int(i),
                    UnitDim::zero(),
                )))
            } else if let Some(u) = num.as_u64() {
                let signed = if u > i64::MAX as u64 {
                    i64::MAX
                } else {
                    u as i64
                };
                Ok(Value::Num(Quantity::new(
                    Fixed64::from_int(signed),
                    UnitDim::zero(),
                )))
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
        let esc = chars
            .next()
            .ok_or_else(|| "문자열 이스케이프가 끝나지 않았습니다".to_string())?;
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

struct FailedRunOutcome {
    error: RunError,
    output: Option<EvalOutput>,
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

struct InputLatencyQueue {
    latency_madi: u64,
    pending: VecDeque<InputLatencyPending>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InputLatencyPending {
    accept_madi: u64,
    target_madi: u64,
    frame: OpenInputFrame,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InputLatencyApplied {
    accept_madi: u64,
    target_madi: u64,
    late: bool,
    dropped: bool,
}

fn latency_drop_policy_label() -> &'static str {
    LATENCY_DROP_POLICY_LATE_DROP
}

impl InputLatencyQueue {
    fn new(latency_madi: u64) -> Self {
        Self {
            latency_madi,
            pending: VecDeque::new(),
        }
    }

    fn schedule_and_take(
        &mut self,
        accept_madi: u64,
        frame: OpenInputFrame,
    ) -> (OpenInputFrame, Option<InputLatencyApplied>) {
        if self.latency_madi == 0 {
            return (
                frame,
                Some(InputLatencyApplied {
                    accept_madi,
                    target_madi: accept_madi,
                    late: false,
                    dropped: false,
                }),
            );
        }
        let target_madi = accept_madi.saturating_add(self.latency_madi);
        self.pending.push_back(InputLatencyPending {
            accept_madi,
            target_madi,
            frame,
        });
        if let Some(pending) = self.pending.front().copied() {
            if pending.target_madi <= accept_madi {
                self.pending.pop_front();
                let late = accept_madi > pending.target_madi;
                return (
                    if late {
                        OpenInputFrame::new(0, 0, 0)
                    } else {
                        pending.frame
                    },
                    Some(InputLatencyApplied {
                        accept_madi: pending.accept_madi,
                        target_madi: pending.target_madi,
                        late,
                        dropped: late,
                    }),
                );
            }
        }
        (OpenInputFrame::new(0, 0, 0), None)
    }
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
        write_manifest(
            &self.out_dir,
            start_madi,
            end_madi,
            &self.frames,
            self.codec.tag(),
        )
        .map_err(|e| RunError::Io {
            path: self.out_dir.join("manifest.detjson"),
            message: e,
        })?;
        Ok(())
    }
}

impl SamPlan {
    fn frame_for_tick(&mut self, madi: u64) -> OpenInputFrame {
        let idx = madi as usize;
        let held = self.masks.get(idx).copied().unwrap_or(0);
        let pressed = (!self.last_mask) & held;
        let released = self.last_mask & !held;
        self.last_mask = held;
        OpenInputFrame::new(held, pressed, released)
    }

    fn apply_frame(&mut self, state: &mut State, madi: u64, frame: OpenInputFrame) {
        clear_sam_keys(state);
        apply_keyboard_mask(
            state,
            frame.held_mask,
            frame.pressed_mask,
            frame.released_mask,
        );
        apply_net_events(state, self.net_events.as_deref(), madi);
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
        return Err(
            "E_SAM_MODE_CONFLICT --sam과 --record-sam은 동시에 사용할 수 없습니다".to_string(),
        );
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

    let ticks = ticks.ok_or_else(|| {
        "E_SAM_RECORD_MADI_REQUIRED --record-sam은 --madi가 필요합니다.".to_string()
    })?;
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
    let tape = InputTape {
        madi_hz: hz,
        records,
    };
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
            let payload =
                serde_json::to_string(payload).map_err(|e| format!("E_SAM_DETJSON_PAYLOAD {e}"))?;
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
    set_flag_text(state, "샘.네트워크.이벤트_요약".to_string(), summary);
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
        set_flag_number(state, format!("샘.키보드.누르고있음.{}", key), held_on);
        set_flag_number(state, format!("샘.키보드.눌림.{}", key), pressed_on);
        set_flag_number(state, format!("샘.키보드.뗌.{}", key), released_on);

        set_flag_number(state, format!("입력상태.키_누르고있음.{}", key), held_on);
        set_flag_number(state, format!("입력상태.키_눌림.{}", key), pressed_on);
        set_flag_number(state, format!("입력상태.키_뗌.{}", key), released_on);

        for alias in key_aliases(key) {
            set_flag_number(state, format!("샘.키보드.누르고있음.{}", alias), held_on);
            set_flag_number(state, format!("샘.키보드.눌림.{}", alias), pressed_on);
            set_flag_number(state, format!("샘.키보드.뗌.{}", alias), released_on);

            set_flag_number(state, format!("입력상태.키_누르고있음.{}", alias), held_on);
            set_flag_number(state, format!("입력상태.키_눌림.{}", alias), pressed_on);
            set_flag_number(state, format!("입력상태.키_뗌.{}", alias), released_on);
        }
    }
}

fn apply_input_frame(state: &mut State, madi: u64, frame: OpenInputFrame) {
    clear_sam_keys(state);
    apply_keyboard_mask(
        state,
        frame.held_mask,
        frame.pressed_mask,
        frame.released_mask,
    );
    apply_net_events(state, None, madi);
}

fn input_open_site_id(file_label: &str) -> String {
    format!("{}:input", file_label)
}

fn input_open_span() -> crate::lang::span::Span {
    crate::lang::span::Span::new(1, 1, 1, 1)
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
        file.write_all(header.as_bytes())
            .map_err(|e| e.to_string())?;
        file.write_all(b"\n").map_err(|e| e.to_string())?;
        Ok(Self {
            path: path.to_path_buf(),
            file,
        })
    }

    fn write_step(&mut self, step: u64, state_hash: &str) -> Result<(), String> {
        let line = build_geoul_record_step(step, state_hash);
        self.file
            .write_all(line.as_bytes())
            .map_err(|e| e.to_string())?;
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
    let component_count =
        read_state_u64(state, "컴포넌트수").or_else(|| read_state_u64(state, "살림.컴포넌트수"))?;
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
    let starting_balance =
        read_state_u64(state, "초기_잔고").or_else(|| read_state_u64(state, "살림.초기_잔고"))?;
    let starting_inventory =
        read_state_u64(state, "초기_재고").or_else(|| read_state_u64(state, "살림.초기_재고"))?;
    let base_price =
        read_state_u64(state, "기본_가격").or_else(|| read_state_u64(state, "살림.기본_가격"))?;
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
    let starting_balance =
        read_state_u64(state, "초기_잔고").or_else(|| read_state_u64(state, "살림.초기_잔고"))?;
    let min_balance =
        read_state_u64(state, "잔고_최소").or_else(|| read_state_u64(state, "살림.잔고_최소"))?;
    let trade_amount =
        read_state_u64(state, "거래_금액").or_else(|| read_state_u64(state, "살림.거래_금액"))?;
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
    let base_price =
        read_state_u64(state, "기본_가격").or_else(|| read_state_u64(state, "살림.기본_가격"))?;
    let trade_amount =
        read_state_u64(state, "거래_금액").or_else(|| read_state_u64(state, "살림.거래_금액"))?;
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
    let alert_chain =
        read_state_u64(state, "알림_연쇄").or_else(|| read_state_u64(state, "살림.알림_연쇄"))?;
    let step_value =
        read_state_u64(state, "반응_증분").or_else(|| read_state_u64(state, "살림.반응_증분"))?;
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
    let approval_tokens =
        read_state_u64(state, "승인_토큰").or_else(|| read_state_u64(state, "살림.승인_토큰"))?;
    let apply_requests =
        read_state_u64(state, "적용_요청").or_else(|| read_state_u64(state, "살림.적용_요청"))?;
    let approval_required =
        read_state_u64(state, "승인_필수").or_else(|| read_state_u64(state, "살림.승인_필수"))?;
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
    let guest_inputs =
        read_state_u64(state, "손님_입력").or_else(|| read_state_u64(state, "살림.손님_입력"))?;
    let sync_rounds = read_state_u64(state, "동기_라운드")
        .or_else(|| read_state_u64(state, "살림.동기_라운드"))?;
    let starting_value =
        read_state_u64(state, "시작_값").or_else(|| read_state_u64(state, "살림.시작_값"))?;
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
        let mut rest = if let Some(rest) = trimmed.strip_prefix("#열림") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("#너머") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("#효과") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("#바깥") {
            rest
        } else {
            continue;
        };
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
    Sealed,
    Approx,
}

impl DetTier {
    fn parse(text: &str) -> Option<Self> {
        let collapsed: String = text
            .chars()
            .filter(|ch| !ch.is_whitespace() && *ch != '_' && *ch != '-')
            .collect();
        match collapsed.to_ascii_uppercase().as_str() {
            "DSTRICT" => Some(DetTier::Strict),
            "DSEALED" | "DFAST" => Some(DetTier::Sealed),
            "DAPPROX" | "DULTRA" => Some(DetTier::Approx),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            DetTier::Strict => "D-STRICT",
            DetTier::Sealed => "D-SEALED",
            DetTier::Approx => "D-APPROX",
        }
    }

    fn as_u32(self) -> u32 {
        match self {
            DetTier::Strict => 1,
            DetTier::Sealed => 2,
            DetTier::Approx => 3,
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
        Some(DetTier::Sealed) => "D-SEALED",
        Some(DetTier::Approx) => "D-APPROX",
        None => "",
    }
}

fn ensure_det_tier_supported(det_tier: DetTier) -> Result<(), String> {
    match det_tier {
        DetTier::Strict => Ok(()),
        DetTier::Sealed | DetTier::Approx => Err(format!(
            "E_CONTRACT_TIER_UNSUPPORTED 현재 실행기는 det_tier={} 집행을 지원하지 않습니다. D-STRICT만 허용됩니다.",
            det_tier.label()
        )),
    }
}

fn enforce_det_tier_contract(
    det_tier: DetTier,
    age_target: AgeTarget,
    detmath_seal_hash: Option<&str>,
    nuri_lock_hash: Option<&str>,
) -> Result<(), String> {
    match det_tier {
        DetTier::Strict => Ok(()),
        DetTier::Sealed => {
            if age_target < AgeTarget::Age3 {
                return ensure_det_tier_supported(det_tier);
            }
            let has_seal = detmath_seal_hash
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false);
            let has_lock = nuri_lock_hash
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false);
            if !has_seal || !has_lock {
                return Err(
                    "E_RUNTIME_CONTRACT_MISMATCH D-SEALED는 detmath_seal_hash와 nuri_lock_hash가 모두 필요합니다."
                        .to_string(),
                );
            }
            Ok(())
        }
        DetTier::Approx => {
            if age_target < AgeTarget::Age3 {
                return ensure_det_tier_supported(det_tier);
            }
            Ok(())
        }
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
    age_not_available_error(feature, need, current)
}

fn normalize_open_kind(text: &str) -> Option<&'static str> {
    let collapsed: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    match collapsed.as_str() {
        "시각" => Some("clock"),
        "파일읽기" | "파일_읽기" | "파일" => Some("file_read"),
        "입력" | "사용자입력" | "키보드입력" => Some("input"),
        "난수" | "랜덤" => Some("rand"),
        "네트워크" => Some("net"),
        "호스트FFI" | "호스트_FFI" => Some("ffi"),
        "풀이" | "해찾기" => Some("solver"),
        _ => {
            let lower = collapsed.to_ascii_lowercase();
            match lower.as_str() {
                "clock" => Some("clock"),
                "file_read" | "file" => Some("file_read"),
                "input" | "user_input" | "keyboard_input" => Some("input"),
                "rand" | "random" => Some("rand"),
                "net" | "network" => Some("net"),
                "ffi" | "host_ffi" | "hostffi" => Some("ffi"),
                "solver" => Some("solver"),
                "gpu" => Some("gpu"),
                _ => None,
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExecMode {
    Strict,
    General,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EffectPolicy {
    Isolated,
    Allowed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SeulgiHookPolicy {
    Record,
    Execute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExecPolicy {
    mode: ExecMode,
    effect: EffectPolicy,
    seulgi_hook: SeulgiHookPolicy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExecPolicyExtract {
    policy: ExecPolicy,
    strict_effect_ignored: bool,
}

fn parse_exec_mode_value(text: &str) -> Option<ExecMode> {
    match text.trim() {
        "엄밀" | "strict" | "STRICT" => Some(ExecMode::Strict),
        "일반" | "general" | "GENERAL" => Some(ExecMode::General),
        _ => None,
    }
}

fn parse_effect_policy_value(text: &str) -> Option<EffectPolicy> {
    match text.trim() {
        "격리" | "isolate" | "isolated" | "ISOLATE" | "ISOLATED" => Some(EffectPolicy::Isolated),
        "허용" | "allow" | "ALLOWED" | "ALLOW" => Some(EffectPolicy::Allowed),
        _ => None,
    }
}

fn parse_seulgi_hook_policy_value(text: &str) -> Option<SeulgiHookPolicy> {
    match text.trim() {
        "기록" | "record" | "RECORDED" | "RECORD" => Some(SeulgiHookPolicy::Record),
        "실행" | "execute" | "EXECUTE" => Some(SeulgiHookPolicy::Execute),
        _ => None,
    }
}

fn seulgi_hook_policy_label(policy: SeulgiHookPolicy) -> &'static str {
    match policy {
        SeulgiHookPolicy::Record => "기록",
        SeulgiHookPolicy::Execute => "실행",
    }
}

fn parse_exec_policy_args(args: &str) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    for (key, value) in parse_exec_policy_arg_entries(args) {
        out.insert(key, value);
    }
    out
}

fn parse_exec_policy_arg_entries(args: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for part in args.split(',') {
        let item = part.trim();
        if item.is_empty() {
            continue;
        }
        if let Some((k, v)) = item.split_once('=') {
            out.push((k.trim().to_string(), v.trim().to_string()));
            continue;
        }
        if let Some((k, v)) = item.split_once(':') {
            out.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    out
}

fn extract_exec_policy_open_allow(program: &Program) -> Vec<String> {
    let mut allow = Vec::new();
    for stmt in &program.stmts {
        let Stmt::Pragma { name, args, .. } = stmt else {
            continue;
        };
        if name != "실행정책" {
            continue;
        }
        for (key, raw_value) in parse_exec_policy_arg_entries(args) {
            if !matches!(
                key.as_str(),
                "열림허용" | "열림" | "open_allow" | "open"
            ) {
                continue;
            }
            let Some(kind) = normalize_open_kind(&raw_value) else {
                continue;
            };
            if !allow.iter().any(|entry| entry == kind) {
                allow.push(kind.to_string());
            }
        }
    }
    allow
}

fn extract_exec_policy(program: &Program) -> Result<Option<ExecPolicyExtract>, String> {
    let mut found_block = false;
    let mut has_runtime_policy_field = false;
    let mut mode: Option<ExecMode> = None;
    let mut effect: Option<EffectPolicy> = None;
    let mut effect_explicit: Option<EffectPolicy> = None;
    let mut seulgi_hook: Option<SeulgiHookPolicy> = None;

    for stmt in &program.stmts {
        let Stmt::Pragma { name, args, .. } = stmt else {
            continue;
        };
        if name == "실행정책" {
            if found_block {
                return Err(
                    "E_EXEC_POLICY_DUPLICATE 실행정책 블록은 파일 최상위 1회만 허용됩니다."
                        .to_string(),
                );
            }
            found_block = true;
            let fields = parse_exec_policy_args(args);
            if let Some(raw_mode) = fields.get("실행모드") {
                let parsed = parse_exec_mode_value(raw_mode)
                    .ok_or_else(|| format!("E_EXEC_ENUM_INVALID 실행모드 값 오류: {}", raw_mode))?;
                mode = Some(parsed);
                has_runtime_policy_field = true;
            }
            if let Some(raw_effect) = fields.get("효과정책") {
                let parsed = parse_effect_policy_value(raw_effect).ok_or_else(|| {
                    format!("E_EXEC_ENUM_INVALID 효과정책 값 오류: {}", raw_effect)
                })?;
                effect = Some(parsed);
                effect_explicit = Some(parsed);
                has_runtime_policy_field = true;
            }
            if let Some(raw_hook_policy) = fields.get("슬기훅정책") {
                let parsed = parse_seulgi_hook_policy_value(raw_hook_policy).ok_or_else(|| {
                    format!(
                        "E_EXEC_ENUM_INVALID 슬기훅정책 값 오류: {}",
                        raw_hook_policy
                    )
                })?;
                seulgi_hook = Some(parsed);
                has_runtime_policy_field = true;
            }
        }
    }

    if !found_block {
        return Ok(None);
    }
    if !has_runtime_policy_field {
        return Ok(None);
    }

    let mode = mode.unwrap_or(ExecMode::Strict);
    let strict_effect_ignored = mode == ExecMode::Strict
        && matches!(effect_explicit, Some(parsed) if parsed != EffectPolicy::Isolated);
    let mut effect = effect.unwrap_or(EffectPolicy::Isolated);
    if mode == ExecMode::Strict {
        effect = EffectPolicy::Isolated;
    }
    let seulgi_hook = seulgi_hook.unwrap_or(SeulgiHookPolicy::Record);
    Ok(Some(ExecPolicyExtract {
        policy: ExecPolicy {
            mode,
            effect,
            seulgi_hook,
        },
        strict_effect_ignored,
    }))
}

fn resolve_effective_open_mode(
    cli_mode: Option<OpenMode>,
    open_policy: Option<&OpenPolicy>,
) -> OpenMode {
    if let Some(mode) = cli_mode {
        return mode;
    }
    open_policy
        .map(|policy| policy.default_mode())
        .unwrap_or(OpenMode::Deny)
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
        Stmt::Hook { body, .. }
        | Stmt::BeatBlock { body, .. }
        | Stmt::LifecycleBlock { body, .. } => stmts_contain_open_block(body),
        Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => expr_contains_open_call(condition) || stmts_contain_open_block(body),
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
            else_body
                .as_ref()
                .is_some_and(|body| stmts_contain_open_block(body))
        }
        Stmt::Repeat { body, .. } => stmts_contain_open_block(body),
        Stmt::While { body, .. } => stmts_contain_open_block(body),
        Stmt::ForEach { body, .. } | Stmt::Quantifier { body, .. } => {
            stmts_contain_open_block(body)
        }
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => {
            sender.as_ref().is_some_and(expr_contains_open_call)
                || expr_contains_open_call(payload)
                || expr_contains_open_call(receiver)
        }
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

fn program_contains_open_call(program: &Program) -> bool {
    stmts_contain_open_call(&program.stmts)
}

fn stmts_contain_open_call(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        if stmt_contains_open_call(stmt) {
            return true;
        }
    }
    false
}

fn stmt_contains_open_call(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::ImportBlock { .. } | Stmt::ExportBlock { .. } => false,
        Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
            item.value.as_ref().is_some_and(expr_contains_open_call)
                || item.maegim.as_ref().is_some_and(|spec| {
                    spec.fields
                        .iter()
                        .any(|field| expr_contains_open_call(&field.value))
                })
        }),
        Stmt::SeedDef { body, .. } => stmts_contain_open_call(body),
        Stmt::Assign { value, .. }
        | Stmt::Expr { value, .. }
        | Stmt::Return { value, .. }
        | Stmt::Show { value, .. }
        | Stmt::Inspect { value, .. } => expr_contains_open_call(value),
        Stmt::Receive {
            condition, body, ..
        } => {
            condition.as_ref().is_some_and(expr_contains_open_call) || stmts_contain_open_call(body)
        }
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => {
            sender.as_ref().is_some_and(expr_contains_open_call)
                || expr_contains_open_call(payload)
                || expr_contains_open_call(receiver)
        }
        Stmt::Hook { body, .. }
        | Stmt::BeatBlock { body, .. }
        | Stmt::LifecycleBlock { body, .. }
        | Stmt::OpenBlock { body, .. }
        | Stmt::Repeat { body, .. }
        | Stmt::While { body, .. }
        | Stmt::ForEach { body, .. }
        | Stmt::Quantifier { body, .. } => stmts_contain_open_call(body),
        Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => expr_contains_open_call(condition) || stmts_contain_open_call(body),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expr_contains_open_call(condition)
                || stmts_contain_open_call(then_body)
                || else_body
                    .as_ref()
                    .is_some_and(|body| stmts_contain_open_call(body))
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            branches.iter().any(|branch| {
                expr_contains_open_call(&branch.condition) || stmts_contain_open_call(&branch.body)
            }) || else_body
                .as_ref()
                .is_some_and(|body| stmts_contain_open_call(body))
        }
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expr_contains_open_call(condition)
                || then_body
                    .as_ref()
                    .is_some_and(|body| stmts_contain_open_call(body))
                || stmts_contain_open_call(else_body)
        }
        Stmt::Pragma { .. } | Stmt::BogaeDraw { .. } | Stmt::Break { .. } => false,
    }
}

fn expr_contains_open_call(expr: &Expr) -> bool {
    match expr {
        Expr::Call { name, args, .. } => {
            let canon = ddonirang_lang::stdlib::canonicalize_stdlib_alias(name);
            matches!(canon, "반례찾기" | "해찾기" | "열림.풀이.확인")
                || canon.starts_with("열림.")
                || args.iter().any(|arg| expr_contains_open_call(&arg.expr))
        }
        Expr::Unary { expr, .. } => expr_contains_open_call(expr),
        Expr::Binary { left, right, .. } => {
            expr_contains_open_call(left) || expr_contains_open_call(right)
        }
        Expr::FieldAccess { target, .. } => expr_contains_open_call(target),
        Expr::SeedLiteral { body, .. } => expr_contains_open_call(body),
        Expr::TemplateFill {
            template, bindings, ..
        } => {
            expr_contains_open_call(template)
                || bindings
                    .iter()
                    .any(|binding| expr_contains_open_call(&binding.value))
        }
        Expr::Pack { bindings, .. } | Expr::FormulaEval { bindings, .. } => bindings
            .iter()
            .any(|binding| expr_contains_open_call(&binding.value)),
        Expr::FormulaFill {
            formula, bindings, ..
        } => {
            expr_contains_open_call(formula)
                || bindings
                    .iter()
                    .any(|binding| expr_contains_open_call(&binding.value))
        }
        Expr::Literal(_, _)
        | Expr::Path(_)
        | Expr::Atom { .. }
        | Expr::Formula { .. }
        | Expr::Template { .. }
        | Expr::Assertion { .. } => false,
    }
}

fn program_contains_regex_call(program: &Program) -> bool {
    stmts_contain_regex_call(&program.stmts)
}

fn program_contains_input_builtin(program: &Program) -> bool {
    stmts_contain_input_builtin(&program.stmts)
}

fn stmts_contain_input_builtin(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        if stmt_contains_input_builtin(stmt) {
            return true;
        }
    }
    false
}

fn stmt_contains_input_builtin(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::ImportBlock { .. } | Stmt::ExportBlock { .. } => false,
        Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
            item.value.as_ref().is_some_and(expr_contains_input_builtin)
                || item.maegim.as_ref().is_some_and(|spec| {
                    spec.fields
                        .iter()
                        .any(|field| expr_contains_input_builtin(&field.value))
                })
        }),
        Stmt::SeedDef { body, .. } => stmts_contain_input_builtin(body),
        Stmt::Assign { value, .. }
        | Stmt::Expr { value, .. }
        | Stmt::Return { value, .. }
        | Stmt::Show { value, .. }
        | Stmt::Inspect { value, .. } => expr_contains_input_builtin(value),
        Stmt::Receive {
            condition, body, ..
        } => {
            condition.as_ref().is_some_and(expr_contains_input_builtin)
                || stmts_contain_input_builtin(body)
        }
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => {
            sender.as_ref().is_some_and(expr_contains_input_builtin)
                || expr_contains_input_builtin(payload)
                || expr_contains_input_builtin(receiver)
        }
        Stmt::Hook { body, .. }
        | Stmt::BeatBlock { body, .. }
        | Stmt::LifecycleBlock { body, .. }
        | Stmt::OpenBlock { body, .. }
        | Stmt::Repeat { body, .. }
        | Stmt::While { body, .. }
        | Stmt::ForEach { body, .. }
        | Stmt::Quantifier { body, .. } => stmts_contain_input_builtin(body),
        Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => expr_contains_input_builtin(condition) || stmts_contain_input_builtin(body),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expr_contains_input_builtin(condition)
                || stmts_contain_input_builtin(then_body)
                || else_body
                    .as_ref()
                    .is_some_and(|body| stmts_contain_input_builtin(body))
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            branches.iter().any(|branch| {
                expr_contains_input_builtin(&branch.condition)
                    || stmts_contain_input_builtin(&branch.body)
            }) || else_body
                .as_ref()
                .is_some_and(|body| stmts_contain_input_builtin(body))
        }
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expr_contains_input_builtin(condition)
                || then_body
                    .as_ref()
                    .is_some_and(|body| stmts_contain_input_builtin(body))
                || stmts_contain_input_builtin(else_body)
        }
        Stmt::Pragma { .. } | Stmt::BogaeDraw { .. } | Stmt::Break { .. } => false,
    }
}

fn expr_contains_input_builtin(expr: &Expr) -> bool {
    match expr {
        Expr::Call { name, args, .. } => {
            let canon = ddonirang_lang::stdlib::canonicalize_stdlib_alias(name);
            matches!(
                canon,
                "입력키" | "입력키?" | "입력키!" | "눌렸나" | "막눌렸나"
            ) || args
                .iter()
                .any(|arg| expr_contains_input_builtin(&arg.expr))
        }
        Expr::Unary { expr, .. } => expr_contains_input_builtin(expr),
        Expr::Binary { left, right, .. } => {
            expr_contains_input_builtin(left) || expr_contains_input_builtin(right)
        }
        Expr::FieldAccess { target, .. } => expr_contains_input_builtin(target),
        Expr::SeedLiteral { body, .. } => expr_contains_input_builtin(body),
        Expr::TemplateFill {
            template, bindings, ..
        } => {
            expr_contains_input_builtin(template)
                || bindings
                    .iter()
                    .any(|binding| expr_contains_input_builtin(&binding.value))
        }
        Expr::Pack { bindings, .. } | Expr::FormulaEval { bindings, .. } => bindings
            .iter()
            .any(|binding| expr_contains_input_builtin(&binding.value)),
        Expr::FormulaFill {
            formula, bindings, ..
        } => {
            expr_contains_input_builtin(formula)
                || bindings
                    .iter()
                    .any(|binding| expr_contains_input_builtin(&binding.value))
        }
        Expr::Literal(_, _)
        | Expr::Path(_)
        | Expr::Atom { .. }
        | Expr::Formula { .. }
        | Expr::Template { .. }
        | Expr::Assertion { .. } => false,
    }
}

fn stmts_contain_regex_call(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        if stmt_contains_regex_call(stmt) {
            return true;
        }
    }
    false
}

fn stmt_contains_regex_call(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::ImportBlock { .. } | Stmt::ExportBlock { .. } => false,
        Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
            item.value.as_ref().is_some_and(expr_contains_regex_call)
                || item.maegim.as_ref().is_some_and(|spec| {
                    spec.fields
                        .iter()
                        .any(|field| expr_contains_regex_call(&field.value))
                })
        }),
        Stmt::SeedDef { body, .. } => stmts_contain_regex_call(body),
        Stmt::Assign { value, .. }
        | Stmt::Expr { value, .. }
        | Stmt::Return { value, .. }
        | Stmt::Show { value, .. }
        | Stmt::Inspect { value, .. } => expr_contains_regex_call(value),
        Stmt::Receive {
            condition, body, ..
        } => {
            condition.as_ref().is_some_and(expr_contains_regex_call)
                || stmts_contain_regex_call(body)
        }
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => {
            sender.as_ref().is_some_and(expr_contains_regex_call)
                || expr_contains_regex_call(payload)
                || expr_contains_regex_call(receiver)
        }
        Stmt::Hook { body, .. }
        | Stmt::BeatBlock { body, .. }
        | Stmt::LifecycleBlock { body, .. }
        | Stmt::OpenBlock { body, .. }
        | Stmt::Repeat { body, .. }
        | Stmt::While { body, .. }
        | Stmt::ForEach { body, .. }
        | Stmt::Quantifier { body, .. } => stmts_contain_regex_call(body),
        Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => expr_contains_regex_call(condition) || stmts_contain_regex_call(body),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expr_contains_regex_call(condition)
                || stmts_contain_regex_call(then_body)
                || else_body
                    .as_ref()
                    .is_some_and(|body| stmts_contain_regex_call(body))
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            branches.iter().any(|branch| {
                expr_contains_regex_call(&branch.condition)
                    || stmts_contain_regex_call(&branch.body)
            }) || else_body
                .as_ref()
                .is_some_and(|body| stmts_contain_regex_call(body))
        }
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expr_contains_regex_call(condition)
                || then_body
                    .as_ref()
                    .is_some_and(|body| stmts_contain_regex_call(body))
                || stmts_contain_regex_call(else_body)
        }
        Stmt::Pragma { .. } | Stmt::BogaeDraw { .. } | Stmt::Break { .. } => false,
    }
}

fn expr_contains_regex_call(expr: &Expr) -> bool {
    match expr {
        Expr::Call { name, args, .. } => {
            let canon = ddonirang_lang::stdlib::canonicalize_stdlib_alias(name);
            let is_regex = matches!(
                canon,
                "정규식"
                    | "정규맞추기"
                    | "정규찾기"
                    | "정규캡처하기"
                    | "정규이름캡처하기"
                    | "정규바꾸기"
                    | "정규나누기"
            );
            is_regex || args.iter().any(|arg| expr_contains_regex_call(&arg.expr))
        }
        Expr::Unary { expr, .. } => expr_contains_regex_call(expr),
        Expr::Binary { left, right, .. } => {
            expr_contains_regex_call(left) || expr_contains_regex_call(right)
        }
        Expr::FieldAccess { target, .. } => expr_contains_regex_call(target),
        Expr::SeedLiteral { body, .. } => expr_contains_regex_call(body),
        Expr::TemplateFill {
            template, bindings, ..
        } => {
            expr_contains_regex_call(template)
                || bindings
                    .iter()
                    .any(|binding| expr_contains_regex_call(&binding.value))
        }
        Expr::Pack { bindings, .. } | Expr::FormulaEval { bindings, .. } => bindings
            .iter()
            .any(|binding| expr_contains_regex_call(&binding.value)),
        Expr::FormulaFill {
            formula, bindings, ..
        } => {
            expr_contains_regex_call(formula)
                || bindings
                    .iter()
                    .any(|binding| expr_contains_regex_call(&binding.value))
        }
        Expr::Literal(_, _)
        | Expr::Path(_)
        | Expr::Atom { .. }
        | Expr::Formula { .. }
        | Expr::Template { .. }
        | Expr::Assertion { .. } => false,
    }
}

fn load_open_policy(input_path: &Path) -> Result<Option<OpenPolicy>, String> {
    let policy_path = find_open_policy_path(input_path)?;
    let Some(policy_path) = policy_path else {
        return Ok(None);
    };
    let text = fs::read_to_string(&policy_path)
        .map_err(|e| format!("open.policy 읽기 실패: {} ({})", policy_path.display(), e))?;
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
        return Err(format!(
            "open.policy 파일이 여러 개입니다: {}",
            list.join(", ")
        ));
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
    let value: JsonValue = serde_json::from_str(&text).map_err(|e| {
        format!(
            "ddn.project.json JSON 파싱 실패: {} ({})",
            path.display(),
            e
        )
    })?;
    let obj = value
        .as_object()
        .ok_or_else(|| format!("ddn.project.json은 객체여야 합니다: {}", path.display()))?;
    let age_text = obj.get("age_target").and_then(|value| value.as_str());
    let age_target = if let Some(text) = age_text {
        Some(AgeTarget::parse(text).ok_or_else(|| {
            format!(
                "E_PROJECT_AGE_TARGET_INVALID ddn.project.json age_target 오류: {}",
                text
            )
        })?)
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
        Some(
            parse_lang_mode(text)
                .ok_or_else(|| format!("ddn.project.json lang_mode 오류: {}", text))?,
        )
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
    let obj = value
        .as_object()
        .ok_or_else(|| format!("open.policy JSON은 객체여야 합니다: {}", path.display()))?;
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
    let default_mode = parse_open_mode_value(&default_value)
        .map_err(|message| format!("open.policy default 오류: {} ({})", message, path.display()))?;
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
            return Err(format!(
                "open.policy {} 알 수 없는 open_kind: {}",
                label, item
            ));
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
    let mut open_allow = parse_open_allow_directives(&source);
    let open_policy =
        load_open_policy(path).map_err(|message| format!("E_OPEN_POLICY {}", message))?;
    let project_policy = load_project_policy(path)?;
    let parse_mode = resolve_lang_mode(options.lang_mode, &project_policy)?;
    let (program_for_gate, _prepared_source) =
        parse_program_for_runtime_with_mode(&source, parse_mode).map_err(|err| match err {
            FrontdoorParseFailure::Guard(message) => message,
            FrontdoorParseFailure::Lex(err) => RunError::Lex(err).format(&file_label),
            FrontdoorParseFailure::Parse(err) => RunError::Parse(err).format(&file_label),
        })?;
    let exec_policy_extract = extract_exec_policy(&program_for_gate)?;
    for kind in extract_exec_policy_open_allow(&program_for_gate) {
        if !open_allow.iter().any(|entry| entry == &kind) {
            open_allow.push(kind);
        }
    }
    if let Some(extracted) = exec_policy_extract {
        if extracted.strict_effect_ignored {
            emit.err(
                "warning: W_EFFECT_POLICY_IGNORED_IN_STRICT 엄밀 모드에서는 효과정책 값이 무시되고 격리(isolated)로 고정됩니다.",
            );
        }
    }
    let exec_policy = exec_policy_extract.map(|it| it.policy);
    let seulgi_hook_policy = exec_policy
        .map(|policy| policy.seulgi_hook)
        .unwrap_or(SeulgiHookPolicy::Record);
    let open_mode = resolve_effective_open_mode(options.open_mode, open_policy.as_ref());
    let uses_open_block = program_contains_open_block(&program_for_gate);
    let uses_input_surface = program_contains_input_builtin(&program_for_gate);
    if let Some(policy) = exec_policy {
        let uses_effect = uses_open_block
            || program_contains_open_call(&program_for_gate)
            || (uses_input_surface && open_mode != OpenMode::Deny);
        if policy.mode == ExecMode::Strict && uses_effect {
            return Err(
                "E_EFFECT_IN_STRICT_MODE 엄밀 모드에서는 너머 호출을 사용할 수 없습니다."
                    .to_string(),
            );
        }
        if policy.mode == ExecMode::General
            && policy.effect == EffectPolicy::Isolated
            && uses_effect
        {
            return Err(
                "E_EFFECT_IN_ISOLATED_MODE 격리 효과정책에서는 너머 호출을 사용할 수 없습니다."
                    .to_string(),
            );
        }
    }
    let age_decision = resolve_age_target(options.age_target.as_deref(), &project_policy)?;
    let age_target = age_decision.value;
    let age_target_source = age_decision.source;
    if program_contains_regex_call(&program_for_gate) && age_target < AgeTarget::Age3 {
        return Err(err_age_not_available("regex", age_target, AgeTarget::Age3));
    }
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
    if seulgi_hook_policy == SeulgiHookPolicy::Execute {
        if age_target < AgeTarget::Age2 {
            return Err(err_age_not_available(
                "seulgi_hook",
                age_target,
                AgeTarget::Age2,
            ));
        }
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
    enforce_det_tier_contract(
        det_tier,
        age_target,
        project_policy.detmath_seal_hash.as_deref(),
        project_policy.nuri_lock_hash.as_deref(),
    )?;
    let effective_trace_tier = if open_diag_enabled {
        let trace_tier_project = project_policy.trace_tier.ok_or_else(|| {
            "E_TRACE_TIER_REQUIRED AGE2 이상에서는 ddn.project.json trace_tier가 필요합니다"
                .to_string()
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
        return Err(
            "E_OPEN_BUNDLE_CONFLICT --open-bundle과 --open-log는 함께 사용할 수 없습니다."
                .to_string(),
        );
    }
    if open_mode == OpenMode::Deny && options.open_bundle.is_some() {
        return Err(
            "E_OPEN_BUNDLE_MODE --open-bundle은 open 모드(record/replay)가 필요합니다.".to_string(),
        );
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
        if let Err(write_err) = append_run_config_diag(
            diag_path,
            age_target_source,
            age_target,
            options.latency_madi,
        ) {
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
        let diag_path = diag_jsonl
            .as_ref()
            .ok_or_else(|| "E_OPEN_DIAG_REQUIRED geoul.diag.jsonl 경로가 필요합니다".to_string())?;
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
        return Err(
            "E_SAM_MODE_CONFLICT --sam-live와 --sam은 동시에 사용할 수 없습니다.".to_string(),
        );
    }
    let madi_arg = madi;
    let explicit_infinite = matches!(madi_arg, Some(MadiLimit::Infinite));
    if explicit_infinite && options.sam_path.is_some() {
        return Err(
            "E_SAM_MADI_INFINITE --sam과 --madi infinite는 함께 쓸 수 없습니다.".to_string(),
        );
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
        return Err(
            "E_BOGAE_PLAYBACK_INFINITE 무한 실행에서는 playback을 만들 수 없습니다.".to_string(),
        );
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
        if let (Some(BogaeMode::Web), Some(index_path)) =
            (options.bogae_mode, web_index_path.as_ref())
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
        writer.set_age_target(age_target_source.label(), age_target.label());
        writer.set_seulgi_latency_madi(options.latency_madi);
        writer.set_seulgi_latency_drop_policy(latency_drop_policy_label());
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
    let input_open_site = input_open_site_id(&open_source);
    let run_result = run_source_with_state_ticks_observe(
        &source,
        parse_mode,
        initial_state,
        ticks,
        seed,
        options.latency_madi,
        open_runtime,
        &open_source,
        open_mode,
        uses_input_surface,
        &input_open_site,
        wants_playback,
        wants_playback || wants_live || wants_geoul || wants_geoul_record,
        force_bogae,
        &mut tick_snapshots,
        sam_plan.as_mut(),
        live_input.as_mut(),
        diag_jsonl.as_deref(),
        &file_label,
        stop_enabled,
        &mut should_stop,
        |madi, state, tick_requested| {
            let wants_geoul_bytes =
                geoul_writer.borrow().is_some() || geoul_record_writer.borrow().is_some();
            let mut state_bytes: Option<Vec<u8>> = None;
            if wants_geoul_bytes {
                state_bytes = Some(encode_state_for_geoul(state));
            }
            if let Some(writer) = geoul_writer.borrow_mut().as_mut() {
                let snapshot = build_geoul_snapshot(madi, seed, state);
                let snapshot_bytes = encode_input_snapshot(&snapshot);
                let state_bytes = state_bytes.as_ref().ok_or_else(|| RunError::Io {
                    path: writer.audit_path().to_path_buf(),
                    message: "state bytes 누락".to_string(),
                })?;
                let mut patch_buf = Vec::new();
                let mut alrim_buf = Vec::new();
                let mut full_blob = None;
                if matches!(
                    trace_tier,
                    TraceTier::Patch | TraceTier::Alrim | TraceTier::Full
                ) {
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
                let state_bytes = state_bytes.as_ref().ok_or_else(|| RunError::Io {
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
        Err(failure) => {
            if let (Some(path), Some(output)) =
                (options.proof_out.as_ref(), failure.output.as_ref())
            {
                if let Err(write_err) = write_failed_proof_detjson(
                    path,
                    &file_label,
                    seed,
                    failure.ticks,
                    &source,
                    parse_mode,
                    &failure.error,
                    output,
                    options.proof_cert_key.as_deref(),
                ) {
                    emit.err(&format!("E_PROOF_WRITE {}", write_err));
                }
            }
            if let Some(diag_path) = diag_jsonl.as_ref() {
                if let Err(write_err) =
                    write_diag_jsonl(diag_path, &file_label, &failure.error, diag_append)
                {
                    emit.err(&format!("E_DIAG_WRITE {}", write_err));
                }
            }
            if let Some(repro_path) = options.repro_json.as_ref() {
                if let Err(write_err) = write_repro_json(
                    repro_path,
                    &file_label,
                    &failure.error,
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
            return Err(failure.error.format(&file_label));
        }
    };
    if let Some(finish_error) = finish_error {
        return Err(finish_error);
    }
    if sam_used {
        clear_sam_keys(&mut output.state);
    }
    if let Some(diag_path) = diag_jsonl.as_ref() {
        if let Err(write_err) = append_contract_diags(
            diag_path,
            &file_label,
            &source,
            &output.contract_diags,
            seulgi_hook_policy,
        ) {
            emit.err(&format!("E_DIAG_WRITE {}", write_err));
        }
    }
    if let Some(report_path) = options.diag_report_out.as_ref() {
        if let Err(write_err) =
            write_diagnostic_report(report_path, seed, ticks_run, &output.diagnostics)
        {
            let run_err = RunError::Io {
                path: report_path.clone(),
                message: format!("E_DIAG_REPORT_WRITE {}", write_err),
            };
            if let Some(diag_path) = diag_jsonl.as_ref() {
                if let Err(diag_write_err) =
                    write_diag_jsonl(diag_path, &file_label, &run_err, diag_append)
                {
                    emit.err(&format!("E_DIAG_WRITE {}", diag_write_err));
                }
            }
            if let Some(repro_path) = options.repro_json.as_ref() {
                if let Err(repro_write_err) = write_repro_json(
                    repro_path,
                    &file_label,
                    &run_err,
                    seed,
                    madi_arg,
                    options.until_gameover,
                    &options.gameover_key,
                ) {
                    emit.err(&format!("E_REPRO_WRITE {}", repro_write_err));
                }
            }
            return Err(run_err.format(&file_label));
        }
    }
    let diagnostic_failure = output.diagnostic_failures.first().cloned();

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
    if let Some(failure) = diagnostic_failure {
        let run_err = RunError::Runtime(diagnostic_failure_to_runtime_error(&failure));
        if let Some(path) = options.proof_out.as_ref() {
            if let Err(write_err) = write_failed_proof_detjson(
                path,
                &file_label,
                seed,
                ticks_run,
                &source,
                parse_mode,
                &run_err,
                &output,
                options.proof_cert_key.as_deref(),
            ) {
                emit.err(&format!("E_PROOF_WRITE {}", write_err));
            }
        }
        if let Some(diag_path) = diag_jsonl.as_ref() {
            if let Err(write_err) = write_diag_jsonl(diag_path, &file_label, &run_err, diag_append)
            {
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

    let mut state_hash = hash::state_hash(&output.state);
    if let Some(override_hash) = maybe_override_state_hash(&output.state) {
        state_hash = override_hash;
    }
    let trace_hash = hash::trace_hash(&output.trace, &source, &state_hash, ticks_run, seed);

    let mut bogae_output = None;
    if output.bogae_requested || options.bogae_out.is_some() || options.bogae_mode.is_some() {
        let trace_lines = output.trace.log_lines();
        let (built, policy_event) = match build_bogae_output_with_trace(
            &output.state,
            &trace_lines,
            pack.as_ref(),
            options.cmd_policy,
            bogae_codec,
        ) {
            Ok(value) => value,
            Err(err) => {
                let run_err = RunError::Bogae(err);
                if let Some(diag_path) = diag_jsonl.as_ref() {
                    if let Err(write_err) =
                        write_diag_jsonl(diag_path, &file_label, &run_err, diag_append)
                    {
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
                    if let Err(write_err) =
                        write_diag_jsonl(diag_path, &file_label, &err, diag_append)
                    {
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
    if let Some(path) = options.proof_out.as_ref() {
        write_proof_detjson(
            path,
            &file_label,
            seed,
            ticks_run,
            &source,
            parse_mode,
            &state_hash,
            &trace_hash,
            &output,
            options.proof_cert_key.as_deref(),
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
    let (program, prepared_source) = parse_program_for_runtime(source).map_err(|err| match err {
        FrontdoorParseFailure::Guard(message) => RunError::Frontdoor { message },
        FrontdoorParseFailure::Lex(err) => RunError::Lex(err),
        FrontdoorParseFailure::Parse(err) => RunError::Parse(err),
    })?;
    let evaluator = Evaluator::with_state_seed_open(
        state,
        seed,
        OpenRuntime::deny(),
        "<memory>".to_string(),
        Some(prepared_source),
    );
    evaluator
        .run_with_ticks(&program, ticks)
        .map_err(RunError::Runtime)
}

fn run_source_with_state_ticks_observe<F, G>(
    source: &str,
    parse_mode: ParseMode,
    state: State,
    ticks: u64,
    seed: u64,
    latency_madi: u64,
    open_runtime: OpenRuntime,
    open_source: &str,
    open_mode: OpenMode,
    uses_input_surface: bool,
    input_open_site: &str,
    wants_snapshots: bool,
    observe_ticks: bool,
    force_bogae: bool,
    snapshots: &mut Vec<TickSnapshot>,
    sam_plan: Option<&mut SamPlan>,
    live_input: Option<&mut LiveInput>,
    latency_diag_path: Option<&Path>,
    latency_diag_file: &str,
    stop_enabled: bool,
    mut should_stop: G,
    mut on_tick_extra: F,
) -> Result<RunOutcome, FailedRunOutcome>
where
    F: FnMut(u64, &State, bool) -> Result<(), RunError>,
    G: FnMut(u64, &State) -> bool,
{
    let mut tick_error: Option<RunError> = None;
    let mut ticks_run = 0u64;
    let (program, prepared_source) =
        parse_program_for_runtime_with_mode(source, parse_mode).map_err(|error| {
            let error = match error {
                FrontdoorParseFailure::Guard(message) => RunError::Frontdoor { message },
                FrontdoorParseFailure::Lex(error) => RunError::Lex(error),
                FrontdoorParseFailure::Parse(error) => RunError::Parse(error),
            };
            FailedRunOutcome {
                error,
                output: None,
                ticks: ticks_run,
            }
        })?;
    let evaluator = Evaluator::with_state_seed_open(
        state,
        seed,
        open_runtime,
        open_source.to_string(),
        Some(prepared_source),
    );
    let input_open_active = uses_input_surface
        && open_mode != OpenMode::Deny
        && (sam_plan.is_some() || live_input.is_some() || open_mode == OpenMode::Replay);
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
        let mut latency_queue = InputLatencyQueue::new(latency_madi);
        let mut before_tick =
            |madi: u64, state: &mut State, open: &mut OpenRuntime| -> Result<(), RuntimeError> {
                let raw_frame = sam_plan.frame_for_tick(madi);
                let frame = if input_open_active {
                    open.open_input(input_open_site, madi, Some(raw_frame), input_open_span())?
                } else {
                    raw_frame
                };
                let (delayed, applied) = latency_queue.schedule_and_take(madi, frame);
                if let (Some(path), Some(applied)) = (latency_diag_path, applied) {
                    let _ = append_latency_schedule_diag(path, latency_diag_file, madi, applied);
                }
                sam_plan.apply_frame(state, madi, delayed);
                Ok(())
            };
        if stop_enabled {
            evaluator
                .run_with_ticks_observe_and_inject_stop_open_capture_failure(
                    &program,
                    ticks,
                    &mut before_tick,
                    &mut on_tick,
                    &mut should_stop,
                )
                .map_err(|failure| failed_eval_run_outcome(failure, ticks_run))?
        } else {
            evaluator
                .run_with_ticks_observe_and_inject_open_capture_failure(
                    &program,
                    ticks,
                    &mut before_tick,
                    &mut on_tick,
                )
                .map_err(|failure| failed_eval_run_outcome(failure, ticks_run))?
        }
    } else if let Some(live_input) = live_input {
        let mut latency_queue = InputLatencyQueue::new(latency_madi);
        let mut before_tick =
            |madi: u64, state: &mut State, open: &mut OpenRuntime| -> Result<(), RuntimeError> {
                let tick = live_input.sample_tick(madi);
                let raw_frame = OpenInputFrame::new(tick.held, tick.pressed, tick.released);
                let frame = if input_open_active {
                    open.open_input(input_open_site, madi, Some(raw_frame), input_open_span())?
                } else {
                    raw_frame
                };
                let (delayed, applied) = latency_queue.schedule_and_take(madi, frame);
                if let (Some(path), Some(applied)) = (latency_diag_path, applied) {
                    let _ = append_latency_schedule_diag(path, latency_diag_file, madi, applied);
                }
                apply_input_frame(state, madi, delayed);
                Ok(())
            };
        evaluator
            .run_with_ticks_observe_and_inject_stop_open_capture_failure(
                &program,
                ticks,
                &mut before_tick,
                &mut on_tick,
                &mut should_stop,
            )
            .map_err(|failure| failed_eval_run_outcome(failure, ticks_run))?
    } else if input_open_active {
        let mut latency_queue = InputLatencyQueue::new(latency_madi);
        let mut before_tick =
            |madi: u64, state: &mut State, open: &mut OpenRuntime| -> Result<(), RuntimeError> {
                let frame = open.open_input(input_open_site, madi, None, input_open_span())?;
                let (delayed, applied) = latency_queue.schedule_and_take(madi, frame);
                if let (Some(path), Some(applied)) = (latency_diag_path, applied) {
                    let _ = append_latency_schedule_diag(path, latency_diag_file, madi, applied);
                }
                apply_input_frame(state, madi, delayed);
                Ok(())
            };
        if stop_enabled {
            evaluator
                .run_with_ticks_observe_and_inject_stop_open_capture_failure(
                    &program,
                    ticks,
                    &mut before_tick,
                    &mut on_tick,
                    &mut should_stop,
                )
                .map_err(|failure| failed_eval_run_outcome(failure, ticks_run))?
        } else {
            evaluator
                .run_with_ticks_observe_and_inject_open_capture_failure(
                    &program,
                    ticks,
                    &mut before_tick,
                    &mut on_tick,
                )
                .map_err(|failure| failed_eval_run_outcome(failure, ticks_run))?
        }
    } else if observe_ticks || wants_snapshots || stop_enabled {
        if stop_enabled {
            let mut before_tick =
                |_: u64, _: &mut State, _: &mut OpenRuntime| -> Result<(), RuntimeError> { Ok(()) };
            evaluator
                .run_with_ticks_observe_and_inject_stop_open_capture_failure(
                    &program,
                    ticks,
                    &mut before_tick,
                    &mut on_tick,
                    &mut should_stop,
                )
                .map_err(|failure| failed_eval_run_outcome(failure, ticks_run))?
        } else {
            evaluator
                .run_with_ticks_observe_capture_failure(&program, ticks, &mut on_tick)
                .map_err(|failure| failed_eval_run_outcome(failure, ticks_run))?
        }
    } else {
        let output = evaluator
            .run_with_ticks_capture_failure(&program, ticks)
            .map_err(|failure| failed_eval_run_outcome(failure, ticks_run))?;
        ticks_run = ticks;
        output
    };
    if let Some(err) = tick_error {
        return Err(FailedRunOutcome {
            error: err,
            output: Some(output),
            ticks: ticks_run,
        });
    }
    Ok(RunOutcome {
        output,
        ticks: ticks_run,
    })
}

fn failed_eval_run_outcome(failure: EvalFailure, ticks: u64) -> FailedRunOutcome {
    FailedRunOutcome {
        error: RunError::Runtime(failure.error),
        output: Some(failure.output),
        ticks,
    }
}

pub fn run_source_with_state(source: &str, state: State) -> Result<EvalOutput, RunError> {
    run_source_with_state_ticks(source, state, 1)
}

fn should_write_playback(ticks: u64, mode: Option<BogaeMode>, bogae_out: Option<&Path>) -> bool {
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
    write_manifest(out_dir, start_madi, end_madi, &frames, codec.tag()).map_err(|e| {
        RunError::Io {
            path: out_dir.join("manifest.detjson"),
            message: e,
        }
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
        LexError::UnterminatedAssertion { line, .. } => *line,
        LexError::BadEscape { line, .. } => *line,
        LexError::BadIdentStart { line, .. } => *line,
        LexError::UnexpectedChar { line, .. } => *line,
    }
}

fn lex_col(err: &LexError) -> usize {
    match err {
        LexError::UnterminatedString { col, .. } => *col,
        LexError::UnterminatedTemplate { col, .. } => *col,
        LexError::UnterminatedAssertion { col, .. } => *col,
        LexError::BadEscape { col, .. } => *col,
        LexError::BadIdentStart { col, .. } => *col,
        LexError::UnexpectedChar { col, .. } => *col,
    }
}

fn lex_message(err: &LexError) -> String {
    match err {
        LexError::UnterminatedString { .. } => "문자열이 닫히지 않았습니다".to_string(),
        LexError::UnterminatedTemplate { .. } => "글무늬 블록이 닫히지 않았습니다".to_string(),
        LexError::UnterminatedAssertion { .. } => "세움 블록이 닫히지 않았습니다".to_string(),
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
        ParseError::CompatMaticEntryDisabled { span } => span.start_line,
        ParseError::BlockHeaderColonForbidden { span } => span.start_line,
        ParseError::EventSurfaceAliasForbidden { span } => span.start_line,
        ParseError::EffectSurfaceAliasForbidden { span } => span.start_line,
        ParseError::ImportAliasDuplicate { span } => span.start_line,
        ParseError::ImportAliasReserved { span } => span.start_line,
        ParseError::ImportPathInvalid { span } => span.start_line,
        ParseError::ImportVersionConflict { span } => span.start_line,
        ParseError::ExportBlockDuplicate { span } => span.start_line,
        ParseError::LifecycleNameDuplicate { span, .. } => span.start_line,
        ParseError::ReceiveOutsideImja { span } => span.start_line,
        ParseError::MaegimRequiresGroupedValue { span } => span.start_line,
        ParseError::MaegimStepSplitConflict { span } => span.start_line,
        ParseError::MaegimNestedSectionUnsupported { span, .. } => span.start_line,
        ParseError::MaegimNestedFieldUnsupported { span, .. } => span.start_line,
        ParseError::HookEveryNMadiIntervalInvalid { span } => span.start_line,
        ParseError::HookEveryNMadiUnitUnsupported { span, .. } => span.start_line,
        ParseError::HookEveryNMadiSuffixUnsupported { span, .. } => span.start_line,
        ParseError::DeferredAssignOutsideBeat { span } => span.start_line,
        ParseError::QuantifierMutationForbidden { span } => span.start_line,
        ParseError::QuantifierShowForbidden { span } => span.start_line,
        ParseError::QuantifierIoForbidden { span } => span.start_line,
        ParseError::ImmediateProofMutationForbidden { span } => span.start_line,
        ParseError::ImmediateProofShowForbidden { span } => span.start_line,
        ParseError::ImmediateProofIoForbidden { span } => span.start_line,
        ParseError::CaseCompletionRequired { span } => span.start_line,
        ParseError::CaseElseNotLast { span } => span.start_line,
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
        ParseError::CompatMaticEntryDisabled { span } => span.start_col,
        ParseError::BlockHeaderColonForbidden { span } => span.start_col,
        ParseError::EventSurfaceAliasForbidden { span } => span.start_col,
        ParseError::EffectSurfaceAliasForbidden { span } => span.start_col,
        ParseError::ImportAliasDuplicate { span } => span.start_col,
        ParseError::ImportAliasReserved { span } => span.start_col,
        ParseError::ImportPathInvalid { span } => span.start_col,
        ParseError::ImportVersionConflict { span } => span.start_col,
        ParseError::ExportBlockDuplicate { span } => span.start_col,
        ParseError::LifecycleNameDuplicate { span, .. } => span.start_col,
        ParseError::ReceiveOutsideImja { span } => span.start_col,
        ParseError::MaegimRequiresGroupedValue { span } => span.start_col,
        ParseError::MaegimStepSplitConflict { span } => span.start_col,
        ParseError::MaegimNestedSectionUnsupported { span, .. } => span.start_col,
        ParseError::MaegimNestedFieldUnsupported { span, .. } => span.start_col,
        ParseError::HookEveryNMadiIntervalInvalid { span } => span.start_col,
        ParseError::HookEveryNMadiUnitUnsupported { span, .. } => span.start_col,
        ParseError::HookEveryNMadiSuffixUnsupported { span, .. } => span.start_col,
        ParseError::DeferredAssignOutsideBeat { span } => span.start_col,
        ParseError::QuantifierMutationForbidden { span } => span.start_col,
        ParseError::QuantifierShowForbidden { span } => span.start_col,
        ParseError::QuantifierIoForbidden { span } => span.start_col,
        ParseError::ImmediateProofMutationForbidden { span } => span.start_col,
        ParseError::ImmediateProofShowForbidden { span } => span.start_col,
        ParseError::ImmediateProofIoForbidden { span } => span.start_col,
        ParseError::CaseCompletionRequired { span } => span.start_col,
        ParseError::CaseElseNotLast { span } => span.start_col,
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
        ParseError::CompatMaticEntryDisabled { .. } => {
            "strict 모드에서는 매틱:움직씨가 비활성화됩니다. 정본 표면(매마디:움직씨)으로 전환하세요."
                .to_string()
        }
        ParseError::BlockHeaderColonForbidden { .. } => {
            "블록 머릿말의 ':'는 허용되지 않습니다".to_string()
        }
        ParseError::EventSurfaceAliasForbidden { .. } => {
            "비정본 이벤트 문형입니다. 정본: \"KIND\"라는 알림이 오면 { ... }.".to_string()
        }
        ParseError::EffectSurfaceAliasForbidden { .. } => {
            "비정본 너머 문형입니다. 정본: 너머 { ... }.".to_string()
        }
        ParseError::ImportAliasDuplicate { .. } => "같은 쓰임 별명을 중복 선언했습니다".to_string(),
        ParseError::ImportAliasReserved { .. } => {
            "쓰임 별명이 예약된 키워드와 충돌합니다".to_string()
        }
        ParseError::ImportPathInvalid { .. } => "쓰임 경로 형식이 잘못되었습니다".to_string(),
        ParseError::ImportVersionConflict { .. } => {
            "같은 패키지에 서로 다른 버전을 동시에 선언했습니다".to_string()
        }
        ParseError::ExportBlockDuplicate { .. } => {
            "드러냄(또는 공개) 블록은 파일당 한 번만 허용됩니다".to_string()
        }
        ParseError::LifecycleNameDuplicate {
            name, first_span, ..
        } => {
            format!(
                "lifecycle 이름을 중복 선언했습니다: {} (첫 선언 {}:{})",
                name, first_span.start_line, first_span.start_col
            )
        }
        ParseError::ReceiveOutsideImja { .. } => {
            "`받으면` 훅은 `임자` 본문 안에서만 사용할 수 있습니다".to_string()
        }
        ParseError::MaegimRequiresGroupedValue { .. } => {
            "매김은 `(<식>) 매김 { ... }` 형태만 허용됩니다".to_string()
        }
        ParseError::MaegimStepSplitConflict { .. } => {
            "`매김`에서 `간격`과 `분할수`는 동시에 사용할 수 없습니다".to_string()
        }
        ParseError::MaegimNestedSectionUnsupported { section, .. } => {
            format!(
                "`매김` 중첩 섹션은 `가늠`/`갈래`만 허용됩니다: {}",
                section
            )
        }
        ParseError::MaegimNestedFieldUnsupported { section, field, .. } => {
            format!(
                "`매김` 섹션 `{}`에서 지원하지 않는 항목입니다: {}",
                section, field
            )
        }
        ParseError::HookEveryNMadiIntervalInvalid { .. } => {
            "(N마디)마다에서 N은 양의 정수여야 합니다".to_string()
        }
        ParseError::HookEveryNMadiUnitUnsupported { unit, .. } => {
            format!("(N마디)마다에서 단위는 `마디`만 허용됩니다: {}", unit)
        }
        ParseError::HookEveryNMadiSuffixUnsupported { suffix, .. } => {
            format!("(N마디) 훅 접미는 `마다`만 허용됩니다: {}", suffix)
        }
        ParseError::DeferredAssignOutsideBeat { .. } => {
            "`미루기` 대입은 `덩이 {}` 안에서만 사용할 수 있습니다".to_string()
        }
        ParseError::QuantifierMutationForbidden { .. } => {
            "AGE4: 양화 블록 안에서는 '<-'를 사용할 수 없습니다".to_string()
        }
        ParseError::QuantifierShowForbidden { .. } => {
            "AGE4: 양화 블록 안에서는 '보여주기'를 사용할 수 없습니다".to_string()
        }
        ParseError::QuantifierIoForbidden { .. } => {
            "AGE4: 양화 블록 안에서는 외부 I/O를 사용할 수 없습니다".to_string()
        }
        ParseError::ImmediateProofMutationForbidden { .. } => {
            "AGE1: 밝히기 본문 안에서는 '<-'를 사용할 수 없습니다".to_string()
        }
        ParseError::ImmediateProofShowForbidden { .. } => {
            "AGE1: 밝히기 본문 안에서는 '보여주기'를 사용할 수 없습니다".to_string()
        }
        ParseError::ImmediateProofIoForbidden { .. } => {
            "AGE1: 밝히기 본문 안에서는 solver hook 외 외부 I/O를 사용할 수 없습니다".to_string()
        }
        ParseError::CaseCompletionRequired { .. } => {
            "`인 경우`가 있으면 `그밖의 경우 { ... }` 또는 `모든 경우 다룸.`이 필요합니다"
                .to_string()
        }
        ParseError::CaseElseNotLast { .. } => "`그밖의 경우`는 마지막 분기여야 합니다".to_string(),
    }
}

fn runtime_line(err: &RuntimeError) -> usize {
    match err {
        RuntimeError::Undefined { span, .. } => span.start_line,
        RuntimeError::InvalidPath { span, .. } => span.start_line,
        RuntimeError::JeOutsideImja { span } => span.start_line,
        RuntimeError::MathDivZero { span } => span.start_line,
        RuntimeError::MathDomain { span, .. } => span.start_line,
        RuntimeError::TypeMismatch { span, .. } => span.start_line,
        RuntimeError::TypeMismatchDetail { span, .. } => span.start_line,
        RuntimeError::LifecycleTargetUnknown { span, .. } => span.start_line,
        RuntimeError::LifecycleTargetArity { span, .. } => span.start_line,
        RuntimeError::LifecycleTargetFamilyConflict { span, .. } => span.start_line,
        RuntimeError::LifecycleTargetFamilyAmbiguous { span, .. } => span.start_line,
        RuntimeError::StringIndexOutOfRange { span } => span.start_line,
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
        RuntimeError::ProofIncomplete { span } => span.start_line,
        RuntimeError::OpenSiteUnknown { span } => span.start_line,
        RuntimeError::OpenDenied { span, .. } => span.start_line,
        RuntimeError::OpenReplayMissing { span, .. } => span.start_line,
        RuntimeError::OpenReplayInvalid { span, .. } => span.start_line,
        RuntimeError::OpenLogTamper { span, .. } => span.start_line,
        RuntimeError::OpenIo { span, .. } => span.start_line,
        RuntimeError::RegexFlagsInvalid { span, .. } => span.start_line,
        RuntimeError::RegexPatternInvalid { span, .. } => span.start_line,
        RuntimeError::RegexReplacementInvalid { span, .. } => span.start_line,
        RuntimeError::InputKeyMissing { span } => span.start_line,
        RuntimeError::MapDotKeyMissing { span, .. } => span.start_line,
        RuntimeError::EcoDivergenceDetected { span, .. } => span.start_line,
        RuntimeError::SfcIdentityViolation { span, .. } => span.start_line,
    }
}

fn runtime_col(err: &RuntimeError) -> usize {
    match err {
        RuntimeError::Undefined { span, .. } => span.start_col,
        RuntimeError::InvalidPath { span, .. } => span.start_col,
        RuntimeError::JeOutsideImja { span } => span.start_col,
        RuntimeError::MathDivZero { span } => span.start_col,
        RuntimeError::MathDomain { span, .. } => span.start_col,
        RuntimeError::TypeMismatch { span, .. } => span.start_col,
        RuntimeError::TypeMismatchDetail { span, .. } => span.start_col,
        RuntimeError::LifecycleTargetUnknown { span, .. } => span.start_col,
        RuntimeError::LifecycleTargetArity { span, .. } => span.start_col,
        RuntimeError::LifecycleTargetFamilyConflict { span, .. } => span.start_col,
        RuntimeError::LifecycleTargetFamilyAmbiguous { span, .. } => span.start_col,
        RuntimeError::StringIndexOutOfRange { span } => span.start_col,
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
        RuntimeError::ProofIncomplete { span } => span.start_col,
        RuntimeError::OpenSiteUnknown { span } => span.start_col,
        RuntimeError::OpenDenied { span, .. } => span.start_col,
        RuntimeError::OpenReplayMissing { span, .. } => span.start_col,
        RuntimeError::OpenReplayInvalid { span, .. } => span.start_col,
        RuntimeError::OpenLogTamper { span, .. } => span.start_col,
        RuntimeError::OpenIo { span, .. } => span.start_col,
        RuntimeError::RegexFlagsInvalid { span, .. } => span.start_col,
        RuntimeError::RegexPatternInvalid { span, .. } => span.start_col,
        RuntimeError::RegexReplacementInvalid { span, .. } => span.start_col,
        RuntimeError::InputKeyMissing { span } => span.start_col,
        RuntimeError::MapDotKeyMissing { span, .. } => span.start_col,
        RuntimeError::EcoDivergenceDetected { span, .. } => span.start_col,
        RuntimeError::SfcIdentityViolation { span, .. } => span.start_col,
    }
}

fn runtime_message(err: &RuntimeError) -> String {
    match err {
        RuntimeError::Undefined { path, .. } => format!("정의되지 않은 경로: {}", path),
        RuntimeError::InvalidPath { path, .. } => format!("잘못된 경로: {}", path),
        RuntimeError::JeOutsideImja { .. } => {
            "`제`는 임자 본문 안에서만 사용할 수 있습니다".to_string()
        }
        RuntimeError::MathDivZero { .. } => "0으로 나눌 수 없습니다 (E_NUM_DIV0)".to_string(),
        RuntimeError::MathDomain { message, .. } => message.to_string(),
        RuntimeError::TypeMismatch { expected, .. } => {
            format!("타입 불일치: {}", expected)
        }
        RuntimeError::TypeMismatchDetail {
            expected, actual, ..
        } => {
            format!("타입 불일치: 기대={} 실제={}", expected, actual)
        }
        RuntimeError::LifecycleTargetUnknown {
            verb,
            target,
            family,
            ..
        } => match *verb {
            "시작하기" => {
                format!("등록되지 않은 시작하기 target: {} (family={})", target, family)
            }
            "넘어가기" => {
                format!("등록되지 않은 넘어가기 target: {} (family={})", target, family)
            }
            "불러오기" => {
                format!("등록되지 않은 불러오기 target: {} (family={})", target, family)
            }
            _ => format!(
                "등록되지 않은 lifecycle target: {} (동사={}, family={})",
                target, verb, family
            ),
        },
        RuntimeError::LifecycleTargetArity { verb, got, .. } => match *verb {
            "시작하기" => {
                format!("시작하기 target 인자 개수 오류: 기대=0또는1 실제={}", got)
            }
            "넘어가기" => {
                format!("넘어가기 target 인자 개수 오류: 기대=0또는1 실제={}", got)
            }
            "불러오기" => {
                format!("불러오기 target 인자 개수 오류: 기대=0또는1 실제={}", got)
            }
            _ => format!(
                "lifecycle target 인자 개수 오류: 동사={} 기대=0또는1 실제={}",
                verb, got
            ),
        },
        RuntimeError::LifecycleTargetFamilyConflict {
            verb,
            target,
            hint_family,
            declared_family,
            ..
        } => match *verb {
            "시작하기" => format!(
                "시작하기 target family 충돌: {} (힌트={} 선언={})",
                target, hint_family, declared_family
            ),
            "넘어가기" => format!(
                "넘어가기 target family 충돌: {} (힌트={} 선언={})",
                target, hint_family, declared_family
            ),
            "불러오기" => format!(
                "불러오기 target family 충돌: {} (힌트={} 선언={})",
                target, hint_family, declared_family
            ),
            _ => format!(
                "lifecycle target family 충돌: {} (동사={} 힌트={} 선언={})",
                target, verb, hint_family, declared_family
            ),
        },
        RuntimeError::LifecycleTargetFamilyAmbiguous { verb, target, .. } => match *verb {
            "시작하기" => format!("시작하기 target family 애매: {} (판/마당 동시 포함)", target),
            "넘어가기" => format!("넘어가기 target family 애매: {} (판/마당 동시 포함)", target),
            "불러오기" => format!("불러오기 target family 애매: {} (판/마당 동시 포함)", target),
            _ => format!(
                "lifecycle target family 애매: {} (동사={} 판/마당 동시 포함)",
                target, verb
            ),
        },
        RuntimeError::StringIndexOutOfRange { .. } => "글 인덱스 범위를 벗어났습니다".to_string(),
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
        RuntimeError::BreakOutsideLoop { .. } => {
            "멈추기는 반복 안에서만 사용할 수 있습니다".to_string()
        }
        RuntimeError::ReturnOutsideSeed { .. } => {
            "돌려주기는 씨앗 안에서만 사용할 수 있습니다".to_string()
        }
        RuntimeError::ProofIncomplete { .. } => "모든 경우를 다루지 못했습니다".to_string(),
        RuntimeError::OpenSiteUnknown { .. } => "열림 위치를 알 수 없습니다".to_string(),
        RuntimeError::OpenDenied { open_kind, .. } => {
            format!("열림이 차단되었습니다: {}", open_kind)
        }
        RuntimeError::OpenReplayMissing {
            open_kind,
            site_id,
            key,
            ..
        } => {
            format!(
                "열림 리플레이 로그 없음: kind={} site_id={} key={}",
                open_kind, site_id, key
            )
        }
        RuntimeError::OpenReplayInvalid { message, .. } => {
            format!("열림 로그 파싱 오류: {}", message)
        }
        RuntimeError::OpenLogTamper { message, .. } => format!("열림 로그 변조: {}", message),
        RuntimeError::OpenIo { message, .. } => message.clone(),
        RuntimeError::RegexFlagsInvalid { flags, .. } => {
            format!("정규식 깃발이 유효하지 않습니다: {}", flags)
        }
        RuntimeError::RegexPatternInvalid { message, .. } => {
            format!("정규식 패턴이 유효하지 않습니다: {}", message)
        }
        RuntimeError::RegexReplacementInvalid {
            replacement,
            message,
            ..
        } => {
            format!(
                "정규식 치환값이 유효하지 않습니다: {} ({})",
                message, replacement
            )
        }
        RuntimeError::InputKeyMissing { .. } => {
            "입력키!를 평가했지만 현재 입력키가 없습니다".to_string()
        }
        RuntimeError::MapDotKeyMissing { key, .. } => {
            format!("짝맞춤 점접근 키가 없습니다: {}", key)
        }
        RuntimeError::EcoDivergenceDetected {
            tick,
            name,
            delta,
            threshold,
            ..
        } => format!(
            "경제 진단 발산: {} (tick={}, delta={} > threshold={})",
            name, tick, delta, threshold
        ),
        RuntimeError::SfcIdentityViolation {
            tick,
            name,
            delta,
            threshold,
            ..
        } => format!(
            "SFC 항등식 위반: {} (tick={}, delta={} > threshold={})",
            name, tick, delta, threshold
        ),
    }
}

fn build_proof_runtime_error(err: &RunError) -> Option<JsonValue> {
    match err {
        RunError::Runtime(runtime) => Some(json!({
            "code": runtime.code(),
            "message": runtime_message(runtime),
            "line": runtime_line(runtime),
            "col": runtime_col(runtime),
        })),
        _ => None,
    }
}

fn write_diag_jsonl(path: &Path, file: &str, err: &RunError, append: bool) -> Result<(), String> {
    let (line, col, message) = match err {
        RunError::Frontdoor { message } => (1, 1, message.clone()),
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
        kind, file, event.span.start_line, event.span.start_col, index
    )
}

fn append_contract_hook_event(
    file_handle: &mut std::fs::File,
    fault_id: &str,
    file: &str,
    code: &str,
    event: &ContractDiag,
    hook_policy: SeulgiHookPolicy,
) -> Result<(), String> {
    let contract_kind = match event.kind {
        ContractKind::Pre => "pre",
        ContractKind::Post => "post",
    };
    let mode_label = match event.mode {
        ContractMode::Alert => "알림",
        ContractMode::Abort => "물림",
    };
    let json = format!(
        "{{\"event_kind\":\"hook\",\"hook_name\":\"슬기.계약위반\",\"hook_policy\":\"{}\",\"hook_input_ref\":\"{}\",\"hook_input\":{{\"code\":\"{}\",\"file\":\"{}\",\"line\":{},\"col\":{},\"message\":\"{}\",\"fault_id\":\"{}\",\"contract_kind\":\"{}\",\"mode\":\"{}\"}}}}\n",
        seulgi_hook_policy_label(hook_policy),
        escape_json(fault_id),
        escape_json(code),
        escape_json(file),
        event.span.start_line,
        event.span.start_col,
        escape_json(&event.message),
        escape_json(fault_id),
        contract_kind,
        mode_label
    );
    file_handle
        .write_all(json.as_bytes())
        .map_err(|e| e.to_string())
}

fn build_contract_hook_suggestions(event: &ContractDiag) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    match event.kind {
        ContractKind::Pre => out.push((
            "guard",
            "전제 조건이 참이 되도록 입력값과 초기 상태를 먼저 점검하세요.".to_string(),
        )),
        ContractKind::Post => out.push((
            "post_state",
            "보장 조건이 참이 되도록 본문 이후 상태와 되돌림 값을 점검하세요.".to_string(),
        )),
    }
    match event.mode {
        ContractMode::Alert => out.push((
            "mode",
            "알림 모드 계약이므로 아니면 블록의 안내 메시지와 복구 경로를 우선 정리하세요."
                .to_string(),
        )),
        ContractMode::Abort => out.push((
            "mode",
            "물림 모드 계약이므로 실패 원인을 먼저 줄여 실행 물림을 막으세요.".to_string(),
        )),
    }
    out
}

fn parse_contract_const_bool(token: &str) -> Option<bool> {
    match token.trim() {
        "참" => Some(true),
        "거짓" => Some(false),
        value if value.eq_ignore_ascii_case("true") => Some(true),
        value if value.eq_ignore_ascii_case("false") => Some(false),
        _ => None,
    }
}

fn parse_contract_const_int(token: &str) -> Option<i64> {
    token.trim().parse::<i64>().ok()
}

fn parse_observed_const_int_expr(token: &str) -> Option<i64> {
    let token = strip_wrapping_parens(token);
    parse_contract_const_int(token).or_else(|| ConstIntExprParser::new(token).parse())
}

struct ConstIntExprParser<'a> {
    text: &'a str,
    pos: usize,
}

impl<'a> ConstIntExprParser<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, pos: 0 }
    }

    fn parse(mut self) -> Option<i64> {
        let value = self.parse_add_sub()?;
        self.skip_ws();
        if self.pos == self.text.len() {
            Some(value)
        } else {
            None
        }
    }

    fn parse_add_sub(&mut self) -> Option<i64> {
        let mut value = self.parse_mul_div()?;
        loop {
            self.skip_ws();
            match self.peek_char() {
                Some('+') => {
                    self.bump_char();
                    let rhs = self.parse_mul_div()?;
                    value = value.checked_add(rhs)?;
                }
                Some('-') => {
                    self.bump_char();
                    let rhs = self.parse_mul_div()?;
                    value = value.checked_sub(rhs)?;
                }
                _ => break,
            }
        }
        Some(value)
    }

    fn parse_mul_div(&mut self) -> Option<i64> {
        let mut value = self.parse_unary()?;
        loop {
            self.skip_ws();
            match self.peek_char() {
                Some('*') => {
                    self.bump_char();
                    let rhs = self.parse_unary()?;
                    value = value.checked_mul(rhs)?;
                }
                Some('/') => {
                    self.bump_char();
                    let rhs = self.parse_unary()?;
                    if rhs == 0 {
                        return None;
                    }
                    value = value.checked_div(rhs)?;
                }
                Some('%') => {
                    self.bump_char();
                    let rhs = self.parse_unary()?;
                    if rhs == 0 {
                        return None;
                    }
                    value = value.checked_rem(rhs)?;
                }
                _ => break,
            }
        }
        Some(value)
    }

    fn parse_unary(&mut self) -> Option<i64> {
        self.skip_ws();
        match self.peek_char() {
            Some('+') => {
                self.bump_char();
                self.parse_unary()
            }
            Some('-') => {
                self.bump_char();
                self.parse_unary()?.checked_neg()
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Option<i64> {
        self.skip_ws();
        if self.peek_char() == Some('(') {
            self.bump_char();
            let value = self.parse_add_sub()?;
            self.skip_ws();
            if self.peek_char() != Some(')') {
                return None;
            }
            self.bump_char();
            return Some(value);
        }
        self.parse_int_literal()
    }

    fn parse_int_literal(&mut self) -> Option<i64> {
        self.skip_ws();
        let start = self.pos;
        while matches!(self.peek_char(), Some(ch) if ch.is_ascii_digit()) {
            self.bump_char();
        }
        if self.pos == start {
            return None;
        }
        self.text[start..self.pos].parse::<i64>().ok()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek_char(), Some(ch) if ch.is_ascii_whitespace()) {
            self.bump_char();
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.text.get(self.pos..)?.chars().next()
    }

    fn bump_char(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.pos += ch.len_utf8();
        }
    }
}

fn normalize_observed_assignment_target(text: &str) -> &str {
    let text = strip_wrapping_parens(text).trim();
    let text = text.split_once(':').map_or(text, |(name, _)| name);
    let text = strip_wrapping_parens(text).trim();
    text.strip_prefix("살림.")
        .or_else(|| text.strip_prefix("바탕."))
        .unwrap_or(text)
        .trim()
}

fn observed_assignment_affects_target(lhs: &str, target: &str) -> bool {
    if lhs.is_empty() || target.is_empty() {
        return false;
    }
    lhs == target
        || target
            .strip_prefix(lhs)
            .is_some_and(|rest| rest.starts_with('.'))
}

fn find_last_constant_int_assignment(
    source: &str,
    upto_line_index: usize,
    target: &str,
) -> Option<i64> {
    let target = normalize_observed_assignment_target(target);
    if target.is_empty() {
        return None;
    }
    let lines: Vec<&str> = source.lines().collect();
    for line in lines[..upto_line_index.min(lines.len())].iter().rev() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }
        let Some((lhs, rhs)) = line.split_once("<-") else {
            continue;
        };
        let lhs = normalize_observed_assignment_target(lhs);
        if !observed_assignment_affects_target(lhs, target) {
            continue;
        }
        if lhs != target {
            return None;
        }
        let rhs = rhs.trim().trim_end_matches('.').trim();
        if let Some(value) = parse_observed_const_int_expr(rhs) {
            return Some(value);
        }
        return None;
    }
    None
}

fn strip_wrapping_parens(mut text: &str) -> &str {
    loop {
        let trimmed = text.trim();
        if !(trimmed.starts_with('(') && trimmed.ends_with(')')) {
            return trimmed;
        }
        let mut depth = 0usize;
        let mut wraps = true;
        for (idx, ch) in trimmed.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    if depth == 0 {
                        wraps = false;
                        break;
                    }
                    depth -= 1;
                    if depth == 0 && idx != trimmed.len() - 1 {
                        wraps = false;
                        break;
                    }
                }
                _ => {}
            }
        }
        if !wraps || depth != 0 {
            return trimmed;
        }
        text = &trimmed[1..trimmed.len() - 1];
    }
}

fn is_constant_false_contract_cond(cond: &str) -> bool {
    let cond = strip_wrapping_parens(cond);
    if cond == "0" {
        return true;
    }
    if let Some(value) = parse_contract_const_bool(cond) {
        return !value;
    }

    for op in [">=", "<=", "==", "!=", ">", "<"] {
        if let Some((lhs, rhs)) = cond.split_once(op) {
            let lhs = strip_wrapping_parens(lhs);
            let rhs = strip_wrapping_parens(rhs);
            if lhs.is_empty() || rhs.is_empty() {
                return false;
            }
            if let (Some(lhs), Some(rhs)) = (
                parse_contract_const_bool(lhs),
                parse_contract_const_bool(rhs),
            ) {
                return match op {
                    "==" => lhs != rhs,
                    "!=" => lhs == rhs,
                    _ => false,
                };
            }
            if let (Some(lhs), Some(rhs)) =
                (parse_contract_const_int(lhs), parse_contract_const_int(rhs))
            {
                return match op {
                    ">=" => lhs < rhs,
                    "<=" => lhs > rhs,
                    "==" => lhs != rhs,
                    "!=" => lhs == rhs,
                    ">" => lhs <= rhs,
                    "<" => lhs >= rhs,
                    _ => false,
                };
            }
        }
    }

    false
}

fn build_observed_value_relaxation_patch(
    file: &str,
    source: &str,
    line_index: usize,
    before_line: &str,
    cond: &str,
) -> Option<String> {
    for op in [">=", "<=", "==", ">", "<"] {
        let Some((lhs, rhs)) = cond.split_once(op) else {
            continue;
        };
        let lhs = strip_wrapping_parens(lhs);
        let rhs = strip_wrapping_parens(rhs);
        let Some(rhs_value) = parse_contract_const_int(rhs) else {
            continue;
        };
        let Some(observed) = find_last_constant_int_assignment(source, line_index, lhs) else {
            continue;
        };
        let replacement_cond = match op {
            ">" if observed <= rhs_value => format!("{} >= {}", lhs.trim(), observed),
            "<" if observed >= rhs_value => format!("{} <= {}", lhs.trim(), observed),
            ">=" if observed < rhs_value => format!("{} >= {}", lhs.trim(), observed),
            "<=" if observed > rhs_value => format!("{} <= {}", lhs.trim(), observed),
            "==" if observed != rhs_value => format!("{} == {}", lhs.trim(), observed),
            _ => continue,
        };
        let brace_start = before_line.find('{')?;
        let brace_end = before_line[brace_start + 1..].find('}')? + brace_start + 1;
        let after_line = format!(
            "{} {} {}",
            before_line[..brace_start + 1].trim_end(),
            replacement_cond,
            before_line[brace_end..].trim_start()
        );
        let patch = format!(
            "{{\"patch_version\":\"0.1-draft\",\"changes\":[{{\"kind\":\"replace_block\",\"target\":{{\"file\":\"{}\",\"anchor\":\"{}\"}},\"before\":[\"{}\"],\"after\":[\"{}\"],\"reason\":\"{}\"}}]}}",
            escape_json(file),
            escape_json(before_line),
            escape_json(before_line),
            escape_json(&after_line),
            escape_json("슬기 계약 수선 후보: 관측된 값 기준으로 계약 경계를 완화")
        );
        return Some(patch);
    }
    None
}

fn build_contract_patch_candidate(
    file: &str,
    source: &str,
    event: &ContractDiag,
) -> Option<String> {
    let line_index = event.span.start_line.checked_sub(1)? as usize;
    let line = source.lines().nth(line_index)?.trim_end_matches('\r');
    let brace_start = line.find('{')?;
    let brace_end = line[brace_start + 1..].find('}')? + brace_start + 1;
    let cond = line[brace_start + 1..brace_end].trim();
    let before_line = line.to_string();
    let replacement = if cond == "0" {
        "1"
    } else if is_constant_false_contract_cond(cond) {
        "참"
    } else {
        return build_observed_value_relaxation_patch(file, source, line_index, &before_line, cond);
    };
    let after_line = format!(
        "{} {} {}",
        line[..brace_start + 1].trim_end(),
        replacement,
        line[brace_end..].trim_start()
    );
    let patch = format!(
        "{{\"patch_version\":\"0.1-draft\",\"changes\":[{{\"kind\":\"replace_block\",\"target\":{{\"file\":\"{}\",\"anchor\":\"{}\"}},\"before\":[\"{}\"],\"after\":[\"{}\"],\"reason\":\"{}\"}}]}}",
        escape_json(file),
        escape_json(&before_line),
        escape_json(&before_line),
        escape_json(&after_line),
        escape_json("슬기 계약 수선 후보: 상수 거짓 계약식을 참으로 교정")
    );
    Some(patch)
}

fn append_contract_hook_result_event(
    file_handle: &mut std::fs::File,
    fault_id: &str,
    file: &str,
    source: &str,
    event: &ContractDiag,
) -> Result<(), String> {
    let suggestions = build_contract_hook_suggestions(event);
    let suggestions_json = suggestions
        .iter()
        .map(|(kind, message)| {
            format!(
                "{{\"kind\":\"{}\",\"message\":\"{}\"}}",
                escape_json(kind),
                escape_json(message)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let patch_candidates = build_contract_patch_candidate(file, source, event)
        .map(|candidate| vec![candidate])
        .unwrap_or_default();
    let patch_candidates_json = if patch_candidates.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", patch_candidates.join(","))
    };
    let json = format!(
        "{{\"event_kind\":\"hook_result\",\"hook_name\":\"슬기.계약위반\",\"hook_policy\":\"실행\",\"hook_input_ref\":\"{}\",\"executor\":\"builtin.seulgi.contract.v1\",\"result\":\"suggested\",\"suggestion_count\":{},\"suggestions\":[{}],\"patch_candidate_count\":{},\"patch_candidates\":{}}}\n",
        escape_json(fault_id),
        suggestions.len(),
        suggestions_json,
        patch_candidates.len(),
        patch_candidates_json
    );
    file_handle
        .write_all(json.as_bytes())
        .map_err(|e| e.to_string())
}

fn append_contract_diags(
    path: &Path,
    file: &str,
    source: &str,
    events: &[ContractDiag],
    hook_policy: SeulgiHookPolicy,
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
            ContractMode::Abort => "물림",
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
        file_handle
            .write_all(json.as_bytes())
            .map_err(|e| e.to_string())?;
        append_contract_hook_event(&mut file_handle, &fault_id, file, &code, event, hook_policy)?;
        if hook_policy == SeulgiHookPolicy::Execute {
            append_contract_hook_result_event(&mut file_handle, &fault_id, file, source, event)?;
        }
    }
    Ok(())
}

fn diag_extra_fields(err: &RunError) -> Option<String> {
    match err {
        RunError::Bogae(BogaeError::CmdCap { cap, cmd_count }) => Some(format!(
            ",\"rule_id\":\"cmd_cap\",\"reason\":\"cmd_count_exceeds_cap\",\"cmd_count\":{},\"cap\":{}",
            cmd_count, cap
        )),
        RunError::Runtime(RuntimeError::EcoDivergenceDetected { delta, threshold, .. }) => {
            Some(format!(
                ",\"rule_id\":\"eco_diag\",\"reason\":\"divergence_detected\",\"delta\":{},\"threshold\":{}",
                delta, threshold
            ))
        }
        RunError::Runtime(RuntimeError::SfcIdentityViolation { delta, threshold, .. }) => {
            Some(format!(
                ",\"rule_id\":\"sfc_identity\",\"reason\":\"identity_violation\",\"delta\":{},\"threshold\":{}",
                delta, threshold
            ))
        }
        _ => None,
    }
}

fn diagnostic_failure_to_runtime_error(failure: &DiagnosticFailure) -> RuntimeError {
    match failure.code.as_str() {
        "E_SFC_IDENTITY_VIOLATION" => RuntimeError::SfcIdentityViolation {
            tick: failure.tick,
            name: failure.name.clone(),
            delta: failure.delta.clone(),
            threshold: failure.threshold.clone(),
            span: failure.span,
        },
        _ => RuntimeError::EcoDivergenceDetected {
            tick: failure.tick,
            name: failure.name.clone(),
            delta: failure.delta.clone(),
            threshold: failure.threshold.clone(),
            span: failure.span,
        },
    }
}

fn write_diagnostic_report(
    path: &Path,
    seed: u64,
    tick: u64,
    diagnostics: &[DiagnosticRecord],
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut out = String::new();
    out.push_str("{\"schema\":\"ddn.diagnostic_report.v0\",\"seed\":");
    out.push_str(&seed.to_string());
    out.push_str(",\"tick\":");
    out.push_str(&tick.to_string());
    out.push_str(",\"diagnostics\":[");
    for (idx, diag) in diagnostics.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push('{');
        out.push_str("\"tick\":");
        out.push_str(&diag.tick.to_string());
        out.push(',');
        out.push_str("\"name\":\"");
        out.push_str(&escape_json(&diag.name));
        out.push_str("\",\"lhs\":");
        out.push_str(&diag.lhs);
        out.push_str(",\"rhs\":");
        out.push_str(&diag.rhs);
        out.push_str(",\"delta\":");
        out.push_str(&diag.delta);
        out.push_str(",\"threshold\":");
        out.push_str(&diag.threshold);
        out.push_str(",\"result\":\"");
        out.push_str(&escape_json(&diag.result));
        out.push('"');
        if let Some(error_code) = diag.error_code.as_ref() {
            out.push_str(",\"error_code\":\"");
            out.push_str(&escape_json(error_code));
            out.push('"');
        }
        out.push('}');
    }
    out.push_str("]}\n");
    fs::write(path, out).map_err(|e| e.to_string())
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
    file_handle
        .write_all(json.as_bytes())
        .map_err(|e| e.to_string())
}

fn append_latency_schedule_diag(
    path: &Path,
    file: &str,
    applied_madi: u64,
    event: InputLatencyApplied,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = format!(
        "{{\"level\":\"info\",\"event\":\"latency_schedule\",\"file\":\"{}\",\"applied_madi\":{},\"accept_madi\":{},\"target_madi\":{},\"late\":{},\"dropped\":{},\"drop_policy\":\"{}\"}}\n",
        escape_json(file),
        applied_madi,
        event.accept_madi,
        event.target_madi,
        if event.late { "true" } else { "false" },
        if event.dropped { "true" } else { "false" },
        latency_drop_policy_label(),
    );
    let mut file_handle = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file_handle
        .write_all(json.as_bytes())
        .map_err(|e| e.to_string())
}

fn append_run_config_diag(
    path: &Path,
    age_target_source: AgeTargetSource,
    age_target: AgeTarget,
    latency_madi: u64,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = format!(
        "{{\"level\":\"info\",\"event\":\"run_config\",\"age_target_source\":\"{}\",\"age_target_value\":\"{}\",\"latency_madi\":{},\"latency_drop_policy\":\"{}\",\"tick\":0,\"seq\":0}}\n",
        escape_json(age_target_source.label()),
        escape_json(age_target.label()),
        latency_madi,
        latency_drop_policy_label(),
    );
    let mut file_handle = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file_handle
        .write_all(json.as_bytes())
        .map_err(|e| e.to_string())
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
    fs::write(path, format!("{}\n", text)).map_err(|e| format!("E_RUN_MANIFEST_WRITE {}", e))?;
    Ok(())
}

fn unary_op_label(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "neg",
        UnaryOp::Not => "not",
    }
}

fn binary_op_label(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "add",
        BinaryOp::Sub => "sub",
        BinaryOp::Mul => "mul",
        BinaryOp::Div => "div",
        BinaryOp::Mod => "mod",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
        BinaryOp::Eq => "eq",
        BinaryOp::NotEq => "ne",
        BinaryOp::Lt => "lt",
        BinaryOp::Lte => "lte",
        BinaryOp::Gt => "gt",
        BinaryOp::Gte => "gte",
    }
}

fn path_to_json(path: &AstPath) -> JsonValue {
    json!({
        "segments": path.segments,
        "implicit_root": path.implicit_root,
    })
}

fn literal_to_json(literal: &Literal) -> JsonValue {
    match literal {
        Literal::None => json!({"kind": "none"}),
        Literal::Bool(flag) => json!({"kind": "bool", "value": flag}),
        Literal::Num(number) => json!({
            "kind": "num",
            "raw": number.raw,
            "unit": number.unit.as_ref().map(|unit| format!("{unit:?}")),
        }),
        Literal::Str(text) => json!({"kind": "str", "value": text}),
    }
}

fn binding_to_json(binding: &Binding) -> JsonValue {
    json!({
        "name": binding.name,
        "value": expr_to_json(&binding.value),
    })
}

fn arg_binding_to_json(binding: &ArgBinding) -> JsonValue {
    json!({
        "expr": expr_to_json(&binding.expr),
        "josa": binding.josa,
        "resolved_pin": binding.resolved_pin,
        "binding_reason": format!("{:?}", binding.binding_reason),
    })
}

fn expr_to_json(expr: &Expr) -> JsonValue {
    match expr {
        Expr::Literal(literal, _) => json!({
            "kind": "literal",
            "literal": literal_to_json(literal),
        }),
        Expr::Path(path) => json!({
            "kind": "path",
            "path": path_to_json(path),
        }),
        Expr::FieldAccess { target, field, .. } => json!({
            "kind": "field_access",
            "target": expr_to_json(target),
            "field": field,
        }),
        Expr::Atom { text, .. } => json!({
            "kind": "atom",
            "text": text,
        }),
        Expr::Unary { op, expr, .. } => json!({
            "kind": "unary",
            "op": unary_op_label(op),
            "expr": expr_to_json(expr),
        }),
        Expr::Binary {
            left, op, right, ..
        } => json!({
            "kind": "binary",
            "op": binary_op_label(op),
            "left": expr_to_json(left),
            "right": expr_to_json(right),
        }),
        Expr::SeedLiteral { param, body, .. } => json!({
            "kind": "seed_literal",
            "param": param,
            "body": expr_to_json(body),
        }),
        Expr::Call { name, args, .. } => json!({
            "kind": "call",
            "name": name,
            "args": args.iter().map(arg_binding_to_json).collect::<Vec<_>>(),
        }),
        Expr::Formula { dialect, body, .. } => json!({
            "kind": "formula",
            "dialect": format!("{dialect:?}"),
            "body": body,
        }),
        Expr::Assertion { assertion, .. } => json!({
            "kind": "assertion",
            "canon": assertion.canon,
            "body_source": assertion.body_source,
        }),
        Expr::FormulaEval {
            dialect,
            body,
            bindings,
            ..
        } => json!({
            "kind": "formula_eval",
            "dialect": format!("{dialect:?}"),
            "body": body,
            "bindings": bindings.iter().map(binding_to_json).collect::<Vec<_>>(),
        }),
        Expr::Template { body, .. } => json!({
            "kind": "template",
            "body": body,
        }),
        Expr::TemplateFill {
            template, bindings, ..
        } => json!({
            "kind": "template_fill",
            "template": expr_to_json(template),
            "bindings": bindings.iter().map(binding_to_json).collect::<Vec<_>>(),
        }),
        Expr::Pack { bindings, .. } => json!({
            "kind": "pack",
            "bindings": bindings.iter().map(binding_to_json).collect::<Vec<_>>(),
        }),
        Expr::FormulaFill {
            formula, bindings, ..
        } => json!({
            "kind": "formula_fill",
            "formula": expr_to_json(formula),
            "bindings": bindings.iter().map(binding_to_json).collect::<Vec<_>>(),
        }),
    }
}

fn stmt_to_json(stmt: &Stmt) -> JsonValue {
    match stmt {
        Stmt::ImportBlock { items, .. } => json!({
            "kind": "import_block",
            "items": items.iter().map(|item| json!({
                "alias": item.alias,
                "path": item.path,
            })).collect::<Vec<_>>(),
        }),
        Stmt::ExportBlock { items, .. } => json!({
            "kind": "export_block",
            "items": items.iter().map(|item| json!({
                "external_name": item.external_name,
                "internal_name": item.internal_name,
            })).collect::<Vec<_>>(),
        }),
        Stmt::DeclBlock { items, .. } => json!({
            "kind": "decl_block",
            "items": items.iter().map(|item| json!({
                "name": item.name,
                "decl_kind": format!("{:?}", item.kind),
                "type_name": item.type_name,
                "value": item.value.as_ref().map(expr_to_json),
                "maegim": item.maegim.as_ref().map(|spec| {
                    spec.fields.iter().map(binding_to_json).collect::<Vec<_>>()
                }),
            })).collect::<Vec<_>>(),
        }),
        Stmt::SeedDef {
            name,
            params,
            kind,
            body,
            ..
        } => json!({
            "kind": "seed_def",
            "name": name,
            "seed_kind": format!("{kind:?}"),
            "params": params.iter().map(|param| json!({
                "name": param.name,
                "josa_list": param.josa_list,
            })).collect::<Vec<_>>(),
            "body": body.iter().map(stmt_to_json).collect::<Vec<_>>(),
        }),
        Stmt::Assign {
            target,
            value,
            deferred,
            ..
        } => json!({
            "kind": "assign",
            "target": path_to_json(target),
            "value": expr_to_json(value),
            "deferred": deferred,
        }),
        Stmt::Expr { value, .. } => json!({
            "kind": "expr",
            "value": expr_to_json(value),
        }),
        Stmt::Receive {
            kind,
            binding,
            condition,
            body,
            ..
        } => json!({
            "kind": "receive",
            "event_kind": kind,
            "binding": binding,
            "condition": condition.as_ref().map(expr_to_json),
            "body": body.iter().map(stmt_to_json).collect::<Vec<_>>(),
        }),
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => json!({
            "kind": "send",
            "sender": sender.as_ref().map(expr_to_json),
            "payload": expr_to_json(payload),
            "receiver": expr_to_json(receiver),
        }),
        Stmt::Return { value, .. } => json!({
            "kind": "return",
            "value": expr_to_json(value),
        }),
        Stmt::Show { value, .. } => json!({
            "kind": "show",
            "value": expr_to_json(value),
        }),
        Stmt::Inspect { value, .. } => json!({
            "kind": "inspect",
            "value": expr_to_json(value),
        }),
        Stmt::BogaeDraw { .. } => json!({"kind": "bogae_draw"}),
        Stmt::Hook { kind, body, .. } => json!({
            "kind": "hook",
            "hook_kind": format!("{kind:?}"),
            "body": body.iter().map(stmt_to_json).collect::<Vec<_>>(),
        }),
        Stmt::HookWhenBecomes {
            condition, body, ..
        } => json!({
            "kind": "hook_when_becomes",
            "condition": expr_to_json(condition),
            "body": body.iter().map(stmt_to_json).collect::<Vec<_>>(),
        }),
        Stmt::HookWhile {
            condition, body, ..
        } => json!({
            "kind": "hook_while",
            "condition": expr_to_json(condition),
            "body": body.iter().map(stmt_to_json).collect::<Vec<_>>(),
        }),
        Stmt::OpenBlock { body, .. } => json!({
            "kind": "open_block",
            "body": body.iter().map(stmt_to_json).collect::<Vec<_>>(),
        }),
        Stmt::BeatBlock { body, .. } => json!({
            "kind": "beat_block",
            "body": body.iter().map(stmt_to_json).collect::<Vec<_>>(),
        }),
        Stmt::LifecycleBlock {
            name, kind, body, ..
        } => json!({
            "kind": "lifecycle_block",
            "name": name,
            "lifecycle_kind": format!("{kind:?}"),
            "body": body.iter().map(stmt_to_json).collect::<Vec<_>>(),
        }),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => json!({
            "kind": "if",
            "condition": expr_to_json(condition),
            "then_body": then_body.iter().map(stmt_to_json).collect::<Vec<_>>(),
            "else_body": else_body.as_ref().map(|body| body.iter().map(stmt_to_json).collect::<Vec<_>>()),
        }),
        Stmt::Choose {
            branches,
            else_body,
            exhaustive,
            ..
        } => json!({
            "kind": "choose",
            "branches": branches.iter().map(|branch| json!({
                "condition": expr_to_json(&branch.condition),
                "body": branch.body.iter().map(stmt_to_json).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "else_body": else_body.as_ref().map(|body| body.iter().map(stmt_to_json).collect::<Vec<_>>()),
            "exhaustive": exhaustive,
        }),
        Stmt::Repeat { body, .. } => json!({
            "kind": "repeat",
            "body": body.iter().map(stmt_to_json).collect::<Vec<_>>(),
        }),
        Stmt::While {
            condition, body, ..
        } => json!({
            "kind": "while",
            "condition": expr_to_json(condition),
            "body": body.iter().map(stmt_to_json).collect::<Vec<_>>(),
        }),
        Stmt::ForEach {
            item,
            iterable,
            body,
            ..
        } => json!({
            "kind": "foreach",
            "item": item,
            "iterable": expr_to_json(iterable),
            "body": body.iter().map(stmt_to_json).collect::<Vec<_>>(),
        }),
        Stmt::Quantifier {
            kind,
            variable,
            domain,
            body,
            ..
        } => json!({
            "kind": "quantifier",
            "quantifier": quantifier_kind_label(*kind),
            "binder": variable,
            "domain": domain,
            "body": body.iter().map(stmt_to_json).collect::<Vec<_>>(),
        }),
        Stmt::Break { .. } => json!({"kind": "break"}),
        Stmt::Contract {
            kind,
            mode,
            condition,
            then_body,
            else_body,
            ..
        } => json!({
            "kind": "contract",
            "contract_kind": contract_kind_label(*kind),
            "mode": contract_mode_label(*mode),
            "condition": expr_to_json(condition),
            "then_body": then_body.as_ref().map(|body| body.iter().map(stmt_to_json).collect::<Vec<_>>()),
            "else_body": else_body.iter().map(stmt_to_json).collect::<Vec<_>>(),
        }),
        Stmt::Pragma { name, args, .. } => json!({
            "kind": "pragma",
            "name": name,
            "args": args,
        }),
    }
}

fn fallback_structural_body_hash_with_mode(
    source: &str,
    parse_mode: ParseMode,
) -> Result<String, String> {
    let program = parse_program_for_proof_with_mode(source, parse_mode)?;
    let root = json!({
        "schema": "ddn.proof.ast_fallback.v0",
        "stmts": program.stmts.iter().map(stmt_to_json).collect::<Vec<_>>(),
    });
    let text = serde_json::to_string(&root).map_err(|e| format!("E_PROOF_AST_JSON {}", e))?;
    let digest = Sha256::digest(text.as_bytes());
    Ok(format!("sha256:{:x}", digest))
}

fn canonical_body_hash_with_mode(
    source: &str,
    parse_mode: ParseMode,
) -> Result<(String, &'static str), String> {
    match canon::canonicalize(source, false) {
        Ok(output) => {
            let digest = Sha256::digest(output.ddn.as_bytes());
            Ok((format!("sha256:{:x}", digest), "canon"))
        }
        Err(_) => Ok((fallback_structural_body_hash_with_mode(source, parse_mode)?, "ast_fallback")),
    }
}

#[cfg(test)]
fn canonical_body_hash(source: &str) -> Result<(String, &'static str), String> {
    canonical_body_hash_with_mode(source, ParseMode::Strict)
}

fn contract_kind_label(kind: ContractKind) -> &'static str {
    match kind {
        ContractKind::Pre => "pre",
        ContractKind::Post => "post",
    }
}

fn contract_mode_label(mode: ContractMode) -> &'static str {
    match mode {
        ContractMode::Alert => "alert",
        ContractMode::Abort => "abort",
    }
}

fn quantifier_kind_label(kind: QuantifierKind) -> &'static str {
    match kind {
        QuantifierKind::ForAll => "forall",
        QuantifierKind::Exists => "exists",
        QuantifierKind::ExistsUnique => "exists_unique",
    }
}

fn parse_program_for_proof_with_mode(source: &str, parse_mode: ParseMode) -> Result<Program, String> {
    let (program, _) = parse_program_for_runtime_with_mode(source, parse_mode).map_err(|err| {
        match err {
            FrontdoorParseFailure::Guard(message) => message,
            FrontdoorParseFailure::Lex(err) => format!("{} {}", err.code(), lex_message(&err)),
            FrontdoorParseFailure::Parse(err) => format!("{} {}", err.code(), parse_message(&err)),
        }
    })?;
    Ok(program)
}

fn extract_solver_query_hint(args: &[ArgBinding]) -> Option<String> {
    if args.len() != 1 {
        return None;
    }
    match &args[0].expr {
        Expr::Literal(Literal::Str(text), _) => Some(text.clone()),
        Expr::Pack { bindings, .. } => bindings.iter().find_map(|binding| {
            if binding.name == "질의" || binding.name == "query" {
                match &binding.value {
                    Expr::Literal(Literal::Str(text), _) => Some(text.clone()),
                    _ => None,
                }
            } else {
                None
            }
        }),
        _ => None,
    }
}

fn collect_solver_translation_expr(expr: &Expr, items: &mut Vec<JsonValue>) {
    match expr {
        Expr::Call { name, args, span } => {
            let canon = ddonirang_lang::stdlib::canonicalize_stdlib_alias(name);
            if canon == "살피기" {
                let mut obj = serde_json::Map::new();
                obj.insert(
                    "kind".to_string(),
                    JsonValue::String("proof_check".to_string()),
                );
                obj.insert(
                    "arg_count".to_string(),
                    JsonValue::Number(serde_json::Number::from(args.len() as u64)),
                );
                obj.insert(
                    "line".to_string(),
                    JsonValue::Number(serde_json::Number::from(span.start_line as u64)),
                );
                obj.insert(
                    "col".to_string(),
                    JsonValue::Number(serde_json::Number::from(span.start_col as u64)),
                );
                let target = match args.as_slice() {
                    [arg, ..] => match &arg.expr {
                        Expr::Path(path) => Some(JsonValue::String(path.segments.join("."))),
                        Expr::Assertion { assertion, .. } => {
                            Some(JsonValue::String(assertion.canon.clone()))
                        }
                        Expr::Pack { bindings, .. } => bindings.iter().find_map(|binding| {
                            if binding.name == "세움"
                                || binding.name == "assertion"
                                || binding.name == "검사"
                                || binding.name == "target"
                            {
                                match &binding.value {
                                    Expr::Path(path) => {
                                        Some(JsonValue::String(path.segments.join(".")))
                                    }
                                    Expr::Assertion { assertion, .. } => {
                                        Some(JsonValue::String(assertion.canon.clone()))
                                    }
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        }),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(target) = target {
                    obj.insert("target".to_string(), target);
                }
                let binding_count = match args.as_slice() {
                    [_, arg] => match &arg.expr {
                        Expr::Pack { bindings, .. } => Some(bindings.len() as u64),
                        _ => None,
                    },
                    [arg] => match &arg.expr {
                        Expr::Pack { bindings, .. } => bindings.iter().find_map(|binding| {
                            if binding.name == "값들" || binding.name == "bindings" {
                                match &binding.value {
                                    Expr::Pack { bindings, .. } => Some(bindings.len() as u64),
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        }),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(binding_count) = binding_count {
                    obj.insert(
                        "binding_count".to_string(),
                        JsonValue::Number(serde_json::Number::from(binding_count)),
                    );
                }
                items.push(JsonValue::Object(obj));
            }
            let operation = match canon {
                "열림.풀이.확인" => Some("check"),
                "반례찾기" => Some("counterexample"),
                "해찾기" => Some("solve"),
                _ => None,
            };
            if let Some(operation) = operation {
                let mut obj = serde_json::Map::new();
                obj.insert(
                    "kind".to_string(),
                    JsonValue::String("solver_open".to_string()),
                );
                obj.insert(
                    "operation".to_string(),
                    JsonValue::String(operation.to_string()),
                );
                obj.insert(
                    "arg_count".to_string(),
                    JsonValue::Number(serde_json::Number::from(args.len() as u64)),
                );
                obj.insert(
                    "line".to_string(),
                    JsonValue::Number(serde_json::Number::from(span.start_line as u64)),
                );
                obj.insert(
                    "col".to_string(),
                    JsonValue::Number(serde_json::Number::from(span.start_col as u64)),
                );
                if let Some(query) = extract_solver_query_hint(args) {
                    obj.insert("query".to_string(), JsonValue::String(query));
                }
                items.push(JsonValue::Object(obj));
            }
            for arg in args {
                collect_solver_translation_expr(&arg.expr, items);
            }
        }
        Expr::Unary { expr, .. } => collect_solver_translation_expr(expr, items),
        Expr::Binary { left, right, .. } => {
            collect_solver_translation_expr(left, items);
            collect_solver_translation_expr(right, items);
        }
        Expr::FieldAccess { target, .. } => collect_solver_translation_expr(target, items),
        Expr::SeedLiteral { body, .. } => collect_solver_translation_expr(body, items),
        Expr::TemplateFill {
            template, bindings, ..
        } => {
            collect_solver_translation_expr(template, items);
            for binding in bindings {
                collect_solver_translation_expr(&binding.value, items);
            }
        }
        Expr::Pack { bindings, .. } | Expr::FormulaEval { bindings, .. } => {
            for binding in bindings {
                collect_solver_translation_expr(&binding.value, items);
            }
        }
        Expr::FormulaFill {
            formula, bindings, ..
        } => {
            collect_solver_translation_expr(formula, items);
            for binding in bindings {
                collect_solver_translation_expr(&binding.value, items);
            }
        }
        Expr::Literal(_, _)
        | Expr::Path(_)
        | Expr::Atom { .. }
        | Expr::Assertion { .. }
        | Expr::Formula { .. }
        | Expr::Template { .. } => {}
    }
}

fn collect_solver_translation_stmt(stmt: &Stmt, items: &mut Vec<JsonValue>) {
    match stmt {
        Stmt::ImportBlock { .. }
        | Stmt::ExportBlock { .. }
        | Stmt::BogaeDraw { .. }
        | Stmt::Break { .. }
        | Stmt::Pragma { .. } => {}
        Stmt::DeclBlock { items: decls, .. } => {
            for decl in decls {
                if let Some(value) = decl.value.as_ref() {
                    collect_solver_translation_expr(value, items);
                }
                if let Some(maegim) = decl.maegim.as_ref() {
                    for field in &maegim.fields {
                        collect_solver_translation_expr(&field.value, items);
                    }
                }
            }
        }
        Stmt::SeedDef {
            name,
            kind,
            body,
            span,
            ..
        } => {
            if matches!(kind, SeedKind::Balhigi) {
                let mut obj = serde_json::Map::new();
                obj.insert(
                    "kind".to_string(),
                    JsonValue::String("proof_block".to_string()),
                );
                obj.insert("name".to_string(), JsonValue::String(name.clone()));
                obj.insert(
                    "line".to_string(),
                    JsonValue::Number(serde_json::Number::from(span.start_line as u64)),
                );
                obj.insert(
                    "col".to_string(),
                    JsonValue::Number(serde_json::Number::from(span.start_col as u64)),
                );
                items.push(JsonValue::Object(obj));
            }
            collect_solver_translation_items(body, items);
        }
        Stmt::Hook { body, .. }
        | Stmt::OpenBlock { body, .. }
        | Stmt::BeatBlock { body, .. }
        | Stmt::LifecycleBlock { body, .. }
        | Stmt::Repeat { body, .. }
        | Stmt::ForEach { body, .. } => collect_solver_translation_items(body, items),
        Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => {
            collect_solver_translation_expr(condition, items);
            collect_solver_translation_items(body, items);
        }
        Stmt::Assign { value, .. }
        | Stmt::Expr { value, .. }
        | Stmt::Return { value, .. }
        | Stmt::Show { value, .. }
        | Stmt::Inspect { value, .. } => {
            collect_solver_translation_expr(value, items);
        }
        Stmt::Receive {
            condition, body, ..
        } => {
            if let Some(condition) = condition.as_ref() {
                collect_solver_translation_expr(condition, items);
            }
            collect_solver_translation_items(body, items);
        }
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => {
            if let Some(sender) = sender.as_ref() {
                collect_solver_translation_expr(sender, items);
            }
            collect_solver_translation_expr(payload, items);
            collect_solver_translation_expr(receiver, items);
        }
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_solver_translation_expr(condition, items);
            collect_solver_translation_items(then_body, items);
            if let Some(body) = else_body {
                collect_solver_translation_items(body, items);
            }
        }
        Stmt::Choose {
            branches,
            else_body,
            exhaustive,
            span,
        } => {
            let completion = match (else_body.is_some(), *exhaustive) {
                (true, true) => "else+exhaustive",
                (true, false) => "else",
                (false, true) => "exhaustive",
                (false, false) => "none",
            };
            items.push(json!({
                "kind": "case_analysis",
                "branch_count": branches.len(),
                "has_else": else_body.is_some(),
                "exhaustive": exhaustive,
                "completion": completion,
                "line": span.start_line,
                "col": span.start_col,
            }));
            for branch in branches {
                collect_solver_translation_expr(&branch.condition, items);
                collect_solver_translation_items(&branch.body, items);
            }
            if let Some(body) = else_body {
                collect_solver_translation_items(body, items);
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            collect_solver_translation_expr(condition, items);
            collect_solver_translation_items(body, items);
        }
        Stmt::Quantifier {
            kind,
            variable,
            domain,
            body,
            span,
        } => {
            items.push(json!({
                "kind": "quantifier",
                "quantifier": quantifier_kind_label(*kind),
                "binder": variable,
                "domain": domain,
                "body_stmt_count": body.len(),
                "line": span.start_line,
                "col": span.start_col,
            }));
            collect_solver_translation_items(body, items);
        }
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_solver_translation_expr(condition, items);
            if let Some(body) = then_body {
                collect_solver_translation_items(body, items);
            }
            collect_solver_translation_items(else_body, items);
        }
    }
}

fn collect_solver_translation_items(stmts: &[Stmt], items: &mut Vec<JsonValue>) {
    for stmt in stmts {
        collect_solver_translation_stmt(stmt, items);
    }
}

fn build_solver_translation(program: &Program) -> JsonValue {
    let mut items = Vec::new();
    collect_solver_translation_items(&program.stmts, &mut items);
    let quantifier_count = items
        .iter()
        .filter(|item| item["kind"] == JsonValue::String("quantifier".to_string()))
        .count();
    let case_analysis_count = items
        .iter()
        .filter(|item| item["kind"] == JsonValue::String("case_analysis".to_string()))
        .count();
    let solver_open_count = items
        .iter()
        .filter(|item| item["kind"] == JsonValue::String("solver_open".to_string()))
        .count();
    let proof_check_count = items
        .iter()
        .filter(|item| item["kind"] == JsonValue::String("proof_check".to_string()))
        .count();
    json!({
        "schema": "ddn.proof.solver_translation.v0",
        "item_count": items.len(),
        "quantifier_count": quantifier_count,
        "case_analysis_count": case_analysis_count,
        "solver_open_count": solver_open_count,
        "proof_check_count": proof_check_count,
        "items": items,
    })
}

fn build_proof_runtime(events: &[ProofRuntimeEvent]) -> JsonValue {
    let items = events
        .iter()
        .map(|event| match event {
            ProofRuntimeEvent::ProofBlock {
                tick,
                name,
                result,
                error_code,
            } => json!({
                "kind": "proof_block",
                "tick": tick,
                "name": name,
                "result": result,
                "error_code": error_code,
            }),
            ProofRuntimeEvent::ProofCheck {
                tick,
                target,
                binding_count,
                passed,
                error_code,
                span,
            } => json!({
                "kind": "proof_check",
                "tick": tick,
                "target": target,
                "binding_count": binding_count,
                "passed": passed,
                "error_code": error_code,
                "line": span.start_line,
                "col": span.start_col,
            }),
            ProofRuntimeEvent::SolverCheck {
                tick,
                query,
                satisfied,
                error_code,
                span,
            } => json!({
                "kind": "solver_check",
                "tick": tick,
                "query": query,
                "satisfied": satisfied,
                "error_code": error_code,
                "line": span.start_line,
                "col": span.start_col,
            }),
            ProofRuntimeEvent::SolverSearch {
                tick,
                operation,
                query,
                found,
                value,
                error_code,
                span,
            } => json!({
                "kind": "solver_search",
                "tick": tick,
                "operation": operation,
                "query": query,
                "found": found,
                "value": value,
                "error_code": error_code,
                "line": span.start_line,
                "col": span.start_col,
            }),
        })
        .collect::<Vec<_>>();
    let proof_check_count = items
        .iter()
        .filter(|item| item["kind"] == JsonValue::String("proof_check".to_string()))
        .count();
    let proof_block_count = items
        .iter()
        .filter(|item| item["kind"] == JsonValue::String("proof_block".to_string()))
        .count();
    let solver_check_count = items
        .iter()
        .filter(|item| item["kind"] == JsonValue::String("solver_check".to_string()))
        .count();
    let solver_search_count = items
        .iter()
        .filter(|item| item["kind"] == JsonValue::String("solver_search".to_string()))
        .count();
    json!({
        "schema": "ddn.proof.runtime.v0",
        "item_count": items.len(),
        "proof_block_count": proof_block_count,
        "proof_check_count": proof_check_count,
        "solver_check_count": solver_check_count,
        "solver_search_count": solver_search_count,
        "items": items,
    })
}

fn json_sha256(value: &JsonValue) -> Result<String, String> {
    let text = serde_json::to_string(value).map_err(|e| format!("E_PROOF_JSON {}", e))?;
    let digest = Sha256::digest(text.as_bytes());
    Ok(format!("sha256:{:x}", digest))
}

fn build_proof_detjson_with_mode(
    entry: &str,
    seed: u64,
    ticks: u64,
    source: &str,
    parse_mode: ParseMode,
    state_hash: &str,
    trace_hash: &str,
    output: &EvalOutput,
    runtime_error: Option<&RunError>,
) -> Result<JsonValue, String> {
    let (canonical_hash, canonical_hash_method) = canonical_body_hash_with_mode(source, parse_mode)?;
    let program = parse_program_for_proof_with_mode(source, parse_mode)?;
    let solver_translation = build_solver_translation(&program);
    let solver_translation_hash = json_sha256(&solver_translation)?;
    let proof_runtime = build_proof_runtime(&output.proof_runtime);
    let proof_runtime_hash = json_sha256(&proof_runtime)?;
    let contract_diags: Vec<JsonValue> = output
        .contract_diags
        .iter()
        .map(|diag| {
            json!({
                "kind": contract_kind_label(diag.kind),
                "mode": contract_mode_label(diag.mode),
                "message": diag.message,
                "line": diag.span.start_line,
                "col": diag.span.start_col,
            })
        })
        .collect();
    let diagnostic_failures: Vec<JsonValue> = output
        .diagnostic_failures
        .iter()
        .map(|failure| {
            json!({
                "code": failure.code,
                "tick": failure.tick,
                "name": failure.name,
                "delta": failure.delta,
                "threshold": failure.threshold,
                "line": failure.span.start_line,
                "col": failure.span.start_col,
            })
        })
        .collect();
    let runtime_error_json = runtime_error.and_then(build_proof_runtime_error);
    let verified =
        contract_diags.is_empty() && diagnostic_failures.is_empty() && runtime_error_json.is_none();
    let mut root = json!({
        "schema": "ddn.proof.detjson.v0",
        "kind": "run_contract_certificate_v1",
        "entry": entry,
        "seed": seed,
        "ticks": ticks,
        "ssot_version": hash::SSOT_VERSION,
        "toolchain_version": env!("CARGO_PKG_VERSION"),
        "canonical_body_hash": canonical_hash,
        "canonical_body_hash_method": canonical_hash_method,
        "solver_translation_hash": solver_translation_hash,
        "proof_runtime_hash": proof_runtime_hash,
        "state_hash": state_hash,
        "trace_hash": trace_hash,
        "verified": verified,
        "contract_diag_count": output.contract_diags.len(),
        "diagnostic_failure_count": output.diagnostic_failures.len(),
        "trace_log_count": output.trace.log_lines().len(),
        "proof_runtime_count": output.proof_runtime.len(),
        "contract_diags": contract_diags,
        "diagnostic_failures": diagnostic_failures,
        "solver_translation": solver_translation,
        "proof_runtime": proof_runtime,
    });
    if let Some(runtime_error) = runtime_error_json {
        root["runtime_error"] = runtime_error;
    }
    Ok(root)
}

#[cfg(test)]
fn build_proof_detjson(
    entry: &str,
    seed: u64,
    ticks: u64,
    source: &str,
    state_hash: &str,
    trace_hash: &str,
    output: &EvalOutput,
    runtime_error: Option<&RunError>,
) -> Result<JsonValue, String> {
    build_proof_detjson_with_mode(
        entry,
        seed,
        ticks,
        source,
        ParseMode::Strict,
        state_hash,
        trace_hash,
        output,
        runtime_error,
    )
}

fn sidecar_proof_output_path(path: &Path, label: &str) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("proof.detjson");
    let stem = file_name.strip_suffix(".detjson").unwrap_or(file_name);
    path.with_file_name(format!("{stem}.{label}.detjson"))
}

fn required_json_string_field(root: &JsonValue, key: &str) -> Result<String, String> {
    root.get(key)
        .and_then(JsonValue::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| format!("E_PROOF_CERT_FIELD_MISSING {}", key))
}

fn required_json_bool_field(root: &JsonValue, key: &str) -> Result<bool, String> {
    root.get(key)
        .and_then(JsonValue::as_bool)
        .ok_or_else(|| format!("E_PROOF_CERT_FIELD_MISSING {}", key))
}

fn required_json_u64_field(root: &JsonValue, key: &str) -> Result<u64, String> {
    root.get(key)
        .and_then(JsonValue::as_u64)
        .ok_or_else(|| format!("E_PROOF_CERT_FIELD_MISSING {}", key))
}

fn write_pretty_json_file(path: &Path, value: &JsonValue) -> Result<String, String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| format!("E_PROOF_JSON {}", e))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("E_PROOF_WRITE {}", e))?;
    }
    let text_with_newline = format!("{text}\n");
    fs::write(path, &text_with_newline).map_err(|e| format!("E_PROOF_WRITE {}", e))?;
    Ok(text_with_newline)
}

fn build_runtime_proof_certificate_candidate(
    proof_path: &Path,
    proof_bytes: &[u8],
    proof_doc: &JsonValue,
) -> Result<JsonValue, String> {
    let verified = required_json_bool_field(proof_doc, "verified")?;
    let profile = if verified { "clean" } else { "abort" };
    let contract_diag_count = required_json_u64_field(proof_doc, "contract_diag_count")?;
    let source_proof_schema = required_json_string_field(proof_doc, "schema")?;
    let source_proof_kind = required_json_string_field(proof_doc, "kind")?;
    let canonical_body_hash = required_json_string_field(proof_doc, "canonical_body_hash")?;
    let proof_runtime_hash = required_json_string_field(proof_doc, "proof_runtime_hash")?;
    let solver_translation_hash = required_json_string_field(proof_doc, "solver_translation_hash")?;
    let state_hash = required_json_string_field(proof_doc, "state_hash")?;
    let trace_hash = required_json_string_field(proof_doc, "trace_hash")?;
    let proof_subject_hash = format!("sha256:{}", super::detjson::sha256_hex(proof_bytes));
    Ok(json!({
        "schema": "ddn.proof_certificate_v1_runtime_candidate.v1",
        "source_proof_path": proof_path.to_string_lossy().replace('\\', "/"),
        "source_proof_schema": source_proof_schema,
        "source_proof_kind": source_proof_kind,
        "profile": profile,
        "cert_manifest_schema": "ddn.cert_manifest.v1",
        "cert_algo": "sha256-proto",
        "verified": verified,
        "contract_diag_count": contract_diag_count,
        "proof_subject_hash": proof_subject_hash,
        "canonical_body_hash": canonical_body_hash,
        "proof_runtime_hash": proof_runtime_hash,
        "solver_translation_hash": solver_translation_hash,
        "state_hash": state_hash,
        "trace_hash": trace_hash,
    }))
}

fn build_runtime_proof_certificate_draft_artifact(
    proof_path: &Path,
    candidate: &JsonValue,
) -> Result<JsonValue, String> {
    let profile = required_json_string_field(candidate, "profile")?;
    let shared_shell = json!({
        "source_proof_schema": required_json_string_field(candidate, "source_proof_schema")?,
        "source_proof_kind": required_json_string_field(candidate, "source_proof_kind")?,
        "cert_manifest_schema": required_json_string_field(candidate, "cert_manifest_schema")?,
        "cert_algo": required_json_string_field(candidate, "cert_algo")?,
        "proof_runtime_hash": required_json_string_field(candidate, "proof_runtime_hash")?,
        "solver_translation_hash": required_json_string_field(candidate, "solver_translation_hash")?,
    });
    let state_delta = json!({
        "verified": required_json_bool_field(candidate, "verified")?,
        "contract_diag_count": required_json_u64_field(candidate, "contract_diag_count")?,
        "proof_subject_hash": required_json_string_field(candidate, "proof_subject_hash")?,
        "canonical_body_hash": required_json_string_field(candidate, "canonical_body_hash")?,
        "state_hash": required_json_string_field(candidate, "state_hash")?,
        "trace_hash": required_json_string_field(candidate, "trace_hash")?,
    });
    Ok(json!({
        "schema": "ddn.proof_certificate_v1_runtime_draft_artifact.v1",
        "source_proof_path": proof_path.to_string_lossy().replace('\\', "/"),
        "profile": profile,
        "shared_shell_key_count": 6,
        "state_delta_key_count": 6,
        "candidate_manifest": candidate,
        "shared_shell": shared_shell,
        "state_delta": state_delta,
    }))
}

fn build_runtime_proof_certificate_signed_bundle(
    proof_path: &Path,
    candidate: &JsonValue,
    artifact: &JsonValue,
    cert_manifest: &JsonValue,
) -> Result<JsonValue, String> {
    Ok(json!({
        "schema": "ddn.proof_certificate_v1.v1",
        "source_proof_path": proof_path.to_string_lossy().replace('\\', "/"),
        "source_proof_schema": required_json_string_field(candidate, "source_proof_schema")?,
        "source_proof_kind": required_json_string_field(candidate, "source_proof_kind")?,
        "profile": required_json_string_field(candidate, "profile")?,
        "cert_manifest_schema": required_json_string_field(candidate, "cert_manifest_schema")?,
        "cert_algo": required_json_string_field(candidate, "cert_algo")?,
        "verified": required_json_bool_field(candidate, "verified")?,
        "contract_diag_count": required_json_u64_field(candidate, "contract_diag_count")?,
        "proof_subject_hash": required_json_string_field(candidate, "proof_subject_hash")?,
        "canonical_body_hash": required_json_string_field(candidate, "canonical_body_hash")?,
        "proof_runtime_hash": required_json_string_field(candidate, "proof_runtime_hash")?,
        "solver_translation_hash": required_json_string_field(candidate, "solver_translation_hash")?,
        "state_hash": required_json_string_field(candidate, "state_hash")?,
        "trace_hash": required_json_string_field(candidate, "trace_hash")?,
        "cert_pubkey": required_json_string_field(cert_manifest, "pubkey")?,
        "cert_signature": required_json_string_field(cert_manifest, "signature")?,
        "cert_manifest": cert_manifest,
        "runtime_candidate": candidate,
        "runtime_draft_artifact": artifact,
    }))
}

fn write_runtime_proof_certificate_sidecars(
    proof_path: &Path,
    proof_text: &str,
    proof_doc: &JsonValue,
    proof_cert_key: Option<&Path>,
) -> Result<(), String> {
    let candidate =
        build_runtime_proof_certificate_candidate(proof_path, proof_text.as_bytes(), proof_doc)?;
    let artifact = build_runtime_proof_certificate_draft_artifact(proof_path, &candidate)?;
    let artifact_path =
        sidecar_proof_output_path(proof_path, "proof_certificate_v1_draft_artifact");
    let candidate_path = sidecar_proof_output_path(proof_path, "proof_certificate_v1_candidate");
    write_pretty_json_file(&artifact_path, &artifact)?;
    write_pretty_json_file(&candidate_path, &candidate)?;
    if let Some(key_path) = proof_cert_key {
        let cert_manifest =
            cert::build_signed_manifest_json(proof_path, proof_text.as_bytes(), key_path)?;
        let cert_manifest_path = sidecar_proof_output_path(proof_path, "cert_manifest");
        let bundle = build_runtime_proof_certificate_signed_bundle(
            proof_path,
            &candidate,
            &artifact,
            &cert_manifest,
        )?;
        let bundle_path = sidecar_proof_output_path(proof_path, "proof_certificate_v1");
        write_pretty_json_file(&cert_manifest_path, &cert_manifest)?;
        write_pretty_json_file(&bundle_path, &bundle)?;
    }
    Ok(())
}

fn write_proof_detjson(
    path: &Path,
    entry: &str,
    seed: u64,
    ticks: u64,
    source: &str,
    parse_mode: ParseMode,
    state_hash: &str,
    trace_hash: &str,
    output: &EvalOutput,
    proof_cert_key: Option<&Path>,
) -> Result<(), String> {
    let root = build_proof_detjson_with_mode(
        entry,
        seed,
        ticks,
        source,
        parse_mode,
        state_hash,
        trace_hash,
        output,
        None,
    )?;
    let text = write_pretty_json_file(path, &root)?;
    write_runtime_proof_certificate_sidecars(path, &text, &root, proof_cert_key)?;
    Ok(())
}

fn write_failed_proof_detjson(
    path: &Path,
    entry: &str,
    seed: u64,
    ticks: u64,
    source: &str,
    parse_mode: ParseMode,
    error: &RunError,
    output: &EvalOutput,
    proof_cert_key: Option<&Path>,
) -> Result<(), String> {
    let mut state_hash = hash::state_hash(&output.state);
    if let Some(override_hash) = maybe_override_state_hash(&output.state) {
        state_hash = override_hash;
    }
    let trace_hash = hash::trace_hash(&output.trace, source, &state_hash, ticks, seed);
    let root = build_proof_detjson_with_mode(
        entry,
        seed,
        ticks,
        source,
        parse_mode,
        &state_hash,
        &trace_hash,
        output,
        Some(error),
    )?;
    let text = write_pretty_json_file(path, &root)?;
    write_runtime_proof_certificate_sidecars(path, &text, &root, proof_cert_key)?;
    Ok(())
}

fn write_trace_json(
    path: &Path,
    trace: &Trace,
    state_hash: &str,
    trace_hash: &str,
) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::lexer::Lexer;
    use crate::lang::parser::Parser;
    use crate::lang::span::Span;

    struct CaptureEmitter {
        out: Vec<String>,
        err: Vec<String>,
    }

    impl CaptureEmitter {
        fn new() -> Self {
            Self {
                out: Vec::new(),
                err: Vec::new(),
            }
        }
    }

    impl RunEmitSink for CaptureEmitter {
        fn out(&mut self, line: &str) {
            self.out.push(line.to_string());
        }

        fn err(&mut self, line: &str) {
            self.err.push(line.to_string());
        }
    }

    fn parse_program_for_test(source: &str) -> Program {
        let (program, _) =
            parse_program_for_runtime(source).expect("parse program for runtime");
        program
    }

    fn default_run_options() -> RunOptions {
        RunOptions {
            diag_jsonl: None,
            diag_report_out: None,
            repro_json: None,
            trace_json: None,
            proof_out: None,
            proof_cert_key: None,
            geoul_out: None,
            geoul_record_out: None,
            latency_madi: 0,
            run_command: None,
            init_state: Vec::new(),
            init_state_files: Vec::new(),
            trace_tier: TraceTier::Off,
            age_target: None,
            lang_mode: None,
            bogae_mode: None,
            bogae_codec: BogaeCodec::Bdl1,
            bogae_out: None,
            cmd_policy: CmdPolicyConfig::none(),
            overlay: OverlayConfig::empty(),
            bogae_cache_log: false,
            bogae_live: false,
            console_config: ConsoleRenderConfig::default(),
            until_gameover: false,
            gameover_key: "게임끝".to_string(),
            sam_path: None,
            record_sam_path: None,
            sam_live: None,
            sam_live_host: "127.0.0.1".to_string(),
            sam_live_port: 0,
            madi_hz: None,
            open_mode: None,
            open_log: None,
            open_bundle: None,
            bogae_skin: None,
            no_open: false,
            unsafe_open: false,
            run_manifest: None,
            artifact_pins: Vec::new(),
        }
    }

    fn matic_seed_source() -> &'static str {
        r#"
매틱:움직씨 = {
  살림.t <- 1.
}.
"#
    }

    #[test]
    fn input_latency_queue_passthrough_when_zero() {
        let mut queue = InputLatencyQueue::new(0);
        let frame = OpenInputFrame::new(0b0010, 0b0010, 0);
        let (applied, meta) = queue.schedule_and_take(0, frame);
        assert_eq!(applied, frame);
        assert_eq!(
            meta,
            Some(InputLatencyApplied {
                accept_madi: 0,
                target_madi: 0,
                late: false,
                dropped: false,
            })
        );
    }

    #[test]
    fn input_latency_queue_delays_frames_until_target_madi() {
        let mut queue = InputLatencyQueue::new(2);
        let zero = OpenInputFrame::new(0, 0, 0);
        let f0 = OpenInputFrame::new(0b0010, 0b0010, 0);
        let f1 = OpenInputFrame::new(0b0000, 0, 0b0010);
        let f2 = OpenInputFrame::new(0b0001, 0b0001, 0);

        let (applied0, meta0) = queue.schedule_and_take(0, f0);
        assert_eq!(applied0, zero);
        assert_eq!(meta0, None);
        let (applied1, meta1) = queue.schedule_and_take(1, f1);
        assert_eq!(applied1, zero);
        assert_eq!(meta1, None);
        let (applied2, meta2) = queue.schedule_and_take(2, f2);
        assert_eq!(applied2, f0);
        assert_eq!(
            meta2,
            Some(InputLatencyApplied {
                accept_madi: 0,
                target_madi: 2,
                late: false,
                dropped: false,
            })
        );
    }

    #[test]
    fn input_latency_queue_marks_late_when_applied_after_target_madi() {
        let mut queue = InputLatencyQueue::new(2);
        let f0 = OpenInputFrame::new(0b0010, 0b0010, 0);
        let f1 = OpenInputFrame::new(0b0001, 0b0001, 0);
        let zero = OpenInputFrame::new(0, 0, 0);
        let _ = queue.schedule_and_take(0, f0);
        let (applied, meta) = queue.schedule_and_take(5, f1);
        assert_eq!(applied, zero);
        assert_eq!(
            meta,
            Some(InputLatencyApplied {
                accept_madi: 0,
                target_madi: 2,
                late: true,
                dropped: true,
            })
        );
    }

    #[test]
    fn strict_mode_rejects_matic_entry_without_compat_flag() {
        let tokens = Lexer::tokenize(matic_seed_source()).expect("tokenize");
        let default_root = Parser::default_root_for_source(matic_seed_source());
        let err = Parser::parse_with_default_root_mode(tokens, default_root, ParseMode::Strict)
            .expect_err("strict");
        assert_eq!(err.code(), "E_LANG_COMPAT_MATIC_ENTRY_DISABLED");
    }

    #[test]
    fn strict_mode_rejects_matic_entry_even_with_compat_flag() {
        let project_policy = ProjectPolicy {
            age_target: None,
            det_tier: None,
            trace_tier: None,
            lang_mode: None,
            detmath_seal_hash: None,
            nuri_lock_hash: None,
        };
        let mode = resolve_lang_mode(None, &project_policy).expect("lang mode");
        let tokens = Lexer::tokenize(matic_seed_source()).expect("tokenize");
        let default_root = Parser::default_root_for_source(matic_seed_source());
        let err =
            Parser::parse_with_default_root_mode(tokens, default_root, mode).expect_err("hard-cut");
        assert_eq!(err.code(), "E_LANG_COMPAT_MATIC_ENTRY_DISABLED");
    }

    #[test]
    fn run_file_rejects_matic_entry_without_compat_flag() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_matic_entry_strict_{nonce}.ddn"));
        fs::write(&path, matic_seed_source()).expect("write source");
        let mut emitter = CaptureEmitter::new();
        let err = run_file_with_emitter(
            &path,
            Some(MadiLimit::Finite(1)),
            0,
            default_run_options(),
            &mut emitter,
        )
        .expect_err("strict run must fail");
        assert!(err.contains("E_LANG_COMPAT_MATIC_ENTRY_DISABLED"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn parse_maegim_step_split_conflict_maps_to_specific_cli_diagnostic() {
        let err = ParseError::MaegimStepSplitConflict {
            span: Span::new(7, 3, 7, 12),
        };
        assert_eq!(parse_line(&err), 7);
        assert_eq!(parse_col(&err), 3);
        assert_eq!(
            parse_message(&err),
            "`매김`에서 `간격`과 `분할수`는 동시에 사용할 수 없습니다".to_string()
        );
        assert_eq!(
            RunError::Parse(err).code(),
            "E_PARSE_MAEGIM_STEP_SPLIT_CONFLICT"
        );
    }

    #[test]
    fn parse_maegim_nested_section_unsupported_maps_to_specific_cli_diagnostic() {
        let err = ParseError::MaegimNestedSectionUnsupported {
            section: "실험".to_string(),
            span: Span::new(8, 5, 8, 9),
        };
        assert_eq!(parse_line(&err), 8);
        assert_eq!(parse_col(&err), 5);
        assert_eq!(
            parse_message(&err),
            "`매김` 중첩 섹션은 `가늠`/`갈래`만 허용됩니다: 실험".to_string()
        );
        assert_eq!(
            RunError::Parse(err).code(),
            "E_PARSE_MAEGIM_NESTED_SECTION_UNSUPPORTED"
        );
    }

    #[test]
    fn parse_maegim_nested_field_unsupported_maps_to_specific_cli_diagnostic() {
        let err = ParseError::MaegimNestedFieldUnsupported {
            section: "가늠".to_string(),
            field: "최소값".to_string(),
            span: Span::new(9, 7, 9, 11),
        };
        assert_eq!(parse_line(&err), 9);
        assert_eq!(parse_col(&err), 7);
        assert_eq!(
            parse_message(&err),
            "`매김` 섹션 `가늠`에서 지원하지 않는 항목입니다: 최소값".to_string()
        );
        assert_eq!(
            RunError::Parse(err).code(),
            "E_PARSE_MAEGIM_NESTED_FIELD_UNSUPPORTED"
        );
    }

    #[test]
    fn parse_hook_every_n_madi_interval_invalid_maps_to_specific_cli_diagnostic() {
        let err = ParseError::HookEveryNMadiIntervalInvalid {
            span: Span::new(6, 2, 6, 3),
        };
        assert_eq!(parse_line(&err), 6);
        assert_eq!(parse_col(&err), 2);
        assert_eq!(
            parse_message(&err),
            "(N마디)마다에서 N은 양의 정수여야 합니다".to_string()
        );
        assert_eq!(
            RunError::Parse(err).code(),
            "E_PARSE_HOOK_EVERY_N_MADI_INTERVAL_INVALID"
        );
    }

    #[test]
    fn parse_hook_every_n_madi_unit_unsupported_maps_to_specific_cli_diagnostic() {
        let err = ParseError::HookEveryNMadiUnitUnsupported {
            unit: "foo".to_string(),
            span: Span::new(7, 4, 7, 5),
        };
        assert_eq!(parse_line(&err), 7);
        assert_eq!(parse_col(&err), 4);
        assert_eq!(
            parse_message(&err),
            "(N마디)마다에서 단위는 `마디`만 허용됩니다: foo".to_string()
        );
        assert_eq!(
            RunError::Parse(err).code(),
            "E_PARSE_HOOK_EVERY_N_MADI_UNIT_UNSUPPORTED"
        );
    }

    #[test]
    fn parse_hook_every_n_madi_suffix_unsupported_maps_to_specific_cli_diagnostic() {
        let err = ParseError::HookEveryNMadiSuffixUnsupported {
            suffix: "할때".to_string(),
            span: Span::new(8, 8, 8, 9),
        };
        assert_eq!(parse_line(&err), 8);
        assert_eq!(parse_col(&err), 8);
        assert_eq!(
            parse_message(&err),
            "(N마디) 훅 접미는 `마다`만 허용됩니다: 할때".to_string()
        );
        assert_eq!(
            RunError::Parse(err).code(),
            "E_PARSE_HOOK_EVERY_N_MADI_SUFFIX_UNSUPPORTED"
        );
    }

    #[test]
    fn parse_receive_outside_imja_maps_to_specific_cli_diagnostic() {
        let err = ParseError::ReceiveOutsideImja {
            span: Span::new(5, 7, 5, 20),
        };
        assert_eq!(parse_line(&err), 5);
        assert_eq!(parse_col(&err), 7);
        assert_eq!(
            parse_message(&err),
            "`받으면` 훅은 `임자` 본문 안에서만 사용할 수 있습니다".to_string()
        );
        assert_eq!(RunError::Parse(err).code(), "E_RECEIVE_OUTSIDE_IMJA");
    }

    #[test]
    fn diagnostic_failure_maps_to_sfc_runtime_error() {
        let failure = DiagnosticFailure {
            code: "E_SFC_IDENTITY_VIOLATION".to_string(),
            tick: 9,
            name: "SFC 항등식".to_string(),
            delta: "2".to_string(),
            threshold: "0.01".to_string(),
            span: Span::new(3, 1, 3, 20),
        };
        let err = diagnostic_failure_to_runtime_error(&failure);
        assert_eq!(err.code(), "E_SFC_IDENTITY_VIOLATION");
    }

    #[test]
    fn write_diagnostic_report_serializes_v0_schema() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_diag_report_{nonce}.detjson"));
        let diagnostics = vec![DiagnosticRecord {
            tick: 3,
            name: "거시↔미시".to_string(),
            lhs: "1".to_string(),
            rhs: "1".to_string(),
            delta: "0".to_string(),
            threshold: "0.01".to_string(),
            result: "수렴".to_string(),
            error_code: None,
        }];
        write_diagnostic_report(&path, 42, 7, &diagnostics).expect("write report");
        let text = fs::read_to_string(&path).expect("read report");
        assert!(text.contains("\"schema\":\"ddn.diagnostic_report.v0\""));
        assert!(text.contains("\"seed\":42"));
        assert!(text.contains("\"tick\":7"));
        assert!(text.contains("\"tick\":3"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_latency_schedule_diag_writes_expected_fields() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_latency_diag_{nonce}.jsonl"));
        append_latency_schedule_diag(
            &path,
            "main.ddn",
            7,
            InputLatencyApplied {
                accept_madi: 3,
                target_madi: 5,
                late: true,
                dropped: true,
            },
        )
        .expect("write latency diag");
        let text = fs::read_to_string(&path).expect("read latency diag");
        assert!(text.contains("\"event\":\"latency_schedule\""));
        assert!(text.contains("\"accept_madi\":3"));
        assert!(text.contains("\"target_madi\":5"));
        assert!(text.contains("\"applied_madi\":7"));
        assert!(text.contains("\"late\":true"));
        assert!(text.contains("\"dropped\":true"));
        assert!(text.contains("\"drop_policy\":\"late_drop\""));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_records_age1_hook_event_with_same_fault_id() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_diag_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Post,
            mode: ContractMode::Alert,
            message: "보장하고 조건이 실패했습니다".to_string(),
            span: Span::new(1, 3, 1, 8),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "{ 0 }인것 다짐하고(알림)\n",
            &events,
            SeulgiHookPolicy::Record,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"code\":\"W_CONTRACT_POST\""));
        assert!(lines[0].contains("\"fault_id\":\"contract:post:main.ddn:1:3:0\""));
        assert!(lines[1].contains("\"event_kind\":\"hook\""));
        assert!(lines[1].contains("\"hook_name\":\"슬기.계약위반\""));
        assert!(lines[1].contains("\"hook_policy\":\"기록\""));
        assert!(lines[1].contains("\"hook_input_ref\":\"contract:post:main.ddn:1:3:0\""));
        assert!(lines[1].contains("\"hook_input\":{"));
        assert!(lines[1].contains("\"code\":\"W_CONTRACT_POST\""));
        assert!(lines[1].contains("\"file\":\"main.ddn\""));
        assert!(lines[1].contains("\"contract_kind\":\"post\""));
        assert!(lines[1].contains("\"mode\":\"알림\""));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_can_emit_execute_hook_policy() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_policy_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(1, 4, 1, 9),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "{ 0 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[1].contains("\"hook_policy\":\"실행\""));
        assert!(lines[2].contains("\"event_kind\":\"hook_result\""));
        assert!(lines[2].contains("\"executor\":\"builtin.seulgi.contract.v1\""));
        assert!(lines[2].contains("\"suggestion_count\":2"));
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"patch_version\":\"0.1-draft\""));
        assert!(lines[2].contains("\"anchor\":\"{ 0 }인것 바탕으로(알림)\""));
        assert!(lines[2].contains("\"after\":[\"{ 1 }인것 바탕으로(알림)\"]"));
        assert!(lines[2].contains("전제 조건이 참이 되도록 입력값과 초기 상태를 먼저 점검하세요."));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_emits_patch_for_false_constant_compare() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_compare_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(1, 4, 1, 14),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "{ 1 == 2 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"anchor\":\"{ 1 == 2 }인것 바탕으로(알림)\""));
        assert!(lines[2].contains("\"after\":[\"{ 참 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_emits_patch_for_false_constant_relational_compare() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_rel_compare_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(1, 4, 1, 13),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "{ (1 < 0) }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"anchor\":\"{ (1 < 0) }인것 바탕으로(알림)\""));
        assert!(lines[2].contains("\"after\":[\"{ 참 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_emits_patch_for_observed_variable_relaxation() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_observed_relax_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 10),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "x <- 5.\n{ x > 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"anchor\":\"{ x > 10 }인것 바탕으로(알림)\""));
        assert!(lines[2].contains("\"after\":[\"{ x >= 5 }인것 바탕으로(알림)\"]"));
        assert!(lines[2].contains("관측된 값 기준으로 계약 경계를 완화"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_emits_patch_for_observed_variable_equality() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_observed_eq_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "x <- 5.\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"anchor\":\"{ x == 10 }인것 바탕으로(알림)\""));
        assert!(lines[2].contains("\"after\":[\"{ x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_parenthesized_observed_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_observed_paren_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "x <- (5).\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_parenthesized_negative_observed_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_neg_paren_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "x <- (-5).\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ x == -5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_plain_negative_observed_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_neg_plain_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "x <- -5.\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ x == -5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn normalize_observed_assignment_target_strips_type_and_root_prefixes() {
        assert_eq!(normalize_observed_assignment_target("x:수"), "x");
        assert_eq!(normalize_observed_assignment_target("살림.x"), "x");
        assert_eq!(normalize_observed_assignment_target("바탕.x"), "x");
        assert_eq!(
            normalize_observed_assignment_target("바탕.공.속도.x:수"),
            "공.속도.x"
        );
        assert_eq!(
            normalize_observed_assignment_target("((살림.공.속도.x:수))"),
            "공.속도.x"
        );
    }

    #[test]
    fn parse_observed_const_int_expr_supports_arithmetic_and_rejects_non_constant() {
        assert_eq!(parse_observed_const_int_expr("2 + 3"), Some(5));
        assert_eq!(
            parse_observed_const_int_expr("((2 + 3) * 4 - (6 / 3))"),
            Some(18)
        );
        assert_eq!(parse_observed_const_int_expr("-(2 + 3)"), Some(-5));
        assert_eq!(parse_observed_const_int_expr("2 + y"), None);
        assert_eq!(parse_observed_const_int_expr("1 / 0"), None);
    }

    #[test]
    fn find_last_constant_int_assignment_uses_normalized_typed_root_targets() {
        let source = "바탕.공.속도.x:수 <- +5.\n";
        assert_eq!(
            find_last_constant_int_assignment(source, 1, "공.속도.x"),
            Some(5)
        );
    }

    #[test]
    fn append_contract_diags_execute_hook_uses_most_recent_observed_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_observed_recent_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(3, 4, 3, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "x <- 1.\nx <- 7.\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ x == 7 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn observed_assignment_affects_target_matches_exact_and_ancestor_only() {
        assert!(observed_assignment_affects_target("공.속도.x", "공.속도.x"));
        assert!(observed_assignment_affects_target("공", "공.속도.x"));
        assert!(observed_assignment_affects_target("공.속도", "공.속도.x"));
        assert!(!observed_assignment_affects_target("공.다른", "공.속도.x"));
        assert!(!observed_assignment_affects_target(
            "공.속도.x.세부",
            "공.속도.x"
        ));
    }

    #[test]
    fn observed_assignment_affects_target_does_not_treat_root_binding_as_parent_of_bare_leaf() {
        assert!(!observed_assignment_affects_target("살림", "x"));
        assert!(!observed_assignment_affects_target("바탕", "x"));
        assert!(!observed_assignment_affects_target("살림", "공.속도.x"));
        assert!(!observed_assignment_affects_target("바탕", "공.속도.x"));
    }

    #[test]
    fn find_last_constant_int_assignment_ignores_root_object_reassignment_for_bare_leaf_target() {
        let source = "살림.x <- 5.\n살림 <- (\"x\", 7) 짝맞춤.\n";
        assert_eq!(find_last_constant_int_assignment(source, 2, "x"), Some(5));
    }

    #[test]
    fn find_last_constant_int_assignment_ignores_descendant_assignment() {
        let source = "살림.공.속도.x <- 5.\n살림.공.속도.x.세부 <- 7.\n";
        assert_eq!(
            find_last_constant_int_assignment(source, 2, "공.속도.x"),
            Some(5)
        );
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_unary_plus_observed_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_observed_plus_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "x <- +5.\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_parenthesized_unary_plus_observed_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_plus_paren_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "x <- (+5).\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_typed_observed_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_observed_typed_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "x:수 <- +5.\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_default_root_observed_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_observed_root_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.x <- +5.\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_default_root_contract_target() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_contract_root_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 16),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.x <- +5.\n{ 살림.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 살림.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_stale_default_root_contract_target() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_stale_root_target_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(4, 4, 4, 16),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.y <- +5.\n살림.x <- 5.\n살림.x <- y.\n{ 살림.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_cross_root_contract_target_from_sallim_to_batang() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_contract_cross_sb_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 16),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.x <- +5.\n{ 바탕.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 바탕.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_cross_root_nested_contract_target_from_sallim_to_batang(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_contract_cross_nested_sb_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(3, 4, 3, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n살림.공.속도.x <- +5.\n{ 바탕.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 바탕.공.속도.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_cross_root_nested_contract_target_after_sallim_parent_overwrite(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_cross_nested_sallim_parent_overwrite_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(5, 4, 5, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n살림.공.속도.x <- 5.\n살림.공 <- (\"속도\", (\"x\", 7) 짝맞춤) 짝맞춤.\n살림.y <- +5.\n{ 바탕.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_keeps_cross_root_nested_contract_target_after_sallim_sibling_overwrite(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_cross_nested_sallim_sibling_overwrite_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(5, 4, 5, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n살림.공.속도.x <- 5.\n살림.공.다른 <- 7.\n살림.y <- +5.\n{ 바탕.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 바탕.공.속도.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_stale_cross_root_nested_contract_target_after_sallim_indirect_assignment(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_cross_nested_sallim_stale_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(5, 4, 5, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n살림.y <- +5.\n살림.공.속도.x <- 5.\n살림.공.속도.x <- y.\n{ 바탕.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_nested_default_root_observed_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_root_nested_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 18),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.공.속도.x <- +5.\n{ 공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 공.속도.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_nested_default_root_contract_target() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_contract_root_nested_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(3, 4, 3, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n살림.공.속도.x <- +5.\n{ 살림.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 살림.공.속도.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_stale_nested_default_root_contract_target() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_stale_root_nested_target_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(5, 4, 5, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n살림.y <- +5.\n살림.공.속도.x <- 5.\n살림.공.속도.x <- y.\n{ 살림.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_rooted_nested_contract_target_after_parent_overwrite(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_rooted_nested_parent_overwrite_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(5, 4, 5, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n살림.공.속도.x <- 5.\n살림.공 <- (\"속도\", (\"x\", 7) 짝맞춤) 짝맞춤.\n살림.y <- +5.\n{ 살림.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_cross_root_contract_target_from_batang_to_sallim() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_contract_cross_bs_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 16),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "바탕.x <- +5.\n{ 살림.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 살림.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_cross_root_nested_contract_target_from_batang_to_sallim(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_contract_cross_nested_bs_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(3, 4, 3, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "바탕.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n바탕.공.속도.x <- +5.\n{ 살림.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 살림.공.속도.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_cross_root_nested_contract_target_after_batang_parent_overwrite(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_cross_nested_batang_parent_overwrite_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(5, 4, 5, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "바탕.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n바탕.공.속도.x <- 5.\n바탕.공 <- (\"속도\", (\"x\", 7) 짝맞춤) 짝맞춤.\n살림.y <- +5.\n{ 살림.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_keeps_cross_root_nested_contract_target_after_batang_sibling_overwrite(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_cross_nested_batang_sibling_overwrite_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(5, 4, 5, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "바탕.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n바탕.공.속도.x <- 5.\n바탕.공.다른 <- 7.\n살림.y <- +5.\n{ 살림.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 살림.공.속도.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_stale_cross_root_nested_contract_target_after_batang_indirect_assignment(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_cross_nested_batang_stale_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(5, 4, 5, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "바탕.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n살림.y <- +5.\n바탕.공.속도.x <- 5.\n바탕.공.속도.x <- y.\n{ 살림.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_explicit_batang_observed_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_observed_batang_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "바탕.x <- +5.\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_explicit_batang_contract_target() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_contract_batang_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 16),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "바탕.x <- +5.\n{ 바탕.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 바탕.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_stale_explicit_batang_contract_target() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_stale_batang_target_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(4, 4, 4, 16),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.y <- +5.\n바탕.x <- 5.\n바탕.x <- y.\n{ 바탕.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_nested_explicit_batang_contract_target() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_contract_batang_nested_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(3, 4, 3, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "바탕.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n바탕.공.속도.x <- +5.\n{ 바탕.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 바탕.공.속도.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_stale_nested_explicit_batang_contract_target() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_stale_batang_nested_target_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(5, 4, 5, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "바탕.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n살림.y <- +5.\n바탕.공.속도.x <- 5.\n바탕.공.속도.x <- y.\n{ 바탕.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_rooted_batang_nested_contract_target_after_parent_overwrite(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_rooted_batang_nested_parent_overwrite_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(5, 4, 5, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "바탕.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n바탕.공.속도.x <- 5.\n바탕.공 <- (\"속도\", (\"x\", 7) 짝맞춤) 짝맞춤.\n살림.y <- +5.\n{ 바탕.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_nested_batang_observed_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_batang_nested_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(3, 4, 3, 18),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "바탕.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n바탕.공.속도.x <- +5.\n{ 공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 공.속도.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_patch_for_unrelated_observed_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_unrelated_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.y <- +5.\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_patch_for_indirect_observed_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_indirect_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(3, 4, 3, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.y <- +5.\nx <- y.\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_stale_constant_before_newer_indirect_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_observed_stale_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(4, 4, 4, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.y <- +5.\nx <- 5.\nx <- y.\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_newer_expr_assignment_before_contract() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("teul_contract_hook_observed_expr_{nonce}.jsonl"));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(3, 4, 3, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "x <- 5.\nx <- 2 + 3.\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_stale_batang_constant_before_newer_indirect_assignment(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_batang_stale_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(4, 4, 4, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.y <- +5.\n바탕.x <- 5.\n바탕.x <- y.\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_parenthesized_expr_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_paren_expr_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(2, 4, 2, 11),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "x <- (2 + 3).\n{ x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_stale_nested_constant_before_newer_indirect_assignment(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_nested_stale_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(5, 4, 5, 18),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.y <- +5.\n살림.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n살림.공.속도.x <- 5.\n살림.공.속도.x <- y.\n{ 공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_reads_nested_parenthesized_expr_assignment() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_nested_expr_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(4, 4, 4, 18),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n살림.공.속도.x <- 5.\n살림.공.속도.x <- (2 + 3).\n{ 공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 공.속도.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_stale_nested_constant_before_parent_overwrite() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_nested_parent_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(4, 4, 4, 18),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n살림.공.속도.x <- 5.\n살림.공 <- (\"속도\", (\"x\", 7) 짝맞춤) 짝맞춤.\n살림.y <- +5.\n{ 공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_skips_stale_batang_nested_constant_before_parent_overwrite(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_batang_parent_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(4, 4, 4, 18),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "바탕.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n바탕.공.속도.x <- 5.\n바탕.공 <- (\"속도\", (\"x\", 7) 짝맞춤) 짝맞춤.\n살림.y <- +5.\n{ 공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":0"));
        assert!(lines[2].contains("\"patch_candidates\":[]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_keeps_patch_after_nested_sibling_overwrite() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_nested_sibling_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(4, 4, 4, 18),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n살림.공.속도.x <- 5.\n살림.공.다른 <- 7.\n{ 공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 공.속도.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_keeps_rooted_nested_contract_target_after_sibling_overwrite(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_rooted_nested_sibling_contract_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(5, 4, 5, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "살림.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n살림.공.속도.x <- 5.\n살림.공.다른 <- 7.\n살림.y <- +5.\n{ 살림.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 살림.공.속도.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_keeps_patch_after_batang_nested_sibling_overwrite() {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_observed_batang_sibling_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(4, 4, 4, 18),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "바탕.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n바탕.공.속도.x <- 5.\n바탕.공.다른 <- 7.\n{ 공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 공.속도.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn append_contract_diags_execute_hook_keeps_rooted_batang_nested_contract_target_after_sibling_overwrite(
    ) {
        let mut path = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "teul_contract_hook_rooted_batang_nested_sibling_contract_{nonce}.jsonl"
        ));
        let events = vec![ContractDiag {
            kind: ContractKind::Pre,
            mode: ContractMode::Alert,
            message: "전제하에 조건이 실패했습니다".to_string(),
            span: Span::new(5, 4, 5, 22),
        }];
        append_contract_diags(
            &path,
            "main.ddn",
            "바탕.공 <- (\"속도\", (\"x\", 0) 짝맞춤) 짝맞춤.\n바탕.공.속도.x <- 5.\n바탕.공.다른 <- 7.\n살림.y <- +5.\n{ 바탕.공.속도.x == 10 }인것 바탕으로(알림)\n",
            &events,
            SeulgiHookPolicy::Execute,
        )
        .expect("write contract diags");
        let text = fs::read_to_string(&path).expect("read contract diag report");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].contains("\"patch_candidate_count\":1"));
        assert!(lines[2].contains("\"after\":[\"{ 바탕.공.속도.x == 5 }인것 바탕으로(알림)\"]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn build_proof_detjson_marks_clean_run_as_verified() {
        let source = "#이름: proof_clean\nx <- 1.\n";
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
        };
        let doc = build_proof_detjson(
            "sample.ddn",
            7,
            1,
            source,
            "blake3:state",
            "blake3:trace",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["schema"], "ddn.proof.detjson.v0");
        assert_eq!(doc["verified"], true);
        assert_eq!(doc["contract_diag_count"], 0);
        assert_eq!(doc["diagnostic_failure_count"], 0);
        assert_eq!(
            doc["canonical_body_hash"],
            JsonValue::String(canonical_body_hash(source).expect("canon hash").0)
        );
        assert_eq!(doc["canonical_body_hash_method"], "canon");
        assert_eq!(
            doc["solver_translation_hash"],
            JsonValue::String(json_sha256(&doc["solver_translation"]).expect("solver hash"))
        );
    }

    #[test]
    fn build_proof_detjson_serializes_contract_violation_summary() {
        let source = "x <- 1.\n";
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: vec![ContractDiag {
                kind: ContractKind::Pre,
                mode: ContractMode::Abort,
                message: "전제하에 조건이 실패했습니다".to_string(),
                span: Span::new(3, 2, 3, 7),
            }],
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
        };
        let doc = build_proof_detjson(
            "sample_abort.ddn",
            9,
            1,
            source,
            "blake3:state2",
            "blake3:trace2",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["verified"], false);
        assert_eq!(doc["contract_diag_count"], 1);
        assert_eq!(doc["contract_diags"][0]["kind"], "pre");
        assert_eq!(doc["contract_diags"][0]["mode"], "abort");
        assert_eq!(doc["contract_diags"][0]["line"], 3);
        assert_eq!(doc["contract_diags"][0]["col"], 2);
    }

    #[test]
    fn build_proof_detjson_includes_solver_translation_summary() {
        let source = r#"
n 이 자연수 낱낱에 대해 {
  없음.
}.
x 가 실수 중 하나가 {
  없음.
}.
y 가 정수 중 딱 하나가 {
  없음.
}.
고르기:
{ 1 == 1 } 인 경우 {
  없음.
}
모든 경우 다룸.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
        };
        let doc = build_proof_detjson(
            "proof_surface.ddn",
            1,
            1,
            source,
            "blake3:state3",
            "blake3:trace3",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(
            doc["solver_translation"]["schema"],
            "ddn.proof.solver_translation.v0"
        );
        assert_eq!(doc["solver_translation"]["quantifier_count"], 3);
        assert_eq!(doc["solver_translation"]["case_analysis_count"], 1);
        assert_eq!(doc["solver_translation"]["solver_open_count"], 0);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][0]["quantifier"],
            "forall"
        );
        assert_eq!(doc["solver_translation"]["items"][1]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][1]["quantifier"],
            "exists"
        );
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][2]["quantifier"],
            "exists_unique"
        );
        assert_eq!(
            doc["solver_translation"]["items"][3]["kind"],
            "case_analysis"
        );
        assert_eq!(
            doc["solver_translation"]["items"][3]["completion"],
            "exhaustive"
        );
        assert_eq!(doc["canonical_body_hash_method"], "ast_fallback");
        assert_eq!(
            doc["solver_translation_hash"],
            JsonValue::String(json_sha256(&doc["solver_translation"]).expect("solver hash"))
        );
    }

    #[test]
    fn build_proof_detjson_collects_solver_open_summary() {
        let source = r#"
증명:셈씨 = {
  y 가 정수 중 딱 하나가 {
    없음.
  }.

  판정 <- ((질의="forall n. n = n")) 열림.풀이.확인.
  판정 보여주기.
}.
() 증명.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
        };
        let doc = build_proof_detjson(
            "proof_solver.ddn",
            1,
            1,
            source,
            "blake3:state4",
            "blake3:trace4",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][0]["quantifier"],
            "exists_unique"
        );
        assert_eq!(doc["solver_translation"]["solver_open_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][1]["kind"], "solver_open");
        assert_eq!(doc["solver_translation"]["items"][1]["operation"], "check");
        assert_eq!(
            doc["solver_translation"]["items"][1]["query"],
            JsonValue::String("forall n. n = n".to_string())
        );
    }

    #[test]
    fn build_proof_detjson_collects_case_analysis_with_solver_open_summary() {
        let source = r#"
증명:셈씨 = {
  y 가 정수 중 딱 하나가 {
    없음.
  }.

  고르기:
  { 1 == 1 } 인 경우 {
    판정 <- ((질의="forall n. n = n")) 열림.풀이.확인.
    판정 보여주기.
  }
  모든 경우 다룸.
}.
() 증명.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
        };
        let doc = build_proof_detjson(
            "proof_case_solver.ddn",
            1,
            1,
            source,
            "blake3:state4_case",
            "blake3:trace4_case",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][0]["quantifier"],
            "exists_unique"
        );
        assert_eq!(doc["solver_translation"]["case_analysis_count"], 1);
        assert_eq!(
            doc["solver_translation"]["items"][1]["kind"],
            "case_analysis"
        );
        assert_eq!(
            doc["solver_translation"]["items"][1]["completion"],
            "exhaustive"
        );
        assert_eq!(doc["solver_translation"]["solver_open_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "solver_open");
        assert_eq!(doc["solver_translation"]["items"][2]["operation"], "check");
        assert_eq!(
            doc["solver_translation"]["items"][2]["query"],
            JsonValue::String("forall n. n = n".to_string())
        );
    }

    #[test]
    fn build_proof_detjson_collects_case_analysis_with_solver_search_summary() {
        let source = r#"
증명:셈씨 = {
  y 가 정수 중 딱 하나가 {
    없음.
  }.

  고르기:
  { 1 == 1 } 인 경우 {
    반례 <- ((질의="forall n. n >= 0")) 반례찾기.
    해 <- ((질의="x * x = 4")) 해찾기.
    반례 보여주기.
    해 보여주기.
  }
  모든 경우 다룸.
}.
() 증명.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
        };
        let doc = build_proof_detjson(
            "proof_case_solver_search.ddn",
            1,
            1,
            source,
            "blake3:state4_case_search",
            "blake3:trace4_case_search",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][0]["quantifier"],
            "exists_unique"
        );
        assert_eq!(doc["solver_translation"]["case_analysis_count"], 1);
        assert_eq!(
            doc["solver_translation"]["items"][1]["kind"],
            "case_analysis"
        );
        assert_eq!(
            doc["solver_translation"]["items"][1]["completion"],
            "exhaustive"
        );
        assert_eq!(doc["solver_translation"]["solver_open_count"], 2);
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][2]["operation"],
            JsonValue::String("counterexample".to_string())
        );
        assert_eq!(
            doc["solver_translation"]["items"][3]["kind"],
            JsonValue::String("solver_open".to_string())
        );
        assert_eq!(
            doc["solver_translation"]["items"][3]["operation"],
            JsonValue::String("solve".to_string())
        );
    }

    #[test]
    fn build_proof_detjson_collects_case_analysis_with_solver_open_and_search_summary() {
        let source = r#"
증명:셈씨 = {
  y 가 정수 중 딱 하나가 {
    없음.
  }.

  고르기:
  { 1 == 1 } 인 경우 {
    판정 <- ((질의="forall n. n = n")) 열림.풀이.확인.
    반례 <- ((질의="forall n. n >= 0")) 반례찾기.
    해 <- ((질의="x * x = 4")) 해찾기.
    판정 보여주기.
    반례 보여주기.
    해 보여주기.
  }
  모든 경우 다룸.
}.
() 증명.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
        };
        let doc = build_proof_detjson(
            "proof_case_solver_open_search.ddn",
            1,
            1,
            source,
            "blake3:state4_case_open_search",
            "blake3:trace4_case_open_search",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["case_analysis_count"], 1);
        assert_eq!(doc["solver_translation"]["solver_open_count"], 3);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][0]["quantifier"],
            "exists_unique"
        );
        assert_eq!(
            doc["solver_translation"]["items"][1]["kind"],
            "case_analysis"
        );
        assert_eq!(
            doc["solver_translation"]["items"][1]["completion"],
            "exhaustive"
        );
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "solver_open");
        assert_eq!(doc["solver_translation"]["items"][2]["operation"], "check");
        assert_eq!(doc["solver_translation"]["items"][3]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][3]["operation"],
            JsonValue::String("counterexample".to_string())
        );
        assert_eq!(doc["solver_translation"]["items"][4]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][4]["operation"],
            JsonValue::String("solve".to_string())
        );
    }

    #[test]
    fn build_proof_detjson_collects_exists_case_analysis_with_solver_open_and_search_summary() {
        let source = r#"
증명:셈씨 = {
  x 가 실수 중 하나가 {
    없음.
  }.

  고르기:
  { 1 == 1 } 인 경우 {
    판정 <- ((질의="forall n. n = n")) 열림.풀이.확인.
    반례 <- ((질의="forall n. n >= 0")) 반례찾기.
    해 <- ((질의="x * x = 4")) 해찾기.
    판정 보여주기.
    반례 보여주기.
    해 보여주기.
  }
  모든 경우 다룸.
}.
() 증명.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
        };
        let doc = build_proof_detjson(
            "proof_case_exists_solver_open_search.ddn",
            1,
            1,
            source,
            "blake3:state4_case_exists_open_search",
            "blake3:trace4_case_exists_open_search",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["case_analysis_count"], 1);
        assert_eq!(doc["solver_translation"]["solver_open_count"], 3);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][0]["quantifier"],
            "exists"
        );
        assert_eq!(
            doc["solver_translation"]["items"][1]["kind"],
            "case_analysis"
        );
        assert_eq!(
            doc["solver_translation"]["items"][1]["completion"],
            "exhaustive"
        );
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "solver_open");
        assert_eq!(doc["solver_translation"]["items"][2]["operation"], "check");
        assert_eq!(doc["solver_translation"]["items"][3]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][3]["operation"],
            JsonValue::String("counterexample".to_string())
        );
        assert_eq!(doc["solver_translation"]["items"][4]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][4]["operation"],
            JsonValue::String("solve".to_string())
        );
    }

    #[test]
    fn build_proof_detjson_collects_forall_case_analysis_with_solver_open_and_search_summary() {
        let source = r#"
증명:셈씨 = {
  n 이 자연수 낱낱에 대해 {
    없음.
  }.

  고르기:
  { 1 == 1 } 인 경우 {
    판정 <- ((질의="forall n. n = n")) 열림.풀이.확인.
    반례 <- ((질의="forall n. n >= 0")) 반례찾기.
    해 <- ((질의="x * x = 4")) 해찾기.
    판정 보여주기.
    반례 보여주기.
    해 보여주기.
  }
  모든 경우 다룸.
}.
() 증명.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
        };
        let doc = build_proof_detjson(
            "proof_case_forall_solver_open_search.ddn",
            1,
            1,
            source,
            "blake3:state4_case_forall_open_search",
            "blake3:trace4_case_forall_open_search",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["case_analysis_count"], 1);
        assert_eq!(doc["solver_translation"]["solver_open_count"], 3);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][0]["quantifier"],
            "forall"
        );
        assert_eq!(
            doc["solver_translation"]["items"][1]["kind"],
            "case_analysis"
        );
        assert_eq!(
            doc["solver_translation"]["items"][1]["completion"],
            "exhaustive"
        );
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "solver_open");
        assert_eq!(doc["solver_translation"]["items"][2]["operation"], "check");
        assert_eq!(doc["solver_translation"]["items"][3]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][3]["operation"],
            JsonValue::String("counterexample".to_string())
        );
        assert_eq!(doc["solver_translation"]["items"][4]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][4]["operation"],
            JsonValue::String("solve".to_string())
        );
    }

    #[test]
    fn build_proof_detjson_collects_else_case_analysis_with_solver_open_and_search_summary() {
        let source = r#"
증명:셈씨 = {
  y 가 정수 중 딱 하나가 {
    없음.
  }.

  고르기:
  { 1 == 2 } 인 경우 {
    "wrong" 보여주기.
  }
  그밖의 경우 {
    판정 <- ((질의="forall n. n = n")) 열림.풀이.확인.
    반례 <- ((질의="forall n. n >= 0")) 반례찾기.
    해 <- ((질의="x * x = 4")) 해찾기.
    판정 보여주기.
    반례 보여주기.
    해 보여주기.
  }.
}.
() 증명.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
        };
        let doc = build_proof_detjson(
            "proof_case_else_solver_open_search.ddn",
            1,
            1,
            source,
            "blake3:state4_case_else_open_search",
            "blake3:trace4_case_else_open_search",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["case_analysis_count"], 1);
        assert_eq!(doc["solver_translation"]["solver_open_count"], 3);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][0]["quantifier"],
            "exists_unique"
        );
        assert_eq!(
            doc["solver_translation"]["items"][1]["kind"],
            "case_analysis"
        );
        assert_eq!(doc["solver_translation"]["items"][1]["completion"], "else");
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "solver_open");
        assert_eq!(doc["solver_translation"]["items"][2]["operation"], "check");
        assert_eq!(doc["solver_translation"]["items"][3]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][3]["operation"],
            JsonValue::String("counterexample".to_string())
        );
        assert_eq!(doc["solver_translation"]["items"][4]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][4]["operation"],
            JsonValue::String("solve".to_string())
        );
    }

    #[test]
    fn build_proof_detjson_collects_solver_search_summary() {
        let source = r#"
증명:셈씨 = {
  y 가 정수 중 딱 하나가 { 없음. }.
  반례 <- ((질의="forall n. n >= 0")) 반례찾기.
  해 <- ((질의="x * x = 4")) 해찾기.
  반례 보여주기.
  해 보여주기.
}.
() 증명.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
        };
        let doc = build_proof_detjson(
            "proof_solver_search.ddn",
            1,
            1,
            source,
            "blake3:state5",
            "blake3:trace5",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][0]["quantifier"],
            JsonValue::String("exists_unique".to_string())
        );
        assert_eq!(doc["solver_translation"]["solver_open_count"], 2);
        assert_eq!(doc["solver_translation"]["items"][1]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][1]["operation"],
            JsonValue::String("counterexample".to_string())
        );
        assert_eq!(
            doc["solver_translation"]["items"][2]["operation"],
            JsonValue::String("solve".to_string())
        );
    }

    #[test]
    fn build_proof_detjson_collects_immediate_proof_block() {
        let source = r#"
자연수_제곱_0이상 밝히기 {
  n 이 자연수 낱낱에 대해 {
    없음.
  }.
}.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
        };
        let doc = build_proof_detjson(
            "proof_immediate.ddn",
            1,
            1,
            source,
            "blake3:state6",
            "blake3:trace6",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "proof_block");
        assert_eq!(
            doc["solver_translation"]["items"][0]["name"],
            "자연수_제곱_0이상"
        );
    }

    #[test]
    fn build_proof_detjson_collects_assertion_check_summary() {
        let source = r#"
거리_0이상 <- 세움{
  { 거리 >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
검사 밝히기 {
  (세움=거리_0이상, 값들=(거리=3)) 살피기.
  y 가 정수 중 딱 하나가 {
    없음.
  }.
  ((질의="forall n. n >= 0")) 반례찾기.
  ((질의="forall n. n = n")) 열림.풀이.확인.
}.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: vec![
                ProofRuntimeEvent::ProofBlock {
                    tick: 1,
                    name: "검사".to_string(),
                    result: "성공".to_string(),
                    error_code: None,
                },
                ProofRuntimeEvent::ProofCheck {
                    tick: 1,
                    target: "세움{거리 >= 0}".to_string(),
                    binding_count: 1,
                    passed: true,
                    error_code: None,
                    span: Span::new(7, 3, 7, 20),
                },
                ProofRuntimeEvent::SolverSearch {
                    tick: 1,
                    operation: "counterexample".to_string(),
                    query: "forall n. n >= 0".to_string(),
                    found: Some(false),
                    value: None,
                    error_code: None,
                    span: Span::new(11, 3, 11, 35),
                },
                ProofRuntimeEvent::SolverCheck {
                    tick: 1,
                    query: "forall n. n = n".to_string(),
                    satisfied: Some(true),
                    error_code: None,
                    span: Span::new(12, 3, 12, 30),
                },
            ],
        };
        let doc = build_proof_detjson(
            "proof_assertion_check.ddn",
            1,
            1,
            source,
            "blake3:state7",
            "blake3:trace7",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["proof_check_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][1]["kind"], "proof_check");
        assert_eq!(
            doc["solver_translation"]["items"][1]["target"],
            JsonValue::String("살림.거리_0이상".to_string())
        );
        assert_eq!(doc["solver_translation"]["items"][1]["binding_count"], 1);
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][2]["quantifier"],
            "exists_unique"
        );
        assert_eq!(doc["solver_translation"]["solver_open_count"], 2);
        assert_eq!(doc["solver_translation"]["items"][3]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][3]["operation"],
            JsonValue::String("counterexample".to_string())
        );
        assert_eq!(doc["solver_translation"]["items"][4]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][4]["operation"],
            JsonValue::String("check".to_string())
        );
        assert_eq!(doc["proof_runtime"]["proof_block_count"], 1);
        assert_eq!(doc["proof_runtime"]["proof_check_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_search_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_check_count"], 1);
        assert_eq!(doc["proof_runtime"]["items"][0]["kind"], "proof_block");
        assert_eq!(doc["proof_runtime"]["items"][1]["kind"], "proof_check");
        assert_eq!(doc["proof_runtime"]["items"][2]["kind"], "solver_search");
        assert_eq!(doc["proof_runtime"]["items"][3]["kind"], "solver_check");
    }

    #[test]
    fn build_proof_detjson_collects_assertion_check_with_solver_solve_summary() {
        let source = r#"
거리_0이상 <- 세움{
  { 거리 >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
검사 밝히기 {
  (세움=거리_0이상, 값들=(거리=3)) 살피기.
  y 가 정수 중 딱 하나가 {
    없음.
  }.
  ((질의="x * x = 4")) 해찾기.
}.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: vec![
                ProofRuntimeEvent::ProofBlock {
                    tick: 1,
                    name: "검사".to_string(),
                    result: "성공".to_string(),
                    error_code: None,
                },
                ProofRuntimeEvent::ProofCheck {
                    tick: 1,
                    target: "세움{거리 >= 0}".to_string(),
                    binding_count: 1,
                    passed: true,
                    error_code: None,
                    span: Span::new(7, 3, 7, 20),
                },
                ProofRuntimeEvent::SolverSearch {
                    tick: 1,
                    operation: "solve".to_string(),
                    query: "x * x = 4".to_string(),
                    found: Some(true),
                    value: Some("x=2".to_string()),
                    error_code: None,
                    span: Span::new(11, 3, 11, 21),
                },
            ],
        };
        let doc = build_proof_detjson(
            "proof_assertion_check_solve.ddn",
            1,
            1,
            source,
            "blake3:state7_solve",
            "blake3:trace7_solve",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["proof_check_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][1]["kind"], "proof_check");
        assert_eq!(
            doc["solver_translation"]["items"][1]["target"],
            JsonValue::String("살림.거리_0이상".to_string())
        );
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][2]["quantifier"],
            "exists_unique"
        );
        assert_eq!(doc["solver_translation"]["solver_open_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][3]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][3]["operation"],
            JsonValue::String("solve".to_string())
        );
        assert_eq!(doc["proof_runtime"]["proof_block_count"], 1);
        assert_eq!(doc["proof_runtime"]["proof_check_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_search_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_check_count"], 0);
        assert_eq!(doc["proof_runtime"]["items"][0]["kind"], "proof_block");
        assert_eq!(doc["proof_runtime"]["items"][1]["kind"], "proof_check");
        assert_eq!(doc["proof_runtime"]["items"][2]["kind"], "solver_search");
        assert_eq!(
            doc["proof_runtime"]["items"][2]["operation"],
            JsonValue::String("solve".to_string())
        );
        assert_eq!(
            doc["proof_runtime"]["items"][2]["found"],
            JsonValue::Bool(true)
        );
        assert_eq!(
            doc["proof_runtime"]["items"][2]["value"],
            JsonValue::String("x=2".to_string())
        );
    }

    #[test]
    fn build_proof_detjson_collects_assertion_check_with_case_analysis_summary() {
        let source = r#"
거리_0이상 <- 세움{
  { 거리 >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
검사 밝히기 {
  (세움=거리_0이상, 값들=(거리=3)) 살피기.
  y 가 정수 중 딱 하나가 {
    없음.
  }.
  고르기:
  { 1 == 1 } 인 경우 {
    없음.
  }
  모든 경우 다룸.
}.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: vec![
                ProofRuntimeEvent::ProofBlock {
                    tick: 1,
                    name: "검사".to_string(),
                    result: "성공".to_string(),
                    error_code: None,
                },
                ProofRuntimeEvent::ProofCheck {
                    tick: 1,
                    target: "세움{거리 >= 0}".to_string(),
                    binding_count: 1,
                    passed: true,
                    error_code: None,
                    span: Span::new(7, 3, 7, 20),
                },
            ],
        };
        let doc = build_proof_detjson(
            "proof_assertion_check_case.ddn",
            1,
            1,
            source,
            "blake3:state7_case",
            "blake3:trace7_case",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["proof_check_count"], 1);
        assert_eq!(doc["solver_translation"]["case_analysis_count"], 1);
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "proof_block");
        assert_eq!(doc["solver_translation"]["items"][1]["kind"], "proof_check");
        assert_eq!(
            doc["solver_translation"]["items"][1]["target"],
            JsonValue::String("살림.거리_0이상".to_string())
        );
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][2]["quantifier"],
            "exists_unique"
        );
        assert_eq!(
            doc["solver_translation"]["items"][3]["kind"],
            "case_analysis"
        );
        assert_eq!(
            doc["solver_translation"]["items"][3]["completion"],
            "exhaustive"
        );
        assert_eq!(doc["proof_runtime"]["proof_block_count"], 1);
        assert_eq!(doc["proof_runtime"]["proof_check_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_search_count"], 0);
        assert_eq!(doc["proof_runtime"]["solver_check_count"], 0);
        assert_eq!(doc["proof_runtime"]["items"][0]["kind"], "proof_block");
        assert_eq!(doc["proof_runtime"]["items"][1]["kind"], "proof_check");
    }

    #[test]
    fn build_proof_detjson_collects_assertion_check_with_case_analysis_and_solver_check_summary() {
        let source = r#"
거리_0이상 <- 세움{
  { 거리 >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
검사 밝히기 {
  (세움=거리_0이상, 값들=(거리=3)) 살피기.
  y 가 정수 중 딱 하나가 {
    없음.
  }.
  고르기:
  { 1 == 1 } 인 경우 {
    ((질의="forall n. n = n")) 열림.풀이.확인.
  }
  모든 경우 다룸.
}.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: vec![
                ProofRuntimeEvent::ProofBlock {
                    tick: 1,
                    name: "검사".to_string(),
                    result: "성공".to_string(),
                    error_code: None,
                },
                ProofRuntimeEvent::ProofCheck {
                    tick: 1,
                    target: "세움{거리 >= 0}".to_string(),
                    binding_count: 1,
                    passed: true,
                    error_code: None,
                    span: Span::new(7, 3, 7, 20),
                },
                ProofRuntimeEvent::SolverCheck {
                    tick: 1,
                    query: "forall n. n = n".to_string(),
                    satisfied: Some(true),
                    error_code: None,
                    span: Span::new(13, 5, 13, 32),
                },
            ],
        };
        let doc = build_proof_detjson(
            "proof_assertion_check_case_solver_open.ddn",
            1,
            1,
            source,
            "blake3:state7_case_check",
            "blake3:trace7_case_check",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["proof_check_count"], 1);
        assert_eq!(doc["solver_translation"]["case_analysis_count"], 1);
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["solver_open_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "proof_block");
        assert_eq!(doc["solver_translation"]["items"][1]["kind"], "proof_check");
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][3]["kind"],
            "case_analysis"
        );
        assert_eq!(doc["solver_translation"]["items"][4]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][4]["operation"],
            JsonValue::String("check".to_string())
        );
        assert_eq!(
            doc["solver_translation"]["items"][4]["query"],
            JsonValue::String("forall n. n = n".to_string())
        );
        assert_eq!(doc["proof_runtime"]["proof_block_count"], 1);
        assert_eq!(doc["proof_runtime"]["proof_check_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_search_count"], 0);
        assert_eq!(doc["proof_runtime"]["solver_check_count"], 1);
        assert_eq!(doc["proof_runtime"]["items"][0]["kind"], "proof_block");
        assert_eq!(doc["proof_runtime"]["items"][1]["kind"], "proof_check");
        assert_eq!(doc["proof_runtime"]["items"][2]["kind"], "solver_check");
    }

    #[test]
    fn build_proof_detjson_collects_assertion_check_with_case_analysis_and_solver_search_summary() {
        let source = r#"
거리_0이상 <- 세움{
  { 거리 >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
검사 밝히기 {
  (세움=거리_0이상, 값들=(거리=3)) 살피기.
  y 가 정수 중 딱 하나가 {
    없음.
  }.
  고르기:
  { 1 == 1 } 인 경우 {
    ((질의="forall n. n >= 0")) 반례찾기.
  }
  모든 경우 다룸.
}.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: vec![
                ProofRuntimeEvent::ProofBlock {
                    tick: 1,
                    name: "검사".to_string(),
                    result: "성공".to_string(),
                    error_code: None,
                },
                ProofRuntimeEvent::ProofCheck {
                    tick: 1,
                    target: "세움{거리 >= 0}".to_string(),
                    binding_count: 1,
                    passed: true,
                    error_code: None,
                    span: Span::new(7, 3, 7, 20),
                },
                ProofRuntimeEvent::SolverSearch {
                    tick: 1,
                    operation: "counterexample".to_string(),
                    query: "forall n. n >= 0".to_string(),
                    found: Some(false),
                    value: None,
                    error_code: None,
                    span: Span::new(13, 5, 13, 35),
                },
            ],
        };
        let doc = build_proof_detjson(
            "proof_assertion_check_case_solver_search.ddn",
            1,
            1,
            source,
            "blake3:state7_case_search",
            "blake3:trace7_case_search",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["proof_check_count"], 1);
        assert_eq!(doc["solver_translation"]["case_analysis_count"], 1);
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["solver_open_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "proof_block");
        assert_eq!(doc["solver_translation"]["items"][1]["kind"], "proof_check");
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][3]["kind"],
            "case_analysis"
        );
        assert_eq!(doc["solver_translation"]["items"][4]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][4]["operation"],
            JsonValue::String("counterexample".to_string())
        );
        assert_eq!(
            doc["solver_translation"]["items"][4]["query"],
            JsonValue::String("forall n. n >= 0".to_string())
        );
        assert_eq!(doc["proof_runtime"]["proof_block_count"], 1);
        assert_eq!(doc["proof_runtime"]["proof_check_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_search_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_check_count"], 0);
        assert_eq!(doc["proof_runtime"]["items"][0]["kind"], "proof_block");
        assert_eq!(doc["proof_runtime"]["items"][1]["kind"], "proof_check");
        assert_eq!(doc["proof_runtime"]["items"][2]["kind"], "solver_search");
        assert_eq!(
            doc["proof_runtime"]["items"][2]["operation"],
            JsonValue::String("counterexample".to_string())
        );
        assert_eq!(
            doc["proof_runtime"]["items"][2]["found"],
            JsonValue::Bool(false)
        );
    }

    #[test]
    fn build_proof_detjson_collects_assertion_check_with_case_analysis_and_solver_search_solve_summary(
    ) {
        let source = r#"
거리_0이상 <- 세움{
  { 거리 >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
검사 밝히기 {
  (세움=거리_0이상, 값들=(거리=3)) 살피기.
  y 가 정수 중 딱 하나가 {
    없음.
  }.
  고르기:
  { 1 == 1 } 인 경우 {
    ((질의="x * x = 4")) 해찾기.
  }
  모든 경우 다룸.
}.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: vec![
                ProofRuntimeEvent::ProofBlock {
                    tick: 1,
                    name: "검사".to_string(),
                    result: "성공".to_string(),
                    error_code: None,
                },
                ProofRuntimeEvent::ProofCheck {
                    tick: 1,
                    target: "세움{거리 >= 0}".to_string(),
                    binding_count: 1,
                    passed: true,
                    error_code: None,
                    span: Span::new(7, 3, 7, 20),
                },
                ProofRuntimeEvent::SolverSearch {
                    tick: 1,
                    operation: "solve".to_string(),
                    query: "x * x = 4".to_string(),
                    found: Some(true),
                    value: Some("x=2".to_string()),
                    error_code: None,
                    span: Span::new(13, 5, 13, 25),
                },
            ],
        };
        let doc = build_proof_detjson(
            "proof_assertion_check_case_solver_search_solve.ddn",
            1,
            1,
            source,
            "blake3:state7_case_search_solve",
            "blake3:trace7_case_search_solve",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["proof_check_count"], 1);
        assert_eq!(doc["solver_translation"]["case_analysis_count"], 1);
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["solver_open_count"], 1);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "proof_block");
        assert_eq!(doc["solver_translation"]["items"][1]["kind"], "proof_check");
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][3]["kind"],
            "case_analysis"
        );
        assert_eq!(doc["solver_translation"]["items"][4]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][4]["operation"],
            JsonValue::String("solve".to_string())
        );
        assert_eq!(
            doc["solver_translation"]["items"][4]["query"],
            JsonValue::String("x * x = 4".to_string())
        );
        assert_eq!(doc["proof_runtime"]["proof_block_count"], 1);
        assert_eq!(doc["proof_runtime"]["proof_check_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_search_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_check_count"], 0);
        assert_eq!(doc["proof_runtime"]["items"][0]["kind"], "proof_block");
        assert_eq!(doc["proof_runtime"]["items"][1]["kind"], "proof_check");
        assert_eq!(doc["proof_runtime"]["items"][2]["kind"], "solver_search");
        assert_eq!(
            doc["proof_runtime"]["items"][2]["operation"],
            JsonValue::String("solve".to_string())
        );
        assert_eq!(
            doc["proof_runtime"]["items"][2]["found"],
            JsonValue::Bool(true)
        );
        assert_eq!(
            doc["proof_runtime"]["items"][2]["value"],
            JsonValue::String("x=2".to_string())
        );
    }

    #[test]
    fn build_proof_detjson_collects_assertion_check_with_case_analysis_and_solver_open_search_summary(
    ) {
        let source = r#"
거리_0이상 <- 세움{
  { 거리 >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
검사 밝히기 {
  (세움=거리_0이상, 값들=(거리=3)) 살피기.
  y 가 정수 중 딱 하나가 {
    없음.
  }.
  고르기:
  { 1 == 1 } 인 경우 {
    ((질의="forall n. n = n", 결과=참)) 열림.풀이.확인.
    ((질의="forall n. n >= 0", 찾음=거짓)) 반례찾기.
    ((질의="x * x = 4", 찾음=참, 값="x=2")) 해찾기.
  }
  모든 경우 다룸.
}.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: vec![
                ProofRuntimeEvent::ProofCheck {
                    tick: 1,
                    target: "세움{거리 >= 0}".to_string(),
                    binding_count: 1,
                    passed: true,
                    error_code: None,
                    span: Span::new(8, 3, 8, 20),
                },
                ProofRuntimeEvent::SolverCheck {
                    tick: 1,
                    query: "forall n. n = n".to_string(),
                    satisfied: Some(true),
                    error_code: None,
                    span: Span::new(14, 5, 14, 34),
                },
                ProofRuntimeEvent::SolverSearch {
                    tick: 1,
                    operation: "counterexample".to_string(),
                    query: "forall n. n >= 0".to_string(),
                    found: Some(false),
                    value: None,
                    error_code: None,
                    span: Span::new(15, 5, 15, 37),
                },
                ProofRuntimeEvent::SolverSearch {
                    tick: 1,
                    operation: "solve".to_string(),
                    query: "x * x = 4".to_string(),
                    found: Some(true),
                    value: Some("x=2".to_string()),
                    error_code: None,
                    span: Span::new(16, 5, 16, 39),
                },
                ProofRuntimeEvent::ProofBlock {
                    tick: 1,
                    name: "검사".to_string(),
                    result: "성공".to_string(),
                    error_code: None,
                },
            ],
        };
        let doc = build_proof_detjson(
            "proof_assertion_check_case_solver_open_search.ddn",
            1,
            1,
            source,
            "blake3:state8_case_open_search",
            "blake3:trace8_case_open_search",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["proof_check_count"], 1);
        assert_eq!(doc["solver_translation"]["case_analysis_count"], 1);
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["solver_open_count"], 3);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "proof_block");
        assert_eq!(doc["solver_translation"]["items"][1]["kind"], "proof_check");
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][3]["kind"],
            "case_analysis"
        );
        assert_eq!(doc["solver_translation"]["items"][4]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][4]["operation"],
            JsonValue::String("check".to_string())
        );
        assert_eq!(doc["solver_translation"]["items"][5]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][5]["operation"],
            JsonValue::String("counterexample".to_string())
        );
        assert_eq!(doc["solver_translation"]["items"][6]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][6]["operation"],
            JsonValue::String("solve".to_string())
        );
        assert_eq!(doc["proof_runtime"]["proof_block_count"], 1);
        assert_eq!(doc["proof_runtime"]["proof_check_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_check_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_search_count"], 2);
        assert_eq!(doc["proof_runtime"]["items"][0]["kind"], "proof_check");
        assert_eq!(doc["proof_runtime"]["items"][1]["kind"], "solver_check");
        assert_eq!(doc["proof_runtime"]["items"][2]["kind"], "solver_search");
        assert_eq!(
            doc["proof_runtime"]["items"][2]["operation"],
            JsonValue::String("counterexample".to_string())
        );
        assert_eq!(doc["proof_runtime"]["items"][3]["kind"], "solver_search");
        assert_eq!(
            doc["proof_runtime"]["items"][3]["operation"],
            JsonValue::String("solve".to_string())
        );
        assert_eq!(doc["proof_runtime"]["items"][4]["kind"], "proof_block");
    }

    #[test]
    fn build_proof_detjson_collects_assertion_check_with_else_case_analysis_and_solver_open_search_summary(
    ) {
        let source = r#"
거리_0이상 <- 세움{
  { 거리 >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
검사 밝히기 {
  (세움=거리_0이상, 값들=(거리=3)) 살피기.
  y 가 정수 중 딱 하나가 {
    없음.
  }.
  고르기:
  { 1 == 2 } 인 경우 {
    없음.
  }
  그밖의 경우 {
    ((질의="forall n. n = n", 결과=참)) 열림.풀이.확인.
    ((질의="forall n. n >= 0", 찾음=거짓)) 반례찾기.
    ((질의="x * x = 4", 찾음=참, 값="x=2")) 해찾기.
  }.
}.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: vec![
                ProofRuntimeEvent::ProofCheck {
                    tick: 1,
                    target: "세움{거리 >= 0}".to_string(),
                    binding_count: 1,
                    passed: true,
                    error_code: None,
                    span: Span::new(8, 3, 8, 20),
                },
                ProofRuntimeEvent::SolverCheck {
                    tick: 1,
                    query: "forall n. n = n".to_string(),
                    satisfied: Some(true),
                    error_code: None,
                    span: Span::new(18, 5, 18, 34),
                },
                ProofRuntimeEvent::SolverSearch {
                    tick: 1,
                    operation: "counterexample".to_string(),
                    query: "forall n. n >= 0".to_string(),
                    found: Some(false),
                    value: None,
                    error_code: None,
                    span: Span::new(19, 5, 19, 37),
                },
                ProofRuntimeEvent::SolverSearch {
                    tick: 1,
                    operation: "solve".to_string(),
                    query: "x * x = 4".to_string(),
                    found: Some(true),
                    value: Some("x=2".to_string()),
                    error_code: None,
                    span: Span::new(20, 5, 20, 39),
                },
                ProofRuntimeEvent::ProofBlock {
                    tick: 1,
                    name: "검사".to_string(),
                    result: "성공".to_string(),
                    error_code: None,
                },
            ],
        };
        let doc = build_proof_detjson(
            "proof_assertion_check_case_else_solver_open_search.ddn",
            1,
            1,
            source,
            "blake3:state9_case_else_open_search",
            "blake3:trace9_case_else_open_search",
            &output,
            None,
        )
        .expect("proof");
        assert_eq!(doc["solver_translation"]["proof_check_count"], 1);
        assert_eq!(doc["solver_translation"]["case_analysis_count"], 1);
        assert_eq!(doc["solver_translation"]["quantifier_count"], 1);
        assert_eq!(doc["solver_translation"]["solver_open_count"], 3);
        assert_eq!(doc["solver_translation"]["items"][0]["kind"], "proof_block");
        assert_eq!(doc["solver_translation"]["items"][1]["kind"], "proof_check");
        assert_eq!(doc["solver_translation"]["items"][2]["kind"], "quantifier");
        assert_eq!(
            doc["solver_translation"]["items"][3]["kind"],
            "case_analysis"
        );
        assert_eq!(
            doc["solver_translation"]["items"][3]["completion"],
            JsonValue::String("else".to_string())
        );
        assert_eq!(doc["solver_translation"]["items"][4]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][4]["operation"],
            JsonValue::String("check".to_string())
        );
        assert_eq!(doc["solver_translation"]["items"][5]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][5]["operation"],
            JsonValue::String("counterexample".to_string())
        );
        assert_eq!(doc["solver_translation"]["items"][6]["kind"], "solver_open");
        assert_eq!(
            doc["solver_translation"]["items"][6]["operation"],
            JsonValue::String("solve".to_string())
        );
        assert_eq!(doc["proof_runtime"]["proof_block_count"], 1);
        assert_eq!(doc["proof_runtime"]["proof_check_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_check_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_search_count"], 2);
        assert_eq!(doc["proof_runtime"]["items"][0]["kind"], "proof_check");
        assert_eq!(doc["proof_runtime"]["items"][1]["kind"], "solver_check");
        assert_eq!(doc["proof_runtime"]["items"][2]["kind"], "solver_search");
        assert_eq!(
            doc["proof_runtime"]["items"][2]["operation"],
            JsonValue::String("counterexample".to_string())
        );
        assert_eq!(doc["proof_runtime"]["items"][3]["kind"], "solver_search");
        assert_eq!(
            doc["proof_runtime"]["items"][3]["operation"],
            JsonValue::String("solve".to_string())
        );
        assert_eq!(doc["proof_runtime"]["items"][4]["kind"], "proof_block");
    }

    #[test]
    fn build_proof_detjson_marks_runtime_failure_as_unverified() {
        let source = r#"
검사 밝히기 {
  ((질의="forall n. n = n")) 열림.풀이.확인.
}.
"#;
        let output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: vec![
                ProofRuntimeEvent::SolverCheck {
                    tick: 0,
                    query: "forall n. n = n".to_string(),
                    satisfied: None,
                    error_code: Some("E_OPEN_DENIED".to_string()),
                    span: Span::new(3, 3, 3, 28),
                },
                ProofRuntimeEvent::ProofBlock {
                    tick: 0,
                    name: "검사".to_string(),
                    result: "실패".to_string(),
                    error_code: Some("E_OPEN_DENIED".to_string()),
                },
            ],
        };
        let doc = build_proof_detjson(
            "proof_runtime_fail.ddn",
            1,
            0,
            source,
            "blake3:state8",
            "blake3:trace8",
            &output,
            Some(&RunError::Runtime(RuntimeError::OpenDenied {
                open_kind: "solver".to_string(),
                span: Span::new(3, 3, 3, 28),
            })),
        )
        .expect("proof");
        assert_eq!(doc["verified"], false);
        assert_eq!(doc["runtime_error"]["code"], "E_OPEN_DENIED");
        assert_eq!(doc["runtime_error"]["line"], 3);
        assert_eq!(doc["runtime_error"]["col"], 3);
        assert_eq!(doc["proof_runtime"]["proof_block_count"], 1);
        assert_eq!(doc["proof_runtime"]["solver_check_count"], 1);
    }

    #[test]
    fn build_proof_detjson_runtime_failure_keeps_pre_failure_state_hash() {
        let source = r#"
검사 밝히기 {
  ((질의="forall n. n = n")) 열림.풀이.확인.
}.
"#;
        let mut state = State::new();
        state.set(
            Key::new("값".to_string()),
            Value::Num(Quantity::new(Fixed64::from_int(1), UnitDim::zero())),
        );
        let state_hash = crate::core::hash::state_hash(&state);
        let output = EvalOutput {
            state,
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: vec![
                ProofRuntimeEvent::SolverCheck {
                    tick: 0,
                    query: "forall n. n = n".to_string(),
                    satisfied: None,
                    error_code: Some("E_OPEN_REPLAY_MISS".to_string()),
                    span: Span::new(3, 3, 3, 28),
                },
                ProofRuntimeEvent::ProofBlock {
                    tick: 0,
                    name: "검사".to_string(),
                    result: "실패".to_string(),
                    error_code: Some("E_OPEN_REPLAY_MISS".to_string()),
                },
            ],
        };
        let doc = build_proof_detjson(
            "proof_runtime_fail_state.ddn",
            1,
            0,
            source,
            &state_hash,
            "blake3:trace-state",
            &output,
            Some(&RunError::Runtime(RuntimeError::OpenReplayMissing {
                open_kind: "solver".to_string(),
                site_id: "proof_runtime_fail_state.ddn:3:3".to_string(),
                key: "check:forall n. n = n".to_string(),
                span: Span::new(3, 3, 3, 28),
            })),
        )
        .expect("proof");
        assert_eq!(doc["verified"], false);
        assert_eq!(doc["state_hash"], JsonValue::String(state_hash));
        assert_eq!(doc["runtime_error"]["code"], "E_OPEN_REPLAY_MISS");
    }

    #[test]
    fn build_runtime_proof_certificate_candidate_tracks_profile_from_proof() {
        let source = "x <- 1.\n";
        let clean_output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
        };
        let clean_doc = build_proof_detjson(
            "proof_clean.ddn",
            1,
            1,
            source,
            "blake3:state-clean",
            "blake3:trace-clean",
            &clean_output,
            None,
        )
        .expect("clean proof");
        let clean_text = format!(
            "{}\n",
            serde_json::to_string_pretty(&clean_doc).expect("clean proof text")
        );
        let clean_candidate = build_runtime_proof_certificate_candidate(
            Path::new("build/tmp/clean.proof.detjson"),
            clean_text.as_bytes(),
            &clean_doc,
        )
        .expect("clean candidate");
        assert_eq!(
            clean_candidate["schema"],
            "ddn.proof_certificate_v1_runtime_candidate.v1"
        );
        assert_eq!(clean_candidate["profile"], "clean");
        assert_eq!(clean_candidate["verified"], true);
        assert_eq!(clean_candidate["contract_diag_count"], 0);
        assert_eq!(
            clean_candidate["proof_subject_hash"],
            JsonValue::String(format!(
                "sha256:{}",
                crate::cli::detjson::sha256_hex(clean_text.as_bytes())
            ))
        );
        assert_eq!(
            clean_candidate["canonical_body_hash"],
            clean_doc["canonical_body_hash"]
        );

        let abort_output = EvalOutput {
            state: State::new(),
            trace: Trace::new(),
            bogae_requested: false,
            contract_diags: vec![ContractDiag {
                kind: ContractKind::Post,
                mode: ContractMode::Abort,
                message: "조건이 실패했습니다".to_string(),
                span: Span::new(2, 1, 2, 5),
            }],
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
        };
        let abort_doc = build_proof_detjson(
            "proof_abort.ddn",
            1,
            1,
            source,
            "blake3:state-abort",
            "blake3:trace-abort",
            &abort_output,
            None,
        )
        .expect("abort proof");
        let abort_text = format!(
            "{}\n",
            serde_json::to_string_pretty(&abort_doc).expect("abort proof text")
        );
        let abort_candidate = build_runtime_proof_certificate_candidate(
            Path::new("build/tmp/abort.proof.detjson"),
            abort_text.as_bytes(),
            &abort_doc,
        )
        .expect("abort candidate");
        assert_eq!(abort_candidate["profile"], "abort");
        assert_eq!(abort_candidate["verified"], false);
        assert_eq!(abort_candidate["contract_diag_count"], 1);
        assert_eq!(
            abort_candidate["source_proof_kind"],
            JsonValue::String("run_contract_certificate_v1".to_string())
        );
    }

    #[test]
    fn build_runtime_proof_certificate_draft_artifact_splits_shell_and_state() {
        let candidate = json!({
            "schema": "ddn.proof_certificate_v1_runtime_candidate.v1",
            "source_proof_path": "build/tmp/proof.detjson",
            "source_proof_schema": "ddn.proof.detjson.v0",
            "source_proof_kind": "run_contract_certificate_v1",
            "profile": "clean",
            "cert_manifest_schema": "ddn.cert_manifest.v1",
            "cert_algo": "sha256-proto",
            "verified": true,
            "contract_diag_count": 0,
            "proof_subject_hash": "sha256:abc",
            "canonical_body_hash": "sha256:def",
            "proof_runtime_hash": "sha256:ghi",
            "solver_translation_hash": "sha256:jkl",
            "state_hash": "blake3:state",
            "trace_hash": "blake3:trace"
        });
        let artifact = build_runtime_proof_certificate_draft_artifact(
            Path::new("build/tmp/proof.detjson"),
            &candidate,
        )
        .expect("artifact");
        assert_eq!(
            artifact["schema"],
            "ddn.proof_certificate_v1_runtime_draft_artifact.v1"
        );
        assert_eq!(artifact["profile"], "clean");
        assert_eq!(artifact["shared_shell_key_count"], 6);
        assert_eq!(artifact["state_delta_key_count"], 6);
        assert_eq!(artifact["candidate_manifest"], candidate);
        assert_eq!(
            artifact["shared_shell"]["source_proof_schema"],
            "ddn.proof.detjson.v0"
        );
        assert_eq!(artifact["state_delta"]["verified"], true);
        assert_eq!(artifact["state_delta"]["proof_subject_hash"], "sha256:abc");
    }

    #[test]
    fn run_file_writes_proof_certificate_v1_sidecars() {
        let mut dir = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        dir.push(format!("teul_proof_certificate_v1_sidecars_{nonce}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let input_path = dir.join("input.ddn");
        let proof_path = dir.join("proof.detjson");
        let candidate_path =
            sidecar_proof_output_path(&proof_path, "proof_certificate_v1_candidate");
        let artifact_path =
            sidecar_proof_output_path(&proof_path, "proof_certificate_v1_draft_artifact");
        fs::write(&input_path, "x <- 1.\n").expect("write source");

        let mut emitter = CaptureEmitter::new();
        let mut options = default_run_options();
        options.proof_out = Some(proof_path.clone());
        run_file_with_emitter(
            &input_path,
            Some(MadiLimit::Finite(1)),
            0,
            options,
            &mut emitter,
        )
        .expect("run should write proof sidecars");
        assert!(emitter.err.is_empty());

        let proof_doc: JsonValue =
            serde_json::from_str(&fs::read_to_string(&proof_path).expect("read proof"))
                .expect("parse proof");
        let candidate_doc: JsonValue =
            serde_json::from_str(&fs::read_to_string(&candidate_path).expect("read candidate"))
                .expect("parse candidate");
        let artifact_doc: JsonValue =
            serde_json::from_str(&fs::read_to_string(&artifact_path).expect("read artifact"))
                .expect("parse artifact");
        assert_eq!(
            candidate_doc["schema"],
            "ddn.proof_certificate_v1_runtime_candidate.v1"
        );
        assert_eq!(
            artifact_doc["schema"],
            "ddn.proof_certificate_v1_runtime_draft_artifact.v1"
        );
        assert_eq!(candidate_doc["profile"], "clean");
        assert_eq!(candidate_doc["verified"], true);
        assert_eq!(candidate_doc["source_proof_schema"], proof_doc["schema"]);
        assert_eq!(artifact_doc["candidate_manifest"], candidate_doc);
        assert_eq!(artifact_doc["shared_shell_key_count"], 6);
        assert_eq!(artifact_doc["state_delta_key_count"], 6);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&proof_path);
        let _ = fs::remove_file(&candidate_path);
        let _ = fs::remove_file(&artifact_path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn run_file_writes_signed_proof_certificate_v1_bundle() {
        let mut dir = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        dir.push(format!("teul_proof_certificate_v1_signed_{nonce}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let input_path = dir.join("input.ddn");
        let proof_path = dir.join("proof.detjson");
        let key_dir = dir.join("cert");
        fs::write(&input_path, "x <- 1.\n").expect("write source");
        crate::cli::cert::run_keygen(&key_dir, Some("proof-certificate-v1-test")).expect("keygen");
        let private_key = key_dir.join("cert_private.key");
        let cert_manifest_path = sidecar_proof_output_path(&proof_path, "cert_manifest");
        let bundle_path = sidecar_proof_output_path(&proof_path, "proof_certificate_v1");

        let mut emitter = CaptureEmitter::new();
        let mut options = default_run_options();
        options.proof_out = Some(proof_path.clone());
        options.proof_cert_key = Some(private_key.clone());
        run_file_with_emitter(
            &input_path,
            Some(MadiLimit::Finite(1)),
            0,
            options,
            &mut emitter,
        )
        .expect("run should write signed proof certificate bundle");
        assert!(emitter.err.is_empty());
        crate::cli::cert::run_verify(&cert_manifest_path).expect("verify cert manifest");
        crate::cli::cert::run_verify_proof_certificate(&bundle_path, None)
            .expect("verify proof certificate bundle");

        let proof_doc: JsonValue =
            serde_json::from_str(&fs::read_to_string(&proof_path).expect("read proof"))
                .expect("parse proof");
        let cert_manifest_doc: JsonValue = serde_json::from_str(
            &fs::read_to_string(&cert_manifest_path).expect("read cert manifest"),
        )
        .expect("parse cert manifest");
        let bundle_doc: JsonValue =
            serde_json::from_str(&fs::read_to_string(&bundle_path).expect("read bundle"))
                .expect("parse bundle");
        assert_eq!(bundle_doc["schema"], "ddn.proof_certificate_v1.v1");
        assert_eq!(bundle_doc["profile"], "clean");
        assert_eq!(bundle_doc["verified"], true);
        assert_eq!(bundle_doc["contract_diag_count"], 0);
        assert_eq!(bundle_doc["cert_manifest"], cert_manifest_doc);
        assert_eq!(
            bundle_doc["proof_subject_hash"],
            cert_manifest_doc["subject_hash"]
        );
        assert_eq!(bundle_doc["source_proof_schema"], proof_doc["schema"]);
        assert_eq!(bundle_doc["source_proof_kind"], proof_doc["kind"]);
        assert_eq!(
            bundle_doc["runtime_candidate"]["proof_subject_hash"],
            cert_manifest_doc["subject_hash"]
        );
        assert_eq!(
            bundle_doc["runtime_draft_artifact"]["candidate_manifest"]["proof_subject_hash"],
            cert_manifest_doc["subject_hash"]
        );

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&proof_path);
        let _ = fs::remove_file(&cert_manifest_path);
        let _ = fs::remove_file(&bundle_path);
        let _ = fs::remove_file(key_dir.join("cert_private.key"));
        let _ = fs::remove_file(key_dir.join("cert_public.key"));
        let _ = fs::remove_dir(&key_dir);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn run_file_writes_signed_proof_certificate_v1_bundle_abort_profile() {
        let mut dir = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        dir.push(format!("teul_proof_certificate_v1_signed_abort_{nonce}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let input_path = dir.join("input_abort.ddn");
        let proof_path = dir.join("proof.detjson");
        let key_dir = dir.join("cert");
        fs::write(
            &input_path,
            "#이름: proof_abort\nx <- 1.\n{ 거짓 }인것 바탕으로(물림) 아니면 {\n}.\nx 보여주기.\n",
        )
        .expect("write source");
        crate::cli::cert::run_keygen(&key_dir, Some("proof-certificate-v1-abort-test"))
            .expect("keygen");
        let private_key = key_dir.join("cert_private.key");
        let cert_manifest_path = sidecar_proof_output_path(&proof_path, "cert_manifest");
        let bundle_path = sidecar_proof_output_path(&proof_path, "proof_certificate_v1");

        let mut emitter = CaptureEmitter::new();
        let mut options = default_run_options();
        options.proof_out = Some(proof_path.clone());
        options.proof_cert_key = Some(private_key.clone());
        run_file_with_emitter(
            &input_path,
            Some(MadiLimit::Finite(1)),
            0,
            options,
            &mut emitter,
        )
        .expect("run should write signed proof certificate bundle for abort profile");
        assert!(emitter.err.is_empty());
        crate::cli::cert::run_verify(&cert_manifest_path).expect("verify cert manifest");
        crate::cli::cert::run_verify_proof_certificate(&bundle_path, None)
            .expect("verify proof certificate bundle");

        let proof_doc: JsonValue =
            serde_json::from_str(&fs::read_to_string(&proof_path).expect("read proof"))
                .expect("parse proof");
        let cert_manifest_doc: JsonValue = serde_json::from_str(
            &fs::read_to_string(&cert_manifest_path).expect("read cert manifest"),
        )
        .expect("parse cert manifest");
        let bundle_doc: JsonValue =
            serde_json::from_str(&fs::read_to_string(&bundle_path).expect("read bundle"))
                .expect("parse bundle");
        assert_eq!(bundle_doc["schema"], "ddn.proof_certificate_v1.v1");
        assert_eq!(bundle_doc["profile"], "abort");
        assert_eq!(bundle_doc["verified"], false);
        assert_eq!(bundle_doc["contract_diag_count"], 1);
        assert_eq!(bundle_doc["cert_manifest"], cert_manifest_doc);
        assert_eq!(
            bundle_doc["proof_subject_hash"],
            cert_manifest_doc["subject_hash"]
        );
        assert_eq!(bundle_doc["source_proof_schema"], proof_doc["schema"]);
        assert_eq!(bundle_doc["source_proof_kind"], proof_doc["kind"]);
        assert_eq!(bundle_doc["runtime_candidate"]["profile"], "abort");
        assert_eq!(bundle_doc["runtime_candidate"]["verified"], false);
        assert_eq!(bundle_doc["runtime_candidate"]["contract_diag_count"], 1);
        assert_eq!(
            bundle_doc["runtime_candidate"]["proof_subject_hash"],
            cert_manifest_doc["subject_hash"]
        );
        assert_eq!(
            bundle_doc["runtime_draft_artifact"]["candidate_manifest"]["proof_subject_hash"],
            cert_manifest_doc["subject_hash"]
        );

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&proof_path);
        let _ = fs::remove_file(&cert_manifest_path);
        let _ = fs::remove_file(&bundle_path);
        let _ = fs::remove_file(key_dir.join("cert_private.key"));
        let _ = fs::remove_file(key_dir.join("cert_public.key"));
        let _ = fs::remove_dir(&key_dir);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn parse_open_allow_directives_accepts_neomeo_header() {
        let source = r#"
#너머 허용(난수, gpu)
채비 {
  x <- 1.
}.
"#;
        let allow = parse_open_allow_directives(source);
        assert_eq!(allow, vec!["rand".to_string(), "gpu".to_string()]);
    }

    #[test]
    fn extract_exec_policy_open_allow_collects_repeated_fields() {
        let source = r#"
실행정책 {
  열림허용: 시각.
  열림허용: 파일읽기.
  열림허용: 시각.
}.
"#;
        let program = parse_program_for_test(source);
        let allow = extract_exec_policy_open_allow(&program);
        assert_eq!(allow, vec!["clock".to_string(), "file_read".to_string()]);
    }

    #[test]
    fn extract_exec_policy_parses_general_allowed() {
        let source = r#"
실행정책 {
  실행모드: 일반.
  효과정책: 허용.
}.
"#;
        let program = parse_program_for_test(source);
        let extracted = extract_exec_policy(&program)
            .expect("extract ok")
            .expect("policy exists");
        let policy = extracted.policy;
        assert!(!extracted.strict_effect_ignored);
        assert_eq!(policy.mode, ExecMode::General);
        assert_eq!(policy.effect, EffectPolicy::Allowed);
        assert_eq!(policy.seulgi_hook, SeulgiHookPolicy::Record);
    }

    #[test]
    fn extract_exec_policy_ignores_open_allow_only_block() {
        let source = r#"
실행정책 {
  열림허용: 시각.
  열림허용: 파일읽기.
}.
"#;
        let program = parse_program_for_test(source);
        let extracted = extract_exec_policy(&program).expect("extract ok");
        assert!(extracted.is_none());
    }

    #[test]
    fn extract_exec_policy_strict_forces_effect_isolated() {
        let source = r#"
실행정책 {
  실행모드: 엄밀.
  효과정책: 허용.
}.
"#;
        let program = parse_program_for_test(source);
        let extracted = extract_exec_policy(&program)
            .expect("extract ok")
            .expect("policy exists");
        let policy = extracted.policy;
        assert!(extracted.strict_effect_ignored);
        assert_eq!(policy.mode, ExecMode::Strict);
        assert_eq!(policy.effect, EffectPolicy::Isolated);
        assert_eq!(policy.seulgi_hook, SeulgiHookPolicy::Record);
    }

    #[test]
    fn extract_exec_policy_parses_explicit_seulgi_hook_execute() {
        let source = r#"
실행정책 {
  슬기훅정책: 실행.
}.
"#;
        let program = parse_program_for_test(source);
        let extracted = extract_exec_policy(&program)
            .expect("extract ok")
            .expect("policy exists");
        assert_eq!(extracted.policy.seulgi_hook, SeulgiHookPolicy::Execute);
    }

    #[test]
    fn resolve_effective_open_mode_prioritizes_cli_then_policy() {
        let policy = OpenPolicy::new(OpenMode::Record, vec!["clock".to_string()], Vec::new());
        assert_eq!(
            resolve_effective_open_mode(Some(OpenMode::Deny), Some(&policy)),
            OpenMode::Deny
        );
        assert_eq!(
            resolve_effective_open_mode(None, Some(&policy)),
            OpenMode::Record
        );
        assert_eq!(resolve_effective_open_mode(None, None), OpenMode::Deny);
    }

    #[test]
    fn extract_exec_policy_rejects_duplicate_blocks() {
        let source = r#"
실행정책 {
  실행모드: 엄밀.
}.
실행정책 {
  실행모드: 일반.
}.
"#;
        let program = parse_program_for_test(source);
        let err = extract_exec_policy(&program).expect_err("duplicate must fail");
        assert!(err.contains("E_EXEC_POLICY_DUPLICATE"));
    }

    #[test]
    fn run_file_rejects_seulgi_hook_execute_before_age2() {
        let mut dir = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        dir.push(format!("teul_seulgi_hook_age1_{nonce}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("input.ddn");
        fs::write(&path, "실행정책 {\n  슬기훅정책: 실행.\n}.\n값 <- 1.\n").expect("write source");
        let mut emitter = CaptureEmitter::new();
        let mut options = default_run_options();
        options.age_target = Some("AGE1".to_string());
        let err =
            run_file_with_emitter(&path, Some(MadiLimit::Finite(1)), 0, options, &mut emitter)
                .expect_err("age1 must reject execute policy");
        assert!(err.contains("E_AGE_NOT_AVAILABLE"));
        assert!(err.contains("seulgi_hook"));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn run_file_execute_policy_age2_records_hook_result() {
        let mut dir = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        dir.push(format!("teul_seulgi_hook_age2_{nonce}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let project_path = dir.join("ddn.project.json");
        let input_path = dir.join("input.ddn");
        let diag_path = dir.join("geoul.diag.jsonl");
        fs::write(
            &project_path,
            "{\n  \"age_target\": \"AGE2\",\n  \"det_tier\": \"D-STRICT\",\n  \"trace_tier\": \"T-PATCH\"\n}\n",
        )
        .expect("write project policy");
        fs::write(
            &input_path,
            "실행정책 {\n  슬기훅정책: 실행.\n}.\n{ 0 }인것 바탕으로(알림)\n  아니면 { \"실패\" 보여주기. }\n  맞으면 { \"성공\" 보여주기. }.\n\"끝\" 보여주기.\n",
        )
        .expect("write source");
        let mut emitter = CaptureEmitter::new();
        let mut options = default_run_options();
        options.diag_jsonl = Some(diag_path.clone());
        run_file_with_emitter(
            &input_path,
            Some(MadiLimit::Finite(1)),
            0,
            options,
            &mut emitter,
        )
        .expect("age2 execute policy should record hook result");
        assert!(emitter.err.is_empty());
        let diag_text = fs::read_to_string(&diag_path).expect("read diag");
        assert!(diag_text.contains("\"hook_policy\":\"실행\""));
        assert!(diag_text.contains("\"event_kind\":\"hook_result\""));
        assert!(diag_text.contains("\"executor\":\"builtin.seulgi.contract.v1\""));
        assert!(diag_text.contains("\"patch_candidate_count\":1"));
        assert!(diag_text.contains("\"patch_version\":\"0.1-draft\""));
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&project_path);
        let _ = fs::remove_file(&diag_path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn run_file_latency_madi_records_latency_schedule_diag_event() {
        let mut dir = std::env::temp_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        dir.push(format!("teul_latency_diag_age2_{nonce}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let project_path = dir.join("ddn.project.json");
        let input_path = dir.join("input.ddn");
        let sam_path = dir.join("sam.input.bin");
        let diag_path = dir.join("geoul.diag.jsonl");
        fs::write(
            &project_path,
            "{\n  \"age_target\": \"AGE2\",\n  \"det_tier\": \"D-STRICT\",\n  \"trace_tier\": \"T-PATCH\"\n}\n",
        )
        .expect("write project policy");
        fs::write(
            &input_path,
            "살림.점수 <- 0.\n(매마디)마다 {\n  살림.점수 <- 살림.점수 + 샘.키보드.누르고있음.ArrowRight.\n}\n",
        )
        .expect("write source");
        let tape = InputTape {
            madi_hz: 60,
            records: vec![
                InputRecord {
                    madi: 0,
                    held_mask: mask_to_bytes(0b0010),
                },
                InputRecord {
                    madi: 1,
                    held_mask: mask_to_bytes(0b0010),
                },
                InputRecord {
                    madi: 2,
                    held_mask: mask_to_bytes(0),
                },
                InputRecord {
                    madi: 3,
                    held_mask: mask_to_bytes(0),
                },
            ],
        };
        write_input_tape(&sam_path, &tape).expect("write sam");
        let mut emitter = CaptureEmitter::new();
        let mut options = default_run_options();
        options.age_target = Some("AGE2".to_string());
        options.sam_path = Some(sam_path.clone());
        options.diag_jsonl = Some(diag_path.clone());
        options.latency_madi = 2;
        run_file_with_emitter(
            &input_path,
            Some(MadiLimit::Finite(4)),
            0,
            options,
            &mut emitter,
        )
        .expect("run should record latency diag");
        assert!(emitter.err.is_empty());
        let diag_text = fs::read_to_string(&diag_path).expect("read diag");
        assert!(diag_text.contains("\"event\":\"latency_schedule\""));
        assert!(diag_text.contains("\"accept_madi\":0"));
        assert!(diag_text.contains("\"target_madi\":2"));
        assert!(diag_text.contains("\"applied_madi\":2"));
        assert!(diag_text.contains("\"late\":false"));
        assert!(diag_text.contains("\"dropped\":false"));
        assert!(diag_text.contains("\"drop_policy\":\"late_drop\""));
        assert!(diag_text.contains("\"event\":\"run_config\""));
        assert!(diag_text.contains("\"latency_madi\":2"));
        assert!(diag_text.contains("\"latency_drop_policy\":\"late_drop\""));
        let _ = fs::remove_file(&project_path);
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&sam_path);
        let _ = fs::remove_file(&diag_path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn program_contains_open_call_detects_neomeo_alias() {
        let source = r#"
x <- () 너머.시각.지금.
"#;
        let program = parse_program_for_test(source);
        assert!(program_contains_open_call(&program));
    }

    #[test]
    fn program_contains_regex_call_detects_regex_builtins() {
        let source = r#"
패턴 <- 정규식{"[0-9]+"}.
찾음 <- ("a1", 패턴) 정규찾기.
"#;
        let program = parse_program_for_test(source);
        assert!(program_contains_regex_call(&program));
    }

    #[test]
    fn program_contains_input_builtin_detects_keyboard_surface() {
        let source = r#"
눌림 <- ("왼쪽") 막눌렸나.
"#;
        let program = parse_program_for_test(source);
        assert!(program_contains_input_builtin(&program));
    }

    #[test]
    fn normalize_open_kind_accepts_input_aliases() {
        assert_eq!(normalize_open_kind("입력"), Some("input"));
        assert_eq!(normalize_open_kind("사용자입력"), Some("input"));
        assert_eq!(normalize_open_kind("keyboard_input"), Some("input"));
        assert_eq!(normalize_open_kind("풀이"), Some("solver"));
        assert_eq!(normalize_open_kind("solver"), Some("solver"));
    }

    #[test]
    fn det_tier_parse_accepts_sealed_and_approx_aliases() {
        assert_eq!(DetTier::parse("D-STRICT"), Some(DetTier::Strict));
        assert_eq!(DetTier::parse("D-SEALED"), Some(DetTier::Sealed));
        assert_eq!(DetTier::parse("D-APPROX"), Some(DetTier::Approx));
        assert_eq!(DetTier::parse("D-FAST"), Some(DetTier::Sealed));
        assert_eq!(DetTier::parse("D-ULTRA"), Some(DetTier::Approx));
    }

    #[test]
    fn ensure_det_tier_supported_rejects_non_strict() {
        let sealed_err = ensure_det_tier_supported(DetTier::Sealed).expect_err("sealed must fail");
        assert!(sealed_err.contains("E_CONTRACT_TIER_UNSUPPORTED"));
        let approx_err = ensure_det_tier_supported(DetTier::Approx).expect_err("approx must fail");
        assert!(approx_err.contains("E_CONTRACT_TIER_UNSUPPORTED"));
        assert!(ensure_det_tier_supported(DetTier::Strict).is_ok());
    }

    #[test]
    fn enforce_det_tier_contract_age2_keeps_unsupported() {
        let sealed_err = enforce_det_tier_contract(DetTier::Sealed, AgeTarget::Age2, None, None)
            .expect_err("age2 sealed must fail");
        assert!(sealed_err.contains("E_CONTRACT_TIER_UNSUPPORTED"));
        let approx_err = enforce_det_tier_contract(DetTier::Approx, AgeTarget::Age2, None, None)
            .expect_err("age2 approx must fail");
        assert!(approx_err.contains("E_CONTRACT_TIER_UNSUPPORTED"));
    }

    #[test]
    fn enforce_det_tier_contract_age3_requires_seal_and_lock_for_sealed() {
        let missing_err = enforce_det_tier_contract(DetTier::Sealed, AgeTarget::Age3, None, None)
            .expect_err("age3 sealed missing pins must fail");
        assert!(missing_err.contains("E_RUNTIME_CONTRACT_MISMATCH"));
        assert!(enforce_det_tier_contract(
            DetTier::Sealed,
            AgeTarget::Age3,
            Some("sha256:seal-a"),
            Some("sha256:lock-a"),
        )
        .is_ok());
    }

    #[test]
    fn enforce_det_tier_contract_age3_allows_approx() {
        assert!(enforce_det_tier_contract(DetTier::Approx, AgeTarget::Age3, None, None).is_ok());
    }

    #[test]
    fn contract_label_for_manifest_preserves_supported_labels() {
        assert_eq!(
            contract_label_for_manifest(Some(DetTier::Strict)),
            "D-STRICT"
        );
        assert_eq!(
            contract_label_for_manifest(Some(DetTier::Sealed)),
            "D-SEALED"
        );
        assert_eq!(
            contract_label_for_manifest(Some(DetTier::Approx)),
            "D-APPROX"
        );
        assert_eq!(contract_label_for_manifest(None), "");
    }

    #[test]
    fn frontdoor_code_and_detail_maps_lang_parser_gap_code() {
        let message = "E_FRONTDOOR_LANG_PARSER_GAP lang_code=E_PARSE detail=파서 오류: .";
        let (code, detail) = frontdoor_code_and_detail(message);
        assert_eq!(code, "E_FRONTDOOR_LANG_PARSER_GAP");
        assert!(detail.contains("lang_code=E_PARSE"));
    }

    #[test]
    fn frontdoor_code_and_detail_uses_generic_frontdoor_code_for_unknown_message() {
        let (code, detail) = frontdoor_code_and_detail("E_FRONTDOOR_CUSTOM detail=custom");
        assert_eq!(code, "E_FRONTDOOR");
        assert_eq!(detail, "E_FRONTDOOR_CUSTOM detail=custom");
    }
}
