//! Surface canonicalization parser; method bodies and owning AST types unchanged.
use super::*;

impl Parser {
    pub(super) fn new(tokens: Vec<Token>, bridge: bool) -> Self {
        Self {
            tokens,
            pos: 0,
            bridge,
            deprecated_block_header_colon_count: 0,
            seed_kind_stack: Vec::new(),
            chaebi_forbidden_depth: 0,
        }
    }

    pub(super) fn deprecated_block_header_colon_count(&self) -> usize {
        self.deprecated_block_header_colon_count
    }

    pub(super) fn unexpected_token_error(&self, context: &str) -> CanonError {
        CanonError::new(
            "E_CANON_UNEXPECTED_TOKEN",
            format!(
                "{}; at_token_index={} found={:?}",
                context,
                self.pos,
                self.peek_kind()
            ),
        )
    }

    pub(super) fn parse_program(&mut self) -> Result<Vec<SurfaceStmt>, CanonError> {
        let mut stmts = Vec::new();
        loop {
            self.skip_separators();
            if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
                break;
            }
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }
        Ok(stmts)
    }

    pub(super) fn in_imja_seed_body(&self) -> bool {
        self.seed_kind_stack
            .last()
            .is_some_and(|kind| kind == "임자")
    }

    pub(super) fn parse_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        if self.is_seed_def_start() {
            return self.parse_seed_def();
        }

        if let Some(kind) = self.peek_decl_block_kind() {
            return self.parse_decl_block(kind);
        }

        if self.peek_is(|k| matches!(k, TokenKind::Prompt)) {
            if self.peek_n_is(1, |k| matches!(k, TokenKind::Colon)) {
                return self.parse_prompt_choose_stmt();
            }
            if self.peek_n_is(1, |k| matches!(k, TokenKind::LBrace)) {
                return self.parse_prompt_block_stmt();
            }
        }

        if let Some(stmt) = self.try_parse_shared_bound_call_stmt()? {
            return Ok(stmt);
        }

        if self.peek_is_ident_colon() {
            let name = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let type_name = self.parse_decl_type_name()?;
            self.expect(TokenKind::Equals)?;
            let value = self.parse_expr()?;
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Decl {
                name,
                type_name,
                value,
            });
        }

        if self.is_open_block_start() {
            return self.parse_open_block_stmt();
        }

        if self.peek_is(|k| matches!(k, TokenKind::Repeat)) {
            return self.parse_repeat_stmt();
        }

        if self.peek_is(|k| matches!(k, TokenKind::Break)) {
            self.advance();
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Break);
        }
        if self.peek_is(|k| matches!(k, TokenKind::ContinueLoop)) {
            self.advance();
            self.consume_terminator()?;
            return Ok(SurfaceStmt::ContinueLoop);
        }

        if self.peek_is(|k| matches!(k, TokenKind::Goreugi)) {
            return self.parse_choose_stmt();
        }

        if self.is_hook_start() {
            return self.parse_hook_stmt();
        }

        if self.is_foreach_start() {
            return self.parse_foreach_stmt();
        }

        if self.is_boim_block_start() {
            return self.parse_boim_block_stmt();
        }

        if self.peek_is(|k| {
            matches!(
                k,
                TokenKind::BogaeMadangBlock(_) | TokenKind::BogaeJangmyeonBlock(_)
            )
        }) {
            return self.parse_bogae_madang_block_stmt();
        }
        if self.peek_is(|k| matches!(k, TokenKind::ExecPolicyBlock(_))) {
            return self.parse_exec_policy_block_stmt();
        }
        if self.peek_is(|k| matches!(k, TokenKind::JjaimBlock(_) | TokenKind::GuseongBlock(_))) {
            return self.parse_jjaim_block_stmt();
        }

        if self.is_bogae_draw_stmt() {
            self.advance();
            self.advance();
            self.consume_terminator()?;
            return Ok(SurfaceStmt::BogaeDraw);
        }

        if self.peek_is(|k| matches!(k, TokenKind::Manyak)) {
            return self.parse_manyak_if_stmt();
        }

        if self.peek_is(|k| matches!(k, TokenKind::LBrace)) {
            let condition = self.parse_condition_expr(false)?;
            if self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "이면")) {
                self.advance();
                return self.parse_if_stmt_from_condition(condition);
            }
            if self.peek_is(|k| matches!(k, TokenKind::Ilttae)) {
                return self.parse_if_stmt(condition);
            }
            if self.peek_is(|k| matches!(k, TokenKind::During)) {
                return self.parse_while_stmt(condition);
            }
            if self.peek_is(|k| matches!(k, TokenKind::Jeonjehae | TokenKind::Bojanghago)) {
                return self.parse_contract_stmt(condition);
            }
            if self.peek_is(|k| matches!(k, TokenKind::Prompt)) {
                return self.parse_prompt_condition_stmt(condition);
            }
            return Err(CanonError::new(
                "E_CANON_EXPECTED_ILTTTAE",
                "일때 또는 동안/바탕으로/다짐하고가 필요합니다.",
            ));
        }

        if let Some(stmt) = self.try_parse_event_react_stmt()? {
            return Ok(stmt);
        }

        if let Some(stmt) = self.try_parse_receive_stmt()? {
            return Ok(stmt);
        }

        let expr = self.parse_expr()?;
        let mut sender = None;
        let mut payload = expr;
        if self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "의")) {
            self.advance();
            sender = Some(payload);
            payload = self.parse_expr()?;
        }
        if self.peek_is(|k| matches!(k, TokenKind::SignalArrow)) {
            self.advance();
            let receiver = self.parse_expr()?;
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Send {
                sender,
                payload,
                receiver,
            });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Ilttae)) {
            return self.parse_if_stmt(Condition {
                expr: payload,
                style: ConditionStyle::Plain,
                negated: false,
            });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Jeonjehae | TokenKind::Bojanghago)) {
            return self.parse_contract_stmt(Condition {
                expr: payload,
                style: ConditionStyle::Plain,
                negated: false,
            });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Prompt)) {
            return self.parse_prompt_after_stmt(payload);
        }
        if self.peek_is(|k| matches!(k, TokenKind::PlusEqual | TokenKind::MinusEqual)) {
            return Err(CanonError::new(
                "E_CANON_UNSUPPORTED_COMPOUND_UPDATE",
                "+=/-=는 미지원입니다. +<-/ -<-를 사용하세요.",
            ));
        }

        if self.peek_is(|k| {
            matches!(
                k,
                TokenKind::Arrow | TokenKind::PlusArrow | TokenKind::MinusArrow
            )
        }) {
            let op = match self.peek_kind() {
                TokenKind::PlusArrow => Some(BinaryOp::Add),
                TokenKind::MinusArrow => Some(BinaryOp::Sub),
                _ => None,
            };
            self.advance();
            let mut consumed_dot = false;
            let value = if self.peek_is(|k| matches!(k, TokenKind::Dot)) {
                self.advance();
                consumed_dot = true;
                Expr::Literal(Literal::None)
            } else {
                self.parse_expr()?
            };

            let stmt = if let Some(op) = op {
                let Expr::Path(path) = payload else {
                    return Err(CanonError::new(
                        "E_CANON_UNSUPPORTED_COMPOUND_TARGET",
                        "복합 갱신(+<-, -<-)은 이름 대상만 허용됩니다.",
                    ));
                };
                let left = Expr::Path(path.clone());
                let value_expr = Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(value),
                };
                SurfaceStmt::Assign {
                    target: path,
                    value: value_expr,
                }
            } else {
                Self::lower_simple_assignment(payload, value)?
            };

            if !consumed_dot {
                self.consume_terminator()?;
            } else {
                self.skip_newlines();
            }
            return Ok(stmt);
        }
        if self.peek_is(|k| matches!(k, TokenKind::Show)) {
            self.advance();
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Show { value: payload });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Inspect)) {
            self.advance();
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Inspect { value: payload });
        }
        if self.peek_is(|k| {
            matches!(
                k,
                TokenKind::Ident(name)
                    if name == "되돌림" || name == "반환" || name == "돌려줘"
            )
        }) {
            self.advance();
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Return { value: payload });
        }
        if matches!(payload, Expr::Call { .. } | Expr::CallIn { .. }) {
            self.consume_terminator()?;
            return Ok(SurfaceStmt::Expr { value: payload });
        }
        Err(self.unexpected_token_error("예상하지 못한 토큰입니다"))
    }

    /// Consumes the shared-lang stable-pin canon (`값:핀 ... 호출꼬리.`).
    /// Pin selection has already happened in shared lang; this legacy plan
    /// adapter keeps parameter-order values for its existing call node while
    /// the public canon bytes remain the shared explicit form.
    pub(super) fn try_parse_shared_bound_call_stmt(&mut self) -> Result<Option<SurfaceStmt>, CanonError> {
        let checkpoint = self.pos;
        let mut args = Vec::new();
        loop {
            let value = match self.parse_expr() {
                Ok(value) => value,
                Err(_) => {
                    self.pos = checkpoint;
                    return Ok(None);
                }
            };
            if !self.peek_is(|kind| matches!(kind, TokenKind::Colon)) {
                self.pos = checkpoint;
                return Ok(None);
            }
            self.advance();
            if self.expect_ident().is_err() {
                self.pos = checkpoint;
                return Ok(None);
            }
            args.push(value);

            let is_call_tail = self.peek_is(|kind| matches!(kind, TokenKind::Ident(_)))
                && self.peek_n_is(1, |kind| {
                    matches!(
                        kind,
                        TokenKind::Dot
                            | TokenKind::Question
                            | TokenKind::Bang
                            | TokenKind::Newline
                            | TokenKind::RBrace
                            | TokenKind::Eof
                    )
                });
            if is_call_tail {
                let name = self.parse_call_name()?;
                self.consume_terminator()?;
                return Ok(Some(SurfaceStmt::Expr {
                    value: Expr::Call { name, args },
                }));
            }
        }
    }

    pub(super) fn try_parse_event_react_stmt(&mut self) -> Result<Option<SurfaceStmt>, CanonError> {
        if let Some(alias) = self.detect_forbidden_event_alias() {
            return Err(CanonError::new(
                "E_EVENT_SURFACE_ALIAS_FORBIDDEN",
                format!(
                    "비정본 이벤트 문형 금지({}) — 정본은 \"KIND\"라는 알림이 오면 {{ ... }}.",
                    alias
                ),
            ));
        }

        let TokenKind::String(kind) = self.peek_kind() else {
            return Ok(None);
        };
        if !self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(text) if text == "라는")) {
            return Ok(None);
        }
        if !self.peek_n_is(
            2,
            |k| matches!(k, TokenKind::Ident(text) if is_event_noun_canonical(text)),
        ) {
            return Ok(None);
        }
        if !self.peek_n_is(3, |k| matches!(k, TokenKind::Ident(text) if text == "오면")) {
            return Ok(None);
        }

        self.advance(); // "KIND"
        self.advance(); // 라는
        self.advance(); // 알림/알림이
        self.advance(); // 오면
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(Some(SurfaceStmt::EventReact { kind, body }))
    }

    pub(super) fn try_parse_receive_stmt(&mut self) -> Result<Option<SurfaceStmt>, CanonError> {
        let checkpoint = self.pos;
        let mut binding = None;
        let mut condition = None;

        if self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            if !self.has_receive_binding_head() {
                return Ok(None);
            }
            self.advance();
            let name = self.expect_ident()?;
            binding = Some(name);
            if !self.peek_is(|k| matches!(k, TokenKind::RParen)) {
                condition = Some(self.parse_expr()?);
            }
            self.expect(TokenKind::RParen)?;
            if !self.peek_is(|k| matches!(k, TokenKind::Ident(text) if text == "인")) {
                self.pos = checkpoint;
                return Ok(None);
            }
            self.advance();
        }

        let Some(kind) = self.peek_receive_kind_name() else {
            self.pos = checkpoint;
            return Ok(None);
        };
        self.advance();
        if !self.peek_is(|k| matches!(k, TokenKind::Ident(text) if text == "받으면")) {
            self.pos = checkpoint;
            return Ok(None);
        }
        if !self.in_imja_seed_body() {
            return Err(CanonError::new(
                "E_CANON_RECEIVE_OUTSIDE_IMJA",
                "`받으면` 훅은 `임자` 본문 안에서만 사용할 수 있습니다.",
            ));
        }
        self.advance();
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(Some(SurfaceStmt::Receive {
            kind,
            binding,
            condition,
            body,
        }))
    }

    pub(super) fn has_receive_binding_head(&self) -> bool {
        if !self.peek_is(|k| matches!(k, TokenKind::LParen)) {
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

    pub(super) fn peek_receive_kind_name(&self) -> Option<Option<String>> {
        let TokenKind::Ident(text) = self.peek_kind() else {
            return None;
        };
        let kind = strip_receive_object_particle(&text)?;
        if kind == "알림" {
            Some(None)
        } else {
            Some(Some(kind))
        }
    }

    pub(super) fn detect_forbidden_event_alias(&self) -> Option<&'static str> {
        match self.peek_kind() {
            TokenKind::Ident(ref text) if is_event_noun_any(text) => {
                if self.peek_n_is(1, |k| matches!(k, TokenKind::String(_))) {
                    return Some("prefix_form");
                }
            }
            TokenKind::String(_) => {
                if self.peek_n_is(1, |k| matches!(k, TokenKind::Ilttae))
                    || (self
                        .peek_n_is(1, |k| matches!(k, TokenKind::Ident(text) if text == "일때"))
                        && self.peek_n_is(2, |k| matches!(k, TokenKind::Colon)))
                {
                    return Some("ilttae_form");
                }
                if self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(text) if text == "라는"))
                    && self.peek_n_is(
                        2,
                        |k| matches!(k, TokenKind::Ident(text) if is_event_noun_alias(text)),
                    )
                {
                    return Some("noun_alias");
                }
            }
            _ => {}
        }
        None
    }

    pub(super) fn peek_decl_block_kind(&self) -> Option<DeclKind> {
        if !self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "채비")) {
            return None;
        }
        if self.peek_next_non_newline_is(|k| matches!(k, TokenKind::LBrace | TokenKind::Colon)) {
            return Some(DeclKind::Gureut);
        }
        None
    }

    pub(super) fn parse_decl_block(&mut self, _kind: DeclKind) -> Result<SurfaceStmt, CanonError> {
        let keyword = self.expect_ident()?;
        if keyword != "채비" {
            return Err(CanonError::new(
                "E_CANON_DECL_HEADER_ONLY_CHAEVI",
                "선언 블록 머릿말은 `채비 {`만 사용합니다",
            ));
        }
        self.consume_optional_block_header_colon();
        if self.chaebi_forbidden_depth > 0 {
            return Err(CanonError::new(
                "E_CHAEBI_IN_LOOP",
                "`채비 {}`는 루프/훅/순회 body 안에서 사용할 수 없습니다.",
            ));
        }
        self.expect(TokenKind::LBrace)?;

        let mut items = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek_is(|k| matches!(k, TokenKind::RBrace)) {
                break;
            }
            if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_RBRACE",
                    "닫는 중괄호가 필요합니다.",
                ));
            }

            let name = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let type_name = self.parse_decl_type_name()?;

            let mut value = None;
            let mut maegim = None;
            let mut kind = DeclKind::Gureut;
            if self.peek_is(|k| matches!(k, TokenKind::Arrow)) {
                self.advance();
                let (expr, item_maegim) = self.parse_decl_item_value_and_maegim()?;
                value = Some(expr);
                maegim = item_maegim;
            } else if self.peek_is(|k| matches!(k, TokenKind::Define)) {
                // V25 `채비` introduces a name with `:=`. It remains a
                // runtime setup control, distinct from the legacy `=` form.
                self.advance();
                let (expr, item_maegim) = self.parse_decl_item_value_and_maegim()?;
                value = Some(expr);
                maegim = item_maegim;
            } else if self.peek_is(|k| matches!(k, TokenKind::Equals)) {
                kind = DeclKind::Butbak;
                self.advance();
                let (expr, item_maegim) = self.parse_decl_item_value_and_maegim()?;
                value = Some(expr);
                maegim = item_maegim;
            }

            self.consume_terminator()?;
            items.push(DeclItem {
                name,
                kind,
                type_name,
                value,
                maegim,
            });
        }

        self.expect(TokenKind::RBrace)?;
        self.consume_terminator()?;
        Ok(SurfaceStmt::RootDecl { items })
    }

    pub(super) fn parse_decl_type_name(&mut self) -> Result<String, CanonError> {
        if self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            self.advance();
            let inner = self.expect_ident()?;
            self.expect(TokenKind::RParen)?;
            let outer = self.expect_ident()?;
            return Ok(format!("({inner}) {outer}"));
        }
        self.expect_ident()
    }

    pub(super) fn parse_decl_item_value_and_maegim(
        &mut self,
    ) -> Result<(Expr, Option<MaegimSpec>), CanonError> {
        if self.peek_is(|k| matches!(k, TokenKind::LParen)) {
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
            return Err(CanonError::new(
                "E_CANON_MAEGIM_GROUPED_VALUE_REQUIRED",
                "매김은 `(<식>) 매김 { ... }` 형태만 허용됩니다.",
            ));
        }
        Ok((expr, None))
    }

    pub(super) fn try_parse_grouped_expr_for_maegim(&mut self) -> Result<Expr, CanonError> {
        self.expect(TokenKind::LParen)?;
        let expr = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        Ok(expr)
    }

    pub(super) fn peek_maegim_keyword(&self) -> bool {
        self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "조건" || name == "매김"))
    }

    pub(super) fn is_supported_maegim_nested_section(name: &str) -> bool {
        matches!(name, "가늠" | "갈래")
    }

    pub(super) fn is_supported_maegim_nested_field(section: &str, field: &str) -> bool {
        match section {
            "가늠" => matches!(field, "범위" | "간격" | "분할수"),
            "갈래" => matches!(field, "가만히" | "벗남다룸"),
            _ => false,
        }
    }

    pub(super) fn parse_maegim_spec(&mut self) -> Result<MaegimSpec, CanonError> {
        let keyword = self.expect_ident()?;
        if keyword != "조건" && keyword != "매김" {
            return Err(CanonError::new(
                "E_CANON_BAD_MAEGIM",
                "매김 블록은 `조건 {` 또는 `매김 {`만 허용됩니다.",
            ));
        }
        self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();
        let mut has_step = false;
        let mut has_split_count = false;
        loop {
            self.skip_newlines();
            if self.peek_is(|k| matches!(k, TokenKind::RBrace)) {
                break;
            }
            if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_RBRACE",
                    "닫는 중괄호가 필요합니다.",
                ));
            }
            let name = self.expect_ident()?;
            self.skip_newlines();
            if self.peek_is(|k| matches!(k, TokenKind::LBrace)) {
                if !Self::is_supported_maegim_nested_section(&name) {
                    return Err(CanonError::new(
                        "E_CANON_MAEGIM_NESTED_SECTION_UNSUPPORTED",
                        format!("`매김` 중첩 섹션은 `가늠`/`갈래`만 허용됩니다: {}", name),
                    ));
                }
                self.expect(TokenKind::LBrace)?;
                loop {
                    self.skip_newlines();
                    if self.peek_is(|k| matches!(k, TokenKind::RBrace)) {
                        break;
                    }
                    if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
                        return Err(CanonError::new(
                            "E_CANON_EXPECTED_RBRACE",
                            "닫는 중괄호가 필요합니다.",
                        ));
                    }
                    let nested_name = self.expect_ident()?;
                    self.expect(TokenKind::Colon)?;
                    let value = self.parse_expr()?;
                    if !Self::is_supported_maegim_nested_field(&name, &nested_name) {
                        return Err(CanonError::new(
                            "E_CANON_MAEGIM_NESTED_FIELD_UNSUPPORTED",
                            format!(
                                "`매김` 섹션 `{}`에서 지원하지 않는 항목입니다: {}",
                                name, nested_name
                            ),
                        ));
                    }
                    if nested_name == "간격" {
                        has_step = true;
                    } else if nested_name == "분할수" {
                        has_split_count = true;
                    }
                    if has_step && has_split_count {
                        return Err(CanonError::new(
                            "E_CANON_MAEGIM_STEP_SPLIT_CONFLICT",
                            "`매김`에서 `간격`과 `분할수`는 동시에 사용할 수 없습니다.",
                        ));
                    }
                    self.consume_terminator()?;
                    fields.push(Binding {
                        name: format!("{}.{}", name, nested_name),
                        value,
                    });
                }
                self.expect(TokenKind::RBrace)?;
                if self.peek_is(|k| matches!(k, TokenKind::Dot | TokenKind::Newline)) {
                    self.consume_terminator()?;
                } else if !self.peek_is(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
                    return Err(CanonError::new(
                        "E_CANON_EXPECTED_TERMINATOR",
                        "문장 끝 구분자(점 또는 줄바꿈)가 필요합니다.",
                    ));
                }
            } else {
                self.expect(TokenKind::Colon)?;
                let value = self.parse_expr()?;
                if name == "간격" {
                    has_step = true;
                } else if name == "분할수" {
                    has_split_count = true;
                }
                if has_step && has_split_count {
                    return Err(CanonError::new(
                        "E_CANON_MAEGIM_STEP_SPLIT_CONFLICT",
                        "`매김`에서 `간격`과 `분할수`는 동시에 사용할 수 없습니다.",
                    ));
                }
                self.consume_terminator()?;
                fields.push(Binding { name, value });
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(MaegimSpec { fields })
    }

    pub(super) fn is_hook_start(&self) -> bool {
        if !self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            return false;
        }
        let mut idx = self.pos + 1;
        let number_head = self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Number(_)))
            .unwrap_or(false)
            && self
                .tokens
                .get(idx + 1)
                .map(|t| matches!(t.kind, TokenKind::Ident(_)))
                .unwrap_or(false);
        let head_is_ident = self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Ident(_)))
            .unwrap_or(false);
        if head_is_ident {
            idx += 1;
        } else if number_head {
            idx += 2;
        } else {
            return false;
        }
        if !self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::RParen))
            .unwrap_or(false)
        {
            return false;
        }
        idx += 1;
        let Some(Token { kind }) = self.tokens.get(idx) else {
            return false;
        };
        let suffix_ok = if number_head {
            matches!(kind, TokenKind::Ident(_))
        } else {
            matches!(kind, TokenKind::Ident(name) if name == "할때" || name == "마다")
        };
        if !suffix_ok {
            return false;
        }
        idx += 1;
        self.hook_header_has_block_start(idx)
    }

    pub(super) fn hook_header_has_block_start(&self, mut idx: usize) -> bool {
        while self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Newline))
            .unwrap_or(false)
        {
            idx += 1;
        }
        if self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Colon))
            .unwrap_or(false)
        {
            idx += 1;
            while self
                .tokens
                .get(idx)
                .map(|t| matches!(t.kind, TokenKind::Newline))
                .unwrap_or(false)
            {
                idx += 1;
            }
        }
        self.tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::LBrace))
            .unwrap_or(false)
    }

    pub(super) fn parse_hook_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::LParen)?;
        let mut number_head = false;
        let mut name = if self.peek_is(|k| matches!(k, TokenKind::Number(_))) {
            number_head = true;
            let raw = match self.advance().kind {
                TokenKind::Number(value) => value,
                _ => unreachable!("number branch must consume number token"),
            };
            let interval = raw
                .parse::<u64>()
                .ok()
                .filter(|value| *value > 0)
                .ok_or_else(|| {
                    CanonError::new(
                        "E_CANON_HOOK_EVERY_N_MADI_INTERVAL_INVALID",
                        "(N마디)마다에서 N은 양의 정수여야 합니다.",
                    )
                })?;
            let unit = match self.advance().kind {
                TokenKind::Ident(text) => text,
                other => {
                    return Err(CanonError::new(
                        "E_CANON_HOOK_EVERY_N_MADI_UNIT_UNSUPPORTED",
                        format!("(N마디)마다에서 단위는 `마디`만 허용됩니다: {other:?}"),
                    ))
                }
            };
            if unit != "마디" {
                return Err(CanonError::new(
                    "E_CANON_HOOK_EVERY_N_MADI_UNIT_UNSUPPORTED",
                    format!("(N마디)마다에서 단위는 `마디`만 허용됩니다: {unit}"),
                ));
            }
            format!("{}마디", interval)
        } else {
            self.expect_ident()?
        };
        self.expect(TokenKind::RParen)?;
        let raw_suffix = self.expect_ident()?;
        let suffix = match raw_suffix.as_str() {
            "할때" if number_head => {
                return Err(CanonError::new(
                    "E_CANON_HOOK_EVERY_N_MADI_SUFFIX_UNSUPPORTED",
                    format!("(N마디) 훅 접미는 `마다`만 허용됩니다: {raw_suffix}"),
                ))
            }
            "할때" => HookSuffix::Halttae,
            "마다" => HookSuffix::Mada,
            _ if number_head => {
                return Err(CanonError::new(
                    "E_CANON_HOOK_EVERY_N_MADI_SUFFIX_UNSUPPORTED",
                    format!("(N마디) 훅 접미는 `마다`만 허용됩니다: {raw_suffix}"),
                ))
            }
            _ => {
                return Err(CanonError::new(
                    "E_CANON_HOOK_SUFFIX",
                    "훅 접미는 할때/마다만 허용됩니다.",
                ))
            }
        };
        name = Self::canonicalize_hook_name(name, suffix);
        self.skip_newlines();
        if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
            return Err(CanonError::new(
                "E_BLOCK_HEADER_COLON_FORBIDDEN",
                "훅 블록 헤더의 `:` 표기는 허용되지 않습니다. `(시작)할때 { ... }`처럼 사용하세요.",
            ));
        }
        let body = self.parse_block_with_chaebi_restriction()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::Hook { name, suffix, body })
    }

    pub(super) fn canonicalize_hook_name(name: String, suffix: HookSuffix) -> String {
        match (name.as_str(), suffix) {
            ("처음", HookSuffix::Halttae) => "시작".to_string(),
            ("매틱", HookSuffix::Mada) => "매마디".to_string(),
            _ => name,
        }
    }

    pub(super) fn is_open_block_start(&self) -> bool {
        if !self.peek_is(|k| matches!(k, TokenKind::Ident(name) if is_open_block_keyword(name))) {
            return false;
        }
        self.peek_next_non_newline_is(|k| matches!(k, TokenKind::LBrace))
            || (self.peek_next_non_newline_is(|k| matches!(k, TokenKind::Colon))
                && self.peek_n_is(2, |k| matches!(k, TokenKind::LBrace)))
    }

    pub(super) fn parse_open_block_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        let name = self.expect_ident()?;
        if !is_open_block_keyword(&name) {
            return Err(CanonError::new(
                "E_CANON_OPEN_BLOCK",
                "너머 블록은 '너머'로 시작해야 합니다.",
            ));
        }
        if name != "너머" {
            return Err(CanonError::new(
                "E_EFFECT_SURFACE_ALIAS_FORBIDDEN",
                format!("비정본 너머 문형 금지({}) — 정본은 너머 {{ ... }}.", name),
            ));
        }
        self.consume_optional_block_header_colon();
        if !self.peek_is(|k| matches!(k, TokenKind::LBrace)) {
            return Err(CanonError::new(
                "E_CANON_EXPECTED_LBRACE",
                "너머 블록에는 '{'가 필요합니다.",
            ));
        }
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::OpenBlock { body })
    }

    pub(super) fn is_seed_def_start(&self) -> bool {
        let mut idx = self.pos;
        if self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::LParen))
            == Some(true)
        {
            let mut depth = 1usize;
            idx += 1;
            while idx < self.tokens.len() && depth > 0 {
                match self.tokens[idx].kind {
                    TokenKind::LParen => depth += 1,
                    TokenKind::RParen => depth -= 1,
                    _ => {}
                }
                idx += 1;
            }
            if depth != 0 {
                return false;
            }
            while self
                .tokens
                .get(idx)
                .map(|t| matches!(t.kind, TokenKind::Newline))
                .unwrap_or(false)
            {
                idx += 1;
            }
        }
        if !self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Ident(_)))
            .unwrap_or(false)
        {
            return false;
        }
        idx += 1;
        while self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Newline))
            .unwrap_or(false)
        {
            idx += 1;
        }
        if !self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Colon))
            .unwrap_or(false)
        {
            return false;
        }
        idx += 1;
        while self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Newline))
            .unwrap_or(false)
        {
            idx += 1;
        }
        if !self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Ident(_)))
            .unwrap_or(false)
        {
            return false;
        }
        idx += 1;
        while self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Newline))
            .unwrap_or(false)
        {
            idx += 1;
        }
        if !self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Equals))
            .unwrap_or(false)
        {
            return false;
        }
        idx += 1;
        while self
            .tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::Newline))
            .unwrap_or(false)
        {
            idx += 1;
        }
        self.tokens
            .get(idx)
            .map(|t| matches!(t.kind, TokenKind::LBrace))
            .unwrap_or(false)
    }

    pub(super) fn parse_seed_def(&mut self) -> Result<SurfaceStmt, CanonError> {
        let params = if self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            self.parse_params()?
        } else {
            Vec::new()
        };
        let name = self.expect_ident()?;
        self.expect(TokenKind::Colon)?;
        let kind = self.expect_ident()?;
        self.expect(TokenKind::Equals)?;
        self.seed_kind_stack.push(kind.clone());
        let body = self.parse_block();
        self.seed_kind_stack.pop();
        let body = body?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::SeedDef {
            name,
            kind,
            params,
            body,
        })
    }

    pub(super) fn parse_params(&mut self) -> Result<Vec<Param>, CanonError> {
        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        if self.peek_is(|k| matches!(k, TokenKind::RParen)) {
            self.advance();
            return Ok(params);
        }
        loop {
            let name = self.expect_ident()?;
            let mut type_name = None;
            let mut default = None;
            if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
                self.advance();
                type_name = Some(self.expect_ident()?);
            }
            if self.peek_is(|k| matches!(k, TokenKind::Equals)) {
                self.advance();
                default = Some(self.parse_expr()?);
            }
            params.push(Param {
                name,
                type_name,
                default,
            });
            if self.peek_is(|k| matches!(k, TokenKind::Comma)) {
                self.advance();
                continue;
            }
            break;
        }
        self.expect(TokenKind::RParen)?;
        Ok(params)
    }

    pub(super) fn expect_string(&mut self) -> Result<String, CanonError> {
        if let Some(TokenKind::String(value)) = self.peek() {
            self.advance();
            return Ok(value.clone());
        }
        Err(CanonError::new(
            "E_CANON_EXPECTED_STRING",
            "문자열이 필요합니다.",
        ))
    }

    pub(super) fn parse_repeat_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        self.advance();
        self.consume_optional_block_header_colon();
        let body = self.parse_block_with_chaebi_restriction()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::Repeat { body })
    }

    pub(super) fn parse_while_stmt(&mut self, condition: Condition) -> Result<SurfaceStmt, CanonError> {
        self.advance();
        if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
            return Err(CanonError::new(
                "E_WHILE_SURFACE",
                "동안 다음에는 콜론 없이 '{'가 와야 합니다.",
            ));
        }
        let body = self.parse_block_with_chaebi_restriction()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::While { condition, body })
    }

    pub(super) fn is_foreach_start(&self) -> bool {
        if !self.peek_is(|k| matches!(k, TokenKind::LParen)) {
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

    pub(super) fn parse_foreach_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        let item = self.parse_foreach_var()?;
        let mut iterable = self.parse_expr()?;
        let is_modu = self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "모두"));
        if is_modu {
            self.advance();
        } else {
            self.expect(TokenKind::Daehae)?;
            if let Expr::Path(path) = &mut iterable {
                if let Some(last) = path.segments.last_mut() {
                    if let Some(trimmed) = last.strip_suffix('에') {
                        if !trimmed.is_empty() {
                            *last = trimmed.to_string();
                        }
                    }
                }
            }
            self.consume_optional_block_header_colon();
        }
        let body = self.parse_block_with_chaebi_restriction()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::ForEach {
            item,
            iterable,
            body,
        })
    }

    pub(super) fn parse_foreach_var(&mut self) -> Result<String, CanonError> {
        self.expect(TokenKind::LParen)?;
        let name = self.expect_ident()?;
        if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.parse_decl_type_name()?;
        }
        self.expect(TokenKind::RParen)?;
        Ok(name)
    }

    pub(super) fn parse_expr(&mut self) -> Result<Expr, CanonError> {
        self.parse_pipe()
    }

    pub(super) fn consume_optional_block_header_colon(&mut self) -> bool {
        self.skip_newlines();
        if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
            self.deprecated_block_header_colon_count += 1;
            return true;
        }
        false
    }

    pub(super) fn parse_pipe(&mut self) -> Result<Expr, CanonError> {
        let mut expr = self.parse_logical_or()?;
        loop {
            let kind = match self.peek_ident_text() {
                Some(text) if text == "해서" => Some(PipeKind::Haseo),
                Some(text) if text == "하고" => Some(PipeKind::Hago),
                _ => None,
            };
            let Some(kind) = kind else { break };
            self.advance();
            let right = self.parse_logical_or()?;
            expr = Expr::Pipe {
                left: Box::new(expr),
                kind,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    pub(super) fn parse_logical_or(&mut self) -> Result<Expr, CanonError> {
        let mut expr = self.parse_logical_and()?;
        loop {
            let op = match self.peek() {
                Some(TokenKind::Or) => Some(BinaryOp::Or),
                Some(TokenKind::Ident(name)) if name == "또는" => Some(BinaryOp::Or),
                _ => None,
            };
            let Some(op) = op else { break };
            self.advance();
            let right = self.parse_logical_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    pub(super) fn parse_logical_and(&mut self) -> Result<Expr, CanonError> {
        let mut expr = self.parse_comparison()?;
        loop {
            let op = match self.peek() {
                Some(TokenKind::And) => Some(BinaryOp::And),
                Some(TokenKind::Ident(name)) if name == "그리고" => Some(BinaryOp::And),
                _ => None,
            };
            let Some(op) = op else { break };
            self.advance();
            let right = self.parse_comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    pub(super) fn parse_comparison(&mut self) -> Result<Expr, CanonError> {
        let mut expr = self.parse_range()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::RelationEq => Some(BinaryOp::RelationEq),
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
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    pub(super) fn parse_range(&mut self) -> Result<Expr, CanonError> {
        let left = self.parse_additive()?;
        let inclusive = match self.peek_kind() {
            TokenKind::DotDot => false,
            TokenKind::DotDotEq => true,
            _ => return Ok(left),
        };
        self.advance();
        let right = self.parse_additive()?;
        let flag = Expr::Literal(Literal::Num(if inclusive { "1" } else { "0" }.to_string()));
        Ok(Expr::Call {
            name: "표준.범위".to_string(),
            args: vec![left, right, flag],
        })
    }

    pub(super) fn parse_additive(&mut self) -> Result<Expr, CanonError> {
        let mut expr = self.parse_multiplicative()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus => Some(BinaryOp::Add),
                TokenKind::Minus => Some(BinaryOp::Sub),
                _ => None,
            };
            let Some(op) = op else { break };
            self.advance();
            let right = self.parse_multiplicative()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    pub(super) fn parse_multiplicative(&mut self) -> Result<Expr, CanonError> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Star => Some(BinaryOp::Mul),
                TokenKind::Slash => Some(BinaryOp::Div),
                TokenKind::Percent => Some(BinaryOp::Mod),
                _ => None,
            };
            let Some(op) = op else { break };
            self.advance();
            let right = self.parse_unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    pub(super) fn parse_unary(&mut self) -> Result<Expr, CanonError> {
        if self.peek_is(|k| matches!(k, TokenKind::Minus)) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
            });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Plus)) {
            self.advance();
            return self.parse_unary();
        }
        self.parse_postfix()
    }

    pub(super) fn parse_postfix(&mut self) -> Result<Expr, CanonError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.peek_is(|k| matches!(k, TokenKind::At)) {
                self.advance();
                let unit = self.expect_ident()?;
                expr = Expr::Unit {
                    value: Box::new(expr),
                    unit,
                };
                continue;
            }
            if self.peek_is(|k| matches!(k, TokenKind::LBracket)) {
                self.advance();
                let index_expr = self.parse_expr()?;
                self.expect(TokenKind::RBracket)?;
                expr = Expr::Call {
                    name: "차림.값".to_string(),
                    args: vec![expr, index_expr],
                };
                continue;
            }
            if let Some(field) = self.try_parse_dot_segment()? {
                expr = Expr::FieldAccess {
                    target: Box::new(expr),
                    field,
                };
                continue;
            }
            break;
        }
        Ok(expr)
    }

    pub(super) fn parse_primary(&mut self) -> Result<Expr, CanonError> {
        if let Some(TokenKind::PromptBlock(body)) = self.peek() {
            self.advance();
            return Ok(Expr::PromptBlock { body });
        }
        if self.peek_is(|k| matches!(k, TokenKind::Prompt)) {
            self.advance();
            if self.peek_is(|k| matches!(k, TokenKind::LParen)) {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Expr::PromptExpr {
                    expr: Box::new(expr),
                });
            }
            return Err(CanonError::new(
                "E_CANON_EXPECTED_EXPR",
                "?? 뒤에는 (표현식) 형태가 필요합니다.",
            ));
        }
        if let Some(TokenKind::Atom(text)) = self.peek() {
            self.advance();
            return Ok(Expr::Literal(Literal::Atom(text.clone())));
        }
        if let Some(TokenKind::Template(body)) = self.peek() {
            self.advance();
            return Ok(Expr::Template { body: body.clone() });
        }
        if let Some(TokenKind::Formula(body)) = self.peek() {
            self.advance();
            return Ok(Expr::Formula {
                tag: None,
                body: body.clone(),
            });
        }
        if self.peek_is(|k| matches!(k, TokenKind::LParen))
            && self.peek_n_is(1, |k| matches!(k, TokenKind::Atom(_)))
            && self.peek_n_is(2, |k| matches!(k, TokenKind::RParen))
            && self.peek_n_is(3, |k| matches!(k, TokenKind::Formula(_)))
        {
            self.expect(TokenKind::LParen)?;
            let tag = if let Some(TokenKind::Atom(text)) = self.peek() {
                self.advance();
                text.clone()
            } else {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_ATOM",
                    "수식 태그가 필요합니다.",
                ));
            };
            self.expect(TokenKind::RParen)?;
            let body = if let Some(TokenKind::Formula(text)) = self.peek() {
                self.advance();
                text.clone()
            } else {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_FORMULA",
                    "수식 블록이 필요합니다.",
                ));
            };
            return Ok(Expr::Formula {
                tag: Some(tag),
                body,
            });
        }
        if self.peek_is(|k| matches!(k, TokenKind::At)) {
            self.advance();
            let path = self.expect_string()?;
            return Ok(Expr::Resource(path));
        }
        if self.peek_is(|k| matches!(k, TokenKind::LBracket)) {
            self.advance();
            let mut args = Vec::new();
            if !self.peek_is(|k| matches!(k, TokenKind::RBracket)) {
                loop {
                    let expr = self.parse_expr()?;
                    args.push(expr);
                    if self.peek_is(|k| matches!(k, TokenKind::Comma)) {
                        self.advance();
                        continue;
                    }
                    break;
                }
            }
            self.expect(TokenKind::RBracket)?;
            return self.build_charim_literal(args);
        }

        if self.peek_is(|k| matches!(k, TokenKind::LBrace)) {
            self.advance();
            let Some(param) = self.try_parse_seed_literal_params()? else {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_SEED_PARAM",
                    "씨앗 인자가 필요합니다.",
                ));
            };
            let body = self.parse_expr()?;
            self.expect(TokenKind::RBrace)?;
            return Ok(Expr::SeedLiteral {
                param,
                body: Box::new(body),
            });
        }

        if self.peek_is(|k| matches!(k, TokenKind::LParen))
            && self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(_)))
            && self.peek_n_is(2, |k| matches!(k, TokenKind::Equals | TokenKind::Colon))
        {
            let bindings = self.parse_bindings()?;
            if let Some(text) = self.peek_ident_text() {
                if text == "인" {
                    self.advance();
                    let name = self.parse_call_name()?;
                    return Ok(Expr::CallIn { name, bindings });
                }
            }
            if let Some(TokenKind::Template(body)) = self.peek() {
                self.advance();
                return Ok(Expr::TemplateApply {
                    bindings,
                    body: body.clone(),
                });
            }
            if self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name != "의")) {
                let name = self.parse_call_name()?;
                return Ok(Expr::Call {
                    name,
                    args: vec![Expr::Pack { bindings }],
                });
            }
            return Ok(Expr::Pack { bindings });
        }

        if self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            self.advance();
            let mut args = Vec::new();
            if !self.peek_is(|k| matches!(k, TokenKind::RParen)) {
                loop {
                    let expr = self.parse_expr()?;
                    args.push(expr);
                    if self.peek_is(|k| matches!(k, TokenKind::Comma)) {
                        self.advance();
                        continue;
                    }
                    break;
                }
            }
            self.expect(TokenKind::RParen)?;
            if self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name != "의")) {
                let name = self.parse_call_name()?;
                return Ok(Expr::Call { name, args });
            }
            if args.len() == 1 {
                return Ok(args.into_iter().next().unwrap());
            }
            return Err(CanonError::new(
                "E_CANON_EXPECTED_EXPR",
                "호출 이름이 필요합니다.",
            ));
        }

        if self.bridge {
            if let Some(prefix) = self.peek_ident_text() {
                if is_bridge_alias_type_name(&prefix) {
                    self.advance();
                    let literal = self.parse_literal()?;
                    let ok = bridge_alias_matches_literal(&prefix, &literal);
                    if !ok {
                        return Err(CanonError::new(
                            "E_CANON_BAD_ALIAS",
                            "접두 타입이 맞지 않습니다.",
                        ));
                    }
                    return Ok(Expr::Literal(literal));
                }
                if self.peek_next_is_literal() {
                    return Err(CanonError::new(
                        "E_CANON_BAD_ALIAS",
                        format!("알 수 없는 별칭입니다: {}", prefix),
                    ));
                }
            }
        }

        match self.peek_kind() {
            TokenKind::String(text) => {
                self.advance();
                Ok(Expr::Literal(Literal::Str(text)))
            }
            TokenKind::Number(value) => {
                self.advance();
                Ok(Expr::Literal(Literal::Num(value)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            TokenKind::None => {
                self.advance();
                Ok(Expr::Literal(Literal::None))
            }
            TokenKind::Ident(_) => {
                let path = self.parse_path()?;
                Ok(Expr::Path(path))
            }
            _ => Err(CanonError::new(
                "E_CANON_EXPECTED_EXPR",
                "표현식이 필요합니다.",
            )),
        }
    }

    pub(super) fn try_parse_seed_literal_params(&mut self) -> Result<Option<String>, CanonError> {
        if self.peek_is(|k| matches!(k, TokenKind::Ident(_)))
            && self.peek_n_is(1, |k| matches!(k, TokenKind::Pipe))
        {
            let param = self.expect_ident()?;
            self.expect(TokenKind::Pipe)?;
            return Ok(Some(param));
        }
        if !self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            return Ok(None);
        }
        let checkpoint = self.pos;
        self.advance();
        let mut params = Vec::new();
        loop {
            if !self.peek_is(|k| matches!(k, TokenKind::Ident(_))) {
                self.pos = checkpoint;
                return Ok(None);
            }
            params.push(self.expect_ident()?);
            if self.peek_is(|k| matches!(k, TokenKind::Comma)) {
                self.advance();
                continue;
            }
            break;
        }
        if !self.peek_is(|k| matches!(k, TokenKind::RParen)) {
            self.pos = checkpoint;
            return Ok(None);
        }
        self.advance();
        if !self.peek_is(|k| matches!(k, TokenKind::Pipe)) {
            self.pos = checkpoint;
            return Ok(None);
        }
        self.advance();
        Ok(Some(params.join(", ")))
    }

    pub(super) fn parse_bindings(&mut self) -> Result<Vec<Binding>, CanonError> {
        self.expect(TokenKind::LParen)?;
        let mut bindings = Vec::new();
        if !self.peek_is(|k| matches!(k, TokenKind::RParen)) {
            loop {
                let name = self.expect_ident()?;
                if !self.peek_is(|k| matches!(k, TokenKind::Equals | TokenKind::Colon)) {
                    return Err(CanonError::new(
                        "E_CANON_EXPECTED_EQUALS",
                        "'=' 또는 ':'가 필요합니다.",
                    ));
                }
                self.advance();
                let value = self.parse_expr()?;
                bindings.push(Binding { name, value });
                if self.peek_is(|k| matches!(k, TokenKind::Comma)) {
                    self.advance();
                    continue;
                }
                break;
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(bindings)
    }

    pub(super) fn build_charim_literal(&self, args: Vec<Expr>) -> Result<Expr, CanonError> {
        let has_inner = args
            .iter()
            .any(|arg| matches!(arg, Expr::Call { name, .. } if name == "차림"));
        if !has_inner {
            return Ok(Expr::Call {
                name: "차림".to_string(),
                args,
            });
        }

        let mut rows: Vec<Vec<Expr>> = Vec::new();
        for arg in args {
            match arg {
                Expr::Call { name, args } if name == "차림" => {
                    if args
                        .iter()
                        .any(|item| matches!(item, Expr::Call { name, .. } if name == "차림"))
                    {
                        return Err(CanonError::new(
                            "E_CANON_TENSOR_SHAPE",
                            "중첩 차림은 같은 길이의 2차 차림이어야 합니다.",
                        ));
                    }
                    rows.push(args);
                }
                _ => {
                    return Err(CanonError::new(
                        "E_CANON_TENSOR_SHAPE",
                        "중첩 차림은 같은 길이의 2차 차림이어야 합니다.",
                    ))
                }
            }
        }

        let row_count = rows.len();
        let col_count = rows.first().map(|row| row.len()).unwrap_or(0);
        for row in &rows {
            if row.len() != col_count {
                return Err(CanonError::new(
                    "E_CANON_TENSOR_SHAPE",
                    "중첩 차림은 같은 길이의 2차 차림이어야 합니다.",
                ));
            }
        }

        let mut flat = Vec::new();
        for row in rows {
            flat.extend(row);
        }

        let shape = Expr::Call {
            name: "차림".to_string(),
            args: vec![
                Expr::Literal(Literal::Num(row_count.to_string())),
                Expr::Literal(Literal::Num(col_count.to_string())),
            ],
        };
        let data = Expr::Call {
            name: "차림".to_string(),
            args: flat,
        };
        let layout = Expr::Literal(Literal::Str("가로먼저".to_string()));
        let bindings = vec![
            Binding {
                name: "형상".to_string(),
                value: shape,
            },
            Binding {
                name: "자료".to_string(),
                value: data,
            },
            Binding {
                name: "배치".to_string(),
                value: layout,
            },
        ];
        Ok(Expr::Pack { bindings })
    }

    pub(super) fn parse_literal(&mut self) -> Result<Literal, CanonError> {
        match self.peek_kind() {
            TokenKind::String(text) => {
                self.advance();
                Ok(Literal::Str(text))
            }
            TokenKind::Number(value) => {
                self.advance();
                Ok(Literal::Num(value))
            }
            TokenKind::True => {
                self.advance();
                Ok(Literal::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(Literal::Bool(false))
            }
            TokenKind::None => {
                self.advance();
                Ok(Literal::None)
            }
            _ => Err(CanonError::new(
                "E_CANON_EXPECTED_LITERAL",
                "리터럴이 필요합니다.",
            )),
        }
    }

    pub(super) fn parse_manyak_if_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::Manyak)?;
        let condition = self.parse_condition_expr(false)?;
        self.expect(TokenKind::Iramyeon)?;
        self.parse_if_stmt_from_condition(condition)
    }

    pub(super) fn parse_if_stmt_from_condition(
        &mut self,
        condition: Condition,
    ) -> Result<SurfaceStmt, CanonError> {
        let then_body = self.parse_block()?;
        if let Some(lookahead) = self.find_if_else_lookahead() {
            self.pos = lookahead;
        } else {
            self.skip_newlines();
        }
        if self.peek_is(|k| matches!(k, TokenKind::Anigo)) {
            self.advance();
            self.skip_newlines();
            let else_condition = self.parse_condition_expr(false)?;
            self.expect(TokenKind::Ilttae)?;
            let stmt = self.parse_if_stmt(else_condition)?;
            return Ok(SurfaceStmt::If {
                condition,
                then_body,
                else_body: Some(vec![stmt]),
            });
        }
        let else_body = if self.peek_is(|k| matches!(k, TokenKind::Aniramyeon)) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        self.consume_optional_terminator();
        Ok(SurfaceStmt::If {
            condition,
            then_body,
            else_body,
        })
    }

    pub(super) fn find_if_else_lookahead(&self) -> Option<usize> {
        let mut lookahead = self.pos;
        while self
            .tokens
            .get(lookahead)
            .map(|token| matches!(token.kind, TokenKind::Newline))
            .unwrap_or(false)
        {
            lookahead += 1;
        }
        if matches!(
            self.tokens.get(lookahead).map(|token| &token.kind),
            Some(TokenKind::Dot)
        ) {
            lookahead += 1;
            while self
                .tokens
                .get(lookahead)
                .map(|token| matches!(token.kind, TokenKind::Newline))
                .unwrap_or(false)
            {
                lookahead += 1;
            }
        }
        if self
            .tokens
            .get(lookahead)
            .map(|token| matches!(token.kind, TokenKind::Anigo | TokenKind::Aniramyeon))
            .unwrap_or(false)
        {
            return Some(lookahead);
        }
        None
    }

    pub(super) fn parse_if_stmt(&mut self, condition: Condition) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::Ilttae)?;
        self.parse_if_stmt_from_condition(condition)
    }

    pub(super) fn parse_contract_stmt(&mut self, condition: Condition) -> Result<SurfaceStmt, CanonError> {
        let kind = if self.peek_is(|k| matches!(k, TokenKind::Jeonjehae)) {
            self.advance();
            ContractKind::Pre
        } else {
            self.advance();
            ContractKind::Post
        };
        let mode = self.parse_contract_mode()?;
        self.skip_newlines();
        if !self.peek_is(|k| matches!(k, TokenKind::Aniramyeon)) {
            return Err(CanonError::new(
                "E_CANON_CONTRACT_NEEDS_ELSE",
                "계약에는 아니면 절이 필요합니다.",
            ));
        }
        self.advance();
        let else_body = self.parse_block()?;
        self.skip_newlines();
        let then_body = if self.peek_is(|k| matches!(k, TokenKind::Majeumyeon)) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        self.consume_optional_terminator();
        Ok(SurfaceStmt::Contract {
            kind,
            mode,
            condition,
            then_body,
            else_body,
        })
    }

    pub(super) fn parse_choose_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::Goreugi)?;
        self.expect(TokenKind::Colon)?;
        let mut branches = Vec::new();
        let mut else_body: Option<Vec<SurfaceStmt>> = None;
        let mut exhaustive = false;

        loop {
            self.skip_newlines();
            if self.peek_is(|k| matches!(k, TokenKind::Aniramyeon)) {
                self.advance();
                self.expect(TokenKind::Colon)?;
                else_body = Some(self.parse_block()?);
                break;
            }
            if self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "그밖의"))
                && self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(name) if name == "경우"))
            {
                self.advance();
                self.advance();
                else_body = Some(self.parse_block()?);
                break;
            }
            if self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "모든"))
                && self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(name) if name == "경우"))
                && self.peek_n_is(2, |k| matches!(k, TokenKind::Ident(name) if name == "다룸"))
            {
                self.advance();
                self.advance();
                self.advance();
                exhaustive = true;
                break;
            }
            if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
                break;
            }
            let condition = self.parse_choose_branch_condition()?;
            if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
                self.advance();
            } else if self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "인"))
                && self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(name) if name == "경우"))
            {
                self.advance();
                self.advance();
            } else {
                self.expect(TokenKind::Colon)?;
            }
            let body = self.parse_block()?;
            branches.push(SurfaceChooseBranch { condition, body });
        }

        if else_body.is_none() && !exhaustive {
            return Err(CanonError::new(
                "E_CANON_CHOOSE_NEEDS_ELSE",
                "고르기에는 아니면/그밖의 경우 절 또는 모든 경우 다룸. 표지가 필요합니다.",
            ));
        }
        self.consume_optional_terminator();
        Ok(SurfaceStmt::Choose {
            branches,
            else_body,
            exhaustive,
        })
    }

    pub(super) fn parse_choose_branch_condition(&mut self) -> Result<Condition, CanonError> {
        if !self.peek_is(|k| matches!(k, TokenKind::LBrace)) {
            let expr = self.parse_expr()?;
            return Ok(Condition {
                expr,
                style: ConditionStyle::Plain,
                negated: false,
            });
        }
        self.expect(TokenKind::LBrace)?;
        self.skip_newlines();
        let expr = self.parse_expr()?;
        if self.peek_is(|k| matches!(k, TokenKind::Dot)) {
            self.advance();
        }
        self.skip_newlines();
        self.expect(TokenKind::RBrace)?;
        self.skip_newlines();
        if self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "인"))
            && self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(name) if name == "경우"))
        {
            return Ok(Condition {
                expr,
                style: ConditionStyle::Plain,
                negated: false,
            });
        }
        let name = self.expect_ident()?;
        let negated = match name.as_str() {
            "인것" => false,
            "아닌것" => true,
            _ => {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_THUNK_SUFFIX",
                    "조건은 { ... }인것/아닌것 또는 { ... } 인 경우 형태여야 합니다.",
                ))
            }
        };
        Ok(Condition {
            expr,
            style: ConditionStyle::Thunk,
            negated,
        })
    }

    pub(super) fn parse_prompt_choose_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::Prompt)?;
        self.expect(TokenKind::Colon)?;
        let mut branches = Vec::new();
        let mut else_body: Option<Vec<SurfaceStmt>> = None;

        loop {
            self.skip_newlines();
            if self.peek_is(|k| matches!(k, TokenKind::Prompt)) {
                self.advance();
                if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
                    self.advance();
                    else_body = Some(self.parse_block()?);
                    break;
                }
                return Err(CanonError::new(
                    "E_CANON_PROMPT_ELSE",
                    "??: 뒤에는 :가 필요합니다.",
                ));
            }
            if self.peek_is(|k| matches!(k, TokenKind::Eof | TokenKind::RBrace)) {
                break;
            }
            let condition = self.parse_condition_expr(true)?;
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            branches.push(SurfaceChooseBranch { condition, body });
        }

        self.consume_optional_terminator();
        Ok(SurfaceStmt::PromptChoose {
            branches,
            else_body,
        })
    }

    pub(super) fn parse_prompt_after_stmt(&mut self, value: Expr) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::Prompt)?;
        if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
        }
        self.skip_newlines();
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::PromptAfter { value, body })
    }

    pub(super) fn parse_prompt_condition_stmt(
        &mut self,
        condition: Condition,
    ) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::Prompt)?;
        self.skip_newlines();
        if self.peek_is(|k| matches!(k, TokenKind::Prompt)) {
            self.advance();
            if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
                self.advance();
            }
            self.skip_newlines();
        } else if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
            self.skip_newlines();
        }
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::PromptCondition { condition, body })
    }

    pub(super) fn parse_prompt_block_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        self.expect(TokenKind::Prompt)?;
        if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
        }
        self.skip_newlines();
        let body = self.parse_block()?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::PromptBlock { body })
    }

    pub(super) fn parse_contract_mode(&mut self) -> Result<ContractMode, CanonError> {
        if !self.peek_is(|k| matches!(k, TokenKind::LParen)) {
            return Ok(ContractMode::Abort);
        }
        self.expect(TokenKind::LParen)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::RParen)?;
        match name.as_str() {
            "알림" => Ok(ContractMode::Alert),
            "물림" => Ok(ContractMode::Abort),
            _ => Err(CanonError::new(
                "E_CANON_CONTRACT_MODE",
                "계약 모드는 알림 또는 물림이어야 합니다.",
            )),
        }
    }

    pub(super) fn parse_block(&mut self) -> Result<Vec<SurfaceStmt>, CanonError> {
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek_is(|k| matches!(k, TokenKind::RBrace)) {
                break;
            }
            if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_RBRACE",
                    "닫는 중괄호가 필요합니다.",
                ));
            }
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(stmts)
    }

    pub(super) fn parse_block_with_chaebi_restriction(&mut self) -> Result<Vec<SurfaceStmt>, CanonError> {
        self.chaebi_forbidden_depth += 1;
        let body = self.parse_block();
        self.chaebi_forbidden_depth = self.chaebi_forbidden_depth.saturating_sub(1);
        body
    }

    pub(super) fn parse_condition_expr(&mut self, require_thunk: bool) -> Result<Condition, CanonError> {
        if !self.peek_is(|k| matches!(k, TokenKind::LBrace)) {
            if require_thunk {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_THUNK_CONDITION",
                    "조건은 { ... }인것 형태여야 합니다.",
                ));
            }
            let expr = self.parse_expr()?;
            return Ok(Condition {
                expr,
                style: ConditionStyle::Plain,
                negated: false,
            });
        }
        self.expect(TokenKind::LBrace)?;
        self.skip_newlines();
        let expr = self.parse_expr()?;
        if self.peek_is(|k| matches!(k, TokenKind::Dot)) {
            self.advance();
        }
        self.skip_newlines();
        self.expect(TokenKind::RBrace)?;
        self.skip_newlines();
        if !require_thunk && self.peek_is(|k| matches!(k, TokenKind::Ident(name) if name == "이면")) {
            return Ok(Condition { expr, style: ConditionStyle::Current, negated: false });
        }
        let name = self.expect_ident()?;
        if self.peek_is(|k| matches!(k, TokenKind::During)) {
            if name != "인" {
                return Err(CanonError::new(
                    "E_WHILE_SURFACE",
                    "일반 반복은 '{ 조건 }인 동안 { ... }.'로 작성해야 합니다.",
                ));
            }
            return Ok(Condition {
                expr,
                style: ConditionStyle::Thunk,
                negated: false,
            });
        }
        let negated = match name.as_str() {
            "인것" => false,
            "아닌것" => true,
            _ => {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_THUNK_SUFFIX",
                    "조건은 { ... }인것/아닌것 형태여야 합니다.",
                ))
            }
        };
        Ok(Condition {
            expr,
            style: ConditionStyle::Thunk,
            negated,
        })
    }

    pub(super) fn parse_path(&mut self) -> Result<Path, CanonError> {
        let first = self.expect_ident()?;
        let mut segments = vec![first];
        loop {
            let Some(segment) = self.try_parse_dot_segment()? else {
                break;
            };
            segments.push(segment);
        }
        Ok(Path { segments })
    }

    pub(super) fn try_parse_dot_segment(&mut self) -> Result<Option<String>, CanonError> {
        if !self.peek_is(|k| matches!(k, TokenKind::Dot)) {
            return Ok(None);
        }
        if self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(_))) {
            self.advance();
            return Ok(Some(self.expect_ident()?));
        }
        let Some(next) = self
            .tokens
            .get(self.pos + 1)
            .map(|token| token.kind.clone())
        else {
            return Ok(None);
        };
        let TokenKind::Number(raw) = next else {
            return Ok(None);
        };
        if !raw.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(CanonError::new(
                "E_CANON_TUPLE_INDEX_INVALID",
                "점 접근 인덱스는 정수(.0/.1 등)만 허용됩니다.",
            ));
        }
        self.advance();
        self.advance();
        Ok(Some(raw))
    }

    // Step B/C scope: allow one-level + nested map write.
    // - allowed: 살림.공.x, 바탕.공.x, 공.x, 공.속도.x, 살림.공.속도.x
    pub(super) fn lower_simple_assignment(target_expr: Expr, value: Expr) -> Result<SurfaceStmt, CanonError> {
        match target_expr {
            Expr::Call { name, args } if name == "차림.값" && args.len() == 2 => {
                let target = match &args[0] {
                    Expr::Path(path)
                        if path.segments.len() == 2
                            && matches!(path.segments[0].as_str(), "살림" | "바탕") =>
                    {
                        path.clone()
                    }
                    _ => {
                        return Err(CanonError::new(
                            "E_CANON_EXPECTED_TARGET",
                            "대상이 필요합니다.",
                        ))
                    }
                };
                let index_expr = args[1].clone();
                let value_call = Expr::Call {
                    name: "차림.바꾼값".to_string(),
                    args: vec![Expr::Path(target.clone()), index_expr, value],
                };
                Ok(SurfaceStmt::Assign {
                    target,
                    value: value_call,
                })
            }
            Expr::FieldAccess { target, field } => {
                let Expr::Path(path) = *target else {
                    return Err(CanonError::new(
                        "E_CANON_EXPECTED_TARGET",
                        "점 대입 대상은 이름만 허용됩니다.",
                    ));
                };
                let key_expr = Expr::Literal(Literal::Str(field));
                let value_call = Expr::Call {
                    name: "짝맞춤.바꾼값".to_string(),
                    args: vec![Expr::Path(path.clone()), key_expr, value],
                };
                Ok(SurfaceStmt::Assign {
                    target: path,
                    value: value_call,
                })
            }
            Expr::Path(path) => {
                if let Some((target, key_segments)) = Self::split_map_dot_target(&path) {
                    let value_call = Self::build_map_dot_write_expr(
                        Expr::Path(target.clone()),
                        &key_segments,
                        value,
                    );
                    Ok(SurfaceStmt::Assign {
                        target,
                        value: value_call,
                    })
                } else {
                    Ok(SurfaceStmt::Assign {
                        target: path,
                        value,
                    })
                }
            }
            _ => Err(CanonError::new(
                "E_CANON_EXPECTED_TARGET",
                "대상이 필요합니다.",
            )),
        }
    }

    pub(super) fn split_map_dot_target(path: &Path) -> Option<(Path, Vec<String>)> {
        if path.segments.len() < 2 {
            return None;
        }
        let has_root = matches!(
            path.segments.first().map(|seg| seg.as_str()),
            Some("살림" | "바탕" | "샘")
        );
        if has_root {
            if path.segments.len() < 3 {
                return None;
            }
            let target = Path {
                segments: vec![path.segments[0].clone(), path.segments[1].clone()],
            };
            return Some((target, path.segments[2..].to_vec()));
        }
        let target = Path {
            segments: vec![path.segments[0].clone()],
        };
        Some((target, path.segments[1..].to_vec()))
    }

    pub(super) fn build_map_dot_write_expr(map_expr: Expr, key_segments: &[String], value: Expr) -> Expr {
        debug_assert!(!key_segments.is_empty());
        let key_expr = Expr::Literal(Literal::Str(key_segments[0].clone()));
        if key_segments.len() == 1 {
            return Expr::Call {
                name: "짝맞춤.바꾼값".to_string(),
                args: vec![map_expr, key_expr, value],
            };
        }
        let child_map_expr = Expr::Call {
            name: "짝맞춤.필수값".to_string(),
            args: vec![map_expr.clone(), key_expr.clone()],
        };
        let child_value = Self::build_map_dot_write_expr(child_map_expr, &key_segments[1..], value);
        Expr::Call {
            name: "짝맞춤.바꾼값".to_string(),
            args: vec![map_expr, key_expr, child_value],
        }
    }

    pub(super) fn parse_call_name(&mut self) -> Result<String, CanonError> {
        let mut name = self.expect_ident()?;
        loop {
            if self.peek_is(|k| matches!(k, TokenKind::Dot))
                && self.peek_n_is(1, |k| matches!(k, TokenKind::Ident(_)))
            {
                self.advance();
                let ident = self.expect_ident()?;
                name.push('.');
                name.push_str(&ident);
                continue;
            }
            if self.peek_is(|k| matches!(k, TokenKind::Ident(_))) {
                if self.peek_is(
                    |k| matches!(k, TokenKind::Ident(name) if name == "해서" || name == "하고"),
                ) {
                    break;
                }
                let ident = self.expect_ident()?;
                name.push(' ');
                name.push_str(&ident);
                continue;
            }
            break;
        }
        Ok(name)
    }

    pub(super) fn consume_terminator(&mut self) -> Result<(), CanonError> {
        if self.peek_is(|k| matches!(k, TokenKind::Dot | TokenKind::Question | TokenKind::Bang)) {
            self.advance();
            self.skip_newlines();
            return Ok(());
        }
        if self.peek_is(|k| matches!(k, TokenKind::Newline)) {
            self.skip_newlines();
            return Ok(());
        }
        if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
            return Ok(());
        }
        Err(CanonError::new(
            "E_CANON_EXPECTED_TERMINATOR",
            "문장 끝에 마침표가 필요합니다.",
        ))
    }

    pub(super) fn consume_optional_terminator(&mut self) {
        if self.peek_is(|k| matches!(k, TokenKind::Dot | TokenKind::Question | TokenKind::Bang)) {
            self.advance();
        }
        self.skip_newlines();
    }

    pub(super) fn skip_separators(&mut self) {
        loop {
            if self.peek_is(|k| {
                matches!(
                    k,
                    TokenKind::Newline | TokenKind::Dot | TokenKind::Question | TokenKind::Bang
                )
            }) {
                self.advance();
                continue;
            }
            break;
        }
    }

    pub(super) fn skip_newlines(&mut self) {
        while self.peek_is(|k| matches!(k, TokenKind::Newline)) {
            self.advance();
        }
    }

    pub(super) fn is_bogae_draw_stmt(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Ident(ref name) if name == "보개로")
            && self.peek_n_is(
                1,
                |k| matches!(k, TokenKind::Ident(ref name) if name == "그려"),
            )
    }

    pub(super) fn is_boim_block_start(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Ident(ref name) if name == "보임")
            && (self.peek_n_is(1, |k| matches!(k, TokenKind::LBrace))
                || (self.peek_n_is(1, |k| matches!(k, TokenKind::Colon))
                    && self.peek_n_is(2, |k| matches!(k, TokenKind::LBrace))))
    }

    pub(super) fn parse_boim_block_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        self.advance();
        if self.peek_is(|k| matches!(k, TokenKind::Colon)) {
            self.advance();
        }
        self.expect(TokenKind::LBrace)?;
        self.skip_newlines();
        let mut entries = Vec::new();
        while !self.peek_is(|k| matches!(k, TokenKind::RBrace)) {
            if self.peek_is(|k| matches!(k, TokenKind::Eof)) {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_RBRACE",
                    "보임 블록 끝에 `}`가 필요합니다.",
                ));
            }
            let name = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let value = self.parse_expr()?;
            if !self.peek_is(|k| matches!(k, TokenKind::Dot)) {
                return Err(CanonError::new(
                    "E_CANON_EXPECTED_TERMINATOR",
                    "보임 항목 끝에 마침표가 필요합니다.",
                ));
            }
            self.advance();
            self.skip_newlines();
            entries.push(Binding { name, value });
        }
        self.expect(TokenKind::RBrace)?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::Boim { entries })
    }

    pub(super) fn parse_bogae_madang_block_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        let token = self.advance();
        let body = match token.kind {
            TokenKind::BogaeMadangBlock(body) => body,
            TokenKind::BogaeJangmyeonBlock(body) => body,
            _ => return Err(self.unexpected_token_error("보개마당/보개장면 블록 파싱 실패")),
        };
        self.consume_optional_terminator();
        Ok(SurfaceStmt::BogaeMadangBlock { body })
    }

    pub(super) fn parse_jjaim_block_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        let token = self.advance();
        let body = match token.kind {
            TokenKind::JjaimBlock(body) => body,
            TokenKind::GuseongBlock(body) => body,
            _ => return Err(self.unexpected_token_error("짜임/구성 블록 파싱 실패")),
        };
        validate_jjaim_block_body(&body)?;
        self.consume_optional_terminator();
        Ok(SurfaceStmt::JjaimBlock { body })
    }

    pub(super) fn parse_exec_policy_block_stmt(&mut self) -> Result<SurfaceStmt, CanonError> {
        let token = self.advance();
        let body = match token.kind {
            TokenKind::ExecPolicyBlock(body) => body,
            _ => return Err(self.unexpected_token_error("너머 블록 파싱 실패")),
        };
        self.consume_optional_terminator();
        Ok(SurfaceStmt::ExecPolicyBlock { body })
    }

    pub(super) fn expect_ident(&mut self) -> Result<String, CanonError> {
        match self.peek_kind() {
            TokenKind::Ident(text) => {
                self.advance();
                Ok(text)
            }
            _ => Err(CanonError::new(
                "E_CANON_EXPECTED_IDENT",
                "이름이 필요합니다.",
            )),
        }
    }

    pub(super) fn expect(&mut self, expected: TokenKind) -> Result<(), CanonError> {
        let current = self.peek_kind();
        if std::mem::discriminant(&current) == std::mem::discriminant(&expected) {
            self.advance();
            return Ok(());
        }
        Err(self.unexpected_token_error("토큰 기대값 불일치"))
    }

    pub(super) fn peek_ident_text(&self) -> Option<String> {
        match self.peek_kind() {
            TokenKind::Ident(text) => Some(text),
            _ => None,
        }
    }

    pub(super) fn peek_next_is_literal(&self) -> bool {
        self.peek_n_is(1, |k| {
            matches!(
                k,
                TokenKind::String(_)
                    | TokenKind::Number(_)
                    | TokenKind::True
                    | TokenKind::False
                    | TokenKind::None
            )
        })
    }

    pub(super) fn peek_is_ident_colon(&self) -> bool {
        self.peek_is(|k| matches!(k, TokenKind::Ident(_)))
            && self.peek_n_is(1, |k| matches!(k, TokenKind::Colon))
    }

    pub(super) fn peek_kind(&self) -> TokenKind {
        self.tokens
            .get(self.pos)
            .map(|token| token.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    pub(super) fn peek(&self) -> Option<TokenKind> {
        self.tokens.get(self.pos).map(|token| token.kind.clone())
    }

    pub(super) fn peek_is(&self, f: impl FnOnce(&TokenKind) -> bool) -> bool {
        f(&self.peek_kind())
    }

    pub(super) fn peek_n_is(&self, n: usize, f: impl FnOnce(&TokenKind) -> bool) -> bool {
        self.tokens
            .get(self.pos + n)
            .map(|token| f(&token.kind))
            .unwrap_or(false)
    }

    pub(super) fn peek_next_non_newline_is(&self, f: impl FnOnce(&TokenKind) -> bool) -> bool {
        let mut idx = self.pos + 1;
        while let Some(token) = self.tokens.get(idx) {
            if !matches!(token.kind, TokenKind::Newline) {
                return f(&token.kind);
            }
            idx += 1;
        }
        false
    }

    pub(super) fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }
}

