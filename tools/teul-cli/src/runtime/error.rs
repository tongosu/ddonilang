use crate::lang::span::Span;

#[derive(Debug)]
pub enum RuntimeError {
    Undefined { path: String, span: Span },
    InvalidPath { path: String, span: Span },
    MathDivZero { span: Span },
    MathDomain { message: &'static str, span: Span },
    TypeMismatch { expected: &'static str, span: Span },
    TypeMismatchDetail { expected: &'static str, actual: String, span: Span },
    IndexOutOfRange { span: Span },
    UnitMismatch { span: Span },
    UnitUnknown { unit: String, span: Span },
    FormulaParse { message: String, span: Span },
    FormulaUndefined { name: String, span: Span },
    FormulaIdentNotAscii1 { span: Span },
    FormulaExtUnsupported { name: String, span: Span },
    Template { message: String, span: Span },
    Pack { message: String, span: Span },
    BreakOutsideLoop { span: Span },
    ReturnOutsideSeed { span: Span },
    OpenSiteUnknown { span: Span },
    OpenDenied { open_kind: String, span: Span },
    OpenReplayMissing { open_kind: String, site_id: String, key: String, span: Span },
    OpenReplayInvalid { message: String, span: Span },
    OpenLogTamper { message: String, span: Span },
    OpenIo { message: String, span: Span },
}

impl RuntimeError {
    pub fn code(&self) -> &'static str {
        match self {
            RuntimeError::Undefined { .. } => "E_RUNTIME_UNDEFINED",
            RuntimeError::InvalidPath { .. } => "E_RUNTIME_INVALID_PATH",
            RuntimeError::MathDivZero { .. } => "E_MATH_DIV_ZERO",
            RuntimeError::MathDomain { .. } => "E_MATH_DOMAIN",
            RuntimeError::TypeMismatch { .. } => "E_RUNTIME_TYPE_MISMATCH",
            RuntimeError::TypeMismatchDetail { .. } => "E_RUNTIME_TYPE_MISMATCH",
            RuntimeError::IndexOutOfRange { .. } => "FATAL:CHARIM_INDEX_OUT_OF_RANGE",
            RuntimeError::UnitMismatch { .. } => "E_UNIT_MISMATCH",
            RuntimeError::UnitUnknown { .. } => "E_UNIT_UNKNOWN",
            RuntimeError::FormulaParse { .. } => "E_FORMULA_PARSE",
            RuntimeError::FormulaUndefined { .. } => "E_FORMULA_UNDEFINED",
            RuntimeError::FormulaIdentNotAscii1 { .. } => "FATAL:FORMULA_IDENT_NOT_ASCII1",
            RuntimeError::FormulaExtUnsupported { .. } => "FATAL:FORMULA_EVAL_EXT_UNSUPPORTED",
            RuntimeError::Template { .. } => "E_TEMPLATE",
            RuntimeError::Pack { .. } => "E_PACK",
            RuntimeError::BreakOutsideLoop { .. } => "E_RUNTIME_BREAK_OUTSIDE_LOOP",
            RuntimeError::ReturnOutsideSeed { .. } => "E_RUNTIME_RETURN_OUTSIDE_SEED",
            RuntimeError::OpenSiteUnknown { .. } => "E_OPEN_SITE_UNKNOWN",
            RuntimeError::OpenDenied { .. } => "E_OPEN_DENIED",
            RuntimeError::OpenReplayMissing { .. } => "E_OPEN_REPLAY_MISS",
            RuntimeError::OpenReplayInvalid { .. } => "E_OPEN_LOG_PARSE",
            RuntimeError::OpenLogTamper { .. } => "E_OPEN_LOG_TAMPER",
            RuntimeError::OpenIo { .. } => "E_OPEN_IO",
        }
    }
}
