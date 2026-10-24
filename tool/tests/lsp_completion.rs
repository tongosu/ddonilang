use ddonirang_tool::completion::{
    build_structured_completion_json, complete_snapshot, CompletionCandidateKind,
};
use ddonirang_tool::lsp::{
    byte_offset_to_position, encode_message, DocumentStore, LspPosition, LspServer,
    PositionEncoding, RpcError, RpcMessage, ServerState,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::process::{Command, Stdio};

const LOCAL_SOURCE: &str = "첫함수:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 := { 첫함수. }";
const STDLIB_SOURCE: &str = "첫함수:글 := { (1) 글로. }";

fn snapshot(source: &str, version: i64) -> ddonirang_tool::lsp::DocumentSnapshot {
    let mut store = DocumentStore::new();
    store
        .open(
            "file:///lsp-completion.ddn",
            "ddn",
            version,
            source.as_bytes().to_vec(),
        )
        .unwrap();
    store.get("file:///lsp-completion.ddn").unwrap().clone()
}
fn position_at(source: &str, needle: &str, character_offset: usize) -> LspPosition {
    let byte = source.find(needle).unwrap() + character_offset;
    byte_offset_to_position(source, byte, PositionEncoding::Utf16).unwrap()
}

fn init(server: &mut LspServer) {
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(1),
            method: "initialize".into(),
            params: Some(json!({
                "capabilities": {"general": {"positionEncodings": ["utf-16", "utf-8"]}}
            })),
        })
        .unwrap();
    assert!(matches!(
        event.responses[0],
        RpcMessage::Response { result: Ok(_), .. }
    ));
}

fn open(server: &mut LspServer, source: &str, version: i64) {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didOpen".into(),
            params: Some(json!({
                "textDocument": {
                    "uri": "file:///lsp-completion.ddn",
                    "languageId": "ddn",
                    "version": version,
                    "text": source
                }
            })),
        })
        .unwrap();
}

fn completion_result(server: &mut LspServer, id: i64, position: LspPosition) -> Value {
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: "textDocument/completion".into(),
            params: Some(json!({
                "textDocument": {"uri": "file:///lsp-completion.ddn"},
                "position": {"line": position.line, "character": position.character}
            })),
        })
        .unwrap();
    let RpcMessage::Response {
        result: Ok(result), ..
    } = &event.responses[0]
    else {
        panic!("completion must succeed: {event:?}");
    };
    result.clone()
}

#[test]
fn c_a01_local_definition_prefix_completion() {
    let document = snapshot(LOCAL_SOURCE, 1);
    let result = complete_snapshot(
        &document,
        position_at(LOCAL_SOURCE, "첫함수.", "첫".len()),
        PositionEncoding::Utf16,
    )
    .unwrap();
    assert_eq!(result.candidates[0].label, "첫함수");
    assert_eq!(
        result.candidates[0].candidate_kind,
        CompletionCandidateKind::LocalDefinition
    );
}

#[test]
fn c_a02_registered_stdlib_callable_prefix_completion() {
    let document = snapshot(STDLIB_SOURCE, 1);
    let result = complete_snapshot(
        &document,
        position_at(STDLIB_SOURCE, "글로.", "글".len()),
        PositionEncoding::Utf16,
    )
    .unwrap();
    assert_eq!(result.candidates[0].label, "글로");
    assert_eq!(
        result.candidates[0].candidate_kind,
        CompletionCandidateKind::StdlibCallable
    );
}

#[test]
fn c_b01_structured_wasm_projection_matches_lsp_candidates() {
    let position = position_at(LOCAL_SOURCE, "첫함수.", "첫".len());
    let document = snapshot(LOCAL_SOURCE, 1);
    let native = complete_snapshot(&document, position, PositionEncoding::Utf16).unwrap();
    let structured: Value = serde_json::from_str(
        &build_structured_completion_json(
            &json!({
                "schema": "ddonilang.completion.v1-candidate",
                "profile": "v1-core-v25",
                "document_identity": "file:///lsp-completion.ddn",
                "document_version": 1,
                "source": LOCAL_SOURCE,
                "position": {"line": position.line, "character": position.character},
                "encoding": "utf-16",
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    let candidates = structured["candidates"].as_array().unwrap();
    assert_eq!(candidates.len(), native.candidates.len());
    for (candidate, expected) in candidates.iter().zip(native.candidates.iter()) {
        assert_eq!(candidate["label"], expected.label);
        assert_eq!(candidate["insert_text"], expected.insert_text);
        assert_eq!(candidate["definition_identity"], expected.definition_identity);
        assert_eq!(candidate["canonical_detail"], expected.canonical_detail);
        let expected_range = ddonirang_tool::lsp::byte_range_to_lsp_range(
            LOCAL_SOURCE,
            expected.replacement_range,
            PositionEncoding::Utf16,
        )
        .unwrap();
        assert_eq!(candidate["replacement_range"]["start"]["line"], expected_range.start.line);
        assert_eq!(candidate["replacement_range"]["start"]["character"], expected_range.start.character);
        assert_eq!(candidate["replacement_range"]["end"]["line"], expected_range.end.line);
        assert_eq!(candidate["replacement_range"]["end"]["character"], expected_range.end.character);
    }
}

#[test]
fn c_a03_korean_utf16_cursor_uses_exact_byte_range() {
    let source = LOCAL_SOURCE;
    let document = snapshot(source, 1);
    let position = position_at(source, "첫함수.", "첫".len());
    let result = complete_snapshot(&document, position, PositionEncoding::Utf16).unwrap();
    let start = source.rfind("첫함수").unwrap();
    assert_eq!(result.candidates[0].replacement_range.start, start);
    assert_eq!(
        result.candidates[0].replacement_range.end,
        start + "첫".len()
    );
}

#[test]
fn c_a04_emoji_position_conversion_preserves_utf16_units() {
    let source = "🙂한";
    assert_eq!(
        byte_offset_to_position(source, "🙂".len(), PositionEncoding::Utf16),
        Ok(LspPosition {
            line: 0,
            character: 2
        })
    );
}

#[test]
fn c_a05_lsp_replacement_range_is_exact() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, LOCAL_SOURCE, 1);
    let result = completion_result(
        &mut server,
        2,
        position_at(LOCAL_SOURCE, "첫함수.", "첫".len()),
    );
    let item = &result["items"][0];
    assert_eq!(item["textEdit"]["newText"], "첫함수");
    assert_eq!(item["textEdit"]["range"]["start"]["line"], 1);
    assert_eq!(
        item["data"]["definitionIdentity"],
        "definition:local:첫함수"
    );
}

#[test]
fn c_a06_string_and_comment_contexts_are_empty() {
    let string_source = "첫함수:셈씨 := { \"나\" 보여주기. }";
    let string_document = snapshot(string_source, 1);
    let string_position = position_at(string_source, "나", "나".len());
    assert!(
        complete_snapshot(&string_document, string_position, PositionEncoding::Utf16)
            .unwrap()
            .candidates
            .is_empty()
    );

    let comment_source = "첫함수:셈씨 := { 1 돌려줘. }\n// 나";
    let comment_document = snapshot(comment_source, 1);
    let comment_position = position_at(comment_source, "나", "나".len());
    assert!(
        complete_snapshot(&comment_document, comment_position, PositionEncoding::Utf16)
            .unwrap()
            .candidates
            .is_empty()
    );
}

#[test]
fn c_a07_unknown_and_legacy_source_fail_closed() {
    for source in [
        "첫함수:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 = { 2 돌려줘. }",
        "첫함수:셈씨 := { 보임 { 1 돌려줘. }. }",
    ] {
        let document = snapshot(source, 1);
        let position =
            byte_offset_to_position(source, source.len(), PositionEncoding::Utf16).unwrap();
        assert!(
            complete_snapshot(&document, position, PositionEncoding::Utf16)
                .unwrap()
                .candidates
                .is_empty()
        );
    }
}

#[test]
fn c_a08_stale_document_version_is_rejected() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, LOCAL_SOURCE, 2);
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(8),
            method: "textDocument/completion".into(),
            params: Some(json!({
                "textDocument": {"uri": "file:///lsp-completion.ddn", "version": 1},
                "position": {"line": 1, "character": 13}
            })),
        })
        .unwrap();
    assert!(matches!(
        &event.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32801
    ));
}

#[test]
fn c_a09_candidate_identities_are_unique() {
    let document = snapshot(LOCAL_SOURCE, 1);
    let result = complete_snapshot(
        &document,
        position_at(LOCAL_SOURCE, "첫함수.", 0),
        PositionEncoding::Utf16,
    )
    .unwrap();
    let identities = result
        .candidates
        .iter()
        .map(|candidate| candidate.definition_identity.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(identities.len(), result.candidates.len());
}

#[test]
fn c_a10_repeat_is_byte_stable() {
    let document = snapshot(LOCAL_SOURCE, 1);
    let position = position_at(LOCAL_SOURCE, "첫함수.", 0);
    let first = complete_snapshot(&document, position, PositionEncoding::Utf16).unwrap();
    let second = complete_snapshot(&document, position, PositionEncoding::Utf16).unwrap();
    assert_eq!(first, second);
    let mut expected = first.candidates.clone();
    expected.sort_by(|left, right| {
        left.candidate_kind
            .cmp(&right.candidate_kind)
            .then_with(|| left.label.as_bytes().cmp(right.label.as_bytes()))
            .then_with(|| {
                left.definition_identity
                    .as_bytes()
                    .cmp(right.definition_identity.as_bytes())
            })
    });
    assert_eq!(first.candidates, expected);
}

#[test]
fn c_a11_initialize_is_required_before_completion() {
    let mut server = LspServer::new();
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(11),
            method: "textDocument/completion".into(),
            params: Some(json!({
                "textDocument": {"uri": "file:///lsp-completion.ddn"},
                "position": {"line": 0, "character": 0}
            })),
        })
        .unwrap();
    assert!(matches!(
        event.responses[0],
        RpcMessage::Response {
            result: Err(RpcError { code: -32002, .. }),
            ..
        }
    ));
}

#[test]
fn c_a12_shutdown_and_cancel_publish_no_success() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, LOCAL_SOURCE, 1);
    server.cancel_request(&json!(12));
    let cancelled = server
        .handle_message(RpcMessage::Request {
            id: json!(12),
            method: "textDocument/completion".into(),
            params: Some(json!({
                "textDocument": {"uri": "file:///lsp-completion.ddn"},
                "position": {"line": 1, "character": 13}
            })),
        })
        .unwrap();
    assert!(matches!(
        cancelled.responses[0],
        RpcMessage::Response {
            result: Err(RpcError { code: -32800, .. }),
            ..
        }
    ));
    server
        .handle_message(RpcMessage::Request {
            id: json!(13),
            method: "shutdown".into(),
            params: None,
        })
        .unwrap();
    assert_eq!(server.state(), ServerState::ShutdownRequested);
    let shutdown = server
        .handle_message(RpcMessage::Request {
            id: json!(14),
            method: "textDocument/completion".into(),
            params: Some(json!({
                "textDocument": {"uri": "file:///lsp-completion.ddn"},
                "position": {"line": 1, "character": 13}
            })),
        })
        .unwrap();
    assert!(matches!(
        shutdown.responses[0],
        RpcMessage::Response { result: Err(_), .. }
    ));
}

fn send_frame(stdin: &mut impl Write, message: RpcMessage) {
    stdin.write_all(&encode_message(&message).unwrap()).unwrap();
    stdin.flush().unwrap();
}

fn read_frame(stdout: &mut impl Read) -> Value {
    let mut header = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        stdout.read_exact(&mut byte).unwrap();
        header.push(byte[0]);
        if header.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let header = String::from_utf8(header).unwrap();
    let length = header
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let mut body = vec![0_u8; length];
    stdout.read_exact(&mut body).unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[test]
fn c_a13_child_process_returns_completion_before_eof() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ddonirang-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    send_frame(
        &mut stdin,
        RpcMessage::Request {
            id: json!(1),
            method: "initialize".into(),
            params: Some(json!({"capabilities": {"general": {"positionEncodings": ["utf-16"]}}})),
        },
    );
    assert!(read_frame(&mut stdout)["result"]["capabilities"]["completionProvider"].is_object());
    send_frame(
        &mut stdin,
        RpcMessage::Notification {
            method: "textDocument/didOpen".into(),
            params: Some(json!({"textDocument": {
                "uri": "file:///lsp-completion.ddn",
                "languageId": "ddn",
                "version": 1,
                "text": LOCAL_SOURCE
            }})),
        },
    );
    let diagnostics = read_frame(&mut stdout);
    assert_eq!(diagnostics["method"], "textDocument/publishDiagnostics");
    // The fixture is accepted by the shared check; it may carry its warnings.
    assert!(diagnostics["params"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["severity"] != 1));
    send_frame(
        &mut stdin,
        RpcMessage::Request {
            id: json!(2),
            method: "textDocument/completion".into(),
            params: Some(
                json!({"textDocument": {"uri": "file:///lsp-completion.ddn"},
                "position": {"line": 1, "character": 13}}),
            ),
        },
    );
    let response = read_frame(&mut stdout);
    assert_eq!(response["id"], 2);
    assert!(!response["result"]["items"].as_array().unwrap().is_empty());
    assert!(
        child.try_wait().unwrap().is_none(),
        "child must remain alive before EOF"
    );
    drop(stdin);
    let _ = child.wait();
}

#[test]
fn c_a14_existing_transport_surface_stays_available() {
    let mut decoder = ddonirang_tool::lsp::FrameDecoder::default();
    let frame = encode_message(&RpcMessage::Request {
        id: json!(14),
        method: "textDocument/completion".into(),
        params: Some(
            json!({"textDocument": {"uri": "file:///lsp-completion.ddn"},
            "position": {"line": 0, "character": 0}}),
        ),
    })
    .unwrap();
    assert!(decoder.push(&frame[..5]).unwrap().is_empty());
    assert_eq!(decoder.push(&frame[5..]).unwrap().len(), 1);
    assert!(decoder.finish().is_ok());
}
