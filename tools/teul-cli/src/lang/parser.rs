use crate::core::fixed64::Fixed64;
use crate::core::unit::{UnitExpr, UnitFactor};
use crate::lang::ast::{
    ArgBinding, BinaryOp, Binding, BindingReason, ChooseBranch, ContractKind, ContractMode,
    DeclItem, DeclKind, Expr, FormulaDialect, HookKind, Literal, NumberLiteral, ParamPin, Path,
    Program, SeedKind, Stmt, UnaryOp,
};
use crate::lang::span::Span;
use crate::lang::token::{Token, TokenKind};
use std::collections::HashSet;

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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    Strict,
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
    declared_scopes: Vec<HashSet<String>>,
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
        _mode: ParseMode,
    ) -> Result<Program, ParseError> {
        let mut parser = Parser {
            tokens,
            pos: 0,
            default_root: default_root.to_string(),
            root_hide: default_root == "바탕",
            declared_scopes: vec![HashSet::new()],
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

        let mut injected = false;
        for stmt in rest.iter_mut() {
            if let Stmt::SeedDef {
                name, kind, body, ..
            } = stmt
            {
                if matches!(kind, SeedKind::Umjikssi) && (name == "매틱" || name == "매마디") {
                    let mut new_body = Vec::with_capacity(decls.len() + body.len());
                    new_body.append(&mut decls);
                    new_body.append(body);
                    *body = new_body;
                    injected = true;
                    break;
                }
            }
        }

        if !injected {
            let span = if let (Some(first), Some(last)) = (decls.first(), decls.last()) {
                Self::stmt_span(first).merge(Self::stmt_span(last))
            } else {
                self.peek().span
            };
            rest.push(Stmt::SeedDef {
                name: "매틱".to_string(),
                params: Vec::new(),
                kind: SeedKind::Umjikssi,
                body: decls,
                span,
            });
        }

        *stmts = rest;
        Ok(())
    }

    fn stmt_span(stmt: &Stmt) -> Span {
        match stmt {
            Stmt::DeclBlock { span, .. }
            | Stmt::SeedDef { span, .. }
            | Stmt::Assign { span, .. }
            | Stmt::Expr { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::Show { span, .. }
            | Stmt::Hook { span, .. }
            | Stmt::OpenBlock { span, .. }
            | Stmt::Repeat { span, .. }
            | Stmt::Break { span, .. }
            | Stmt::Choose { span, .. }
            | Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::ForEach { span, .. }
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
            if Self::is_forbidden_setting_pragma(&raw) {
                return Err(ParseError::UnexpectedToken {
                    expected: "길잡이말(#...) 없이 설정:/보개:/슬기: 블록 사용",
                    found: TokenKind::Pragma(raw),
                    span,
                });
            }
            let (name, args) = Self::split_pragma_parts(&raw);
            return Ok(Some(Stmt::Pragma { name, args, span }));
        }

        if !allow_rbrace {
            if let Some(seed_def) = self.try_parse_seed_def()? {
                return Ok(Some(seed_def));
            }
        }

        if let Some(legacy) = self.peek_legacy_decl_block_name() {
            return Err(ParseError::UnexpectedToken {
                expected: "'채비:'",
                found: TokenKind::Ident(legacy.to_string()),
                span: self.peek().span,
            });
        }

        if let Some(kind) = self.peek_decl_block_kind() {
            return Ok(Some(self.parse_decl_block(kind)?));
        }

        if let Some(hook) = self.try_parse_hook()? {
            return Ok(Some(hook));
        }

        if self.is_open_block_start() {
            return Ok(Some(self.parse_open_block_stmt()?));
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

        if self.is_bogae_draw_stmt() {
            return Ok(Some(self.parse_bogae_draw_stmt()?));
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

        let expr = self.parse_expr()?;

        if self.peek_kind_is(|k| matches!(k, TokenKind::Ilttae)) {
            let stmt = self.parse_if_stmt(expr)?;
            return Ok(Some(stmt));
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::Jeonjehae | TokenKind::Bojanghago)) {
            let stmt = self.parse_contract_stmt(expr)?;
            return Ok(Some(stmt));
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
            let span = expr.span().merge(value.span());
            self.consume_terminator()?;
            if op.is_some() {
                let Expr::Path(path) = expr else {
                    return Err(ParseError::UnsupportedCompoundTarget { span: expr.span() });
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
                    span,
                }));
            }

            match expr {
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
                        span,
                    }));
                }
                Expr::Path(path) => {
                    if let Some((target, field)) = Self::split_map_dot_target(&path) {
                        self.ensure_root_declared_for_write(&target)?;
                        let set_span = path.span.merge(value.span());
                        let key_expr = Expr::Literal(Literal::Str(field), path.span);
                        let value_call = Expr::Call {
                            name: "짝맞춤.바꾼값".to_string(),
                            args: vec![
                                self.new_arg_binding(Expr::Path(target.clone())),
                                self.new_arg_binding(key_expr),
                                self.new_arg_binding(value),
                            ],
                            span: set_span,
                        };
                        return Ok(Some(Stmt::Assign {
                            target,
                            value: value_call,
                            span,
                        }));
                    }
                    self.ensure_root_declared_for_write(&path)?;
                    return Ok(Some(Stmt::Assign {
                        target: path,
                        value,
                        span,
                    }));
                }
                other => return Err(ParseError::ExpectedTarget { span: other.span() }),
            }
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "되돌림")) {
            let end_span = self.advance().span;
            let span = expr.span().merge(end_span);
            self.consume_terminator()?;
            return Ok(Some(Stmt::Return { value: expr, span }));
        }

        if self.peek_kind_is(|k| matches!(k, TokenKind::Boyeojugi)) {
            self.advance();
            let span = expr.span();
            self.consume_terminator()?;
            return Ok(Some(Stmt::Show { value: expr, span }));
        }

        let span = expr.span();
        self.consume_terminator()?;
        Ok(Some(Stmt::Expr { value: expr, span }))
    }

    fn try_parse_hook(&mut self) -> Result<Option<Stmt>, ParseError> {
        if self.is_hook_start() {
            return Ok(Some(self.parse_hook(HookKind::Start)?));
        }
        if self.is_hook_every() {
            return Ok(Some(self.parse_hook(HookKind::EveryMadi)?));
        }
        Ok(None)
    }

    fn is_hook_start(&self) -> bool {
        self.peek_kind_is(|k| matches!(k, TokenKind::LParen))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Start))
            && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::RParen))
            && self.peek_kind_n_is(3, |k| matches!(k, TokenKind::Halttae))
    }

    fn is_hook_every(&self) -> bool {
        self.peek_kind_is(|k| matches!(k, TokenKind::LParen))
            && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::EveryMadi))
            && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::RParen))
            && self.peek_kind_n_is(3, |k| matches!(k, TokenKind::Mada))
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

    fn peek_decl_block_kind(&self) -> Option<DeclKind> {
        if !self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Colon)) {
            return None;
        }
        match &self.peek().kind {
            TokenKind::Ident(name) if name == "채비" => Some(DeclKind::Gureut),
            _ => None,
        }
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

    fn parse_decl_block(&mut self, _kind: DeclKind) -> Result<Stmt, ParseError> {
        let start_span = self.advance().span;
        let TokenKind::Ident(keyword) = &self.tokens[self.pos - 1].kind else {
            unreachable!("decl block keyword must be identifier")
        };
        if keyword != "채비" {
            return Err(ParseError::UnexpectedToken {
                expected: "'채비:'",
                found: TokenKind::Ident(keyword.clone()),
                span: start_span,
            });
        }
        self.advance(); // colon
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
            let mut item_kind = DeclKind::Gureut;
            let mut end_span = type_token.span;
            if self.peek_kind_is(|k| matches!(k, TokenKind::Arrow)) {
                self.advance();
                let expr = self.parse_expr()?;
                end_span = end_span.merge(expr.span());
                value = Some(expr);
            } else if self.peek_kind_is(|k| matches!(k, TokenKind::Equal)) {
                item_kind = DeclKind::Butbak;
                self.advance();
                let expr = self.parse_expr()?;
                end_span = end_span.merge(expr.span());
                value = Some(expr);
            }

            let item_span = name_token.span.merge(end_span);
            self.consume_terminator()?;

            self.declare_name(&name);

            items.push(DeclItem {
                name,
                kind: item_kind,
                type_name,
                value,
                span: item_span,
            });
        }

        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;

        Ok(Stmt::DeclBlock { items, span })
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
        Ok(Stmt::Hook { kind, body, span })
    }

    fn is_open_block_start(&self) -> bool {
        if !self.peek_kind_is(|k| matches!(k, TokenKind::Ident(name) if name == "열림")) {
            return false;
        }
        self.peek_next_non_newline_is(|k| matches!(k, TokenKind::LBrace))
    }

    fn parse_open_block_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.advance().span;
        self.skip_newlines();
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

    fn is_forbidden_setting_pragma(raw: &str) -> bool {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return false;
        }
        let head = trimmed
            .split(|ch: char| ch == ':' || ch == '(' || ch.is_whitespace())
            .next()
            .unwrap_or("")
            .trim();
        matches!(head, "설정" | "보개" | "슬기" | "설정보개")
    }

    fn split_pragma_parts(raw: &str) -> (String, String) {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return (String::new(), String::new());
        }
        if let Some((name, rest)) = trimmed.split_once(':') {
            return (name.trim().to_string(), rest.trim().to_string());
        }
        if let Some(idx) = trimmed.find(|ch: char| ch.is_whitespace()) {
            let (name, rest) = trimmed.split_at(idx);
            return (name.trim().to_string(), rest.trim().to_string());
        }
        (trimmed.to_string(), String::new())
    }

    fn parse_repeat_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.advance().span;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            return Err(ParseError::UnexpectedToken {
                expected: "':'",
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
        if !self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
            return Err(ParseError::UnexpectedToken {
                expected: "':'",
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
                        .map(|tok| matches!(tok.kind, TokenKind::Colon))
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
            if !self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "':'",
                    found: self.peek().kind.clone(),
                    span: self.peek().span,
                });
            }
            self.advance();
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
            if self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
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
                } else {
                    if !self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
                        self.pos = start_pos;
                        return Ok(None);
                    }
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
                }
            }
            _ => {
                self.pos = start_pos;
                return Ok(None);
            }
        };

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
        let body = self.parse_block()?;
        if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
            return Err(ParseError::ExpectedRBrace {
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        Ok(Some(Stmt::SeedDef {
            name,
            params,
            kind,
            body,
            span,
        }))
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
        let mut end_span = start_span;

        loop {
            self.skip_newlines();
            if self.peek_kind_is(|k| matches!(k, TokenKind::Aniramyeon)) {
                self.advance();
                if !self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
                    return Err(ParseError::UnexpectedToken {
                        expected: "':'",
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
                let body = self.parse_block()?;
                if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
                    return Err(ParseError::ExpectedRBrace {
                        span: self.peek().span,
                    });
                }
                end_span = self.advance().span;
                else_body = Some(body);
                break;
            }
            if self.peek_kind_is(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
                break;
            }
            if !self.peek_kind_is(|k| matches!(k, TokenKind::LBrace)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "'{'",
                    found: self.peek().kind.clone(),
                    span: self.peek().span,
                });
            }
            let condition = self.parse_condition_expr()?;
            if !self.peek_kind_is(|k| matches!(k, TokenKind::Colon)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "':'",
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
            let body = self.parse_block()?;
            if !self.peek_kind_is(|k| matches!(k, TokenKind::RBrace)) {
                return Err(ParseError::ExpectedRBrace {
                    span: self.peek().span,
                });
            }
            end_span = self.advance().span;
            branches.push(ChooseBranch { condition, body });
        }

        let Some(else_body) = else_body else {
            return Err(ParseError::UnexpectedToken {
                expected: "'아니면:'",
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        };
        let span = start_span.merge(end_span);
        self.consume_terminator()?;
        Ok(Stmt::Choose {
            branches,
            else_body,
            span,
        })
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
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "'알림' or '중단'",
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
            "중단" => Ok(ContractMode::Abort),
            _ => Err(ParseError::UnexpectedToken {
                expected: "'알림' or '중단'",
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
            if self.peek_kind_is(|k| matches!(k, TokenKind::Dot))
                && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Ident(_)))
            {
                let start_span = expr.span();
                self.advance();
                let field_token = self.advance();
                let field = match field_token.kind {
                    TokenKind::Ident(name) => name,
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            expected: "field name",
                            found: field_token.kind,
                            span: field_token.span,
                        })
                    }
                };
                let span = start_span.merge(field_token.span);
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
            if self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Ident(_)))
                && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::Equal))
            {
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
            TokenKind::Ident(_) => {
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
                if self.peek_kind_is(|k| matches!(k, TokenKind::Ident(_) | TokenKind::Salim)) {
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
        Err(ParseError::UnexpectedToken {
            expected: "'채우기' or '풀기'",
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
            if !self.peek_kind_is(|k| matches!(k, TokenKind::Equal)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "'='",
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
            if !self.peek_kind_is(|k| matches!(k, TokenKind::Dot)) {
                break;
            }
            if !self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Ident(_))) {
                break;
            }
            self.advance();
            let ident_token = self.advance();
            if let TokenKind::Ident(name) = ident_token.kind {
                segments.push(name);
                span = span.merge(ident_token.span);
            }
        }

        if !matches!(
            segments.first().map(|seg| seg.as_str()),
            Some("살림" | "바탕" | "샘")
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

    // Step B scope: one-level map field write only.
    // - allowed: 살림.공.x, 바탕.공.x
    // - rejected: 살림.공.속도.x (Step C)
    fn split_map_dot_target(path: &Path) -> Option<(Path, String)> {
        if path.segments.len() != 3 {
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
        Some((target, path.segments[2].clone()))
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
    fn parse_pragma_graph_stmt_noop() {
        let source = "#그래프(y축=x)\n살림.x <- 1.\n";
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("pragma parsed");
        assert!(matches!(program.stmts.first(), Some(Stmt::Pragma { .. })));
        assert!(program
            .stmts
            .iter()
            .any(|stmt| matches!(stmt, Stmt::Assign { .. })));
    }

    #[test]
    fn parse_pragma_import_like_stmt_noop() {
        let source = "#가져오기 누리/물리/역학 (중력씨, 이동씨)\n";
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("pragma parsed");
        assert_eq!(program.stmts.len(), 1);
        assert!(matches!(program.stmts[0], Stmt::Pragma { .. }));
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
채비: {
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
    fn parse_decl_block_legacy_header_rejected() {
        let source = r#"
그릇채비: {
  점수:수 <- 0.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let err = Parser::parse_with_default_root(tokens, "살림").expect_err("legacy header");
        assert_eq!(err.code(), "E_PARSE_UNEXPECTED_TOKEN");
    }

    #[test]
    fn parse_hook_every_allows_optional_colon() {
        let source = r#"
(매마디)마다: {
  살림.t <- 살림.t + 1.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let program = Parser::parse_with_default_root(tokens, "살림").expect("parse");
        assert!(program.stmts.iter().any(|stmt| matches!(
            stmt,
            Stmt::Hook {
                kind: HookKind::EveryMadi,
                ..
            }
        )));
    }
}
