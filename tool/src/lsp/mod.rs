pub mod definition;
#[path = "../diagnostics.rs"]
pub mod diagnostics;
pub mod document_highlight;
pub mod document_store;
pub mod document_symbol;
pub mod folding_range;
#[path = "../hover.rs"]
pub mod hover;
pub mod position;
pub mod protocol;
pub mod references;
pub mod rename;
pub mod selection_range;
pub mod semantic_tokens;
pub mod server;
pub mod signature_help;
pub mod symbol;

pub use crate::completion::{
    complete_snapshot, CompletionCandidate, CompletionCandidateKind, CompletionError,
    CompletionResult,
};
pub use definition::{definition_snapshot, DefinitionError, DefinitionResult};
pub use document_store::{DocumentChange, DocumentSnapshot, DocumentStore, SourceSnapshotIdentity};
pub use hover::{hover_snapshot, HoverError, HoverResult};
pub use position::{
    byte_offset_to_position, byte_range_to_lsp_range, position_to_byte_offset, range_to_byte_range,
    ByteRange, LspPosition, LspRange, PositionEncoding, PositionError,
};
pub use protocol::{encode_message, FrameDecoder, FrameError, RpcError, RpcMessage};
pub use server::{
    DiagnosticRequest, LspServer, PublishRejection, ServerEvent, ServerState, SnapshotRequest,
};
