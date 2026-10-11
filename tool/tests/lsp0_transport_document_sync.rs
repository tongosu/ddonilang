use ddonirang_tool::lsp::protocol::{
    MAX_CONTENT_LENGTH_BYTES, MAX_HEADER_SECTION_BYTES, MAX_PENDING_FRAME_BYTES,
};
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

fn raw_frame(message: &RpcMessage) -> Vec<u8> {
    encode_message(message).unwrap()
}

fn header_with_section_len(content_length: usize, section_len: usize) -> Vec<u8> {
    let prefix = format!("Content-Length: {content_length}\r\n").into_bytes();
    assert!(prefix.len() + 2 <= section_len);
    let padding_len = section_len - prefix.len() - 2;
    assert!(padding_len >= 4);
    let mut header = prefix;
    header.extend_from_slice(b"X:");
    header.extend(std::iter::repeat(b'a').take(padding_len - 4));
    header.extend_from_slice(b"\r\n\r\n");
    assert_eq!(header.len(), section_len);
    header
}

fn max_sized_valid_body() -> Vec<u8> {
    let prefix = br#"{"jsonrpc":"2.0","method":"initialized","params":{},"padding":""#;
    let suffix = br#""}"#;
    let padding_len = MAX_CONTENT_LENGTH_BYTES - prefix.len() - suffix.len();
    let mut body = Vec::with_capacity(MAX_CONTENT_LENGTH_BYTES);
    body.extend_from_slice(prefix);
    body.extend(std::iter::repeat(b'a').take(padding_len));
    body.extend_from_slice(suffix);
    assert_eq!(body.len(), MAX_CONTENT_LENGTH_BYTES);
    body
}

#[test]
fn lsp_st01_partial_header_buffers_without_error() {
    let raw = raw_frame(&RpcMessage::Notification {
        method: "initialized".into(),
        params: Some(json!({})),
    });
    let mut decoder = FrameDecoder::default();
    assert!(decoder.push(&raw[..3]).unwrap().is_empty());
    assert_eq!(decoder.push(&raw[3..]).unwrap().len(), 1);
}

#[test]
fn lsp_st02_partial_body_buffers_without_error() {
    let raw = raw_frame(&RpcMessage::Notification {
        method: "initialized".into(),
        params: Some(json!({})),
    });
    let separator = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap()
        + 4;
    let mut decoder = FrameDecoder::default();
    assert!(decoder.push(&raw[..separator + 1]).unwrap().is_empty());
    assert_eq!(decoder.push(&raw[separator + 1..]).unwrap().len(), 1);
}

#[test]
fn lsp_st03_utf8_split_is_accepted_after_full_body_arrives() {
    let raw = raw_frame(&RpcMessage::Notification {
        method: "한글🙂".into(),
        params: Some(json!({"text": "한글🙂"})),
    });
    let separator = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap()
        + 4;
    let split = raw[separator..]
        .iter()
        .position(|byte| *byte >= 0x80)
        .map(|offset| separator + offset + 1)
        .unwrap();
    let mut decoder = FrameDecoder::default();
    assert!(decoder.push(&raw[..split]).unwrap().is_empty());
    let messages = decoder.push(&raw[split..]).unwrap();
    assert!(matches!(&messages[0], RpcMessage::Notification { method, .. } if method == "한글🙂"));
}

#[test]
fn lsp_st04_multiple_complete_frames_are_returned_in_order() {
    let first = raw_frame(&RpcMessage::Notification {
        method: "initialized".into(),
        params: Some(json!({})),
    });
    let second = raw_frame(&RpcMessage::Notification {
        method: "textDocument/didClose".into(),
        params: Some(json!({})),
    });
    let mut payload = first;
    payload.extend_from_slice(&second);
    let messages = FrameDecoder::default().push(&payload).unwrap();
    assert!(
        matches!(&messages[0], RpcMessage::Notification { method, .. } if method == "initialized")
    );
    assert!(
        matches!(&messages[1], RpcMessage::Notification { method, .. } if method == "textDocument/didClose")
    );
}

#[test]
fn lsp_st05_finish_rejects_buffered_partial_frame() {
    let mut decoder = FrameDecoder::default();
    decoder.push(b"Content-Length: 10\r\n\r\n{}").unwrap();
    assert_eq!(decoder.finish(), Err(FrameError::TruncatedBody));
}

#[test]
fn lsp_st06_completed_frame_leaves_zero_buffered_bytes() {
    let raw = raw_frame(&RpcMessage::Notification {
        method: "initialized".into(),
        params: Some(json!({})),
    });
    let mut decoder = FrameDecoder::default();
    assert_eq!(decoder.push(&raw).unwrap().len(), 1);
    assert_eq!(decoder.buffered_len(), 0);
}

fn deterministic_initialize_shutdown_frames() -> Vec<Vec<u8>> {
    let mut server = LspServer::new();
    let initialize = server
        .handle_message(RpcMessage::Request {
            id: json!(1),
            method: "initialize".into(),
            params: Some(json!({
                "capabilities": {"general": {"positionEncodings": ["utf-16", "utf-8"]}}
            })),
        })
        .unwrap();
    let shutdown = server
        .handle_message(RpcMessage::Request {
            id: json!(2),
            method: "shutdown".into(),
            params: None,
        })
        .unwrap();
    initialize
        .responses
        .into_iter()
        .chain(shutdown.responses)
        .map(|response| encode_message(&response).unwrap())
        .collect()
}

#[test]
fn lsp_st07_initialize_shutdown_response_frames_remain_deterministic() {
    assert_eq!(
        deterministic_initialize_shutdown_frames(),
        deterministic_initialize_shutdown_frames()
    );
}

#[test]
fn lsp_st08_exit_status_is_returned_without_waiting_for_eof() {
    let mut server = LspServer::new();
    initialize(&mut server, &["utf-16"]);
    server
        .handle_message(RpcMessage::Request {
            id: json!(2),
            method: "shutdown".into(),
            params: None,
        })
        .unwrap();
    let exit = server
        .handle_message(RpcMessage::Notification {
            method: "exit".into(),
            params: None,
        })
        .unwrap();
    assert_eq!(exit.exit_status, Some(0));
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
fn lsp_rb02_header_section_at_exact_limit_is_accepted() {
    let body = raw_frame(&RpcMessage::Notification {
        method: "initialized".into(),
        params: Some(json!({})),
    });
    let separator = body
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap();
    let body = body[separator + 4..].to_vec();
    let mut input = header_with_section_len(body.len(), MAX_HEADER_SECTION_BYTES);
    input.extend_from_slice(&body);

    let mut decoder = FrameDecoder::default();
    assert_eq!(decoder.push(&input).unwrap().len(), 1);
    assert_eq!(decoder.buffered_len(), 0);
}

#[test]
fn lsp_rb03_body_at_exact_limit_is_accepted() {
    let body = max_sized_valid_body();
    let mut input = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    input.extend_from_slice(&body);

    let mut decoder = FrameDecoder::default();
    assert_eq!(decoder.push(&input).unwrap().len(), 1);
    assert_eq!(decoder.buffered_len(), 0);
}

#[test]
fn lsp_rb06_header_without_terminator_over_limit_is_poisoned() {
    let input = vec![b'x'; MAX_HEADER_SECTION_BYTES + 1];
    let mut decoder = FrameDecoder::default();
    assert_eq!(
        decoder.push(&input).unwrap_err(),
        FrameError::HeaderTooLarge
    );
    assert_eq!(decoder.buffered_len(), 0);
    assert_eq!(
        decoder.push(b"Content-Length: 0\r\n\r\n").unwrap_err(),
        FrameError::DecoderLimitExceeded
    );
}

#[test]
fn lsp_rb07_header_section_over_limit_is_rejected() {
    let mut input = vec![b'x'; MAX_HEADER_SECTION_BYTES - 3];
    input.extend_from_slice(b"\r\n\r\n");
    let mut decoder = FrameDecoder::default();
    assert_eq!(
        decoder.push(&input).unwrap_err(),
        FrameError::HeaderTooLarge
    );
    assert_eq!(decoder.buffered_len(), 0);
}

#[test]
fn lsp_rb08_content_length_over_limit_is_rejected_before_body() {
    let input = format!("Content-Length: {}\r\n\r\n", MAX_CONTENT_LENGTH_BYTES + 1);
    let mut decoder = FrameDecoder::default();
    assert_eq!(
        decoder.push(input.as_bytes()).unwrap_err(),
        FrameError::ContentLengthTooLarge
    );
    assert_eq!(decoder.buffered_len(), 0);
}

#[test]
fn lsp_rb09_checked_frame_length_overflow_fails_closed() {
    let input = format!("Content-Length: {}\r\n\r\n", usize::MAX);
    let mut decoder = FrameDecoder::default();
    assert_eq!(
        decoder.push(input.as_bytes()).unwrap_err(),
        FrameError::FrameTooLarge
    );
    assert_eq!(decoder.buffered_len(), 0);
}

#[test]
fn lsp_rb10_pending_frame_stays_within_total_bound() {
    let header = header_with_section_len(MAX_CONTENT_LENGTH_BYTES, MAX_HEADER_SECTION_BYTES);
    let body = vec![b'a'; MAX_CONTENT_LENGTH_BYTES - 1];
    let mut input = header;
    input.extend_from_slice(&body);

    let mut decoder = FrameDecoder::default();
    assert!(decoder.push(&input).unwrap().is_empty());
    assert_eq!(decoder.buffered_len(), MAX_PENDING_FRAME_BYTES - 1);
    assert!(decoder.finish().is_err());
}

#[test]
fn lsp_rb11_limit_rejection_returns_no_message_and_clears_buffer() {
    let mut decoder = FrameDecoder::default();
    let input = format!("Content-Length: {}\r\n\r\n", MAX_CONTENT_LENGTH_BYTES + 1);
    assert!(decoder.push(input.as_bytes()).is_err());
    assert_eq!(decoder.buffered_len(), 0);
}

#[test]
fn lsp_rb12_limit_rejection_cannot_recover_with_a_normal_frame() {
    let mut decoder = FrameDecoder::default();
    let oversized = format!("Content-Length: {}\r\n\r\n", MAX_CONTENT_LENGTH_BYTES + 1);
    assert_eq!(
        decoder.push(oversized.as_bytes()).unwrap_err(),
        FrameError::ContentLengthTooLarge
    );
    assert_eq!(
        decoder
            .push(&raw_frame(&RpcMessage::Notification {
                method: "initialized".into(),
                params: Some(json!({})),
            }))
            .unwrap_err(),
        FrameError::DecoderLimitExceeded
    );
    assert_eq!(decoder.buffered_len(), 0);
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
