use std::fs;
use std::path::Path;

use crate::artifact_output::{write_text_artifact_atomic, write_text_artifacts_atomic};
use crate::state_trace_wire::escape_json_string_contents as escape_json;

pub fn run_gen(geoul: &Path, out_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(out_dir).map_err(|error| error.to_string())?;
    let manifest = format!(
        "{{\"schema\":\"workshop.v0\",\"geoul_dir\":\"{}\"}}",
        escape_json(&geoul.display().to_string())
    );
    let html = build_html(&geoul.display().to_string());
    let manifest_path = out_dir.join("manifest.detjson");
    let index_path = out_dir.join("index.html");
    write_text_artifacts_atomic(&[
        (manifest_path.as_path(), manifest.as_str()),
        (index_path.as_path(), html.as_str()),
    ])?;
    println!("workshop_out={}", out_dir.display());
    Ok(())
}

pub fn run_apply(workshop_dir: &Path, patch: &Path) -> Result<(), String> {
    fs::create_dir_all(workshop_dir).map_err(|error| error.to_string())?;
    let content = fs::read_to_string(patch).map_err(|error| error.to_string())?;
    let target = workshop_dir.join("applied.patch.detjson");
    write_text_artifact_atomic(&target, &content)?;
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

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
