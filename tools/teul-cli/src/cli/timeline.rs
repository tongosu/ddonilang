use std::fs;
use std::path::Path;

use serde_json::{Map, Value};

use crate::core::geoul::GeoulBundleReader;

pub fn run_make(geoul_dir: &Path, story_path: &Path, out_path: &Path) -> Result<(), String> {
    let reader = GeoulBundleReader::open(geoul_dir)?;
    let frame_count = reader.frame_count();
    let story_text = fs::read_to_string(story_path).map_err(|e| e.to_string())?;
    let story_json: Value = serde_json::from_str(&story_text).map_err(|e| e.to_string())?;
    let scenes = story_json
        .get("scenes")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    let mut items = Vec::with_capacity(scenes.len());
    for scene in scenes {
        let Some(obj) = scene.as_object() else {
            continue;
        };
        let t0 = obj.get("t0").cloned().unwrap_or(Value::Number(0.into()));
        let t1 = obj.get("t1").cloned().unwrap_or(Value::Number(0.into()));
        let kind = obj
            .get("kind")
            .cloned()
            .unwrap_or(Value::String("summary".to_string()));
        let text = obj
            .get("text")
            .cloned()
            .unwrap_or(Value::String(String::new()));
        let mut item = Map::new();
        item.insert("t0".to_string(), t0);
        item.insert("t1".to_string(), t1);
        item.insert("kind".to_string(), kind);
        item.insert("text".to_string(), text);
        items.push(Value::Object(item));
    }

    let mut root = Map::new();
    root.insert("version".to_string(), Value::Number(1.into()));
    root.insert("frames".to_string(), Value::Number(frame_count.into()));
    root.insert("items".to_string(), Value::Array(items));
    let text = serde_json::to_string_pretty(&Value::Object(root))
        .map_err(|e| format!("E_TIMELINE_JSON {}", e))?;

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(out_path, text + "\n").map_err(|e| e.to_string())?;
    println!("timeline_written={}", out_path.display());
    Ok(())
}
