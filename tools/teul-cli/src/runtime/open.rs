use std::collections::{BTreeMap, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, path::Component};

use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};

use crate::core::fixed64::Fixed64;
use crate::core::unit::UnitDim;
use crate::core::value::{PackValue, Quantity, Value};
use crate::lang::span::Span;
use crate::runtime::error::RuntimeError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenMode {
    Deny,
    Record,
    Replay,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct OpenKey {
    open_kind: String,
    site_id: String,
    key: String,
}

#[derive(Clone, Debug)]
struct OpenValue {
    value: JsonValue,
    detjson_hash: String,
}

#[derive(Clone, Debug)]
pub enum OpenLogError {
    Missing { message: String },
    Parse { message: String },
}

impl OpenLogError {
    pub fn code(&self) -> &'static str {
        match self {
            OpenLogError::Missing { .. } => "E_OPEN_LOG_MISSING",
            OpenLogError::Parse { .. } => "E_OPEN_LOG_PARSE",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            OpenLogError::Missing { message } => message,
            OpenLogError::Parse { message } => message,
        }
    }
}

pub struct OpenRuntime {
    mode: OpenMode,
    #[allow(dead_code)]
    log_path: Option<PathBuf>,
    record_out: Option<File>,
    replay: BTreeMap<OpenKey, VecDeque<OpenValue>>,
    allow: BTreeMap<String, ()>,
    policy: Option<OpenPolicy>,
}

impl OpenRuntime {
    pub fn deny() -> Self {
        Self {
            mode: OpenMode::Deny,
            log_path: None,
            record_out: None,
            replay: BTreeMap::new(),
            allow: BTreeMap::new(),
            policy: None,
        }
    }

    pub fn new(
        mode: OpenMode,
        log_path: Option<PathBuf>,
        allow: Vec<String>,
        policy: Option<OpenPolicy>,
    ) -> Result<Self, OpenLogError> {
        let allow = allow
            .into_iter()
            .map(|item| (item, ()))
            .collect::<BTreeMap<_, _>>();
        if mode == OpenMode::Deny {
            return Ok(Self {
                mode,
                log_path: None,
                record_out: None,
                replay: BTreeMap::new(),
                allow,
                policy,
            });
        }
        let path = log_path.ok_or_else(|| OpenLogError::Parse {
            message: "open.log.jsonl 경로가 필요합니다.".to_string(),
        })?;
        match mode {
            OpenMode::Record => {
                let file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&path)
                    .map_err(|e| OpenLogError::Parse {
                        message: format!("open.log 생성 실패: {} ({})", path.display(), e),
                    })?;
                Ok(Self {
                    mode,
                    log_path: Some(path),
                    record_out: Some(file),
                    replay: BTreeMap::new(),
                    allow,
                    policy,
                })
            }
            OpenMode::Replay => {
                let replay = load_replay_log(&path)?;
                Ok(Self {
                    mode,
                    log_path: Some(path),
                    record_out: None,
                    replay,
                    allow,
                    policy,
                })
            }
            OpenMode::Deny => Ok(Self::deny()),
        }
    }

    #[allow(dead_code)]
    pub fn mode(&self) -> OpenMode {
        self.mode
    }

    pub fn open_clock(&mut self, site_id: &str, span: Span) -> Result<Value, RuntimeError> {
        self.ensure_policy_allows("clock", span)?;
        self.warn_not_declared("clock", site_id);
        match self.mode {
            OpenMode::Deny => Err(RuntimeError::OpenDenied {
                open_kind: "clock".to_string(),
                span,
            }),
            OpenMode::Record => {
                let unix_sec = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| RuntimeError::OpenIo {
                        message: format!("시각 측정 실패: {}", e),
                        span,
                    })?
                    .as_secs() as i64;
                let value_json = serde_json::json!({
                    "schema": "open.clock.v1",
                    "unix_sec": unix_sec,
                });
                self.append_event("clock", site_id, "now", &value_json, span)?;
                Ok(Value::Pack(clock_pack(unix_sec)))
            }
            OpenMode::Replay => {
                let value = self.take_replay("clock", site_id, "now", span)?;
                let unix_sec = parse_unix_sec(&value.value, span)?;
                Ok(Value::Pack(clock_pack(unix_sec)))
            }
        }
    }

    pub fn open_file_read(
        &mut self,
        site_id: &str,
        path: &str,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        self.ensure_policy_allows("file_read", span)?;
        let (key, fs_path) = normalize_open_path(path);
        self.warn_not_declared("file_read", site_id);
        match self.mode {
            OpenMode::Deny => Err(RuntimeError::OpenDenied {
                open_kind: "file_read".to_string(),
                span,
            }),
            OpenMode::Record => {
                let text = fs::read_to_string(&fs_path).map_err(|e| RuntimeError::OpenIo {
                    message: format!("파일 읽기 실패: {} ({})", fs_path.display(), e),
                    span,
                })?;
                let value_json = serde_json::json!({
                    "schema": "open.file_read.v1",
                    "text": text,
                });
                self.append_event("file_read", site_id, &key, &value_json, span)?;
                Ok(Value::Str(text))
            }
            OpenMode::Replay => {
                let value = self.take_replay("file_read", site_id, &key, span)?;
                let text = parse_text(&value.value, span)?;
                Ok(Value::Str(text))
            }
        }
    }

    pub fn open_rand(&mut self, site_id: &str, span: Span) -> Result<Value, RuntimeError> {
        self.ensure_policy_allows("rand", span)?;
        self.warn_not_declared("rand", site_id);
        match self.mode {
            OpenMode::Deny => Err(RuntimeError::OpenDenied {
                open_kind: "rand".to_string(),
                span,
            }),
            OpenMode::Record => {
                let nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| RuntimeError::OpenIo {
                        message: format!("난수 시각 측정 실패: {}", e),
                        span,
                    })?
                    .as_nanos() as u64;
                let mut value = (nanos ^ (nanos >> 32)) as i64;
                let modulo = 1_000_000i64;
                value = value % modulo;
                if value < 0 {
                    value += modulo;
                }
                let value_json = serde_json::json!({
                    "schema": "open.rand.v1",
                    "value": value,
                });
                self.append_event("rand", site_id, "rng", &value_json, span)?;
                Ok(Value::Num(Quantity::new(
                    Fixed64::from_int(value),
                    UnitDim::zero(),
                )))
            }
            OpenMode::Replay => {
                let value = self.take_replay("rand", site_id, "rng", span)?;
                let value = parse_rand_value(&value.value, span)?;
                Ok(Value::Num(Quantity::new(
                    Fixed64::from_int(value),
                    UnitDim::zero(),
                )))
            }
        }
    }

    pub fn open_net(
        &mut self,
        site_id: &str,
        url: &str,
        method: &str,
        body: Option<&str>,
        response: Option<&str>,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        self.ensure_policy_allows("net", span)?;
        self.warn_not_declared("net", site_id);
        match self.mode {
            OpenMode::Deny => Err(RuntimeError::OpenDenied {
                open_kind: "net".to_string(),
                span,
            }),
            OpenMode::Record => {
                let Some(response) = response else {
                    return Err(RuntimeError::OpenIo {
                        message: "open.net 기록에는 응답이 필요합니다".to_string(),
                        span,
                    });
                };
                let value_json = build_open_net_value(url, method, body, response);
                self.append_event("net", site_id, url, &value_json, span)?;
                Ok(Value::Str(response.to_string()))
            }
            OpenMode::Replay => {
                let value = self.take_replay("net", site_id, url, span)?;
                let text = parse_open_net_text(&value.value, span)?;
                Ok(Value::Str(text))
            }
        }
    }

    pub fn open_ffi(
        &mut self,
        site_id: &str,
        name: &str,
        args: Option<&[String]>,
        result: Option<&str>,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        self.ensure_policy_allows("ffi", span)?;
        self.warn_not_declared("ffi", site_id);
        let key = build_open_ffi_key(name, args);
        match self.mode {
            OpenMode::Deny => Err(RuntimeError::OpenDenied {
                open_kind: "ffi".to_string(),
                span,
            }),
            OpenMode::Record => {
                let Some(result) = result else {
                    return Err(RuntimeError::OpenIo {
                        message: "open.ffi 기록에는 결과가 필요합니다".to_string(),
                        span,
                    });
                };
                let value_json = build_open_ffi_value(name, args, result);
                self.append_event("ffi", site_id, &key, &value_json, span)?;
                Ok(Value::Str(result.to_string()))
            }
            OpenMode::Replay => {
                let value = self.take_replay("ffi", site_id, &key, span)?;
                let text = parse_open_ffi_result(&value.value, span)?;
                Ok(Value::Str(text))
            }
        }
    }

    pub fn open_gpu(
        &mut self,
        site_id: &str,
        kernel: &str,
        payload: Option<&str>,
        result: Option<&str>,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        self.ensure_policy_allows("gpu", span)?;
        self.warn_not_declared("gpu", site_id);
        match self.mode {
            OpenMode::Deny => Err(RuntimeError::OpenDenied {
                open_kind: "gpu".to_string(),
                span,
            }),
            OpenMode::Record => {
                let Some(result) = result else {
                    return Err(RuntimeError::OpenIo {
                        message: "open.gpu 기록에는 결과가 필요합니다".to_string(),
                        span,
                    });
                };
                let value_json = build_open_gpu_value(kernel, payload, result);
                self.append_event("gpu", site_id, kernel, &value_json, span)?;
                Ok(Value::Str(result.to_string()))
            }
            OpenMode::Replay => {
                let value = self.take_replay("gpu", site_id, kernel, span)?;
                let text = parse_open_gpu_result(&value.value, span)?;
                Ok(Value::Str(text))
            }
        }
    }

    fn append_event(
        &mut self,
        open_kind: &str,
        site_id: &str,
        key: &str,
        value: &JsonValue,
        span: Span,
    ) -> Result<(), RuntimeError> {
        let Some(file) = self.record_out.as_mut() else {
            return Err(RuntimeError::OpenIo {
                message: "open.log 출력 대상이 없습니다".to_string(),
                span,
            });
        };
        let detjson_text = serde_json::to_string(value).map_err(|e| RuntimeError::OpenIo {
            message: format!("open.log value 직렬화 실패: {}", e),
            span,
        })?;
        let detjson_hash = format!("sha256:{}", sha256_hex(detjson_text.as_bytes()));
        let event = serde_json::json!({
            "event_kind": "open",
            "open_kind": open_kind,
            "site_id": site_id,
            "key": key,
            "value": value,
            "detjson_hash": detjson_hash,
        });
        let line = serde_json::to_string(&event).map_err(|e| RuntimeError::OpenIo {
            message: format!("open.log 직렬화 실패: {}", e),
            span,
        })?;
        file.write_all(line.as_bytes()).map_err(|e| RuntimeError::OpenIo {
            message: format!("open.log 기록 실패: {}", e),
            span,
        })?;
        file.write_all(b"\n").map_err(|e| RuntimeError::OpenIo {
            message: format!("open.log 기록 실패: {}", e),
            span,
        })?;
        file.flush().map_err(|e| RuntimeError::OpenIo {
            message: format!("open.log flush 실패: {}", e),
            span,
        })?;
        Ok(())
    }

    fn take_replay(
        &mut self,
        open_kind: &str,
        site_id: &str,
        key: &str,
        span: Span,
    ) -> Result<OpenValue, RuntimeError> {
        let lookup = OpenKey {
            open_kind: open_kind.to_string(),
            site_id: site_id.to_string(),
            key: key.to_string(),
        };
        let Some(queue) = self.replay.get_mut(&lookup) else {
            return Err(RuntimeError::OpenReplayMissing {
                open_kind: open_kind.to_string(),
                site_id: site_id.to_string(),
                key: key.to_string(),
                span,
            });
        };
        let value = queue.pop_front().ok_or(RuntimeError::OpenReplayMissing {
            open_kind: open_kind.to_string(),
            site_id: site_id.to_string(),
            key: key.to_string(),
            span,
        })?;
        let detjson_text = serde_json::to_string(&value.value).map_err(|e| RuntimeError::OpenReplayInvalid {
            message: format!("open.log value 직렬화 실패: {}", e),
            span,
        })?;
        let actual_hash = format!("sha256:{}", sha256_hex(detjson_text.as_bytes()));
        if actual_hash != value.detjson_hash {
            return Err(RuntimeError::OpenLogTamper {
                message: format!(
                    "open.log detjson_hash 불일치: expected={} actual={}",
                    value.detjson_hash, actual_hash
                ),
                span,
            });
        }
        Ok(value)
    }

    fn warn_not_declared(&self, open_kind: &str, site_id: &str) {
        if self.allow.is_empty() {
            return;
        }
        if self.allow.contains_key(open_kind) {
            return;
        }
        eprintln!(
            "W_OPEN_NOT_DECLARED open_kind={} site_id={}",
            open_kind, site_id
        );
    }

    fn ensure_policy_allows(&self, open_kind: &str, span: Span) -> Result<(), RuntimeError> {
        let Some(policy) = &self.policy else {
            return Ok(());
        };
        if policy.allows(open_kind) {
            return Ok(());
        }
        Err(RuntimeError::OpenDenied {
            open_kind: open_kind.to_string(),
            span,
        })
    }
}

#[derive(Clone, Debug)]
pub struct OpenPolicy {
    default_mode: OpenMode,
    allow: BTreeMap<String, ()>,
    deny: BTreeMap<String, ()>,
}

impl OpenPolicy {
    pub fn new(default_mode: OpenMode, allow: Vec<String>, deny: Vec<String>) -> Self {
        Self {
            default_mode,
            allow: allow
                .into_iter()
                .map(|item| (item, ()))
                .collect::<BTreeMap<_, _>>(),
            deny: deny
                .into_iter()
                .map(|item| (item, ()))
                .collect::<BTreeMap<_, _>>(),
        }
    }

    pub fn default_mode(&self) -> OpenMode {
        self.default_mode
    }

    pub fn allows(&self, open_kind: &str) -> bool {
        if self.deny.contains_key(open_kind) {
            return false;
        }
        self.allow.contains_key(open_kind)
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn load_replay_log(path: &Path) -> Result<BTreeMap<OpenKey, VecDeque<OpenValue>>, OpenLogError> {
    let file = File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            OpenLogError::Missing {
                message: format!("open.log 없음: {}", path.display()),
            }
        } else {
            OpenLogError::Parse {
                message: format!("open.log 읽기 실패: {} ({})", path.display(), e),
            }
        }
    })?;
    let reader = BufReader::new(file);
    let mut map: BTreeMap<OpenKey, VecDeque<OpenValue>> = BTreeMap::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| OpenLogError::Parse {
            message: format!("open.log 읽기 실패: {}", e),
        })?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: JsonValue = serde_json::from_str(trimmed)
            .map_err(|e| OpenLogError::Parse {
                message: format!("open.log 파싱 실패 {}:{} ({})", path.display(), idx + 1, e),
            })?;
        let event_kind = value
            .get("event_kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OpenLogError::Parse {
                message: format!("open.log event_kind 누락: {}:{}", path.display(), idx + 1),
            })?;
        if event_kind != "open" {
            continue;
        }
        let open_kind = value
            .get("open_kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OpenLogError::Parse {
                message: format!("open.log open_kind 누락: {}:{}", path.display(), idx + 1),
            })?;
        let site_id = value
            .get("site_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OpenLogError::Parse {
                message: format!("open.log site_id 누락: {}:{}", path.display(), idx + 1),
            })?;
        let key = value
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OpenLogError::Parse {
                message: format!("open.log key 누락: {}:{}", path.display(), idx + 1),
            })?;
        let detjson_hash = value
            .get("detjson_hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OpenLogError::Parse {
                message: format!("open.log detjson_hash 누락: {}:{}", path.display(), idx + 1),
            })?;
        let payload = value
            .get("value")
            .ok_or_else(|| OpenLogError::Parse {
                message: format!("open.log value 누락: {}:{}", path.display(), idx + 1),
            })?;
        let key = if open_kind == "file_read" {
            normalize_open_key(key)
        } else {
            key.to_string()
        };
        let open_key = OpenKey {
            open_kind: open_kind.to_string(),
            site_id: site_id.to_string(),
            key,
        };
        let entry = OpenValue {
            value: payload.clone(),
            detjson_hash: detjson_hash.to_string(),
        };
        map.entry(open_key).or_default().push_back(entry);
    }
    Ok(map)
}

fn clock_pack(unix_sec: i64) -> PackValue {
    let mut fields = BTreeMap::new();
    fields.insert(
        "unix_sec".to_string(),
        Value::Num(Quantity::new(Fixed64::from_int(unix_sec), UnitDim::zero())),
    );
    PackValue { fields }
}

fn parse_unix_sec(value: &JsonValue, span: Span) -> Result<i64, RuntimeError> {
    let Some(obj) = value.as_object() else {
        return Err(RuntimeError::OpenReplayInvalid {
            message: "open.clock value는 객체여야 합니다".to_string(),
            span,
        });
    };
    let schema = obj
        .get("schema")
        .and_then(|v| v.as_str())
        .ok_or(RuntimeError::OpenReplayInvalid {
            message: "open.clock schema 누락".to_string(),
            span,
        })?;
    if !matches!(schema, "open.clock.v1" | "open.clock.v2") {
        return Err(RuntimeError::OpenReplayInvalid {
            message: format!(
                "open.clock schema 불일치: expected=open.clock.v1,open.clock.v2 actual={}",
                schema
            ),
            span,
        });
    }
    let unix_sec = obj.get("unix_sec").and_then(|v| v.as_i64()).ok_or(RuntimeError::OpenReplayInvalid {
        message: "open.clock unix_sec 누락".to_string(),
        span,
    })?;
    Ok(unix_sec)
}

fn parse_text(value: &JsonValue, span: Span) -> Result<String, RuntimeError> {
    let Some(obj) = value.as_object() else {
        return Err(RuntimeError::OpenReplayInvalid {
            message: "open.file_read value는 객체여야 합니다".to_string(),
            span,
        });
    };
    let schema = obj
        .get("schema")
        .and_then(|v| v.as_str())
        .ok_or(RuntimeError::OpenReplayInvalid {
            message: "open.file_read schema 누락".to_string(),
            span,
        })?;
    if !matches!(schema, "open.file_read.v1" | "open.file_read.v2") {
        return Err(RuntimeError::OpenReplayInvalid {
            message: format!(
                "open.file_read schema 불일치: expected=open.file_read.v1,open.file_read.v2 actual={}",
                schema
            ),
            span,
        });
    }
    let text = obj.get("text").and_then(|v| v.as_str()).ok_or(RuntimeError::OpenReplayInvalid {
        message: "open.file_read text 누락".to_string(),
        span,
    })?;
    Ok(text.to_string())
}

fn parse_rand_value(value: &JsonValue, span: Span) -> Result<i64, RuntimeError> {
    let Some(obj) = value.as_object() else {
        return Err(RuntimeError::OpenReplayInvalid {
            message: "open.rand value는 객체여야 합니다".to_string(),
            span,
        });
    };
    let schema = obj
        .get("schema")
        .and_then(|v| v.as_str())
        .ok_or(RuntimeError::OpenReplayInvalid {
            message: "open.rand schema 누락".to_string(),
            span,
        })?;
    if schema != "open.rand.v1" {
        return Err(RuntimeError::OpenReplayInvalid {
            message: format!("open.rand schema 불일치: expected=open.rand.v1 actual={}", schema),
            span,
        });
    }
    let value = obj.get("value").and_then(|v| v.as_i64()).ok_or(RuntimeError::OpenReplayInvalid {
        message: "open.rand value 누락".to_string(),
        span,
    })?;
    Ok(value)
}

fn build_open_net_value(url: &str, method: &str, body: Option<&str>, response: &str) -> JsonValue {
    let mut obj = serde_json::Map::new();
    obj.insert("schema".to_string(), JsonValue::String("open.net.v1".to_string()));
    obj.insert("url".to_string(), JsonValue::String(url.to_string()));
    obj.insert("method".to_string(), JsonValue::String(method.to_string()));
    if let Some(body) = body {
        obj.insert("body".to_string(), JsonValue::String(body.to_string()));
    }
    obj.insert("text".to_string(), JsonValue::String(response.to_string()));
    JsonValue::Object(obj)
}

fn parse_open_net_text(value: &JsonValue, span: Span) -> Result<String, RuntimeError> {
    let Some(obj) = value.as_object() else {
        return Err(RuntimeError::OpenReplayInvalid {
            message: "open.net value는 객체여야 합니다".to_string(),
            span,
        });
    };
    let schema = obj
        .get("schema")
        .and_then(|v| v.as_str())
        .ok_or(RuntimeError::OpenReplayInvalid {
            message: "open.net schema 누락".to_string(),
            span,
        })?;
    if schema != "open.net.v1" {
        return Err(RuntimeError::OpenReplayInvalid {
            message: format!("open.net schema 불일치: expected=open.net.v1 actual={}", schema),
            span,
        });
    }
    let text = obj.get("text").and_then(|v| v.as_str()).ok_or(RuntimeError::OpenReplayInvalid {
        message: "open.net text 누락".to_string(),
        span,
    })?;
    Ok(text.to_string())
}

fn build_open_ffi_key(name: &str, args: Option<&[String]>) -> String {
    match args {
        Some(items) if !items.is_empty() => format!("{}({})", name, items.join(",")),
        _ => name.to_string(),
    }
}

fn build_open_ffi_value(name: &str, args: Option<&[String]>, result: &str) -> JsonValue {
    let mut obj = serde_json::Map::new();
    obj.insert("schema".to_string(), JsonValue::String("open.ffi.v1".to_string()));
    obj.insert("name".to_string(), JsonValue::String(name.to_string()));
    if let Some(items) = args {
        let array = items
            .iter()
            .map(|item| JsonValue::String(item.to_string()))
            .collect();
        obj.insert("args".to_string(), JsonValue::Array(array));
    }
    obj.insert("result".to_string(), JsonValue::String(result.to_string()));
    JsonValue::Object(obj)
}

fn parse_open_ffi_result(value: &JsonValue, span: Span) -> Result<String, RuntimeError> {
    let Some(obj) = value.as_object() else {
        return Err(RuntimeError::OpenReplayInvalid {
            message: "open.ffi value는 객체여야 합니다".to_string(),
            span,
        });
    };
    let schema = obj
        .get("schema")
        .and_then(|v| v.as_str())
        .ok_or(RuntimeError::OpenReplayInvalid {
            message: "open.ffi schema 누락".to_string(),
            span,
        })?;
    if schema != "open.ffi.v1" {
        return Err(RuntimeError::OpenReplayInvalid {
            message: format!("open.ffi schema 불일치: expected=open.ffi.v1 actual={}", schema),
            span,
        });
    }
    let text = obj
        .get("result")
        .and_then(|v| v.as_str())
        .ok_or(RuntimeError::OpenReplayInvalid {
            message: "open.ffi result 누락".to_string(),
            span,
        })?;
    Ok(text.to_string())
}

fn build_open_gpu_value(kernel: &str, payload: Option<&str>, result: &str) -> JsonValue {
    let mut obj = serde_json::Map::new();
    obj.insert("schema".to_string(), JsonValue::String("open.gpu.v1".to_string()));
    obj.insert("kernel".to_string(), JsonValue::String(kernel.to_string()));
    if let Some(payload) = payload {
        obj.insert("payload".to_string(), JsonValue::String(payload.to_string()));
    }
    obj.insert("result".to_string(), JsonValue::String(result.to_string()));
    JsonValue::Object(obj)
}

fn parse_open_gpu_result(value: &JsonValue, span: Span) -> Result<String, RuntimeError> {
    let Some(obj) = value.as_object() else {
        return Err(RuntimeError::OpenReplayInvalid {
            message: "open.gpu value는 객체여야 합니다".to_string(),
            span,
        });
    };
    let schema = obj
        .get("schema")
        .and_then(|v| v.as_str())
        .ok_or(RuntimeError::OpenReplayInvalid {
            message: "open.gpu schema 누락".to_string(),
            span,
        })?;
    if schema != "open.gpu.v1" {
        return Err(RuntimeError::OpenReplayInvalid {
            message: format!("open.gpu schema 불일치: expected=open.gpu.v1 actual={}", schema),
            span,
        });
    }
    let text = obj
        .get("result")
        .and_then(|v| v.as_str())
        .ok_or(RuntimeError::OpenReplayInvalid {
            message: "open.gpu result 누락".to_string(),
            span,
        })?;
    Ok(text.to_string())
}

fn normalize_open_key(path: &str) -> String {
    let (key, _) = normalize_open_path(path);
    key
}

fn normalize_open_path(path: &str) -> (String, PathBuf) {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let input = Path::new(path);
    let abs = if input.is_absolute() {
        input.to_path_buf()
    } else {
        cwd.join(input)
    };
    let abs = normalize_path(&abs);
    let mut key = if let Ok(rel) = abs.strip_prefix(&cwd) {
        path_to_slash(rel)
    } else if let Some(rel) = strip_prefix_case_insensitive(&abs, &cwd) {
        rel
    } else {
        path_to_slash(&abs)
    };
    key = trim_dot_slash(&key);
    if cfg!(windows) {
        key = key.to_ascii_lowercase();
    }
    (key, abs)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut prefix: Option<std::ffi::OsString> = None;
    let mut has_root = false;
    let mut stack: Vec<std::ffi::OsString> = Vec::new();
    for comp in path.components() {
        match comp {
            Component::Prefix(prefix_component) => {
                prefix = Some(prefix_component.as_os_str().to_os_string());
            }
            Component::RootDir => {
                has_root = true;
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if !stack.is_empty() {
                    stack.pop();
                }
            }
            Component::Normal(value) => {
                stack.push(value.to_os_string());
            }
        }
    }
    let mut out = PathBuf::new();
    if let Some(prefix) = prefix {
        out.push(prefix);
    }
    if has_root {
        out.push(Path::new(std::path::MAIN_SEPARATOR_STR));
    }
    for part in stack {
        out.push(part);
    }
    out
}

fn path_to_slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn trim_dot_slash(path: &str) -> String {
    path.strip_prefix("./").unwrap_or(path).to_string()
}

fn strip_prefix_case_insensitive(path: &Path, base: &Path) -> Option<String> {
    let path_str = path_to_slash(path);
    let base_str = path_to_slash(base);
    let path_lower = path_str.to_ascii_lowercase();
    let base_lower = base_str.to_ascii_lowercase();
    let prefix = if base_lower.ends_with('/') {
        base_lower.clone()
    } else {
        format!("{}/", base_lower)
    };
    if path_lower.starts_with(&prefix) {
        let rel = &path_str[prefix.len()..];
        Some(trim_dot_slash(rel))
    } else {
        None
    }
}
