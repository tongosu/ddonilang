// lang/src/parser.rs
use crate::ast::*;
use crate::lexer::{Lexer, Token, TokenKind};
use crate::normalizer::{NormalizationLevel, Normalizer};
use crate::stdlib::minimal_stdlib_sigs;
use crate::term_map;
use ddonirang_core::{is_known_unit, unit_spec_from_symbol, Fixed64, UnitDim};
use std::collections::{HashMap, HashSet};
use std::fmt;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    nid: u64,
    root_hide: bool,
    beat_depth: usize,
    declared_scopes: Vec<HashSet<String>>,
}
struct ArgSuffix {
    josa: Option<String>,
    binding_reason: BindingReason,
    fixed_pin: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    Strict,
}
#[derive(Debug, Clone, Copy)]
enum DimState {
    Known(UnitDim),
    Unknown,
}
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self::new_with_mode(tokens, ParseMode::Strict)
    }
    pub fn new_with_mode(tokens: Vec<Token>, _mode: ParseMode) -> Self {
        Self {
            tokens,
            pos: 0,
            nid: 1,
            root_hide: false,
            beat_depth: 0,
            declared_scopes: vec![HashSet::new()],
        }
    }
    fn next_id(&mut self) -> NodeId {
        let id = self.nid;
        self.nid += 1;
        id
    }
    pub fn parse_program(
        &mut self,
        source: String,
        file_path: String,
    ) -> Result<CanonProgram, ParseError> {
        self.root_hide = false;
        self.beat_depth = 0;
        self.declared_scopes.clear();
        self.declared_scopes.push(HashSet::new());
        let mut items = Vec::new();
        let mut top_level_decl = Vec::new();
        while !self.is_at_end() {
            if matches!(self.current().kind, TokenKind::Pragma(_)) {
                return Err(ParseError {
                    span: self.current_span(),
                    message: "길잡이말(#...)은 더 이상 허용하지 않습니다. 설정:/보개:/슬기: 블록을 사용하세요".to_string(),
                });
            }
            if matches!(self.current().kind, TokenKind::BogeaJangmyeonBlock(_)) {
                return Err(ParseError {
                    span: self.current_span(),
                    message:
                        "`보개장면`은 더 이상 허용하지 않습니다. `보개마당`을 사용하세요"
                            .to_string(),
                });
            }
            if let Some(legacy) = self.peek_legacy_decl_block_name() {
                return Err(ParseError {
                    span: self.current_span(),
                    message: format!(
                        "`{legacy}:`는 더 이상 허용하지 않습니다. 선언 블록은 `채비`만 사용합니다"
                    ),
                });
            }
            if matches!(self.current().kind, TokenKind::BogeaMadangBlock(_)) {
                let _ = self.parse_bogae_madang_block_stmt()?;
                continue;
            }
            if matches!(self.current().kind, TokenKind::JjaimBlock(_)) {
                let _ = self.parse_jjaim_block_stmt()?;
                continue;
            }
            if self.peek_meta_block_kind().is_some() {
                let _ = self.parse_meta_block_stmt()?;
                continue;
            }
            if let Some(kind) = self.peek_decl_block_kind() {
                let stmt = self.parse_decl_block(kind)?;
                top_level_decl.push(stmt);
                continue;
            }
            items.push(self.parse_top_level_item()?);
        }
        if !top_level_decl.is_empty() {
            self.inject_top_level_decl_blocks(&mut items, top_level_decl)?;
        }
        let mut program = CanonProgram {
            id: self.next_id(),
            items,
            origin: OriginMap {
                file_path,
                source,
                node_spans: std::collections::HashMap::new(),
            },
        };
        self.validate_seed_name_conflicts(&program)?;
        self.apply_default_args(&mut program)?;
        self.validate_units(&program)?;
        Ok(program)
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
    fn ensure_root_declared_for_write(&self, target: &Expr) -> Result<(), ParseError> {
        if !self.root_hide {
            return Ok(());
        }
        let ExprKind::Var(name) = &target.kind else {
            return Ok(());
        };
        if self.is_declared(name) {
            return Ok(());
        }
        Err(ParseError {
            span: target.span,
            message: format!("바탕숨김에서 미등록 그릇 쓰기: {}", name),
        })
    }
    fn parse_top_level_item(&mut self) -> Result<TopLevelItem, ParseError> {
        Ok(TopLevelItem::SeedDef(self.parse_seed_def()?))
    }
    fn parse_seed_def(&mut self) -> Result<SeedDef, ParseError> {
        let start = self.current_span();
        let params = if self.check(&TokenKind::LParen) {
            self.parse_params()?
        } else {
            Vec::new()
        };
        let name = self.expect_ident("씨앗 이름")?.raw.clone();
        self.validate_seed_name_tail(&name, self.previous_span())?;
        self.expect(&TokenKind::Colon, ":")?;
        let kind = self.parse_seed_kind()?;
        self.expect(&TokenKind::Equals, "=")?;
        self.enter_scope();
        for param in &params {
            self.declare_name(&param.pin_name);
        }
        let body = if self.check(&TokenKind::LBrace) {
            let body = self.parse_body();
            match body {
                Ok(body) => body,
                Err(err) => {
                    self.exit_scope();
                    return Err(err);
                }
            }
        } else {
            let expr = self.parse_expr()?;
            let mood = self.consume_optional_terminator()?;
            Body {
                id: self.next_id(),
                span: expr.span,
                stmts: vec![Stmt::Return {
                    id: self.next_id(),
                    span: expr.span,
                    mood,
                    value: expr,
                }],
            }
        };
        self.exit_scope();
        Ok(SeedDef {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            canonical_name: name,
            seed_kind: kind,
            params,
            body: Some(body),
            modifiers: Vec::new(),
        })
    }
    fn parse_params(&mut self) -> Result<Vec<ParamPin>, ParseError> {
        self.expect(&TokenKind::LParen, "(")?;
        let mut ps = Vec::new();
        let mut seen_default = false;
        while !self.check(&TokenKind::RParen) {
            let s = self.current_span();
            let name = self.expect_ident("파라미터")?.raw.clone();
            self.expect(&TokenKind::Colon, ":")?;
            let type_ref = self.parse_type_ref()?;
            let mut josa_list = Vec::new();
            while self.check(&TokenKind::Tilde) {
                self.advance();
                if let TokenKind::Josa(josa) = &self.current().kind {
                    josa_list.push(josa.clone());
                    self.advance();
                } else {
                    return Err(self.error("조사"));
                }
            }
            let mut optional = false;
            if self.check(&TokenKind::Question) {
                self.advance();
                optional = true;
            }
            let mut default_value = None;
            if self.check(&TokenKind::Equals) {
                self.advance();
                default_value = Some(self.parse_expr()?);
                seen_default = true;
            } else if seen_default {
                return Err(self.error("기본값 매개변수 뒤에는 필수 인자를 둘 수 없음"));
            }
            ps.push(ParamPin {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                pin_name: name,
                type_ref,
                default_value,
                optional,
                josa_list,
            });
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&TokenKind::RParen, ")")?;
        Ok(ps)
    }

    fn parse_type_ref(&mut self) -> Result<TypeRef, ParseError> {
        if self.check(&TokenKind::LParen) {
            self.advance();
            let mut args = Vec::new();
            if self.check(&TokenKind::RParen) {
                return Err(self.error("타입 인자"));
            }
            while !self.check(&TokenKind::RParen) {
                args.push(self.parse_type_ref()?);
                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }
            self.expect(&TokenKind::RParen, ")")?;
            let name = self.expect_ident("타입")?.raw.clone();
            if name == "_" {
                return Err(self.error("타입 이름"));
            }
            return Ok(TypeRef::Applied { name, args });
        }
        let name = self.expect_ident("타입")?.raw.clone();
        if name == "_" {
            Ok(TypeRef::Infer)
        } else {
            Ok(TypeRef::Named(name))
        }
    }
    fn parse_seed_kind(&mut self) -> Result<SeedKind, ParseError> {
        let t = self.advance();
        match &t.kind {
            TokenKind::KwImeumssi => Ok(SeedKind::Imeumssi),
            TokenKind::KwUmjikssi => Ok(SeedKind::Umjikssi),
            TokenKind::KwValueFunc => Ok(SeedKind::Semssi),
            TokenKind::KwHeureumssi => Ok(SeedKind::Heureumssi),
            TokenKind::KwGallaessi => Ok(SeedKind::Gallaessi),
            TokenKind::KwRelationssi => Ok(SeedKind::Relationssi),
            TokenKind::KwSam => Ok(SeedKind::Sam),
            TokenKind::KwIeumssi => Ok(SeedKind::Ieumssi),
            TokenKind::KwSemssi => Ok(SeedKind::Semssi),
            TokenKind::Ident(name) | TokenKind::Josa(name) => {
                if let Some(replacement) = legacy_seed_kind_replacement(name) {
                    return Err(ParseError {
                        span: self.to_ast_span(t.span),
                        message: format!(
                            "`{name}`는 더 이상 허용하지 않습니다. `{replacement}`를 사용하세요"
                        ),
                    });
                }
                self.validate_ident_token(&t)?;
                Ok(SeedKind::Named(name.clone()))
            }
            _ => Err(self.error("씨앗 종류")),
        }
    }
    fn parse_body(&mut self) -> Result<Body, ParseError> {
        let s = self.current_span();
        self.expect(&TokenKind::LBrace, "{")?;
        self.enter_scope();
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) {
            stmts.push(self.parse_stmt(false)?);
        }
        self.expect(&TokenKind::RBrace, "}")?;
        self.exit_scope();
        Ok(Body {
            id: self.next_id(),
            span: s.merge(&self.previous_span()),
            stmts,
        })
    }

    fn expect_colon_or_lbrace(&mut self, context: &str) -> Result<(), ParseError> {
        if self.check(&TokenKind::Colon) {
            self.advance();
        }
        if !self.check(&TokenKind::LBrace) {
            return Err(self.error(context));
        }
        Ok(())
    }

    fn parse_thunk_body_after_lbrace(&mut self, start: Span) -> Result<Body, ParseError> {
        self.enter_scope();
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) {
            stmts.push(self.parse_stmt(true)?);
        }
        self.expect(&TokenKind::RBrace, "}")?;
        self.exit_scope();
        Ok(Body {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            stmts,
        })
    }

    fn parse_stmt(&mut self, allow_implicit_terminator: bool) -> Result<Stmt, ParseError> {
        let s = self.current_span();
        if matches!(self.current().kind, TokenKind::Pragma(_)) {
            return Err(ParseError {
                span: s,
                message: "길잡이말(#...)은 더 이상 허용하지 않습니다. 설정:/보개:/슬기: 블록을 사용하세요".to_string(),
            });
        }
        if matches!(self.current().kind, TokenKind::BogeaJangmyeonBlock(_)) {
            return Err(ParseError {
                span: s,
                message: "`보개장면`은 더 이상 허용하지 않습니다. `보개마당`을 사용하세요"
                    .to_string(),
            });
        }
        if matches!(self.current().kind, TokenKind::BogeaMadangBlock(_)) {
            return self.parse_bogae_madang_block_stmt();
        }
        if matches!(self.current().kind, TokenKind::JjaimBlock(_)) {
            return self.parse_jjaim_block_stmt();
        }
        if self.peek_meta_block_kind().is_some() {
            return self.parse_meta_block_stmt();
        }
        if self.check(&TokenKind::KwGoreugi) {
            return self.parse_choose_stmt();
        }
        if self.check(&TokenKind::KwBanbok) {
            return self.parse_repeat_stmt();
        }
        if self.check(&TokenKind::KwMeomchugi) {
            return self.parse_break_stmt();
        }
        if self.is_foreach_start() {
            return self.parse_foreach_stmt();
        }
        if self.is_quantifier_start() {
            return self.parse_quantifier_stmt();
        }
        if let Some(legacy) = self.peek_legacy_decl_block_name() {
            return Err(ParseError {
                span: self.current_span(),
                message: format!(
                    "`{legacy}:`는 더 이상 허용하지 않습니다. 선언 블록은 `채비`만 사용합니다"
                ),
            });
        }
        if let Some(kind) = self.peek_decl_block_kind() {
            return self.parse_decl_block(kind);
        }
        if self.is_hook_start() {
            return self.parse_hook_stmt();
        }
        if self.is_beat_block_start() {
            return self.parse_beat_block_stmt();
        }

        let e = self.parse_expr()?;

        if self.check(&TokenKind::KwNeuljikeobogo) {
            self.advance();
            let body = self.parse_body()?;
            self.validate_guard_body(&body, s.merge(&self.previous_span()))?;
            let mood = self.consume_optional_terminator()?;
            return Ok(Stmt::Guard {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                mood,
                condition: e,
                body,
            });
        }

        if self.check(&TokenKind::KwHaebogo) {
            self.advance();
            self.expect(&TokenKind::Colon, ":")?;
            let body = self.parse_body()?;
            let mood = self.consume_optional_terminator()?;
            return Ok(Stmt::Try {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                mood,
                action: e,
                body,
            });
        }

        if self.check(&TokenKind::KwJeonjehae) || self.check(&TokenKind::KwBojanghago) {
            let kind = if self.check(&TokenKind::KwJeonjehae) {
                self.advance();
                ContractKind::Pre
            } else {
                self.advance();
                ContractKind::Post
            };
            let mode = self.parse_contract_mode()?;
            self.ensure_eval_condition(&e, "계약 조건")?;
            let else_body = if self.check(&TokenKind::KwAniramyeon) {
                self.advance();
                self.parse_body()?
            } else {
                return Err(self.error("아니면"));
            };
            let then_body = if self.check(&TokenKind::KwMajeumyeon) {
                self.advance();
                Some(self.parse_body()?)
            } else {
                None
            };
            let mood = self.consume_optional_terminator()?;
            return Ok(Stmt::Contract {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                mood,
                kind,
                mode,
                condition: e,
                then_body,
                else_body,
            });
        }

        if self.check(&TokenKind::KwIlttae) || self.check(&TokenKind::KwMajeumyeon) {
            self.advance();
            let then_body = self.parse_body()?;
            let else_body = if self.check(&TokenKind::KwAniramyeon) {
                self.advance();
                Some(self.parse_body()?)
            } else {
                None
            };
            let mood = self.consume_optional_terminator()?;
            return Ok(Stmt::If {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                mood,
                condition: e,
                then_body,
                else_body,
            });
        }

        if self.check(&TokenKind::KwDongan) {
            self.advance();
            self.ensure_eval_condition(&e, "동안 조건")?;
            self.expect_colon_or_lbrace("동안 본문")?;
            let body = self.parse_body()?;
            let mood = self.consume_optional_terminator()?;
            return Ok(Stmt::While {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                mood,
                condition: e,
                body,
            });
        }

        if matches!(
            self.current().kind,
            TokenKind::PlusEqual | TokenKind::MinusEqual
        ) {
            return Err(self.error("'+='/'-='는 미지원입니다. '+<-'/'-<-'를 사용하세요"));
        }
        if let Some(op) = self.consume_compound_update() {
            let v = self.parse_expr()?;
            let deferred_span = self.consume_deferred_marker()?;
            let deferred = deferred_span.is_some();
            let mood = self.consume_stmt_terminator()?;
            if !self.is_simple_target(&e) {
                return Err(self.error("복합 갱신(+<-, -<-)은 이름 대상만 허용됩니다"));
            }
            self.ensure_root_declared_for_write(&e)?;
            let span = e.span.merge(&v.span);
            let value_expr = Expr::new(
                self.next_id(),
                span,
                ExprKind::Infix {
                    left: Box::new(e.clone()),
                    op: op.to_string(),
                    right: Box::new(v),
                },
            );
            return Ok(Stmt::Mutate {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                mood,
                target: e,
                value: value_expr,
                deferred,
                deferred_span,
            });
        }
        if self.consume_right_arrow() {
            let target = self.parse_expr()?;
            let deferred_span = self.consume_deferred_marker()?;
            let deferred = deferred_span.is_some();
            let mood = self.consume_stmt_terminator()?;
            self.build_mutate_stmt_from_assignment(s, target, e, mood, deferred, deferred_span)
        } else if self.consume_arrow() {
            let value = self.parse_expr()?;
            let deferred_span = self.consume_deferred_marker()?;
            let deferred = deferred_span.is_some();
            let mood = self.consume_stmt_terminator()?;
            self.build_mutate_stmt_from_assignment(s, e, value, mood, deferred, deferred_span)
        } else if self.check(&TokenKind::KwBoyeojugi) {
            self.advance();
            let mood = self.consume_stmt_terminator()?;
            Ok(Stmt::Show {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                mood,
                expr: e,
            })
        } else if self.check(&TokenKind::KwTolabogi) {
            self.advance();
            let mood = self.consume_stmt_terminator()?;
            Ok(Stmt::Inspect {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                mood,
                expr: e,
            })
        } else if self.check(&TokenKind::KwDollyeojwo) {
            self.advance();
            let mood = self.consume_stmt_terminator()?;
            Ok(Stmt::Return {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                mood,
                value: e,
            })
        } else {
            if allow_implicit_terminator && self.check(&TokenKind::RBrace) {
                return Ok(Stmt::Expr {
                    id: self.next_id(),
                    span: s.merge(&self.previous_span()),
                    mood: Mood::Declarative,
                    expr: e,
                });
            }
            let mood = self.consume_stmt_terminator()?;
            Ok(Stmt::Expr {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                mood,
                expr: e,
            })
        }
    }

    fn is_beat_block_start(&self) -> bool {
        if !self.is_beat_block_head() {
            return false;
        }
        self.peek_kind_n_is(1, |k| matches!(k, TokenKind::LBrace))
            || (self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Colon))
                && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::LBrace)))
    }

    fn is_beat_block_head(&self) -> bool {
        matches!(self.current().kind, TokenKind::KwBeat)
    }

    fn token_text_is(token: &TokenKind, expected: &str) -> bool {
        match token {
            TokenKind::Ident(name) | TokenKind::Josa(name) => name == expected,
            _ => false,
        }
    }

    fn is_hook_start(&self) -> bool {
        self.is_every_madi_hook_start()
            || self.is_start_end_hook_start()
            || self.is_condition_hook_start()
    }

    fn is_start_end_hook_start(&self) -> bool {
        if !self.check(&TokenKind::LParen) {
            return false;
        }
        let is_start_end_head = self.peek_kind_n_is(1, |k| {
            matches!(
                k,
                TokenKind::Ident(name) | TokenKind::Josa(name)
                    if name == "시작" || name == "처음" || name == "끝"
            )
        }) && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::RParen));
        if !is_start_end_head {
            return false;
        }
        self.peek_kind_n_is(3, |k| Self::token_text_is(k, "할때"))
    }

    fn find_matching_rparen_after_current_lparen(&self) -> Option<usize> {
        if !self.check(&TokenKind::LParen) {
            return None;
        }
        let mut depth = 0usize;
        for idx in self.pos..self.tokens.len() {
            match self.tokens.get(idx).map(|t| &t.kind) {
                Some(TokenKind::LParen) => depth += 1,
                Some(TokenKind::RParen) => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Some(idx);
                    }
                }
                Some(TokenKind::Eof) | None => return None,
                _ => {}
            }
        }
        None
    }

    fn is_condition_hook_start(&self) -> bool {
        if !self.check(&TokenKind::LParen) {
            return false;
        }
        if self.is_every_madi_hook_start() || self.is_start_end_hook_start() {
            return false;
        }
        let Some(rparen_idx) = self.find_matching_rparen_after_current_lparen() else {
            return false;
        };
        let after = rparen_idx + 1;
        let Some(first) = self.tokens.get(after).map(|t| &t.kind) else {
            return false;
        };
        if matches!(first, TokenKind::Josa(name) if name == "이" || name == "가")
            && self
                .tokens
                .get(after + 1)
                .map(|t| Self::token_text_is(&t.kind, "될때"))
                .unwrap_or(false)
        {
            return true;
        }
        Self::token_text_is(first, "인")
            && self
                .tokens
                .get(after + 1)
                .map(|t| matches!(t.kind, TokenKind::KwDongan) || Self::token_text_is(&t.kind, "동안"))
                .unwrap_or(false)
    }

    fn parse_hook_stmt(&mut self) -> Result<Stmt, ParseError> {
        if self.is_every_madi_hook_start() {
            return self.parse_every_madi_hook_stmt();
        }
        if self.is_start_end_hook_start() {
            return self.parse_start_end_hook_stmt();
        }
        self.parse_condition_hook_stmt()
    }

    fn is_every_madi_hook_start(&self) -> bool {
        if !self.check(&TokenKind::LParen) {
            return false;
        }
        let every_madi = self.peek_kind_n_is(1, |k| {
            matches!(k, TokenKind::Ident(name) | TokenKind::Josa(name) if name == "매마디" || name == "매틱")
        }) && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::RParen))
            && self.peek_kind_n_is(3, |k| Self::token_text_is(k, "마다"));
        if every_madi {
            return true;
        }
        self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Integer(_)))
            && self.peek_kind_n_is(2, |k| Self::token_text_is(k, "마디"))
            && self.peek_kind_n_is(3, |k| matches!(k, TokenKind::RParen))
            && self.peek_kind_n_is(4, |k| Self::token_text_is(k, "마다"))
    }

    fn parse_every_madi_hook_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::LParen, "(")?;
        let kind = match &self.current().kind {
            TokenKind::Ident(name) | TokenKind::Josa(name)
                if name == "매마디" || name == "매틱" =>
            {
                self.advance();
                HookKind::EveryMadi
            }
            TokenKind::Integer(value) => {
                let interval = *value;
                self.advance();
                if interval <= 0 {
                    return Err(ParseError {
                        span: self.current_span(),
                        message: "양의 정수 마디 간격만 허용됩니다".to_string(),
                    });
                }
                if !Self::token_text_is(&self.current().kind, "마디") {
                    return Err(self.error("마디"));
                }
                self.advance();
                HookKind::EveryNMadi(interval as u64)
            }
            _ => return Err(self.error("매마디 또는 N마디")),
        };
        self.expect(&TokenKind::RParen, ")")?;
        if !Self::token_text_is(&self.current().kind, "마다") {
            return Err(self.error("마다"));
        }
        self.advance();
        let (body, mood, span) =
            self.parse_hook_body_and_span(start, "`(매마디)마다` 뒤에는 ':' 없이 바로 '{'를 사용하세요")?;
        Ok(Stmt::Hook {
            id: self.next_id(),
            span,
            mood,
            kind,
            body,
        })
    }

    fn parse_start_end_hook_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::LParen, "(")?;
        let kind = match &self.current().kind {
            TokenKind::Ident(name) | TokenKind::Josa(name) if name == "시작" => {
                self.advance();
                HookKind::Start
            }
            TokenKind::Ident(name) | TokenKind::Josa(name) if name == "처음" => {
                self.advance();
                HookKind::Start
            }
            TokenKind::Ident(name) | TokenKind::Josa(name) if name == "끝" => {
                self.advance();
                HookKind::End
            }
            _ => return Err(self.error("시작 또는 끝")),
        };
        self.expect(&TokenKind::RParen, ")")?;
        if !Self::token_text_is(&self.current().kind, "할때") {
            return Err(self.error("할때"));
        }
        self.advance();
        let (body, mood, span) = self.parse_hook_body_and_span(
            start,
            "`(시작)할때`/`(끝)할때` 뒤에는 ':' 없이 바로 '{'를 사용하세요",
        )?;
        Ok(Stmt::Hook {
            id: self.next_id(),
            span,
            mood,
            kind,
            body,
        })
    }

    fn parse_condition_hook_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::LParen, "(")?;
        let condition = self.parse_expr()?;
        self.expect(&TokenKind::RParen, ")")?;
        if matches!(&self.current().kind, TokenKind::Josa(name) if name == "이" || name == "가") {
            self.advance();
            if !Self::token_text_is(&self.current().kind, "될때") {
                return Err(self.error("될때"));
            }
            self.advance();
            let (body, mood, span) = self
                .parse_hook_body_and_span(start, "`(조건)이 될때` 뒤에는 ':' 없이 바로 '{'를 사용하세요")?;
            return Ok(Stmt::HookWhenBecomes {
                id: self.next_id(),
                span,
                mood,
                condition,
                body,
            });
        }
        if Self::token_text_is(&self.current().kind, "인") {
            self.advance();
            if !matches!(self.current().kind, TokenKind::KwDongan)
                && !Self::token_text_is(&self.current().kind, "동안")
            {
                return Err(self.error("동안"));
            }
            self.advance();
            let (body, mood, span) = self
                .parse_hook_body_and_span(start, "`(조건)인 동안` 뒤에는 ':' 없이 바로 '{'를 사용하세요")?;
            return Ok(Stmt::HookWhile {
                id: self.next_id(),
                span,
                mood,
                condition,
                body,
            });
        }
        Err(self.error("훅 조건 머릿말"))
    }

    fn parse_hook_body_and_span(
        &mut self,
        start: Span,
        colon_message: &str,
    ) -> Result<(Body, Mood, Span), ParseError> {
        if self.check(&TokenKind::Colon) {
            return Err(ParseError {
                span: self.current_span(),
                message: colon_message.to_string(),
            });
        }
        let body = self.parse_body()?;
        let mood = self.consume_optional_terminator()?;
        let span = start.merge(&self.previous_span());
        Ok((body, mood, span))
    }

    fn parse_beat_block_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        self.advance();
        self.expect_colon_or_lbrace("덩이 본문")?;
        self.expect(&TokenKind::LBrace, "{")?;
        self.enter_scope();
        self.beat_depth += 1;

        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) {
            if self.check(&TokenKind::Eof) {
                self.beat_depth = self.beat_depth.saturating_sub(1);
                self.exit_scope();
                return Err(self.error("}"));
            }
            match self.parse_stmt(false) {
                Ok(stmt) => stmts.push(stmt),
                Err(err) => {
                    self.beat_depth = self.beat_depth.saturating_sub(1);
                    self.exit_scope();
                    return Err(err);
                }
            }
        }
        self.expect(&TokenKind::RBrace, "}")?;
        self.beat_depth = self.beat_depth.saturating_sub(1);
        self.exit_scope();
        let body = Body {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            stmts,
        };
        let mood = self.consume_optional_terminator()?;
        Ok(Stmt::BeatBlock {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            mood,
            body,
        })
    }

    fn consume_deferred_marker(&mut self) -> Result<Option<Span>, ParseError> {
        if !matches!(self.current().kind, TokenKind::KwReserve) {
            return Ok(None);
        }
        if self.beat_depth == 0 {
            return Err(ParseError {
                span: self.current_span(),
                message: "`미루기` 대입은 `덩이 {}` 안에서만 사용할 수 있습니다".to_string(),
            });
        }
        let span = self.current_span();
        self.advance();
        Ok(Some(span))
    }

    fn peek_meta_block_kind(&self) -> Option<MetaBlockKind> {
        let name = match &self.current().kind {
            TokenKind::Ident(name) | TokenKind::Josa(name) => name.as_str(),
            _ => return None,
        };
        let kind = match name {
            "설정" => MetaBlockKind::Setting,
            "보개" | "모양" => MetaBlockKind::Bogae,
            "슬기" => MetaBlockKind::Seulgi,
            _ => return None,
        };
        if self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Colon)) {
            if self.peek_kind_n_is(2, |k| matches!(k, TokenKind::LBrace)) {
                return Some(kind);
            }
            return None;
        }
        if self.peek_kind_n_is(1, |k| matches!(k, TokenKind::LBrace)) {
            return Some(kind);
        }
        None
    }

    fn parse_meta_block_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        let kind = self
            .peek_meta_block_kind()
            .ok_or_else(|| self.error("설정/보개/슬기 블록"))?;
        self.advance();
        self.expect_colon_or_lbrace("메타 블록 본문")?;
        self.expect(&TokenKind::LBrace, "{")?;

        let mut entries = Vec::new();
        while !self.check(&TokenKind::RBrace) {
            if self.check(&TokenKind::Eof) {
                return Err(self.error("}"));
            }
            let mut entry = String::new();
            while !self.check(&TokenKind::Dot) && !self.check(&TokenKind::RBrace) {
                let token = self.advance();
                append_meta_entry_token(&mut entry, &token);
            }
            if self.check(&TokenKind::Dot) {
                self.advance();
            }
            let trimmed = entry.trim();
            if !trimmed.is_empty() {
                entries.push(trimmed.to_string());
            }
        }
        self.expect(&TokenKind::RBrace, "}")?;
        let mood = self.consume_optional_terminator()?;
        Ok(Stmt::MetaBlock {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            mood,
            kind,
            entries,
        })
    }

    fn parse_bogae_madang_block_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        let token = self.advance();
        let raw = match &token.kind {
            TokenKind::BogeaMadangBlock(s) => s.clone(),
            _ => unreachable!(),
        };
        let mood = self.consume_optional_terminator()?;
        Ok(Stmt::MetaBlock {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            mood,
            kind: MetaBlockKind::BogeaMadang,
            entries: vec![raw],
        })
    }

    fn parse_jjaim_block_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        let token = self.advance();
        let raw = match &token.kind {
            TokenKind::JjaimBlock(s) => s.clone(),
            _ => unreachable!(),
        };
        let mood = self.consume_optional_terminator()?;
        Ok(Stmt::MetaBlock {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            mood,
            kind: MetaBlockKind::Jjaim,
            entries: vec![raw],
        })
    }

    fn peek_decl_block_kind(&self) -> Option<DeclKind> {
        let Some(t0) = self.tokens.get(self.pos) else {
            return None;
        };
        let TokenKind::Ident(name) = &t0.kind else {
            return None;
        };
        let Some(t1) = self.tokens.get(self.pos + 1) else {
            return None;
        };
        if !matches!(t1.kind, TokenKind::Colon | TokenKind::LBrace) {
            return None;
        }
        match name.as_str() {
            "채비" => Some(DeclKind::Gureut),
            _ => None,
        }
    }

    fn peek_legacy_decl_block_name(&self) -> Option<&'static str> {
        let Some(t0) = self.tokens.get(self.pos) else {
            return None;
        };
        let TokenKind::Ident(name) = &t0.kind else {
            return None;
        };
        let Some(t1) = self.tokens.get(self.pos + 1) else {
            return None;
        };
        if !matches!(t1.kind, TokenKind::Colon) {
            return None;
        }
        match name.as_str() {
            "그릇채비" => Some("그릇채비"),
            "붙박이마련" => Some("붙박이마련"),
            "붙박이채비" => Some("붙박이채비"),
            "바탕칸" => Some("바탕칸"),
            "바탕칸표" => Some("바탕칸표"),
            _ => None,
        }
    }

    fn parse_decl_block(&mut self, _kind: DeclKind) -> Result<Stmt, ParseError> {
        let s = self.current_span();
        let keyword = self.advance().raw.clone(); // consume keyword ident
        if keyword != "채비" {
            return Err(ParseError {
                span: s,
                message: "선언 블록 머릿말은 `채비`만 사용합니다".to_string(),
            });
        }
        self.expect_colon_or_lbrace("선언 블록 본문")?;
        self.expect(&TokenKind::LBrace, "{")?;
        let mut items = Vec::new();
        while !self.check(&TokenKind::RBrace) {
            let item_start = self.current_span();
            let name = self.expect_ident("선언 이름")?.raw.clone();
            self.expect(&TokenKind::Colon, ":")?;
            let type_ref = self.parse_type_ref()?;
            let mut item_kind = DeclKind::Gureut;
            let mut value = None;
            if self.check(&TokenKind::Arrow) {
                self.advance();
                value = Some(self.parse_expr()?);
            } else if self.check(&TokenKind::Equals) {
                item_kind = DeclKind::Butbak;
                self.advance();
                value = Some(self.parse_expr()?);
            }
            self.expect(&TokenKind::Dot, ".")?;
            let span = item_start.merge(&self.previous_span());
            let item = DeclItem {
                id: self.next_id(),
                span,
                name: name.clone(),
                kind: item_kind,
                type_ref,
                value,
            };
            self.declare_name(&name);
            items.push(item);
        }
        self.expect(&TokenKind::RBrace, "}")?;
        let mood = self.consume_optional_terminator()?;
        Ok(Stmt::DeclBlock {
            id: self.next_id(),
            span: s.merge(&self.previous_span()),
            mood,
            kind: DeclKind::Gureut,
            items,
        })
    }

    fn parse_choose_stmt(&mut self) -> Result<Stmt, ParseError> {
        let s = self.current_span();
        self.expect(&TokenKind::KwGoreugi, "고르기")?;
        self.expect(&TokenKind::Colon, ":")?;
        let mut branches = Vec::new();
        let mut else_body = None;
        loop {
            if self.check(&TokenKind::KwAniramyeon) {
                self.advance();
                self.expect(&TokenKind::Colon, ":")?;
                else_body = Some(self.parse_body()?);
                break;
            }
            if self.check(&TokenKind::RBrace) || self.check(&TokenKind::Eof) {
                break;
            }
            let cond = self.parse_expr()?;
            self.ensure_eval_condition(&cond, "고르기 조건")?;
            self.expect(&TokenKind::Colon, ":")?;
            let body = self.parse_body()?;
            branches.push(ChooseBranch {
                condition: cond,
                body,
            });
        }
        let Some(else_body) = else_body else {
            return Err(self.error("고르기에는 아니면 절이 필요합니다"));
        };
        let mood = self.consume_optional_terminator()?;
        Ok(Stmt::Choose {
            id: self.next_id(),
            span: s.merge(&self.previous_span()),
            mood,
            branches,
            else_body,
        })
    }

    fn parse_repeat_stmt(&mut self) -> Result<Stmt, ParseError> {
        let s = self.current_span();
        self.expect(&TokenKind::KwBanbok, "반복")?;
        self.expect_colon_or_lbrace("반복 본문")?;
        let body = self.parse_body()?;
        let mood = self.consume_optional_terminator()?;
        Ok(Stmt::Repeat {
            id: self.next_id(),
            span: s.merge(&self.previous_span()),
            mood,
            body,
        })
    }

    fn parse_break_stmt(&mut self) -> Result<Stmt, ParseError> {
        let s = self.current_span();
        self.expect(&TokenKind::KwMeomchugi, "멈추기")?;
        let mood = self.consume_stmt_terminator()?;
        Ok(Stmt::Break {
            id: self.next_id(),
            span: s.merge(&self.previous_span()),
            mood,
        })
    }

    fn is_foreach_start(&self) -> bool {
        if !self.check(&TokenKind::LParen) {
            return false;
        }
        let mut idx = self.pos + 1;
        let mut depth = 0usize;
        let mut saw_name = false;
        while let Some(token) = self.tokens.get(idx) {
            match &token.kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    if depth == 0 {
                        idx += 1;
                        break;
                    }
                    depth -= 1;
                }
                TokenKind::Ident(_) | TokenKind::Josa(_) if depth == 0 => {
                    saw_name = true;
                }
                TokenKind::Comma if depth == 0 => {
                    return false;
                }
                _ => {}
            }
            idx += 1;
        }
        if !saw_name {
            return false;
        }
        let mut depth = 0usize;
        while let Some(token) = self.tokens.get(idx) {
            match &token.kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    if depth > 0 {
                        depth -= 1;
                    }
                }
                TokenKind::KwDaehae if depth == 0 => {
                    return self
                        .tokens
                        .get(idx + 1)
                        .map(|tok| matches!(tok.kind, TokenKind::Colon | TokenKind::LBrace))
                        .unwrap_or(false);
                }
                TokenKind::Dot => {
                    if !self
                        .tokens
                        .get(idx + 1)
                        .map(|tok| matches!(tok.kind, TokenKind::Ident(_) | TokenKind::Josa(_)))
                        .unwrap_or(false)
                    {
                        break;
                    }
                }
                TokenKind::RBrace | TokenKind::Eof => break,
                _ => {}
            }
            idx += 1;
        }
        false
    }

    fn is_quantifier_start(&self) -> bool {
        if !matches!(
            self.current().kind,
            TokenKind::Ident(_) | TokenKind::Josa(_)
        ) {
            return false;
        }
        if !self.peek_kind_n_is(
            1,
            |kind| matches!(kind, TokenKind::Josa(name) if name == "이" || name == "가"),
        ) {
            return false;
        }
        if !self.peek_kind_n_is(2, |kind| {
            matches!(kind, TokenKind::Ident(_) | TokenKind::Josa(_))
        }) {
            return false;
        }
        if self.peek_kind_n_is(
            3,
            |kind| matches!(kind, TokenKind::Ident(name) if name == "낱낱"),
        ) && self.peek_kind_n_is(
            4,
            |kind| matches!(kind, TokenKind::Josa(name) if name == "에"),
        ) && self.peek_kind_n_is(5, |kind| matches!(kind, TokenKind::KwDaehae))
        {
            return self
                .tokens
                .get(self.pos + 6)
                .map(|token| matches!(token.kind, TokenKind::Colon | TokenKind::LBrace))
                .unwrap_or(false);
        }
        if self.peek_kind_n_is(
            3,
            |kind| matches!(kind, TokenKind::Ident(name) if name == "중"),
        ) && self.peek_kind_n_is(
            4,
            |kind| matches!(kind, TokenKind::Ident(name) if name == "하나"),
        ) && self.peek_kind_n_is(
            5,
            |kind| matches!(kind, TokenKind::Josa(name) if name == "가"),
        ) {
            return self
                .tokens
                .get(self.pos + 6)
                .map(|token| matches!(token.kind, TokenKind::Colon | TokenKind::LBrace))
                .unwrap_or(false);
        }
        if self.peek_kind_n_is(
            3,
            |kind| matches!(kind, TokenKind::Ident(name) if name == "중"),
        ) && self.peek_kind_n_is(
            4,
            |kind| matches!(kind, TokenKind::Ident(name) if name == "딱"),
        ) && self.peek_kind_n_is(
            5,
            |kind| matches!(kind, TokenKind::Ident(name) if name == "하나"),
        ) && self.peek_kind_n_is(
            6,
            |kind| matches!(kind, TokenKind::Josa(name) if name == "가"),
        ) {
            return self
                .tokens
                .get(self.pos + 7)
                .map(|token| matches!(token.kind, TokenKind::Colon | TokenKind::LBrace))
                .unwrap_or(false);
        }
        false
    }

    fn parse_foreach_stmt(&mut self) -> Result<Stmt, ParseError> {
        let s = self.current_span();
        self.expect(&TokenKind::LParen, "(")?;
        let name = self.expect_ident("항목")?.raw.clone();
        let item_type = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        self.expect(&TokenKind::RParen, ")")?;
        let iterable = self.parse_expr()?;
        if let TokenKind::Josa(value) = &self.current().kind {
            if value == "에" {
                self.advance();
            }
        }
        self.expect(&TokenKind::KwDaehae, "대해")?;
        self.expect_colon_or_lbrace("대해 본문")?;
        self.enter_scope();
        self.declare_name(&name);
        let body = match self.parse_body() {
            Ok(body) => body,
            Err(err) => {
                self.exit_scope();
                return Err(err);
            }
        };
        self.exit_scope();
        let mood = self.consume_optional_terminator()?;
        Ok(Stmt::ForEach {
            id: self.next_id(),
            span: s.merge(&self.previous_span()),
            mood,
            item: name,
            item_type,
            iterable,
            body,
        })
    }

    fn parse_quantifier_stmt(&mut self) -> Result<Stmt, ParseError> {
        let s = self.current_span();
        let variable = self.expect_ident("양화 변수")?.raw.clone();
        let token = self.advance();
        let subject = match &token.kind {
            TokenKind::Josa(name) => name.as_str(),
            _ => "",
        };
        if subject != "이" && subject != "가" {
            return Err(ParseError {
                span: self.to_ast_span(token.span),
                message: "양화 머릿말은 '<변수> 이/가 <영역> ...' 형태여야 합니다".to_string(),
            });
        }
        let domain = self.parse_type_ref()?;
        let kind = self.parse_quantifier_kind()?;
        self.expect_colon_or_lbrace("양화 본문")?;
        self.enter_scope();
        self.declare_name(&variable);
        let body = match self.parse_body() {
            Ok(body) => body,
            Err(err) => {
                self.exit_scope();
                return Err(err);
            }
        };
        self.exit_scope();
        self.validate_quantifier_body(&body, s.merge(&self.previous_span()))?;
        let mood = self.consume_optional_terminator()?;
        Ok(Stmt::Quantifier {
            id: self.next_id(),
            span: s.merge(&self.previous_span()),
            mood,
            kind,
            variable,
            domain,
            body,
        })
    }

    fn parse_quantifier_kind(&mut self) -> Result<QuantifierKind, ParseError> {
        if matches!(&self.current().kind, TokenKind::Ident(name) if name == "낱낱") {
            self.advance();
            let token = self.advance();
            if !matches!(token.kind, TokenKind::Josa(ref name) if name == "에") {
                return Err(ParseError {
                    span: self.to_ast_span(token.span),
                    message: "전칭 양화는 '낱낱에 대해' 형태여야 합니다".to_string(),
                });
            }
            self.expect(&TokenKind::KwDaehae, "대해")?;
            return Ok(QuantifierKind::ForAll);
        }
        if matches!(&self.current().kind, TokenKind::Ident(name) if name == "중") {
            self.advance();
            if matches!(&self.current().kind, TokenKind::Ident(name) if name == "딱") {
                self.advance();
                let one = self.expect_ident("하나")?;
                if one.raw != "하나" {
                    return Err(ParseError {
                        span: self.to_ast_span(one.span),
                        message: "유일 양화는 '중 딱 하나가' 형태여야 합니다".to_string(),
                    });
                }
                let subject = self.advance();
                if !matches!(subject.kind, TokenKind::Josa(ref name) if name == "가") {
                    return Err(ParseError {
                        span: self.to_ast_span(subject.span),
                        message: "유일 양화는 '중 딱 하나가' 형태여야 합니다".to_string(),
                    });
                }
                return Ok(QuantifierKind::ExistsUnique);
            }
            let one = self.expect_ident("하나")?;
            if one.raw != "하나" {
                return Err(ParseError {
                    span: self.to_ast_span(one.span),
                    message: "존재 양화는 '중 하나가' 형태여야 합니다".to_string(),
                });
            }
            let subject = self.advance();
            if !matches!(subject.kind, TokenKind::Josa(ref name) if name == "가") {
                return Err(ParseError {
                    span: self.to_ast_span(subject.span),
                    message: "존재 양화는 '중 하나가' 형태여야 합니다".to_string(),
                });
            }
            return Ok(QuantifierKind::Exists);
        }
        Err(ParseError {
            span: self.current_span(),
            message: "양화 머릿말은 '낱낱에 대해' / '중 하나가' / '중 딱 하나가'만 허용됩니다"
                .to_string(),
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_pipe()
    }
    fn parse_pipe(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_logical_or()?;
        if !self.check(&TokenKind::KwHaeseo) {
            return Ok(expr);
        }
        let mut stages = vec![expr];
        while self.check(&TokenKind::KwHaeseo) {
            if let Some(prev) = stages.last() {
                match &prev.kind {
                    ExprKind::Eval {
                        mode: ThunkEvalMode::Pipe,
                        ..
                    } => {}
                    ExprKind::Eval { .. } => {
                        return Err(ParseError {
                            span: prev.span,
                            message: "Gate0: {..}한것/인것/아닌것/하고 해서 표기는 금지입니다. {..}해서를 사용하세요".to_string(),
                        });
                    }
                    ExprKind::Thunk(_) => {
                        return Err(ParseError {
                            span: prev.span,
                            message: "Gate0: 토막 파이프는 {..}해서로 연결해야 합니다".to_string(),
                        });
                    }
                    _ => {}
                }
            }
            self.advance();
            let stage = self.parse_logical_or()?;
            if !matches!(stage.kind, ExprKind::Call { .. }) {
                return Err(ParseError {
                    span: stage.span,
                    message: "PIPE-CALL-ONLY-01: 파이프 단계는 호출식만 허용합니다".to_string(),
                });
            }
            stages.push(stage);
        }
        let span = stages
            .first()
            .expect("pipe stage")
            .span
            .merge(&stages.last().expect("pipe stage").span);
        Ok(Expr::new(self.next_id(), span, ExprKind::Pipe { stages }))
    }
    fn parse_logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut l = self.parse_logical_and()?;
        while self.is_logical_or_op() {
            let op = self.advance().raw.clone();
            let r = self.parse_logical_and()?;
            l = Expr::new(
                self.next_id(),
                l.span.merge(&r.span),
                ExprKind::Infix {
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            );
        }
        Ok(l)
    }
    fn parse_logical_and(&mut self) -> Result<Expr, ParseError> {
        let mut l = self.parse_equality()?;
        while self.is_logical_and_op() {
            let op = self.advance().raw.clone();
            let r = self.parse_equality()?;
            l = Expr::new(
                self.next_id(),
                l.span.merge(&r.span),
                ExprKind::Infix {
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            );
        }
        Ok(l)
    }
    fn is_logical_and_op(&self) -> bool {
        matches!(self.current().kind, TokenKind::And)
            || matches!(&self.current().kind, TokenKind::Ident(name) if name == "그리고")
    }
    fn is_logical_or_op(&self) -> bool {
        matches!(self.current().kind, TokenKind::Or)
            || matches!(&self.current().kind, TokenKind::Ident(name) if name == "또는")
    }
    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut l = self.parse_comparison()?;
        while matches!(self.current().kind, TokenKind::EqEq | TokenKind::NotEq) {
            let op = self.advance().raw.clone();
            let r = self.parse_comparison()?;
            l = Expr::new(
                self.next_id(),
                l.span.merge(&r.span),
                ExprKind::Infix {
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            );
        }
        Ok(l)
    }
    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut l = self.parse_range()?;
        while matches!(
            self.current().kind,
            TokenKind::Lt | TokenKind::LtEq | TokenKind::Gt | TokenKind::GtEq
        ) {
            let op = self.advance().raw.clone();
            let r = self.parse_range()?;
            l = Expr::new(
                self.next_id(),
                l.span.merge(&r.span),
                ExprKind::Infix {
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            );
        }
        Ok(l)
    }
    fn parse_range(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_addition()?;
        if !matches!(self.current().kind, TokenKind::DotDot | TokenKind::DotDotEq) {
            return Ok(left);
        }
        let op = self.advance();
        let inclusive = matches!(op.kind, TokenKind::DotDotEq);
        let right = self.parse_addition()?;
        let span = left.span.merge(&right.span);
        let mut arg_start = self.new_arg_binding(left);
        arg_start.resolved_pin = Some("시작".to_string());
        arg_start.binding_reason = BindingReason::UserFixed;
        let mut arg_end = self.new_arg_binding(right);
        arg_end.resolved_pin = Some("끝".to_string());
        arg_end.binding_reason = BindingReason::UserFixed;
        let flag_value = if inclusive { 1 } else { 0 };
        let flag_expr = Expr::new(
            self.next_id(),
            self.to_ast_span(op.span),
            ExprKind::Literal(Literal::Fixed64(Fixed64::from_i64(flag_value))),
        );
        let mut arg_flag = self.new_arg_binding(flag_expr);
        arg_flag.resolved_pin = Some("끝포함".to_string());
        arg_flag.binding_reason = BindingReason::UserFixed;
        Ok(Expr::new(
            self.next_id(),
            span,
            ExprKind::Call {
                args: vec![arg_start, arg_end, arg_flag],
                func: "표준.범위".to_string(),
            },
        ))
    }
    fn parse_addition(&mut self) -> Result<Expr, ParseError> {
        let mut l = self.parse_multiplication()?;
        while matches!(self.current().kind, TokenKind::Plus | TokenKind::Minus) {
            let op = self.advance().raw.clone();
            let r = self.parse_multiplication()?;
            l = Expr::new(
                self.next_id(),
                l.span.merge(&r.span),
                ExprKind::Infix {
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            );
        }
        Ok(l)
    }
    fn parse_multiplication(&mut self) -> Result<Expr, ParseError> {
        let mut l = self.parse_unary()?;
        while matches!(
            self.current().kind,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent
        ) {
            let op = self.advance().raw.clone();
            let r = self.parse_unary()?;
            l = Expr::new(
                self.next_id(),
                l.span.merge(&r.span),
                ExprKind::Infix {
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            );
        }
        Ok(l)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.check(&TokenKind::Minus) {
            let op_token = self.advance();
            let rhs = self.parse_unary()?;
            let zero = Expr::new(
                self.next_id(),
                self.to_ast_span(op_token.span),
                ExprKind::Literal(Literal::Fixed64(Fixed64::from_i64(0))),
            );
            let span = zero.span.merge(&rhs.span);
            return Ok(Expr::new(
                self.next_id(),
                span,
                ExprKind::Infix {
                    left: Box::new(zero),
                    op: op_token.raw.clone(),
                    right: Box::new(rhs),
                },
            ));
        }
        if self.check(&TokenKind::Plus) {
            self.advance();
            return self.parse_unary();
        }
        self.parse_primary()
    }
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        self.parse_primary_inner(true)
    }
    fn parse_primary_inner(&mut self, allow_bare_bound_call: bool) -> Result<Expr, ParseError> {
        if self.check(&TokenKind::LParen) && self.peek_tagged_template() {
            return self.parse_tagged_template();
        }
        if self.check(&TokenKind::LParen) && self.peek_tagged_formula() {
            return self.parse_tagged_formula();
        }
        let t = self.advance();
        let mut expr = match &t.kind {
            TokenKind::Integer(n) => Expr::new(
                self.next_id(),
                self.to_ast_span(t.span),
                ExprKind::Literal(Literal::Fixed64(Fixed64::from_i64(*n))),
            ),
            TokenKind::Float(raw) => Expr::new(
                self.next_id(),
                self.to_ast_span(t.span),
                ExprKind::Literal(Literal::Fixed64(Fixed64::from_f64_lossy(
                    raw.parse::<f64>().unwrap_or(0.0),
                ))),
            ),
            TokenKind::StringLit(s) => Expr::new(
                self.next_id(),
                self.to_ast_span(t.span),
                ExprKind::Literal(Literal::String(s.clone())),
            ),
            TokenKind::Atom(a) => Expr::new(
                self.next_id(),
                self.to_ast_span(t.span),
                ExprKind::Literal(Literal::Atom(a.clone())),
            ),
            TokenKind::TemplateBlock(raw) => {
                let template = self.parse_template_block(raw, self.to_ast_span(t.span), None)?;
                Expr::new(
                    self.next_id(),
                    self.to_ast_span(t.span),
                    ExprKind::Template(template),
                )
            }
            TokenKind::FormulaBlock(raw) => {
                let formula =
                    self.parse_formula_block(raw, self.to_ast_span(t.span), None, false)?;
                Expr::new(
                    self.next_id(),
                    self.to_ast_span(t.span),
                    ExprKind::Formula(formula),
                )
            }
            TokenKind::AssertionBlock(raw) => {
                let assertion = self.parse_assertion_block(raw, self.to_ast_span(t.span))?;
                Expr::new(
                    self.next_id(),
                    self.to_ast_span(t.span),
                    ExprKind::Assertion(assertion),
                )
            }
            TokenKind::StateMachineBlock(raw) => {
                let machine = self.parse_state_machine_block(raw, self.to_ast_span(t.span))?;
                Expr::new(
                    self.next_id(),
                    self.to_ast_span(t.span),
                    ExprKind::StateMachine(machine),
                )
            }
            TokenKind::Nuance(level) => {
                let inner = self.parse_primary()?;
                Expr::new(
                    self.next_id(),
                    self.to_ast_span(t.span).merge(&inner.span),
                    ExprKind::Nuance {
                        level: level.clone(),
                        expr: Box::new(inner),
                    },
                )
            }
            TokenKind::At => {
                let path = match &self.current().kind {
                    TokenKind::StringLit(path) => path.clone(),
                    _ => return Err(self.error("@\"자원\"")),
                };
                let token = self.advance();
                Expr::new(
                    self.next_id(),
                    self.to_ast_span(t.span)
                        .merge(&self.to_ast_span(token.span)),
                    ExprKind::Literal(Literal::Resource(path)),
                )
            }
            TokenKind::LBrace => {
                if self.check_ident() && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Pipe)) {
                    let param = self.expect_ident("씨앗 인자")?.raw.clone();
                    self.expect(&TokenKind::Pipe, "|")?;
                    let body = self.parse_expr()?;
                    let close = self.expect(&TokenKind::RBrace, "}")?;
                    let span = self
                        .to_ast_span(t.span)
                        .merge(&body.span)
                        .merge(&self.to_ast_span(close.span));
                    Expr::new(
                        self.next_id(),
                        span,
                        ExprKind::SeedLiteral {
                            param,
                            body: Box::new(body),
                        },
                    )
                } else {
                    let body = self.parse_thunk_body_after_lbrace(self.to_ast_span(t.span))?;
                    let mut expr = Expr::new(self.next_id(), body.span, ExprKind::Thunk(body));
                    if let Some(mode) = self.consume_eval_marker(expr.span.end)? {
                        let span = expr.span.merge(&self.previous_span());
                        if let ExprKind::Thunk(body) = &expr.kind {
                            self.validate_eval_body(body, mode, span)?;
                        }
                        expr = Expr::new(
                            self.next_id(),
                            span,
                            ExprKind::Eval {
                                thunk: Box::new(expr),
                                mode,
                            },
                        );
                    } else if self.check_adjacent_kw_haeseo(expr.span.end) {
                        let span = expr.span;
                        if let ExprKind::Thunk(body) = &expr.kind {
                            self.validate_eval_body(body, ThunkEvalMode::Pipe, span)?;
                        }
                        expr = Expr::new(
                            self.next_id(),
                            span,
                            ExprKind::Eval {
                                thunk: Box::new(expr),
                                mode: ThunkEvalMode::Pipe,
                            },
                        );
                    }
                    expr
                }
            }
            TokenKind::LBracket => {
                let mut args = Vec::new();
                if !self.check(&TokenKind::RBracket) {
                    loop {
                        let value = self.parse_expr()?;
                        args.push(self.new_arg_binding(value));
                        if self.check(&TokenKind::Comma) {
                            self.advance();
                            continue;
                        }
                        break;
                    }
                }
                let close = self.expect(&TokenKind::RBracket, "]")?;
                let span = self
                    .to_ast_span(t.span)
                    .merge(&self.to_ast_span(close.span));
                Expr::new(
                    self.next_id(),
                    span,
                    ExprKind::Call {
                        args,
                        func: "차림".to_string(),
                    },
                )
            }
            TokenKind::Ident(n) | TokenKind::Josa(n) => {
                self.validate_ident_token(&t)?;
                if n == "없음" {
                    return Ok(Expr::new(
                        self.next_id(),
                        self.to_ast_span(t.span),
                        ExprKind::Literal(Literal::None),
                    ));
                }
                if n == "정규식" && self.check(&TokenKind::LBrace) {
                    let brace = self.current();
                    if brace.span.start != t.span.end {
                        return Err(self.error("Gate0: 정규식{ 는 붙여쓰기만 허용됩니다"));
                    }
                    let (regex, span) = self.parse_regex_literal_block(self.to_ast_span(t.span))?;
                    return Ok(Expr::new(
                        self.next_id(),
                        span,
                        ExprKind::Literal(Literal::Regex(regex)),
                    ));
                }
                let mut e = Expr::new(
                    self.next_id(),
                    self.to_ast_span(t.span),
                    ExprKind::Var(n.clone()),
                );
                let mut end = t.span.end;
                if n == "글무늬" && self.check(&TokenKind::LBrace) {
                    return Err(self.error("Gate0: 글무늬{ 는 붙여쓰기만 허용됩니다"));
                }
                if n == "수식" && self.check(&TokenKind::LBrace) {
                    return Err(self.error("Gate0: 수식{ 는 붙여쓰기만 허용됩니다"));
                }
                if n == "세움" && self.check(&TokenKind::LBrace) {
                    return Err(self.error("Gate0: 세움{ 는 붙여쓰기만 허용됩니다"));
                }
                if n == "상태머신" && self.check(&TokenKind::LBrace) {
                    return Err(self.error("Gate0: 상태머신{ 는 붙여쓰기만 허용됩니다"));
                }
                while self.check(&TokenKind::Dot) {
                    let dot = self.current();
                    if dot.span.start != end {
                        break;
                    }
                    let Some(next) = self.tokens.get(self.pos + 1) else {
                        break;
                    };
                    if next.span.start != dot.span.end {
                        break;
                    }
                    self.advance();
                    let field_token = self.expect_ident("필드")?;
                    let field = match &field_token.kind {
                        TokenKind::Ident(name) | TokenKind::Josa(name) => name.clone(),
                        _ => field_token.raw.clone(),
                    };
                    let span = e.span.merge(&self.to_ast_span(field_token.span));
                    e = Expr::new(
                        self.next_id(),
                        span,
                        ExprKind::FieldAccess {
                            target: Box::new(e),
                            field,
                        },
                    );
                    end = field_token.span.end;
                }
                if self.check(&TokenKind::LParen) {
                    return Err(self.error(
                        "Gate0: 이름(인자) 호출은 금지입니다. (인자) 이름 형태를 사용하세요",
                    ));
                }
                e
            }
            TokenKind::LParen => {
                let mut args = Vec::new();
                let mut pack_fields = Vec::new();
                let mut saw_pack = false;
                if !self.check(&TokenKind::RParen) {
                    loop {
                        if self.check_ident() && self.peek_is_colon() {
                            saw_pack = true;
                            let name = self.expect_ident("묶음 필드")?.raw.clone();
                            self.expect(&TokenKind::Colon, ":")?;
                            let value = self.parse_expr()?;
                            pack_fields.push((name, value));
                        } else if saw_pack {
                            return Err(self.error("묶음 필드는 이름:값 형태여야 합니다"));
                        } else {
                            let arg = self.parse_call_arg()?;
                            args.push(arg);
                        }
                        if self.check(&TokenKind::Comma) {
                            self.advance();
                            continue;
                        }
                        break;
                    }
                }
                let close = self.expect(&TokenKind::RParen, ")")?;
                let span = self
                    .to_ast_span(t.span)
                    .merge(&self.to_ast_span(close.span));

                if !saw_pack {
                    if self.check_adjacent_in(close.span.end) {
                        self.advance();
                        let mut value_expr = self.parse_primary()?;
                        value_expr = self.parse_index_suffix(value_expr)?;
                        value_expr = self.parse_at_suffix(value_expr)?;
                        let func_token = self.expect_ident("함수")?;
                        let func_name = match &func_token.kind {
                            TokenKind::Ident(name) | TokenKind::Josa(name) => name.clone(),
                            _ => return Err(self.error("함수")),
                        };
                        if !matches!(func_name.as_str(), "채우기" | "풀기" | "살피기") {
                            return Err(ParseError {
                                span: span.merge(&self.to_ast_span(func_token.span)),
                                message: "Gate0: (주입)인 표기는 채우기/풀기/살피기만 허용됩니다"
                                    .to_string(),
                            });
                        }
                        let inject = self.parse_injection_fields(args, span)?;
                        let merged_span = span
                            .merge(&value_expr.span)
                            .merge(&self.to_ast_span(func_token.span));
                        if func_name == "풀기" {
                            if let ExprKind::Formula(formula) = value_expr.kind {
                                return Ok(Expr::new(
                                    self.next_id(),
                                    merged_span,
                                    ExprKind::FormulaEval { formula, inject },
                                ));
                            }
                        }
                        if func_name == "채우기" {
                            if matches!(value_expr.kind, ExprKind::Template(_)) {
                                return Err(ParseError {
                                    span: value_expr.span,
                                    message: "Gate0: 글무늬{...} 채우기는 금지입니다. (<키=값, ...>) 글무늬{...}를 사용하세요".to_string(),
                                });
                            }
                        }
                        if func_name == "살피기" {
                            let pack_expr =
                                Expr::new(self.next_id(), span, ExprKind::Pack { fields: inject });
                            let mut arg_value = self.new_arg_binding(value_expr);
                            arg_value.resolved_pin = Some("세움".to_string());
                            arg_value.binding_reason = BindingReason::UserFixed;
                            let mut arg_pack = self.new_arg_binding(pack_expr);
                            arg_pack.resolved_pin = Some("값들".to_string());
                            arg_pack.binding_reason = BindingReason::UserFixed;
                            return Ok(Expr::new(
                                self.next_id(),
                                merged_span,
                                ExprKind::Call {
                                    args: vec![arg_value, arg_pack],
                                    func: func_name,
                                },
                            ));
                        }
                        let pack_expr =
                            Expr::new(self.next_id(), span, ExprKind::Pack { fields: inject });
                        let mut arg_value = self.new_arg_binding(value_expr);
                        arg_value.resolved_pin = Some(
                            if func_name == "채우기" {
                                "무늬"
                            } else {
                                "식"
                            }
                            .to_string(),
                        );
                        arg_value.binding_reason = BindingReason::UserFixed;
                        let mut arg_pack = self.new_arg_binding(pack_expr);
                        arg_pack.resolved_pin = Some("주입".to_string());
                        arg_pack.binding_reason = BindingReason::UserFixed;
                        return Ok(Expr::new(
                            self.next_id(),
                            merged_span,
                            ExprKind::Call {
                                args: vec![arg_value, arg_pack],
                                func: func_name,
                            },
                        ));
                    }
                    if matches!(self.current().kind, TokenKind::TemplateBlock(_))
                        || (self.check(&TokenKind::LParen) && self.peek_tagged_template())
                    {
                        let (template, template_span) =
                            if matches!(self.current().kind, TokenKind::TemplateBlock(_)) {
                                let token = self.advance();
                                let TokenKind::TemplateBlock(raw) = &token.kind else {
                                    return Err(self.error("글무늬 블록"));
                                };
                                (
                                    self.parse_template_block(
                                        raw,
                                        self.to_ast_span(token.span),
                                        None,
                                    )?,
                                    self.to_ast_span(token.span),
                                )
                            } else {
                                let tagged = self.parse_tagged_template()?;
                                let ExprKind::Template(template) = tagged.kind else {
                                    return Err(self.error("글무늬 블록"));
                                };
                                (template, tagged.span)
                            };
                        let inject = self.parse_injection_fields(args, span)?;
                        let merged_span = span.merge(&template_span);
                        return Ok(Expr::new(
                            self.next_id(),
                            merged_span,
                            ExprKind::TemplateRender { template, inject },
                        ));
                    }
                    if matches!(self.current().kind, TokenKind::FormulaBlock(_))
                        || (self.check(&TokenKind::LParen) && self.peek_tagged_formula())
                    {
                        let (formula, formula_span) =
                            if matches!(self.current().kind, TokenKind::FormulaBlock(_)) {
                                let token = self.advance();
                                let TokenKind::FormulaBlock(raw) = &token.kind else {
                                    return Err(self.error("수식 블록"));
                                };
                                (
                                    self.parse_formula_block(
                                        raw,
                                        self.to_ast_span(token.span),
                                        None,
                                        false,
                                    )?,
                                    self.to_ast_span(token.span),
                                )
                            } else {
                                let tagged = self.parse_tagged_formula()?;
                                let ExprKind::Formula(formula) = tagged.kind else {
                                    return Err(self.error("수식 블록"));
                                };
                                (formula, tagged.span)
                            };
                        let inject = self.parse_injection_fields(args, span)?;
                        let merged_span = span.merge(&formula_span);
                        return Ok(Expr::new(
                            self.next_id(),
                            merged_span,
                            ExprKind::FormulaEval { formula, inject },
                        ));
                    }
                }

                if saw_pack {
                    let pack_expr = Expr::new(
                        self.next_id(),
                        span,
                        ExprKind::Pack {
                            fields: pack_fields,
                        },
                    );
                    if let Some((func_name, func_span)) = self.parse_call_name()? {
                        let arg = ArgBinding {
                            id: self.next_id(),
                            span: pack_expr.span,
                            expr: pack_expr,
                            josa: None,
                            resolved_pin: None,
                            binding_reason: BindingReason::Positional,
                        };
                        return Ok(Expr::new(
                            self.next_id(),
                            span.merge(&func_span),
                            ExprKind::Call {
                                args: vec![arg],
                                func: func_name,
                            },
                        ));
                    }
                    pack_expr
                } else {
                    if let Some((func_name, func_span)) = self.parse_call_name()? {
                        Expr::new(
                            self.next_id(),
                            span.merge(&func_span),
                            ExprKind::Call {
                                args,
                                func: func_name,
                            },
                        )
                    } else {
                        if args.len() == 1
                            && args[0].josa.is_none()
                            && args[0].resolved_pin.is_none()
                        {
                            args.into_iter().next().unwrap().expr
                        } else {
                            return Err(self.error("함수 호출"));
                        }
                    }
                }
            }
            _ => return Err(self.error("표현식")),
        };
        expr = self.parse_index_suffix(expr)?;
        expr = self.parse_at_suffix(expr)?;
        expr = self.parse_not_suffix(expr)?;
        if allow_bare_bound_call {
            return self.try_parse_bare_bound_call(expr);
        }
        Ok(expr)
    }
    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.tokens[self.pos - 1].clone()
    }
    fn is_at_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }
    fn check(&self, k: &TokenKind) -> bool {
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(k)
    }
    fn check_ident(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Ident(_) | TokenKind::Josa(_)
        )
    }
    fn peek_is_colon(&self) -> bool {
        self.tokens
            .get(self.pos + 1)
            .map(|t| matches!(t.kind, TokenKind::Colon))
            .unwrap_or(false)
    }
    fn peek_is_equals(&self) -> bool {
        self.tokens
            .get(self.pos + 1)
            .map(|t| matches!(t.kind, TokenKind::Equals))
            .unwrap_or(false)
    }
    fn peek_kind_n_is(&self, n: usize, f: impl FnOnce(&TokenKind) -> bool) -> bool {
        self.tokens
            .get(self.pos + n)
            .map(|token| f(&token.kind))
            .unwrap_or(false)
    }
    fn check_adjacent_kw_haeseo(&self, end: usize) -> bool {
        self.check(&TokenKind::KwHaeseo) && self.current().span.start == end
    }
    fn check_adjacent_in(&self, end: usize) -> bool {
        matches!(&self.current().kind, TokenKind::Ident(name) if name == "인")
            && self.current().span.start == end
    }

    fn consume_right_arrow(&mut self) -> bool {
        if matches!(self.current().kind, TokenKind::RightArrow) {
            self.advance();
            return true;
        }
        false
    }

    fn consume_arrow(&mut self) -> bool {
        if matches!(self.current().kind, TokenKind::Arrow) {
            self.advance();
            return true;
        }
        if matches!(self.current().kind, TokenKind::Lt) {
            if let Some(next) = self.tokens.get(self.pos + 1) {
                if matches!(next.kind, TokenKind::Minus) {
                    self.advance();
                    self.advance();
                    return true;
                }
            }
        }
        false
    }

    fn build_mutate_stmt_from_assignment(
        &mut self,
        stmt_start: Span,
        target: Expr,
        value: Expr,
        mood: Mood,
        deferred: bool,
        deferred_span: Option<Span>,
    ) -> Result<Stmt, ParseError> {
        if let ExprKind::Call { func, args } = &target.kind {
            if func == "차림.값" && args.len() == 2 {
                let target_expr = args[0].expr.clone();
                let index_expr = args[1].expr.clone();
                if !matches!(target_expr.kind, ExprKind::Var(_)) {
                    return Err(ParseError {
                        span: target_expr.span,
                        message: "차림 인덱스 대입 대상은 이름만 허용됩니다".to_string(),
                    });
                }
                let mut arg_target = self.new_arg_binding(target_expr.clone());
                arg_target.resolved_pin = Some("대상".to_string());
                arg_target.binding_reason = BindingReason::UserFixed;
                let mut arg_index = self.new_arg_binding(index_expr);
                arg_index.resolved_pin = Some("i".to_string());
                arg_index.binding_reason = BindingReason::UserFixed;
                let mut arg_value = self.new_arg_binding(value);
                arg_value.resolved_pin = Some("값".to_string());
                arg_value.binding_reason = BindingReason::UserFixed;
                let value_expr = Expr::new(
                    self.next_id(),
                    target.span.merge(&arg_value.span),
                    ExprKind::Call {
                        args: vec![arg_target, arg_index, arg_value],
                        func: "차림.바꾼값".to_string(),
                    },
                );
                self.ensure_root_declared_for_write(&target_expr)?;
                return Ok(Stmt::Mutate {
                    id: self.next_id(),
                    span: stmt_start.merge(&self.previous_span()),
                    mood,
                    target: target_expr,
                    value: value_expr,
                    deferred,
                    deferred_span,
                });
            }
        }
        self.ensure_root_declared_for_write(&target)?;
        Ok(Stmt::Mutate {
            id: self.next_id(),
            span: stmt_start.merge(&self.previous_span()),
            mood,
            target,
            value,
            deferred,
            deferred_span,
        })
    }

    fn inject_top_level_decl_blocks(
        &mut self,
        items: &mut Vec<TopLevelItem>,
        mut decls: Vec<Stmt>,
    ) -> Result<(), ParseError> {
        let mut target_idx: Option<usize> = None;
        for (idx, item) in items.iter_mut().enumerate() {
            let TopLevelItem::SeedDef(seed) = item;
            if matches!(seed.seed_kind, SeedKind::Umjikssi)
                && (seed.canonical_name == "매마디" || seed.canonical_name == "매틱")
            {
                target_idx = Some(idx);
                break;
            }
        }
        if let Some(idx) = target_idx {
            let TopLevelItem::SeedDef(seed) = &mut items[idx];
            if let Some(body) = seed.body.as_mut() {
                if let Some(first) = decls.first() {
                    let first_span = Self::stmt_span(first);
                    body.span = body.span.merge(&first_span);
                }
                let mut new_stmts = Vec::with_capacity(decls.len() + body.stmts.len());
                new_stmts.append(&mut decls);
                new_stmts.append(&mut body.stmts);
                body.stmts = new_stmts;
            } else {
                let span = decls
                    .first()
                    .map(|stmt| Self::stmt_span(stmt))
                    .unwrap_or_else(|| Span::new(0, 0));
                seed.body = Some(Body {
                    id: self.next_id(),
                    span,
                    stmts: decls,
                });
            }
            return Ok(());
        }

        let span = decls
            .first()
            .map(|stmt| Self::stmt_span(stmt))
            .unwrap_or_else(|| Span::new(0, 0));
        let body = Body {
            id: self.next_id(),
            span,
            stmts: decls,
        };
        let seed = SeedDef {
            id: self.next_id(),
            span,
            canonical_name: "매마디".to_string(),
            seed_kind: SeedKind::Umjikssi,
            params: Vec::new(),
            body: Some(body),
            modifiers: Vec::new(),
        };
        items.push(TopLevelItem::SeedDef(seed));
        Ok(())
    }

    fn stmt_span(stmt: &Stmt) -> Span {
        match stmt {
            Stmt::DeclBlock { span, .. }
            | Stmt::Mutate { span, .. }
            | Stmt::Expr { span, .. }
            | Stmt::Show { span, .. }
            | Stmt::Inspect { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::If { span, .. }
            | Stmt::Try { span, .. }
            | Stmt::Choose { span, .. }
            | Stmt::Repeat { span, .. }
            | Stmt::While { span, .. }
            | Stmt::ForEach { span, .. }
            | Stmt::Quantifier { span, .. }
            | Stmt::Break { span, .. }
            | Stmt::Contract { span, .. }
            | Stmt::Guard { span, .. }
            | Stmt::BeatBlock { span, .. }
            | Stmt::Hook { span, .. }
            | Stmt::HookWhenBecomes { span, .. }
            | Stmt::HookWhile { span, .. }
            | Stmt::MetaBlock { span, .. }
            | Stmt::Pragma { span, .. } => *span,
        }
    }

    fn consume_compound_update(&mut self) -> Option<&'static str> {
        match self.current().kind {
            TokenKind::PlusArrow => {
                self.advance();
                Some("+")
            }
            TokenKind::MinusArrow => {
                self.advance();
                Some("-")
            }
            _ => None,
        }
    }

    fn is_simple_target(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Var(_) => true,
            ExprKind::FieldAccess { target, .. } => self.is_simple_target(target),
            _ => false,
        }
    }
    fn expect(&mut self, k: &TokenKind, m: &str) -> Result<Token, ParseError> {
        if self.check(k) {
            Ok(self.advance())
        } else {
            Err(self.error(m))
        }
    }
    fn expect_ident(&mut self, m: &str) -> Result<Token, ParseError> {
        if matches!(
            self.current().kind,
            TokenKind::Ident(_) | TokenKind::Josa(_)
        ) {
            let token = self.advance();
            self.validate_ident_token(&token)?;
            Ok(token)
        } else {
            Err(self.error(m))
        }
    }

    fn validate_ident_token(&self, token: &Token) -> Result<(), ParseError> {
        let name = match &token.kind {
            TokenKind::Ident(name) | TokenKind::Josa(name) => name.as_str(),
            _ => return Ok(()),
        };
        if let Some(entry) = term_map::find_fatal_term(name) {
            return Err(ParseError {
                span: self.to_ast_span(token.span),
                message: format!(
                    "TERM-LINT-01: 치명 금지어 '{}'({})는 사용할 수 없습니다. 정본 '{}'를 사용하세요",
                    entry.input, entry.code, entry.canonical
                ),
            });
        }
        if term_map::is_josa_only(name) {
            return Err(ParseError {
                span: self.to_ast_span(token.span),
                message: format!(
                    "NAME-LINT-01: 조사 '{}'는 단독 식별자로 사용할 수 없습니다",
                    name
                ),
            });
        }
        if term_map::is_reserved_word(name) {
            return Err(ParseError {
                span: self.to_ast_span(token.span),
                message: format!(
                    "NAME-LINT-01: 예약어 '{}'는 식별자로 사용할 수 없습니다",
                    name
                ),
            });
        }
        Ok(())
    }

    fn parse_call_name(&mut self) -> Result<Option<(String, Span)>, ParseError> {
        if !self.check_ident() {
            return Ok(None);
        }
        if self.should_skip_logical_call_name() {
            return Ok(None);
        }
        let first = self.expect_ident("함수")?;
        let mut name = first.raw.clone();
        let mut span = self.to_ast_span(first.span);
        let mut end = first.span.end;
        while self.check(&TokenKind::Dot) {
            let dot = self.current();
            if dot.span.start != end {
                break;
            }
            let next = match self.tokens.get(self.pos + 1) {
                Some(token) if matches!(token.kind, TokenKind::Ident(_) | TokenKind::Josa(_)) => {
                    token
                }
                _ => break,
            };
            if next.span.start != dot.span.end {
                break;
            }
            self.advance();
            let next = self.expect_ident("함수")?;
            name.push('.');
            name.push_str(&next.raw);
            span = span.merge(&self.to_ast_span(next.span));
            end = next.span.end;
        }
        Ok(Some((name, span)))
    }
    fn should_skip_logical_call_name(&self) -> bool {
        let (TokenKind::Ident(name) | TokenKind::Josa(name)) = &self.current().kind else {
            return false;
        };
        if name != "그리고" && name != "또는" {
            return false;
        }
        self.peek_starts_expr()
    }
    fn peek_starts_expr(&self) -> bool {
        self.tokens
            .get(self.pos + 1)
            .map(|token| self.token_starts_expr(&token.kind))
            .unwrap_or(false)
    }
    fn token_starts_expr(&self, kind: &TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Integer(_)
                | TokenKind::Float(_)
                | TokenKind::StringLit(_)
                | TokenKind::Atom(_)
                | TokenKind::Variable(_)
                | TokenKind::Nuance(_)
                | TokenKind::TemplateBlock(_)
                | TokenKind::FormulaBlock(_)
                | TokenKind::Ident(_)
                | TokenKind::Josa(_)
                | TokenKind::At
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::LParen
                | TokenKind::LBrace
                | TokenKind::LBracket
        )
    }
    fn current_span(&self) -> Span {
        Span {
            start: self.current().span.start,
            end: self.current().span.end,
        }
    }
    fn previous_span(&self) -> Span {
        let s = self.tokens[self.pos - 1].span;
        Span {
            start: s.start,
            end: s.end,
        }
    }
    fn to_ast_span(&self, s: crate::lexer::Span) -> Span {
        Span {
            start: s.start,
            end: s.end,
        }
    }
    fn error(&self, m: &str) -> ParseError {
        ParseError {
            span: self.current_span(),
            message: m.to_string(),
        }
    }

    fn consume_stmt_terminator(&mut self) -> Result<Mood, ParseError> {
        if self.check(&TokenKind::Dot) {
            let mood = self.infer_mood_from_suffix();
            self.advance();
            return Ok(mood);
        }
        if self.check(&TokenKind::Question) {
            self.advance();
            return Ok(Mood::Interrogative);
        }
        if self.check(&TokenKind::Bang) {
            self.advance();
            return Ok(Mood::Exclamative);
        }
        Err(self.error("문장 종결"))
    }

    fn consume_optional_terminator(&mut self) -> Result<Mood, ParseError> {
        if self.check(&TokenKind::Dot)
            || self.check(&TokenKind::Question)
            || self.check(&TokenKind::Bang)
        {
            return self.consume_stmt_terminator();
        }
        Ok(Mood::Declarative)
    }

    fn parse_index_suffix(&mut self, mut expr: Expr) -> Result<Expr, ParseError> {
        loop {
            if !self.check(&TokenKind::LBracket) {
                break;
            }
            self.advance();
            let index_expr = self.parse_expr()?;
            let close = self.expect(&TokenKind::RBracket, "]")?;
            let span = expr.span.merge(&self.to_ast_span(close.span));
            let mut arg_target = self.new_arg_binding(expr);
            arg_target.resolved_pin = Some("대상".to_string());
            arg_target.binding_reason = BindingReason::UserFixed;
            let mut arg_index = self.new_arg_binding(index_expr);
            arg_index.resolved_pin = Some("i".to_string());
            arg_index.binding_reason = BindingReason::UserFixed;
            expr = Expr::new(
                self.next_id(),
                span,
                ExprKind::Call {
                    args: vec![arg_target, arg_index],
                    func: "차림.값".to_string(),
                },
            );
        }
        Ok(expr)
    }

    fn parse_at_suffix(&mut self, mut expr: Expr) -> Result<Expr, ParseError> {
        let mut seen = false;
        loop {
            if !self.check(&TokenKind::At) {
                break;
            }
            let at = self.current().clone();
            if self.to_ast_span(at.span).start != expr.span.end {
                break;
            }
            if seen {
                return Err(self.error("접미 기호(@)는 1회만 허용됩니다"));
            }
            seen = true;
            self.advance();
            let suffix = self.parse_at_payload(at.span.end)?;
            let span = expr.span.merge(&self.previous_span());
            expr = Expr::new(
                self.next_id(),
                span,
                ExprKind::Suffix {
                    value: Box::new(expr),
                    at: suffix,
                },
            );
        }
        Ok(expr)
    }

    fn parse_not_suffix(&mut self, expr: Expr) -> Result<Expr, ParseError> {
        if matches!(&self.current().kind, TokenKind::Ident(name) if name == "아님") {
            let token = self.advance();
            let span = expr.span.merge(&self.to_ast_span(token.span));
            let arg = self.new_arg_binding(expr);
            return Ok(Expr::new(
                self.next_id(),
                span,
                ExprKind::Call {
                    args: vec![arg],
                    func: "아님".to_string(),
                },
            ));
        }
        Ok(expr)
    }

    fn consume_eval_marker(
        &mut self,
        close_end: usize,
    ) -> Result<Option<ThunkEvalMode>, ParseError> {
        if self.current().span.start != close_end {
            return Ok(None);
        }
        let mode = match &self.current().kind {
            TokenKind::Ident(name) => match name.as_str() {
                "한것" => Some(ThunkEvalMode::Value),
                "인것" | "is" => Some(ThunkEvalMode::Bool),
                "아닌것" | "not" => Some(ThunkEvalMode::Not),
                "하고" => Some(ThunkEvalMode::Do),
                "것" => {
                    return Err(ParseError {
                        span: self.current_span(),
                        message: "Gate0: }것은 }한것으로 바꿔주세요".to_string(),
                    })
                }
                _ => None,
            },
            TokenKind::EqEq => Some(ThunkEvalMode::Bool),
            TokenKind::NotEq => Some(ThunkEvalMode::Not),
            _ => None,
        };
        if mode.is_some() {
            self.advance();
        }
        Ok(mode)
    }

    fn ensure_eval_condition(&self, expr: &Expr, label: &str) -> Result<(), ParseError> {
        match &expr.kind {
            ExprKind::Eval {
                mode: ThunkEvalMode::Bool,
                ..
            } => Ok(()),
            _ => Err(ParseError {
                span: expr.span,
                message: format!("{}은 {{...}}인것 형태여야 합니다", label),
            }),
        }
    }

    fn parse_contract_mode(&mut self) -> Result<ContractMode, ParseError> {
        if !self.check(&TokenKind::LParen) {
            return Ok(ContractMode::Abort);
        }
        self.advance();
        let token = self.expect_ident("계약 모드")?;
        let name = match &token.kind {
            TokenKind::Ident(name) | TokenKind::Josa(name) => name.as_str(),
            _ => return Err(self.error("계약 모드")),
        };
        let mode = match name {
            "알림" => ContractMode::Alert,
            "물림" => ContractMode::Abort,
            _ => {
                return Err(ParseError {
                    span: self.to_ast_span(token.span),
                    message: "계약 모드는 알림/물림만 허용됩니다".to_string(),
                });
            }
        };
        self.expect(&TokenKind::RParen, ")")?;
        Ok(mode)
    }

    fn validate_eval_body(
        &self,
        body: &Body,
        mode: ThunkEvalMode,
        span: Span,
    ) -> Result<(), ParseError> {
        let label = match mode {
            ThunkEvalMode::Value => "}한것",
            ThunkEvalMode::Bool => "}인것",
            ThunkEvalMode::Not => "}아닌것",
            ThunkEvalMode::Do => "}하고",
            ThunkEvalMode::Pipe => "}해서",
        };
        if matches!(mode, ThunkEvalMode::Bool | ThunkEvalMode::Not) {
            if self.body_has_mutation(body) {
                return Err(ParseError {
                    span,
                    message: format!("Gate0: {} 안에서는 '<-'를 사용할 수 없습니다", label),
                });
            }
            if self.body_has_eval_do(body) {
                return Err(ParseError {
                    span,
                    message: format!("Gate0: {} 안에서는 '}}하고'를 사용할 수 없습니다", label),
                });
            }
            if self.body_has_random(body) {
                return Err(ParseError {
                    span,
                    message: format!("Gate0: {} 안에서는 무작위 함수를 사용할 수 없습니다", label),
                });
            }
        }
        if matches!(mode, ThunkEvalMode::Value | ThunkEvalMode::Pipe)
            && self.body_has_mutation(body)
        {
            return Err(ParseError {
                span,
                message: format!("Gate0: {} 안에서는 '<-'를 사용할 수 없습니다", label),
            });
        }
        Ok(())
    }

    fn validate_guard_body(&self, body: &Body, span: Span) -> Result<(), ParseError> {
        if self.body_has_mutation(body) {
            return Err(ParseError {
                span,
                message: "Gate0: 늘지켜보고 안에서는 '<-'를 사용할 수 없습니다".to_string(),
            });
        }
        if self.body_has_eval_do(body) {
            return Err(ParseError {
                span,
                message: "Gate0: 늘지켜보고 안에서는 '}하고'를 사용할 수 없습니다".to_string(),
            });
        }
        if self.body_has_random(body) {
            return Err(ParseError {
                span,
                message: "Gate0: 늘지켜보고 안에서는 무작위 함수를 사용할 수 없습니다".to_string(),
            });
        }
        for stmt in &body.stmts {
            match stmt {
                Stmt::Expr { expr, .. } => {
                    if !matches!(expr.kind, ExprKind::Call { .. }) {
                        return Err(ParseError {
                            span: expr.span,
                            message: "Gate0: 늘지켜보고 안에서는 호출식만 허용합니다".to_string(),
                        });
                    }
                }
                _ => {
                    return Err(ParseError {
                        span,
                        message: "Gate0: 늘지켜보고 안에서는 호출식만 허용합니다".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    fn validate_quantifier_body(&self, body: &Body, span: Span) -> Result<(), ParseError> {
        if self.body_has_mutation(body) {
            return Err(ParseError {
                span,
                message: "AGE4: 양화 블록 안에서는 '<-'를 사용할 수 없습니다".to_string(),
            });
        }
        if self.body_has_show(body) {
            return Err(ParseError {
                span,
                message: "AGE4: 양화 블록 안에서는 '보여주기'를 사용할 수 없습니다".to_string(),
            });
        }
        Ok(())
    }

    fn body_has_mutation(&self, body: &Body) -> bool {
        body.stmts.iter().any(|stmt| self.stmt_has_mutation(stmt))
    }

    fn body_has_eval_do(&self, body: &Body) -> bool {
        body.stmts.iter().any(|stmt| self.stmt_has_eval_do(stmt))
    }

    fn body_has_random(&self, body: &Body) -> bool {
        body.stmts.iter().any(|stmt| self.stmt_has_random(stmt))
    }

    fn body_has_show(&self, body: &Body) -> bool {
        body.stmts.iter().any(|stmt| self.stmt_has_show(stmt))
    }

    fn stmt_has_mutation(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_mutation(expr))
            }),
            Stmt::Mutate { .. } => true,
            Stmt::Expr { expr, .. } => self.expr_has_mutation(expr),
            Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => self.expr_has_mutation(expr),
            Stmt::Return { value, .. } => self.expr_has_mutation(value),
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
                        .map_or(false, |body| self.body_has_mutation(body))
            }
            Stmt::Try { action, body, .. } => {
                self.expr_has_mutation(action) || self.body_has_mutation(body)
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                branches.iter().any(|branch| {
                    self.expr_has_mutation(&branch.condition)
                        || self.body_has_mutation(&branch.body)
                }) || self.body_has_mutation(else_body)
            }
            Stmt::Repeat { body, .. } => self.body_has_mutation(body),
            Stmt::BeatBlock { body, .. } => self.body_has_mutation(body),
            Stmt::Hook { body, .. } => self.body_has_mutation(body),
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => self.expr_has_mutation(condition) || self.body_has_mutation(body),
            Stmt::While {
                condition, body, ..
            } => self.expr_has_mutation(condition) || self.body_has_mutation(body),
            Stmt::ForEach { iterable, body, .. } => {
                self.expr_has_mutation(iterable) || self.body_has_mutation(body)
            }
            Stmt::Quantifier { body, .. } => self.body_has_mutation(body),
            Stmt::Break { .. } => false,
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_mutation(condition)
                    || self.body_has_mutation(else_body)
                    || then_body
                        .as_ref()
                        .map_or(false, |body| self.body_has_mutation(body))
            }
            Stmt::Guard {
                condition, body, ..
            } => self.expr_has_mutation(condition) || self.body_has_mutation(body),
            Stmt::MetaBlock { .. } => false,
            Stmt::Pragma { .. } => false,
        }
    }

    fn stmt_has_eval_do(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_eval_do(expr))
            }),
            Stmt::Expr { expr, .. } => self.expr_has_eval_do(expr),
            Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => self.expr_has_eval_do(expr),
            Stmt::Return { value, .. } => self.expr_has_eval_do(value),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_eval_do(condition)
                    || self.body_has_eval_do(then_body)
                    || else_body
                        .as_ref()
                        .map_or(false, |body| self.body_has_eval_do(body))
            }
            Stmt::Try { action, body, .. } => {
                self.expr_has_eval_do(action) || self.body_has_eval_do(body)
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                branches.iter().any(|branch| {
                    self.expr_has_eval_do(&branch.condition) || self.body_has_eval_do(&branch.body)
                }) || self.body_has_eval_do(else_body)
            }
            Stmt::Repeat { body, .. } => self.body_has_eval_do(body),
            Stmt::BeatBlock { body, .. } => self.body_has_eval_do(body),
            Stmt::Hook { body, .. } => self.body_has_eval_do(body),
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => self.expr_has_eval_do(condition) || self.body_has_eval_do(body),
            Stmt::While {
                condition, body, ..
            } => self.expr_has_eval_do(condition) || self.body_has_eval_do(body),
            Stmt::ForEach { iterable, body, .. } => {
                self.expr_has_eval_do(iterable) || self.body_has_eval_do(body)
            }
            Stmt::Quantifier { body, .. } => self.body_has_eval_do(body),
            Stmt::Break { .. } => false,
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_eval_do(condition)
                    || self.body_has_eval_do(else_body)
                    || then_body
                        .as_ref()
                        .map_or(false, |body| self.body_has_eval_do(body))
            }
            Stmt::Guard {
                condition, body, ..
            } => self.expr_has_eval_do(condition) || self.body_has_eval_do(body),
            Stmt::Mutate { .. } => false,
            Stmt::MetaBlock { .. } => false,
            Stmt::Pragma { .. } => false,
        }
    }

    fn stmt_has_random(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_random(expr))
            }),
            Stmt::Expr { expr, .. } => self.expr_has_random(expr),
            Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => self.expr_has_random(expr),
            Stmt::Return { value, .. } => self.expr_has_random(value),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_random(condition)
                    || self.body_has_random(then_body)
                    || else_body
                        .as_ref()
                        .map_or(false, |body| self.body_has_random(body))
            }
            Stmt::Try { action, body, .. } => {
                self.expr_has_random(action) || self.body_has_random(body)
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                branches.iter().any(|branch| {
                    self.expr_has_random(&branch.condition) || self.body_has_random(&branch.body)
                }) || self.body_has_random(else_body)
            }
            Stmt::Repeat { body, .. } => self.body_has_random(body),
            Stmt::BeatBlock { body, .. } => self.body_has_random(body),
            Stmt::Hook { body, .. } => self.body_has_random(body),
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => self.expr_has_random(condition) || self.body_has_random(body),
            Stmt::While {
                condition, body, ..
            } => self.expr_has_random(condition) || self.body_has_random(body),
            Stmt::ForEach { iterable, body, .. } => {
                self.expr_has_random(iterable) || self.body_has_random(body)
            }
            Stmt::Quantifier { body, .. } => self.body_has_random(body),
            Stmt::Break { .. } => false,
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_random(condition)
                    || self.body_has_random(else_body)
                    || then_body
                        .as_ref()
                        .map_or(false, |body| self.body_has_random(body))
            }
            Stmt::Guard {
                condition, body, ..
            } => self.expr_has_random(condition) || self.body_has_random(body),
            Stmt::Mutate { .. } => false,
            Stmt::MetaBlock { .. } => false,
            Stmt::Pragma { .. } => false,
        }
    }

    fn stmt_has_show(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Show { .. } | Stmt::Inspect { .. } => true,
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_show(expr))
            }),
            Stmt::Expr { expr, .. } => self.expr_has_show(expr),
            Stmt::Return { value, .. } => self.expr_has_show(value),
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
                        .map_or(false, |body| self.body_has_show(body))
            }
            Stmt::Try { action, body, .. } => {
                self.expr_has_show(action) || self.body_has_show(body)
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                branches.iter().any(|branch| {
                    self.expr_has_show(&branch.condition) || self.body_has_show(&branch.body)
                }) || self.body_has_show(else_body)
            }
            Stmt::Repeat { body, .. } => self.body_has_show(body),
            Stmt::BeatBlock { body, .. } => self.body_has_show(body),
            Stmt::Hook { body, .. } => self.body_has_show(body),
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => self.expr_has_show(condition) || self.body_has_show(body),
            Stmt::While {
                condition, body, ..
            } => self.expr_has_show(condition) || self.body_has_show(body),
            Stmt::ForEach { iterable, body, .. } => {
                self.expr_has_show(iterable) || self.body_has_show(body)
            }
            Stmt::Quantifier { body, .. } => self.body_has_show(body),
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr_has_show(condition)
                    || self.body_has_show(else_body)
                    || then_body
                        .as_ref()
                        .map_or(false, |body| self.body_has_show(body))
            }
            Stmt::Guard {
                condition, body, ..
            } => self.expr_has_show(condition) || self.body_has_show(body),
            Stmt::Mutate { target, value, .. } => {
                self.expr_has_show(target) || self.expr_has_show(value)
            }
            Stmt::Break { .. } | Stmt::MetaBlock { .. } | Stmt::Pragma { .. } => false,
        }
    }

    fn expr_has_mutation(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Eval { thunk, .. } => self.expr_exec_has_mutation(thunk),
            ExprKind::Thunk(_) => false,
            ExprKind::FieldAccess { target, .. } => self.expr_has_mutation(target),
            ExprKind::SeedLiteral { body, .. } => self.expr_has_mutation(body),
            ExprKind::Call { args, .. } => args.iter().any(|arg| self.expr_has_mutation(&arg.expr)),
            ExprKind::Infix { left, right, .. } => {
                self.expr_has_mutation(left) || self.expr_has_mutation(right)
            }
            ExprKind::Suffix { value, .. } => self.expr_has_mutation(value),
            ExprKind::Pipe { stages } => stages.iter().any(|stage| self.expr_has_mutation(stage)),
            ExprKind::Pack { fields } => {
                fields.iter().any(|(_, expr)| self.expr_has_mutation(expr))
            }
            ExprKind::Assertion(_) => false,
            ExprKind::Formula(_) => false,
            ExprKind::Template(_) => false,
            ExprKind::TemplateRender { inject, .. } => {
                inject.iter().any(|(_, expr)| self.expr_has_mutation(expr))
            }
            ExprKind::FormulaEval { inject, .. } => {
                inject.iter().any(|(_, expr)| self.expr_has_mutation(expr))
            }
            ExprKind::Nuance { expr, .. } => self.expr_has_mutation(expr),
            _ => false,
        }
    }

    fn expr_exec_has_mutation(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Thunk(body) => self.body_has_mutation(body),
            ExprKind::Eval { thunk, .. } => self.expr_exec_has_mutation(thunk),
            _ => self.expr_has_mutation(expr),
        }
    }

    fn expr_has_eval_do(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Eval { thunk, mode } => {
                if matches!(mode, ThunkEvalMode::Do) {
                    return true;
                }
                self.expr_has_eval_do(thunk)
            }
            ExprKind::Thunk(_) => false,
            ExprKind::FieldAccess { target, .. } => self.expr_has_eval_do(target),
            ExprKind::SeedLiteral { body, .. } => self.expr_has_eval_do(body),
            ExprKind::Call { args, .. } => args.iter().any(|arg| self.expr_has_eval_do(&arg.expr)),
            ExprKind::Infix { left, right, .. } => {
                self.expr_has_eval_do(left) || self.expr_has_eval_do(right)
            }
            ExprKind::Suffix { value, .. } => self.expr_has_eval_do(value),
            ExprKind::Pipe { stages } => stages.iter().any(|stage| self.expr_has_eval_do(stage)),
            ExprKind::Pack { fields } => fields.iter().any(|(_, expr)| self.expr_has_eval_do(expr)),
            ExprKind::Assertion(_) => false,
            ExprKind::Formula(_) => false,
            ExprKind::Template(_) => false,
            ExprKind::TemplateRender { inject, .. } => {
                inject.iter().any(|(_, expr)| self.expr_has_eval_do(expr))
            }
            ExprKind::FormulaEval { inject, .. } => {
                inject.iter().any(|(_, expr)| self.expr_has_eval_do(expr))
            }
            ExprKind::Nuance { expr, .. } => self.expr_has_eval_do(expr),
            _ => false,
        }
    }

    fn expr_has_random(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Eval { thunk, .. } => self.expr_exec_has_random(thunk),
            ExprKind::Thunk(_) => false,
            ExprKind::FieldAccess { target, .. } => self.expr_has_random(target),
            ExprKind::SeedLiteral { body, .. } => self.expr_has_random(body),
            ExprKind::Call { func, args } => {
                if is_random_func(func) {
                    return true;
                }
                args.iter().any(|arg| self.expr_has_random(&arg.expr))
            }
            ExprKind::Infix { left, right, .. } => {
                self.expr_has_random(left) || self.expr_has_random(right)
            }
            ExprKind::Suffix { value, .. } => self.expr_has_random(value),
            ExprKind::Pipe { stages } => stages.iter().any(|stage| self.expr_has_random(stage)),
            ExprKind::Pack { fields } => fields.iter().any(|(_, expr)| self.expr_has_random(expr)),
            ExprKind::Assertion(_) => false,
            ExprKind::Formula(_) => false,
            ExprKind::Template(_) => false,
            ExprKind::TemplateRender { inject, .. } => {
                inject.iter().any(|(_, expr)| self.expr_has_random(expr))
            }
            ExprKind::FormulaEval { inject, .. } => {
                inject.iter().any(|(_, expr)| self.expr_has_random(expr))
            }
            ExprKind::Nuance { expr, .. } => self.expr_has_random(expr),
            _ => false,
        }
    }

    fn expr_has_show(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Eval { thunk, .. } => self.expr_exec_has_show(thunk),
            ExprKind::Thunk(_) => false,
            ExprKind::FieldAccess { target, .. } => self.expr_has_show(target),
            ExprKind::SeedLiteral { body, .. } => self.expr_has_show(body),
            ExprKind::Call { args, .. } => args.iter().any(|arg| self.expr_has_show(&arg.expr)),
            ExprKind::Infix { left, right, .. } => {
                self.expr_has_show(left) || self.expr_has_show(right)
            }
            ExprKind::Suffix { value, .. } => self.expr_has_show(value),
            ExprKind::Pipe { stages } => stages.iter().any(|stage| self.expr_has_show(stage)),
            ExprKind::Pack { fields } => fields.iter().any(|(_, expr)| self.expr_has_show(expr)),
            ExprKind::Assertion(_) => false,
            ExprKind::Formula(_) => false,
            ExprKind::Template(_) => false,
            ExprKind::StateMachine(_) => false,
            ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
                inject.iter().any(|(_, expr)| self.expr_has_show(expr))
            }
            ExprKind::Nuance { expr, .. } => self.expr_has_show(expr),
            _ => false,
        }
    }

    fn expr_exec_has_show(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Thunk(body) => self.body_has_show(body),
            ExprKind::Eval { thunk, .. } => self.expr_exec_has_show(thunk),
            _ => self.expr_has_show(expr),
        }
    }

    fn expr_exec_has_random(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Thunk(body) => self.body_has_random(body),
            ExprKind::Eval { thunk, .. } => self.expr_exec_has_random(thunk),
            _ => self.expr_has_random(expr),
        }
    }

    fn parse_at_payload(&mut self, at_end: usize) -> Result<AtSuffix, ParseError> {
        if let TokenKind::StringLit(path) = &self.current().kind {
            let path = path.clone();
            let token = self.advance();
            if token.span.start != at_end {
                return Err(self.error("@ 뒤 자원은 붙여서 써야 합니다"));
            }
            return Ok(AtSuffix::Asset(path));
        }
        let unit = self.parse_unit_chain(at_end)?;
        if !is_known_unit(&unit) {
            return Err(ParseError {
                span: self.current_span(),
                message: format!(
                    "단위 '{}'를 알 수 없습니다. 핀 고정은 값:핀을 사용하세요",
                    unit
                ),
            });
        }
        Ok(AtSuffix::Unit(unit))
    }

    fn parse_unit_chain(&mut self, at_end: usize) -> Result<String, ParseError> {
        let mut out = String::new();
        let mut prev_end = at_end;
        while !self.is_at_end() {
            let token = self.current().clone();
            if token.span.start != prev_end {
                break;
            }
            match &token.kind {
                TokenKind::Ident(name) => out.push_str(name),
                TokenKind::Integer(n) => out.push_str(&n.to_string()),
                TokenKind::Slash => out.push('/'),
                TokenKind::Caret => out.push('^'),
                _ => break,
            }
            prev_end = token.span.end;
            self.advance();
        }
        if out.is_empty() {
            return Err(self.error("@ 뒤 단위 또는 자원"));
        }
        Ok(out)
    }

    fn infer_mood_from_suffix(&self) -> Mood {
        let interrogative = ["까", "니", "냐", "나요", "습니까", "인가", "인가요"];
        let suggestive = ["자", "합시다"];
        let imperative = ["해줘", "주세요", "하세요", "해라", "하라", "줘"];
        let exclamative = ["야", "네", "구나"];

        let mut idx = self.pos;
        while idx > 0 {
            idx -= 1;
            match &self.tokens[idx].kind {
                TokenKind::Ident(raw) => {
                    if interrogative.iter().any(|s| raw.ends_with(s)) {
                        return Mood::Interrogative;
                    }
                    if suggestive.iter().any(|s| raw.ends_with(s)) {
                        return Mood::Suggestive;
                    }
                    if imperative.iter().any(|s| raw.ends_with(s)) {
                        return Mood::Imperative;
                    }
                    if exclamative.iter().any(|s| raw.ends_with(s)) {
                        return Mood::Exclamative;
                    }
                    return Mood::Declarative;
                }
                TokenKind::Dot
                | TokenKind::Question
                | TokenKind::Bang
                | TokenKind::Comma
                | TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::RBrace => {
                    continue;
                }
                _ => return Mood::Declarative,
            }
        }

        Mood::Declarative
    }

    fn parse_arg_suffix(&mut self) -> Result<ArgSuffix, ParseError> {
        let mut fixed_pin = None;
        let mut josa = None;
        if self.check(&TokenKind::Colon) {
            self.advance();
            let pin = self.expect_ident("핀")?.raw.clone();
            fixed_pin = Some(pin);
        }
        if self.check(&TokenKind::Tilde) {
            self.advance();
            if let TokenKind::Josa(value) = &self.current().kind {
                josa = Some(value.clone());
                self.advance();
            } else {
                return Err(self.error("~ 뒤 조사"));
            }
        } else if let TokenKind::Josa(value) = &self.current().kind {
            josa = Some(value.clone());
            self.advance();
        }
        if self.check(&TokenKind::At) {
            return Err(self.error("접미 순서는 값@단위/자원:핀~조사 여야 합니다 (@는 :핀 앞)"));
        }
        if self.check(&TokenKind::Colon) {
            return Err(self.error("접미 순서는 값@단위/자원:핀~조사 여야 합니다 (:핀은 ~조사 앞)"));
        }
        if self.check(&TokenKind::Tilde) {
            return Err(self.error("접미 순서는 값@단위/자원:핀~조사 여야 합니다 (~조사는 맨 끝)"));
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
            binding_reason: reason,
            fixed_pin,
        })
    }

    fn try_parse_arg_suffix(&mut self) -> Result<Option<ArgSuffix>, ParseError> {
        let colon_starts_pin = self.check(&TokenKind::Colon)
            && self
                .tokens
                .get(self.pos + 1)
                .map(|token| matches!(token.kind, TokenKind::Ident(_) | TokenKind::Josa(_)))
                .unwrap_or(false);
        if !(colon_starts_pin
            || self.check(&TokenKind::Tilde)
            || matches!(self.current().kind, TokenKind::Josa(_)))
        {
            return Ok(None);
        }
        self.parse_arg_suffix().map(Some)
    }

    fn bind_arg_with_suffix(&mut self, expr: Expr, suffix: ArgSuffix) -> ArgBinding {
        ArgBinding {
            id: self.next_id(),
            span: expr.span,
            expr,
            josa: suffix.josa,
            resolved_pin: suffix.fixed_pin,
            binding_reason: suffix.binding_reason,
        }
    }

    fn try_parse_bare_bound_call(&mut self, first_expr: Expr) -> Result<Expr, ParseError> {
        let checkpoint = self.pos;
        let Some(first_suffix) = self.try_parse_arg_suffix()? else {
            return Ok(first_expr);
        };

        let mut args = vec![self.bind_arg_with_suffix(first_expr.clone(), first_suffix)];
        loop {
            let arg_checkpoint = self.pos;
            if !self.token_starts_expr(&self.current().kind) {
                break;
            }
            let expr = self.parse_primary_inner(false)?;
            let Some(suffix) = self.try_parse_arg_suffix()? else {
                self.pos = arg_checkpoint;
                break;
            };
            args.push(self.bind_arg_with_suffix(expr, suffix));
        }

        if let Some((func_name, func_span)) = self.parse_call_name()? {
            let span = args
                .first()
                .map(|arg| arg.span)
                .unwrap_or(first_expr.span)
                .merge(&func_span);
            return Ok(Expr::new(
                self.next_id(),
                span,
                ExprKind::Call {
                    args,
                    func: func_name,
                },
            ));
        }

        self.pos = checkpoint;
        Ok(first_expr)
    }

    fn parse_call_arg(&mut self) -> Result<ArgBinding, ParseError> {
        if self.check_ident() && self.peek_is_equals() {
            let pin = self.expect_ident("핀")?.raw.clone();
            self.expect(&TokenKind::Equals, "=")?;
            let arg_expr = self.parse_expr()?;
            let suffix = self.parse_arg_suffix()?;
            return Ok(ArgBinding {
                id: self.next_id(),
                span: arg_expr.span,
                expr: arg_expr,
                josa: suffix.josa,
                resolved_pin: Some(pin),
                binding_reason: BindingReason::UserFixed,
            });
        }
        let arg_expr = self.parse_expr()?;
        let suffix = self.parse_arg_suffix()?;
        Ok(ArgBinding {
            id: self.next_id(),
            span: arg_expr.span,
            expr: arg_expr,
            josa: suffix.josa,
            resolved_pin: suffix.fixed_pin,
            binding_reason: suffix.binding_reason,
        })
    }

    fn parse_injection_fields(
        &mut self,
        args: Vec<ArgBinding>,
        span: Span,
    ) -> Result<Vec<(String, Expr)>, ParseError> {
        let mut fields = Vec::new();
        let mut seen = HashSet::new();
        for arg in args {
            if arg.josa.is_some() {
                return Err(ParseError {
                    span,
                    message: "주입은 이름=값 형태만 허용됩니다".to_string(),
                });
            }
            let Some(pin) = arg.resolved_pin else {
                return Err(ParseError {
                    span,
                    message: "주입은 이름=값 형태만 허용됩니다".to_string(),
                });
            };
            if !matches!(arg.binding_reason, BindingReason::UserFixed) {
                return Err(ParseError {
                    span,
                    message: "주입은 이름=값 형태만 허용됩니다".to_string(),
                });
            }
            if seen.contains(&pin) {
                return Err(ParseError {
                    span,
                    message: format!("주입 키 '{}'가 중복되었습니다", pin),
                });
            }
            seen.insert(pin.clone());
            fields.push((pin, arg.expr));
        }
        Ok(fields)
    }

    fn peek_tagged_template(&self) -> bool {
        let Some(t0) = self.tokens.get(self.pos) else {
            return false;
        };
        if !matches!(t0.kind, TokenKind::LParen) {
            return false;
        }
        let Some(t1) = self.tokens.get(self.pos + 1) else {
            return false;
        };
        if !matches!(t1.kind, TokenKind::Atom(_)) {
            return false;
        }
        let Some(t2) = self.tokens.get(self.pos + 2) else {
            return false;
        };
        if !matches!(t2.kind, TokenKind::RParen) {
            return false;
        }
        let Some(t3) = self.tokens.get(self.pos + 3) else {
            return false;
        };
        matches!(t3.kind, TokenKind::TemplateBlock(_))
    }

    fn parse_tagged_template(&mut self) -> Result<Expr, ParseError> {
        let lparen = self.expect(&TokenKind::LParen, "(")?;
        let tag_token = self.advance();
        let tag = match tag_token.kind {
            TokenKind::Atom(tag) => tag,
            _ => {
                return Err(ParseError {
                    span: self.to_ast_span(tag_token.span),
                    message: "글무늬 태그는 (#tag) 형태여야 합니다".to_string(),
                })
            }
        };
        self.expect(&TokenKind::RParen, ")")?;
        let template_token = self.advance();
        let TokenKind::TemplateBlock(raw) = &template_token.kind else {
            return Err(ParseError {
                span: self.to_ast_span(template_token.span),
                message: "글무늬 블록이 필요합니다".to_string(),
            });
        };
        let span = self
            .to_ast_span(lparen.span)
            .merge(&self.to_ast_span(template_token.span));
        let template = self.parse_template_block(raw, span, Some(tag))?;
        Ok(Expr::new(
            self.next_id(),
            span,
            ExprKind::Template(template),
        ))
    }

    fn peek_tagged_formula(&self) -> bool {
        let Some(t0) = self.tokens.get(self.pos) else {
            return false;
        };
        if !matches!(t0.kind, TokenKind::LParen) {
            return false;
        }
        let Some(t1) = self.tokens.get(self.pos + 1) else {
            return false;
        };
        if !matches!(t1.kind, TokenKind::Atom(_)) {
            return false;
        }
        let Some(t2) = self.tokens.get(self.pos + 2) else {
            return false;
        };
        if !matches!(t2.kind, TokenKind::RParen) {
            return false;
        }
        let Some(t3) = self.tokens.get(self.pos + 3) else {
            return false;
        };
        matches!(t3.kind, TokenKind::FormulaBlock(_))
    }

    fn parse_tagged_formula(&mut self) -> Result<Expr, ParseError> {
        let lparen = self.expect(&TokenKind::LParen, "(")?;
        let tag_token = self.advance();
        let tag = match tag_token.kind {
            TokenKind::Atom(tag) => tag,
            _ => {
                return Err(ParseError {
                    span: self.to_ast_span(tag_token.span),
                    message: "수식 태그는 (#tag) 형태여야 합니다".to_string(),
                })
            }
        };
        self.expect(&TokenKind::RParen, ")")?;
        let formula_token = self.advance();
        let TokenKind::FormulaBlock(raw) = &formula_token.kind else {
            return Err(ParseError {
                span: self.to_ast_span(formula_token.span),
                message: "수식 블록이 필요합니다".to_string(),
            });
        };
        let span = self
            .to_ast_span(lparen.span)
            .merge(&self.to_ast_span(formula_token.span));
        let formula = self.parse_formula_block(raw, span, Some(tag), true)?;
        Ok(Expr::new(self.next_id(), span, ExprKind::Formula(formula)))
    }

    fn parse_regex_literal_block(
        &mut self,
        ident_span: Span,
    ) -> Result<(RegexLiteral, Span), ParseError> {
        let open = self.expect(&TokenKind::LBrace, "{")?;
        let pattern_token = self.current().clone();
        let pattern = match &pattern_token.kind {
            TokenKind::StringLit(value) => value.clone(),
            _ => {
                return Err(ParseError {
                    span: self.to_ast_span(pattern_token.span),
                    message: "정규식 본문은 첫째 인자로 문자열 패턴이 필요합니다".to_string(),
                })
            }
        };
        self.advance();

        let mut flags = String::new();
        if self.check(&TokenKind::Comma) {
            self.advance();
            let flags_token = self.current().clone();
            flags = match &flags_token.kind {
                TokenKind::StringLit(value) => normalize_regex_flags(value),
                _ => {
                    return Err(ParseError {
                        span: self.to_ast_span(flags_token.span),
                        message: "정규식 둘째 인자는 문자열 깃발이어야 합니다".to_string(),
                    })
                }
            };
            self.advance();
        }

        if self.check(&TokenKind::Comma) {
            return Err(ParseError {
                span: self.current_span(),
                message: "정규식은 패턴과 깃발만 허용됩니다".to_string(),
            });
        }

        let close = self.expect(&TokenKind::RBrace, "}")?;
        let span = ident_span
            .merge(&self.to_ast_span(open.span))
            .merge(&self.to_ast_span(close.span));
        Ok((RegexLiteral { pattern, flags }, span))
    }

    fn parse_template_block(
        &self,
        raw: &str,
        span: Span,
        tag: Option<String>,
    ) -> Result<Template, ParseError> {
        let body = self.decode_template_body(raw, span)?;
        let parts = self.parse_template_parts(&body, span)?;
        Ok(Template {
            raw: body,
            parts,
            tag,
        })
    }

    fn parse_formula_block(
        &self,
        raw: &str,
        span: Span,
        tag: Option<String>,
        explicit_tag: bool,
    ) -> Result<Formula, ParseError> {
        let body = self.normalize_formula_body(raw);
        let dialect = match tag.as_deref() {
            Some("ascii") => FormulaDialect::Ascii,
            Some("ascii1") => FormulaDialect::Ascii1,
            Some("latex") => FormulaDialect::Latex,
            Some(other) => FormulaDialect::Other(other.to_string()),
            None => FormulaDialect::Ascii,
        };
        if matches!(dialect, FormulaDialect::Ascii | FormulaDialect::Ascii1)
            && !formula_body_is_ascii(&body)
        {
            return Err(ParseError {
                span,
                message: "FATAL:FORMULA_BODY_NONASCII".to_string(),
            });
        }
        Ok(Formula {
            raw: body,
            dialect,
            explicit_tag,
        })
    }

    fn parse_state_machine_block(&self, raw: &str, span: Span) -> Result<StateMachine, ParseError> {
        let clauses = split_state_machine_clauses(raw);
        let mut states = Vec::new();
        let mut initial: Option<String> = None;
        let mut transitions = Vec::new();
        let mut checks = Vec::new();

        for clause in clauses {
            if let Some(prefix) = clause.strip_suffix("으로 이뤄짐") {
                if !states.is_empty() {
                    return Err(ParseError {
                        span,
                        message: "상태머신: 상태 선언은 한 번만 허용됩니다".to_string(),
                    });
                }
                states = prefix
                    .split(',')
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(ToString::to_string)
                    .collect();
                if states.is_empty() {
                    return Err(ParseError {
                        span,
                        message: "상태머신: 하나 이상의 상태가 필요합니다".to_string(),
                    });
                }
                continue;
            }
            if let Some(prefix) = clause.strip_suffix("으로 시작") {
                let name = prefix.trim();
                if name.is_empty() {
                    return Err(ParseError {
                        span,
                        message: "상태머신: 시작 상태 이름이 필요합니다".to_string(),
                    });
                }
                initial = Some(name.to_string());
                continue;
            }
            if let Some(rest) = clause.strip_prefix("바뀔때마다 ") {
                let Some(check) = rest.strip_suffix(" 살피기") else {
                    return Err(ParseError {
                        span,
                        message: "상태머신: 바뀔때마다 절은 '<세움값> 살피기'만 허용됩니다"
                            .to_string(),
                    });
                };
                let check = check.trim();
                if check.is_empty() {
                    return Err(ParseError {
                        span,
                        message: "상태머신: 바뀔때마다 절에는 세움값 이름이 필요합니다"
                            .to_string(),
                    });
                }
                checks.push(check.to_string());
                continue;
            }
            if let Some((from, rhs)) = clause.split_once(" 에서 ") {
                if let Some((to, guard_name, action_name)) =
                    parse_state_machine_transition_clause(rhs)
                {
                    let from = from.trim();
                    let to = to.trim();
                    if from.is_empty() || to.is_empty() {
                        return Err(ParseError {
                            span,
                            message: "상태머신: 전이에는 출발/도착 상태가 모두 필요합니다"
                                .to_string(),
                        });
                    }
                    transitions.push(StateTransition {
                        from: from.to_string(),
                        to: to.to_string(),
                        guard_name,
                        action_name,
                    });
                    continue;
                }
            }
            return Err(ParseError {
                span,
                message: format!("상태머신: 해석할 수 없는 절입니다: {}", clause),
            });
        }

        if states.is_empty() {
            return Err(ParseError {
                span,
                message: "상태머신: '으로 이뤄짐' 절이 필요합니다".to_string(),
            });
        }
        let initial = initial.ok_or_else(|| ParseError {
            span,
            message: "상태머신: '으로 시작' 절이 필요합니다".to_string(),
        })?;
        if !states.iter().any(|state| state == &initial) {
            return Err(ParseError {
                span,
                message: format!("E_STATE_UNDECLARED: {}", initial),
            });
        }
        for transition in &transitions {
            if !states.iter().any(|state| state == &transition.from) {
                return Err(ParseError {
                    span,
                    message: format!("E_STATE_UNDECLARED: {}", transition.from),
                });
            }
            if !states.iter().any(|state| state == &transition.to) {
                return Err(ParseError {
                    span,
                    message: format!("E_STATE_UNDECLARED: {}", transition.to),
                });
            }
        }

        Ok(StateMachine {
            states,
            initial,
            transitions,
            on_transition_checks: checks,
        })
    }

    fn parse_assertion_block(&self, raw: &str, span: Span) -> Result<Assertion, ParseError> {
        let body_source = raw.replace("\r\n", "\n").replace('\r', "\n");
        let wrapped = format!("검사:셈씨 = {{{}}}", body_source);
        let tokens = Lexer::new(&wrapped).tokenize().map_err(|err| ParseError {
            span,
            message: err.message,
        })?;
        let mut parser = Parser::new_with_mode(tokens, ParseMode::Strict);
        let program = parser
            .parse_program(wrapped, "#assertion".to_string())
            .map_err(|err| ParseError {
                span,
                message: err.message,
            })?;
        let mut normalizer = Normalizer::new(NormalizationLevel::N1);
        let normalized = normalizer.normalize_program(&program);
        let open = normalized.find('{').ok_or_else(|| ParseError {
            span,
            message: "세움: 본문 시작을 찾을 수 없습니다".to_string(),
        })?;
        let close = normalized.rfind('}').ok_or_else(|| ParseError {
            span,
            message: "세움: 본문 끝을 찾을 수 없습니다".to_string(),
        })?;
        let canon = format!("세움{}", &normalized[open..=close]);
        Ok(Assertion { body_source, canon })
    }

    fn normalize_formula_body(&self, raw: &str) -> String {
        raw.replace("\r\n", "\n").replace('\r', "\n")
    }

    fn decode_template_body(&self, raw: &str, span: Span) -> Result<String, ParseError> {
        let trimmed = raw.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            return self.unescape_string_literal(trimmed, span);
        }
        Ok(self.normalize_raw_template(raw))
    }

    fn unescape_string_literal(&self, raw: &str, span: Span) -> Result<String, ParseError> {
        let mut out = String::new();
        let mut chars = raw.chars();
        let Some('"') = chars.next() else {
            return Err(ParseError {
                span,
                message: "글무늬 문자열은 \"...\" 형태여야 합니다".to_string(),
            });
        };
        let mut escaped = false;
        while let Some(ch) = chars.next() {
            if escaped {
                let mapped = match ch {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '"' => '"',
                    '\\' => '\\',
                    other => other,
                };
                out.push(mapped);
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                if chars.next().is_some() {
                    return Err(ParseError {
                        span,
                        message: "글무늬 문자열 뒤에는 추가 문자가 올 수 없습니다".to_string(),
                    });
                }
                return Ok(out);
            }
            out.push(ch);
        }
        Err(ParseError {
            span,
            message: "글무늬 문자열이 닫히지 않았습니다".to_string(),
        })
    }

    fn normalize_raw_template(&self, raw: &str) -> String {
        let mut text = raw.replace("\r\n", "\n").replace('\r', "\n");
        if text.starts_with('\n') {
            text.remove(0);
        }
        if text.ends_with('\n') {
            text.pop();
        }
        let mut min_indent: Option<usize> = None;
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let indent = line
                .chars()
                .take_while(|ch| matches!(ch, ' ' | '\t'))
                .count();
            min_indent = Some(min_indent.map_or(indent, |min| min.min(indent)));
        }
        let indent = min_indent.unwrap_or(0);
        let mut dedented = String::new();
        let mut lines = text.lines().peekable();
        while let Some(line) = lines.next() {
            let mut idx = 0usize;
            let mut removed = 0usize;
            for (i, ch) in line.char_indices() {
                if removed < indent && matches!(ch, ' ' | '\t') {
                    removed += 1;
                    idx = i + ch.len_utf8();
                } else {
                    break;
                }
            }
            dedented.push_str(&line[idx..]);
            if lines.peek().is_some() {
                dedented.push('\n');
            }
        }
        let mut out = String::new();
        let mut lines = dedented.split('\n').peekable();
        while let Some(line) = lines.next() {
            if line.ends_with('\\') {
                out.push_str(&line[..line.len() - 1]);
                continue;
            }
            out.push_str(line);
            if lines.peek().is_some() {
                out.push('\n');
            }
        }
        out
    }

    fn parse_template_parts(
        &self,
        body: &str,
        span: Span,
    ) -> Result<Vec<TemplatePart>, ParseError> {
        let mut parts = Vec::new();
        let mut buf = String::new();
        let mut chars = body.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                if matches!(chars.peek(), Some('{')) {
                    chars.next();
                    buf.push('{');
                    continue;
                }
                if !buf.is_empty() {
                    parts.push(TemplatePart::Text(std::mem::take(&mut buf)));
                }
                let mut placeholder = String::new();
                let mut closed = false;
                while let Some(pc) = chars.next() {
                    if pc == '}' {
                        closed = true;
                        break;
                    }
                    if pc == '{' {
                        return Err(ParseError {
                            span,
                            message: "글무늬 자리표시자 안에는 '{'를 사용할 수 없습니다"
                                .to_string(),
                        });
                    }
                    placeholder.push(pc);
                }
                if !closed {
                    return Err(ParseError {
                        span,
                        message: "글무늬 자리표시자가 닫히지 않았습니다".to_string(),
                    });
                }
                let placeholder = placeholder.trim();
                if placeholder.is_empty() {
                    return Err(ParseError {
                        span,
                        message: "글무늬 자리표시자 키가 비었습니다".to_string(),
                    });
                }
                let (path, format) = if let Some((left, right)) = placeholder.split_once('|') {
                    let format_raw = right.trim();
                    if !format_raw.starts_with('@') {
                        return Err(ParseError {
                            span,
                            message: "글무늬 자리표시자 포맷은 @로 시작해야 합니다".to_string(),
                        });
                    }
                    let format = self.parse_template_format(&format_raw[1..], span)?;
                    (left.trim(), Some(format))
                } else {
                    (placeholder, None)
                };
                let path = self.parse_template_path(path, span)?;
                parts.push(TemplatePart::Placeholder(TemplatePlaceholder {
                    path,
                    format,
                }));
                continue;
            }
            if ch == '}' {
                if matches!(chars.peek(), Some('}')) {
                    chars.next();
                    buf.push('}');
                    continue;
                }
                return Err(ParseError {
                    span,
                    message: "글무늬 본문에 '}'가 잘못 쓰였습니다".to_string(),
                });
            }
            buf.push(ch);
        }
        if !buf.is_empty() {
            parts.push(TemplatePart::Text(buf));
        }
        Ok(parts)
    }

    fn parse_template_path(&self, raw: &str, span: Span) -> Result<Vec<String>, ParseError> {
        let mut out = Vec::new();
        for seg in raw.split('.') {
            if seg.is_empty() {
                return Err(ParseError {
                    span,
                    message: "글무늬 자리표시자 경로가 비어 있습니다".to_string(),
                });
            }
            if !seg.chars().all(|c| matches!(c, '가'..='힣' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')) {
                return Err(ParseError { span, message: "글무늬 자리표시자 키에는 식별자 문자를 사용해야 합니다".to_string() });
            }
            out.push(seg.to_string());
        }
        Ok(out)
    }

    fn parse_template_format(&self, raw: &str, span: Span) -> Result<TemplateFormat, ParseError> {
        if raw.is_empty() {
            return Err(ParseError {
                span,
                message: "글무늬 포맷이 비었습니다".to_string(),
            });
        }
        if let Some(rest) = raw.strip_prefix('.') {
            let mut digits = String::new();
            let mut chars = rest.chars();
            while let Some(ch) = chars.next() {
                if ch.is_ascii_digit() {
                    digits.push(ch);
                } else {
                    let unit: String = std::iter::once(ch).chain(chars).collect();
                    let precision = digits.parse::<u8>().map_err(|_| ParseError {
                        span,
                        message: "글무늬 포맷 소수 자릿수가 올바르지 않습니다".to_string(),
                    })?;
                    if precision > 9 {
                        return Err(ParseError {
                            span,
                            message: "글무늬 포맷 소수 자릿수는 0..9 범위입니다".to_string(),
                        });
                    }
                    return Ok(TemplateFormat {
                        raw: format!("@{}", raw),
                        width: None,
                        zero_pad: false,
                        precision: Some(precision),
                        unit: Some(unit),
                    });
                }
            }
            if digits.is_empty() {
                return Err(ParseError {
                    span,
                    message: "글무늬 포맷 소수 자릿수가 비었습니다".to_string(),
                });
            }
            let precision = digits.parse::<u8>().map_err(|_| ParseError {
                span,
                message: "글무늬 포맷 소수 자릿수가 올바르지 않습니다".to_string(),
            })?;
            if precision > 9 {
                return Err(ParseError {
                    span,
                    message: "글무늬 포맷 소수 자릿수는 0..9 범위입니다".to_string(),
                });
            }
            return Ok(TemplateFormat {
                raw: format!("@{}", raw),
                width: None,
                zero_pad: false,
                precision: Some(precision),
                unit: None,
            });
        }
        let mut zero_pad = false;
        let mut digits = raw;
        if let Some(rest) = raw.strip_prefix('0') {
            zero_pad = true;
            digits = rest;
        }
        if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
            return Err(ParseError {
                span,
                message: "글무늬 포맷 폭은 숫자여야 합니다".to_string(),
            });
        }
        let width = digits.parse::<usize>().map_err(|_| ParseError {
            span,
            message: "글무늬 포맷 폭이 올바르지 않습니다".to_string(),
        })?;
        if width < 1 || width > 99 {
            return Err(ParseError {
                span,
                message: "글무늬 포맷 폭은 1..99 범위입니다".to_string(),
            });
        }
        Ok(TemplateFormat {
            raw: format!("@{}", raw),
            width: Some(width),
            zero_pad,
            precision: None,
            unit: None,
        })
    }

    fn validate_seed_name_conflicts(&self, program: &CanonProgram) -> Result<(), ParseError> {
        let mut names: HashMap<String, Span> = HashMap::new();
        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            names.insert(seed.canonical_name.clone(), seed.span);
        }
        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            let name = &seed.canonical_name;
            if let Some(base) = name.strip_suffix("하") {
                if !base.is_empty() && names.contains_key(base) {
                    return Err(ParseError {
                        span: seed.span,
                        message: format!(
                            "E_SEED_NAME_CONFLICT_HA: '{}'와 '{}'를 동시에 정의할 수 없습니다",
                            base, name
                        ),
                    });
                }
            } else {
                let ha_name = format!("{}하", name);
                if names.contains_key(&ha_name) {
                    return Err(ParseError {
                        span: seed.span,
                        message: format!(
                            "E_SEED_NAME_CONFLICT_HA: '{}'와 '{}'를 동시에 정의할 수 없습니다",
                            name, ha_name
                        ),
                    });
                }
            }
        }
        Ok(())
    }

    fn validate_seed_name_tail(&self, name: &str, span: Span) -> Result<(), ParseError> {
        let tails = ["기", "하기", "고", "하고", "면", "하면", "면서", "하면서"];
        if tails.iter().any(|tail| name.ends_with(tail)) {
            return Err(ParseError {
                span,
                message: format!("Gate0: 정의 이름에는 꼬리({})를 붙일 수 없습니다", name),
            });
        }
        Ok(())
    }

    fn apply_default_args(&mut self, program: &mut CanonProgram) -> Result<(), ParseError> {
        let mut signatures = HashMap::new();
        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            signatures.insert(seed.canonical_name.clone(), seed.params.clone());
        }
        let mut known_seeds: HashSet<String> = signatures.keys().cloned().collect();
        for sig in minimal_stdlib_sigs() {
            known_seeds.insert(sig.name.to_string());
        }

        for item in &mut program.items {
            let TopLevelItem::SeedDef(seed) = item;
            if let Some(body) = &mut seed.body {
                self.apply_defaults_in_body(body, &signatures, &known_seeds)?;
            }
        }

        Ok(())
    }

    fn validate_units(&self, program: &CanonProgram) -> Result<(), ParseError> {
        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            if let Some(body) = &seed.body {
                self.validate_body_units(body)?;
            }
        }
        Ok(())
    }

    fn validate_body_units(&self, body: &Body) -> Result<(), ParseError> {
        for stmt in &body.stmts {
            self.validate_stmt_units(stmt)?;
        }
        Ok(())
    }

    fn validate_stmt_units(&self, stmt: &Stmt) -> Result<(), ParseError> {
        match stmt {
            Stmt::DeclBlock { items, .. } => {
                for item in items {
                    if let Some(value) = &item.value {
                        self.infer_expr_dim(value)?;
                    }
                }
                Ok(())
            }
            Stmt::Mutate { target, value, .. } => {
                let target_dim = self.infer_expr_dim(target)?;
                let value_dim = self.infer_expr_dim(value)?;
                if let (DimState::Known(left), DimState::Known(right)) = (target_dim, value_dim) {
                    if left != right {
                        return Err(self.unit_mismatch_error(value.span, left, right));
                    }
                }
                Ok(())
            }
            Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                self.infer_expr_dim(expr)?;
                Ok(())
            }
            Stmt::Return { value, .. } => {
                self.infer_expr_dim(value)?;
                Ok(())
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.infer_expr_dim(condition)?;
                self.validate_body_units(then_body)?;
                if let Some(body) = else_body {
                    self.validate_body_units(body)?;
                }
                Ok(())
            }
            Stmt::Try { action, body, .. } => {
                self.infer_expr_dim(action)?;
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    self.infer_expr_dim(&branch.condition)?;
                    self.validate_body_units(&branch.body)?;
                }
                self.validate_body_units(else_body)?;
                Ok(())
            }
            Stmt::Repeat { body, .. } => {
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::BeatBlock { body, .. } => {
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::Hook { body, .. } => {
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => {
                self.infer_expr_dim(condition)?;
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.infer_expr_dim(condition)?;
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::ForEach { iterable, body, .. } => {
                self.infer_expr_dim(iterable)?;
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::Quantifier { body, .. } => {
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::Break { .. } => Ok(()),
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.infer_expr_dim(condition)?;
                if let Some(body) = then_body {
                    self.validate_body_units(body)?;
                }
                self.validate_body_units(else_body)?;
                Ok(())
            }
            Stmt::Guard {
                condition, body, ..
            } => {
                self.infer_expr_dim(condition)?;
                self.validate_body_units(body)?;
                Ok(())
            }
            Stmt::MetaBlock { .. } => Ok(()),
            Stmt::Pragma { .. } => Ok(()),
        }
    }

    fn infer_expr_dim(&self, expr: &Expr) -> Result<DimState, ParseError> {
        match &expr.kind {
            ExprKind::Literal(lit) => match lit {
                Literal::Int(_) | Literal::Fixed64(_) => Ok(DimState::Known(UnitDim::NONE)),
                _ => Ok(DimState::Unknown),
            },
            ExprKind::Var(_) => Ok(DimState::Unknown),
            ExprKind::FieldAccess { target, .. } => {
                self.infer_expr_dim(target)?;
                Ok(DimState::Unknown)
            }
            ExprKind::Call { args, .. } => {
                for arg in args {
                    self.infer_expr_dim(&arg.expr)?;
                }
                Ok(DimState::Unknown)
            }
            ExprKind::Thunk(body) => {
                self.validate_body_units(body)?;
                Ok(DimState::Unknown)
            }
            ExprKind::Eval { thunk, mode } => {
                self.infer_expr_dim(thunk)?;
                if matches!(mode, ThunkEvalMode::Bool | ThunkEvalMode::Not) {
                    Ok(DimState::Known(UnitDim::NONE))
                } else {
                    Ok(DimState::Unknown)
                }
            }
            ExprKind::Pipe { stages } => {
                for stage in stages {
                    self.infer_expr_dim(stage)?;
                }
                Ok(DimState::Unknown)
            }
            ExprKind::FlowValue => Ok(DimState::Unknown),
            ExprKind::Pack { fields } => {
                for (_, value) in fields {
                    self.infer_expr_dim(value)?;
                }
                Ok(DimState::Unknown)
            }
            ExprKind::Assertion(_) => Ok(DimState::Unknown),
            ExprKind::Formula(_) => Ok(DimState::Unknown),
            ExprKind::Template(_) => Ok(DimState::Unknown),
            ExprKind::StateMachine(_) => Ok(DimState::Unknown),
            ExprKind::TemplateRender { inject, .. } => {
                for (_, value) in inject {
                    self.infer_expr_dim(value)?;
                }
                Ok(DimState::Unknown)
            }
            ExprKind::FormulaEval { inject, .. } => {
                for (_, value) in inject {
                    self.infer_expr_dim(value)?;
                }
                Ok(DimState::Unknown)
            }
            ExprKind::SeedLiteral { body, .. } => {
                self.infer_expr_dim(body)?;
                Ok(DimState::Unknown)
            }
            ExprKind::Nuance { expr, .. } => self.infer_expr_dim(expr),
            ExprKind::Suffix { value, at } => match at {
                AtSuffix::Unit(unit) => {
                    let base_dim = self.infer_expr_dim(value)?;
                    let Some(spec) = unit_spec_from_symbol(unit) else {
                        return Ok(DimState::Unknown);
                    };
                    if let DimState::Known(dim) = base_dim {
                        if dim != UnitDim::NONE && dim != spec.dim {
                            return Err(self.unit_mismatch_error(expr.span, dim, spec.dim));
                        }
                    }
                    Ok(DimState::Known(spec.dim))
                }
                AtSuffix::Asset(_) => {
                    self.infer_expr_dim(value)?;
                    Ok(DimState::Unknown)
                }
            },
            ExprKind::Infix { left, op, right } => {
                let left_dim = self.infer_expr_dim(left)?;
                let right_dim = self.infer_expr_dim(right)?;
                match op.as_str() {
                    "+" | "-" | "%" => self.combine_add_sub(expr.span, left_dim, right_dim),
                    "*" => self.combine_mul(expr.span, left_dim, right_dim),
                    "/" => self.combine_div(expr.span, left_dim, right_dim),
                    "==" | "!=" | "<" | "<=" | ">" | ">=" => {
                        if let (DimState::Known(left), DimState::Known(right)) =
                            (left_dim, right_dim)
                        {
                            if left != right {
                                return Err(self.unit_mismatch_error(expr.span, left, right));
                            }
                        }
                        Ok(DimState::Known(UnitDim::NONE))
                    }
                    _ => Ok(DimState::Unknown),
                }
            }
        }
    }

    fn combine_add_sub(
        &self,
        span: Span,
        left_dim: DimState,
        right_dim: DimState,
    ) -> Result<DimState, ParseError> {
        if let (DimState::Known(left), DimState::Known(right)) = (left_dim, right_dim) {
            if left != right {
                return Err(self.unit_mismatch_error(span, left, right));
            }
            return Ok(DimState::Known(left));
        }
        Ok(DimState::Unknown)
    }

    fn combine_mul(
        &self,
        _span: Span,
        left_dim: DimState,
        right_dim: DimState,
    ) -> Result<DimState, ParseError> {
        if let (DimState::Known(left), DimState::Known(right)) = (left_dim, right_dim) {
            return Ok(DimState::Known(left.add(right)));
        }
        Ok(DimState::Unknown)
    }

    fn combine_div(
        &self,
        _span: Span,
        left_dim: DimState,
        right_dim: DimState,
    ) -> Result<DimState, ParseError> {
        if let (DimState::Known(left), DimState::Known(right)) = (left_dim, right_dim) {
            return Ok(DimState::Known(left.sub(right)));
        }
        Ok(DimState::Unknown)
    }

    fn unit_mismatch_error(&self, span: Span, left: UnitDim, right: UnitDim) -> ParseError {
        ParseError {
            span,
            message: format!(
                "단위 차원이 다릅니다: {} vs {}",
                left.format(),
                right.format()
            ),
        }
    }

    fn apply_defaults_in_body(
        &mut self,
        body: &mut Body,
        signatures: &HashMap<String, Vec<ParamPin>>,
        known_seeds: &HashSet<String>,
    ) -> Result<(), ParseError> {
        for stmt in &mut body.stmts {
            match stmt {
                Stmt::DeclBlock { items, .. } => {
                    for item in items {
                        if let Some(value) = &mut item.value {
                            self.apply_defaults_in_expr(value, signatures, known_seeds)?;
                        }
                    }
                }
                Stmt::Mutate { target, value, .. } => {
                    self.apply_defaults_in_expr(target, signatures, known_seeds)?;
                    self.apply_defaults_in_expr(value, signatures, known_seeds)?;
                }
                Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                    self.apply_defaults_in_expr(expr, signatures, known_seeds)?
                }
                Stmt::Return { value, .. } => {
                    self.apply_defaults_in_expr(value, signatures, known_seeds)?
                }
                Stmt::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    self.apply_defaults_in_expr(condition, signatures, known_seeds)?;
                    self.apply_defaults_in_body(then_body, signatures, known_seeds)?;
                    if let Some(body) = else_body {
                        self.apply_defaults_in_body(body, signatures, known_seeds)?;
                    }
                }
                Stmt::Try { action, body, .. } => {
                    self.apply_defaults_in_expr(action, signatures, known_seeds)?;
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::Choose {
                    branches,
                    else_body,
                    ..
                } => {
                    for branch in branches.iter_mut() {
                        self.apply_defaults_in_expr(
                            &mut branch.condition,
                            signatures,
                            known_seeds,
                        )?;
                        self.apply_defaults_in_body(&mut branch.body, signatures, known_seeds)?;
                    }
                    self.apply_defaults_in_body(else_body, signatures, known_seeds)?;
                }
                Stmt::Repeat { body, .. } => {
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::BeatBlock { body, .. } => {
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::Hook { body, .. } => {
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::HookWhenBecomes {
                    condition, body, ..
                }
                | Stmt::HookWhile {
                    condition, body, ..
                } => {
                    self.apply_defaults_in_expr(condition, signatures, known_seeds)?;
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::While {
                    condition, body, ..
                } => {
                    self.apply_defaults_in_expr(condition, signatures, known_seeds)?;
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::ForEach { iterable, body, .. } => {
                    self.apply_defaults_in_expr(iterable, signatures, known_seeds)?;
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::Quantifier { body, .. } => {
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::Break { .. } => {}
                Stmt::Contract {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    self.apply_defaults_in_expr(condition, signatures, known_seeds)?;
                    if let Some(body) = then_body {
                        self.apply_defaults_in_body(body, signatures, known_seeds)?;
                    }
                    self.apply_defaults_in_body(else_body, signatures, known_seeds)?;
                }
                Stmt::Guard {
                    condition, body, ..
                } => {
                    self.apply_defaults_in_expr(condition, signatures, known_seeds)?;
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::MetaBlock { .. } => {}
                Stmt::Pragma { .. } => {}
            }
        }
        Ok(())
    }

    fn apply_defaults_in_expr(
        &mut self,
        expr: &mut Expr,
        signatures: &HashMap<String, Vec<ParamPin>>,
        known_seeds: &HashSet<String>,
    ) -> Result<(), ParseError> {
        match &mut expr.kind {
            ExprKind::Call { args, func } => {
                for arg in args.iter_mut() {
                    self.apply_defaults_in_expr(&mut arg.expr, signatures, known_seeds)?;
                }
                let func_name = func.clone();
                let resolved = self.resolve_call_target(&func_name, known_seeds, expr.span)?;
                if *func != resolved.1 {
                    *func = resolved.1;
                }
                if self.is_transform_call(&resolved.0) {
                    self.normalize_transform_call(args, &resolved.0, expr.span)?;
                } else {
                    self.apply_defaults_to_call(args, &resolved.0, expr.span, signatures)?;
                }
            }
            ExprKind::FieldAccess { target, .. } => {
                self.apply_defaults_in_expr(target, signatures, known_seeds)?;
            }
            ExprKind::Infix { left, right, .. } => {
                self.apply_defaults_in_expr(left, signatures, known_seeds)?;
                self.apply_defaults_in_expr(right, signatures, known_seeds)?;
            }
            ExprKind::Suffix { value, .. } => {
                self.apply_defaults_in_expr(value, signatures, known_seeds)?;
            }
            ExprKind::Thunk(body) => {
                self.apply_defaults_in_body(body, signatures, known_seeds)?;
            }
            ExprKind::Eval { thunk, .. } => {
                self.apply_defaults_in_expr(thunk, signatures, known_seeds)?;
            }
            ExprKind::Pipe { stages } => {
                self.apply_defaults_in_pipe(stages, signatures, known_seeds)?;
            }
            ExprKind::Pack { fields } => {
                for (_, value) in fields.iter_mut() {
                    self.apply_defaults_in_expr(value, signatures, known_seeds)?;
                }
            }
            ExprKind::TemplateRender { inject, .. } => {
                for (_, value) in inject.iter_mut() {
                    self.apply_defaults_in_expr(value, signatures, known_seeds)?;
                }
            }
            ExprKind::FormulaEval { inject, .. } => {
                for (_, value) in inject.iter_mut() {
                    self.apply_defaults_in_expr(value, signatures, known_seeds)?;
                }
            }
            ExprKind::Nuance { expr, .. } => {
                self.apply_defaults_in_expr(expr, signatures, known_seeds)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn apply_defaults_in_pipe(
        &mut self,
        stages: &mut Vec<Expr>,
        signatures: &HashMap<String, Vec<ParamPin>>,
        known_seeds: &HashSet<String>,
    ) -> Result<(), ParseError> {
        for index in 0..stages.len() {
            let prev_is_template = if index > 0 {
                matches!(stages[index - 1].kind, ExprKind::Template(_))
            } else {
                false
            };
            let stage = &mut stages[index];
            match &mut stage.kind {
                ExprKind::Call { args, func } => {
                    for arg in args.iter_mut() {
                        self.apply_defaults_in_expr(&mut arg.expr, signatures, known_seeds)?;
                    }
                    let func_name = func.clone();
                    let resolved = self.resolve_call_target(&func_name, known_seeds, stage.span)?;
                    if *func != resolved.1 {
                        *func = resolved.1;
                    }
                    if index > 0 && resolved.0 == "채우기" && prev_is_template {
                        return Err(ParseError {
                            span: stage.span,
                            message: "Gate0: 글무늬{...} 해서 (키=값, ...) 채우기는 금지입니다. (<키=값, ...>) 글무늬{...}를 사용하세요".to_string(),
                        });
                    }
                    if index > 0 {
                        if let Some(params) = signatures.get(&resolved.0) {
                            self.inject_flow_into_call(args, params, stage.span)?;
                        } else if self.is_transform_call(&resolved.0) {
                            self.inject_flow_into_transform(args, &resolved.0, stage.span)?;
                        }
                    }
                    if self.is_transform_call(&resolved.0) {
                        self.normalize_transform_call(args, &resolved.0, stage.span)?;
                    } else {
                        self.apply_defaults_to_call(args, &resolved.0, stage.span, signatures)?;
                    }
                }
                _ => {
                    self.apply_defaults_in_expr(stage, signatures, known_seeds)?;
                }
            }
        }
        Ok(())
    }

    fn inject_flow_into_call(
        &mut self,
        args: &mut Vec<ArgBinding>,
        params: &[ParamPin],
        span: Span,
    ) -> Result<(), ParseError> {
        if args
            .iter()
            .any(|arg| matches!(arg.binding_reason, BindingReason::FlowInjected))
        {
            return Ok(());
        }
        let used = self.collect_used_pins_for_flow(args, params, span)?;
        let mut object_candidates = Vec::new();
        for (idx, param) in params.iter().enumerate() {
            if used[idx] {
                continue;
            }
            if param.josa_list.iter().any(|j| j == "을" || j == "를") {
                object_candidates.push(idx);
            }
        }
        let target = if object_candidates.len() == 1 {
            Some(object_candidates[0])
        } else if object_candidates.len() > 1 {
            return Err(ParseError {
                span,
                message: "PIPE-FLOW-INJECT-AMBIGUOUS: 흐름값 주입 대상이 모호합니다".to_string(),
            });
        } else {
            let mut missing_required = Vec::new();
            for (idx, param) in params.iter().enumerate() {
                if used[idx] {
                    continue;
                }
                if !param.optional && param.default_value.is_none() {
                    missing_required.push(idx);
                }
            }
            if missing_required.len() == 1 {
                Some(missing_required[0])
            } else if missing_required.len() > 1 {
                return Err(ParseError {
                    span,
                    message: "PIPE-FLOW-INJECT-AMBIGUOUS: 흐름값 주입 대상이 모호합니다"
                        .to_string(),
                });
            } else {
                None
            }
        };

        if let Some(idx) = target {
            let expr = Expr::new(self.next_id(), span, ExprKind::FlowValue);
            args.push(ArgBinding {
                id: self.next_id(),
                span,
                expr,
                josa: None,
                resolved_pin: Some(params[idx].pin_name.clone()),
                binding_reason: BindingReason::FlowInjected,
            });
        }
        Ok(())
    }

    fn is_transform_call(&self, func: &str) -> bool {
        matches!(func, "채우기" | "풀기")
    }

    fn inject_flow_into_transform(
        &mut self,
        args: &mut Vec<ArgBinding>,
        func: &str,
        span: Span,
    ) -> Result<(), ParseError> {
        if args
            .iter()
            .any(|arg| matches!(arg.binding_reason, BindingReason::FlowInjected))
        {
            return Ok(());
        }
        let expr = Expr::new(self.next_id(), span, ExprKind::FlowValue);
        args.insert(
            0,
            ArgBinding {
                id: self.next_id(),
                span,
                expr,
                josa: None,
                resolved_pin: if func == "채우기" {
                    Some("무늬".to_string())
                } else {
                    Some("식".to_string())
                },
                binding_reason: BindingReason::FlowInjected,
            },
        );
        Ok(())
    }

    fn normalize_transform_call(
        &mut self,
        args: &mut Vec<ArgBinding>,
        func: &str,
        span: Span,
    ) -> Result<(), ParseError> {
        let mut value_expr: Option<Expr> = None;
        let mut pack_expr: Option<Expr> = None;
        let mut pack_fields: Vec<(String, Expr)> = Vec::new();
        let mut seen_keys = HashSet::new();

        for arg in args.drain(..) {
            if matches!(arg.binding_reason, BindingReason::FlowInjected) {
                if value_expr.is_some() {
                    return Err(ParseError {
                        span,
                        message: format!("{}: 흐름값이 중복되었습니다", func),
                    });
                }
                value_expr = Some(arg.expr);
                continue;
            }
            if let Some(pin) = &arg.resolved_pin {
                if pin == "무늬" || pin == "식" {
                    if value_expr.is_some() {
                        return Err(ParseError {
                            span,
                            message: format!("{}: 값 인자가 중복되었습니다", func),
                        });
                    }
                    value_expr = Some(arg.expr);
                    continue;
                }
                if pin == "주입" {
                    if pack_expr.is_some() || !pack_fields.is_empty() {
                        return Err(ParseError {
                            span,
                            message: format!("{}: 주입은 하나만 허용됩니다", func),
                        });
                    }
                    pack_expr = Some(arg.expr);
                    continue;
                }
                if seen_keys.contains(pin) {
                    return Err(ParseError {
                        span,
                        message: format!("{}: 키 '{}'가 중복되었습니다", func, pin),
                    });
                }
                seen_keys.insert(pin.clone());
                pack_fields.push((pin.clone(), arg.expr));
                continue;
            }
            if value_expr.is_none() {
                value_expr = Some(arg.expr);
                continue;
            }
            return Err(ParseError {
                span,
                message: format!("{}: 인자는 값:핀 또는 파이프 값만 허용됩니다", func),
            });
        }

        let Some(value_expr) = value_expr else {
            return Err(ParseError {
                span,
                message: format!("{}: 값 인자가 필요합니다", func),
            });
        };
        if func == "채우기" && matches!(value_expr.kind, ExprKind::Template(_)) {
            return Err(ParseError {
                span: value_expr.span,
                message: "Gate0: 글무늬{...} 채우기는 금지입니다. (<키=값, ...>) 글무늬{...}를 사용하세요".to_string(),
            });
        }
        let pack_expr = if let Some(pack_expr) = pack_expr {
            pack_expr
        } else {
            Expr::new(
                self.next_id(),
                span,
                ExprKind::Pack {
                    fields: pack_fields,
                },
            )
        };

        let mut value_arg = self.new_arg_binding(value_expr);
        value_arg.resolved_pin = Some(if func == "채우기" { "무늬" } else { "식" }.to_string());
        let mut pack_arg = self.new_arg_binding(pack_expr);
        pack_arg.resolved_pin = Some("주입".to_string());
        args.push(value_arg);
        args.push(pack_arg);
        Ok(())
    }

    fn collect_used_pins_for_flow(
        &self,
        args: &[ArgBinding],
        params: &[ParamPin],
        span: Span,
    ) -> Result<Vec<bool>, ParseError> {
        let mut used = vec![false; params.len()];
        let mut positional = Vec::new();

        for arg in args {
            if let Some(fixed_pin) = &arg.resolved_pin {
                let idx = params
                    .iter()
                    .position(|p| p.pin_name == *fixed_pin)
                    .ok_or_else(|| ParseError {
                        span,
                        message: format!("핀 '{}'을(를) 찾을 수 없습니다", fixed_pin),
                    })?;
                if used[idx] {
                    return Err(ParseError {
                        span,
                        message: format!("핀 '{}' 인자가 중복되었습니다", params[idx].pin_name),
                    });
                }
                if let Some(josa) = &arg.josa {
                    if !params[idx].josa_list.iter().any(|j| j == josa) {
                        return Err(ParseError {
                            span,
                            message: format!(
                                "핀 '{}'에 조사 '{}'를 사용할 수 없습니다",
                                params[idx].pin_name, josa
                            ),
                        });
                    }
                }
                used[idx] = true;
                continue;
            }
            if let Some(josa) = &arg.josa {
                let candidates: Vec<usize> = params
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.josa_list.iter().any(|j| j == josa))
                    .map(|(i, _)| i)
                    .collect();
                if candidates.is_empty() {
                    return Err(ParseError {
                        span,
                        message: format!("조사 '{}'에 해당하는 핀이 없습니다", josa),
                    });
                }
                if candidates.len() > 1 {
                    return Err(ParseError {
                        span,
                        message: format!("조사 '{}'가 모호합니다: 값:핀으로 고정하세요", josa),
                    });
                }
                let idx = candidates[0];
                if used[idx] {
                    return Err(ParseError {
                        span,
                        message: format!("핀 '{}' 인자가 중복되었습니다", params[idx].pin_name),
                    });
                }
                used[idx] = true;
                continue;
            }
            positional.push(arg);
        }

        let mut pos_iter = positional.into_iter();
        for used_slot in used.iter_mut() {
            if *used_slot {
                continue;
            }
            if pos_iter.next().is_some() {
                *used_slot = true;
            } else {
                break;
            }
        }
        Ok(used)
    }

    fn resolve_call_target(
        &self,
        func: &str,
        known_seeds: &HashSet<String>,
        span: Span,
    ) -> Result<(String, String), ParseError> {
        if known_seeds.contains(func) {
            return Ok((func.to_string(), func.to_string()));
        }
        // '-서' 꼬리는 금지 (단, '면서'/'하면서'는 허용 꼬리)
        if func.ends_with("서") && !func.ends_with("면서") && !func.ends_with("하면서") {
            return Err(ParseError {
                span,
                message:
                    "Gate0: '-서' 꼬리는 문법 오류입니다. '~기 해서' 또는 '~하기 해서'를 사용하세요"
                        .to_string(),
            });
        }
        let tail_pairs = [
            ("기", "하기"),
            ("고", "하고"),
            ("면", "하면"),
            ("면서", "하면서"),
        ];
        for (short_tail, long_tail) in tail_pairs {
            if func.ends_with(long_tail) {
                return self.resolve_call_tail_candidates(
                    func,
                    short_tail,
                    Some(long_tail),
                    known_seeds,
                    span,
                );
            }
        }
        for (short_tail, _) in tail_pairs {
            if func.ends_with(short_tail) {
                return self.resolve_call_tail_candidates(
                    func,
                    short_tail,
                    None,
                    known_seeds,
                    span,
                );
            }
        }
        Ok((func.to_string(), func.to_string()))
    }

    fn resolve_call_tail_candidates(
        &self,
        func: &str,
        short_tail: &str,
        long_tail: Option<&str>,
        known_seeds: &HashSet<String>,
        span: Span,
    ) -> Result<(String, String), ParseError> {
        let mut candidates = Vec::new();
        if let Some(long_tail) = long_tail {
            if let Some(stem) = func.strip_suffix(long_tail) {
                if !stem.is_empty() {
                    candidates.push(stem);
                }
            }
        }
        if let Some(stem) = func.strip_suffix(short_tail) {
            if !stem.is_empty() {
                candidates.push(stem);
            }
        }
        candidates.sort();
        candidates.dedup();
        let mut matches: Vec<&str> = candidates
            .into_iter()
            .filter(|stem| known_seeds.contains(*stem))
            .collect();
        if matches.is_empty() {
            return Err(ParseError {
                span,
                message: format!("E_CALL_TAIL_NO_SEED: '{}'에 대응하는 씨앗이 없습니다", func),
            });
        }
        if matches.len() > 1 {
            matches.sort();
            return Err(ParseError {
                span,
                message: format!(
                    "E_CALL_TAIL_AMBIGUOUS: '{}'는 다음 씨앗 중 모호합니다: {}",
                    func,
                    matches.join(", ")
                ),
            });
        }
        let stem = matches[0];
        let canonical_call = format!("{}{}", stem, short_tail);
        Ok((stem.to_string(), canonical_call))
    }

    fn apply_defaults_to_call(
        &mut self,
        args: &mut Vec<ArgBinding>,
        func: &str,
        span: Span,
        signatures: &HashMap<String, Vec<ParamPin>>,
    ) -> Result<(), ParseError> {
        let Some(params) = signatures.get(func) else {
            return Ok(());
        };
        let mut bound = vec![None; params.len()];
        let mut used = vec![false; params.len()];
        let mut positional = Vec::new();

        for mut arg in args.drain(..) {
            if let Some(fixed_pin) = &arg.resolved_pin {
                let idx = params
                    .iter()
                    .position(|p| p.pin_name == *fixed_pin)
                    .ok_or_else(|| ParseError {
                        span,
                        message: format!("핀 '{}'을 찾을 수 없습니다", fixed_pin),
                    })?;
                if used[idx] {
                    return Err(ParseError {
                        span,
                        message: format!("핀 '{}'에 인자가 중복되었습니다", params[idx].pin_name),
                    });
                }
                if let Some(josa) = &arg.josa {
                    if !params[idx].josa_list.iter().any(|j| j == josa) {
                        return Err(ParseError {
                            span,
                            message: format!(
                                "핀 '{}'에 조사 '{}'는 허용되지 않습니다",
                                params[idx].pin_name, josa
                            ),
                        });
                    }
                }
                if !matches!(arg.binding_reason, BindingReason::FlowInjected) {
                    arg.binding_reason = BindingReason::UserFixed;
                }
                bound[idx] = Some(arg);
                used[idx] = true;
                continue;
            }
            if let Some(josa) = arg.josa.clone() {
                let candidates: Vec<usize> = params
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.josa_list.iter().any(|j| j == &josa))
                    .map(|(i, _)| i)
                    .collect();
                if candidates.is_empty() {
                    let mut arg = arg;
                    let _ = self.merge_josa_into_expr(&mut arg.expr, &josa);
                    arg.josa = None;
                    if !matches!(arg.binding_reason, BindingReason::FlowInjected) {
                        arg.binding_reason = BindingReason::Positional;
                    }
                    positional.push(arg);
                    continue;
                }
                if candidates.len() > 1 {
                    return Err(ParseError {
                        span,
                        message: format!(
                            "조사 '{}'가 모호합니다. 값:핀 또는 ~조사로 고정하세요",
                            josa
                        ),
                    });
                }
                let idx = candidates[0];
                if used[idx] {
                    return Err(ParseError {
                        span,
                        message: format!("핀 '{}'에 인자가 중복되었습니다", params[idx].pin_name),
                    });
                }
                let mut arg = arg;
                arg.resolved_pin = Some(params[idx].pin_name.clone());
                arg.binding_reason = if matches!(arg.binding_reason, BindingReason::UserFixed) {
                    BindingReason::UserFixed
                } else {
                    BindingReason::Dictionary
                };
                bound[idx] = Some(arg);
                used[idx] = true;
            } else {
                positional.push(arg);
            }
        }

        let mut pos_iter = positional.into_iter();
        for (idx, slot) in bound.iter_mut().enumerate() {
            if slot.is_some() {
                continue;
            }
            if let Some(mut arg) = pos_iter.next() {
                arg.resolved_pin = Some(params[idx].pin_name.clone());
                arg.binding_reason = BindingReason::Positional;
                *slot = Some(arg);
                continue;
            }
            if let Some(default_value) = &params[idx].default_value {
                let mut arg = self.new_arg_binding(default_value.clone());
                arg.resolved_pin = Some(params[idx].pin_name.clone());
                *slot = Some(arg);
                continue;
            }
            if params[idx].optional {
                let expr = Expr::new(self.next_id(), span, ExprKind::Literal(Literal::None));
                let mut arg = self.new_arg_binding(expr);
                arg.resolved_pin = Some(params[idx].pin_name.clone());
                *slot = Some(arg);
                continue;
            }
            return Err(ParseError {
                span,
                message: format!("필수 인자 '{}' 누락", params[idx].pin_name),
            });
        }

        if pos_iter.next().is_some() {
            return Err(ParseError {
                span,
                message: "인자 수가 너무 많습니다".to_string(),
            });
        }

        *args = bound.into_iter().flatten().collect();
        Ok(())
    }

    fn merge_josa_into_expr(&self, expr: &mut Expr, josa: &str) -> bool {
        match &mut expr.kind {
            ExprKind::Var(name) => {
                name.push_str(josa);
                true
            }
            ExprKind::FieldAccess { field, .. } => {
                field.push_str(josa);
                true
            }
            _ => false,
        }
    }

    fn new_arg_binding(&mut self, expr: Expr) -> ArgBinding {
        ArgBinding {
            id: self.next_id(),
            span: expr.span,
            expr,
            josa: None,
            resolved_pin: None,
            binding_reason: BindingReason::Positional,
        }
    }
}

fn normalize_regex_flags(raw: &str) -> String {
    let mut seen_i = false;
    let mut seen_m = false;
    let mut seen_s = false;
    for ch in raw.chars() {
        match ch {
            'i' if !seen_i => seen_i = true,
            'm' if !seen_m => seen_m = true,
            's' if !seen_s => seen_s = true,
            _ => return raw.to_string(),
        }
    }
    let mut out = String::new();
    if seen_i {
        out.push('i');
    }
    if seen_m {
        out.push('m');
    }
    if seen_s {
        out.push('s');
    }
    out
}

fn is_random_func(name: &str) -> bool {
    matches!(name, "무작위" | "무작위정수" | "무작위선택")
}

fn legacy_seed_kind_replacement(name: &str) -> Option<&'static str> {
    match name {
        "값함수" => Some("셈씨"),
        "일묶음씨" => Some("갈래씨"),
        _ => None,
    }
}

fn append_meta_entry_token(out: &mut String, token: &Token) {
    if token.raw.is_empty() {
        return;
    }
    let no_space_before = matches!(
        token.kind,
        TokenKind::Colon
            | TokenKind::Comma
            | TokenKind::RParen
            | TokenKind::RBracket
            | TokenKind::RBrace
            | TokenKind::Dot
            | TokenKind::DotDot
            | TokenKind::DotDotEq
    );
    let no_space_after_prev =
        out.ends_with('(') || out.ends_with('[') || out.ends_with('{') || out.ends_with(':');
    if !out.is_empty() && !no_space_before && !no_space_after_prev {
        out.push(' ');
    }
    out.push_str(&token.raw);
}

fn formula_body_is_ascii(body: &str) -> bool {
    body.chars().all(|ch| match ch {
        '\n' | '\t' => true,
        _ => matches!(ch as u32, 0x20..=0x7E),
    })
}

fn split_state_machine_clauses(raw: &str) -> Vec<String> {
    let mut clauses = Vec::new();
    let mut start = 0usize;
    let chars: Vec<char> = raw.chars().collect();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        if let Some(active) = quote {
            if ch == active && (i == 0 || chars[i - 1] != '\\') {
                quote = None;
            }
            i += 1;
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            '.' if depth == 0 => {
                let clause: String = chars[start..i].iter().collect();
                let trimmed = clause.trim();
                if !trimmed.is_empty() {
                    clauses.push(trimmed.to_string());
                }
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    let tail: String = chars[start..].iter().collect();
    let trimmed = tail.trim();
    if !trimmed.is_empty() {
        clauses.push(trimmed.to_string());
    }
    clauses
}

fn parse_state_machine_transition_clause(
    raw: &str,
) -> Option<(String, Option<String>, Option<String>)> {
    let (to, rest) = raw.split_once(" 으로")?;
    let to = to.trim().to_string();
    if to.is_empty() {
        return None;
    }
    let mut rest = rest.trim();
    let mut guard_name = None;
    let mut action_name = None;

    if let Some(tail) = rest.strip_prefix("걸러서 ") {
        let tail = tail.trim();
        if let Some((guard, next)) = tail.split_once(" 하고 ") {
            let guard = guard.trim();
            let action = next.trim();
            if guard.is_empty() || action.is_empty() {
                return None;
            }
            guard_name = Some(guard.to_string());
            action_name = Some(action.to_string());
            rest = "";
        } else {
            if tail.is_empty() {
                return None;
            }
            guard_name = Some(tail.to_string());
            rest = "";
        }
    } else if let Some(tail) = rest.strip_prefix("하고 ") {
        let action = tail.trim();
        if action.is_empty() {
            return None;
        }
        action_name = Some(action.to_string());
        rest = "";
    }

    if !rest.is_empty() {
        return None;
    }
    Some((to, guard_name, action_name))
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl ParseError {
    pub fn code(&self) -> &'static str {
        if self.message.starts_with("E_SEED_NAME_CONFLICT_HA:") {
            return "E_SEED_NAME_CONFLICT_HA";
        }
        if self.message.starts_with("PIPE-FLOW-INJECT-AMBIGUOUS:") {
            return "PIPE-FLOW-INJECT-AMBIGUOUS";
        }
        if self.message.starts_with("E_CALL_TAIL_AMBIGUOUS:") {
            return "E_CALL_TAIL_AMBIGUOUS";
        }
        if self.message.contains("조사 '")
            && self.message.contains("모호합니다")
            && self.message.contains("값:핀")
        {
            return "E_PARSE_CALL_JOSA_AMBIGUOUS";
        }
        if self.message.contains("핀 '") && self.message.contains("인자가 중복되었습니다") {
            return "E_PARSE_CALL_PIN_DUPLICATE";
        }
        if self
            .message
            .contains("`미루기` 대입은 `덩이 {}` 안에서만 사용할 수 있습니다")
        {
            return "E_PARSE_DEFERRED_ASSIGN_OUTSIDE_BEAT";
        }
        "E_PARSE"
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "파서 오류: {}", self.message)
    }
}

impl std::error::Error for ParseError {}
