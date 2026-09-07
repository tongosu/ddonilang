use std::fs;
use std::path::Path;

use crate::artifact_output::write_text_artifact_atomic;
use crate::canon::extract_prompt_string_literals;
use crate::state_trace_wire::escape_json_string_contents as escape_json;

pub fn extract(in_path: &Path, out_path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(in_path).map_err(|error| error.to_string())?;
    let prompts = extract_prompt_string_literals(&source).map_err(|error| error.to_string())?;

    let mut output = String::new();
    output.push_str("{\"version\":0,\"holes\":[");
    for (index, prompt) in prompts.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str("{\"id\":");
        output.push_str(&index.to_string());
        output.push_str(",\"prompt\":\"");
        output.push_str(&escape_json(prompt));
        output.push_str("\"}");
    }
    output.push_str("],\"prompt\":\"");
    output.push_str(&escape_json(
        prompts.first().map(String::as_str).unwrap_or(""),
    ));
    output.push_str("\"}\n");

    write_text_artifact_atomic(out_path, &output)
}
