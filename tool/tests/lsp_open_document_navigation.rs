use ddonirang_tool::lsp::{byte_offset_to_position, LspServer, PositionEncoding, RpcMessage};
use serde_json::{json, Value};

const DECL_URI: &str = "file:///a-declaration.ddn";
const REF_URI: &str = "file:///b-reference.ddn";
const DECL_SOURCE: &str = "공유함수:셈씨 := { 1 돌려줘. }";
const REF_SOURCE: &str = "호출함수:셈씨 := { () 공유함수. }";
const CALL_DECL_SOURCE: &str = "공유함수:셈씨 := { 1 돌려줘. }\n내부호출:셈씨 := { () 공유함수. }\n재귀함수:셈씨 := { () 재귀함수. }";
const TYPE_DECL_URI: &str = "file:///a-type-declaration.ddn";
const TYPE_USE_URI: &str = "file:///b-type-use.ddn";
const TYPE_DECL_SOURCE: &str = "새값:이름씨 := 2.";
const TYPE_USE_SOURCE: &str =
    "이모지:글 := \"🙂\".\n(값:새값, 앞:글 = \"🙂\") 사용하는함수:셈씨 := { 1 돌려줘. }";

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
        .responses
        .into_iter()
        .find_map(|response| match response {
            RpcMessage::Response {
                result: Ok(result), ..
            } => Some(result),
            _ => None,
        })
        .expect("initialize response missing")
}

fn open(server: &mut LspServer, uri: &str, source: &str, version: i64) {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didOpen".into(),
            params: Some(json!({
                "textDocument": {"uri": uri, "languageId": "ddn", "version": version, "text": source}
            })),
        })
        .unwrap();
}

fn change(server: &mut LspServer, uri: &str, source: &str, version: i64) {
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

fn close(server: &mut LspServer, uri: &str) {
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didClose".into(),
            params: Some(json!({"textDocument": {"uri": uri}})),
        })
        .unwrap();
}

fn folding_ranges(server: &mut LspServer, uri: &str, version: i64, extra: Value) -> Value {
    let mut document = json!({"uri": uri, "version": version});
    if let Some(extra_object) = extra.as_object() {
        document
            .as_object_mut()
            .expect("document object")
            .extend(extra_object.clone());
    }
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(30),
            method: "textDocument/foldingRange".into(),
            params: Some(json!({"textDocument": document})),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result.clone(),
        _ => panic!("folding range response missing: {event:?}"),
    }
}

fn folding_range_error_code(server: &mut LspServer, uri: &str, version: i64, extra: Value) -> i64 {
    let mut document = json!({"uri": uri, "version": version});
    if let Some(extra_object) = extra.as_object() {
        document
            .as_object_mut()
            .expect("document object")
            .extend(extra_object.clone());
    }
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(31),
            method: "textDocument/foldingRange".into(),
            params: Some(json!({"textDocument": document})),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Err(error), ..
        } => i64::from(error.code),
        _ => panic!("folding range error response missing: {event:?}"),
    }
}

fn selection_ranges(server: &mut LspServer, uri: &str, version: i64, positions: Value) -> Value {
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(32),
            method: "textDocument/selectionRange".into(),
            params: Some(json!({
                "textDocument": {"uri": uri, "version": version},
                "positions": positions,
            })),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result.clone(),
        _ => panic!("selection range response missing: {event:?}"),
    }
}

fn position(source: &str, needle: &str) -> Value {
    let byte = source.find(needle).unwrap();
    position_at_byte(source, byte)
}

fn position_at_byte(source: &str, byte: usize) -> Value {
    let position = byte_offset_to_position(source, byte, PositionEncoding::Utf16).unwrap();
    json!({"line": position.line, "character": position.character})
}

fn definition(server: &mut LspServer, uri: &str, source: &str, version: i64) -> Value {
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(2),
            method: "textDocument/definition".into(),
            params: Some(json!({
                "textDocument": {"uri": uri, "version": version},
                "position": position(source, "공유함수")
            })),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result.clone(),
        _ => panic!("definition response missing: {event:?}"),
    }
}

fn references(server: &mut LspServer, uri: &str, source: &str, version: i64) -> Value {
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(3),
            method: "textDocument/references".into(),
            params: Some(json!({
                "textDocument": {"uri": uri, "version": version},
                "position": position(source, "공유함수"),
                "context": {"includeDeclaration": true}
            })),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result.clone(),
        _ => panic!("references response missing: {event:?}"),
    }
}

fn type_definition(
    server: &mut LspServer,
    uri: &str,
    source: &str,
    version: i64,
    needle: &str,
    extra: Value,
) -> Value {
    let mut document = json!({"uri": uri, "version": version});
    if let Some(extra_object) = extra.as_object() {
        document
            .as_object_mut()
            .expect("document object")
            .extend(extra_object.clone());
    }
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(40),
            method: "textDocument/typeDefinition".into(),
            params: Some(json!({
                "textDocument": document,
                "position": position_at_byte(source, source.rfind(needle).unwrap())
            })),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result.clone(),
        _ => panic!("type definition response missing: {event:?}"),
    }
}

fn call_hierarchy_prepare(
    server: &mut LspServer,
    uri: &str,
    source: &str,
    version: i64,
    needle: &str,
) -> Value {
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(20),
            method: "textDocument/prepareCallHierarchy".into(),
            params: Some(json!({
                "textDocument": {"uri": uri, "version": version},
                "position": position(source, needle)
            })),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result.clone(),
        _ => panic!("call hierarchy prepare response missing: {event:?}"),
    }
}

fn call_hierarchy_calls(server: &mut LspServer, method: &str, item: &Value) -> Value {
    let event = server
        .handle_message(RpcMessage::Request {
            id: json!(21),
            method: method.into(),
            params: Some(json!({"item": item})),
        })
        .unwrap();
    match &event.responses[0] {
        RpcMessage::Response {
            result: Ok(result), ..
        } => result.clone(),
        _ => panic!("call hierarchy calls response missing: {event:?}"),
    }
}

#[test]
fn lsp10_open_documents_resolve_definition_and_references_in_uri_order() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, DECL_URI, DECL_SOURCE, 1);
    open(&mut server, REF_URI, REF_SOURCE, 1);

    let definition = definition(&mut server, REF_URI, REF_SOURCE, 1);
    assert_eq!(definition["uri"], DECL_URI);
    assert_eq!(
        definition["range"]["start"],
        json!({"line": 0, "character": 0})
    );
    assert_eq!(
        definition["data"]["definitionIdentity"],
        "definition:local:공유함수"
    );

    let references = references(&mut server, REF_URI, REF_SOURCE, 1);
    assert_eq!(references.as_array().unwrap().len(), 2);
    assert_eq!(references[0]["uri"], DECL_URI);
    assert_eq!(references[1]["uri"], REF_URI);
    assert_eq!(
        references[1]["range"]["start"],
        json!({"line": 0, "character": 16})
    );
}

#[test]
fn lsp10_open_document_lifecycle_is_current_and_fail_closed() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, DECL_URI, DECL_SOURCE, 1);
    open(&mut server, REF_URI, REF_SOURCE, 1);

    let changed = "호출함수:셈씨 := { (\"🙂\") 공유함수. }";
    change(&mut server, REF_URI, changed, 2);
    let current = references(&mut server, REF_URI, changed, 2);
    assert_eq!(current.as_array().unwrap().len(), 2);
    assert_eq!(current[1]["range"]["start"]["character"], 20);

    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(4),
            method: "textDocument/definition".into(),
            params: Some(json!({
                "textDocument": {"uri": REF_URI, "version": 1},
                "position": position(changed, "공유함수")
            })),
        })
        .unwrap();
    assert!(matches!(
        &stale.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32801
    ));

    close(&mut server, DECL_URI);
    assert_eq!(definition(&mut server, REF_URI, changed, 2), Value::Null);
    assert_eq!(references(&mut server, REF_URI, changed, 2), json!([]));

    open(&mut server, DECL_URI, DECL_SOURCE, 3);
    open(&mut server, "file:///c-duplicate.ddn", DECL_SOURCE, 1);
    assert_eq!(definition(&mut server, REF_URI, changed, 2), Value::Null);
    assert_eq!(references(&mut server, REF_URI, changed, 2), json!([]));
}

#[test]
fn lsp15_call_hierarchy_is_open_document_bound_and_deterministic() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, DECL_URI, CALL_DECL_SOURCE, 1);
    open(&mut server, REF_URI, REF_SOURCE, 1);

    let declaration =
        call_hierarchy_prepare(&mut server, DECL_URI, CALL_DECL_SOURCE, 1, "공유함수");
    assert_eq!(
        declaration["data"]["callableIdentity"],
        "definition:local:공유함수"
    );
    assert_eq!(declaration["uri"], DECL_URI);
    let call_site = call_hierarchy_prepare(&mut server, REF_URI, REF_SOURCE, 1, "공유함수");
    assert_eq!(call_site, declaration);

    let incoming = call_hierarchy_calls(&mut server, "callHierarchy/incomingCalls", &declaration);
    assert_eq!(incoming.as_array().unwrap().len(), 2);
    assert_eq!(incoming[0]["from"]["uri"], DECL_URI);
    assert_eq!(incoming[1]["from"]["uri"], REF_URI);
    assert!(
        incoming[1]["fromRanges"][0]["start"]["character"]
            .as_u64()
            .unwrap()
            > 13
    );

    let internal = call_hierarchy_prepare(&mut server, DECL_URI, CALL_DECL_SOURCE, 1, "내부호출");
    let internal_outgoing =
        call_hierarchy_calls(&mut server, "callHierarchy/outgoingCalls", &internal);
    assert_eq!(internal_outgoing.as_array().unwrap().len(), 1);
    assert_eq!(internal_outgoing[0]["to"], declaration);

    let recursive = call_hierarchy_prepare(&mut server, DECL_URI, CALL_DECL_SOURCE, 1, "재귀함수");
    let recursive_outgoing =
        call_hierarchy_calls(&mut server, "callHierarchy/outgoingCalls", &recursive);
    assert_eq!(recursive_outgoing.as_array().unwrap().len(), 1);
    assert_eq!(recursive_outgoing[0]["to"], recursive);

    let caller = call_hierarchy_prepare(&mut server, REF_URI, REF_SOURCE, 1, "호출함수");
    let outgoing = call_hierarchy_calls(&mut server, "callHierarchy/outgoingCalls", &caller);
    assert_eq!(outgoing.as_array().unwrap().len(), 1);
    assert_eq!(outgoing[0]["to"], declaration);
    assert_eq!(
        outgoing,
        call_hierarchy_calls(&mut server, "callHierarchy/outgoingCalls", &caller)
    );

    let emoji_source = "호출함수:셈씨 := { (\"🙂\") 공유함수. }";
    change(&mut server, REF_URI, emoji_source, 2);
    assert_eq!(
        call_hierarchy_prepare(&mut server, REF_URI, emoji_source, 2, "공유함수"),
        declaration
    );
    let emoji_incoming =
        call_hierarchy_calls(&mut server, "callHierarchy/incomingCalls", &declaration);
    assert_eq!(emoji_incoming.as_array().unwrap().len(), 2);
    assert_eq!(emoji_incoming[1]["fromRanges"][0]["start"]["character"], 20);
    assert_eq!(
        call_hierarchy_calls(&mut server, "callHierarchy/outgoingCalls", &caller),
        json!([])
    );

    let string_source = "호출함수:셈씨 := { \"공유함수\" 보여주기. }";
    change(&mut server, REF_URI, string_source, 3);
    assert_eq!(
        call_hierarchy_prepare(&mut server, REF_URI, string_source, 3, "공유함수"),
        Value::Null
    );
    let stale = call_hierarchy_calls(&mut server, "callHierarchy/outgoingCalls", &caller);
    assert_eq!(stale, json!([]));

    close(&mut server, DECL_URI);
    assert_eq!(
        call_hierarchy_prepare(&mut server, DECL_URI, CALL_DECL_SOURCE, 1, "공유함수"),
        Value::Null
    );
    assert_eq!(
        call_hierarchy_calls(&mut server, "callHierarchy/incomingCalls", &declaration),
        json!([])
    );
}

#[test]
fn lsp15_call_hierarchy_rejects_duplicate_declaration_and_stale_payload() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, DECL_URI, DECL_SOURCE, 1);
    open(&mut server, REF_URI, REF_SOURCE, 1);
    let declaration = call_hierarchy_prepare(&mut server, DECL_URI, DECL_SOURCE, 1, "공유함수");
    open(&mut server, "file:///c-duplicate-call.ddn", DECL_SOURCE, 1);
    assert_eq!(
        call_hierarchy_prepare(&mut server, REF_URI, REF_SOURCE, 1, "공유함수"),
        Value::Null
    );
    close(&mut server, "file:///c-duplicate-call.ddn");
    change(&mut server, DECL_URI, DECL_SOURCE, 2);
    assert_eq!(
        call_hierarchy_calls(&mut server, "callHierarchy/incomingCalls", &declaration),
        json!([])
    );
}

#[test]
fn lsp16_folding_ranges_are_ast_bound_utf16_deterministic_and_current() {
    let mut server = LspServer::new();
    let initialize = init(&mut server);
    assert_eq!(initialize["capabilities"]["foldingRangeProvider"], true);

    let uri = "file:///folding.ddn";
    let source = "공유함수:셈씨 := {\n1 돌려줘.\n}\n";
    open(&mut server, uri, source, 1);

    let ranges = folding_ranges(&mut server, uri, 1, json!({}));
    let ranges_again = folding_ranges(&mut server, uri, 1, json!({}));
    assert_eq!(ranges, ranges_again);
    let ranges = ranges.as_array().expect("folding range array");
    assert_eq!(ranges.len(), 2);
    assert_eq!(ranges[0]["startLine"], 0);
    assert_eq!(ranges[0]["endLine"], 2);
    assert_eq!(ranges[1]["startLine"], 0);
    assert_eq!(ranges[1]["endLine"], 2);
    assert!(ranges.iter().all(|range| range["kind"] == "region"));
    assert_eq!(ranges[0]["startCharacter"], 0);
    assert_eq!(ranges[1]["startCharacter"], 11);

    assert_eq!(
        folding_range_error_code(&mut server, uri, 0, json!({})),
        -32801
    );
    assert_eq!(
        folding_range_error_code(
            &mut server,
            uri,
            1,
            json!({"snapshotIdentity": "snapshot:forged"}),
        ),
        -32802
    );

    let changed = "공유함수:셈씨 := { 1 돌려줘. }";
    change(&mut server, uri, changed, 2);
    assert_eq!(folding_ranges(&mut server, uri, 2, json!({})), json!([]));
    assert_eq!(
        folding_range_error_code(&mut server, uri, 1, json!({})),
        -32801
    );

    let malformed_uri = "file:///folding-malformed.ddn";
    open(&mut server, malformed_uri, "}", 1);
    assert_eq!(
        folding_ranges(&mut server, malformed_uri, 1, json!({})),
        json!([])
    );
    close(&mut server, uri);
    assert_eq!(folding_ranges(&mut server, uri, 2, json!({})), json!([]));
}

#[test]
fn lsp17_selection_ranges_are_ordered_ast_parent_chains_and_current() {
    let mut server = LspServer::new();
    let initialize = init(&mut server);
    assert_eq!(initialize["capabilities"]["selectionRangeProvider"], true);

    let uri = "file:///selection.ddn";
    let source = "공유함수:셈씨 := { \"🙂\" 보여주기. }\n호출함수:셈씨 := { () 공유함수. }\n";
    open(&mut server, uri, source, 1);
    let declaration = source.find("공유함수").unwrap();
    let call = source.rfind("공유함수").unwrap();
    let emoji = source.find("🙂").unwrap();
    let emoji_position = position_at_byte(source, emoji);
    let whitespace = position_at_byte(source, source.find(":= {").unwrap() + 2);
    let positions = json!([
        position_at_byte(source, declaration),
        position_at_byte(source, call),
        {"line": emoji_position["line"], "character": emoji_position["character"].as_u64().unwrap() + 1},
        whitespace,
        {"line": 99, "character": 0}
    ]);
    let result = selection_ranges(&mut server, uri, 1, positions);
    assert_eq!(result.as_array().unwrap().len(), 5);
    assert_eq!(
        result[0]["range"]["start"],
        position_at_byte(source, declaration)
    );
    assert!(result[0]["parent"].is_object());
    assert!(result[1]["parent"].is_object());
    let mut chain = result[1].clone();
    let mut chain_length = 0;
    while chain.is_object() {
        chain_length += 1;
        let parent = &chain["parent"];
        if !parent.is_object() {
            break;
        }
        let chain_start = (
            chain["range"]["start"]["line"].as_u64().unwrap(),
            chain["range"]["start"]["character"].as_u64().unwrap(),
        );
        let chain_end = (
            chain["range"]["end"]["line"].as_u64().unwrap(),
            chain["range"]["end"]["character"].as_u64().unwrap(),
        );
        let parent_start = (
            parent["range"]["start"]["line"].as_u64().unwrap(),
            parent["range"]["start"]["character"].as_u64().unwrap(),
        );
        let parent_end = (
            parent["range"]["end"]["line"].as_u64().unwrap(),
            parent["range"]["end"]["character"].as_u64().unwrap(),
        );
        assert!(parent_start <= chain_start && chain_end <= parent_end);
        chain = parent.clone();
    }
    assert!(chain_length >= 3, "chain={result:?}");
    assert_eq!(result[2], Value::Null);
    assert_eq!(result[3], Value::Null);
    assert_eq!(result[4], Value::Null);
    assert_eq!(
        result,
        selection_ranges(
            &mut server,
            uri,
            1,
            json!([
                position_at_byte(source, declaration),
                position_at_byte(source, call),
                {"line": emoji_position["line"], "character": emoji_position["character"].as_u64().unwrap() + 1},
                whitespace,
                {"line": 99, "character": 0}
            ]),
        )
    );

    let changed = "새함수:셈씨 := { 1 돌려줘. }\n";
    change(&mut server, uri, changed, 2);
    assert!(
        selection_ranges(&mut server, uri, 2, json!([position(changed, "새함수")]))[0].is_object()
    );
    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(33),
            method: "textDocument/selectionRange".into(),
            params: Some(json!({
                "textDocument": {"uri": uri, "version": 1},
                "positions": [position(changed, "새함수")],
            })),
        })
        .unwrap();
    assert!(matches!(
        &stale.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32801
    ));
    let stale_snapshot = server
        .handle_message(RpcMessage::Request {
            id: json!(34),
            method: "textDocument/selectionRange".into(),
            params: Some(json!({
                "textDocument": {
                    "uri": uri,
                    "version": 2,
                    "snapshotIdentity": "snapshot:forged"
                },
                "positions": [position(changed, "새함수")],
            })),
        })
        .unwrap();
    assert!(matches!(
        &stale_snapshot.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32802
    ));
    close(&mut server, uri);
    assert_eq!(
        selection_ranges(&mut server, uri, 2, json!([{"line": 0, "character": 0}])),
        json!([])
    );
}

#[test]
fn pg_cli_result_flow_selection_place_has_a_typed_lsp_parent_chain() {
    let mut server = LspServer::new();
    init(&mut server);
    let uri = "file:///pg-cli-result-flow.ddn";
    let source = "매마디:움직씨 := {\n    판:(수)차림 := 차림[0, 0].\n    1 -> 판[0].\n    판 보여주기.\n}\n";
    open(&mut server, uri, source, 1);
    let selection = source.rfind("판[0]").expect("result-flow selection");
    let result = selection_ranges(
        &mut server,
        uri,
        1,
        json!([position_at_byte(source, selection)]),
    );
    assert!(result[0].is_object(), "selection range must be published: {result:?}");
    assert_eq!(result[0]["range"]["start"], position_at_byte(source, selection));
    assert!(result[0]["parent"].is_object());
    close(&mut server, uri);
}

#[test]
fn lsp19_type_definition_is_exact_open_document_bound_and_utf16_current() {
    let mut server = LspServer::new();
    let initialize = init(&mut server);
    assert_eq!(initialize["capabilities"]["typeDefinitionProvider"], true);
    open(&mut server, TYPE_DECL_URI, TYPE_DECL_SOURCE, 1);
    open(&mut server, TYPE_USE_URI, TYPE_USE_SOURCE, 1);

    let result = type_definition(
        &mut server,
        TYPE_USE_URI,
        TYPE_USE_SOURCE,
        1,
        "새값",
        json!({}),
    );
    assert_eq!(result["uri"], TYPE_DECL_URI);
    assert_eq!(result["range"]["start"], json!({"line": 0, "character": 0}));
    assert_eq!(result["range"]["end"], json!({"line": 0, "character": 2}));
    assert_eq!(result["data"]["typeIdentity"], "type:local:새값");
    assert_eq!(result["data"]["documentVersion"], 1);
    assert_eq!(
        result["data"]["snapshotIdentity"],
        server.documents().get(TYPE_DECL_URI).unwrap().identity.as_str()
    );

    let use_position = position(TYPE_USE_SOURCE, "새값");
    assert_eq!(use_position["character"], 3);
    assert_eq!(result, type_definition(
        &mut server,
        TYPE_USE_URI,
        TYPE_USE_SOURCE,
        1,
        "새값",
        json!({}),
    ));

    let applied = "상자:이름씨 := 2.\n(값:(새값)상자) 적용함수:셈씨 := { 1 돌려줘. }";
    open(&mut server, "file:///c-applied.ddn", applied, 1);
    let applied_result = type_definition(
        &mut server,
        "file:///c-applied.ddn",
        applied,
        1,
        "상자",
        json!({}),
    );
    assert_eq!(applied_result["uri"], "file:///c-applied.ddn");
    assert_eq!(applied_result["data"]["typeIdentity"], "type:local:상자");

    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(41),
            method: "textDocument/typeDefinition".into(),
            params: Some(json!({
                "textDocument": {"uri": TYPE_USE_URI, "version": 0},
                "position": position(TYPE_USE_SOURCE, "새값")
            })),
        })
        .unwrap();
    assert!(matches!(
        &stale.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32801
    ));
    let stale_snapshot = server
        .handle_message(RpcMessage::Request {
            id: json!(42),
            method: "textDocument/typeDefinition".into(),
            params: Some(json!({
                "textDocument": {
                    "uri": TYPE_USE_URI,
                    "version": 1,
                    "snapshotIdentity": "snapshot:forged"
                },
                "position": position(TYPE_USE_SOURCE, "새값")
            })),
        })
        .unwrap();
    assert!(matches!(
        &stale_snapshot.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32802
    ));

    close(&mut server, TYPE_DECL_URI);
    assert_eq!(
        type_definition(&mut server, TYPE_USE_URI, TYPE_USE_SOURCE, 1, "새값", json!({})),
        Value::Null
    );

    let false_positive = "거짓:셈씨 := { \"새값\" 보여주기. }";
    open(&mut server, "file:///d-false-positive.ddn", false_positive, 1);
    assert_eq!(
        type_definition(
            &mut server,
            "file:///d-false-positive.ddn",
            false_positive,
            1,
            "새값",
            json!({})
        ),
        Value::Null
    );
    let unknown = "미지:셈씨 := { (값:수) 값 보여주기. }";
    open(&mut server, "file:///e-unknown.ddn", unknown, 1);
    assert_eq!(
        type_definition(
            &mut server,
            "file:///e-unknown.ddn",
            unknown,
            1,
            "수",
            json!({})
        ),
        Value::Null
    );
    let callable_only = "가짜:셈씨 := { 1 돌려줘. }";
    let callable_use = "(값:가짜) 사용함수:셈씨 := { 1 돌려줘. }";
    open(&mut server, "file:///f-callable-only.ddn", callable_only, 1);
    open(&mut server, "file:///g-callable-use.ddn", callable_use, 1);
    assert_eq!(
        type_definition(
            &mut server,
            "file:///g-callable-use.ddn",
            callable_use,
            1,
            "가짜",
            json!({})
        ),
        Value::Null
    );
}

#[test]
fn lsp19_type_definition_rejects_duplicate_and_current_snapshot_only() {
    let mut server = LspServer::new();
    init(&mut server);
    open(&mut server, TYPE_DECL_URI, TYPE_DECL_SOURCE, 1);
    open(&mut server, TYPE_USE_URI, TYPE_USE_SOURCE, 1);
    open(&mut server, "file:///c-duplicate-type.ddn", TYPE_DECL_SOURCE, 1);
    assert_eq!(
        type_definition(&mut server, TYPE_USE_URI, TYPE_USE_SOURCE, 1, "새값", json!({})),
        Value::Null
    );

    close(&mut server, "file:///c-duplicate-type.ddn");
    let changed = "(값:수) 다른함수:셈씨 := { 1 돌려줘. }";
    change(&mut server, TYPE_USE_URI, changed, 2);
    assert_eq!(
        type_definition(&mut server, TYPE_USE_URI, changed, 2, "수", json!({})),
        Value::Null
    );
    let stale = server
        .handle_message(RpcMessage::Request {
            id: json!(43),
            method: "textDocument/typeDefinition".into(),
            params: Some(json!({
                "textDocument": {"uri": TYPE_USE_URI, "version": 1},
                "position": position(changed, "수")
            })),
        })
        .unwrap();
    assert!(matches!(
        &stale.responses[0],
        RpcMessage::Response { result: Err(error), .. } if error.code == -32801
    ));
}
