//! Editor first flow: open a file, see the shared `검사` diagnostics, edit,
//! and get diagnostics for exactly the newest text. Every message goes through
//! the transport entry `handle_message_or_report`, as in the shipped server.
use ddonirang_tool::lsp::{LspServer, RpcMessage, ServerEvent, ServerState};
use ddonirang_tool::source_analysis::{analyze_source, SourceAnalysisRequest};
use serde_json::{json, Value};

const URI: &str = "file:///C:/work/%ED%95%9C%EA%B8%80.ddn";
const GOOD: &str = "// 😀 한글\n값 <- (1 + 2).\n값 보여주기.\n";
const BAD: &str = "// 😀 한글\n값 <- (1 + 2.\n";

fn send(server: &mut LspServer, message: RpcMessage) -> ServerEvent {
    server.handle_message_or_report(message)
}

fn notify(server: &mut LspServer, method: &str, params: Value) -> ServerEvent {
    send(
        server,
        RpcMessage::Notification {
            method: method.into(),
            params: Some(params),
        },
    )
}

fn request(server: &mut LspServer, id: i64, method: &str, params: Value) -> RpcMessage {
    let event = send(
        server,
        RpcMessage::Request {
            id: json!(id),
            method: method.into(),
            params: Some(params),
        },
    );
    assert_eq!(event.responses.len(), 1, "{event:?}");
    event.responses[0].clone()
}

/// VS Code (vscode-languageclient 8.1.0) offers UTF-16 only and pulls diagnostics.
fn init_vscode(server: &mut LspServer) -> Value {
    init_with(
        server,
        json!({
            "general": {"positionEncodings": ["utf-16"]},
            "textDocument": {"diagnostic": {"dynamicRegistration": true}}
        }),
    )
}

fn init_with(server: &mut LspServer, capabilities: Value) -> Value {
    match request(server, 1, "initialize", json!({"capabilities": capabilities})) {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result,
        other => panic!("initialize failed: {other:?}"),
    }
}

fn open(server: &mut LspServer, uri: &str, language_id: &str, version: i64, text: &str) -> ServerEvent {
    notify(
        server,
        "textDocument/didOpen",
        json!({"textDocument": {"uri": uri, "languageId": language_id, "version": version, "text": text}}),
    )
}

fn change(server: &mut LspServer, uri: &str, version: i64, changes: Value) -> ServerEvent {
    notify(
        server,
        "textDocument/didChange",
        json!({"textDocument": {"uri": uri, "version": version}, "contentChanges": changes}),
    )
}

fn pull(server: &mut LspServer, id: i64, uri: &str, version: Option<i64>) -> Result<Value, i32> {
    let mut document = json!({"uri": uri});
    if let Some(version) = version {
        document["version"] = json!(version);
    }
    match request(server, id, "textDocument/diagnostic", json!({"textDocument": document})) {
        RpcMessage::Response {
            result: Ok(result), ..
        } => Ok(result),
        RpcMessage::Response {
            result: Err(error), ..
        } => Err(error.code),
        other => panic!("unexpected: {other:?}"),
    }
}

fn pulled_items(server: &mut LspServer, id: i64, uri: &str) -> Vec<Value> {
    let result = pull(server, id, uri, None).expect("pull result");
    assert_eq!(result["kind"], "full");
    result["items"].as_array().unwrap().clone()
}

fn methods(event: &ServerEvent) -> Vec<String> {
    event
        .responses
        .iter()
        .map(|message| match message {
            RpcMessage::Notification { method, .. } => method.clone(),
            RpcMessage::Request { method, .. } => format!("request:{method}"),
            RpcMessage::Response { .. } => "response".into(),
        })
        .collect()
}

fn published(event: &ServerEvent) -> Vec<Value> {
    event
        .responses
        .iter()
        .filter_map(|message| match message {
            RpcMessage::Notification { method, params } if method == "textDocument/publishDiagnostics" => {
                params.clone()
            }
            _ => None,
        })
        .collect()
}

fn range(item: &Value) -> (u64, u64, u64, u64) {
    let range = &item["range"];
    (
        range["start"]["line"].as_u64().unwrap(),
        range["start"]["character"].as_u64().unwrap(),
        range["end"]["line"].as_u64().unwrap(),
        range["end"]["character"].as_u64().unwrap(),
    )
}

fn shared_check(source: &str) -> (String, Vec<(String, usize, usize)>) {
    let snapshot = analyze_source(SourceAnalysisRequest {
        source_text: source.to_string(),
        path: URI.to_string(),
        analysis_generation: 0,
        source_profile_identity: "v1-core-v25".to_string(),
    });
    (
        snapshot.outcome.to_string(),
        snapshot
            .diagnostics
            .into_iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.span.start, diagnostic.span.end))
            .collect(),
    )
}

#[test]
fn f1_vscode_language_id_gets_the_same_diagnostics_as_cli_check() {
    let mut server = LspServer::new();
    init_vscode(&mut server);
    open(&mut server, URI, "ddonilang", 1, BAD);
    let items = pulled_items(&mut server, 2, URI);
    let (outcome, expected) = shared_check(BAD);
    assert_eq!(outcome, "REJECTED");
    assert_eq!(expected.len(), 1);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["code"], expected[0].0.as_str());
    // `.` on line 2 is byte 28..29 after "// 😀 한글\n" (15 bytes) and 13 bytes.
    assert_eq!((expected[0].1, expected[0].2), (28, 29));
    assert_eq!(range(&items[0]), (1, 11, 1, 12));
    assert_eq!(items[0]["severity"], 1);

    let (outcome, expected) = shared_check(GOOD);
    assert_eq!((outcome.as_str(), expected.len()), ("ACCEPTED", 0));
    change(&mut server, URI, 2, json!([{"text": GOOD}]));
    assert!(pulled_items(&mut server, 3, URI).is_empty());
}

#[test]
fn f2_other_registered_surfaces_are_not_analyzed_as_ddn_source() {
    let mut server = LspServer::new();
    init_vscode(&mut server);
    for (id, uri) in [
        (2, "file:///C:/work/a.ddl"),
        (3, "file:///C:/work/a.%EC%98%AE%EA%B9%80"),
        (4, "untitled:Untitled-1"),
    ] {
        open(&mut server, uri, "ddonilang", 1, BAD);
        assert!(pulled_items(&mut server, id, uri).is_empty(), "{uri}");
    }
}

#[test]
fn f3_hangul_and_emoji_prefix_ranges_follow_the_negotiated_encoding() {
    // The error `.` follows a Hangul name and a non-BMP emoji on the same line:
    // 22 UTF-8 bytes, 18 UTF-16 units, 17 scalars.
    let source = "값 <- \"😀\" + (1 + 2.\n";
    let (_, expected) = shared_check(source);
    assert_eq!((expected[0].1, expected[0].2), (22, 23));
    for (encoding, start) in [("utf-16", 18), ("utf-8", 22), ("utf-32", 17)] {
        let mut server = LspServer::new();
        let result = init_with(&mut server, json!({"general": {"positionEncodings": [encoding]}}));
        assert_eq!(result["capabilities"]["positionEncoding"], encoding);
        let event = open(&mut server, URI, "ddonilang", 1, source);
        let items = published(&event)[0]["diagnostics"].as_array().unwrap().clone();
        // The span is the one-byte `.`, one unit wide in every encoding.
        assert_eq!(range(&items[0]), (0, start, 0, start + 1), "{encoding}");
    }
    // A client that offers no known encoding still gets the protocol default.
    let mut server = LspServer::new();
    let result = init_with(&mut server, json!({"general": {"positionEncodings": ["utf-7"]}}));
    assert_eq!(result["capabilities"]["positionEncoding"], "utf-16");
}

#[test]
fn f4_crlf_and_multiline_ranged_edits_keep_positions() {
    let mut server = LspServer::new();
    init_with(&mut server, json!({"general": {"positionEncodings": ["utf-16"]}}));
    let crlf = "// 😀\r\n값 <- (1 + 2).\r\n값 보여주기.\r\n";
    let event = open(&mut server, URI, "ddonilang", 1, crlf);
    assert!(published(&event)[0]["diagnostics"].as_array().unwrap().is_empty());
    // Replace "(1 + 2)." on line 1 and the first char of line 2 with two new
    // lines that leave a parenthesis open on line 2.
    let event = change(
        &mut server,
        URI,
        2,
        json!([{
            "range": {"start": {"line": 1, "character": 5}, "end": {"line": 2, "character": 1}},
            "text": "1.\r\n😀\r\n값 <- (3"
        }]),
    );
    let text = server.documents().get(URI).unwrap().text().unwrap().to_string();
    assert_eq!(text, "// 😀\r\n값 <- 1.\r\n😀\r\n값 <- (3 보여주기.\r\n");
    let items = published(&event)[0]["diagnostics"].as_array().unwrap().clone();
    let (_, expected) = shared_check(&text);
    assert_eq!(items.len(), expected.len());
    assert!(!items.is_empty());
    for (item, (_, start, end)) in items.iter().zip(expected) {
        let line_start = text[..start].rfind('\n').map_or(0, |index| index + 1);
        let line = text[..start].matches('\n').count() as u64;
        let character = text[line_start..start].encode_utf16().count() as u64;
        let end_character = text[line_start..end].encode_utf16().count() as u64;
        assert_eq!(range(item), (line, character, line, end_character));
    }
}

#[test]
fn f5_older_version_never_replaces_the_newer_result() {
    let mut server = LspServer::new();
    init_with(&mut server, json!({"general": {"positionEncodings": ["utf-16"]}}));
    let first = open(&mut server, URI, "ddonilang", 10, BAD);
    assert_eq!(published(&first)[0]["version"], 10);
    let second = change(&mut server, URI, 11, json!([{"text": GOOD}]));
    let report = &published(&second)[0];
    assert_eq!(report["version"], 11);
    assert!(report["diagnostics"].as_array().unwrap().is_empty());
    // A pull for v10 arriving after v11 is refused instead of answered with v10.
    assert_eq!(pull(&mut server, 2, URI, Some(10)), Err(-32801));
    assert!(pull(&mut server, 3, URI, Some(11)).unwrap()["items"]
        .as_array()
        .unwrap()
        .is_empty());
    // A late v10 edit cannot be applied; the server drops the document and
    // clears its diagnostics rather than keeping v11 results for unknown text.
    let late = change(&mut server, URI, 10, json!([{"text": BAD}]));
    assert_eq!(
        methods(&late),
        ["window/logMessage", "textDocument/publishDiagnostics", "window/showMessage"]
    );
    let cleared = &published(&late)[0];
    assert_eq!(cleared["uri"], URI);
    assert!(cleared["diagnostics"].as_array().unwrap().is_empty());
    assert!(server.documents().get(URI).is_none());
    assert_eq!(server.state(), ServerState::Running);
}

#[test]
fn f6_unsaved_editor_text_is_analyzed_not_the_file_on_disk() {
    let dir = std::env::temp_dir().join(format!("ddn-lsp-first-flow-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("저장본.ddn");
    std::fs::write(&path, GOOD).unwrap();
    let uri = format!("file:///{}", path.display().to_string().replace('\\', "/"));
    let mut server = LspServer::new();
    init_vscode(&mut server);
    open(&mut server, &uri, "ddonilang", 1, BAD);
    assert_eq!(pulled_items(&mut server, 2, &uri).len(), 1, "unsaved buffer has the error");
    std::fs::write(&path, BAD).unwrap();
    change(&mut server, &uri, 2, json!([{"text": GOOD}]));
    assert!(pulled_items(&mut server, 3, &uri).is_empty(), "disk error is not shown");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn f7_close_reopen_and_restart_never_show_previous_session_results() {
    let mut server = LspServer::new();
    init_with(&mut server, json!({"general": {"positionEncodings": ["utf-16"]}}));
    open(&mut server, URI, "ddonilang", 5, BAD);
    let closed = notify(&mut server, "textDocument/didClose", json!({"textDocument": {"uri": URI}}));
    let cleared = &published(&closed)[0];
    assert!(cleared["diagnostics"].as_array().unwrap().is_empty());
    assert!(cleared["version"].is_null());
    assert_eq!(pull(&mut server, 2, URI, None), Err(-32602));
    // Reopen restarts the client version at 1 with new text.
    let reopened = open(&mut server, URI, "ddonilang", 1, GOOD);
    let report = &published(&reopened)[0];
    assert_eq!(report["version"], 1);
    assert!(report["diagnostics"].as_array().unwrap().is_empty());

    // A restarted server has no documents until the client opens them again.
    let mut restarted = LspServer::new();
    init_vscode(&mut restarted);
    assert_eq!(pull(&mut restarted, 2, URI, None), Err(-32602));
}

#[test]
fn f8_broken_text_while_typing_keeps_the_server_running() {
    let mut server = LspServer::new();
    init_vscode(&mut server);
    open(&mut server, URI, "ddonilang", 1, GOOD);
    let steps = [
        "값 <- (",
        "값 <- (1",
        "값 <- (1 +",
        "값 <- (1 + 2",
        "값 <- \"닫히지 않은 글",
        "😀}}}{{{.",
        "\u{feff}",
        "",
        "값 <- (1 + 2).\n값 보여주기.\n",
    ];
    for (index, text) in steps.iter().enumerate() {
        let version = index as i64 + 2;
        let event = change(&mut server, URI, version, json!([{"text": text}]));
        assert!(event.exit_status.is_none());
        assert!(
            !methods(&event).contains(&"window/showMessage".to_string()),
            "{text:?}: {event:?}"
        );
        let result = pull(&mut server, version, URI, Some(version)).expect("pull");
        let (outcome, expected) = shared_check(text);
        assert_eq!(result["items"].as_array().unwrap().len(), expected.len(), "{text:?} {outcome}");
    }
    assert!(pulled_items(&mut server, 99, URI).is_empty());

    // Malformed notifications are reported, not fatal.
    let event = notify(&mut server, "textDocument/didChange", json!({"textDocument": {"uri": URI}}));
    assert_eq!(methods(&event)[0], "window/logMessage");
    assert_eq!(server.state(), ServerState::Running);
    let response = request(&mut server, 500, "textDocument/hover", json!({}));
    assert!(matches!(response, RpcMessage::Response { result: Err(ref error), .. } if error.code == -32602));
    let ignored = send(
        &mut server,
        RpcMessage::Response {
            id: json!(77),
            result: Ok(Value::Null),
        },
    );
    assert!(ignored.responses.is_empty() && ignored.exit_status.is_none());
}

/// An opening bracket at the end of text, the usual state while typing. The
/// shared parser rejects it with `E_PARSE`; the server shows that and lives.
#[test]
fn f10_open_bracket_at_end_is_an_ordinary_parse_error() {
    let mut server = LspServer::new();
    init_vscode(&mut server);
    open(&mut server, URI, "ddonilang", 1, GOOD);
    for (index, text) in ["x <- (", "값 <- (", "값 := (", "값 <- ((", "값 <- [", "값 <- {", "값 <- (\n"]
        .iter()
        .enumerate()
    {
        let version = index as i64 + 2;
        let event = change(&mut server, URI, version, json!([{"text": text}]));
        assert!(event.exit_status.is_none() && event.responses.is_empty(), "{text:?}: {event:?}");
        let (outcome, expected) = shared_check(text);
        assert_eq!(outcome, "REJECTED", "{text:?}");
        let items = pulled_items(&mut server, version, URI);
        assert_eq!(items.len(), expected.len(), "{text:?}");
        assert_eq!(items[0]["code"], "E_PARSE", "{text:?}");
    }
    assert_eq!(server.state(), ServerState::Running);
}

#[test]
fn f9_pull_client_gets_one_delivery_path() {
    let mut server = LspServer::new();
    let result = init_vscode(&mut server);
    let provider = &result["capabilities"]["diagnosticProvider"];
    assert_eq!(provider["workspaceDiagnostics"], false);
    assert_eq!(provider["interFileDependencies"], false);
    assert!(published(&open(&mut server, URI, "ddonilang", 1, BAD)).is_empty());
    assert!(published(&change(&mut server, URI, 2, json!([{"text": GOOD}]))).is_empty());

    let mut legacy = LspServer::new();
    init_with(&mut legacy, json!({}));
    assert_eq!(published(&open(&mut legacy, URI, "ddonilang", 1, BAD)).len(), 1);
}
