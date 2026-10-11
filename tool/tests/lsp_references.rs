use ddonirang_tool::lsp::{
    byte_offset_to_position, LspServer, PositionEncoding, RpcMessage, ServerEvent,
};
use serde_json::{json, Value};

const URI: &str = "file:///lsp-references.ddn";
const SOURCE: &str = "(문자:글) 첫함수:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 := { (\"🙂\") 첫함수. }\n셋함수:셈씨 := { 첫함수. }";

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
                "textDocument": {"uri": URI, "languageId": "ddn", "version": version, "text": source}
            })),
        })
        .unwrap()
}

fn change(server: &mut LspServer, source: &str, version: i64) -> ServerEvent {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didChange".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": version},
                "contentChanges": [{"text": source}]
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

fn occurrence(source: &str, needle: &str, ordinal: usize) -> usize {
    source
        .match_indices(needle)
        .nth(ordinal)
        .map(|(byte, _)| byte)
        .unwrap()
}

fn position(source: &str, byte: usize) -> Value {
    let position = byte_offset_to_position(source, byte, PositionEncoding::Utf16).unwrap();
    json!({"line": position.line, "character": position.character})
}

fn location(source: &str, byte: usize, name: &str) -> Value {
    let start = position(source, byte);
    let end = position(source, byte + name.len());
    json!({"uri": URI, "range": {"start": start, "end": end}})
}

fn references(
    server: &mut LspServer,
    id: i64,
    source: &str,
    needle: &str,
    ordinal: usize,
    version: Option<i64>,
    include_declaration: bool,
) -> Value {
    let byte = occurrence(source, needle, ordinal);
    let mut document = json!({"uri": URI});
    if let Some(version) = version {
        document["version"] = json!(version);
    }
    let position = position(source, byte);
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: "textDocument/references".into(),
            params: Some(json!({
                "textDocument": document,
                "position": position,
                "context": {"includeDeclaration": include_declaration}
            })),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result.clone(),
        _ => panic!("references request must return a success response: {event:?}"),
    }
}

#[test]
fn i01_identity_order_utf16_and_declaration_context() {
    let mut server = LspServer::new();
    let initialized = init(&mut server);
    assert_eq!(
        initialized.responses[0].clone_response().unwrap()["capabilities"]
            ["referencesProvider"],
        true
    );
    open(&mut server, SOURCE, 1);

    let without_declaration = references(&mut server, 2, SOURCE, "첫함수", 1, Some(1), false);
    assert_eq!(
        without_declaration,
        json!([
            location(SOURCE, occurrence(SOURCE, "첫함수", 1), "첫함수"),
            location(SOURCE, occurrence(SOURCE, "첫함수", 2), "첫함수"),
        ])
    );
    let with_declaration = references(&mut server, 3, SOURCE, "첫함수", 1, Some(1), true);
    assert_eq!(
        with_declaration,
        json!([
            location(SOURCE, occurrence(SOURCE, "첫함수", 0), "첫함수"),
            location(SOURCE, occurrence(SOURCE, "첫함수", 1), "첫함수"),
            location(SOURCE, occurrence(SOURCE, "첫함수", 2), "첫함수"),
        ])
    );
    assert_eq!(
        references(&mut server, 4, SOURCE, "첫함수", 2, Some(1), true),
        with_declaration
    );
}

#[test]
fn i02_change_stale_malformed_and_close_empty() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, SOURCE, 1);
    let changed = "새함수:셈씨 := { 1 돌려줘. }\n다른함수:셈씨 := { 새함수. }";
    change(&mut server, changed, 2);

    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(2),
            method: "textDocument/references".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": 1},
                "position": {"line": 0, "character": 0},
                "context": {"includeDeclaration": true}
            })),
        })
        .unwrap();
    assert!(matches!(
        &stale.responses[0],
        RpcMessage::Response {
            result: Err(error), ..
        } if error.code == -32801
    ));

    let changed_reference = occurrence(changed, "새함수", 1);
    assert_eq!(
        references(
            &mut server,
            3,
            changed,
            "새함수",
            1,
            Some(2),
            true,
        ),
        json!([
            location(changed, occurrence(changed, "새함수", 0), "새함수"),
            location(changed, changed_reference, "새함수"),
        ])
    );

    let legacy = "새함수:셈씨 = { 1 돌려줘. }";
    change(&mut server, legacy, 3);
    assert_eq!(
        references(&mut server, 4, legacy, "새함수", 0, Some(3), true),
        json!([])
    );
    close(&mut server);
    assert_eq!(
        server
            .handle_message(RpcMessage::Request {
                id: json!(5),
                method: "textDocument/references".into(),
                params: Some(json!({
                    "textDocument": {"uri": URI},
                    "position": {"line": 0, "character": 0},
                    "context": {"includeDeclaration": true}
                })),
            })
            .unwrap()
            .responses[0]
            .clone_response()
            .unwrap(),
        json!([])
    );
}

trait RpcMessageTestExt {
    fn clone_response(&self) -> Option<Value>;
}

impl RpcMessageTestExt for RpcMessage {
    fn clone_response(&self) -> Option<Value> {
        match self {
            RpcMessage::Response {
                result: Ok(value), ..
            } => Some(value.clone()),
            _ => None,
        }
    }
}
