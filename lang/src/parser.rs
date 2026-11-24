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
    piecewise_definitions: HashMap<NodeId, PiecewiseDefinition>,
    tokens: Vec<Token>,
    source_epoch: SourceEpoch,
    type_reference_sites: Option<Vec<(String,Span,bool)>>,
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
    authored_call_candidates: HashMap<NodeId, Vec<NodeId>>,
    authored_call_name_spans: HashMap<NodeId,Span>,
    authored_argument_order: HashMap<NodeId,Vec<NodeId>>,
    authored_pin_label_spans: HashMap<NodeId,Span>,
    static_structure_calls: HashMap<NodeId, StaticStructureBinding>,
    static_algebra_calls: HashMap<NodeId, StaticAlgebraBinding>,
    registered_call_spans: HashMap<NodeId, Span>,
    registered_definitions: HashMap<NodeId, SeedDef>,
    rd714_niladic_calls: HashMap<NodeId, String>,
    explicitly_registered_owners: HashSet<String>,
    product_definitions: HashMap<String, (NodeId, Vec<ParamPin>)>,
    schema_categories: HashMap<String, SchemaCategory>,
    schema_header_names: HashSet<String>,
    schema_construction_nodes: HashSet<NodeId>,
    sum_definitions: HashMap<String, Vec<SumBranchDecl>>,
    bytes_calls: HashMap<NodeId, crate::bytes::BytesConversion>,
    vc17_named_calls: HashMap<NodeId, crate::stdlib::vc17::Definition>,
    atom_value_occurrences: HashMap<NodeId, AtomValueOccurrence>,
    retirement_commands: HashMap<NodeId,RetirementCommandBinding>,
    subject_reference_calls: HashMap<NodeId,SubjectReferenceCallBinding>,
    manipulation_phase_calls: HashMap<NodeId,ManipulationPhaseCallBinding>,
    mode: ParseMode,
    public_callable_context: Option<std::sync::Arc<RegisteredPublicCallableContext>>,
    public_callable_module: bool,
    first_callable_ddl: bool,
    first_callable_context: Option<std::sync::Arc<RegisteredFirstCallableContext>>,
}

/// Immutable compilation context; neither a source-profile version nor a
/// persisted semantic identity. The original public source and exact DDL stay
/// owned here, so callers cannot attach a binding from a different analysis.
#[derive(Debug, Clone)]
pub struct RegisteredFirstCallableContext {
    pub(crate) original: crate::frontdoor::ModuleDirectiveFrontdoorResult,
    pub(crate) projection: SeedDef,
    pub(crate) ddl_source: String,
    pub(crate) ddl_path: String,
    pub(crate) activation:Option<FirstCallableModuleActivation>,
    pub(crate) prefix_profile:Option<crate::ddl::CheckedFirstCallableProfile>,
}
#[derive(Debug,Clone)]
pub(crate) struct FirstCallableModuleActivation {
    pub profile:crate::ddl::CheckedFirstCallableProfile,
    pub binding:crate::module_directive::ModuleDirectiveEntry,
}

impl RegisteredFirstCallableContext {
    /// Checked lexical supply for module-request inspection only. This does
    /// not satisfy activation; parsing a consumer still requires `옮김`.
    pub fn with_prefix_profile(mut self,profile:crate::ddl::CheckedFirstCallableProfile)->Self{self.prefix_profile=Some(profile);self}

    pub fn from_public_and_ddl(
        original: crate::frontdoor::ModuleDirectiveFrontdoorResult,
        ddl_source: &str, ddl_path: &str,
        language_profile_identity: &str, registration_revision: u32,
    ) -> Result<Self, ParseError> {
        let fail = |span, message: &str| ParseError { span, message: message.into() };
        if !(ddl_path.ends_with(".ddl") || ddl_path.ends_with(".옮김")) {
            return Err(fail(Span::new(0, 0), "E_DDL_EXTENSION_UNSUPPORTED: .ddl 또는 .옮김이 필요합니다"));
        }
        let binding = crate::ddl::bind_first_callable_ja_declaration(
            &original, language_profile_identity, registration_revision,
        ).map_err(|e| ParseError { span: e.span, message: format!("{}: {}", e.code, e.message) })?;
        let tokens = Lexer::with_first_callable_registration(ddl_source, &binding)
            .and_then(|mut lexer| lexer.tokenize())
            .map_err(|e| fail(Span::new(e.pos, e.pos + ddl_source[e.pos..].chars().next().map_or(0, char::len_utf8)), &e.message))?;
        let mut parser = Parser::new_v25_with_mode(tokens, ParseMode::Strict);
        parser.source = ddl_source.into();
        parser.first_callable_ddl = true;
        let projection = parser.parse_seed_def()?;
        if !parser.is_at_end() {
            return Err(parser.error("E_DDL_FIRST_EXTRA_DECLARATION: 첫 범위의 공개 함수 하나만 필요합니다"));
        }
        let registry = crate::ddl::FIRST_CALLABLE_JA_REGISTRATION;
        if projection.canonical_name != registry.callable_name || !projection.alternate_forms.is_empty()
            || projection.seed_kind != SeedKind::Semssi || projection.body.is_some()
            || projection.result_type != Some(TypeRef::Named(registry.rational_type_name.into()))
            || projection.params.len() != 2
        {
            return Err(fail(projection.span, "E_DDL_FIRST_PROJECTION_MISMATCH: 실제 원본의 전체 공개 선언과 일치해야 합니다"));
        }
        for (index, pin) in projection.params.iter().enumerate() {
            if pin.pin_name != registry.pin_names[index] || pin.optional || pin.default_value.is_some()
                || pin.type_ref != TypeRef::Named(registry.rational_type_name.into())
                || pin.josa_list.as_slice() != [registry.pin_particles[index]]
            {
                return Err(fail(pin.span, "E_DDL_FIRST_PIN_PROJECTION_MISMATCH: 필수 핀·타입·역할을 모두 투영해야 합니다"));
            }
        }
        Ok(Self { original, projection, ddl_source: ddl_source.into(), ddl_path: ddl_path.into(),activation:None,prefix_profile:None })
    }

    /// Compilation-only handoff. Tool's original module/G1 owner remains
    /// responsible for exported-DDL and durable admission; no serde success.
    pub fn with_module_activation(mut self,profile:crate::ddl::CheckedFirstCallableProfile,
        binding:crate::module_directive::ModuleDirectiveEntry)->Result<Self,ParseError>{
        if binding.local_name.is_empty()||binding.target.is_empty(){return Err(ParseError{span:Span::new(0,0),message:"E_DDL_ACTIVATION_BINDING_REQUIRED".into()});}
        self.activation=Some(FirstCallableModuleActivation{profile,binding});Ok(self)
    }
    pub fn original_public(&self) -> &crate::frontdoor::ModuleDirectiveFrontdoorResult { &self.original }
    pub fn ddl_source(&self) -> &str { &self.ddl_source }
    pub fn ddl_path(&self) -> &str { &self.ddl_path }
    pub fn projection(&self) -> &SeedDef { &self.projection }
}
/// Compilation-only whole-public correspondence. Durable identities and
/// actual export admission remain with the tool's checked source owner.
#[derive(Debug,Clone)]
pub struct PublicCallableBinding {
    pub definition:String,pub pin_identities:Vec<String>,pub original:SeedDef,
    pub source_path:String,pub source_text:String,
}
#[derive(Debug,Clone)]
pub struct RegisteredPublicCallableContext {
    pub(crate) profile:crate::ddl::RegisteredSuccessorProfile,
    pub(crate) bindings:Vec<PublicCallableBinding>,
    pub(crate) projections:Vec<SeedDef>,pub(crate) ddl_source:String,pub(crate) ddl_path:String,
    pub(crate) activation:Option<crate::module_directive::ModuleDirectiveEntry>,
    pub(crate) next_node:u64,
    pub(crate) consumer_j1:Option<crate::ddl::CheckedJ1Profile>,
}
impl RegisteredPublicCallableContext {
    pub fn from_public_and_ddl(profile:crate::ddl::RegisteredSuccessorProfile,
        bindings:Vec<PublicCallableBinding>,ddl_source:&str,ddl_path:&str,next_node:u64,
    )->Result<Self,ParseError>{
        let fail=|message:&str|ParseError{span:Span::new(0,ddl_source.len()),message:message.into()};
        if profile.is_logic_builtin(){return Err(fail("E_DDL_PUBLIC_PROFILE_KIND"));}
        if !(ddl_path.ends_with(".ddl")||ddl_path.ends_with(".옮김"))||ddl_source.len()>16384||bindings.is_empty()
            ||bindings.len()!=profile.surfaces().len(){return Err(fail("E_DDL_PUBLIC_COVERAGE"));}
        let tokens=Lexer::with_public_callable_projection(ddl_source,&profile,true)
            .and_then(|mut lexer|lexer.tokenize()).map_err(|e|ParseError{span:Span::new(e.pos,e.pos+ddl_source.get(e.pos..).and_then(|s|s.chars().next()).map_or(0,char::len_utf8)),message:e.message})?;
        let mut parser=Parser::new_v25_with_mode(tokens,ParseMode::Strict);parser.source=ddl_source.into();parser.first_callable_ddl=true;
        let mut projections=Vec::new();
        while !parser.is_at_end(){projections.push(parser.parse_seed_def()?);if projections.len()>128{return Err(fail("E_DDL_PUBLIC_LIMIT"));}}
        if projections.len()!=bindings.len(){return Err(fail("E_DDL_PUBLIC_COVERAGE"));}
        let mut ids=HashSet::new();let mut originals=HashSet::new();
        for row in profile.surfaces(){
            let matched=bindings.iter().filter(|binding|binding.definition==row.identity).collect::<Vec<_>>();
            if matched.len()!=1||!ids.insert(row.identity.clone()){return Err(fail("E_DDL_PUBLIC_DEFINITION_OWNER"));}
            let binding=matched[0];let original=&binding.original;
            if !originals.insert(original.canonical_name.clone())||original.seed_kind!=SeedKind::Semssi||original.body.is_none()||!original.type_params.is_empty()||original.proof_header.is_some()||!original.modifiers.is_empty()
                ||original.params.len()!=row.pins.len()||binding.pin_identities.len()!=row.pins.len()
                ||original.result_type!=Some(TypeRef::Named("나눔수".into())){return Err(fail("E_DDL_PUBLIC_ORIGINAL_REQUIRED"));}
            let found=projections.iter().filter(|projection|projection.canonical_name==row.surface).collect::<Vec<_>>();
            if found.len()!=1{return Err(fail("E_DDL_PUBLIC_COVERAGE"));}let projection=found[0];
            if projection.seed_kind!=SeedKind::Semssi||projection.body.is_some()||!projection.alternate_forms.is_empty()||!projection.type_params.is_empty()||projection.proof_header.is_some()||!projection.modifiers.is_empty()
                ||projection.params.len()!=row.pins.len()||projection.result_type!=Some(TypeRef::Named("有理数".into())){return Err(fail("E_DDL_PUBLIC_PROJECTION"));}
            for (index,pin) in row.pins.iter().enumerate(){
                let original_pin=&original.params[index];let projected=&projection.params[index];
                if binding.pin_identities[index]!=pin.identity||original_pin.optional||original_pin.default_value.is_some()
                    ||original_pin.type_ref!=TypeRef::Named("나눔수".into())||projected.pin_name!=pin.surface
                    ||projected.type_ref!=TypeRef::Named("有理数".into())||projected.optional||projected.default_value.is_some()
                    ||projected.josa_list.as_slice()!=[pin.particle.as_str()]{return Err(ParseError{span:projected.span,message:"E_DDL_PUBLIC_PIN_OWNER".into()});}
            }
        }
        Ok(Self{profile,bindings,projections,ddl_source:ddl_source.into(),ddl_path:ddl_path.into(),activation:None,consumer_j1:None,next_node})
    }
    pub fn profile(&self)->&crate::ddl::RegisteredSuccessorProfile{&self.profile}
    pub fn ddl_source(&self)->&str{&self.ddl_source}
    pub fn ddl_path(&self)->&str{&self.ddl_path}
    pub fn bindings(&self)->&[PublicCallableBinding]{&self.bindings}
    /// Selected by the verified consumer artifact, never by a header alone.
    pub fn with_consumer_j1(mut self,profile:Option<crate::ddl::CheckedJ1Profile>)->Self {
        self.consumer_j1=profile;self
    }
    pub fn consumer_j1(&self)->Option<crate::ddl::CheckedJ1Profile>{self.consumer_j1}
    pub fn with_module_activation(mut self,binding:crate::module_directive::ModuleDirectiveEntry)->Result<Self,ParseError>{
        if binding.local_name.is_empty()||binding.target.is_empty(){return Err(ParseError{span:Span::new(0,0),message:"E_DDL_ACTIVATION_BINDING_REQUIRED".into()});}
        self.activation=Some(binding);Ok(self)
    }
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
    /// Compiler-local syntax provenance for a known AST type prefix. This
    /// invokes the same grammar without adding a type resolver or public wire.
    /// A separate parser owns these sites, so speculative program lookahead
    /// cannot leave ghost type occurrences in a completed module.
    pub fn parse_type_reference_prefix(source:&str,tokens:Vec<Token>)
        ->Result<(TypeRef,Vec<(String,Span,bool)>),ParseError> {
        let mut parser=Self::new_v25_with_mode(tokens,ParseMode::Strict);
        parser.source=source.into();parser.type_reference_sites=Some(Vec::new());
        let ty=parser.parse_type_ref()?;
        Ok((ty,parser.type_reference_sites.take().unwrap_or_default()))
    }
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
            type_reference_sites:None,
            piecewise_definitions: HashMap::new(),
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
            authored_call_candidates: HashMap::new(),
            authored_call_name_spans: HashMap::new(),
            authored_argument_order: HashMap::new(),
            authored_pin_label_spans: HashMap::new(),
                static_structure_calls: HashMap::new(),
                static_algebra_calls: HashMap::new(),
            registered_call_spans: HashMap::new(),
            registered_definitions: HashMap::new(),
            rd714_niladic_calls: HashMap::new(),
            explicitly_registered_owners: HashSet::new(),
            product_definitions: HashMap::new(),
            schema_categories: HashMap::new(),
            schema_header_names: HashSet::new(),
            schema_construction_nodes: HashSet::new(),
            sum_definitions: HashMap::new(),
            bytes_calls: HashMap::new(),
            vc17_named_calls: HashMap::new(),
            atom_value_occurrences: HashMap::new(),
            retirement_commands: HashMap::new(),
            subject_reference_calls: HashMap::new(),
            manipulation_phase_calls: HashMap::new(),
            mode,
            public_callable_context: None,
            public_callable_module: false,
            first_callable_ddl: false,
            first_callable_context: None,
        }
    }

    /// Reparse retained original tails through the same V25 parser, carrying
    /// one compilation-node counter across files; no source concatenation or
    /// path-derived semantic identity is used.
    pub fn parse_public_original_tail(source:&str,path:&str,next_node:u64)->Result<(CanonProgram,u64),ParseError>{
        let tokens=Lexer::new(source).tokenize().map_err(|e|ParseError{span:Span::new(e.pos,e.pos+1),message:e.message})?;
        let mut parser=Self::new_v25_with_mode(tokens,ParseMode::Strict);parser.nid=next_node;
        let program=parser.parse_program(source.into(),path.into())?;Ok((program,parser.nid))
    }
    pub fn parse_with_public_callable_context(source:&str,path:&str,context:std::sync::Arc<RegisteredPublicCallableContext>)->Result<CanonProgram,ParseError>{
        Self::parse_public_callable_consumer(source,path,context,false)
    }

    /// Internal building block for a linked J1 module; this does not select
    /// an execution entry or accept a storage carrier.
    pub fn parse_with_public_callable_module_context(
        source:&str,path:&str,context:std::sync::Arc<RegisteredPublicCallableContext>,
    )->Result<CanonProgram,ParseError>{
        if context.consumer_j1().is_none(){
            return Err(ParseError{span:Span::new(0,0),message:"E_J1_PROFILE_REQUIRED".into()});
        }
        Self::parse_public_callable_consumer(source,path,context,true)
    }

    /// Collect only local, uniquely declared targets before validating a module export.
    pub fn j1_local_export_targets(
        source: &str,
        path: &str,
        syntax: crate::ddl::CheckedJ1Profile,
    ) -> Result<std::collections::BTreeSet<String>, ParseError> {
        let program = Self::parse_unlinked_j1_module(source, path, syntax)?;
        Ok(Self::local_export_targets(&program))
    }

    fn local_export_targets(program: &CanonProgram) -> std::collections::BTreeSet<String> {
        let mut counts = HashMap::new();
        for TopLevelItem::SeedDef(seed) in &program.items {
            let (count, has_body) = counts.entry(seed.canonical_name.as_str()).or_insert((0, false));
            *count += 1;
            *has_body |= seed.body.is_some();
        }
        counts.into_iter()
            .filter(|(_, (count, has_body))| *count == 1 && *has_body)
            .map(|(name, _)| name.to_owned())
            .collect()
    }

    pub fn public_callable_module_prefix(
        source: &str,
        path: &str,
        context: &RegisteredPublicCallableContext,
    ) -> Result<crate::ModuleDirectiveDocument, ParseError> {
        Self::public_callable_consumer_tokens(source, path, context, true)
            .map(|(document, _)| document)
    }

    fn public_callable_consumer_tokens(
        source: &str,
        path: &str,
        context: &RegisteredPublicCallableContext,
        module: bool,
    ) -> Result<(crate::ModuleDirectiveDocument, Vec<Token>), ParseError> {
        let activation=context.activation.as_ref().ok_or_else(||ParseError{span:Span::new(0,0),message:"E_DDL_ACTIVATION_REQUIRED".into()})?;
        let mut directive_context = crate::ModuleDirectiveContext::default()
            .with_known_imports([activation.target.clone()]);
        if module {
            let syntax = context.consumer_j1().ok_or_else(|| ParseError {
                span: Span::new(0, 0), message: "E_J1_PROFILE_REQUIRED".into(),
            })?;
            directive_context = directive_context.with_public_definitions(
                Self::j1_local_export_targets(source, path, syntax)?,
            );
        }
        let (document,tokens)=crate::module_directive::public_callable_module_tokens(source,path,&directive_context,&context)?;
        let imports=document.imports.as_ref().ok_or_else(||ParseError{span:Span::new(0,0),message:"E_DDL_ACTIVATION_REQUIRED".into()})?;
        let request=imports.translation.as_ref().ok_or_else(||ParseError{span:imports.span,message:"E_DDL_ACTIVATION_REQUIRED".into()})?;
        if imports.entries.as_slice()!=std::slice::from_ref(activation)||(!module && document.exports.is_some())
            ||request.target_language!=context.profile.canonical_name()||request.target_version!=context.profile.version(){return Err(ParseError{span:imports.span,message:"E_DDL_ACTIVATION_OWNER_OR_PROFILE".into()});}
        Ok((document, tokens))
    }

    fn parse_public_callable_consumer(
        source:&str,path:&str,context:std::sync::Arc<RegisteredPublicCallableContext>,module:bool,
    )->Result<CanonProgram,ParseError>{
        let (document, tokens) = Self::public_callable_consumer_tokens(source, path, &context, module)?;
        let start=source.len()-document.declaration_source.len();
        let tokens=tokens.into_iter().filter(|token|token.span.start>=start).collect::<Vec<_>>();
        let syntax=context.consumer_j1();
        let shows=tokens.iter().filter(|t|matches!(t.kind,TokenKind::KwBoyeojugi)&&t.raw=="表示")
            .map(|t|t.span.start).collect::<Vec<_>>();
        let mut parser=Self::new_v25_with_mode(tokens,ParseMode::Strict);
        parser.nid=context.next_node;parser.public_callable_context=Some(context);
        parser.public_callable_module=module;
        let mut program=parser.parse_program(source.into(),path.into())?;
        if let Some(syntax)=syntax {
            crate::canonicalizer::resolve_j1_type_roles(&mut program,syntax)?;
            resolve_j1_expression_roles(&mut program,&shows)?;
        }
        Ok(program)
    }

    pub fn parse_with_first_callable_context(
        source: &str, path: &str, context: std::sync::Arc<RegisteredFirstCallableContext>,
    ) -> Result<CanonProgram, ParseError> {
        let binding = crate::ddl::bind_first_callable_ja_declaration(
            &context.original, crate::ddl::FIRST_CALLABLE_JA_REGISTRATION.language_profile_identity,
            crate::ddl::FIRST_CALLABLE_JA_REGISTRATION.registration_revision,
        ).map_err(|e| ParseError { span: e.span, message: format!("{}: {}", e.code, e.message) })?;
        let tokens=if let Some(activation)=&context.activation {
            let directive_context=crate::module_directive::ModuleDirectiveContext::default().with_known_imports([activation.binding.target.clone()]);
            let (document,tokens)=crate::module_directive::first_callable_module_tokens(source,path,&directive_context,&context)?;
            let imports=document.imports.as_ref().ok_or_else(||ParseError{span:Span::new(0,0),message:"E_DDL_ACTIVATION_REQUIRED".into()})?;
            let request=imports.translation.as_ref().ok_or_else(||ParseError{span:imports.span,message:"E_DDL_ACTIVATION_REQUIRED".into()})?;
            let profile=crate::ddl::resolve_first_callable_profile_name(&request.target_language,&request.target_version)
                .map_err(|e|ParseError{span:request.span,message:e.to_string()})?;
            if imports.entries.as_slice()!=[activation.binding.clone()] || document.exports.is_some()
                ||profile.identity()!=activation.profile.identity()||profile.version()!=activation.profile.version(){
                return Err(ParseError{span:imports.span,message:"E_DDL_ACTIVATION_OWNER_OR_PROFILE: 명시된 실제 모듈·말씨 판본과 다릅니다".into()});
            }
            let start=source.len()-document.declaration_source.len();
            tokens.into_iter().filter(|token|token.span.start>=start).collect()
        } else {
            Lexer::with_first_callable_registration(source, &binding).and_then(|mut lexer| lexer.tokenize())
                .map_err(|e| ParseError { span: Span::new(e.pos, e.pos + source[e.pos..].chars().next().map_or(0, char::len_utf8)), message: e.message })?
        };
        let mut parser = Self::new_v25_with_mode(tokens, ParseMode::Strict);
        parser.nid = context.original.declaration_program.as_ref().unwrap().id.checked_add(1)
            .ok_or_else(|| ParseError { span: Span::new(0, 0), message: "E_DDL_NODE_LIMIT".into() })?;
        parser.first_callable_context = Some(context);
        parser.parse_program(source.into(), path.into())
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
    /// Preserve the exact token span supplied by parse_call_name separately
    /// from the whole call/argument syntax. Neither span selects a target.
    fn parsed_source_call(&mut self,span:Span,args:Vec<ArgBinding>,func:String,name_span:Span)->Expr {
        let id=self.next_id();self.authored_call_name_spans.insert(id,name_span);
        self.authored_argument_order.insert(id,args.iter().map(|arg|arg.id).collect());
        Expr::new(id,span,ExprKind::Call{args,func})
    }

    pub fn parse_program(
        &mut self,
        source: String,
        file_path: String,
    ) -> Result<CanonProgram, ParseError> {
        self.parse_program_phase(source, file_path, true, &[])
    }

    /// Declaration discovery only. Imported call targets are resolved after
    /// linking; this result must never be used as an executable program.
    pub fn parse_unlinked_module(
        &mut self,
        source: String,
        file_path: String,
    ) -> Result<CanonProgram, ParseError> {
        self.parse_program_phase(source, file_path, false, &[])
    }

    /// Syntax-only headers from the same already selected module closure.
    /// This does not resolve or admit a constructor; the linked program must
    /// still contain its actual declaration and pass ordinary vc04 validation.
    pub fn parse_unlinked_module_with_schema_headers(&mut self,source:String,file_path:String,headers:&[String])
        ->Result<CanonProgram,ParseError>{self.parse_program_phase(source,file_path,false,headers)}
    pub fn discover_schema_header_names(source:&str)->Result<Vec<String>,ParseError>{
        let(mut lexer,_)=Lexer::with_korean_source_header(source).map_err(|e|ParseError{span:Span::new(e.pos,e.pos+1),message:e.message})?;
        let tokens=lexer.tokenize().map_err(|e|ParseError{span:Span::new(e.pos,e.pos+1),message:e.message})?;
        let mut parser=Self::new_v25_with_mode(tokens,ParseMode::Strict);parser.source=source.into();
        let mut names=parser.scan_schema_header_names().into_iter().collect::<Vec<_>>();names.sort();Ok(names)
    }
    fn scan_schema_header_names(&self)->HashSet<String>{
        let mut names=HashSet::new();let mut depth=0usize;
        if self.source_epoch==SourceEpoch::V25{for index in 0..self.tokens.len(){
            if depth==0&&self.is_vc04_schema_header_at(index){names.insert(self.tokens[index].raw.clone());}
            match self.tokens[index].kind{TokenKind::LBrace=>depth+=1,TokenKind::RBrace=>depth=depth.saturating_sub(1),_=>{}}
        }}names
    }
    fn parse_program_phase(
        &mut self,
        source: String,
        file_path: String,
        resolve_calls: bool,
        module_schema_headers:&[String],
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
        self.schema_header_names=self.scan_schema_header_names();
        if !resolve_calls{self.schema_header_names.extend(module_schema_headers.iter().cloned());}
        self.sum_definitions.clear();
        self.bytes_calls.clear();
        self.vc17_named_calls.clear();
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
        self.authored_call_candidates.clear();
        self.authored_call_name_spans.clear();
        self.authored_argument_order.clear();
        self.authored_pin_label_spans.clear();
        self.piecewise_definitions.clear();
        self.static_structure_calls.clear();
        self.static_algebra_calls.clear();
        self.registered_call_spans.clear();
        self.registered_definitions.clear();
        self.rd714_niladic_calls.clear();
        self.explicitly_registered_owners.clear();
        let mut items = Vec::new();
        let mut root_stmts = Vec::new();
        let mut top_level_decl = Vec::new();
        let mut authored_jjaim_blocks = Vec::new();
        let mut checked_structure_source = None;
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
            if self.source_epoch == SourceEpoch::V25 && self.is_profiled_structure_start() {
                if self.pos != 0 {
                    return Err(ParseError {
                        span: self.current_span(),
                        message: "E_STRUCTURE_SYNTAX: 등록 짜임은 한 원문 전체를 차지해야 합니다".into(),
                    });
                }
                let descriptor = self.parse_profiled_structure()?;
                let structure = crate::structure_grid::resolve(&source, &file_path)
                    .ok_or_else(|| ParseError {
                        span: descriptor.source_group_span.unwrap_or(descriptor.span),
                        message: "E_STRUCTURE_SYNTAX: 등록된 차림새 이름을 직접 선택해야 합니다".into(),
                    })??;
                checked_structure_source = Some(structure.canonical_source);
                for (span, text) in structure.output {
                    let expr = Expr {
                        id: self.next_id(),
                        span,
                        kind: ExprKind::Literal(Literal::String(text)),
                        numeric_literal_source: None,
                        source_unary_minus: false,
                        source_group_span: None,
                    };
                    root_stmts.push(Stmt::Show {
                        id: self.next_id(),
                        span,
                        mood: Mood::Imperative,
                        expr,
                    });
                }
                break;
            }
            if matches!(self.current().kind, TokenKind::JjaimBlock(_)) {
                let stmt = self.parse_jjaim_block_stmt()?;
                if let Stmt::MetaBlock { span, .. } = stmt {
                    authored_jjaim_blocks.push(span);
                }
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
            if self.source_epoch == SourceEpoch::V25
                && self.find_matching_rparen_after_current_lparen().is_some_and(|end|
                    self.tokens.get(end + 1).is_some_and(|token| token.raw == "두고")) {
                items.push(self.parse_top_level_item()?);
            } else if self.is_vc04_schema_start() {
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
                registered_public_sites: Vec::new(),
                lexical_names: None,
                piecewise_definitions: self.piecewise_definitions.clone(),
                file_path,
                source,
                authored_jjaim_blocks,
                checked_structure_source,
                node_spans: std::collections::HashMap::new(),
                registered_sum_pin_order:None,
                registered_call_candidates: HashMap::new(),
                authored_call_candidates: HashMap::new(),
            authored_call_name_spans: HashMap::new(),
            authored_argument_order: HashMap::new(),
            authored_pin_label_spans: HashMap::new(),
                static_structure_calls: HashMap::new(),
                static_algebra_calls: HashMap::new(),
                bytes_calls: HashMap::new(),
            vc17_named_calls: HashMap::new(),
                atom_value_occurrences: self.atom_value_occurrences.clone(),
                logic_callable_calls:HashMap::new(),
                retirement_commands: HashMap::new(),
                subject_reference_calls: HashMap::new(),
                manipulation_phase_calls: HashMap::new(),
            },
        };
        if let Some(context) = &self.first_callable_context {
            crate::canonicalizer::bind_first_callable_registered_program(&mut program, context)?;
        }
        program.origin.authored_call_name_spans = self.authored_call_name_spans.clone();
        program.origin.authored_pin_label_spans = self.authored_pin_label_spans.clone();
        if let Some(context)=&self.public_callable_context {
            crate::canonicalizer::bind_public_callable_registered_program(&mut program,context,self.public_callable_module)?;
        }
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
        program.origin.authored_call_candidates = self.authored_call_candidates.clone();
        program.origin.authored_argument_order = self.authored_argument_order.clone();
        program.origin.static_structure_calls = self.static_structure_calls.clone();
        program.origin.static_algebra_calls = self.static_algebra_calls.clone();
        program.origin.bytes_calls = self.bytes_calls.clone();
        program.origin.vc17_named_calls = self.vc17_named_calls.clone();
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
        if resolve_calls && program.source_epoch == SourceEpoch::V25 {
            let names=crate::name_resolution::resolve(&program,&self.tokens);
            if let Some((span,message))=names.first_admission_error() {
                return Err(ParseError { span,message:message.into() });
            }
            program.origin.lexical_names=Some(names);
        }
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
            type_params: Vec::new(),
            proof_header: None,
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
        // RD-542 S08: consume only these exact source tokens. Keep the
        // authored phrase in the AST/canon and the full UTF-8 call-name span.
        if self.source_epoch == SourceEpoch::V25 {
            let at = self.pos;
            let count = if self.tokens.get(at).is_some_and(|t|t.raw=="숫자로")
                && self.tokens.get(at+1).is_some_and(|t|t.raw=="바꾸기") {Some(2)}
            else if self.tokens.get(at).is_some_and(|t|t.raw=="숫자")
                && self.tokens.get(at+1).is_some_and(|t|t.raw=="로"
                    && self.tokens[at].span.end==t.span.start)
                && self.tokens.get(at+2).is_some_and(|t|t.raw=="바꾸기") {Some(3)}
            else {None};
            if let Some(count)=count {
                let start=self.to_ast_span(self.tokens[at].span);
                let end=self.to_ast_span(self.tokens[at+count-1].span);
                self.pos+=count;
                return Ok(Some((crate::stdlib::vc17::SUJARO_CALL_TAIL.into(),start.merge(&end))));
            }
        }
        // A particle following a grouped value belongs to the argument,
        // not to a callable-name segment.
        if matches!(self.current().kind, TokenKind::Josa(_)) {
            return Ok(None);
        }
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
        ExprKind::FunctionBlock{statements,result}=>{let mut nested=env.clone();validate_field_body(statements,&mut nested)?;validate_field_expr(result,&mut nested)},
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

pub const STRUCTURE_PROFILE_REGISTRY_UNAVAILABLE: &str = "E_STRUCTURE_PROFILE_REGISTRY_UNAVAILABLE";

impl ParseError {
    /// Additional source-pack diagnostics are selected only by the J1 producer;
    /// the existing code() projection stays unchanged for every old profile.
    pub fn j1_source_code(&self)->&'static str{
        for code in ["E_MULTILANG_FULLWIDTH_SYNTAX","E_MULTILANG_UNREGISTERED_SURFACE",
            "E_J1_HEADER_PROFILE_MISMATCH","E_J1_PROFILE_UNSUPPORTED","E_J1_ROLE_AMBIGUOUS",
            "E_J1_UNREGISTERED_SURFACE","E_J1_DUPLICATE_HEADER","E_FIXED64_LITERAL_RANGE",
            // Tags emitted by the same checked J1/B1 source producer. Consumers
            // use this error directly, never reverse-parse a formatted report.
            "E_DDL_ACTIVATION_REQUIRED","E_DDL_ACTIVATION_OWNER_OR_PROFILE","E_DDL_PUBLIC_PIN_OWNER",
            "E_LOGIC_SOURCE_LIMIT","E_LOGIC_SOURCE_CALL_NAME_REQUIRED","E_LOGIC_ALIAS_SHADOWED",
            "E_LOGIC_CALL_UNREGISTERED","E_LOGIC_CALL_ARITY","E_LOGIC_CALL_EXPLICIT_PIN_REQUIRED",
            "E_LOGIC_CALL_PIN_OWNER","E_LOGIC_QUALIFIED_CALL_REQUIRED"]{
            if self.message==code || self.message.starts_with(&format!("{code}:")){return code;}
        }
        self.code()
    }
    pub fn code(&self) -> &'static str {
        if let Some(code) = crate::structure_grid::diagnostic_code(&self.message) {
            return code;
        }

        // Preserve producer-owned machine codes before presentation. UTF-8 spans
        // remain the actual ParseError span; no rendered-message position parsing.
        for code in [
            STRUCTURE_PROFILE_REGISTRY_UNAVAILABLE,
            "E_PROOF_ARGUMENT_BINDING",
            "E_PROOF_ARGUMENT_COUNT",
            "E_PROOF_ASSUMPTION",
            "E_PROOF_BINDER_DUPLICATE",
            "E_PROOF_BUDGET",
            "E_PROOF_CALL_AMBIGUOUS",
            "E_PROOF_CALL_SHADOWED",
            "E_PROOF_CALL_UNRESOLVED",
            "E_PROOF_CALL_UNSUPPORTED",
            "E_PROOF_CONTEXT_MISMATCH",
            "E_PROOF_COORDINATE_ADMISSION",
            "E_PROOF_COORDINATE_ASSUMPTIONS",
            "E_PROOF_COORDINATE_DEFINITION",
            "E_PROOF_COORDINATE_LITERAL",
            "E_PROOF_COORDINATE_TYPE",
            "E_PROOF_COORDINATE_UNSUPPORTED",
            "E_PROOF_DEFINITION",
            "E_PROOF_EVIDENCE",
            "E_PROOF_GOAL",
            "E_PROOF_IDENTITY_DEFINITION",
            "E_PROOF_INFERENCE_UNRESOLVED",
            "E_PROOF_NUMERIC_CONTEXT",
            "E_PROOF_NUMERIC_CONTRACT",
            "E_PROOF_NUMERIC_SUPPLY_REQUIRED",
            "E_PROOF_NUMERIC_TYPE",
            "E_PROOF_PROJECTION",
            "E_PROOF_RESULT",
            "E_PROOF_ROOT_UNSUPPORTED",
            "E_PROOF_RULE_UNSUPPORTED",
            "E_PROOF_STALE_SOURCE",
            "E_PROOF_STRUCTURE_ADMISSION",
            "E_PROOF_STRUCTURE_CONSUMPTION",
            "E_PROOF_STRUCTURE_DESCRIPTOR",
            "E_PROOF_STRUCTURE_EXECUTION",
            "E_PROOF_STRUCTURE_MEMBER",
            "E_PROOF_STRUCTURE_POLICY",
            "E_PROOF_STRUCTURE_TYPE",
            "E_PROOF_SUBSTITUTION",
            "E_PROOF_TYPE",
        ] {
            if self.message==code || self.message.strip_prefix(code).is_some_and(|rest|rest.starts_with(':')) {
                return code;
            }
        }

        for code in ["E_VC04_RETIRE_AMBIGUOUS", "E_VC04_RETIRE_BINDING", "E_VC04_RETIRE_PIN", "E_VC04_RETIRE_TARGET", "E_VC04_COMMAND_VALUE", "E_VC04_PURE_WORLD_WRITE", "E_SCHEMA_CONSTRUCTION_SURFACE",
            "E_PRODUCT_DEFINITION_DUPLICATE", "E_PRODUCT_SCHEMA_BODY", "E_PRODUCT_FIELD_SCHEMA", "E_PRODUCT_FIELD_DUPLICATE",
            "E_PRODUCT_NAMED_FIELD_REQUIRED", "E_PRODUCT_FIELD_UNKNOWN", "E_PRODUCT_FIELD_MISSING",
            "E_RUNTIME_TYPE_MISMATCH", "E_BYTES_ARGUMENT_BINDING", "E_BYTES_LITERAL_INVALID",
            "E_TYPE_MISMATCH", "E_CALL_POSITIONAL_ARGUMENT_EXCESS"] {
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

#[cfg(test)]
mod wo24_incomplete_expression_tests {
    #[test]
    fn wo24_incomplete_expression_eof_rejects_without_recursing() {
        for raw in ["x <- (", "값 <- (", "값 := (", "값 <- ((", "값 <- [", "값 <- {"] {
            for suffix in ["", "\n"] {
                let source = format!("{raw}{suffix}");
                let error = crate::parse_v25(&source, "incomplete.ddn").unwrap_err();
                assert_eq!(error.code(), "E_PARSE", "{source:?}: {error:?}");
                assert!(error.span.start <= error.span.end && error.span.end <= source.len());
                assert!(source.is_char_boundary(error.span.start) && source.is_char_boundary(error.span.end));
            }
        }
    }

    #[test]
    fn wo24_existing_incomplete_errors_and_complete_expressions_remain_supported() {
        for source in ["(", "(1", "값 <- (1", "\"가\" 보여주기("] {
            assert_eq!(crate::parse_v25(source, "existing.ddn").unwrap_err().code(), "E_PARSE");
        }
        for source in ["값 <- (1 + 2).", "값 <- ((1 + 2))."] {
            crate::parse_v25(source, "complete.ddn").unwrap();
        }
    }
}

mod statements;

mod elaboration;

mod contracts_literals;

mod expressions;

mod callables;
mod piecewise;
pub(crate) mod logic_callable;
pub use logic_callable::{RegisteredLogicCallableContext,ResolvedLogicCall};

#[cfg(test)]
mod multilang_first_tests {
    use super::*;
    use std::sync::Arc;
    pub const ORIGINAL: &str = "(총액합하다=총액합하다) 드러냄.\n(금액:나눔수~에, 세액:나눔수~을~를) 총액합하다{총액합하기}:나눔수:셈씨 := { 총액합하다 <- 금액 + 세액. }.\n";
    pub const DDL: &str = "#!日本語\n(金額:有理数~に, 税額:有理数~を) 合計する:有理数:関数 :={...}.\n";
    const CALL: &str = "#!日本語\n金額:有理数 := (金額=(100,1) 有理数, 税額=(10,1) 有理数) 合計する.\n";

    fn context(ddl: &str) -> Result<Arc<RegisteredFirstCallableContext>, ParseError> {
        let public = crate::parse_module_directive_frontdoor_with_mode(ORIGINAL, "세금.ddn", ParseMode::Strict,
            &crate::ModuleDirectiveContext::default().with_public_definitions(["총액합하다"])).unwrap();
        RegisteredFirstCallableContext::from_public_and_ddl(public, ddl, "세금.일본어.ddl", "language-profile:ja", 1).map(Arc::new)
    }

    #[test]
    fn multilang_first_common_ddl_parser_preserves_source_and_has_no_executable_projection_body() {
        let context = context(DDL).unwrap();
        assert!(context.projection.body.is_none());
        assert!(context.original.declaration_program.as_ref().unwrap().items.iter().all(|TopLevelItem::SeedDef(seed)| seed.body.is_some()));
        let binding = crate::ddl::bind_first_callable_ja_declaration(&context.original, "language-profile:ja", 1).unwrap();
        let tokens = Lexer::with_first_callable_registration(DDL, &binding).unwrap().tokenize().unwrap();
        for token in tokens.iter().filter(|token| !matches!(token.kind, TokenKind::Eof)) {
            assert_eq!(&DDL[token.span.start..token.span.end], token.raw);
        }
        assert_eq!(context.ddl_source(), DDL);
        assert_eq!(context.projection.params[0].josa_list, ["に"]);
    }

    #[test]
    fn multilang_first_scoped_binding_keeps_local_names_and_original_definition_candidates() {
        let context = context(DDL).unwrap();
        let program = Parser::parse_with_first_callable_context(CALL, "利用.ddn", context.clone()).unwrap();
        assert_eq!(program.origin.source, CALL);
        let TopLevelItem::SeedDef(original) = &context.original.declaration_program.as_ref().unwrap().items[0];
        let TopLevelItem::SeedDef(actual) = &program.items[0];
        assert_eq!(actual.id, original.id);
        assert_eq!(actual.params[0].id, original.params[0].id);
        assert_eq!(actual.params[1].id, original.params[1].id);
        let Stmt::Definition { name, value, type_ref, .. } = &program.root_stmts[0] else { panic!("definition"); };
        assert_eq!(name, "金額"); // a local binder is not the public pin
        assert_eq!(*type_ref, TypeRef::Named("나눔수".into()));
        let ExprKind::Call { func, args } = &value.kind else { panic!("call"); };
        assert_eq!(func, &original.canonical_name);
        assert_eq!(args[0].resolved_pin.as_deref(), Some("금액"));
        assert_eq!(args[1].resolved_pin.as_deref(), Some("세액"));
        assert_eq!(program.origin.authored_call_candidates.get(&value.id), Some(&vec![original.id]));
    }

    #[test]
    fn multilang_first_ddl_refuses_partial_projection_actual_body_and_unsupported_forms() {
        for ddl in [DDL.replace(", 税額:有理数~を", ""), DDL.replace("{...}", "{ 合計する. }"),
            DDL.replace("金額:有理数", "金額:関数"), DDL.replace("合計する:", "合計する{合計する}:"),
            format!("{DDL}{DDL}"), DDL.replace("#!日本語", "#!ja")] {
            assert!(context(&ddl).is_err(), "{ddl}");
        }
    }

    #[test]
    fn multilang_first_refuses_unknown_fullwidth_and_language_fallback_with_utf8_spans() {
        let context = context(DDL).unwrap();
        for source in [CALL.replace("100", "１００"), CALL.replace("合計する", "合計する余"),
            CALL.replace("有理数", "rational"), CALL.replace("税額", "세액"), format!("{CALL}#!日本語\n")] {
            assert!(Parser::parse_with_first_callable_context(&source, "利用.ddn", context.clone()).is_err(), "{source}");
        }
        let source = CALL.replace("\n金額", "\r\n// 😀\r\n余金額");
        let failure = Parser::parse_with_first_callable_context(&source, "利用.ddn", context).unwrap_err();
        assert_eq!(failure.span.start, source.find('余').unwrap());
        assert_eq!(&source[failure.span.start..failure.span.end], "余");
    }

    #[test]
    fn multilang_first_common_required_pin_and_unknown_pin_rejections() {
        let context = context(DDL).unwrap();
        for source in [CALL.replace(", 税額=(10,1) 有理数", ""),
            CALL.replace("税額=(10,1)", "有理数=(10,1)"),
            CALL.replace("税額=(10,1)", "金額=(10,1)")] {
            assert!(Parser::parse_with_first_callable_context(&source, "利用.ddn", context.clone()).is_err(), "{source}");
        }
        assert!(Lexer::new(DDL).tokenize().is_err());
        assert!(crate::parse_v25(CALL, "利用.ddn").is_err());
    }
}

#[cfg(test)]
mod r14_closed_profile_activation_tests {
use super::*;
const ORIGINAL:&str="(총액합하다=총액합하다) 드러냄.\n(금액:나눔수~에, 세액:나눔수~을~를)\n총액합하다{총액합하기}:나눔수:셈씨 := {\n    총액합하다 <- 금액 + 세액.\n}.\n";
const DDL:&str="#!日本語\n(金額:有理数~に, 税額:有理数~を)\n合計する:有理数:関数 :={...}.\n";

 fn context()->std::sync::Arc<RegisteredFirstCallableContext>{
  let public=crate::parse_module_directive_frontdoor_with_mode(ORIGINAL,"세금.ddn",ParseMode::Strict,&crate::ModuleDirectiveContext::default().with_public_definitions(["총액합하다"])).unwrap();
  let base=RegisteredFirstCallableContext::from_public_and_ddl(public,DDL,"세금.ddl","language-profile:ja",1).unwrap();
  std::sync::Arc::new(base.with_module_activation(crate::ddl::issued_first_callable_profile().unwrap(),crate::module_directive::ModuleDirectiveEntry{local_name:"세금".into(),target:"./세금".into()}).unwrap())
 }
 fn source()->String{format!("#!日本語\n(세금=\"./세금\", 옮김=(대상말씨=\"日本語\", 대상판본=\"{}\")) 쓰임.\n金額 <- (金額=(100,1) 有理数, 税額=(10,1) 有理数) 合計する.\n",crate::ddl::issued_first_callable_profile().unwrap().version())}
 #[test]fn r14_real_registry_exact_and_legacy_labels_refuse(){
  let p=crate::ddl::issued_first_callable_profile().unwrap();assert!(p.identity().starts_with("language-profile:"));assert_ne!(p.identity(),"language-profile:ja");
  crate::ddl::lookup_first_callable_profile(p.identity(),p.version(),p.content_sha256(),p.registration_revision()).unwrap();
  for (id,v,h,rev) in [("language-profile:ja",p.version(),p.content_sha256(),1),(p.identity(),"first-callable-ja-r1",p.content_sha256(),1),(p.identity(),p.version(),"0000000000000000000000000000000000000000000000000000000000000000",1),(p.identity(),p.version(),p.content_sha256(),0)]{
   assert!(crate::ddl::lookup_first_callable_profile(id,v,h,rev).is_err());
  }
  for name in ["ja","日本語 ","日本","English"]{assert!(crate::ddl::resolve_first_callable_profile_name(name,p.version()).is_err());}
 }
 #[test]fn r14_module_activation_uses_common_body_and_canon(){
  let ctx=context();let raw=source();let program=Parser::parse_with_first_callable_context(&raw,"利用.ddn",ctx.clone()).unwrap();assert_eq!(program.origin.source,raw);assert_eq!(program.root_stmts.len(),1);assert_eq!(program.items.len(),1);
  let canon=crate::canonicalizer::canonicalize_first_callable_registered_source(&program,&ctx).unwrap();assert!(canon.contains("옮김="));assert!(canon.starts_with("#!日本語\n"));Parser::parse_with_first_callable_context(&canon,"正本.ddn",ctx).unwrap();
 }

 #[test]fn r14_korean_entry_japanese_call_preserves_raw_and_rejects_unregistered_surface(){
  let ctx=context();let prefix=source().lines().nth(1).unwrap().to_string();
  let raw=format!("{prefix}\n실행:움직씨 := {{ ((金額=(100,1) 有理数, 税額=(10,1) 有理数) 合計する) 보여주기. }}.\n");
  let program=Parser::parse_with_first_callable_context(&raw,"main.ddn",ctx.clone()).unwrap();
  assert_eq!(program.origin.source,raw);assert!(program.root_stmts.is_empty());assert_eq!(program.items.len(),2);
  let entry=program.items.iter().find_map(|TopLevelItem::SeedDef(seed)|(seed.canonical_name=="실행").then_some(seed)).unwrap();
  assert_eq!(&raw[entry.span.start..entry.span.end],"실행:움직씨 := { ((金額=(100,1) 有理数, 税額=(10,1) 有理数) 合計する) 보여주기. }.");
  let canon=crate::canonicalizer::canonicalize_first_callable_registered_source(&program,&ctx).unwrap();
  assert!(canon.contains("움직씨"));assert!(canon.contains("合計する"));assert!(!canon.contains("총액합하다"));assert!(!canon.starts_with("#!日本語"));
  Parser::parse_with_first_callable_context(&canon,"main.ddn",ctx.clone()).unwrap();
  assert!(crate::parse_v25_with_mode(&raw,"main.ddn",ParseMode::Strict).is_err());
  for bad in [raw.replace("合計する","未登録"),raw.replace("(100,1)","（100,1）"),raw.replace("税額=","余分="),raw.replace("실행:움직씨","실행:셈씨"),raw.replace("움직씨 := {","움직씨 := { 0 보여주기."),raw.replace("대상판본=\"1\"","대상판본=\"latest\"")]{
   assert!(Parser::parse_with_first_callable_context(&bad,"main.ddn",ctx.clone()).is_err(),"{bad}");
  }
 }
 #[test]fn r14_missing_duplicate_wrong_profile_module_and_bad_pin_refuse(){
  let ctx=context();let raw=source();let prefix=raw.lines().nth(1).unwrap();for bad in [raw.replace(prefix,""),raw.replace("./세금","./다른모듈"),raw.replace("대상판본=\"1\"","대상판본=\"latest\""),raw.replace(", 税額=(10,1) 有理数",""),raw.replace(prefix,&format!("{prefix}\n{prefix}"))]{assert!(Parser::parse_with_first_callable_context(&bad,"利用.ddn",ctx.clone()).is_err(),"{bad}");}
 }
}


/// J1 role resolution operates on shared tokens, never translated source text.
pub(crate) fn j1_role_tokens(source:&str,profile:crate::ddl::CheckedJ1Profile)->Result<Vec<Token>,ParseError>{
    let mut tokens = Lexer::tokenize_j1_source(source, profile)
        .map_err(|(error, span)| ParseError { span: Span::new(span.start, span.end), message: error.message })?;
    for index in 0..tokens.len(){
        let TokenKind::Ident(name)=&tokens[index].kind else{continue};let name=name.clone();
        let colon=index>0&&matches!(tokens[index-1].kind,TokenKind::Colon);
        let after_value=index>0&&matches!(tokens[index-1].kind,TokenKind::RParen|TokenKind::RBracket);
        let expression_end=index>0&&matches!(tokens[index-1].kind,TokenKind::RParen|TokenKind::RBracket|TokenKind::Ident(_)|TokenKind::Josa(_)|TokenKind::Integer(_)|TokenKind::Float(_)|TokenKind::StringLit(_));
        let next=tokens.get(index+1).map(|t|&t.kind);
        let kind=match name.as_str(){
            "手続き" if colon=>Some(TokenKind::KwUmjikssi),
            "関数" if colon=>Some(TokenKind::KwSemssi),
            "有理数" if colon||after_value=>Some(TokenKind::Ident("나눔수".into())),
            "真偽" if colon=>Some(TokenKind::Ident("참거짓".into())),
            "列" if matches!(next,Some(TokenKind::LBracket))=>{
                Some(TokenKind::Ident("차림".into()))
            },
            "表示" if expression_end&&matches!(next,Some(TokenKind::Dot))=>{
                Some(TokenKind::KwBoyeojugi)
            },
            _=>None,
        };
        if let Some(kind)=kind{tokens[index].kind=kind;}
    }Ok(tokens)
}
impl Parser {
    /// Shared declaration-only grammar for a producer-selected J1 DDL text.
    /// Parsing returns syntax, never an exported library or activation authority.
    /// The owner must still validate the complete Definition/Pin/type denominator.
    pub fn parse_j1_ddl_declarations(source:&str,path:&str,profile:crate::ddl::CheckedJ1Profile)
        ->Result<Vec<SeedDef>,ParseError>{
        if source.len()>16384||!(path.ends_with(".ddl")||path.ends_with(".옮김")){
            return Err(ParseError{span:Span::new(0,0),message:"E_DDL_DECLARATION_INPUT".into()});
        }
        let tokens=j1_role_tokens(source,profile)?;
        let mut parser=Self::new_v25_with_mode(tokens,ParseMode::Strict);
        parser.source=source.into();parser.first_callable_ddl=true;
        let mut declarations=Vec::new();
        while !parser.is_at_end(){
            declarations.push(parser.parse_seed_def()?);
            if declarations.len()>128{return Err(ParseError{span:parser.current_span(),message:"E_DDL_PUBLIC_LIMIT".into()});}
        }
        Ok(declarations)
    }
    /// Declaration discovery only. Candidate spellings allow prefix syntax
    /// inspection; real graph/public owners must validate targets afterwards.
    pub fn parse_unlinked_j1_module(source:&str,path:&str,profile:crate::ddl::CheckedJ1Profile)->Result<CanonProgram,ParseError>{
        Self::parse_j1_public_original(source,path,1,profile).map(|(program,_)|program)
    }
    pub fn parse_j1_public_original(source:&str,path:&str,next_node:u64,profile:crate::ddl::CheckedJ1Profile)->Result<(CanonProgram,u64),ParseError>{
        Self::parse_j1_module_source(source,path,next_node,profile,false)
    }

    fn parse_j1_module_source(
        source:&str,path:&str,next_node:u64,profile:crate::ddl::CheckedJ1Profile,resolve_calls:bool,
    )->Result<(CanonProgram,u64),ParseError>{
        let tokens=j1_role_tokens(source,profile)?;
        let candidates=tokens.iter().filter_map(|token|match &token.kind{
            TokenKind::Ident(name)|TokenKind::Josa(name)|TokenKind::StringLit(name)=>Some(name.clone()),_=>None,
        }).collect::<Vec<_>>();
        let context=crate::module_directive::ModuleDirectiveContext::default().with_known_imports(candidates.clone()).with_public_definitions(candidates);
        let (document,tokens)=crate::module_directive::j1_module_tokens(source,path,&context,profile)?;
        let start=source.len()-document.declaration_source.len();
        let tokens=tokens.into_iter().filter(|token|token.span.start>=start).collect::<Vec<_>>();
        let shows=tokens.iter().filter(|token|matches!(token.kind,TokenKind::KwBoyeojugi)&&token.raw=="表示").map(|token|token.span.start).collect::<Vec<_>>();
        let mut parser=Self::new_v25_with_mode(tokens,ParseMode::Strict);parser.nid=next_node;
        let mut program=parser.parse_program_phase(source.into(),path.into(),resolve_calls,&[])?;
        crate::canonicalizer::resolve_j1_type_roles(&mut program,profile)?;
        resolve_j1_expression_roles(&mut program,&shows)?;Ok((program,parser.nid))
    }
    pub fn parse_j1_v25_with_mode(source:&str,path:&str,profile:crate::ddl::CheckedJ1Profile,mode:ParseMode)->Result<CanonProgram,ParseError>{
        let tokens=j1_role_tokens(source,profile)?;
        Self::parse_j1_tokens(source,path,tokens,mode,profile)
    }

    /// Imported checked definitions reserve their existing node range.
    pub fn parse_j1_linked_public_consumer(
        source: &str, path: &str, profile: crate::ddl::CheckedJ1Profile,
        context: &RegisteredPublicCallableContext,
    ) -> Result<CanonProgram, ParseError> {
        let tokens = j1_role_tokens(source, profile)?;
        Self::parse_j1_tokens_from(source, path, tokens, ParseMode::Strict, profile, context.next_node)
    }

    pub(crate) fn parse_j1_tokens(source:&str,path:&str,tokens:Vec<Token>,mode:ParseMode,profile:crate::ddl::CheckedJ1Profile)->Result<CanonProgram,ParseError>{
        Self::parse_j1_tokens_from(source, path, tokens, mode, profile, 1)
    }

    fn parse_j1_tokens_from(
        source: &str, path: &str, tokens: Vec<Token>, mode: ParseMode,
        profile: crate::ddl::CheckedJ1Profile, next_node: NodeId,
    ) -> Result<CanonProgram, ParseError> {
        let show_roles=tokens.iter().filter(|t|matches!(t.kind,TokenKind::KwBoyeojugi)&&t.raw=="表示").map(|t|t.span.start).collect::<Vec<_>>();
        let mut parser=Self::new_v25_with_mode(tokens,mode);
        parser.nid = next_node;
        let mut program=parser.parse_program(source.into(),path.into())?;
        crate::canonicalizer::resolve_j1_type_roles(&mut program,profile)?;
        resolve_j1_expression_roles(&mut program,&show_roles)?;Ok(program)
    }
}
#[cfg(test)]
mod r31_j1_parser_tests {
    use super::*;
    #[test] fn r31_j1_actual_definition_and_output_use_existing_ast(){
        let raw="#!日本語\n(金額:有理数~に, 税額:有理数~を) 合計:有理数:関数 := { 合計 <- 金額 + 税額. }.\n";
        let p=Parser::parse_j1_v25_with_mode(raw,"calc.ddn",crate::ddl::CheckedJ1Profile::resolve(crate::ddl::J1_SOURCE_PROFILE).unwrap(),ParseMode::Strict).unwrap();
        assert_eq!(p.origin.source,raw);let TopLevelItem::SeedDef(seed)=&p.items[0];
        assert_eq!(seed.canonical_name,"合計");assert_eq!(seed.seed_kind,SeedKind::Semssi);
        assert_eq!(seed.params[0].type_ref,TypeRef::Named("나눔수".into()));assert_eq!(seed.params[0].pin_name,"金額");
        let raw="#!日本語\n実行:手続き := { ((3,1) 有理数) 表示. }.";
        let p=Parser::parse_j1_v25_with_mode(raw,"main.ddn",crate::ddl::CheckedJ1Profile::resolve(crate::ddl::J1_SOURCE_PROFILE).unwrap(),ParseMode::Strict).unwrap();
        let TopLevelItem::SeedDef(seed)=&p.items[0];assert_eq!(seed.seed_kind,SeedKind::Umjikssi);
        assert!(matches!(seed.body.as_ref().unwrap().stmts[0],Stmt::Show{..}));
    }
}

fn resolve_j1_expression_roles(program:&mut CanonProgram,show_roles:&[usize])->Result<(),ParseError>{
    fn expr(e:&mut Expr,bound:&HashSet<String>)->Result<(),ParseError>{
        match &mut e.kind{
            ExprKind::Var(name) if matches!(name.as_str(),"真"|"偽")&&!bound.contains(name)=>e.kind=ExprKind::Literal(Literal::Bool(name=="真")),
            ExprKind::Call{func,args}=>{
                if func=="有理数"{
                    if bound.contains(func){return Err(ParseError{span:e.span,message:"E_J1_ROLE_AMBIGUOUS: 나눔수 생성 역할과 사용자 함수가 충돌합니다".into()});}
                    *func="나눔수".into();
                }
                for arg in args{expr(&mut arg.expr,bound)?;}
            },
            ExprKind::Infix{left,right,..}=>{expr(left,bound)?;expr(right,bound)?;},
            ExprKind::CollectionConstructor{elements}=>{
                if bound.contains("列"){return Err(ParseError{span:e.span,message:"E_J1_ROLE_AMBIGUOUS: 목록 역할과 사용자 이름이 충돌합니다".into()});}
                for element in elements{expr(element,bound)?;}
            },
            _=>{},
        }Ok(())
    }
    fn stmts(stmts:&mut [Stmt],bound:&mut HashSet<String>,show_roles:&[usize])->Result<(),ParseError>{
        for stmt in stmts{match stmt{
            Stmt::Show{expr:e,span,..}=>{
                if bound.contains("表示")&&show_roles.iter().any(|at|span.start<=*at&&*at<span.end){return Err(ParseError{span:*span,message:"E_J1_ROLE_AMBIGUOUS: 출력 역할과 보이는 사용자 이름이 충돌합니다".into()});}
                expr(e,bound)?;
            },
            Stmt::Expr{expr:e,..}|Stmt::Return{value:e,..}=>expr(e,bound)?,
            Stmt::Definition{name,value,..}=>{expr(value,bound)?;bound.insert(name.clone());},
            Stmt::Mutate{target,value,..}=>{expr(value,bound)?;if let ExprKind::Var(name)=&target.kind{bound.insert(name.clone());}},
            _=>{},
        }}Ok(())
    }
    let global=program.items.iter().map(|TopLevelItem::SeedDef(seed)|seed.canonical_name.clone()).collect::<HashSet<_>>();
    for TopLevelItem::SeedDef(seed) in &mut program.items{
        let mut bound=global.clone();bound.extend(seed.params.iter().map(|pin|pin.pin_name.clone()));
        if let Some(body)=&mut seed.body{stmts(&mut body.stmts,&mut bound,show_roles)?;}
    }
    stmts(&mut program.root_stmts,&mut global.clone(),show_roles)
}
#[cfg(test)]
mod r31_j1_scope_tests{
    use super::*;
    fn parse(s:&str)->Result<CanonProgram,ParseError>{Parser::parse_j1_v25_with_mode(s,"main.ddn",crate::ddl::CheckedJ1Profile::resolve(crate::ddl::J1_SOURCE_PROFILE).unwrap(),ParseMode::Strict)}
    #[test]fn r31_j1_role_spellings_are_not_global_user_name_reservations(){
        let p=parse("#!日本語\n(表示:有理数~を) F:有理数:関数 := { F <- 表示. }.\n実行:手続き := { ((3,1) 有理数) 表示. }.").unwrap();
        assert_eq!(p.items.len(),2);
        let p=parse("#!日本語\n(真:真偽~を) F:真偽:関数 := { F <- 真. }.\n実行:手続き := { 真 表示. }.").unwrap();
        let TopLevelItem::SeedDef(seed)=&p.items[1];assert!(matches!(&seed.body.as_ref().unwrap().stmts[0],Stmt::Show{expr:Expr{kind:ExprKind::Literal(Literal::Bool(true)),..},..}));
        assert!(parse("#!日本語\n(表示:有理数~を) F:有理数:関数 := { ((1,1) 有理数) 表示. }.").unwrap_err().message.contains("ROLE_AMBIGUOUS"));
    }
}

#[cfg(test)]
mod r31_pin_label_source_tests{
    use super::*;
    fn arg(program:&CanonProgram)->&ArgBinding{
        let seed=program.items.iter().map(|TopLevelItem::SeedDef(seed)|seed)
            .find(|seed|matches!(seed.seed_kind,SeedKind::Umjikssi)).unwrap();
        let Stmt::Show{expr,..}=&seed.body.as_ref().unwrap().stmts[0] else{panic!("show")};
        let ExprKind::Call{args,..}=&expr.kind else{panic!("call")}; &args[0]
    }
    #[test]fn r31_j1_pin_label_ko_value_range_stays_separate(){
        let raw="실행:움직씨 := { ((값=거짓) 아님) 보여주기. }.";
        let program=crate::parse_v25_with_mode(raw,"main.ddn",ParseMode::Strict).unwrap();
        let arg=arg(&program);let label=program.origin.authored_pin_label_spans[&arg.id];
        assert_eq!(&raw[label.start..label.end],"값");assert_eq!(&raw[arg.span.start..arg.span.end],"거짓");
        assert_eq!(arg.span,arg.expr.span);assert_ne!(label,arg.span);
    }
    #[test]fn r31_j1_pin_label_japanese_token_survives_canonicalization(){
        let raw="#!日本語\n(値:真偽) F:真偽:関数 := { F <- 値. }.\n実行:手続き := { ((値=偽) F) 表示. }.";
        let syntax=crate::ddl::CheckedJ1Profile::resolve(crate::ddl::J1_SOURCE_PROFILE).unwrap();
        let mut program=Parser::parse_j1_v25_with_mode(raw,"main.ddn",syntax,ParseMode::Strict).unwrap();
        crate::canonicalize(&mut program).unwrap();
        let arg=arg(&program);let label=program.origin.authored_pin_label_spans[&arg.id];
        assert_eq!(&raw[label.start..label.end],"値");assert_eq!(&raw[arg.span.start..arg.span.end],"偽");
        assert_eq!(label.end-label.start,3);assert_eq!(arg.span,arg.expr.span);assert_eq!(program.origin.source,raw);
    }
}


#[cfg(test)]
mod type_reference_prefix_sites {
    use super::*;
    fn parse(source:&str)->(TypeRef,Vec<(String,Span,bool)>){
        let tokens=Lexer::new(source).tokenize().unwrap();Parser::parse_type_reference_prefix(source,tokens).unwrap()
    }
    #[test]
    fn named_type_and_applied_constructor_have_distinct_exact_syntax_roles(){
        let source="(점수표)차림";let(ty,sites)=parse(source);
        assert_eq!(ty,TypeRef::Applied{name:"차림".into(),args:vec![TypeRef::Named("점수표".into())]});
        assert_eq!(sites.len(),2);assert!(sites[0].2);assert!(!sites[1].2);
        for(name,span,_)in sites{assert_eq!(&source[span.start..span.end],name);}
    }
    #[test]
    fn proof_expression_variables_are_not_named_type_sites(){
        let(ty,sites)=parse("(점수표 != 0)보임");assert!(matches!(ty,TypeRef::Applied{..}));
        assert!(sites.iter().all(|(_,_,named)|!*named));
    }
}
