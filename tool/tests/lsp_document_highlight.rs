use ddonirang_tool::lsp::{byte_offset_to_position, LspServer, PositionEncoding, RpcMessage};
use serde_json::{json, Value};

const URI: &str = "file:///lsp-document-highlight.ddn";
const SOURCE: &str = "(문자:글) 첫함수:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 := { (\"🙂\") 첫함수. }\n셋함수:셈씨 := { 첫함수. }";

fn init(server: &mut LspServer) -> Value {
    server
        .handle_message(RpcMessage::Request {
            id: json!(1),
            method: "initialize".into(),
            params: Some(json!({
                "capabilities": {"general": {"positionEncodings": ["utf-16", "utf-8"]}}
            })),
        })
        .unwrap()
        .responses[0]
        .clone_response()
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

fn position(source: &str, needle: &str, ordinal: usize) -> Value {
    let byte = source.match_indices(needle).nth(ordinal).unwrap().0;
    let position = byte_offset_to_position(source, byte, PositionEncoding::Utf16).unwrap();
    json!({"line": position.line, "character": position.character})
}

fn request(server: &mut LspServer, id: i64, method: &str, params: Value) -> Value {
    server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: method.into(),
            params: Some(params),
        })
        .unwrap()
        .responses[0]
        .clone_response()
        .unwrap()
}

#[test]
fn lsp8_highlight_reuses_reference_ranges_and_utf16_order() {
    let mut server = LspServer::new();
    let initialize = init(&mut server);
    assert_eq!(
        initialize["capabilities"]["documentHighlightProvider"],
        true
    );
    open(&mut server, SOURCE, 1);

    let highlights = request(
        &mut server,
        2,
        "textDocument/documentHighlight",
        json!({
            "textDocument": {"uri": URI, "version": 1},
            "position": position(SOURCE, "첫함수", 1)
        }),
    );
    assert_eq!(highlights.as_array().unwrap().len(), 3);
    assert_eq!(
        highlights
            .as_array()
            .unwrap()
            .iter()
            .map(|highlight| highlight["kind"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![3, 2, 2]
    );
    assert_eq!(
        highlights[1]["range"],
        json!({
            "start": {"line": 1, "character": 19},
            "end": {"line": 1, "character": 22}
        })
    );

    let references = request(
        &mut server,
        3,
        "textDocument/references",
        json!({
            "textDocument": {"uri": URI, "version": 1},
            "position": position(SOURCE, "첫함수", 1),
            "context": {"includeDeclaration": true}
        }),
    );
    let reference_ranges = references
        .as_array()
        .unwrap()
        .iter()
        .map(|location| location["range"].clone())
        .collect::<Vec<_>>();
    assert_eq!(
        highlights
            .as_array()
            .unwrap()
            .iter()
            .map(|highlight| highlight["range"].clone())
            .collect::<Vec<_>>(),
        reference_ranges
    );
    assert_eq!(
        highlights,
        request(
            &mut server,
            4,
            "textDocument/documentHighlight",
            json!({
                "textDocument": {"uri": URI, "version": 1},
                "position": position(SOURCE, "첫함수", 2)
            }),
        )
    );
}

#[test]
fn lsp8_highlight_stale_change_false_positive_and_close_fail_closed() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, SOURCE, 1);

    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(2),
            method: "textDocument/documentHighlight".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": 0},
                "position": position(SOURCE, "첫함수", 1)
            })),
        })
        .unwrap();
    assert!(matches!(
        &stale.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32801
    ));

    let snapshot_stale = server
        .handle_message(RpcMessage::Request {
            id: json!(3),
            method: "textDocument/documentHighlight".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": 1, "snapshotIdentity": "sha256:stale"},
                "position": position(SOURCE, "첫함수", 1)
            })),
        })
        .unwrap();
    assert!(matches!(
        &snapshot_stale.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32802
    ));

    let changed = "새함수:셈씨 := { 1 돌려줘. }\n다른함수:셈씨 := { 새함수. }";
    change(&mut server, changed, 2);
    let current = request(
        &mut server,
        4,
        "textDocument/documentHighlight",
        json!({
            "textDocument": {"uri": URI, "version": 2},
            "position": position(changed, "새함수", 1)
        }),
    );
    assert_eq!(current.as_array().unwrap().len(), 2);

    let false_positive = "새함수:셈씨 := { (\"새함수\") # 새함수 }";
    change(&mut server, false_positive, 3);
    assert_eq!(
        request(
            &mut server,
            5,
            "textDocument/documentHighlight",
            json!({
                "textDocument": {"uri": URI, "version": 3},
                "position": {"line": 0, "character": 25}
            }),
        ),
        json!([])
    );
    assert_eq!(
        request(
            &mut server,
            51,
            "textDocument/documentHighlight",
            json!({
                "textDocument": {"uri": URI, "version": 3},
                "position": position(false_positive, "\"새함수\"", 0)
            }),
        ),
        json!([])
    );

    let stdlib = "첫함수:글 := { (1) 글로. }";
    change(&mut server, stdlib, 4);
    assert_eq!(
        request(
            &mut server,
            6,
            "textDocument/documentHighlight",
            json!({
                "textDocument": {"uri": URI, "version": 4},
                "position": position(stdlib, "글로", 0)
            }),
        ),
        json!([])
    );

    let valid_string = "새함수:셈씨 := { \"새함수\" 보여주기. }";
    change(&mut server, valid_string, 5);
    assert_eq!(
        request(
            &mut server,
            8,
            "textDocument/documentHighlight",
            json!({
                "textDocument": {"uri": URI, "version": 5},
                "position": position(valid_string, "\"새함수\"", 0)
            }),
        ),
        json!([])
    );

    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didClose".into(),
            params: Some(json!({"textDocument": {"uri": URI}})),
        })
        .unwrap();
    assert_eq!(
        request(
            &mut server,
            7,
            "textDocument/documentHighlight",
            json!({
                "textDocument": {"uri": URI},
                "position": {"line": 0, "character": 0}
            }),
        ),
        json!([])
    );
}

#[test]
fn lsp14_highlight_ambiguous_duplicate_definition_is_empty() {
    let source =
        "첫함수:셈씨 := { 1 돌려줘. }\n첫함수:셈씨 := { 2 돌려줘. }\n셋함수:셈씨 := { 첫함수. }";
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, source, 1);
    assert_eq!(
        request(
            &mut server,
            20,
            "textDocument/documentHighlight",
            json!({
                "textDocument": {"uri": URI, "version": 1},
                "position": position(source, "첫함수", 2)
            }),
        ),
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
