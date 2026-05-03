use std::fs;
use std::path::Path;

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
    let text = ddonirang_symbolic::to_detjson(&cert)?;
    if let Some(path) = out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(path, text).map_err(|e| e.to_string())?;
    } else {
        println!("{text}");
    }
    println!("proof_ok=true equivalent={}", cert.equivalent);
    Ok(())
}
