use std::fs;
use std::path::Path;

use crate::core::geoul::GeoulBundleReader;

pub fn run_make(geoul_dir: &Path, out_path: &Path) -> Result<(), String> {
    let reader = GeoulBundleReader::open(geoul_dir)?;
    let frame_count = reader.frame_count();
    let t1 = if frame_count == 0 { 0 } else { frame_count - 1 };
    let summary = format!("frames={}", frame_count);
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"version\": 1,\n");
    out.push_str(&format!("  \"summary\": \"{}\",\n", summary));
    out.push_str("  \"scenes\": [\n");
    out.push_str(&format!(
        "    {{\"t0\": 0, \"t1\": {}, \"kind\": \"summary\", \"text\": \"{}\"}}\n",
        t1, summary
    ));
    out.push_str("  ],\n");
    out.push_str("  \"suggested_intents\": [\n");
    if frame_count > 0 {
        out.push_str(&format!(
            "    {{\"agent_id\": 1, \"recv_seq\": 1, \"intent\": {{\"kind\": \"말하기\", \"text\": \"{}\"}}}}\n",
            summary
        ));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(out_path, out).map_err(|e| e.to_string())?;
    println!("story_written={}", out_path.display());
    Ok(())
}
