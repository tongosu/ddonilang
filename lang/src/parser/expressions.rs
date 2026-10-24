//! Existing expression precedence and primary parser methods; syntax and source spans unchanged.
use super::*;

impl Parser {
    pub(super) fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_pipe()
    }
    pub(super) fn parse_pipe(&mut self) -> Result<Expr, ParseError> {
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
    pub(super) fn parse_logical_or(&mut self) -> Result<Expr, ParseError> {
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
    pub(super) fn parse_logical_and(&mut self) -> Result<Expr, ParseError> {
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
    pub(super) fn is_logical_and_op(&self) -> bool {
        matches!(self.current().kind, TokenKind::And)
            || matches!(&self.current().kind, TokenKind::Ident(name) if name == "그리고")
    }
    pub(super) fn is_logical_or_op(&self) -> bool {
        matches!(self.current().kind, TokenKind::Or)
            || matches!(&self.current().kind, TokenKind::Ident(name) if name == "또는")
    }
    pub(super) fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut l = self.parse_comparison()?;
        while matches!(
            self.current().kind,
            TokenKind::RelationEq | TokenKind::EqEq | TokenKind::NotEq
        ) {
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
    pub(super) fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
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
    pub(super) fn parse_range(&mut self) -> Result<Expr, ParseError> {
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
    pub(super) fn parse_addition(&mut self) -> Result<Expr, ParseError> {
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
    pub(super) fn parse_multiplication(&mut self) -> Result<Expr, ParseError> {
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

    pub(super) fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.check(&TokenKind::Minus) {
            let op_token = self.advance();
            let number = self.current().clone();
            if matches!(number.kind, TokenKind::Integer(_) | TokenKind::Float(_))
                && Fixed64::parse_decimal_exact(&number.raw).is_none()
            {
                // 양수 magnitude가 표현 범위 밖이어도 signed MIN은 유효하다.
                // 일반 음수의 기존 0 - rhs AST는 이 경로에서 바꾸지 않는다.
                if let Some(value) = Fixed64::parse_decimal_exact(&format!("-{}", number.raw)) {
                    self.advance();
                    let span = self.to_ast_span(op_token.span)
                        .merge(&self.to_ast_span(number.span));
                    let expr = Expr::new(
                        self.next_id(), span, ExprKind::Literal(Literal::Fixed64(value)),
                    );
                    return self.parse_primary_tail(expr, true);
                }
            }
            let rhs = self.parse_unary()?;
            let zero = Expr::new(
                self.next_id(),
                self.to_ast_span(op_token.span),
                ExprKind::Literal(Literal::Fixed64(Fixed64::from_i64(0))),
            );
            let span = zero.span.merge(&rhs.span);
            let mut expr = Expr::new(
                self.next_id(),
                span,
                ExprKind::Infix {
                    left: Box::new(zero),
                    op: op_token.raw.clone(),
                    right: Box::new(rhs),
                },
            );
            expr.source_unary_minus = true;
            return Ok(expr);
        }
        if self.check(&TokenKind::Plus) {
            self.advance();
            return self.parse_unary();
        }
        self.parse_primary()
    }
    pub(super) fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        self.parse_primary_inner(true)
    }
    pub(super) fn parse_primary_inner(&mut self, allow_bare_bound_call: bool) -> Result<Expr, ParseError> {
        // EOF에서 advance()는 직전 토큰을 돌려준다. 열린 괄호를 다시
        // 읽어 재귀하지 말고 기존 표현식 오류로 끝낸다.
        if self.is_at_end() {
            return Err(self.error("표현식"));
        }
        if let Some(end)=self.find_matching_rparen_after_current_lparen() {
            if self.tokens.get(end+1).is_some_and(|token|matches!(token.kind,TokenKind::Ident(_)))
                && self.tokens.get(end+2).is_some_and(|token|matches!(token.kind,TokenKind::Dot))
                && self.tokens.get(end+3).is_some_and(|token|matches!(token.kind,TokenKind::BranchRole(_))) {
                let start=self.current_span();
                let kind=self.parse_type_ref()?;
                let resolved=crate::sum_owner::resolve_applied_sum_owner(&kind,start)
                    .map_err(|message|ParseError{span:start,message})?;
                self.expect_adjacent_dot(self.previous_span().end)?;
                let value=self.parse_sum_branch_value(resolved.key,Some(resolved.type_ref),start)?;
                return self.parse_primary_tail(value,allow_bare_bound_call);
            }
        }
        if let Some(end) = self.find_matching_rparen_after_current_lparen() {
            if let Some(provider) = self.tokens.get(end + 1).map(|token| token.raw.clone()) {
                if provider == "텐서로" || provider == "차림으로" {
                    return self.parse_tensor_provider(&provider);
                }
            }
        }
        if self.check(&TokenKind::LParen) && self.peek_tagged_template() {
            return self.parse_tagged_template();
        }
        if self.check(&TokenKind::LParen) && self.peek_tagged_formula() {
            return self.parse_tagged_formula();
        }
        let t = self.advance();
        let expr = match &t.kind {
            TokenKind::Integer(_) | TokenKind::Float(_) => {
                let value = Fixed64::parse_decimal_exact(&t.raw).ok_or_else(|| ParseError {
                    span: self.to_ast_span(t.span),
                    message: "E_FIXED64_LITERAL_RANGE: 수치 리터럴 범위 초과 또는 잘못된 10진 표기"
                        .to_string(),
                })?;
                let mut expr = Expr::new(
                    self.next_id(),
                    self.to_ast_span(t.span),
                    ExprKind::Literal(Literal::Fixed64(value)),
                );
                // Preserve the producer's token spelling before any Fixed64
                // rounding. B01 exact-context supply must use these digits,
                // never reverse-convert the evaluated numeric value.
                expr.numeric_literal_source = Some(NumericLiteralSource::from_token(
                    &t.raw, matches!(t.kind, TokenKind::Float(_)),
                ));
                expr
            }
            TokenKind::BytesLit(bytes) => Expr::new(
                self.next_id(),
                self.to_ast_span(t.span),
                ExprKind::Literal(Literal::Bytes(ddonirang_core::bytes::ByteSequence::new(bytes.clone()))),
            ),
            TokenKind::StringLit(s) => Expr::new(
                self.next_id(),
                self.to_ast_span(t.span),
                ExprKind::Literal(Literal::String(s.clone())),
            ),
            TokenKind::Atom(a) => {
                let id = self.next_id();
                let span = self.to_ast_span(t.span);
                self.atom_value_occurrences.insert(id, AtomValueOccurrence {span, label:a.clone(),role:AtomValueRole::LocalValue});
                Expr::new(id, span, ExprKind::Literal(Literal::Atom(a.clone())))
            },
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
            TokenKind::TypeArrow => {
                return Err(ParseError {
                    span: self.to_ast_span(t.span),
                    message: "E_FUNCTION_TYPE_ARROW_VALUE_POSITION: `-->`는 타입 위치에서만 사용할 수 있습니다".to_string(),
                });
            }
            TokenKind::BranchRole(_) => {
                return Err(ParseError {
                    span: self.to_ast_span(t.span),
                    message: "E_SUM_BRANCH_OWNER_REQUIRED: `$갈래` 앞에는 정확한 고름씨 이름이 필요합니다".to_string(),
                });
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
                if let Some(param) = self.try_parse_seed_literal_params()? {
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
                    if matches!(next.kind, TokenKind::Eof | TokenKind::RBrace) {
                        break;
                    }
                    if next.span.start != dot.span.end {
                        break;
                    }
                    self.advance();
                    if matches!(self.current().kind,TokenKind::BranchRole(_)) {
                        let owner = match &e.kind {
                            ExprKind::Var(owner) => owner.clone(),
                            _ => {
                                return Err(ParseError {
                                    span: e.span,
                                    message: "E_SUM_BRANCH_OWNER_REQUIRED: `$갈래` 앞에는 정확한 고름씨 이름이 필요합니다".to_string(),
                                })
                            }
                        };
                        e=self.parse_sum_branch_value(owner,None,e.span)?;
                        end = self.previous_span().end;
                        continue;
                    }
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
                            field_span: self.to_ast_span(field_token.span),
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
                let mut positional_comma = false;
                if !self.check(&TokenKind::RParen) {
                    loop {
                        if self.check_ident() && self.peek_is_colon() {
                            // RD090: a named pack cannot absorb and discard
                            // positional arguments or fixed call-pin bindings.
                            if !args.is_empty() {
                                return Err(self.error("E_PACK_MIXED_SHAPE: 위치값·고정 인자와 이름형 값꾸러미 필드를 섞을 수 없습니다"));
                            }
                            saw_pack = true;
                            let name = self.expect_ident("묶음 필드")?.raw.clone();
                            self.expect(&TokenKind::Colon, ":")?;
                            let value = self.parse_expr()?;
                            pack_fields.push((name, value));
                        } else if saw_pack {
                            return Err(self.error("E_PACK_MIXED_SHAPE: 이름형 값꾸러미 필드는 이름:값 형태여야 하며 위치값·고정 인자를 섞을 수 없습니다"));
                        } else {
                            let arg = self.parse_call_arg()?;
                            args.push(arg);
                        }
                        if self.check(&TokenKind::Comma) {
                            self.advance();
                            positional_comma = true;
                            if !saw_pack && self.check(&TokenKind::RParen) { break; }
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
                        if self.tokens.get(self.pos+1).is_some_and(|token|self.schema_header_names.contains(&token.raw)) {
                            self.advance();
                            let owner=self.expect_ident("명목 schema 이름")?;
                            let id=self.next_id();self.schema_construction_nodes.insert(id);
                            return Ok(Expr::new(id,span.merge(&self.to_ast_span(owner.span)),ExprKind::Call{func:owner.raw,args}));
                        }
                        self.advance();
                        let mut value_expr = self.parse_primary_inner(false)?;
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
                            && !positional_comma
                            && args[0].josa.is_none()
                            && args[0].resolved_pin.is_none()
                        {
                            args.into_iter().next().unwrap().expr
                        } else {
                            if args.iter().any(|arg| arg.josa.is_some() || arg.resolved_pin.is_some()) {
                                return Err(self.error("E_PACK_MIXED_SHAPE: 위치형 값꾸러미에는 조사나 고정 받는자리를 넣을 수 없습니다"));
                            }
                            Expr::new(self.next_id(),span,ExprKind::TupleConstructor {
                                elements:args.into_iter().map(|arg| arg.expr).collect(),
                            })
                        }
                    }
                }
            }
            _ => return Err(self.error("표현식")),
        };
        self.parse_primary_tail(expr, allow_bare_bound_call)
    }

    pub(super) fn parse_primary_tail(&mut self, mut expr: Expr, allow_bare_bound_call: bool) -> Result<Expr, ParseError> {
        expr = self.parse_index_suffix(expr)?;
        expr = self.parse_at_suffix(expr)?;
        expr = self.parse_not_suffix(expr)?;
        if allow_bare_bound_call {
            return self.try_parse_bare_bound_call(expr);
        }
        Ok(expr)
    }

    pub(super) fn try_parse_seed_literal_params(&mut self) -> Result<Option<String>, ParseError> {
        if self.check_ident() && self.peek_kind_n_is(1, |k| matches!(k, TokenKind::Pipe)) {
            let param = self.expect_ident("씨앗 인자")?.raw.clone();
            self.expect(&TokenKind::Pipe, "|")?;
            return Ok(Some(param));
        }
        if !self.check(&TokenKind::LParen) {
            return Ok(None);
        }
        let checkpoint = self.pos;
        self.advance();
        let mut params = Vec::new();
        loop {
            if !self.check_ident() {
                self.pos = checkpoint;
                return Ok(None);
            }
            params.push(self.expect_ident("씨앗 인자")?.raw.clone());
            if self.check(&TokenKind::Comma) {
                self.advance();
                continue;
            }
            break;
        }
        if !self.check(&TokenKind::RParen) {
            self.pos = checkpoint;
            return Ok(None);
        }
        self.advance();
        if !self.check(&TokenKind::Pipe) {
            self.pos = checkpoint;
            return Ok(None);
        }
        self.advance();
        Ok(Some(params.join(", ")))
    }
}
