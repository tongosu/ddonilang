//! Existing statement, hook and iteration parser methods; Parser state and all method bodies unchanged.
use super::*;

impl Parser {
    pub(super) fn parse_body(&mut self) -> Result<Body, ParseError> {
        self.body_depth+=1;
        let result=self.parse_body_inner();
        self.body_depth=self.body_depth.saturating_sub(1);
        result
    }
    pub(super) fn parse_body_inner(&mut self) -> Result<Body, ParseError> {
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

    pub(super) fn expect_colon_or_lbrace(&mut self, context: &str) -> Result<(), ParseError> {
        if self.check(&TokenKind::Colon) {
            self.advance();
        }
        if !self.check(&TokenKind::LBrace) {
            return Err(self.error(context));
        }
        Ok(())
    }

    pub(super) fn parse_thunk_body_after_lbrace(&mut self, start: Span) -> Result<Body, ParseError> {
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

    pub(super) fn is_ordinary_while_start(&self) -> bool {
        if !self.check(&TokenKind::LBrace) {
            return false;
        }
        let mut depth = 0usize;
        for (offset, token) in self.tokens[self.pos..].iter().enumerate() {
            match token.kind {
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        return self
                            .tokens
                            .get(self.pos + offset + 1)
                            .is_some_and(|next| Self::token_text_is(&next.kind, "인"))
                            && self
                                .tokens
                                .get(self.pos + offset + 2)
                                .is_some_and(|next| matches!(next.kind, TokenKind::KwDongan));
                    }
                }
                _ => {}
            }
        }
        false
    }

    pub(super) fn parse_body_with_chaebi_restriction(&mut self) -> Result<Body, ParseError> {
        self.chaebi_forbidden_depth += 1;
        let body = self.parse_body();
        self.chaebi_forbidden_depth = self.chaebi_forbidden_depth.saturating_sub(1);
        body
    }

    pub(super) fn parse_stmt(&mut self, allow_implicit_terminator: bool) -> Result<Stmt, ParseError> {
        let stmt=self.parse_stmt_inner(allow_implicit_terminator)?;
        let binding=match &stmt {
            Stmt::Mutate {target,..}=>match &target.kind {ExprKind::Var(name)=>Some(name),_=>None},
            Stmt::Definition {name,..}=>Some(name),
            _=>None,
        };
        if let Some(name)=binding {
            if let Some(scope)=self.declared_scopes.last_mut() {scope.ordinary.insert(name.clone());}
        }
        Ok(stmt)
    }

    pub(super) fn parse_stmt_inner(&mut self, allow_implicit_terminator: bool) -> Result<Stmt, ParseError> {
        let s = self.current_span();
        if let Some(provider)=self.try_parse_reaction_provider()? {
            return Ok(provider);
        }
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
        if self.peek_legacy_boim_block() {
            return Err(ParseError {
                span: self.current_span(),
                message: "E_BOIM_LEGACY_REMOVED: `보임 {}` 블록은 제거되었습니다. `(<이름>: <식>, ...) 내보임.`을 사용하세요".to_string(),
            });
        }
        if self.source_epoch == SourceEpoch::V25 && self.peek_legacy_l11_brace_block() {
            return Err(ParseError {
                span: self.current_span(),
                message: "E_V25_DEFINITION_MIGRATION_REQUIRED: V25에서는 이전 brace형 L11 선언을 사용할 수 없습니다. `(<항목>=value) 쓰임.` 또는 `(<항목>=value) 드러냄.`처럼 현재 typed directive를 사용하세요".to_string(),
            });
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
        if self.check(&TokenKind::KwGeonneottwigi) {
            return self.parse_continue_stmt();
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
        if self.is_sum_definition_stmt_start() {
            return self.parse_sum_definition_stmt();
        }
        if self.source_epoch == SourceEpoch::V25 && self.is_legacy_definition_stmt_start() {
            return Err(ParseError {
                span: self.current_span(),
                message: "E_V25_DEFINITION_MIGRATION_REQUIRED: V25 정의에는 `:=`가 필요하며 이전 `=` 원문은 명시적 마이그레이션 입력으로 분리해야 합니다".to_string(),
            });
        }
        if self.source_epoch == SourceEpoch::V25 && self.is_definition_stmt_start() {
            return self.parse_definition_stmt();
        }
        if self.is_hook_start() {
            return self.parse_hook_stmt();
        }
        if self.is_natural_foreach_start() {
            return self.parse_natural_foreach_stmt();
        }
        if self.is_beat_block_start() {
            return self.parse_beat_block_stmt();
        }
        if self.check(&TokenKind::KwManyak) {
            if self.source_epoch == SourceEpoch::V25 {
                return Err(self.error("E_V25_IF_SURFACE: 조건문은 '{ 조건 }이면 { ... } 아니면 { ... }.'로 작성하세요"));
            }
            return self.parse_manyak_if_stmt();
        }
        if self.is_sum_match_stmt_start() {
            if let Some(stmt) = self.try_parse_sum_match_stmt()? { return Ok(stmt); }
        }
        if let Some(stmt) = self.try_parse_receive_stmt()? {
            return Ok(stmt);
        }
        if self.peek_public_observation_stmt() {
            return self.parse_public_observation_stmt();
        }

        if self.source_epoch == SourceEpoch::V25 && self.is_current_if_start() {
            self.advance();
            let thunk_body = self.parse_thunk_body_after_lbrace(s)?;
            self.validate_eval_body(&thunk_body, ThunkEvalMode::Bool, thunk_body.span)?;
            let thunk = Expr::new(self.next_id(), thunk_body.span, ExprKind::Thunk(thunk_body));
            self.advance(); // 정확한 `이면`
            let condition = Expr::new(self.next_id(), thunk.span, ExprKind::Eval {
                thunk: Box::new(thunk), mode: ThunkEvalMode::Bool,
            });
            return self.parse_if_stmt_from_condition(s, condition);
        }
        if self.is_ordinary_while_start() {
            self.advance(); // 여는 중괄호
            let thunk_body = self.parse_thunk_body_after_lbrace(s)?;
            self.validate_eval_body(&thunk_body, ThunkEvalMode::Bool, thunk_body.span)?;
            let thunk = Expr::new(self.next_id(), thunk_body.span, ExprKind::Thunk(thunk_body));
            self.advance(); // 구조적 미리보기에서 확인한 정확한 `인`
            self.expect(&TokenKind::KwDongan, "동안")?;
            let condition = Expr::new(
                self.next_id(),
                thunk.span,
                ExprKind::Eval {
                    thunk: Box::new(thunk),
                    mode: ThunkEvalMode::Bool,
                },
            );
            if !self.check(&TokenKind::LBrace) {
                return Err(
                    self.error("E_WHILE_SURFACE: 동안 다음에는 콜론 없이 '{'가 와야 합니다")
                );
            }
            let body = self.parse_body_with_chaebi_restriction()?;
            let mood = self.consume_optional_terminator()?;
            return Ok(Stmt::While {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                mood,
                condition,
                body,
            });
        }

        let explicit_sender = self.try_parse_message_sender_prefix()?;
        let e = self.parse_expr()?;
        if self.check(&TokenKind::TypeArrow) {
            return Err(ParseError {
                span: self.current_span(),
                message: "E_FUNCTION_TYPE_ARROW_VALUE_POSITION: `-->`는 타입 위치에서만 사용할 수 있습니다".to_string(),
            });
        }
        if self.check(&TokenKind::AsyncLeftArrow) || self.check(&TokenKind::RendezvousLeftArrow) {
            let transfer = if self.check(&TokenKind::AsyncLeftArrow) {
                MessageTransfer::ASYNC
            } else {
                MessageTransfer::RENDEZVOUS
            };
            self.advance();
            let payload = self.parse_expr()?;
            let mood = self.consume_stmt_terminator()?;
            return Ok(Stmt::Send {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                mood,
                transfer,
                sender: None,
                payload,
                receiver: e,
            });
        }

        if self.check(&TokenKind::PersistentDependencyArrow) {
            if self.peek_kind_n_is(1,|kind|matches!(kind,TokenKind::Gt)) {
                return Err(ParseError {span:self.current_span(),message:"E_BIDIRECTIONAL_BINDING_UNSUPPORTED: `<<->>` 양방향 맞춤은 베타2에서 지원하지 않습니다. 두 `<<-`로 대신하지 않습니다".into()});
            }
            let ExprKind::Var(name)=&e.kind else {
                return Err(ParseError {span:e.span,message:"E_DERIVED_TARGET: 지속 파생 대상은 맨 바깥 이름 하나여야 합니다. 칸·선택 자리는 지원하지 않습니다".into()});
            };
            if explicit_sender.is_some() || !self.seed_kind_stack.is_empty() || self.body_depth!=0 {
                return Err(ParseError {span:e.span,message:"E_DERIVED_SCOPE: 지속 파생은 맨 바깥 이름·채비에만 선언합니다. 셈씨·움직씨 안에서는 지원하지 않습니다".into()});
            }
            let name=name.clone();self.advance();
            let value=self.parse_expr()?;
            let mood=self.consume_stmt_terminator()?;
            return Ok(Stmt::DerivedBinding {id:self.next_id(),span:s.merge(&self.previous_span()),mood,name,value});
        }

        let mut sender = explicit_sender;
        let mut payload = e;

        if Self::token_text_is(&self.current().kind, "의") {
            self.advance();
            sender = Some(payload);
            payload = self.parse_expr()?;
        }

        if self.check(&TokenKind::DoubleArrow) || self.check(&TokenKind::AsyncRightArrow) {
            let transfer = if self.check(&TokenKind::AsyncRightArrow) {
                MessageTransfer::ASYNC
            } else if self.mode == ParseMode::HistoricalMigrationEvidence {
                MessageTransfer::historical_async_for_migration()
            } else {
                MessageTransfer::RENDEZVOUS
            };
            self.advance();
            let receiver = self.parse_expr()?;
            let mood = self.consume_stmt_terminator()?;
            return Ok(Stmt::Send {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                mood,
                transfer,
                sender,
                payload,
                receiver,
            });
        }

        let e = payload;

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
            if self.source_epoch == SourceEpoch::V25 {
                return Err(self.error("E_V25_IF_SURFACE: 조건문은 '{ 조건 }이면 { ... } 아니면 { ... }.'로 작성하세요"));
            }
            self.advance();
            return self.parse_if_stmt_from_condition(s, e);
        }

        if self.check(&TokenKind::KwDongan) {
            return Err(ParseError {
                span: e.span.merge(&self.current_span()),
                message:
                    "E_WHILE_SURFACE: 일반 반복은 '{ 조건 }인 동안 { ... }.'로 작성해야 합니다"
                        .to_string(),
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
            if self.source_epoch != SourceEpoch::V25 {
                let target = self.parse_expr()?;
                let deferred_span = self.consume_deferred_marker()?;
                let deferred = deferred_span.is_some();
                let mood = self.consume_stmt_terminator()?;
                return self.build_mutate_stmt_from_assignment(
                    s,
                    target,
                    e,
                    mood,
                    deferred,
                    deferred_span,
                );
            }
            let destination_expr = self.parse_expr()?;
            let destination = self.build_result_flow_destination(destination_expr)?;
            if self.consume_deferred_marker()?.is_some() {
                return Err(self.error(
                    "E_RESULT_FLOW_DEFERRED_FORBIDDEN: `->` 결과흐름에는 미루기를 붙일 수 없습니다",
                ));
            }
            let mood = self.consume_stmt_terminator()?;
            if !matches!(destination.kind, ExprKind::SelectionPlace { .. } | ExprKind::MultiAxisSelection(_)) {
                return Err(ParseError {
                    span: destination.span,
                    message: "E_RESULT_FLOW_DESTINATION_PLACE_REQUIRED: `->`의 대상은 indexed writable SelectionPlace여야 합니다".to_string(),
                });
            }
            Ok(Stmt::ResultFlow {
                id: self.next_id(),
                span: s.merge(&self.previous_span()),
                mood,
                producer: e,
                destination,
            })
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

    pub(super) fn is_beat_block_start(&self) -> bool {
        if !self.is_beat_block_head() {
            return false;
        }
        self.peek_kind_n_is(1, |k| matches!(k, TokenKind::LBrace))
            || (self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Colon))
                && self.peek_kind_n_is(2, |k| matches!(k, TokenKind::LBrace)))
    }

    pub(super) fn is_beat_block_head(&self) -> bool {
        matches!(self.current().kind, TokenKind::KwBeat)
    }

    pub(super) fn token_text_is(token: &TokenKind, expected: &str) -> bool {
        match token {
            TokenKind::Ident(name) | TokenKind::Josa(name) => name == expected,
            TokenKind::KwIlttae => matches!(expected, "일때" | "할때"),
            TokenKind::KwDongan => expected == "동안",
            _ => false,
        }
    }

    pub(super) fn try_parse_receive_stmt(&mut self) -> Result<Option<Stmt>, ParseError> {
        let checkpoint = self.pos;
        let statement_start = self.current_span();
        let sender = if let Some(sender_end) = self.receive_sender_filter_index() {
            let marker = self.tokens[sender_end].clone();
            self.tokens[sender_end].kind = TokenKind::Eof;
            let parsed = self.parse_expr();
            self.tokens[sender_end] = marker;
            let sender = parsed?;
            if self.pos != sender_end {
                return Err(ParseError {
                    span: self.current_span(),
                    message: "E_ASYNC_SENDER_FILTER_PARSE_CANDIDATE: `에게서` 앞에는 하나의 sender identity 식이 필요합니다".to_string(),
                });
            }
            self.advance();
            Some(sender)
        } else {
            None
        };
        let mut binding = None;
        let mut payload_type = None;
        let mut condition = None;

        if self.has_typed_receive_binding_head() {
            self.advance();
            let binding_name = match self.current().kind.clone() {
                TokenKind::Ident(name) | TokenKind::Josa(name) => name,
                _ => return Err(self.error("sender-filter receive binding name")),
            };
            self.advance();
            self.expect(&TokenKind::Colon, ":")?;
            let type_name = match self.current().kind.clone() {
                TokenKind::Ident(name) | TokenKind::Josa(name) => name,
                _ => return Err(self.error("sender-filter receive payload type")),
            };
            self.advance();
            self.expect(&TokenKind::RParen, ")")?;
            if !matches!(&self.current().kind, TokenKind::Josa(josa) if josa == "을" || josa == "를")
            {
                return Err(self.error("을/를"));
            }
            self.advance();
            binding = Some(binding_name);
            payload_type = Some(type_name);
        } else if self.check(&TokenKind::LParen) {
            if !self.has_receive_binding_head() {
                if sender.is_some() {
                    return Err(ParseError {
                        span: self.current_span(),
                        message: "E_ASYNC_SENDER_FILTER_PARSE_CANDIDATE: sender-filter 뒤에는 `(이름:형)를 받으면`이 필요합니다".to_string(),
                    });
                }
                self.pos = checkpoint;
                return Ok(None);
            }
            self.advance();
            let (TokenKind::Ident(name) | TokenKind::Josa(name)) = self.current().kind.clone()
            else {
                self.pos = checkpoint;
                return Ok(None);
            };
            self.advance();
            binding = Some(name);
            if !self.check(&TokenKind::RParen) {
                condition = Some(self.parse_expr()?);
            }
            if !self.check(&TokenKind::RParen) {
                return Err(self.error(")"));
            }
            self.advance();
            if !Self::token_text_is(&self.current().kind, "인") {
                self.pos = checkpoint;
                return Ok(None);
            }
            self.advance();
        }

        let (kind, start) = if payload_type.is_some() {
            (None, statement_start)
        } else {
            let Some((kind, start)) = self.try_consume_receive_kind_name() else {
                self.pos = checkpoint;
                return Ok(None);
            };
            (kind, start)
        };
        if !Self::token_text_is(&self.current().kind, "받으면") {
            self.pos = checkpoint;
            return Ok(None);
        }
        if !self.in_imja_seed_body() {
            return Err(ParseError {
                span: start,
                message: "`받으면` 훅은 `임자` 본문 안에서만 사용할 수 있습니다".to_string(),
            });
        }
        self.advance();
        let body = self.parse_body()?;
        let mood = self.consume_optional_terminator()?;
        Ok(Some(Stmt::Receive {
            id: self.next_id(),
            span: statement_start.merge(&self.previous_span()),
            mood,
            sender,
            kind,
            binding,
            payload_type,
            condition,
            body,
        }))
    }

    pub(super) fn try_parse_message_sender_prefix(&mut self) -> Result<Option<Expr>, ParseError> {
        let Some(sender_end) = self.message_sender_possessive_index() else {
            return Ok(None);
        };
        let marker = self.tokens[sender_end].clone();
        self.tokens[sender_end].kind = TokenKind::Eof;
        let parsed = self.parse_expr();
        self.tokens[sender_end] = marker;
        let sender = parsed?;
        if self.pos != sender_end {
            return Err(ParseError {
                span: self.current_span(),
                message: "E_ASYNC_SENDER_IDENTITY_PARSE_CANDIDATE: message sender 앞에는 하나의 identity 식이 필요합니다".to_string(),
            });
        }
        self.advance();
        Ok(Some(sender))
    }

    pub(super) fn message_sender_possessive_index(&self) -> Option<usize> {
        let mut possessive = None;
        for (index, token) in self.tokens.iter().enumerate().skip(self.pos) {
            match token.kind {
                TokenKind::Dot | TokenKind::Question | TokenKind::Bang | TokenKind::Eof => {
                    return None
                }
                TokenKind::Josa(ref josa) if josa == "의" && possessive.is_none() => {
                    possessive = Some(index)
                }
                TokenKind::AsyncRightArrow | TokenKind::DoubleArrow => return possessive,
                _ => {}
            }
        }
        None
    }

    pub(super) fn receive_sender_filter_index(&self) -> Option<usize> {
        self.tokens
            .iter()
            .enumerate()
            .skip(self.pos)
            .take(32)
            .take_while(|(_, token)| {
                !matches!(
                    token.kind,
                    TokenKind::Eof
                        | TokenKind::Dot
                        | TokenKind::AsyncLeftArrow
                        | TokenKind::AsyncRightArrow
                        | TokenKind::RendezvousLeftArrow
                        | TokenKind::DoubleArrow
                )
            })
            .find_map(|(index, token)| Self::token_text_is(&token.kind, "에게서").then_some(index))
    }

    pub(super) fn has_typed_receive_binding_head(&self) -> bool {
        if !self.check(&TokenKind::LParen) {
            return false;
        }
        let Some(name) = self.tokens.get(self.pos + 1) else {
            return false;
        };
        let Some(colon) = self.tokens.get(self.pos + 2) else {
            return false;
        };
        let Some(type_name) = self.tokens.get(self.pos + 3) else {
            return false;
        };
        let Some(close) = self.tokens.get(self.pos + 4) else {
            return false;
        };
        let Some(object) = self.tokens.get(self.pos + 5) else {
            return false;
        };
        let Some(receive) = self.tokens.get(self.pos + 6) else {
            return false;
        };
        matches!(name.kind, TokenKind::Ident(_) | TokenKind::Josa(_))
            && matches!(colon.kind, TokenKind::Colon)
            && matches!(type_name.kind, TokenKind::Ident(_) | TokenKind::Josa(_))
            && matches!(close.kind, TokenKind::RParen)
            && matches!(&object.kind, TokenKind::Josa(josa) if josa == "을" || josa == "를")
            && Self::token_text_is(&receive.kind, "받으면")
    }

    pub(super) fn has_receive_binding_head(&self) -> bool {
        if !self.check(&TokenKind::LParen) {
            return false;
        }
        let mut depth = 0usize;
        for idx in self.pos..self.tokens.len() {
            match self.tokens.get(idx).map(|token| &token.kind) {
                Some(TokenKind::LParen) => depth += 1,
                Some(TokenKind::RParen) => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        let Some(in_token) = self.tokens.get(idx + 1) else {
                            return false;
                        };
                        return Self::token_text_is(&in_token.kind, "인")
                            && self.receive_tail_starts_at(idx + 2);
                    }
                }
                Some(TokenKind::Eof) | None => return false,
                _ => {}
            }
        }
        false
    }

    pub(super) fn receive_tail_starts_at(&self, idx: usize) -> bool {
        let Some(token) = self.tokens.get(idx) else {
            return false;
        };
        let text = match &token.kind {
            TokenKind::Ident(text) | TokenKind::Josa(text) => text,
            _ => return false,
        };
        let after_kind_idx = if text
            .strip_suffix('를')
            .or_else(|| text.strip_suffix('을'))
            .is_some()
        {
            idx + 1
        } else if self.tokens.get(idx + 1).is_some_and(
            |token| matches!(&token.kind, TokenKind::Josa(josa) if josa == "을" || josa == "를"),
        ) {
            idx + 2
        } else {
            return false;
        };
        self.tokens
            .get(after_kind_idx)
            .is_some_and(|token| Self::token_text_is(&token.kind, "받으면"))
    }

    pub(super) fn try_consume_receive_kind_name(&mut self) -> Option<(Option<String>, Span)> {
        let start = self.current_span();
        let text = match &self.current().kind {
            TokenKind::Ident(text) | TokenKind::Josa(text) => text.clone(),
            _ => return None,
        };
        let kind = if let Some(raw) = text.strip_suffix('를').or_else(|| text.strip_suffix('을'))
        {
            self.advance();
            raw.to_string()
        } else if self.peek_kind_n_is(
            1,
            |kind| matches!(kind, TokenKind::Josa(josa) if josa == "을" || josa == "를"),
        ) {
            self.advance();
            self.advance();
            text
        } else {
            return None;
        };
        if kind == "알림" {
            Some((None, start))
        } else {
            Some((Some(kind), start))
        }
    }

    pub(super) fn is_hook_start(&self) -> bool {
        self.is_every_madi_hook_start()
            || self.is_start_end_hook_start()
            || self.is_condition_hook_start()
    }

    pub(super) fn is_start_end_hook_start(&self) -> bool {
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

    pub(super) fn find_matching_rparen_after_current_lparen(&self) -> Option<usize> {
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

    pub(super) fn is_condition_hook_start(&self) -> bool {
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
                .map(|t| {
                    matches!(t.kind, TokenKind::KwDongan) || Self::token_text_is(&t.kind, "동안")
                })
                .unwrap_or(false)
    }

    pub(super) fn parse_manyak_if_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        self.advance();
        let condition = self.parse_expr()?;
        self.expect(&TokenKind::KwIramyeon, "이라면")?;
        self.parse_if_stmt_from_condition(start, condition)
    }

    pub(super) fn is_current_if_start(&self) -> bool {
        if !self.check(&TokenKind::LBrace) { return false; }
        let mut depth = 0usize;
        for (offset, token) in self.tokens[self.pos..].iter().enumerate() {
            match token.kind {
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        return self.tokens.get(self.pos + offset + 1)
                            .is_some_and(|next| next.raw == "이면");
                    }
                }
                _ => {}
            }
        }
        false
    }

    pub(super) fn parse_if_stmt_from_condition(
        &mut self,
        start: Span,
        condition: Expr,
    ) -> Result<Stmt, ParseError> {
        let then_body = self.parse_body()?;
        let else_body = if let Some(pos) = self.find_if_else_lead_pos() {
            if self.source_epoch == SourceEpoch::V25
                && (pos != self.pos || self.tokens[pos].raw != "아니면") {
                return Err(self.error("E_V25_IF_SURFACE: '아니면'은 앞 본문의 닫는 중괄호 바로 뒤에 작성하세요"));
            }
            self.pos = pos;
            self.advance();
            if self.source_epoch == SourceEpoch::V25 && self.is_current_if_start() {
                return Err(self.error("E_V25_IF_CHAIN_UNSUPPORTED: 연쇄 조건 대신 '아니면 { { 조건 }이면 { ... }. }'처럼 중첩하세요"));
            }
            Some(self.parse_body()?)
        } else {
            None
        };
        let mood = self.consume_optional_terminator()?;
        Ok(Stmt::If {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            mood,
            condition,
            then_body,
            else_body,
        })
    }

    pub(super) fn find_if_else_lead_pos(&self) -> Option<usize> {
        if matches!(self.current().kind, TokenKind::KwAniramyeon) {
            return Some(self.pos);
        }
        if matches!(
            self.current().kind,
            TokenKind::Dot | TokenKind::Question | TokenKind::Bang | TokenKind::Semicolon
        ) {
            let next = self.pos + 1;
            if matches!(
                self.tokens.get(next).map(|token| &token.kind),
                Some(TokenKind::KwAniramyeon)
            ) {
                return Some(next);
            }
        }
        None
    }

    pub(super) fn parse_hook_stmt(&mut self) -> Result<Stmt, ParseError> {
        if self.is_every_madi_hook_start() {
            return self.parse_every_madi_hook_stmt();
        }
        if self.is_start_end_hook_start() {
            return self.parse_start_end_hook_stmt();
        }
        self.parse_condition_hook_stmt()
    }

    pub(super) fn is_every_madi_hook_start(&self) -> bool {
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

    pub(super) fn parse_every_madi_hook_stmt(&mut self) -> Result<Stmt, ParseError> {
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
        let (body, mood, span) = self.parse_hook_body_and_span(
            start,
            "`(매마디)마다` 뒤에는 ':' 없이 바로 '{'를 사용하세요",
        )?;
        Ok(Stmt::Hook {
            id: self.next_id(),
            span,
            mood,
            kind,
            body,
        })
    }

    pub(super) fn parse_start_end_hook_stmt(&mut self) -> Result<Stmt, ParseError> {
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

    pub(super) fn parse_condition_hook_stmt(&mut self) -> Result<Stmt, ParseError> {
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
            let (body, mood, span) = self.parse_hook_body_and_span(
                start,
                "`(조건)이 될때` 뒤에는 ':' 없이 바로 '{'를 사용하세요",
            )?;
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
            let (body, mood, span) = self.parse_hook_body_and_span(
                start,
                "`(조건)인 동안` 뒤에는 ':' 없이 바로 '{'를 사용하세요",
            )?;
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

    pub(super) fn parse_hook_body_and_span(
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
        let body = self.parse_body_with_chaebi_restriction()?;
        let mood = self.consume_optional_terminator()?;
        let span = start.merge(&self.previous_span());
        Ok((body, mood, span))
    }

    pub(super) fn parse_beat_block_stmt(&mut self) -> Result<Stmt, ParseError> {
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

    pub(super) fn consume_deferred_marker(&mut self) -> Result<Option<Span>, ParseError> {
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

    pub(super) fn peek_meta_block_kind(&self) -> Option<MetaBlockKind> {
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

    pub(super) fn peek_legacy_boim_block(&self) -> bool {
        if !Self::token_text_is(&self.current().kind, "보임") {
            return false;
        }
        if self.peek_kind_n_is(1, |kind| matches!(kind, TokenKind::LBrace)) {
            return true;
        }
        self.peek_kind_n_is(1, |kind| matches!(kind, TokenKind::Colon))
            && self.peek_kind_n_is(2, |kind| matches!(kind, TokenKind::LBrace))
    }

    pub(super) fn peek_public_observation_stmt(&self) -> bool {
        if !self.check(&TokenKind::LParen) {
            return false;
        }
        let mut depth = 0usize;
        for index in self.pos..self.tokens.len() {
            match self.tokens[index].kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return self
                            .tokens
                            .get(index + 1)
                            .map(|token| Self::token_text_is(&token.kind, "내보임")
                                || Self::token_text_is(&token.kind, "보임"))
                            .unwrap_or(false);
                    }
                }
                TokenKind::Eof => return false,
                _ => {}
            }
        }
        false
    }

    pub(super) fn parse_public_observation_stmt(&mut self) -> Result<Stmt, ParseError> {
        let open = self.expect(&TokenKind::LParen, "(")?;
        let start = self.to_ast_span(open.span);
        if self.check(&TokenKind::RParen) {
            return Err(ParseError {
                span: start.merge(&self.current_span()),
                message: "E_BOIM_EMPTY_PACK: 공개 관찰 값꾸러미는 비어 있을 수 없습니다"
                    .to_string(),
            });
        }

        let mut fields = Vec::new();
        let mut names = HashSet::new();
        loop {
            if !self.check_ident() {
                return Err(ParseError {
                    span: self.current_span(),
                    message: "E_BOIM_FIELD_NAME_REQUIRED: 공개 관찰 식에는 `이름: 식`이 필요합니다"
                        .to_string(),
                });
            }
            if self.peek_is_equals() {
                return Err(ParseError {
                    span: self.current_span(),
                    message: "E_BOIM_FIXED_BINDING_FORBIDDEN: 공개 관찰 필드는 `=`가 아니라 `:`을 사용해야 합니다".to_string(),
                });
            }

            let name_token = self.expect_ident("공개 관찰 필드 이름")?;
            let name = match &name_token.kind {
                TokenKind::Ident(name) | TokenKind::Josa(name) => self
                    .source
                    .get(name_token.span.start..name_token.span.end)
                    .map(str::to_string)
                    .unwrap_or_else(|| name.clone()),
                _ => unreachable!(),
            };
            let name_span = self.to_ast_span(name_token.span);
            let value = if self.check(&TokenKind::Colon) {
                self.advance();
                self.parse_expr()?
            } else if fields.is_empty() && self.check(&TokenKind::RParen) {
                Expr::new(self.next_id(), name_span, ExprKind::Var(name.clone()))
            } else {
                return Err(ParseError {
                    span: name_span,
                    message: "E_BOIM_FIELD_NAME_REQUIRED: 여러 공개 관찰 필드는 모두 `이름: 식`으로 작성해야 합니다".to_string(),
                });
            };

            if !names.insert(name.clone()) {
                return Err(ParseError {
                    span: name_span,
                    message: format!(
                        "E_BOIM_DUPLICATE_FIELD: 공개 관찰 필드 `{name}`이 중복되었습니다"
                    ),
                });
            }
            let field_span = name_span.merge(&value.span);
            fields.push(PublicObservationField {
                id: self.next_id(),
                span: field_span,
                name,
                name_span,
                value,
            });

            if self.check(&TokenKind::Comma) {
                self.advance();
                if self.check(&TokenKind::RParen) {
                    return Err(ParseError {
                        span: self.current_span(),
                        message:
                            "E_BOIM_FIELD_NAME_REQUIRED: 마지막 쉼표 뒤에 공개 관찰 필드가 필요합니다"
                                .to_string(),
                    });
                }
                continue;
            }
            break;
        }

        self.expect(&TokenKind::RParen, ")")?;
        if Self::token_text_is(&self.current().kind, "보임") {
            return Err(ParseError {
                span: self.current_span(),
                message: "E_BOIM_LEGACY_REMOVED: 공개 관찰 선언은 `(필드들) 내보임.`을 사용하세요".to_string(),
            });
        }
        if !Self::token_text_is(&self.current().kind, "내보임") {
            return Err(self.error("내보임"));
        }
        self.advance();
        let mood = self.consume_stmt_terminator()?;
        Ok(Stmt::PublicObservation {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            mood,
            fields,
        })
    }

    pub(super) fn parse_meta_block_stmt(&mut self) -> Result<Stmt, ParseError> {
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

    pub(super) fn parse_bogae_madang_block_stmt(&mut self) -> Result<Stmt, ParseError> {
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

    pub(super) fn parse_jjaim_block_stmt(&mut self) -> Result<Stmt, ParseError> {
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

    pub(super) fn peek_decl_block_kind(&self) -> Option<DeclKind> {
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
            "성질" if self.in_imja_seed_body() => Some(DeclKind::Gureut),
            _ => None,
        }
    }

    pub(super) fn peek_legacy_decl_block_name(&self) -> Option<&'static str> {
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

    pub(super) fn parse_decl_block(&mut self, _kind: DeclKind) -> Result<Stmt, ParseError> {
        let s = self.current_span();
        let keyword = self.advance().raw.clone(); // consume keyword ident
        if keyword != "채비" && !(keyword == "성질" && self.in_imja_seed_body()) {
            return Err(ParseError {
                span: s,
                message: "선언 블록 머릿말은 `채비`만 사용합니다. `성질`은 `임자` 본문 안에서만 허용됩니다".to_string(),
            });
        }
        self.expect_colon_or_lbrace("선언 블록 본문")?;
        self.expect(&TokenKind::LBrace, "{")?;
        if self.chaebi_forbidden_depth > 0 {
            return Err(ParseError {
                span: s,
                message:
                    "E_CHAEBI_IN_LOOP: `채비 {}`는 루프/훅/순회 body 안에서 사용할 수 없습니다."
                        .to_string(),
            });
        }
        let mut items = Vec::new();
        while !self.check(&TokenKind::RBrace) {
            let item_start = self.current_span();
            let name = self.expect_ident("선언 이름")?.raw.clone();
            if name == "없음" && self.check(&TokenKind::Dot) {
                return Err(ParseError {
                    span: self.current_span(),
                    message:
                        "채비 항목은 `이름:타입 <- 값.` 형태여야 합니다. 비울 때는 `채비 { }.`를 사용하세요"
                            .to_string(),
                });
            }
            let type_ref = if self.check(&TokenKind::Colon) {
                self.advance();
                self.parse_type_ref()?
            } else if self.check(&TokenKind::Arrow)
                || self.check(&TokenKind::Equals)
                || self.check(&TokenKind::Define)
            {
                TypeRef::Infer
            } else {
                return Err(
                    self.error("채비 항목은 `이름:타입 <- 값.` 또는 `이름 <- 값.` 형태여야 합니다")
                );
            };
            let mut item_kind = DeclKind::Gureut;
            let mut value = None;
            if self.source_epoch == SourceEpoch::V25 && self.check(&TokenKind::Arrow) {
                return Err(ParseError {
                    span: self.current_span(),
                    message: "E_V25_DEFINITION_BINDING_CATEGORY: `채비`의 이름 도입에는 `:=`를 사용하고 이후 다시결속에만 `<-`를 사용하세요".to_string(),
                });
            }
            if self.check(&TokenKind::Define) {
                self.advance();
                value = Some(self.parse_expr()?);
            } else if self.check(&TokenKind::Arrow) {
                self.advance();
                value = Some(self.parse_expr()?);
            } else if self.check(&TokenKind::Equals) {
                item_kind = DeclKind::Butbak;
                self.advance();
                value = Some(self.parse_expr()?);
            }
            self.consume_decl_item_control_suffix_if_present()?;
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
            self.declare_typed_name(&name, &item.type_ref);
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

    pub(super) fn is_definition_stmt_start(&self) -> bool {
        if !self.check_ident() {
            return false;
        }
        self.peek_kind_n_is(1, |kind| {
            matches!(kind, TokenKind::Define | TokenKind::Colon)
        })
    }

    pub(super) fn is_legacy_definition_stmt_start(&self) -> bool {
        self.check_ident() && self.peek_kind_n_is(1, |kind| matches!(kind, TokenKind::Equals))
    }

    pub(super) fn parse_definition_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        let name = self.expect_ident("정의 이름")?.raw.clone();
        let type_ref = if self.check(&TokenKind::Colon) {
            self.advance();
            self.parse_type_ref()?
        } else {
            TypeRef::Infer
        };
        if self.source_epoch == SourceEpoch::V25 && self.check(&TokenKind::Equals) {
            return Err(ParseError {
                span: self.current_span(),
                message: "E_V25_DEFINITION_MIGRATION_REQUIRED: V25 정의에는 `:=`가 필요하며 이전 `=` 원문은 명시적 마이그레이션 입력으로 분리해야 합니다".to_string(),
            });
        }
        self.expect(&TokenKind::Define, ":=")?;
        let value = self.parse_expr()?;
        let mood = self.consume_stmt_terminator()?;
        self.declare_typed_name(&name, &type_ref);
        Ok(Stmt::Definition {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            mood,
            name,
            type_ref,
            value,
            reaction: None,
        })
    }

    pub(super) fn try_parse_reaction_provider(&mut self) -> Result<Option<Stmt>,ParseError> {
        // A3-P4 physical placement: closed RD-445 registration before :=.
        if self.source_epoch!=SourceEpoch::V25 || !self.check(&TokenKind::LParen) {
            return Ok(None);
        }
        let mut depth=0;
        let mut header_end=None;
        for (index,token) in self.tokens.iter().enumerate().skip(self.pos) {
            match token.kind {
                TokenKind::LParen=>depth+=1,
                TokenKind::RParen=>{depth-=1;if depth==0 {header_end=Some(index);break;}},
                TokenKind::Dot|TokenKind::Eof=>break,
                _=>{}
            }
        }
        let Some(end)=header_end else {return Ok(None)};
        let is_provider=self.tokens.get(end+2).is_some_and(|t|matches!(t.kind,TokenKind::Colon))
            && self.tokens.get(end+3).is_some_and(|t|t.raw=="움직씨")
            && self.tokens.get(end+4).is_some_and(|t|matches!(t.kind,TokenKind::LBrace));
        if !is_provider {return Ok(None)}
        if !self.in_imja_seed_body() {return Err(self.error("E_REACTION_OWNER: 반응 움직씨는 임자 정의 안에서만 선언합니다"));}
        let start=self.current_span();
        let params=self.parse_params()?;
        if params.len()!=1 {return Err(self.error("E_REACTION_PAYLOAD: 반응에는 형을 밝힌 알림 받는자리 하나가 필요합니다"));}
        let payload=params.into_iter().next().unwrap();
        let name=self.expect_ident("반응 움직씨 이름")?.raw.clone();
        self.expect(&TokenKind::Colon,":")?;
        self.parse_seed_kind()?;
        self.expect(&TokenKind::LBrace,"{")?;
        let alert_type=self.expect_ident("알림씨 이름")?.raw.clone();
        let particle=self.advance();
        if !matches!(&particle.kind,TokenKind::Josa(josa) if josa=="을"||josa=="를") {
            return Err(self.error("E_REACTION_REGISTRATION: 알림씨 뒤에는 을/를 받으면이 필요합니다"));
        }
        if !Self::token_text_is(&self.current().kind,"받으면") {return Err(self.error("E_REACTION_REGISTRATION: 받으면이 필요합니다"));}
        self.advance();self.expect(&TokenKind::RBrace,"}")?;
        if payload.type_ref!=TypeRef::Named(alert_type.clone()) || payload.default_value.is_some() || payload.optional {
            return Err(self.error("E_REACTION_PAYLOAD: 등록 알림씨와 받는자리 씨가 같아야 합니다"));
        }
        self.expect(&TokenKind::Define,":=")?;
        self.enter_scope();self.declare_typed_name(&payload.pin_name,&payload.type_ref);
        self.declare_name("받는이");
        let body=self.parse_body();self.exit_scope();
        let body=body?;
        let value=Expr::new(self.next_id(),body.span,ExprKind::Thunk(body));
        let mood=self.consume_optional_terminator()?;
        Ok(Some(Stmt::Definition {id:self.next_id(),span:start.merge(&self.previous_span()),mood,
            name,type_ref:TypeRef::Named("움직씨".into()),value,
            reaction:Some(ReactionRegistration{alert_type,payload})}))
    }

    pub(super) fn consume_decl_item_control_suffix_if_present(&mut self) -> Result<(), ParseError> {
        while self.is_decl_item_control_suffix_head() {
            self.advance(); // 매김|조건
            if self.check(&TokenKind::Colon) {
                self.advance();
            }
            self.expect(&TokenKind::LBrace, "{")?;
            let mut depth = 1usize;
            while depth > 0 {
                if self.is_at_end() {
                    return Err(self.error("`매김/조건` 블록 닫는 `}`가 필요합니다"));
                }
                match self.advance().kind {
                    TokenKind::LBrace => depth += 1,
                    TokenKind::RBrace => depth -= 1,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub(super) fn is_decl_item_control_suffix_head(&self) -> bool {
        matches!(
            &self.current().kind,
            TokenKind::Ident(name) | TokenKind::Josa(name) if name == "매김" || name == "조건"
        )
    }

    pub(super) fn parse_choose_stmt(&mut self) -> Result<Stmt, ParseError> {
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

    pub(super) fn parse_repeat_stmt(&mut self) -> Result<Stmt, ParseError> {
        let s = self.current_span();
        self.expect(&TokenKind::KwBanbok, "반복")?;
        self.expect_colon_or_lbrace("반복 본문")?;
        let body = self.parse_body_with_chaebi_restriction()?;
        let mood = self.consume_optional_terminator()?;
        Ok(Stmt::Repeat {
            id: self.next_id(),
            span: s.merge(&self.previous_span()),
            mood,
            body,
        })
    }

    pub(super) fn parse_break_stmt(&mut self) -> Result<Stmt, ParseError> {
        let s = self.current_span();
        self.expect(&TokenKind::KwMeomchugi, "멈추기")?;
        let mood = self.consume_stmt_terminator()?;
        Ok(Stmt::Break {
            id: self.next_id(),
            span: s.merge(&self.previous_span()),
            mood,
        })
    }

    pub(super) fn parse_continue_stmt(&mut self) -> Result<Stmt, ParseError> {
        let s = self.current_span();
        self.expect(&TokenKind::KwGeonneottwigi, "건너뛰기")?;
        let mood = self.consume_stmt_terminator()?;
        Ok(Stmt::ContinueLoop {
            id: self.next_id(),
            span: s.merge(&self.previous_span()),
            mood,
        })
    }

    pub(super) fn is_foreach_start(&self) -> bool {
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

    pub(super) fn is_quantifier_start(&self) -> bool {
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

    pub(super) fn parse_foreach_stmt(&mut self) -> Result<Stmt, ParseError> {
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
        self.declare_typed_name(&name, item_type.as_ref().unwrap_or(&TypeRef::Infer));
        let body = match self.parse_body_with_chaebi_restriction() {
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

    pub(super) fn is_natural_foreach_start(&mut self) -> bool {
        let saved_pos = self.pos;
        let saved_nid = self.nid;
        let parsed = self.parse_expr();
        let is_mada = parsed.is_ok()
            && Self::token_text_is(&self.current().kind, "마다")
            && self
                .tokens
                .get(self.pos + 1)
                .is_some_and(|token| matches!(token.kind, TokenKind::LBrace));
        self.pos = saved_pos;
        self.nid = saved_nid;
        is_mada
    }

    pub(super) fn parse_natural_foreach_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        let iterable = self.parse_expr()?;
        if !Self::token_text_is(&self.current().kind, "마다") {
            return Err(self.error("마다"));
        }
        self.advance();
        if self.check(&TokenKind::Colon) {
            return Err(ParseError {
                span: self.current_span(),
                message: "E_NATURAL_FOREACH_COLON_FORBIDDEN: `대상마다` 뒤에는 ':' 없이 바로 '{'를 사용하세요".to_string(),
            });
        }
        if !self.check(&TokenKind::LBrace) {
            return Err(self.error("짧은 `마다` 본문"));
        }
        let item = match &iterable.kind {
            ExprKind::Var(name) => name.clone(),
            ExprKind::FieldAccess { field, .. } => field.clone(),
            _ => {
                return Err(ParseError {
                    span: iterable.span,
                    message: "E_NATURAL_FOREACH_BINDER_NAME_REQUIRED: 짧은 `마다`의 대상은 binder 이름을 보존할 수 있는 direct path여야 합니다".to_string(),
                })
            }
        };
        let item_type = self.natural_iterable_item_type(&iterable);
        self.enter_scope();
        self.declare_typed_name(&item, &item_type);
        let body = match self.parse_body_with_chaebi_restriction() {
            Ok(body) => body,
            Err(err) => {
                self.exit_scope();
                return Err(err);
            }
        };
        self.exit_scope();
        let mood = self.consume_optional_terminator()?;
        let id = self.next_id();
        self.natural_foreach_ids.insert(id);
        Ok(Stmt::ForEach {
            id,
            span: start.merge(&self.previous_span()),
            mood,
            item,
            item_type: Some(item_type),
            iterable,
            body,
        })
    }

    pub(super) fn natural_iterable_item_type(&self, iterable: &Expr) -> TypeRef {
        let container = match &iterable.kind {
            ExprKind::Var(name) => self.declared_type(name),
            _ => None,
        };
        match container {
            Some(TypeRef::Applied { name, args })
                if matches!(name.as_str(), "차림" | "모음") && args.len() == 1 =>
            {
                args[0].clone()
            }
            Some(TypeRef::Named(name)) if matches!(name.as_str(), "차림" | "모음" | "짝맞춤") => {
                TypeRef::Infer
            }
            Some(TypeRef::Infer) | None => TypeRef::Infer,
            Some(other) => other,
        }
    }

    pub(super) fn parse_quantifier_stmt(&mut self) -> Result<Stmt, ParseError> {
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
        let body = match self.parse_body_with_chaebi_restriction() {
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

    pub(super) fn parse_quantifier_kind(&mut self) -> Result<QuantifierKind, ParseError> {
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

}
