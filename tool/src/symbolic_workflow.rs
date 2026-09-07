use std::path::Path;

use crate::artifact_output::write_text_artifact_atomic;

pub fn run_canon(input: &str) -> Result<(), String> {
    println!("{}", ddonirang_symbolic::canon(input)?);
    Ok(())
}

pub fn run_simplify(input: &str) -> Result<(), String> {
    println!("{}", ddonirang_symbolic::simplify(input)?);
    Ok(())
}

pub fn run_expand(input: &str) -> Result<(), String> {
    println!("{}", ddonirang_symbolic::expand(input)?);
    Ok(())
}

pub fn run_factor(input: &str) -> Result<(), String> {
    println!("{}", ddonirang_symbolic::factor(input)?);
    Ok(())
}

pub fn run_diff(input: &str, var: &str) -> Result<(), String> {
    println!("{}", ddonirang_symbolic::diff(input, var)?);
    Ok(())
}

pub fn run_integrate(input: &str, var: &str) -> Result<(), String> {
    println!("{}", ddonirang_symbolic::integrate(input, var)?);
    Ok(())
}

pub fn run_equiv(lhs: &str, rhs: &str) -> Result<(), String> {
    println!("equivalent={}", ddonirang_symbolic::equivalent(lhs, rhs)?);
    Ok(())
}

pub fn run_relation_canon(lhs: &str, rhs: &str) -> Result<(), String> {
    println!("{}", ddonirang_symbolic::relation_canon(lhs, rhs)?);
    Ok(())
}

pub fn run_prove_eq(lhs: &str, rhs: &str, out: Option<&Path>) -> Result<(), String> {
    let cert = ddonirang_symbolic::prove_equivalent(lhs, rhs)?;
    if !cert.equivalent {
        return Err("E_SYMBOLIC_EQUIVALENCE_NOT_PROVEN_INTERNAL".to_string());
    }
    let text = ddonirang_symbolic::to_detjson(&cert)?;
    if let Some(path) = out {
        write_text_artifact_atomic(path, &text)?;
    } else {
        println!("{text}");
    }
    println!("proof_ok=true equivalent=true");
    Ok(())
}
