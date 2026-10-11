use ddonirang_tool::lsp::{
    byte_offset_to_position, LspServer, PositionEncoding, RpcMessage, ServerEvent,
};
use serde_json::{json, Value};

const URI: &str = "file:///lsp-document-symbol.ddn";
const SOURCE: &str = "새값:이름씨 := 2.\n첫함수:셈씨 := { \"🙂\" 보여주기. }";

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
    open_uri(server, URI, source, version)
}

fn open_uri(server: &mut LspServer, uri: &str, source: &str, version: i64) -> ServerEvent {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didOpen".into(),
            params: Some(json!({
                "textDocument": {"uri": uri, "languageId": "ddn", "version": version, "text": source}
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

fn symbols(server: &mut LspServer, id: i64, version: Option<i64>) -> Value {
    let mut document = json!({"uri": URI});
    if let Some(version) = version {
        document["version"] = json!(version);
    }
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: "textDocument/documentSymbol".into(),
            params: Some(json!({"textDocument": document})),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result.clone(),
        _ => panic!("document symbol request must return a success response: {event:?}"),
    }
}

fn workspace_symbols(server: &mut LspServer, id: i64, query: &str, documents: Value) -> Value {
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: "workspace/symbol".into(),
            params: Some(json!({"query": query, "documents": documents})),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result.clone(),
        _ => panic!("workspace symbol request must return a success response: {event:?}"),
    }
}

fn position(source: &str, byte: usize) -> Value {
    let position = byte_offset_to_position(source, byte, PositionEncoding::Utf16).unwrap();
    json!({"line": position.line, "character": position.character})
}

#[test]
fn h01_order_kind_full_and_selection_ranges() {
    let mut server = LspServer::new();
    let initialized = init(&mut server);
    assert_eq!(
        initialized.responses[0].clone_response().unwrap()["capabilities"]
            ["documentSymbolProvider"],
        true
    );
    open(&mut server, SOURCE, 1);

    let result = symbols(&mut server, 2, Some(1));
    assert_eq!(result.as_array().unwrap().len(), 2, "result={result:?}");
    assert_eq!(result[0]["name"], "새값");
    assert_eq!(result[0]["kind"], 13);
    assert_eq!(result[1]["name"], "첫함수");
    assert_eq!(result[1]["kind"], 12);

    let first_end = SOURCE.find('\n').unwrap();
    assert_eq!(
        result[0]["range"],
        json!({
            "start": position(SOURCE, 0),
            "end": position(SOURCE, first_end),
        })
    );
    let first_name = SOURCE.find("새값").unwrap();
    assert_eq!(
        result[0]["selectionRange"],
        json!({
            "start": position(SOURCE, first_name),
            "end": position(SOURCE, first_name + "새값".len()),
        })
    );
    let second_name = SOURCE.find("첫함수").unwrap();
    assert_eq!(
        result[1]["selectionRange"],
        json!({
            "start": position(SOURCE, second_name),
            "end": position(SOURCE, second_name + "첫함수".len()),
        })
    );
    assert_eq!(
        result[0]["data"]["definitionIdentity"],
        "definition:local:새값"
    );
    assert_eq!(result[1]["data"]["documentVersion"], 1);
    assert_eq!(symbols(&mut server, 3, Some(1)), result);
}

#[test]
fn h02_change_stale_rejection_malformed_and_close_empty() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, SOURCE, 1);
    let changed = "새값:이름씨 := 2.\n새함수:셈씨 := { 새값. }";
    change(&mut server, changed, 2);

    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(2),
            method: "textDocument/documentSymbol".into(),
            params: Some(json!({"textDocument": {"uri": URI, "version": 1}})),
        })
        .unwrap();
    assert!(matches!(
        &stale.responses[0],
        RpcMessage::Response {
            result: Err(error), ..
        } if error.code == -32801
    ));
    let current = symbols(&mut server, 3, Some(2));
    assert_eq!(current[0]["name"], "새값");

    let legacy = "새값:이름씨 = 2.";
    change(&mut server, legacy, 3);
    assert_eq!(symbols(&mut server, 4, Some(3)), json!([]));

    close(&mut server);
    assert_eq!(symbols(&mut server, 5, None), json!([]));
}

#[test]
fn lsp12_workspace_symbol_is_open_document_bounded_and_snapshot_bound() {
    const URI_A: &str = "file:///workspace-a.ddn";
    const URI_B: &str = "file:///workspace-b.ddn";
    const URI_C: &str = "file:///workspace-c.ddn";
    const SOURCE_A: &str = "첫함수:셈씨 := { 1 돌려줘. }\n";
    const SOURCE_B: &str = "둘함수:셈씨 := { \"🙂\" 보여주기. }\n";
    const SOURCE_C: &str = "첫함수:셈씨 := { 3 돌려줘. }\n";

    let mut server = LspServer::new();
    let initialized = init(&mut server);
    assert_eq!(
        initialized.responses[0].clone_response().unwrap()["capabilities"]
            ["workspaceSymbolProvider"],
        true
    );
    open_uri(&mut server, URI_B, SOURCE_B, 2);
    open_uri(&mut server, URI_A, SOURCE_A, 1);

    let live_documents = json!([
        {
            "uri": URI_A,
            "version": 1,
            "snapshotIdentity": server.documents().get(URI_A).unwrap().identity.as_str()
        },
        {
            "uri": URI_B,
            "version": 2,
            "snapshotIdentity": server.documents().get(URI_B).unwrap().identity.as_str()
        }
    ]);
    let result = workspace_symbols(&mut server, 10, "함수", live_documents.clone());
    assert_eq!(result.as_array().unwrap().len(), 2);
    assert_eq!(result[0]["location"]["uri"], URI_A);
    assert_eq!(result[1]["location"]["uri"], URI_B);
    assert_eq!(result[0]["name"], "첫함수");
    assert_eq!(
        result[0]["location"]["range"]["start"],
        json!({"line": 0, "character": 0})
    );
    assert_eq!(result[0]["data"]["documentVersion"], 1);
    assert_eq!(
        result,
        workspace_symbols(&mut server, 11, "함수", live_documents)
    );
    assert!(
        workspace_symbols(&mut server, 12, "", json!([]))
            .as_array()
            .unwrap()
            .len()
            <= 1_000
    );

    let stale = workspace_symbols(
        &mut server,
        13,
        "함수",
        json!([{
            "uri": URI_A,
            "version": 1,
            "snapshotIdentity": "sha256:stale"
        }]),
    );
    assert_eq!(stale, json!([]));

    let changed = "새함수:셈씨 := { 4 돌려줘. }\n";
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didChange".into(),
            params: Some(json!({
                "textDocument": {"uri": URI_B, "version": 3},
                "contentChanges": [{"text": changed}]
            })),
        })
        .unwrap();
    assert_eq!(
        workspace_symbols(&mut server, 14, "둘함수", json!([])),
        json!([])
    );
    let current = workspace_symbols(&mut server, 15, "새함수", json!([]));
    assert_eq!(current.as_array().unwrap().len(), 1);
    assert_eq!(current[0]["data"]["documentVersion"], 3);

    open_uri(&mut server, URI_C, SOURCE_C, 1);
    let c_identity = server.documents().get(URI_C).unwrap().identity.as_str().to_string();
    let duplicates = workspace_symbols(&mut server, 16, "첫함수", json!([]));
    assert_eq!(duplicates.as_array().unwrap().len(), 2);
    assert_eq!(duplicates[0]["location"]["uri"], URI_A);
    assert_eq!(duplicates[1]["location"]["uri"], URI_C);

    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didClose".into(),
            params: Some(json!({"textDocument": {"uri": URI_C}})),
        })
        .unwrap();
    let after_close = workspace_symbols(&mut server, 17, "첫함수", json!([]));
    assert_eq!(after_close.as_array().unwrap().len(), 1);
    assert_eq!(after_close[0]["location"]["uri"], URI_A);
    assert_eq!(
        workspace_symbols(
            &mut server,
            18,
            "첫함수",
            json!([{
                "uri": URI_C,
                "version": 1,
                "snapshotIdentity": c_identity
            }]),
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
