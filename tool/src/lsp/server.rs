use super::document_store::{
    DocumentChange, DocumentStore, DocumentStoreError, SourceSnapshotIdentity,
};
use super::position::{ByteRange, LspPosition, LspRange, PositionEncoding};
use super::protocol::{RpcError, RpcMessage};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    PreInitialize,
    Running,
    ShutdownRequested,
    Exited,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerEvent {
    pub responses: Vec<RpcMessage>,
    pub exit_status: Option<i32>,
}

impl ServerEvent {
    fn empty() -> Self {
        Self {
            responses: Vec::new(),
            exit_status: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerError {
    InvalidParams(String),
    Document(DocumentStoreError),
    NotInitialized,
    ShuttingDown,
    Exited,
    UnexpectedResponse,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParams(message) => write!(f, "잘못된 LSP parameters: {message}"),
            Self::Document(error) => write!(f, "문서 상태 오류: {error}"),
            Self::NotInitialized => f.write_str("initialize 이전에는 요청을 처리할 수 없습니다"),
            Self::ShuttingDown => f.write_str("shutdown 이후에는 요청을 처리할 수 없습니다"),
            Self::Exited => f.write_str("종료된 session입니다"),
            Self::UnexpectedResponse => f.write_str("server 입력으로 response를 받을 수 없습니다"),
        }
    }
}

impl std::error::Error for ServerError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishRejection {
    NotRunning,
    DocumentClosed,
    VersionMismatch,
    SnapshotMismatch,
    Cancelled,
    InvalidRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotRequest {
    pub request_id: Value,
    pub uri: String,
    pub version: i64,
    pub snapshot_identity: SourceSnapshotIdentity,
    pub byte_range: ByteRange,
}

pub struct LspServer {
    state: ServerState,
    position_encoding: Option<PositionEncoding>,
    documents: DocumentStore,
    cancelled: BTreeSet<String>,
}

impl Default for LspServer {
    fn default() -> Self {
        Self::new()
    }
}

impl LspServer {
    pub fn new() -> Self {
        Self {
            state: ServerState::PreInitialize,
            position_encoding: None,
            documents: DocumentStore::new(),
            cancelled: BTreeSet::new(),
        }
    }

    pub fn state(&self) -> ServerState {
        self.state
    }

    pub fn position_encoding(&self) -> Option<PositionEncoding> {
        self.position_encoding
    }

    pub fn documents(&self) -> &DocumentStore {
        &self.documents
    }

    pub fn handle_message(&mut self, message: RpcMessage) -> Result<ServerEvent, ServerError> {
        match message {
            RpcMessage::Request { id, method, params } => self.handle_request(id, &method, params),
            RpcMessage::Notification { method, params } => {
                self.handle_notification(&method, params)
            }
            RpcMessage::Response { .. } => Err(ServerError::UnexpectedResponse),
        }
    }

    pub fn begin_snapshot_request(
        &self,
        request_id: Value,
        uri: &str,
        byte_range: ByteRange,
    ) -> Result<SnapshotRequest, PublishRejection> {
        if self.state != ServerState::Running {
            return Err(PublishRejection::NotRunning);
        }
        if self.cancelled.contains(&request_key(&request_id)) {
            return Err(PublishRejection::Cancelled);
        }
        let snapshot = self
            .documents
            .get(uri)
            .ok_or(PublishRejection::DocumentClosed)?;
        if !snapshot.byte_range_is_valid(byte_range) {
            return Err(PublishRejection::InvalidRange);
        }
        Ok(SnapshotRequest {
            request_id,
            uri: uri.to_string(),
            version: snapshot.version,
            snapshot_identity: snapshot.identity.clone(),
            byte_range,
        })
    }

    pub fn publish_allowed(&self, request: &SnapshotRequest) -> Result<(), PublishRejection> {
        if self.state != ServerState::Running {
            return Err(PublishRejection::NotRunning);
        }
        if self.cancelled.contains(&request_key(&request.request_id)) {
            return Err(PublishRejection::Cancelled);
        }
        let snapshot = self
            .documents
            .get(&request.uri)
            .ok_or(PublishRejection::DocumentClosed)?;
        if snapshot.version != request.version {
            return Err(PublishRejection::VersionMismatch);
        }
        if snapshot.identity != request.snapshot_identity {
            return Err(PublishRejection::SnapshotMismatch);
        }
        if !snapshot.byte_range_is_valid(request.byte_range) {
            return Err(PublishRejection::InvalidRange);
        }
        Ok(())
    }

    pub fn cancel_request(&mut self, request_id: &Value) {
        self.cancelled.insert(request_key(request_id));
    }

    fn handle_request(
        &mut self,
        id: Value,
        method: &str,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        if self.state == ServerState::Exited {
            return Err(ServerError::Exited);
        }
        if method == "initialize" {
            return self.initialize(id, params);
        }
        if let Some(response) = self.state_error_response(&id) {
            return Ok(response);
        }
        if self.cancelled.remove(&request_key(&id)) {
            return Ok(error_event(id, RpcError::new(-32800, "Request cancelled")));
        }
        match method {
            "shutdown" => {
                self.state = ServerState::ShutdownRequested;
                Ok(success_event(id, Value::Null))
            }
            _ => Ok(error_event(id, RpcError::new(-32601, "Method not found"))),
        }
    }

    fn handle_notification(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        if method == "exit" {
            let status = if self.state == ServerState::ShutdownRequested {
                0
            } else {
                1
            };
            self.state = ServerState::Exited;
            return Ok(ServerEvent {
                responses: Vec::new(),
                exit_status: Some(status),
            });
        }
        if self.state == ServerState::Exited {
            return Err(ServerError::Exited);
        }
        if method == "$/cancelRequest" {
            if self.state != ServerState::Running {
                return Err(self.state_error());
            }
            let params = object_params(params)?;
            let id = params
                .get("id")
                .cloned()
                .ok_or_else(|| ServerError::InvalidParams("cancel id가 없습니다".into()))?;
            self.cancel_request(&id);
            return Ok(ServerEvent::empty());
        }
        if self.state != ServerState::Running {
            return Err(self.state_error());
        }
        match method {
            "initialized" => Ok(ServerEvent::empty()),
            "textDocument/didOpen" => {
                self.did_open(params)?;
                Ok(ServerEvent::empty())
            }
            "textDocument/didChange" => {
                self.did_change(params)?;
                Ok(ServerEvent::empty())
            }
            "textDocument/didClose" => {
                self.did_close(params)?;
                Ok(ServerEvent::empty())
            }
            _ => Ok(ServerEvent::empty()),
        }
    }

    fn initialize(&mut self, id: Value, params: Option<Value>) -> Result<ServerEvent, ServerError> {
        if self.state != ServerState::PreInitialize {
            return Ok(error_event(
                id,
                RpcError::new(-32600, "이미 initialize 되었습니다"),
            ));
        }
        let encoding = negotiate_position_encoding(params.as_ref())?;
        self.position_encoding = Some(encoding);
        self.state = ServerState::Running;
        Ok(success_event(
            id,
            json!({
                "capabilities": {
                    "positionEncoding": encoding.as_lsp_name(),
                    "textDocumentSync": {"openClose": true, "change": 1}
                },
                "serverInfo": {"name": "ddonirang-lsp0", "version": "0.1"}
            }),
        ))
    }

    fn did_open(&mut self, params: Option<Value>) -> Result<(), ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let language_id = string_field(document, "languageId")?;
        let version = integer_field(document, "version")?;
        let text = string_field(document, "text")?;
        self.documents
            .open(uri, language_id, version, text.as_bytes().to_vec())
            .map(|_| ())
            .map_err(ServerError::Document)
    }

    fn did_change(&mut self, params: Option<Value>) -> Result<(), ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let version = integer_field(document, "version")?;
        let changes = params
            .get("contentChanges")
            .and_then(Value::as_array)
            .ok_or_else(|| ServerError::InvalidParams("contentChanges 배열이 없습니다".into()))?;
        let mut parsed = Vec::with_capacity(changes.len());
        for change in changes {
            let change = change.as_object().ok_or_else(|| {
                ServerError::InvalidParams("contentChanges 항목이 object가 아닙니다".into())
            })?;
            let text = change
                .get("text")
                .and_then(Value::as_str)
                .ok_or_else(|| ServerError::InvalidParams("change text가 없습니다".into()))?;
            let range = change.get("range").map(parse_range).transpose()?;
            parsed.push(DocumentChange {
                range,
                text: text.to_string(),
                range_snapshot: None,
            });
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        self.documents
            .apply_changes(uri, version, &parsed, encoding)
            .map(|_| ())
            .map_err(ServerError::Document)
    }

    fn did_close(&mut self, params: Option<Value>) -> Result<(), ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        self.documents
            .close(uri)
            .map(|_| ())
            .map_err(ServerError::Document)
    }

    fn state_error_response(&self, id: &Value) -> Option<ServerEvent> {
        match self.state {
            ServerState::PreInitialize => Some(error_event(
                id.clone(),
                RpcError::new(-32002, "Server not initialized"),
            )),
            ServerState::ShutdownRequested => Some(error_event(
                id.clone(),
                RpcError::new(-32600, "Server is shutting down"),
            )),
            ServerState::Running => None,
            ServerState::Exited => Some(error_event(
                id.clone(),
                RpcError::new(-32600, "Server has exited"),
            )),
        }
    }

    fn state_error(&self) -> ServerError {
        match self.state {
            ServerState::PreInitialize => ServerError::NotInitialized,
            ServerState::ShutdownRequested => ServerError::ShuttingDown,
            ServerState::Exited => ServerError::Exited,
            ServerState::Running => ServerError::NotInitialized,
        }
    }
}

fn negotiate_position_encoding(params: Option<&Value>) -> Result<PositionEncoding, ServerError> {
    let Some(params) = params else {
        return Ok(PositionEncoding::Utf16);
    };
    let object = params.as_object().ok_or_else(|| {
        ServerError::InvalidParams("initialize params가 object가 아닙니다".into())
    })?;
    let Some(encodings) = object
        .get("capabilities")
        .and_then(Value::as_object)
        .and_then(|capabilities| capabilities.get("general"))
        .and_then(Value::as_object)
        .and_then(|general| general.get("positionEncodings"))
    else {
        return Ok(PositionEncoding::Utf16);
    };
    let encodings = encodings
        .as_array()
        .ok_or_else(|| ServerError::InvalidParams("positionEncodings가 배열이 아닙니다".into()))?;
    for encoding in encodings {
        if let Some(name) = encoding.as_str() {
            if let Some(encoding) = PositionEncoding::from_lsp_name(name) {
                return Ok(encoding);
            }
        }
    }
    Err(ServerError::InvalidParams(
        "지원하는 position encoding이 없습니다".into(),
    ))
}

fn object_params(params: Option<Value>) -> Result<serde_json::Map<String, Value>, ServerError> {
    params
        .ok_or_else(|| ServerError::InvalidParams("params가 없습니다".into()))?
        .as_object()
        .cloned()
        .ok_or_else(|| ServerError::InvalidParams("params가 object가 아닙니다".into()))
}

fn text_document<'a>(
    params: &'a serde_json::Map<String, Value>,
) -> Result<&'a serde_json::Map<String, Value>, ServerError> {
    params
        .get("textDocument")
        .and_then(Value::as_object)
        .ok_or_else(|| ServerError::InvalidParams("textDocument object가 없습니다".into()))
}

fn string_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    name: &str,
) -> Result<&'a str, ServerError> {
    object
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| ServerError::InvalidParams(format!("{name} 문자열이 없습니다")))
}

fn integer_field(object: &serde_json::Map<String, Value>, name: &str) -> Result<i64, ServerError> {
    object
        .get(name)
        .and_then(Value::as_i64)
        .ok_or_else(|| ServerError::InvalidParams(format!("{name} 정수가 없습니다")))
}

fn parse_range(value: &Value) -> Result<LspRange, ServerError> {
    let object = value
        .as_object()
        .ok_or_else(|| ServerError::InvalidParams("range가 object가 아닙니다".into()))?;
    let start = parse_position(
        object
            .get("start")
            .ok_or_else(|| ServerError::InvalidParams("range.start가 없습니다".into()))?,
    )?;
    let end = parse_position(
        object
            .get("end")
            .ok_or_else(|| ServerError::InvalidParams("range.end가 없습니다".into()))?,
    )?;
    Ok(LspRange { start, end })
}

fn parse_position(value: &Value) -> Result<LspPosition, ServerError> {
    let object = value
        .as_object()
        .ok_or_else(|| ServerError::InvalidParams("position이 object가 아닙니다".into()))?;
    let line = object
        .get("line")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| ServerError::InvalidParams("position.line이 올바르지 않습니다".into()))?;
    let character = object
        .get("character")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| {
            ServerError::InvalidParams("position.character가 올바르지 않습니다".into())
        })?;
    Ok(LspPosition { line, character })
}

fn request_key(id: &Value) -> String {
    id.to_string()
}

fn success_event(id: Value, result: Value) -> ServerEvent {
    ServerEvent {
        responses: vec![RpcMessage::Response {
            id,
            result: Ok(result),
        }],
        exit_status: None,
    }
}

fn error_event(id: Value, error: RpcError) -> ServerEvent {
    ServerEvent {
        responses: vec![RpcMessage::Response {
            id,
            result: Err(error),
        }],
        exit_status: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn init(server: &mut LspServer) {
        let event = server
            .handle_message(RpcMessage::Request {
                id: json!(1),
                method: "initialize".into(),
                params: Some(json!({
                    "capabilities": {"general": {"positionEncodings": ["utf-16", "utf-8"]}}
                })),
            })
            .unwrap();
        assert_eq!(event.responses.len(), 1);
        assert_eq!(server.state(), ServerState::Running);
        assert_eq!(server.position_encoding(), Some(PositionEncoding::Utf16));
    }

    #[test]
    fn lifecycle_rejects_illegal_transitions() {
        let mut server = LspServer::new();
        let event = server
            .handle_message(RpcMessage::Request {
                id: json!(9),
                method: "shutdown".into(),
                params: None,
            })
            .unwrap();
        assert!(matches!(
            event.responses[0],
            RpcMessage::Response {
                result: Err(RpcError { code: -32002, .. }),
                ..
            }
        ));
        init(&mut server);
        server
            .handle_message(RpcMessage::Request {
                id: json!(2),
                method: "shutdown".into(),
                params: None,
            })
            .unwrap();
        assert_eq!(server.state(), ServerState::ShutdownRequested);
        let event = server
            .handle_message(RpcMessage::Request {
                id: json!(3),
                method: "anything".into(),
                params: None,
            })
            .unwrap();
        assert!(matches!(
            event.responses[0],
            RpcMessage::Response {
                result: Err(RpcError { code: -32600, .. }),
                ..
            }
        ));
        let exit = server
            .handle_message(RpcMessage::Notification {
                method: "exit".into(),
                params: None,
            })
            .unwrap();
        assert_eq!(exit.exit_status, Some(0));
        assert_eq!(server.state(), ServerState::Exited);
    }

    #[test]
    fn stale_and_cancelled_consumer_results_are_rejected() {
        let mut server = LspServer::new();
        init(&mut server);
        server
            .handle_message(RpcMessage::Notification {
                method: "textDocument/didOpen".into(),
                params: Some(json!({
                    "textDocument": {"uri": "file:///a.ddn", "languageId": "ddn", "version": 1, "text": "한글"}
                })),
            })
            .unwrap();
        let request = server
            .begin_snapshot_request(json!(7), "file:///a.ddn", ByteRange { start: 0, end: 3 })
            .unwrap();
        server
            .handle_message(RpcMessage::Notification {
                method: "textDocument/didChange".into(),
                params: Some(json!({
                    "textDocument": {"uri": "file:///a.ddn", "version": 2},
                    "contentChanges": [{"text": "변경"}]
                })),
            })
            .unwrap();
        assert_eq!(
            server.publish_allowed(&request),
            Err(PublishRejection::VersionMismatch)
        );
        server.cancel_request(&json!(8));
        assert_eq!(
            server.begin_snapshot_request(
                json!(8),
                "file:///a.ddn",
                ByteRange { start: 0, end: 3 }
            ),
            Err(PublishRejection::Cancelled)
        );
    }
}
