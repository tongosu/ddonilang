use ddonirang_tool::lsp::{
    byte_offset_to_position, LspServer, PositionEncoding, RpcMessage, ServerEvent,
};
use serde_json::{json, Value};

const URI: &str = "file:///lsp-signature-help.ddn";
const LOCAL_SOURCE: &str =
    "(왼:수, 오른:수) 더하:셈씨 := { 0 돌려줘. }\n호출:셈씨 := { (1, 2) 더하. }";

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

fn open(server: &mut LspServer, source: &str, version: i64) {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didOpen".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "languageId": "ddn", "version": version, "text": source}
            })),
        })
        .unwrap();
}

fn change(server: &mut LspServer, source: &str, version: i64) {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didChange".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": version},
                "contentChanges": [{"text": source}]
            })),
        })
        .unwrap();
}

fn close(server: &mut LspServer) {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didClose".into(),
            params: Some(json!({"textDocument": {"uri": URI}})),
        })
        .unwrap();
}

fn signature_at(
    server: &mut LspServer,
    id: i64,
    source: &str,
    needle: &str,
    ordinal: usize,
    version: Option<i64>,
) -> Value {
    let byte = source.match_indices(needle).nth(ordinal).unwrap().0;
    let position = byte_offset_to_position(source, byte, PositionEncoding::Utf16).unwrap();
    let mut document = json!({"uri": URI});
    if let Some(version) = version {
        document["version"] = json!(version);
    }
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: "textDocument/signatureHelp".into(),
            params: Some(json!({
                "textDocument": document,
                "position": {"line": position.line, "character": position.character}
            })),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result.clone(),
        _ => panic!("signature help request must return a success response: {event:?}"),
    }
}

fn inlay_hints(server: &mut LspServer, id: i64, version: Option<i64>) -> Value {
    let mut document = json!({"uri": URI});
    if let Some(version) = version {
        document["version"] = json!(version);
    }
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: "textDocument/inlayHint".into(),
            params: Some(json!({"textDocument": document})),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result.clone(),
        _ => panic!("inlay hint request must return a success response: {event:?}"),
    }
}

#[test]
fn sh01_sh09_local_param_order_identity_and_utf16() {
    let mut server = LspServer::new();
    let initialized = init(&mut server);
    let capabilities = match &initialized.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => &result["capabilities"],
        _ => panic!("initialize must return a response: {initialized:?}"),
    };
    assert_eq!(
        capabilities["signatureHelpProvider"]["triggerCharacters"],
        json!(["(", ","])
    );
    open(&mut server, LOCAL_SOURCE, 1);

    let first = signature_at(&mut server, 2, LOCAL_SOURCE, "1", 0, Some(1));
    assert_eq!(first["activeParameter"], 0);
    assert_eq!(first["data"]["definitionIdentity"], "definition:local:더하");
    assert_eq!(first["signatures"][0]["label"], "더하(왼:수, 오른:수):셈씨 := …");
    let parameters = first["signatures"][0]["parameters"].as_array().unwrap();
    assert_eq!(parameters[0]["label"], "왼:수");
    assert_eq!(parameters[1]["label"], "오른:수");
    assert!(parameters[0]["data"]["paramPinId"].is_u64());
    assert!(parameters[1]["data"]["paramPinId"].is_u64());

    let second = signature_at(&mut server, 3, LOCAL_SOURCE, "2", 0, Some(1));
    assert_eq!(second["activeParameter"], 1);
    assert_eq!(second, signature_at(&mut server, 4, LOCAL_SOURCE, "2", 0, Some(1)));

    let emoji_source =
        "(왼:글, 오른:수) 더하:셈씨 := { 0 돌려줘. }\n호출:셈씨 := { (\"🙂\", 2) 더하. }";
    change(&mut server, emoji_source, 2);
    let emoji = signature_at(&mut server, 5, emoji_source, "2", 0, Some(2));
    assert_eq!(emoji["activeParameter"], 1);
    assert_eq!(emoji["data"]["documentVersion"], 2);
}

#[test]
fn sh03_stdlib_nested_and_fail_closed_positions() {
    let mut server = LspServer::new();
    init(&mut server);
    let source = "호출:셈씨 := { (\"가\", \"나\") 합치기. }";
    open(&mut server, source, 1);
    let stdlib = signature_at(&mut server, 2, source, "\"나\"", 0, Some(1));
    assert_eq!(stdlib["activeParameter"], 1);
    assert_eq!(stdlib["data"]["definitionIdentity"], "definition:stdlib:합치기");
    assert_eq!(stdlib["signatures"][0]["parameters"], json!([{"label": "앞"}, {"label": "뒤"}]));

    let nested = "(왼:수, 오른:수) 더하:셈씨 := { 0 돌려줘. }\n호출:셈씨 := { ((1, 2) 더하, 3) 더하. }";
    change(&mut server, nested, 2);
    let inner = signature_at(&mut server, 3, nested, "2", 0, Some(2));
    assert_eq!(inner["activeParameter"], 1);

    assert_eq!(signature_at(&mut server, 4, nested, "호출", 0, Some(2)), Value::Null);
    assert_eq!(signature_at(&mut server, 5, nested, " 3", 0, Some(2)), Value::Null);
    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(6),
            method: "textDocument/signatureHelp".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": 1},
                "position": {"line": 1, "character": 4}
            })),
        })
        .unwrap();
    assert!(matches!(
        &stale.responses[0],
        RpcMessage::Response {
            result: Err(error), ..
        } if error.code == -32801
    ));
    close(&mut server);
    assert_eq!(signature_at(&mut server, 7, nested, "2", 0, None), Value::Null);
}

#[test]
fn lsp18_inlay_hints_reuse_param_identity_and_live_snapshot() {
    let mut server = LspServer::new();
    let initialized = init(&mut server);
    let capabilities = match &initialized.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => &result["capabilities"],
        _ => panic!("initialize must return a response: {initialized:?}"),
    };
    assert_eq!(capabilities["inlayHintProvider"], true);
    open(&mut server, LOCAL_SOURCE, 1);

    let hints = inlay_hints(&mut server, 8, Some(1));
    let hints = hints.as_array().unwrap();
    assert_eq!(hints.len(), 2);
    assert_eq!(hints[0]["label"], "왼:");
    assert_eq!(hints[1]["label"], "오른:");
    assert_eq!(
        hints[0]["data"]["definitionIdentity"],
        "definition:local:더하"
    );
    assert!(hints[0]["data"]["paramPinId"].is_u64());
    let first_position = byte_offset_to_position(
        LOCAL_SOURCE,
        LOCAL_SOURCE.rfind('1').unwrap(),
        PositionEncoding::Utf16,
    )
    .unwrap();
    let second_position = byte_offset_to_position(
        LOCAL_SOURCE,
        LOCAL_SOURCE.rfind('2').unwrap(),
        PositionEncoding::Utf16,
    )
    .unwrap();
    assert_eq!(
        hints[0]["position"],
        json!({"line": first_position.line, "character": first_position.character})
    );
    assert_eq!(
        hints[1]["position"],
        json!({"line": second_position.line, "character": second_position.character})
    );
    assert_eq!(
        inlay_hints(&mut server, 9, Some(1)),
        Value::Array(hints.to_vec())
    );

    let named =
        "(왼:수, 오른:수) 더하:셈씨 := { 0 돌려줘. }\n호출:셈씨 := { (왼=1, 오른=2) 더하. }";
    change(&mut server, named, 2);
    assert_eq!(inlay_hints(&mut server, 10, Some(2)), json!([]));

    let nested =
        "(왼:수, 오른:수) 더하:셈씨 := { 0 돌려줘. }\n호출:셈씨 := { ((1, 2) 더하, 3) 더하. }";
    change(&mut server, nested, 3);
    let nested_hints = inlay_hints(&mut server, 11, Some(3));
    assert_eq!(nested_hints.as_array().unwrap().len(), 4);
    assert!(nested_hints[0]["position"]["character"].is_u64());

    let string_source = "호출:셈씨 := { \"더하\" 보여주기. }\n// (1, 2) 더하.";
    change(&mut server, string_source, 4);
    assert_eq!(inlay_hints(&mut server, 12, Some(4)), json!([]));

    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(13),
            method: "textDocument/inlayHint".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": 3},
            })),
        })
        .unwrap();
    assert!(matches!(
        &stale.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32801
    ));
    let stale_snapshot = server
        .handle_message(RpcMessage::Request {
            id: json!(14),
            method: "textDocument/inlayHint".into(),
            params: Some(json!({
                "textDocument": {
                    "uri": URI,
                    "version": 4,
                    "snapshotIdentity": "snapshot:forged"
                },
            })),
        })
        .unwrap();
    assert!(matches!(
        &stale_snapshot.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32802
    ));
    close(&mut server);
    assert_eq!(inlay_hints(&mut server, 15, None), json!([]));
}
