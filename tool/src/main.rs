#![cfg(not(target_arch = "wasm32"))]

pub mod lsp;
pub mod proof;
mod ddn_runtime;
mod detmath_assets;
mod gate0_registry;
mod preprocess;
mod schema;
mod ai_prompt;
mod project_meta;
mod paths;

use ddonirang_core::{
    EngineLoop, Fixed64, Geoul, InputSnapshot, Nuri, Unit, KEY_A, KEY_D, KEY_S, KEY_W,
    alrim::{AlrimHandler, AlrimLoop, VecAlrimLogger},
    platform::{Bogae, DetNuri, DetSam, InMemoryGeoul, Iyagi, Origin, Patch, PatchOp, Sam},
    resource_tag_with_unit,
    signals::{DiagEvent, Signal, TickId, VecSignalSink},
};
use ddonirang_lang::runtime::Value;
use ddonirang_lang::{canonicalize, normalize, parse_with_mode, NormalizationLevel, ParseMode};
use ddonirang_lang::{Expr, ExprKind, Literal, SeedDef, SeedKind, Stmt, TopLevelItem};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use crossterm::{cursor, event, execute, terminal};
use ddn_runtime::{DdnProgram, DdnRunner};
use preprocess::{
    format_file_meta,
    preprocess_ai_calls,
    preprocess_source_for_parse,
    split_file_meta,
    AiMeta,
};
use schema::{build_schema, write_schema};
use project_meta::{FeatureGate, load_project_policy, ProjectPolicy};

enum SimpleValue {
    Fixed64(Fixed64),
    Text(String),
    Bool(bool),
}

const DEFAULT_GOLDEN_PATH: &str = "docs/steps/000/artifacts/golden.json";
const DEFAULT_REPLAY_PATH: &str = "docs/steps/000/artifacts/replay.json";
const DEFAULT_MAZE_MAP_PATH: &str = "docs/EXAMPLES/tracks/11단계/artifacts/maps/maze.txt";
const DEFAULT_COIN_MAZE_MAP_PATH: &str = "docs/EXAMPLES/tracks/11단계/artifacts/maps/coin_maze.txt";
const DEFAULT_MAZE_SCRIPT_PATH: &str = "docs/EXAMPLES/tracks/11단계/artifacts/scripts/maze_v0.ddn";
const DEFAULT_RUN_ONCE_INPUT_PATH: &str = "docs/steps/001/artifacts/input.ddn";
const DEFAULT_DIAG_PATH: &str = "geoul.diag.jsonl";
const DEFAULT_TEST_DIR: &str = "golden";
const LINT_VERSION: &str = "v18.12.12";
const TERM_MAP_VERSION: &str = "v18.12.12";

fn default_repro_path() -> PathBuf {
    paths::build_dir().join("repro").join("ddn.repro.last.json")
}

#[derive(Serialize, Deserialize)]
struct DiagEventJson {
    #[serde(default = "diag_kind")]
    kind: String,
    madi: u64,
    seq: u64,
    fault_id: String,
    rule_id: String,
    reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sub_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contract_kind: Option<String>,
    origin: String,
    targets: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sam_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_span: Option<SourceSpanJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expr: Option<ExprTraceJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct SourceSpanJson {
    file: String,
    start_line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_col: Option<u32>,
    end_line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_col: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct ExprTraceJson {
    tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NetEventJson {
    sender: String,
    seq: u64,
    order_key: String,
    payload: serde_json::Value,
}

#[cfg(test)]
#[derive(Debug, Deserialize)]
struct InputSnapshotDetJson {
    #[serde(default)]
    schema: Option<String>,
    #[serde(default)]
    net_events: Vec<NetEventJson>,
}

fn diag_kind() -> String {
    "Diag".to_string()
}

fn write_diag_jsonl(events: &[DiagEvent], path: &str) -> Result<(), String> {
    if events.is_empty() {
        return Ok(());
    }
    let mut file = std::fs::File::create(path)
        .map_err(|e| format!("diag 파일 생성 실패: {}", e))?;
    for event in events {
        let json = serde_json::to_string(&DiagEventJson {
            kind: "Diag".to_string(),
            madi: event.madi,
            seq: event.seq,
            fault_id: event.fault_id.clone(),
            rule_id: event.rule_id.clone(),
            reason: event.reason.clone(),
            sub_reason: event.sub_reason.clone(),
            mode: event.mode.clone(),
            contract_kind: event.contract_kind.clone(),
            origin: event.origin.clone(),
            targets: event.targets.clone(),
            sam_hash: event.sam_hash.clone(),
            source_span: event.source_span.as_ref().map(|span| SourceSpanJson {
                file: span.file.clone(),
                start_line: span.start_line,
                start_col: span.start_col,
                end_line: span.end_line,
                end_col: span.end_col,
            }),
            expr: event.expr.as_ref().map(|expr| ExprTraceJson {
                tag: expr.tag.clone(),
                text: expr.text.clone(),
            }),
            message: event.message.clone(),
        })
        .map_err(|e| format!("diag 직렬화 실패: {}", e))?;
        writeln!(file, "{}", json).map_err(|e| format!("diag 쓰기 실패: {}", e))?;
    }
    Ok(())
}

fn write_diag_if_any(sink: &VecSignalSink) -> Result<(), String> {
    write_diag_jsonl(&sink.diag_events, DEFAULT_DIAG_PATH)
}

#[derive(Debug, Deserialize)]
struct GoldenTestFile {
    name: String,
    sam_input: String,
    max_madi: u64,
    #[serde(default)]
    det_expected: Option<DetExpected>,
    #[serde(default)]
    trace_expected: Option<TraceExpected>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    script: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DetExpected {
    #[serde(default)]
    expected_state_hash: Option<String>,
    #[serde(default)]
    no_fault: Option<bool>,
    #[serde(default)]
    expected_observations: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    expected_error_contains: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TraceExpected {
    #[serde(default)]
    #[allow(dead_code)]
    events: Vec<TraceExpectedEvent>,
}

#[derive(Debug, Deserialize)]
struct TraceExpectedEvent {
    #[allow(dead_code)]
    kind: String,
    #[allow(dead_code)]
    count: u64,
}

#[derive(Debug, Serialize)]
struct ReproReport {
    toolchain_version: String,
    schema_hash: Option<String>,
    realm_initial_hash: Option<String>,
    lint_version: String,
    term_map_version: String,
    failed_madi: u64,
    sam_sequence_hash: String,
    fault_id: String,
    rule_id: String,
    origin: String,
    targets: Vec<String>,
    repro_command: String,
    diag_path: Option<String>,
}

struct ScenarioRun {
    hashes: Vec<String>,
    sink: VecSignalSink,
}

fn collect_test_files(path: Option<String>) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    if let Some(path) = path {
        let path = PathBuf::from(path);
        if path.is_dir() {
            for entry in std::fs::read_dir(&path)
                .map_err(|e| format!("테스트 폴더 읽기 실패: {e}"))?
            {
                let entry = entry.map_err(|e| format!("테스트 폴더 읽기 실패: {e}"))?;
                let path = entry.path();
                if path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("json"))
                    .unwrap_or(false)
                    && path.to_string_lossy().ends_with(".test.json")
                {
                    files.push(path);
                }
            }
        } else {
            files.push(path);
        }
    } else {
        let root = PathBuf::from(DEFAULT_TEST_DIR);
        if !root.exists() {
            return Err(format!("테스트 폴더가 없습니다: {}", DEFAULT_TEST_DIR));
        }
        for entry in std::fs::read_dir(&root)
            .map_err(|e| format!("테스트 폴더 읽기 실패: {e}"))?
        {
            let entry = entry.map_err(|e| format!("테스트 폴더 읽기 실패: {e}"))?;
            let path = entry.path();
            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("json"))
                .unwrap_or(false)
                && path.to_string_lossy().ends_with(".test.json")
            {
                files.push(path);
            }
        }
    }
    files.sort();
    if files.is_empty() {
        return Err("테스트 파일이 없습니다".to_string());
    }
    Ok(files)
}

fn run_tests(path: Option<String>) -> Result<(), String> {
    let files = collect_test_files(path)?;
    let mut failures = Vec::new();
    for path in files {
        let text = read_text_from_path(path.to_str().ok_or("테스트 경로 오류")?)?;
        let test: GoldenTestFile =
            serde_json::from_str(&text).map_err(|e| format!("테스트 파일 파싱 실패: {e}"))?;
        let result = run_test_case(&test)?;
        if let Some(err) = result {
            failures.push(err);
        }
    }
    if let Some(failure) = failures.first() {
        write_repro_last(failure)?;
        return Err(format!("테스트 실패: {}", failure.name));
    }
    println!("test_ok: true");
    println!("test_failures: 0");
    Ok(())
}

struct TestFailure {
    name: String,
    failed_madi: u64,
    fault_id: String,
    rule_id: String,
    origin: String,
    targets: Vec<String>,
    sam_input: String,
    diag_path: Option<String>,
    initial_hash: Option<String>,
}

fn run_test_case(test: &GoldenTestFile) -> Result<Option<TestFailure>, String> {
    let det_expected = test.det_expected.as_ref().ok_or_else(|| {
        format!("DetTest만 지원합니다: {}", test.name)
    })?;
    let expected_error = det_expected.expected_error_contains.as_deref();
    if det_expected.expected_observations.is_some() {
        return Err(format!(
            "expected_observations는 Gate0에서 미지원입니다: {}",
            test.name
        ));
    }
    if test.trace_expected.is_some() {
        return Err(format!(
            "TraceTest는 Gate0에서 미지원입니다: {}",
            test.name
        ));
    }

    let run = match run_scenario(test) {
        Ok(run) => {
            if let Some(expect) = expected_error {
                return Ok(Some(TestFailure {
                    name: test.name.clone(),
                    failed_madi: test.max_madi.saturating_sub(1),
                    fault_id: "EXPECTED_ERROR_MISSING".to_string(),
                    rule_id: String::new(),
                    origin: "tool:test".to_string(),
                    targets: vec![expect.to_string()],
                    sam_input: test.sam_input.clone(),
                    diag_path: Some(DEFAULT_DIAG_PATH.to_string()),
                    initial_hash: run.hashes.first().cloned(),
                }));
            }
            run
        }
        Err(err) => {
            if let Some(expect) = expected_error {
                if err.contains(expect) {
                    return Ok(None);
                }
                return Err(format!("예상 오류 불일치: {} (got: {})", test.name, err));
            }
            return Err(err);
        }
    };
    write_diag_if_any(&run.sink)?;

    let expected = det_expected
        .expected_state_hash
        .as_ref()
        .ok_or_else(|| format!("expected_state_hash 누락: {}", test.name))?;
    let last_hash = run
        .hashes
        .last()
        .ok_or_else(|| format!("hashes 비어있음: {}", test.name))?;
    if last_hash != expected {
        return Ok(Some(TestFailure {
            name: test.name.clone(),
            failed_madi: test.max_madi.saturating_sub(1),
            fault_id: "STATE_HASH_MISMATCH".to_string(),
            rule_id: String::new(),
            origin: "tool:test".to_string(),
            targets: vec!["state_hash".to_string()],
            sam_input: test.sam_input.clone(),
            diag_path: Some(DEFAULT_DIAG_PATH.to_string()),
            initial_hash: run.hashes.first().cloned(),
        }));
    }
    if det_expected.no_fault.unwrap_or(false) {
        if !run.sink.diag_events.is_empty()
            || run
                .sink
                .signals
                .iter()
                .any(|signal| matches!(signal, Signal::ArithmeticFault { .. }))
        {
            return Ok(Some(TestFailure {
                name: test.name.clone(),
                failed_madi: test.max_madi.saturating_sub(1),
                fault_id: "FAULT_DETECTED".to_string(),
                rule_id: String::new(),
                origin: "tool:test".to_string(),
                targets: vec!["fault".to_string()],
                sam_input: test.sam_input.clone(),
                diag_path: Some(DEFAULT_DIAG_PATH.to_string()),
                initial_hash: run.hashes.first().cloned(),
            }));
        }
    }
    Ok(None)
}

fn run_scenario(test: &GoldenTestFile) -> Result<ScenarioRun, String> {
    let kind = test
        .kind
        .as_deref()
        .ok_or_else(|| format!("kind 누락: {}", test.name))?;
    match kind {
        "gate0_pipe" => {
            let script = test
                .script
                .as_deref()
                .or_else(|| Path::new(&test.sam_input).to_str())
                .ok_or_else(|| format!("script 누락: {}", test.name))?;
            run_gate0_pipe_with_sink(script, test.max_madi)
        }
        "gate0_unit" => {
            let script = test
                .script
                .as_deref()
                .or_else(|| Path::new(&test.sam_input).to_str())
                .ok_or_else(|| format!("script 누락: {}", test.name))?;
            run_gate0_unit_with_sink(script, test.max_madi)
        }
        "gate0_fault" => run_gate0_fault_with_sink(test.max_madi),
        _ => Err(format!("지원하지 않는 kind: {}", kind)),
    }
}

fn run_gate0_pipe_with_sink(script_path: &str, ticks: u64) -> Result<ScenarioRun, String> {
    let source = read_text_from_path(script_path)?;
    let program = DdnProgram::from_source(&source, script_path)?;
    let mut defaults = HashMap::new();
    defaults.insert("결과".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
    let iyagi = Gate0ScriptIyagi::new(program, defaults, Vec::new());
    run_gate0_script_with_sink(iyagi, ticks)
}

fn run_gate0_unit_with_sink(script_path: &str, ticks: u64) -> Result<ScenarioRun, String> {
    let source = read_text_from_path(script_path)?;
    let program = DdnProgram::from_source(&source, script_path)?;
    let mut defaults = HashMap::new();
    defaults.insert("결과".to_string(), Value::Fixed64(Fixed64::from_i64(7)));
    defaults.insert(
        "입력값".to_string(),
        Value::Unit(ddonirang_core::UnitValue::new(
            Fixed64::from_i64(1),
            Unit::Second,
        )),
    );
    let startup_ops = vec![PatchOp::SetResourceFixed64 {
        tag: "결과".to_string(),
        value: Fixed64::from_i64(7),
    }];
    let iyagi = Gate0ScriptIyagi::new(program, defaults, startup_ops);
    run_gate0_script_with_sink(iyagi, ticks)
}

fn run_gate0_fault_with_sink(ticks: u64) -> Result<ScenarioRun, String> {
    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = Div0Iyagi;
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();
    let mut hashes = Vec::with_capacity(ticks as usize);
    let ticks = ticks.max(1);
    for tick in 0..ticks {
        let frame = loop_.tick_once(tick, &mut sink);
        hashes.push(frame.state_hash.to_hex());
    }
    Ok(ScenarioRun { hashes, sink })
}

fn run_gate0_script_with_sink(iyagi: Gate0ScriptIyagi, ticks: u64) -> Result<ScenarioRun, String> {
    let sam = DetSam::new(Fixed64::from_i64(1));
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();
    let mut hashes = Vec::with_capacity(ticks as usize);
    let ticks = ticks.max(1);
    for tick in 0..ticks {
        let frame = loop_.tick_once(tick, &mut sink);
        hashes.push(frame.state_hash.to_hex());
    }
    if let Some(err) = loop_.iyagi.take_error() {
        return Err(err);
    }
    Ok(ScenarioRun { hashes, sink })
}

fn write_repro_last(failure: &TestFailure) -> Result<(), String> {
    let repro_path = default_repro_path();
    let repro_dir = repro_path
        .parent()
        .ok_or_else(|| "repro 경로의 상위 폴더를 찾지 못했습니다".to_string())?;
    std::fs::create_dir_all(repro_dir)
        .map_err(|e| format!("repro 폴더 생성 실패: {e}"))?;
    let schema_hash = read_schema_hash();
    let sam_sequence_hash = blake3::hash(failure.sam_input.as_bytes())
        .to_hex()
        .to_string();
    let report = ReproReport {
        toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_hash,
        realm_initial_hash: failure.initial_hash.clone(),
        lint_version: LINT_VERSION.to_string(),
        term_map_version: TERM_MAP_VERSION.to_string(),
        failed_madi: failure.failed_madi,
        sam_sequence_hash,
        fault_id: failure.fault_id.clone(),
        rule_id: failure.rule_id.clone(),
        origin: failure.origin.clone(),
        targets: failure.targets.clone(),
        repro_command: format!(
            "cargo run -p ddonirang-tool -- test {}",
            DEFAULT_TEST_DIR
        ),
        diag_path: failure.diag_path.clone(),
    };
    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| format!("repro 직렬화 실패: {e}"))?;
    std::fs::write(&repro_path, json)
        .map_err(|e| format!("repro 저장 실패: {e}"))?;
    println!("repro_written: {}", repro_path.display());
    Ok(())
}

fn run_geoul_query(query: &str, path: Option<&str>) -> Result<(), String> {
    let path = path.unwrap_or(DEFAULT_DIAG_PATH);
    let text = read_text_from_path(path)?;
    let mut matches = 0usize;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let event: DiagEventJson =
            serde_json::from_str(trimmed).map_err(|e| format!("diag 파싱 실패: {e}"))?;
        if diag_matches(&event, query) {
            println!("{}", trimmed);
            matches += 1;
        }
    }
    println!("matches: {}", matches);
    Ok(())
}

fn diag_matches(event: &DiagEventJson, query: &str) -> bool {
    if let Some(rule) = query.strip_prefix("rule_id:") {
        return event.rule_id.contains(rule.trim());
    }
    if query == "#고장" {
        return matches!(
            event.reason.as_str(),
            "ARITH_FAULT" | "UNIT_MISMATCH" | "GUARD_VIOLATION"
        );
    }
    if let Some(origin) = query.strip_prefix("origin:") {
        return event.origin.contains(origin.trim());
    }
    if event.reason.contains(query)
        || event.fault_id.contains(query)
        || event.rule_id.contains(query)
        || event.origin.contains(query)
        || event.targets.iter().any(|t| t.contains(query))
    {
        return true;
    }
    false
}

fn ensure_detmath_ready() -> Result<(), String> {
    detmath_assets::ensure_detmath_assets()?;
    gate0_registry::ensure_gate0_registries()
}

fn simple_value_from_expr(expr: &Expr) -> Option<SimpleValue> {
    match &expr.kind {
        ExprKind::Literal(Literal::Fixed64(v)) => Some(SimpleValue::Fixed64(*v)),
        ExprKind::Literal(Literal::Int(v)) => Some(SimpleValue::Fixed64(Fixed64::from_i64(*v))),
        ExprKind::Literal(Literal::String(s)) => Some(SimpleValue::Text(s.clone())),
        ExprKind::Literal(Literal::Atom(a)) => Some(SimpleValue::Text(a.clone())),
        ExprKind::Literal(Literal::Resource(path)) => Some(SimpleValue::Text(path.clone())),
        ExprKind::Literal(Literal::Bool(b)) => Some(SimpleValue::Bool(*b)),
        ExprKind::Literal(Literal::None) => None,
        ExprKind::Suffix { value, .. } => simple_value_from_expr(value),
        ExprKind::Var(name) if name == "참" => Some(SimpleValue::Bool(true)),
        ExprKind::Var(name) if name == "거짓" => Some(SimpleValue::Bool(false)),
        _ => None,
    }
}

fn simple_value_from_stmt(stmt: &Stmt) -> Option<SimpleValue> {
    match stmt {
        Stmt::Return { value, .. } => simple_value_from_expr(value),
        Stmt::Expr { expr, .. } => simple_value_from_expr(expr),
        _ => None,
    }
}

fn simple_value_from_seed(seed: &SeedDef) -> Option<SimpleValue> {
    let body = seed.body.as_ref()?;
    for stmt in &body.stmts {
        if let Some(value) = simple_value_from_stmt(stmt) {
            return Some(value);
        }
    }
    None
}

struct InputIyagi {
    line: String,
}

impl Iyagi for InputIyagi {
    fn run_startup(&mut self, _world: &ddonirang_core::platform::NuriWorld) -> Patch {
        Patch::default()
    }

    fn run_update(
        &mut self,
        _world: &ddonirang_core::platform::NuriWorld,
        _input: &ddonirang_core::platform::InputSnapshot,
    ) -> Patch {
        let len = self.line.chars().count() as i64;
        let mut ops = vec![
            PatchOp::SetResourceJson {
                tag: "입력".to_string(),
                json: self.line.clone(),
            },
            PatchOp::SetResourceFixed64 {
                tag: "입력길이".to_string(),
                value: Fixed64::from_i64(len),
            },
        ];

        if let Ok(cleaned) = preprocess_source_for_parse(&self.line) {
            if let Ok(mut program) =
                parse_with_mode(&cleaned, "stdin.ddoni", ddn_runtime::default_parse_mode())
            {
                if canonicalize(&mut program).is_ok() {
                    let normalized = normalize(&program, NormalizationLevel::N1);
                    ops.push(PatchOp::SetResourceJson {
                        tag: "정본".to_string(),
                        json: normalized.clone(),
                    });

                    for item in &program.items {
                        let TopLevelItem::SeedDef(seed) = item;
                        if !seed.params.is_empty() {
                            continue;
                        }
                        let kind_name = match &seed.seed_kind {
                            SeedKind::Named(name) => name.as_str(),
                            _ => continue,
                        };
                        let Some(value) = simple_value_from_seed(seed) else {
                            continue;
                        };
                        match (kind_name, value) {
                            ("수", SimpleValue::Fixed64(v)) => {
                                ops.push(PatchOp::SetResourceFixed64 {
                                    tag: seed.canonical_name.clone(),
                                    value: v,
                                });
                            }
                            ("글", SimpleValue::Text(s)) => {
                                ops.push(PatchOp::SetResourceJson {
                                    tag: seed.canonical_name.clone(),
                                    json: s,
                                });
                            }
                            ("참거짓", SimpleValue::Bool(b)) => {
                                let text = if b { "참" } else { "거짓" }.to_string();
                                ops.push(PatchOp::SetResourceJson {
                                    tag: seed.canonical_name.clone(),
                                    json: text,
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Patch {
            ops,
            origin: Origin::system("tool"),
        }
    }
}

struct NoopBogae;

impl Bogae for NoopBogae {
    fn render(&mut self, _world: &ddonirang_core::platform::NuriWorld, _tick_id: TickId) {}
}

struct ConsoleBogae;

impl ConsoleBogae {
    fn render_to_string(world: &ddonirang_core::platform::NuriWorld) -> String {
        let mut out = if let Some(map) = world.get_resource_json("맵") {
            map
        } else {
            let w = world
                .get_resource_fixed64("맵_w")
                .unwrap_or(Fixed64::from_i64(5))
                .int_part()
                .max(1);
            let h = world
                .get_resource_fixed64("맵_h")
                .unwrap_or(Fixed64::from_i64(5))
                .int_part()
                .max(1);
            let px = world
                .get_resource_fixed64("플레이어_x")
                .unwrap_or(Fixed64::ZERO)
                .int_part();
            let py = world
                .get_resource_fixed64("플레이어_y")
                .unwrap_or(Fixed64::ZERO)
                .int_part();

            let mut map_out = String::new();
            for y in 0..h {
                for x in 0..w {
                    let ch = if x == px && y == py { 'P' } else { '.' };
                    map_out.push(ch);
                }
                if y + 1 < h {
                    map_out.push('\n');
                }
            }
            map_out
        };

        if let Some(score) = world.get_resource_fixed64("점수") {
            out.push('\n');
            out.push_str(&format!("점수: {}", score));
        }
        if let Some(coins) = world.get_resource_fixed64("코인남음") {
            out.push('\n');
            out.push_str(&format!("코인남음: {}", coins));
        }
        if let Some(goal) = world.get_resource_json("목표") {
            out.push('\n');
            out.push_str(&format!("목표: {}", goal));
        }
        if let Some(msg) = world.get_resource_json("메시지") {
            out.push('\n');
            out.push_str(&msg);
        }
        out
    }
}

impl Bogae for ConsoleBogae {
    fn render(&mut self, world: &ddonirang_core::platform::NuriWorld, _tick_id: TickId) {
        let s = Self::render_to_string(world);
        println!("{s}");
    }
}

struct LiveBogae {
    stdout: io::Stdout,
}

impl LiveBogae {
    fn new() -> Self {
        Self { stdout: io::stdout() }
    }
}

impl Bogae for LiveBogae {
    fn render(&mut self, world: &ddonirang_core::platform::NuriWorld, _tick_id: TickId) {
        let out = ConsoleBogae::render_to_string(world);
        let _ = execute!(
            self.stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::All)
        );
        let _ = self.stdout.write_all(out.as_bytes());
        let _ = self.stdout.flush();
    }
}

struct LiveTerminalGuard;

impl LiveTerminalGuard {
    fn enter(stdout: &mut io::Stdout) -> Result<Self, String> {
        terminal::enable_raw_mode().map_err(|e| format!("터미널 raw 모드 실패: {e}"))?;
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)
            .map_err(|e| format!("터미널 화면 전환 실패: {e}"))?;
        Ok(Self)
    }
}

impl Drop for LiveTerminalGuard {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

struct ReplaySam {
    snapshots: Vec<InputSnapshot>,
    index: usize,
}

impl ReplaySam {
    fn new(snapshots: Vec<InputSnapshot>) -> Self {
        Self { snapshots, index: 0 }
    }
}

impl Sam for ReplaySam {
    fn begin_tick(&mut self, _tick_id: TickId) -> InputSnapshot {
        let snapshot = self
            .snapshots
            .get(self.index)
            .expect("replay snapshot")
            .clone();
        self.index += 1;
        snapshot
    }

    fn push_async_ai(
        &mut self,
        _agent_id: u64,
        _recv_seq: u64,
        _accepted_madi: u64,
        _target_madi: u64,
        _intent: ddonirang_core::SeulgiIntent,
    ) {
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ReplayLog {
    frames: Vec<ReplayFrame>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReplayFrame {
    tick_id: TickId,
    dt_raw: i64,
    keys_pressed: u64,
    #[serde(default)]
    last_key_name: String,
    pointer_x_i32: i32,
    pointer_y_i32: i32,
    #[serde(default)]
    net_events: Vec<NetEventJson>,
    rng_seed: u64,
    state_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoldenFile {
    version: u32,
    scenarios: Vec<GoldenScenario>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoldenScenario {
    name: String,
    kind: GoldenScenarioKind,
    hashes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum GoldenScenarioKind {
    Maze { map: String, moves: String },
    Parabola { ticks: u64, vx: i64, vy: i64 },
    Gate0Pipe { script: String },
    Gate0Unit { script: String },
    Gate0Fault,
}

struct Div0Iyagi;

impl Iyagi for Div0Iyagi {
    fn run_startup(&mut self, _world: &ddonirang_core::platform::NuriWorld) -> Patch {
        Patch::default()
    }

    fn run_update(
        &mut self,
        _world: &ddonirang_core::platform::NuriWorld,
        input: &ddonirang_core::platform::InputSnapshot,
    ) -> Patch {
        Patch {
            ops: vec![
                PatchOp::SetResourceFixed64 {
                    tag: "x".to_string(),
                    value: Fixed64::from_i64(5),
                },
                PatchOp::DivAssignResourceFixed64 {
                    tag: "x".to_string(),
                    rhs: Fixed64::ZERO,
                    tick_id: input.tick_id,
                    location: "tool:Div0Iyagi/x_div0",
                    source_span: Some(ddonirang_core::SourceSpan {
                        file: "tool/src/main.rs".to_string(),
                        start_line: 1,
                        start_col: Some(1),
                        end_line: 1,
                        end_col: Some(1),
                    }),
                    expr: Some(ddonirang_core::ExprTrace {
                        tag: "arith:DIV0".to_string(),
                        text: None,
                    }),
                },
            ],
            origin: Origin::system("tool"),
        }
    }
}

struct GridIyagi;

impl Iyagi for GridIyagi {
    fn run_startup(&mut self, _world: &ddonirang_core::platform::NuriWorld) -> Patch {
        Patch::default()
    }

    fn run_update(
        &mut self,
        _world: &ddonirang_core::platform::NuriWorld,
        _input: &ddonirang_core::platform::InputSnapshot,
    ) -> Patch {
        Patch {
            ops: vec![
                PatchOp::SetResourceFixed64 { tag: "맵_w".to_string(), value: Fixed64::from_i64(5) },
                PatchOp::SetResourceFixed64 { tag: "맵_h".to_string(), value: Fixed64::from_i64(4) },
                PatchOp::SetResourceFixed64 { tag: "플레이어_x".to_string(), value: Fixed64::from_i64(2) },
                PatchOp::SetResourceFixed64 { tag: "플레이어_y".to_string(), value: Fixed64::from_i64(1) },
            ],
            origin: Origin::system("tool"),
        }
    }
}

#[derive(Clone)]
struct MazeData {
    width: i64,
    height: i64,
    tiles: Vec<Vec<char>>,
    goal: Option<(i64, i64)>,
    coins: Vec<(i64, i64)>,
}

struct MazeIyagi {
    data: MazeData,
    player_x: i64,
    player_y: i64,
    goal_reached: bool,
    score: i64,
}

impl MazeIyagi {
    fn new(data: MazeData, start_x: i64, start_y: i64) -> Self {
        Self {
            data,
            player_x: start_x,
            player_y: start_y,
            goal_reached: false,
            score: 0,
        }
    }

    fn render_map(&self) -> String {
        let mut out = String::new();
        for y in 0..self.data.height {
            for x in 0..self.data.width {
                let ch = if x == self.player_x && y == self.player_y {
                    'P'
                } else if self
                    .data
                    .coins
                    .iter()
                    .any(|(cx, cy)| *cx == x && *cy == y)
                {
                    'C'
                } else {
                    self.data.tiles[y as usize][x as usize]
                };
                out.push(ch);
            }
            if y + 1 < self.data.height {
                out.push('\n');
            }
        }
        out
    }

    fn take_coin_at(&mut self, x: i64, y: i64) -> bool {
        if let Some(idx) = self
            .data
            .coins
            .iter()
            .position(|(cx, cy)| *cx == x && *cy == y)
        {
            self.data.coins.remove(idx);
            return true;
        }
        false
    }

    fn try_move(&mut self, dx: i64, dy: i64) {
        let nx = self.player_x + dx;
        let ny = self.player_y + dy;
        if nx < 0 || ny < 0 || nx >= self.data.width || ny >= self.data.height {
            return;
        }
        let tile = self.data.tiles[ny as usize][nx as usize];
        if tile == '#' {
            return;
        }
        self.player_x = nx;
        self.player_y = ny;
    }
}

impl Iyagi for MazeIyagi {
    fn run_startup(&mut self, _world: &ddonirang_core::platform::NuriWorld) -> Patch {
        Patch::default()
    }

    fn run_update(
        &mut self,
        _world: &ddonirang_core::platform::NuriWorld,
        input: &ddonirang_core::platform::InputSnapshot,
    ) -> Patch {
        let keys = input.keys_pressed;
        if keys & KEY_W != 0 {
            self.try_move(0, -1);
        } else if keys & KEY_A != 0 {
            self.try_move(-1, 0);
        } else if keys & KEY_S != 0 {
            self.try_move(0, 1);
        } else if keys & KEY_D != 0 {
            self.try_move(1, 0);
        }
        if self.take_coin_at(self.player_x, self.player_y) {
            self.score += 1;
        }

        let reached = self
            .data
            .goal
            .map(|(gx, gy)| gx == self.player_x && gy == self.player_y)
            .unwrap_or(false);
        if reached && !self.goal_reached {
            self.goal_reached = true;
            self.score += 1;
        }

        let mut ops = vec![
            PatchOp::SetResourceFixed64 { tag: "맵_w".to_string(), value: Fixed64::from_i64(self.data.width) },
            PatchOp::SetResourceFixed64 { tag: "맵_h".to_string(), value: Fixed64::from_i64(self.data.height) },
            PatchOp::SetResourceFixed64 { tag: "플레이어_x".to_string(), value: Fixed64::from_i64(self.player_x) },
            PatchOp::SetResourceFixed64 { tag: "플레이어_y".to_string(), value: Fixed64::from_i64(self.player_y) },
            PatchOp::SetResourceJson { tag: "맵".to_string(), json: self.render_map() },
            PatchOp::SetResourceFixed64 { tag: "점수".to_string(), value: Fixed64::from_i64(self.score) },
            PatchOp::SetResourceJson { tag: "목표".to_string(), json: if self.goal_reached { "달성".to_string() } else { "진행중".to_string() } },
            PatchOp::SetResourceFixed64 { tag: "코인남음".to_string(), value: Fixed64::from_i64(self.data.coins.len() as i64) },
        ];

        if reached {
            ops.push(PatchOp::SetResourceJson { tag: "메시지".to_string(), json: "도착".to_string() });
        }

        Patch {
            ops,
            origin: Origin::system("tool"),
        }
    }
}

struct DdnMazeIyagi {
    runner: DdnRunner,
    base_tiles: Vec<Vec<char>>,
    width: i64,
    height: i64,
    start_x: i64,
    start_y: i64,
    base_map_text: String,
}

impl DdnMazeIyagi {
    fn new(data: MazeData, start_x: i64, start_y: i64, program: DdnProgram) -> Self {
        let base_map_text = map_text_from_data(&data);
        Self {
            runner: DdnRunner::new(program, "매틱"),
            base_tiles: data.tiles,
            width: data.width,
            height: data.height,
            start_x,
            start_y,
            base_map_text,
        }
    }

    fn render_map(&self, map_text: &str, px: i64, py: i64) -> String {
        let lines: Vec<&str> = map_text.lines().collect();
        let mut out = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let ch = if x == px && y == py {
                    'P'
                } else {
                    lines
                        .get(y as usize)
                        .and_then(|line| line.chars().nth(x as usize))
                        .unwrap_or(self.base_tiles[y as usize][x as usize])
                };
                out.push(ch);
            }
            if y + 1 < self.height {
                out.push('\n');
            }
        }
        out
    }
}

impl Iyagi for DdnMazeIyagi {
    fn run_startup(&mut self, _world: &ddonirang_core::platform::NuriWorld) -> Patch {
        Patch::default()
    }

    fn run_update(
        &mut self,
        world: &ddonirang_core::platform::NuriWorld,
        input: &ddonirang_core::platform::InputSnapshot,
    ) -> Patch {
        let mut defaults = HashMap::new();
        defaults.insert("맵원본".to_string(), Value::String(self.base_map_text.clone()));
        defaults.insert("플레이어_x".to_string(), Value::Fixed64(Fixed64::from_i64(self.start_x)));
        defaults.insert("플레이어_y".to_string(), Value::Fixed64(Fixed64::from_i64(self.start_y)));
        defaults.insert("입력키".to_string(), Value::String(input.last_key_name.clone()));
        defaults.insert("점수".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
        defaults.insert("목표".to_string(), Value::String("진행중".to_string()));
        defaults.insert("메시지".to_string(), Value::String(String::new()));

        let output = match self.runner.run_update(world, input, &defaults) {
            Ok(out) => out,
            Err(err) => {
                return Patch {
                    ops: vec![PatchOp::SetResourceJson {
                        tag: "메시지".to_string(),
                        json: format!("오류: {}", err),
                    }],
                    origin: Origin::system("ddn"),
                };
            }
        };

        let mut ops = output.patch.ops;
        if world.get_resource_json("맵원본").is_none() {
            ops.push(PatchOp::SetResourceJson {
                tag: "맵원본".to_string(),
                json: self.base_map_text.clone(),
            });
        }
        if world.get_resource_fixed64("플레이어_x").is_none() {
            ops.push(PatchOp::SetResourceFixed64 {
                tag: "플레이어_x".to_string(),
                value: Fixed64::from_i64(self.start_x),
            });
        }
        if world.get_resource_fixed64("플레이어_y").is_none() {
            ops.push(PatchOp::SetResourceFixed64 {
                tag: "플레이어_y".to_string(),
                value: Fixed64::from_i64(self.start_y),
            });
        }
        if world.get_resource_json("목표").is_none() {
            ops.push(PatchOp::SetResourceJson {
                tag: "목표".to_string(),
                json: "진행중".to_string(),
            });
        }
        if world.get_resource_json("메시지").is_none() {
            ops.push(PatchOp::SetResourceJson {
                tag: "메시지".to_string(),
                json: String::new(),
            });
        }
        if world.get_resource_fixed64("점수").is_none() {
            ops.push(PatchOp::SetResourceFixed64 {
                tag: "점수".to_string(),
                value: Fixed64::from_i64(0),
            });
        }

        let px = read_fixed64_from_context("플레이어_x", &output.resources, world, &defaults)
            .unwrap_or(Fixed64::from_i64(self.start_x))
            .int_part();
        let py = read_fixed64_from_context("플레이어_y", &output.resources, world, &defaults)
            .unwrap_or(Fixed64::from_i64(self.start_y))
            .int_part();
        let map_text = read_text_from_context("맵원본", &output.resources, world, &defaults)
            .unwrap_or_else(|| self.base_map_text.clone());
        ops.push(PatchOp::SetResourceJson {
            tag: "맵".to_string(),
            json: self.render_map(&map_text, px, py),
        });

        Patch {
            ops,
            origin: Origin::system("ddn"),
        }
    }
}

struct ParabolaIyagi {
    x: Fixed64,
    y: Fixed64,
    vx: Fixed64,
    vy: Fixed64,
    g: Fixed64,
    t: Fixed64,
    tag_x: String,
    tag_y: String,
    tag_t: String,
}

impl ParabolaIyagi {
    fn new(vx: Fixed64, vy: Fixed64, g: Fixed64) -> Self {
        Self {
            x: Fixed64::ZERO,
            y: Fixed64::ZERO,
            vx,
            vy,
            g,
            t: Fixed64::ZERO,
            tag_x: resource_tag_with_unit("x", Unit::Meter),
            tag_y: resource_tag_with_unit("y", Unit::Meter),
            tag_t: resource_tag_with_unit("t", Unit::Second),
        }
    }
}

impl Iyagi for ParabolaIyagi {
    fn run_startup(&mut self, _world: &ddonirang_core::platform::NuriWorld) -> Patch {
        Patch::default()
    }

    fn run_update(
        &mut self,
        _world: &ddonirang_core::platform::NuriWorld,
        input: &ddonirang_core::platform::InputSnapshot,
    ) -> Patch {
        let dt = input.dt;
        self.x = self.x + self.vx * dt;
        self.y = self.y + self.vy * dt;
        self.vy = self.vy - self.g * dt;
        self.t = self.t + dt;

        Patch {
            ops: vec![
                PatchOp::SetResourceFixed64 { tag: self.tag_x.clone(), value: self.x },
                PatchOp::SetResourceFixed64 { tag: self.tag_y.clone(), value: self.y },
                PatchOp::SetResourceFixed64 { tag: self.tag_t.clone(), value: self.t },
            ],
            origin: Origin::system("tool"),
        }
    }
}

struct Gate0ScriptIyagi {
    runner: DdnRunner,
    defaults: HashMap<String, Value>,
    startup_ops: Vec<PatchOp>,
    last_error: Option<String>,
}

impl Gate0ScriptIyagi {
    fn new(program: DdnProgram, defaults: HashMap<String, Value>, startup_ops: Vec<PatchOp>) -> Self {
        Self {
            runner: DdnRunner::new(program, "매틱"),
            defaults,
            startup_ops,
            last_error: None,
        }
    }

    fn take_error(&mut self) -> Option<String> {
        self.last_error.take()
    }
}

impl Iyagi for Gate0ScriptIyagi {
    fn run_startup(&mut self, _world: &ddonirang_core::platform::NuriWorld) -> Patch {
        Patch::default()
    }

    fn run_update(
        &mut self,
        world: &ddonirang_core::platform::NuriWorld,
        input: &ddonirang_core::platform::InputSnapshot,
    ) -> Patch {
        let mut ops = Vec::new();
        if input.tick_id == 0 && !self.startup_ops.is_empty() {
            ops.extend(self.startup_ops.clone());
        }
        match self.runner.run_update(world, input, &self.defaults) {
            Ok(out) => ops.extend(out.patch.ops),
            Err(err) => {
                self.last_error = Some(err);
            }
        }
        Patch {
            ops,
            origin: Origin::system("ddn"),
        }
    }
}

fn run_once_from(file_path: Option<&str>) -> Result<(), String> {
    ensure_detmath_ready()?;
    let mut buf = Vec::new();
    if let Some(path) = file_path {
        let mut f = std::fs::File::open(path).map_err(|e| format!("파일 열기 실패: {e}"))?;
        f.read_to_end(&mut buf).map_err(|e| format!("파일 읽기 실패: {e}"))?;
    } else {
        io::stdin()
            .read_to_end(&mut buf)
            .map_err(|e| format!("입력 읽기 실패: {e}"))?;
    }
    let input = decode_stdin(&buf);
    let line = input.trim().to_string();

    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = InputIyagi { line: line.clone() };
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let frame = loop_.tick_once(0, &mut sink);
    let len = loop_
        .nuri
        .world()
        .get_resource_fixed64("입력길이")
        .unwrap_or(Fixed64::ZERO);

    let canonical = loop_
        .nuri
        .world()
        .get_resource_fixed64("입력길이")
        .unwrap_or(Fixed64::ZERO);

    println!("입력: {}", line);
    println!("입력길이: {}", len);
    println!("입력길이(누리): {}", canonical);
    println!("state_hash: {}", frame.state_hash.to_hex());
    println!("signals: {}", sink.signals.len());
    write_diag_if_any(&sink)?;
    Ok(())
}
fn preprocess_ai_file(input_path: &str, output_path: Option<&str>) -> Result<(), String> {
    let source = read_text_from_path(input_path)?;
    let schema_hash = read_schema_hash();
    let meta = AiMeta::default_with_schema(schema_hash);
    let output = preprocess_ai_calls(&source, &meta)?;

    let out_path = output_path
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{input_path}.gen.ddn"));
    std::fs::write(&out_path, output.as_bytes())
        .map_err(|e| format!("AI_PREPROCESS_WRITE_FAILED: {}", e))?;
    println!("preprocess_ai_written: {}", out_path);
    Ok(())
}

fn build_schema_file(input_path: &str, output_path: Option<&str>) -> Result<(), String> {
    let source = read_text_from_path(input_path)?;
    let schema = build_schema(&source, input_path)?;
    let out_path = output_path
        .map(|s| s.to_string())
        .unwrap_or_else(|| "ddn.schema.json".to_string());
    let schema_hash = write_schema(&out_path, &schema)?;
    println!("schema_written: {}", out_path);
    println!("schema_hash: {}", schema_hash);
    Ok(())
}

fn read_schema_hash() -> Option<String> {
    let path = std::path::Path::new("ddn.schema.json");
    if !path.exists() {
        return None;
    }
    let data = std::fs::read(path).ok()?;
    Some(blake3::hash(&data).to_hex().to_string())
}

fn run_div0() -> Result<(), String> {
    ensure_detmath_ready()?;
    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = Div0Iyagi;
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let frame = loop_.tick_once(0, &mut sink);
    let x = loop_
        .nuri
        .world()
        .get_resource_fixed64("x")
        .unwrap_or(Fixed64::ZERO);

    println!("x: {}", x);
    println!("state_hash: {}", frame.state_hash.to_hex());
    println!("signals: {}", sink.signals.len());
    write_diag_if_any(&sink)?;
    Ok(())
}

struct AlrimEcho;

impl AlrimHandler for AlrimEcho {
    fn on_signal(&mut self, signal: &Signal, out: &mut dyn ddonirang_core::SignalSink) {
        out.emit(signal.clone());
    }
}

fn run_alrim_demo() -> Result<(), String> {
    ensure_detmath_ready()?;
    let mut loop_ = AlrimLoop::new();
    let mut logger = VecAlrimLogger::default();
    let mut handler = AlrimEcho;

    loop_.run_tick(0, vec![Signal::Alrim { name: "연쇄" }], &mut handler, &mut logger);

    for entry in &logger.entries {
        println!(
            "madi: {} pass: {} signal: {} carry: {}",
            entry.tick_id, entry.pass_index, entry.name, entry.carried
        );
    }
    println!("carryover: {}", loop_.carryover_len());
    Ok(())
}

fn run_grid() -> Result<(), String> {
    ensure_detmath_ready()?;
    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = GridIyagi;
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = ConsoleBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let frame = loop_.tick_once(0, &mut sink);
    println!("state_hash: {}", frame.state_hash.to_hex());
    println!("signals: {}", sink.signals.len());
    write_diag_if_any(&sink)?;
    Ok(())
}

fn parse_maze(text: &str) -> Result<(MazeData, i64, i64), String> {
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim_end().is_empty()).collect();
    if lines.is_empty() {
        return Err("맵이 비었습니다".to_string());
    }
    let mut width = 0usize;
    for line in &lines {
        width = width.max(line.chars().count());
    }
    if width == 0 {
        return Err("맵 너비가 0입니다".to_string());
    }

    let height = lines.len();
    let mut tiles = vec![vec!['.'; width]; height];
    let mut start = None;
    let mut goal = None;
    let mut coins = Vec::new();

    for (y, line) in lines.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            let tile = match ch {
                '#' => '#',
                'G' => {
                    goal = Some((x as i64, y as i64));
                    'G'
                }
                'P' => {
                    start = Some((x as i64, y as i64));
                    '.'
                }
                'C' => {
                    coins.push((x as i64, y as i64));
                    '.'
                }
                '.' | ' ' => '.',
                _ => '.',
            };
            tiles[y][x] = tile;
        }
    }

    let (sx, sy) = start.ok_or_else(|| "플레이어 시작점(P)이 없습니다".to_string())?;
    let data = MazeData {
        width: width as i64,
        height: height as i64,
        tiles,
        goal,
        coins,
    };
    Ok((data, sx, sy))
}

fn map_text_from_data(data: &MazeData) -> String {
    let mut out = String::new();
    for y in 0..data.height {
        for x in 0..data.width {
            let mut ch = data.tiles[y as usize][x as usize];
            if let Some((gx, gy)) = data.goal {
                if gx == x && gy == y {
                    ch = 'G';
                }
            }
            if data.coins.iter().any(|(cx, cy)| *cx == x && *cy == y) {
                ch = 'C';
            }
            out.push(ch);
        }
        if y + 1 < data.height {
            out.push('\n');
        }
    }
    out
}

fn read_fixed64_from_context(
    name: &str,
    resources: &HashMap<String, Value>,
    world: &ddonirang_core::platform::NuriWorld,
    defaults: &HashMap<String, Value>,
) -> Option<Fixed64> {
    if let Some(Value::Fixed64(v)) = resources.get(name) {
        return Some(*v);
    }
    if let Some(Value::Unit(unit)) = resources.get(name) {
        if unit.is_dimensionless() {
            return Some(unit.value);
        }
    }
    if let Some(v) = world.get_resource_fixed64(name) {
        return Some(v);
    }
    if let Some(Value::Fixed64(v)) = defaults.get(name) {
        return Some(*v);
    }
    if let Some(Value::Unit(unit)) = defaults.get(name) {
        if unit.is_dimensionless() {
            return Some(unit.value);
        }
    }
    None
}

fn read_text_from_context(
    name: &str,
    resources: &HashMap<String, Value>,
    world: &ddonirang_core::platform::NuriWorld,
    defaults: &HashMap<String, Value>,
) -> Option<String> {
    if let Some(Value::String(v)) = resources.get(name) {
        return Some(v.clone());
    }
    if let Some(v) = world.get_resource_json(name) {
        return Some(v);
    }
    if let Some(Value::String(v)) = defaults.get(name) {
        return Some(v.clone());
    }
    None
}

fn read_text_from_path(path: &str) -> Result<String, String> {
    let mut buf = Vec::new();
    let mut f = std::fs::File::open(path).map_err(|e| format!("파일 열기 실패: {e}"))?;
    f.read_to_end(&mut buf).map_err(|e| format!("파일 읽기 실패: {e}"))?;
    Ok(decode_stdin(&buf))
}

#[cfg(test)]
fn read_net_events_detjson(path: &str) -> Result<Vec<ddonirang_core::platform::NetEvent>, String> {
    let text = read_text_from_path(path)?;
    let snapshot: InputSnapshotDetJson =
        serde_json::from_str(&text).map_err(|e| format!("입력샘 파싱 실패: {e}"))?;
    if let Some(schema) = snapshot.schema.as_deref() {
        if schema != "ddn.input_snapshot.v1" {
            return Err(format!(
                "E_NET_SAM_SCHEMA 입력샘 schema 불일치: {schema}"
            ));
        }
    }
    let mut events = Vec::new();
    for event in snapshot.net_events {
        let payload_detjson =
            serde_json::to_string(&event.payload).map_err(|e| format!("입력샘 payload 직렬화 실패: {e}"))?;
        events.push(ddonirang_core::platform::NetEvent {
            sender: event.sender,
            seq: event.seq,
            order_key: event.order_key,
            payload_detjson,
        });
    }
    events.sort();
    Ok(events)
}

fn read_text_from_stdin() -> Result<String, String> {
    let mut buf = Vec::new();
    io::stdin()
        .read_to_end(&mut buf)
        .map_err(|e| format!("입력 읽기 실패: {e}"))?;
    Ok(decode_stdin(&buf))
}

fn format_parse_error_basic(source: &str, err: &ddonirang_lang::ParseError) -> String {
    let mut line = 1usize;
    let mut col = 1usize;
    let mut line_start = 0usize;
    for (idx, ch) in source.char_indices() {
        if idx >= err.span.start {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
            line_start = idx + ch.len_utf8();
        } else {
            col += 1;
        }
    }
    let line_end = source[line_start..]
        .find('\n')
        .map(|offset| line_start + offset)
        .unwrap_or(source.len());
    let line_text = &source[line_start..line_end];
    let caret = " ".repeat(col.saturating_sub(1)) + "^";
    format!("파싱 실패: {} ({}:{})\n{}\n{}", err.message, line, col, line_text, caret)
}

#[derive(Debug, Clone, Copy)]
enum CanonBridge {
    Age0Step01,
}

impl CanonBridge {
    fn parse(input: &str) -> Result<Self, String> {
        match input {
            "age0_step01" => Ok(Self::Age0Step01),
            _ => Err(format!("알 수 없는 bridge 값: {input}")),
        }
    }
}

fn apply_canon_bridge(source: &str, bridge: CanonBridge) -> Result<String, String> {
    match bridge {
        CanonBridge::Age0Step01 => apply_age0_step01_bridge(source),
    }
}

fn apply_age0_step01_bridge(source: &str) -> Result<String, String> {
    let mut declared = HashSet::new();
    let mut out = String::new();
    let lines: Vec<&str> = source.split('\n').collect();
    for (idx, line) in lines.iter().enumerate() {
        let line_no = idx + 1;
        let processed = apply_age0_step01_line(line, line_no, &mut declared)?;
        out.push_str(&processed);
        if idx + 1 < lines.len() {
            out.push('\n');
        }
    }
    Ok(out)
}

fn apply_age0_step01_line(
    line: &str,
    line_no: usize,
    declared: &mut HashSet<String>,
) -> Result<String, String> {
    let (code_raw, comment) = split_line_comment(line);
    if code_raw.trim().is_empty() {
        return Ok(line.to_string());
    }

    let trimmed_len = code_raw
        .trim_end_matches(|c| c == ' ' || c == '\t')
        .len();
    let trailing_ws = &code_raw[trimmed_len..];
    let mut code = code_raw[..trimmed_len].to_string();

    if let Some((converted, name)) = try_convert_simple_decl(&code) {
        code = converted;
        declared.insert(name);
    }

    code = remove_type_prefixes(&code, line_no)?;

    let code_trimmed = code.trim_end();
    if !code_trimmed.is_empty()
        && !code_trimmed.ends_with('.')
        && !code_trimmed.ends_with('?')
        && !code_trimmed.ends_with('!')
        && !code_trimmed.ends_with('{')
        && !code_trimmed.ends_with('}')
    {
        code.push('.');
    }

    let mut rebuilt = String::new();
    rebuilt.push_str(&code);
    rebuilt.push_str(trailing_ws);
    rebuilt.push_str(&comment);
    Ok(rebuilt)
}

fn split_line_comment(line: &str) -> (String, String) {
    let mut in_string = false;
    let mut escaped = false;
    let mut chars = line.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
            continue;
        }
        if ch == '/' {
            if let Some((_, next_ch)) = chars.peek() {
                if *next_ch == '/' {
                    return (line[..idx].to_string(), line[idx..].to_string());
                }
            }
        }
    }
    (line.to_string(), String::new())
}

fn try_convert_simple_decl(code: &str) -> Option<(String, String)> {
    let trimmed = code.trim_start();
    if trimmed.starts_with('(') {
        return None;
    }
    let indent_len = code.len() - trimmed.len();
    let eq_index = find_single_equals(trimmed)?;
    let lhs = trimmed[..eq_index].trim();
    let rhs = trimmed[eq_index + 1..].trim();
    if rhs.is_empty() || rhs.starts_with('{') {
        return None;
    }
    if lhs.contains('(') || lhs.contains(')') {
        return None;
    }
    let mut parts = lhs.splitn(2, ':');
    let name = parts.next()?.trim();
    let ty = parts.next()?.trim();
    if name.is_empty() || ty.is_empty() {
        return None;
    }
    if !is_ident(name) || !is_ident(ty) {
        return None;
    }
    let indent = &code[..indent_len];
    let converted = format!("{indent}살림.{name} <- {rhs}");
    Some((converted, name.to_string()))
}

fn find_single_equals(text: &str) -> Option<usize> {
    let mut in_string = false;
    let mut escaped = false;
    let mut prev: Option<char> = None;
    let mut chars = text.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
            continue;
        }
        if ch == '=' {
            let next = chars.peek().map(|(_, c)| *c);
            if matches!(prev, Some('=') | Some('<') | Some('>') | Some('!')) {
                prev = Some(ch);
                continue;
            }
            if next == Some('=') {
                prev = Some(ch);
                continue;
            }
            return Some(idx);
        }
        prev = Some(ch);
    }
    None
}

fn remove_type_prefixes(code: &str, line_no: usize) -> Result<String, String> {
    let mut out = String::new();
    let chars: Vec<char> = code.chars().collect();
    let mut i = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    while i < chars.len() {
        let ch = chars[i];
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if ch == '"' {
            in_string = true;
            out.push(ch);
            i += 1;
            continue;
        }
        if (ch == '글' || ch == '수')
            && (i == 0 || !is_ident_char(chars[i - 1]))
            && i + 1 < chars.len()
            && chars[i + 1].is_whitespace()
        {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j >= chars.len() {
                return Err(format!("BRIDGE_TYPE_PREFIX_FAIL: {line_no}행 끝에서 타입 접두 사용"));
            }
            let next = chars[j];
            if ch == '글' {
                if next != '"' {
                    return Err(format!(
                        "BRIDGE_TYPE_PREFIX_MISMATCH: {line_no}행 글 접두 뒤에 문자열이 없습니다"
                    ));
                }
            } else if !(next.is_ascii_digit() || next == '-' || next == '+') {
                return Err(format!(
                    "BRIDGE_TYPE_PREFIX_MISMATCH: {line_no}행 수 접두 뒤에 숫자가 없습니다"
                ));
            }
            i = j;
            continue;
        }
        out.push(ch);
        i += 1;
    }
    Ok(out)
}

fn is_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_alphabetic() || first == '_') {
        return false;
    }
    chars.all(is_ident_char)
}

fn is_ident_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn run_canon(
    input_path: &str,
    emit: &str,
    out: Option<&str>,
    check: bool,
    bridge: Option<CanonBridge>,
) -> Result<(), String> {
    if emit != "ddn" {
        return Err(format!("지원하지 않는 --emit 값: {emit}"));
    }
    let source = read_text_from_path(input_path)?;
    let meta_parse = split_file_meta(&source);
    let source_body = match bridge {
        Some(bridge) => apply_canon_bridge(&meta_parse.stripped, bridge)?,
        None => meta_parse.stripped,
    };
    let cleaned = preprocess_source_for_parse(&source_body)?;
    let mut program = parse_with_mode(
        &cleaned,
        input_path,
        ddn_runtime::default_parse_mode(),
    )
    .map_err(|e| format_parse_error_basic(&cleaned, &e))?;
    let report =
        canonicalize(&mut program).map_err(|e| format_parse_error_basic(&cleaned, &e))?;
    for warning in report.warnings {
        eprintln!("{}: {}", warning.code, warning.message);
    }
    let mut output_body = normalize(&program, NormalizationLevel::N1);
    if !output_body.ends_with('\n') {
        output_body.push('\n');
    }
    let cleaned_for_check = strip_leading_lines(&cleaned, meta_parse.meta_lines);
    if check && output_body.trim_end() != cleaned_for_check.trim_end() {
        return Err("CANON_CHECK_FAIL: 정본 출력과 입력이 다릅니다".to_string());
    }
    if !meta_parse.dup_keys.is_empty() {
        eprintln!(
            "warning: META_DUP_KEY {}",
            meta_parse.dup_keys.join(", ")
        );
    }
    let mut output = String::new();
    output.push_str(&format_file_meta(&meta_parse.meta));
    output.push_str(&output_body);

    if let Some(out_path) = out {
        let out_path = Path::new(out_path);
        let final_path = if out_path.is_dir() {
            let stem = Path::new(input_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            out_path.join(format!("{stem}.canon.ddn"))
        } else {
            out_path.to_path_buf()
        };
        std::fs::write(&final_path, output.as_bytes())
            .map_err(|e| format!("canon 출력 실패: {e}"))?;
        println!("canon_written: {}", final_path.display());
    } else {
        let mut stdout = io::stdout();
        stdout
            .write_all(output.as_bytes())
            .map_err(|e| format!("canon 출력 실패: {e}"))?;
    }
    Ok(())
}

fn strip_leading_lines(text: &str, count: usize) -> &str {
    if count == 0 {
        return text;
    }
    let mut idx = 0usize;
    let mut skipped = 0usize;
    for chunk in text.split_inclusive('\n') {
        if skipped >= count {
            break;
        }
        idx += chunk.len();
        skipped += 1;
    }
    &text[idx..]
}

fn filter_moves(moves_text: &str) -> Vec<char> {
    moves_text
        .chars()
        .filter(|c| matches!(c.to_ascii_lowercase(), 'w' | 'a' | 's' | 'd' | 'i' | 'j' | 'k' | 'l'))
        .collect()
}

fn frames_to_log(frames: &[ddonirang_core::TickFrame]) -> Result<ReplayLog, String> {
    let mut out = Vec::with_capacity(frames.len());
    for frame in frames {
        if !frame.snapshot.ai_injections.is_empty() {
            return Err("ai_injections 기록은 아직 지원하지 않습니다".to_string());
        }
        let mut net_events = Vec::new();
        for event in &frame.snapshot.net_events {
            let payload = serde_json::from_str(&event.payload_detjson)
                .map_err(|e| format!("net_events payload 파싱 실패: {e}"))?;
            net_events.push(NetEventJson {
                sender: event.sender.clone(),
                seq: event.seq,
                order_key: event.order_key.clone(),
                payload,
            });
        }
        out.push(ReplayFrame {
            tick_id: frame.snapshot.tick_id,
            dt_raw: frame.snapshot.dt.raw_i64(),
            keys_pressed: frame.snapshot.keys_pressed,
            last_key_name: frame.snapshot.last_key_name.clone(),
            pointer_x_i32: frame.snapshot.pointer_x_i32,
            pointer_y_i32: frame.snapshot.pointer_y_i32,
            net_events,
            rng_seed: frame.snapshot.rng_seed,
            state_hash: frame.state_hash.to_hex(),
        });
    }
    Ok(ReplayLog { frames: out })
}

fn log_to_snapshots(log: &ReplayLog) -> Vec<(InputSnapshot, String)> {
    log.frames
        .iter()
        .map(|frame| {
            let net_events = frame
                .net_events
                .iter()
                .map(|event| ddonirang_core::platform::NetEvent {
                    sender: event.sender.clone(),
                    seq: event.seq,
                    order_key: event.order_key.clone(),
                    payload_detjson: serde_json::to_string(&event.payload)
                        .expect("net_events payload serialize"),
                })
                .collect();
            let snapshot = InputSnapshot {
                tick_id: frame.tick_id,
                dt: Fixed64::from_raw_i64(frame.dt_raw),
                keys_pressed: frame.keys_pressed,
                last_key_name: frame.last_key_name.clone(),
                pointer_x_i32: frame.pointer_x_i32,
                pointer_y_i32: frame.pointer_y_i32,
                ai_injections: Vec::new(),
                net_events,
                rng_seed: frame.rng_seed,
            };
            (snapshot, frame.state_hash.clone())
        })
        .collect()
}

fn write_replay_log(path: &str, log: &ReplayLog) -> Result<(), String> {
    let json =
        serde_json::to_string_pretty(log).map_err(|e| format!("리플레이 로그 직렬화 실패: {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("로그 저장 실패: {e}"))?;
    Ok(())
}

fn read_replay_log(path: &str) -> Result<ReplayLog, String> {
    let text = read_text_from_path(path)?;
    serde_json::from_str(&text).map_err(|e| format!("로그 읽기 실패: {e}"))
}

fn collect_moves(moves_arg: Option<String>) -> Result<Vec<char>, String> {
    let moves_text = if let Some(m) = moves_arg {
        m
    } else {
        read_text_from_stdin()?
    };
    Ok(filter_moves(&moves_text))
}

fn run_maze_file(path: &str, moves_arg: Option<String>) -> Result<(), String> {
    ensure_detmath_ready()?;
    let map_text = read_text_from_path(path)?;
    let (data, start_x, start_y) = parse_maze(&map_text)?;

    let moves = collect_moves(moves_arg)?;

    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = MazeIyagi::new(data, start_x, start_y);
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = ConsoleBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let ticks = if moves.is_empty() { 1 } else { moves.len() };
    let mut last_frame = None;
    for tick in 0..ticks {
        let mv = moves.get(tick).copied();
        loop_.sam.keys_pressed = mv.map(move_to_key_bits).unwrap_or(0);
        loop_.sam.last_key_name = mv.map(move_to_key_name).unwrap_or_default();
        last_frame = Some(loop_.tick_once(tick as u64, &mut sink));
    }
    if let Some(frame) = last_frame {
        println!("state_hash: {}", frame.state_hash.to_hex());
        println!("signals: {}", sink.signals.len());
    }
    write_diag_if_any(&sink)?;
    Ok(())
}

fn run_maze_ddn(map_path: &str, script_path: &str, moves_arg: Option<String>) -> Result<(), String> {
    ensure_detmath_ready()?;
    let map_text = read_text_from_path(map_path)?;
    let script_text = read_text_from_path(script_path)?;
    let (data, start_x, start_y) = parse_maze(&map_text)?;

    let program = DdnProgram::from_source(&script_text, script_path)?;
    let moves = collect_moves(moves_arg)?;

    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = DdnMazeIyagi::new(data, start_x, start_y, program);
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = ConsoleBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let ticks = if moves.is_empty() { 1 } else { moves.len() };
    let mut last_frame = None;
    for tick in 0..ticks {
        let mv = moves.get(tick).copied();
        loop_.sam.keys_pressed = mv.map(move_to_key_bits).unwrap_or(0);
        loop_.sam.last_key_name = mv.map(move_to_key_name).unwrap_or_default();
        last_frame = Some(loop_.tick_once(tick as u64, &mut sink));
    }
    if let Some(frame) = last_frame {
        println!("state_hash: {}", frame.state_hash.to_hex());
        println!("signals: {}", sink.signals.len());
    }
    write_diag_if_any(&sink)?;
    Ok(())
}

fn run_maze_ddn_live(map_path: &str, script_path: &str) -> Result<(), String> {
    ensure_detmath_ready()?;
    let map_text = read_text_from_path(map_path)?;
    let script_text = read_text_from_path(script_path)?;
    let (data, start_x, start_y) = parse_maze(&map_text)?;

    let program = DdnProgram::from_source(&script_text, script_path)?;

    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = DdnMazeIyagi::new(data, start_x, start_y, program);
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = LiveBogae::new();

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();
    let mut stdout = io::stdout();
    let _guard = LiveTerminalGuard::enter(&mut stdout)?;

    let mut tick = 0u64;
    let mut last_frame = None;
    loop {
        let (key_bits, quit, key_name) = read_key_for_tick(Duration::from_millis(120))?;
        if quit {
            break;
        }
        loop_.sam.keys_pressed = key_bits;
        loop_.sam.last_key_name = key_name;
        last_frame = Some(loop_.tick_once(tick, &mut sink));
        tick += 1;
        if loop_
            .nuri
            .world()
            .get_resource_json("메시지")
            .as_deref()
            == Some("도착")
        {
            break;
        }
    }

    drop(_guard);
    if let Some(frame) = last_frame {
        println!("state_hash: {}", frame.state_hash.to_hex());
        println!("signals: {}", sink.signals.len());
    }
    write_diag_if_any(&sink)?;
    Ok(())
}

fn run_maze_replay(path: &str, moves_arg: Option<String>) -> Result<(), String> {
    ensure_detmath_ready()?;
    let map_text = read_text_from_path(path)?;
    let (data, start_x, start_y) = parse_maze(&map_text)?;
    let moves = collect_moves(moves_arg)?;

    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = MazeIyagi::new(data.clone(), start_x, start_y);
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let ticks = if moves.is_empty() { 1 } else { moves.len() };
    for tick in 0..ticks {
        let mv = moves.get(tick).copied();
        loop_.sam.keys_pressed = mv.map(move_to_key_bits).unwrap_or(0);
        loop_.sam.last_key_name = mv.map(move_to_key_name).unwrap_or_default();
        loop_.tick_once(tick as u64, &mut sink);
    }

    let mut recorded = Vec::new();
    while let Some(frame) = loop_.geoul.replay_next() {
        recorded.push(frame);
    }
    if recorded.is_empty() {
        return Err("리플레이할 기록이 없습니다".to_string());
    }
    write_diag_if_any(&sink)?;

    let expected: Vec<(TickId, String)> = recorded
        .iter()
        .map(|frame| (frame.snapshot.tick_id, frame.state_hash.to_hex()))
        .collect();
    let snapshots: Vec<InputSnapshot> = recorded.iter().map(|frame| frame.snapshot.clone()).collect();

    let sam = ReplaySam::new(snapshots);
    let iyagi = MazeIyagi::new(data, start_x, start_y);
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let mut mismatches = Vec::new();
    for (idx, (tick_id, expected_hash)) in expected.iter().enumerate() {
        let frame = loop_.tick_once(*tick_id, &mut sink);
        let actual_hash = frame.state_hash.to_hex();
        if actual_hash != *expected_hash {
            mismatches.push((idx, *tick_id, expected_hash.clone(), actual_hash));
        }
    }

    println!("replay_ok: {}", mismatches.is_empty());
    println!("mismatches: {}", mismatches.len());
    if let Some((_, tick_id, expected_hash, actual_hash)) = mismatches.first() {
        println!("first_mismatch_tick: {tick_id}");
        println!("expected_hash: {expected_hash}");
        println!("actual_hash: {actual_hash}");
    }
    write_diag_if_any(&sink)?;
    Ok(())
}

fn run_maze_record(path: &str, log_path: &str, moves_arg: Option<String>) -> Result<(), String> {
    ensure_detmath_ready()?;
    let map_text = read_text_from_path(path)?;
    let (data, start_x, start_y) = parse_maze(&map_text)?;
    let moves = collect_moves(moves_arg)?;

    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = MazeIyagi::new(data, start_x, start_y);
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let ticks = if moves.is_empty() { 1 } else { moves.len() };
    for tick in 0..ticks {
        let mv = moves.get(tick).copied();
        loop_.sam.keys_pressed = mv.map(move_to_key_bits).unwrap_or(0);
        loop_.sam.last_key_name = mv.map(move_to_key_name).unwrap_or_default();
        loop_.tick_once(tick as u64, &mut sink);
    }

    let mut recorded = Vec::new();
    while let Some(frame) = loop_.geoul.replay_next() {
        recorded.push(frame);
    }
    let log = frames_to_log(&recorded)?;
    write_replay_log(log_path, &log)?;

    println!("recorded_frames: {}", recorded.len());
    println!("log_path: {}", log_path);
    write_diag_if_any(&sink)?;
    Ok(())
}

fn run_maze_replay_file(path: &str, log_path: &str) -> Result<(), String> {
    ensure_detmath_ready()?;
    let map_text = read_text_from_path(path)?;
    let (data, start_x, start_y) = parse_maze(&map_text)?;
    let log = read_replay_log(log_path)?;

    let entries = log_to_snapshots(&log);
    if entries.is_empty() {
        return Err("리플레이할 기록이 없습니다".to_string());
    }

    let snapshots: Vec<InputSnapshot> = entries.iter().map(|(s, _)| s.clone()).collect();
    let sam = ReplaySam::new(snapshots);
    let iyagi = MazeIyagi::new(data, start_x, start_y);
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let mut mismatches = Vec::new();
    for (idx, (snapshot, expected_hash)) in entries.iter().enumerate() {
        let frame = loop_.tick_once(snapshot.tick_id, &mut sink);
        let actual_hash = frame.state_hash.to_hex();
        if actual_hash != *expected_hash {
            mismatches.push((idx, snapshot.tick_id, expected_hash.clone(), actual_hash));
        }
    }

    println!("replay_ok: {}", mismatches.is_empty());
    println!("mismatches: {}", mismatches.len());
    if let Some((_, tick_id, expected_hash, actual_hash)) = mismatches.first() {
        println!("first_mismatch_tick: {tick_id}");
        println!("expected_hash: {expected_hash}");
        println!("actual_hash: {actual_hash}");
    }
    write_diag_if_any(&sink)?;
    Ok(())
}

fn compute_maze_hashes(path: &str, moves_text: &str) -> Result<Vec<String>, String> {
    let map_text = read_text_from_path(path)?;
    let (data, start_x, start_y) = parse_maze(&map_text)?;
    let moves = filter_moves(moves_text);

    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = MazeIyagi::new(data, start_x, start_y);
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let ticks = if moves.is_empty() { 1 } else { moves.len() };
    let mut hashes = Vec::with_capacity(ticks);
    for tick in 0..ticks {
        let mv = moves.get(tick).copied();
        loop_.sam.keys_pressed = mv.map(move_to_key_bits).unwrap_or(0);
        loop_.sam.last_key_name = mv.map(move_to_key_name).unwrap_or_default();
        let frame = loop_.tick_once(tick as u64, &mut sink);
        hashes.push(frame.state_hash.to_hex());
    }
    Ok(hashes)
}

fn compute_parabola_hashes(ticks: u64, vx: i64, vy: i64) -> Vec<String> {
    let dt = fixed64_ratio(1, 10);
    let g = fixed64_ratio(98, 10);

    let sam = DetSam::new(dt);
    let iyagi = ParabolaIyagi::new(Fixed64::from_i64(vx), Fixed64::from_i64(vy), g);
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();
    let mut hashes = Vec::with_capacity(ticks as usize);

    for tick in 0..ticks {
        let frame = loop_.tick_once(tick, &mut sink);
        hashes.push(frame.state_hash.to_hex());
    }
    hashes
}

fn run_gate0_script_hashes(iyagi: Gate0ScriptIyagi, ticks: u64) -> Result<Vec<String>, String> {
    let sam = DetSam::new(Fixed64::from_i64(1));
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();
    let mut hashes = Vec::with_capacity(ticks as usize);

    for tick in 0..ticks {
        let frame = loop_.tick_once(tick, &mut sink);
        hashes.push(frame.state_hash.to_hex());
    }

    if let Some(err) = loop_.iyagi.take_error() {
        return Err(err);
    }
    Ok(hashes)
}

fn compute_gate0_pipe_hashes(script_path: &str) -> Result<Vec<String>, String> {
    let source = read_text_from_path(script_path)?;
    let program = DdnProgram::from_source(&source, script_path)?;
    let mut defaults = HashMap::new();
    defaults.insert("결과".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
    let iyagi = Gate0ScriptIyagi::new(program, defaults, Vec::new());
    run_gate0_script_hashes(iyagi, 1)
}

fn compute_gate0_unit_hashes(script_path: &str) -> Result<Vec<String>, String> {
    let source = read_text_from_path(script_path)?;
    let program = DdnProgram::from_source(&source, script_path)?;
    let mut defaults = HashMap::new();
    defaults.insert("결과".to_string(), Value::Fixed64(Fixed64::from_i64(7)));
    defaults.insert(
        "입력값".to_string(),
        Value::Unit(ddonirang_core::UnitValue::new(
            Fixed64::from_i64(1),
            Unit::Second,
        )),
    );
    let startup_ops = vec![PatchOp::SetResourceFixed64 {
        tag: "결과".to_string(),
        value: Fixed64::from_i64(7),
    }];
    let iyagi = Gate0ScriptIyagi::new(program, defaults, startup_ops);
    run_gate0_script_hashes(iyagi, 1)
}

fn compute_gate0_fault_hashes() -> Vec<String> {
    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = Div0Iyagi;
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();
    let frame = loop_.tick_once(0, &mut sink);
    vec![frame.state_hash.to_hex()]
}

fn default_golden_file() -> GoldenFile {
    GoldenFile {
        version: 2,
        scenarios: vec![
            GoldenScenario {
                name: "maze".to_string(),
                kind: GoldenScenarioKind::Maze {
                    map: DEFAULT_MAZE_MAP_PATH.to_string(),
                    moves: "DDDDSS".to_string(),
                },
                hashes: Vec::new(),
            },
            GoldenScenario {
                name: "coin_maze".to_string(),
                kind: GoldenScenarioKind::Maze {
                    map: DEFAULT_COIN_MAZE_MAP_PATH.to_string(),
                    moves: "DDDDSS".to_string(),
                },
                hashes: Vec::new(),
            },
            GoldenScenario {
                name: "parabola".to_string(),
                kind: GoldenScenarioKind::Parabola {
                    ticks: 20,
                    vx: 5,
                    vy: 10,
                },
                hashes: Vec::new(),
            },
            GoldenScenario {
                name: "gate0_pipe".to_string(),
                kind: GoldenScenarioKind::Gate0Pipe {
                    script: "docs/steps/000/artifacts/gate0_pipe.ddn".to_string(),
                },
                hashes: Vec::new(),
            },
            GoldenScenario {
                name: "gate0_unit".to_string(),
                kind: GoldenScenarioKind::Gate0Unit {
                    script: "docs/steps/000/artifacts/gate0_unit.ddn".to_string(),
                },
                hashes: Vec::new(),
            },
            GoldenScenario {
                name: "gate0_fault".to_string(),
                kind: GoldenScenarioKind::Gate0Fault,
                hashes: Vec::new(),
            },
        ],
    }
}

fn run_golden(path: &str) -> Result<(), String> {
    ensure_detmath_ready()?;
    let text = read_text_from_path(path)?;
    let golden: GoldenFile =
        serde_json::from_str(&text).map_err(|e| format!("골든 파일 읽기 실패: {e}"))?;

    let mut mismatches = 0usize;
    for scenario in golden.scenarios {
        let expected = scenario.hashes;
        let actual = match &scenario.kind {
            GoldenScenarioKind::Maze { map, moves } => compute_maze_hashes(map, moves)?,
            GoldenScenarioKind::Parabola { ticks, vx, vy } => {
                compute_parabola_hashes(*ticks, *vx, *vy)
            }
            GoldenScenarioKind::Gate0Pipe { script } => compute_gate0_pipe_hashes(script)?,
            GoldenScenarioKind::Gate0Unit { script } => compute_gate0_unit_hashes(script)?,
            GoldenScenarioKind::Gate0Fault => compute_gate0_fault_hashes(),
        };
        if expected != actual {
            mismatches += 1;
            println!("golden_mismatch: {}", scenario.name);
            println!("expected_len: {}", expected.len());
            println!("actual_len: {}", actual.len());
            if let Some((idx, expected_hash, actual_hash)) =
                expected.iter().zip(actual.iter()).enumerate().find_map(|(idx, (e, a))| {
                    if e != a { Some((idx, e, a)) } else { None }
                })
            {
                println!("first_mismatch_tick: {}", idx);
                println!("expected_hash: {}", expected_hash);
                println!("actual_hash: {}", actual_hash);
            }
        }
    }

    println!("golden_ok: {}", mismatches == 0);
    println!("golden_mismatches: {}", mismatches);
    Ok(())
}

fn run_golden_update(path: &str) -> Result<(), String> {
    ensure_detmath_ready()?;
    let mut golden = default_golden_file();
    for scenario in &mut golden.scenarios {
        scenario.hashes = match &scenario.kind {
            GoldenScenarioKind::Maze { map, moves } => compute_maze_hashes(map, moves)?,
            GoldenScenarioKind::Parabola { ticks, vx, vy } => {
                compute_parabola_hashes(*ticks, *vx, *vy)
            }
            GoldenScenarioKind::Gate0Pipe { script } => compute_gate0_pipe_hashes(script)?,
            GoldenScenarioKind::Gate0Unit { script } => compute_gate0_unit_hashes(script)?,
            GoldenScenarioKind::Gate0Fault => compute_gate0_fault_hashes(),
        };
    }
    let json = serde_json::to_string_pretty(&golden)
        .map_err(|e| format!("골든 파일 직렬화 실패: {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("골든 파일 저장 실패: {e}"))?;
    println!("golden_written: {}", path);
    Ok(())
}

fn run_parabola(ticks_arg: Option<String>, vx_arg: Option<String>, vy_arg: Option<String>) -> Result<(), String> {
    ensure_detmath_ready()?;
    let ticks = parse_u64_arg(ticks_arg, 20, "ticks")?;
    let vx = parse_i64_arg(vx_arg, 5, "vx")?;
    let vy = parse_i64_arg(vy_arg, 10, "vy")?;

    let dt = fixed64_ratio(1, 10);
    let g = fixed64_ratio(98, 10);

    let sam = DetSam::new(dt);
    let iyagi = ParabolaIyagi::new(Fixed64::from_i64(vx), Fixed64::from_i64(vy), g);
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let tag_x = resource_tag_with_unit("x", Unit::Meter);
    let tag_y = resource_tag_with_unit("y", Unit::Meter);
    let tag_t = resource_tag_with_unit("t", Unit::Second);

    for tick in 0..ticks {
        let frame = loop_.tick_once(tick, &mut sink);
        let world = loop_.nuri.world();
        let x = world.get_resource_fixed64(&tag_x).unwrap_or(Fixed64::ZERO);
        let y = world.get_resource_fixed64(&tag_y).unwrap_or(Fixed64::ZERO);
        let t = world.get_resource_fixed64(&tag_t).unwrap_or(Fixed64::ZERO);
        println!(
            "tick: {tick} t: {t} x: {x} y: {y} state_hash: {}",
            frame.state_hash.to_hex()
        );
    }
    write_diag_if_any(&sink)?;
    Ok(())
}

fn run_maze_live(path: &str) -> Result<(), String> {
    ensure_detmath_ready()?;
    let map_text = read_text_from_path(path)?;
    let (data, start_x, start_y) = parse_maze(&map_text)?;

    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = MazeIyagi::new(data, start_x, start_y);
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = LiveBogae::new();

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();
    let mut stdout = io::stdout();
    let _guard = LiveTerminalGuard::enter(&mut stdout)?;

    let mut tick = 0u64;
    let mut last_frame = None;
    loop {
        let (key_bits, quit, key_name) = read_key_for_tick(Duration::from_millis(120))?;
        if quit {
            break;
        }
        loop_.sam.keys_pressed = key_bits;
        loop_.sam.last_key_name = key_name;
        last_frame = Some(loop_.tick_once(tick, &mut sink));
        tick += 1;
        if loop_
            .nuri
            .world()
            .get_resource_json("메시지")
            .as_deref()
            == Some("도착")
        {
            break;
        }
    }

    drop(_guard);
    if let Some(frame) = last_frame {
        println!("state_hash: {}", frame.state_hash.to_hex());
        println!("signals: {}", sink.signals.len());
    }
    write_diag_if_any(&sink)?;
    Ok(())
}

fn move_to_key_bits(mv: char) -> u64 {
    match mv.to_ascii_lowercase() {
        'w' => KEY_W,
        'a' => KEY_A,
        's' => KEY_S,
        'd' => KEY_D,
        'i' => KEY_W,
        'j' => KEY_A,
        'k' => KEY_S,
        'l' => KEY_D,
        _ => 0,
    }
}

fn move_to_key_name(mv: char) -> String {
    mv.to_ascii_lowercase().to_string()
}

fn fixed64_ratio(n: i64, d: i64) -> Fixed64 {
    if d == 0 {
        return Fixed64::ZERO;
    }
    Fixed64::from_i64(n)
        .try_div(Fixed64::from_i64(d))
        .unwrap_or(Fixed64::ZERO)
}

fn parse_u64_arg(arg: Option<String>, default: u64, name: &str) -> Result<u64, String> {
    match arg {
        Some(v) => v.parse::<u64>().map_err(|_| format!("{name}는 정수여야 합니다")),
        None => Ok(default),
    }
}

fn parse_i64_arg(arg: Option<String>, default: i64, name: &str) -> Result<i64, String> {
    match arg {
        Some(v) => v.parse::<i64>().map_err(|_| format!("{name}는 정수여야 합니다")),
        None => Ok(default),
    }
}

fn read_key_for_tick(timeout: Duration) -> Result<(u64, bool, String), String> {
    let mut key_bits = 0;
    let mut quit = false;
    let mut last_key_name = String::new();
    if event::poll(timeout).map_err(|e| format!("키 입력 대기 실패: {e}"))? {
        loop {
            match event::read().map_err(|e| format!("키 입력 읽기 실패: {e}"))? {
                event::Event::Key(key) => {
                    if !matches!(
                        key.kind,
                        event::KeyEventKind::Press | event::KeyEventKind::Repeat
                    ) {
                        continue;
                    }
                    if is_quit_key(key.code) {
                        quit = true;
                    }
                    let bits = keycode_to_bits(key.code);
                    if bits != 0 {
                        key_bits = bits;
                    }
                    if let Some(name) = keycode_to_name(key.code) {
                        last_key_name = name;
                    }
                }
                _ => {}
            }
            if !event::poll(Duration::from_millis(0)).map_err(|e| format!("키 입력 대기 실패: {e}"))? {
                break;
            }
        }
    }
    Ok((key_bits, quit, last_key_name))
}

fn is_quit_key(code: event::KeyCode) -> bool {
    match code {
        event::KeyCode::Esc => true,
        event::KeyCode::Char(c) => c.to_ascii_lowercase() == 'q',
        _ => false,
    }
}

fn keycode_to_bits(code: event::KeyCode) -> u64 {
    match code {
        event::KeyCode::Up => KEY_W,
        event::KeyCode::Left => KEY_A,
        event::KeyCode::Down => KEY_S,
        event::KeyCode::Right => KEY_D,
        event::KeyCode::Char(c) => match c.to_ascii_lowercase() {
            'w' => KEY_W,
            'a' => KEY_A,
            's' => KEY_S,
            'd' => KEY_D,
            'i' => KEY_W,
            'j' => KEY_A,
            'k' => KEY_S,
            'l' => KEY_D,
            _ => 0,
        },
        _ => 0,
    }
}

fn keycode_to_name(code: event::KeyCode) -> Option<String> {
    match code {
        event::KeyCode::Up => Some("up".to_string()),
        event::KeyCode::Left => Some("left".to_string()),
        event::KeyCode::Down => Some("down".to_string()),
        event::KeyCode::Right => Some("right".to_string()),
        event::KeyCode::Char(c) => Some(c.to_ascii_lowercase().to_string()),
        _ => None,
    }
}

fn decode_stdin(buf: &[u8]) -> String {
    if buf.starts_with(&[0xFF, 0xFE]) {
        return decode_utf16_le(&buf[2..]);
    }
    if buf.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&buf[3..]).to_string();
    }
    let utf8 = String::from_utf8_lossy(buf);
    if utf8.contains('\u{FFFD}') && buf.len() % 2 == 0 {
        return decode_utf16_le(buf);
    }
    if looks_like_utf16le(buf) {
        return decode_utf16_le(buf);
    }
    utf8.to_string()
}

fn decode_utf16_le(buf: &[u8]) -> String {
    let mut units = Vec::with_capacity(buf.len() / 2);
    for chunk in buf.chunks_exact(2) {
        units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    String::from_utf16_lossy(&units)
}

fn looks_like_utf16le(buf: &[u8]) -> bool {
    if buf.len() < 4 {
        return false;
    }
    let mut zero_odd = 0usize;
    let mut total = 0usize;
    for (i, b) in buf.iter().enumerate() {
        if i % 2 == 1 {
            total += 1;
            if *b == 0 {
                zero_odd += 1;
            }
        }
    }
    total > 0 && zero_odd * 2 >= total
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    if let Some(index) = args.iter().position(|arg| arg == flag) {
        args.remove(index);
        true
    } else {
        false
    }
}

fn take_arg_value(args: &mut Vec<String>, flag: &str) -> Option<String> {
    let prefix = format!("{flag}=");
    if let Some(index) = args.iter().position(|arg| arg.starts_with(&prefix)) {
        let raw = args.remove(index);
        return Some(raw[prefix.len()..].to_string());
    }
    if let Some(index) = args.iter().position(|arg| arg == flag) {
        let value = if index + 1 < args.len() {
            args.remove(index + 1)
        } else {
            String::new()
        };
        args.remove(index);
        if value.is_empty() {
            return None;
        }
        return Some(value);
    }
    None
}

fn parse_lang_mode(raw: Option<&str>) -> Result<ParseMode, String> {
    let Some(raw) = raw else {
        return Ok(ParseMode::Compat);
    };
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "compat" => Ok(ParseMode::Compat),
        "strict" => Ok(ParseMode::Strict),
        _ => Err(format!("지원하지 않는 --lang-mode 값: {raw} (compat|strict)")),
    }
}

fn ensure_feature(policy: Option<&ProjectPolicy>, feature: FeatureGate) -> Result<(), String> {
    if let Some(policy) = policy {
        policy.require_feature(feature)?;
    }
    Ok(())
}

fn main() {
    let mut raw_args: Vec<String> = std::env::args().skip(1).collect();
    let unsafe_compat = take_flag(&mut raw_args, "--unsafe-compat");
    let lang_mode_raw = take_arg_value(&mut raw_args, "--lang-mode");
    let mut args = raw_args.into_iter();
    let cmd = args.next();

    let parse_mode = match parse_lang_mode(lang_mode_raw.as_deref()) {
        Ok(mode) => mode,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };
    ddn_runtime::set_default_parse_mode(parse_mode);
    let requires_project = matches!(
        cmd.as_deref(),
        Some(
            "run-once"
                | "test"
                | "run-div0"
                | "run-grid"
                | "run-alrim-demo"
                | "run-maze-file"
                | "run-maze-ddn"
                | "run-maze-ddn-live"
                | "run-maze-replay"
                | "run-maze-record"
                | "run-maze-replay-file"
                | "run-parabola"
                | "run-golden"
                | "run-golden-update"
                | "run-maze-live"
                | "run-once-file"
                | "preprocess-ai"
                | "build-schema"
        )
    );
    let policy = if requires_project {
        match load_project_policy(unsafe_compat) {
            Ok(policy) => Some(policy),
            Err(err) => {
                eprintln!("{err}");
                return;
            }
        }
    } else {
        None
    };

    match cmd.as_deref() {
        Some("run-once") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            if let Err(err) = run_once_from(None) {
                eprintln!("{err}");
            }
        }
        Some("test") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            let path = args.next();
            if let Err(err) = run_tests(path) {
                eprintln!("{err}");
            }
        }
        Some("run-div0") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            if let Err(err) = run_div0() {
                eprintln!("{err}");
            }
        }
        Some("run-grid") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            if let Err(err) = run_grid() {
                eprintln!("{err}");
            }
        }
        Some("run-alrim-demo") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            if let Err(err) = run_alrim_demo() {
                eprintln!("{err}");
            }
        }
        Some("run-maze-file") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            let path = args.next();
            if path.is_none() {
                eprintln!("사용법: cargo run -p ddonirang-tool -- run-maze-file <맵파일> [이동]");
                return;
            }
            let moves = args.next();
            if let Err(err) = run_maze_file(path.as_deref().unwrap(), moves) {
                eprintln!("{err}");
            }
        }
        Some("run-maze-ddn") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            let map_path = args.next();
            let script_path = args.next();
            if map_path.is_none() || script_path.is_none() {
                eprintln!("사용법: cargo run -p ddonirang-tool -- run-maze-ddn <맵파일> <스크립트> [이동]");
                return;
            }
            let moves = args.next();
            if let Err(err) = run_maze_ddn(map_path.as_deref().unwrap(), script_path.as_deref().unwrap(), moves) {
                eprintln!("{err}");
            }
        }
        Some("run-maze-ddn-live") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            let map_path = args.next();
            let script_path = args.next();
            if map_path.is_none() || script_path.is_none() {
                eprintln!("사용법: cargo run -p ddonirang-tool -- run-maze-ddn-live <맵파일> <스크립트>");
                return;
            }
            if let Err(err) = run_maze_ddn_live(map_path.as_deref().unwrap(), script_path.as_deref().unwrap()) {
                eprintln!("{err}");
            }
        }
        Some("run-maze-replay") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            let path = args.next();
            if path.is_none() {
                eprintln!("사용법: cargo run -p ddonirang-tool -- run-maze-replay <맵파일> [이동]");
                return;
            }
            let moves = args.next();
            if let Err(err) = run_maze_replay(path.as_deref().unwrap(), moves) {
                eprintln!("{err}");
            }
        }
        Some("run-maze-record") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            let path = args.next();
            let log_path = args.next();
            if path.is_none() || log_path.is_none() {
                eprintln!("사용법: cargo run -p ddonirang-tool -- run-maze-record <맵파일> <로그파일> [이동]");
                return;
            }
            let moves = args.next();
            if let Err(err) = run_maze_record(path.as_deref().unwrap(), log_path.as_deref().unwrap(), moves) {
                eprintln!("{err}");
            }
        }
        Some("run-maze-replay-file") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            let path = args.next();
            let log_path = args.next();
            if path.is_none() || log_path.is_none() {
                eprintln!("사용법: cargo run -p ddonirang-tool -- run-maze-replay-file <맵파일> <로그파일>");
                return;
            }
            if let Err(err) = run_maze_replay_file(path.as_deref().unwrap(), log_path.as_deref().unwrap()) {
                eprintln!("{err}");
            }
        }
        Some("geoul") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            let sub = args.next();
            match sub.as_deref() {
                Some("query") => {
                    let query = args.next();
                    if query.is_none() {
                        eprintln!("사용법: cargo run -p ddonirang-tool -- geoul query <질의> [diag_path]");
                        return;
                    }
                    let path = args.next();
                    if let Err(err) = run_geoul_query(query.as_deref().unwrap(), path.as_deref()) {
                        eprintln!("{err}");
                    }
                }
                _ => {
                    eprintln!("사용법: cargo run -p ddonirang-tool -- geoul query <질의> [diag_path]");
                }
            }
        }
        Some("run-parabola") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            let ticks = args.next();
            let vx = args.next();
            let vy = args.next();
            if let Err(err) = run_parabola(ticks, vx, vy) {
                eprintln!("{err}");
            }
        }
        Some("run-golden") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            let path = args.next().unwrap_or_else(|| DEFAULT_GOLDEN_PATH.to_string());
            if let Err(err) = run_golden(&path) {
                eprintln!("{err}");
            }
        }
        Some("run-golden-update") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            let path = args.next().unwrap_or_else(|| DEFAULT_GOLDEN_PATH.to_string());
            if let Err(err) = run_golden_update(&path) {
                eprintln!("{err}");
            }
        }
        Some("run-maze-live") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            let path = args.next();
            if path.is_none() {
                eprintln!("사용법: cargo run -p ddonirang-tool -- run-maze-live <맵파일>");
                return;
            }
            if let Err(err) = run_maze_live(path.as_deref().unwrap()) {
                eprintln!("{err}");
            }
        }
        Some("run-once-file") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            let path = args.next();
            if path.is_none() {
                eprintln!("사용법: cargo run -p ddonirang-tool -- run-once-file <파일경로>");
                return;
            }
            if let Err(err) = run_once_from(path.as_deref()) {
                eprintln!("{err}");
            }
        }
        Some("canon") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::ClosedCore) {
                eprintln!("{err}");
                return;
            }
            let mut input: Option<String> = None;
            let mut emit = "ddn".to_string();
            let mut out: Option<String> = None;
            let mut check = false;
            let mut bridge: Option<CanonBridge> = None;
            let remaining: Vec<String> = args.collect();
            let mut idx = 0usize;
            while idx < remaining.len() {
                match remaining[idx].as_str() {
                    "--emit" => {
                        idx += 1;
                        if idx >= remaining.len() {
                            eprintln!("사용법: cargo run -p ddonirang-tool -- canon <file.ddn> [--emit ddn] [--out <path>] [--bridge age0_step01] [--check]");
                            return;
                        }
                        emit = remaining[idx].clone();
                    }
                    "--out" => {
                        idx += 1;
                        if idx >= remaining.len() {
                            eprintln!("사용법: cargo run -p ddonirang-tool -- canon <file.ddn> [--emit ddn] [--out <path>] [--bridge age0_step01] [--check]");
                            return;
                        }
                        out = Some(remaining[idx].clone());
                    }
                    "--bridge" => {
                        idx += 1;
                        if idx >= remaining.len() {
                            eprintln!("사용법: cargo run -p ddonirang-tool -- canon <file.ddn> [--emit ddn] [--out <path>] [--bridge age0_step01] [--check]");
                            return;
                        }
                        match CanonBridge::parse(&remaining[idx]) {
                            Ok(value) => bridge = Some(value),
                            Err(err) => {
                                eprintln!("{err}");
                                return;
                            }
                        }
                    }
                    "--check" => {
                        check = true;
                        idx += 1;
                        continue;
                    }
                    value => {
                        if input.is_none() {
                            input = Some(value.to_string());
                        } else {
                            eprintln!("알 수 없는 인자: {value}");
                            return;
                        }
                    }
                }
                idx += 1;
            }

            let input = match input {
                Some(v) => v,
                None => {
                    eprintln!("사용법: cargo run -p ddonirang-tool -- canon <file.ddn> [--emit ddn] [--out <path>] [--bridge age0_step01] [--check]");
                    return;
                }
            };
            if let Err(err) = run_canon(&input, &emit, out.as_deref(), check, bridge) {
                eprintln!("{err}");
            }
        }
        Some("preprocess-ai") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::AiTooling) {
                eprintln!("{err}");
                return;
            }
            let input = args.next();
            if input.is_none() {
                eprintln!("???: cargo run -p ddonirang-tool -- preprocess-ai <??.ddn> [??.gen.ddn]");
                return;
            }
            let output = args.next();
            if let Err(err) = preprocess_ai_file(input.as_deref().unwrap(), output.as_deref()) {
                eprintln!("{err}");
            }
        }
        Some("build-schema") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::AiTooling) {
                eprintln!("{err}");
                return;
            }
            let input = args.next();
            if input.is_none() {
                eprintln!("???: cargo run -p ddonirang-tool -- build-schema <??.ddn> [??.json]");
                return;
            }
            let output = args.next();
            if let Err(err) = build_schema_file(input.as_deref().unwrap(), output.as_deref()) {
                eprintln!("{err}");
            }
        }

        Some("ai") => {
            if let Err(err) = ensure_feature(policy.as_ref(), FeatureGate::AiTooling) {
                eprintln!("{err}");
                return;
            }
            let sub = args.next();
            match sub.as_deref() {
                Some("prompt") => {
                    let parsed = ai_prompt::parse_ai_prompt_args(&mut args);
                    match parsed {
                        Ok(parsed) => {
                            if let Err(err) = ai_prompt::run_ai_prompt(parsed) {
                                eprintln!("{err}");
                            }
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            eprintln!(
                                "사용법: cargo run -p ddonirang-tool -- ai prompt --profile <name> [--out <path>] [--bundle <zip|dir>]"
                            );
                        }
                    }
                }
                _ => {
                    eprintln!(
                        "사용법: cargo run -p ddonirang-tool -- ai prompt --profile <name> [--out <path>] [--bundle <zip|dir>]"
                    );
                }
            }
        }
        _ => {
            println!("또니랑 도구줄(Toolchain) v0.1");
            println!("사용법:");
            let ai_prompt_out = paths::build_dir().join("ai.prompt.txt");
            println!("  cargo run -p ddonirang-tool -- run-once");
            println!("  cargo run -p ddonirang-tool -- test [경로]");
            println!("  cargo run -p ddonirang-tool -- run-div0");
            println!("  cargo run -p ddonirang-tool -- run-grid");
            println!("  cargo run -p ddonirang-tool -- run-alrim-demo");
            println!("  cargo run -p ddonirang-tool -- run-maze-file <맵파일> [이동]");
            println!("  cargo run -p ddonirang-tool -- run-maze-ddn <맵파일> <스크립트> [이동]");
            println!("  cargo run -p ddonirang-tool -- run-maze-ddn-live <맵파일> <스크립트>");
            println!("  cargo run -p ddonirang-tool -- run-maze-replay <맵파일> [이동]");
            println!("  cargo run -p ddonirang-tool -- run-maze-record <맵파일> <로그파일> [이동]");
            println!("  cargo run -p ddonirang-tool -- run-maze-replay-file <맵파일> <로그파일>");
            println!("  cargo run -p ddonirang-tool -- run-parabola [틱] [vx] [vy]");
            println!("  cargo run -p ddonirang-tool -- run-golden [파일]");
            println!("  cargo run -p ddonirang-tool -- run-golden-update [파일]");
            println!("  cargo run -p ddonirang-tool -- run-maze-live <맵파일>");
            println!("  cargo run -p ddonirang-tool -- run-once-file <파일경로>");
            println!("  cargo run -p ddonirang-tool -- canon <file.ddn> --emit ddn [--out <path>] [--bridge age0_step01] [--check]");
            println!("  cargo run -p ddonirang-tool -- geoul query <질의> [diag_path]");
            println!("  cargo run -p ddonirang-tool -- preprocess-ai <??.ddn> [??.gen.ddn]");
            println!("  cargo run -p ddonirang-tool -- build-schema <??.ddn> [??.json]");
            println!(
                "  cargo run -p ddonirang-tool -- ai prompt --profile lean --out {}",
                ai_prompt_out.display()
            );
            println!("예시:");
            println!("  echo \"나이:수 = 10\" | cargo run -p ddonirang-tool -- run-once");
            println!("  cargo run -p ddonirang-tool -- test");
            println!("  cargo run -p ddonirang-tool -- run-div0");
            println!("  cargo run -p ddonirang-tool -- run-grid");
            println!("  cargo run -p ddonirang-tool -- run-alrim-demo");
            println!(
                "  cargo run -p ddonirang-tool -- run-maze-file {} wasd",
                DEFAULT_MAZE_MAP_PATH
            );
            println!(
                "  cargo run -p ddonirang-tool -- run-maze-file {} wasd",
                DEFAULT_COIN_MAZE_MAP_PATH
            );
            println!(
                "  cargo run -p ddonirang-tool -- run-maze-ddn {} {} ijkl",
                DEFAULT_MAZE_MAP_PATH,
                DEFAULT_MAZE_SCRIPT_PATH
            );
            println!(
                "  cargo run -p ddonirang-tool -- run-maze-ddn-live {} {}",
                DEFAULT_MAZE_MAP_PATH,
                DEFAULT_MAZE_SCRIPT_PATH
            );
            println!(
                "  cargo run -p ddonirang-tool -- run-maze-replay {} wasd",
                DEFAULT_MAZE_MAP_PATH
            );
            println!(
                "  cargo run -p ddonirang-tool -- run-maze-record {} {} wasd",
                DEFAULT_MAZE_MAP_PATH, DEFAULT_REPLAY_PATH
            );
            println!(
                "  cargo run -p ddonirang-tool -- run-maze-replay-file {} {}",
                DEFAULT_MAZE_MAP_PATH, DEFAULT_REPLAY_PATH
            );
            println!("  cargo run -p ddonirang-tool -- run-parabola 20 5 10");
            println!("  cargo run -p ddonirang-tool -- run-golden {}", DEFAULT_GOLDEN_PATH);
            println!(
                "  cargo run -p ddonirang-tool -- run-golden-update {}",
                DEFAULT_GOLDEN_PATH
            );
            println!(
                "  cargo run -p ddonirang-tool -- run-maze-live {}",
                DEFAULT_MAZE_MAP_PATH
            );
            println!(
                "  cargo run -p ddonirang-tool -- run-once-file {}",
                DEFAULT_RUN_ONCE_INPUT_PATH
            );
            println!("  cargo run -p ddonirang-tool -- geoul query \"#고장\"");
            println!(
                "  cargo run -p ddonirang-tool -- ai prompt --profile lean --out {}",
                ai_prompt_out.display()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddonirang_core::ArithmeticFaultKind;
    use ddonirang_core::gogae3::{
        compute_w24_state_hash, compute_w25_state_hash, compute_w26_state_hash,
        compute_w27_state_hash, compute_w28_state_hash, compute_w29_state_hash,
        compute_w30_state_hash, compute_w31_state_hash, compute_w32_state_hash,
        compute_w33_state_hash, W24Params, W25Params, W26Params, W27Params, W28Params, W29Params,
        W30Params, W31Params, W32Params, W33Params,
    };
    use ddonirang_core::platform::NetEvent;
    use ddonirang_core::{ResourceMapEntry, ResourceValue};
    use std::collections::BTreeMap;

    #[test]
    fn detmath_manifest_is_valid() {
        detmath_assets::ensure_detmath_assets().expect("detmath assets");
    }

    #[test]
    fn net_events_detjson_parse_sample() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("gogae3_w23_network_sam")
            .join("inputs")
            .join("sample_input_snapshot.detjson");
        let events = read_net_events_detjson(path.to_str().expect("sample path"))
            .expect("read detjson");
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].sender, "peer-a");
        assert_eq!(events[0].seq, 1);
        assert_eq!(events[0].order_key, "peer-a#1");
        let payload0: serde_json::Value =
            serde_json::from_str(&events[0].payload_detjson).expect("payload0");
        assert_eq!(payload0, serde_json::json!({"kind":"net_key","key":"W"}));
    }

    #[test]
    fn net_events_detjson_parse_unsorted() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("gogae3_w23_network_sam")
            .join("inputs")
            .join("sample_input_snapshot_unsorted.detjson");
        let events = read_net_events_detjson(path.to_str().expect("sample path"))
            .expect("read detjson");
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].sender, "peer-a");
        assert_eq!(events[0].seq, 1);
        assert_eq!(events[1].sender, "peer-a");
        assert_eq!(events[1].seq, 2);
        assert_eq!(events[2].sender, "peer-b");
        assert_eq!(events[2].seq, 1);
    }

    struct NetEventOrderIyagi;

    impl Iyagi for NetEventOrderIyagi {
        fn run_startup(&mut self, _world: &ddonirang_core::platform::NuriWorld) -> Patch {
            Patch::default()
        }

        fn run_update(
            &mut self,
            _world: &ddonirang_core::platform::NuriWorld,
            input: &InputSnapshot,
        ) -> Patch {
            let order = input
                .net_events
                .iter()
                .map(|event| format!("{}:{}", event.sender, event.seq))
                .collect::<Vec<_>>();
            let json = serde_json::to_string(&order).expect("net_events_order");
            Patch {
                ops: vec![PatchOp::SetResourceJson {
                    tag: "net_events_order".to_string(),
                    json,
                }],
                origin: Origin::system("tool"),
            }
        }
    }

    fn compute_w23_state_hash(net_events: Vec<NetEvent>) -> String {
        let snapshot = InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events,
            rng_seed: 0,
        };
        let sam = ReplaySam::new(vec![snapshot]);
        let iyagi = NetEventOrderIyagi;
        let nuri = DetNuri::new();
        let geoul = InMemoryGeoul::new();
        let bogae = NoopBogae;

        let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
        let mut sink = VecSignalSink::default();
        let frame = loop_.tick_once(0, &mut sink);
        format!("blake3:{}", frame.state_hash.to_hex())
    }


    #[test]
    fn w23_network_sam_state_hash_is_stable() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("gogae3_w23_network_sam");
        let sorted_path = base.join("inputs").join("sample_input_snapshot.detjson");
        let unsorted_path = base
            .join("inputs")
            .join("sample_input_snapshot_unsorted.detjson");
        let expect_path = base.join("expect").join("state_hash.txt");

        let sorted_events = read_net_events_detjson(sorted_path.to_str().expect("sorted path"))
            .expect("read sorted detjson");
        let unsorted_events =
            read_net_events_detjson(unsorted_path.to_str().expect("unsorted path"))
                .expect("read unsorted detjson");
        let expected = read_text_from_path(expect_path.to_str().expect("expect path"))
            .expect("read expect");
        let expected = expected.trim();

        let hash_sorted = compute_w23_state_hash(sorted_events);
        let hash_unsorted = compute_w23_state_hash(unsorted_events);

        assert!(!expected.is_empty(), "state_hash.txt is empty");
        assert_eq!(hash_sorted, expected);
        assert_eq!(hash_unsorted, expected);
    }

    fn parse_param_line(line: &str) -> Option<(String, u64)> {
        let line = line.split("//").next()?.trim();
        if line.is_empty() {
            return None;
        }
        let (left, right) = line.split_once("<-")?;
        let key = left.trim();
        let value_str = right.trim().trim_end_matches('.').trim();
        let value = value_str.parse::<u64>().ok()?;
        Some((key.to_string(), value))
    }

    fn read_w24_params(path: &Path) -> Result<W24Params, String> {
        let text = read_text_from_path(path.to_str().ok_or("invalid path")?)?;
        let mut entity_count = None;
        let mut component_count = None;
        let mut archetype_moves = None;
        let mut perf_cap = None;
        for line in text.lines() {
            let Some((key, value)) = parse_param_line(line) else {
                continue;
            };
            match key.as_str() {
                "살림.개체수" | "ㅅ.개체수" => entity_count = Some(value),
                "살림.컴포넌트수" | "ㅅ.컴포넌트수" => component_count = Some(value),
                "살림.아키타입_이동" | "ㅅ.아키타입_이동" => archetype_moves = Some(value),
                "살림.성능_캡" | "ㅅ.성능_캡" => perf_cap = Some(value),
                _ => {}
            }
        }
        let Some(entity_count) = entity_count else {
            return Err("w24 params: 개체수 누락".to_string());
        };
        let Some(component_count) = component_count else {
            return Err("w24 params: 컴포넌트수 누락".to_string());
        };
        let Some(archetype_moves) = archetype_moves else {
            return Err("w24 params: 아키타입_이동 누락".to_string());
        };
        let Some(perf_cap) = perf_cap else {
            return Err("w24 params: 성능_캡 누락".to_string());
        };
        Ok(W24Params {
            entity_count,
            component_count,
            archetype_moves,
            perf_cap,
        })
    }

    fn read_w25_params(path: &Path) -> Result<W25Params, String> {
        let text = read_text_from_path(path.to_str().ok_or("invalid path")?)?;
        let mut query_target_count = None;
        let mut query_batch = None;
        let mut snapshot_fixed = None;
        for line in text.lines() {
            let Some((key, value)) = parse_param_line(line) else {
                continue;
            };
            match key.as_str() {
                "살림.쿼리_대상수" | "ㅅ.쿼리_대상수" => query_target_count = Some(value),
                "살림.쿼리_배치" | "ㅅ.쿼리_배치" => query_batch = Some(value),
                "살림.스냅샷_고정" | "ㅅ.스냅샷_고정" => snapshot_fixed = Some(value),
                _ => {}
            }
        }
        let Some(query_target_count) = query_target_count else {
            return Err("w25 params: 쿼리_대상수 누락".to_string());
        };
        let Some(query_batch) = query_batch else {
            return Err("w25 params: 쿼리_배치 누락".to_string());
        };
        let Some(snapshot_fixed) = snapshot_fixed else {
            return Err("w25 params: 스냅샷_고정 누락".to_string());
        };
        Ok(W25Params {
            query_target_count,
            query_batch,
            snapshot_fixed,
        })
    }

    fn read_w26_params(path: &Path) -> Result<W26Params, String> {
        let text = read_text_from_path(path.to_str().ok_or("invalid path")?)?;
        let mut agent_count = None;
        let mut item_count = None;
        let mut trade_count = None;
        let mut starting_balance = None;
        let mut starting_inventory = None;
        let mut base_price = None;
        for line in text.lines() {
            let Some((key, value)) = parse_param_line(line) else {
                continue;
            };
            match key.as_str() {
                "살림.임자수" | "ㅅ.임자수" => agent_count = Some(value),
                "살림.상품수" | "ㅅ.상품수" => item_count = Some(value),
                "살림.거래수" | "ㅅ.거래수" => trade_count = Some(value),
                "살림.초기_잔고" | "ㅅ.초기_잔고" => starting_balance = Some(value),
                "살림.초기_재고" | "ㅅ.초기_재고" => starting_inventory = Some(value),
                "살림.기본_가격" | "ㅅ.기본_가격" => base_price = Some(value),
                _ => {}
            }
        }
        let Some(agent_count) = agent_count else {
            return Err("w26 params: 임자수 누락".to_string());
        };
        let Some(item_count) = item_count else {
            return Err("w26 params: 상품수 누락".to_string());
        };
        let Some(trade_count) = trade_count else {
            return Err("w26 params: 거래수 누락".to_string());
        };
        let Some(starting_balance) = starting_balance else {
            return Err("w26 params: 초기_잔고 누락".to_string());
        };
        let Some(starting_inventory) = starting_inventory else {
            return Err("w26 params: 초기_재고 누락".to_string());
        };
        let Some(base_price) = base_price else {
            return Err("w26 params: 기본_가격 누락".to_string());
        };
        Ok(W26Params {
            agent_count,
            item_count,
            trade_count,
            starting_balance,
            starting_inventory,
            base_price,
        })
    }

    fn read_w27_params(path: &Path) -> Result<W27Params, String> {
        let text = read_text_from_path(path.to_str().ok_or("invalid path")?)?;
        let mut agent_count = None;
        let mut trade_count = None;
        let mut starting_balance = None;
        let mut min_balance = None;
        let mut trade_amount = None;
        for line in text.lines() {
            let Some((key, value)) = parse_param_line(line) else {
                continue;
            };
            match key.as_str() {
                "살림.임자수" | "ㅅ.임자수" => agent_count = Some(value),
                "살림.거래수" | "ㅅ.거래수" => trade_count = Some(value),
                "살림.초기_잔고" | "ㅅ.초기_잔고" => starting_balance = Some(value),
                "살림.잔고_최소" | "ㅅ.잔고_최소" => min_balance = Some(value),
                "살림.거래_금액" | "ㅅ.거래_금액" => trade_amount = Some(value),
                _ => {}
            }
        }
        let Some(agent_count) = agent_count else {
            return Err("w27 params: 임자수 누락".to_string());
        };
        let Some(trade_count) = trade_count else {
            return Err("w27 params: 거래수 누락".to_string());
        };
        let Some(starting_balance) = starting_balance else {
            return Err("w27 params: 초기_잔고 누락".to_string());
        };
        let Some(min_balance) = min_balance else {
            return Err("w27 params: 잔고_최소 누락".to_string());
        };
        let Some(trade_amount) = trade_amount else {
            return Err("w27 params: 거래_금액 누락".to_string());
        };
        Ok(W27Params {
            agent_count,
            trade_count,
            starting_balance,
            min_balance,
            trade_amount,
        })
    }

    fn read_w28_params(path: &Path) -> Result<W28Params, String> {
        let text = read_text_from_path(path.to_str().ok_or("invalid path")?)?;
        let mut agent_count = None;
        let mut item_count = None;
        let mut trade_count = None;
        let mut base_price = None;
        let mut trade_amount = None;
        for line in text.lines() {
            let Some((key, value)) = parse_param_line(line) else {
                continue;
            };
            match key.as_str() {
                "살림.임자수" | "ㅅ.임자수" => agent_count = Some(value),
                "살림.상품수" | "ㅅ.상품수" => item_count = Some(value),
                "살림.거래수" | "ㅅ.거래수" => trade_count = Some(value),
                "살림.기본_가격" | "ㅅ.기본_가격" => base_price = Some(value),
                "살림.거래_금액" | "ㅅ.거래_금액" => trade_amount = Some(value),
                _ => {}
            }
        }
        let Some(agent_count) = agent_count else {
            return Err("w28 params: 임자수 누락".to_string());
        };
        let Some(item_count) = item_count else {
            return Err("w28 params: 상품수 누락".to_string());
        };
        let Some(trade_count) = trade_count else {
            return Err("w28 params: 거래수 누락".to_string());
        };
        let Some(base_price) = base_price else {
            return Err("w28 params: 기본_가격 누락".to_string());
        };
        let Some(trade_amount) = trade_amount else {
            return Err("w28 params: 거래_금액 누락".to_string());
        };
        Ok(W28Params {
            agent_count,
            item_count,
            trade_count,
            base_price,
            trade_amount,
        })
    }

    fn read_w29_params(path: &Path) -> Result<W29Params, String> {
        let text = read_text_from_path(path.to_str().ok_or("invalid path")?)?;
        let mut reactive_max_pass = None;
        let mut alert_chain = None;
        let mut step_value = None;
        let mut initial_value = None;
        for line in text.lines() {
            let Some((key, value)) = parse_param_line(line) else {
                continue;
            };
            match key.as_str() {
                "살림.반응_패스_최대" | "ㅅ.반응_패스_최대" => reactive_max_pass = Some(value),
                "살림.알림_연쇄" | "ㅅ.알림_연쇄" => alert_chain = Some(value),
                "살림.반응_증분" | "ㅅ.반응_증분" => step_value = Some(value),
                "살림.초기_값" | "ㅅ.초기_값" => initial_value = Some(value),
                _ => {}
            }
        }
        let Some(reactive_max_pass) = reactive_max_pass else {
            return Err("w29 params: 반응_패스_최대 누락".to_string());
        };
        let Some(alert_chain) = alert_chain else {
            return Err("w29 params: 알림_연쇄 누락".to_string());
        };
        let Some(step_value) = step_value else {
            return Err("w29 params: 반응_증분 누락".to_string());
        };
        let Some(initial_value) = initial_value else {
            return Err("w29 params: 초기_값 누락".to_string());
        };
        Ok(W29Params {
            reactive_max_pass,
            alert_chain,
            step_value,
            initial_value,
        })
    }

    fn read_w30_params(path: &Path) -> Result<W30Params, String> {
        let text = read_text_from_path(path.to_str().ok_or("invalid path")?)?;
        let mut proposal_count = None;
        let mut approval_tokens = None;
        let mut apply_requests = None;
        let mut approval_required = None;
        for line in text.lines() {
            let Some((key, value)) = parse_param_line(line) else {
                continue;
            };
            match key.as_str() {
                "살림.제안_수" | "ㅅ.제안_수" => proposal_count = Some(value),
                "살림.승인_토큰" | "ㅅ.승인_토큰" => approval_tokens = Some(value),
                "살림.적용_요청" | "ㅅ.적용_요청" => apply_requests = Some(value),
                "살림.승인_필수" | "ㅅ.승인_필수" => approval_required = Some(value),
                _ => {}
            }
        }
        let Some(proposal_count) = proposal_count else {
            return Err("w30 params: 제안_수 누락".to_string());
        };
        let Some(approval_tokens) = approval_tokens else {
            return Err("w30 params: 승인_토큰 누락".to_string());
        };
        let Some(apply_requests) = apply_requests else {
            return Err("w30 params: 적용_요청 누락".to_string());
        };
        let Some(approval_required) = approval_required else {
            return Err("w30 params: 승인_필수 누락".to_string());
        };
        Ok(W30Params {
            proposal_count,
            approval_tokens,
            apply_requests,
            approval_required,
        })
    }

    fn read_w31_params(path: &Path) -> Result<W31Params, String> {
        let text = read_text_from_path(path.to_str().ok_or("invalid path")?)?;
        let mut participant_count = None;
        let mut host_inputs = None;
        let mut guest_inputs = None;
        let mut sync_rounds = None;
        let mut starting_value = None;
        for line in text.lines() {
            let Some((key, value)) = parse_param_line(line) else {
                continue;
            };
            match key.as_str() {
                "살림.참가자수" | "ㅅ.참가자수" => participant_count = Some(value),
                "살림.호스트_입력" | "ㅅ.호스트_입력" => host_inputs = Some(value),
                "살림.손님_입력" | "ㅅ.손님_입력" => guest_inputs = Some(value),
                "살림.동기_라운드" | "ㅅ.동기_라운드" => sync_rounds = Some(value),
                "살림.시작_값" | "ㅅ.시작_값" => starting_value = Some(value),
                _ => {}
            }
        }
        let Some(participant_count) = participant_count else {
            return Err("w31 params: 참가자수 누락".to_string());
        };
        let Some(host_inputs) = host_inputs else {
            return Err("w31 params: 호스트_입력 누락".to_string());
        };
        let Some(guest_inputs) = guest_inputs else {
            return Err("w31 params: 손님_입력 누락".to_string());
        };
        let Some(sync_rounds) = sync_rounds else {
            return Err("w31 params: 동기_라운드 누락".to_string());
        };
        let Some(starting_value) = starting_value else {
            return Err("w31 params: 시작_값 누락".to_string());
        };
        Ok(W31Params {
            participant_count,
            host_inputs,
            guest_inputs,
            sync_rounds,
            starting_value,
        })
    }

    fn read_w32_params(path: &Path) -> Result<W32Params, String> {
        let text = read_text_from_path(path.to_str().ok_or("invalid path")?)?;
        let mut diff_count = None;
        let mut code_before_len = None;
        let mut code_after_len = None;
        let mut state_field_count = None;
        let mut summary_cap = None;
        for line in text.lines() {
            let Some((key, value)) = parse_param_line(line) else {
                continue;
            };
            match key.as_str() {
                "살림.차분_개수" | "ㅅ.차분_개수" => diff_count = Some(value),
                "살림.코드_길이_전" | "ㅅ.코드_길이_전" => code_before_len = Some(value),
                "살림.코드_길이_후" | "ㅅ.코드_길이_후" => code_after_len = Some(value),
                "살림.상태_필드_수" | "ㅅ.상태_필드_수" => state_field_count = Some(value),
                "살림.요약_캡" | "ㅅ.요약_캡" => summary_cap = Some(value),
                _ => {}
            }
        }
        let Some(diff_count) = diff_count else {
            return Err("w32 params: 차분_개수 누락".to_string());
        };
        let Some(code_before_len) = code_before_len else {
            return Err("w32 params: 코드_길이_전 누락".to_string());
        };
        let Some(code_after_len) = code_after_len else {
            return Err("w32 params: 코드_길이_후 누락".to_string());
        };
        let Some(state_field_count) = state_field_count else {
            return Err("w32 params: 상태_필드_수 누락".to_string());
        };
        let Some(summary_cap) = summary_cap else {
            return Err("w32 params: 요약_캡 누락".to_string());
        };
        Ok(W32Params {
            diff_count,
            code_before_len,
            code_after_len,
            state_field_count,
            summary_cap,
        })
    }

    fn read_w33_params(path: &Path) -> Result<W33Params, String> {
        let text = read_text_from_path(path.to_str().ok_or("invalid path")?)?;
        let mut agent_count = None;
        let mut item_count = None;
        let mut trade_count = None;
        let mut query_batch = None;
        let mut reactive_max_pass = None;
        for line in text.lines() {
            let Some((key, value)) = parse_param_line(line) else {
                continue;
            };
            match key.as_str() {
                "살림.임자수" | "ㅅ.임자수" => agent_count = Some(value),
                "살림.상품수" | "ㅅ.상품수" => item_count = Some(value),
                "살림.거래수" | "ㅅ.거래수" => trade_count = Some(value),
                "살림.쿼리_배치" | "ㅅ.쿼리_배치" => query_batch = Some(value),
                "살림.반응_패스_최대" | "ㅅ.반응_패스_최대" => reactive_max_pass = Some(value),
                _ => {}
            }
        }
        let Some(agent_count) = agent_count else {
            return Err("w33 params: 임자수 누락".to_string());
        };
        let Some(item_count) = item_count else {
            return Err("w33 params: 상품수 누락".to_string());
        };
        let Some(trade_count) = trade_count else {
            return Err("w33 params: 거래수 누락".to_string());
        };
        let Some(query_batch) = query_batch else {
            return Err("w33 params: 쿼리_배치 누락".to_string());
        };
        let Some(reactive_max_pass) = reactive_max_pass else {
            return Err("w33 params: 반응_패스_최대 누락".to_string());
        };
        Ok(W33Params {
            agent_count,
            item_count,
            trade_count,
            query_batch,
            reactive_max_pass,
        })
    }

    fn format_state_hash(hash: ddonirang_core::StateHash) -> String {
        format!("blake3:{}", hash.to_hex())
    }

    #[test]
    fn w24_ecs_archetype_state_hash_is_stable() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("gogae3_w24_ecs_archetype");
        let input_path = base.join("input.ddn");
        let expect_path = base.join("expect").join("state_hash.txt");

        let params = read_w24_params(&input_path).expect("read w24 params");
        let expected = read_text_from_path(expect_path.to_str().expect("expect path"))
            .expect("read expect");
        let expected = expected.trim();

        let hash_sorted = format_state_hash(compute_w24_state_hash(&params));
        let hash_unsorted = format_state_hash(compute_w24_state_hash(&params));

        assert!(!expected.is_empty(), "state_hash.txt is empty");
        assert_eq!(hash_sorted, expected);
        assert_eq!(hash_unsorted, expected);
    }

    #[test]
    fn w25_query_batch_state_hash_is_stable() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("gogae3_w25_query_batch");
        let input_path = base.join("input.ddn");
        let expect_path = base.join("expect").join("state_hash.txt");

        let params = read_w25_params(&input_path).expect("read w25 params");
        let expected = read_text_from_path(expect_path.to_str().expect("expect path"))
            .expect("read expect");
        let expected = expected.trim();

        let hash = format_state_hash(compute_w25_state_hash(&params));

        assert!(!expected.is_empty(), "state_hash.txt is empty");
        assert_eq!(hash, expected);
    }

    #[test]
    fn w26_econ_seeds_state_hash_is_stable() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("gogae3_w26_econ_seeds");
        let input_path = base.join("input.ddn");
        let expect_path = base.join("expect").join("state_hash.txt");

        let params = read_w26_params(&input_path).expect("read w26 params");
        let expected = read_text_from_path(expect_path.to_str().expect("expect path"))
            .expect("read expect");
        let expected = expected.trim();

        let hash = format_state_hash(compute_w26_state_hash(&params));

        assert!(!expected.is_empty(), "state_hash.txt is empty");
        assert_eq!(hash, expected);
    }

    #[test]
    fn w27_invariant_hook_state_hash_is_stable() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("gogae3_w27_invariant_hook");
        let input_path = base.join("input.ddn");
        let expect_path = base.join("expect").join("state_hash.txt");

        let params = read_w27_params(&input_path).expect("read w27 params");
        let expected = read_text_from_path(expect_path.to_str().expect("expect path"))
            .expect("read expect");
        let expected = expected.trim();

        let hash = format_state_hash(compute_w27_state_hash(&params));

        assert!(!expected.is_empty(), "state_hash.txt is empty");
        assert_eq!(hash, expected);
    }

    #[test]
    fn w28_indicators_state_hash_is_stable() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("gogae3_w28_indicators");
        let input_path = base.join("input.ddn");
        let expect_path = base.join("expect").join("state_hash.txt");

        let params = read_w28_params(&input_path).expect("read w28 params");
        let expected = read_text_from_path(expect_path.to_str().expect("expect path"))
            .expect("read expect");
        let expected = expected.trim();

        let hash = format_state_hash(compute_w28_state_hash(&params));

        assert!(!expected.is_empty(), "state_hash.txt is empty");
        assert_eq!(hash, expected);
    }

    #[test]
    fn w29_reactive_limit_state_hash_is_stable() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("gogae3_w29_reactive_limit");
        let input_path = base.join("input.ddn");
        let expect_path = base.join("expect").join("state_hash.txt");

        let params = read_w29_params(&input_path).expect("read w29 params");
        let expected = read_text_from_path(expect_path.to_str().expect("expect path"))
            .expect("read expect");
        let expected = expected.trim();

        let hash = format_state_hash(compute_w29_state_hash(&params));

        assert!(!expected.is_empty(), "state_hash.txt is empty");
        assert_eq!(hash, expected);
    }

    #[test]
    fn w30_qchain_approval_state_hash_is_stable() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("gogae3_w30_qchain_approval");
        let input_path = base.join("input.ddn");
        let expect_path = base.join("expect").join("state_hash.txt");

        let params = read_w30_params(&input_path).expect("read w30 params");
        let expected = read_text_from_path(expect_path.to_str().expect("expect path"))
            .expect("read expect");
        let expected = expected.trim();

        let hash = format_state_hash(compute_w30_state_hash(&params));

        assert!(!expected.is_empty(), "state_hash.txt is empty");
        assert_eq!(hash, expected);
    }

    #[test]
    fn w31_shared_stage_state_hash_is_stable() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("gogae3_w31_shared_stage");
        let input_path = base.join("input.ddn");
        let expect_path = base.join("expect").join("state_hash.txt");

        let params = read_w31_params(&input_path).expect("read w31 params");
        let expected = read_text_from_path(expect_path.to_str().expect("expect path"))
            .expect("read expect");
        let expected = expected.trim();

        let hash = format_state_hash(compute_w31_state_hash(&params));

        assert!(!expected.is_empty(), "state_hash.txt is empty");
        assert_eq!(hash, expected);
    }

    #[test]
    fn w32_diff_viewer_state_hash_is_stable() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("gogae3_w32_diff_viewer");
        let input_path = base.join("input.ddn");
        let expect_path = base.join("expect").join("state_hash.txt");

        let params = read_w32_params(&input_path).expect("read w32 params");
        let expected = read_text_from_path(expect_path.to_str().expect("expect path"))
            .expect("read expect");
        let expected = expected.trim();

        let hash = format_state_hash(compute_w32_state_hash(&params));

        assert!(!expected.is_empty(), "state_hash.txt is empty");
        assert_eq!(hash, expected);
    }

    #[test]
    fn w33_market_integration_state_hash_is_stable() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("gogae3_w33_market_integration");
        let input_path = base.join("input.ddn");
        let expect_path = base.join("expect").join("state_hash.txt");

        let params = read_w33_params(&input_path).expect("read w33 params");
        let expected = read_text_from_path(expect_path.to_str().expect("expect path"))
            .expect("read expect");
        let expected = expected.trim();

        let hash = format_state_hash(compute_w33_state_hash(&params));

        assert!(!expected.is_empty(), "state_hash.txt is empty");
        assert_eq!(hash, expected);
    }


    #[test]
    fn tool_div0_emits_signal_and_preserves_value() {
        let sam = DetSam::new(Fixed64::from_i64(1));
        let iyagi = Div0Iyagi;
        let nuri = DetNuri::new();
        let geoul = InMemoryGeoul::new();
        let bogae = NoopBogae;

        let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
        let mut sink = VecSignalSink::default();

        loop_.tick_once(0, &mut sink);
        let x = loop_
            .nuri
            .world()
            .get_resource_fixed64("x")
            .unwrap_or(Fixed64::ZERO);

        assert_eq!(x, Fixed64::from_i64(5));
        assert_eq!(sink.signals.len(), 1);
    }

    #[test]
    fn console_bogae_renders_grid() {
        let mut world = ddonirang_core::NuriWorld::new();
        world.set_resource_fixed64("맵_w".to_string(), Fixed64::from_i64(3));
        world.set_resource_fixed64("맵_h".to_string(), Fixed64::from_i64(2));
        world.set_resource_fixed64("플레이어_x".to_string(), Fixed64::from_i64(1));
        world.set_resource_fixed64("플레이어_y".to_string(), Fixed64::from_i64(0));

        let s = ConsoleBogae::render_to_string(&world);
        assert_eq!(s, ".P.\n...");
    }

    #[test]
    fn console_bogae_prefers_map_resource() {
        let mut world = ddonirang_core::NuriWorld::new();
        world.set_resource_json("맵".to_string(), "##\n.P".to_string());

        let s = ConsoleBogae::render_to_string(&world);
        assert_eq!(s, "##\n.P");
    }

    #[test]
    fn parse_maze_finds_start_and_goal() {
        let map = "###\n#P#\n#G#\n###";
        let (data, sx, sy) = parse_maze(map).expect("parse maze");
        assert_eq!(data.width, 3);
        assert_eq!(data.height, 4);
        assert_eq!((sx, sy), (1, 1));
        assert_eq!(data.goal, Some((1, 2)));
        assert!(data.coins.is_empty());
    }

    #[test]
    fn ddn_runner_updates_resource() {
        let script = r#"
매틱:움직씨 = {
    키 <- "".
    (("d") 눌렸나) 일때 { 키 <- "d". }
    (키 == "d") 일때 { 플레이어_x <- 플레이어_x + 1. }
}
"#;
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse ddn");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = ddonirang_core::NuriWorld::new();
        let input = ddonirang_core::InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: KEY_D,
            last_key_name: "d".to_string(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let mut defaults = HashMap::new();
        defaults.insert("플레이어_x".to_string(), Value::Fixed64(Fixed64::from_i64(1)));
        let out = runner
            .run_update(&world, &input, &defaults)
            .expect("run update");
        let px = out.resources.get("플레이어_x").expect("player x");
        assert_eq!(*px, Value::Fixed64(Fixed64::from_i64(2)));
    }

    #[test]
    fn container_resource_state_hash_is_stable() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("age1_container_resource");
        let script_path = base.join("input.ddn");
        let script = std::fs::read_to_string(&script_path).expect("read input");
        let program = DdnProgram::from_source(
            &script,
            script_path.to_str().expect("script path"),
        )
        .expect("parse ddn");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut nuri = DetNuri::new();
        nuri
            .world_mut()
            .set_resource_value("차림".to_string(), ResourceValue::List(Vec::new()));
        nuri
            .world_mut()
            .set_resource_value("모음값".to_string(), ResourceValue::Set(BTreeMap::new()));
        nuri
            .world_mut()
            .set_resource_value("짝".to_string(), ResourceValue::Map(BTreeMap::new()));
        nuri
            .world_mut()
            .set_resource_fixed64("결과".to_string(), Fixed64::ZERO);
        let input = InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let defaults = HashMap::new();
        let out = runner
            .run_update(nuri.world(), &input, &defaults)
            .expect("run update");

        let mut sink = VecSignalSink::default();
        nuri.apply_patch(&out.patch, input.tick_id, &mut sink);

        let expected_list = ResourceValue::List(vec![
            ResourceValue::Fixed64(Fixed64::from_i64(1)),
            ResourceValue::Fixed64(Fixed64::from_i64(2)),
            ResourceValue::Fixed64(Fixed64::from_i64(3)),
        ]);
        assert_eq!(
            nuri.world().get_resource_value("차림"),
            Some(expected_list)
        );

        let expected_set = ResourceValue::set_from_values(vec![
            ResourceValue::Fixed64(Fixed64::from_i64(2)),
            ResourceValue::Fixed64(Fixed64::from_i64(1)),
            ResourceValue::Fixed64(Fixed64::from_i64(2)),
        ]);
        assert_eq!(
            nuri.world().get_resource_value("모음값"),
            Some(expected_set)
        );

        let expected_map = ResourceValue::map_from_entries(vec![
            ResourceMapEntry {
                key: ResourceValue::String("b".to_string()),
                value: ResourceValue::Fixed64(Fixed64::from_i64(2)),
            },
            ResourceMapEntry {
                key: ResourceValue::String("a".to_string()),
                value: ResourceValue::Fixed64(Fixed64::from_i64(1)),
            },
        ]);
        assert_eq!(nuri.world().get_resource_value("짝"), Some(expected_map));

        let result = nuri
            .world()
            .get_resource_fixed64("결과")
            .unwrap_or(Fixed64::ZERO);
        assert_eq!(result, Fixed64::from_i64(3));

        let state_hash = format!("blake3:{}", nuri.world().state_hash().to_hex());
        let expect_path = base.join("expect").join("state_hash.txt");
        let expected_hash = std::fs::read_to_string(&expect_path).expect("read expect");
        assert_eq!(state_hash, expected_hash.trim());
    }

    #[test]
    fn ddn_unit_mismatch_emits_signal_and_preserves_value() {
        let script = r#"
매틱:움직씨 = {
    결과 <- 입력값 + 1@m.
}
"#;
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse ddn");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut nuri = DetNuri::new();
        nuri
            .world_mut()
            .set_resource_fixed64("결과".to_string(), Fixed64::from_i64(7));
        let input = InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let mut defaults = HashMap::new();
        defaults.insert(
            "입력값".to_string(),
            Value::Unit(ddonirang_core::UnitValue::new(
                Fixed64::from_i64(1),
                Unit::Second,
            )),
        );
        defaults.insert("결과".to_string(), Value::Fixed64(Fixed64::from_i64(7)));
        let out = runner
            .run_update(nuri.world(), &input, &defaults)
            .expect("run update");
        assert!(
            !out.patch.ops.iter().any(|op| matches!(
                op,
                PatchOp::SetResourceFixed64 { tag, .. } if tag == "결과"
            ))
        );
        let mut sink = VecSignalSink::default();
        nuri.apply_patch(&out.patch, input.tick_id, &mut sink);
        let result = nuri
            .world()
            .get_resource_fixed64("결과")
            .unwrap_or(Fixed64::ZERO);
        assert_eq!(result, Fixed64::from_i64(7));
        assert!(sink.signals.iter().any(|signal| matches!(
            signal,
            Signal::ArithmeticFault {
                kind: ArithmeticFaultKind::DimensionMismatch { .. },
                ..
            }
        )));
    }

    #[test]
    fn contract_pre_emits_diag_and_stops_after_violation() {
        let script = r#"
  매틱:움직씨 = {
      { 거짓 }인것 전제하에 아니면 { 결과 <- 1. }
      결과 <- 2.
  }
"#;
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse ddn");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut nuri = DetNuri::new();
        nuri
            .world_mut()
            .set_resource_fixed64("결과".to_string(), Fixed64::from_i64(0));
        let input = InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let mut defaults = HashMap::new();
        defaults.insert("결과".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
        let out = runner
            .run_update(nuri.world(), &input, &defaults)
            .expect("run update");

        assert!(!out.patch.ops.iter().any(|op| matches!(
            op,
            PatchOp::SetResourceFixed64 { tag, value } if tag == "결과" && *value == Fixed64::from_i64(2)
        )));

        let mut sink = VecSignalSink::default();
        nuri.apply_patch(&out.patch, input.tick_id, &mut sink);
        let result = nuri
            .world()
            .get_resource_fixed64("결과")
            .unwrap_or(Fixed64::ZERO);
        assert_eq!(result, Fixed64::from_i64(1));

        let diag = sink
            .diag_events
            .iter()
            .find(|event| event.reason == "CONTRACT_PRE")
            .expect("contract diag");
        assert_eq!(diag.contract_kind.as_deref(), Some("pre"));
        assert_eq!(diag.mode.as_deref(), Some("중단"));
    }

    #[test]
    fn formula_ascii_evaluates_with_injection() {
        let script = r#"
  매틱:움직씨 = {
      결과 <- (#ascii) 수식{ y = 2*x + 3/2 } 해서 (x=6) 풀기.
  }
  "#;
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse ddn");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut nuri = DetNuri::new();
        nuri
            .world_mut()
            .set_resource_fixed64("결과".to_string(), Fixed64::from_i64(0));
        let input = InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let mut defaults = HashMap::new();
        defaults.insert("결과".to_string(), Value::Fixed64(Fixed64::from_i64(0)));
        let out = runner
            .run_update(nuri.world(), &input, &defaults)
            .expect("run update");
        let result = out.resources.get("결과").expect("결과");
        assert_eq!(*result, Value::Fixed64(Fixed64::from_f64_lossy(13.5)));
    }

    #[test]
    fn formula_missing_injection_is_error() {
        let script = r#"
  매틱:움직씨 = {
      결과 <- 수식{ x + y } 해서 (x=6) 풀기.
  }
  "#;
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse ddn");
        let mut runner = DdnRunner::new(program, "매틱");
        let nuri = DetNuri::new();
        let input = InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let defaults = HashMap::new();
        let result = runner.run_update(nuri.world(), &input, &defaults);
        assert!(result.is_err());
        let err = result.err().expect("missing injection");
        assert!(err.contains("누락"));
        assert!(err.contains("y"));
    }

    #[test]
    fn formula_extra_injection_is_error() {
        let script = r#"
  매틱:움직씨 = {
      결과 <- 수식{ x + 1 } 해서 (x=6, y=1) 풀기.
  }
  "#;
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse ddn");
        let mut runner = DdnRunner::new(program, "매틱");
        let nuri = DetNuri::new();
        let input = InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let defaults = HashMap::new();
        let result = runner.run_update(nuri.world(), &input, &defaults);
        assert!(result.is_err());
        let err = result.err().expect("extra injection");
        assert!(err.contains("여분"));
        assert!(err.contains("y"));
    }

    #[test]
    fn formula_none_injection_is_error() {
        let script = r#"
  매틱:움직씨 = {
      결과 <- 수식{ x + 1 } 해서 (x=없음) 풀기.
  }
  "#;
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse ddn");
        let mut runner = DdnRunner::new(program, "매틱");
        let nuri = DetNuri::new();
        let input = InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let defaults = HashMap::new();
        let result = runner.run_update(nuri.world(), &input, &defaults);
        assert!(result.is_err());
        let err = result.err().expect("none injection");
        assert!(err.contains("값이 없습니다"));
    }

    #[test]
    fn formula_latex_is_fatal() {
        let script = r#"
  매틱:움직씨 = {
      결과 <- (#latex) 수식{ y = \\frac{1}{2}x } 해서 (x=1) 풀기.
  }
  "#;
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse ddn");
        let mut runner = DdnRunner::new(program, "매틱");
        let nuri = DetNuri::new();
        let input = InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let defaults = HashMap::new();
        let result = runner.run_update(nuri.world(), &input, &defaults);
        assert!(result.is_err());
        let err = result.err().expect("latex formula");
        assert!(err.contains("FORMULA_DIALECT_UNSUPPORTED"));
    }

    #[test]
    fn formula_nonascii_body_is_error() {
        let script = r#"
  매틱:움직씨 = {
      결과 <- (#ascii) 수식{ x + 한 } 해서 (x=1) 풀기.
  }
  "#;
        let result = DdnProgram::from_source(script, "test.ddn");
        assert!(result.is_err());
        let err = result.err().expect("nonascii");
        assert!(err.contains("FORMULA_BODY_NONASCII"));
    }

    #[test]
    fn maze_iyagi_moves_with_input_keys() {
        let map = "#####\n#P..#\n#..G#\n#####";
        let (data, sx, sy) = parse_maze(map).expect("parse maze");
        let mut iyagi = MazeIyagi::new(data, sx, sy);
        let input = ddonirang_core::InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: KEY_D,
            last_key_name: "d".to_string(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let patch = iyagi.run_update(&ddonirang_core::NuriWorld::new(), &input);
        let mut px = None;
        let mut py = None;
        for op in patch.ops {
            if let PatchOp::SetResourceFixed64 { tag, value } = op {
                if tag == "플레이어_x" {
                    px = Some(value);
                }
                if tag == "플레이어_y" {
                    py = Some(value);
                }
            }
        }
        assert_eq!(px, Some(Fixed64::from_i64(sx + 1)));
        assert_eq!(py, Some(Fixed64::from_i64(sy)));
    }

    #[test]
    fn replay_sam_replays_snapshots_in_order() {
        let snapshot1 = InputSnapshot {
            tick_id: 1,
            dt: Fixed64::from_i64(1),
            keys_pressed: KEY_W,
            last_key_name: "w".to_string(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let snapshot2 = InputSnapshot {
            tick_id: 2,
            dt: Fixed64::from_i64(1),
            keys_pressed: KEY_D,
            last_key_name: "d".to_string(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let mut sam = ReplaySam::new(vec![snapshot1.clone(), snapshot2.clone()]);
        let first = sam.begin_tick(0);
        let second = sam.begin_tick(0);
        assert_eq!(first.tick_id, snapshot1.tick_id);
        assert_eq!(second.tick_id, snapshot2.tick_id);
    }

    #[test]
    fn replay_log_roundtrip_json() {
        let log = ReplayLog {
            frames: vec![ReplayFrame {
                tick_id: 1,
                dt_raw: Fixed64::from_i64(1).raw_i64(),
                keys_pressed: KEY_W,
                last_key_name: "w".to_string(),
                pointer_x_i32: 0,
                pointer_y_i32: 0,
                net_events: Vec::new(),
                rng_seed: 7,
                state_hash: "00".repeat(32),
            }],
        };
        let json = serde_json::to_string(&log).expect("serialize");
        let parsed: ReplayLog = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.frames.len(), 1);
        assert_eq!(parsed.frames[0].tick_id, 1);
        assert_eq!(parsed.frames[0].state_hash, "00".repeat(32));
    }

    #[test]
    fn golden_file_roundtrip_json() {
        let golden = GoldenFile {
            version: 2,
            scenarios: vec![GoldenScenario {
                name: "maze".to_string(),
                kind: GoldenScenarioKind::Maze {
                    map: DEFAULT_MAZE_MAP_PATH.to_string(),
                    moves: "DDDDSS".to_string(),
                },
                hashes: vec!["00".repeat(32), "11".repeat(32), "22".repeat(32)],
            }],
        };
        let json = serde_json::to_string(&golden).expect("serialize");
        let parsed: GoldenFile = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.version, 2);
        assert_eq!(parsed.scenarios.len(), 1);
        assert_eq!(parsed.scenarios[0].hashes.len(), 3);
    }

    #[test]
    fn parabola_iyagi_updates_position() {
        let mut iyagi = ParabolaIyagi::new(
            Fixed64::from_i64(2),
            Fixed64::from_i64(3),
            Fixed64::from_i64(1),
        );
        let input = InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let patch = iyagi.run_update(&ddonirang_core::NuriWorld::new(), &input);
        let tag_x = resource_tag_with_unit("x", Unit::Meter);
        let tag_y = resource_tag_with_unit("y", Unit::Meter);
        let tag_t = resource_tag_with_unit("t", Unit::Second);
        let mut x = None;
        let mut y = None;
        let mut t = None;
        for op in patch.ops {
            if let PatchOp::SetResourceFixed64 { tag, value } = op {
                if tag == tag_x {
                    x = Some(value);
                }
                if tag == tag_y {
                    y = Some(value);
                }
                if tag == tag_t {
                    t = Some(value);
                }
            }
        }
        assert_eq!(x, Some(Fixed64::from_i64(2)));
        assert_eq!(y, Some(Fixed64::from_i64(3)));
        assert_eq!(t, Some(Fixed64::from_i64(1)));
    }

    #[test]
    fn maze_goal_sets_score_and_goal_state() {
        let map = "#####\n#PG.#\n#####";
        let (data, sx, sy) = parse_maze(map).expect("parse maze");
        let mut iyagi = MazeIyagi::new(data, sx, sy);
        let input = InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: KEY_D,
            last_key_name: "d".to_string(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let patch = iyagi.run_update(&ddonirang_core::NuriWorld::new(), &input);
        let mut score = None;
        let mut goal = None;
        for op in patch.ops {
            match op {
                PatchOp::SetResourceFixed64 { tag, value } if tag == "점수" => {
                    score = Some(value);
                }
                PatchOp::SetResourceJson { tag, json } if tag == "목표" => {
                    goal = Some(json);
                }
                _ => {}
            }
        }
        assert_eq!(score, Some(Fixed64::from_i64(1)));
        assert_eq!(goal.as_deref(), Some("달성"));
    }

    #[test]
    fn maze_coin_pickup_increments_score() {
        let map = "#####\n#PC.#\n#..G#\n#####";
        let (data, sx, sy) = parse_maze(map).expect("parse maze");
        let mut iyagi = MazeIyagi::new(data, sx, sy);
        let input = InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: KEY_D,
            last_key_name: "d".to_string(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let patch = iyagi.run_update(&ddonirang_core::NuriWorld::new(), &input);
        let mut score = None;
        let mut coins_left = None;
        for op in patch.ops {
            if let PatchOp::SetResourceFixed64 { tag, value } = op {
                if tag == "점수" {
                    score = Some(value);
                }
                if tag == "코인남음" {
                    coins_left = Some(value);
                }
            }
        }
        assert_eq!(score, Some(Fixed64::from_i64(1)));
        assert_eq!(coins_left, Some(Fixed64::from_i64(0)));
    }

    fn run_script_once(script: &str) -> Result<(), String> {
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse ddn");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = ddonirang_core::platform::NuriWorld::new();
        let input = InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        };
        let defaults = HashMap::new();
        runner.run_update(&world, &input, &defaults).map(|_| ())
    }

    #[test]
    fn runtime_typecheck_rejects_mismatch_cases() {
        let cases = vec![
            (
                "pin mismatch",
                r#"
(값:수~을) 두배:셈씨 = {
}

매틱:움직씨 = {
    (값="글") 두배하기.
}
"#,
                vec!["E_RUNTIME_TYPE_MISMATCH", "핀=값", "기대=수", "실제=글"],
            ),
            (
                "int mismatch",
                r#"
(값:정수~을) 받:셈씨 = {
}

매틱:움직씨 = {
    (값=1.5) 받기.
}
"#,
                vec!["E_RUNTIME_TYPE_MISMATCH", "핀=값", "기대=정수", "실제=수"],
            ),
            (
                "unit mismatch",
                r#"
(거리:(m)수~을) 이동:셈씨 = {
}

매틱:움직씨 = {
    (거리=1@s) 이동하기.
}
"#,
                vec!["E_RUNTIME_TYPE_MISMATCH", "핀=거리", "기대=수@m", "실제=수@s"],
            ),
            (
                "list element mismatch",
                r#"
(목록:(수)차림~을) 합:셈씨 = {
}

매틱:움직씨 = {
    (목록=[1, "a"]) 합하기.
}
"#,
                vec![
                    "E_RUNTIME_TYPE_MISMATCH",
                    "핀=목록",
                    "기대=(수)차림",
                    "실제=차림(요소: 글)",
                ],
            ),
            (
                "set element mismatch",
                r#"
(집합:(글)모음~을) 통과:셈씨 = {
}

매틱:움직씨 = {
    (집합=("a", 1) 모음) 통과하기.
}
"#,
                vec![
                    "E_RUNTIME_TYPE_MISMATCH",
                    "핀=모음",
                    "기대=(글)모음",
                    "실제=모음(요소: 정수)",
                ],
            ),
            (
                "map element mismatch",
                r#"
(표:(글 수)짝맞춤~을) 통과:셈씨 = {
}

매틱:움직씨 = {
    (표=("a", 1, 2, 3) 짝맞춤) 통과하기.
}
"#,
                vec![
                    "E_RUNTIME_TYPE_MISMATCH",
                    "핀=표",
                    "기대=(글, 수)짝맞춤",
                    "실제=짝맞춤(열쇠: 정수)",
                ],
            ),
            (
                "pipe injection mismatch",
                r#"
(값:글~을) 받:셈씨 = {
}

매틱:움직씨 = {
    1 해서 () 받기.
}
"#,
                vec!["E_RUNTIME_TYPE_MISMATCH", "핀=값", "기대=글", "실제=정수"],
            ),
        ];

        for (name, script, expected_parts) in cases {
            let err = run_script_once(script).expect_err(name);
            for part in expected_parts {
                assert!(
                    err.contains(part),
                    "case '{}': missing '{}', got '{}'",
                    name,
                    part,
                    err
                );
            }
        }
    }

    #[test]
    fn runtime_typecheck_allows_optional_and_infer() {
        let optional_script = r#"
(값:글~을?) 통과:셈씨 = {
}

매틱:움직씨 = {
    () 통과하기.
}
"#;
        run_script_once(optional_script).expect("optional should pass");

        let infer_script = r#"
(값:_~을) 통과:셈씨 = {
}

매틱:움직씨 = {
    (값="글") 통과하기.
}
"#;
        run_script_once(infer_script).expect("infer should pass");
    }

    #[test]
    fn runtime_typecheck_pack_inputs() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("pack")
            .join("type_runtime_typecheck");
        let mut inputs: Vec<std::path::PathBuf> = std::fs::read_dir(&base)
            .expect("read pack dir")
            .filter_map(|entry| entry.ok().map(|item| item.path()))
            .filter(|path| {
                path.extension().and_then(|ext| ext.to_str()) == Some("ddn")
                    && path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| name.starts_with("input_"))
                        .unwrap_or(false)
            })
            .collect();
        inputs.sort();
        assert!(
            !inputs.is_empty(),
            "pack/type_runtime_typecheck 입력이 비었습니다"
        );
        for path in inputs {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("<unknown>");
            let script = std::fs::read_to_string(&path).expect("read input");
            let result = run_script_once(&script);
            if name.contains("_ok") {
                assert!(
                    result.is_ok(),
                    "case '{}': expected success, got error {:?}",
                    name,
                    result.err()
                );
            } else {
                let err = result.expect_err(name);
                for part in ["E_RUNTIME_TYPE_MISMATCH", "핀=", "기대=", "실제="] {
                    assert!(
                        err.contains(part),
                        "case '{}': missing '{}', got '{}'",
                        name,
                        part,
                        err
                    );
                }
            }
        }
    }
}
