use crate::core::fixed64::Fixed64;
use crate::core::unit::{UnitExpr, UnitFactor};
use crate::lang::ast::{
    ArgBinding, Assertion, BinaryOp, Binding, BindingReason, ChooseBranch, ContractKind,
    ContractMode, DeclItem, DeclKind, Expr, FormulaDialect, HookKind, Literal, MaegimSpec,
    LifecycleUnitKind, ModuleExportItem, ModuleImportItem, NumberLiteral, ParamPin, Path,
    Program, QuantifierKind, SeedKind, Stmt, UnaryOp,
};
use crate::lang::span::Span;
use crate::lang::token::{Token, TokenKind};
use ddonirang_lang::stdlib;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        expected: &'static str,
        #[allow(dead_code)]
        found: TokenKind,
        span: Span,
    },
    ExpectedExpr {
        span: Span,
    },
    ExpectedPath {
        span: Span,
    },
    ExpectedTarget {
        span: Span,
    },
    RootHideUndeclared {
        name: String,
        span: Span,
    },
    UnsupportedCompoundTarget {
        span: Span,
    },
    ExpectedRParen {
        span: Span,
    },
    ExpectedRBrace {
        span: Span,
    },
    ExpectedUnit {
        span: Span,
    },
    InvalidTensor {
        span: Span,
    },
    CompatEqualDisabled {
        span: Span,
    },
    CompatMaticEntryDisabled {
        span: Span,
    },
    BlockHeaderColonForbidden {
        span: Span,
    },
    EventSurfaceAliasForbidden {
        span: Span,
    },
    EffectSurfaceAliasForbidden {
        span: Span,
    },
    ImportAliasDuplicate {
        span: Span,
    },
    ImportAliasReserved {
        span: Span,
    },
    ImportPathInvalid {
        span: Span,
    },
    ImportVersionConflict {
        span: Span,
    },
    ExportBlockDuplicate {
        span: Span,
    },
    LifecycleNameDuplicate {
        name: String,
        span: Span,
        first_span: Span,
    },
    ReceiveOutsideImja {
        span: Span,
    },
    MaegimRequiresGroupedValue {
        span: Span,
    },
    MaegimStepSplitConflict {
        span: Span,
    },
    MaegimNestedSectionUnsupported {
        section: String,
        span: Span,
    },
    MaegimNestedFieldUnsupported {
        section: String,
        field: String,
        span: Span,
    },
    HookEveryNMadiIntervalInvalid {
        span: Span,
    },
    HookEveryNMadiUnitUnsupported {
        unit: String,
        span: Span,
    },
    HookEveryNMadiSuffixUnsupported {
        suffix: String,
        span: Span,
    },
    DeferredAssignOutsideBeat {
        span: Span,
    },
    QuantifierMutationForbidden {
        span: Span,
    },
    QuantifierShowForbidden {
        span: Span,
    },
    QuantifierIoForbidden {
        span: Span,
    },
    ImmediateProofMutationForbidden {
        span: Span,
    },
    ImmediateProofShowForbidden {
        span: Span,
    },
    ImmediateProofIoForbidden {
        span: Span,
    },
    CaseCompletionRequired {
        span: Span,
    },
    CaseElseNotLast {
        span: Span,
    },
}

impl ParseError {
    pub fn code(&self) -> &'static str {
        match self {
            ParseError::UnexpectedToken { .. } => "E_PARSE_UNEXPECTED_TOKEN",
            ParseError::ExpectedExpr { .. } => "E_PARSE_EXPECTED_EXPR",
            ParseError::ExpectedPath { .. } => "E_PARSE_EXPECTED_PATH",
            ParseError::ExpectedTarget { .. } => "E_PARSE_EXPECTED_TARGET",
            ParseError::RootHideUndeclared { .. } => "E_PARSE_ROOT_HIDE_UNDECLARED",
            ParseError::UnsupportedCompoundTarget { .. } => "E_PARSE_UNSUPPORTED_COMPOUND_TARGET",
            ParseError::ExpectedRParen { .. } => "E_PARSE_EXPECTED_RPAREN",
            ParseError::ExpectedRBrace { .. } => "E_PARSE_EXPECTED_RBRACE",
            ParseError::ExpectedUnit { .. } => "E_PARSE_EXPECTED_UNIT",
            ParseError::InvalidTensor { .. } => "E_PARSE_TENSOR_SHAPE",
            ParseError::CompatEqualDisabled { .. } => "E_PARSE_COMPAT_EQUAL_DISABLED",
            ParseError::CompatMaticEntryDisabled { .. } => "E_LANG_COMPAT_MATIC_ENTRY_DISABLED",
            ParseError::BlockHeaderColonForbidden { .. } => "E_BLOCK_HEADER_COLON_FORBIDDEN",
            ParseError::EventSurfaceAliasForbidden { .. } => "E_EVENT_SURFACE_ALIAS_FORBIDDEN",
            ParseError::EffectSurfaceAliasForbidden { .. } => "E_EFFECT_SURFACE_ALIAS_FORBIDDEN",
            ParseError::ImportAliasDuplicate { .. } => "E_IMPORT_ALIAS_DUPLICATE",
            ParseError::ImportAliasReserved { .. } => "E_IMPORT_ALIAS_RESERVED",
            ParseError::ImportPathInvalid { .. } => "E_IMPORT_PATH_INVALID",
            ParseError::ImportVersionConflict { .. } => "E_IMPORT_VERSION_CONFLICT",
            ParseError::ExportBlockDuplicate { .. } => "E_EXPORT_BLOCK_DUPLICATE",
            ParseError::LifecycleNameDuplicate { .. } => "E_PARSE_LIFECYCLE_NAME_DUPLICATE",
            ParseError::ReceiveOutsideImja { .. } => "E_RECEIVE_OUTSIDE_IMJA",
            ParseError::MaegimRequiresGroupedValue { .. } => {
                "E_PARSE_MAEGIM_GROUPED_VALUE_REQUIRED"
            }
            ParseError::MaegimStepSplitConflict { .. } => "E_PARSE_MAEGIM_STEP_SPLIT_CONFLICT",
            ParseError::MaegimNestedSectionUnsupported { .. } => {
                "E_PARSE_MAEGIM_NESTED_SECTION_UNSUPPORTED"
            }
            ParseError::MaegimNestedFieldUnsupported { .. } => {
                "E_PARSE_MAEGIM_NESTED_FIELD_UNSUPPORTED"
            }
            ParseError::HookEveryNMadiIntervalInvalid { .. } => {
                "E_PARSE_HOOK_EVERY_N_MADI_INTERVAL_INVALID"
            }
            ParseError::HookEveryNMadiUnitUnsupported { .. } => {
                "E_PARSE_HOOK_EVERY_N_MADI_UNIT_UNSUPPORTED"
            }
            ParseError::HookEveryNMadiSuffixUnsupported { .. } => {
                "E_PARSE_HOOK_EVERY_N_MADI_SUFFIX_UNSUPPORTED"
            }
            ParseError::DeferredAssignOutsideBeat { .. } => "E_PARSE_DEFERRED_ASSIGN_OUTSIDE_BEAT",
            ParseError::QuantifierMutationForbidden { .. } => {
                "E_PARSE_QUANTIFIER_MUTATION_FORBIDDEN"
            }
            ParseError::QuantifierShowForbidden { .. } => "E_PARSE_QUANTIFIER_SHOW_FORBIDDEN",
            ParseError::QuantifierIoForbidden { .. } => "E_PARSE_QUANTIFIER_IO_FORBIDDEN",
            ParseError::ImmediateProofMutationForbidden { .. } => {
                "E_PARSE_IMMEDIATE_PROOF_MUTATION_FORBIDDEN"
            }
            ParseError::ImmediateProofShowForbidden { .. } => {
                "E_PARSE_IMMEDIATE_PROOF_SHOW_FORBIDDEN"
            }
            ParseError::ImmediateProofIoForbidden { .. } => "E_PARSE_IMMEDIATE_PROOF_IO_FORBIDDEN",
            ParseError::CaseCompletionRequired { .. } => "E_PARSE_CASE_COMPLETION_REQUIRED",
            ParseError::CaseElseNotLast { .. } => "E_PARSE_CASE_ELSE_NOT_LAST",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    Strict,
    StrictCompatMaticEntry,
}

impl ParseMode {
    pub fn with_compat_matic_entry(self, enabled: bool) -> Self {
        if enabled {
            ParseMode::StrictCompatMaticEntry
        } else {
            ParseMode::Strict
        }
    }

    pub fn compat_matic_entry_enabled(self) -> bool {
        matches!(self, ParseMode::StrictCompatMaticEntry)
    }
}

struct ArgSuffix {
    josa: Option<String>,
    fixed_pin: Option<String>,
    binding_reason: BindingReason,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    default_root: String,
    root_hide: bool,
    allow_compat_matic_entry: bool,
    declared_scopes: Vec<HashSet<String>>,
    pending_stmts: VecDeque<Stmt>,
    import_aliases: HashSet<String>,
    import_package_versions: HashMap<String, String>,
    seen_export_block: bool,
    lifecycle_named_units: HashMap<String, Span>,
    seed_kind_stack: Vec<SeedKind>,
    beat_depth: usize,
}

impl Parser {
    #[allow(dead_code)]
    pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseError> {
        Parser::parse_with_default_root(tokens, "살림")
    }

    pub fn parse_with_default_root(
        tokens: Vec<Token>,
        default_root: &str,
    ) -> Result<Program, ParseError> {
        Parser::parse_with_default_root_mode(tokens, default_root, ParseMode::Strict)
    }

    pub fn parse_with_default_root_mode(
        tokens: Vec<Token>,
        default_root: &str,
        mode: ParseMode,
    ) -> Result<Program, ParseError> {
        let mut parser = Parser {
            tokens,
            pos: 0,
            default_root: default_root.to_string(),
            root_hide: default_root == "바탕",
            allow_compat_matic_entry: mode.compat_matic_entry_enabled(),
            declared_scopes: vec![HashSet::new()],
            pending_stmts: VecDeque::new(),
            import_aliases: HashSet::new(),
            import_package_versions: HashMap::new(),
            seen_export_block: false,
            lifecycle_named_units: HashMap::new(),
            seed_kind_stack: Vec::new(),
            beat_depth: 0,
        };
        let mut stmts = Vec::new();
        parser.skip_newlines();
        while let Some(stmt) = parser.parse_stmt()? {
            stmts.push(stmt);
        }
        parser.inject_top_level_decl_blocks(&mut stmts)?;
        Ok(Program { stmts })
    }

    pub fn default_root_for_source(source: &str) -> &'static str {
        let _ = source;
        "살림"
    }

    fn enter_scope(&mut self) {
        self.declared_scopes.push(HashSet::new());
    }

    fn exit_scope(&mut self) {
        self.declared_scopes.pop();
    }

    fn declare_name(&mut self, name: &str) {
        if let Some(scope) = self.declared_scopes.last_mut() {
            scope.insert(name.to_string());
        }
    }

    fn is_declared(&self, name: &str) -> bool {
        self.declared_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    fn in_imja_seed_body(&self) -> bool {
        self.seed_kind_stack
            .last()
            .is_some_and(|kind| matches!(kind, SeedKind::Named(name) if name == "임자"))
    }

    fn ensure_root_declared_for_write(&self, path: &Path) -> Result<(), ParseError> {
        if !self.root_hide || !path.implicit_root {
            return Ok(());
        }
        let name = match path.segments.get(1) {
            Some(name) => name.clone(),
            None => {
                return Err(ParseError::RootHideUndeclared {
                    name: String::new(),
                    span: path.span,
                })
            }
        };
        if self.is_declared(&name) {
            return Ok(());
        }
        Err(ParseError::RootHideUndeclared {
            name,
            span: path.span,
        })
    }

    fn inject_top_level_decl_blocks(&mut self, stmts: &mut Vec<Stmt>) -> Result<(), ParseError> {
        let mut decls: Vec<Stmt> = Vec::new();
        let mut rest: Vec<Stmt> = Vec::with_capacity(stmts.len());
        for stmt in stmts.drain(..) {
            match stmt {
                Stmt::DeclBlock { .. } => decls.push(stmt),
                _ => rest.push(stmt),
            }
        }
        if decls.is_empty() {
            *stmts = rest;
            return Ok(());
        }
        // Frontdoor closure: top-level `채비` 선언은 프로그램 시작 시점에
        // 즉시 상태로 반영되어야 하므로 별도 seed 주입 없이 원위치로 유지한다.
        let mut merged = Vec::with_capacity(decls.len() + rest.len());
        merged.extend(decls);
        merged.extend(rest);
        *stmts = merged;
        Ok(())
    }

    #[allow(dead_code)]
    fn stmt_span(stmt: &Stmt) -> Span {
        match stmt {
            Stmt::ImportBlock { span, .. }
            | Stmt::ExportBlock { span, .. }
            | Stmt::DeclBlock { span, .. }
            | Stmt::SeedDef { span, .. }
            | Stmt::Assign { span, .. }
            | Stmt::Expr { span, .. }
            | Stmt::Receive { span, .. }
            | Stmt::Send { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::Show { span, .. }
            | Stmt::Inspect { span, .. }
            | Stmt::Hook { span, .. }
            | Stmt::HookWhenBecomes { span, .. }
            | Stmt::HookWhile { span, .. }
            | Stmt::OpenBlock { span, .. }
            | Stmt::BeatBlock { span, .. }
            | Stmt::LifecycleBlock { span, .. }
            | Stmt::Repeat { span, .. }
            | Stmt::Break { span, .. }
            | Stmt::Choose { span, .. }
            | Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::ForEach { span, .. }
            | Stmt::Quantifier { span, .. }
            | Stmt::Contract { span, .. }
            | Stmt::Pragma { span, .. }
            | Stmt::BogaeDraw { span, .. } => *span,
        }
    }

    fn parse_stmt(&mut self) -> Result<Option<Stmt>, ParseError> {
        self.parse_stmt_internal(false)
    }

    fn parse_stmt_in_block(&mut self) -> Result<Option<Stmt>, ParseError> {
        self.parse_stmt_internal(true)
    }

    fn parse_stmt_internal(&mut self, allow_rbrace: bool) -> Result<Option<Stmt>, ParseError> {
        self.skip_newlines();

        if let Some(stmt) = self.pending_stmts.pop_front() {
            return Ok(Some(stmt));
        }

        if allow_rbrace && self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Ok(None);
        }
        if self.peek_kind_is(|k| matches!(k, TokenKind::Eof)) {
            if allow_rbrace {
                return Err(ParseError::ExpectedRBrace {
                    span: self.peek().span,
                });
            }
            return Ok(None);
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::Pragma(_))) {
            let token = self.advance();
            let span = token.span;
            let TokenKind::Pragma(raw) = token.kind else {
                unreachable!("pragma branch must receive pragma token")
            };
            return Err(ParseError::UnexpectedToken {
                expected: "길잡이말(#...)은 더 이상 허용하지 않습니다. 설정:/보개:/슬기: 블록을 사용하세요",
                found: TokenKind::Pragma(raw),
                span,
            });
        }

        if !allow_rbrace {
            if self.is_import_block_start() {
                return Ok(Some(self.parse_import_block()?));
            }
            if self.is_export_block_start() {
                return Ok(Some(self.parse_export_block()?));
            }
            if let Some(seed_def) = self.try_parse_seed_def()? {
                return Ok(Some(seed_def));
            }
        }

        if let Some(stmt) = self.try_parse_immediate_proof_stmt()? {
            return Ok(Some(stmt));
        }

        if let Some(legacy) = self.peek_legacy_decl_block_name() {
            let _ = legacy;
            return Err(ParseError::BlockHeaderColonForbidden {
                span: self.peek().span,
            });
        }

        if let Some(kind) = self.peek_decl_block_kind() {
            return Ok(Some(self.parse_decl_block(kind)?));
        }

        if let Some(hook) = self.try_parse_hook()? {
            return Ok(Some(hook));
        }
        if self.is_lifecycle_block_start() {
            return Ok(Some(self.parse_lifecycle_block_stmt()));
        }

        if self.is_open_block_start() {
            return Ok(Some(self.parse_open_block_stmt()?));
        }
        if self.is_beat_block_start() {
            return Ok(Some(self.parse_beat_block_stmt()?));
        }
        if self.is_execution_policy_block_start() {
            return Ok(Some(self.parse_execution_policy_block_stmt()?));
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::Repeat)) {
            return Ok(Some(self.parse_repeat_stmt()?));
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::Break)) {
            return Ok(Some(self.parse_break_stmt()?));
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::Goreugi)) {
            return Ok(Some(self.parse_choose_stmt()?));
        }

        if self.is_foreach_start() {
            return Ok(Some(self.parse_foreach_stmt()?));
        }
        if self.is_quantifier_start() {
            return Ok(Some(self.parse_quantifier_stmt()?));
        }

        if self.is_bogae_draw_stmt() {
            return Ok(Some(self.parse_bogae_draw_stmt()?));
        }
        if self.is_bogae_shape_block_start() {
            let mut lowered = self.parse_bogae_shape_block_stmt()?;
            if let Some(first) = lowered.first().cloned() {
                for stmt in lowered.drain(1..) {
                    self.pending_stmts.push_back(stmt);
                }
                return Ok(Some(first));
            }
            return Ok(None);
        }
        if self.is_jjaim_block_start() {
            return Ok(Some(self.parse_jjaim_block_stub_stmt()?));
        }

        if self.peek_kind_is(|k| {
            matches!(
                k,
                TokenKind::Arrow | TokenKind::PlusArrow | TokenKind::MinusArrow
            )
        }) {
            let span = self.peek().span;
            return Err(ParseError::ExpectedTarget { span });
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            let condition = self.parse_condition_expr()?;
            if self.peek_kind_is(|k| matches!(k, TokenKind::Ilttae)) {
                let stmt = self.parse_if_stmt(condition)?;
                return Ok(Some(stmt));
            }
            if self.peek_kind_is(|k| matches!(k, TokenKind::During)) {
                let stmt = self.parse_while_stmt(condition)?;
                return Ok(Some(stmt));
            }
            if self.peek_kind_is(|k| matches!(k, TokenKind::Jeonjehae | TokenKind::Bojanghago)) {
                let stmt = self.parse_contract_stmt(condition)?;
                return Ok(Some(stmt));
            }
            return Err(ParseError::UnexpectedToken {
                expected: "'일때' or '동안' or '바탕으로' or '다짐하고'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }

        if let Some(stmt) = self.try_parse_event_react_stmt()? {
            return Ok(Some(stmt));
        }

        if let Some(stmt) = self.try_parse_receive_stmt()? {
            return Ok(Some(stmt));
        }

        let expr = self.parse_expr()?;
        let mut sender = None;
        let mut payload = expr;

        if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "의")) {
            self.advance();
            sender = Some(payload);
            payload = self.parse_expr()?;
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::SignalArrow)) {
            let start_span = sender
                .as_ref()
                .map(|value| value.span())
                .unwrap_or_else(|| payload.span());
            self.advance();
            let receiver = self.parse_expr()?;
            let span = start_span.merge(receiver.span());
            self.consume_terminator()?;
            return Ok(Some(Stmt::Send {
                sender,
                payload,
                receiver,
                span,
            }));
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::Ilttae)) {
            let stmt = self.parse_if_stmt(payload)?;
            return Ok(Some(stmt));
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::Jeonjehae | TokenKind::Bojanghago)) {
            let stmt = self.parse_contract_stmt(payload)?;
            return Ok(Some(stmt));
        }

        if let Some(verb_name) = self.peek_lifecycle_transition_verb_name() {
            let verb_span = self.advance().span;
            let target_name =
                Self::lifecycle_target_name_from_expr(&payload).ok_or(ParseError::ExpectedTarget {
                    span: payload.span(),
                })?;
            let span = payload.span().merge(verb_span);
            self.consume_terminator()?;
            return Ok(Some(Stmt::Expr {
                value: Expr::Call {
                    name: verb_name,
                    args: vec![self.new_arg_binding(Expr::Literal(
                        Literal::Str(target_name),
                        payload.span(),
                    ))],
                    span,
                },
                span,
            }));
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::PlusEqual | TokenKind::MinusEqual)) {
            let found = self.peek().kind.clone();
            return Err(ParseError::UnexpectedToken {
                expected: "'+<-' 또는 '-<-' (+=/-=는 미지원)",
                found,
                span: self.peek().span,
            });
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::Equal)) {
            let save_pos = self.pos;
            if let Some(name) = Self::lifecycle_target_name_from_expr(&payload) {
                let name_span = payload.span();
                self.advance();
                self.skip_newlines();
                if self.peek_kind_is(
                    |k| matches!(k, TokenKind::Ident(kind) if kind == "판" || kind == "마당"),
                ) {
                    let stmt = self.parse_lifecycle_block_stmt_named(Some(name), Some(name_span))?;
                    return Ok(Some(stmt));
                }
                self.pos = save_pos;
            }
            return Err(ParseError::CompatEqualDisabled {
                span: self.peek().span,
            });
        }

        if self.peek_kind_is(|k| {
            matches!(
                k,
                TokenKind::Arrow | TokenKind::Equal | TokenKind::PlusArrow | TokenKind::MinusArrow
            )
        }) {
            let op = match self.peek().kind {
                TokenKind::PlusArrow => Some(BinaryOp::Add),
                TokenKind::MinusArrow => Some(BinaryOp::Sub),
                _ => None,
            };
            self.advance();
            let value = self.parse_expr()?;
            let span = payload.span().merge(value.span());
            let mut deferred = false;
            let mut deferred_span = None;
            if self.peek_kind_is(|k| matches!(k, TokenKind::Reserve)) {
                if self.beat_depth == 0 {
                    return Err(ParseError::DeferredAssignOutsideBeat {
                        span: self.peek().span,
                    });
                }
                deferred = true;
                deferred_span = Some(self.advance().span);
            }
            self.consume_terminator()?;
            if op.is_some() {
                let Expr::Path(path) = payload else {
                    return Err(ParseError::UnsupportedCompoundTarget {
                        span: payload.span(),
                    });
                };
                let left = Expr::Path(path.clone());
                let binary_span = left.span().merge(value.span());
                let value_expr = Expr::Binary {
                    left: Box::new(left),
                    op: op.unwrap(),
                    right: Box::new(value),
                    span: binary_span,
                };
                self.ensure_root_declared_for_write(&path)?;
                return Ok(Some(Stmt::Assign {
                    target: path,
                    value: value_expr,
                    deferred,
                    deferred_span,
                    span,
                }));
            }

            match payload {
                Expr::Call {
                    name,
                    args,
                    span: call_span,
                } if name == "차림.값" && args.len() == 2 => {
                    let target = match &args[0].expr {
                        Expr::Path(path)
                            if path.segments.len() == 2
                                && matches!(path.segments[0].as_str(), "살림" | "바탕") =>
                        {
                            path.clone()
                        }
                        other => return Err(ParseError::ExpectedTarget { span: other.span() }),
                    };
                    self.ensure_root_declared_for_write(&target)?;
                    let index_expr = args[1].expr.clone();
                    let set_span = call_span.merge(value.span());
                    let value_call = Expr::Call {
                        name: "차림.바꾼값".to_string(),
                        args: vec![
                            self.new_arg_binding(Expr::Path(target.clone())),
                            self.new_arg_binding(index_expr),
                            self.new_arg_binding(value),
                        ],
                        span: set_span,
                    };
                    return Ok(Some(Stmt::Assign {
                        target,
                        value: value_call,
                        deferred,
                        deferred_span,
                        span,
                    }));
                }
                Expr::Path(path) => {
                    if let Some((target, key_segments)) = Self::split_map_dot_target(&path) {
                        self.ensure_root_declared_for_write(&target)?;
                        let set_span = path.span.merge(value.span());
                        let value_call = self.build_map_dot_write_expr(
                            Expr::Path(target.clone()),
                            &key_segments,
                            value,
                            set_span,
                        );
                        return Ok(Some(Stmt::Assign {
                            target,
                            value: value_call,
                            deferred,
                            deferred_span,
                            span,
                        }));
                    }
                    self.ensure_root_declared_for_write(&path)?;
                    return Ok(Some(Stmt::Assign {
                        target: path,
                        value,
                        deferred,
                        deferred_span,
                        span,
                    }));
                }
                other => return Err(ParseError::ExpectedTarget { span: other.span() }),
            }
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "지키기")) {
            let guard_span = self.advance().span;
            let mut call_name = "지키기".to_string();
            let mut end_span = guard_span;
            if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "끔")) {
                end_span = self.advance().span;
                call_name = "지키기끔".to_string();
            }
            let span = payload.span().merge(end_span);
            self.consume_terminator()?;
            return Ok(Some(Stmt::Expr {
                value: Expr::Call {
                    name: call_name,
                    args: vec![self.new_arg_binding(payload)],
                    span,
                },
                span,
            }));
        }

        if self.peek_kind_is(|k| {
            matches!(
                k,
                TokenKind::Ident(name)
                    if name == "되돌림" || name == "반환" || name == "돌려줘"
            )
        }) {
            let end_span = self.advance().span;
            let span = payload.span().merge(end_span);
            self.consume_terminator()?;
            return Ok(Some(Stmt::Return {
                value: payload,
                span,
            }));
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::Boyeojugi)) {
            self.advance();
            let span = payload.span();
            self.consume_terminator()?;
            return Ok(Some(Stmt::Show {
                value: payload,
                span,
            }));
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::Tolabogi)) {
            self.advance();
            let span = payload.span();
            self.consume_terminator()?;
            return Ok(Some(Stmt::Inspect {
                value: payload,
                span,
            }));
        }

        let span = payload.span();
        self.consume_terminator()?;
        let payload = Self::rewrite_bare_reset_call(payload);
        let payload = Self::rewrite_bare_lifecycle_transition_call(payload);
        Ok(Some(Stmt::Expr {
            value: payload,
            span,
        }))
    }

    fn path_bare_call_name(path: &Path) -> Option<String> {
        if path.segments.len() == 1 {
            return path.segments.first().cloned();
        }
        if path.implicit_root
            && path.segments.len() == 2
            && matches!(path.segments.first().map(|s| s.as_str()), Some("살림" | "바탕"))
        {
            return path.segments.get(1).cloned();
        }
        None
    }

    fn rewrite_bare_reset_call(expr: Expr) -> Expr {
        let Expr::Path(path) = expr else {
            return expr;
        };
        let call_name = Self::path_bare_call_name(&path);
        let Some(name) = call_name else {
            return Expr::Path(path);
        };
        if !Self::is_reset_kind_name(&name) {
            return Expr::Path(path);
        }
        Expr::Call {
            name,
            args: Vec::new(),
            span: path.span,
        }
    }

    fn is_reset_kind_name(name: &str) -> bool {
        matches!(name, "마당다시" | "판다시" | "누리다시" | "보개다시" | "모두다시")
    }

    fn rewrite_bare_lifecycle_transition_call(expr: Expr) -> Expr {
        let Expr::Path(path) = expr else {
            return expr;
        };
        let call_name = Self::path_bare_call_name(&path);
        let Some(name) = call_name else {
            return Expr::Path(path);
        };
        if !Self::is_lifecycle_transition_name(&name) {
            return Expr::Path(path);
        }
        Expr::Call {
            name,
            args: Vec::new(),
            span: path.span,
        }
    }

    fn is_lifecycle_transition_name(name: &str) -> bool {
        matches!(name, "시작하기" | "넘어가기" | "불러오기")
    }

    fn peek_lifecycle_transition_verb_name(&self) -> Option<String> {
        match &self.peek().kind {
            TokenKind::Ident(name) if Self::is_lifecycle_transition_name(name) => Some(name.clone()),
            _ => None,
        }
    }

    fn lifecycle_target_name_from_expr(expr: &Expr) -> Option<String> {
        let Expr::Path(path) = expr else {
            return None;
        };
        if path.segments.is_empty() {
            return None;
        }
        let parts = if path.implicit_root
            && path.segments.len() >= 2
            && matches!(path.segments.first().map(|s| s.as_str()), Some("살림" | "바탕"))
        {
            &path.segments[1..]
        } else {
            &path.segments[..]
        };
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("."))
        }
    }

    fn try_parse_event_react_stmt(&mut self) -> Result<Option<Stmt>, ParseError> {
        if self.detect_forbidden_event_alias() {
            return Err(ParseError::EventSurfaceAliasForbidden {
                span: self.peek().span,
            });
        }

        let TokenKind::String(kind) = self.peek().kind.clone() else {
            return Ok(None);
        };
        if !self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Ident(text) if text == "라는")) {
            return Ok(None);
        }
        if !self.peek_kind_n_is(
            2,
            |k| matches!(k, TokenKind::Ident(text) if Self::is_event_noun_canonical(text)),
        ) {
            return Ok(None);
        }
        if !self.peek_kind_n_is(3, |k| matches!(k, TokenKind::Ident(text) if text == "오면")) {
            return Ok(None);
        }

        let start_span = self.peek().span;
        self.advance(); // "KIND"
        self.advance(); // 라는
        self.advance(); // 알림/알림이
        self.advance(); // 오면
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let body = self.parse_block()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;

        let input_path = Path {
            segments: vec![self.default_root.clone(), "입력사건".to_string()],
            span,
            implicit_root: true,
        };
        let condition = Expr::Call {
            name: "사건.있나".to_string(),
            args: vec![
                self.new_arg_binding(Expr::Path(input_path)),
                self.new_arg_binding(Expr::Literal(Literal::Str(kind), span)),
            ],
            span,
        };
        Ok(Some(Stmt::If {
            condition,
            then_body: body,
            else_body: None,
            span,
        }))
    }

    fn try_parse_receive_stmt(&mut self) -> Result<Option<Stmt>, ParseError> {
        let checkpoint = self.pos;
        let mut binding = None;
        let mut condition = None;

        if self.peek_kind_is(|k| matches!(k, TokenKind::LParen)) {
            if !self.has_receive_binding_head() {
                return Ok(None);
            }
            self.advance();
            let TokenKind::Ident(name) = self.peek().kind.clone() else {
                self.pos = checkpoint;
                return Ok(None);
            };
            self.advance();
            binding = Some(name);
            if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
                condition = Some(self.parse_expr()?);
            }
            if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
                return Err(ParseError::ExpectedRParen {
                    span: self.peek().span,
                });
            }
            self.advance();
            if !self.peek_kind_is(|k| matches!(k, TokenKind::Ident(text) if text == "인")) {
                self.pos = checkpoint;
                return Ok(None);
            }
            self.advance();
        }

        let Some(kind) = self.peek_receive_kind_name() else {
            self.pos = checkpoint;
            return Ok(None);
        };
        let start_span = self.peek().span;
        self.advance();

        if !self.peek_kind_is(|k| matches!(k, TokenKind::Ident(text) if text == "받으면")) {
            self.pos = checkpoint;
            return Ok(None);
        }
        if !self.in_imja_seed_body() {
            return Err(ParseError::ReceiveOutsideImja { span: start_span });
        }
        self.advance();

        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let body = self.parse_block()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        Ok(Some(Stmt::Receive {
            kind,
            binding,
            condition,
            body,
            span,
        }))
    }

    fn has_receive_binding_head(&self) -> bool {
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LParen)) {
            return false;
        }
        let mut depth = 0usize;
        let mut index = self.pos;
        while let Some(token) = self.tokens.get(index) {
            match &token.kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return self.tokens.get(index + 1).is_some_and(
                            |next| matches!(&next.kind, TokenKind::Ident(text) if text == "인"),
                        );
                    }
                }
                _ => {}
            }
            index += 1;
        }
        false
    }

    fn peek_receive_kind_name(&self) -> Option<Option<String>> {
        let TokenKind::Ident(text) = &self.peek().kind else {
            return None;
        };
        let kind = Self::strip_object_particle(text)?;
        if kind == "알림" {
            Some(None)
        } else {
            Some(Some(kind))
        }
    }

    fn strip_object_particle(text: &str) -> Option<String> {
        text.strip_suffix('를')
            .or_else(|| text.strip_suffix('을'))
            .map(|raw| raw.to_string())
    }

    fn detect_forbidden_event_alias(&self) -> bool {
        match &self.peek().kind {
            TokenKind::Ident(text) if Self::is_event_noun_any(text) => {
                self.peek_kind_n_is(1, |k| matches!(k, TokenKind::String(_)))
            }
            TokenKind::String(_) => {
                (self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Ilttae))
                    || self.peek_kind_n_is(
                        1,
                        |k| matches!(k, TokenKind::Ident(text) if text == "일때"),
                    ))
                    || (self.peek_kind_n_is(
                        1,
                        |k| matches!(k, TokenKind::Ident(text) if text == "라는"),
                    ) && self.peek_kind_n_is(
                        2,
                        |k| matches!(k, TokenKind::Ident(text) if Self::is_event_noun_alias(text)),
                    ))
            }
            _ => false,
        }
    }

    fn is_event_noun_canonical(text: &str) -> bool {
        matches!(text, "알림" | "알림이" | "알림가")
    }

    fn is_event_noun_alias(text: &str) -> bool {
        matches!(
            text,
            "소식" | "소식이" | "소식가" | "알람" | "알람이" | "알람가"
        )
    }

    fn is_event_noun_any(text: &str) -> bool {
        Self::is_event_noun_canonical(text) || Self::is_event_noun_alias(text)
    }

    fn try_parse_hook(&mut self) -> Result<Option<Stmt>, ParseError> {
        if self.is_hook_start() {
            return Ok(Some(self.parse_hook(HookKind::Start)?));
        }
        if self.is_hook_end() {
            return Ok(Some(self.parse_hook(HookKind::End)?));
        }
        if self.is_hook_every() {
            return Ok(Some(self.parse_hook(HookKind::EveryMadi)?));
        }
        if self.is_hook_every_n_madi_candidate() {
            return Ok(Some(self.parse_hook_every_n_madi()?));
        }
        if self.is_hook_every_n_madi() {
            return Ok(Some(self.parse_hook_every_n_madi()?));
        }
        if let Some(hook) = self.try_parse_condition_hook()? {
            return Ok(Some(hook));
        }
        Ok(None)
    }

    fn is_hook_start(&self) -> bool {
        self.peek_kind_is(|k| matches!(k, TokenKind::LParen))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Start))
            && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::RParen))
            && self.peek_kind_n_is(3, |k| matches!(k, TokenKind::Halttae))
    }

    fn is_hook_end(&self) -> bool {
        self.peek_kind_is(|k| matches!(k, TokenKind::LParen))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Ident(name) if name == "끝"))
            && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::RParen))
            && self.peek_kind_n_is(3, |k| matches!(k, TokenKind::Halttae))
    }

    fn is_hook_every(&self) -> bool {
        self.peek_kind_is(|k| matches!(k, TokenKind::LParen))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::EveryMadi))
            && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::RParen))
            && self.peek_kind_n_is(3, |k| matches!(k, TokenKind::Mada))
    }

    fn is_hook_every_n_madi(&self) -> bool {
        self.peek_kind_is(|k| matches!(k, TokenKind::LParen))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Number(_)))
            && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::Ident(name) if name == "마디"))
            && self.peek_kind_n_is(3, |k| matches!(k, TokenKind::RParen))
            && self.peek_kind_n_is(4, |k| matches!(k, TokenKind::Mada))
    }

    fn is_hook_every_n_madi_candidate(&self) -> bool {
        if !(self.peek_kind_is(|k| matches!(k, TokenKind::LParen))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Number(_)))
            && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::Ident(_)))
            && self.peek_kind_n_is(3, |k| matches!(k, TokenKind::RParen)))
        {
            return false;
        }
        let suffix_ok = self.peek_kind_n_is(4, |k| {
            matches!(
                k,
                TokenKind::Mada | TokenKind::Halttae | TokenKind::Ident(_)
            )
        });
        suffix_ok && self.hook_header_has_block_start(5)
    }

    fn hook_header_has_block_start(&self, mut offset: usize) -> bool {
        while self.peek_kind_n_is(offset, |k| matches!(k, TokenKind::Newline)) {
            offset += 1;
        }
        if self.peek_kind_n_is(offset, |k| matches!(k, TokenKind::Colon)) {
            offset += 1;
            while self.peek_kind_n_is(offset, |k| matches!(k, TokenKind::Newline)) {
                offset += 1;
            }
        }
        self.peek_kind_n_is(offset, |k| matches!(k, TokenKind::LBrace))
    }

    fn is_bogae_draw_stmt(&self) -> bool {
        self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "보개로"))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Ident(name) if name == "그려"))
    }

    fn parse_bogae_draw_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.advance().span;
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        Ok(Stmt::BogaeDraw { span })
    }

    fn is_bogae_shape_block_start(&self) -> bool {
        if !self.peek_kind_is(
            |k| matches!(k, TokenKind::Ident(name) if name == "모양" || name == "보개"),
        ) {
            return false;
        }
        self.peek_kind_n_is(1, |k| matches!(k, TokenKind::LBrace))
            || (self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Colon))
                && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::LBrace)))
    }

    fn is_beat_block_start(&self) -> bool {
        self.peek_kind_is(Self::is_beat_block_head)
            && (self.peek_kind_n_is(1, |k| matches!(k, TokenKind::LBrace))
                || (self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Colon))
                    && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::LBrace))))
    }

    fn is_beat_block_head(kind: &TokenKind) -> bool {
        match kind {
            TokenKind::Beat => true,
            _ => false,
        }
    }

    fn is_lifecycle_block_start(&self) -> bool {
        if !self.peek_kind_is(
            |k| matches!(k, TokenKind::Ident(name) if name == "판" || name == "마당"),
        ) {
            return false;
        }
        self.peek_kind_n_is(1, |k| matches!(k, TokenKind::LBrace))
            || (self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Colon))
                && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::LBrace)))
    }

    fn parse_beat_block_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.advance().span;
        self.skip_newlines();
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        self.beat_depth += 1;
        let body_result = self.parse_block();
        self.beat_depth = self.beat_depth.saturating_sub(1);
        let body = body_result?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        Ok(Stmt::BeatBlock { body, span })
    }

    fn parse_lifecycle_block_stmt(&mut self) -> Stmt {
        self.parse_lifecycle_block_stmt_named(None, None)
            .expect("lifecycle block start already validated")
    }

    fn parse_lifecycle_block_stmt_named(
        &mut self,
        name: Option<String>,
        name_span: Option<Span>,
    ) -> Result<Stmt, ParseError> {
        let head = self.advance();
        let kind = match &head.kind {
            TokenKind::Ident(name) if name == "판" => LifecycleUnitKind::Pan,
            TokenKind::Ident(name) if name == "마당" => LifecycleUnitKind::Madang,
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "'판' or '마당'",
                    found: other.clone(),
                    span: head.span,
                })
            }
        };
        self.skip_newlines();
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let body = self.parse_block()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = name_span.unwrap_or(head.span).merge(end_span);
        if let Some(name) = &name {
            if let Some(first_span) = self.lifecycle_named_units.get(name) {
                return Err(ParseError::LifecycleNameDuplicate {
                    name: name.clone(),
                    span: name_span.unwrap_or(head.span),
                    first_span: *first_span,
                });
            }
            self.lifecycle_named_units
                .insert(name.clone(), name_span.unwrap_or(head.span));
        }
        self.consume_terminator()?;
        Ok(Stmt::LifecycleBlock {
            name,
            kind,
            body,
            span,
        })
    }

    fn parse_bogae_shape_block_stmt(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let head = self.advance();
        let block_start = head.span;

        self.skip_newlines();
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();

        let mut primitives: Vec<(String, Vec<ArgBinding>, Span)> = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
                break;
            }
            if self.peek_kind_is(|k| matches!(k, TokenKind::Eof)) {
                return Err(ParseError::ExpectedRBrace {
                    span: self.peek().span,
                });
            }

            let token = self.advance();
            let kind = match token.kind {
                TokenKind::Ident(name) if name == "선" || name == "원" || name == "점" => name,
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "'선' or '원' or '점'",
                        found: other,
                        span: token.span,
                    });
                }
            };
            if !self.peek_kind_is(|k| matches!(k, TokenKind::LParen)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "'('",
                    found: self.peek().kind.clone(),
                    span: self.peek().span,
                });
            }
            self.advance();
            let mut args = Vec::new();
            if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
                loop {
                    args.push(self.parse_call_arg()?);
                    if self.peek_kind_is(|k| matches!(k, TokenKind::Comma)) {
                        self.advance();
                        continue;
                    }
                    break;
                }
            }
            if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
                return Err(ParseError::ExpectedRParen {
                    span: self.peek().span,
                });
            }
            let rparen_span = self.advance().span;
            self.consume_terminator()?;
            primitives.push((kind, args, token.span.merge(rparen_span)));
        }

        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end = self.advance().span;
        self.consume_terminator()?;

        let span = block_start.merge(end);
        Ok(self.lower_bogae_shape_primitives(&primitives, span))
    }

    fn is_jjaim_block_start(&self) -> bool {
        if !self.peek_kind_is(
            |k| matches!(k, TokenKind::Ident(name) if name == "짜임" || name == "구성"),
        ) {
            return false;
        }
        self.peek_kind_n_is(1, |k| matches!(k, TokenKind::LBrace))
            || (self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Colon))
                && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::LBrace)))
    }

    fn parse_jjaim_block_stub_stmt(&mut self) -> Result<Stmt, ParseError> {
        let head_span = self.advance().span;
        self.skip_newlines();
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }

        let mut depth = 0usize;
        let mut end_span = head_span;
        while !self.peek_kind_is(|k| matches!(k, TokenKind::Eof)) {
            let token = self.advance();
            end_span = token.span;
            match token.kind {
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth != 0 {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        self.consume_terminator()?;
        let span = head_span.merge(end_span);
        Ok(Stmt::Expr {
            value: Expr::Literal(Literal::None, span),
            span,
        })
    }

    fn lower_bogae_shape_primitives(
        &self,
        primitives: &[(String, Vec<ArgBinding>, Span)],
        span: Span,
    ) -> Vec<Stmt> {
        let mut out = Vec::new();
        out.push(Self::show_str_stmt("space2d", span));
        for (kind, args, shape_span) in primitives {
            out.push(Self::show_str_stmt("space2d.shape", *shape_span));
            match kind.as_str() {
                "선" => {
                    out.push(Self::show_str_stmt("line", *shape_span));
                    let x1 = self.pick_shape_arg_expr(args, &["x1"], 0);
                    let y1 = self.pick_shape_arg_expr(args, &["y1"], 1);
                    let x2 = self.pick_shape_arg_expr(args, &["x2"], 2);
                    let y2 = self.pick_shape_arg_expr(args, &["y2"], 3);
                    let stroke = self.pick_shape_arg_expr(args, &["색", "stroke"], usize::MAX);
                    let width = self.pick_shape_arg_expr(args, &["굵기", "width"], usize::MAX);
                    Self::push_shape_kv(
                        &mut out,
                        "x1",
                        x1.unwrap_or_else(|| Self::num_expr("0", *shape_span)),
                        *shape_span,
                    );
                    Self::push_shape_kv(
                        &mut out,
                        "y1",
                        y1.unwrap_or_else(|| Self::num_expr("0", *shape_span)),
                        *shape_span,
                    );
                    Self::push_shape_kv(
                        &mut out,
                        "x2",
                        x2.unwrap_or_else(|| Self::num_expr("0", *shape_span)),
                        *shape_span,
                    );
                    Self::push_shape_kv(
                        &mut out,
                        "y2",
                        y2.unwrap_or_else(|| Self::num_expr("0", *shape_span)),
                        *shape_span,
                    );
                    Self::push_shape_kv(
                        &mut out,
                        "stroke",
                        stroke.unwrap_or_else(|| Self::str_expr("#9ca3af", *shape_span)),
                        *shape_span,
                    );
                    Self::push_shape_kv(
                        &mut out,
                        "width",
                        width.unwrap_or_else(|| Self::num_expr("0.02", *shape_span)),
                        *shape_span,
                    );
                }
                "원" => {
                    out.push(Self::show_str_stmt("circle", *shape_span));
                    let x = self.pick_shape_arg_expr(args, &["x", "cx"], 0);
                    let y = self.pick_shape_arg_expr(args, &["y", "cy"], 1);
                    let r = self.pick_shape_arg_expr(args, &["r", "반지름"], 2);
                    let fill = self.pick_shape_arg_expr(args, &["색", "fill"], usize::MAX);
                    let stroke = self.pick_shape_arg_expr(args, &["선색", "stroke"], usize::MAX);
                    let width = self.pick_shape_arg_expr(args, &["굵기", "width"], usize::MAX);
                    Self::push_shape_kv(
                        &mut out,
                        "x",
                        x.unwrap_or_else(|| Self::num_expr("0", *shape_span)),
                        *shape_span,
                    );
                    Self::push_shape_kv(
                        &mut out,
                        "y",
                        y.unwrap_or_else(|| Self::num_expr("0", *shape_span)),
                        *shape_span,
                    );
                    Self::push_shape_kv(
                        &mut out,
                        "r",
                        r.unwrap_or_else(|| Self::num_expr("0.08", *shape_span)),
                        *shape_span,
                    );
                    Self::push_shape_kv(
                        &mut out,
                        "fill",
                        fill.unwrap_or_else(|| Self::str_expr("#38bdf8", *shape_span)),
                        *shape_span,
                    );
                    Self::push_shape_kv(
                        &mut out,
                        "stroke",
                        stroke.unwrap_or_else(|| Self::str_expr("#0ea5e9", *shape_span)),
                        *shape_span,
                    );
                    Self::push_shape_kv(
                        &mut out,
                        "width",
                        width.unwrap_or_else(|| Self::num_expr("0.02", *shape_span)),
                        *shape_span,
                    );
                }
                _ => {
                    out.push(Self::show_str_stmt("point", *shape_span));
                    let x = self.pick_shape_arg_expr(args, &["x", "cx"], 0);
                    let y = self.pick_shape_arg_expr(args, &["y", "cy"], 1);
                    let size = self.pick_shape_arg_expr(args, &["크기", "size", "r"], 2);
                    let color = self.pick_shape_arg_expr(args, &["색", "color"], usize::MAX);
                    Self::push_shape_kv(
                        &mut out,
                        "x",
                        x.unwrap_or_else(|| Self::num_expr("0", *shape_span)),
                        *shape_span,
                    );
                    Self::push_shape_kv(
                        &mut out,
                        "y",
                        y.unwrap_or_else(|| Self::num_expr("0", *shape_span)),
                        *shape_span,
                    );
                    Self::push_shape_kv(
                        &mut out,
                        "size",
                        size.unwrap_or_else(|| Self::num_expr("0.05", *shape_span)),
                        *shape_span,
                    );
                    Self::push_shape_kv(
                        &mut out,
                        "color",
                        color.unwrap_or_else(|| Self::str_expr("#22c55e", *shape_span)),
                        *shape_span,
                    );
                }
            }
        }
        out
    }

    fn pick_shape_arg_expr(
        &self,
        args: &[ArgBinding],
        pin_names: &[&str],
        positional_index: usize,
    ) -> Option<Expr> {
        for pin in pin_names {
            if let Some(hit) = args
                .iter()
                .find(|arg| arg.resolved_pin.as_deref() == Some(*pin))
            {
                return Some(hit.expr.clone());
            }
        }
        if positional_index == usize::MAX {
            return None;
        }
        let mut seen = 0usize;
        for arg in args {
            if arg.resolved_pin.is_some() {
                continue;
            }
            if seen == positional_index {
                return Some(arg.expr.clone());
            }
            seen += 1;
        }
        None
    }

    fn push_shape_kv(out: &mut Vec<Stmt>, key: &str, value: Expr, span: Span) {
        out.push(Self::show_str_stmt(key, span));
        out.push(Self::show_expr_stmt(value));
    }

    fn show_str_stmt(text: &str, span: Span) -> Stmt {
        let expr = Expr::Literal(Literal::Str(text.to_string()), span);
        Stmt::Show { value: expr, span }
    }

    fn show_expr_stmt(value: Expr) -> Stmt {
        let span = value.span();
        Stmt::Show { value, span }
    }

    fn str_expr(text: &str, span: Span) -> Expr {
        Expr::Literal(Literal::Str(text.to_string()), span)
    }

    fn num_expr(text: &str, span: Span) -> Expr {
        let raw = Fixed64::parse_literal(text)
            .unwrap_or_else(|| Fixed64::from_int(0))
            .raw();
        Expr::Literal(Literal::Num(NumberLiteral { raw, unit: None }), span)
    }

    fn peek_decl_block_kind(&self) -> Option<DeclKind> {
        if !self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "채비")) {
            return None;
        }
        if self.peek_next_non_newline_is(|k| matches!(k, TokenKind::LBrace | TokenKind::Colon)) {
            return Some(DeclKind::Gureut);
        }
        None
    }

    fn peek_legacy_decl_block_name(&self) -> Option<&'static str> {
        if !self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Colon)) {
            return None;
        }
        match &self.peek().kind {
            TokenKind::Ident(name) if name == "그릇채비" => Some("그릇채비"),
            TokenKind::Ident(name) if name == "붙박이마련" => Some("붙박이마련"),
            TokenKind::Ident(name) if name == "붙박이채비" => Some("붙박이채비"),
            TokenKind::Ident(name) if name == "바탕칸" => Some("바탕칸"),
            TokenKind::Ident(name) if name == "바탕칸표" => Some("바탕칸표"),
            _ => None,
        }
    }

    fn is_import_block_start(&self) -> bool {
        if !self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "쓰임")) {
            return false;
        }
        self.peek_next_non_newline_is(|k| matches!(k, TokenKind::LBrace | TokenKind::Colon))
    }

    fn is_export_block_start(&self) -> bool {
        if !self.peek_kind_is(
            |k| matches!(k, TokenKind::Ident(name) if name == "드러냄" || name == "공개"),
        ) {
            return false;
        }
        self.peek_next_non_newline_is(|k| matches!(k, TokenKind::LBrace | TokenKind::Colon))
    }

    fn parse_import_block(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.advance().span;
        self.skip_newlines();
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();

        let mut items = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
                break;
            }
            if self.peek_kind_is(|k| matches!(k, TokenKind::Eof)) {
                return Err(ParseError::ExpectedRBrace {
                    span: self.peek().span,
                });
            }

            let alias_token = self.peek().clone();
            let alias = match &alias_token.kind {
                TokenKind::Ident(text) => {
                    self.advance();
                    text.clone()
                }
                TokenKind::Salim => {
                    self.advance();
                    "살림".to_string()
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "모듈 별명",
                        found: alias_token.kind,
                        span: alias_token.span,
                    })
                }
            };

            if !self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "':'",
                    found: self.peek().kind.clone(),
                    span: self.peek().span,
                });
            }
            self.advance();

            let path_token = self.peek().clone();
            let path = match &path_token.kind {
                TokenKind::String(text) => {
                    self.advance();
                    text.clone()
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "모듈 경로 문자열",
                        found: path_token.kind,
                        span: path_token.span,
                    })
                }
            };

            if Self::is_reserved_import_alias(&alias) {
                return Err(ParseError::ImportAliasReserved {
                    span: alias_token.span,
                });
            }
            if !self.import_aliases.insert(alias.clone()) {
                return Err(ParseError::ImportAliasDuplicate {
                    span: alias_token.span,
                });
            }
            let (package_id, version) =
                Self::parse_import_path_spec(&path).ok_or(ParseError::ImportPathInvalid {
                    span: path_token.span,
                })?;
            if let Some(version) = version {
                if let Some(prev) = self.import_package_versions.get(&package_id) {
                    if prev != &version {
                        return Err(ParseError::ImportVersionConflict {
                            span: path_token.span,
                        });
                    }
                } else {
                    self.import_package_versions.insert(package_id, version);
                }
            }

            let item_span = alias_token.span.merge(path_token.span);
            self.consume_terminator()?;
            items.push(ModuleImportItem {
                alias,
                path,
                span: item_span,
            });
        }

        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        Ok(Stmt::ImportBlock { items, span })
    }

    fn parse_export_block(&mut self) -> Result<Stmt, ParseError> {
        if self.seen_export_block {
            return Err(ParseError::ExportBlockDuplicate {
                span: self.peek().span,
            });
        }
        let start_span = self.advance().span;
        self.skip_newlines();
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();

        let mut items = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
                break;
            }
            if self.peek_kind_is(|k| matches!(k, TokenKind::Eof)) {
                return Err(ParseError::ExpectedRBrace {
                    span: self.peek().span,
                });
            }

            let external_token = self.peek().clone();
            let external_name = match &external_token.kind {
                TokenKind::Ident(text) => {
                    self.advance();
                    text.clone()
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "내보낼 이름",
                        found: external_token.kind,
                        span: external_token.span,
                    })
                }
            };

            let (internal_name, end_span) = if self.peek_kind_is(|k| matches!(k, TokenKind::Colon))
            {
                self.advance();
                let internal_token = self.peek().clone();
                let internal_name = match &internal_token.kind {
                    TokenKind::Ident(text) => {
                        self.advance();
                        text.clone()
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            expected: "내부 이름",
                            found: internal_token.kind,
                            span: internal_token.span,
                        })
                    }
                };
                (internal_name, internal_token.span)
            } else {
                (external_name.clone(), external_token.span)
            };

            let item_span = external_token.span.merge(end_span);
            self.consume_terminator()?;
            items.push(ModuleExportItem {
                external_name,
                internal_name,
                span: item_span,
            });
        }

        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        self.seen_export_block = true;
        Ok(Stmt::ExportBlock { items, span })
    }

    fn is_reserved_import_alias(alias: &str) -> bool {
        matches!(
            alias,
            "살림"
                | "바탕"
                | "샘"
                | "채비"
                | "쓰임"
                | "드러냄"
                | "공개"
                | "매틱"
                | "매마디"
                | "실행정책"
        )
    }

    fn parse_import_path_spec(path: &str) -> Option<(String, Option<String>)> {
        if path.is_empty() || path.contains("://") {
            return None;
        }
        if path.starts_with("./") {
            if path.len() <= 2 || path.contains('@') {
                return None;
            }
            return Some((path.to_string(), None));
        }
        let (package, version) = if let Some((left, right)) = path.rsplit_once('@') {
            if left.is_empty() || right.is_empty() {
                return None;
            }
            (left, Some(right.to_string()))
        } else {
            (path, None)
        };
        let mut parts = package.split('/');
        let scope = parts.next()?;
        if !matches!(scope, "표준" | "나눔" | "내" | "벌림") {
            return None;
        }
        let mut has_name = false;
        for part in parts {
            if part.is_empty() || part == "." || part == ".." {
                return None;
            }
            has_name = true;
        }
        if !has_name {
            return None;
        }
        Some((package.to_string(), version))
    }

    fn parse_decl_block(&mut self, _kind: DeclKind) -> Result<Stmt, ParseError> {
        let start_span = self.advance().span;
        let TokenKind::Ident(keyword) = &self.tokens[self.pos - 1].kind else {
            unreachable!("decl block keyword must be identifier")
        };
        if keyword != "채비" {
            return Err(ParseError::UnexpectedToken {
                expected: "'채비 {'",
                found: TokenKind::Ident(keyword.clone()),
                span: start_span,
            });
        }
        self.skip_newlines();
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();

        let mut items = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
                break;
            }
            if self.peek_kind_is(|k| matches!(k, TokenKind::Eof)) {
                return Err(ParseError::ExpectedRBrace {
                    span: self.peek().span,
                });
            }

            let name_token = self.peek().clone();
            let name = match &name_token.kind {
                TokenKind::Ident(text) => {
                    self.advance();
                    text.clone()
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "식별자",
                        found: name_token.kind,
                        span: name_token.span,
                    })
                }
            };

            if !self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "':'",
                    found: self.peek().kind.clone(),
                    span: self.peek().span,
                });
            }
            self.advance();

            let type_token = self.peek().clone();
            let type_name = match &type_token.kind {
                TokenKind::Ident(text) => {
                    self.advance();
                    text.clone()
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "형 이름",
                        found: type_token.kind,
                        span: type_token.span,
                    })
                }
            };

            let mut value = None;
            let mut maegim = None;
            let mut item_kind = DeclKind::Gureut;
            let mut end_span = type_token.span;
            if self.peek_kind_is(|k| matches!(k, TokenKind::Arrow)) {
                self.advance();
                let (expr, item_maegim) = self.parse_decl_item_value_and_maegim()?;
                end_span = end_span.merge(expr.span());
                if let Some(spec) = &item_maegim {
                    end_span = end_span.merge(spec.span);
                }
                value = Some(expr);
                maegim = item_maegim;
            } else if self.peek_kind_is(|k| matches!(k, TokenKind::Equal)) {
                item_kind = DeclKind::Butbak;
                self.advance();
                let (expr, item_maegim) = self.parse_decl_item_value_and_maegim()?;
                end_span = end_span.merge(expr.span());
                if let Some(spec) = &item_maegim {
                    end_span = end_span.merge(spec.span);
                }
                value = Some(expr);
                maegim = item_maegim;
            }

            let item_span = name_token.span.merge(end_span);
            self.consume_terminator()?;

            self.declare_name(&name);

            items.push(DeclItem {
                name,
                kind: item_kind,
                type_name,
                value,
                maegim,
                span: item_span,
            });
        }

        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;

        Ok(Stmt::DeclBlock { items, span })
    }

    fn parse_decl_item_value_and_maegim(
        &mut self,
    ) -> Result<(Expr, Option<MaegimSpec>), ParseError> {
        if self.peek_kind_is(|k| matches!(k, TokenKind::LParen)) {
            let checkpoint = self.pos;
            if let Ok(grouped) = self.try_parse_grouped_expr_for_maegim() {
                if self.peek_maegim_keyword() {
                    let maegim = self.parse_maegim_spec()?;
                    return Ok((grouped, Some(maegim)));
                }
            }
            self.pos = checkpoint;
        }

        let expr = self.parse_expr()?;
        if self.peek_maegim_keyword() {
            return Err(ParseError::MaegimRequiresGroupedValue {
                span: self.peek().span,
            });
        }
        Ok((expr, None))
    }

    fn try_parse_grouped_expr_for_maegim(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.advance().span;
        let expr = self.parse_expr()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
            return Err(ParseError::ExpectedRParen {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        Ok(match expr {
            Expr::Literal(literal, _) => Expr::Literal(literal, span),
            Expr::Path(mut path) => {
                path.span = span;
                Expr::Path(path)
            }
            Expr::FieldAccess { target, field, .. } => Expr::FieldAccess {
                target,
                field,
                span,
            },
            Expr::Atom { text, .. } => Expr::Atom { text, span },
            Expr::Unary { op, expr, .. } => Expr::Unary { op, expr, span },
            Expr::Binary {
                left, op, right, ..
            } => Expr::Binary {
                left,
                op,
                right,
                span,
            },
            Expr::SeedLiteral { param, body, .. } => Expr::SeedLiteral { param, body, span },
            Expr::Call { name, args, .. } => Expr::Call { name, args, span },
            Expr::Formula { dialect, body, .. } => Expr::Formula {
                dialect,
                body,
                span,
            },
            Expr::Assertion { assertion, .. } => Expr::Assertion { assertion, span },
            Expr::FormulaEval {
                dialect,
                body,
                bindings,
                ..
            } => Expr::FormulaEval {
                dialect,
                body,
                bindings,
                span,
            },
            Expr::Template { body, .. } => Expr::Template { body, span },
            Expr::TemplateFill {
                template, bindings, ..
            } => Expr::TemplateFill {
                template,
                bindings,
                span,
            },
            Expr::Pack { bindings, .. } => Expr::Pack { bindings, span },
            Expr::FormulaFill {
                formula, bindings, ..
            } => Expr::FormulaFill {
                formula,
                bindings,
                span,
            },
        })
    }

    fn peek_maegim_keyword(&self) -> bool {
        self.peek_kind_is(
            |k| matches!(k, TokenKind::Ident(name) if name == "조건" || name == "매김"),
        )
    }

    fn is_supported_maegim_nested_section(name: &str) -> bool {
        matches!(name, "가늠" | "갈래")
    }

    fn is_supported_maegim_nested_field(section: &str, field: &str) -> bool {
        match section {
            "가늠" => matches!(field, "범위" | "간격" | "분할수"),
            "갈래" => matches!(field, "가만히" | "벗남다룸"),
            _ => false,
        }
    }

    fn parse_maegim_spec(&mut self) -> Result<MaegimSpec, ParseError> {
        let start_token = self.peek().clone();
        let keyword_span = self.advance().span;
        let TokenKind::Ident(keyword) = start_token.kind else {
            unreachable!("maegim branch must start with identifier")
        };
        if keyword != "조건" && keyword != "매김" {
            return Err(ParseError::UnexpectedToken {
                expected: "'조건 {' 또는 '매김 {'",
                found: TokenKind::Ident(keyword),
                span: keyword_span,
            });
        }
        self.skip_newlines();
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();

        let mut fields = Vec::new();
        let mut has_step = false;
        let mut has_split_count = false;
        loop {
            self.skip_newlines();
            if self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
                break;
            }
            if self.peek_kind_is(|k| matches!(k, TokenKind::Eof)) {
                return Err(ParseError::ExpectedRBrace {
                    span: self.peek().span,
                });
            }

            let name_token = self.peek().clone();
            let name = match &name_token.kind {
                TokenKind::Ident(text) => {
                    self.advance();
                    text.clone()
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "매김 항목 이름",
                        found: name_token.kind,
                        span: name_token.span,
                    })
                }
            };
            self.skip_newlines();
            if self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
                if !Self::is_supported_maegim_nested_section(&name) {
                    return Err(ParseError::MaegimNestedSectionUnsupported {
                        section: name,
                        span: name_token.span,
                    });
                }
                self.advance();
                loop {
                    self.skip_newlines();
                    if self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
                        break;
                    }
                    if self.peek_kind_is(|k| matches!(k, TokenKind::Eof)) {
                        return Err(ParseError::ExpectedRBrace {
                            span: self.peek().span,
                        });
                    }
                    let nested_name_token = self.peek().clone();
                    let nested_name = match &nested_name_token.kind {
                        TokenKind::Ident(text) => {
                            self.advance();
                            text.clone()
                        }
                        _ => {
                            return Err(ParseError::UnexpectedToken {
                                expected: "매김 항목 이름",
                                found: nested_name_token.kind,
                                span: nested_name_token.span,
                            })
                        }
                    };
                    if !self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
                        return Err(ParseError::UnexpectedToken {
                            expected: "':'",
                            found: self.peek().kind.clone(),
                            span: self.peek().span,
                        });
                    }
                    self.advance();
                    let value = self.parse_expr()?;
                    if !Self::is_supported_maegim_nested_field(&name, &nested_name) {
                        return Err(ParseError::MaegimNestedFieldUnsupported {
                            section: name.clone(),
                            field: nested_name,
                            span: nested_name_token.span,
                        });
                    }
                    let field_name = format!("{}.{}", name, nested_name);
                    let field_span = nested_name_token.span.merge(value.span());
                    if nested_name == "간격" {
                        has_step = true;
                    } else if nested_name == "분할수" {
                        has_split_count = true;
                    }
                    if has_step && has_split_count {
                        return Err(ParseError::MaegimStepSplitConflict { span: field_span });
                    }
                    self.consume_terminator()?;
                    fields.push(Binding {
                        name: field_name,
                        value,
                        span: field_span,
                    });
                }
                self.advance();
                if self.peek_kind_is(|k| matches!(k, TokenKind::Dot | TokenKind::Newline)) {
                    self.consume_terminator()?;
                } else if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof))
                {
                    return Err(ParseError::UnexpectedToken {
                        expected: "'.' or newline or '}'",
                        found: self.peek().kind.clone(),
                        span: self.peek().span,
                    });
                }
            } else {
                if !self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
                    return Err(ParseError::UnexpectedToken {
                        expected: "':'",
                        found: self.peek().kind.clone(),
                        span: self.peek().span,
                    });
                }
                self.advance();
                let value = self.parse_expr()?;
                let field_span = name_token.span.merge(value.span());
                if name == "간격" {
                    has_step = true;
                } else if name == "분할수" {
                    has_split_count = true;
                }
                if has_step && has_split_count {
                    return Err(ParseError::MaegimStepSplitConflict { span: field_span });
                }
                self.consume_terminator()?;
                fields.push(Binding {
                    name,
                    value,
                    span: field_span,
                });
            }
        }

        let end_span = self.advance().span;
        Ok(MaegimSpec {
            fields,
            span: keyword_span.merge(end_span),
        })
    }

    fn parse_hook(&mut self, kind: HookKind) -> Result<Stmt, ParseError> {
        let start_span = self.advance().span;
        self.advance();
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
            return Err(ParseError::ExpectedRParen {
                span: self.peek().span,
            });
        }
        self.advance();
        self.advance();
        self.parse_hook_tail(start_span, kind)
    }

    fn try_parse_condition_hook(&mut self) -> Result<Option<Stmt>, ParseError> {
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LParen)) {
            return Ok(None);
        }
        let checkpoint = self.pos;
        let start_span = self.advance().span;
        let condition = self.parse_expr()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
            self.pos = checkpoint;
            return Ok(None);
        }
        self.advance();

        if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "이" || name == "가"))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Ident(name) if name == "될때"))
        {
            self.advance();
            self.advance();
            let (body, span) = self.parse_hook_body_and_span(start_span)?;
            return Ok(Some(Stmt::HookWhenBecomes {
                condition,
                body,
                span,
            }));
        }
        if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "인"))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::During))
        {
            self.advance();
            self.advance();
            let (body, span) = self.parse_hook_body_and_span(start_span)?;
            return Ok(Some(Stmt::HookWhile {
                condition,
                body,
                span,
            }));
        }

        self.pos = checkpoint;
        Ok(None)
    }

    fn parse_hook_every_n_madi(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.advance().span;
        let number_token = self.advance();
        let interval = match number_token.kind {
            TokenKind::Number(value) => {
                let segment = Self::parse_nonnegative_index_segment(value, number_token.span)
                    .map_err(|_| ParseError::HookEveryNMadiIntervalInvalid {
                        span: number_token.span,
                    })?;
                let interval = segment
                    .parse::<u64>()
                    .map_err(|_| ParseError::HookEveryNMadiIntervalInvalid {
                        span: number_token.span,
                    })?;
                if interval == 0 {
                    return Err(ParseError::HookEveryNMadiIntervalInvalid {
                        span: number_token.span,
                    });
                }
                interval
            }
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "양의 정수 마디 간격",
                    found: other,
                    span: number_token.span,
                })
            }
        };
        let unit_token = self.advance();
        match unit_token.kind {
            TokenKind::Ident(unit) if unit == "마디" => {}
            TokenKind::Ident(unit) => {
                return Err(ParseError::HookEveryNMadiUnitUnsupported {
                    unit,
                    span: unit_token.span,
                })
            }
            other => {
                return Err(ParseError::HookEveryNMadiUnitUnsupported {
                    unit: format!("{other:?}"),
                    span: unit_token.span,
                })
            }
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
            return Err(ParseError::ExpectedRParen {
                span: self.peek().span,
            });
        }
        self.advance();
        let suffix_token = self.advance();
        if !matches!(suffix_token.kind, TokenKind::Mada) {
            let suffix = match suffix_token.kind {
                TokenKind::Halttae => "할때".to_string(),
                TokenKind::Ident(name) => name,
                other => format!("{other:?}"),
            };
            return Err(ParseError::HookEveryNMadiSuffixUnsupported {
                suffix,
                span: suffix_token.span,
            });
        }
        self.parse_hook_tail(start_span, HookKind::EveryNMadi(interval))
    }

    fn parse_hook_tail(&mut self, start_span: Span, kind: HookKind) -> Result<Stmt, ParseError> {
        let (body, span) = self.parse_hook_body_and_span(start_span)?;
        Ok(Stmt::Hook { kind, body, span })
    }

    fn parse_hook_body_and_span(&mut self, start_span: Span) -> Result<(Vec<Stmt>, Span), ParseError> {
        self.skip_newlines();
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            return Err(ParseError::BlockHeaderColonForbidden {
                span: self.peek().span,
            });
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let body = self.parse_block()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        Ok((body, span))
    }

    fn is_open_block_start(&self) -> bool {
        if !self.peek_kind_is(
            |k| matches!(k, TokenKind::Ident(name) if Self::is_open_block_keyword(name)),
        ) {
            return false;
        }
        self.peek_next_non_newline_is(|k| matches!(k, TokenKind::LBrace))
            || (self.peek_next_non_newline_is(|k| matches!(k, TokenKind::Colon))
                && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::LBrace)))
    }

    fn parse_open_block_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_token = self.advance();
        if !matches!(&start_token.kind, TokenKind::Ident(name) if name == "너머") {
            return Err(ParseError::EffectSurfaceAliasForbidden {
                span: start_token.span,
            });
        }
        let start_span = start_token.span;
        self.skip_newlines();
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let body = self.parse_block()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        Ok(Stmt::OpenBlock { body, span })
    }

    fn is_execution_policy_block_start(&self) -> bool {
        self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "실행정책"))
            && (self.peek_kind_n_is(1, |k| matches!(k, TokenKind::LBrace))
                || (self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Colon))
                    && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::LBrace))))
    }

    fn parse_execution_policy_block_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.advance().span;
        self.skip_newlines();
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();

        let mut fields: Vec<(String, String)> = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
                break;
            }
            if self.peek_kind_is(|k| matches!(k, TokenKind::Eof)) {
                return Err(ParseError::ExpectedRBrace {
                    span: self.peek().span,
                });
            }

            let key_token = self.advance();
            let key = match key_token.kind {
                TokenKind::Ident(name) => name,
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "policy key",
                        found: other,
                        span: key_token.span,
                    })
                }
            };
            if !self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "':'",
                    found: self.peek().kind.clone(),
                    span: self.peek().span,
                });
            }
            self.advance();
            let value_token = self.advance();
            let value = match value_token.kind {
                TokenKind::Ident(name) => name,
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "policy enum",
                        found: other,
                        span: value_token.span,
                    })
                }
            };
            fields.push((key, value));
            self.consume_terminator()?;
        }

        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        let args = fields
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",");
        Ok(Stmt::Pragma {
            name: "실행정책".to_string(),
            args,
            span,
        })
    }

    fn is_open_block_keyword(name: &str) -> bool {
        matches!(name, "열림" | "너머" | "효과" | "바깥")
    }

    fn parse_repeat_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.advance().span;
        self.skip_newlines();
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let body = self.parse_block()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        Ok(Stmt::Repeat { body, span })
    }

    fn parse_break_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span;
        self.consume_terminator()?;
        Ok(Stmt::Break { span })
    }

    fn parse_while_stmt(&mut self, condition: Expr) -> Result<Stmt, ParseError> {
        let start_span = condition.span();
        self.advance();
        self.skip_newlines();
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let body = self.parse_block()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        Ok(Stmt::While {
            condition,
            body,
            span,
        })
    }

    fn is_foreach_start(&self) -> bool {
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LParen)) {
            return false;
        }
        let mut idx = self.pos + 1;
        let Some(first) = self.tokens.get(idx) else {
            return false;
        };
        if !matches!(first.kind, TokenKind::Ident(_)) {
            return false;
        }
        idx += 1;
        if let Some(token) = self.tokens.get(idx) {
            if matches!(token.kind, TokenKind::Colon) {
                idx += 1;
                if !self
                    .tokens
                    .get(idx)
                    .map(|tok| matches!(tok.kind, TokenKind::Ident(_)))
                    .unwrap_or(false)
                {
                    return false;
                }
                idx += 1;
            }
        }
        if !self
            .tokens
            .get(idx)
            .map(|tok| matches!(tok.kind, TokenKind::RParen))
            .unwrap_or(false)
        {
            return false;
        }
        idx += 1;
        let mut depth = 0usize;
        while let Some(token) = self.tokens.get(idx) {
            match &token.kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    if depth > 0 {
                        depth -= 1;
                    }
                }
                TokenKind::Daehae if depth == 0 => {
                    return self
                        .tokens
                        .get(idx + 1)
                        .map(|tok| matches!(tok.kind, TokenKind::LBrace | TokenKind::Colon))
                        .unwrap_or(false);
                }
                TokenKind::Ident(name) if depth == 0 && name == "모두" => {
                    return self
                        .tokens
                        .get(idx + 1)
                        .map(|tok| matches!(tok.kind, TokenKind::LBrace))
                        .unwrap_or(false);
                }
                TokenKind::Dot => {
                    if !self
                        .tokens
                        .get(idx + 1)
                        .map(|tok| matches!(tok.kind, TokenKind::Ident(_)))
                        .unwrap_or(false)
                    {
                        break;
                    }
                    idx += 1;
                }
                TokenKind::Newline | TokenKind::RBrace | TokenKind::Eof => break,
                _ => {}
            }
            idx += 1;
        }
        false
    }

    fn parse_foreach_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek().span;
        let item = self.parse_foreach_var()?;
        let mut iterable = self.parse_expr()?;
        let is_modu = self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "모두"));
        if !is_modu && !self.peek_kind_is(|k| matches!(k, TokenKind::Daehae)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'대해' or '모두'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        if is_modu {
            self.advance();
        } else {
            self.advance();
            if let Expr::Path(path) = &mut iterable {
                if let Some(last) = path.segments.last_mut() {
                    if let Some(trimmed) = last.strip_suffix('에') {
                        if !trimmed.is_empty() {
                            *last = trimmed.to_string();
                        }
                    }
                }
            }
            self.skip_newlines();
            if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
                self.advance();
                self.skip_newlines();
            }
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let body = self.parse_block()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        Ok(Stmt::ForEach {
            item,
            iterable,
            body,
            span,
        })
    }

    fn is_quantifier_start(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Ident(_))
            && self.peek_kind_n_is(1, |k| {
                matches!(k, TokenKind::Josa(name) if matches!(name.as_str(), "이" | "가"))
                    || matches!(k, TokenKind::Ident(name) if matches!(name.as_str(), "이" | "가"))
            })
            && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::Ident(_) | TokenKind::Salim))
            && (self.peek_kind_n_is(
                3,
                |k| matches!(k, TokenKind::Ident(name) if name == "낱낱" || name == "낱낱에"),
            ) || self
                .peek_kind_n_is(3, |k| matches!(k, TokenKind::Ident(name) if name == "중")))
    }

    fn parse_quantifier_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek().span;
        let variable_token = self.advance();
        let variable = match variable_token.kind {
            TokenKind::Ident(name) => name,
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "양화 변수",
                    found: other,
                    span: variable_token.span,
                })
            }
        };
        let subject_token = self.advance();
        let subject = match subject_token.kind {
            TokenKind::Josa(name) | TokenKind::Ident(name) => name,
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "'이' 또는 '가'",
                    found: other,
                    span: subject_token.span,
                })
            }
        };
        if subject != "이" && subject != "가" {
            return Err(ParseError::UnexpectedToken {
                expected: "'이' 또는 '가'",
                found: TokenKind::Josa(subject),
                span: subject_token.span,
            });
        }
        let domain_token = self.advance();
        let domain = match domain_token.kind {
            TokenKind::Ident(name) => name,
            TokenKind::Salim => "살림".to_string(),
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "양화 영역 이름",
                    found: other,
                    span: domain_token.span,
                })
            }
        };
        let kind = self.parse_quantifier_kind()?;
        self.skip_newlines();
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        self.enter_scope();
        self.declare_name(&variable);
        let body = match self.parse_block() {
            Ok(body) => body,
            Err(err) => {
                self.exit_scope();
                return Err(err);
            }
        };
        self.exit_scope();
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.validate_quantifier_body(&body, span)?;
        self.consume_terminator()?;
        Ok(Stmt::Quantifier {
            kind,
            variable,
            domain,
            body,
            span,
        })
    }

    fn parse_quantifier_kind(&mut self) -> Result<QuantifierKind, ParseError> {
        if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "낱낱")) {
            self.advance();
            let josa = self.advance();
            if !matches!(josa.kind, TokenKind::Josa(ref name) if name == "에")
                && !matches!(josa.kind, TokenKind::Ident(ref name) if name == "에")
            {
                return Err(ParseError::UnexpectedToken {
                    expected: "'낱낱에 대해'",
                    found: josa.kind,
                    span: josa.span,
                });
            }
            if !self.peek_kind_is(|k| matches!(k, TokenKind::Daehae)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "'대해'",
                    found: self.peek().kind.clone(),
                    span: self.peek().span,
                });
            }
            self.advance();
            return Ok(QuantifierKind::ForAll);
        }
        if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "낱낱에")) {
            self.advance();
            if !self.peek_kind_is(|k| matches!(k, TokenKind::Daehae)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "'대해'",
                    found: self.peek().kind.clone(),
                    span: self.peek().span,
                });
            }
            self.advance();
            return Ok(QuantifierKind::ForAll);
        }
        if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "중")) {
            self.advance();
            if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "딱")) {
                self.advance();
                let one = self.advance();
                if !matches!(one.kind, TokenKind::Ident(ref name) if name == "하나" || name == "하나가")
                {
                    return Err(ParseError::UnexpectedToken {
                        expected: "'중 딱 하나가'",
                        found: one.kind,
                        span: one.span,
                    });
                }
                if !matches!(one.kind, TokenKind::Ident(ref name) if name == "하나가") {
                    let subject = self.advance();
                    if !matches!(subject.kind, TokenKind::Josa(ref name) if name == "가")
                        && !matches!(subject.kind, TokenKind::Ident(ref name) if name == "가")
                    {
                        return Err(ParseError::UnexpectedToken {
                            expected: "'중 딱 하나가'",
                            found: subject.kind,
                            span: subject.span,
                        });
                    }
                }
                return Ok(QuantifierKind::ExistsUnique);
            }
            let one = self.advance();
            if !matches!(one.kind, TokenKind::Ident(ref name) if name == "하나" || name == "하나가")
            {
                return Err(ParseError::UnexpectedToken {
                    expected: "'중 하나가'",
                    found: one.kind,
                    span: one.span,
                });
            }
            if !matches!(one.kind, TokenKind::Ident(ref name) if name == "하나가") {
                let subject = self.advance();
                if !matches!(subject.kind, TokenKind::Josa(ref name) if name == "가")
                    && !matches!(subject.kind, TokenKind::Ident(ref name) if name == "가")
                {
                    return Err(ParseError::UnexpectedToken {
                        expected: "'중 하나가'",
                        found: subject.kind,
                        span: subject.span,
                    });
                }
            }
            return Ok(QuantifierKind::Exists);
        }
        Err(ParseError::UnexpectedToken {
            expected: "'낱낱에 대해' 또는 '중 하나가' 또는 '중 딱 하나가'",
            found: self.peek().kind.clone(),
            span: self.peek().span,
        })
    }

    fn validate_quantifier_body(&self, body: &[Stmt], span: Span) -> Result<(), ParseError> {
        if self.body_has_mutation(body) {
            return Err(ParseError::QuantifierMutationForbidden { span });
        }
        if self.body_has_show(body) {
            return Err(ParseError::QuantifierShowForbidden { span });
        }
        if self.body_has_io(body) {
            return Err(ParseError::QuantifierIoForbidden { span });
        }
        Ok(())
    }

    fn validate_immediate_proof_body(&self, body: &[Stmt], span: Span) -> Result<(), ParseError> {
        if self.body_has_mutation(body) {
            return Err(ParseError::ImmediateProofMutationForbidden { span });
        }
        if self.body_has_show(body) {
            return Err(ParseError::ImmediateProofShowForbidden { span });
        }
        if self.body_has_forbidden_io(body, true) {
            return Err(ParseError::ImmediateProofIoForbidden { span });
        }
        Ok(())
    }

    fn body_has_mutation(&self, body: &[Stmt]) -> bool {
        body.iter().any(|stmt| self.stmt_has_mutation(stmt))
    }

    fn stmt_has_mutation(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Assign { .. } => true,
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .is_some_and(|expr| self.expr_has_mutation(expr))
            }),
            Stmt::Expr { value, .. }
            | Stmt::Return { value, .. }
            | Stmt::Show { value, .. }
            | Stmt::Inspect { value, .. } => self.expr_has_mutation(value),
            Stmt::Receive {
                condition, body, ..
            } => {
                condition
                    .as_ref()
                    .is_some_and(|expr| self.expr_has_mutation(expr))
                    || self.body_has_mutation(body)
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                sender
                    .as_ref()
                    .is_some_and(|expr| self.expr_has_mutation(expr))
                    || self.expr_has_mutation(payload)
                    || self.expr_has_mutation(receiver)
            }
            Stmt::SeedDef { body, .. }
            | Stmt::Hook { body, .. }
            | Stmt::OpenBlock { body, .. }
            | Stmt::BeatBlock { body, .. }
            | Stmt::LifecycleBlock { body, .. }
            | Stmt::Repeat { body, .. }
            | Stmt::ForEach { body, .. }
            | Stmt::Quantifier { body, .. } => self.body_has_mutation(body),
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => self.expr_has_mutation(condition) || self.body_has_mutation(body),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_mutation(condition)
                    || self.body_has_mutation(then_body)
                    || else_body
                        .as_ref()
                        .is_some_and(|body| self.body_has_mutation(body))
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                branches.iter().any(|branch| {
                    self.expr_has_mutation(&branch.condition)
                        || self.body_has_mutation(&branch.body)
                }) || else_body
                    .as_ref()
                    .is_some_and(|body| self.body_has_mutation(body))
            }
            Stmt::While {
                condition, body, ..
            } => self.expr_has_mutation(condition) || self.body_has_mutation(body),
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_mutation(condition)
                    || then_body
                        .as_ref()
                        .is_some_and(|body| self.body_has_mutation(body))
                    || self.body_has_mutation(else_body)
            }
            Stmt::ImportBlock { .. }
            | Stmt::ExportBlock { .. }
            | Stmt::Break { .. }
            | Stmt::Pragma { .. }
            | Stmt::BogaeDraw { .. } => false,
        }
    }

    fn expr_has_mutation(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Unary { expr, .. } => self.expr_has_mutation(expr),
            Expr::Binary { left, right, .. } => {
                self.expr_has_mutation(left) || self.expr_has_mutation(right)
            }
            Expr::FieldAccess { target, .. } => self.expr_has_mutation(target),
            Expr::SeedLiteral { body, .. } => self.expr_has_mutation(body),
            Expr::Call { args, .. } => args.iter().any(|arg| self.expr_has_mutation(&arg.expr)),
            Expr::TemplateFill {
                template, bindings, ..
            } => {
                self.expr_has_mutation(template)
                    || bindings
                        .iter()
                        .any(|binding| self.expr_has_mutation(&binding.value))
            }
            Expr::Pack { bindings, .. } | Expr::FormulaEval { bindings, .. } => bindings
                .iter()
                .any(|binding| self.expr_has_mutation(&binding.value)),
            Expr::FormulaFill {
                formula, bindings, ..
            } => {
                self.expr_has_mutation(formula)
                    || bindings
                        .iter()
                        .any(|binding| self.expr_has_mutation(&binding.value))
            }
            Expr::Literal(_, _)
            | Expr::Path(_)
            | Expr::Atom { .. }
            | Expr::Assertion { .. }
            | Expr::Formula { .. }
            | Expr::Template { .. } => false,
        }
    }

    fn body_has_show(&self, body: &[Stmt]) -> bool {
        body.iter().any(|stmt| self.stmt_has_show(stmt))
    }

    fn stmt_has_show(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Show { .. } | Stmt::Inspect { .. } => true,
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .is_some_and(|expr| self.expr_has_show(expr))
            }),
            Stmt::Assign { value, .. } | Stmt::Expr { value, .. } | Stmt::Return { value, .. } => {
                self.expr_has_show(value)
            }
            Stmt::Receive {
                condition, body, ..
            } => {
                condition
                    .as_ref()
                    .is_some_and(|expr| self.expr_has_show(expr))
                    || self.body_has_show(body)
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                sender.as_ref().is_some_and(|expr| self.expr_has_show(expr))
                    || self.expr_has_show(payload)
                    || self.expr_has_show(receiver)
            }
            Stmt::SeedDef { body, .. }
            | Stmt::Hook { body, .. }
            | Stmt::OpenBlock { body, .. }
            | Stmt::BeatBlock { body, .. }
            | Stmt::LifecycleBlock { body, .. }
            | Stmt::Repeat { body, .. }
            | Stmt::ForEach { body, .. }
            | Stmt::Quantifier { body, .. } => self.body_has_show(body),
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => self.expr_has_show(condition) || self.body_has_show(body),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_show(condition)
                    || self.body_has_show(then_body)
                    || else_body
                        .as_ref()
                        .is_some_and(|body| self.body_has_show(body))
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                branches.iter().any(|branch| {
                    self.expr_has_show(&branch.condition) || self.body_has_show(&branch.body)
                }) || else_body
                    .as_ref()
                    .is_some_and(|body| self.body_has_show(body))
            }
            Stmt::While {
                condition, body, ..
            } => self.expr_has_show(condition) || self.body_has_show(body),
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_show(condition)
                    || then_body
                        .as_ref()
                        .is_some_and(|body| self.body_has_show(body))
                    || self.body_has_show(else_body)
            }
            Stmt::ImportBlock { .. }
            | Stmt::ExportBlock { .. }
            | Stmt::Break { .. }
            | Stmt::Pragma { .. }
            | Stmt::BogaeDraw { .. } => false,
        }
    }

    fn expr_has_show(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Unary { expr, .. } => self.expr_has_show(expr),
            Expr::Binary { left, right, .. } => {
                self.expr_has_show(left) || self.expr_has_show(right)
            }
            Expr::FieldAccess { target, .. } => self.expr_has_show(target),
            Expr::SeedLiteral { body, .. } => self.expr_has_show(body),
            Expr::Call { args, .. } => args.iter().any(|arg| self.expr_has_show(&arg.expr)),
            Expr::TemplateFill {
                template, bindings, ..
            } => {
                self.expr_has_show(template)
                    || bindings
                        .iter()
                        .any(|binding| self.expr_has_show(&binding.value))
            }
            Expr::Pack { bindings, .. } | Expr::FormulaEval { bindings, .. } => bindings
                .iter()
                .any(|binding| self.expr_has_show(&binding.value)),
            Expr::FormulaFill {
                formula, bindings, ..
            } => {
                self.expr_has_show(formula)
                    || bindings
                        .iter()
                        .any(|binding| self.expr_has_show(&binding.value))
            }
            Expr::Literal(_, _)
            | Expr::Path(_)
            | Expr::Atom { .. }
            | Expr::Assertion { .. }
            | Expr::Formula { .. }
            | Expr::Template { .. } => false,
        }
    }

    fn body_has_io(&self, body: &[Stmt]) -> bool {
        self.body_has_forbidden_io(body, false)
    }

    fn body_has_forbidden_io(&self, body: &[Stmt], allow_solver_hooks: bool) -> bool {
        body.iter()
            .any(|stmt| self.stmt_has_forbidden_io(stmt, allow_solver_hooks))
    }

    fn stmt_has_forbidden_io(&self, stmt: &Stmt, allow_solver_hooks: bool) -> bool {
        match stmt {
            Stmt::OpenBlock { .. } => true,
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .is_some_and(|expr| self.expr_has_forbidden_io(expr, allow_solver_hooks))
                    || item.maegim.as_ref().is_some_and(|spec| {
                        spec.fields.iter().any(|field| {
                            self.expr_has_forbidden_io(&field.value, allow_solver_hooks)
                        })
                    })
            }),
            Stmt::Assign { value, .. } | Stmt::Expr { value, .. } | Stmt::Return { value, .. } => {
                self.expr_has_forbidden_io(value, allow_solver_hooks)
            }
            Stmt::Receive {
                condition, body, ..
            } => {
                condition
                    .as_ref()
                    .is_some_and(|expr| self.expr_has_forbidden_io(expr, allow_solver_hooks))
                    || self.body_has_forbidden_io(body, allow_solver_hooks)
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                sender
                    .as_ref()
                    .is_some_and(|expr| self.expr_has_forbidden_io(expr, allow_solver_hooks))
                    || self.expr_has_forbidden_io(payload, allow_solver_hooks)
                    || self.expr_has_forbidden_io(receiver, allow_solver_hooks)
            }
            Stmt::SeedDef { body, .. }
            | Stmt::Hook { body, .. }
            | Stmt::BeatBlock { body, .. }
            | Stmt::LifecycleBlock { body, .. }
            | Stmt::Repeat { body, .. }
            | Stmt::ForEach { body, .. }
            | Stmt::Quantifier { body, .. } => self.body_has_forbidden_io(body, allow_solver_hooks),
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => {
                self.expr_has_forbidden_io(condition, allow_solver_hooks)
                    || self.body_has_forbidden_io(body, allow_solver_hooks)
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_forbidden_io(condition, allow_solver_hooks)
                    || self.body_has_forbidden_io(then_body, allow_solver_hooks)
                    || else_body
                        .as_ref()
                        .is_some_and(|body| self.body_has_forbidden_io(body, allow_solver_hooks))
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                branches.iter().any(|branch| {
                    self.expr_has_forbidden_io(&branch.condition, allow_solver_hooks)
                        || self.body_has_forbidden_io(&branch.body, allow_solver_hooks)
                }) || else_body
                    .as_ref()
                    .is_some_and(|body| self.body_has_forbidden_io(body, allow_solver_hooks))
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.expr_has_forbidden_io(condition, allow_solver_hooks)
                    || self.body_has_forbidden_io(body, allow_solver_hooks)
            }
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_forbidden_io(condition, allow_solver_hooks)
                    || then_body
                        .as_ref()
                        .is_some_and(|body| self.body_has_forbidden_io(body, allow_solver_hooks))
                    || self.body_has_forbidden_io(else_body, allow_solver_hooks)
            }
            Stmt::ImportBlock { .. }
            | Stmt::ExportBlock { .. }
            | Stmt::Show { .. }
            | Stmt::Inspect { .. }
            | Stmt::Break { .. }
            | Stmt::Pragma { .. }
            | Stmt::BogaeDraw { .. } => false,
        }
    }

    fn expr_has_forbidden_io(&self, expr: &Expr, allow_solver_hooks: bool) -> bool {
        match expr {
            Expr::Call { name, args, .. } => {
                let canon = stdlib::canonicalize_stdlib_alias(name);
                let is_solver_hook = matches!(canon, "열림.풀이.확인" | "반례찾기" | "해찾기");
                (canon.starts_with("열림.") && !(allow_solver_hooks && is_solver_hook))
                    || matches!(
                        canon,
                        "입력키" | "입력키?" | "입력키!" | "눌렸나" | "막눌렸나"
                    )
                    || args
                        .iter()
                        .any(|arg| self.expr_has_forbidden_io(&arg.expr, allow_solver_hooks))
            }
            Expr::Unary { expr, .. } => self.expr_has_forbidden_io(expr, allow_solver_hooks),
            Expr::Binary { left, right, .. } => {
                self.expr_has_forbidden_io(left, allow_solver_hooks)
                    || self.expr_has_forbidden_io(right, allow_solver_hooks)
            }
            Expr::FieldAccess { target, .. } => {
                self.expr_has_forbidden_io(target, allow_solver_hooks)
            }
            Expr::SeedLiteral { body, .. } => self.expr_has_forbidden_io(body, allow_solver_hooks),
            Expr::TemplateFill {
                template, bindings, ..
            } => {
                self.expr_has_forbidden_io(template, allow_solver_hooks)
                    || bindings.iter().any(|binding| {
                        self.expr_has_forbidden_io(&binding.value, allow_solver_hooks)
                    })
            }
            Expr::Pack { bindings, .. } | Expr::FormulaEval { bindings, .. } => bindings
                .iter()
                .any(|binding| self.expr_has_forbidden_io(&binding.value, allow_solver_hooks)),
            Expr::FormulaFill {
                formula, bindings, ..
            } => {
                self.expr_has_forbidden_io(formula, allow_solver_hooks)
                    || bindings.iter().any(|binding| {
                        self.expr_has_forbidden_io(&binding.value, allow_solver_hooks)
                    })
            }
            Expr::Literal(_, _)
            | Expr::Path(_)
            | Expr::Atom { .. }
            | Expr::Assertion { .. }
            | Expr::Formula { .. }
            | Expr::Template { .. } => false,
        }
    }

    fn parse_foreach_var(&mut self) -> Result<String, ParseError> {
        let _start_span = self.advance().span;
        let name_token = self.advance();
        let name = match name_token.kind {
            TokenKind::Ident(name) => name,
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "iteration variable",
                    found: other,
                    span: name_token.span,
                })
            }
        };
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            if !self.peek_kind_is(|k| matches!(k, TokenKind::Ident(_))) {
                return Err(ParseError::UnexpectedToken {
                    expected: "type name",
                    found: self.peek().kind.clone(),
                    span: self.peek().span,
                });
            }
            self.advance();
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
            return Err(ParseError::ExpectedRParen {
                span: self.peek().span,
            });
        }
        self.advance();
        Ok(name)
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.enter_scope();
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            if self.pending_stmts.is_empty()
                && self.peek_kind_is(|k| matches!(k, TokenKind::RBrace))
            {
                break;
            }
            if self.peek_kind_is(|k| matches!(k, TokenKind::Eof)) {
                return Err(ParseError::ExpectedRBrace {
                    span: self.peek().span,
                });
            }
            let stmt = self.parse_stmt_in_block()?;
            if let Some(stmt) = stmt {
                stmts.push(stmt);
            }
        }
        self.exit_scope();
        Ok(stmts)
    }

    fn try_parse_seed_def(&mut self) -> Result<Option<Stmt>, ParseError> {
        let start_pos = self.pos;
        let start_span = self.peek().span;
        let mut params: Vec<ParamPin> = Vec::new();

        if self.peek_kind_is(|k| matches!(k, TokenKind::LParen)) {
            match self.parse_seed_params() {
                Ok(list) => params = list,
                Err(_) => {
                    self.pos = start_pos;
                    return Ok(None);
                }
            }
            self.skip_newlines();
        }

        let name_token = self.peek().clone();
        let name = match name_token.kind {
            TokenKind::Ident(name) => {
                self.advance();
                name
            }
            TokenKind::EveryMadi => {
                self.advance();
                "매마디".to_string()
            }
            _ => {
                self.pos = start_pos;
                return Ok(None);
            }
        };

        let aliases = self.parse_seed_aliases(&name)?;

        if !self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.pos = start_pos;
            return Ok(None);
        }
        self.advance();

        let kind = match self.peek().kind.clone() {
            TokenKind::Ident(kind_name) => {
                self.advance();
                if let Some(kind) = SeedKind::from_name(&kind_name) {
                    kind
                } else if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
                    self.advance();
                    let kind_token = self.peek().clone();
                    let kind_name = match kind_token.kind {
                        TokenKind::Ident(name) => {
                            self.advance();
                            name
                        }
                        _ => {
                            self.pos = start_pos;
                            return Ok(None);
                        }
                    };
                    let Some(kind) = SeedKind::from_name(&kind_name) else {
                        self.pos = start_pos;
                        return Ok(None);
                    };
                    kind
                } else if self.peek_kind_is(|k| matches!(k, TokenKind::Equal)) {
                    SeedKind::Named(kind_name)
                } else {
                    self.pos = start_pos;
                    return Ok(None);
                }
            }
            _ => {
                self.pos = start_pos;
                return Ok(None);
            }
        };

        if name == "매틱" && matches!(kind, SeedKind::Umjikssi) && !self.allow_compat_matic_entry
        {
            return Err(ParseError::CompatMaticEntryDisabled {
                span: name_token.span,
            });
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            let mut ignored = Vec::new();
            self.skip_seed_type_tokens(&mut ignored)?;
        }

        if !self.peek_kind_is(|k| matches!(k, TokenKind::Equal)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'='",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        self.skip_newlines();

        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        self.seed_kind_stack.push(kind.clone());
        let body = self.parse_block();
        self.seed_kind_stack.pop();
        let body = body?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        let primary = Stmt::SeedDef {
            name: name.clone(),
            params: params.clone(),
            kind: kind.clone(),
            body: body.clone(),
            span,
        };
        for alias in aliases {
            self.pending_stmts.push_back(Stmt::SeedDef {
                name: alias,
                params: params.clone(),
                kind: kind.clone(),
                body: body.clone(),
                span,
            });
        }
        Ok(Some(primary))
    }

    fn try_parse_immediate_proof_stmt(&mut self) -> Result<Option<Stmt>, ParseError> {
        let start_pos = self.pos;
        let name_token = self.peek().clone();
        let TokenKind::Ident(name) = name_token.kind.clone() else {
            return Ok(None);
        };
        if !self.peek_kind_n_is(
            1,
            |k| matches!(k, TokenKind::Ident(text) if text == "밝히기"),
        ) {
            return Ok(None);
        }
        if !self.peek_kind_n_is(2, |k| matches!(k, TokenKind::LBrace)) {
            return Ok(None);
        }

        self.advance();
        self.advance();
        self.advance();
        self.seed_kind_stack.push(SeedKind::Balhigi);
        let body = self.parse_block();
        self.seed_kind_stack.pop();
        let body = body?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            self.pos = start_pos;
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = name_token.span.merge(end_span);
        self.consume_terminator()?;
        self.validate_immediate_proof_body(&body, span)?;

        let primary = Stmt::SeedDef {
            name: name.clone(),
            params: Vec::new(),
            kind: SeedKind::Balhigi,
            body,
            span,
        };
        self.pending_stmts.push_back(Stmt::Expr {
            value: Expr::Call {
                name,
                args: Vec::new(),
                span,
            },
            span,
        });
        Ok(Some(primary))
    }

    fn parse_seed_aliases(&mut self, primary_name: &str) -> Result<Vec<String>, ParseError> {
        let mut aliases: Vec<String> = Vec::new();
        loop {
            let token = self.peek().clone();
            let TokenKind::Ident(raw_alias) = token.kind.clone() else {
                break;
            };
            let Some(stripped) = raw_alias.strip_prefix('~') else {
                break;
            };
            if stripped.is_empty() || !Self::is_seed_alias_ident(stripped) {
                return Err(ParseError::UnexpectedToken {
                    expected: "'~별명' (예: ~막으)",
                    found: TokenKind::Ident(raw_alias),
                    span: token.span,
                });
            }
            if stripped == primary_name || aliases.iter().any(|alias| alias == stripped) {
                return Err(ParseError::UnexpectedToken {
                    expected: "중복되지 않는 씨앗 별명",
                    found: TokenKind::Ident(format!("~{}", stripped)),
                    span: token.span,
                });
            }
            self.advance();
            aliases.push(stripped.to_string());
        }
        Ok(aliases)
    }

    fn is_seed_alias_ident(name: &str) -> bool {
        let mut chars = name.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !(first == '_' || first.is_alphabetic()) {
            return false;
        }
        chars.all(|ch| ch == '_' || ch.is_alphabetic() || ch.is_ascii_digit())
    }

    fn parse_seed_params(&mut self) -> Result<Vec<ParamPin>, ParseError> {
        self.advance(); // '('
        self.skip_newlines();
        let mut params: Vec<ParamPin> = Vec::new();
        if self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
            self.advance();
            return Ok(params);
        }
        loop {
            self.skip_newlines();
            let name_token = self.peek().clone();
            let name = match name_token.kind {
                TokenKind::Ident(name) => {
                    self.advance();
                    name
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "seed parameter",
                        found: other,
                        span: name_token.span,
                    })
                }
            };
            let mut josa_list: Vec<String> = Vec::new();
            if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
                self.advance();
                self.skip_seed_type_tokens(&mut josa_list)?;
            }
            if self.peek_kind_is(|k| matches!(k, TokenKind::Equal)) {
                self.advance();
                let _ = self.parse_expr()?;
            }
            if self.peek_kind_is(|k| matches!(k, TokenKind::Josa(_))) {
                if let TokenKind::Josa(josa) = self.advance().kind {
                    if !josa_list.contains(&josa) {
                        josa_list.push(josa);
                    }
                }
            }
            params.push(ParamPin {
                name,
                josa_list,
                span: name_token.span,
            });
            self.skip_newlines();
            if self.peek_kind_is(|k| matches!(k, TokenKind::Comma)) {
                self.advance();
                continue;
            }
            break;
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
            return Err(ParseError::ExpectedRParen {
                span: self.peek().span,
            });
        }
        self.advance();
        Ok(params)
    }

    fn skip_seed_type_tokens(&mut self, josa_list: &mut Vec<String>) -> Result<(), ParseError> {
        while !self
            .peek_kind_is(|k| matches!(k, TokenKind::Comma | TokenKind::RParen | TokenKind::Equal))
        {
            if self.peek_kind_is(|k| matches!(k, TokenKind::Eof | TokenKind::Newline)) {
                break;
            }
            if let TokenKind::Josa(josa) = self.peek().kind.clone() {
                if !josa_list.contains(&josa) {
                    josa_list.push(josa);
                }
            }
            self.advance();
        }
        Ok(())
    }

    fn parse_choose_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.advance().span;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            return Err(ParseError::UnexpectedToken {
                expected: "':'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let mut branches = Vec::new();
        let mut else_body: Option<Vec<Stmt>> = None;
        let mut exhaustive = false;
        let mut end_span = start_span;

        loop {
            self.skip_newlines();
            if self
                .peek_kind_is(|k| matches!(k, TokenKind::Dot | TokenKind::RBrace | TokenKind::Eof))
            {
                break;
            }
            if self.is_choose_legacy_else_start() {
                let (body, span) = self.parse_choose_legacy_else_body()?;
                else_body = Some(body);
                end_span = span;
                self.skip_newlines();
                if !self.peek_kind_is(|k| {
                    matches!(
                        k,
                        TokenKind::Dot | TokenKind::Newline | TokenKind::RBrace | TokenKind::Eof
                    )
                }) {
                    return Err(ParseError::CaseElseNotLast {
                        span: self.peek().span,
                    });
                }
                break;
            }
            if self.is_choose_canonical_else_start() {
                let (body, span) = self.parse_choose_canonical_else_body()?;
                else_body = Some(body);
                end_span = span;
                self.skip_newlines();
                if self.is_choose_complete_marker() {
                    end_span = self.parse_choose_complete_marker()?;
                    exhaustive = true;
                }
                self.skip_newlines();
                if !self.peek_kind_is(|k| {
                    matches!(
                        k,
                        TokenKind::Dot | TokenKind::Newline | TokenKind::RBrace | TokenKind::Eof
                    )
                }) {
                    return Err(ParseError::CaseElseNotLast {
                        span: self.peek().span,
                    });
                }
                break;
            }
            if self.is_choose_complete_marker() {
                end_span = self.parse_choose_complete_marker()?;
                exhaustive = true;
                self.skip_newlines();
                if !self.peek_kind_is(|k| {
                    matches!(
                        k,
                        TokenKind::Dot | TokenKind::Newline | TokenKind::RBrace | TokenKind::Eof
                    )
                }) {
                    return Err(ParseError::UnexpectedToken {
                        expected: "문장 끝",
                        found: self.peek().kind.clone(),
                        span: self.peek().span,
                    });
                }
                break;
            }
            let (condition, body, span) = self.parse_choose_branch()?;
            end_span = span;
            branches.push(ChooseBranch { condition, body });
        }

        if !branches.is_empty() && else_body.is_none() && !exhaustive {
            return Err(ParseError::CaseCompletionRequired { span: end_span });
        }
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        Ok(Stmt::Choose {
            branches,
            else_body,
            exhaustive,
            span,
        })
    }

    fn is_choose_legacy_else_start(&self) -> bool {
        self.peek_kind_is(|k| matches!(k, TokenKind::Aniramyeon))
    }

    fn parse_choose_legacy_else_body(&mut self) -> Result<(Vec<Stmt>, Span), ParseError> {
        let start_span = self.advance().span;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            return Err(ParseError::UnexpectedToken {
                expected: "':'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let (body, end_span) = self.parse_choose_body_block()?;
        Ok((body, start_span.merge(end_span)))
    }

    fn is_choose_canonical_else_start(&self) -> bool {
        self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "그밖의"))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Ident(name) if name == "경우"))
    }

    fn parse_choose_canonical_else_body(&mut self) -> Result<(Vec<Stmt>, Span), ParseError> {
        let start_span = self.advance().span;
        let case_token = self.advance();
        if !matches!(case_token.kind, TokenKind::Ident(ref name) if name == "경우") {
            return Err(ParseError::UnexpectedToken {
                expected: "'경우'",
                found: case_token.kind,
                span: case_token.span,
            });
        }
        let (body, end_span) = self.parse_choose_body_block()?;
        Ok((body, start_span.merge(end_span)))
    }

    fn is_choose_complete_marker(&self) -> bool {
        self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "모든"))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Ident(name) if name == "경우"))
            && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::Ident(name) if name == "다룸"))
    }

    fn parse_choose_complete_marker(&mut self) -> Result<Span, ParseError> {
        let start_span = self.advance().span;
        let case_token = self.advance();
        if !matches!(case_token.kind, TokenKind::Ident(ref name) if name == "경우") {
            return Err(ParseError::UnexpectedToken {
                expected: "'경우'",
                found: case_token.kind,
                span: case_token.span,
            });
        }
        let done_token = self.advance();
        if !matches!(done_token.kind, TokenKind::Ident(ref name) if name == "다룸") {
            return Err(ParseError::UnexpectedToken {
                expected: "'다룸'",
                found: done_token.kind,
                span: done_token.span,
            });
        }
        Ok(start_span.merge(done_token.span))
    }

    fn parse_choose_branch(&mut self) -> Result<(Expr, Vec<Stmt>, Span), ParseError> {
        let condition = self.parse_choose_condition_expr()?;
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            let (body, end_span) = self.parse_choose_body_block()?;
            let span = condition.span().merge(end_span);
            return Ok((condition, body, span));
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "인")) {
            return Err(ParseError::UnexpectedToken {
                expected: "':' 또는 '인 경우'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let case_token = self.advance();
        if !matches!(case_token.kind, TokenKind::Ident(ref name) if name == "경우") {
            return Err(ParseError::UnexpectedToken {
                expected: "'경우'",
                found: case_token.kind,
                span: case_token.span,
            });
        }
        let (body, end_span) = self.parse_choose_body_block()?;
        let span = condition.span().merge(end_span);
        Ok((condition, body, span))
    }

    fn parse_choose_condition_expr(&mut self) -> Result<Expr, ParseError> {
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        let start_span = self.advance().span;
        let expr = self.parse_expr()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        Ok(match expr {
            Expr::Unary { op, expr, .. } => Expr::Unary { op, expr, span },
            Expr::Binary {
                left, op, right, ..
            } => Expr::Binary {
                left,
                op,
                right,
                span,
            },
            Expr::Call { name, args, .. } => Expr::Call { name, args, span },
            Expr::FieldAccess { target, field, .. } => Expr::FieldAccess {
                target,
                field,
                span,
            },
            Expr::SeedLiteral { param, body, .. } => Expr::SeedLiteral { param, body, span },
            Expr::Formula { dialect, body, .. } => Expr::Formula {
                dialect,
                body,
                span,
            },
            Expr::Assertion { assertion, .. } => Expr::Assertion { assertion, span },
            Expr::FormulaEval {
                dialect,
                body,
                bindings,
                ..
            } => Expr::FormulaEval {
                dialect,
                body,
                bindings,
                span,
            },
            Expr::Template { body, .. } => Expr::Template { body, span },
            Expr::TemplateFill {
                template, bindings, ..
            } => Expr::TemplateFill {
                template,
                bindings,
                span,
            },
            Expr::Pack { bindings, .. } => Expr::Pack { bindings, span },
            Expr::FormulaFill {
                formula, bindings, ..
            } => Expr::FormulaFill {
                formula,
                bindings,
                span,
            },
            Expr::Literal(value, _) => Expr::Literal(value, span),
            Expr::Path(mut path) => {
                path.span = span;
                Expr::Path(path)
            }
            Expr::Atom { text, .. } => Expr::Atom { text, span },
        })
    }

    fn parse_choose_body_block(&mut self) -> Result<(Vec<Stmt>, Span), ParseError> {
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let body = self.parse_block()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        Ok((body, end_span))
    }

    fn parse_if_stmt(&mut self, condition: Expr) -> Result<Stmt, ParseError> {
        let start_span = condition.span();
        self.advance();
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let then_body = self.parse_block()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let mut end_span = self.advance().span;

        let mut lookahead = self.pos;
        while self
            .tokens
            .get(lookahead)
            .map(|token| matches!(token.kind, TokenKind::Newline))
            .unwrap_or(false)
        {
            lookahead += 1;
        }

        if self
            .tokens
            .get(lookahead)
            .map(|token| matches!(token.kind, TokenKind::Anigo))
            .unwrap_or(false)
        {
            self.pos = lookahead;
            self.advance();
            self.skip_newlines();
            let else_condition = self.parse_condition_expr()?;
            if !self.peek_kind_is(|k| matches!(k, TokenKind::Ilttae)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "'일때'",
                    found: self.peek().kind.clone(),
                    span: self.peek().span,
                });
            }
            let stmt = self.parse_if_stmt(else_condition)?;
            let stmt_span = match &stmt {
                Stmt::If { span, .. } => *span,
                _ => end_span,
            };
            let span = start_span.merge(stmt_span);
            return Ok(Stmt::If {
                condition,
                then_body,
                else_body: Some(vec![stmt]),
                span,
            });
        }

        let else_body = if self
            .tokens
            .get(lookahead)
            .map(|token| matches!(token.kind, TokenKind::Aniramyeon))
            .unwrap_or(false)
        {
            self.pos = lookahead;
            self.advance();
            if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "'{'",
                    found: self.peek().kind.clone(),
                    span: self.peek().span,
                });
            }
            self.advance();
            let body = self.parse_block()?;
            if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
                return Err(ParseError::ExpectedRBrace {
                    span: self.peek().span,
                });
            }
            end_span = self.advance().span;
            Some(body)
        } else {
            None
        };

        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        Ok(Stmt::If {
            condition,
            then_body,
            else_body,
            span,
        })
    }

    fn parse_contract_stmt(&mut self, condition: Expr) -> Result<Stmt, ParseError> {
        let start_span = condition.span();
        let kind = if self.peek_kind_is(|k| matches!(k, TokenKind::Jeonjehae)) {
            self.advance();
            ContractKind::Pre
        } else {
            self.advance();
            ContractKind::Post
        };
        let mode = self.parse_contract_mode()?;
        self.skip_newlines();
        if !self.peek_kind_is(|k| matches!(k, TokenKind::Aniramyeon)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'아니면'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let else_body = self.parse_block()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let mut end_span = self.advance().span;
        self.skip_newlines();
        let then_body = if self.peek_kind_is(|k| matches!(k, TokenKind::Majeumyeon)) {
            self.advance();
            if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "'{'",
                    found: self.peek().kind.clone(),
                    span: self.peek().span,
                });
            }
            self.advance();
            let body = self.parse_block()?;
            if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
                return Err(ParseError::ExpectedRBrace {
                    span: self.peek().span,
                });
            }
            end_span = self.advance().span;
            Some(body)
        } else {
            None
        };
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        Ok(Stmt::Contract {
            kind,
            mode,
            condition,
            then_body,
            else_body,
            span,
        })
    }

    fn parse_condition_expr(&mut self) -> Result<Expr, ParseError> {
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return self.parse_expr();
        }
        let start_span = self.advance().span;
        let expr = self.parse_expr()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        self.skip_newlines();
        match &self.peek().kind {
            TokenKind::EqEq => {
                self.advance();
                Ok(expr)
            }
            TokenKind::NotEq => {
                self.advance();
                let span = start_span.merge(end_span);
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                if name == "아닌것" || name == "not" {
                    let span = start_span.merge(end_span);
                    return Ok(Expr::Unary {
                        op: UnaryOp::Not,
                        expr: Box::new(expr),
                        span,
                    });
                }
                if name != "인것" && name != "is" {
                    return Err(ParseError::UnexpectedToken {
                        expected: "'인것' or '아닌것' or '==/!='",
                        found: TokenKind::Ident(name),
                        span: self.peek().span,
                    });
                }
                Ok(expr)
            }
            other => Err(ParseError::UnexpectedToken {
                expected: "'인것' or '아닌것' or '==/!='",
                found: other.clone(),
                span: self.peek().span,
            }),
        }
    }

    fn parse_contract_mode(&mut self) -> Result<ContractMode, ParseError> {
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LParen)) {
            return Ok(ContractMode::Abort);
        }
        let start_span = self.advance().span;
        let (name, name_span) = match &self.peek().kind {
            TokenKind::Ident(name) => (name.clone(), self.peek().span),
            TokenKind::Break => ("중단".to_string(), self.peek().span),
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "'알림' or '물림' or '중단'",
                    found: other.clone(),
                    span: self.peek().span,
                })
            }
        };
        self.advance();
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
            return Err(ParseError::ExpectedRParen {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let _ = start_span.merge(end_span);
        match name.as_str() {
            "알림" => Ok(ContractMode::Alert),
            "물림" | "중단" => Ok(ContractMode::Abort),
            _ => Err(ParseError::UnexpectedToken {
                expected: "'알림' or '물림' or '중단'",
                found: TokenKind::Ident(name),
                span: name_span,
            }),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_logical_and()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Or => Some(BinaryOp::Or),
                TokenKind::Ident(ref name) if name == "또는" => Some(BinaryOp::Or),
                _ => None,
            };
            let Some(op) = op else { break };
            self.advance();
            let right = self.parse_logical_and()?;
            let span = expr.span().merge(right.span());
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_comparison()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::And => Some(BinaryOp::And),
                TokenKind::Ident(ref name) if name == "그리고" => Some(BinaryOp::And),
                _ => None,
            };
            let Some(op) = op else { break };
            self.advance();
            let right = self.parse_comparison()?;
            let span = expr.span().merge(right.span());
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_range()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::EqEq => Some(BinaryOp::Eq),
                TokenKind::NotEq => Some(BinaryOp::NotEq),
                TokenKind::Lt => Some(BinaryOp::Lt),
                TokenKind::Lte => Some(BinaryOp::Lte),
                TokenKind::Gt => Some(BinaryOp::Gt),
                TokenKind::Gte => Some(BinaryOp::Gte),
                _ => None,
            };
            let Some(op) = op else { break };
            self.advance();
            let right = self.parse_range()?;
            let span = expr.span().merge(right.span());
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_range(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_additive()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::DotDot | TokenKind::DotDotEq)) {
            return Ok(left);
        }
        let op_token = self.advance();
        let inclusive = matches!(op_token.kind, TokenKind::DotDotEq);
        let right = self.parse_additive()?;
        let span = left.span().merge(right.span());
        let flag_raw = if inclusive {
            Fixed64::from_int(1).raw()
        } else {
            0
        };
        let flag_expr = Expr::Literal(
            Literal::Num(NumberLiteral {
                raw: flag_raw,
                unit: None,
            }),
            op_token.span,
        );
        Ok(Expr::Call {
            name: "표준.범위".to_string(),
            args: vec![
                self.new_arg_binding(left),
                self.new_arg_binding(right),
                self.new_arg_binding(flag_expr),
            ],
            span,
        })
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_multiplicative()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Plus => Some(BinaryOp::Add),
                TokenKind::Minus => Some(BinaryOp::Sub),
                _ => None,
            };
            let Some(op) = op else { break };
            self.advance();
            let right = self.parse_multiplicative()?;
            let span = expr.span().merge(right.span());
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Star => Some(BinaryOp::Mul),
                TokenKind::Slash => Some(BinaryOp::Div),
                TokenKind::Percent => Some(BinaryOp::Mod),
                _ => None,
            };
            let Some(op) = op else { break };
            self.advance();
            let right = self.parse_unary()?;
            let span = expr.span().merge(right.span());
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.peek_kind_is(|k| matches!(k, TokenKind::Minus)) {
            let start = self.advance().span;
            let expr = self.parse_unary()?;
            let span = start.merge(expr.span());
            return Ok(Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
                span,
            });
        }
        if self.peek_kind_is(|k| matches!(k, TokenKind::Plus)) {
            self.advance();
            return self.parse_unary();
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.peek_kind_is(|k| matches!(k, TokenKind::LBracket)) {
                let start_span = expr.span();
                self.advance();
                let index_expr = self.parse_expr()?;
                if !self.peek_kind_is(|k| matches!(k, TokenKind::RBracket)) {
                    return Err(ParseError::UnexpectedToken {
                        expected: "']'",
                        found: self.peek().kind.clone(),
                        span: self.peek().span,
                    });
                }
                let end_span = self.advance().span;
                let span = start_span.merge(end_span);
                expr = Expr::Call {
                    name: "차림.값".to_string(),
                    args: vec![self.new_arg_binding(expr), self.new_arg_binding(index_expr)],
                    span,
                };
                continue;
            }
            if let Some((field, field_span)) = self.try_parse_dot_segment()? {
                let start_span = expr.span();
                let span = start_span.merge(field_span);
                expr = Expr::FieldAccess {
                    target: Box::new(expr),
                    field,
                    span,
                };
                continue;
            }
            break;
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        if self.peek_kind_is(|k| matches!(k, TokenKind::LParen)) {
            if self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Atom(_))) {
                let start_span = self.peek().span;
                return self.parse_formula_expr(None, start_span);
            }
            if self.is_binding_paren_start() {
                let (bindings, start_span) = self.parse_bindings()?;
                self.skip_newlines();
                if self.peek_kind_is(|k| matches!(k, TokenKind::LParen))
                    && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Atom(_)))
                {
                    return self.parse_formula_expr(Some(bindings), start_span);
                }
                self.skip_tag_blocks();
                if self.peek_kind_is(|k| matches!(k, TokenKind::Template(_))) {
                    let template = self.parse_template_expr()?;
                    let span = start_span.merge(template.span());
                    return Ok(Expr::TemplateFill {
                        template: Box::new(template),
                        bindings,
                        span,
                    });
                }
                if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "인")) {
                    return self.parse_in_fill(bindings, start_span);
                }
                if self.peek_kind_is(|k| matches!(k, TokenKind::Chaewugi)) {
                    return self.parse_template_fill_explicit(bindings, start_span);
                }
                if self.peek_kind_is(|k| matches!(k, TokenKind::Salim))
                    || self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name != "의"))
                {
                    let (name, name_span) = self.parse_call_name()?;
                    let pack = Expr::Pack {
                        bindings,
                        span: start_span,
                    };
                    return Ok(Expr::Call {
                        name,
                        args: vec![self.new_arg_binding(pack)],
                        span: start_span.merge(name_span),
                    });
                }
                return Ok(Expr::Pack {
                    bindings,
                    span: start_span,
                });
            }
        }

        let token = self.peek().clone();
        match token.kind {
            TokenKind::String(text) => {
                let span = self.advance().span;
                Ok(Expr::Literal(Literal::Str(text), span))
            }
            TokenKind::Template(body) => {
                let span = self.advance().span;
                Ok(Expr::Template { body, span })
            }
            TokenKind::Assertion(raw) => {
                let span = self.advance().span;
                let assertion = self.parse_assertion_block(&raw, span)?;
                Ok(Expr::Assertion { assertion, span })
            }
            TokenKind::Number(value) => {
                let span = self.advance().span;
                let (unit, span) = self.parse_unit_postfix(span)?;
                Ok(Expr::Literal(
                    Literal::Num(NumberLiteral { raw: value, unit }),
                    span,
                ))
            }
            TokenKind::True => {
                let span = self.advance().span;
                Ok(Expr::Literal(Literal::Bool(true), span))
            }
            TokenKind::False => {
                let span = self.advance().span;
                Ok(Expr::Literal(Literal::Bool(false), span))
            }
            TokenKind::None => {
                let span = self.advance().span;
                Ok(Expr::Literal(Literal::None, span))
            }
            TokenKind::Ident(name) => {
                if name == "정규식" && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::LBrace))
                {
                    let ident_token = self.advance();
                    let open = self.peek().clone();
                    if open.span.start_line != ident_token.span.end_line
                        || open.span.start_col != ident_token.span.end_col
                    {
                        return Err(ParseError::UnexpectedToken {
                            expected: "Gate0: 정규식{ 는 붙여쓰기만 허용됩니다",
                            found: open.kind,
                            span: open.span,
                        });
                    }
                    return self.parse_regex_literal_call(ident_token.span);
                }
                let path = self.parse_path()?;
                Ok(Expr::Path(path))
            }
            TokenKind::Atom(text) => {
                let span = self.advance().span;
                Ok(Expr::Atom { text, span })
            }
            TokenKind::Salim => {
                let path = self.parse_path()?;
                Ok(Expr::Path(path))
            }
            TokenKind::LParen => {
                let start_span = self.advance().span;
                let mut args: Vec<ArgBinding> = Vec::new();
                if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
                    loop {
                        let arg = self.parse_call_arg()?;
                        args.push(arg);
                        if self.peek_kind_is(|k| matches!(k, TokenKind::Comma)) {
                            self.advance();
                            continue;
                        }
                        break;
                    }
                }
                if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
                    return Err(ParseError::ExpectedRParen {
                        span: self.peek().span,
                    });
                }
                self.advance();
                if self.peek_kind_is(|k| matches!(k, TokenKind::Salim))
                    || self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name != "의"))
                {
                    let (name, name_span) = self.parse_call_name()?;
                    return Ok(Expr::Call {
                        name,
                        args,
                        span: start_span.merge(name_span),
                    });
                }
                if args.len() == 1 {
                    let only = args.into_iter().next().unwrap();
                    if only.josa.is_none() && only.resolved_pin.is_none() {
                        return Ok(only.expr);
                    }
                }
                Err(ParseError::UnexpectedToken {
                    expected: "call name",
                    found: self.peek().kind.clone(),
                    span: self.peek().span,
                })
            }
            TokenKind::LBrace => {
                let start_span = self.advance().span;
                if !self.peek_kind_is(|k| matches!(k, TokenKind::Ident(_))) {
                    return Err(ParseError::UnexpectedToken {
                        expected: "seed parameter",
                        found: self.peek().kind.clone(),
                        span: self.peek().span,
                    });
                }
                if !self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Pipe)) {
                    return Err(ParseError::UnexpectedToken {
                        expected: "'|'",
                        found: self.peek().kind.clone(),
                        span: self.peek().span,
                    });
                }
                let param_token = self.advance();
                let param = match param_token.kind {
                    TokenKind::Ident(name) => name,
                    other => {
                        return Err(ParseError::UnexpectedToken {
                            expected: "seed parameter",
                            found: other,
                            span: param_token.span,
                        })
                    }
                };
                self.advance();
                let body = self.parse_expr()?;
                if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
                    return Err(ParseError::ExpectedRBrace {
                        span: self.peek().span,
                    });
                }
                let end_span = self.advance().span;
                let span = start_span.merge(end_span);
                Ok(Expr::SeedLiteral {
                    param,
                    body: Box::new(body),
                    span,
                })
            }
            TokenKind::LBracket => {
                let start_span = self.advance().span;
                let mut args = Vec::new();
                if !self.peek_kind_is(|k| matches!(k, TokenKind::RBracket)) {
                    loop {
                        let expr = self.parse_expr()?;
                        args.push(expr);
                        if self.peek_kind_is(|k| matches!(k, TokenKind::Comma)) {
                            self.advance();
                            continue;
                        }
                        break;
                    }
                }
                if !self.peek_kind_is(|k| matches!(k, TokenKind::RBracket)) {
                    return Err(ParseError::UnexpectedToken {
                        expected: "']'",
                        found: self.peek().kind.clone(),
                        span: self.peek().span,
                    });
                }
                let end_span = self.advance().span;
                let span = start_span.merge(end_span);
                self.build_charim_literal(args, span)
            }
            _ => Err(ParseError::ExpectedExpr { span: token.span }),
        }
    }

    fn parse_regex_literal_call(&mut self, ident_span: Span) -> Result<Expr, ParseError> {
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();

        let pattern_token = self.peek().clone();
        let pattern = match pattern_token.kind {
            TokenKind::String(text) => {
                self.advance();
                text
            }
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "정규식 패턴 문자열",
                    found: other,
                    span: pattern_token.span,
                })
            }
        };
        let pattern_expr = Expr::Literal(Literal::Str(pattern), pattern_token.span);
        let mut args = vec![self.new_arg_binding(pattern_expr)];

        if self.peek_kind_is(|k| matches!(k, TokenKind::Comma)) {
            self.advance();
            let flags_token = self.peek().clone();
            let flags = match flags_token.kind {
                TokenKind::String(text) => {
                    self.advance();
                    text
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "정규식 깃발 문자열",
                        found: other,
                        span: flags_token.span,
                    })
                }
            };
            let flags_expr = Expr::Literal(Literal::Str(flags), flags_token.span);
            args.push(self.new_arg_binding(flags_expr));
        }

        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'}'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        let close_span = self.advance().span;
        let span = ident_span.merge(close_span);
        Ok(Expr::Call {
            name: "정규식".to_string(),
            args,
            span,
        })
    }

    fn new_arg_binding(&self, expr: Expr) -> ArgBinding {
        let span = expr.span();
        ArgBinding {
            expr,
            josa: None,
            resolved_pin: None,
            binding_reason: BindingReason::Positional,
            span,
        }
    }

    fn parse_arg_suffix(&mut self) -> Result<ArgSuffix, ParseError> {
        let mut fixed_pin = None;
        let mut josa = None;
        if self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            let pin_token = self.peek().clone();
            let pin = match pin_token.kind {
                TokenKind::Ident(name) => {
                    self.advance();
                    name
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "핀",
                        found: other,
                        span: pin_token.span,
                    })
                }
            };
            fixed_pin = Some(pin);
        }
        if let TokenKind::Josa(value) = self.peek().kind.clone() {
            self.advance();
            josa = Some(value);
        }
        let reason = if fixed_pin.is_some() {
            BindingReason::UserFixed
        } else if josa.is_some() {
            BindingReason::Dictionary
        } else {
            BindingReason::Positional
        };
        Ok(ArgSuffix {
            josa,
            fixed_pin,
            binding_reason: reason,
        })
    }

    fn parse_call_arg(&mut self) -> Result<ArgBinding, ParseError> {
        if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(_)))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Equal))
        {
            let pin_token = self.advance();
            let pin = match pin_token.kind {
                TokenKind::Ident(name) => name,
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "핀",
                        found: other,
                        span: pin_token.span,
                    })
                }
            };
            self.advance();
            let expr = self.parse_expr()?;
            let expr_span = expr.span();
            let suffix = self.parse_arg_suffix()?;
            return Ok(ArgBinding {
                expr,
                josa: suffix.josa,
                resolved_pin: Some(pin),
                binding_reason: BindingReason::UserFixed,
                span: expr_span,
            });
        }
        let expr = self.parse_expr()?;
        let expr_span = expr.span();
        let suffix = self.parse_arg_suffix()?;
        Ok(ArgBinding {
            expr,
            josa: suffix.josa,
            resolved_pin: suffix.fixed_pin,
            binding_reason: suffix.binding_reason,
            span: expr_span,
        })
    }

    fn is_binding_paren_start(&self) -> bool {
        self.peek_kind_is(|k| matches!(k, TokenKind::LParen))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Ident(_)))
            && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::Equal | TokenKind::Colon))
    }

    fn build_charim_literal(&self, args: Vec<Expr>, span: Span) -> Result<Expr, ParseError> {
        let has_inner = args
            .iter()
            .any(|arg| matches!(arg, Expr::Call { name, .. } if name == "차림"));
        if !has_inner {
            return Ok(Expr::Call {
                name: "차림".to_string(),
                args: args
                    .into_iter()
                    .map(|expr| self.new_arg_binding(expr))
                    .collect(),
                span,
            });
        }

        let mut rows: Vec<Vec<Expr>> = Vec::new();
        for arg in args {
            match arg {
                Expr::Call {
                    name,
                    args,
                    span: row_span,
                } if name == "차림" => {
                    if args
                        .iter()
                        .any(|item| matches!(&item.expr, Expr::Call { name, .. } if name == "차림"))
                    {
                        return Err(ParseError::InvalidTensor { span: row_span });
                    }
                    let mut row = Vec::new();
                    for item in args {
                        if item.josa.is_some() || item.resolved_pin.is_some() {
                            return Err(ParseError::InvalidTensor { span: row_span });
                        }
                        row.push(item.expr);
                    }
                    rows.push(row);
                }
                other => {
                    return Err(ParseError::InvalidTensor { span: other.span() });
                }
            }
        }

        let row_count = rows.len();
        let col_count = rows.first().map(|row| row.len()).unwrap_or(0);
        for row in &rows {
            if row.len() != col_count {
                return Err(ParseError::InvalidTensor { span });
            }
        }

        let mut flat = Vec::new();
        for row in rows {
            flat.extend(row);
        }

        let row_literal = Expr::Literal(
            Literal::Num(NumberLiteral {
                raw: Fixed64::from_int(row_count as i64).raw(),
                unit: None,
            }),
            span,
        );
        let col_literal = Expr::Literal(
            Literal::Num(NumberLiteral {
                raw: Fixed64::from_int(col_count as i64).raw(),
                unit: None,
            }),
            span,
        );
        let shape = Expr::Call {
            name: "차림".to_string(),
            args: vec![
                self.new_arg_binding(row_literal),
                self.new_arg_binding(col_literal),
            ],
            span,
        };
        let data = Expr::Call {
            name: "차림".to_string(),
            args: flat
                .into_iter()
                .map(|expr| self.new_arg_binding(expr))
                .collect(),
            span,
        };
        let layout = Expr::Literal(Literal::Str("가로먼저".to_string()), span);
        let bindings = vec![
            Binding {
                name: "형상".to_string(),
                value: shape,
                span,
            },
            Binding {
                name: "자료".to_string(),
                value: data,
                span,
            },
            Binding {
                name: "배치".to_string(),
                value: layout,
                span,
            },
        ];
        Ok(Expr::Pack { bindings, span })
    }

    fn parse_template_expr(&mut self) -> Result<Expr, ParseError> {
        let token = self.advance();
        match token.kind {
            TokenKind::Template(body) => Ok(Expr::Template {
                body,
                span: token.span,
            }),
            other => Err(ParseError::UnexpectedToken {
                expected: "template",
                found: other,
                span: token.span,
            }),
        }
    }

    fn parse_in_fill(
        &mut self,
        bindings: Vec<Binding>,
        start_span: Span,
    ) -> Result<Expr, ParseError> {
        self.advance();
        let target_expr =
            if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(_) | TokenKind::Salim)) {
                Expr::Path(self.parse_path()?)
            } else {
                return Err(ParseError::ExpectedPath {
                    span: self.peek().span,
                });
            };
        if self.peek_kind_is(|k| matches!(k, TokenKind::Chaewugi)) {
            let end_span = self.advance().span;
            let span = start_span.merge(end_span);
            return Ok(Expr::TemplateFill {
                template: Box::new(target_expr),
                bindings,
                span,
            });
        }
        if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "풀기")) {
            let end_span = self.advance().span;
            let span = start_span.merge(end_span);
            return Ok(Expr::FormulaFill {
                formula: Box::new(target_expr),
                bindings,
                span,
            });
        }
        if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "살피기")) {
            let end_span = self.advance().span;
            let span = start_span.merge(end_span);
            let pack = Expr::Pack {
                bindings,
                span: start_span,
            };
            return Ok(Expr::Call {
                name: "살피기".to_string(),
                args: vec![
                    self.new_arg_binding(target_expr),
                    self.new_arg_binding(pack),
                ],
                span,
            });
        }
        Err(ParseError::UnexpectedToken {
            expected: "'채우기' or '풀기' or '살피기'",
            found: self.peek().kind.clone(),
            span: self.peek().span,
        })
    }

    fn parse_template_fill_explicit(
        &mut self,
        bindings: Vec<Binding>,
        start_span: Span,
    ) -> Result<Expr, ParseError> {
        let mut template_binding: Option<Binding> = None;
        let mut rest = Vec::new();
        for binding in bindings {
            if binding.name == "무늬" {
                if template_binding.is_some() {
                    return Err(ParseError::UnexpectedToken {
                        expected: "single '무늬' binding",
                        found: TokenKind::Chaewugi,
                        span: binding.span,
                    });
                }
                template_binding = Some(binding);
            } else {
                rest.push(binding);
            }
        }
        let template_binding = template_binding.ok_or_else(|| ParseError::UnexpectedToken {
            expected: "무늬 binding",
            found: TokenKind::Chaewugi,
            span: self.peek().span,
        })?;
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        Ok(Expr::TemplateFill {
            template: Box::new(template_binding.value),
            bindings: rest,
            span,
        })
    }

    fn parse_call_name(&mut self) -> Result<(String, Span), ParseError> {
        let first = self.advance();
        let (first_name, mut span) = match first.kind {
            TokenKind::Ident(name) => (name, first.span),
            TokenKind::Salim => ("살림".to_string(), first.span),
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "call name",
                    found: other,
                    span: first.span,
                })
            }
        };
        let mut segments = vec![first_name];
        while self.peek_kind_is(|k| matches!(k, TokenKind::Dot))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Ident(_)))
        {
            self.advance();
            let token = self.advance();
            if let TokenKind::Ident(name) = token.kind {
                span = span.merge(token.span);
                segments.push(name);
            }
        }
        Ok((segments.join("."), span))
    }

    fn parse_bindings(&mut self) -> Result<(Vec<Binding>, Span), ParseError> {
        let start_span = self.advance().span;
        let mut bindings = Vec::new();
        loop {
            let name_token = self.advance();
            let name = match &name_token.kind {
                TokenKind::Ident(name) => name.clone(),
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "binding name",
                        found: name_token.kind.clone(),
                        span: name_token.span,
                    })
                }
            };
            if !self.peek_kind_is(|k| matches!(k, TokenKind::Equal | TokenKind::Colon)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "'=' 또는 ':'",
                    found: self.peek().kind.clone(),
                    span: self.peek().span,
                });
            }
            self.advance();
            let value = self.parse_expr()?;
            let span = name_token.span.merge(value.span());
            bindings.push(Binding { name, value, span });
            if self.peek_kind_is(|k| matches!(k, TokenKind::Comma)) {
                self.advance();
                continue;
            }
            break;
        }
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
            return Err(ParseError::ExpectedRParen {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        Ok((bindings, start_span.merge(end_span)))
    }

    fn parse_formula_expr(
        &mut self,
        bindings: Option<Vec<Binding>>,
        start_span: Span,
    ) -> Result<Expr, ParseError> {
        let dialect = self.parse_formula_head()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::Susic)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'수식'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(ParseError::UnexpectedToken {
                expected: "'{'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }
        self.advance();
        let body = self.collect_formula_body()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        match bindings {
            Some(bindings) => Ok(Expr::FormulaEval {
                dialect,
                body,
                bindings,
                span,
            }),
            None => Ok(Expr::Formula {
                dialect,
                body,
                span,
            }),
        }
    }

    fn parse_formula_head(&mut self) -> Result<FormulaDialect, ParseError> {
        self.advance();
        let name_token = if self.peek_kind_is(|k| matches!(k, TokenKind::Atom(_))) {
            self.advance()
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "atom",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        };
        let dialect = match name_token.kind {
            TokenKind::Atom(text) if text == "#ascii" => FormulaDialect::Ascii,
            TokenKind::Atom(text) if text == "#ascii1" => FormulaDialect::Ascii1,
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "'ascii' or 'ascii1'",
                    found: other,
                    span: name_token.span,
                })
            }
        };
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RParen)) {
            return Err(ParseError::ExpectedRParen {
                span: self.peek().span,
            });
        }
        self.advance();
        Ok(dialect)
    }

    fn parse_assertion_block(&self, raw: &str, span: Span) -> Result<Assertion, ParseError> {
        let body_source = raw.replace("\r\n", "\n").replace('\r', "\n");
        let wrapped = format!("검사:셈씨 = {{{}}}", body_source);
        let normalized = ddonirang_lang::parse_and_normalize(
            &wrapped,
            "#assertion",
            ddonirang_lang::NormalizationLevel::N1,
        )
        .map_err(|_| ParseError::UnexpectedToken {
            expected: "valid assertion body",
            found: TokenKind::Assertion(body_source.clone()),
            span,
        })?;
        let open = normalized.find('{').ok_or(ParseError::UnexpectedToken {
            expected: "assertion body start",
            found: TokenKind::Assertion(body_source.clone()),
            span,
        })?;
        let close = normalized.rfind('}').ok_or(ParseError::UnexpectedToken {
            expected: "assertion body end",
            found: TokenKind::Assertion(body_source.clone()),
            span,
        })?;
        Ok(Assertion {
            body_source,
            canon: format!("세움{}", &normalized[open..=close]),
        })
    }

    fn skip_tag_blocks(&mut self) {
        loop {
            if self.peek_kind_is(|k| matches!(k, TokenKind::LParen))
                && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Atom(_)))
                && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::RParen))
            {
                self.advance();
                self.advance();
                self.advance();
                self.skip_newlines();
                continue;
            }
            break;
        }
    }

    fn collect_formula_body(&mut self) -> Result<String, ParseError> {
        let mut out = String::new();
        loop {
            let token = self.peek().clone();
            if matches!(token.kind, TokenKind::RBrace) {
                break;
            }
            if matches!(token.kind, TokenKind::Eof) {
                return Err(ParseError::ExpectedRBrace { span: token.span });
            }
            self.advance();
            match token.kind {
                TokenKind::Newline => {}
                TokenKind::Ident(name) => out.push_str(&name),
                TokenKind::Number(raw) => out.push_str(&Fixed64::from_raw(raw).format()),
                TokenKind::Plus => out.push('+'),
                TokenKind::Minus => out.push('-'),
                TokenKind::Star => out.push('*'),
                TokenKind::Slash => out.push('/'),
                TokenKind::Percent => out.push('%'),
                TokenKind::Caret => out.push('^'),
                TokenKind::Equal => out.push('='),
                TokenKind::Comma => out.push(','),
                TokenKind::LParen => out.push('('),
                TokenKind::RParen => out.push(')'),
                TokenKind::Dot => out.push('.'),
                TokenKind::String(text) => {
                    out.push('"');
                    out.push_str(&text.replace('"', "\\\""));
                    out.push('"');
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "formula token",
                        found: other,
                        span: token.span,
                    })
                }
            }
        }
        Ok(out.trim().to_string())
    }

    fn parse_unit_postfix(&mut self, span: Span) -> Result<(Option<UnitExpr>, Span), ParseError> {
        if !self.peek_kind_is(|k| matches!(k, TokenKind::At)) {
            return Ok((None, span));
        }
        let mut merged = span;
        self.advance();
        merged = merged.merge(self.peek().span);
        let expr = self.parse_unit_expr()?;
        let span = merged.merge(self.last_span());
        Ok((Some(expr), span))
    }

    fn parse_unit_expr(&mut self) -> Result<UnitExpr, ParseError> {
        let mut factors = Vec::new();
        let mut sign = 1;

        loop {
            let token = self.peek().clone();
            let name = match token.kind {
                TokenKind::Ident(ref name) => name.clone(),
                _ => {
                    return Err(ParseError::ExpectedUnit { span: token.span });
                }
            };
            self.advance();
            let mut exp = 1;
            if self.peek_kind_is(|k| matches!(k, TokenKind::Caret)) {
                self.advance();
                let exp_token = self.peek().clone();
                match exp_token.kind {
                    TokenKind::Number(value) => {
                        if value & 0xFFFF_FFFF != 0 {
                            return Err(ParseError::ExpectedUnit {
                                span: exp_token.span,
                            });
                        }
                        let int_value = value >> 32;
                        if int_value < i32::MIN as i64 || int_value > i32::MAX as i64 {
                            return Err(ParseError::ExpectedUnit {
                                span: exp_token.span,
                            });
                        }
                        exp = int_value as i32;
                        self.advance();
                    }
                    _ => {
                        return Err(ParseError::ExpectedUnit {
                            span: exp_token.span,
                        });
                    }
                }
            }
            factors.push(UnitFactor {
                name,
                exp: exp * sign,
            });

            if self.peek_kind_is(|k| matches!(k, TokenKind::Star)) {
                sign = 1;
                self.advance();
                continue;
            }
            if self.peek_kind_is(|k| matches!(k, TokenKind::Slash)) {
                sign = -1;
                self.advance();
                continue;
            }
            break;
        }

        Ok(UnitExpr { factors })
    }

    fn last_span(&self) -> Span {
        if self.pos == 0 {
            self.peek().span
        } else {
            self.tokens[self.pos - 1].span
        }
    }

    fn parse_path(&mut self) -> Result<Path, ParseError> {
        let first_token = self.peek().clone();
        let first_segment = match &first_token.kind {
            TokenKind::Salim => {
                self.advance();
                "살림".to_string()
            }
            TokenKind::Ident(name) => {
                self.advance();
                name.clone()
            }
            _ => {
                return Err(ParseError::ExpectedPath {
                    span: first_token.span,
                })
            }
        };

        let mut segments = vec![first_segment];
        let mut span = first_token.span;
        let mut implicit_root = false;

        loop {
            let Some((segment, segment_span)) = self.try_parse_dot_segment()? else {
                break;
            };
            segments.push(segment);
            span = span.merge(segment_span);
        }

        if !matches!(
            segments.first().map(|seg| seg.as_str()),
            Some("살림" | "바탕" | "샘" | "제")
        ) {
            segments.insert(0, self.default_root.clone());
            implicit_root = true;
        }

        Ok(Path {
            segments,
            span,
            implicit_root,
        })
    }

    // Step B/C scope: one-level + nested map field write.
    // - allowed: 살림.공.x, 바탕.공.x, 살림.공.속도.x
    fn split_map_dot_target(path: &Path) -> Option<(Path, Vec<String>)> {
        if path.segments.len() < 3 {
            return None;
        }
        if !matches!(
            path.segments.first().map(|seg| seg.as_str()),
            Some("살림" | "바탕" | "샘")
        ) {
            return None;
        }
        let target = Path {
            segments: vec![path.segments[0].clone(), path.segments[1].clone()],
            span: path.span,
            implicit_root: path.implicit_root,
        };
        Some((target, path.segments[2..].to_vec()))
    }

    fn build_map_dot_write_expr(
        &self,
        map_expr: Expr,
        key_segments: &[String],
        value: Expr,
        span: Span,
    ) -> Expr {
        debug_assert!(!key_segments.is_empty());
        let key_expr = Expr::Literal(Literal::Str(key_segments[0].clone()), span);
        if key_segments.len() == 1 {
            return Expr::Call {
                name: "짝맞춤.바꾼값".to_string(),
                args: vec![
                    self.new_arg_binding(map_expr),
                    self.new_arg_binding(key_expr),
                    self.new_arg_binding(value),
                ],
                span,
            };
        }
        let child_map_expr = Expr::Call {
            name: "짝맞춤.필수값".to_string(),
            args: vec![
                self.new_arg_binding(map_expr.clone()),
                self.new_arg_binding(key_expr.clone()),
            ],
            span,
        };
        let child_value =
            self.build_map_dot_write_expr(child_map_expr, &key_segments[1..], value, span);
        Expr::Call {
            name: "짝맞춤.바꾼값".to_string(),
            args: vec![
                self.new_arg_binding(map_expr),
                self.new_arg_binding(key_expr),
                self.new_arg_binding(child_value),
            ],
            span,
        }
    }

    fn try_parse_dot_segment(&mut self) -> Result<Option<(String, Span)>, ParseError> {
        if !self.peek_kind_is(|k| matches!(k, TokenKind::Dot)) {
            return Ok(None);
        }
        if self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Ident(_))) {
            self.advance();
            let token = self.advance();
            if let TokenKind::Ident(name) = token.kind {
                return Ok(Some((name, token.span)));
            }
        }
        if self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Number(_))) {
            self.advance();
            let token = self.advance();
            if let TokenKind::Number(raw) = token.kind {
                let segment = Self::parse_nonnegative_index_segment(raw, token.span)?;
                return Ok(Some((segment, token.span)));
            }
        }
        Ok(None)
    }

    fn parse_nonnegative_index_segment(raw: i64, span: Span) -> Result<String, ParseError> {
        if raw < 0 || (raw & ((1_i64 << Fixed64::SCALE_BITS) - 1)) != 0 {
            return Err(ParseError::UnexpectedToken {
                expected: "integer dot index",
                found: TokenKind::Number(raw),
                span,
            });
        }
        Ok((raw >> Fixed64::SCALE_BITS).to_string())
    }

    fn consume_terminator(&mut self) -> Result<(), ParseError> {
        if self.peek_kind_is(|k| matches!(k, TokenKind::Dot)) {
            self.advance();
            self.skip_newlines();
            return Ok(());
        }
        if self.peek_kind_is(|k| matches!(k, TokenKind::Newline)) {
            self.skip_newlines();
            return Ok(());
        }
        if self.peek_kind_is(|k| matches!(k, TokenKind::Eof)) {
            return Ok(());
        }
        Err(ParseError::UnexpectedToken {
            expected: "'.' or newline",
            found: self.peek().kind.clone(),
            span: self.peek().span,
        })
    }

    fn skip_newlines(&mut self) {
        while self.peek_kind_is(|k| matches!(k, TokenKind::Newline)) {
            self.advance();
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_kind_is(&self, f: impl FnOnce(&TokenKind) -> bool) -> bool {
        f(&self.peek().kind)
    }

    fn peek_kind_n_is(&self, n: usize, f: impl FnOnce(&TokenKind) -> bool) -> bool {
        self.tokens
            .get(self.pos + n)
            .map(|token| f(&token.kind))
            .unwrap_or(false)
    }

    fn peek_next_non_newline_is(&self, f: impl FnOnce(&TokenKind) -> bool) -> bool {
        let mut idx = self.pos + 1;
        while let Some(token) = self.tokens.get(idx) {
            if !matches!(token.kind, TokenKind::Newline) {
                return f(&token.kind);
            }
            idx += 1;
        }
        false
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::lexer::Lexer;

    #[test]
    fn parse_pragma_graph_stmt_rejected() {
        let source = "#그래프(y축=x)\n살림.x <- 1.\n";
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("pragma rejected");
        assert_eq!(err.code(), "E_PARSE_UNEXPECTED_TOKEN");
    }

    #[test]
    fn parse_pragma_import_like_stmt_rejected() {
        let source = "#가져오기 누리/물리/역학 (중력씨, 이동씨)\n";
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("pragma rejected");
        assert_eq!(err.code(), "E_PARSE_UNEXPECTED_TOKEN");
    }

    #[test]
    fn parse_pragma_with_parenthesized_spaces_rejected() {
        let source = "#진단(a ≈ b, 허용오차=0.1, 이름=\"검사\")\n";
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("pragma rejected");
        assert_eq!(err.code(), "E_PARSE_UNEXPECTED_TOKEN");
    }

    #[test]
    fn parse_setting_pragma_rejected() {
        let source = "#설정: { 열림: 허용 }\n살림.x <- 1.\n";
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err =
            Parser::parse_with_default_root(tokens, "살림").expect_err("setting pragma rejected");
        assert_eq!(err.code(), "E_PARSE_UNEXPECTED_TOKEN");
    }

    #[test]
    fn parse_if_with_korean_keywords() {
        let source = r#"
(1 < 2) 일때 {
    살림.x <- 1.
} 아니면 {
    살림.x <- 2.
}
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        assert!(program
            .stmts
            .iter()
            .any(|stmt| matches!(stmt, Stmt::If { .. })));
    }

    #[test]
    fn parse_choose_legacy_else_still_supported() {
        let source = r#"
고르기:
{ 1 == 2 }: {
  살림.x <- 1.
}
아니면: {
  살림.x <- 2.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::Choose {
            branches,
            else_body,
            exhaustive,
            ..
        }) = program.stmts.first()
        else {
            panic!("choose expected");
        };
        assert_eq!(branches.len(), 1);
        assert!(else_body.is_some());
        assert!(!exhaustive);
    }

    #[test]
    fn parse_choose_case_complete_marker() {
        let source = r#"
고르기:
{ 1 == 1 } 인 경우 {
  살림.x <- 1.
}
모든 경우 다룸.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::Choose {
            branches,
            else_body,
            exhaustive,
            ..
        }) = program.stmts.first()
        else {
            panic!("choose expected");
        };
        assert_eq!(branches.len(), 1);
        assert!(else_body.is_none());
        assert!(*exhaustive);
    }

    #[test]
    fn parse_choose_case_requires_completion() {
        let source = r#"
고르기:
{ 1 == 1 } 인 경우 {
  살림.x <- 1.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_CASE_COMPLETION_REQUIRED");
    }

    #[test]
    fn parse_choose_case_else_must_be_last() {
        let source = r#"
고르기:
{ 1 == 2 } 인 경우 {
  살림.x <- 1.
}
그밖의 경우 {
  살림.x <- 2.
}
{ 1 == 1 } 인 경우 {
  살림.x <- 3.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_CASE_ELSE_NOT_LAST");
    }

    #[test]
    fn parse_dialect_header_rejected() {
        let source = r#"#말씨: xx
(1 < 2) if {
    살림.x <- 1.
}
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("parse should fail");
        assert_eq!(err.code(), "E_PARSE_UNEXPECTED_TOKEN");
    }

    #[test]
    fn parse_decl_block_chaebi_item_kinds() {
        let source = r#"
채비 {
  점수:수 <- 0.
  파이:수 = 3.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Stmt::SeedDef { body, .. } = &program.stmts[0] else {
            panic!("seed def expected");
        };
        let Stmt::DeclBlock { items, .. } = &body[0] else {
            panic!("decl block expected");
        };
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0].kind, DeclKind::Gureut));
        assert!(matches!(items[1].kind, DeclKind::Butbak));
    }

    #[test]
    fn parse_decl_block_multi_arg_call_initializer_without_maegim() {
        let source = r#"
채비 {
  값:나눔수 <- (6, 9) 나눔수.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Stmt::SeedDef { body, .. } = &program.stmts[0] else {
            panic!("seed def expected");
        };
        let Stmt::DeclBlock { items, .. } = &body[0] else {
            panic!("decl block expected");
        };
        let value = items[0].value.as_ref().expect("decl value");
        let Expr::Call { name, args, .. } = value else {
            panic!("call expression expected");
        };
        assert_eq!(name, "나눔수");
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn parse_decl_block_maegim_alias_and_fields() {
        let source = r#"
채비 {
  g:수 = (9.8) 조건 {
    범위: 1..20.
    간격: 0.1.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Stmt::SeedDef { body, .. } = &program.stmts[0] else {
            panic!("seed def expected");
        };
        let Stmt::DeclBlock { items, .. } = &body[0] else {
            panic!("decl block expected");
        };
        let item = &items[0];
        assert!(matches!(item.kind, DeclKind::Butbak));
        let maegim = item.maegim.as_ref().expect("maegim");
        assert_eq!(maegim.fields.len(), 2);
        assert_eq!(maegim.fields[0].name, "범위");
        assert_eq!(maegim.fields[1].name, "간격");
    }

    #[test]
    fn parse_decl_block_maegim_supports_split_count() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    범위: 1..20.
    분할수: 40.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Stmt::SeedDef { body, .. } = &program.stmts[0] else {
            panic!("seed def expected");
        };
        let Stmt::DeclBlock { items, .. } = &body[0] else {
            panic!("decl block expected");
        };
        let item = &items[0];
        let maegim = item.maegim.as_ref().expect("maegim");
        assert_eq!(maegim.fields.len(), 2);
        assert_eq!(maegim.fields[0].name, "범위");
        assert_eq!(maegim.fields[1].name, "분할수");
    }

    #[test]
    fn parse_decl_block_maegim_supports_ganeum_and_gallae_blocks() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    가늠 {
      범위: 1..20.
      간격: 0.1.
    }.
    갈래 {
      가만히: 참.
      벗남다룸: "잘림".
    }.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Stmt::SeedDef { body, .. } = &program.stmts[0] else {
            panic!("seed def expected");
        };
        let Stmt::DeclBlock { items, .. } = &body[0] else {
            panic!("decl block expected");
        };
        let maegim = items[0].maegim.as_ref().expect("maegim");
        let names = maegim
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec!["가늠.범위", "가늠.간격", "갈래.가만히", "갈래.벗남다룸"]
        );
    }

    #[test]
    fn parse_decl_block_maegim_nested_rejects_step_and_split_count_together() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    가늠 {
      간격: 0.1.
      분할수: 40.
    }.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_MAEGIM_STEP_SPLIT_CONFLICT");
        assert!(matches!(err, ParseError::MaegimStepSplitConflict { .. }));
    }

    #[test]
    fn parse_decl_block_maegim_nested_rejects_unknown_section() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    실험 {
      범위: 1..20.
    }.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_MAEGIM_NESTED_SECTION_UNSUPPORTED");
        assert!(matches!(
            err,
            ParseError::MaegimNestedSectionUnsupported { .. }
        ));
    }

    #[test]
    fn parse_decl_block_maegim_nested_rejects_unknown_field_in_ganeum() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    가늠 {
      최소값: 1.
    }.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_MAEGIM_NESTED_FIELD_UNSUPPORTED");
        assert!(matches!(
            err,
            ParseError::MaegimNestedFieldUnsupported { .. }
        ));
    }

    #[test]
    fn parse_decl_block_maegim_rejects_step_and_split_count_together() {
        let source = r#"
채비 {
  g:수 = (9.8) 매김 {
    범위: 1..20.
    간격: 0.1.
    분할수: 40.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_MAEGIM_STEP_SPLIT_CONFLICT");
        assert!(matches!(err, ParseError::MaegimStepSplitConflict { .. }));
    }

    #[test]
    fn parse_decl_block_maegim_requires_grouped_value() {
        let source = r#"
채비 {
  g:수 = 9.8 조건 {
    범위: 1..20.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("grouped value");
        assert_eq!(err.code(), "E_PARSE_MAEGIM_GROUPED_VALUE_REQUIRED");
    }

    #[test]
    fn parse_decl_block_legacy_header_rejected() {
        let source = r#"
그릇채비: {
  점수:수 <- 0.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("legacy header");
        assert_eq!(err.code(), "E_BLOCK_HEADER_COLON_FORBIDDEN");
    }

    #[test]
    fn parse_decl_block_colon_forbidden() {
        let source = r#"
채비: {
  점수:수 <- 0.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("colon compatibility");
        assert!(program
            .stmts
            .iter()
            .any(|stmt| matches!(stmt, Stmt::SeedDef { .. })));
    }

    #[test]
    fn parse_import_block_basic() {
        let source = r#"
쓰임 {
  수학: "표준/수학@1.2".
  진자: "./physics/pendulum".
}.
움직:움직씨 = { ("ok") 보여주기. }.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        assert!(matches!(
            program.stmts.first(),
            Some(Stmt::ImportBlock { .. })
        ));
        assert!(program
            .stmts
            .iter()
            .any(|stmt| matches!(stmt, Stmt::SeedDef { .. })));
    }

    #[test]
    fn parse_import_alias_duplicate_error() {
        let source = r#"
쓰임 {
  수학: "표준/수학@1.2".
  수학: "표준/수학@1.2".
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("duplicate alias");
        assert_eq!(err.code(), "E_IMPORT_ALIAS_DUPLICATE");
    }

    #[test]
    fn parse_import_alias_reserved_error() {
        let source = r#"
쓰임 {
  살림: "표준/수학".
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("reserved alias");
        assert_eq!(err.code(), "E_IMPORT_ALIAS_RESERVED");
    }

    #[test]
    fn parse_import_path_invalid_error() {
        let source = r#"
쓰임 {
  수학: "https://example.com/math".
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("invalid path");
        assert_eq!(err.code(), "E_IMPORT_PATH_INVALID");
    }

    #[test]
    fn parse_import_version_conflict_error() {
        let source = r#"
쓰임 {
  수학: "표준/수학@1.2".
  수학2: "표준/수학@2.0".
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("version conflict");
        assert_eq!(err.code(), "E_IMPORT_VERSION_CONFLICT");
    }

    #[test]
    fn parse_export_block_public_alias_is_supported() {
        let source = r#"
공개 {
  셈: 계산.
  기본값.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        assert!(matches!(
            program.stmts.first(),
            Some(Stmt::ExportBlock { .. })
        ));
    }

    #[test]
    fn parse_export_block_duplicate_rejected() {
        let source = r#"
드러냄 { 계산. }.
공개 { 셈: 계산. }.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("duplicate export");
        assert_eq!(err.code(), "E_EXPORT_BLOCK_DUPLICATE");
    }

    #[test]
    fn parse_hook_every_colon_forbidden() {
        let source = r#"
(매마디)마다: {
  살림.t <- 살림.t + 1.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_BLOCK_HEADER_COLON_FORBIDDEN");
    }

    #[test]
    fn parse_hook_every_n_madi_supported() {
        let source = r#"
(3마디)마다 {
  없음.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::Hook { kind, .. }) = program.stmts.first() else {
            panic!("hook expected");
        };
        assert!(matches!(kind, HookKind::EveryNMadi(3)));
    }

    #[test]
    fn parse_hook_every_n_madi_rejects_zero_interval() {
        let source = r#"
(0마디)마다 {
  없음.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_HOOK_EVERY_N_MADI_INTERVAL_INVALID");
    }

    #[test]
    fn parse_hook_every_n_madi_rejects_unsupported_unit() {
        let source = r#"
(3 foo)마다 {
  없음.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_HOOK_EVERY_N_MADI_UNIT_UNSUPPORTED");
    }

    #[test]
    fn parse_hook_every_n_madi_rejects_unsupported_suffix() {
        let source = r#"
(3마디)할때 {
  없음.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_HOOK_EVERY_N_MADI_SUFFIX_UNSUPPORTED");
    }

    #[test]
    fn parse_hook_condition_becomes_supported() {
        let source = r#"
(살림.점수 > 10)이 될때 {
  없음.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::HookWhenBecomes { .. }) = program.stmts.first() else {
            panic!("condition edge hook expected");
        };
    }

    #[test]
    fn parse_hook_condition_while_supported() {
        let source = r#"
(살림.전투중)인 동안 {
  없음.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::HookWhile { .. }) = program.stmts.first() else {
            panic!("condition while hook expected");
        };
    }

    #[test]
    fn parse_hook_end_supported() {
        let source = r#"
(끝)할때 {
  없음.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::Hook { kind, .. }) = program.stmts.first() else {
            panic!("end hook expected");
        };
        assert!(matches!(kind, HookKind::End));
    }

    #[test]
    fn parse_hook_start_alias_choeum_supported() {
        let source = r#"
(처음)할때 {
  없음.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::Hook { kind, .. }) = program.stmts.first() else {
            panic!("start hook expected");
        };
        assert!(matches!(kind, HookKind::Start));
    }

    #[test]
    fn parse_bare_reset_kinds_as_zero_arg_calls() {
        let source = r#"
마당다시.
판다시.
누리다시.
보개다시.
모두다시.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let names: Vec<&str> = program
            .stmts
            .iter()
            .map(|stmt| match stmt {
                Stmt::Expr {
                    value: Expr::Call { name, args, .. },
                    ..
                } => {
                    assert!(args.is_empty());
                    name.as_str()
                }
                _ => panic!("reset call stmt expected"),
            })
            .collect();
        assert_eq!(names, vec!["마당다시", "판다시", "누리다시", "보개다시", "모두다시"]);
    }

    #[test]
    fn parse_lifecycle_transition_verbs_as_postfix_calls() {
        let source = r#"
진자마당 시작하기.
다음마당 넘어가기.
연습판 불러오기.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let mut names: Vec<&str> = Vec::new();
        let mut targets: Vec<&str> = Vec::new();
        for stmt in &program.stmts {
            let Stmt::Expr {
                value: Expr::Call { name, args, .. },
                ..
            } = stmt
            else {
                panic!("lifecycle transition call stmt expected");
            };
            names.push(name.as_str());
            assert_eq!(args.len(), 1);
            let Expr::Literal(Literal::Str(target), _) = &args[0].expr else {
                panic!("lifecycle transition target string expected");
            };
            targets.push(target.as_str());
        }
        assert_eq!(names, vec!["시작하기", "넘어가기", "불러오기"]);
        assert_eq!(targets, vec!["진자마당", "다음마당", "연습판"]);
    }

    #[test]
    fn parse_bare_lifecycle_transition_verbs_as_zero_arg_calls() {
        let source = r#"
시작하기.
넘어가기.
불러오기.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let names: Vec<&str> = program
            .stmts
            .iter()
            .map(|stmt| match stmt {
                Stmt::Expr {
                    value: Expr::Call { name, args, .. },
                    ..
                } => {
                    assert!(args.is_empty());
                    name.as_str()
                }
                _ => panic!("lifecycle transition call stmt expected"),
            })
            .collect();
        assert_eq!(names, vec!["시작하기", "넘어가기", "불러오기"]);
    }

    #[test]
    fn parse_lifecycle_pan_block_supported() {
        let source = r#"
판 {
  살림.x <- 1.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::LifecycleBlock { kind, body, .. }) = program.stmts.first() else {
            panic!("lifecycle block expected");
        };
        assert!(matches!(kind, LifecycleUnitKind::Pan));
        assert_eq!(body.len(), 1);
    }

    #[test]
    fn parse_lifecycle_madang_block_supported() {
        let source = r#"
마당 {
  살림.x <- 2.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::LifecycleBlock { kind, body, .. }) = program.stmts.first() else {
            panic!("lifecycle block expected");
        };
        assert!(matches!(kind, LifecycleUnitKind::Madang));
        assert_eq!(body.len(), 1);
    }

    #[test]
    fn parse_named_lifecycle_blocks_with_equal_surface() {
        let source = r#"
연습판 = 판 {
  살림.x <- 1.
}.
진자마당 = 마당 {
  살림.x <- 2.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::LifecycleBlock { name, kind, .. }) = program.stmts.first() else {
            panic!("first lifecycle block expected");
        };
        assert_eq!(name.as_deref(), Some("연습판"));
        assert!(matches!(kind, LifecycleUnitKind::Pan));
        let Some(Stmt::LifecycleBlock { name, kind, .. }) = program.stmts.get(1) else {
            panic!("second lifecycle block expected");
        };
        assert_eq!(name.as_deref(), Some("진자마당"));
        assert!(matches!(kind, LifecycleUnitKind::Madang));
    }

    #[test]
    fn parse_named_lifecycle_blocks_reject_duplicate_name() {
        let source = r#"
연습판 = 판 {
  살림.x <- 1.
}.
연습판 = 마당 {
  살림.x <- 2.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_LIFECYCLE_NAME_DUPLICATE");
        let ParseError::LifecycleNameDuplicate {
            span, first_span, ..
        } = err
        else {
            panic!("duplicate parse error expected");
        };
        assert_eq!((first_span.start_line, first_span.start_col), (2, 1));
        assert_eq!((span.start_line, span.start_col), (5, 1));
    }

    #[test]
    fn parse_foreach_colon_forbidden() {
        let source = r#"
(x) x목록에 대해: {
  살림.y <- x.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("colon compatibility");
        let Some(Stmt::ForEach { .. }) = program.stmts.first() else {
            panic!("foreach expected");
        };
    }

    #[test]
    fn parse_quantifier_statements() {
        let source = r#"
증명:셈씨 = {
  n 이 자연수 낱낱에 대해 {
    없음.
  }.
  x 가 실수 중 하나가 {
    없음.
  }.
  y 가 정수 중 딱 하나가 {
    없음.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::SeedDef { body, .. }) = program.stmts.first() else {
            panic!("seed def expected");
        };
        assert!(matches!(
            body.first(),
            Some(Stmt::Quantifier {
                kind: QuantifierKind::ForAll,
                ..
            })
        ));
        assert!(matches!(
            body.get(1),
            Some(Stmt::Quantifier {
                kind: QuantifierKind::Exists,
                ..
            })
        ));
        assert!(matches!(
            body.get(2),
            Some(Stmt::Quantifier {
                kind: QuantifierKind::ExistsUnique,
                ..
            })
        ));
    }

    #[test]
    fn parse_quantifier_rejects_mutation_in_body() {
        let source = r#"
증명:셈씨 = {
  n 이 자연수 낱낱에 대해 {
    살림.x <- 1.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_QUANTIFIER_MUTATION_FORBIDDEN");
    }

    #[test]
    fn parse_quantifier_rejects_show_in_body() {
        let source = r#"
증명:셈씨 = {
  x 가 실수 중 하나가 {
    x 보여주기.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_QUANTIFIER_SHOW_FORBIDDEN");
    }

    #[test]
    fn parse_quantifier_rejects_open_call_in_body() {
        let source = r#"
증명:셈씨 = {
  x 가 실수 중 하나가 {
    너머 {
      없음.
    }.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_QUANTIFIER_IO_FORBIDDEN");
    }

    #[test]
    fn parse_immediate_proof_lowers_to_seed_and_call() {
        let source = r#"
자연수_제곱_0이상 밝히기 {
  n 이 자연수 낱낱에 대해 {
    없음.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        assert!(matches!(
            program.stmts.first(),
            Some(Stmt::SeedDef {
                kind: SeedKind::Balhigi,
                ..
            })
        ));
        assert!(matches!(
            program.stmts.get(1),
            Some(Stmt::Expr {
                value: Expr::Call { name, .. },
                ..
            }) if name == "자연수_제곱_0이상"
        ));
    }

    #[test]
    fn parse_immediate_proof_rejects_mutation_in_body() {
        let source = r#"
검사 밝히기 {
  살림.x <- 1.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_IMMEDIATE_PROOF_MUTATION_FORBIDDEN");
    }

    #[test]
    fn parse_immediate_proof_allows_solver_hooks_but_rejects_general_open() {
        let ok_source = r#"
검사 밝히기 {
  ((질의="forall n. n = n")) 열림.풀이.확인.
  ((질의="forall n. n >= 0")) 반례찾기.
}.
"#;
        let tokens = Lexer::tokenize(ok_source).expect("tokenize");
        Parser::parse_with_default_root(tokens, "살림").expect("parse");

        let bad_source = r#"
검사 밝히기 {
  너머 {
    없음.
  }.
}.
"#;
        let tokens = Lexer::tokenize(bad_source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_IMMEDIATE_PROOF_IO_FORBIDDEN");
    }

    #[test]
    fn parse_assertion_literal_and_check_lower_to_call() {
        let source = r#"
거리_0이상 <- 세움{
  { 거리 >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
판정 <- (거리=3)인 거리_0이상 살피기.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::Assign {
            value: Expr::Assertion { assertion, .. },
            ..
        }) = program.stmts.first()
        else {
            panic!("assertion assignment expected");
        };
        assert!(assertion.canon.starts_with("세움{"));
        assert!(assertion.canon.contains("거리 >= 0"));

        let Some(Stmt::Assign {
            value: Expr::Call { name, args, .. },
            ..
        }) = program.stmts.get(1)
        else {
            panic!("check call assignment expected");
        };
        assert_eq!(name, "살피기");
        assert_eq!(args.len(), 2);
        assert!(matches!(
            &args[0].expr,
            Expr::Path(_) | Expr::FieldAccess { .. } | Expr::Assertion { .. }
        ));
        assert!(
            matches!(&args[1].expr, Expr::Pack { bindings, .. } if bindings.len() == 1 && bindings[0].name == "거리")
        );
    }

    #[test]
    fn parse_accepts_unary_plus_literal_assignment() {
        let source = "x <- +5.\ny <- (+5).\n";
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        assert!(matches!(program.stmts.first(), Some(Stmt::Assign { .. })));
        assert!(matches!(program.stmts.get(1), Some(Stmt::Assign { .. })));
    }

    #[test]
    fn parse_moyang_shape_block_lowers_to_show_stmts() {
        let source = r##"
움직:움직씨 = {
  bob_x <- 1.
  bob_y <- -1.
  모양: {
    선(0, 0, bob_x, bob_y, 색="#9ca3af", 굵기=0.02).
    원(bob_x, bob_y, r=0.08, 색="#38bdf8", 선색="#0ea5e9", 굵기=0.02).
    점(0, 0, 크기=0.045, 색="#f59e0b").
  }.
}.
"##;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::SeedDef { body, .. }) = program.stmts.first() else {
            panic!("seed def expected");
        };
        let mut show_strings = Vec::new();
        for stmt in body {
            if let Stmt::Show {
                value: Expr::Literal(Literal::Str(text), _),
                ..
            } = stmt
            {
                show_strings.push(text.clone());
            }
        }
        assert!(show_strings.contains(&"space2d".to_string()));
        assert!(show_strings.contains(&"space2d.shape".to_string()));
        assert!(show_strings.contains(&"line".to_string()));
        assert!(show_strings.contains(&"circle".to_string()));
        assert!(show_strings.contains(&"point".to_string()));
    }

    #[test]
    fn parse_jjaim_block_is_accepted_as_stub() {
        let source = r#"
움직:움직씨 = {
  짜임 {
    상태 { theta <- 0.8. }.
    출력 { 끝점 <- (0.0, 0.0). }.
  }.
  살림.t <- 1.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::SeedDef { body, .. }) = program.stmts.first() else {
            panic!("seed def expected");
        };
        assert!(body.iter().any(|stmt| matches!(stmt, Stmt::Expr { .. })));
        assert!(body.iter().any(|stmt| matches!(stmt, Stmt::Assign { .. })));
    }

    #[test]
    fn parse_beat_block_with_deferred_assignments() {
        let source = r#"
덩이 {
  살림.x <- 1 미루기.
  살림.y <- 2.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::BeatBlock { body, .. }) = program.stmts.first() else {
            panic!("beat block expected");
        };
        let Some(Stmt::Assign { deferred, .. }) = body.first() else {
            panic!("assign expected");
        };
        assert!(*deferred);
        let Some(Stmt::Assign { deferred, .. }) = body.get(1) else {
            panic!("assign expected");
        };
        assert!(!deferred);
    }

    #[test]
    fn parse_legacy_beat_keyword_is_rejected() {
        let source = r#"
박자 {
  살림.x <- 1 미루기.
  살림.y <- 2.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_UNEXPECTED_TOKEN");
    }

    #[test]
    fn parse_bundle_alias_block_is_rejected() {
        let source = r#"
묶음 {
  살림.x <- 1 미루기.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_UNEXPECTED_TOKEN");
    }

    #[test]
    fn parse_deferred_assignment_outside_beat_is_rejected() {
        let source = "살림.x <- 1 미루기.\n";
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_DEFERRED_ASSIGN_OUTSIDE_BEAT");
        assert!(matches!(err, ParseError::DeferredAssignOutsideBeat { .. }));
    }

    #[test]
    fn parse_strict_mode_rejects_compat_matic_entry() {
        let source = r#"
매틱:움직씨 = {
  살림.t <- 1.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root_mode(tokens, "살림", ParseMode::Strict)
            .expect_err("strict");
        assert_eq!(err.code(), "E_LANG_COMPAT_MATIC_ENTRY_DISABLED");
    }

    #[test]
    fn parse_strict_mode_accepts_compat_matic_entry_with_flag() {
        let source = r#"
매틱:움직씨 = {
  살림.t <- 1.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root_mode(
            tokens,
            "살림",
            ParseMode::Strict.with_compat_matic_entry(true),
        )
        .expect("compat-matic-entry");
        assert!(program
            .stmts
            .iter()
            .any(|stmt| matches!(stmt, Stmt::SeedDef { .. })));
    }

    #[test]
    fn parse_seed_tilde_alias_expands_additional_seed_defs() {
        let source = r#"
막~막으~막아:움직씨 = {
  살림.x <- 1.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let names: Vec<String> = program
            .stmts
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::SeedDef { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(names, vec!["막", "막으", "막아"]);
    }

    #[test]
    fn parse_regex_literal_lowers_to_call() {
        let source = r#"
패턴 <- 정규식{"[0-9]+", "i"}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::Assign { value, .. }) = program.stmts.first() else {
            panic!("assign expected");
        };
        let Expr::Call { name, args, .. } = value else {
            panic!("regex literal must lower to call");
        };
        assert_eq!(name, "정규식");
        assert_eq!(args.len(), 2);
        match &args[0].expr {
            Expr::Literal(Literal::Str(text), _) => assert_eq!(text, "[0-9]+"),
            other => panic!("pattern must be string literal: {:?}", other),
        }
        match &args[1].expr {
            Expr::Literal(Literal::Str(text), _) => assert_eq!(text, "i"),
            other => panic!("flags must be string literal: {:?}", other),
        }
    }

    #[test]
    fn parse_numeric_dot_segment_as_field_access() {
        let source = r#"
값목록 <- [1, 2, 3].
둘째 <- 값목록.1.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::Assign { value, .. }) = program.stmts.get(1) else {
            panic!("second stmt must be assign");
        };
        match value {
            Expr::FieldAccess { field, .. } => assert_eq!(field, "1"),
            Expr::Path(path) => assert_eq!(
                path.segments,
                vec!["살림".to_string(), "값목록".to_string(), "1".to_string()]
            ),
            _ => panic!("numeric dot segment must lower to path/field access"),
        }
    }

    #[test]
    fn parse_numeric_dot_segment_rejects_fraction() {
        let source = r#"
값목록 <- [1, 2, 3].
셋째 <- 값목록.1.5.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("fraction rejected");
        assert_eq!(err.code(), "E_PARSE_UNEXPECTED_TOKEN");
    }

    #[test]
    fn parse_path_allows_numeric_dot_segment() {
        let source = r#"
살림.값목록.1 보여주기.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::Show { value, .. }) = program.stmts.first() else {
            panic!("first stmt must be show");
        };
        let Expr::Path(path) = value else {
            panic!("show value must be path");
        };
        assert_eq!(
            path.segments,
            vec!["살림".to_string(), "값목록".to_string(), "1".to_string()]
        );
    }

    #[test]
    fn parse_map_dot_nested_write_lowers_to_nested_set_calls() {
        let source = r#"
살림.공.속도.x <- 1.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::Assign { target, value, .. }) = program.stmts.first() else {
            panic!("first stmt must be assign");
        };
        assert_eq!(target.segments, vec!["살림".to_string(), "공".to_string()]);
        let Expr::Call { name, args, .. } = value else {
            panic!("value must be map set call");
        };
        assert_eq!(name, "짝맞춤.바꾼값");
        assert_eq!(args.len(), 3);
        assert!(matches!(
            &args[1].expr,
            Expr::Literal(Literal::Str(key), _) if key == "속도"
        ));
        let Expr::Call {
            name: inner_name,
            args: inner_args,
            ..
        } = &args[2].expr
        else {
            panic!("nested map write call expected");
        };
        assert_eq!(inner_name, "짝맞춤.바꾼값");
        assert_eq!(inner_args.len(), 3);
        assert!(matches!(
            &inner_args[0].expr,
            Expr::Call { name, .. } if name == "짝맞춤.필수값"
        ));
        assert!(matches!(
            &inner_args[1].expr,
            Expr::Literal(Literal::Str(key), _) if key == "x"
        ));
    }

    #[test]
    fn parse_event_surface_canonical_lowers_to_if() {
        let source = r#"
"jump"라는 알림이 오면 {
  살림.y <- 1.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::If { condition, .. }) = program.stmts.first() else {
            panic!("event surface must lower to if");
        };
        let Expr::Call { name, args, .. } = condition else {
            panic!("condition must be 사건.있나 call");
        };
        assert_eq!(name, "사건.있나");
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn parse_event_surface_alias_forbidden() {
        let source = r#"
"jump"라는 소식이 오면 {
  살림.y <- 1.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("alias forbidden");
        assert_eq!(err.code(), "E_EVENT_SURFACE_ALIAS_FORBIDDEN");
    }

    #[test]
    fn parse_neomeo_block_alias_as_open_block() {
        let source = r#"
너머 {
  살림.x <- () 너머.시각.지금.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        assert!(program
            .stmts
            .iter()
            .any(|stmt| matches!(stmt, Stmt::OpenBlock { .. })));
    }

    #[test]
    fn parse_execution_policy_block_as_pragma() {
        let source = r#"
실행정책 {
  실행모드: 일반.
  효과정책: 허용.
  슬기훅정책: 실행.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Some(Stmt::Pragma { name, args, .. }) = program.stmts.first() else {
            panic!("execution policy should lower to pragma");
        };
        assert_eq!(name, "실행정책");
        assert!(args.contains("실행모드=일반"));
        assert!(args.contains("효과정책=허용"));
        assert!(args.contains("슬기훅정책=실행"));
    }

    #[test]
    fn parse_named_seed_kind_imja_and_alrimssi() {
        let source = r#"
(온도:수, 풍속:수) 기상특보:알림씨 = {
}.

관제탑:임자 = {
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        assert!(program.stmts.iter().any(|stmt| {
            matches!(
                stmt,
                Stmt::SeedDef {
                    name,
                    kind: SeedKind::Named(kind),
                    ..
                } if name == "기상특보" && kind == "알림씨"
            )
        }));
        assert!(program.stmts.iter().any(|stmt| {
            matches!(
                stmt,
                Stmt::SeedDef {
                    name,
                    kind: SeedKind::Named(kind),
                    ..
                } if name == "관제탑" && kind == "임자"
            )
        }));
    }

    #[test]
    fn parse_signal_send_with_payload_and_sender() {
        let source = r#"
기상청:임자 = {
  (온도:1, 풍속:12@m/s) 기상특보 ~~> 관제탑.
  (기상청)의 (온도:2, 풍속:14@m/s) 기상특보 ~~> 관제탑.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Stmt::SeedDef { body, .. } = &program.stmts[0] else {
            panic!("seed def expected");
        };
        assert!(matches!(
            &body[0],
            Stmt::Send {
                sender: None,
                payload: Expr::Call { name, .. },
                receiver: Expr::Path(_),
                ..
            } if name == "기상특보"
        ));
        assert!(matches!(
            &body[1],
            Stmt::Send {
                sender: Some(Expr::Path(_)),
                payload: Expr::Call { name, .. },
                receiver: Expr::Path(_),
                ..
            } if name == "기상특보"
        ));
    }

    #[test]
    fn parse_receive_hooks_typed_and_generic() {
        let source = r#"
관제탑:임자 = {
  기상특보를 받으면 {
    제.경보상태 <- 참.
  }.

  (정보 정보.온도 > 40)인 기상특보를 받으면 {
    제.경보상태 <- 참.
  }.

  알림을 받으면 {
    제.최근알림 <- 알림.이름.
  }.

  (알림 알림.이름 == "기상특보")인 알림을 받으면 {
    제.최근알림 <- 알림.이름.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        let Stmt::SeedDef { body, .. } = &program.stmts[0] else {
            panic!("seed def expected");
        };
        assert!(matches!(
            &body[0],
            Stmt::Receive {
                kind: Some(kind),
                binding: None,
                condition: None,
                ..
            } if kind == "기상특보"
        ));
        assert!(matches!(
            &body[1],
            Stmt::Receive {
                kind: Some(kind),
                binding: Some(binding),
                condition: Some(_),
                ..
            } if kind == "기상특보" && binding == "정보"
        ));
        assert!(matches!(
            &body[2],
            Stmt::Receive {
                kind: None,
                binding: None,
                condition: None,
                ..
            }
        ));
        assert!(matches!(
            &body[3],
            Stmt::Receive {
                kind: None,
                binding: Some(binding),
                condition: Some(_),
                ..
            } if binding == "알림"
        ));
    }

    #[test]
    fn parse_receive_hook_outside_imja_forbidden() {
        let source = r#"
기상청:움직씨 = {
  알림을 받으면 {
    참 보여주기.
  }.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("must reject");
        assert_eq!(err.code(), "E_RECEIVE_OUTSIDE_IMJA");
    }
}
