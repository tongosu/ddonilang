//! First uniform-motion notebook producer. Observation only, not Proof.
//! Exact coefficients and explicit unit bases are supplied by executed DDN.
//! No unit-bearing Fixed64 value is silently converted into an exact rational.
use crate::ddn_runtime::{DdnProgram, DdnRunOutput, DdnRunner};
use ddonirang_core::platform::NuriWorld;
use ddonirang_core::{Fixed64, InputSnapshot, InputSource, UnitDim};
use ddonirang_lang::runtime::Value;
use num_rational::BigRational;
use num_traits::{Signed, Zero};
use serde_json::{json, Value as Json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

fn fail(code: &str, detail: &str) -> String {
    format!("{code}: {detail}")
}
fn exact(v: &Value) -> Result<BigRational, String> {
    match v {
        Value::Rational(q)
            if q.ratio().numer().bits() <= 1024 && q.ratio().denom().bits() <= 1024 =>
        {
            Ok(q.ratio().clone())
        }
        Value::Rational(_) => Err(fail(
            "E_NOTEBOOK_BUDGET",
            "정확 계수 크기 한도를 넘었습니다",
        )),
        _ => Err(fail(
            "E_NOTEBOOK_EXACT_REQUIRED",
            "계수와 시각은 명시한 나눔수여야 합니다",
        )),
    }
}
fn basis(v: &Value, dim: UnitDim) -> Result<(), String> {
    match v {
        Value::Unit(u) if u.dim != dim => {
            Err(fail("E_NOTEBOOK_DIMENSION", "단위 기준의 차원이 다릅니다"))
        }
        Value::Unit(u) if u.value == Fixed64::from_i64(1) => Ok(()),
        _ => Err(fail(
            "E_NOTEBOOK_UNIT_BASIS",
            "첫 모형은 정규 크기 1의 실제 단위 기준을 요구합니다",
        )),
    }
}
fn number(q: &BigRational) -> Json {
    json!({"numerator":q.numer().to_string(),"denominator":q.denom().to_string()})
}
fn quantity(q: &BigRational, dim: UnitDim) -> Json {
    json!({"coefficient":number(q),"dimension":dim.exponents()})
}

/// Executes the original source once through the existing language/runtime.
/// Inputs are ordinary source-defined exact resources and explicit unit bases.
pub fn observe_uniform_motion(source: &str, path: &str) -> Result<String, String> {
    if source.len() > 65536 {
        return Err(fail("E_NOTEBOOK_BUDGET", "원문 크기 한도를 넘었습니다"));
    }
    let program = DdnProgram::from_v25_source(source, path)?;
    observe_runner(DdnRunner::new(program, "매마디"), source, path)
}

/// M6 passes the complete existing authored artifact, never edited metadata.
/// Admission, module ownership and the selected start are checked by their owners.
pub fn observe_uniform_motion_project(artifact: &str) -> Result<String, String> {
    if artifact.len() > 262144 {
        return Err(fail(
            "E_NOTEBOOK_BUDGET",
            "작품 원문 크기 한도를 넘었습니다",
        ));
    }
    let project = crate::authored_project::open(artifact.as_bytes())?;
    if project.source_profile != "v1-core-v25" {
        return Err(fail("E_NOTEBOOK_PROFILE", "지원 글판이 아닙니다"));
    }
    let owned =
        crate::project_revision_envelope_supply::admit_fresh_authored_project(artifact.as_bytes())?;
    let sources = owned.sources();
    let mut program = DdnProgram::from_saved_v25_project_with_mode(
        sources.clone(),
        ddonirang_lang::ParseMode::Strict,
        crate::ddn_runtime::default_age_target(),
    )?;
    program.bind_registered_project_context(&owned)?;
    crate::source_analysis::validate_program_for_execution(program.canonical_program())?;
    let mut runner = DdnRunner::new(program, "매마디");
    runner.configure_project_entry(sources.entry_declaration().name())?;
    let mut report: Json = serde_json::from_str(&observe_runner(
        runner,
        &sources.preparation().source,
        &sources.preparation().target_path,
    )?)
    .map_err(|e| e.to_string())?;
    report["authored_source"] = json!({"artifact_sha256":format!("{:x}",Sha256::digest(artifact.as_bytes())),
        "content_sha256":project.content_sha256,"project_identity":project.authored_identity.logical_dependency_id,
        "artifact_schema":project.schema,"source_profile":project.source_profile});
    Ok(report.to_string())
}

fn observe_runner(mut runner: DdnRunner, source: &str, path: &str) -> Result<String, String> {
    if source.len() > 65536 {
        return Err(fail(
            "E_NOTEBOOK_BUDGET",
            "연결 원문 크기 한도를 넘었습니다",
        ));
    }
    let input = InputSnapshot {
        tick_id: 0,
        dt: Fixed64::from_i64(1),
        keys_pressed: 0,
        last_key_name: String::new(),
        pointer_x_i32: 0,
        pointer_y_i32: 0,
        ai_injections: vec![],
        net_events: vec![],
        frame_source: InputSource::Person,
        rng_seed: 0,
    };
    let out = runner.run_update(&NuriWorld::new(), &input, &HashMap::new())?;
    observe_output(out, source, path)
}

fn observe_output(out: DdnRunOutput, source: &str, path: &str) -> Result<String, String> {
    let Some(Value::Tuple(coefficients)) = out.resources.get("탐구계수") else {
        return Err(fail(
            "E_NOTEBOOK_INPUT",
            "탐구계수는 (처음위치, 속도, 목표위치)의 정확 계수입니다",
        ));
    };
    if coefficients.len() != 3 {
        return Err(fail("E_NOTEBOOK_INPUT", "정확 계수는 세 개여야 합니다"));
    }
    let resource = |key: &str| {
        out.resources
            .get(key)
            .ok_or_else(|| fail("E_NOTEBOOK_INPUT", &format!("필수 입력 {key}가 없습니다")))
    };
    basis(resource("길이기준")?, UnitDim::LENGTH)?;
    basis(resource("시간기준")?, UnitDim::TIME)?;
    basis(resource("속도기준")?, UnitDim::SPEED)?; // Explicit normalization: (length/time)*time=length, using actual dimensions.
    if UnitDim::SPEED.add(UnitDim::TIME) != UnitDim::LENGTH {
        return Err(fail(
            "E_NOTEBOOK_DIMENSION",
            "속도와 시간의 차원 연결이 다릅니다",
        ));
    }
    let x0 = exact(&coefficients[0])?;
    let velocity = exact(&coefficients[1])?;
    let target = exact(&coefficients[2])?;
    let Value::List(times) = resource("탐구시각")? else {
        return Err(fail("E_NOTEBOOK_INPUT", "시각계수는 차림이어야 합니다"));
    };
    if times.is_empty() || times.len() > 256 {
        return Err(fail("E_NOTEBOOK_BUDGET", "시각 표본은 1~256개입니다"));
    }
    let mut rows = vec![];
    for value in times {
        let t = exact(value)?;
        if t.is_negative() {
            return Err(fail(
                "E_NOTEBOOK_TIME_DOMAIN",
                "이 모형의 시각은 0 이상입니다",
            ));
        }
        let x = &x0 + &velocity * &t;
        rows.push(
            json!({"time":quantity(&t,UnitDim::TIME),"position":quantity(&x,UnitDim::LENGTH)}),
        );
    }
    // Internal solver input is derived from validated exact coefficients and
    // explicit bases, never from stripped UI labels or a second DDN parser.
    let q = |n: &BigRational| format!("({}/{})", n.numer(), n.denom());
    let lhs = format!("{}+{}*t", q(&x0), q(&velocity));
    let rhs = q(&target);
    use ddonirang_symbolic::{solve_linear_equation, RelationSolveOutcome};
    let solution = match solve_linear_equation(&lhs, &rhs)? {
        RelationSolveOutcome::NoSolution => json!({"kind":"no_solution"}),
        RelationSolveOutcome::NonUnique => json!({"kind":"non_unique"}),
        RelationSolveOutcome::Solution(bindings) => {
            let b = bindings
                .get("t")
                .ok_or_else(|| fail("E_NOTEBOOK_SOLVER", "시간 해가 없습니다"))?;
            let denominator = b
                .denominator
                .parse()
                .map_err(|_| fail("E_NOTEBOOK_SOLVER", "분모 해석 실패"))?;
            if num_bigint::BigInt::is_zero(&denominator) {
                return Err(fail("E_NOTEBOOK_SOLVER", "분모 0"));
            }
            let t = BigRational::new(
                b.numerator
                    .parse()
                    .map_err(|_| fail("E_NOTEBOOK_SOLVER", "분자 해석 실패"))?,
                denominator,
            );
            // Algebraic substitution is independent of the lesson's time domain.
            let left = &x0 + &velocity * &t;
            let substitution = json!({"left":quantity(&left,UnitDim::LENGTH),
                "right":quantity(&target,UnitDim::LENGTH),"equal":left==target});
            if t.is_negative() {
                json!({"kind":"outside_domain","algebraic_time":quantity(&t,UnitDim::TIME),
                    "substitution":substitution})
            } else {
                json!({"kind":"solution","time":quantity(&t,UnitDim::TIME),
                    "substitution":substitution})
            }
        }
    };
    Ok(json!({"schema":"ddn.math-notebook.observation.v1","model":"uniform_motion","source_sha256":format!("{:x}",Sha256::digest(source.as_bytes())),
        "source_path":path,"proof":false,"time_domain":"t>=0","rows":rows,"solution":solution,
        "inputs":{"initial_position":quantity(&x0,UnitDim::LENGTH),"velocity":quantity(&velocity,UnitDim::SPEED),"target":quantity(&target,UnitDim::LENGTH)},
        "normalization":{"length_basis":1,"time_basis":1,"velocity_basis":1,"equation_left":lhs,"equation_right":rhs,
            "unit_dimension_schema":UnitDim::WIRE_SCHEMA_V2},"source_profile":"v1-core-v25"}).to_string())
}
