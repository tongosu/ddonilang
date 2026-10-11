use ddonirang_tool::lsp::{byte_offset_to_position, LspServer, PositionEncoding, RpcMessage};
use serde_json::{json, Value};

const URI: &str = "file:///lsp-rename.ddn";
const SOURCE: &str = "(문자:글) 첫함수:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 := { (\"첫함수\") 첫함수. }\n셋함수:셈씨 := { 첫함수. }";

fn init(server: &mut LspServer) -> Value {
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(1),
            method: "initialize".into(),
            params: Some(json!({
                "capabilities": {"general": {"positionEncodings": ["utf-16", "utf-8"]}}
            })),
        })
        .unwrap();
    event.responses[0].clone_response().unwrap()
}

fn open(server: &mut LspServer, source: &str, version: i64) {
    open_uri(server, URI, source, version);
}

fn open_uri(server: &mut LspServer, uri: &str, source: &str, version: i64) {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didOpen".into(),
            params: Some(json!({
                "textDocument": {"uri": uri, "languageId": "ddn", "version": version, "text": source}
            })),
        })
        .unwrap();
}

fn change(server: &mut LspServer, source: &str, version: i64) {
    change_uri(server, URI, source, version);
}

fn change_uri(server: &mut LspServer, uri: &str, source: &str, version: i64) {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didChange".into(),
            params: Some(json!({
                "textDocument": {"uri": uri, "version": version},
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
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: method.into(),
            params: Some(params),
        })
        .unwrap();
    event.responses[0].clone_response().unwrap()
}

#[test]
fn lsp7_prepare_and_rename_reuses_local_identity_and_utf16_ranges() {
    let mut server = LspServer::new();
    let initialize = init(&mut server);
    assert_eq!(initialize["capabilities"]["renameProvider"], true);
    open(&mut server, SOURCE, 1);

    let prepare = request(
        &mut server,
        2,
        "textDocument/prepareRename",
        json!({
            "textDocument": {"uri": URI, "version": 1},
            "position": position(SOURCE, "첫함수", 2)
        }),
    );
    assert_eq!(
        prepare["range"],
        json!({
            "start": {"line": 1, "character": 20},
            "end": {"line": 1, "character": 23}
        })
    );
    assert_eq!(prepare["placeholder"], "첫함수");
    assert_eq!(
        prepare["data"]["definitionIdentity"],
        "definition:local:첫함수"
    );

    let renamed = request(
        &mut server,
        3,
        "textDocument/rename",
        json!({
            "textDocument": {"uri": URI, "version": 1},
            "position": position(SOURCE, "첫함수", 2),
            "newName": "새함수"
        }),
    );
    let edits = renamed["documentChanges"][0]["edits"].as_array().unwrap();
    assert_eq!(edits.len(), 3);
    assert!(edits.iter().all(|edit| edit["newText"] == "새함수"));
    assert_eq!(
        edits[1]["range"],
        json!({
            "start": {"line": 1, "character": 20},
            "end": {"line": 1, "character": 23}
        })
    );
    assert_eq!(
        renamed["data"]["definitionIdentity"],
        "definition:local:첫함수"
    );
    assert_eq!(
        renamed,
        request(
            &mut server,
            4,
            "textDocument/rename",
            json!({
                "textDocument": {"uri": URI, "version": 1},
                "position": position(SOURCE, "첫함수", 2),
                "newName": "새함수"
            }),
        )
    );
}

#[test]
fn lsp11_cross_open_document_rename_is_atomic_versioned_and_deterministic() {
    const URI_A: &str = "file:///a.ddn";
    const URI_B: &str = "file:///b.ddn";
    const URI_C: &str = "file:///c.ddn";
    const SOURCE_A: &str = "첫함수:셈씨 := { 1 돌려줘. }\n";
    const SOURCE_B: &str = "둘함수:셈씨 := { (\"🙂\") 첫함수. }\n";
    const SOURCE_C: &str = "셋함수:셈씨 := { 첫함수. }\n";

    let mut server = LspServer::new();
    init(&mut server);
    open_uri(&mut server, URI_C, SOURCE_C, 3);
    open_uri(&mut server, URI_A, SOURCE_A, 1);
    open_uri(&mut server, URI_B, SOURCE_B, 2);

    let prepare = request(
        &mut server,
        20,
        "textDocument/prepareRename",
        json!({
            "textDocument": {"uri": URI_B, "version": 2},
            "position": position(SOURCE_B, "첫함수", 0)
        }),
    );
    assert_eq!(prepare["placeholder"], "첫함수");
    assert_eq!(
        prepare["range"],
        json!({
            "start": {"line": 0, "character": 19},
            "end": {"line": 0, "character": 22}
        })
    );

    let renamed = request(
        &mut server,
        21,
        "textDocument/rename",
        json!({
            "textDocument": {"uri": URI_B, "version": 2},
            "position": position(SOURCE_B, "첫함수", 0),
            "newName": "새함수"
        }),
    );
    let changes = renamed["documentChanges"].as_array().unwrap();
    assert_eq!(
        changes
            .iter()
            .map(|change| change["textDocument"]["uri"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![URI_A, URI_B, URI_C]
    );
    assert_eq!(changes[0]["textDocument"]["version"], 1);
    assert_eq!(changes[1]["textDocument"]["version"], 2);
    assert_eq!(changes[2]["textDocument"]["version"], 3);
    assert_eq!(changes[0]["edits"].as_array().unwrap().len(), 1);
    assert_eq!(changes[1]["edits"].as_array().unwrap().len(), 1);
    assert_eq!(changes[2]["edits"].as_array().unwrap().len(), 1);
    assert_eq!(
        changes[1]["edits"][0]["range"],
        json!({
            "start": {"line": 0, "character": 19},
            "end": {"line": 0, "character": 22}
        })
    );
    assert_eq!(
        renamed,
        request(
            &mut server,
            22,
            "textDocument/rename",
            json!({
                "textDocument": {"uri": URI_B, "version": 2},
                "position": position(SOURCE_B, "첫함수", 0),
                "newName": "새함수"
            }),
        )
    );

    let changed_b = "둘함수:셈씨 := { (\"🙂\") 첫함수. }\n";
    change_uri(&mut server, URI_B, changed_b, 4);
    let changed_rename = request(
        &mut server,
        24,
        "textDocument/rename",
        json!({
            "textDocument": {"uri": URI_B, "version": 4},
            "position": position(changed_b, "첫함수", 0),
            "newName": "새함수"
        }),
    );
    assert_eq!(
        changed_rename["documentChanges"][1]["textDocument"]["version"],
        4
    );

    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(23),
            method: "textDocument/rename".into(),
            params: Some(json!({
                "textDocument": {"uri": URI_B, "version": 1},
                "position": position(SOURCE_B, "첫함수", 0),
                "newName": "새함수"
            })),
        })
        .unwrap();
    assert!(matches!(
        &stale.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32801
    ));
}

#[test]
fn lsp11_closed_or_ambiguous_target_and_invalid_names_fail_closed() {
    const URI_A: &str = "file:///a-ambiguous.ddn";
    const URI_B: &str = "file:///b-reference.ddn";
    const URI_C: &str = "file:///c-duplicate.ddn";
    const SOURCE_A: &str = "첫함수:셈씨 := { 1 돌려줘. }\n";
    const SOURCE_B: &str = "둘함수:셈씨 := { 첫함수. }\n";
    const SOURCE_C: &str = "첫함수:셈씨 := { 2 돌려줘. }\n";

    let mut server = LspServer::new();
    init(&mut server);
    open_uri(&mut server, URI_A, SOURCE_A, 1);
    open_uri(&mut server, URI_B, SOURCE_B, 1);
    open_uri(&mut server, URI_C, SOURCE_C, 1);

    assert_eq!(
        request(
            &mut server,
            30,
            "textDocument/prepareRename",
            json!({
                "textDocument": {"uri": URI_B, "version": 1},
                "position": position(SOURCE_B, "첫함수", 0)
            }),
        ),
        Value::Null
    );
    assert_eq!(
        request(
            &mut server,
            31,
            "textDocument/rename",
            json!({
                "textDocument": {"uri": URI_B, "version": 1},
                "position": position(SOURCE_B, "첫함수", 0),
                "newName": "새함수"
            }),
        ),
        Value::Null
    );

    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didClose".into(),
            params: Some(json!({"textDocument": {"uri": URI_C}})),
        })
        .unwrap();
    let after_close = request(
        &mut server,
        32,
        "textDocument/prepareRename",
        json!({
            "textDocument": {"uri": URI_B, "version": 1},
            "position": position(SOURCE_B, "첫함수", 0)
        }),
    );
    assert_eq!(after_close["placeholder"], "첫함수");

    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didClose".into(),
            params: Some(json!({"textDocument": {"uri": URI_A}})),
        })
        .unwrap();
    assert_eq!(
        request(
            &mut server,
            35,
            "textDocument/prepareRename",
            json!({
                "textDocument": {"uri": URI_B, "version": 1},
                "position": position(SOURCE_B, "첫함수", 0)
            }),
        ),
        Value::Null
    );

    assert_eq!(
        request(
            &mut server,
            33,
            "textDocument/rename",
            json!({
                "textDocument": {"uri": URI_B, "version": 1},
                "position": position(SOURCE_B, "첫함수", 0),
                "newName": ""
            }),
        ),
        Value::Null
    );
    assert_eq!(
        request(
            &mut server,
            34,
            "textDocument/rename",
            json!({
                "textDocument": {"uri": URI_B, "version": 1},
                "position": {"line": 0, "character": 4},
                "newName": "새함수"
            }),
        ),
        Value::Null
    );
}

#[test]
fn lsp7_change_stale_collision_invalid_nonlocal_and_close_fail_closed() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, SOURCE, 1);

    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(2),
            method: "textDocument/rename".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": 0},
                "position": position(SOURCE, "첫함수", 2),
                "newName": "새함수"
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
            method: "textDocument/prepareRename".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": 1, "snapshotIdentity": "sha256:stale"},
                "position": position(SOURCE, "첫함수", 2)
            })),
        })
        .unwrap();
    assert!(matches!(
        &snapshot_stale.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32802
    ));

    assert_eq!(
        request(
            &mut server,
            4,
            "textDocument/rename",
            json!({
                "textDocument": {"uri": URI, "version": 1},
                "position": position(SOURCE, "첫함수", 2),
                "newName": "둘함수"
            }),
        ),
        Value::Null
    );
    assert_eq!(
        request(
            &mut server,
            5,
            "textDocument/rename",
            json!({
                "textDocument": {"uri": URI, "version": 1},
                "position": position(SOURCE, "첫함수", 2),
                "newName": "1"
            }),
        ),
        Value::Null
    );
    assert_eq!(
        request(
            &mut server,
            6,
            "textDocument/rename",
            json!({
                "textDocument": {"uri": URI, "version": 1},
                "position": {"line": 0, "character": 6},
                "newName": "새함수"
            }),
        ),
        Value::Null
    );

    let changed = "(문자:글) 새이름:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 := { (\"🙂\") 새이름. }";
    change(&mut server, changed, 2);
    let current = request(
        &mut server,
        7,
        "textDocument/prepareRename",
        json!({
            "textDocument": {"uri": URI, "version": 2},
            "position": position(changed, "새이름", 1)
        }),
    );
    assert_eq!(current["placeholder"], "새이름");

    let stdlib = "첫함수:글 := { (1) 글로. }";
    change(&mut server, stdlib, 3);
    assert_eq!(
        request(
            &mut server,
            8,
            "textDocument/prepareRename",
            json!({
                "textDocument": {"uri": URI, "version": 3},
                "position": position(stdlib, "글로", 0)
            }),
        ),
        Value::Null
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
            9,
            "textDocument/rename",
            json!({
                "textDocument": {"uri": URI},
                "position": {"line": 0, "character": 0},
                "newName": "새함수"
            }),
        ),
        Value::Null
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
