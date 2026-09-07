use ddonirang_tool::lsp::{
    byte_offset_to_position, encode_message, position_to_byte_offset, ByteRange, DocumentChange,
    DocumentStore, FrameDecoder, FrameError, LspPosition, LspRange, LspServer, PositionEncoding,
    PublishRejection, RpcMessage, ServerState,
};
use serde_json::json;

fn initialize(server: &mut LspServer, encodings: &[&str]) {
    server
        .handle_message(RpcMessage::Request {
            id: json!(1),
            method: "initialize".into(),
            params: Some(json!({
                "capabilities": {"general": {"positionEncodings": encodings}}
            })),
        })
        .expect("initialize");
}

#[test]
fn lsp0_frame_partial_multiple_and_exact_length() {
    let first = encode_message(&RpcMessage::Notification {
        method: "initialized".into(),
        params: Some(json!({})),
    })
    .unwrap();
    let second = encode_message(&RpcMessage::Request {
        id: json!(3),
        method: "shutdown".into(),
        params: None,
    })
    .unwrap();
    let mut input = first.clone();
    input.extend_from_slice(&second);
    let mut decoder = FrameDecoder::default();
    assert!(decoder.push(&input[..7]).unwrap().is_empty());
    let messages = decoder.push(&input[7..]).unwrap();
    assert_eq!(messages.len(), 2);
    assert!(matches!(messages[0], RpcMessage::Notification { .. }));
    assert!(matches!(&messages[1], RpcMessage::Request { id, .. } if id == &json!(3)));
    assert_eq!(decoder.finish(), Ok(()));
}

#[test]
fn lsp0_frame_malformed_headers_and_truncated_body_fail_closed() {
    let mut decoder = FrameDecoder::default();
    assert_eq!(
        decoder.push(b"Content-Length: no\r\n\r\n{}").unwrap_err(),
        FrameError::InvalidContentLength
    );
    let mut decoder = FrameDecoder::default();
    assert!(decoder
        .push(b"Content-Length: 10\r\n\r\n{}")
        .unwrap()
        .is_empty());
    assert_eq!(decoder.finish(), Err(FrameError::TruncatedBody));
    let mut decoder = FrameDecoder::default();
    assert_eq!(
        decoder.push(b"X-Test: 1\r\n\r\n{}").unwrap_err(),
        FrameError::MissingContentLength
    );
}

#[test]
fn lsp0_document_lifecycle_versions_and_atomic_batch() {
    let mut store = DocumentStore::new();
    let identity = store
        .open("file:///sync.ddn", "ddn", 1, "A한🙂".as_bytes().to_vec())
        .unwrap();
    assert_eq!(store.get("file:///sync.ddn").unwrap().identity, identity);
    assert!(store
        .apply_changes(
            "file:///sync.ddn",
            1,
            &[DocumentChange::full("stale")],
            PositionEncoding::Utf8,
        )
        .is_err());
    let original = store.get("file:///sync.ddn").unwrap().clone();
    let failed = store.apply_changes(
        "file:///sync.ddn",
        2,
        &[
            DocumentChange::full("first"),
            DocumentChange {
                range: Some(LspRange {
                    start: LspPosition {
                        line: 9,
                        character: 0,
                    },
                    end: LspPosition {
                        line: 9,
                        character: 1,
                    },
                }),
                text: "second".into(),
                range_snapshot: None,
            },
        ],
        PositionEncoding::Utf8,
    );
    assert!(failed.is_err());
    assert_eq!(*store.get("file:///sync.ddn").unwrap(), original);
    store
        .apply_changes(
            "file:///sync.ddn",
            2,
            &[DocumentChange::full("끝")],
            PositionEncoding::Utf16,
        )
        .unwrap();
    assert_eq!(store.get("file:///sync.ddn").unwrap().version, 2);
    store.close("file:///sync.ddn").unwrap();
    assert!(store.get("file:///sync.ddn").is_none());
}

#[test]
fn lsp0_position_utf8_utf16_utf32_and_invalid_midpoints() {
    let source = "A한🙂\r\n끝\n";
    let emoji = source.find('🙂').unwrap();
    for encoding in [
        PositionEncoding::Utf8,
        PositionEncoding::Utf16,
        PositionEncoding::Utf32,
    ] {
        let position = byte_offset_to_position(source, emoji, encoding).unwrap();
        let expected_character = match encoding {
            PositionEncoding::Utf8 => 4,
            PositionEncoding::Utf16 => 2,
            PositionEncoding::Utf32 => 2,
        };
        assert_eq!(position.character, expected_character);
        assert_eq!(
            position_to_byte_offset(source, position, encoding),
            Ok(emoji)
        );
    }
    assert_eq!(
        byte_offset_to_position(source, source.find('끝').unwrap(), PositionEncoding::Utf16),
        Ok(LspPosition {
            line: 1,
            character: 0
        })
    );
    assert!(position_to_byte_offset(
        "🙂",
        LspPosition {
            line: 0,
            character: 1,
        },
        PositionEncoding::Utf16
    )
    .is_err());
    assert!(position_to_byte_offset(
        "A\r\nB",
        LspPosition {
            line: 0,
            character: 2,
        },
        PositionEncoding::Utf8
    )
    .is_err());
}

#[test]
fn lsp0_server_lifecycle_and_projection_boundary() {
    let mut server = LspServer::new();
    initialize(&mut server, &["utf-16", "utf-8"]);
    assert_eq!(server.state(), ServerState::Running);
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didOpen".into(),
            params: Some(json!({
                "textDocument": {
                    "uri": "file:///sync.ddn",
                    "languageId": "ddn",
                    "version": 1,
                    "text": "한글"
                }
            })),
        })
        .unwrap();
    let request = server
        .begin_snapshot_request(
            json!(77),
            "file:///sync.ddn",
            ByteRange { start: 0, end: 3 },
        )
        .unwrap();
    assert!(server.publish_allowed(&request).is_ok());
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didClose".into(),
            params: Some(json!({"textDocument": {"uri": "file:///sync.ddn"}})),
        })
        .unwrap();
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didOpen".into(),
            params: Some(json!({
                "textDocument": {
                    "uri": "file:///sync.ddn",
                    "languageId": "ddn",
                    "version": 1,
                    "text": "다른"
                }
            })),
        })
        .unwrap();
    assert_eq!(
        server.publish_allowed(&request),
        Err(PublishRejection::SnapshotMismatch)
    );
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didChange".into(),
            params: Some(json!({
                "textDocument": {"uri": "file:///sync.ddn", "version": 2},
                "contentChanges": [{"text": "변경"}]
            })),
        })
        .unwrap();
    assert_eq!(
        server.publish_allowed(&request),
        Err(PublishRejection::VersionMismatch)
    );
    server.cancel_request(&json!(78));
    assert_eq!(
        server.begin_snapshot_request(
            json!(78),
            "file:///sync.ddn",
            ByteRange { start: 0, end: 3 }
        ),
        Err(PublishRejection::Cancelled)
    );
    server
        .handle_message(RpcMessage::Notification {
            method: "textDocument/didClose".into(),
            params: Some(json!({"textDocument": {"uri": "file:///sync.ddn"}})),
        })
        .unwrap();
    let shutdown = server
        .handle_message(RpcMessage::Request {
            id: json!(9),
            method: "shutdown".into(),
            params: None,
        })
        .unwrap();
    assert_eq!(shutdown.responses.len(), 1);
    let exit = server
        .handle_message(RpcMessage::Notification {
            method: "exit".into(),
            params: None,
        })
        .unwrap();
    assert_eq!(exit.exit_status, Some(0));
}
