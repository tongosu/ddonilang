//! Name binding A1: which source spans belong to each top-level callable of one
//! accepted source revision.
//!
//! This module only reads what the shared pipeline already produced: the
//! accepted `CanonProgram`, the shared lexer's tokens of the same text, and the
//! shared call-tail resolution (`resolve_known_call_target`). It never parses
//! partial text, never guesses a binding from spelling, and never fills a gap.
//! An occurrence it cannot classify exactly makes that definition
//! `Unsupported`, so a consumer keeps refusing to edit it.
//!
//! Current scope (SSOT RD370·379·380·085 confirmed; general shadowing, local vs
//! `…하기` callee priority and root value/callable homonyms are UNKNOWN and so
//! unsupported): top-level callables of one document. Parameters are listed but
//! not resolved. Locals are not enumerated; a same-spelling local makes the
//! callable unsupported.
use crate::ddn_runtime::{resolve_supported_source_profile, DdnProgram};
use crate::source_analysis::{analyze_source, SourceAnalysisRequest};
use ddonirang_lang::stdlib::minimal_stdlib_sigs;
use ddonirang_lang::{
    resolve_known_call_target, Body, Expr, ExprKind, Lexer, ParseMode, SeedDef, Span, Stmt,
    Token, TokenKind, TopLevelItem,
};
use std::collections::HashSet;

pub const NAME_BINDING_SCHEMA: &str = "ddn.name_binding.a1.v1";
const SOURCE_PROFILE: &str = "v1-core-v25";

/// Half-open UTF-8 byte range in the analyzed source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingOutcome {
    Analyzed,
    /// The shared check (`ddonilang 검사`) did not accept this text.
    SourceNotAccepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionKind {
    TopLevelCallable,
    Parameter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OccurrenceRole {
    Declaration,
    /// `이름 <- …` inside the callable's own body: the invocation-local result
    /// place (RD370·379), not a use of the callable.
    ResultWrite,
    /// The stem of a call surface such as `두배하기`, resolved by the shared
    /// call-tail rule to this callable.
    CallStem,
    /// The bare name used as a value outside the callable's own body.
    ValueReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameOccurrence {
    pub role: OccurrenceRole,
    /// Exactly the bytes spelling the definition name.
    pub name_range: ByteRange,
    /// The whole shared-lexer token containing the name.
    pub token_range: ByteRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnsupportedReason {
    OutOfA1Scope,
    RegisteredSurfaces,
    DuplicateDefinition,
    StdlibSameName,
    SameSpellingParameter,
    SameSpellingLocalBinding,
    NoDeclarationToken,
    ResultPlaceRead,
    CallResolutionMismatch,
    /// The AST holds a variable whose spelling the shared call-tail rule maps
    /// to this callable (e.g. a body statement `첫함수하기.`). Whether that is a
    /// use of the callable is not decided by the accepted AST.
    CallTailParsedAsVariable,
    UnclassifiedOccurrence,
    NameInsideOtherToken,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedSite {
    pub reason: UnsupportedReason,
    pub range: Option<ByteRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Every occurrence of this definition in the document, nothing else.
    Exact(Vec<NameOccurrence>),
    Unsupported(Vec<UnsupportedSite>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameDefinition {
    /// Valid only together with `NameBindingReport::source_revision_identity`.
    pub definition_id: String,
    pub name: String,
    pub kind: DefinitionKind,
    /// For a callable, the name token; for a parameter, its whole pin syntax.
    pub declaration: ByteRange,
    pub resolution: Resolution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameBindingReport {
    pub schema: &'static str,
    pub source_revision_identity: String,
    pub outcome: BindingOutcome,
    pub definitions: Vec<NameDefinition>,
}

impl NameBindingReport {
    /// The definition and occurrence whose name range covers `offset`, if the
    /// definition is resolved exactly.
    pub fn occurrence_at(&self, offset: usize) -> Option<(&NameDefinition, &NameOccurrence)> {
        self.definitions.iter().find_map(|definition| match &definition.resolution {
            Resolution::Exact(occurrences) => occurrences
                .iter()
                .find(|o| o.name_range.start <= offset && offset <= o.name_range.end)
                .map(|o| (definition, o)),
            Resolution::Unsupported(_) => None,
        })
    }
}

pub fn bind_document(source: &str, path: &str) -> NameBindingReport {
    let mut report = NameBindingReport {
        schema: NAME_BINDING_SCHEMA,
        source_revision_identity: format!("blake3:{}", blake3::hash(source.as_bytes()).to_hex()),
        outcome: BindingOutcome::SourceNotAccepted,
        definitions: Vec::new(),
    };
    let checked = analyze_source(SourceAnalysisRequest {
        source_text: source.to_string(),
        path: path.to_string(),
        analysis_generation: 0,
        source_profile_identity: SOURCE_PROFILE.to_string(),
    });
    if checked.outcome != "ACCEPTED" {
        return report;
    }
    let Ok(profile) = resolve_supported_source_profile(Some(SOURCE_PROFILE)) else {
        return report;
    };
    let Ok(program) = DdnProgram::from_supported_source_profile_with_mode(
        source,
        path,
        ParseMode::Strict,
        profile,
    ) else {
        return report;
    };
    let canon = program.canonical_program();
    let Ok(tokens) = Lexer::new(source).tokenize() else {
        return report;
    };
    // Spans must address this exact text.
    if canon.origin.source != source {
        return report;
    }
    report.outcome = BindingOutcome::Analyzed;

    let seeds: Vec<&SeedDef> = canon
        .items
        .iter()
        .map(|item| {
            let TopLevelItem::SeedDef(seed) = item;
            seed
        })
        .collect();
    let mut facts = Facts::default();
    for seed in &seeds {
        if let Some(body) = &seed.body {
            facts.body(body);
        }
        for param in &seed.params {
            if let Some(value) = &param.default_value {
                facts.expr(value);
            }
        }
    }
    for statement in &canon.root_stmts {
        facts.stmt(statement);
    }
    let known_seeds: HashSet<String> = seeds.iter().map(|s| s.canonical_name.clone()).collect();
    let stdlib: HashSet<&str> = minimal_stdlib_sigs().iter().map(|sig| sig.name).collect();
    let any_registered_surface = seeds.iter().any(|s| !s.alternate_forms.is_empty());

    for seed in &seeds {
        let resolution = resolve_callable(
            seed,
            &seeds,
            &tokens,
            &facts,
            &known_seeds,
            &stdlib,
            any_registered_surface,
        );
        let declaration = declaration_token(seed, &tokens)
            .map(|t| range(t.span.start, t.span.end))
            .unwrap_or(range(seed.span.start, seed.span.start));
        report.definitions.push(NameDefinition {
            definition_id: format!("definition@{}:{}", declaration.start, declaration.end),
            name: seed.canonical_name.clone(),
            kind: DefinitionKind::TopLevelCallable,
            declaration,
            resolution,
        });
        for param in &seed.params {
            let declaration = range(param.span.start, param.span.end);
            report.definitions.push(NameDefinition {
                definition_id: format!("parameter@{}:{}", declaration.start, declaration.end),
                name: param.pin_name.clone(),
                kind: DefinitionKind::Parameter,
                declaration,
                resolution: Resolution::Unsupported(vec![UnsupportedSite {
                    reason: UnsupportedReason::OutOfA1Scope,
                    range: Some(declaration),
                }]),
            });
        }
    }
    report
}

fn resolve_callable(
    seed: &SeedDef,
    seeds: &[&SeedDef],
    tokens: &[Token],
    facts: &Facts,
    known_seeds: &HashSet<String>,
    stdlib: &HashSet<&str>,
    any_registered_surface: bool,
) -> Resolution {
    let name = seed.canonical_name.as_str();
    let mut unsupported = Vec::new();
    let mut flag = |reason, range: Option<ByteRange>| unsupported.push(UnsupportedSite { reason, range });

    if any_registered_surface {
        flag(UnsupportedReason::RegisteredSurfaces, None);
    }
    if seeds.iter().filter(|s| s.canonical_name == name).count() != 1 {
        flag(UnsupportedReason::DuplicateDefinition, None);
    }
    if stdlib.contains(name) {
        flag(UnsupportedReason::StdlibSameName, None);
    }
    for other in seeds {
        for param in &other.params {
            if param.pin_name == name {
                flag(UnsupportedReason::SameSpellingParameter, Some(range(param.span.start, param.span.end)));
            }
        }
    }
    for (binder, span) in &facts.binders {
        if binder == name {
            flag(UnsupportedReason::SameSpellingLocalBinding, Some(range(span.start, span.end)));
        }
    }

    let declaration = declaration_token(seed, tokens);
    if declaration.is_none() {
        flag(UnsupportedReason::NoDeclarationToken, Some(range(seed.span.start, seed.span.end)));
    }
    let body_span = seed.body.as_ref().map(|b| b.span);
    let inside_body = |span: Span| body_span.is_some_and(|b| b.start <= span.start && span.end <= b.end);

    let mut occurrences = Vec::new();
    for token in tokens {
        let token_range = range(token.span.start, token.span.end);
        let token_span = Span::new(token.span.start, token.span.end);
        let TokenKind::Ident(_) = &token.kind else {
            if !matches!(token.kind, TokenKind::StringLit(_)) && !name.is_empty() && token.raw.contains(name) {
                flag(UnsupportedReason::NameInsideOtherToken, Some(token_range));
            }
            continue;
        };
        let raw = token.raw.as_str();
        if !raw.starts_with(name) {
            continue;
        }
        if declaration.is_some_and(|d| std::ptr::eq(d, token)) {
            occurrences.push(occurrence(OccurrenceRole::Declaration, token, name));
            continue;
        }
        if facts.other_symbol_spans.iter().any(|s| same(*s, token_span)) {
            // A field name or observation field: a different symbol.
            continue;
        }
        if raw != name
            && facts.binders.iter().any(|(binder, span)| {
                binder == raw && span.start <= token.span.start && token.span.end <= span.end
            })
        {
            // The declaration of another binder whose whole spelling differs,
            // such as `두배수 := 5.`.
            continue;
        }
        if let Some((var, span)) = facts.vars.iter().find(|(_, s)| same(*s, token_span)) {
            if var != name {
                if matches!(
                    resolve_known_call_target(var, known_seeds, token_span),
                    Ok((stem, _)) if stem == name
                ) {
                    flag(UnsupportedReason::CallTailParsedAsVariable, Some(token_range));
                }
                continue; // otherwise an exact different name such as `두배수`
            }
            if inside_body(*span) {
                if facts.mutate_targets.iter().any(|s| same(*s, *span)) {
                    occurrences.push(occurrence(OccurrenceRole::ResultWrite, token, name));
                } else {
                    flag(UnsupportedReason::ResultPlaceRead, Some(token_range));
                }
            } else {
                occurrences.push(occurrence(OccurrenceRole::ValueReference, token, name));
            }
            continue;
        }
        if let Some(call) = facts.call_owning(token_span) {
            match resolve_known_call_target(raw, known_seeds, token_span) {
                Ok((stem, canonical)) if canonical == call.func => {
                    if stem == name {
                        occurrences.push(occurrence(OccurrenceRole::CallStem, token, name));
                    }
                    // otherwise a call to another callable whose spelling
                    // merely starts with this name
                }
                _ => flag(UnsupportedReason::CallResolutionMismatch, Some(token_range)),
            }
            continue;
        }
        flag(UnsupportedReason::UnclassifiedOccurrence, Some(token_range));
    }

    if unsupported.is_empty() {
        occurrences.sort_by_key(|o| o.name_range);
        Resolution::Exact(occurrences)
    } else {
        Resolution::Unsupported(unsupported)
    }
}

/// The single shared-lexer token spelling the callable's name in its own
/// declaration: inside the definition, before its body, outside parameters.
fn declaration_token<'a>(seed: &SeedDef, tokens: &'a [Token]) -> Option<&'a Token> {
    let header_end = seed.body.as_ref().map_or(seed.span.end, |b| b.span.start);
    let mut found = tokens.iter().filter(|token| {
        token.raw == seed.canonical_name
            && matches!(token.kind, TokenKind::Ident(_))
            && seed.span.start <= token.span.start
            && token.span.end <= header_end
            && !seed
                .params
                .iter()
                .any(|p| p.span.start <= token.span.start && token.span.end <= p.span.end)
    });
    let first = found.next();
    if found.next().is_some() {
        return None;
    }
    first
}

fn occurrence(role: OccurrenceRole, token: &Token, name: &str) -> NameOccurrence {
    NameOccurrence {
        role,
        name_range: range(token.span.start, token.span.start + name.len()),
        token_range: range(token.span.start, token.span.end),
    }
}

fn range(start: usize, end: usize) -> ByteRange {
    ByteRange { start, end }
}

fn same(left: Span, right: Span) -> bool {
    left.start == right.start && left.end == right.end
}

struct CallFact {
    func: String,
    span: Span,
    arg_spans: Vec<Span>,
}

/// What the accepted AST says, collected by one walk. Statement or expression
/// forms the walk does not enter leave their tokens unclassified, which makes
/// the affected definition unsupported rather than wrongly resolved.
#[derive(Default)]
struct Facts {
    vars: Vec<(String, Span)>,
    calls: Vec<CallFact>,
    mutate_targets: Vec<Span>,
    /// Field names and observation field names: other symbols.
    other_symbol_spans: Vec<Span>,
    /// Statement-level binders (locals, root values, loop items, …).
    binders: Vec<(String, Span)>,
}

impl Facts {
    /// The innermost call whose own surface (not its arguments) holds `span`.
    fn call_owning(&self, span: Span) -> Option<&CallFact> {
        self.calls
            .iter()
            .filter(|call| {
                call.span.start <= span.start
                    && span.end <= call.span.end
                    && !call.arg_spans.iter().any(|a| a.start <= span.start && span.end <= a.end)
            })
            .min_by_key(|call| call.span.end - call.span.start)
    }

    fn body(&mut self, body: &Body) {
        for statement in &body.stmts {
            self.stmt(statement);
        }
    }

    fn stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::DerivedBinding { name, value, span, .. } => {
                self.binders.push((name.clone(), *span));
                self.expr(value);
            }
            Stmt::SumDefinition { name, span, .. } => self.binders.push((name.clone(), *span)),
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                self.expr(subject);
                for arm in arms {
                    for binder in &arm.binder {
                        self.binders.push((binder.pin.clone(), binder.span));
                    }
                    self.body(&arm.body);
                }
                if let Some(body) = default_body {
                    self.body(body);
                }
            }
            Stmt::DeclBlock { items, .. } => {
                for item in items {
                    self.binders.push((item.name.clone(), item.span));
                    if let Some(value) = &item.value {
                        self.expr(value);
                    }
                }
            }
            Stmt::Definition { name, value, span, .. } => {
                self.binders.push((name.clone(), *span));
                self.expr(value);
            }
            Stmt::Mutate { target, value, .. } => {
                if let ExprKind::Var(_) = &target.kind {
                    self.mutate_targets.push(target.span);
                }
                self.expr(target);
                self.expr(value);
            }
            Stmt::ResultFlow { producer, destination, .. } => {
                self.expr(producer);
                self.expr(destination);
            }
            Stmt::Expr { expr, .. }
            | Stmt::Show { expr, .. }
            | Stmt::Inspect { expr, .. }
            | Stmt::Return { value: expr, .. } => self.expr(expr),
            Stmt::Receive { sender, condition, body, binding, span, .. } => {
                if let Some(binding) = binding {
                    self.binders.push((binding.clone(), *span));
                }
                if let Some(sender) = sender {
                    self.expr(sender);
                }
                if let Some(condition) = condition {
                    self.expr(condition);
                }
                self.body(body);
            }
            Stmt::Send { sender, payload, receiver, .. } => {
                if let Some(sender) = sender {
                    self.expr(sender);
                }
                self.expr(payload);
                self.expr(receiver);
            }
            Stmt::PublicObservation { fields, .. } => {
                for field in fields {
                    self.other_symbol_spans.push(field.name_span);
                    self.expr(&field.value);
                }
            }
            Stmt::If { condition, then_body, else_body, .. } => {
                self.expr(condition);
                self.body(then_body);
                if let Some(body) = else_body {
                    self.body(body);
                }
            }
            Stmt::Try { action, body, .. } => {
                self.expr(action);
                self.body(body);
            }
            Stmt::Choose { branches, else_body, .. } => {
                for branch in branches {
                    self.expr(&branch.condition);
                    self.body(&branch.body);
                }
                self.body(else_body);
            }
            Stmt::Repeat { body, .. }
            | Stmt::BeatBlock { body, .. }
            | Stmt::Hook { body, .. } => self.body(body),
            Stmt::While { condition, body, .. }
            | Stmt::Guard { condition, body, .. }
            | Stmt::HookWhenBecomes { condition, body, .. }
            | Stmt::HookWhile { condition, body, .. } => {
                self.expr(condition);
                self.body(body);
            }
            Stmt::ForEach { item, iterable, body, span, .. } => {
                self.binders.push((item.clone(), *span));
                self.expr(iterable);
                self.body(body);
            }
            Stmt::Quantifier { variable, body, span, .. } => {
                self.binders.push((variable.clone(), *span));
                self.body(body);
            }
            Stmt::Contract { condition, then_body, else_body, .. } => {
                self.expr(condition);
                if let Some(body) = then_body {
                    self.body(body);
                }
                self.body(else_body);
            }
            Stmt::MetaBlock { .. }
            | Stmt::Pragma { .. }
            | Stmt::Break { .. }
            | Stmt::ContinueLoop { .. } => {}
        }
    }

    fn expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Var(name) => self.vars.push((name.clone(), expr.span)),
            ExprKind::Call { args, func } => {
                self.calls.push(CallFact {
                    func: func.clone(),
                    span: expr.span,
                    arg_spans: args.iter().map(|a| a.expr.span).collect(),
                });
                for arg in args {
                    self.expr(&arg.expr);
                }
            }
            ExprKind::FieldAccess { target, field_span, .. } => {
                self.other_symbol_spans.push(*field_span);
                self.expr(target);
            }
            ExprKind::SeedLiteral { param, body } => {
                self.binders.push((param.clone(), expr.span));
                self.expr(body);
            }
            ExprKind::ProductConstruction { fields, .. } => {
                for field in fields {
                    self.expr(&field.expr);
                }
            }
            ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
                for element in elements {
                    self.expr(element);
                }
            }
            ExprKind::SelectionPlace { target, index } => {
                self.expr(target);
                self.expr(index);
            }
            ExprKind::Infix { left, right, .. } => {
                self.expr(left);
                self.expr(right);
            }
            ExprKind::Suffix { value, .. } => self.expr(value),
            ExprKind::Thunk(body) => self.body(body),
            ExprKind::Eval { thunk, .. } => self.expr(thunk),
            ExprKind::Pipe { stages } => {
                for stage in stages {
                    self.expr(stage);
                }
            }
            ExprKind::Pack { fields } => {
                for (_, value) in fields {
                    self.expr(value);
                }
            }
            ExprKind::Nuance { expr, .. } => self.expr(expr),
            // Not entered in A1: their tokens stay unclassified.
            _ => {}
        }
    }
}
