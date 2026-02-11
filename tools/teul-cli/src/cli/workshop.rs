use std::fs;
use std::path::Path;

use super::detjson::write_text;

pub fn run_gen(geoul: &Path, out_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let manifest = format!(
        "{{\"schema\":\"workshop.v0\",\"geoul_dir\":\"{}\"}}",
        escape_json(&geoul.display().to_string())
    );
    write_text(&out_dir.join("manifest.detjson"), &manifest)?;

    let html = build_html(&geoul.display().to_string());
    write_text(&out_dir.join("index.html"), &html)?;
    println!("workshop_out={}", out_dir.display());
    Ok(())
}

pub fn run_apply(workshop_dir: &Path, patch: &Path) -> Result<(), String> {
    fs::create_dir_all(workshop_dir).map_err(|e| e.to_string())?;
    let content = fs::read_to_string(patch).map_err(|e| e.to_string())?;
    let target = workshop_dir.join("applied.patch.detjson");
    write_text(&target, &content)?;
    println!("applied_patch={}", target.display());
    Ok(())
}

pub fn run_open(workshop_dir: &Path) -> Result<(), String> {
    let index = workshop_dir.join("index.html");
    if !index.exists() {
        return Err("E_WORKSHOP_MISSING index.html이 없습니다".to_string());
    }
    println!("workshop_index={}", index.display());
    Ok(())
}

fn build_html(geoul: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"ko\">\n<head>\n<meta charset=\"utf-8\">\n<title>Workshop v0</title>\n</head>\n<body>\n<h1>Workshop v0</h1>\n<p>geoul: {}</p>\n</body>\n</html>\n",
        escape_html(geoul)
    )
}

fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}