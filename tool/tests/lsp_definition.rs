use ddonirang_tool::lsp::{
    byte_offset_to_position, encode_message, FrameDecoder, LspPosition, LspServer,
    PositionEncoding, RpcMessage, ServerEvent,
};
use serde_json::{json, Value};

const URI: &str = "file:///lsp-definition.ddn";
const SOURCE: &str = "첫함수:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 := { 첫함수. }";
const EMOJI_SOURCE: &str = r#"(문자:글) 첫함수:셈씨 := { 1 돌려줘. }
둘함수:셈씨 := { ("🙂") 첫함수. }"#;

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

fn close(server: &mut LspServer) -> ServerEvent {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didClose".into(),
            params: Some(json!({"textDocument": {"uri": URI}})),
        })
        .unwrap()
}

fn definition_at(
    server: &mut LspServer,
    id: i64,
    source: &str,
    needle: &str,
    version: Option<i64>,
) -> Value {
    let byte = source.find(needle).unwrap();
    let position = byte_offset_to_position(source, byte, PositionEncoding::Utf16).unwrap();
    let mut document = json!({"uri": URI});
    if let Some(version) = version {
        document["version"] = json!(version);
    }
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: "textDocument/definition".into(),
            params: Some(json!({
                "textDocument": document,
                "position": {"line": position.line, "character": position.character}
            })),
        })
        .unwrap();
    let RpcMessage::Response {
        result: Ok(result), ..
    } = &event.responses[0]
    else {
        panic!("definition request must return a response: {event:?}");
    };
    result.clone()
}

fn definition_at_position(
    server: &mut LspServer,
    id: i64,
    position: LspPosition,
    version: Option<i64>,
) -> Value {
    let mut document = json!({"uri": URI});
    if let Some(version) = version {
        document["version"] = json!(version);
    }
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: "textDocument/definition".into(),
            params: Some(json!({
                "textDocument": document,
                "position": {"line": position.line, "character": position.character}
            })),
        })
        .unwrap();
    let RpcMessage::Response {
        result: Ok(result), ..
    } = &event.responses[0]
    else {
        panic!("definition request must return a response: {event:?}");
    };
    result.clone()
}

#[test]
fn g01_g08_local_definition_identity_and_utf16_range() {
    let mut server = LspServer::new();
    let initialized = init(&mut server);
    assert_eq!(
        initialized.responses[0].clone_response().unwrap()["capabilities"]["definitionProvider"],
        true
    );
    open(&mut server, SOURCE, 1);

    let declaration = definition_at(&mut server, 2, SOURCE, "첫함수", Some(1));
    assert_eq!(declaration["uri"], URI);
    assert_eq!(
        declaration["range"]["start"],
        json!({"line": 0, "character": 0})
    );
    assert_eq!(
        declaration["range"]["end"],
        json!({"line": 0, "character": 3})
    );

    let callable = definition_at(&mut server, 3, SOURCE, "첫함수.", Some(1));
    assert_eq!(callable["uri"], URI);
    assert_eq!(callable["range"], declaration["range"]);

    close(&mut server);
    open(&mut server, EMOJI_SOURCE, 2);
    let emoji = definition_at(&mut server, 4, EMOJI_SOURCE, "첫함수.", Some(2));
    assert_eq!(emoji["uri"], URI);
    assert_eq!(
        emoji["range"],
        json!({
            "start": {"line": 0, "character": 7},
            "end": {"line": 0, "character": 10}
        })
    );
}

#[test]
fn g06_g11_null_stale_close_and_stable() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, SOURCE, 1);

    close(&mut server);
    let stdlib_source = "첫함수:글 := { (1) 글로. }";
    open(&mut server, stdlib_source, 2);
    let stdlib = definition_at(&mut server, 2, stdlib_source, "글로", Some(2));
    assert_eq!(stdlib, Value::Null);
    close(&mut server);
    open(&mut server, SOURCE, 3);
    let whitespace = definition_at_position(
        &mut server,
        3,
        LspPosition {
            line: 0,
            character: 7,
        },
        Some(3),
    );
    assert_eq!(whitespace, Value::Null);
    let punctuation = definition_at_position(
        &mut server,
        4,
        LspPosition {
            line: 0,
            character: 8,
        },
        Some(3),
    );
    assert_eq!(punctuation, Value::Null);
    let unknown = definition_at_position(
        &mut server,
        5,
        LspPosition {
            line: 0,
            character: 14,
        },
        Some(3),
    );
    assert_eq!(unknown, Value::Null);

    let first = definition_at(&mut server, 6, SOURCE, "첫함수.", Some(3));
    let second = definition_at(&mut server, 7, SOURCE, "첫함수.", Some(3));
    assert_eq!(first, second);

    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(8),
            method: "textDocument/definition".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": 2},
                "position": {"line": 0, "character": 0}
            })),
        })
        .unwrap();
    assert!(matches!(
        &stale.responses[0],
        RpcMessage::Response {
            result: Err(error),
            ..
        } if error.code == -32801
    ));

    close(&mut server);
    let closed = definition_at_position(
        &mut server,
        9,
        LspPosition {
            line: 0,
            character: 0,
        },
        None,
    );
    assert_eq!(closed, Value::Null);
}

#[test]
fn g12_fragmented_initialize_and_definition_before_eof() {
    let reference = byte_offset_to_position(
        SOURCE,
        SOURCE.find("첫함수.").unwrap(),
        PositionEncoding::Utf16,
    )
    .unwrap();
    let messages = [
        RpcMessage::Request {
            id: json!(1),
            method: "initialize".into(),
            params: Some(json!({"capabilities": {"general": {"positionEncodings": ["utf-16"]}}})),
        },
        RpcMessage::Notification {
            method: "textDocument/didOpen".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "languageId": "ddn", "version": 1, "text": SOURCE}
            })),
        },
        RpcMessage::Request {
            id: json!(2),
            method: "textDocument/definition".into(),
            params: Some(json!({
                "textDocument": {"uri": URI},
                "position": {"line": reference.line, "character": reference.character}
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
    for chunk in bytes.chunks(2) {
        for message in decoder.push(chunk).unwrap() {
            events.push(server.handle_message(message).unwrap());
        }
    }
    assert_eq!(
        events[0].responses[0].clone_response().unwrap()["capabilities"]["definitionProvider"],
        true
    );
    let RpcMessage::Response {
        result: Ok(result), ..
    } = &events[2].responses[0]
    else {
        panic!("fragmented definition response missing: {:?}", events[2]);
    };
    assert_eq!(result["uri"], URI);
    assert_eq!(result["range"]["start"], json!({"line": 0, "character": 0}));
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
