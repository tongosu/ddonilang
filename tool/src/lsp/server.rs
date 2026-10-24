use super::definition::DefinitionError;
use super::diagnostics::{analyze_snapshot, DiagnosticError, DiagnosticReport, SOURCE_PROFILE_V25};
use super::document_highlight::{document_highlights_snapshot, DocumentHighlightError};
use super::document_store::{
    DocumentChange, DocumentStore, DocumentStoreError, SourceSnapshotIdentity,
};
use super::document_symbol::{document_symbols_snapshot, SymbolError as DocumentSymbolError};
use super::folding_range::{folding_ranges_snapshot, FoldingRangeError};
use super::hover::{hover_snapshot, HoverError};
use super::position::{ByteRange, LspPosition, LspRange, PositionEncoding};
use super::protocol::{RpcError, RpcMessage};
use super::references::SymbolError as ReferencesError;
use super::rename::SymbolError as RenameError;
use super::selection_range::{selection_ranges_snapshot, SelectionRangeError};
use super::semantic_tokens::{
    semantic_tokens_delta, semantic_tokens_snapshot, SemanticTokensError, SemanticTokensResolution,
    LEGEND_TOKEN_TYPES,
};
use super::signature_help::{inlay_hints_snapshot, signature_help_snapshot, SignatureHelpError};
use super::symbol::{
    call_hierarchy_call_at_position, collect_call_hierarchy_calls, collect_document_symbols,
    collect_local_references_for_identity, resolve_symbol_across_documents,
    resolve_type_definition_across_documents, DocumentSymbolResolution,
};
use crate::completion::{complete_snapshot, CompletionCandidate, CompletionCandidateKind};
use ddonirang_lang::{
    canonicalize, normalize, parse_v25, validate_no_legacy_boim_surface, NormalizationLevel,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

const MAX_WORKSPACE_SYMBOLS: usize = 1_000;

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
    Diagnostics(DiagnosticError),
    Definition(DefinitionError),
    DocumentSymbols(DocumentSymbolError),
    DocumentHighlights(DocumentHighlightError),
    FoldingRanges(FoldingRangeError),
    SelectionRanges(SelectionRangeError),
    References(ReferencesError),
    Rename(RenameError),
    SemanticTokens(SemanticTokensError),
    SignatureHelp(SignatureHelpError),
    Hover(HoverError),
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
            Self::Diagnostics(error) => write!(f, "diagnostics 분석 오류: {error}"),
            Self::Definition(error) => write!(f, "definition 분석 오류: {error}"),
            Self::DocumentSymbols(error) => write!(f, "document symbol 분석 오류: {error}"),
            Self::DocumentHighlights(error) => write!(f, "document highlight 분석 오류: {error}"),
            Self::FoldingRanges(error) => write!(f, "folding range 분석 오류: {error}"),
            Self::SelectionRanges(error) => write!(f, "selection range 분석 오류: {error}"),
            Self::References(error) => write!(f, "references 분석 오류: {error}"),
            Self::Rename(error) => write!(f, "rename 분석 오류: {error}"),
            Self::SemanticTokens(error) => write!(f, "semantic tokens 분석 오류: {error}"),
            Self::SignatureHelp(error) => write!(f, "signature help 분석 오류: {error}"),
            Self::Hover(error) => write!(f, "hover 분석 오류: {error}"),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticRequest {
    pub uri: String,
    pub version: i64,
    pub snapshot_identity: SourceSnapshotIdentity,
}

enum SemanticTokensRequestSnapshot {
    Missing,
    StaleVersion {
        requested: i64,
        current: i64,
        snapshot_identity: String,
    },
    StaleIdentity {
        requested: String,
        current: String,
    },
    Current(super::document_store::DocumentSnapshot),
}

pub struct LspServer {
    state: ServerState,
    position_encoding: Option<PositionEncoding>,
    documents: DocumentStore,
    semantic_tokens_cache: BTreeMap<String, SemanticTokensResolution>,
    cancelled: BTreeSet<String>,
    /// The client pulls `textDocument/diagnostic`; pushing the same report
    /// would show every diagnostic twice in one editor.
    pull_diagnostics: bool,
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
            semantic_tokens_cache: BTreeMap::new(),
            cancelled: BTreeSet::new(),
            pull_diagnostics: false,
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

    /// Transport entry. A message that fails is answered or reported to the
    /// client instead of ending the session; only `exit` ends the process.
    /// A document whose edit could not be applied is dropped, so its last
    /// diagnostics are never shown as the result of the newer text.
    pub fn handle_message_or_report(&mut self, message: RpcMessage) -> ServerEvent {
        let (request_id, method, uri) = match &message {
            RpcMessage::Request { id, method, params } => {
                (Some(id.clone()), method.clone(), document_uri(params.as_ref()))
            }
            RpcMessage::Notification { method, params } => {
                (None, method.clone(), document_uri(params.as_ref()))
            }
            RpcMessage::Response { .. } => return ServerEvent::empty(),
        };
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.handle_message(message)
        }));
        let (code, detail) = match outcome {
            Ok(Ok(event)) => return event,
            Ok(Err(ServerError::Exited)) => {
                return ServerEvent {
                    responses: Vec::new(),
                    exit_status: Some(1),
                }
            }
            Ok(Err(error @ ServerError::InvalidParams(_))) => (-32602, error.to_string()),
            Ok(Err(error)) => (-32603, error.to_string()),
            Err(panic) => (-32603, format!("서버 내부 오류: {}", panic_text(&panic))),
        };
        if let Some(id) = request_id {
            return error_event(id, RpcError::new(code, detail));
        }
        let mut responses = vec![log_message(1, format!("{method}: {detail}"))];
        if matches!(
            method.as_str(),
            "textDocument/didOpen" | "textDocument/didChange"
        ) {
            if let Some(uri) = uri {
                if self.documents.close(&uri).is_ok() {
                    self.semantic_tokens_cache.remove(&uri);
                }
                responses.extend(empty_diagnostics_event(&uri).responses);
                responses.push(show_message(
                    2,
                    "또니랑 언어 서버가 이 문서의 편집을 반영하지 못해 진단을 지웠습니다. 파일을 닫았다 다시 열어 주세요.",
                ));
            }
        }
        ServerEvent {
            responses,
            exit_status: None,
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

    pub fn begin_diagnostic_request(
        &self,
        uri: &str,
    ) -> Result<DiagnosticRequest, PublishRejection> {
        if self.state != ServerState::Running {
            return Err(PublishRejection::NotRunning);
        }
        let snapshot = self
            .documents
            .get(uri)
            .ok_or(PublishRejection::DocumentClosed)?;
        Ok(DiagnosticRequest {
            uri: uri.to_string(),
            version: snapshot.version,
            snapshot_identity: snapshot.identity.clone(),
        })
    }

    pub fn publish_diagnostic_request(
        &self,
        request: &DiagnosticRequest,
    ) -> Result<ServerEvent, PublishRejection> {
        if self.state != ServerState::Running {
            return Err(PublishRejection::NotRunning);
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
        self.diagnostics_event(snapshot)
            .map_err(|_| PublishRejection::InvalidRange)
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
            "textDocument/completion" => self.completion(id, params),
            "textDocument/definition" => self.definition(id, params),
            "textDocument/typeDefinition" => self.type_definition(id, params),
            "textDocument/documentSymbol" => self.document_symbols(id, params),
            "textDocument/foldingRange" => self.folding_ranges(id, params),
            "textDocument/selectionRange" => self.selection_ranges(id, params),
            "workspace/symbol" => self.workspace_symbol(id, params),
            "textDocument/formatting" => self.formatting(id, params),
            "textDocument/documentHighlight" => self.document_highlights(id, params),
            "textDocument/references" => self.references(id, params),
            "textDocument/prepareCallHierarchy" => self.prepare_call_hierarchy(id, params),
            "callHierarchy/incomingCalls" => self.incoming_calls(id, params),
            "callHierarchy/outgoingCalls" => self.outgoing_calls(id, params),
            "textDocument/prepareRename" => self.prepare_rename(id, params),
            "textDocument/rename" => self.rename(id, params),
            "textDocument/semanticTokens/full" => self.semantic_tokens_full(id, params),
            "textDocument/semanticTokens/full/delta" => self.semantic_tokens_full_delta(id, params),
            "textDocument/signatureHelp" => self.signature_help(id, params),
            "textDocument/inlayHint" => self.inlay_hints(id, params),
            "textDocument/hover" => self.hover(id, params),
            "textDocument/diagnostic" => self.text_document_diagnostic(id, params),
            "workspace/diagnostic" => self.workspace_diagnostic(id, params),
            _ => Ok(error_event(id, RpcError::new(-32601, "Method not found"))),
        }
    }

    fn definition(&mut self, id: Value, params: Option<Value>) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let position = parse_position(
            params
                .get("position")
                .ok_or_else(|| ServerError::InvalidParams("position이 없습니다".into()))?,
        )?;
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(success_event(id, Value::Null));
        };
        if let Some(requested_version) = document.get("version").and_then(Value::as_i64) {
            if requested_version != snapshot.version {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32801,
                        "definition document version이 stale합니다",
                        json!({
                            "requestedVersion": requested_version,
                            "currentVersion": snapshot.version,
                            "snapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        if let Some(requested_identity) = document.get("snapshotIdentity").and_then(Value::as_str) {
            if requested_identity != snapshot.identity.as_str() {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32802,
                        "definition snapshot identity가 stale합니다",
                        json!({
                            "requestedSnapshotIdentity": requested_identity,
                            "currentSnapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let documents = self
            .documents
            .iter()
            .map(|(_, document)| document)
            .collect::<Vec<_>>();
        let Some(symbol) =
            resolve_symbol_across_documents(snapshot, position, encoding, &documents)
                .map_err(ServerError::Definition)?
        else {
            return Ok(success_event(id, Value::Null));
        };
        let mut definitions = Vec::new();
        for (target_uri, target) in self.documents.iter() {
            for candidate in collect_document_symbols(target).map_err(ServerError::Definition)? {
                if candidate.identity != symbol.identity {
                    continue;
                }
                let source = target.text().map_err(ServerError::Document)?;
                let range = super::position::byte_range_to_lsp_range(
                    source,
                    candidate.selection_range,
                    encoding,
                )
                .map_err(|error| {
                    ServerError::InvalidParams(format!(
                        "definition range가 잘못되었습니다: {error}"
                    ))
                })?;
                definitions.push((
                    target_uri.to_string(),
                    range,
                    candidate.document_version,
                    candidate.snapshot_identity,
                    candidate.source_profile,
                ));
            }
        }
        if definitions.len() != 1 {
            return Ok(success_event(id, Value::Null));
        }
        let (target_uri, range, document_version, snapshot_identity, source_profile) =
            definitions.pop().expect("definition count checked");
        Ok(success_event(
            id,
            json!({
                "uri": target_uri,
                "range": {
                    "start": {"line": range.start.line, "character": range.start.character},
                    "end": {"line": range.end.line, "character": range.end.character},
                },
                "data": {
                    "definitionIdentity": symbol.identity,
                    "documentVersion": document_version,
                    "snapshotIdentity": snapshot_identity,
                    "sourceProfile": source_profile,
                },
            }),
        ))
    }

    fn type_definition(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let position = parse_position(
            params
                .get("position")
                .ok_or_else(|| ServerError::InvalidParams("position이 없습니다".into()))?,
        )?;
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(success_event(id, Value::Null));
        };
        if let Some(requested_version) = document.get("version").and_then(Value::as_i64) {
            if requested_version != snapshot.version {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32801,
                        "type definition document version이 stale합니다",
                        json!({
                            "requestedVersion": requested_version,
                            "currentVersion": snapshot.version,
                            "snapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        if let Some(requested_identity) = document.get("snapshotIdentity").and_then(Value::as_str) {
            if requested_identity != snapshot.identity.as_str() {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32802,
                        "type definition snapshot identity가 stale합니다",
                        json!({
                            "requestedSnapshotIdentity": requested_identity,
                            "currentSnapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let documents = self
            .documents
            .iter()
            .map(|(_, document)| document)
            .collect::<Vec<_>>();
        let Some(result) = resolve_type_definition_across_documents(
            snapshot,
            position,
            encoding,
            &documents,
        )
        .map_err(ServerError::Definition)? else {
            return Ok(success_event(id, Value::Null));
        };
        let source = self
            .documents
            .get(&result.uri)
            .ok_or_else(|| ServerError::InvalidParams("type definition target가 닫혔습니다".into()))?
            .text()
            .map_err(ServerError::Document)?;
        let range = super::position::byte_range_to_lsp_range(source, result.range, encoding)
            .map_err(|error| {
                ServerError::InvalidParams(format!("type definition range가 잘못되었습니다: {error}"))
            })?;
        Ok(success_event(
            id,
            json!({
                "uri": result.uri,
                "range": {
                    "start": {"line": range.start.line, "character": range.start.character},
                    "end": {"line": range.end.line, "character": range.end.character},
                },
                "data": {
                    "typeIdentity": result.identity,
                    "name": result.name,
                    "documentVersion": result.document_version,
                    "snapshotIdentity": result.snapshot_identity,
                    "sourceProfile": result.source_profile,
                },
            }),
        ))
    }

    fn document_symbols(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(success_event(id, json!([])));
        };
        if let Some(requested_version) = document.get("version").and_then(Value::as_i64) {
            if requested_version != snapshot.version {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32801,
                        "document symbol document version이 stale합니다",
                        json!({
                            "requestedVersion": requested_version,
                            "currentVersion": snapshot.version,
                            "snapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        if let Some(requested_identity) = document.get("snapshotIdentity").and_then(Value::as_str) {
            if requested_identity != snapshot.identity.as_str() {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32802,
                        "document symbol snapshot identity가 stale합니다",
                        json!({
                            "requestedSnapshotIdentity": requested_identity,
                            "currentSnapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let source = snapshot.text().map_err(ServerError::Document)?;
        let symbols = document_symbols_snapshot(snapshot)
            .map_err(ServerError::DocumentSymbols)?
            .into_iter()
            .map(|symbol| {
                let range = super::position::byte_range_to_lsp_range(
                    source,
                    symbol.full_range,
                    encoding,
                )
                .map_err(|error| {
                    ServerError::InvalidParams(format!(
                        "document symbol range가 잘못되었습니다: {error}"
                    ))
                })?;
                let selection_range = super::position::byte_range_to_lsp_range(
                    source,
                    symbol.selection_range,
                    encoding,
                )
                .map_err(|error| {
                    ServerError::InvalidParams(format!(
                        "document symbol selection range가 잘못되었습니다: {error}"
                    ))
                })?;
                Ok(json!({
                    "name": symbol.name,
                    "kind": symbol.kind,
                    "range": {
                        "start": {"line": range.start.line, "character": range.start.character},
                        "end": {"line": range.end.line, "character": range.end.character},
                    },
                    "selectionRange": {
                        "start": {"line": selection_range.start.line, "character": selection_range.start.character},
                        "end": {"line": selection_range.end.line, "character": selection_range.end.character},
                    },
                    "data": {
                        "definitionIdentity": symbol.identity,
                        "documentVersion": symbol.document_version,
                        "snapshotIdentity": symbol.snapshot_identity,
                        "sourceProfile": symbol.source_profile,
                    },
                }))
            })
            .collect::<Result<Vec<_>, ServerError>>()?;
        Ok(success_event(id, json!(symbols)))
    }

    fn folding_ranges(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(success_event(id, json!([])));
        };
        if let Some(requested_version) = document.get("version").and_then(Value::as_i64) {
            if requested_version != snapshot.version {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32801,
                        "folding range document version이 stale합니다",
                        json!({
                            "requestedVersion": requested_version,
                            "currentVersion": snapshot.version,
                            "snapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        if let Some(requested_identity) = document.get("snapshotIdentity").and_then(Value::as_str) {
            if requested_identity != snapshot.identity.as_str() {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32802,
                        "folding range snapshot identity가 stale합니다",
                        json!({
                            "requestedSnapshotIdentity": requested_identity,
                            "currentSnapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let ranges = folding_ranges_snapshot(snapshot, encoding)
            .map_err(ServerError::FoldingRanges)?
            .into_iter()
            .map(|folding| {
                json!({
                    "startLine": folding.range.start.line,
                    "startCharacter": folding.range.start.character,
                    "endLine": folding.range.end.line,
                    "endCharacter": folding.range.end.character,
                    "kind": folding.kind,
                })
            })
            .collect::<Vec<_>>();
        Ok(success_event(id, json!(ranges)))
    }

    fn selection_ranges(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let positions = params
            .get("positions")
            .and_then(Value::as_array)
            .ok_or_else(|| ServerError::InvalidParams("positions 배열이 없습니다".into()))?
            .iter()
            .map(parse_position)
            .collect::<Result<Vec<_>, _>>()?;
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(success_event(id, json!([])));
        };
        if let Some(requested_version) = document.get("version").and_then(Value::as_i64) {
            if requested_version != snapshot.version {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32801,
                        "selection range document version이 stale합니다",
                        json!({
                            "requestedVersion": requested_version,
                            "currentVersion": snapshot.version,
                            "snapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        if let Some(requested_identity) = document.get("snapshotIdentity").and_then(Value::as_str) {
            if requested_identity != snapshot.identity.as_str() {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32802,
                        "selection range snapshot identity가 stale합니다",
                        json!({
                            "requestedSnapshotIdentity": requested_identity,
                            "currentSnapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let ranges = selection_ranges_snapshot(snapshot, &positions, encoding)
            .map_err(ServerError::SelectionRanges)?
            .into_iter()
            .map(selection_range_value)
            .collect::<Vec<_>>();
        Ok(success_event(id, json!(ranges)))
    }

    fn workspace_symbol(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let query = params
            .get("query")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if let Some(requested_documents) = params.get("documents") {
            let Some(requested_documents) = requested_documents.as_array() else {
                return Ok(success_event(id, json!([])));
            };
            for requested in requested_documents {
                let Some(requested) = requested.as_object() else {
                    return Ok(success_event(id, json!([])));
                };
                let Some(uri) = requested.get("uri").and_then(Value::as_str) else {
                    return Ok(success_event(id, json!([])));
                };
                let Some(snapshot) = self.documents.get(uri) else {
                    return Ok(success_event(id, json!([])));
                };
                if requested.get("version").and_then(Value::as_i64) != Some(snapshot.version)
                    || requested.get("snapshotIdentity").and_then(Value::as_str)
                        != Some(snapshot.identity.as_str())
                {
                    return Ok(success_event(id, json!([])));
                }
            }
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let mut results = Vec::new();
        for (uri, snapshot) in self.documents.iter() {
            let source = snapshot.text().map_err(ServerError::Document)?;
            let symbols =
                document_symbols_snapshot(snapshot).map_err(ServerError::DocumentSymbols)?;
            for symbol in symbols {
                if !query.is_empty() && !symbol.name.contains(&query) {
                    continue;
                }
                let range = super::position::byte_range_to_lsp_range(
                    source,
                    symbol.selection_range,
                    encoding,
                )
                .map_err(|error| {
                    ServerError::InvalidParams(format!(
                        "workspace symbol range가 잘못되었습니다: {error}"
                    ))
                })?;
                results.push(json!({
                    "name": symbol.name,
                    "kind": symbol.kind,
                    "containerName": "ddn",
                    "location": {
                        "uri": uri,
                        "range": {
                            "start": {"line": range.start.line, "character": range.start.character},
                            "end": {"line": range.end.line, "character": range.end.character},
                        },
                    },
                    "data": {
                        "definitionIdentity": symbol.identity,
                        "documentVersion": symbol.document_version,
                        "snapshotIdentity": symbol.snapshot_identity,
                        "sourceProfile": symbol.source_profile,
                    },
                }));
                if results.len() >= MAX_WORKSPACE_SYMBOLS {
                    return Ok(success_event(id, json!(results)));
                }
            }
        }
        Ok(success_event(id, json!(results)))
    }

    fn formatting(&mut self, id: Value, params: Option<Value>) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(success_event(id, json!([])));
        };
        if snapshot.language_id != "ddn"
            || (document.contains_key("version")
                && document.get("version").and_then(Value::as_i64) != Some(snapshot.version))
            || (document.contains_key("snapshotIdentity")
                && document.get("snapshotIdentity").and_then(Value::as_str)
                    != Some(snapshot.identity.as_str()))
        {
            return Ok(success_event(id, json!([])));
        }
        let Ok(source) = snapshot.text() else {
            return Ok(success_event(id, json!([])));
        };
        if validate_no_legacy_boim_surface(source).is_err() {
            return Ok(success_event(id, json!([])));
        }
        let Ok(mut program) = parse_v25(source, uri) else {
            return Ok(success_event(id, json!([])));
        };
        if canonicalize(&mut program).is_err() {
            return Ok(success_event(id, json!([])));
        }
        let canonical = normalize(&program, NormalizationLevel::N1);
        if canonical == source {
            return Ok(success_event(id, json!([])));
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let Ok(range) = super::position::byte_range_to_lsp_range(
            source,
            ByteRange {
                start: 0,
                end: source.len(),
            },
            encoding,
        ) else {
            return Ok(success_event(id, json!([])));
        };
        Ok(success_event(
            id,
            json!([{
                "range": {
                    "start": {"line": range.start.line, "character": range.start.character},
                    "end": {"line": range.end.line, "character": range.end.character},
                },
                "newText": canonical,
            }]),
        ))
    }

    fn references(&mut self, id: Value, params: Option<Value>) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let position = parse_position(
            params
                .get("position")
                .ok_or_else(|| ServerError::InvalidParams("position이 없습니다".into()))?,
        )?;
        let include_declaration = params
            .get("context")
            .and_then(Value::as_object)
            .and_then(|context| context.get("includeDeclaration"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(success_event(id, json!([])));
        };
        if let Some(requested_version) = document.get("version").and_then(Value::as_i64) {
            if requested_version != snapshot.version {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32801,
                        "references document version이 stale합니다",
                        json!({
                            "requestedVersion": requested_version,
                            "currentVersion": snapshot.version,
                            "snapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        if let Some(requested_identity) = document.get("snapshotIdentity").and_then(Value::as_str) {
            if requested_identity != snapshot.identity.as_str() {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32802,
                        "references snapshot identity가 stale합니다",
                        json!({
                            "requestedSnapshotIdentity": requested_identity,
                            "currentSnapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let documents = self
            .documents
            .iter()
            .map(|(_, document)| document)
            .collect::<Vec<_>>();
        let Some(symbol) =
            resolve_symbol_across_documents(snapshot, position, encoding, &documents)
                .map_err(ServerError::References)?
        else {
            return Ok(success_event(id, json!([])));
        };
        let mut definition_count = 0;
        for document in &documents {
            definition_count += collect_document_symbols(document)
                .map_err(ServerError::References)?
                .into_iter()
                .filter(|candidate| candidate.identity == symbol.identity)
                .count();
        }
        if !symbol.identity.starts_with("definition:local:") || definition_count != 1 {
            return Ok(success_event(id, json!([])));
        }
        let mut locations = Vec::new();
        for (target_uri, target) in self.documents.iter() {
            let ranges = collect_local_references_for_identity(
                target,
                &symbol.identity,
                include_declaration,
            )
            .map_err(ServerError::References)?;
            let source = target.text().map_err(ServerError::Document)?;
            for range in ranges {
                let range = super::position::byte_range_to_lsp_range(source, range, encoding)
                    .map_err(|error| {
                        ServerError::InvalidParams(format!(
                            "references range가 잘못되었습니다: {error}"
                        ))
                    })?;
                locations.push(json!({
                    "uri": target_uri,
                    "range": {
                        "start": {"line": range.start.line, "character": range.start.character},
                        "end": {"line": range.end.line, "character": range.end.character},
                    },
                }));
            }
        }
        Ok(success_event(id, json!(locations)))
    }

    fn prepare_call_hierarchy(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let position = parse_position(
            params
                .get("position")
                .ok_or_else(|| ServerError::InvalidParams("position이 없습니다".into()))?,
        )?;
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(success_event(id, Value::Null));
        };
        if let Some(requested_version) = document.get("version").and_then(Value::as_i64) {
            if requested_version != snapshot.version {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32801,
                        "prepare call hierarchy document version이 stale합니다",
                        json!({
                            "requestedVersion": requested_version,
                            "currentVersion": snapshot.version,
                            "snapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        if let Some(requested_identity) = document.get("snapshotIdentity").and_then(Value::as_str) {
            if requested_identity != snapshot.identity.as_str() {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32802,
                        "prepare call hierarchy snapshot identity가 stale합니다",
                        json!({
                            "requestedSnapshotIdentity": requested_identity,
                            "currentSnapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let source = snapshot.text().map_err(ServerError::Document)?;
        let cursor = match super::position::position_to_byte_offset(source, position, encoding) {
            Ok(cursor) => cursor,
            Err(_) => return Ok(success_event(id, Value::Null)),
        };
        let symbols = self.call_hierarchy_symbols()?;
        let local_symbols = symbols
            .iter()
            .filter(|symbol| symbol.uri == uri)
            .filter(|symbol| {
                symbol.selection_range.start <= cursor && cursor < symbol.selection_range.end
            })
            .collect::<Vec<_>>();
        if local_symbols.len() == 1
            && symbols
                .iter()
                .filter(|candidate| candidate.identity == local_symbols[0].identity)
                .count()
                == 1
        {
            return Ok(success_event(
                id,
                self.call_hierarchy_item(local_symbols[0], encoding)?,
            ));
        }
        let Some(call) = call_hierarchy_call_at_position(snapshot, cursor)
            .map_err(ServerError::DocumentSymbols)?
        else {
            return Ok(success_event(id, Value::Null));
        };
        let identity = format!("definition:local:{}", call.callee_name);
        let targets = symbols
            .iter()
            .filter(|symbol| symbol.identity == identity)
            .collect::<Vec<_>>();
        if targets.len() != 1 {
            return Ok(success_event(id, Value::Null));
        }
        Ok(success_event(
            id,
            self.call_hierarchy_item(targets[0], encoding)?,
        ))
    }

    fn incoming_calls(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let Some(item) = params.get("item") else {
            return Ok(success_event(id, json!([])));
        };
        let Some(target) = self.call_hierarchy_payload(item) else {
            return Ok(success_event(id, json!([])));
        };
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let edges = self.call_hierarchy_edges()?;
        let mut grouped: Vec<(DocumentSymbolResolution, Vec<ByteRange>)> = Vec::new();
        for (caller, callee, range) in edges {
            if callee.identity != target.identity {
                continue;
            }
            if let Some((_, ranges)) = grouped
                .iter_mut()
                .find(|(candidate, _)| candidate.identity == caller.identity)
            {
                ranges.push(range);
            } else {
                grouped.push((caller, vec![range]));
            }
        }
        grouped.sort_by(|left, right| left.0.uri.cmp(&right.0.uri));
        let mut result = Vec::new();
        for (caller, ranges) in grouped {
            let from = self.call_hierarchy_item(&caller, encoding)?;
            let source = self
                .documents
                .get(&caller.uri)
                .ok_or_else(|| ServerError::Document(DocumentStoreError::NotOpen))?
                .text()
                .map_err(ServerError::Document)?;
            let ranges = ranges
                .into_iter()
                .map(|range| call_hierarchy_range(source, range, encoding))
                .collect::<Result<Vec<_>, _>>()?;
            result.push(json!({"from": from, "fromRanges": ranges}));
        }
        Ok(success_event(id, json!(result)))
    }

    fn outgoing_calls(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let Some(item) = params.get("item") else {
            return Ok(success_event(id, json!([])));
        };
        let Some(caller) = self.call_hierarchy_payload(item) else {
            return Ok(success_event(id, json!([])));
        };
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let edges = self.call_hierarchy_edges()?;
        let mut grouped: Vec<(DocumentSymbolResolution, Vec<ByteRange>)> = Vec::new();
        for (edge_caller, callee, range) in edges {
            if edge_caller.identity != caller.identity || edge_caller.uri != caller.uri {
                continue;
            }
            if let Some((_, ranges)) = grouped
                .iter_mut()
                .find(|(candidate, _)| candidate.identity == callee.identity)
            {
                ranges.push(range);
            } else {
                grouped.push((callee, vec![range]));
            }
        }
        grouped.sort_by(|left, right| left.0.uri.cmp(&right.0.uri));
        let source = self
            .documents
            .get(&caller.uri)
            .ok_or_else(|| ServerError::Document(DocumentStoreError::NotOpen))?
            .text()
            .map_err(ServerError::Document)?;
        let mut result = Vec::new();
        for (callee, ranges) in grouped {
            let to = self.call_hierarchy_item(&callee, encoding)?;
            let ranges = ranges
                .into_iter()
                .map(|range| call_hierarchy_range(source, range, encoding))
                .collect::<Result<Vec<_>, _>>()?;
            result.push(json!({"to": to, "fromRanges": ranges}));
        }
        Ok(success_event(id, json!(result)))
    }

    fn call_hierarchy_symbols(&self) -> Result<Vec<DocumentSymbolResolution>, ServerError> {
        let mut symbols = Vec::new();
        for (_, snapshot) in self.documents.iter() {
            symbols
                .extend(collect_document_symbols(snapshot).map_err(ServerError::DocumentSymbols)?);
        }
        symbols.sort_by(|left, right| {
            left.uri
                .cmp(&right.uri)
                .then(left.selection_range.start.cmp(&right.selection_range.start))
        });
        Ok(symbols)
    }

    fn call_hierarchy_edges(
        &self,
    ) -> Result<
        Vec<(
            DocumentSymbolResolution,
            DocumentSymbolResolution,
            ByteRange,
        )>,
        ServerError,
    > {
        let symbols = self.call_hierarchy_symbols()?;
        let mut counts = BTreeMap::<String, usize>::new();
        for symbol in &symbols {
            *counts.entry(symbol.identity.clone()).or_default() += 1;
        }
        let unique = symbols
            .into_iter()
            .filter(|symbol| counts.get(&symbol.identity) == Some(&1))
            .map(|symbol| (symbol.identity.clone(), symbol))
            .collect::<BTreeMap<_, _>>();
        let mut edges = Vec::new();
        for (_, snapshot) in self.documents.iter() {
            for call in
                collect_call_hierarchy_calls(snapshot).map_err(ServerError::DocumentSymbols)?
            {
                let Some(caller) = unique.get(&call.caller_identity) else {
                    continue;
                };
                let callee_identity = format!("definition:local:{}", call.callee_name);
                let Some(callee) = unique.get(&callee_identity) else {
                    continue;
                };
                edges.push((caller.clone(), callee.clone(), call.range));
            }
        }
        edges.sort_by(|left, right| {
            left.0
                .uri
                .cmp(&right.0.uri)
                .then(left.2.start.cmp(&right.2.start))
                .then(left.1.uri.cmp(&right.1.uri))
        });
        Ok(edges)
    }

    fn call_hierarchy_payload(&self, item: &Value) -> Option<DocumentSymbolResolution> {
        let object = item.as_object()?;
        let uri = object.get("uri")?.as_str()?;
        let data = object.get("data")?.as_object()?;
        if data.get("uri").and_then(Value::as_str) != Some(uri) {
            return None;
        }
        let identity = data
            .get("callableIdentity")
            .or_else(|| data.get("definitionIdentity"))
            .and_then(Value::as_str)?;
        let version = data.get("documentVersion")?.as_i64()?;
        let snapshot_identity = data.get("snapshotIdentity")?.as_str()?;
        let snapshot = self.documents.get(uri)?;
        if snapshot.version != version || snapshot.identity.as_str() != snapshot_identity {
            return None;
        }
        let symbols = self.call_hierarchy_symbols().ok()?;
        let matches = symbols
            .into_iter()
            .filter(|symbol| {
                symbol.uri == uri
                    && symbol.identity == identity
                    && symbol.document_version == version
                    && symbol.snapshot_identity == snapshot_identity
            })
            .collect::<Vec<_>>();
        (matches.len() == 1).then(|| matches.into_iter().next().expect("one match"))
    }

    fn call_hierarchy_item(
        &self,
        symbol: &DocumentSymbolResolution,
        encoding: PositionEncoding,
    ) -> Result<Value, ServerError> {
        let source = self
            .documents
            .get(&symbol.uri)
            .ok_or_else(|| ServerError::Document(DocumentStoreError::NotOpen))?
            .text()
            .map_err(ServerError::Document)?;
        let range = super::position::byte_range_to_lsp_range(source, symbol.full_range, encoding)
            .map_err(|error| {
            ServerError::InvalidParams(format!("call hierarchy range가 잘못되었습니다: {error}"))
        })?;
        let selection_range =
            super::position::byte_range_to_lsp_range(source, symbol.selection_range, encoding)
                .map_err(|error| {
                    ServerError::InvalidParams(format!(
                        "call hierarchy selection range가 잘못되었습니다: {error}"
                    ))
                })?;
        Ok(json!({
            "name": symbol.name,
            "kind": symbol.kind,
            "uri": symbol.uri,
            "range": lsp_range_value(range),
            "selectionRange": lsp_range_value(selection_range),
            "data": {
                "uri": symbol.uri,
                "callableIdentity": symbol.identity,
                "definitionIdentity": symbol.identity,
                "documentVersion": symbol.document_version,
                "snapshotIdentity": symbol.snapshot_identity,
                "sourceProfile": symbol.source_profile,
            }
        }))
    }

    fn document_highlights(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let position = parse_position(
            params
                .get("position")
                .ok_or_else(|| ServerError::InvalidParams("position이 없습니다".into()))?,
        )?;
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(success_event(id, json!([])));
        };
        if let Some(requested_version) = document.get("version").and_then(Value::as_i64) {
            if requested_version != snapshot.version {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32801,
                        "document highlight document version이 stale합니다",
                        json!({
                            "requestedVersion": requested_version,
                            "currentVersion": snapshot.version,
                            "snapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        if let Some(requested_identity) = document.get("snapshotIdentity").and_then(Value::as_str) {
            if requested_identity != snapshot.identity.as_str() {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32802,
                        "document highlight snapshot identity가 stale합니다",
                        json!({
                            "requestedSnapshotIdentity": requested_identity,
                            "currentSnapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let Some(result) = document_highlights_snapshot(snapshot, position, encoding)
            .map_err(ServerError::DocumentHighlights)?
        else {
            return Ok(success_event(id, json!([])));
        };
        let source = snapshot.text().map_err(ServerError::Document)?;
        let highlights = result
            .highlights
            .into_iter()
            .map(|highlight| {
                let range =
                    super::position::byte_range_to_lsp_range(source, highlight.range, encoding)
                        .map_err(|error| {
                            ServerError::InvalidParams(format!(
                                "document highlight range가 잘못되었습니다: {error}"
                            ))
                        })?;
                Ok(json!({
                    "range": {
                        "start": {"line": range.start.line, "character": range.start.character},
                        "end": {"line": range.end.line, "character": range.end.character},
                    },
                    "kind": highlight.kind,
                }))
            })
            .collect::<Result<Vec<_>, ServerError>>()?;
        Ok(success_event(id, json!(highlights)))
    }

    fn prepare_rename(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let (snapshot, offset) = match self.rename_cursor(&id, params.as_ref())? {
            Ok(found) => found,
            Err(event) => return Ok(event),
        };
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        match super::rename::prepare_rename(&snapshot, offset) {
            Ok(target) => {
                let source = snapshot.text().map_err(ServerError::Document)?;
                let range = byte_range_value(source, target.range, encoding)?;
                Ok(success_event(id, json!({"range": range, "placeholder": target.name})))
            }
            Err(refusal) => Ok(rename_refused_event(id, refusal)),
        }
    }

    fn rename(&mut self, id: Value, params: Option<Value>) -> Result<ServerEvent, ServerError> {
        let new_name = params
            .as_ref()
            .and_then(|p| p.get("newName"))
            .and_then(Value::as_str)
            .ok_or_else(|| ServerError::InvalidParams("newName이 없습니다".into()))?
            .to_string();
        let (snapshot, offset) = match self.rename_cursor(&id, params.as_ref())? {
            Ok(found) => found,
            Err(event) => return Ok(event),
        };
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let others: Vec<String> = self
            .documents
            .iter()
            .filter(|(uri, document)| *uri != snapshot.uri && document.language_id == "ddn")
            .filter_map(|(_, document)| document.text().ok().map(str::to_string))
            .collect();
        let other_refs: Vec<&str> = others.iter().map(String::as_str).collect();
        match super::rename::plan_rename(&snapshot, offset, &new_name, &other_refs) {
            Ok(plan) => {
                let source = snapshot.text().map_err(ServerError::Document)?;
                let edits = plan
                    .ranges
                    .iter()
                    .map(|range| {
                        Ok(json!({"range": byte_range_value(source, *range, encoding)?, "newText": plan.new_name}))
                    })
                    .collect::<Result<Vec<_>, ServerError>>()?;
                // Versioned edit: the client applies it only to this exact version.
                Ok(success_event(
                    id,
                    json!({"documentChanges": [{
                        "textDocument": {"uri": snapshot.uri, "version": snapshot.version},
                        "edits": edits,
                    }]}),
                ))
            }
            Err(refusal) => Ok(rename_refused_event(id, refusal)),
        }
    }

    /// The open snapshot and byte offset of a rename request, or the response
    /// that ends it (closed document, stale version or identity).
    fn rename_cursor(
        &self,
        id: &Value,
        params: Option<&Value>,
    ) -> Result<Result<(super::document_store::DocumentSnapshot, usize), ServerEvent>, ServerError> {
        let params = object_params(params.cloned())?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let position = parse_position(
            params
                .get("position")
                .ok_or_else(|| ServerError::InvalidParams("position이 없습니다".into()))?,
        )?;
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(Err(rename_refused_event(
                id.clone(),
                super::rename::RenameRefusal("이름을 바꿀 수 없습니다: 열려 있는 문서가 아닙니다.".into()),
            )));
        };
        if let Some(requested) = document.get("version").and_then(Value::as_i64) {
            if requested != snapshot.version {
                return Ok(Err(error_event(
                    id.clone(),
                    RpcError::with_data(
                        -32801,
                        "rename document version이 stale합니다",
                        json!({"requestedVersion": requested, "currentVersion": snapshot.version}),
                    ),
                )));
            }
        }
        if let Some(requested) = document.get("snapshotIdentity").and_then(Value::as_str) {
            if requested != snapshot.identity.as_str() {
                return Ok(Err(error_event(
                    id.clone(),
                    RpcError::with_data(
                        -32802,
                        "rename snapshot identity가 stale합니다",
                        json!({"requestedSnapshotIdentity": requested, "currentSnapshotIdentity": snapshot.identity.as_str()}),
                    ),
                )));
            }
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let source = snapshot.text().map_err(ServerError::Document)?;
        let offset = super::position::position_to_byte_offset(source, position, encoding)
            .map_err(|error| ServerError::InvalidParams(format!("rename position이 잘못되었습니다: {error}")))?;
        Ok(Ok((snapshot.clone(), offset)))
    }

    fn semantic_tokens_full(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let snapshot = match self.semantic_tokens_snapshot_for_request(document)? {
            SemanticTokensRequestSnapshot::Missing => {
                return Ok(success_event(id, json!({"data": []})));
            }
            SemanticTokensRequestSnapshot::StaleVersion {
                requested,
                current,
                snapshot_identity,
            } => {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32801,
                        "semantic tokens document version이 stale합니다",
                        json!({
                            "requestedVersion": requested,
                            "currentVersion": current,
                            "snapshotIdentity": snapshot_identity,
                        }),
                    ),
                ));
            }
            SemanticTokensRequestSnapshot::StaleIdentity { requested, current } => {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32802,
                        "semantic tokens snapshot identity가 stale합니다",
                        json!({
                            "requestedSnapshotIdentity": requested,
                            "currentSnapshotIdentity": current,
                        }),
                    ),
                ));
            }
            SemanticTokensRequestSnapshot::Current(snapshot) => snapshot,
        };
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let result =
            semantic_tokens_snapshot(&snapshot, encoding).map_err(ServerError::SemanticTokens)?;
        self.semantic_tokens_cache
            .insert(uri.to_string(), result.clone());
        Ok(success_event(id, semantic_tokens_full_value(&result)))
    }

    fn semantic_tokens_full_delta(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let previous_result_id = string_field(&params, "previousResultId")?;
        let snapshot = match self.semantic_tokens_snapshot_for_request(document)? {
            SemanticTokensRequestSnapshot::Missing => {
                return Ok(success_event(
                    id,
                    json!({"data": [], "resultId": Value::Null}),
                ));
            }
            SemanticTokensRequestSnapshot::StaleVersion {
                requested,
                current,
                snapshot_identity,
            } => {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32801,
                        "semantic tokens document version이 stale합니다",
                        json!({
                            "requestedVersion": requested,
                            "currentVersion": current,
                            "snapshotIdentity": snapshot_identity,
                        }),
                    ),
                ));
            }
            SemanticTokensRequestSnapshot::StaleIdentity { requested, current } => {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32802,
                        "semantic tokens snapshot identity가 stale합니다",
                        json!({
                            "requestedSnapshotIdentity": requested,
                            "currentSnapshotIdentity": current,
                        }),
                    ),
                ));
            }
            SemanticTokensRequestSnapshot::Current(snapshot) => snapshot,
        };
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let result =
            semantic_tokens_snapshot(&snapshot, encoding).map_err(ServerError::SemanticTokens)?;
        let previous = self.semantic_tokens_cache.get(uri).cloned();
        let response = if let Some(previous) = previous {
            if previous.result_id == previous_result_id {
                self.semantic_tokens_cache
                    .insert(uri.to_string(), result.clone());
                if previous.data == result.data {
                    json!({
                        "edits": [],
                        "resultId": result.result_id,
                        "dataVersion": result.document_version,
                        "snapshotIdentity": result.snapshot_identity,
                        "sourceProfile": result.source_profile,
                    })
                } else {
                    let edit = semantic_tokens_delta(&previous.data, &result.data);
                    json!({
                        "edits": [{
                            "start": edit.start,
                            "deleteCount": edit.delete_count,
                            "data": edit.data,
                        }],
                        "resultId": result.result_id,
                        "dataVersion": result.document_version,
                        "snapshotIdentity": result.snapshot_identity,
                        "sourceProfile": result.source_profile,
                    })
                }
            } else {
                self.semantic_tokens_cache
                    .insert(uri.to_string(), result.clone());
                semantic_tokens_full_value(&result)
            }
        } else {
            self.semantic_tokens_cache
                .insert(uri.to_string(), result.clone());
            semantic_tokens_full_value(&result)
        };
        Ok(success_event(id, response))
    }

    fn semantic_tokens_snapshot_for_request(
        &self,
        document: &serde_json::Map<String, Value>,
    ) -> Result<SemanticTokensRequestSnapshot, ServerError> {
        let uri = string_field(document, "uri")?;
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(SemanticTokensRequestSnapshot::Missing);
        };
        if let Some(requested_version) = document.get("version").and_then(Value::as_i64) {
            if requested_version != snapshot.version {
                return Ok(SemanticTokensRequestSnapshot::StaleVersion {
                    requested: requested_version,
                    current: snapshot.version,
                    snapshot_identity: snapshot.identity.as_str().to_string(),
                });
            }
        }
        if let Some(requested_identity) = document.get("snapshotIdentity").and_then(Value::as_str) {
            if requested_identity != snapshot.identity.as_str() {
                return Ok(SemanticTokensRequestSnapshot::StaleIdentity {
                    requested: requested_identity.to_string(),
                    current: snapshot.identity.as_str().to_string(),
                });
            }
        }
        Ok(SemanticTokensRequestSnapshot::Current(snapshot.clone()))
    }

    fn signature_help(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let position = parse_position(
            params
                .get("position")
                .ok_or_else(|| ServerError::InvalidParams("position이 없습니다".into()))?,
        )?;
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(success_event(id, Value::Null));
        };
        if let Some(requested_version) = document.get("version").and_then(Value::as_i64) {
            if requested_version != snapshot.version {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32801,
                        "signature help document version이 stale합니다",
                        json!({
                            "requestedVersion": requested_version,
                            "currentVersion": snapshot.version,
                            "snapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        if let Some(requested_identity) = document.get("snapshotIdentity").and_then(Value::as_str) {
            if requested_identity != snapshot.identity.as_str() {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32802,
                        "signature help snapshot identity가 stale합니다",
                        json!({
                            "requestedSnapshotIdentity": requested_identity,
                            "currentSnapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let result = signature_help_snapshot(snapshot, position, encoding)
            .map_err(ServerError::SignatureHelp)?;
        let Some(result) = result else {
            return Ok(success_event(id, Value::Null));
        };
        let parameters = result
            .parameters
            .into_iter()
            .map(|parameter| {
                let mut value = json!({"label": parameter.label});
                if let Some(param_pin_id) = parameter.param_pin_id {
                    value["data"] = json!({"paramPinId": param_pin_id});
                }
                value
            })
            .collect::<Vec<_>>();
        Ok(success_event(
            id,
            json!({
                "signatures": [{
                    "label": result.label,
                    "parameters": parameters,
                }],
                "activeSignature": 0,
                "activeParameter": result.active_parameter,
                "data": {
                    "definitionIdentity": result.identity,
                    "documentVersion": result.document_version,
                    "snapshotIdentity": result.snapshot_identity,
                    "sourceProfile": result.source_profile,
                },
            }),
        ))
    }

    fn inlay_hints(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(success_event(id, json!([])));
        };
        if let Some(requested_version) = document.get("version").and_then(Value::as_i64) {
            if requested_version != snapshot.version {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32801,
                        "inlay hint document version이 stale합니다",
                        json!({
                            "requestedVersion": requested_version,
                            "currentVersion": snapshot.version,
                            "snapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        if let Some(requested_identity) = document.get("snapshotIdentity").and_then(Value::as_str) {
            if requested_identity != snapshot.identity.as_str() {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32802,
                        "inlay hint snapshot identity가 stale합니다",
                        json!({
                            "requestedSnapshotIdentity": requested_identity,
                            "currentSnapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let hints = inlay_hints_snapshot(snapshot, encoding)
            .map_err(ServerError::SignatureHelp)?
            .into_iter()
            .map(|hint| {
                json!({
                    "position": lsp_position_value(hint.position),
                    "label": hint.label,
                    "kind": 2,
                    "paddingRight": true,
                    "data": {
                        "definitionIdentity": hint.definition_identity,
                        "paramPinId": hint.param_pin_id,
                        "documentVersion": hint.document_version,
                        "snapshotIdentity": hint.snapshot_identity,
                        "sourceProfile": hint.source_profile,
                    },
                })
            })
            .collect::<Vec<_>>();
        Ok(success_event(id, json!(hints)))
    }

    fn hover(&mut self, id: Value, params: Option<Value>) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let position = parse_position(
            params
                .get("position")
                .ok_or_else(|| ServerError::InvalidParams("position이 없습니다".into()))?,
        )?;
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(success_event(id, Value::Null));
        };
        if let Some(requested_version) = document.get("version").and_then(Value::as_i64) {
            if requested_version != snapshot.version {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32801,
                        "hover document version이 stale합니다",
                        json!({
                            "requestedVersion": requested_version,
                            "currentVersion": snapshot.version,
                            "snapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        if let Some(requested_identity) = document.get("snapshotIdentity").and_then(Value::as_str) {
            if requested_identity != snapshot.identity.as_str() {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32802,
                        "hover snapshot identity가 stale합니다",
                        json!({
                            "requestedSnapshotIdentity": requested_identity,
                            "currentSnapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let result = hover_snapshot(snapshot, position, encoding).map_err(ServerError::Hover)?;
        let result = result
            .map(|result| {
                let source = snapshot.text().map_err(ServerError::Document)?;
                let range =
                    super::position::byte_range_to_lsp_range(source, result.range, encoding)
                        .map_err(|error| {
                            ServerError::InvalidParams(format!(
                                "hover range가 잘못되었습니다: {error}"
                            ))
                        })?;
                Ok(json!({
                    "contents": {
                        "kind": "markdown",
                        "value": result.signature,
                    },
                    "range": {
                        "start": {"line": range.start.line, "character": range.start.character},
                        "end": {"line": range.end.line, "character": range.end.character},
                    },
                    "data": {
                        "definitionIdentity": result.identity,
                        "definitionKind": result.kind,
                        "documentVersion": result.document_version,
                        "snapshotIdentity": result.snapshot_identity,
                        "sourceProfile": result.source_profile,
                    },
                }))
            })
            .transpose()?;
        Ok(success_event(id, result.unwrap_or(Value::Null)))
    }

    fn completion(&mut self, id: Value, params: Option<Value>) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let position = parse_position(
            params
                .get("position")
                .ok_or_else(|| ServerError::InvalidParams("position이 없습니다".into()))?,
        )?;
        let snapshot = self.documents.get(uri).ok_or_else(|| {
            ServerError::InvalidParams("completion 문서가 열려 있지 않습니다".into())
        })?;
        if let Some(requested_version) = document.get("version").and_then(Value::as_i64) {
            if requested_version != snapshot.version {
                return Ok(error_event(
                    id,
                    RpcError::with_data(
                        -32801,
                        "completion document version이 stale합니다",
                        json!({
                            "requestedVersion": requested_version,
                            "currentVersion": snapshot.version,
                            "snapshotIdentity": snapshot.identity.as_str(),
                        }),
                    ),
                ));
            }
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let result = complete_snapshot(snapshot, position, encoding).map_err(|error| {
            ServerError::InvalidParams(format!(
                "completion position/source가 잘못되었습니다: {error}"
            ))
        })?;
        let source = snapshot.text().map_err(ServerError::Document)?;
        let items = result
            .candidates
            .iter()
            .map(|candidate| completion_item(candidate, source, encoding))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(success_event(
            id,
            json!({
                "isIncomplete": false,
                "items": items,
                "data": {
                    "documentVersion": result.document_version,
                    "snapshotIdentity": result.snapshot_identity,
                }
            }),
        ))
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
                let uri = self.did_open(params)?;
                self.pushed_diagnostics_event(&uri)
            }
            "textDocument/didChange" => {
                let uri = self.did_change(params)?;
                self.pushed_diagnostics_event(&uri)
            }
            "textDocument/didClose" => {
                let uri = self.did_close(params)?;
                Ok(empty_diagnostics_event(&uri))
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
        self.pull_diagnostics = params
            .as_ref()
            .and_then(|params| params.pointer("/capabilities/textDocument/diagnostic"))
            .is_some_and(Value::is_object);
        self.state = ServerState::Running;
        Ok(success_event(
            id,
            json!({
                "capabilities": {
                    "positionEncoding": encoding.as_lsp_name(),
                    "textDocumentSync": {"openClose": true, "change": 1},
                    "completionProvider": {"resolveProvider": false},
                    "definitionProvider": true,
                    "typeDefinitionProvider": true,
                    "documentSymbolProvider": true,
                    "foldingRangeProvider": true,
                    "selectionRangeProvider": true,
                    "workspaceSymbolProvider": true,
                    "documentFormattingProvider": true,
                    "documentHighlightProvider": true,
                    "referencesProvider": true,
                    "callHierarchyProvider": true,
                    "renameProvider": {"prepareProvider": true},
                    "semanticTokensProvider": {
                        "full": {"delta": true},
                        "legend": {
                            "tokenTypes": LEGEND_TOKEN_TYPES,
                            "tokenModifiers": [],
                        },
                    },
                    "signatureHelpProvider": {"triggerCharacters": ["(", ","]},
                    "inlayHintProvider": true,
                    "hoverProvider": true,
                    "diagnosticProvider": {
                        "identifier": "ddonirang-v25",
                        "interFileDependencies": false,
                        "workspaceDiagnostics": false
                    }
                },
                "serverInfo": {"name": "ddonirang-lsp0", "version": "0.1"}
            }),
        ))
    }

    fn did_open(&mut self, params: Option<Value>) -> Result<String, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let language_id = source_language_id(uri, string_field(document, "languageId")?);
        let version = integer_field(document, "version")?;
        let text = string_field(document, "text")?;
        self.documents
            .open(uri, language_id, version, text.as_bytes().to_vec())
            .map_err(ServerError::Document)?;
        Ok(uri.to_string())
    }

    fn did_change(&mut self, params: Option<Value>) -> Result<String, ServerError> {
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
            .map(|_| uri.to_string())
            .map_err(ServerError::Document)
    }

    fn did_close(&mut self, params: Option<Value>) -> Result<String, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        self.documents.close(uri).map_err(ServerError::Document)?;
        self.semantic_tokens_cache.remove(uri);
        Ok(uri.to_string())
    }

    fn pushed_diagnostics_event(&self, uri: &str) -> Result<ServerEvent, ServerError> {
        if self.pull_diagnostics {
            return Ok(ServerEvent::empty());
        }
        self.current_diagnostics_event(uri)
    }

    fn current_diagnostics_event(&self, uri: &str) -> Result<ServerEvent, ServerError> {
        let snapshot = self
            .documents
            .get(uri)
            .ok_or_else(|| ServerError::Document(DocumentStoreError::NotOpen))?;
        self.diagnostics_event(snapshot)
    }

    fn diagnostics_event(
        &self,
        snapshot: &super::document_store::DocumentSnapshot,
    ) -> Result<ServerEvent, ServerError> {
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let report = analyze_snapshot(snapshot, encoding).map_err(ServerError::Diagnostics)?;
        Ok(ServerEvent {
            responses: vec![diagnostics_notification(snapshot, &report)],
            exit_status: None,
        })
    }

    fn text_document_diagnostic(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let document = text_document(&params)?;
        let uri = string_field(document, "uri")?;
        let Some(snapshot) = self.documents.get(uri) else {
            return Ok(error_event(
                id,
                RpcError::new(-32602, "diagnostic 문서가 열려 있지 않습니다"),
            ));
        };
        if let Some(response) = stale_diagnostic_snapshot(id.clone(), document, snapshot) {
            return Ok(response);
        }
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let report = analyze_snapshot(snapshot, encoding).map_err(ServerError::Diagnostics)?;
        let items = diagnostic_values(snapshot, &report);
        let result_id = diagnostic_result_id(snapshot, &items);
        let previous_result_id = params.get("previousResultId").and_then(Value::as_str);
        let result = if previous_result_id == Some(result_id.as_str()) {
            json!({"kind": "unchanged", "resultId": result_id})
        } else {
            json!({
                "kind": "full",
                "items": items,
                "resultId": result_id,
            })
        };
        Ok(success_event(id, result))
    }

    fn workspace_diagnostic(
        &mut self,
        id: Value,
        params: Option<Value>,
    ) -> Result<ServerEvent, ServerError> {
        let params = object_params(params)?;
        let previous = workspace_previous_result_ids(&params)?;
        let encoding = self.position_encoding.ok_or(ServerError::NotInitialized)?;
        let mut items = Vec::new();
        for (uri, snapshot) in self.documents.iter() {
            if self.cancelled.contains(&request_key(&id)) {
                self.cancelled.remove(&request_key(&id));
                return Ok(error_event(id, RpcError::new(-32800, "Request cancelled")));
            }
            let report = analyze_snapshot(snapshot, encoding).map_err(ServerError::Diagnostics)?;
            let diagnostics = diagnostic_values(snapshot, &report);
            let result_id = diagnostic_result_id(snapshot, &diagnostics);
            if previous.get(uri).map(String::as_str) == Some(result_id.as_str()) {
                items.push(json!({
                    "uri": uri,
                    "version": report.document_version,
                    "kind": "unchanged",
                    "resultId": result_id,
                }));
            } else {
                items.push(json!({
                    "uri": uri,
                    "version": report.document_version,
                    "kind": "full",
                    "items": diagnostics,
                    "resultId": result_id,
                }));
            }
        }
        if self.cancelled.remove(&request_key(&id)) {
            return Ok(error_event(id, RpcError::new(-32800, "Request cancelled")));
        }
        Ok(success_event(id, json!({"items": items})))
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
    // Every client supports UTF-16; it is the protocol default.
    Ok(PositionEncoding::Utf16)
}

/// Maps the registered editor language id to the document kind the analysis
/// owners accept. `ddonilang` covers several file kinds, so the exact URI
/// extension decides; only `.ddn` text is DDN source. Other kinds stay open
/// but are not analyzed.
fn source_language_id<'a>(uri: &str, language_id: &'a str) -> &'a str {
    if language_id != "ddonilang" {
        return language_id;
    }
    let path = uri.split(['?', '#']).next().unwrap_or(uri);
    let name = path.rsplit('/').next().unwrap_or(path);
    if name.ends_with(".ddn") {
        "ddn"
    } else if name.ends_with(".ddl") || name.ends_with(".%EC%98%AE%EA%B9%80") {
        "ddl"
    } else {
        "unsupported"
    }
}

fn document_uri(params: Option<&Value>) -> Option<String> {
    params?
        .pointer("/textDocument/uri")?
        .as_str()
        .map(str::to_string)
}

fn panic_text(panic: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(text) = panic.downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = panic.downcast_ref::<String>() {
        text.clone()
    } else {
        "알 수 없는 오류".to_string()
    }
}

fn log_message(kind: u8, message: String) -> RpcMessage {
    RpcMessage::Notification {
        method: "window/logMessage".into(),
        params: Some(json!({"type": kind, "message": message})),
    }
}

fn show_message(kind: u8, message: &str) -> RpcMessage {
    RpcMessage::Notification {
        method: "window/showMessage".into(),
        params: Some(json!({"type": kind, "message": message})),
    }
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

fn lsp_range_value(range: LspRange) -> Value {
    json!({
        "start": {"line": range.start.line, "character": range.start.character},
        "end": {"line": range.end.line, "character": range.end.character},
    })
}

fn lsp_position_value(position: LspPosition) -> Value {
    json!({"line": position.line, "character": position.character})
}

fn selection_range_value(selection: Option<super::selection_range::SelectionRange>) -> Value {
    let Some(selection) = selection else {
        return Value::Null;
    };
    json!({
        "range": lsp_range_value(selection.range),
        "parent": selection.parent.map(|parent| selection_range_value(Some(*parent))),
    })
}

fn call_hierarchy_range(
    source: &str,
    range: ByteRange,
    encoding: PositionEncoding,
) -> Result<Value, ServerError> {
    let range =
        super::position::byte_range_to_lsp_range(source, range, encoding).map_err(|error| {
            ServerError::InvalidParams(format!(
                "call hierarchy call range가 잘못되었습니다: {error}"
            ))
        })?;
    Ok(lsp_range_value(range))
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

/// A refused rename: LSP RequestFailed, whose message the client shows.
fn rename_refused_event(id: Value, refusal: super::rename::RenameRefusal) -> ServerEvent {
    error_event(id, RpcError::new(-32803, refusal.0))
}

fn byte_range_value(source: &str, range: ByteRange, encoding: PositionEncoding) -> Result<Value, ServerError> {
    let range = super::position::byte_range_to_lsp_range(source, range, encoding)
        .map_err(|error| ServerError::InvalidParams(format!("rename range가 잘못되었습니다: {error}")))?;
    Ok(json!({
        "start": {"line": range.start.line, "character": range.start.character},
        "end": {"line": range.end.line, "character": range.end.character},
    }))
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

fn semantic_tokens_full_value(result: &SemanticTokensResolution) -> Value {
    json!({
        "data": result.data,
        "resultId": result.result_id,
        "dataVersion": result.document_version,
        "snapshotIdentity": result.snapshot_identity,
        "sourceProfile": result.source_profile,
    })
}

fn diagnostic_values(
    snapshot: &super::document_store::DocumentSnapshot,
    report: &DiagnosticReport,
) -> Vec<Value> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| {
            json!({
                "range": {
                    "start": {
                        "line": diagnostic.range.start.line,
                        "character": diagnostic.range.start.character,
                    },
                    "end": {
                        "line": diagnostic.range.end.line,
                        "character": diagnostic.range.end.character,
                    },
                },
                "severity": diagnostic.severity,
                "code": &diagnostic.code,
                "message": &diagnostic.message,
                "source": "ddonirang-v25",
                "data": {
                    "documentVersion": report.document_version,
                    "sourceProfile": report.source_profile,
                    "snapshotIdentity": snapshot.identity.as_str(),
                },
            })
        })
        .collect()
}

fn diagnostic_result_id(
    snapshot: &super::document_store::DocumentSnapshot,
    diagnostics: &[Value],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"ddonirang.lsp0.pull-diagnostics.v1\0");
    hasher.update((snapshot.uri.len() as u64).to_le_bytes());
    hasher.update(snapshot.uri.as_bytes());
    hasher.update(snapshot.version.to_le_bytes());
    hasher.update(snapshot.identity.as_str().as_bytes());
    hasher.update(serde_json::to_vec(diagnostics).expect("diagnostics JSON serialization"));
    format!(
        "sha256:{:x}:{}",
        hasher.finalize(),
        snapshot.identity.as_str()
    )
}

fn stale_diagnostic_snapshot(
    id: Value,
    document: &serde_json::Map<String, Value>,
    snapshot: &super::document_store::DocumentSnapshot,
) -> Option<ServerEvent> {
    if let Some(requested_version) = document.get("version").and_then(Value::as_i64) {
        if requested_version != snapshot.version {
            return Some(error_event(
                id,
                RpcError::with_data(
                    -32801,
                    "diagnostic document version이 stale합니다",
                    json!({
                        "requestedVersion": requested_version,
                        "currentVersion": snapshot.version,
                        "snapshotIdentity": snapshot.identity.as_str(),
                    }),
                ),
            ));
        }
    }
    if let Some(requested_identity) = document.get("snapshotIdentity").and_then(Value::as_str) {
        if requested_identity != snapshot.identity.as_str() {
            return Some(error_event(
                id,
                RpcError::with_data(
                    -32802,
                    "diagnostic snapshot identity가 stale합니다",
                    json!({
                        "requestedSnapshotIdentity": requested_identity,
                        "currentSnapshotIdentity": snapshot.identity.as_str(),
                        "currentVersion": snapshot.version,
                    }),
                ),
            ));
        }
    }
    None
}

fn workspace_previous_result_ids(
    params: &serde_json::Map<String, Value>,
) -> Result<BTreeMap<String, String>, ServerError> {
    let Some(previous) = params.get("previousResultIds") else {
        return Ok(BTreeMap::new());
    };
    let previous = previous.as_array().ok_or_else(|| {
        ServerError::InvalidParams("previousResultIds가 배열이 아닙니다".into())
    })?;
    let mut result = BTreeMap::new();
    for item in previous {
        let item = item.as_object().ok_or_else(|| {
            ServerError::InvalidParams("previousResultIds 항목이 object가 아닙니다".into())
        })?;
        let uri = item
            .get("uri")
            .and_then(Value::as_str)
            .ok_or_else(|| ServerError::InvalidParams("previousResultIds.uri가 없습니다".into()))?;
        let value = item
            .get("value")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ServerError::InvalidParams("previousResultIds.value가 없습니다".into())
            })?;
        result.insert(uri.to_string(), value.to_string());
    }
    Ok(result)
}

fn diagnostics_notification(
    snapshot: &super::document_store::DocumentSnapshot,
    report: &DiagnosticReport,
) -> RpcMessage {
    let diagnostics = diagnostic_values(snapshot, report);
    RpcMessage::Notification {
        method: "textDocument/publishDiagnostics".into(),
        params: Some(json!({
            "uri": &snapshot.uri,
            "version": report.document_version,
            "diagnostics": diagnostics,
            "data": {
                "documentVersion": report.document_version,
                "sourceProfile": report.source_profile,
                "snapshotIdentity": snapshot.identity.as_str(),
            },
        })),
    }
}

fn empty_diagnostics_event(uri: &str) -> ServerEvent {
    ServerEvent {
        responses: vec![RpcMessage::Notification {
            method: "textDocument/publishDiagnostics".into(),
            params: Some(json!({
                "uri": uri,
                "version": Value::Null,
                "diagnostics": [],
                "data": {
                    "documentVersion": Value::Null,
                    "sourceProfile": SOURCE_PROFILE_V25,
                },
            })),
        }],
        exit_status: None,
    }
}

fn completion_item(
    candidate: &CompletionCandidate,
    source: &str,
    encoding: PositionEncoding,
) -> Result<Value, ServerError> {
    let range =
        super::position::byte_range_to_lsp_range(source, candidate.replacement_range, encoding)
            .map_err(|error| {
                ServerError::InvalidParams(format!("completion range가 잘못되었습니다: {error}"))
            })?;
    let kind = match candidate.candidate_kind {
        CompletionCandidateKind::LocalDefinition => 6,
        CompletionCandidateKind::StdlibCallable => 3,
    };
    Ok(json!({
        "label": candidate.label,
        "kind": kind,
        "detail": candidate.canonical_detail,
        "textEdit": {
            "range": {
                "start": {"line": range.start.line, "character": range.start.character},
                "end": {"line": range.end.line, "character": range.end.character}
            },
            "newText": candidate.insert_text,
        },
        "data": {"definitionIdentity": candidate.definition_identity}
    }))
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
