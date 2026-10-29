//! Existing callable declarations, forms, parameters and type syntax; all method bodies and identities retained.
use super::*;

impl Parser {
    pub(super) fn is_root_statement_start(&self) -> bool {
        match &self.current().kind {
            TokenKind::Ident(_) | TokenKind::Josa(_) => {
                let v25 = self.source_epoch == SourceEpoch::V25;
                if v25 && self.peek_kind_n_is(1, |kind| matches!(kind, TokenKind::Colon)) {
                    // Typed values use the statement grammar, including applied
                    // types. Preserve callable kinds and declaration bodies.
                    for (index, token) in self.tokens.iter().enumerate().skip(self.pos + 2) {
                        match token.kind {
                            TokenKind::KwImeumssi
                            | TokenKind::KwUmjikssi
                            | TokenKind::KwValueFunc
                            | TokenKind::KwHeureumssi
                            | TokenKind::KwGallaessi
                            | TokenKind::KwRelationssi
                            | TokenKind::KwSam
                            | TokenKind::KwIeumssi
                            | TokenKind::KwSemssi => return false,
                            TokenKind::Define | TokenKind::Equals => {
                                return !matches!(
                                    self.tokens.get(index + 1).map(|next| &next.kind),
                                    Some(TokenKind::LBrace)
                                );
                            }
                            TokenKind::Dot | TokenKind::Eof => return false,
                            _ => {}
                        }
                    }
                    return false;
                }
                if v25
                    && !self.peek_kind_n_is(1, |kind| {
                        matches!(kind, TokenKind::Colon | TokenKind::LParen | TokenKind::LBrace)
                    })
                {
                    return true;
                }
                self.peek_kind_n_is(1, |kind| {
                    matches!(
                        kind,
                        TokenKind::Define
                            | TokenKind::Arrow
                            | TokenKind::RightArrow
                            | TokenKind::PlusArrow
                            | TokenKind::MinusArrow
                            | TokenKind::PlusEqual
                            | TokenKind::MinusEqual
                        | TokenKind::Dot
                        | TokenKind::KwBoyeojugi
                            | TokenKind::KwTolabogi
                    ) || (v25 && matches!(kind, TokenKind::Equals))
                })
            }
            // Callable headers begin with an identifier or parameter list.
            // Other statement heads (condition/loop/control and braced
            // expression conditions) are root-program statements.
            TokenKind::LParen if self.source_epoch == SourceEpoch::V25 => {
                let mut depth = 0usize;
                for (index, token) in self.tokens.iter().enumerate().skip(self.pos) {
                    match token.kind {
                        TokenKind::LParen => depth += 1,
                        TokenKind::RParen => {
                            depth -= 1;
                            if depth == 0 {
                                // A declaration continues as `(pins) name:` or
                                // `(pins) name{alternate~...}:`.
                                // Empty and ordinary argument lists belong to
                                // the root statement/call parser.
                                if !matches!(
                                    self.tokens.get(index + 1).map(|token| &token.kind),
                                    Some(TokenKind::Ident(_) | TokenKind::Josa(_))
                                ) {
                                    return true;
                                }
                                if matches!(
                                    self.tokens.get(index + 2).map(|token| &token.kind),
                                    Some(TokenKind::Colon)
                                ) {
                                    return false;
                                }
                                if matches!(
                                    self.tokens.get(index + 2).map(|token| &token.kind),
                                    Some(TokenKind::LBrace)
                                ) {
                                    for (close_index, token) in
                                        self.tokens.iter().enumerate().skip(index + 3)
                                    {
                                        match token.kind {
                                            TokenKind::RBrace => {
                                                return !matches!(
                                                    self.tokens
                                                        .get(close_index + 1)
                                                        .map(|next| &next.kind),
                                                    Some(TokenKind::Colon)
                                                );
                                            }
                                            TokenKind::Eof => return true,
                                            _ => {}
                                        }
                                    }
                                }
                                return true;
                            }
                        }
                        TokenKind::Eof => break,
                        _ => {}
                    }
                }
                false
            }
            TokenKind::LParen => {
                let mut depth = 0;
                for (offset, token) in self.tokens[self.pos..].iter().enumerate() {
                    match token.kind {
                        TokenKind::LParen => depth += 1,
                        TokenKind::RParen => {
                            depth -= 1;
                            if depth == 0 {
                                return !self.peek_kind_n_is(offset + 2, |kind| matches!(kind, TokenKind::Colon));
                            }
                        }
                        _ => {}
                    }
                }
                false
            }
            _ => true,
        }
    }
    pub(super) fn is_vc04_schema_start(&self) -> bool {
        self.is_vc04_schema_header_at(self.pos)
    }
    pub(super) fn is_vc04_schema_header_at(&self,index:usize)->bool {
        let Some(row)=self.tokens.get(index..index+5)else{return false};
        self.source_epoch==SourceEpoch::V25&&matches!(row[0].kind,TokenKind::Ident(_))
            &&matches!(row[1].kind,TokenKind::Colon)&&matches!(row[2].raw.as_str(),"임자"|"이름씨")
            &&matches!(row[3].kind,TokenKind::Define)&&matches!(row[4].kind,TokenKind::LBrace)
            &&!(row[2].raw=="임자"&&self.is_existing_reaction_owner_body(index+5))
    }
    /// Preserve the already-supported RD-293/445 owner body, not a permissive
    /// fallback for malformed slot schemas. The actual legacy parser remains
    /// responsible for admitting its receiver/provider declaration.
    pub(super) fn is_existing_reaction_owner_body(&self,start:usize)->bool {
        if !self.tokens.get(start).is_some_and(|t|matches!(t.kind,TokenKind::LParen)){return false;}
        let mut depth=0;
        for (index,token) in self.tokens.iter().enumerate().skip(start){
            match token.kind{
                TokenKind::LParen=>depth+=1,
                TokenKind::RParen=>{depth-=1;if depth==0 {
                    let raw=|offset:usize|self.tokens.get(index+offset).map(|t|t.raw.as_str());
                    let provider=self.tokens.get(index+2).is_some_and(|t|matches!(t.kind,TokenKind::Colon))
                        &&raw(3)==Some("움직씨")&&self.tokens.get(index+4).is_some_and(|t|matches!(t.kind,TokenKind::LBrace));
                    let receiver=matches!(raw(1),Some("를"|"을"))&&raw(2)==Some("받으면");
                    return provider||receiver;
                }},
                TokenKind::Dot|TokenKind::Eof=>return false,
                _=>{},
            }
        }
        false
    }

    pub(super) fn parse_vc04_schema(&mut self) -> Result<SeedDef, ParseError> {
        let start = self.current_span();
        let name = self.expect_ident("schema 이름")?.raw.clone();
        self.expect(&TokenKind::Colon, ":")?;
        let category = match self.advance().raw.as_str() {
            "임자" => SchemaCategory::Subject,
            "이름씨" => SchemaCategory::NominalValue,
            _ => return Err(self.error("E_SCHEMA_CATEGORY: 임자 또는 이름씨가 필요합니다")),
        };
        self.expect(&TokenKind::Define, ":=")?;
        self.expect(&TokenKind::LBrace, "{")?;
        let mut params = Vec::new();
        let mut names = HashSet::new();
        while !self.check(&TokenKind::RBrace) {
            let slot_start = self.current_span();
            let pin_name = self.expect_ident("schema 성분 이름")?.raw.clone();
            if !names.insert(pin_name.clone()) {
                return Err(self.error("E_SCHEMA_SLOT_DUPLICATE: 같은 성분 이름을 중복 선언할 수 없습니다"));
            }
            self.expect(&TokenKind::Colon, ":")?;
            let type_ref = self.parse_type_ref()?;
            if type_ref == TypeRef::Infer {
                return Err(self.error("E_SCHEMA_SLOT_TYPE: 성분의 씨를 명시해야 합니다"));
            }
            self.expect(&TokenKind::Dot, "schema 성분 선언의 마침표 `.`")?;
            params.push(ParamPin { id: self.next_id(), span: slot_start.merge(&self.previous_span()),
                pin_name, type_ref, default_value: None, optional: false, josa_list: Vec::new() });
        }
        self.expect(&TokenKind::RBrace, "}")?;
        self.expect(&TokenKind::Dot, "schema 정의의 마침표 `.`")?;
        Ok(SeedDef { type_params: Vec::new(), proof_header: None, id: self.next_id(), span: start.merge(&self.previous_span()),
            canonical_name: name, alternate_forms: Vec::new(), seed_kind: SeedKind::Schema(category),
            params, result_type: None, body: None, modifiers: Vec::new() })
    }

    pub(super) fn parse_seed_def(&mut self) -> Result<SeedDef, ParseError> {
        if self.check_ident() && self.peek_kind_n_is(1, |kind| matches!(kind, TokenKind::LParen)) {
            return self.parse_named_single_expression_callable();
        }
        let start = self.current_span();
        let mut proof_header = None;
        let mut type_params = Vec::new();
        if self.source_epoch == SourceEpoch::V25
            && self.find_matching_rparen_after_current_lparen().is_some_and(|end|
                self.tokens.get(end + 1).is_some_and(|token| token.raw == "두고"))
        {
            self.expect(&TokenKind::LParen, "(")?;
            let mut propositions = Vec::new();
            loop {
                let span = self.current_span();
                let name = self.expect_ident("정적 매개변수")?.raw.clone();
                self.expect(&TokenKind::Colon, ":")?;
                let kind = self.expect_ident("세움 또는 씨자리")?.raw.clone();
                if !matches!(kind.as_str(), "세움" | "씨자리") || name == "_" {
                    return Err(self.error("E_PROOF_STATIC_PARAMETER: 명시적인 세움 매개변수가 필요합니다"));
                }
                if propositions.iter().any(|(prior, _)| prior == &name)
                    || type_params.iter().any(|prior: &TypeParamDef| prior.name == name) {
                    return Err(self.error("E_GENERIC_DUPLICATE_PARAMETER: 정적 매개변수 이름이 중복됩니다"));
                }
                let span = span.merge(&self.previous_span());
                if kind == "씨자리" {
                    type_params.push(TypeParamDef { id: self.next_id(), name, span });
                } else {
                    propositions.push((name, span));
                }
                if !self.check(&TokenKind::Comma) { break; }
                self.advance();
            }
            self.expect(&TokenKind::RParen, ")")?;
            self.expect_ident("두고")?;
            if !propositions.is_empty() {
                if !type_params.is_empty() {
                    return Err(self.error("E_GENERIC_PROOF_HEADER_UNSUPPORTED: 씨자리와 세움을 함께 쓰는 머리는 아직 지원하지 않습니다"));
                }
                proof_header = Some(ProofHeader { propositions, evidence_bindings: Vec::new() });
            }
        }
        let params = if self.check(&TokenKind::LParen) {
            self.parse_params_with_proof_header(proof_header.as_mut())?
        } else {
            Vec::new()
        };
        // R3 has an already named structure goal, rather than a fresh P:세움
        // binder. Preserve the actual authored pin/local binding for consumers.
        // This is parser provenance, not admission or checker acceptance.
        if proof_header.is_none() && params.iter().any(|pin| matches!(
            &pin.type_ref, TypeRef::Applied { name, args }
                if name == "보임" && matches!(args.as_slice(), [TypeRef::StructureLaw { .. } | TypeRef::AlgebraEquality { .. }])
        )) {
            proof_header = Some(ProofHeader {
                propositions: Vec::new(),
                evidence_bindings: params.iter().map(|pin| ProofEvidenceBinding {
                    public_pin: pin.pin_name.clone(), local_name: pin.pin_name.clone(), span: pin.span,
                }).collect(),
            });
        }
        let name = self.expect_ident("씨앗 이름")?.raw.clone();
        let alternate_forms =
            if self.source_epoch == SourceEpoch::V25 && self.check(&TokenKind::LBrace) {
                self.parse_callable_alternate_forms(&name)?
            } else {
                Vec::new()
            };
        self.expect(&TokenKind::Colon, ":")?;
        // Full callable declarations place an explicit result type between the
        // preferred name and the callable kind:
        // `(pins) name:Result:셈씨 := { ... }`.
        let has_result_type = self.source_epoch == SourceEpoch::V25
            && self.tokens[self.pos..]
                .iter()
                .take_while(|token| {
                    !matches!(
                        token.kind,
                        TokenKind::Define | TokenKind::Equals | TokenKind::LBrace | TokenKind::Eof
                    )
                })
                .any(|token| matches!(token.kind, TokenKind::Colon));
        let result_type = if has_result_type {
            let result_type = self.parse_type_ref()?;
            self.expect(&TokenKind::Colon, ":")?;
            Some(result_type)
        } else {
            None
        };
        let kind = self.parse_seed_kind()?;
        if kind != SeedKind::Product && alternate_forms.is_empty() && !self.first_callable_ddl {
            self.validate_seed_name_tail(&name, start)?;
        }
        if result_type.is_some() && !matches!(kind, SeedKind::Semssi | SeedKind::Umjikssi) {
            return Err(self.error("결과 타입이 있는 셈씨 또는 움직씨 선언"));
        }
        if self.source_epoch == SourceEpoch::V25 && self.check(&TokenKind::Equals) {
            return Err(ParseError {
                span: self.current_span(),
                message: "E_V25_DEFINITION_MIGRATION_REQUIRED: V25 정의에는 `:=`가 필요하며 이전 `=` 원문은 명시적 마이그레이션 입력으로 분리해야 합니다".to_string(),
            });
        }
        match self.source_epoch {
            SourceEpoch::V25 => self.expect(&TokenKind::Define, ":=")?,
            SourceEpoch::PreV25 => self.expect(&TokenKind::Equals, "=")?,
        };
        if self.first_callable_ddl {
            self.expect(&TokenKind::LBrace, "DDL 본문 생략의 {")?;
            self.expect(&TokenKind::DotDot, "DDL 본문 생략의 ..")?;
            self.expect(&TokenKind::Dot, "DDL 본문 생략의 .")?;
            self.expect(&TokenKind::RBrace, "DDL 본문 생략의 }")?;
            self.expect(&TokenKind::Dot, "DDL 선언 마침표")?;
            return Ok(SeedDef {
                type_params, proof_header, id: self.next_id(), span: start.merge(&self.previous_span()),
                canonical_name: name, alternate_forms, seed_kind: kind, params, result_type,
                body: None, modifiers: Vec::new(),
            });
        }
        if result_type.is_some() && !self.check(&TokenKind::LBrace) {
            return Err(self.error("결과 타입이 있는 선언의 `{ ... }` 본문"));
        }
        self.enter_scope();
        if let Some(result_type) = &result_type {
            self.declare_typed_name(&name, result_type);
        }
        for param in &params {
            let local_name = proof_header.as_ref().and_then(|header|
                header.evidence_bindings.iter().find(|binding| binding.public_pin == param.pin_name))
                .map(|binding| binding.local_name.as_str()).unwrap_or(&param.pin_name);
            self.declare_typed_name(local_name, &param.type_ref);
        }
        self.seed_kind_stack.push(kind.clone());
        let body_result = if self.check(&TokenKind::LBrace) {
            let body = self.parse_body();
            match body {
                Ok(body) => {
                    // V25의 `이름:씨 := { ... }.` 종결자는 선언에 속한다.
                    // root statement로 넘기거나 본문 span에 포함하지 않는다.
                    if kind != SeedKind::Product
                        && self.source_epoch == SourceEpoch::V25 && self.check(&TokenKind::Dot) {
                        self.advance();
                    }
                    Ok(body)
                }
                Err(err) => {
                    self.seed_kind_stack.pop();
                    self.exit_scope();
                    return Err(err);
                }
            }
        } else {
            let expr = self.parse_expr()?;
            let mood = self.consume_optional_terminator()?;
            Ok(Body {
                id: self.next_id(),
                span: expr.span,
                stmts: vec![Stmt::Return {
                    id: self.next_id(),
                    span: expr.span,
                    mood,
                    value: expr,
                }],
            })
        };
        self.seed_kind_stack.pop();
        let body = body_result?;
        if kind == SeedKind::Product {
            if self.source_epoch == SourceEpoch::V25 {
                self.expect(&TokenKind::Dot, "묶음씨 정의의 마침표 `.`")?;
            }
            if !body.stmts.is_empty() {
                return Err(ParseError { span: body.span, message:
                    "E_PRODUCT_SCHEMA_BODY: 묶음씨 본문은 실행문 없는 빈 schema여야 합니다".into() });
            }
            let mut names = HashSet::new();
            for field in &params {
                if !names.insert(&field.pin_name) {
                    return Err(ParseError { span: field.span, message:
                        format!("E_PRODUCT_FIELD_DUPLICATE: 필드 `{}`가 중복되었습니다", field.pin_name) });
                }
                if field.type_ref == TypeRef::Infer || field.optional || field.default_value.is_some()
                    || !field.josa_list.is_empty() {
                    return Err(ParseError { span: field.span, message:
                        "E_PRODUCT_FIELD_SCHEMA: 묶음씨 필드는 명시적 `이름:씨`이며 callable 받는자리·기본값을 갖지 않습니다".into() });
                }
            }
        }
        self.exit_scope();
        Ok(SeedDef {
            type_params,
            proof_header,
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            canonical_name: name,
            alternate_forms,
            seed_kind: kind,
            params,
            result_type,
            body: Some(body),
            modifiers: Vec::new(),
        })
    }

    /// RD-406 current surface: `name (pin:type, ... --> result) := expr.`
    ///
    /// This remains a named callable (`SeedKind::ValueFunc`) instead of being
    /// lowered into an ordinary binding or an anonymous expression.
    pub(super) fn parse_named_single_expression_callable(&mut self) -> Result<SeedDef, ParseError> {
        let start = self.current_span();
        let name = self.expect_ident("호출가능 정의 이름")?.raw.clone();
        let conditional_body = self.tokens[self.pos..].windows(3)
            .take_while(|pair| !matches!(pair[0].kind, TokenKind::Eof))
            .find(|pair| matches!(pair[0].kind, TokenKind::Define))
            .is_some_and(|pair| pair[1].raw == "조건별" && matches!(pair[2].kind,TokenKind::LBrace));
        if !conditional_body { self.validate_seed_name_tail(&name, self.previous_span())?; }
        self.expect(&TokenKind::LParen, "(")?;

        let mut params = Vec::new();
        loop {
            if self.check(&TokenKind::TypeArrow) {
                break;
            }
            let param_start = self.current_span();
            let pin_name = self.expect_ident("호출가능 매개변수")?.raw.clone();
            self.expect(&TokenKind::Colon, ":")?;
            let type_ref = match self.parse_type_operand()? {
                ParsedTypeOperand::Single(type_ref) => type_ref,
                ParsedTypeOperand::FunctionParams(_) => {
                    return Err(self.error("호출가능 매개변수 타입"));
                }
            };
            let mut conditional_josa = Vec::new();
            while conditional_body && self.check(&TokenKind::Tilde) {
                self.advance();
                if let TokenKind::Josa(josa) = &self.current().kind {
                    conditional_josa.push(josa.clone()); self.advance();
                } else { return Err(self.error("조건별 입력의 조사")); }
            }
            params.push(ParamPin {
                id: self.next_id(),
                span: param_start.merge(&self.previous_span()),
                pin_name,
                type_ref,
                default_value: None,
                optional: false,
                josa_list: conditional_josa,
            });
            if self.check(&TokenKind::Comma) {
                self.advance();
                continue;
            }
            if self.check(&TokenKind::TypeArrow) {
                break;
            }
            return Err(self.error("`,` 또는 `-->`"));
        }
        self.expect(&TokenKind::TypeArrow, "-->")?;
        let result_type = match self.parse_type_operand()? {
            ParsedTypeOperand::Single(type_ref) => type_ref,
            ParsedTypeOperand::FunctionParams(_) => return Err(self.error("호출가능 결과 타입")),
        };
        self.expect(&TokenKind::RParen, ")")?;
        self.expect(&TokenKind::Define, ":=")?;
        if conditional_body {
            return self.parse_piecewise_definition(start,name,Vec::new(),SeedKind::ValueFunc,
                params,Some(result_type),Vec::new(),None);
        }
        if self.check(&TokenKind::LBrace) {
            return Err(ParseError {
                span: self.current_span(),
                message: "E_RD406_SINGLE_EXPRESSION_BODY_REQUIRED: named callable 본문은 하나의 식이어야 합니다".to_string(),
            });
        }

        self.enter_scope();
        for param in &params {
            self.declare_typed_name(&param.pin_name, &param.type_ref);
        }
        self.seed_kind_stack.push(SeedKind::ValueFunc);
        let expr = self.parse_expr();
        self.seed_kind_stack.pop();
        let expr = match expr {
            Ok(expr) => expr,
            Err(error) => {
                self.exit_scope();
                return Err(error);
            }
        };
        let mood = match self.consume_stmt_terminator() {
            Ok(mood) => mood,
            Err(error) => {
                self.exit_scope();
                return Err(error);
            }
        };
        self.exit_scope();
        let expr_span = expr.span;
        Ok(SeedDef {
            type_params: Vec::new(),
            proof_header: None,
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            canonical_name: name,
            alternate_forms: Vec::new(),
            seed_kind: SeedKind::ValueFunc,
            params,
            result_type: Some(result_type),
            body: Some(Body {
                id: self.next_id(),
                span: expr_span,
                stmts: vec![Stmt::Return {
                    id: self.next_id(),
                    span: expr_span,
                    mood,
                    value: expr,
                }],
            }),
            modifiers: Vec::new(),
        })
    }

    pub(super) fn parse_callable_alternate_forms(
        &mut self,
        canonical_name: &str,
    ) -> Result<Vec<String>, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::LBrace, "{")?;
        let mut forms = Vec::new();
        loop {
            let token = self.expect_ident("명시 꼴바꿈")?;
            let form = token.raw.clone();
            let form_span = self.previous_span();
            if form == canonical_name || forms.iter().any(|existing| existing == &form) {
                return Err(ParseError {
                    span: form_span,
                    message: format!(
                        "E_CALLABLE_SURFACE_DUPLICATE: '{}' 표면이 같은 정의에 중복 등록되었습니다",
                        form
                    ),
                });
            }
            forms.push(form);
            if self.check(&TokenKind::Tilde) {
                self.advance();
                continue;
            }
            break;
        }
        self.expect(&TokenKind::RBrace, "}")
            .map_err(|_| ParseError {
                span: start,
                message: "E_CALLABLE_SURFACE_SET_UNCLOSED: 명시 꼴바꿈은 `}`로 닫아야 합니다"
                    .to_string(),
            })?;
        Ok(forms)
    }

    pub(super) fn bind_registered_forms(&mut self, program: &CanonProgram) -> Result<(), ParseError> {
        let preferred: HashSet<String> = program
            .items
            .iter()
            .map(|item| {
                let TopLevelItem::SeedDef(seed) = item;
                seed.canonical_name.clone()
            })
            .collect();
        let builtins: HashSet<String> = minimal_stdlib_sigs()
            .iter()
            .map(|signature| signature.name.to_string())
            .collect();

        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            self.registered_definitions.insert(seed.id, seed.clone());
            if seed.alternate_forms.is_empty() {
                continue;
            }
            if !crate::ordinary_callable::is_v1_ordinary_callable(seed) {
                return Err(ParseError {
                    span: seed.span,
                    message: "E_REGISTERED_FORM_CONTEXT: 현재 등록 표면은 호출가능 정의 문맥에서만 허용됩니다"
                        .to_string(),
                });
            }
            self.explicitly_registered_owners
                .insert(seed.canonical_name.clone());
            for form in std::iter::once(&seed.canonical_name).chain(&seed.alternate_forms) {
                if builtins.contains(form)
                    || (form != &seed.canonical_name && preferred.contains(form))
                    || self
                        .registered_forms
                        .get(form)
                        .is_some_and(|owner| owner != &seed.canonical_name)
                {
                    return Err(ParseError {
                        span: seed.span,
                        message: format!(
                            "E_CALLABLE_SURFACE_AMBIGUOUS: '{}' 표면에 둘 이상의 정의 소유자가 있습니다",
                            form
                        ),
                    });
                }
                self.registered_forms
                    .insert(form.clone(), seed.canonical_name.clone());
                self.registered_form_candidates
                    .entry(form.clone())
                    .or_default()
                    .push(seed.id);
            }
        }

        // The preferred spelling denotes the complete overload family. Each
        // alternate remains the exact subset that explicitly registered it.
        for owner in &self.explicitly_registered_owners {
            let candidates = program
                .items
                .iter()
                .filter_map(|item| {
                    let TopLevelItem::SeedDef(seed) = item;
                    (&seed.canonical_name == owner).then_some(seed.id)
                })
                .collect();
            self.registered_form_candidates
                .insert(owner.clone(), candidates);
        }
        Ok(())
    }
    pub(super) fn parse_params(&mut self) -> Result<Vec<ParamPin>, ParseError> {
        self.parse_params_with_proof_header(None)
    }

    fn parse_params_with_proof_header(&mut self, mut header: Option<&mut ProofHeader>) -> Result<Vec<ParamPin>, ParseError> {
        self.expect(&TokenKind::LParen, "(")?;
        let mut ps = Vec::new();
        let mut seen_default = false;
        while !self.check(&TokenKind::RParen) {
            let s = self.current_span();
            let name = self.expect_ident("파라미터")?.raw.clone();
            if let Some(header) = header.as_deref_mut() {
                self.expect(&TokenKind::Equals, "공개핀=본문변수")?;
                let local_name = self.expect_ident("증거 본문 변수")?.raw.clone();
                header.evidence_bindings.push(ProofEvidenceBinding {
                    public_pin: name.clone(), local_name,
                    span: s.merge(&self.previous_span()),
                });
            }
            let type_ref = if self.check(&TokenKind::Colon) {
                self.advance();
                self.parse_type_ref()?
            } else if self.check(&TokenKind::Arrow) || self.check(&TokenKind::Equals) {
                TypeRef::Infer
            } else {
                return Err(
                    self.error("선언 항목은 `이름:타입 <- 값.` 또는 `이름 <- 값.` 형태여야 합니다")
                );
            };
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

    pub(super) fn parse_type_ref(&mut self) -> Result<TypeRef, ParseError> {
        let operand = self.parse_type_operand()?;
        if self.current().raw == "이고" {
            let ParsedTypeOperand::Single(first) = operand else {
                return Err(self.error("논리곱의 명제 항"));
            };
            let mut parts = vec![first];
            while self.current().raw == "이고" {
                self.advance();
                let ParsedTypeOperand::Single(part) = self.parse_type_operand()? else {
                    return Err(self.error("논리곱의 명제 항"));
                };
                parts.push(part);
            }
            return Ok(TypeRef::PropositionConjunction(parts));
        }
        if self.check(&TokenKind::TypeArrow) {
            self.advance();
            let params = match operand {
                ParsedTypeOperand::Single(param) => vec![param],
                ParsedTypeOperand::FunctionParams(params) => params,
            };
            let result = match self.parse_type_operand()? {
                ParsedTypeOperand::Single(result) => result,
                ParsedTypeOperand::FunctionParams(_) => {
                    return Err(ParseError {
                        span: self.current_span(),
                        message: "E_FUNCTION_TYPE_ARROW_NON_ASSOCIATIVE: 고차 함수 타입은 안쪽 함수 타입을 괄호로 묶어야 합니다".to_string(),
                    });
                }
            };
            if self.check(&TokenKind::TypeArrow) {
                return Err(ParseError {
                    span: self.current_span(),
                    message: "E_FUNCTION_TYPE_ARROW_NON_ASSOCIATIVE: `-->`는 비결합입니다. 고차 함수 타입은 괄호로 묶으세요".to_string(),
                });
            }
            if self.check(&TokenKind::RightArrow) {
                return Err(ParseError {
                    span: self.current_span(),
                    message: "E_FUNCTION_TYPE_RIGHT_ARROW_FORBIDDEN: 함수 타입에는 `->`가 아니라 `-->`를 사용하세요".to_string(),
                });
            }
            return Ok(TypeRef::Function {
                params,
                result: Box::new(result),
            });
        }
        if self.check(&TokenKind::RightArrow) {
            return Err(ParseError {
                span: self.current_span(),
                message: "E_FUNCTION_TYPE_RIGHT_ARROW_FORBIDDEN: 함수 타입에는 `->`가 아니라 `-->`를 사용하세요".to_string(),
            });
        }
        match operand {
            ParsedTypeOperand::Single(type_ref) => Ok(type_ref),
            ParsedTypeOperand::FunctionParams(_) => Err(ParseError {
                span: self.current_span(),
                message: "E_FUNCTION_TYPE_ARROW_EXPECTED: 타입 목록 뒤에는 `-->`와 결과 타입이 필요합니다".to_string(),
            }),
        }
    }

    pub(super) fn parse_type_operand(&mut self) -> Result<ParsedTypeOperand, ParseError> {
        if let Some(end) = self.find_matching_rparen_after_current_lparen() {
            if self.source_epoch == SourceEpoch::V25
                && self.tokens.get(end + 1).is_some_and(|token| token.raw == "보임")
                && self.tokens[self.pos+1..end].iter().any(|token| matches!(token.kind,TokenKind::RelationEq)) {
                self.advance();
                let expression=self.parse_expr()?;
                self.expect(&TokenKind::RParen,")")?;
                self.expect_ident("보임")?;
                let ExprKind::Infix {left,op,right}=&expression.kind else {
                    return Err(self.error("E_PROOF_ALGEBRA_GOAL: 정적 등식 목표가 필요합니다"));
                };
                if op!="=:=" { return Err(self.error("E_PROOF_ALGEBRA_GOAL: 정적 등식 목표가 필요합니다")); }
                let goal=TypeRef::AlgebraEquality {
                    left:crate::algebra_syntax::term(left)?,right:crate::algebra_syntax::term(right)?,
                };
                return Ok(ParsedTypeOperand::Single(TypeRef::Applied {name:"보임".into(),args:vec![goal]}));
            }
            if self.tokens.get(end + 1).is_some_and(|token| token.raw == "텐서씨") {
                return self.parse_tensor_type().map(ParsedTypeOperand::Single);
            }
        }
        if self.check(&TokenKind::LParen) {
            let proof_goal = self.find_matching_rparen_after_current_lparen().is_some_and(|end|
                self.tokens.get(end + 1).is_some_and(|token| token.raw == "보임"));
            self.advance();
            let mut args = Vec::new();
            if self.check(&TokenKind::RParen) {
                self.advance();
                if self.check(&TokenKind::TypeArrow) {
                    return Ok(ParsedTypeOperand::FunctionParams(args));
                }
                return Err(ParseError {
                    span: self.current_span(),
                    message: "E_FUNCTION_TYPE_ARROW_EXPECTED: 빈 타입 목록 뒤에는 `-->`와 결과 타입이 필요합니다".to_string(),
                });
            }
            loop {
                if proof_goal && self.check_ident()
                    && self.peek_kind_n_is(1, |kind| matches!(kind, TokenKind::Dot)) {
                    let structure=self.expect_ident("구조 목표")?.raw.clone();
                    self.advance();
                    let member=self.expect_ident("구조 법칙 목표")?.raw.clone();
                    if structure=="_" || (member!="법칙" && !COORDINATE_GOAL_MEMBERS.contains(&member.as_str())) {
                        return Err(self.error("E_PROOF_GOAL: 구조의 명시된 법칙 목표가 필요합니다"));
                    }
                    args.push(TypeRef::StructureLaw{structure,member});
                } else {
                    args.push(self.parse_type_ref()?);
                }
                if self.check(&TokenKind::Comma) {
                    self.advance();
                    continue;
                }
                break;
            }
            self.expect(&TokenKind::RParen, ")")?;
            if self.check_ident() && self.current().raw != "이고" {
                let name = self.expect_ident("타입")?.raw.clone();
                if name == "_" {
                    return Err(self.error("타입 이름"));
                }
                return Ok(ParsedTypeOperand::Single(TypeRef::Applied { name, args }));
            }
            if self.check(&TokenKind::TypeArrow) {
                return Ok(ParsedTypeOperand::FunctionParams(args));
            }
            if args.len() == 1 && matches!(args.first(), Some(TypeRef::Function { .. } | TypeRef::Tuple(_) | TypeRef::PropositionConjunction(_))) {
                return Ok(ParsedTypeOperand::Single(args.pop().expect("grouped type")));
            }
            if args.len() >= 2 {
                return Ok(ParsedTypeOperand::Single(TypeRef::Tuple(args)));
            }
            return Err(ParseError {
                span: self.current_span(),
                message: "E_FUNCTION_TYPE_GROUP_INVALID: 괄호 타입 목록에는 적용 타입 이름 또는 `-->`가 필요합니다".to_string(),
            });
        }
        let name = self.expect_ident("타입")?.raw.clone();
        if name == "_" {
            Ok(ParsedTypeOperand::Single(TypeRef::Infer))
        } else {
            Ok(ParsedTypeOperand::Single(TypeRef::Named(name)))
        }
    }
    pub(super) fn parse_seed_kind(&mut self) -> Result<SeedKind, ParseError> {
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
                if name == "묶음씨" {
                    return Ok(SeedKind::Product);
                }
                if let Some(replacement) = legacy_seed_kind_replacement(name) {
                    return Err(ParseError {
                        span: self.to_ast_span(t.span),
                        message: format!(
                            "`{name}`는 더 이상 허용하지 않습니다. `{replacement}`를 사용하세요"
                        ),
                    });
                }
                self.validate_ident_token_allow_reserved(&t, &["임자"])?;
                Ok(SeedKind::Named(name.clone()))
            }
            _ => Err(self.error("씨앗 종류")),
        }
    }
}
