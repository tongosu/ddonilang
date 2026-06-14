use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

use super::detjson::{sha256_hex, write_text};
use super::frontdoor_input::{prepare_frontdoor_canon_input, validate_no_legacy_frontdoor_surface};
use super::paths;

const SPEC_SCHEMA: &str = "ddn.gogae9.w89.evolve_spec.v1";
const META_SCHEMA: &str = "ddn.gogae9.w89.evolve_meta.v1";
const REQUIRED_MUTATION_OP_COUNT: usize = 5;

pub struct EvolveRunOptions {
    pub pack: PathBuf,
    pub seed: u64,
    pub out: Option<PathBuf>,
}

pub struct EvolveEmitOptions {
    pub pack: PathBuf,
    pub seed: u64,
    pub out: PathBuf,
    pub meta: PathBuf,
}

#[derive(Clone, Deserialize)]
pub struct EvolveSpec {
    schema: String,
    seed_program_ast: ProgramAst,
    fitness: FitnessSpec,
    budget: BudgetSpec,
    mutation_ops: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ProgramAst {
    program_id: String,
    constants: Vec<i64>,
    operator: String,
    statements: Vec<String>,
}

#[derive(Clone, Deserialize)]
struct FitnessSpec {
    target_value: i64,
}

#[derive(Clone, Deserialize)]
struct BudgetSpec {
    generations: u64,
    max_candidates: u64,
    step_limit: u64,
    timeout_ms: u64,
}

#[derive(Clone)]
struct Candidate {
    id: String,
    score: i64,
    value: i64,
    canon: String,
    canon_hash: String,
    state_hash: String,
}

#[derive(Serialize)]
pub struct EvolveMeta {
    schema: &'static str,
    master_seed: u64,
    pack_dir: String,
    spec_hash: String,
    mutation_ops: Vec<String>,
    generations: u64,
    evaluated_candidates: usize,
    sandbox_step_limit: u64,
    sandbox_timeout_ms: u64,
    deterministic_tiebreak: &'static str,
    best_score: i64,
    best_value: i64,
    best_program_id: String,
    best_program_canon_hash: String,
    final_state_hash: String,
}

#[derive(Clone)]
pub struct EvolveRunResult {
    pub best_program_canon_hash: String,
    pub final_state_hash: String,
    pub best_score: i64,
}

pub fn run(options: EvolveRunOptions) -> Result<EvolveRunResult, String> {
    let out_dir = options
        .out
        .unwrap_or_else(|| paths::build_dir().join("evolve"));
    fs::create_dir_all(&out_dir)
        .map_err(|e| format!("E_EVOLVE_OUT_DIR_CREATE {} ({})", out_dir.display(), e))?;
    let generated_path = out_dir.join("generated.ddn");
    let meta_path = out_dir.join("evolve_meta.json");
    let result = run_to_files(&options.pack, options.seed, &generated_path, &meta_path)?;
    println!("evolve_generated={}", generated_path.display());
    println!("evolve_meta={}", meta_path.display());
    println!("best_program_canon_hash={}", result.best_program_canon_hash);
    println!("final_state_hash={}", result.final_state_hash);
    println!("best_score={}", result.best_score);
    Ok(result)
}

pub fn emit(options: EvolveEmitOptions) -> Result<EvolveRunResult, String> {
    let result = run_to_files(&options.pack, options.seed, &options.out, &options.meta)?;
    println!("evolve_emit_out={}", options.out.display());
    println!("evolve_meta={}", options.meta.display());
    println!("best_program_canon_hash={}", result.best_program_canon_hash);
    println!("final_state_hash={}", result.final_state_hash);
    Ok(result)
}

pub fn run_to_files(
    pack: &Path,
    seed: u64,
    generated_path: &Path,
    meta_path: &Path,
) -> Result<EvolveRunResult, String> {
    let spec_path = pack.join("evolve_spec.json");
    let spec_bytes = fs::read(&spec_path)
        .map_err(|e| format!("E_EVOLVE_SPEC_READ {} ({})", spec_path.display(), e))?;
    let spec_text = String::from_utf8(spec_bytes.clone())
        .map_err(|_| format!("E_EVOLVE_SPEC_UTF8 {}", spec_path.display()))?;
    let spec: EvolveSpec =
        serde_json::from_str(&spec_text).map_err(|e| format!("E_EVOLVE_SPEC_PARSE {}", e))?;
    validate_spec(&spec)?;

    let best = evaluate_spec(&spec, seed)?;
    if let Some(parent) = generated_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("E_EVOLVE_OUT_DIR_CREATE {} ({})", parent.display(), e))?;
    }
    if let Some(parent) = meta_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("E_EVOLVE_META_DIR_CREATE {} ({})", parent.display(), e))?;
    }
    write_text(generated_path, &best.canon)
        .map_err(|e| format!("E_EVOLVE_GENERATED_WRITE {} ({})", generated_path.display(), e))?;

    let meta = EvolveMeta {
        schema: META_SCHEMA,
        master_seed: seed,
        pack_dir: stable_path_text(pack),
        spec_hash: format!("sha256:{}", sha256_hex(&spec_bytes)),
        mutation_ops: spec.mutation_ops.clone(),
        generations: spec.budget.generations,
        evaluated_candidates: (spec.budget.max_candidates as usize).saturating_add(1),
        sandbox_step_limit: spec.budget.step_limit,
        sandbox_timeout_ms: spec.budget.timeout_ms,
        deterministic_tiebreak: "score_desc_then_canon_hash_then_id",
        best_score: best.score,
        best_value: best.value,
        best_program_id: best.id.clone(),
        best_program_canon_hash: best.canon_hash.clone(),
        final_state_hash: best.state_hash.clone(),
    };
    let meta_text = serde_json::to_string_pretty(&meta)
        .map_err(|e| format!("E_EVOLVE_META_SERIALIZE {}", e))?
        + "\n";
    write_text(meta_path, &meta_text)
        .map_err(|e| format!("E_EVOLVE_META_WRITE {} ({})", meta_path.display(), e))?;

    Ok(EvolveRunResult {
        best_program_canon_hash: best.canon_hash,
        final_state_hash: best.state_hash,
        best_score: best.score,
    })
}

pub fn load_meta(path: &Path) -> Result<serde_json::Value, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("E_EVOLVE_META_READ {} ({})", path.display(), e))?;
    serde_json::from_str(&text).map_err(|e| format!("E_EVOLVE_META_PARSE {}", e))
}

fn validate_spec(spec: &EvolveSpec) -> Result<(), String> {
    if spec.schema != SPEC_SCHEMA {
        return Err(format!("E_EVOLVE_SPEC_SCHEMA schema={}", spec.schema));
    }
    if spec.seed_program_ast.constants.is_empty() {
        return Err("E_EVOLVE_SPEC_CONSTANTS_EMPTY".to_string());
    }
    if spec.seed_program_ast.statements.is_empty() {
        return Err("E_EVOLVE_SPEC_STATEMENTS_EMPTY".to_string());
    }
    if spec.budget.generations == 0 || spec.budget.max_candidates == 0 {
        return Err("E_EVOLVE_BUDGET_INVALID generations/max_candidates must be >= 1".to_string());
    }
    if spec.budget.step_limit == 0 || spec.budget.timeout_ms == 0 {
        return Err("E_EVOLVE_TIMEOUT sandbox limits must be >= 1".to_string());
    }
    if spec.mutation_ops.len() < REQUIRED_MUTATION_OP_COUNT {
        return Err(format!(
            "E_EVOLVE_MUTATION_OPS_TOO_FEW count={}",
            spec.mutation_ops.len()
        ));
    }
    for required in [
        "constant_delta",
        "operator_replace",
        "statement_insert",
        "statement_delete",
        "subtree_swap",
    ] {
        if !spec.mutation_ops.iter().any(|op| op == required) {
            return Err(format!("E_EVOLVE_MUTATION_OP_MISSING op={required}"));
        }
    }
    Ok(())
}

fn evaluate_spec(spec: &EvolveSpec, seed: u64) -> Result<Candidate, String> {
    let mut candidates = Vec::new();
    candidates.push(build_candidate(
        "seed".to_string(),
        spec.seed_program_ast.clone(),
        spec.fitness.target_value,
        Vec::new(),
    )?);
    for idx in 0..spec.budget.max_candidates {
        let op = &spec.mutation_ops[idx as usize % spec.mutation_ops.len()];
        let generation = (idx / spec.mutation_ops.len() as u64).saturating_add(1);
        if generation > spec.budget.generations {
            break;
        }
        let ast = apply_mutation(&spec.seed_program_ast, op, seed, idx)?;
        candidates.push(build_candidate(
            format!("g{generation:02}_{idx:03}_{op}"),
            ast,
            spec.fitness.target_value,
            vec![op.clone()],
        )?);
    }
    candidates.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.canon_hash.cmp(&b.canon_hash))
            .then_with(|| a.id.cmp(&b.id))
    });
    candidates
        .into_iter()
        .next()
        .ok_or_else(|| "E_EVOLVE_NO_CANDIDATE".to_string())
}

fn build_candidate(
    id: String,
    ast: ProgramAst,
    target: i64,
    op_trace: Vec<String>,
) -> Result<Candidate, String> {
    let value = eval_ast(&ast)?;
    let score = 0i64.saturating_sub((target - value).abs());
    let raw = render_program(&ast, value, score, &op_trace);
    let canon = canonicalize_program(&raw)?;
    let canon_hash = format!("sha256:{}", sha256_hex(canon.as_bytes()));
    let state_hash = state_hash(&canon_hash, value, score);
    Ok(Candidate {
        id,
        score,
        value,
        canon,
        canon_hash,
        state_hash,
    })
}

fn apply_mutation(ast: &ProgramAst, op: &str, seed: u64, idx: u64) -> Result<ProgramAst, String> {
    let mut out = ast.clone();
    match op {
        "constant_delta" => {
            let pos = choose_index(seed, idx, out.constants.len());
            let delta = if mix(seed, idx) % 2 == 0 { 1 } else { -1 };
            out.constants[pos] = out.constants[pos].saturating_add(delta);
        }
        "operator_replace" => {
            out.operator = match ast.operator.as_str() {
                "add" => "mul",
                "mul" => "add",
                "sub" => "add",
                _ => "add",
            }
            .to_string();
        }
        "statement_insert" => {
            let value = (mix(seed, idx) % 7) as i64;
            out.statements.push(format!("삽입값:수 <- {value}."));
        }
        "statement_delete" => {
            if out.statements.len() > 1 {
                let pos = choose_index(seed, idx, out.statements.len());
                out.statements.remove(pos);
            }
        }
        "subtree_swap" => {
            if out.constants.len() > 1 {
                let left = choose_index(seed, idx, out.constants.len());
                let right = (left + 1) % out.constants.len();
                out.constants.swap(left, right);
            }
        }
        other => return Err(format!("E_EVOLVE_MUTATION_OP_UNKNOWN op={other}")),
    }
    Ok(out)
}

fn eval_ast(ast: &ProgramAst) -> Result<i64, String> {
    let mut iter = ast.constants.iter().copied();
    let Some(first) = iter.next() else {
        return Err("E_EVOLVE_SPEC_CONSTANTS_EMPTY".to_string());
    };
    let value = match ast.operator.as_str() {
        "add" => iter.fold(first, |acc, v| acc.saturating_add(v)),
        "sub" => iter.fold(first, |acc, v| acc.saturating_sub(v)),
        "mul" => iter.fold(first, |acc, v| acc.saturating_mul(v)),
        other => return Err(format!("E_EVOLVE_OPERATOR operator={other}")),
    };
    Ok(value)
}

fn render_program(ast: &ProgramAst, value: i64, score: i64, op_trace: &[String]) -> String {
    let constants = ast
        .constants
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let trace = if op_trace.is_empty() {
        "seed".to_string()
    } else {
        op_trace.join(",")
    };
    let mut body = String::new();
    body.push_str("채비 {\n");
    body.push_str(&format!("  산출:수 <- {value}.\n"));
    body.push_str(&format!("  점수:수 <- {score}.\n"));
    body.push_str(&format!("  상수들:글 <- \"{constants}\".\n"));
    body.push_str(&format!("  연산:글 <- \"{}\".\n", ast.operator));
    body.push_str(&format!("  변이:글 <- \"{trace}\".\n"));
    for statement in &ast.statements {
        body.push_str("  ");
        body.push_str(statement.trim());
        if !statement.trim_end().ends_with('.') {
            body.push('.');
        }
        body.push('\n');
    }
    body.push_str("}.\n");
    body
}

fn canonicalize_program(source: &str) -> Result<String, String> {
    validate_no_legacy_frontdoor_surface(source)?;
    let input = prepare_frontdoor_canon_input(source);
    let output = crate::canon::canonicalize(&input.prepared, false)
        .map_err(|err| format!("E_EVOLVE_CANON_FAIL {}", err))?;
    Ok(output.ddn)
}

fn choose_index(seed: u64, idx: u64, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    (mix(seed, idx) as usize) % len
}

fn mix(seed: u64, idx: u64) -> u64 {
    let mut x = seed ^ idx.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 33;
    x = x.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    x ^= x >> 33;
    x = x.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    x ^= x >> 33;
    x
}

fn state_hash(canon_hash: &str, value: i64, score: i64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(canon_hash.as_bytes());
    hasher.update(value.to_string().as_bytes());
    hasher.update(score.to_string().as_bytes());
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

fn stable_path_text(path: &Path) -> String {
    let normalized = if let Ok(cwd) = std::env::current_dir() {
        path.strip_prefix(&cwd).unwrap_or(path)
    } else {
        path
    };
    normalized.to_string_lossy().replace('\\', "/")
}
