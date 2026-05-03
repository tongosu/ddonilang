use crate::lang::span::Span;

#[derive(Debug)]
pub enum RuntimeError {
    Undefined {
        path: String,
        span: Span,
    },
    InvalidPath {
        path: String,
        span: Span,
    },
    JeOutsideImja {
        span: Span,
    },
    MathDivZero {
        span: Span,
    },
    MathDomain {
        message: &'static str,
        span: Span,
    },
    TypeMismatch {
        expected: &'static str,
        span: Span,
    },
    TypeMismatchDetail {
        expected: &'static str,
        actual: String,
        span: Span,
    },
    LifecycleTargetUnknown {
        verb: &'static str,
        target: String,
        family: &'static str,
        span: Span,
    },
    LifecycleTargetArity {
        verb: &'static str,
        got: usize,
        span: Span,
    },
    LifecycleTargetFamilyConflict {
        verb: &'static str,
        target: String,
        hint_family: &'static str,
        declared_family: &'static str,
        span: Span,
    },
    LifecycleTargetFamilyAmbiguous {
        verb: &'static str,
        target: String,
        span: Span,
    },
    StringIndexOutOfRange {
        span: Span,
    },
    IndexOutOfRange {
        span: Span,
    },
    UnitMismatch {
        span: Span,
    },
    UnitUnknown {
        unit: String,
        span: Span,
    },
    FormulaParse {
        message: String,
        span: Span,
    },
    FormulaUndefined {
        name: String,
        span: Span,
    },
    FormulaIdentNotAscii1 {
        span: Span,
    },
    FormulaExtUnsupported {
        name: String,
        span: Span,
    },
    Template {
        message: String,
        span: Span,
    },
    Pack {
        message: String,
        span: Span,
    },
    BreakOutsideLoop {
        span: Span,
    },
    ContinueOutsideForeach {
        span: Span,
    },
    ReturnOutsideSeed {
        span: Span,
    },
    ProofIncomplete {
        span: Span,
    },
    OpenSiteUnknown {
        span: Span,
    },
    OpenDenied {
        open_kind: String,
        span: Span,
    },
    OpenReplayMissing {
        open_kind: String,
        site_id: String,
        key: String,
        span: Span,
    },
    OpenReplayInvalid {
        message: String,
        span: Span,
    },
    OpenLogTamper {
        message: String,
        span: Span,
    },
    OpenIo {
        message: String,
        span: Span,
    },
    RegexFlagsInvalid {
        flags: String,
        span: Span,
    },
    RegexPatternInvalid {
        message: String,
        span: Span,
    },
    RegexReplacementInvalid {
        replacement: String,
        message: String,
        span: Span,
    },
    InputKeyMissing {
        span: Span,
    },
    MapDotKeyMissing {
        key: String,
        span: Span,
    },
    EcoDivergenceDetected {
        tick: u64,
        name: String,
        delta: String,
        threshold: String,
        span: Span,
    },
    SfcIdentityViolation {
        tick: u64,
        name: String,
        delta: String,
        threshold: String,
        span: Span,
    },
}

impl RuntimeError {
    fn lifecycle_target_unknown_code(verb: &str) -> &'static str {
        match verb {
            "시작하기" => "E_RUNTIME_LIFECYCLE_START_TARGET_UNKNOWN",
            "넘어가기" => "E_RUNTIME_LIFECYCLE_NEXT_TARGET_UNKNOWN",
            "불러오기" => "E_RUNTIME_LIFECYCLE_CALL_TARGET_UNKNOWN",
            _ => "E_RUNTIME_LIFECYCLE_TARGET_UNKNOWN",
        }
    }

    fn lifecycle_target_arity_code(verb: &str) -> &'static str {
        match verb {
            "시작하기" => "E_RUNTIME_LIFECYCLE_START_TARGET_ARITY",
            "넘어가기" => "E_RUNTIME_LIFECYCLE_NEXT_TARGET_ARITY",
            "불러오기" => "E_RUNTIME_LIFECYCLE_CALL_TARGET_ARITY",
            _ => "E_RUNTIME_LIFECYCLE_TARGET_ARITY",
        }
    }

    fn lifecycle_target_family_conflict_code(verb: &str) -> &'static str {
        match verb {
            "시작하기" => "E_RUNTIME_LIFECYCLE_START_TARGET_FAMILY_CONFLICT",
            "넘어가기" => "E_RUNTIME_LIFECYCLE_NEXT_TARGET_FAMILY_CONFLICT",
            "불러오기" => "E_RUNTIME_LIFECYCLE_CALL_TARGET_FAMILY_CONFLICT",
            _ => "E_RUNTIME_LIFECYCLE_TARGET_FAMILY_CONFLICT",
        }
    }

    fn lifecycle_target_family_ambiguous_code(verb: &str) -> &'static str {
        match verb {
            "시작하기" => "E_RUNTIME_LIFECYCLE_START_TARGET_FAMILY_AMBIGUOUS",
            "넘어가기" => "E_RUNTIME_LIFECYCLE_NEXT_TARGET_FAMILY_AMBIGUOUS",
            "불러오기" => "E_RUNTIME_LIFECYCLE_CALL_TARGET_FAMILY_AMBIGUOUS",
            _ => "E_RUNTIME_LIFECYCLE_TARGET_FAMILY_AMBIGUOUS",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            RuntimeError::Undefined { .. } => "E_RUNTIME_UNDEFINED",
            RuntimeError::InvalidPath { .. } => "E_RUNTIME_INVALID_PATH",
            RuntimeError::JeOutsideImja { .. } => "E_SELF_OUTSIDE_IMJA",
            RuntimeError::MathDivZero { .. } => "E_MATH_DIV_ZERO",
            RuntimeError::MathDomain { .. } => "E_MATH_DOMAIN",
            RuntimeError::TypeMismatch { .. } => "E_RUNTIME_TYPE_MISMATCH",
            RuntimeError::TypeMismatchDetail { .. } => "E_RUNTIME_TYPE_MISMATCH",
            RuntimeError::LifecycleTargetUnknown { verb, .. } => {
                Self::lifecycle_target_unknown_code(verb)
            }
            RuntimeError::LifecycleTargetArity { verb, .. } => {
                Self::lifecycle_target_arity_code(verb)
            }
            RuntimeError::LifecycleTargetFamilyConflict { verb, .. } => {
                Self::lifecycle_target_family_conflict_code(verb)
            }
            RuntimeError::LifecycleTargetFamilyAmbiguous { verb, .. } => {
                Self::lifecycle_target_family_ambiguous_code(verb)
            }
            RuntimeError::StringIndexOutOfRange { .. } => "E_STR_INDEX_OOB",
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
            RuntimeError::ContinueOutsideForeach { .. } => "E_RUNTIME_CONTINUE_OUTSIDE_FOREACH",
            RuntimeError::ReturnOutsideSeed { .. } => "E_RUNTIME_RETURN_OUTSIDE_SEED",
            RuntimeError::ProofIncomplete { .. } => "E_PROOF_INCOMPLETE",
            RuntimeError::OpenSiteUnknown { .. } => "E_OPEN_SITE_UNKNOWN",
            RuntimeError::OpenDenied { .. } => "E_OPEN_DENIED",
            RuntimeError::OpenReplayMissing { .. } => "E_OPEN_REPLAY_MISS",
            RuntimeError::OpenReplayInvalid { .. } => "E_OPEN_LOG_PARSE",
            RuntimeError::OpenLogTamper { .. } => "E_OPEN_LOG_TAMPER",
            RuntimeError::OpenIo { .. } => "E_OPEN_IO",
            RuntimeError::RegexFlagsInvalid { .. } => "E_REGEX_FLAGS_INVALID",
            RuntimeError::RegexPatternInvalid { .. } => "E_REGEX_PATTERN_INVALID",
            RuntimeError::RegexReplacementInvalid { .. } => "E_REGEX_REPLACEMENT_INVALID",
            RuntimeError::InputKeyMissing { .. } => "E_INPUTKEY_MISSING",
            RuntimeError::MapDotKeyMissing { .. } => "E_MAP_DOT_KEY_MISSING",
            RuntimeError::EcoDivergenceDetected { .. } => "E_ECO_DIVERGENCE_DETECTED",
            RuntimeError::SfcIdentityViolation { .. } => "E_SFC_IDENTITY_VIOLATION",
        }
    }
}
