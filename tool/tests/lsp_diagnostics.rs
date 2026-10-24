use ddonirang_tool::lsp::{
    byte_range_to_lsp_range,
    diagnostics::{sort_diagnostics, Diagnostic},
    encode_message, ByteRange, FrameDecoder, LspPosition, LspRange, LspServer, PositionEncoding,
    PublishRejection, RpcMessage, ServerEvent,
};
use ddonirang_tool::source_analysis::{
    analyze_source, SourceAnalysisDiagnostic, SourceAnalysisRequest,
};
use serde_json::{json, Value};

const URI: &str = "file:///lsp-diagnostics.ddn";
// Accepted by `ddonilang 검사 --source-profile v1-core-v25` with no diagnostic.
const VALID: &str = "첫함수:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 := { 첫함수하기. }";
const LEGACY_EQUALS: &str = "검산:움직씨 = {\n}\n";
// A well-formed Tuple type is supported by VC13; a missing position keeps
// this fixture a parser error, not a runtime Tuple/value mismatch.
const SYNTAX_ERROR: &str = "검산:움직씨 := {\n    값:(수,, 글) := 1.\n}\n";

fn init(server: &mut LspServer) -> ServerEvent {
    server
        .handle_message(RpcMessage::Request {
            id: json!(1),
            method: "initialize".into(),
            params: Some(json!({
                "capabilities": {"general": {"positionEncodings": ["utf-16", "utf-8"]}}
            })),
        })
        .unwrap()
}

fn open(server: &mut LspServer, source: &str, version: i64) -> ServerEvent {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didOpen".into(),
            params: Some(json!({
                "textDocument": {
                    "uri": URI,
                    "languageId": "ddn",
                    "version": version,
                    "text": source,
                }
            })),
        })
        .unwrap()
}

fn change_full(server: &mut LspServer, source: &str, version: i64) -> ServerEvent {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didChange".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": version},
                "contentChanges": [{"text": source}],
            })),
        })
        .unwrap()
}

fn close(server: &mut LspServer) -> ServerEvent {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didClose".into(),
            params: Some(json!({"textDocument": {"uri": URI}})),
        })
        .unwrap()
}

fn publish_params(event: &ServerEvent) -> Value {
    assert_eq!(event.responses.len(), 1);
    let RpcMessage::Notification { method, params } = &event.responses[0] else {
        panic!("expected diagnostic notification: {event:?}");
    };
    assert_eq!(method, "textDocument/publishDiagnostics");
    params.clone().expect("publishDiagnostics params")
}

fn diagnostics(event: &ServerEvent) -> Vec<Value> {
    publish_params(event)["diagnostics"]
        .as_array()
        .expect("diagnostics array")
        .clone()
}

fn pull_diagnostics(
    server: &mut LspServer,
    id: i64,
    uri: &str,
    version: Option<i64>,
    previous_result_id: Option<&str>,
) -> Value {
    let mut document = json!({"uri": uri});
    if let Some(version) = version {
        document["version"] = json!(version);
    }
    let mut params = json!({"textDocument": document});
    if let Some(previous_result_id) = previous_result_id {
        params["previousResultId"] = json!(previous_result_id);
    }
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: "textDocument/diagnostic".into(),
            params: Some(params),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result.clone(),
        _ => panic!("pull diagnostic response missing: {event:?}"),
    }
}

fn workspace_diagnostics(
    server: &mut LspServer,
    id: i64,
    previous_result_ids: Option<Value>,
) -> Value {
    let params = previous_result_ids
        .map(|previous| json!({"previousResultIds": previous}))
        .unwrap_or_else(|| json!({}));
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: "workspace/diagnostic".into(),
            params: Some(params),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result.clone(),
        _ => panic!("workspace diagnostic response missing: {event:?}"),
    }
}

/// The first diagnostic of the shared check that `ddonilang 검사` runs.
fn shared_check(source: &str) -> SourceAnalysisDiagnostic {
    analyze_source(SourceAnalysisRequest {
        source_text: source.to_string(),
        path: URI.to_string(),
        analysis_generation: 0,
        source_profile_identity: "v1-core-v25".to_string(),
    })
    .diagnostics
    .into_iter()
    .next()
    .expect("fixture must produce a shared check diagnostic")
}

fn expected_range(source: &str) -> Value {
    let error = shared_check(source);
    let start = previous_char_boundary(source, error.span.start.min(source.len()));
    let end = next_char_boundary(source, error.span.end.min(source.len())).max(start);
    let range =
        byte_range_to_lsp_range(source, ByteRange { start, end }, PositionEncoding::Utf16).unwrap();
    json!({
        "start": {"line": range.start.line, "character": range.start.character},
        "end": {"line": range.end.line, "character": range.end.character},
    })
}

fn previous_char_boundary(source: &str, mut offset: usize) -> usize {
    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn next_char_boundary(source: &str, mut offset: usize) -> usize {
    while offset < source.len() && !source.is_char_boundary(offset) {
        offset += 1;
    }
    offset
}

#[test]
fn d01_d04_current_v25_diagnostics_reuse_parser_identity() {
    let mut server = LspServer::new();
    assert!(matches!(
        init(&mut server).responses[0],
        RpcMessage::Response { result: Ok(_), .. }
    ));
    assert!(diagnostics(&open(&mut server, VALID, 1)).is_empty());

    let legacy = diagnostics(&open_after_close(&mut server, LEGACY_EQUALS, 2));
    assert_eq!(legacy[0]["code"], "E_V25_DEFINITION_MIGRATION_REQUIRED");
    assert_eq!(legacy[0]["severity"], 1);
    assert_eq!(legacy[0]["range"], expected_range(LEGACY_EQUALS));
    assert_eq!(legacy[0]["data"]["documentVersion"], 2);
    assert_eq!(legacy[0]["data"]["sourceProfile"], "v25");

    let syntax = diagnostics(&open_after_close(&mut server, SYNTAX_ERROR, 3));
    let shared_error = shared_check(SYNTAX_ERROR);
    assert_eq!(syntax[0]["code"], shared_error.code.as_str());
    assert_eq!(syntax[0]["message"], shared_error.message.as_str());
    assert_eq!(syntax[0]["range"], expected_range(SYNTAX_ERROR));

    let legacy_chaebi = diagnostics(&open_after_close(
        &mut server,
        "첫함수:셈씨 := { 보임 { 1 돌려줘. }. }",
        4,
    ));
    assert_eq!(legacy_chaebi.len(), 1);
    assert!(matches!(
        legacy_chaebi[0]["code"].as_str(),
        Some("E_BOIM_LEGACY_REMOVED") | Some("E_V25_DEFINITION_MIGRATION_REQUIRED")
    ));
}

#[test]
fn crlf_multichar_parser_token_matches_shared_cli_diagnostic() {
    let source = "// 😀 한글\r\n값 <- 1.\r\n// 😀 주석\r\n값 <- (3 보여주기.\r\n";
    let mut server = LspServer::new();
    init(&mut server);
    let actual = diagnostics(&open(&mut server, source, 1));
    let expected = shared_check(source);
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0]["code"], expected.code);
    assert_eq!(actual[0]["message"], expected.message);
    assert_eq!(actual[0]["range"], expected_range(source));
}

fn open_after_close(server: &mut LspServer, source: &str, version: i64) -> ServerEvent {
    let _ = close(server);
    open(server, source, version)
}

#[test]
fn d05_d06_utf16_ranges_cover_korean_and_non_bmp() {
    let korean = "가\n검산:움직씨 = {\n}\n";
    let emoji = "🙂검산:움직씨 = {\n}\n";
    let mut server = LspServer::new();
    init(&mut server);
    let korean_diagnostic = diagnostics(&open(&mut server, korean, 1));
    assert_eq!(korean_diagnostic[0]["range"], expected_range(korean));
    close(&mut server);
    let emoji_diagnostic = diagnostics(&open(&mut server, emoji, 2));
    assert_eq!(emoji_diagnostic[0]["range"], expected_range(emoji));
    assert_eq!(
        byte_range_to_lsp_range(
            "🙂한",
            ByteRange { start: 4, end: 7 },
            PositionEncoding::Utf16
        )
        .unwrap()
        .start
        .character,
        2
    );
}

#[test]
fn d07_d08_change_reanalyzes_current_version() {
    let mut server = LspServer::new();
    init(&mut server);
    assert_eq!(diagnostics(&open(&mut server, LEGACY_EQUALS, 1)).len(), 1);
    assert!(diagnostics(&change_full(&mut server, VALID, 2)).is_empty());
    let next = diagnostics(&change_full(&mut server, LEGACY_EQUALS, 3));
    assert_eq!(
        publish_params(&change_full(&mut server, LEGACY_EQUALS, 4))["version"],
        4
    );
    assert_eq!(next[0]["code"], "E_V25_DEFINITION_MIGRATION_REQUIRED");
}

#[test]
fn d09_deterministic_order_is_range_then_code() {
    let mut diagnostics = vec![
        Diagnostic {
            code: "Z".into(),
            severity: 1,
            message: "z".into(),
            range: LspRange {
                start: LspPosition {
                    line: 1,
                    character: 0,
                },
                end: LspPosition {
                    line: 1,
                    character: 1,
                },
            },
        },
        Diagnostic {
            code: "A".into(),
            severity: 1,
            message: "a".into(),
            range: LspRange {
                start: LspPosition {
                    line: 0,
                    character: 4,
                },
                end: LspPosition {
                    line: 0,
                    character: 5,
                },
            },
        },
    ];
    sort_diagnostics(&mut diagnostics);
    assert_eq!(diagnostics[0].code, "A");
    assert_eq!(diagnostics[1].code, "Z");
}

#[test]
fn d10_close_clears_diagnostics_and_d11_rejects_stale_result() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, LEGACY_EQUALS, 1);
    let stale = server.begin_diagnostic_request(URI).unwrap();
    let current = change_full(&mut server, VALID, 2);
    assert!(diagnostics(&current).is_empty());
    let mut stale = stale;
    stale.snapshot_identity = server.documents().get(URI).unwrap().identity.clone();
    assert_eq!(
        server.publish_diagnostic_request(&stale),
        Err(PublishRejection::VersionMismatch)
    );
    let cleared = close(&mut server);
    assert!(diagnostics(&cleared).is_empty());
    assert_eq!(
        server.begin_diagnostic_request(URI),
        Err(PublishRejection::DocumentClosed)
    );
}

#[test]
fn d12_fragmented_transport_publishes_after_initialize() {
    let messages = [
        RpcMessage::Request {
            id: json!(1),
            method: "initialize".into(),
            params: Some(json!({"capabilities": {"general": {"positionEncodings": ["utf-16"]}}})),
        },
        RpcMessage::Notification {
            method: "textDocument/didOpen".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "languageId": "ddn", "version": 1, "text": LEGACY_EQUALS}
            })),
        },
    ];
    let mut bytes = Vec::new();
    for message in messages {
        bytes.extend(encode_message(&message).unwrap());
    }
    let mut decoder = FrameDecoder::default();
    let mut server = LspServer::new();
    let mut events = Vec::new();
    for chunk in bytes.chunks(3) {
        for message in decoder.push(chunk).unwrap() {
            events.push(server.handle_message(message).unwrap());
        }
    }
    assert!(matches!(
        events[0].responses[0],
        RpcMessage::Response { result: Ok(_), .. }
    ));
    assert_eq!(
        diagnostics(&events[1])[0]["code"],
        "E_V25_DEFINITION_MIGRATION_REQUIRED"
    );
}

#[test]
fn d13_pull_diagnostics_reuses_push_payload_and_result_id() {
    let mut server = LspServer::new();
    let initialize = init(&mut server);
    let capabilities = match &initialize.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result,
        _ => panic!("initialize response missing"),
    };
    assert_eq!(
        capabilities["capabilities"]["diagnosticProvider"]["identifier"],
        "ddonirang-v25"
    );
    assert_eq!(
        capabilities["capabilities"]["diagnosticProvider"]["workspaceDiagnostics"],
        false
    );
    assert_eq!(
        capabilities["capabilities"]["diagnosticProvider"]["interFileDependencies"],
        false
    );

    let push = open(&mut server, "🙂검산:움직씨 = {\n}\n", 1);
    let full = pull_diagnostics(&mut server, 40, URI, Some(1), None);
    assert_eq!(full["kind"], "full");
    assert_eq!(full["items"], publish_params(&push)["diagnostics"]);
    let result_id = full["resultId"].as_str().expect("resultId").to_string();
    assert!(result_id.starts_with("sha256:"));
    let snapshot_identity = server
        .documents()
        .get(URI)
        .expect("open snapshot")
        .identity
        .as_str()
        .to_string();
    assert!(result_id.ends_with(&format!(":{snapshot_identity}")));

    let unchanged = pull_diagnostics(&mut server, 41, URI, Some(1), Some(&result_id));
    assert_eq!(unchanged, json!({"kind": "unchanged", "resultId": result_id}));

    let repeated = pull_diagnostics(&mut server, 42, URI, Some(1), None);
    assert_eq!(repeated, full);
}

#[test]
fn d14_pull_diagnostics_change_refreshes_and_stale_fails_closed() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, LEGACY_EQUALS, 1);
    let first = pull_diagnostics(&mut server, 50, URI, Some(1), None);
    let first_id = first["resultId"].as_str().unwrap().to_string();
    change_full(&mut server, VALID, 2);
    let refreshed = pull_diagnostics(&mut server, 51, URI, Some(2), Some(&first_id));
    assert_eq!(refreshed["kind"], "full");
    assert!(refreshed["items"].as_array().unwrap().is_empty());
    assert_ne!(refreshed["resultId"], first_id);

    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(52),
            method: "textDocument/diagnostic".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": 1}
            })),
        })
        .unwrap();
    match &stale.responses[0] {
        RpcMessage::Response { result: Err(error), .. } => assert_eq!(error.code, -32801),
        _ => panic!("stale diagnostic request was not rejected: {stale:?}"),
    }
}

#[test]
fn d15_workspace_pull_is_open_only_and_uri_deterministic() {
    const URI_A: &str = "file:///a-open.ddn";
    const URI_B: &str = "file:///b-open.ddn";
    let mut server = LspServer::new();
    init(&mut server);
    open_uri(&mut server, URI_B, LEGACY_EQUALS, 1);
    open_uri(&mut server, URI_A, "🙂검산:움직씨 = {\n}\n", 1);
    let report = workspace_diagnostics(&mut server, 60, None);
    let items = report["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["uri"], URI_A);
    assert_eq!(items[1]["uri"], URI_B);
    assert_eq!(items[0]["kind"], "full");
    let previous = items
        .iter()
        .map(|item| json!({"uri": item["uri"], "value": item["resultId"]}))
        .collect::<Vec<_>>();
    let unchanged = workspace_diagnostics(&mut server, 61, Some(json!(previous)));
    assert!(unchanged["items"].as_array().unwrap().iter().all(|item| {
        item["kind"] == "unchanged" && item.get("items").is_none()
    }));

    close_uri(&mut server, URI_A);
    let after_close = workspace_diagnostics(&mut server, 62, None);
    let after_close_items = after_close["items"].as_array().unwrap();
    assert_eq!(after_close_items.len(), 1);
    assert_eq!(after_close_items[0]["uri"], URI_B);
}

fn close_uri(server: &mut LspServer, uri: &str) {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didClose".into(),
            params: Some(json!({"textDocument": {"uri": uri}})),
        })
        .unwrap();
}

fn open_uri(server: &mut LspServer, uri: &str, source: &str, version: i64) -> ServerEvent {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didOpen".into(),
            params: Some(json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": "ddn",
                    "version": version,
                    "text": source,
                }
            })),
        })
        .unwrap()
}

#[test]
fn d16_pull_diagnostics_cancellation_has_no_partial_result() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, LEGACY_EQUALS, 1);
    server.cancel_request(&json!(70));
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(70),
            method: "workspace/diagnostic".into(),
            params: Some(json!({})),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response { result: Err(error), .. } => assert_eq!(error.code, -32800),
        _ => panic!("cancelled diagnostic request published partial result: {event:?}"),
    }
}
