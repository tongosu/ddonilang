use ddonirang_tool::lsp::{
    byte_offset_to_position, encode_message, FrameDecoder, LspPosition, LspServer,
    PositionEncoding, RpcMessage, ServerEvent,
};
use serde_json::{json, Value};

const URI: &str = "file:///lsp-hover.ddn";
const LOCAL_SOURCE: &str = "첫함수:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 := { 첫함수. }";
const STDLIB_SOURCE: &str = "첫함수:글 := { (1) 글로. }";

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

fn hover(
    server: &mut LspServer,
    id: i64,
    source: &str,
    needle: &str,
    version: Option<i64>,
) -> Value {
    let byte = source.find(needle).unwrap();
    let position = byte_offset_to_position(source, byte, PositionEncoding::Utf16).unwrap();
    hover_at(server, id, position, version)
}

fn hover_at(server: &mut LspServer, id: i64, position: LspPosition, version: Option<i64>) -> Value {
    let mut document = json!({"uri": URI});
    if let Some(version) = version {
        document["version"] = json!(version);
    }
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: "textDocument/hover".into(),
            params: Some(json!({"textDocument": document, "position": {"line": position.line, "character": position.character}})),
        })
        .unwrap();
    let RpcMessage::Response {
        result: Ok(result), ..
    } = &event.responses[0]
    else {
        panic!("hover request must return a response: {event:?}");
    };
    result.clone()
}

#[test]
fn h01_h08_initialize_and_shared_local_stdlib_hover() {
    let mut server = LspServer::new();
    let initialized = init(&mut server);
    assert!(
        initialized.responses[0].clone_response().unwrap()["capabilities"]["hoverProvider"] == true
    );
    open(&mut server, LOCAL_SOURCE, 1);

    let local = hover(&mut server, 2, LOCAL_SOURCE, "첫함수", Some(1));
    assert_eq!(
        local["data"]["definitionIdentity"],
        "definition:local:첫함수"
    );
    assert_eq!(local["data"]["definitionKind"], "local_definition");
    assert!(local["contents"]["value"].as_str().unwrap().contains(":= "));
    assert!(local["contents"]["value"]
        .as_str()
        .unwrap()
        .contains("셈씨"));
    assert!(!local["contents"]["value"].as_str().unwrap().contains('<'));
    assert_eq!(local["range"]["start"]["character"], 0);

    let callable = hover(&mut server, 3, LOCAL_SOURCE, "첫함수.", Some(1));
    assert_eq!(
        callable["data"]["definitionIdentity"],
        "definition:local:첫함수"
    );

    close(&mut server);
    open(&mut server, STDLIB_SOURCE, 2);
    let stdlib = hover(&mut server, 4, STDLIB_SOURCE, "글로", Some(2));
    assert_eq!(
        stdlib["data"]["definitionIdentity"],
        "definition:stdlib:글로"
    );
    assert_eq!(stdlib["data"]["definitionKind"], "stdlib_callable");
    assert!(stdlib["contents"]["value"].as_str().unwrap().contains("->"));

    let whitespace = hover_at(
        &mut server,
        5,
        LspPosition {
            line: 0,
            character: 7,
        },
        Some(2),
    );
    assert_eq!(whitespace, Value::Null);
    let unknown = hover_at(
        &mut server,
        6,
        LspPosition {
            line: 0,
            character: 20,
        },
        Some(2),
    );
    assert_eq!(unknown, Value::Null);

    let emoji_source = "첫함수:글 := { (\"🙂\") 글로. }";
    close(&mut server);
    open(&mut server, emoji_source, 3);
    let emoji = hover(&mut server, 7, emoji_source, "글로", Some(3));
    assert_eq!(emoji["range"]["start"]["character"], 18);
    assert_eq!(emoji["data"]["documentVersion"], 3);
}

#[test]
fn h09_h11_live_snapshot_close_and_byte_stability() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, LOCAL_SOURCE, 1);
    change(&mut server, "새함수:셈씨 := { 1 돌려줘. }", 2);
    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(2),
            method: "textDocument/hover".into(),
            params: Some(json!({"textDocument": {"uri": URI, "version": 1}, "position": {"line": 0, "character": 0}})),
        })
        .unwrap();
    let fresh = hover(
        &mut server,
        3,
        "새함수:셈씨 := { 1 돌려줘. }",
        "새함수",
        Some(2),
    );
    assert_eq!(
        fresh["data"]["definitionIdentity"],
        "definition:local:새함수"
    );
    let repeated = hover(
        &mut server,
        4,
        "새함수:셈씨 := { 1 돌려줘. }",
        "새함수",
        Some(2),
    );
    assert_eq!(fresh, repeated);
    let RpcMessage::Response {
        result: Err(error), ..
    } = &stale.responses[0]
    else {
        panic!("stale hover must be rejected: {stale:?}");
    };
    assert_eq!(error.code, -32801);
    close(&mut server);
    assert_eq!(
        hover_at(
            &mut server,
            5,
            LspPosition {
                line: 0,
                character: 0
            },
            None
        ),
        Value::Null
    );
}

#[test]
fn h12_fragmented_initialize_and_hover_transport() {
    let messages = [
        RpcMessage::Request {
            id: json!(1),
            method: "initialize".into(),
            params: Some(json!({"capabilities": {"general": {"positionEncodings": ["utf-16"]}}})),
        },
        RpcMessage::Notification {
            method: "textDocument/didOpen".into(),
            params: Some(
                json!({"textDocument": {"uri": URI, "languageId": "ddn", "version": 1, "text": LOCAL_SOURCE}}),
            ),
        },
        RpcMessage::Request {
            id: json!(2),
            method: "textDocument/hover".into(),
            params: Some(
                json!({"textDocument": {"uri": URI}, "position": {"line": 0, "character": 0}}),
            ),
        },
    ];
    let mut bytes = Vec::new();
    for message in messages {
        bytes.extend(encode_message(&message).unwrap());
    }
    let mut decoder = FrameDecoder::default();
    let mut server = LspServer::new();
    let mut events = Vec::new();
    for chunk in bytes.chunks(2) {
        for message in decoder.push(chunk).unwrap() {
            events.push(server.handle_message(message).unwrap());
        }
    }
    assert!(
        events[0].responses[0].clone_response().unwrap()["capabilities"]["hoverProvider"] == true
    );
    assert_eq!(
        events[1].responses[0].method_name(),
        Some("textDocument/publishDiagnostics")
    );
    let RpcMessage::Response {
        result: Ok(result), ..
    } = &events[2].responses[0]
    else {
        panic!("fragmented hover response missing: {:?}", events[2]);
    };
    assert_eq!(
        result["data"]["definitionIdentity"],
        "definition:local:첫함수"
    );
}

trait RpcMessageTestExt {
    fn clone_response(&self) -> Option<Value>;
    fn method_name(&self) -> Option<&str>;
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

    fn method_name(&self) -> Option<&str> {
        match self {
            RpcMessage::Notification { method, .. } => Some(method),
            _ => None,
        }
    }
}
