// lang/src/parser.rs
use crate::ast::*;
use crate::delivery::MessageTransfer;
use crate::lexer::{Lexer, Token, TokenKind};
use crate::normalizer::{NormalizationLevel, Normalizer};
use crate::stdlib::minimal_stdlib_sigs;
use crate::term_map;
use ddonirang_core::{is_known_unit, unit_spec_from_symbol, Fixed64, UnitDim};
use std::collections::{HashMap, HashSet};
use std::fmt;

pub struct Parser {
    tokens: Vec<Token>,
    source_epoch: SourceEpoch,
    source: String,
    pos: usize,
    nid: u64,
    root_hide: bool,
    beat_depth: usize,
    body_depth: usize,
    chaebi_forbidden_depth: usize,
    declared_scopes: Vec<DeclaredNames>,
    declared_type_scopes: Vec<HashMap<String, TypeRef>>,
    seed_kind_stack: Vec<SeedKind>,
    natural_foreach_ids: HashSet<NodeId>,
    current_item_stack: Vec<(String, TypeRef, Span)>,
    registered_forms: HashMap<String, String>,
    registered_form_candidates: HashMap<String, Vec<NodeId>>,
    registered_call_candidates: HashMap<NodeId, Vec<NodeId>>,
    registered_call_spans: HashMap<NodeId, Span>,
    registered_definitions: HashMap<NodeId, SeedDef>,
    explicitly_registered_owners: HashSet<String>,
    product_definitions: HashMap<String, (NodeId, Vec<ParamPin>)>,
    schema_categories: HashMap<String, SchemaCategory>,
    schema_header_names: HashSet<String>,
    schema_construction_nodes: HashSet<NodeId>,
    sum_definitions: HashMap<String, Vec<SumBranchDecl>>,
    bytes_calls: HashMap<NodeId, crate::bytes::BytesConversion>,
    atom_value_occurrences: HashMap<NodeId, AtomValueOccurrence>,
    retirement_commands: HashMap<NodeId,RetirementCommandBinding>,
    subject_reference_calls: HashMap<NodeId,SubjectReferenceCallBinding>,
    manipulation_phase_calls: HashMap<NodeId,ManipulationPhaseCallBinding>,
    mode: ParseMode,
}
#[derive(Clone,Default)]
struct DeclaredNames {
    // Keep root-hide authorization distinct from ordinary value bindings.
    explicit: HashSet<String>,
    ordinary: HashSet<String>,
}
struct ArgSuffix {
    josa: Option<String>,
    binding_reason: BindingReason,
    fixed_pin: Option<String>,
}
enum ParsedTypeOperand {
    Single(TypeRef),
    FunctionParams(Vec<TypeRef>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    SuccessorDefault,
    HistoricalMigrationEvidence,
}
impl ParseMode {
    #[allow(non_upper_case_globals)]
    pub const Strict: Self = Self::SuccessorDefault;
}
#[derive(Debug, Clone, Copy)]
enum DimState {
    Known(UnitDim),
    Unknown,
}
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self::new_with_mode(tokens, ParseMode::SuccessorDefault)
    }
    pub fn new_with_mode(tokens: Vec<Token>, mode: ParseMode) -> Self {
        Self::new_with_epoch(tokens, SourceEpoch::PreV25, mode)
    }
    pub fn new_v25_with_mode(tokens: Vec<Token>, mode: ParseMode) -> Self {
        Self::new_with_epoch(tokens, SourceEpoch::V25, mode)
    }
    fn new_with_epoch(tokens: Vec<Token>, source_epoch: SourceEpoch, mode: ParseMode) -> Self {
        Self {
            tokens,
            source_epoch,
            source: String::new(),
            pos: 0,
            nid: 1,
            root_hide: false,
            beat_depth: 0,
            body_depth: 0,
            chaebi_forbidden_depth: 0,
            declared_scopes: vec![DeclaredNames::default()],
            declared_type_scopes: vec![HashMap::new()],
            seed_kind_stack: Vec::new(),
            natural_foreach_ids: HashSet::new(),
            current_item_stack: Vec::new(),
            registered_forms: HashMap::new(),
            registered_form_candidates: HashMap::new(),
            registered_call_candidates: HashMap::new(),
            registered_call_spans: HashMap::new(),
            registered_definitions: HashMap::new(),
            explicitly_registered_owners: HashSet::new(),
            product_definitions: HashMap::new(),
            schema_categories: HashMap::new(),
            schema_header_names: HashSet::new(),
            schema_construction_nodes: HashSet::new(),
            sum_definitions: HashMap::new(),
            bytes_calls: HashMap::new(),
            atom_value_occurrences: HashMap::new(),
            retirement_commands: HashMap::new(),
            subject_reference_calls: HashMap::new(),
            manipulation_phase_calls: HashMap::new(),
            mode,
        }
    }
    fn is_sum_match_stmt_start(&self) -> bool {
        self.sum_match_boundary().is_some()
    }

    // RD-613 owns a general Expression subject, not an identifier-only form.
    // Locate its grammatical boundary without rewriting authored source.
    fn sum_match_boundary(&self) -> Option<(usize,Option<usize>)> {
        let word=|kind:&TokenKind,text:&str|matches!(kind,TokenKind::Ident(name)|TokenKind::Josa(name) if name==text);
        let mut depth=0usize;
        for index in self.pos..self.tokens.len() {
            let token=&self.tokens[index];
            if depth==0 {
                let followed_by_body=self.tokens.get(index+1).is_some_and(|next|word(&next.kind,"따라"))
                    && self.tokens.get(index+2).is_some_and(|next|matches!(next.kind,TokenKind::LBrace));
                if followed_by_body && index>self.pos && word(&token.kind,"에") {
                    return Some((index,None));
                }
                if followed_by_body && matches!(&token.kind,TokenKind::Ident(name) if name.len()>'에'.len_utf8() && name.ends_with('에')) {
                    return Some((index+1,Some(index)));
                }
                match &token.kind {
                    TokenKind::Eof|TokenKind::Semicolon|TokenKind::Arrow|TokenKind::Define => return None,
                    TokenKind::Dot => {
                        let postfix=self.tokens.get(index.wrapping_sub(1)).is_some_and(|previous|previous.span.end==token.span.start)
                            && self.tokens.get(index+1).is_some_and(|next|next.span.start==token.span.end
                                && matches!(next.kind,TokenKind::Ident(_)|TokenKind::BranchRole(_)));
                        if !postfix {return None;}
                    }
                    _=>{}
                }
            }
            match &token.kind {
                TokenKind::LParen|TokenKind::LBracket|TokenKind::LBrace=>depth+=1,
                TokenKind::RParen|TokenKind::RBracket|TokenKind::RBrace=>{
                    if depth==0 {return None;}
                    depth-=1;
                }
                _=>{}
            }
        }
        None
    }

    fn is_sum_definition_stmt_start(&self) -> bool {
        self.check_ident()
            && self.peek_kind_n_is(1, |kind| matches!(kind, TokenKind::Colon))
            && self.peek_kind_n_is(2, |kind| matches!(kind, TokenKind::KwGallaessi))
            && self.peek_kind_n_is(3, |kind| matches!(kind, TokenKind::Define))
    }

    fn parse_sum_definition_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        let name = self.expect_ident("고름씨 이름")?.raw.clone();
        if self.sum_definitions.contains_key(&name) {
            return Err(ParseError {
                span: start,
                message: format!("E_SUM_DEFINITION_DUPLICATE: `{name}` 고름씨가 이미 선언되었습니다"),
            });
        }
        self.expect(&TokenKind::Colon, ":")?;
        if self.current().raw!="고름씨" {
            return Err(self.error("E_SUM_DEFINITION_KIND: 고름씨 선언에는 `고름씨`를 사용해야 합니다"));
        }
        self.expect(&TokenKind::KwGallaessi, "고름씨")?;
        self.expect(&TokenKind::Define, ":=")?;
        self.expect(&TokenKind::LBrace, "{")?;
        let mut branches = Vec::new();
        let mut branch_names = HashSet::new();
        while !self.check(&TokenKind::RBrace) {
            let branch_start = self.current_span();
            let branch_token = match self.current().kind.clone() {
                TokenKind::BranchRole(branch) => {
                    let token = self.advance();
                    (branch, self.to_ast_span(token.span))
                }
                _ => return Err(self.error("E_SUM_SCHEMA_BRANCH_NAME_REQUIRED: 고름씨 선언 안에는 소유자 접두어 없이 `$이름`을 사용해야 합니다")),
            };
            if !branch_names.insert(branch_token.0.clone()) {
                return Err(ParseError {
                    span: branch_token.1,
                    message: format!("E_SUM_BRANCH_DUPLICATE: `{name}.${}` 갈래가 중복되었습니다", branch_token.0),
                });
            }
            let mut payload = Vec::new();
            if self.check(&TokenKind::LParen) {
                self.advance();
                let mut pins=HashSet::new();
                loop {
                    let binder_token = self.expect_ident("고름씨 값 갈래 받는 이름")?;
                    let binder = binder_token.raw.clone();
                    if !pins.insert(binder.clone()) {
                        return Err(ParseError {span:self.to_ast_span(binder_token.span),
                            message:format!("E_SUM_PAYLOAD_PIN_DUPLICATE: 갈래 받는 이름이 중복되었습니다: {binder}")});
                    }
                    self.expect(&TokenKind::Colon, ":")?;
                    let type_ref = self.parse_type_ref()?;
                    payload.push(SumBranchPayload {
                        binder, binder_span:self.to_ast_span(binder_token.span), type_ref,
                    });
                    if !self.check(&TokenKind::Comma) {break;}
                    self.advance();
                }
                self.expect(&TokenKind::RParen, ")")?;
            }
            self.expect(&TokenKind::Dot, ".")?;
            branches.push(SumBranchDecl {
                id: self.next_id(),
                span: branch_start.merge(&self.previous_span()),
                name: branch_token.0,
                payload,
            });
        }
        if branches.is_empty() {
            return Err(ParseError {
                span: start,
                message: "E_SUM_BRANCH_REQUIRED: 고름씨에는 하나 이상의 갈래가 필요합니다".to_string(),
            });
        }
        self.expect(&TokenKind::RBrace, "}")?;
        let mood = self.consume_stmt_terminator()?;
        self.sum_definitions.insert(name.clone(), branches.clone());
        Ok(Stmt::SumDefinition {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            mood,
            name,
            branches,
        })
    }

    fn sum_branch_schemas(&self,owner:&str,applied:Option<&TypeRef>,span:Span)
        ->Result<Vec<crate::sum_owner::ResolvedSumBranch>,ParseError> {
        if let Some(kind)=applied {
            return crate::sum_owner::resolve_applied_sum_owner(kind,span)
                .map(|resolved|resolved.branches).map_err(|message|ParseError{span,message});
        }
        self.sum_definitions.get(owner).map(|branches|branches.iter().map(|branch|
            crate::sum_owner::ResolvedSumBranch{name:branch.name.clone(),payload:branch.payload.clone()}).collect())
            .ok_or_else(||ParseError{span,message:format!("E_SUM_OWNER_UNKNOWN: `{owner}` 고름씨가 선언되지 않았습니다")})
    }

    fn parse_sum_branch_value(&mut self,owner:String,applied_owner:Option<TypeRef>,start:Span)->Result<Expr,ParseError> {
        let branch_token=self.advance();
        let TokenKind::BranchRole(branch)=branch_token.kind.clone() else {return Err(self.error("owner.$갈래"));};
        let definitions=self.sum_branch_schemas(&owner,applied_owner.as_ref(),start)?;
        let declaration=definitions.iter().find(|schema|schema.name==branch)
            .ok_or_else(||ParseError{span:self.to_ast_span(branch_token.span),message:format!("E_SUM_BRANCH_UNKNOWN: `{owner}.${branch}` 갈래가 선언되지 않았습니다")})?;
        let mut payload=Vec::new();
        if self.check(&TokenKind::LParen) {
            if declaration.payload.is_empty() {return Err(ParseError{span:self.current_span(),
                message:format!("E_SUM_BRANCH_NULLARY_PAYLOAD: `{owner}.${branch}`는 값을 받지 않는 갈래입니다")});}
            self.advance();
            let mut supplied=HashSet::new();
            loop {
                let pin_span=self.current_span();
                let pin=self.expect_ident("고름씨 값 이름")?.raw.clone();
                if !declaration.payload.iter().any(|schema|schema.binder==pin) {return Err(ParseError{span:pin_span,
                    message:format!("E_SUM_BRANCH_PAYLOAD_PIN: `{owner}.${branch}`에 선언되지 않은 값 이름입니다: {pin}")});}
                if !supplied.insert(pin.clone()) {return Err(ParseError{span:pin_span,
                    message:format!("E_SUM_BRANCH_PAYLOAD_DUPLICATE: 값 이름이 중복되었습니다: {pin}")});}
                self.expect(&TokenKind::Equals,"=")?;
                let value=self.parse_expr()?;
                payload.push(SumPayloadArgument{pin,span:pin_span,value});
                if !self.check(&TokenKind::Comma) {break;}
                self.advance();
            }
            self.expect(&TokenKind::RParen,")")?;
        }
        if declaration.payload.len()!=payload.len() {return Err(ParseError{span:self.to_ast_span(branch_token.span),
            message:format!("E_SUM_BRANCH_PAYLOAD_REQUIRED: `{owner}.${branch}`에는 명시적 값이 필요합니다")});}
        Ok(Expr::new(self.next_id(),start.merge(&self.previous_span()),
            ExprKind::SumBranchValue{owner,applied_owner,branch,payload}))
    }

    fn try_parse_sum_match_stmt(&mut self) -> Result<Option<Stmt>, ParseError> {
        let Some((boundary,attached))=self.sum_match_boundary() else {return Ok(None);};
        let start = self.current_span();
        let marker=std::mem::replace(&mut self.tokens[boundary].kind,TokenKind::Eof);
        let original=attached.map(|index|(index,self.tokens[index].clone()));
        if let Some(index)=attached {
            let stem=self.tokens[index].raw.strip_suffix('에').expect("attached case suffix").to_string();
            self.tokens[index].raw=stem.clone();
            self.tokens[index].kind=TokenKind::Ident(stem);
            self.tokens[index].span.end-='에'.len_utf8();
        }
        let parsed=self.parse_expr();
        self.tokens[boundary].kind=marker;
        if let Some((index,token))=original {self.tokens[index]=token;}
        let subject=parsed?;
        if self.pos!=boundary {return Err(self.error("E_SUM_MATCH_SUBJECT_EXPRESSION: `에 따라` 앞에는 하나의 식이 필요합니다"));}
        if attached.is_none() {
            self.advance(); // 에
        }
        self.advance(); // 따라
        self.expect(&TokenKind::LBrace, "{")?;

        let mut arms = Vec::new();
        let mut arm_names = HashSet::new();
        let mut match_owner: Option<String> = None;
        let mut match_branches=Vec::new();
        let mut default_body = None;
        while !self.check(&TokenKind::RBrace) {
            if self.check(&TokenKind::KwAniramyeon) {
                if self.current().raw!="아니면" {
                    return Err(self.error("E_SUM_MATCH_DEFAULT_RAW: 고름씨 기본 갈래에는 `아니면`만 사용할 수 있습니다"));
                }
                if default_body.is_some() {
                    return Err(ParseError {
                        span: self.current_span(),
                        message: "E_SUM_MATCH_DEFAULT_DUPLICATE: `아니면`은 한 번만 허용됩니다".to_string(),
                    });
                }
                self.advance();
                let body = self.parse_body()?;
                self.consume_stmt_terminator()?;
                default_body = Some(body);
                if !self.check(&TokenKind::RBrace) {
                    return Err(ParseError {
                        span: self.current_span(),
                        message: "E_SUM_MATCH_DEFAULT_LAST: `아니면`은 마지막 갈래여야 합니다".to_string(),
                    });
                }
                break;
            }

            let arm_start = self.current_span();
            let owner_span=self.current_span();
            let (owner,applied_owner)=if self.check(&TokenKind::LParen) {
                let kind=self.parse_type_ref()?;
                let resolved=crate::sum_owner::resolve_applied_sum_owner(&kind,owner_span)
                    .map_err(|message|ParseError{span:owner_span,message})?;
                (resolved.key,Some(resolved.type_ref))
            } else {(self.expect_ident("고름씨 소유자")?.raw.clone(),None)};
            self.expect_adjacent_dot(self.previous_span().end)?;
            let branch_token = match self.current().kind.clone() {
                TokenKind::BranchRole(branch) => {
                    let token = self.advance();
                    (branch, token)
                }
                _ => return Err(self.error("owner.$갈래")),
            };
            let branch = branch_token.0;
            let branch_span = self.to_ast_span(branch_token.1.span);
            let definitions = self.sum_branch_schemas(&owner,applied_owner.as_ref(),owner_span)?;
            let declaration = definitions.iter().find(|candidate| candidate.name == branch).cloned().ok_or_else(|| ParseError {
                span: branch_span,
                message: format!("E_SUM_BRANCH_UNKNOWN: `{owner}.${branch}` 갈래가 선언되지 않았습니다"),
            })?;
            if let Some(expected_owner) = &match_owner {
                if expected_owner != &owner {
                    return Err(ParseError {
                        span: owner_span,
                        message: "E_SUM_MATCH_OWNER_MISMATCH: 한 `에 따라` 문장은 하나의 정확한 고름씨 소유자만 다룰 수 있습니다".to_string(),
                    });
                }
            } else {
                match_owner = Some(owner.clone());
                match_branches=definitions.iter().map(|branch|branch.name.clone()).collect();
            }
            if !arm_names.insert(branch.clone()) {
                return Err(ParseError {
                    span: branch_span,
                    message: format!("E_SUM_MATCH_BRANCH_DUPLICATE: `{owner}.${branch}` 갈래가 중복되었습니다"),
                });
            }
            let mut binder = Vec::new();
            if self.check(&TokenKind::LParen) {
                if declaration.payload.is_empty() {
                    return Err(ParseError {
                        span: self.current_span(),
                        message: format!("E_SUM_MATCH_NULLARY_BINDER: `{owner}.${branch}`는 값을 받지 않는 갈래입니다"),
                    });
                }
                self.advance();
                if self.check(&TokenKind::RParen) {
                    return Err(self.error("E_SUM_MATCH_BINDER_EMPTY: 받는 이름 목록은 비워 둘 수 없습니다"));
                }
                let mut selected=HashSet::new();
                loop {
                let token = self.expect_ident("값 갈래 받는 이름")?;
                let pin = token.raw.clone();
                if !declaration.payload.iter().any(|schema|schema.binder==pin) {
                    return Err(ParseError {
                        span: self.to_ast_span(token.span),
                        message: format!("E_SUM_MATCH_BINDER_UNKNOWN: `{owner}.${branch}`의 선언된 받는 이름만 선택할 수 있습니다: {pin}"),
                    });
                }
                if !selected.insert(pin.clone()) {
                    return Err(ParseError {span:self.to_ast_span(token.span),
                        message:format!("E_SUM_MATCH_BINDER_DUPLICATE: 받는 이름이 중복되었습니다: {pin}")});
                }
                if self.pattern_name_is_bound(&pin) {
                    return Err(ParseError {
                        span: self.to_ast_span(token.span),
                        message: format!("E_SUM_MATCH_BINDER_CAPTURE: 이미 결속된 이름을 갈래 받는 이름으로 잡을 수 없습니다: {pin}"),
                    });
                }
                binder.push(SumPatternBinder {id:self.next_id(),span:self.to_ast_span(token.span),pin,registered_pin_identity:None});
                if !self.check(&TokenKind::Comma) {break;}
                self.advance();
                }
                self.expect(&TokenKind::RParen, ")")?;
            }
            if !self.check(&TokenKind::KwMajeumyeon) || self.current().raw!="이면" {
                return Err(self.error("E_SUM_MATCH_ARM_CONDITION: 고름씨 갈래 조건에는 `이면`만 사용할 수 있습니다"));
            }
            self.advance();
            let body = self.parse_sum_match_arm_body(&binder)?;
            self.consume_stmt_terminator()?;
            arms.push(SumMatchArm {
                id: self.next_id(),
                span: arm_start.merge(&self.previous_span()),
                owner,
                applied_owner,
                branch,
                binder,
                body,
            });
        }
        self.expect(&TokenKind::RBrace, "}")?;
        if arms.is_empty() {
            return Err(ParseError {
                span: start,
                message: "E_SUM_MATCH_ARM_REQUIRED: `에 따라`에는 하나 이상의 갈래가 필요합니다".to_string(),
            });
        }
        if default_body.is_none() {
            if arms.len() != match_branches.len() || match_branches.iter().any(|branch| !arm_names.contains(branch)) {
                return Err(ParseError {
                    span: start,
                    message: "E_SUM_MATCH_EXHAUSTIVE_REQUIRED: `아니면`이 없으면 선언된 모든 갈래를 정확히 한 번씩 다뤄야 합니다".to_string(),
                });
            }
        }
        let mood = self.consume_stmt_terminator()?;
        Ok(Some(Stmt::SumMatch {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            mood,
            subject,
            arms,
            default_body,
        }))
    }

    fn parse_sum_match_arm_body(&mut self, binders: &[SumPatternBinder]) -> Result<Body, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::LBrace, "{")?;
        self.enter_scope();
        for binder in binders {
            self.declare_name(&binder.pin);
        }
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) {
            stmts.push(self.parse_stmt(false)?);
        }
        self.expect(&TokenKind::RBrace, "}")?;
        self.exit_scope();
        Ok(Body {
            id: self.next_id(),
            span: start.merge(&self.previous_span()),
            stmts,
        })
    }

    fn expect_adjacent_dot(&mut self, expected_end: usize) -> Result<(), ParseError> {
        if !self.check(&TokenKind::Dot) || self.current().span.start != expected_end {
            return Err(self.error("owner.$갈래의 붙은 `.`"));
        }
        self.advance();
        if self.current().span.start != expected_end + 1 {
            return Err(self.error("owner.$갈래의 붙은 `$`"));
        }
        Ok(())
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
        self.parse_program_phase(source, file_path, true)
    }

    /// Declaration discovery only. Imported call targets are resolved after
    /// linking; this result must never be used as an executable program.
    pub fn parse_unlinked_module(
        &mut self,
        source: String,
        file_path: String,
    ) -> Result<CanonProgram, ParseError> {
        self.parse_program_phase(source, file_path, false)
    }

    fn parse_program_phase(
        &mut self,
        source: String,
        file_path: String,
        resolve_calls: bool,
    ) -> Result<CanonProgram, ParseError> {
        self.source = source.clone();
        self.root_hide = false;
        self.beat_depth = 0;
        self.body_depth = 0;
        self.chaebi_forbidden_depth = 0;
        self.declared_scopes.clear();
        self.declared_scopes.push(DeclaredNames::default());
        self.declared_type_scopes.clear();
        self.product_definitions.clear();
        self.schema_categories.clear();
        self.schema_header_names.clear();
        self.schema_construction_nodes.clear();
        // Grammar lookahead only. Actual category/owner/slot admission happens
        // after the complete program has been parsed, including forward uses.
        if self.source_epoch==SourceEpoch::V25 {
            let mut depth=0usize;
            for index in 0..self.tokens.len() {
                if depth==0&&self.is_vc04_schema_header_at(index) {self.schema_header_names.insert(self.tokens[index].raw.clone());}
                match self.tokens[index].kind {TokenKind::LBrace=>depth+=1,TokenKind::RBrace=>depth=depth.saturating_sub(1),_=>{}}
            }
        }
        self.sum_definitions.clear();
        self.bytes_calls.clear();
        self.atom_value_occurrences.clear();
        self.retirement_commands.clear();
        self.subject_reference_calls.clear();
        self.manipulation_phase_calls.clear();
        self.declared_type_scopes.push(HashMap::new());
        self.seed_kind_stack.clear();
        self.natural_foreach_ids.clear();
        self.current_item_stack.clear();
        self.registered_forms.clear();
        self.registered_form_candidates.clear();
        self.registered_call_candidates.clear();
        self.registered_call_spans.clear();
        self.registered_definitions.clear();
        self.explicitly_registered_owners.clear();
        let mut items = Vec::new();
        let mut root_stmts = Vec::new();
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
                    message: "`보개장면`은 더 이상 허용하지 않습니다. `보개마당`을 사용하세요"
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
            if self.source_epoch == SourceEpoch::V25 && self.peek_legacy_l11_brace_block() {
                return Err(ParseError {
                    span: self.current_span(),
                    message: "E_V25_DEFINITION_MIGRATION_REQUIRED: V25에서는 이전 brace형 L11 선언을 사용할 수 없습니다. `(<항목>=value) 쓰임.` 또는 `(<항목>=value) 드러냄.`처럼 현재 typed directive를 사용하세요".to_string(),
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
                let stmt = self.parse_meta_block_stmt()?;
                // Drawing is executable at the root just as it is in a body.
                // Keep configuration-only metadata out of the statement stream.
                if matches!(&stmt, Stmt::MetaBlock { kind: MetaBlockKind::Bogae, .. }) {
                    root_stmts.push(stmt);
                }
                continue;
            }
            if let Some(kind) = self.peek_decl_block_kind() {
                let stmt = self.parse_decl_block(kind)?;
                top_level_decl.push(stmt);
                continue;
            }
            if self.is_vc04_schema_start() {
                items.push(TopLevelItem::SeedDef(self.parse_vc04_schema()?));
            } else if self.is_sum_definition_stmt_start()
                || self.is_sum_match_stmt_start()
                || self.is_root_statement_start() {
                root_stmts.push(self.parse_stmt(false)?);
            } else {
                items.push(self.parse_top_level_item()?);
            }
        }
        if !top_level_decl.is_empty() {
            self.inject_top_level_decl_blocks(&mut items, top_level_decl)?;
        }
        let mut program = CanonProgram {
            id: self.next_id(),
            source_epoch: self.source_epoch,
            items,
            root_stmts,
            origin: OriginMap {
                file_path,
                source,
                node_spans: std::collections::HashMap::new(),
                registered_sum_pin_order:None,
                registered_call_candidates: HashMap::new(),
                bytes_calls: HashMap::new(),
                atom_value_occurrences: self.atom_value_occurrences.clone(),
                retirement_commands: HashMap::new(),
                subject_reference_calls: HashMap::new(),
                manipulation_phase_calls: HashMap::new(),
            },
        };
        self.validate_seed_name_conflicts(&program)?;
        self.bind_registered_forms(&program)?;
        crate::ordinary_callable::validate_ordinary_callable_admission(&program).map_err(
            |failure| ParseError {
                span: Span::new(failure.span_start, failure.span_end),
                message: failure.diagnostic_message(),
            },
        )?;
        if resolve_calls {
            self.apply_default_args(&mut program)?;
        }
        program.origin.registered_call_candidates = self.registered_call_candidates.clone();
        program.origin.bytes_calls = self.bytes_calls.clone();
        program.origin.retirement_commands = self.retirement_commands.clone();
        program.origin.subject_reference_calls = self.subject_reference_calls.clone();
        program.origin.manipulation_phase_calls = self.manipulation_phase_calls.clone();
        if resolve_calls {crate::vc04_semantics::validate(&mut program)?;}
        program
            .origin
            .node_spans
            .extend(self.registered_call_spans.clone());
        self.validate_units(&program)?;
        validate_record_field_accesses(&program)?;
        Ok(program)
    }
    fn enter_scope(&mut self) {
        self.declared_scopes.push(DeclaredNames::default());
        self.declared_type_scopes.push(HashMap::new());
    }
    fn exit_scope(&mut self) {
        self.declared_scopes.pop();
        self.declared_type_scopes.pop();
    }
    fn declare_name(&mut self, name: &str) {
        if let Some(scope) = self.declared_scopes.last_mut() {
            scope.explicit.insert(name.to_string());
        }
    }
    fn declare_typed_name(&mut self, name: &str, type_ref: &TypeRef) {
        self.declare_name(name);
        if let Some(scope) = self.declared_type_scopes.last_mut() {
            scope.insert(name.to_string(), type_ref.clone());
        }
    }
    fn declared_type(&self, name: &str) -> Option<TypeRef> {
        self.declared_type_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }
    fn is_declared(&self, name: &str) -> bool {
        self.declared_scopes
            .iter()
            .rev()
            .any(|scope| scope.explicit.contains(name))
    }
    fn pattern_name_is_bound(&self, name: &str) -> bool {
        self.declared_scopes.iter().rev().any(|scope|
            scope.explicit.contains(name)||scope.ordinary.contains(name))
    }
    fn in_imja_seed_body(&self) -> bool {
        self.seed_kind_stack
            .last()
            .is_some_and(|kind| matches!(kind, SeedKind::Named(name) if name == "임자"))
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
            message: format!("채비에 선언되지 않은 전역 쓰기: {}", name),
        })
    }
    fn parse_top_level_item(&mut self) -> Result<TopLevelItem, ParseError> {
        Ok(TopLevelItem::SeedDef(self.parse_seed_def()?))
    }

    /// Keep the established callable grammar authoritative for callable
    /// headers, while admitting the current root-program statement forms.
    /// In particular, `name <- value.` and the V25 short Definition form
    /// `name := literal.` must not be reinterpreted as seed definitions merely
    /// because they appear before any callable.
    fn is_root_statement_start(&self) -> bool {
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
    fn is_vc04_schema_start(&self) -> bool {
        self.is_vc04_schema_header_at(self.pos)
    }
    fn is_vc04_schema_header_at(&self,index:usize)->bool {
        let Some(row)=self.tokens.get(index..index+5)else{return false};
        self.source_epoch==SourceEpoch::V25&&matches!(row[0].kind,TokenKind::Ident(_))
            &&matches!(row[1].kind,TokenKind::Colon)&&matches!(row[2].raw.as_str(),"임자"|"이름씨")
            &&matches!(row[3].kind,TokenKind::Define)&&matches!(row[4].kind,TokenKind::LBrace)
            &&!(row[2].raw=="임자"&&self.is_existing_reaction_owner_body(index+5))
    }
    /// Preserve the already-supported RD-293/445 owner body, not a permissive
    /// fallback for malformed slot schemas. The actual legacy parser remains
    /// responsible for admitting its receiver/provider declaration.
    fn is_existing_reaction_owner_body(&self,start:usize)->bool {
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

    fn parse_vc04_schema(&mut self) -> Result<SeedDef, ParseError> {
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
        Ok(SeedDef { id: self.next_id(), span: start.merge(&self.previous_span()),
            canonical_name: name, alternate_forms: Vec::new(), seed_kind: SeedKind::Schema(category),
            params, result_type: None, body: None, modifiers: Vec::new() })
    }

    fn parse_seed_def(&mut self) -> Result<SeedDef, ParseError> {
        if self.check_ident() && self.peek_kind_n_is(1, |kind| matches!(kind, TokenKind::LParen)) {
            return self.parse_named_single_expression_callable();
        }
        let start = self.current_span();
        let params = if self.check(&TokenKind::LParen) {
            self.parse_params()?
        } else {
            Vec::new()
        };
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
        if kind != SeedKind::Product && alternate_forms.is_empty() {
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
        if result_type.is_some() && !self.check(&TokenKind::LBrace) {
            return Err(self.error("결과 타입이 있는 선언의 `{ ... }` 본문"));
        }
        self.enter_scope();
        if let Some(result_type) = &result_type {
            self.declare_typed_name(&name, result_type);
        }
        for param in &params {
            self.declare_typed_name(&param.pin_name, &param.type_ref);
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
    fn parse_named_single_expression_callable(&mut self) -> Result<SeedDef, ParseError> {
        let start = self.current_span();
        let name = self.expect_ident("호출가능 정의 이름")?.raw.clone();
        self.validate_seed_name_tail(&name, self.previous_span())?;
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
            params.push(ParamPin {
                id: self.next_id(),
                span: param_start.merge(&self.previous_span()),
                pin_name,
                type_ref,
                default_value: None,
                optional: false,
                josa_list: Vec::new(),
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

    fn parse_callable_alternate_forms(
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

    fn bind_registered_forms(&mut self, program: &CanonProgram) -> Result<(), ParseError> {
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
    fn parse_params(&mut self) -> Result<Vec<ParamPin>, ParseError> {
        self.expect(&TokenKind::LParen, "(")?;
        let mut ps = Vec::new();
        let mut seen_default = false;
        while !self.check(&TokenKind::RParen) {
            let s = self.current_span();
            let name = self.expect_ident("파라미터")?.raw.clone();
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

    fn parse_type_ref(&mut self) -> Result<TypeRef, ParseError> {
        let operand = self.parse_type_operand()?;
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

    fn parse_type_operand(&mut self) -> Result<ParsedTypeOperand, ParseError> {
        if let Some(end) = self.find_matching_rparen_after_current_lparen() {
            if self.tokens.get(end + 1).is_some_and(|token| token.raw == "텐서씨") {
                return self.parse_tensor_type().map(ParsedTypeOperand::Single);
            }
        }
        if self.check(&TokenKind::LParen) {
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
                args.push(self.parse_type_ref()?);
                if self.check(&TokenKind::Comma) {
                    self.advance();
                    continue;
                }
                break;
            }
            self.expect(&TokenKind::RParen, ")")?;
            if self.check_ident() {
                let name = self.expect_ident("타입")?.raw.clone();
                if name == "_" {
                    return Err(self.error("타입 이름"));
                }
                return Ok(ParsedTypeOperand::Single(TypeRef::Applied { name, args }));
            }
            if self.check(&TokenKind::TypeArrow) {
                return Ok(ParsedTypeOperand::FunctionParams(args));
            }
            if args.len() == 1 && matches!(args.first(), Some(TypeRef::Function { .. } | TypeRef::Tuple(_))) {
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
    fn parse_body(&mut self) -> Result<Body, ParseError> {
        self.body_depth+=1;
        let result=self.parse_body_inner();
        self.body_depth=self.body_depth.saturating_sub(1);
        result
    }
    fn parse_body_inner(&mut self) -> Result<Body, ParseError> {
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

    fn is_ordinary_while_start(&self) -> bool {
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

    fn parse_body_with_chaebi_restriction(&mut self) -> Result<Body, ParseError> {
        self.chaebi_forbidden_depth += 1;
        let body = self.parse_body();
        self.chaebi_forbidden_depth = self.chaebi_forbidden_depth.saturating_sub(1);
        body
    }

    fn parse_stmt(&mut self, allow_implicit_terminator: bool) -> Result<Stmt, ParseError> {
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

    fn parse_stmt_inner(&mut self, allow_implicit_terminator: bool) -> Result<Stmt, ParseError> {
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
                message: "E_BOIM_LEGACY_REMOVED: `보임 {}` 블록은 제거되었습니다. `(<이름>: <식>, ...) 보임.`을 사용하세요".to_string(),
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
            TokenKind::KwIlttae => matches!(expected, "일때" | "할때"),
            TokenKind::KwDongan => expected == "동안",
            _ => false,
        }
    }

    fn try_parse_receive_stmt(&mut self) -> Result<Option<Stmt>, ParseError> {
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

    fn try_parse_message_sender_prefix(&mut self) -> Result<Option<Expr>, ParseError> {
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

    fn message_sender_possessive_index(&self) -> Option<usize> {
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

    fn receive_sender_filter_index(&self) -> Option<usize> {
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

    fn has_typed_receive_binding_head(&self) -> bool {
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

    fn has_receive_binding_head(&self) -> bool {
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

    fn receive_tail_starts_at(&self, idx: usize) -> bool {
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

    fn try_consume_receive_kind_name(&mut self) -> Option<(Option<String>, Span)> {
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
                .map(|t| {
                    matches!(t.kind, TokenKind::KwDongan) || Self::token_text_is(&t.kind, "동안")
                })
                .unwrap_or(false)
    }

    fn parse_manyak_if_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        self.advance();
        let condition = self.parse_expr()?;
        self.expect(&TokenKind::KwIramyeon, "이라면")?;
        self.parse_if_stmt_from_condition(start, condition)
    }

    fn is_current_if_start(&self) -> bool {
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

    fn parse_if_stmt_from_condition(
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

    fn find_if_else_lead_pos(&self) -> Option<usize> {
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
        let body = self.parse_body_with_chaebi_restriction()?;
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

    fn peek_legacy_boim_block(&self) -> bool {
        if !Self::token_text_is(&self.current().kind, "보임") {
            return false;
        }
        if self.peek_kind_n_is(1, |kind| matches!(kind, TokenKind::LBrace)) {
            return true;
        }
        self.peek_kind_n_is(1, |kind| matches!(kind, TokenKind::Colon))
            && self.peek_kind_n_is(2, |kind| matches!(kind, TokenKind::LBrace))
    }

    fn peek_public_observation_stmt(&self) -> bool {
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
                            .map(|token| Self::token_text_is(&token.kind, "보임"))
                            .unwrap_or(false);
                    }
                }
                TokenKind::Eof => return false,
                _ => {}
            }
        }
        false
    }

    fn parse_public_observation_stmt(&mut self) -> Result<Stmt, ParseError> {
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
        if !Self::token_text_is(&self.current().kind, "보임") {
            return Err(self.error("보임"));
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
            "성질" if self.in_imja_seed_body() => Some(DeclKind::Gureut),
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

    fn is_definition_stmt_start(&self) -> bool {
        if !self.check_ident() {
            return false;
        }
        self.peek_kind_n_is(1, |kind| {
            matches!(kind, TokenKind::Define | TokenKind::Colon)
        })
    }

    fn is_legacy_definition_stmt_start(&self) -> bool {
        self.check_ident() && self.peek_kind_n_is(1, |kind| matches!(kind, TokenKind::Equals))
    }

    fn parse_definition_stmt(&mut self) -> Result<Stmt, ParseError> {
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

    fn try_parse_reaction_provider(&mut self) -> Result<Option<Stmt>,ParseError> {
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

    fn consume_decl_item_control_suffix_if_present(&mut self) -> Result<(), ParseError> {
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

    fn is_decl_item_control_suffix_head(&self) -> bool {
        matches!(
            &self.current().kind,
            TokenKind::Ident(name) | TokenKind::Josa(name) if name == "매김" || name == "조건"
        )
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
        let body = self.parse_body_with_chaebi_restriction()?;
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

    fn parse_continue_stmt(&mut self) -> Result<Stmt, ParseError> {
        let s = self.current_span();
        self.expect(&TokenKind::KwGeonneottwigi, "건너뛰기")?;
        let mood = self.consume_stmt_terminator()?;
        Ok(Stmt::ContinueLoop {
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

    fn is_natural_foreach_start(&mut self) -> bool {
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

    fn parse_natural_foreach_stmt(&mut self) -> Result<Stmt, ParseError> {
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

    fn natural_iterable_item_type(&self, iterable: &Expr) -> TypeRef {
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
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        self.parse_primary_inner(true)
    }
    fn parse_primary_inner(&mut self, allow_bare_bound_call: bool) -> Result<Expr, ParseError> {
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

    fn parse_primary_tail(&mut self, mut expr: Expr, allow_bare_bound_call: bool) -> Result<Expr, ParseError> {
        expr = self.parse_index_suffix(expr)?;
        expr = self.parse_at_suffix(expr)?;
        expr = self.parse_not_suffix(expr)?;
        if allow_bare_bound_call {
            return self.try_parse_bare_bound_call(expr);
        }
        Ok(expr)
    }

    fn try_parse_seed_literal_params(&mut self) -> Result<Option<String>, ParseError> {
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
    fn peek_legacy_l11_brace_block(&self) -> bool {
        self.peek_kind_n_is(1, |kind| matches!(kind, TokenKind::LBrace))
            && ["쓰임", "드러냄", "공개"]
                .iter()
                .any(|name| Self::token_text_is(&self.current().kind, name))
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

    fn indexed_root(mut expr: &Expr) -> &Expr {
        loop {
            match &expr.kind {
                ExprKind::Call {func,args} if func == "차림.값" && args.len() == 2 => expr = &args[0].expr,
                ExprKind::MultiAxisSelection(value) => expr = &value.target,
                _ => return expr,
            }
        }
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
        if let ExprKind::MultiAxisSelection(selection) = &target.kind {
            let root = Self::indexed_root(&selection.target);
            if !matches!(root.kind, ExprKind::Var(_)) {
                return Err(ParseError {span:root.span,message:"E_TENSOR_TEMPORARY_UPDATE: 임시 텐서에는 다시결속할 수 없습니다".into()});
            }
            self.ensure_root_declared_for_write(root)?;
            if let [TensorAxisSelector::Index(index)] = selection.selectors.as_slice() {
                return Ok(Stmt::Mutate {id:self.next_id(),span:stmt_start.merge(&self.previous_span()),mood,
                    target:Expr::new(self.next_id(),target.span,ExprKind::SelectionPlace {
                        target:selection.target.clone(),index:index.clone(),
                    }),value,deferred,deferred_span});
            }
            let mut selection = selection.clone();
            selection.writable = true;
            return Ok(Stmt::Mutate {id:self.next_id(),span:stmt_start.merge(&self.previous_span()),mood,
                target:Expr::new(self.next_id(),target.span,ExprKind::MultiAxisSelection(selection)),value,deferred,deferred_span});
        }
        if let ExprKind::Call { func, args } = &target.kind {
            if func == "차림.값" && args.len() == 2 {
                let target_expr = args[0].expr.clone();
                let index_expr = args[1].expr.clone();
                let root = Self::indexed_root(&target_expr);
                if !matches!(root.kind, ExprKind::Var(_)) {
                    return Err(ParseError {
                        span: root.span,
                        message: "차림 인덱스 대입 대상은 이름만 허용됩니다".to_string(),
                    });
                }
                self.ensure_root_declared_for_write(root)?;
                return Ok(Stmt::Mutate {
                    id: self.next_id(),
                    span: stmt_start.merge(&self.previous_span()),
                    mood,
                    target: Expr::new(
                        self.next_id(),
                        target.span,
                        ExprKind::SelectionPlace {
                            target: Box::new(target_expr),
                            index: Box::new(index_expr),
                        },
                    ),
                    value,
                    deferred,
                    deferred_span,
                });
            }
        }
        if !self.is_simple_target(&target) {
            return Err(ParseError {
                span: target.span,
                message: "E_PARSE_EXPECTED_TARGET: 대입 대상은 이름 또는 기존 허용 경로여야 합니다"
                    .to_string(),
            });
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

    fn build_result_flow_destination(&mut self, destination: Expr) -> Result<Expr, ParseError> {
        if let ExprKind::MultiAxisSelection(selection) = &destination.kind {
            if !matches!(selection.target.kind, ExprKind::Var(_)) {
                return Err(ParseError {span:selection.target.span,message:"E_RESULT_FLOW_ROOT_REQUIRED: 결과흐름의 텐서 root는 이름이어야 합니다".into()});
            }
            self.ensure_root_declared_for_write(&selection.target)?;
            if let [TensorAxisSelector::Index(index)] = selection.selectors.as_slice() {
                return Ok(Expr::new(self.next_id(),destination.span,ExprKind::SelectionPlace {
                    target:selection.target.clone(),index:index.clone(),
                }));
            }
            let mut selection = selection.clone();
            selection.writable = true;
            return Ok(Expr::new(self.next_id(),destination.span,ExprKind::MultiAxisSelection(selection)));
        }
        if let ExprKind::Call { func, args } = &destination.kind {
            if func == "차림.값" && args.len() == 2 {
                let target = args[0].expr.clone();
                let index = args[1].expr.clone();
                if !matches!(target.kind, ExprKind::Var(_)) {
                    return Err(ParseError {
                        span: target.span,
                        message:
                            "E_RESULT_FLOW_ROOT_REQUIRED: `->` 대상 차림의 root는 이름이어야 합니다"
                                .to_string(),
                    });
                }
                self.ensure_root_declared_for_write(&target)?;
                return Ok(Expr::new(
                    self.next_id(),
                    destination.span,
                    ExprKind::SelectionPlace {
                        target: Box::new(target),
                        index: Box::new(index),
                    },
                ));
            }
        }
        Err(ParseError {
            span: destination.span,
            message: "E_RESULT_FLOW_DESTINATION_PLACE_REQUIRED: `->`의 대상은 indexed writable SelectionPlace여야 합니다".to_string(),
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
            alternate_forms: Vec::new(),
            seed_kind: SeedKind::Umjikssi,
            params: Vec::new(),
            result_type: None,
            body: Some(body),
            modifiers: Vec::new(),
        };
        items.push(TopLevelItem::SeedDef(seed));
        Ok(())
    }

    fn stmt_span(stmt: &Stmt) -> Span {
        match stmt {
            Stmt::SumDefinition { span, .. }
            | Stmt::SumMatch { span, .. }
            | Stmt::DeclBlock { span, .. }
            | Stmt::Definition { span, .. }
            | Stmt::DerivedBinding { span, .. }
            | Stmt::Mutate { span, .. }
            | Stmt::ResultFlow { span, .. }
            | Stmt::Expr { span, .. }
            | Stmt::Receive { span, .. }
            | Stmt::Send { span, .. }
            | Stmt::Show { span, .. }
            | Stmt::Inspect { span, .. }
            | Stmt::PublicObservation { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::If { span, .. }
            | Stmt::Try { span, .. }
            | Stmt::Choose { span, .. }
            | Stmt::Repeat { span, .. }
            | Stmt::While { span, .. }
            | Stmt::ForEach { span, .. }
            | Stmt::Quantifier { span, .. }
            | Stmt::Break { span, .. }
            | Stmt::ContinueLoop { span, .. }
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
        self.validate_ident_token_allow_reserved(token, &[])
    }

    fn validate_call_name_segment(
        &self,
        token: &Token,
        current_name: Option<&str>,
    ) -> Result<(), ParseError> {
        let name = match &token.kind {
            TokenKind::Ident(name) | TokenKind::Josa(name) => name.as_str(),
            _ => return Ok(()),
        };
        if name == "상태" && current_name == Some("격자게임상태") {
            return Ok(());
        }
        self.validate_ident_token(token)
    }

    fn validate_ident_token_allow_reserved(
        &self,
        token: &Token,
        allowed_reserved: &[&str],
    ) -> Result<(), ParseError> {
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
        if term_map::is_reserved_word(name)
            && !allowed_reserved.iter().any(|allowed| *allowed == name)
        {
            return Err(ParseError {
                span: self.to_ast_span(token.span),
                message: format!(
                    "NAME-LINT-01: 예약어 '{}'는 식별자로 사용할 수 없습니다",
                    name
                ),
            });
        }
        if !is_valid_prime_derivative_ident(name) {
            return Err(ParseError {
                span: self.to_ast_span(token.span),
                message: format!(
                    "NAME-LINT-02: prime 미분 표기는 식별자 끝의 1~2개 `'`만 허용합니다: {}",
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
        if self.should_skip_decl_control_call_name() {
            return Ok(None);
        }
        let first = if self.check_ident() {
            let token = self.advance();
            self.validate_call_name_segment(&token, None)?;
            token
        } else {
            return Err(self.error("함수"));
        };
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
            let next = if matches!(
                self.current().kind,
                TokenKind::Ident(_) | TokenKind::Josa(_)
            ) {
                let token = self.advance();
                self.validate_call_name_segment(&token, Some(&name))?;
                token
            } else {
                return Err(self.error("함수"));
            };
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

    fn should_skip_decl_control_call_name(&self) -> bool {
        let (TokenKind::Ident(name) | TokenKind::Josa(name)) = &self.current().kind else {
            return false;
        };
        if name != "매김" && name != "조건" {
            return false;
        }
        if self
            .tokens
            .get(self.pos + 1)
            .map(|token| matches!(token.kind, TokenKind::LBrace))
            .unwrap_or(false)
        {
            return true;
        }
        self.tokens
            .get(self.pos + 1)
            .map(|token| matches!(token.kind, TokenKind::Colon))
            .unwrap_or(false)
            && self
                .tokens
                .get(self.pos + 2)
                .map(|token| matches!(token.kind, TokenKind::LBrace))
                .unwrap_or(false)
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
                | TokenKind::BytesLit(_)
                | TokenKind::Atom(_)
                | TokenKind::Variable(_)
                | TokenKind::Nuance(_)
                | TokenKind::BranchRole(_)
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
            let open = self.advance();
            if matches!(&expr.kind, ExprKind::Var(name) if name == "차림") {
                let mut elements = Vec::new();
                if self.check(&TokenKind::RBracket) {
                    return Err(ParseError {
                        span: self.to_ast_span(open.span),
                        message: "E_COLLECTION_CONSTRUCTOR_ELEMENT_TYPE_REQUIRED: 빈 `차림[]`은 원소 타입을 결정할 수 없습니다".to_string(),
                    });
                }
                loop {
                    elements.push(self.parse_expr()?);
                    if self.check(&TokenKind::Comma) {
                        self.advance();
                        if self.check(&TokenKind::RBracket) {
                            return Err(self.error("차림 원소"));
                        }
                        continue;
                    }
                    break;
                }
                let close = self.expect(&TokenKind::RBracket, "]")?;
                let span = expr.span.merge(&self.to_ast_span(close.span));
                expr = Expr::new(
                    self.next_id(),
                    span,
                    ExprKind::CollectionConstructor { elements },
                );
                continue;
            }
            if self.check(&TokenKind::RBracket) {
                return Err(self.error("E_TENSOR_SELECTION: 비어 있는 축 선택은 허용되지 않습니다"));
            }
            let mut selectors = Vec::new();
            loop {
                if self.check(&TokenKind::Colon) {
                    self.advance();
                    selectors.push(TensorAxisSelector::Full);
                } else {
                    let index = self.parse_expr()?;
                    let range = match &index.kind {
                        ExprKind::Call {func,args} if func == "표준.범위" && args.len() == 3 => Some(args),
                        _ => None,
                    };
                    if let Some(args) = range {
                        let static_integer = |expr: &Expr| -> Option<u64> {
                            match &expr.kind {
                                ExprKind::Literal(Literal::Int(n)) => u64::try_from(*n).ok(),
                                ExprKind::Literal(Literal::Fixed64(n)) if n.raw_i64() >= 0 && n.frac_part() == 0 => u64::try_from(n.int_part()).ok(),
                                _ => None,
                            }
                        };
                        if static_integer(&args[2].expr) != Some(0) {
                            return Err(ParseError {span:index.span,message:"E_SELECTION_HALF_OPEN_RANGE: 선택 범위는 반열림 범위여야 합니다".into()});
                        }
                        match (static_integer(&args[0].expr),static_integer(&args[1].expr)) {
                            (Some(start),Some(end)) => {
                                if start > end { return Err(ParseError {span:index.span,message:"E_TENSOR_STATIC_RANGE: 범위 시작은 끝을 넘을 수 없습니다".into()}); }
                                selectors.push(TensorAxisSelector::Range {start,end});
                            }
                            _ if matches!(args[0].expr.kind,ExprKind::Literal(_))
                                && matches!(args[1].expr.kind,ExprKind::Literal(_)) => {
                                return Err(ParseError {span:index.span,message:"E_TENSOR_STATIC_RANGE: 범위는 0 이상의 정수여야 합니다".into()});
                            }
                            // RD-352 permits runtime list bounds. Preserve the
                            // authored range expression; Tensor still requires
                            // static bounds at its shared target-owned boundary.
                            _ => selectors.push(TensorAxisSelector::Index(Box::new(index))),
                        }
                    } else { selectors.push(TensorAxisSelector::Index(Box::new(index))); }
                }
                if !self.check(&TokenKind::Comma) { break; }
                self.advance();
                if self.check(&TokenKind::RBracket) { return Err(self.error("축 선택")); }
            }
            let close = self.expect(&TokenKind::RBracket, "]")?;
            let span = expr.span.merge(&self.to_ast_span(close.span));
            if selectors.len()>1 && selectors.iter().any(|selector| matches!(selector,
                TensorAxisSelector::Index(index) if matches!(&index.kind,
                    ExprKind::Call {func,..} if func=="표준.범위"))) {
                return Err(ParseError {span,message:"E_TENSOR_STATIC_RANGE: 다축 범위는 정적이어야 합니다".into()});
            }
            // RD-351/357: postfix selection retains its AST category. It must
            // not become an explicit list-provider call before target typing.
            expr = Expr::new(self.next_id(), span, ExprKind::MultiAxisSelection(MultiAxisSelection {
                target: Box::new(expr), selectors, writable: false,
            }));
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
            Stmt::SumDefinition { .. } => false,
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                self.expr_has_mutation(subject)
                    || arms.iter().any(|arm| self.body_has_mutation(&arm.body))
                    || default_body.as_ref().map_or(false, |body| self.body_has_mutation(body))
            }
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_mutation(expr))
            }),
            Stmt::Definition { value, .. } => self.expr_has_mutation(value),
            Stmt::Mutate { .. } | Stmt::ResultFlow { .. } | Stmt::DerivedBinding {..} => true,
            Stmt::Expr { expr, .. } => self.expr_has_mutation(expr),
            Stmt::Receive {
                condition, body, ..
            } => {
                condition
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_mutation(expr))
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
                    .map_or(false, |expr| self.expr_has_mutation(expr))
                    || self.expr_has_mutation(payload)
                    || self.expr_has_mutation(receiver)
            }
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
            Stmt::Break { .. } | Stmt::ContinueLoop { .. } => false,
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
            Stmt::PublicObservation { fields, .. } => fields
                .iter()
                .any(|field| self.expr_has_mutation(&field.value)),
            Stmt::Pragma { .. } => false,
        }
    }

    fn stmt_has_eval_do(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::SumDefinition { .. } => false,
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                self.expr_has_eval_do(subject)
                    || arms.iter().any(|arm| self.body_has_eval_do(&arm.body))
                    || default_body.as_ref().map_or(false, |body| self.body_has_eval_do(body))
            }
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_eval_do(expr))
            }),
            Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => self.expr_has_eval_do(value),
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => self.expr_has_eval_do(producer) || self.expr_has_eval_do(destination),
            Stmt::Expr { expr, .. } => self.expr_has_eval_do(expr),
            Stmt::Receive {
                condition, body, ..
            } => {
                condition
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_eval_do(expr))
                    || self.body_has_eval_do(body)
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                sender
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_eval_do(expr))
                    || self.expr_has_eval_do(payload)
                    || self.expr_has_eval_do(receiver)
            }
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
            Stmt::Break { .. } | Stmt::ContinueLoop { .. } => false,
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
            Stmt::PublicObservation { fields, .. } => fields
                .iter()
                .any(|field| self.expr_has_eval_do(&field.value)),
            Stmt::Pragma { .. } => false,
        }
    }

    fn stmt_has_random(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::SumDefinition { .. } => false,
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                self.expr_has_random(subject)
                    || arms.iter().any(|arm| self.body_has_random(&arm.body))
                    || default_body.as_ref().map_or(false, |body| self.body_has_random(body))
            }
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_random(expr))
            }),
            Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => self.expr_has_random(value),
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => self.expr_has_random(producer) || self.expr_has_random(destination),
            Stmt::Expr { expr, .. } => self.expr_has_random(expr),
            Stmt::Receive {
                condition, body, ..
            } => {
                condition
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_random(expr))
                    || self.body_has_random(body)
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                sender
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_random(expr))
                    || self.expr_has_random(payload)
                    || self.expr_has_random(receiver)
            }
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
            Stmt::Break { .. } | Stmt::ContinueLoop { .. } => false,
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
            Stmt::PublicObservation { fields, .. } => fields
                .iter()
                .any(|field| self.expr_has_random(&field.value)),
            Stmt::Pragma { .. } => false,
        }
    }

    fn stmt_has_show(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::SumDefinition { .. } => false,
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                self.expr_has_show(subject)
                    || arms.iter().any(|arm| self.body_has_show(&arm.body))
                    || default_body.as_ref().map_or(false, |body| self.body_has_show(body))
            }
            Stmt::Show { .. } | Stmt::Inspect { .. } => true,
            Stmt::DeclBlock { items, .. } => items.iter().any(|item| {
                item.value
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_show(expr))
            }),
            Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => self.expr_has_show(value),
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => self.expr_has_show(producer) || self.expr_has_show(destination),
            Stmt::Expr { expr, .. } => self.expr_has_show(expr),
            Stmt::Receive {
                condition, body, ..
            } => {
                condition
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_show(expr))
                    || self.body_has_show(body)
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                sender
                    .as_ref()
                    .map_or(false, |expr| self.expr_has_show(expr))
                    || self.expr_has_show(payload)
                    || self.expr_has_show(receiver)
            }
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
            Stmt::Break { .. }
            | Stmt::ContinueLoop { .. }
            | Stmt::MetaBlock { .. }
            | Stmt::Pragma { .. } => false,
            Stmt::PublicObservation { fields, .. } => {
                fields.iter().any(|field| self.expr_has_show(&field.value))
            }
        }
    }

    fn expr_has_mutation(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::SumBranchValue {payload,..} => payload.iter().any(|value|self.expr_has_mutation(&value.value)),
            ExprKind::TupleConstructor { elements } => elements.iter().any(|e| self.expr_has_mutation(e)),
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
            ExprKind::SumBranchValue {payload,..} => payload.iter().any(|value|self.expr_has_eval_do(&value.value)),
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
            ExprKind::TupleConstructor { elements } => elements.iter().any(|e| self.expr_has_eval_do(e)),
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
            ExprKind::SumBranchValue {payload,..} => payload.iter().any(|value|self.expr_has_random(&value.value)),
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
            ExprKind::TupleConstructor { elements } => elements.iter().any(|e| self.expr_has_random(e)),
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
            ExprKind::SumBranchValue {payload,..} => payload.iter().any(|value|self.expr_has_show(&value.value)),
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
            ExprKind::TupleConstructor { elements } => elements.iter().any(|e| self.expr_has_show(e)),
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
            let pin_token = self.expect_ident("핀")?;
            let mut pin = pin_token.raw.clone();
            let mut pin_end = pin_token.span.end;
            while let TokenKind::Josa(value) = &self.current().kind {
                if self.current().span.start != pin_end {
                    break;
                }
                pin.push_str(value);
                pin_end = self.current().span.end;
                self.advance();
            }
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
        // R1.1 retains the approved postfix value receiver without inventing
        // a josa or a public named pin. Other bare calls keep their grammar.
        if self.source_epoch==SourceEpoch::V25&&matches!(self.current().raw.as_str(),"가리키기"|"읽기") {
            let token=self.advance();
            let span=first_expr.span.merge(&self.to_ast_span(token.span));
            let arg=ArgBinding{id:self.next_id(),span:first_expr.span,expr:first_expr,josa:None,
                resolved_pin:None,binding_reason:BindingReason::Positional};
            return Ok(Expr::new(self.next_id(),span,ExprKind::Call{args:vec![arg],func:token.raw}));
        }
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

    fn parse_tensor_type(&mut self) -> Result<TypeRef, ParseError> {
        self.expect(&TokenKind::LParen, "텐서씨 (")?;
        let mut pins = HashSet::new();
        let mut element = None;
        let mut shape = None;
        while !self.check(&TokenKind::RParen) {
            let token = self.expect_ident("텐서씨 이름 핀")?;
            let pin = self.source.get(token.span.start..token.span.end)
                .ok_or_else(|| self.error("텐서씨 핀 원문 범위"))?.to_owned();
            if !pins.insert(pin.clone()) { return Err(self.error("중복 텐서씨 핀")); }
            self.expect(&TokenKind::Equals, "텐서씨 핀 =")?;
            match pin.as_str() {
                "원소씨" => {
                    let kind = self.parse_type_ref()?;
                    if !matches!(&kind, TypeRef::Named(name) if name == "셈수") {
                        return Err(self.error("V1 Tensor 원소씨는 정본 셈수만 허용됩니다"));
                    }
                    element = Some(Box::new(kind));
                }
                "모양" => shape = Some(self.parse_tensor_static_shape()?),
                _ => return Err(self.error("등록되지 않은 텐서씨 핀")),
            }
            if !self.check(&TokenKind::Comma) { break; }
            self.advance();
        }
        self.expect(&TokenKind::RParen, "텐서씨 )")?;
        if self.advance().raw != "텐서씨" { return Err(self.error("정본 텐서씨가 필요합니다")); }
        Ok(TypeRef::Tensor {
            element: element.ok_or_else(|| self.error("텐서씨 원소씨 누락"))?,
            shape: shape.ok_or_else(|| self.error("텐서씨 모양 누락"))?,
        })
    }

    fn parse_tensor_static_shape(&mut self) -> Result<Vec<u64>, ParseError> {
        self.expect(&TokenKind::LParen, "정적 모양 (")?;
        let mut axes = Vec::new();
        if self.check(&TokenKind::RParen) { self.advance(); return Ok(axes); }
        loop {
            let token = self.advance();
            if !matches!(token.kind, TokenKind::Integer(_)) || !token.raw.bytes().all(|b| b.is_ascii_digit()) {
                return Err(self.error("Tensor 축은 정적인 음이 아닌 정수여야 합니다"));
            }
            let axis = token.raw.parse::<u64>().map_err(|_| self.error("Tensor 축 범위 초과"))?;
            axes.push(axis);
            if self.check(&TokenKind::Comma) {
                self.advance();
                if self.check(&TokenKind::RParen) { self.advance(); break; }
            } else {
                if axes.len() == 1 { return Err(self.error("rank-1 모양은 (d,)로 적어야 합니다")); }
                self.expect(&TokenKind::RParen, "정적 모양 )")?;
                break;
            }
        }
        if !axes.contains(&0) {
            axes.iter().try_fold(1u64, |n, axis| n.checked_mul(*axis))
                .ok_or_else(|| self.error("Tensor 원소 수 곱셈 범위 초과"))?;
        }
        Ok(axes)
    }

    fn parse_tensor_provider(&mut self, provider: &str) -> Result<Expr, ParseError> {
        let start_token = self.expect(&TokenKind::LParen, "(")?;
        let start = self.to_ast_span(start_token.span);
        let mut pins = HashSet::new();
        let mut input = None;
        let mut element_type = None;
        let mut shape = None;
        while !self.check(&TokenKind::RParen) {
            let pin_token = self.expect_ident("Tensor 이름 핀")?;
            // RD-510 pins are exact authored spellings, not dialect aliases.
            let pin = self.source.get(pin_token.span.start..pin_token.span.end)
                .ok_or_else(|| self.error("Tensor 핀 원문 범위를 확인할 수 없습니다"))?
                .to_owned();
            if !pins.insert(pin.clone()) { return Err(self.error("중복 Tensor 핀")); }
            self.expect(&TokenKind::Equals, "Tensor 핀 =")?;
            match (provider, pin.as_str()) {
                ("텐서로", "차림") | ("차림으로", "텐서") => input = Some(Box::new(self.parse_expr()?)),
                ("텐서로", "원소씨") => {
                    let kind = self.parse_type_ref()?;
                    if !matches!(&kind, TypeRef::Named(name) if name == "셈수") {
                        return Err(self.error("V1 Tensor 원소씨는 정본 셈수만 허용됩니다"));
                    }
                    element_type = Some(kind);
                }
                ("텐서로", "모양") => shape = Some(self.parse_tensor_static_shape()?),
                _ => return Err(self.error(&format!("등록되지 않은 Tensor 핀: {provider} / {pin}"))),
            }
            if !self.check(&TokenKind::Comma) { break; }
            self.advance();
        }
        self.expect(&TokenKind::RParen, ")")?;
        let end = self.advance();
        if end.raw != provider { return Err(self.error("정확한 Tensor provider가 필요합니다")); }
        let input = input.ok_or_else(|| self.error("Tensor 입력 핀 누락"))?;
        let value = if provider == "텐서로" {
            TensorProviderExpr::FromCollection {
                input,
                element_type: element_type.ok_or_else(|| self.error("원소씨 핀 누락"))?,
                shape: shape.ok_or_else(|| self.error("모양 핀 누락"))?,
            }
        } else { TensorProviderExpr::ToCollection { input } };
        Ok(Expr::new(self.next_id(), start.merge(&self.to_ast_span(end.span)), ExprKind::Tensor(value)))
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
                        message: "상태머신: 바뀔때마다 절에는 세움값 이름이 필요합니다".to_string(),
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
        let program = match parser.parse_program(wrapped, "#assertion".to_string()) {
            Ok(program) => program,
            Err(_) => {
                if let Some((lhs, rhs)) = parse_symbolic_equivalence_assertion(&body_source) {
                    return Ok(Assertion {
                        body_source,
                        canon: format!("세움{{수식동치: {} = {}}}", lhs, rhs),
                    });
                }
                if let Some((lhs, rhs)) = parse_symbolic_relation_assertion(&body_source) {
                    return Ok(Assertion {
                        body_source,
                        canon: format!("세움{{수식관계: {} =:= {}}}", lhs, rhs),
                    });
                }
                return Err(ParseError {
                    span,
                    message: "세움: 유효한 본문을 해석할 수 없습니다".to_string(),
                });
            }
        };
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
        if !raw.starts_with('"') || !raw.ends_with('"') || raw.len() < 2 {
            return Err(ParseError {
                span,
                message: "글무늬 문자열은 \"...\" 형태여야 합니다".to_string(),
            });
        }
        let body = &raw[1..raw.len() - 1];
        let chars: Vec<char> = body.chars().collect();
        let mut out = String::new();
        let mut idx = 0usize;
        while idx < chars.len() {
            let ch = chars[idx];
            if ch == '\\' {
                idx += 1;
                out.push_str(&self.decode_text_escape_chars(&chars, &mut idx, span)?);
                continue;
            }
            out.push(ch);
            idx += 1;
        }
        Ok(out)
    }

    fn decode_text_escape_chars(
        &self,
        chars: &[char],
        idx: &mut usize,
        span: Span,
    ) -> Result<String, ParseError> {
        if *idx >= chars.len() {
            return Err(ParseError {
                span,
                message: "글무늬 문자열이 닫히지 않았습니다".to_string(),
            });
        }
        for (surface, value) in [
            ("줄", "\n"),
            ("칸", "\t"),
            ("앞", "\r"),
            ("따옴", "\""),
            ("역빗금", "\\"),
        ] {
            if chars_match_at(chars, *idx, surface) {
                *idx += surface.chars().count();
                return Ok(value.to_string());
            }
        }
        for surface in ["반전끝", "굵게끝", "반전", "굵게", "되돌림"] {
            if chars_match_at(chars, *idx, surface) {
                *idx += surface.chars().count();
                return Ok(format!("\\{}", surface));
            }
        }
        for surface in ["색", "배경"] {
            if chars_match_at(chars, *idx, surface) {
                *idx += surface.chars().count();
                return self.decode_rich_markup_escape_chars(surface, chars, idx, span);
            }
        }
        let ch = chars[*idx];
        match ch {
            'n' => {
                *idx += 1;
                Ok("\n".to_string())
            }
            'r' => {
                *idx += 1;
                Ok("\r".to_string())
            }
            't' => {
                *idx += 1;
                Ok("\t".to_string())
            }
            '"' => {
                *idx += 1;
                Ok("\"".to_string())
            }
            '\\' => {
                *idx += 1;
                Ok("\\".to_string())
            }
            other => Err(ParseError {
                span,
                message: format!("알 수 없는 글 이스케이프: \\{}", other),
            }),
        }
    }

    fn decode_rich_markup_escape_chars(
        &self,
        name: &str,
        chars: &[char],
        idx: &mut usize,
        span: Span,
    ) -> Result<String, ParseError> {
        if chars.get(*idx).copied() != Some('{') {
            return Err(ParseError {
                span,
                message: format!("콘솔 {} 표지는 '{{...}}'가 필요합니다", name),
            });
        }
        *idx += 1;
        let mut body = String::new();
        while *idx < chars.len() {
            let ch = chars[*idx];
            *idx += 1;
            if ch == '}' {
                return Ok(format!("\\{}{{{}}}", name, body));
            }
            if ch == '\n' || ch == '\r' {
                return Err(ParseError {
                    span,
                    message: "콘솔 표지 본문이 닫히지 않았습니다".to_string(),
                });
            }
            body.push(ch);
        }
        Err(ParseError {
            span,
            message: "콘솔 표지 본문이 닫히지 않았습니다".to_string(),
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
        let mut all_names = HashMap::new();
        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            if let Some(previous_product) = all_names.insert(&seed.canonical_name, seed.seed_kind.is_schema()) {
                if previous_product || seed.seed_kind.is_schema() {
                    return Err(ParseError { span: seed.span, message: format!(
                        "E_PRODUCT_DEFINITION_DUPLICATE: 정의 `{}`의 소유자가 중복되었습니다", seed.canonical_name) });
                }
            }
            if !seed.seed_kind.is_schema() {
                names.insert(seed.canonical_name.clone(), seed.span);
            }
        }
        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            if seed.seed_kind.is_schema() { continue; }
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

        // An exact surface may be shared only when typed resolution can still
        // distinguish the definitions. Identical typed call shapes would make
        // declaration/source order an implicit winner, so reject them here.
        let mut exact_surface_owners: HashMap<(String, String), (String, Span)> = HashMap::new();
        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            if seed.seed_kind.is_schema() { continue; }
            let typed_shape = format!(
                "{:?}",
                seed.params
                    .iter()
                    .map(|param| (&param.type_ref, &param.josa_list, param.optional))
                    .collect::<Vec<_>>()
            );
            for surface in seed.callable_surface_names() {
                let key = (surface.clone(), typed_shape.clone());
                if let Some((owner, _)) = exact_surface_owners.get(&key) {
                    if owner != &seed.canonical_name {
                        return Err(ParseError {
                            span: seed.span,
                            message: format!(
                                "E_CALLABLE_SURFACE_AMBIGUOUS: '{}' 표면이 '{}'와 '{}'에 같은 typed call shape로 등록되었습니다",
                                surface, owner, seed.canonical_name
                            ),
                        });
                    }
                } else {
                    exact_surface_owners.insert(key, (seed.canonical_name.clone(), seed.span));
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
            if seed.callable_surface_names().iter().any(|name|
                crate::bytes::BytesConversion::from_registered_surface(name).is_some()) {
                return Err(ParseError { span: seed.span,
                    message: "E_BYTES_ARGUMENT_BINDING: UTF-8 변환의 기존 정의 정체를 다시 선언할 수 없습니다".into() });
            }
            if seed.seed_kind == SeedKind::Product {
                self.product_definitions.insert(seed.canonical_name.clone(), (seed.id, seed.params.clone()));
                continue;
            }
            if matches!(seed.seed_kind, SeedKind::Schema(_)) {
                let SeedKind::Schema(category)=seed.seed_kind else {unreachable!()};
                self.schema_categories.insert(seed.canonical_name.clone(),category);
                self.product_definitions.insert(seed.canonical_name.clone(),(seed.id,seed.params.clone()));
                continue;
            }
            for surface in seed.callable_surface_names() {
                signatures.insert(surface, seed.params.clone());
            }
        }
        let mut known_seeds: HashSet<String> = signatures.keys().cloned().collect();
        for sig in minimal_stdlib_sigs() {
            known_seeds.insert(sig.name.to_string());
        }

        for item in &mut program.items {
            match item {
                TopLevelItem::SeedDef(seed) => {
                    if let Some(body) = &mut seed.body {
                        self.apply_defaults_in_body(body, &signatures, &known_seeds)?;
                    }
                }
            }
        }
        self.apply_defaults_in_stmts(&mut program.root_stmts, &signatures, &known_seeds)?;
        Ok(())
    }

    fn validate_units(&self, program: &CanonProgram) -> Result<(), ParseError> {
        for item in &program.items {
            let TopLevelItem::SeedDef(seed) = item;
            if let Some(body) = &seed.body {
                self.validate_body_units(body)?;
            }
        }
        for stmt in &program.root_stmts {
            self.validate_stmt_units(stmt)?;
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
            Stmt::SumDefinition { .. } => Ok(()),
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                self.infer_expr_dim(subject)?;
                for arm in arms { self.validate_body_units(&arm.body)?; }
                if let Some(body) = default_body { self.validate_body_units(body)?; }
                Ok(())
            }
            Stmt::DeclBlock { items, .. } => {
                for item in items {
                    if let Some(value) = &item.value {
                        self.infer_expr_dim(value)?;
                    }
                }
                Ok(())
            }
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => {
                self.infer_expr_dim(producer)?;
                self.infer_expr_dim(destination)?;
                Ok(())
            }
            Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => {
                self.infer_expr_dim(value)?;
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
            Stmt::Receive {
                condition, body, ..
            } => {
                if let Some(condition) = condition {
                    self.infer_expr_dim(condition)?;
                }
                self.validate_body_units(body)
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                if let Some(sender) = sender {
                    self.infer_expr_dim(sender)?;
                }
                self.infer_expr_dim(payload)?;
                self.infer_expr_dim(receiver)?;
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
            Stmt::Break { .. } | Stmt::ContinueLoop { .. } => Ok(()),
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
            Stmt::PublicObservation { fields, .. } => {
                for field in fields {
                    self.infer_expr_dim(&field.value)?;
                }
                Ok(())
            }
            Stmt::Pragma { .. } => Ok(()),
        }
    }

    fn infer_expr_dim(&self, expr: &Expr) -> Result<DimState, ParseError> {
        match &expr.kind {
            ExprKind::SumBranchValue { payload, .. } => {
                for payload in payload { self.infer_expr_dim(&payload.value)?; }
                Ok(DimState::Unknown)
            }
            ExprKind::ProductConstruction { fields, .. } => {
                for field in fields { self.infer_expr_dim(&field.expr)?; }
                Ok(DimState::Unknown)
            }
            ExprKind::Tensor(value) => {
                self.infer_expr_dim(value.input())?;
                Ok(DimState::Unknown)
            }
            ExprKind::MultiAxisSelection(value) => {
                for child in value.children() { self.infer_expr_dim(child)?; }
                Ok(DimState::Unknown)
            }
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
            ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
                for element in elements {
                    self.infer_expr_dim(element)?;
                }
                Ok(DimState::Unknown)
            }
            ExprKind::SelectionPlace { target, index } => {
                self.infer_expr_dim(target)?;
                self.infer_expr_dim(index)?;
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
                    "=:=" => Ok(DimState::Unknown),
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
        self.apply_defaults_in_stmts(&mut body.stmts, signatures, known_seeds)
    }

    fn apply_defaults_in_stmts(
        &mut self,
        stmts: &mut [Stmt],
        signatures: &HashMap<String, Vec<ParamPin>>,
        known_seeds: &HashSet<String>,
    ) -> Result<(), ParseError> {
        for stmt in stmts {
            match stmt {
                Stmt::SumDefinition { .. } => {}
                Stmt::SumMatch { subject, arms, default_body, .. } => {
                    self.apply_defaults_in_expr(subject, signatures, known_seeds)?;
                    for arm in arms { self.apply_defaults_in_body(&mut arm.body, signatures, known_seeds)?; }
                    if let Some(body) = default_body { self.apply_defaults_in_body(body, signatures, known_seeds)?; }
                }
                Stmt::DeclBlock { items, .. } => {
                    for item in items {
                        if let Some(value) = &mut item.value {
                            self.apply_defaults_in_expr(value, signatures, known_seeds)?;
                        }
                    }
                }
                Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => {
                    self.apply_defaults_in_expr(value, signatures, known_seeds)?;
                }
                Stmt::Mutate { target, value, .. } => {
                    self.apply_defaults_in_expr(target, signatures, known_seeds)?;
                    self.apply_defaults_in_expr(value, signatures, known_seeds)?;
                }
                Stmt::ResultFlow {
                    producer,
                    destination,
                    ..
                } => {
                    self.apply_defaults_in_expr(producer, signatures, known_seeds)?;
                    self.apply_defaults_in_expr(destination, signatures, known_seeds)?;
                }
                Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                    self.apply_defaults_in_expr(expr, signatures, known_seeds)?
                }
                Stmt::Receive {
                    condition, body, ..
                } => {
                    if let Some(condition) = condition {
                        self.apply_defaults_in_expr(condition, signatures, known_seeds)?;
                    }
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::Send {
                    sender,
                    payload,
                    receiver,
                    ..
                } => {
                    if let Some(sender) = sender {
                        self.apply_defaults_in_expr(sender, signatures, known_seeds)?;
                    }
                    self.apply_defaults_in_expr(payload, signatures, known_seeds)?;
                    self.apply_defaults_in_expr(receiver, signatures, known_seeds)?;
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
                Stmt::ForEach {
                    id,
                    item,
                    item_type,
                    iterable,
                    body,
                    ..
                } => {
                    let item_span = iterable.span;
                    self.apply_defaults_in_expr(iterable, signatures, known_seeds)?;
                    if self.natural_foreach_ids.contains(id) {
                        self.current_item_stack.push((
                            item.clone(),
                            item_type.clone().unwrap_or(TypeRef::Infer),
                            item_span,
                        ));
                        let result = self.apply_defaults_in_body(body, signatures, known_seeds);
                        self.current_item_stack.pop();
                        result?;
                    } else {
                        self.apply_defaults_in_body(body, signatures, known_seeds)?;
                    }
                }
                Stmt::Quantifier { body, .. } => {
                    self.apply_defaults_in_body(body, signatures, known_seeds)?;
                }
                Stmt::Break { .. } | Stmt::ContinueLoop { .. } => {}
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
                Stmt::PublicObservation { fields, .. } => {
                    for field in fields {
                        self.apply_defaults_in_expr(&mut field.value, signatures, known_seeds)?;
                    }
                }
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
            ExprKind::SumBranchValue {payload,..} => {
                for value in payload {self.apply_defaults_in_expr(&mut value.value,signatures,known_seeds)?;}
            }
            ExprKind::Call { args, func } => {
                for arg in args.iter_mut() {
                    self.apply_defaults_in_expr(&mut arg.expr, signatures, known_seeds)?;
                }
                if let Some((definition, schema)) = self.product_definitions.get(func) {
                    let category=self.schema_categories.get(func).copied();
                    if category.is_some()&&!self.schema_construction_nodes.contains(&expr.id) {
                        return Err(ParseError{span:expr.span,message:"E_SCHEMA_CONSTRUCTION_SURFACE: 명목 구성은 '(성분=값)인 이름'으로 쓰세요".into()});
                    }
                    let mut seen = HashSet::new();
                    for arg in args.iter() {
                        let pin = arg.resolved_pin.as_ref().ok_or_else(|| ParseError {
                            span: arg.span, message: "E_PRODUCT_NAMED_FIELD_REQUIRED: 구성할 필드 이름을 명시하세요".into()
                        })?;
                        if arg.josa.is_some() || !schema.iter().any(|field| field.pin_name == *pin) {
                            return Err(ParseError { span: arg.span, message:
                                format!("E_PRODUCT_FIELD_UNKNOWN: `{func}`의 필드 `{pin}`가 아닙니다") });
                        }
                        if !seen.insert(pin) {
                            return Err(ParseError { span: arg.span, message:
                                format!("E_PRODUCT_FIELD_DUPLICATE: 필드 `{pin}`가 중복되었습니다") });
                        }
                        let expected = &schema.iter().find(|field| field.pin_name == *pin).unwrap().type_ref;
                        let literal = match &arg.expr.kind {
                            ExprKind::Literal(Literal::Int(value)) => i32::try_from(*value).ok()
                                .map(|value| crate::runtime::Value::Fixed64(Fixed64::from_i32(value))),
                            ExprKind::Literal(Literal::Fixed64(value)) => Some(crate::runtime::Value::Fixed64(*value)),
                            ExprKind::Literal(Literal::String(value) | Literal::Atom(value)) => Some(crate::runtime::Value::String(value.clone())),
                            ExprKind::Literal(Literal::Bool(value)) => Some(crate::runtime::Value::Bool(*value)),
                            ExprKind::Literal(Literal::None) => Some(crate::runtime::Value::None),
                            _ => None,
                        };
                        if let Some(literal) = literal {
                            if let Err(detail) = crate::runtime::check_value_type(&literal, expected) {
                                return Err(ParseError { span: arg.expr.span, message: format!(
                                    "E_RUNTIME_TYPE_MISMATCH: 필드={pin} 기대={} 실제={}", detail.expected, detail.actual) });
                            }
                        }
                    }
                    if seen.len() != schema.len() {
                        return Err(ParseError { span: expr.span, message:
                            format!("E_PRODUCT_FIELD_MISSING: `{func}`의 모든 필드를 명시해야 합니다") });
                    }
                    // Retain authored value evaluation order. Record identity
                    // is owned by the schema, never declaration/source order.
                    expr.kind = ExprKind::ProductConstruction {
                        owner: func.clone(), definition: *definition, category, fields: args.clone(),
                    };
                    return Ok(());
                }
                if self.source_epoch==SourceEpoch::V25&&func=="조작단계"&&!signatures.contains_key(func) {
                    if args.len()!=1||args[0].resolved_pin.as_deref()!=Some("순서")||args[0].josa.is_some(){
                        return Err(ParseError{span:expr.span,message:"E_INPUT_PHASE_PIN: '(순서=원순번) 조작단계'의 필수 받는자리 하나를 명시하세요".into()});
                    }
                    self.manipulation_phase_calls.insert(expr.id,ManipulationPhaseCallBinding{
                        definition:"B1.DEF.INPUT_PHASE.R1",ordinal_pin:"B1.PIN.INPUT_PHASE.ORDINAL.R1"});
                    return Ok(());
                }
                if self.source_epoch==SourceEpoch::V25&&func=="폐기하기" {
                    if signatures.contains_key(func){return Err(ParseError{span:expr.span,message:"E_VC04_RETIRE_AMBIGUOUS: 폐기하기의 사용자 정의와 등록 정의가 경쟁합니다".into()});}
                    if args.len()!=1||args[0].resolved_pin.as_deref()!=Some("대상")||args[0].josa.is_some(){return Err(ParseError{span:expr.span,message:"E_VC04_RETIRE_PIN: '(대상=표적) 폐기하기'의 필수 받는자리 하나만 사용하세요".into()});}
                    self.retirement_commands.insert(expr.id,RetirementCommandBinding{definition:"VC04.DEF.PYEGIHAGI.R1",target_pin:"VC04.PIN.PYEGIHAGI.TARGET.R1"});
                    return Ok(());
                }
                if self.source_epoch==SourceEpoch::V25&&matches!(func.as_str(),"가리키기"|"읽기") {
                    if signatures.contains_key(func){return Err(ParseError{span:expr.span,message:format!("E_VC04_REFERENCE_AMBIGUOUS: 사용자 정의와 등록 `{func}`가 경쟁합니다")});}
                    if args.len()!=1||args[0].resolved_pin.is_some()||args[0].josa.is_some(){return Err(ParseError{span:expr.span,message:format!("E_VC04_REFERENCE_RECEIVER: '{} {func}'의 값 받는자리 하나만 사용하세요",if func=="읽기"{"가리킴값"}else{"임자값"})});}
                    let binding=if func=="가리키기" {SubjectReferenceCallBinding{definition:"VC04.DEF.GARIKIGI.R1",receiver_pin:"VC04.PIN.GARIKIGI.TARGET.R1"}}
                        else {SubjectReferenceCallBinding{definition:"VC04.DEF.ILGI.R1",receiver_pin:"VC04.PIN.ILGI.REFERENCE.R1"}};
                    self.subject_reference_calls.insert(expr.id,binding);
                    return Ok(());
                }
                let func_name = func.clone();
                self.record_registered_call(expr.id, expr.span, &func_name);
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
            ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
                for element in elements {
                    self.apply_defaults_in_expr(element, signatures, known_seeds)?;
                }
            }
            ExprKind::SelectionPlace { target, index } => {
                self.apply_defaults_in_expr(target, signatures, known_seeds)?;
                self.apply_defaults_in_expr(index, signatures, known_seeds)?;
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
            ExprKind::ProductConstruction { fields, .. } => {
                for field in fields {
                    self.apply_defaults_in_expr(&mut field.expr, signatures, known_seeds)?;
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
        self.bind_bytes_call(expr)?;
        Ok(())
    }

    fn bind_bytes_call(&mut self, expr: &Expr) -> Result<(), ParseError> {
        if let ExprKind::Call { func, args } = &expr.kind {
            if let Some(operation) = crate::bytes::BytesConversion::from_registered_surface(func) {
                operation.validate(args, expr.span)?;
                self.bytes_calls.insert(expr.id, operation);
            }
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
            if matches!(&stage.kind, ExprKind::Call { func, .. } if self.product_definitions.contains_key(func)) {
                // Explicit Product fields have no implicit flow/current-item pin.
                self.apply_defaults_in_expr(stage, signatures, known_seeds)?;
                continue;
            }
            match &mut stage.kind {
                ExprKind::Call { args, func } => {
                    for arg in args.iter_mut() {
                        self.apply_defaults_in_expr(&mut arg.expr, signatures, known_seeds)?;
                    }
                    let func_name = func.clone();
                    self.record_registered_call(stage.id, stage.span, &func_name);
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
            self.bind_bytes_call(stage)?;
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
        if let Some(owner) = self.registered_forms.get(func) {
            return Ok((owner.clone(), owner.clone()));
        }
        let resolved = resolve_known_call_target(func, known_seeds, span)?;
        if crate::bytes::BytesConversion::from_registered_surface(&resolved.0).is_some()
            && func != resolved.0 {
            return Err(ParseError {
                span,
                message: format!("E_REGISTERED_FORM_REQUIRED: '{}'은 등록된 UTF-8 변환 표면이 아닙니다", func),
            });
        }
        if self.explicitly_registered_owners.contains(&resolved.0) {
            return Err(ParseError {
                span,
                message: format!(
                    "E_REGISTERED_FORM_REQUIRED: '{}'은 명시적으로 등록된 호출 표면이 아닙니다",
                    func
                ),
            });
        }
        Ok(resolved)
    }

    fn record_registered_call(&mut self, id: NodeId, span: Span, form: &str) {
        if let Some(candidates) = self.registered_form_candidates.get(form) {
            self.registered_call_candidates
                .insert(id, candidates.clone());
            self.registered_call_spans.insert(id, span);
        }
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
        let authored_arg_count = args.len();
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
        let mut missing_required = Vec::new();
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
            missing_required.push(idx);
        }

        if pos_iter.next().is_some() {
            return Err(ParseError {
                span,
                message: "인자 수가 너무 많습니다".to_string(),
            });
        }

        if let Some((item_name, item_type, item_span)) = self.current_item_stack.last().cloned() {
            if missing_required.is_empty() && authored_arg_count == 0 {
                return Err(ParseError {
                    span,
                    message: "E_CURRENT_ITEM_NO_REQUIRED_PIN: current item을 받을 미충족 필수 pin이 없습니다".to_string(),
                });
            }
            if !missing_required.is_empty() {
                if missing_required.len() != 1 {
                    let pins = missing_required
                        .iter()
                        .map(|idx| params[*idx].pin_name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(ParseError {
                        span,
                        message: format!(
                            "E_CURRENT_ITEM_AMBIGUOUS: 현재 항목 결속 후보가 하나가 아닙니다: {pins}"
                        ),
                    });
                }
                let idx = missing_required[0];
                let expected = &params[idx].type_ref;
                if matches!(expected, TypeRef::Function { .. }) {
                    return Err(ParseError {
                        span,
                        message: format!(
                            "E_CURRENT_ITEM_OWNERSHIP_UNSAFE: 함수형 핀 '{}'에는 current item을 자동 결속하지 않습니다",
                            params[idx].pin_name
                        ),
                    });
                }
                if !Self::current_item_type_compatible(&item_type, expected) {
                    return Err(ParseError {
                        span,
                        message: format!(
                            "E_CURRENT_ITEM_TYPE_MISMATCH: current item은 핀 '{}' 타입과 안전하게 맞지 않습니다",
                            params[idx].pin_name
                        ),
                    });
                }
                let expr = Expr::new(self.next_id(), item_span, ExprKind::Var(item_name));
                bound[idx] = Some(ArgBinding {
                    id: self.next_id(),
                    span,
                    expr,
                    josa: None,
                    resolved_pin: Some(params[idx].pin_name.clone()),
                    binding_reason: BindingReason::ContextItemInjected,
                });
            }
        } else if !missing_required.is_empty() {
            let pin = &params[missing_required[0]].pin_name;
            return Err(ParseError {
                span,
                message: format!("E_CALL_REQUIRED_PIN_MISSING: 필수 인자 '{pin}' 누락"),
            });
        }

        *args = bound.into_iter().flatten().collect();
        Ok(())
    }

    fn current_item_type_compatible(actual: &TypeRef, expected: &TypeRef) -> bool {
        match (actual, expected) {
            (_, TypeRef::Infer) => true,
            (TypeRef::Infer, _) => false,
            _ => actual == expected,
        }
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

fn parse_symbolic_equivalence_assertion(raw: &str) -> Option<(String, String)> {
    let (lhs, rest) = parse_ascii_formula_prefix(raw)?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let (rhs, rest) = parse_ascii_formula_prefix(rest)?;
    let rest = rest.trim();
    if !rest.is_empty() && rest != "." {
        return None;
    }
    Some((lhs, rhs))
}

fn is_valid_prime_derivative_ident(name: &str) -> bool {
    let mut trailing_primes = 0usize;
    let mut seen_non_prime = false;
    for ch in name.chars().rev() {
        if ch == '\'' && !seen_non_prime {
            trailing_primes += 1;
            continue;
        }
        if ch == '\'' {
            return false;
        }
        seen_non_prime = true;
    }
    trailing_primes <= 2
}

fn parse_symbolic_relation_assertion(raw: &str) -> Option<(String, String)> {
    let (lhs, rest) = parse_ascii_formula_prefix(raw)?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix("=:=")?.trim_start();
    let (rhs, rest) = parse_ascii_formula_prefix(rest)?;
    let rest = rest.trim();
    if !rest.is_empty() && rest != "." {
        return None;
    }
    Some((lhs, rhs))
}

fn parse_ascii_formula_prefix(raw: &str) -> Option<(String, &str)> {
    let mut rest = raw.trim_start();
    rest = rest.strip_prefix('(')?.trim_start();
    rest = rest.strip_prefix("#ascii")?.trim_start();
    rest = rest.strip_prefix(')')?.trim_start();
    rest = rest.strip_prefix("수식")?.trim_start();
    rest = rest.strip_prefix('{')?;
    let mut depth = 1usize;
    for (idx, ch) in rest.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let body = rest[..idx].trim().to_string();
                    let tail = &rest[idx + ch.len_utf8()..];
                    return Some((body, tail));
                }
            }
            _ => {}
        }
    }
    None
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

fn chars_match_at(chars: &[char], idx: usize, surface: &str) -> bool {
    let surface_chars: Vec<char> = surface.chars().collect();
    if idx + surface_chars.len() > chars.len() {
        return false;
    }
    chars[idx..idx + surface_chars.len()] == surface_chars[..]
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

pub fn resolve_known_call_target(
    func: &str,
    known_seeds: &HashSet<String>,
    span: Span,
) -> Result<(String, String), ParseError> {
    if known_seeds.contains(func) {
        return Ok((func.to_string(), func.to_string()));
    }
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
            return resolve_call_tail_candidates(
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
            return resolve_call_tail_candidates(func, short_tail, None, known_seeds, span);
        }
    }
    Ok((func.to_string(), func.to_string()))
}

fn resolve_call_tail_candidates(
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

#[derive(Clone, Debug)]
enum FieldShape {
    Unknown,
    Scalar,
    Record(HashMap<String, FieldShape>),
}

fn validate_record_field_accesses(program: &CanonProgram) -> Result<(), ParseError> {
    let root = Body { id: program.id, span: Span::new(0, program.origin.source.len()),
        stmts: program.root_stmts.clone() };
    validate_field_body(&root, &mut HashMap::new())?;
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        let mut env = HashMap::new();
        for param in &seed.params {
            env.insert(param.pin_name.clone(), FieldShape::Unknown);
            if let Some(default) = &param.default_value {
                validate_field_expr(default, &env)?;
            }
        }
        if let Some(body) = &seed.body {
            validate_field_body(body, &mut env)?;
        }
    }
    Ok(())
}

fn validate_field_body(
    body: &Body,
    env: &mut HashMap<String, FieldShape>,
) -> Result<(), ParseError> {
    for stmt in &body.stmts {
        match stmt {
            Stmt::SumDefinition { .. } => {}
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                validate_field_expr(subject, env)?;
                for arm in arms {
                    let mut nested = env.clone();
                    for binder in &arm.binder { nested.insert(binder.pin.clone(), FieldShape::Unknown); }
                    validate_field_body(&arm.body, &mut nested)?;
                }
                if let Some(body) = default_body { validate_field_body(body, &mut env.clone())?; }
            }
            Stmt::DeclBlock { items, .. } => {
                for item in items {
                    let shape = match &item.value {
                        Some(value) => validate_field_expr(value, env)?,
                        None => FieldShape::Unknown,
                    };
                    env.insert(item.name.clone(), shape);
                }
            }
            Stmt::Definition { name, value, .. } | Stmt::DerivedBinding {name,value,..} => {
                let shape = validate_field_expr(value, env)?;
                env.insert(name.clone(), shape);
            }
            Stmt::Mutate { target, value, .. } => {
                let shape = validate_field_expr(value, env)?;
                validate_field_expr(target, env)?;
                if let ExprKind::Var(name) = &target.kind {
                    env.insert(name.clone(), shape);
                }
            }
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => {
                validate_field_expr(producer, env)?;
                validate_field_expr(destination, env)?;
            }
            Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                validate_field_expr(expr, env)?;
            }
            Stmt::PublicObservation { fields, .. } => {
                for field in fields {
                    validate_field_expr(&field.value, env)?;
                }
            }
            Stmt::Receive {
                condition, body, ..
            } => {
                if let Some(condition) = condition {
                    validate_field_expr(condition, env)?;
                }
                validate_field_body(body, &mut env.clone())?;
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                if let Some(sender) = sender {
                    validate_field_expr(sender, env)?;
                }
                validate_field_expr(payload, env)?;
                validate_field_expr(receiver, env)?;
            }
            Stmt::Return { value, .. } => {
                validate_field_expr(value, env)?;
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                validate_field_expr(condition, env)?;
                validate_field_body(then_body, &mut env.clone())?;
                if let Some(else_body) = else_body {
                    validate_field_body(else_body, &mut env.clone())?;
                }
            }
            Stmt::Try { action, body, .. } => {
                validate_field_expr(action, env)?;
                validate_field_body(body, &mut env.clone())?;
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    validate_field_expr(&branch.condition, env)?;
                    validate_field_body(&branch.body, &mut env.clone())?;
                }
                validate_field_body(else_body, &mut env.clone())?;
            }
            Stmt::Repeat { body, .. } | Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. } => {
                validate_field_body(body, &mut env.clone())?
            }
            Stmt::While {
                condition, body, ..
            }
            | Stmt::Guard {
                condition, body, ..
            }
            | Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => {
                validate_field_expr(condition, env)?;
                validate_field_body(body, &mut env.clone())?;
            }
            Stmt::ForEach {
                item,
                iterable,
                body,
                ..
            } => {
                validate_field_expr(iterable, env)?;
                let mut nested = env.clone();
                nested.insert(item.clone(), FieldShape::Unknown);
                validate_field_body(body, &mut nested)?;
            }
            Stmt::Quantifier { variable, body, .. } => {
                let mut nested = env.clone();
                nested.insert(variable.clone(), FieldShape::Unknown);
                validate_field_body(body, &mut nested)?;
            }
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                validate_field_expr(condition, env)?;
                if let Some(then_body) = then_body {
                    validate_field_body(then_body, &mut env.clone())?;
                }
                validate_field_body(else_body, &mut env.clone())?;
            }
            Stmt::Break { .. }
            | Stmt::ContinueLoop { .. }
            | Stmt::MetaBlock { .. }
            | Stmt::Pragma { .. } => {}
        }
    }
    Ok(())
}

fn validate_field_expr(
    expr: &Expr,
    env: &HashMap<String, FieldShape>,
) -> Result<FieldShape, ParseError> {
    match &expr.kind {
        ExprKind::SumBranchValue { payload, .. } => {
            for payload in payload { validate_field_expr(&payload.value, env)?; }
            Ok(FieldShape::Unknown)
        }
        ExprKind::ProductConstruction { fields, .. } => {
            let mut shape = HashMap::new();
            for field in fields {
                let name = field.resolved_pin.as_ref().ok_or_else(|| ParseError {
                    span: field.span, message: "E_PRODUCT_NAMED_FIELD_REQUIRED: 구성 필드 이름이 없습니다".into()
                })?;
                shape.insert(name.clone(), validate_field_expr(&field.expr, env)?);
            }
            Ok(FieldShape::Record(shape))
        }
        ExprKind::Tensor(value) => {
            validate_field_expr(value.input(), env)?;
            Ok(FieldShape::Unknown)
        }
        ExprKind::MultiAxisSelection(value) => {
            for child in value.children() { validate_field_expr(child, env)?; }
            Ok(FieldShape::Unknown)
        }
        ExprKind::Literal(_) | ExprKind::Formula(_) | ExprKind::Template(_) => Ok(FieldShape::Scalar),
        ExprKind::Var(name) => Ok(env.get(name).cloned().unwrap_or(FieldShape::Unknown)),
        ExprKind::FieldAccess { target, field, field_span } => match validate_field_expr(target, env)? {
            FieldShape::Record(fields) => fields.get(field).cloned().ok_or_else(|| ParseError {
                span: *field_span,
                message: format!("E_FIELD_MISSING: 묶음에 `{field}` 필드가 없습니다; `A.B`의 B 이름을 확인하세요"),
            }),
            FieldShape::Scalar => Err(ParseError {
                span: *field_span,
                message: format!("E_FIELD_TARGET_TYPE: `{field}` 앞의 값은 이름형 값꾸러미가 아닙니다"),
            }),
            FieldShape::Unknown => Ok(FieldShape::Unknown),
        },
        ExprKind::Pack { fields } => {
            let mut shape = HashMap::new();
            for (name, value) in fields {
                shape.insert(name.clone(), validate_field_expr(value, env)?);
            }
            Ok(FieldShape::Record(shape))
        }
        ExprKind::SeedLiteral { body, .. }
        | ExprKind::Nuance { expr: body, .. }
        | ExprKind::Suffix { value: body, .. } => {
            validate_field_expr(body, env)?;
            Ok(FieldShape::Unknown)
        }
        ExprKind::Call { args, .. } => {
            for arg in args {
                validate_field_expr(&arg.expr, env)?;
            }
            Ok(FieldShape::Unknown)
        }
        ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
            for element in elements {
                validate_field_expr(element, env)?;
            }
            Ok(FieldShape::Unknown)
        }
        ExprKind::SelectionPlace { target, index } => {
            validate_field_expr(target, env)?;
            validate_field_expr(index, env)?;
            Ok(FieldShape::Unknown)
        }
        ExprKind::Infix { left, right, .. } => {
            validate_field_expr(left, env)?;
            validate_field_expr(right, env)?;
            Ok(FieldShape::Unknown)
        }
        ExprKind::Thunk(body) => {
            validate_field_body(body, &mut env.clone())?;
            Ok(FieldShape::Unknown)
        }
        ExprKind::Eval { thunk, .. } => {
            validate_field_expr(thunk, env)?;
            Ok(FieldShape::Unknown)
        }
        ExprKind::Pipe { stages } => {
            for stage in stages {
                validate_field_expr(stage, env)?;
            }
            Ok(FieldShape::Unknown)
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                validate_field_expr(value, env)?;
            }
            Ok(FieldShape::Unknown)
        }
        ExprKind::FlowValue | ExprKind::Assertion(_) | ExprKind::StateMachine(_) => Ok(FieldShape::Unknown),
    }
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl ParseError {
    pub fn code(&self) -> &'static str {
        for code in ["E_VC04_RETIRE_AMBIGUOUS", "E_VC04_RETIRE_BINDING", "E_VC04_RETIRE_PIN", "E_VC04_RETIRE_TARGET", "E_VC04_COMMAND_VALUE", "E_VC04_PURE_WORLD_WRITE", "E_SCHEMA_CONSTRUCTION_SURFACE",
            "E_PRODUCT_DEFINITION_DUPLICATE", "E_PRODUCT_SCHEMA_BODY", "E_PRODUCT_FIELD_SCHEMA", "E_PRODUCT_FIELD_DUPLICATE",
            "E_PRODUCT_NAMED_FIELD_REQUIRED", "E_PRODUCT_FIELD_UNKNOWN", "E_PRODUCT_FIELD_MISSING",
            "E_RUNTIME_TYPE_MISMATCH", "E_BYTES_ARGUMENT_BINDING", "E_BYTES_LITERAL_INVALID",
            "E_TYPE_MISMATCH"] {
            if self.message.starts_with(code) { return code; }
        }
        if self.message.starts_with("E_V25_IF_SURFACE:") {
            return "E_V25_IF_SURFACE";
        }
        if self
            .message
            .starts_with("E_V25_DEFINITION_MIGRATION_REQUIRED:")
        {
            return "E_V25_DEFINITION_MIGRATION_REQUIRED";
        }
        if self.message.starts_with("E_PARSE_EXPECTED_TARGET:") {
            return "E_PARSE_EXPECTED_TARGET";
        }
        if self.message.starts_with("E_CHAEBI_IN_LOOP:") {
            return "E_CHAEBI_IN_LOOP";
        }
        if self.message.starts_with("E_BOIM_LEGACY_REMOVED:") {
            return "E_BOIM_LEGACY_REMOVED";
        }
        if self.message.starts_with("E_BOIM_EMPTY_PACK:") {
            return "E_BOIM_EMPTY_PACK";
        }
        if self.message.starts_with("E_BOIM_DUPLICATE_FIELD:") {
            return "E_BOIM_DUPLICATE_FIELD";
        }
        if self.message.starts_with("E_BOIM_FIXED_BINDING_FORBIDDEN:") {
            return "E_BOIM_FIXED_BINDING_FORBIDDEN";
        }
        if self.message.starts_with("E_BOIM_FIELD_NAME_REQUIRED:") {
            return "E_BOIM_FIELD_NAME_REQUIRED";
        }
        if self.message.starts_with("E_SEED_NAME_CONFLICT_HA:") {
            return "E_SEED_NAME_CONFLICT_HA";
        }
        if self.message.starts_with("PIPE-FLOW-INJECT-AMBIGUOUS:") {
            return "PIPE-FLOW-INJECT-AMBIGUOUS";
        }
        if self.message.starts_with("E_CALL_TAIL_AMBIGUOUS:") {
            return "E_CALL_TAIL_AMBIGUOUS";
        }
        if self.message.starts_with("E_CURRENT_ITEM_AMBIGUOUS:") {
            return "E_CURRENT_ITEM_AMBIGUOUS";
        }
        if self.message.starts_with("E_CURRENT_ITEM_NO_REQUIRED_PIN:") {
            return "E_CURRENT_ITEM_NO_REQUIRED_PIN";
        }
        if self.message.starts_with("E_CURRENT_ITEM_TYPE_MISMATCH:") {
            return "E_CURRENT_ITEM_TYPE_MISMATCH";
        }
        if self.message.starts_with("E_CURRENT_ITEM_OWNERSHIP_UNSAFE:") {
            return "E_CURRENT_ITEM_OWNERSHIP_UNSAFE";
        }
        if self.message.starts_with("E_CALL_REQUIRED_PIN_MISSING:") {
            return "E_CALL_REQUIRED_PIN_MISSING";
        }
        if self
            .message
            .starts_with("E_FIELD_POSSESSIVE_SURFACE_FORBIDDEN:")
        {
            return "E_FIELD_POSSESSIVE_SURFACE_FORBIDDEN";
        }
        if self.message.starts_with("E_FIELD_MISSING:") {
            return "E_FIELD_MISSING";
        }
        if self.message.starts_with("E_FIELD_TARGET_TYPE:") {
            return "E_FIELD_TARGET_TYPE";
        }
        if self
            .message
            .starts_with("E_NATURAL_FOREACH_BINDER_NAME_REQUIRED:")
        {
            return "E_NATURAL_FOREACH_BINDER_NAME_REQUIRED";
        }
        if self
            .message
            .starts_with("E_NATURAL_FOREACH_COLON_FORBIDDEN:")
        {
            return "E_NATURAL_FOREACH_COLON_FORBIDDEN";
        }
        if self.message.contains("조사 '")
            && self.message.contains("모호합니다")
            && self.message.contains("값:핀")
        {
            return "E_PARSE_CALL_JOSA_AMBIGUOUS";
        }
        if self.message.contains("핀 '") && self.message.contains("인자가 중복되었습니다")
        {
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

#[cfg(test)]
mod wo09_signed_min_tests {
    use super::*;
    use crate::lexer::Lexer;

    fn expression(source: &str) -> Expr {
        let mut parser = Parser::new(Lexer::new(source).tokenize().unwrap());
        let result = parser.parse_unary().unwrap();
        assert!(matches!(parser.current().kind, TokenKind::Eof));
        result
    }

    #[test]
    fn wo09_signed_min_keeps_ordinary_negative_ast_and_primary_tail() {
        let ordinary = expression("-0.5");
        let ExprKind::Infix { left, op, right } = ordinary.kind else { panic!("ordinary negative AST changed") };
        assert_eq!(op, "-");
        assert!(matches!(left.kind, ExprKind::Literal(Literal::Fixed64(v)) if v.raw_i64() == 0));
        assert!(matches!(right.kind, ExprKind::Literal(Literal::Fixed64(v)) if v.raw_i64() == 1i64 << 31));
        for source in ["-2147483648", "(-2147483648)"] {
            assert!(matches!(expression(source).kind, ExprKind::Literal(Literal::Fixed64(v)) if v.raw_i64() == i64::MIN));
        }
        let ExprKind::Suffix { value, at } = expression("-2147483648@m").kind else { panic!("unit suffix lost") };
        assert!(matches!(value.kind, ExprKind::Literal(Literal::Fixed64(v)) if v.raw_i64() == i64::MIN));
        assert!(matches!(at, AtSuffix::Unit(ref unit) if unit == "m"));
        assert!(matches!(expression("-2147483648 아님").kind, ExprKind::Call { ref func, .. } if func == "아님"));
        let ExprKind::Infix { left, op, right } = expression("- -2147483648").kind else { panic!("outer negation lost") };
        assert_eq!(op, "-");
        assert!(matches!(left.kind, ExprKind::Literal(Literal::Fixed64(v)) if v.raw_i64() == 0));
        assert!(matches!(right.kind, ExprKind::Literal(Literal::Fixed64(v)) if v.raw_i64() == i64::MIN));
    }
}
