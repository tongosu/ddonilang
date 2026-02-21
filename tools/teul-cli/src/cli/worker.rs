use clap::Parser;
use serde_json::Value;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::Command;

use crate::cli::run::RunEmitSink;
use crate::{build_command_string_from_parts, execute_run_command, Cli, Commands, RunCommandArgs};

pub fn run() -> Result<(), String> {
    let exec_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();

    loop {
        let frame = match read_frame(&mut reader) {
            Ok(Some(frame)) => frame,
            Ok(None) => break,
            Err(err) => return Err(err),
        };
        let request: Value = match serde_json::from_slice(&frame) {
            Ok(value) => value,
            Err(err) => {
                let response =
                    jsonrpc_error(Value::Null, -32700, &format!("요청 파싱 실패: {err}"));
                write_frame(&mut writer, &response)?;
                continue;
            }
        };
        let response = handle_request(&exec_path, request);
        write_frame(&mut writer, &response)?;
    }

    Ok(())
}

fn handle_request(exec_path: &Path, request: Value) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let jsonrpc = request.get("jsonrpc").and_then(|v| v.as_str());
    if jsonrpc != Some("2.0") {
        return jsonrpc_error(id, -32600, "jsonrpc=2.0 이어야 합니다");
    }
    let method = match request.get("method").and_then(|v| v.as_str()) {
        Some(method) => method,
        None => return jsonrpc_error(id, -32600, "method 누락"),
    };
    match method {
        "reset" => reset_request(id, request.get("params")),
        "run_file" => run_file_request(exec_path, id, request.get("params")),
        _ => jsonrpc_error(id, -32601, "지원하지 않는 method"),
    }
}

fn reset_request(id: Value, params: Option<&Value>) -> Value {
    if !is_empty_params(params) {
        return jsonrpc_error(id, -32602, "reset은 params를 받지 않습니다");
    }
    jsonrpc_result(id, Value::Object(serde_json::Map::new()))
}

fn is_empty_params(params: Option<&Value>) -> bool {
    match params {
        None => true,
        Some(Value::Null) => true,
        Some(Value::Array(items)) => items.is_empty(),
        Some(Value::Object(map)) => map.is_empty(),
        _ => false,
    }
}

fn run_file_request(exec_path: &Path, id: Value, params: Option<&Value>) -> Value {
    let params = match params.and_then(|value| value.as_object()) {
        Some(value) => value,
        None => return jsonrpc_error(id, -32602, "params는 객체여야 합니다"),
    };
    let path = match params.get("path").and_then(|value| value.as_str()) {
        Some(value) => value,
        None => return jsonrpc_error(id, -32602, "params.path 누락"),
    };
    let args = match params.get("args") {
        Some(value) => match value.as_array() {
            Some(items) => {
                let mut out = Vec::new();
                for item in items {
                    let Some(text) = item.as_str() else {
                        return jsonrpc_error(id, -32602, "params.args는 문자열 배열이어야 합니다");
                    };
                    out.push(text.to_string());
                }
                out
            }
            None => return jsonrpc_error(id, -32602, "params.args는 배열이어야 합니다"),
        },
        None => Vec::new(),
    };
    if args.iter().any(|arg| arg == "worker") {
        return jsonrpc_error(id, -32602, "params.args에 worker는 허용되지 않습니다");
    }
    let mode = params
        .get("mode")
        .and_then(|value| value.as_str())
        .unwrap_or("inproc");
    match mode {
        "spawn" => run_file_spawn(exec_path, id, path, &args),
        "inproc" => run_file_inproc(id, path, &args),
        _ => jsonrpc_error(id, -32602, "params.mode는 inproc|spawn 이어야 합니다"),
    }
}

fn run_file_spawn(exec_path: &Path, id: Value, path: &str, args: &[String]) -> Value {
    let mut command = Command::new(exec_path);
    command.arg("run").arg(path);
    for arg in args {
        command.arg(arg);
    }
    let output = match command.output() {
        Ok(output) => output,
        Err(err) => {
            return jsonrpc_error(id, -32000, &format!("실행 실패: {err}"));
        }
    };
    let stdout_text = String::from_utf8_lossy(&output.stdout);
    let stderr_text = String::from_utf8_lossy(&output.stderr);
    let mut stdout_lines: Vec<String> = stdout_text.lines().map(|line| line.to_string()).collect();
    let stderr_lines: Vec<String> = stderr_text.lines().map(|line| line.to_string()).collect();
    let mut state_hash = None;
    let mut trace_hash = None;
    let mut bogae_hash = None;
    stdout_lines.retain(|line| {
        if let Some(value) = line.strip_prefix("state_hash=") {
            state_hash = Some(value.to_string());
            return false;
        }
        if let Some(value) = line.strip_prefix("trace_hash=") {
            trace_hash = Some(value.to_string());
            return false;
        }
        if let Some(value) = line.strip_prefix("bogae_hash=") {
            bogae_hash = Some(value.to_string());
            return false;
        }
        true
    });
    let mut result = serde_json::Map::new();
    result.insert("ok".to_string(), Value::Bool(output.status.success()));
    let exit_code = output.status.code().unwrap_or(-1) as i64;
    result.insert("exit_code".to_string(), Value::Number(exit_code.into()));
    result.insert(
        "stdout".to_string(),
        Value::Array(stdout_lines.into_iter().map(Value::String).collect()),
    );
    result.insert(
        "stderr".to_string(),
        Value::Array(stderr_lines.into_iter().map(Value::String).collect()),
    );
    if let Some(value) = state_hash {
        result.insert("state_hash".to_string(), Value::String(value));
    }
    if let Some(value) = trace_hash {
        result.insert("trace_hash".to_string(), Value::String(value));
    }
    if let Some(value) = bogae_hash {
        result.insert("bogae_hash".to_string(), Value::String(value));
    }
    jsonrpc_result(id, Value::Object(result))
}

fn run_file_inproc(id: Value, path: &str, args: &[String]) -> Value {
    match run_file_inproc_inner(path, args) {
        Ok(report) => {
            let mut result = serde_json::Map::new();
            result.insert("ok".to_string(), Value::Bool(report.ok));
            result.insert(
                "exit_code".to_string(),
                Value::Number(report.exit_code.into()),
            );
            result.insert(
                "stdout".to_string(),
                Value::Array(report.stdout.into_iter().map(Value::String).collect()),
            );
            result.insert(
                "stderr".to_string(),
                Value::Array(report.stderr.into_iter().map(Value::String).collect()),
            );
            if let Some(value) = report.state_hash {
                result.insert("state_hash".to_string(), Value::String(value));
            }
            if let Some(value) = report.trace_hash {
                result.insert("trace_hash".to_string(), Value::String(value));
            }
            if let Some(value) = report.bogae_hash {
                result.insert("bogae_hash".to_string(), Value::String(value));
            }
            jsonrpc_result(id, Value::Object(result))
        }
        Err(message) => jsonrpc_error(id, -32000, &message),
    }
}

struct InprocReport {
    ok: bool,
    exit_code: i64,
    stdout: Vec<String>,
    stderr: Vec<String>,
    state_hash: Option<String>,
    trace_hash: Option<String>,
    bogae_hash: Option<String>,
}

struct CaptureEmitter {
    stdout: Vec<String>,
    stderr: Vec<String>,
}

impl CaptureEmitter {
    fn new() -> Self {
        Self {
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    fn push_lines(target: &mut Vec<String>, text: &str) {
        for line in text.split('\n') {
            target.push(line.to_string());
        }
    }
}

impl RunEmitSink for CaptureEmitter {
    fn out(&mut self, line: &str) {
        Self::push_lines(&mut self.stdout, line);
    }

    fn err(&mut self, line: &str) {
        Self::push_lines(&mut self.stderr, line);
    }
}

fn split_lines(text: &str) -> Vec<String> {
    text.lines().map(|line| line.to_string()).collect()
}

fn run_file_inproc_inner(path: &str, args: &[String]) -> Result<InprocReport, String> {
    let mut cli_args = Vec::with_capacity(args.len() + 3);
    cli_args.push("teul-cli".to_string());
    cli_args.push("run".to_string());
    cli_args.push(path.to_string());
    cli_args.extend(args.iter().cloned());

    let cli = match Cli::try_parse_from(&cli_args) {
        Ok(cli) => cli,
        Err(err) => {
            return Ok(InprocReport {
                ok: false,
                exit_code: 2,
                stdout: Vec::new(),
                stderr: split_lines(&err.to_string()),
                state_hash: None,
                trace_hash: None,
                bogae_hash: None,
            });
        }
    };

    let Commands::Run {
        file,
        madi,
        seed,
        age_target,
        state,
        state_file,
        diag_jsonl,
        diag_report_out,
        enable_repro,
        repro_json,
        run_manifest,
        artifact,
        trace_json,
        geoul_out,
        geoul_record_out,
        trace_tier,
        bogae,
        bogae_codec,
        bogae_out,
        bogae_skin,
        bogae_overlay,
        bogae_cmd_policy,
        bogae_cmd_cap,
        bogae_cache_log,
        bogae_live,
        console_cell_aspect,
        console_grid,
        console_panel_cols,
        until_gameover,
        gameover_key,
        sam,
        record_sam,
        sam_live,
        sam_live_host,
        sam_live_port,
        madi_hz,
        open_mode,
        open_log,
        open_bundle,
        no_open,
        unsafe_open,
        lang_mode,
    } = cli.command
    else {
        return Ok(InprocReport {
            ok: false,
            exit_code: 2,
            stdout: Vec::new(),
            stderr: vec!["worker는 run 명령만 지원합니다".to_string()],
            state_hash: None,
            trace_hash: None,
            bogae_hash: None,
        });
    };

    let mut emitter = CaptureEmitter::new();
    let run_args = RunCommandArgs {
        file,
        madi,
        seed,
        age_target,
        state,
        state_file,
        diag_jsonl,
        diag_report_out,
        enable_repro,
        repro_json,
        run_manifest,
        artifact,
        trace_json,
        geoul_out,
        geoul_record_out,
        trace_tier,
        bogae,
        bogae_codec,
        bogae_out,
        bogae_skin,
        bogae_overlay,
        bogae_cmd_policy,
        bogae_cmd_cap,
        bogae_cache_log,
        bogae_live,
        console_cell_aspect,
        console_grid,
        console_panel_cols,
        until_gameover,
        gameover_key,
        sam,
        record_sam,
        sam_live,
        sam_live_host,
        sam_live_port,
        madi_hz,
        open_mode,
        open_log,
        open_bundle,
        no_open,
        unsafe_open,
        lang_mode,
        run_command_override: Some(build_command_string_from_parts(&cli_args)),
    };

    let result = execute_run_command(run_args, &mut emitter);
    let ok = result.is_ok();
    let exit_code = if ok { 0 } else { 1 };
    if let Err(err) = result {
        emitter.err(&err);
    }

    let mut stdout_lines = emitter.stdout;
    let stderr_lines = emitter.stderr;
    let mut state_hash = None;
    let mut trace_hash = None;
    let mut bogae_hash = None;
    stdout_lines.retain(|line| {
        if let Some(value) = line.strip_prefix("state_hash=") {
            state_hash = Some(value.to_string());
            return false;
        }
        if let Some(value) = line.strip_prefix("trace_hash=") {
            trace_hash = Some(value.to_string());
            return false;
        }
        if let Some(value) = line.strip_prefix("bogae_hash=") {
            bogae_hash = Some(value.to_string());
            return false;
        }
        true
    });

    Ok(InprocReport {
        ok,
        exit_code,
        stdout: stdout_lines,
        stderr: stderr_lines,
        state_hash,
        trace_hash,
        bogae_hash,
    })
}

fn jsonrpc_result(id: Value, result: Value) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("jsonrpc".to_string(), Value::String("2.0".to_string()));
    obj.insert("id".to_string(), id);
    obj.insert("result".to_string(), result);
    Value::Object(obj)
}

fn jsonrpc_error(id: Value, code: i64, message: &str) -> Value {
    let mut err = serde_json::Map::new();
    err.insert("code".to_string(), Value::Number(code.into()));
    err.insert("message".to_string(), Value::String(message.to_string()));
    let mut obj = serde_json::Map::new();
    obj.insert("jsonrpc".to_string(), Value::String("2.0".to_string()));
    obj.insert("id".to_string(), id);
    obj.insert("error".to_string(), Value::Object(err));
    Value::Object(obj)
}

fn read_frame(reader: &mut BufReader<impl Read>) -> Result<Option<Vec<u8>>, String> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line).map_err(|e| e.to_string())?;
        if bytes == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end_matches(|ch| ch == '\r' || ch == '\n');
        if trimmed.is_empty() {
            break;
        }
        let lower = trimmed.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("content-length:") {
            let value = rest
                .trim()
                .parse::<usize>()
                .map_err(|_| "content-length 값이 숫자가 아닙니다".to_string())?;
            content_length = Some(value);
        }
    }
    let length = content_length.ok_or_else(|| "content-length 헤더가 없습니다".to_string())?;
    let mut buf = vec![0u8; length];
    reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(Some(buf))
}

fn write_frame(writer: &mut impl Write, value: &Value) -> Result<(), String> {
    let json = detjson_string(value);
    let header = format!("Content-Length: {}\r\n\r\n", json.as_bytes().len());
    writer
        .write_all(header.as_bytes())
        .map_err(|e| e.to_string())?;
    writer
        .write_all(json.as_bytes())
        .map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

fn detjson_string(value: &Value) -> String {
    let ordered = order_value(value);
    serde_json::to_string(&ordered).unwrap_or_else(|_| "{}".to_string())
}

fn order_value(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(order_value).collect()),
        Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            let mut ordered = serde_json::Map::new();
            for key in keys {
                if let Some(value) = map.get(key) {
                    ordered.insert(key.to_string(), order_value(value));
                }
            }
            Value::Object(ordered)
        }
        _ => value.clone(),
    }
}
