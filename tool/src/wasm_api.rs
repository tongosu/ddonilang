use std::collections::{BTreeMap, HashMap};

use blake3::hash as blake3_hash;
use ddonirang_core::platform::{
    DetNuri, InputSnapshot, Patch, PatchOp, ResourceMapEntry, ResourceValue,
};
use ddonirang_core::{Fixed64, Nuri, ResourceHandle, SeulgiIntent, SeulgiPacket};
use ddonirang_lang::ParseMode;
use ddonirang_lang::runtime::Value;
use serde_json::{json, Map, Value as JsonValue};
use wasm_bindgen::prelude::*;

use crate::ddn_runtime::{DdnProgram, DdnRunner};
use crate::preprocess::{preprocess_source_for_parse, split_file_meta};

const DEFAULT_UPDATE_NAME: &str = "매마디";
const ENGINE_RESPONSE_SCHEMA: &str = "seamgrim.engine_response.v0";
const OBSERVATION_MANIFEST_SCHEMA: &str = "ddn.observation_manifest.v0";
const OBSERVATION_MANIFEST_VERSION: &str = "20.6.33";
const SPACE2D_SCHEMA: &str = "seamgrim.space2d.v0";
const BOGAE_DRAWLIST_TAG: &str = "보개_그림판_목록";
const BOGAE_WIDTH_TAG: &str = "보개_그림판_가로";
const BOGAE_HEIGHT_TAG: &str = "보개_그림판_세로";
const GRAPH_SCHEMA: &str = "seamgrim.graph.v0";
const DEFAULT_VIEW_PREFIXES: &[&str] = &["보개_", "__view_"];
const GRAPH_POINTS_TAGS: &[(&str, &str, &str)] = &[
    ("그래프_점목록_f", "f", "f(x)"),
    ("그래프_점목록_df", "df", "f'(x)"),
    ("그래프_점목록_fi", "fi", "int(f)"),
    ("graph_points_f", "f", "f(x)"),
    ("graph_points_df", "df", "f'(x)"),
    ("graph_points_fi", "fi", "int(f)"),
];

fn parse_mode_from_str(mode: &str) -> Result<ParseMode, JsValue> {
    let normalized = mode.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "" => Ok(ParseMode::Strict),
        "strict" => Ok(ParseMode::Strict),
        _ => Err(JsValue::from_str(&format!(
            "지원하지 않는 lang-mode: {mode} (strict)"
        ))),
    }
}

#[wasm_bindgen]
pub fn wasm_build_info() -> String {
    format!(
        "pkg={} version={} state_schema={} update={}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        ENGINE_RESPONSE_SCHEMA,
        DEFAULT_UPDATE_NAME
    )
}

#[wasm_bindgen]
pub fn wasm_preprocess_source(source: &str) -> Result<String, JsValue> {
    let meta = split_file_meta(source);
    preprocess_source_for_parse(&meta.stripped).map_err(|err| JsValue::from_str(&err))
}

#[wasm_bindgen]
pub struct DdnWasmVm {
    runner: DdnRunner,
    world: DetNuri,
    defaults: HashMap<String, Value>,
    param_overrides: BTreeMap<String, Value>,
    tick_id: u64,
    rng_seed: u64,
    view_prefixes: Vec<String>,
    input_keys_pressed: u64,
    input_last_key_name: String,
    input_pointer_x_i32: i32,
    input_pointer_y_i32: i32,
    input_dt: Fixed64,
    last_patch: Option<Patch>,
    pending_ai_injections: Vec<(String, String)>,
    lang_mode: ParseMode,
}

#[wasm_bindgen]
impl DdnWasmVm {
    #[wasm_bindgen(constructor)]
    pub fn new(source: &str) -> Result<DdnWasmVm, JsValue> {
        let program = DdnProgram::from_source_with_mode(source, "<wasm>", ParseMode::Strict)
            .map_err(|err| JsValue::from_str(&err))?;
        let mut defaults = HashMap::new();
        seed_bogae_defaults(&mut defaults);
        Ok(DdnWasmVm {
            runner: DdnRunner::new(program, DEFAULT_UPDATE_NAME),
            world: DetNuri::new(),
            defaults,
            param_overrides: BTreeMap::new(),
            tick_id: 0,
            rng_seed: 0,
            view_prefixes: DEFAULT_VIEW_PREFIXES
                .iter()
                .map(|prefix| (*prefix).to_string())
                .collect(),
            input_keys_pressed: 0,
            input_last_key_name: String::new(),
            input_pointer_x_i32: 0,
            input_pointer_y_i32: 0,
            input_dt: Fixed64::from_i64(1),
            last_patch: None,
            pending_ai_injections: Vec::new(),
            lang_mode: ParseMode::Strict,
        })
    }

    pub fn new_with_mode(source: &str, mode: &str) -> Result<DdnWasmVm, JsValue> {
        let mode = parse_mode_from_str(mode)?;
        let program = DdnProgram::from_source_with_mode(source, "<wasm>", mode)
            .map_err(|err| JsValue::from_str(&err))?;
        let mut defaults = HashMap::new();
        seed_bogae_defaults(&mut defaults);
        Ok(DdnWasmVm {
            runner: DdnRunner::new(program, DEFAULT_UPDATE_NAME),
            world: DetNuri::new(),
            defaults,
            param_overrides: BTreeMap::new(),
            tick_id: 0,
            rng_seed: 0,
            view_prefixes: DEFAULT_VIEW_PREFIXES
                .iter()
                .map(|prefix| (*prefix).to_string())
                .collect(),
            input_keys_pressed: 0,
            input_last_key_name: String::new(),
            input_pointer_x_i32: 0,
            input_pointer_y_i32: 0,
            input_dt: Fixed64::from_i64(1),
            last_patch: None,
            pending_ai_injections: Vec::new(),
            lang_mode: mode,
        })
    }

    pub fn update_logic(&mut self, source: &str) -> Result<(), JsValue> {
        let program = DdnProgram::from_source_with_mode(source, "<wasm>", self.lang_mode)
            .map_err(|err| JsValue::from_str(&err))?;
        self.runner = DdnRunner::new(program, DEFAULT_UPDATE_NAME);
        Ok(())
    }

    pub fn update_logic_with_mode(&mut self, source: &str, mode: &str) -> Result<(), JsValue> {
        let mode = parse_mode_from_str(mode)?;
        let program = DdnProgram::from_source_with_mode(source, "<wasm>", mode)
            .map_err(|err| JsValue::from_str(&err))?;
        self.runner = DdnRunner::new(program, DEFAULT_UPDATE_NAME);
        self.lang_mode = mode;
        Ok(())
    }

    pub fn get_build_info(&self) -> String {
        format!(
            "pkg={} version={} state_schema={} update={}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            ENGINE_RESPONSE_SCHEMA,
            DEFAULT_UPDATE_NAME
        )
    }

    pub fn set_rng_seed(&mut self, seed: u64) {
        self.rng_seed = seed;
    }

    pub fn add_view_prefix(&mut self, prefix: &str) {
        let trimmed = prefix.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.view_prefixes.iter().any(|p| p == trimmed) {
            return;
        }
        self.view_prefixes.push(trimmed.to_string());
    }

    pub fn clear_view_prefixes(&mut self) {
        self.view_prefixes.clear();
    }

    pub fn set_input(
        &mut self,
        keys_pressed: u32,
        last_key_name: &str,
        pointer_x_i32: i32,
        pointer_y_i32: i32,
        dt: f64,
    ) {
        self.input_keys_pressed = u64::from(keys_pressed);
        self.input_last_key_name = last_key_name.to_string();
        self.input_pointer_x_i32 = pointer_x_i32;
        self.input_pointer_y_i32 = pointer_y_i32;
        if dt.is_finite() {
            self.input_dt = fixed64_from_f64_checked(dt);
        }
    }

    pub fn set_keys_pressed(&mut self, keys_pressed: u32) {
        self.input_keys_pressed = u64::from(keys_pressed);
    }

    pub fn set_last_key_name(&mut self, last_key_name: &str) {
        self.input_last_key_name = last_key_name.to_string();
    }

    pub fn set_pointer(&mut self, pointer_x_i32: i32, pointer_y_i32: i32) {
        self.input_pointer_x_i32 = pointer_x_i32;
        self.input_pointer_y_i32 = pointer_y_i32;
    }

    pub fn set_dt_f64(&mut self, dt: f64) {
        if dt.is_finite() {
            self.input_dt = fixed64_from_f64_checked(dt);
        }
    }

    pub fn columns(&self) -> JsValue {
        let (columns, row) = collect_columns_and_row(self.world.world());
        let observation_manifest = build_observation_manifest(&columns);
        let payload = json!({
            "columns": columns,
            "row": row,
            "observation_manifest": observation_manifest,
            "deprecated": true,
            "use_instead": "step_one() 또는 get_state_json()의 channels/row",
        });
        JsValue::from_str(&payload.to_string())
    }

    pub fn set_param(&mut self, key: &str, value: JsValue) -> Result<JsValue, JsValue> {
        let key = key.trim();
        if key.is_empty() {
            return Err(wasm_error(
                "PARAM_KEY_EMPTY",
                "set_param: key가 비어 있습니다",
                "변수 이름을 입력해 주세요. 예: set_param(\"속도\", 10)",
            ));
        }
        let parsed = js_scalar_to_runtime_value(&value).ok_or_else(|| {
            wasm_error(
                "PARAM_VALUE_INVALID",
                "set_param: value는 수/참거짓/글 스칼라만 허용됩니다",
                "수/참거짓/글 스칼라만 전달해 주세요.",
            )
        })?;
        apply_param_value(self.world.world_mut(), key, &parsed)
            .map_err(|err| wasm_error("PARAM_APPLY_FAILED", &err, "대상 변수 이름과 타입을 확인해 주세요."))?;
        self.param_overrides.insert(key.to_string(), parsed);

        let payload = json!({
            "ok": true,
            "state_hash": current_state_hash(&self.world, &self.view_prefixes),
            "diag": [],
        });
        Ok(JsValue::from_str(&payload.to_string()))
    }

    pub fn set_param_fixed64(&mut self, key: &str, raw_i64: i64) -> Result<JsValue, JsValue> {
        let key = key.trim();
        if key.is_empty() {
            return Err(wasm_error(
                "PARAM_KEY_EMPTY",
                "set_param_fixed64: key가 비어 있습니다",
                "변수 이름을 입력해 주세요. 예: set_param_fixed64(\"속도\", 4294967296)",
            ));
        }
        let parsed = Value::Fixed64(Fixed64::from_raw_i64(raw_i64));
        apply_param_value(self.world.world_mut(), key, &parsed)
            .map_err(|err| wasm_error("PARAM_APPLY_FAILED", &err, "대상 변수 이름과 타입을 확인해 주세요."))?;
        self.param_overrides.insert(key.to_string(), parsed);
        let payload = json!({
            "ok": true,
            "state_hash": current_state_hash(&self.world, &self.view_prefixes),
            "diag": [],
            "raw_i64": raw_i64,
        });
        Ok(JsValue::from_str(&payload.to_string()))
    }

    pub fn set_param_fixed64_str(&mut self, key: &str, raw_i64: &str) -> Result<JsValue, JsValue> {
        let key = key.trim();
        if key.is_empty() {
            return Err(wasm_error(
                "PARAM_KEY_EMPTY",
                "set_param_fixed64_str: key가 비어 있습니다",
                "변수 이름을 입력해 주세요. 예: set_param_fixed64_str(\"속도\", \"4294967296\")",
            ));
        }
        let trimmed = raw_i64.trim();
        if trimmed.is_empty() {
            return Err(wasm_error(
                "PARAM_RAW_I64_EMPTY",
                "set_param_fixed64_str: raw_i64가 비어 있습니다",
                "raw_i64 정수 문자열을 입력해 주세요. 예: \"4294967296\"",
            ));
        }
        let parsed = trimmed.parse::<i64>().map_err(|_| {
            wasm_error(
                "PARAM_RAW_I64_INVALID",
                "set_param_fixed64_str: raw_i64는 i64 정수 문자열이어야 합니다",
                "정수 문자열만 허용됩니다. 예: -123, 4294967296",
            )
        })?;
        let value = Value::Fixed64(Fixed64::from_raw_i64(parsed));
        apply_param_value(self.world.world_mut(), key, &value)
            .map_err(|err| wasm_error("PARAM_APPLY_FAILED", &err, "대상 변수 이름과 타입을 확인해 주세요."))?;
        self.param_overrides.insert(key.to_string(), value);
        let payload = json!({
            "ok": true,
            "state_hash": current_state_hash(&self.world, &self.view_prefixes),
            "diag": [],
            "raw_i64": trimmed,
        });
        Ok(JsValue::from_str(&payload.to_string()))
    }

    pub fn reset(&mut self, keep_params: Option<bool>) -> Result<JsValue, JsValue> {
        let keep_params = keep_params.unwrap_or(false);
        self.world = DetNuri::new();
        self.tick_id = 0;
        self.rng_seed = 0;
        self.input_keys_pressed = 0;
        self.input_last_key_name.clear();
        self.input_pointer_x_i32 = 0;
        self.input_pointer_y_i32 = 0;
        self.input_dt = Fixed64::from_i64(1);
        self.last_patch = None;
        self.pending_ai_injections.clear();
        self.runner.reset_transient_state();

        if !keep_params {
            self.param_overrides.clear();
        } else {
            for (key, value) in &self.param_overrides {
                apply_param_value(self.world.world_mut(), key, value)
                    .map_err(|err| JsValue::from_str(&err))?;
            }
        }

        let (columns, row) = collect_columns_and_row(self.world.world());
        let observation_manifest = build_observation_manifest(&columns);
        let payload = json!({
            "ok": true,
            "tick": self.tick_id,
            "columns": columns,
            "row": row,
            "observation_manifest": observation_manifest,
            "state_hash": current_state_hash(&self.world, &self.view_prefixes),
            "diag": [],
        });
        Ok(JsValue::from_str(&payload.to_string()))
    }

    pub fn step_one_with_input(
        &mut self,
        keys_pressed: u32,
        last_key_name: &str,
        pointer_x_i32: i32,
        pointer_y_i32: i32,
        dt: f64,
    ) -> Result<JsValue, JsValue> {
        self.set_input(
            keys_pressed,
            last_key_name,
            pointer_x_i32,
            pointer_y_i32,
            dt,
        );
        self.step_one()
    }

    pub fn step_one(&mut self) -> Result<JsValue, JsValue> {
        let tick_seed = self.rng_seed ^ self.tick_id;
        let ai_injections = self
            .pending_ai_injections
            .drain(..)
            .enumerate()
            .map(|(idx, (key, value_json))| SeulgiPacket {
                agent_id: 0,
                recv_seq: idx as u64,
                accepted_madi: self.tick_id,
                target_madi: self.tick_id,
                intent: SeulgiIntent::Say {
                    text: format!("{key}={value_json}"),
                },
            })
            .collect();
        let input = InputSnapshot {
            tick_id: self.tick_id,
            dt: self.input_dt,
            keys_pressed: self.input_keys_pressed,
            last_key_name: self.input_last_key_name.clone(),
            pointer_x_i32: self.input_pointer_x_i32,
            pointer_y_i32: self.input_pointer_y_i32,
            ai_injections,
            net_events: Vec::new(),
            rng_seed: tick_seed,
        };

        let output = self
            .runner
            .run_update(self.world.world(), &input, &self.defaults)
            .map_err(|err| JsValue::from_str(&err))?;

        let mut sink = ddonirang_core::signals::VecSignalSink::default();
        self.world
            .apply_patch(&output.patch, input.tick_id, &mut sink);

        self.last_patch = Some(output.patch);
        let state_hash = current_state_hash(&self.world, &self.view_prefixes);
        let resources = serialize_world_resources(self.world.world());
        let (columns, row) = collect_columns_and_row(self.world.world());
        let streams = collect_streams(self.world.world());
        let view_meta = build_view_meta(self.world.world());
        let view_hash = hash_json_string(&view_meta);
        let snapshot_resources = serialize_world_resources_snapshot(self.world.world());
        let snapshot_params = serialize_param_overrides(&self.param_overrides);
        let observation_manifest = build_observation_manifest(&columns);
        let patch_json = self
            .last_patch
            .as_ref()
            .map(serialize_patch)
            .unwrap_or_else(|| JsonValue::Array(Vec::new()));

        self.tick_id = self.tick_id.saturating_add(1);

        let input_json = serialize_input_snapshot(&input, Some(self.rng_seed));
        let payload = json!({
            "schema": ENGINE_RESPONSE_SCHEMA,
            "tick_id": input.tick_id,
            "state_hash": state_hash,
            "input": input_json.clone(),
            "resources": resources,
            "channels": columns.clone(),
            "row": row.clone(),
            "observation_manifest": observation_manifest.clone(),
            "streams": streams.clone(),
            "patch": patch_json,
            "state": {
                "tick_id": input.tick_id,
                "input": input_json,
                "resources": serialize_world_resources(self.world.world()),
                "channels": columns,
                "row": row,
                "observation_manifest": observation_manifest,
                "patch": self
                    .last_patch
                    .as_ref()
                    .map(serialize_patch)
                    .unwrap_or_else(|| JsonValue::Array(Vec::new())),
                "streams": streams,
                "snapshot_v1": {
                    "resources": snapshot_resources,
                    "param_overrides": snapshot_params,
                }
            },
            "view_meta": view_meta,
            "view_hash": view_hash,
        });
        Ok(JsValue::from_str(&payload.to_string()))
    }

    pub fn get_state_hash(&self) -> String {
        current_state_hash(&self.world, &self.view_prefixes)
    }

    pub fn get_state_json(&self) -> JsValue {
        let resources = serialize_world_resources(self.world.world());
        let (columns, row) = collect_columns_and_row(self.world.world());
        let streams = collect_streams(self.world.world());
        let view_meta = build_view_meta(self.world.world());
        let view_hash = hash_json_string(&view_meta);
        let snapshot_resources = serialize_world_resources_snapshot(self.world.world());
        let snapshot_params = serialize_param_overrides(&self.param_overrides);
        let observation_manifest = build_observation_manifest(&columns);
        let payload = json!({
            "schema": ENGINE_RESPONSE_SCHEMA,
            "tick_id": self.tick_id,
            "state_hash": current_state_hash(&self.world, &self.view_prefixes),
            "input": serialize_input_state(self),
            "resources": resources,
            "channels": columns.clone(),
            "row": row.clone(),
            "observation_manifest": observation_manifest.clone(),
            "streams": streams.clone(),
            "state": {
                "tick_id": self.tick_id,
                "input": serialize_input_state(self),
                "resources": serialize_world_resources(self.world.world()),
                "channels": columns,
                "row": row,
                "observation_manifest": observation_manifest,
                "patch": JsonValue::Array(Vec::new()),
                "streams": streams,
                "snapshot_v1": {
                    "resources": snapshot_resources,
                    "param_overrides": snapshot_params,
                }
            },
            "view_meta": view_meta,
            "view_hash": view_hash,
        });
        JsValue::from_str(&payload.to_string())
    }

    pub fn restore_state(&mut self, state_json: &str) -> Result<JsValue, JsValue> {
        let payload: JsonValue = serde_json::from_str(state_json)
            .map_err(|err| JsValue::from_str(&format!("restore_state: JSON 파싱 실패: {err}")))?;
        let schema = payload
            .get("schema")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if schema != ENGINE_RESPONSE_SCHEMA {
            return Err(JsValue::from_str(&format!(
                "restore_state: 지원하지 않는 schema: {schema}"
            )));
        }

        let state_channel = payload
            .get("state")
            .cloned()
            .unwrap_or_else(|| JsonValue::Object(Map::new()));
        let snapshot_v1 = state_channel
            .get("snapshot_v1")
            .cloned()
            .unwrap_or_else(|| JsonValue::Object(Map::new()));
        let snapshot_resources = snapshot_v1
            .get("resources")
            .cloned()
            .or_else(|| state_channel.get("resources").cloned())
            .or_else(|| payload.get("resources").cloned())
            .unwrap_or_else(|| JsonValue::Object(Map::new()));

        self.world = DetNuri::new();
        restore_world_resources(self.world.world_mut(), &snapshot_resources)
            .map_err(|err| JsValue::from_str(&format!("restore_state: {err}")))?;

        self.param_overrides.clear();
        if let Some(params) = snapshot_v1.get("param_overrides") {
            let restored = parse_param_overrides(params)
                .map_err(|err| JsValue::from_str(&format!("restore_state: {err}")))?;
            self.param_overrides = restored;
        }

        let tick_id = state_channel
            .get("tick_id")
            .or_else(|| payload.get("tick_id"))
            .and_then(json_to_u64)
            .unwrap_or(0);
        self.tick_id = tick_id;
        self.last_patch = None;

        if let Some(input) = state_channel.get("input").or_else(|| payload.get("input")) {
            restore_input_state(self, input);
        } else {
            self.rng_seed = 0;
            self.input_keys_pressed = 0;
            self.input_last_key_name.clear();
            self.input_pointer_x_i32 = 0;
            self.input_pointer_y_i32 = 0;
            self.input_dt = Fixed64::from_i64(1);
        }
        self.pending_ai_injections.clear();
        self.runner.reset_transient_state();

        let (columns, row) = collect_columns_and_row(self.world.world());
        let observation_manifest = build_observation_manifest(&columns);
        let state_hash = current_state_hash(&self.world, &self.view_prefixes);
        let view_hash = hash_json_string(&build_view_meta(self.world.world()));
        let diag = restore_hash_diags(&payload, &state_hash, &view_hash);
        let out = json!({
            "ok": true,
            "tick": self.tick_id,
            "columns": columns,
            "row": row,
            "observation_manifest": observation_manifest,
            "state_hash": state_hash,
            "view_hash": view_hash,
            "diag": diag,
        });
        Ok(JsValue::from_str(&out.to_string()))
    }

    pub fn inject_ai_action(&mut self, key: &str, value_json: &str) {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return;
        }
        self.pending_ai_injections
            .push((trimmed.to_string(), value_json.to_string()));
    }

    pub fn clear_ai_injections(&mut self) {
        self.pending_ai_injections.clear();
    }
}

fn current_state_hash(world: &DetNuri, view_prefixes: &[String]) -> String {
    let refs: Vec<&str> = view_prefixes.iter().map(String::as_str).collect();
    format!(
        "blake3:{}",
        world
            .world()
            .state_hash_excluding_resource_prefixes(&refs)
            .to_hex()
    )
}

fn hash_json_string(value: &JsonValue) -> String {
    let canonical = value.to_string();
    let digest = blake3_hash(canonical.as_bytes());
    format!("blake3:{}", hex::encode(digest.as_bytes()))
}

fn restore_hash_diags(
    payload: &JsonValue,
    actual_state_hash: &str,
    actual_view_hash: &str,
) -> Vec<JsonValue> {
    let mut diags = Vec::new();

    if let Some(expected_state_hash) = payload.get("state_hash").and_then(JsonValue::as_str) {
        if expected_state_hash != actual_state_hash {
            diags.push(json!({
                "code": "STATE_HASH_MISMATCH",
                "expected": expected_state_hash,
                "actual": actual_state_hash,
                "detail": "복원 후 state_hash 불일치",
            }));
        }
    }

    if let Some(expected_view_hash) = payload.get("view_hash").and_then(JsonValue::as_str) {
        if expected_view_hash != actual_view_hash {
            diags.push(json!({
                "code": "VIEW_HASH_MISMATCH",
                "expected": expected_view_hash,
                "actual": actual_view_hash,
                "detail": "복원 후 view_hash 불일치 (view_meta 변경 가능)",
            }));
        }
    }

    diags
}

fn wasm_error(code: &str, detail: &str, hint: &str) -> JsValue {
    let payload = json!({
        "error": true,
        "code": code,
        "detail": detail,
        "hint": hint,
    });
    JsValue::from_str(&payload.to_string())
}

fn fixed64_from_f64_checked(value: f64) -> Fixed64 {
    let result = Fixed64::from_f64_lossy(value);
    #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
    {
        let roundtrip = result.raw_i64() as f64 / Fixed64::ONE_RAW as f64;
        if (value - roundtrip).abs() > 1e-10 {
            web_sys::console::warn_1(
                &format!("Fixed64 정밀도 손실: {value} -> {roundtrip}").into(),
            );
        }
    }
    result
}

fn build_view_meta(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();

    if let Some(width) = world.get_resource_fixed64(BOGAE_WIDTH_TAG) {
        out.insert("canvas_width".to_string(), json!(fixed64_to_f64(width)));
    }
    if let Some(height) = world.get_resource_fixed64(BOGAE_HEIGHT_TAG) {
        out.insert("canvas_height".to_string(), json!(fixed64_to_f64(height)));
    }

    if let Some(draw_list_value) = world.get_resource_value(BOGAE_DRAWLIST_TAG) {
        if let ResourceValue::List(items) = draw_list_value {
            let draw_items: Vec<JsonValue> = items
                .iter()
                .filter_map(resource_value_to_draw_item)
                .collect();
            out.insert("draw_list".to_string(), JsonValue::Array(draw_items));
            out.insert(
                "draw_list_meta".to_string(),
                json!({
                    "deprecated": true,
                    "use_instead": "view_meta.space2d",
                }),
            );
        }
    }

    if let Some(space2d_json) = derive_space2d_from_bogae(world) {
        if let Ok(space2d) = serde_json::from_str::<JsonValue>(&space2d_json) {
            out.insert("space2d".to_string(), space2d);
        }
    }
    if let Some(graph_json) = derive_graph_from_points(world) {
        if let Ok(graph) = serde_json::from_str::<JsonValue>(&graph_json) {
            out.insert("graph".to_string(), graph);
        }
    }

    let mut graph_hints = Vec::new();
    for (tag, id, label) in GRAPH_POINTS_TAGS.iter() {
        let Some(value) = world.get_resource_value(tag) else {
            continue;
        };
        let Some(points) = points_from_resource_value(&value) else {
            continue;
        };
        if points.is_empty() {
            continue;
        }
        graph_hints.push(json!({
            "series_id": *id,
            "source": *tag,
            "y_label": *label,
            "overlay": true,
        }));
    }
    out.insert("graph_hints".to_string(), JsonValue::Array(graph_hints));

    JsonValue::Object(out)
}

fn collect_streams(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();
    let sidecars = collect_stream_sidecars(world);
    for (tag, value) in world.resource_value_entries() {
        let meta = sidecars.get(&tag).copied();
        if !is_stream_resource(&tag, &value, meta.as_ref()) {
            continue;
        }
        let Some(stream) = extract_ring_buffer(&value, meta.as_ref()) else {
            continue;
        };
        out.insert(tag, stream);
    }
    JsonValue::Object(out)
}

#[derive(Clone, Copy, Default)]
struct StreamMeta {
    capacity: Option<u64>,
    head: Option<u64>,
    len: Option<u64>,
}

impl StreamMeta {
    fn has_any(self) -> bool {
        self.capacity.is_some() || self.head.is_some() || self.len.is_some()
    }

    fn set_from_kind(&mut self, kind: &str, value: Option<u64>) {
        if value.is_none() {
            return;
        }
        match kind {
            "capacity" => self.capacity = value,
            "head" => self.head = value,
            "len" => self.len = value,
            _ => {}
        }
    }
}

fn collect_stream_sidecars(world: &ddonirang_core::platform::NuriWorld) -> HashMap<String, StreamMeta> {
    let mut out: HashMap<String, StreamMeta> = HashMap::new();
    for (tag, value) in world.resource_fixed64_entries() {
        let Some((base, kind)) = parse_stream_sidecar_tag(&tag) else {
            continue;
        };
        let numeric = u64::try_from(value.raw_i64()).ok();
        out.entry(base).or_default().set_from_kind(kind, numeric);
    }
    for (tag, value) in world.resource_value_entries() {
        let Some((base, kind)) = parse_stream_sidecar_tag(&tag) else {
            continue;
        };
        let numeric = resource_value_to_u64(&value);
        out.entry(base).or_default().set_from_kind(kind, numeric);
    }
    out
}

fn parse_stream_sidecar_tag(tag: &str) -> Option<(String, &'static str)> {
    const PATTERNS: [(&str, &str); 16] = [
        ("_stream_capacity", "capacity"),
        (".stream_capacity", "capacity"),
        ("_capacity", "capacity"),
        (".capacity", "capacity"),
        ("_head", "head"),
        (".head", "head"),
        ("_length", "len"),
        (".length", "len"),
        ("_len", "len"),
        (".len", "len"),
        ("_용량", "capacity"),
        (".용량", "capacity"),
        ("_머리", "head"),
        (".머리", "head"),
        ("_길이", "len"),
        (".길이", "len"),
    ];
    let lower = tag.to_ascii_lowercase();
    for (suffix, kind) in PATTERNS {
        let is_match = if suffix.is_ascii() {
            lower.ends_with(suffix)
        } else {
            tag.ends_with(suffix)
        };
        if !is_match {
            continue;
        }
        // "foo_stream_capacity"는 base를 "foo_stream"으로 해석한다.
        let raw_base = if suffix == "_stream_capacity" {
            tag.strip_suffix("_capacity").unwrap_or_default()
        } else if suffix == ".stream_capacity" {
            tag.strip_suffix(".capacity").unwrap_or_default()
        } else {
            tag.strip_suffix(suffix).unwrap_or_default()
        };
        let base = raw_base
            .trim_matches(|c| c == '.' || c == '_' || c == '/')
            .to_string();
        if base.is_empty() {
            continue;
        }
        return Some((base, kind));
    }
    None
}

fn is_stream_resource(tag: &str, value: &ResourceValue, sidecar: Option<&StreamMeta>) -> bool {
    if sidecar.copied().unwrap_or_default().has_any() {
        return matches!(value, ResourceValue::List(_) | ResourceValue::Map(_));
    }
    let lower = tag.to_ascii_lowercase();
    if lower.contains("stream") || tag.contains("흐름") {
        return matches!(value, ResourceValue::List(_) | ResourceValue::Map(_));
    }
    let ResourceValue::Map(entries) = value else {
        return false;
    };
    let mut has_buffer = false;
    let mut has_meta = false;
    for entry in entries.values() {
        let key = resource_key_to_string(&entry.key);
        match key.as_str() {
            "buffer" | "버퍼" => has_buffer = true,
            "capacity" | "stream_capacity" | "용량" | "head" | "머리" | "len" | "length"
            | "길이" => has_meta = true,
            _ => {}
        }
    }
    has_buffer || has_meta
}

fn extract_ring_buffer(value: &ResourceValue, sidecar: Option<&StreamMeta>) -> Option<JsonValue> {
    match value {
        ResourceValue::List(items) => {
            let side = sidecar.copied().unwrap_or_default();
            let mut buffer = items.iter().map(resource_value_to_json).collect::<Vec<_>>();
            let capacity = side.capacity.unwrap_or(items.len() as u64).max(items.len() as u64);
            if buffer.len() < capacity as usize {
                buffer.resize(capacity as usize, JsonValue::Null);
            }
            let len = side.len.unwrap_or(items.len() as u64).min(buffer.len() as u64);
            let head = if buffer.is_empty() {
                0
            } else {
                side.head
                    .unwrap_or_else(|| len.saturating_sub(1))
                    .min(buffer.len().saturating_sub(1) as u64)
            };
            Some(json!({
                "capacity": capacity,
                "head": head,
                "len": len,
                "buffer": JsonValue::Array(buffer),
            }))
        }
        ResourceValue::Map(entries) => {
            let side = sidecar.copied().unwrap_or_default();
            let mut buffer_json: Option<Vec<JsonValue>> = None;
            let mut capacity: Option<u64> = None;
            let mut head: Option<u64> = None;
            let mut len: Option<u64> = None;

            for entry in entries.values() {
                let key = resource_key_to_string(&entry.key);
                match key.as_str() {
                    "buffer" | "버퍼" => {
                        buffer_json = match &entry.value {
                            ResourceValue::List(items) => Some(
                                items.iter().map(resource_value_to_json).collect::<Vec<_>>(),
                            ),
                            other => Some(vec![resource_value_to_json(other)]),
                        };
                    }
                    "capacity" | "stream_capacity" | "용량" => {
                        capacity = resource_value_to_u64(&entry.value);
                    }
                    "head" | "머리" => {
                        head = resource_value_to_u64(&entry.value);
                    }
                    "len" | "length" | "길이" => {
                        len = resource_value_to_u64(&entry.value);
                    }
                    _ => {}
                }
            }
            if capacity.is_none() {
                capacity = side.capacity;
            }
            if head.is_none() {
                head = side.head;
            }
            if len.is_none() {
                len = side.len;
            }

            let mut buffer = buffer_json.unwrap_or_default();
            let cap = capacity.unwrap_or(buffer.len() as u64);
            let cap_clamped = cap.max(buffer.len() as u64);
            if buffer.len() < cap_clamped as usize {
                buffer.resize(cap_clamped as usize, JsonValue::Null);
            }
            let raw_len = len.unwrap_or(buffer.len() as u64);
            let safe_len = raw_len.min(buffer.len() as u64);
            let safe_head = head
                .unwrap_or_else(|| safe_len.saturating_sub(1))
                .min(buffer.len().saturating_sub(1) as u64);

            Some(json!({
                "capacity": cap_clamped,
                "head": if buffer.is_empty() { 0 } else { safe_head },
                "len": safe_len,
                "buffer": buffer,
            }))
        }
        _ => None,
    }
}

fn resource_value_to_u64(value: &ResourceValue) -> Option<u64> {
    match value {
        ResourceValue::Fixed64(v) => u64::try_from(v.raw_i64()).ok(),
        ResourceValue::Unit(v) => u64::try_from(v.value.raw_i64()).ok(),
        ResourceValue::Bool(v) => Some(if *v { 1 } else { 0 }),
        ResourceValue::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stream_sidecar_tag_supports_ascii_and_ko_suffix() {
        assert_eq!(
            parse_stream_sidecar_tag("energy_stream_head"),
            Some(("energy_stream".to_string(), "head"))
        );
        assert_eq!(
            parse_stream_sidecar_tag("energy_stream_capacity"),
            Some(("energy_stream".to_string(), "capacity"))
        );
        assert_eq!(
            parse_stream_sidecar_tag("가격흐름_길이"),
            Some(("가격흐름".to_string(), "len"))
        );
        assert_eq!(parse_stream_sidecar_tag("plain_value"), None);
    }

    #[test]
    fn extract_ring_buffer_list_merges_sidecar_meta() {
        let value = ResourceValue::List(vec![
            ResourceValue::Fixed64(Fixed64::from_i64(10)),
            ResourceValue::Fixed64(Fixed64::from_i64(20)),
        ]);
        let meta = StreamMeta {
            capacity: Some(4),
            head: Some(1),
            len: Some(2),
        };
        let stream = extract_ring_buffer(&value, Some(&meta)).expect("stream json");
        let obj = stream.as_object().expect("object");
        assert_eq!(obj.get("capacity").and_then(JsonValue::as_u64), Some(4));
        assert_eq!(obj.get("head").and_then(JsonValue::as_u64), Some(1));
        assert_eq!(obj.get("len").and_then(JsonValue::as_u64), Some(2));
        let buffer = obj
            .get("buffer")
            .and_then(JsonValue::as_array)
            .expect("buffer array");
        assert_eq!(buffer.len(), 4);
        assert!(buffer[2].is_null());
        assert!(buffer[3].is_null());
    }

    #[test]
    fn state_hash_is_stable_when_only_view_resource_changes() {
        let mut world = DetNuri::new();
        world
            .world_mut()
            .set_resource_fixed64("경제.자본".to_string(), Fixed64::from_i64(1));
        world
            .world_mut()
            .set_resource_fixed64(BOGAE_WIDTH_TAG.to_string(), Fixed64::from_i64(640));

        let view_prefixes = DEFAULT_VIEW_PREFIXES
            .iter()
            .map(|prefix| (*prefix).to_string())
            .collect::<Vec<_>>();
        let state_hash_a = current_state_hash(&world, &view_prefixes);
        let view_hash_a = hash_json_string(&build_view_meta(world.world()));

        world
            .world_mut()
            .set_resource_fixed64(BOGAE_WIDTH_TAG.to_string(), Fixed64::from_i64(800));
        let state_hash_b = current_state_hash(&world, &view_prefixes);
        let view_hash_b = hash_json_string(&build_view_meta(world.world()));

        assert_eq!(state_hash_a, state_hash_b);
        assert_ne!(view_hash_a, view_hash_b);
    }

    #[test]
    fn restore_hash_diags_detect_mismatch_codes() {
        let payload = json!({
            "state_hash": "blake3:expected-state",
            "view_hash": "blake3:expected-view",
        });
        let diags = restore_hash_diags(&payload, "blake3:actual-state", "blake3:actual-view");
        assert_eq!(diags.len(), 2);
        let codes = diags
            .iter()
            .filter_map(|diag| diag.get("code").and_then(JsonValue::as_str))
            .collect::<Vec<_>>();
        assert!(codes.contains(&"STATE_HASH_MISMATCH"));
        assert!(codes.contains(&"VIEW_HASH_MISMATCH"));
    }

}

fn serialize_param_overrides(params: &BTreeMap<String, Value>) -> JsonValue {
    let mut out = Map::new();
    for (key, value) in params {
        let raw = match value {
            Value::Fixed64(v) => json!(fixed64_to_f64(*v)),
            Value::Bool(v) => JsonValue::Bool(*v),
            Value::String(v) => JsonValue::String(v.clone()),
            _ => continue,
        };
        out.insert(key.clone(), raw);
    }
    JsonValue::Object(out)
}

fn parse_param_overrides(payload: &JsonValue) -> Result<BTreeMap<String, Value>, String> {
    let mut out = BTreeMap::new();
    let JsonValue::Object(obj) = payload else {
        return Ok(out);
    };
    for (key, value) in obj {
        let parsed = match value {
            JsonValue::Bool(v) => Value::Bool(*v),
            JsonValue::String(v) => Value::String(v.clone()),
            JsonValue::Number(_) => {
                let n = value
                    .as_f64()
                    .ok_or_else(|| format!("param_overrides[{key}] 숫자 파싱 실패"))?;
                if !n.is_finite() {
                    return Err(format!("param_overrides[{key}] 유한 숫자가 아닙니다"));
                }
                Value::Fixed64(fixed64_from_f64_checked(n))
            }
            _ => continue,
        };
        out.insert(key.clone(), parsed);
    }
    Ok(out)
}

fn serialize_world_resources_snapshot(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();

    let mut json_map = Map::new();
    for (tag, value) in world.resource_json_entries() {
        json_map.insert(tag, JsonValue::String(value));
    }
    out.insert("json".to_string(), JsonValue::Object(json_map));

    let mut fixed_map = Map::new();
    for (tag, value) in world.resource_fixed64_entries() {
        fixed_map.insert(tag, JsonValue::String(value.to_string()));
    }
    out.insert("fixed64".to_string(), JsonValue::Object(fixed_map));

    let mut handle_map = Map::new();
    for (tag, handle) in world.resource_handle_entries() {
        handle_map.insert(tag, JsonValue::String(handle_to_string(handle)));
    }
    out.insert("handle".to_string(), JsonValue::Object(handle_map));

    let mut value_det = Map::new();
    for (tag, value) in world.resource_value_entries() {
        value_det.insert(tag, resource_value_to_detjson(&value));
    }
    out.insert("value_det".to_string(), JsonValue::Object(value_det));

    JsonValue::Object(out)
}

fn restore_world_resources(
    world: &mut ddonirang_core::platform::NuriWorld,
    resources: &JsonValue,
) -> Result<(), String> {
    let JsonValue::Object(root) = resources else {
        return Ok(());
    };

    if let Some(JsonValue::Object(json_map)) = root.get("json") {
        for (tag, value) in json_map {
            if let Some(raw) = value.as_str() {
                world.set_resource_json(tag.clone(), raw.to_string());
            }
        }
    }

    if let Some(JsonValue::Object(fixed_map)) = root.get("fixed64") {
        for (tag, value) in fixed_map {
            let parsed = json_to_fixed64(value)
                .ok_or_else(|| format!("fixed64[{tag}] 파싱 실패"))?;
            world.set_resource_fixed64(tag.clone(), parsed);
        }
    }

    if let Some(JsonValue::Object(handle_map)) = root.get("handle") {
        for (tag, value) in handle_map {
            let raw = match value {
                JsonValue::String(s) => parse_handle_string(s)?,
                JsonValue::Number(n) => n
                    .as_u64()
                    .ok_or_else(|| format!("handle[{tag}] 파싱 실패"))?,
                _ => return Err(format!("handle[{tag}] 형식 오류")),
            };
            world.set_resource_handle(tag.clone(), ResourceHandle::from_raw(raw));
        }
    }

    if let Some(JsonValue::Object(value_map)) = root.get("value_det") {
        for (tag, value) in value_map {
            let parsed = resource_value_from_detjson(value)?;
            world.set_resource_value(tag.clone(), parsed);
        }
    }

    Ok(())
}

fn json_to_fixed64(value: &JsonValue) -> Option<Fixed64> {
    match value {
        JsonValue::Number(n) => n.as_f64().map(fixed64_from_f64_checked),
        JsonValue::String(s) => s.parse::<f64>().ok().map(fixed64_from_f64_checked),
        _ => None,
    }
}

fn parse_handle_string(text: &str) -> Result<u64, String> {
    let trimmed = text.trim();
    let without_handle = trimmed.strip_prefix("handle:").unwrap_or(trimmed);
    let hex = without_handle.strip_prefix("자원#").unwrap_or(without_handle);
    u64::from_str_radix(hex, 16).map_err(|_| format!("handle 파싱 실패: {text}"))
}

fn resource_value_to_detjson(value: &ResourceValue) -> JsonValue {
    match value {
        ResourceValue::None => json!({ "type": "none" }),
        ResourceValue::Bool(v) => json!({ "type": "bool", "value": v }),
        ResourceValue::Fixed64(v) => json!({ "type": "fixed64", "value": fixed64_to_f64(*v) }),
        ResourceValue::Unit(v) => json!({ "type": "fixed64", "value": fixed64_to_f64(v.value) }),
        ResourceValue::String(v) => json!({ "type": "string", "value": v }),
        ResourceValue::ResourceHandle(v) => json!({ "type": "handle", "value": handle_to_string(*v) }),
        ResourceValue::List(items) => json!({
            "type": "list",
            "items": items.iter().map(resource_value_to_detjson).collect::<Vec<_>>(),
        }),
        ResourceValue::Set(items) => json!({
            "type": "set",
            "items": items.values().map(resource_value_to_detjson).collect::<Vec<_>>(),
        }),
        ResourceValue::Map(entries) => {
            let rows: Vec<JsonValue> = entries
                .values()
                .map(|entry| {
                    json!({
                        "key": resource_value_to_detjson(&entry.key),
                        "value": resource_value_to_detjson(&entry.value),
                    })
                })
                .collect();
            json!({ "type": "map", "entries": rows })
        }
    }
}

fn resource_value_from_detjson(payload: &JsonValue) -> Result<ResourceValue, String> {
    let JsonValue::Object(obj) = payload else {
        return Err("value_det 형식 오류(object 아님)".to_string());
    };
    let kind = obj
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    match kind {
        "none" => Ok(ResourceValue::None),
        "bool" => Ok(ResourceValue::Bool(
            obj.get("value").and_then(|v| v.as_bool()).unwrap_or(false),
        )),
        "fixed64" => {
            let value = obj
                .get("value")
                .and_then(json_to_fixed64)
                .ok_or_else(|| "fixed64 value 파싱 실패".to_string())?;
            Ok(ResourceValue::Fixed64(value))
        }
        "string" => Ok(ResourceValue::String(
            obj.get("value")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        )),
        "handle" => {
            let raw = obj
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "handle value 누락".to_string())
                .and_then(parse_handle_string)?;
            Ok(ResourceValue::ResourceHandle(ResourceHandle::from_raw(raw)))
        }
        "list" => {
            let items = obj
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(resource_value_from_detjson(&item)?);
            }
            Ok(ResourceValue::List(out))
        }
        "set" => {
            let items = obj
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(resource_value_from_detjson(&item)?);
            }
            Ok(ResourceValue::set_from_values(out))
        }
        "map" => {
            let rows = obj
                .get("entries")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let mut entries = Vec::with_capacity(rows.len());
            for row in rows {
                let JsonValue::Object(row_obj) = row else {
                    return Err("map entry 형식 오류".to_string());
                };
                let key = row_obj
                    .get("key")
                    .ok_or_else(|| "map entry key 누락".to_string())
                    .and_then(resource_value_from_detjson)?;
                let value = row_obj
                    .get("value")
                    .ok_or_else(|| "map entry value 누락".to_string())
                    .and_then(resource_value_from_detjson)?;
                entries.push(ResourceMapEntry { key, value });
            }
            Ok(ResourceValue::map_from_entries(entries))
        }
        _ => Err(format!("지원하지 않는 value_det type: {kind}")),
    }
}

fn json_to_u64(value: &JsonValue) -> Option<u64> {
    match value {
        JsonValue::Number(n) => n.as_u64(),
        JsonValue::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}

fn json_to_i32(value: &JsonValue) -> Option<i32> {
    match value {
        JsonValue::Number(n) => n.as_i64().and_then(|v| i32::try_from(v).ok()),
        JsonValue::String(s) => s.parse::<i32>().ok(),
        _ => None,
    }
}

fn restore_input_state(vm: &mut DdnWasmVm, payload: &JsonValue) {
    let JsonValue::Object(obj) = payload else {
        return;
    };
    vm.input_keys_pressed = obj
        .get("keys_pressed")
        .and_then(json_to_u64)
        .unwrap_or(0);
    vm.input_last_key_name = obj
        .get("last_key_name")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    vm.input_pointer_x_i32 = obj
        .get("pointer_x_i32")
        .and_then(json_to_i32)
        .unwrap_or(0);
    vm.input_pointer_y_i32 = obj
        .get("pointer_y_i32")
        .and_then(json_to_i32)
        .unwrap_or(0);
    vm.rng_seed = obj
        .get("rng_base_seed")
        .and_then(json_to_u64)
        .or_else(|| obj.get("rng_seed").and_then(json_to_u64))
        .unwrap_or(0);

    vm.input_dt = obj
        .get("dt")
        .and_then(json_to_fixed64)
        .unwrap_or_else(|| Fixed64::from_i64(1));
}

fn serialize_world_resources(world: &ddonirang_core::platform::NuriWorld) -> JsonValue {
    let mut out = Map::new();

    let mut json_map = Map::new();
    for (tag, json) in world.resource_json_entries() {
        json_map.insert(tag, JsonValue::String(json));
    }
    out.insert("json".to_string(), JsonValue::Object(json_map));

    let mut fixed_map = Map::new();
    for (tag, value) in world.resource_fixed64_entries() {
        fixed_map.insert(tag, JsonValue::String(value.to_string()));
    }
    out.insert("fixed64".to_string(), JsonValue::Object(fixed_map));

    let mut handle_map = Map::new();
    for (tag, handle) in world.resource_handle_entries() {
        handle_map.insert(tag, JsonValue::String(handle_to_string(handle)));
    }
    out.insert("handle".to_string(), JsonValue::Object(handle_map));

    let mut value_map = Map::new();
    for (tag, value) in world.resource_value_entries() {
        value_map.insert(tag, JsonValue::String(value.canon_key()));
    }
    out.insert("value".to_string(), JsonValue::Object(value_map));

    JsonValue::Object(out)
}

fn serialize_input_snapshot(input: &InputSnapshot, rng_base_seed: Option<u64>) -> JsonValue {
    let mut out = Map::new();
    out.insert("keys_pressed".to_string(), json!(input.keys_pressed));
    out.insert("last_key_name".to_string(), json!(input.last_key_name));
    out.insert("pointer_x_i32".to_string(), json!(input.pointer_x_i32));
    out.insert("pointer_y_i32".to_string(), json!(input.pointer_y_i32));
    out.insert("dt".to_string(), json!(input.dt.to_string()));
    out.insert("rng_seed".to_string(), json!(input.rng_seed));
    if let Some(base_seed) = rng_base_seed {
        out.insert("rng_base_seed".to_string(), json!(base_seed));
    }
    JsonValue::Object(out)
}

fn serialize_input_state(vm: &DdnWasmVm) -> JsonValue {
    json!({
        "keys_pressed": vm.input_keys_pressed,
        "last_key_name": vm.input_last_key_name,
        "pointer_x_i32": vm.input_pointer_x_i32,
        "pointer_y_i32": vm.input_pointer_y_i32,
        "dt": vm.input_dt.to_string(),
        "rng_seed": vm.rng_seed,
        "rng_base_seed": vm.rng_seed,
    })
}

fn serialize_patch(patch: &Patch) -> JsonValue {
    let mut items = Vec::new();
    for op in &patch.ops {
        match op {
            PatchOp::SetResourceJson { tag, json } => {
                items.push(json!({ "op": "set_resource_json", "tag": tag, "value": json }));
            }
            PatchOp::SetResourceFixed64 { tag, value } => {
                items.push(json!({ "op": "set_resource_fixed64", "tag": tag, "value": value.to_string() }));
            }
            PatchOp::SetResourceHandle { tag, handle } => {
                items.push(json!({ "op": "set_resource_handle", "tag": tag, "value": handle_to_string(*handle) }));
            }
            PatchOp::SetResourceValue { tag, value } => {
                items.push(json!({ "op": "set_resource_value", "tag": tag, "value": value.canon_key() }));
            }
            PatchOp::DivAssignResourceFixed64 { tag, rhs, .. } => {
                items.push(json!({ "op": "div_assign_resource_fixed64", "tag": tag, "value": rhs.to_string() }));
            }
            PatchOp::SetComponentJson { entity, tag, json } => {
                items.push(json!({ "op": "set_component_json", "entity": entity.0, "tag": tag.0, "value": json }));
            }
            PatchOp::RemoveComponent { entity, tag } => {
                items.push(json!({ "op": "remove_component", "entity": entity.0, "tag": tag.0 }));
            }
            PatchOp::EmitSignal { signal, targets } => {
                items.push(json!({ "op": "emit_signal", "signal": signal.name(), "targets": targets }));
            }
            PatchOp::GuardViolation { entity, rule_id } => {
                items.push(json!({ "op": "guard_violation", "entity": entity.0, "rule_id": rule_id }));
            }
        }
    }
    JsonValue::Array(items)
}

fn collect_columns_and_row(
    world: &ddonirang_core::platform::NuriWorld,
) -> (Vec<JsonValue>, Vec<JsonValue>) {
    let mut table: BTreeMap<String, (String, JsonValue)> = BTreeMap::new();

    for (tag, value) in world.resource_fixed64_entries() {
        table.insert(
            tag,
            ("num".to_string(), json!(fixed64_to_f64(value))),
        );
    }

    for (tag, value) in world.resource_json_entries() {
        if let Some((dtype, scalar)) = json_text_to_scalar(&value) {
            table.insert(tag, (dtype, scalar));
        }
    }

    for (tag, value) in world.resource_value_entries() {
        if let Some((dtype, scalar)) = resource_value_to_scalar(&value) {
            table.insert(tag, (dtype, scalar));
        }
    }

    let mut columns = Vec::with_capacity(table.len());
    let mut row = Vec::with_capacity(table.len());
    for (key, (dtype, value)) in table {
        columns.push(json!({
            "key": key,
            "dtype": dtype,
            "role": "state",
        }));
        row.push(value);
    }
    (columns, row)
}

fn classify_observation_role(key: &str) -> &'static str {
    if key == "__tick__"
        || key == "tick_id"
        || key == "tick"
        || key == "state_hash"
        || key == "view_hash"
        || key == "frame_id"
        || key == "tick_time_ms"
    {
        return "파생";
    }
    if key.starts_with("보개_") || key.starts_with("__view_") {
        return "뷰";
    }
    "상태"
}

fn build_observation_manifest(channels: &[JsonValue]) -> JsonValue {
    let mut nodes = Vec::new();
    let mut keys = Vec::new();

    for entry in channels {
        let JsonValue::Object(obj) = entry else {
            continue;
        };
        let Some(key) = obj.get("key").and_then(|v| v.as_str()) else {
            continue;
        };
        if key.trim().is_empty() {
            continue;
        }
        keys.push(key.to_string());
        let dtype = obj
            .get("dtype")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let role = obj
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| classify_observation_role(key));
        nodes.push(json!({
            "name": key,
            "dtype": dtype,
            "role": role,
        }));
    }

    let x_channel = if keys.iter().any(|key| key == "프레임수") {
        "프레임수".to_string()
    } else if keys.iter().any(|key| key == "t") {
        "t".to_string()
    } else if keys.iter().any(|key| key == "tick_id") {
        "tick_id".to_string()
    } else {
        keys.first().cloned().unwrap_or_default()
    };
    let y_channels: Vec<String> = keys
        .iter()
        .filter(|key| *key != &x_channel)
        .take(2)
        .cloned()
        .collect();

    json!({
        "schema": OBSERVATION_MANIFEST_SCHEMA,
        "version": OBSERVATION_MANIFEST_VERSION,
        "nodes": nodes,
        "params": [],
        "traces": [],
        "diagnostics": [],
        "default_pivot": {
            "x_channel": x_channel,
            "y_channels": y_channels,
        }
    })
}

fn json_text_to_scalar(text: &str) -> Option<(String, JsonValue)> {
    let trimmed = text.trim();
    if trimmed == "참" {
        return Some(("bool".to_string(), JsonValue::Bool(true)));
    }
    if trimmed == "거짓" {
        return Some(("bool".to_string(), JsonValue::Bool(false)));
    }
    if let Ok(parsed) = serde_json::from_str::<JsonValue>(trimmed) {
        return match parsed {
            JsonValue::Bool(v) => Some(("bool".to_string(), JsonValue::Bool(v))),
            JsonValue::Number(v) => Some(("num".to_string(), JsonValue::Number(v))),
            JsonValue::String(v) => Some(("str".to_string(), JsonValue::String(v))),
            _ => None,
        };
    }
    if let Ok(number) = trimmed.parse::<f64>() {
        if number.is_finite() {
            return Some(("num".to_string(), json!(number)));
        }
    }
    Some(("str".to_string(), JsonValue::String(text.to_string())))
}

fn resource_value_to_scalar(value: &ResourceValue) -> Option<(String, JsonValue)> {
    match value {
        ResourceValue::Bool(v) => Some(("bool".to_string(), JsonValue::Bool(*v))),
        ResourceValue::Fixed64(v) => Some(("num".to_string(), json!(fixed64_to_f64(*v)))),
        ResourceValue::Unit(v) => Some(("num".to_string(), json!(fixed64_to_f64(v.value)))),
        ResourceValue::String(v) => Some(("str".to_string(), JsonValue::String(v.clone()))),
        _ => None,
    }
}

fn js_scalar_to_runtime_value(value: &JsValue) -> Option<Value> {
    if let Some(v) = value.as_f64() {
        if v.is_finite() {
            return Some(Value::Fixed64(fixed64_from_f64_checked(v)));
        }
        return None;
    }
    if let Some(v) = value.as_bool() {
        return Some(Value::Bool(v));
    }
    value.as_string().map(Value::String)
}

fn apply_param_value(
    world: &mut ddonirang_core::platform::NuriWorld,
    key: &str,
    value: &Value,
) -> Result<(), String> {
    match value {
        Value::Fixed64(v) => {
            world.set_resource_fixed64(key.to_string(), *v);
            Ok(())
        }
        Value::Bool(v) => {
            world.set_resource_json(
                key.to_string(),
                if *v { "참" } else { "거짓" }.to_string(),
            );
            Ok(())
        }
        Value::String(v) => {
            world.set_resource_json(key.to_string(), v.clone());
            Ok(())
        }
        _ => Err("set_param: 수/참거짓/글 스칼라만 허용됩니다".to_string()),
    }
}

fn handle_to_string(handle: ResourceHandle) -> String {
    format!("handle:{}", handle.to_hex())
}

fn seed_bogae_defaults(defaults: &mut HashMap<String, Value>) {
    defaults.insert(BOGAE_DRAWLIST_TAG.to_string(), Value::List(Vec::new()));
    defaults.insert(
        BOGAE_WIDTH_TAG.to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert(
        BOGAE_HEIGHT_TAG.to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert(
        "프레임수".to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert(
        "__wasm_start_once".to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert(
        "회전_x".to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
    defaults.insert(
        "회전_y".to_string(),
        Value::Fixed64(Fixed64::from_i64(0)),
    );
}

fn derive_space2d_from_bogae(world: &ddonirang_core::platform::NuriWorld) -> Option<String> {
    let list = world.get_resource_value(BOGAE_DRAWLIST_TAG)?;
    let ResourceValue::List(items) = list else {
        return None;
    };
    let mut draw_items = Vec::new();
    for item in &items {
        if let Some(obj) = resource_value_to_draw_item(item) {
            draw_items.push(obj);
        }
    }
    if draw_items.is_empty() {
        return None;
    }
    let mut space2d = Map::new();
    space2d.insert(
        "schema".to_string(),
        JsonValue::String(SPACE2D_SCHEMA.to_string()),
    );
    space2d.insert("drawlist".to_string(), JsonValue::Array(draw_items));
    if let (Some(w), Some(h)) = (
        world.get_resource_fixed64(BOGAE_WIDTH_TAG),
        world.get_resource_fixed64(BOGAE_HEIGHT_TAG),
    ) {
        let w = fixed64_to_f64(w);
        let h = fixed64_to_f64(h);
        if w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 {
            space2d.insert(
                "camera".to_string(),
                json!({
                    "x_min": 0.0,
                    "x_max": w,
                    "y_min": 0.0,
                    "y_max": h,
                }),
            );
        }
    }
    Some(JsonValue::Object(space2d).to_string())
}

fn derive_graph_from_points(world: &ddonirang_core::platform::NuriWorld) -> Option<String> {
    let mut series = Vec::new();
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    for (tag, id, label) in GRAPH_POINTS_TAGS.iter() {
        let Some(value) = world.get_resource_value(tag) else {
            continue;
        };
        let Some(points) = points_from_resource_value(&value) else {
            continue;
        };
        if points.is_empty() {
            continue;
        }
        for (x, y) in &points {
            if x.is_finite() {
                x_min = x_min.min(*x);
                x_max = x_max.max(*x);
            }
            if y.is_finite() {
                y_min = y_min.min(*y);
                y_max = y_max.max(*y);
            }
        }
        let point_list: Vec<JsonValue> = points
            .into_iter()
            .map(|(x, y)| json!([x, y]))
            .collect();
        series.push(json!({
            "id": *id,
            "label": *label,
            "points": point_list,
        }));
    }

    if series.is_empty() {
        return None;
    }

    let mut graph = Map::new();
    graph.insert("schema".to_string(), JsonValue::String(GRAPH_SCHEMA.to_string()));
    graph.insert("series".to_string(), JsonValue::Array(series));
    if x_min.is_finite() && x_max.is_finite() && y_min.is_finite() && y_max.is_finite() {
        graph.insert(
            "axis".to_string(),
            json!({
                "x_min": x_min,
                "x_max": x_max,
                "y_min": y_min,
                "y_max": y_max,
            }),
        );
    }
    graph.insert(
        "meta".to_string(),
        json!({
            "graph_kind": "curve",
            "update": "replace",
        }),
    );
    Some(JsonValue::Object(graph).to_string())
}

fn points_from_resource_value(value: &ResourceValue) -> Option<Vec<(f64, f64)>> {
    let ResourceValue::List(items) = value else {
        return None;
    };
    let mut out = Vec::new();
    for item in items {
        if let Some(point) = resource_value_to_point(item) {
            out.push(point);
        }
    }
    Some(out)
}

fn resource_value_to_point(value: &ResourceValue) -> Option<(f64, f64)> {
    match value {
        ResourceValue::Map(entries) => {
            let mut x = None;
            let mut y = None;
            for entry in entries.values() {
                let key = resource_key_to_string(&entry.key);
                if key == "x" {
                    x = resource_value_to_number(&entry.value);
                } else if key == "y" {
                    y = resource_value_to_number(&entry.value);
                }
            }
            match (x, y) {
                (Some(x), Some(y)) => Some((x, y)),
                _ => None,
            }
        }
        ResourceValue::List(items) => {
            if items.len() < 2 {
                return None;
            }
            let x = resource_value_to_number(&items[0])?;
            let y = resource_value_to_number(&items[1])?;
            Some((x, y))
        }
        _ => None,
    }
}

fn resource_value_to_number(value: &ResourceValue) -> Option<f64> {
    match value {
        ResourceValue::Fixed64(v) => Some(fixed64_to_f64(*v)),
        ResourceValue::Unit(v) => Some(fixed64_to_f64(v.value)),
        ResourceValue::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
        ResourceValue::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn resource_value_to_draw_item(value: &ResourceValue) -> Option<JsonValue> {
    let ResourceValue::Map(entries) = value else {
        return None;
    };
    let mut obj = Map::new();
    let mut kind_raw: Option<String> = None;
    for entry in entries.values() {
        let key = resource_key_to_string(&entry.key);
        let json_value = resource_value_to_json(&entry.value);
        match key.as_str() {
            "결" | "kind" | "shape" | "도형" | "형태" => {
                kind_raw = Some(resource_value_to_string(&entry.value));
            }
            "채움색" | "채움" => {
                obj.insert("fill".to_string(), json_value);
            }
            "색" | "선색" => {
                obj.insert("color".to_string(), json_value);
            }
            "굵기" => {
                obj.insert("width".to_string(), json_value);
            }
            "크기" => {
                obj.insert("size".to_string(), json_value);
            }
            "글" | "내용" => {
                obj.insert("text".to_string(), json_value);
            }
            _ => {
                obj.insert(key, json_value);
            }
        }
    }
    if let Some(raw) = kind_raw {
        obj.insert("kind".to_string(), JsonValue::String(map_bogae_kind(&raw)));
    }
    Some(JsonValue::Object(obj))
}

fn resource_key_to_string(value: &ResourceValue) -> String {
    match value {
        ResourceValue::String(s) => s.clone(),
        _ => value.canon_key(),
    }
}

fn resource_value_to_string(value: &ResourceValue) -> String {
    match value {
        ResourceValue::String(s) => s.clone(),
        _ => value.canon_key(),
    }
}

fn resource_value_to_json(value: &ResourceValue) -> JsonValue {
    match value {
        ResourceValue::None => JsonValue::Null,
        ResourceValue::Bool(v) => JsonValue::Bool(*v),
        ResourceValue::Fixed64(v) => json!(fixed64_to_f64(*v)),
        ResourceValue::Unit(v) => json!(fixed64_to_f64(v.value)),
        ResourceValue::String(s) => JsonValue::String(s.clone()),
        ResourceValue::ResourceHandle(handle) => JsonValue::String(handle_to_string(*handle)),
        ResourceValue::List(items) => JsonValue::Array(items.iter().map(resource_value_to_json).collect()),
        ResourceValue::Set(items) => {
            JsonValue::Array(items.values().map(resource_value_to_json).collect())
        }
        ResourceValue::Map(entries) => {
            let mut obj = Map::new();
            for entry in entries.values() {
                let key = resource_key_to_string(&entry.key);
                obj.insert(key, resource_value_to_json(&entry.value));
            }
            JsonValue::Object(obj)
        }
    }
}

fn map_bogae_kind(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("rect") {
        return "rect".to_string();
    }
    if lower.contains("text") {
        return "text".to_string();
    }
    if lower.contains("circle") {
        return "circle".to_string();
    }
    if lower.contains("line") {
        return "line".to_string();
    }
    if lower.contains("point") {
        return "point".to_string();
    }
    if lower.contains("poly") {
        return "polyline".to_string();
    }
    if lower.contains("arrow") {
        return "arrow".to_string();
    }
    if lower.contains("sprite") {
        return "rect".to_string();
    }
    if lower.contains("사각") || lower.contains("네모") {
        return "rect".to_string();
    }
    if lower.contains("동그라미") || lower == "원" || lower.contains("원형") {
        return "circle".to_string();
    }
    if lower.contains("선분") || lower == "선" {
        return "line".to_string();
    }
    if lower == "점" || lower.contains("점형") {
        return "point".to_string();
    }
    if lower.contains("화살") || lower.contains("살표") {
        return "arrow".to_string();
    }
    if lower == "글" || lower.contains("글자") || lower.contains("문자") {
        return "text".to_string();
    }
    if lower.contains("다각") {
        return "polygon".to_string();
    }
    raw.to_string()
}

fn fixed64_to_f64(value: Fixed64) -> f64 {
    value.raw_i64() as f64 / Fixed64::ONE_RAW as f64
}
