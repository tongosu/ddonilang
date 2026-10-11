use ddonirang_tool::lsp::{LspServer, RpcMessage};
use serde_json::{json, Value};

const URI: &str = "file:///lsp-semantic-tokens.ddn";
const SOURCE: &str = "(문자:글) 첫함수:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 := { (\"🙂\") 첫함수. }";

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

fn tokens(server: &mut LspServer, id: i64, version: i64) -> Value {
    server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: "textDocument/semanticTokens/full".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": version}
            })),
        })
        .unwrap()
        .responses[0]
        .clone_response()
        .unwrap()
}

fn delta(
    server: &mut LspServer,
    id: i64,
    uri: &str,
    version: i64,
    previous_result_id: &str,
) -> Value {
    server
        .handle_message(RpcMessage::Request {
            id: json!(id),
            method: "textDocument/semanticTokens/full/delta".into(),
            params: Some(json!({
                "textDocument": {"uri": uri, "version": version},
                "previousResultId": previous_result_id,
            })),
        })
        .unwrap()
        .responses[0]
        .clone_response()
        .unwrap()
}

fn decode(data: &[Value]) -> Vec<(u32, u32, u32, u32)> {
    let mut line = 0;
    let mut start = 0;
    data.chunks_exact(5)
        .map(|chunk| {
            line += chunk[0].as_u64().unwrap() as u32;
            start = if chunk[0] == 0 {
                start + chunk[1].as_u64().unwrap() as u32
            } else {
                chunk[1].as_u64().unwrap() as u32
            };
            (
                line,
                start,
                chunk[2].as_u64().unwrap() as u32,
                chunk[3].as_u64().unwrap() as u32,
            )
        })
        .collect()
}

#[test]
fn lsp9_full_tokens_use_lexer_categories_utf16_and_deterministic_delta() {
    let mut server = LspServer::new();
    let initialize = init(&mut server);
    assert_eq!(
        initialize["capabilities"]["semanticTokensProvider"]["full"]["delta"],
        true
    );
    assert_eq!(
        initialize["capabilities"]["semanticTokensProvider"]["legend"]["tokenTypes"][0],
        "function"
    );
    open(&mut server, SOURCE, 1);

    let first = tokens(&mut server, 2, 1);
    assert_eq!(first["data"].as_array().unwrap().len() % 5, 0);
    let decoded = decode(first["data"].as_array().unwrap());
    assert!(decoded.iter().any(|token| *token == (1, 19, 3, 0)));
    assert!(decoded.iter().any(|token| token.3 == 1));
    assert!(decoded.iter().any(|token| token.3 == 4));
    assert!(decoded.iter().any(|token| token.3 == 5));
    assert!(decoded.iter().any(|token| token.3 == 6));
    assert_eq!(first["dataVersion"], 1);
    assert_eq!(first["snapshotIdentity"].as_str().unwrap().len(), 71);
    assert_eq!(first, tokens(&mut server, 3, 1));
}

#[test]
fn lsp9d_delta_is_deterministic_bounded_and_reconstructs_current_stream() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, SOURCE, 1);

    let first = tokens(&mut server, 2, 1);
    let first_data = first["data"].as_array().unwrap().clone();
    let first_result_id = first["resultId"].as_str().unwrap().to_string();
    let unchanged = delta(&mut server, 3, URI, 1, &first_result_id);
    assert_eq!(unchanged["edits"], json!([]));
    assert_eq!(unchanged["resultId"], first["resultId"]);

    let changed_source = "첫함수:셈씨 := { 2 돌려줘. }\n둘함수:셈씨 := { 첫함수. }";
    change(&mut server, changed_source, 2);
    let changed = delta(&mut server, 4, URI, 2, &first_result_id);
    let edit = &changed["edits"][0];
    let start = edit["start"].as_u64().unwrap() as usize;
    let delete_count = edit["deleteCount"].as_u64().unwrap() as usize;
    let replacement = edit["data"].as_array().unwrap().clone();
    let mut reconstructed = first_data;
    reconstructed.splice(start..start + delete_count, replacement);
    assert_eq!(
        reconstructed,
        tokens(&mut server, 5, 2)["data"]
            .as_array()
            .unwrap()
            .clone()
    );
    assert_ne!(changed["resultId"], first["resultId"]);
    let current_result_id = changed["resultId"].as_str().unwrap().to_string();

    let forged = delta(&mut server, 6, URI, 2, "sha256:forged");
    assert!(forged["data"].is_array());
    assert!(forged["resultId"].is_string());

    open_uri(
        &mut server,
        "file:///lsp-semantic-tokens-other.ddn",
        SOURCE,
        1,
    );
    let other_full = server
        .handle_message(RpcMessage::Request {
            id: json!(7),
            method: "textDocument/semanticTokens/full".into(),
            params: Some(json!({
                "textDocument": {"uri": "file:///lsp-semantic-tokens-other.ddn", "version": 1}
            })),
        })
        .unwrap()
        .responses[0]
        .clone_response()
        .unwrap();
    let other_id = other_full["resultId"].as_str().unwrap();
    assert_ne!(other_id, first_result_id);
    let wrong_document = delta(&mut server, 8, URI, 2, other_id);
    assert!(wrong_document["data"].is_array());

    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didClose".into(),
            params: Some(json!({"textDocument": {"uri": URI}})),
        })
        .unwrap();
    let closed = delta(&mut server, 9, URI, 2, other_id);
    assert_eq!(closed["data"], json!([]));

    open(&mut server, changed_source, 3);
    let reopened = delta(&mut server, 10, URI, 3, &current_result_id);
    assert!(reopened["data"].is_array());
}

#[test]
fn lsp9_full_tokens_stale_change_malformed_and_close_fail_closed() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, SOURCE, 1);

    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(2),
            method: "textDocument/semanticTokens/full".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": 0}
            })),
        })
        .unwrap();
    assert!(matches!(
        &stale.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32801
    ));

    let changed = "새함수:셈씨 := { 1 돌려줘. }";
    change(&mut server, changed, 2);
    let current = tokens(&mut server, 3, 2);
    assert!(!current["data"].as_array().unwrap().is_empty());

    change(&mut server, "새함수:셈씨 = {", 3);
    assert_eq!(tokens(&mut server, 4, 3)["data"], json!([]));

    change(&mut server, "새함수:셈씨 := { 1 돌려줘. }", 4);
    let snapshot_stale = server
        .handle_message(RpcMessage::Request {
            id: json!(5),
            method: "textDocument/semanticTokens/full".into(),
            params: Some(json!({
                "textDocument": {"uri": URI, "version": 4, "snapshotIdentity": "sha256:stale"}
            })),
        })
        .unwrap();
    assert!(matches!(
        &snapshot_stale.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32802
    ));

    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didClose".into(),
            params: Some(json!({"textDocument": {"uri": URI}})),
        })
        .unwrap();
    assert_eq!(tokens(&mut server, 6, 4)["data"], json!([]));
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
