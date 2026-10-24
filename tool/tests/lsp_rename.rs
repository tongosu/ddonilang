//! Rename is offered only where the shared name binding A1 resolves every
//! occurrence exactly; everything else is refused with the reason. Accepted
//! edits are applied and the program must run with the same output.
use ddonirang_tool::lsp::{byte_offset_to_position, LspServer, PositionEncoding, RpcMessage};
use ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile;
use serde_json::{json, Value};

const A: &str = "file:///C:/w/a.ddn";
const B: &str = "file:///C:/w/b.ddn";
const E1: &str = "(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\n결과 := 21을 두배하기. 결과 보여주기.\n";
const E2: &str = "(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\n(값:수~을) 세배:수:셈씨 := { 두배:수 := 값 * 3. 세배 <- 두배. }.\n결과 := 7을 세배하기. 결과 보여주기.\n";
const E3: &str = "(두배:수~을) 세배:수:셈씨 := { 세배 <- 두배 * 3. }.\n(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\n결과 := 7을 세배하기. 결과 보여주기.\n";

fn open_server(docs: &[(&str, &str)]) -> (LspServer, Value) {
    let mut server = LspServer::new();
    let init = server.handle_message_or_report(RpcMessage::Request {
        id: json!(1),
        method: "initialize".into(),
        params: Some(json!({"capabilities": {"general": {"positionEncodings": ["utf-16"]}}})),
    });
    let capabilities = match &init.responses[0] {
        RpcMessage::Response { result: Ok(value), .. } => value["capabilities"].clone(),
        other => panic!("initialize failed: {other:?}"),
    };
    for (uri, text) in docs {
        server.handle_message_or_report(RpcMessage::Notification {
            method: "textDocument/didOpen".into(),
            params: Some(json!({"textDocument": {"uri": uri, "languageId": "ddonilang", "version": 7, "text": text}})),
        });
    }
    (server, capabilities)
}

fn position(source: &str, needle: &str, nth: usize) -> Value {
    let byte = source.match_indices(needle).nth(nth).expect("needle").0;
    let p = byte_offset_to_position(source, byte, PositionEncoding::Utf16).unwrap();
    json!({"line": p.line, "character": p.character})
}

fn request(server: &mut LspServer, method: &str, params: Value) -> Result<Value, (i32, String)> {
    let event = server.handle_message_or_report(RpcMessage::Request {
        id: json!(9),
        method: method.into(),
        params: Some(params),
    });
    match &event.responses[0] {
        RpcMessage::Response { result: Ok(value), .. } => Ok(value.clone()),
        RpcMessage::Response { result: Err(error), .. } => Err((error.code, error.message.clone())),
        other => panic!("unexpected {other:?}"),
    }
}

fn rename(server: &mut LspServer, uri: &str, source: &str, needle: &str, nth: usize, new_name: &str) -> Result<Value, (i32, String)> {
    request(server, "textDocument/rename", json!({"textDocument": {"uri": uri}, "position": position(source, needle, nth), "newName": new_name}))
}

fn prepare(server: &mut LspServer, uri: &str, source: &str, needle: &str, nth: usize) -> Result<Value, (i32, String)> {
    request(server, "textDocument/prepareRename", json!({"textDocument": {"uri": uri}, "position": position(source, needle, nth)}))
}

/// Applies a single-document WorkspaceEdit given in UTF-16 positions.
fn apply(source: &str, edit: &Value, uri: &str) -> String {
    let changes = edit["documentChanges"].as_array().expect("documentChanges");
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0]["textDocument"]["uri"], uri);
    assert_eq!(changes[0]["textDocument"]["version"], 7);
    let offset = |p: &Value| {
        let line = p["line"].as_u64().unwrap() as usize;
        let character = p["character"].as_u64().unwrap() as usize;
        let start: usize = source.split('\n').take(line).map(|l| l.len() + 1).sum();
        let mut units = 0;
        let mut bytes = 0;
        for c in source[start..].chars() {
            if units >= character {
                break;
            }
            units += c.len_utf16();
            bytes += c.len_utf8();
        }
        start + bytes
    };
    let mut edits: Vec<(usize, usize, String)> = changes[0]["edits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| (offset(&e["range"]["start"]), offset(&e["range"]["end"]), e["newText"].as_str().unwrap().to_string()))
        .collect();
    edits.sort_by(|a, b| b.0.cmp(&a.0));
    let mut text = source.to_string();
    for (s, e, t) in edits {
        text.replace_range(s..e, &t);
    }
    text
}

fn output(source: &str) -> Value {
    run_summary_from_supported_source_profile(source, "a.ddn", Some(1), Some("v1-core-v25"))
        .unwrap_or_else(|error| panic!("{source}\n{error}"))["output_log_texts"]
        .clone()
}

fn refused(result: Result<Value, (i32, String)>, contains: &str) {
    match result {
        Err((code, message)) => {
            assert_eq!(code, -32803, "{message}");
            assert!(message.starts_with("이름을 바꿀 수 없습니다"), "{message}");
            assert!(message.contains(contains), "{message}");
        }
        Ok(value) => panic!("expected a refusal containing {contains:?}, got {value}"),
    }
}

#[test]
fn rename_capability_prepares_first() {
    let (_, capabilities) = open_server(&[]);
    assert_eq!(capabilities["renameProvider"], json!({"prepareProvider": true}));
}

#[test]
fn exact_callable_is_renamed_at_every_use_and_runs_the_same() {
    let (mut server, _) = open_server(&[(A, E1)]);
    // From the declaration, the result place and the call stem alike.
    for (needle, nth) in [("두배", 0), ("두배", 1), ("두배하기", 0)] {
        let prepared = prepare(&mut server, A, E1, needle, nth).unwrap();
        assert_eq!(prepared["placeholder"], "두배");
        let edit = rename(&mut server, A, E1, needle, nth, "곱절").unwrap();
        let after = apply(E1, &edit, A);
        assert_eq!(after, E1.replace("두배", "곱절"));
        assert_eq!(output(&after), output(E1));
    }
    // A callable without same-spelling bindings in a document that has others.
    let (mut server, _) = open_server(&[(A, E2)]);
    let after = apply(E2, &rename(&mut server, A, E2, "세배", 0, "세곱").unwrap(), A);
    assert_eq!(after, E2.replace("세배", "세곱"));
    assert_eq!(output(&after), json!(["21"]));
}

#[test]
fn hangul_emoji_and_crlf_positions_follow_utf16() {
    let source = "// 😀 한글 주석\r\n(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\r\n결과 := \"😀\".\r\n결과2 := 21을 두배하기. 결과2 보여주기.\r\n";
    let (mut server, _) = open_server(&[(A, source)]);
    let edit = rename(&mut server, A, source, "두배하기", 0, "곱절").unwrap();
    assert_eq!(apply(source, &edit, A), source.replace("두배", "곱절"));
}

#[test]
fn unresolved_cases_are_refused_with_reasons() {
    let cases: [(&str, &str, &str); 5] = [
        (E2, "두배", "같은 철자의 지역 이름이나 매개변수"),
        (E3, "두배:수:셈씨", "같은 철자의 지역 이름이나 매개변수"),
        ("합:셈씨 := { 1 돌려줘. }\n둘:셈씨 := { 합:수 := 2. 합 돌려줘. }\n", "합", "같은 철자의 지역 이름이나 매개변수"),
        ("첫함수:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 := { 합:수 := 2. 첫함수하기. 합 돌려줘. }\n", "첫함수", "인자 없는 `…하기` 문장 호출"),
        ("첫함수:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 := { \"첫함수\" 보여주기. 첫함수하기. }\n", "첫함수", "인자 없는 `…하기` 문장 호출"),
    ];
    for (source, needle, reason) in cases {
        let (mut server, _) = open_server(&[(A, source)]);
        refused(prepare(&mut server, A, source, needle, 0), reason);
        refused(rename(&mut server, A, source, needle, 0, "새이름"), reason);
        assert_eq!(server.documents().get(A).unwrap().text().unwrap(), source);
    }
    // Not a top-level callable name (a parameter or a root value).
    let (mut server, _) = open_server(&[(A, E1)]);
    refused(rename(&mut server, A, E1, "결과", 0, "새결과"), "최상위 함수 이름만");
    refused(rename(&mut server, A, E1, "값", 0, "새값"), "최상위 함수 이름만");
}

#[test]
fn new_names_that_could_capture_or_break_are_refused() {
    let (mut server, _) = open_server(&[(A, E2)]);
    // `두배` already occurs: renaming 세배 to it could capture or be captured.
    refused(rename(&mut server, A, E2, "세배", 0, "두배"), "이미 있습니다");
    refused(rename(&mut server, A, E2, "세배", 0, "1잘못"), "이름 하나로 읽히지 않습니다");
    refused(rename(&mut server, A, E2, "세배", 0, "입력"), "금지어");
    refused(rename(&mut server, A, E2, "세배", 0, "세배"), "같습니다");
}

#[test]
fn other_open_documents_closed_and_stale_requests() {
    // Another open document uses the spelling: no multi-file binding yet.
    let (mut server, _) = open_server(&[(A, E1), (B, "둘함수:셈씨 := { 두배하기. }\n")]);
    refused(rename(&mut server, A, E1, "두배", 0, "곱절"), "다른 열린 문서");
    // Not open.
    let (mut server, _) = open_server(&[]);
    refused(rename(&mut server, A, E1, "두배", 0, "곱절"), "열려 있는 문서가 아닙니다");
    // Text the shared check rejects.
    let broken = "(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\n결과 := (";
    let (mut server, _) = open_server(&[(A, broken)]);
    refused(rename(&mut server, A, broken, "두배", 0, "곱절"), "검사를 통과한 원문에서만");
    // Stale version.
    let (mut server, _) = open_server(&[(A, E1)]);
    let stale = request(&mut server, "textDocument/rename", json!({"textDocument": {"uri": A, "version": 6}, "position": position(E1, "두배", 0), "newName": "곱절"}));
    assert_eq!(stale.unwrap_err().0, -32801);
}
