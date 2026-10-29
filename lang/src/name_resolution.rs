//! RD710 compilation-local lexical facts from the shared, fully parsed AST.
//! No saved/registered identity, runtime capability or editor edit is issued.
//! Authored call candidates remain candidates; an overload winner is not guessed.
use crate::ast::*;
use crate::lexer::{Token, TokenKind};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefinitionKey {
    revision: [u8; 32],
    ordinal: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionKind {
    Callable,
    Parameter,
    Local,
    ResultPlace,
}
/// The existing coarse name kind does not confer ordinary lexical ownership.
/// Relocated setup sites belong to the actual parser's top-level setup owner;
/// their key locates source only, never issues a registered resource identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionOwnerScope {
    Lexical,
    RelocatedTopLevelSetup,
}
#[derive(Debug, Clone)]
pub struct NameDefinition {
    key: DefinitionKey,
    name: String,
    kind: DefinitionKind,
    span: Span,
    name_span: Option<Span>,
    non_callable: bool,
    owner_callable: Option<DefinitionKey>,
    owner_scope: DefinitionOwnerScope,
}
impl NameDefinition {
    pub fn owner_scope(&self) -> DefinitionOwnerScope { self.owner_scope }
    pub fn key(&self) -> DefinitionKey {
        self.key
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn kind(&self) -> DefinitionKind {
        self.kind
    }
    pub fn syntax_span(&self) -> Span {
        self.span
    }
    pub fn name_span(&self) -> Option<Span> {
        self.name_span
    }
    pub fn owner_callable(&self) -> Option<DefinitionKey> {
        self.owner_callable
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Issue {
    UnknownName,
    NoUniqueNameToken,
    OwnerSpecificScope,
    CallOwnerNotSupplied,
    CallableValueTypeNotSupplied,
    DuplicateBinding,
    RootValueCallableConflict,
    SelfNameShadow,
    ShadowedNonCallable,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingState {
    Final(DefinitionKey),
    Candidates(Vec<DefinitionKey>),
    Unresolved(Issue),
    Error(Issue),
}
#[derive(Debug, Clone)]
pub struct NameUse {
    node: NodeId,
    span: Span,
    spelling: String,
    state: BindingState,
}
impl NameUse {
    pub fn node(&self) -> NodeId {
        self.node
    }
    pub fn name_span(&self) -> Span {
        self.span
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub fn state(&self) -> &BindingState {
        &self.state
    }
}
#[derive(Debug, Clone)]
pub struct NameResolutionFacts {
    revision: [u8; 32],
    source_hash: [u8; 32],
    path: String,
    definitions: Vec<NameDefinition>,
    uses: Vec<NameUse>,
    errors: Vec<(Span, &'static str)>,
}
impl NameResolutionFacts {
    pub fn is_for_source(&self, source: &str, path: &str) -> bool {
        self.path == path && self.source_hash == *blake3::hash(source.as_bytes()).as_bytes()
    }
    pub fn definitions(&self) -> &[NameDefinition] {
        &self.definitions
    }
    pub fn uses(&self) -> &[NameUse] {
        &self.uses
    }
    pub fn definition(&self, key: DefinitionKey) -> Option<&NameDefinition> {
        (key.revision == self.revision)
            .then(|| self.definitions.get(key.ordinal))
            .flatten()
    }
    pub fn use_at(&self, offset: usize) -> Option<&NameUse> {
        let index = self
            .uses
            .partition_point(|u| u.span.start <= offset)
            .checked_sub(1)?;
        let usage = self.uses.get(index)?;
        (offset < usage.span.end && usage.spelling.is_char_boundary(offset - usage.span.start))
            .then_some(usage)
    }
    pub(crate) fn first_admission_error(&self) -> Option<(Span, &'static str)> {
        self.errors.first().copied()
    }
}
type Scope = HashMap<String, Vec<DefinitionKey>>;
struct Resolver<'a> {
    program: &'a CanonProgram,
    tokens: Vec<&'a Token>,
    use_index: HashMap<(usize, usize), usize>,
    facts: NameResolutionFacts,
    scopes: Vec<Scope>,
    uncertain: Vec<HashSet<String>>,
    callables: HashMap<NodeId, DefinitionKey>,
    unknown_global: HashSet<String>,
    self_name: Option<String>,
    setup_resources: HashMap<NodeId, DefinitionKey>,
    phase_scope: bool,
}
impl Resolver<'_> {
    fn source_name_token(&self, token: &Token) -> bool {
        matches!(token.kind, TokenKind::Ident(_))
            && self
                .program
                .origin
                .source
                .get(token.span.start..token.span.end)
                == Some(token.raw.as_str())
    }
    fn tokens_in(&self, span: Span) -> impl Iterator<Item = &Token> + '_ {
        let start = self.tokens.partition_point(|t| t.span.start < span.start);
        self.tokens[start..]
            .iter()
            .copied()
            .take_while(move |t| t.span.start < span.end)
            .filter(move |t| t.span.end <= span.end)
    }
    fn declaration_span(&self, name: &str, span: Span, leading: bool) -> Option<Span> {
        let mut found = self.tokens_in(span).filter(|t| {
            self.source_name_token(t)
                && t.raw == name
                && span.start <= t.span.start
                && t.span.end <= span.end
                && (!leading || t.span.start == span.start)
        });
        let first = found.next()?;
        if found.next().is_some() {
            None
        } else {
            Some(Span::new(first.span.start, first.span.end))
        }
    }
    fn add(
        &mut self,
        name: &str,
        kind: DefinitionKind,
        span: Span,
        name_span: Option<Span>,
        non_callable: bool,
    ) -> DefinitionKey {
        let key = DefinitionKey {
            revision: self.facts.revision,
            ordinal: self.facts.definitions.len(),
        };
        self.facts.definitions.push(NameDefinition {
            key,
            name: name.into(),
            kind,
            span,
            name_span,
            non_callable,
            owner_callable: None,
            owner_scope: DefinitionOwnerScope::Lexical,
        });
        key
    }
    fn note(&mut self, node: NodeId, span: Span, state: BindingState) {
        let Some(token) = self.tokens_in(span).find(|t| {
            t.span.start == span.start && t.span.end == span.end && self.source_name_token(t)
        }) else {
            return;
        };
        let spelling = token.raw.clone();
        if let Some(index) = self.use_index.get(&(span.start, span.end)).copied() {
            // Only private note() inserts indices into this matching vector.
            let old = &mut self.facts.uses[index];
            if old.state != state {
                old.state = BindingState::Unresolved(Issue::OwnerSpecificScope);
            }
            return;
        }
        self.use_index
            .insert((span.start, span.end), self.facts.uses.len());
        self.facts.uses.push(NameUse {
            node,
            span,
            spelling,
            state,
        });
    }
    fn lookup(&self, name: &str, call: bool) -> Option<Vec<DefinitionKey>> {
        for (index, scope) in self.scopes.iter().enumerate().rev() {
            if self.uncertain[index].contains(name) {
                return None;
            }
            if let Some(keys) = scope.get(name) {
                if call
                    && keys.iter().all(|k| {
                        self.facts.definitions[k.ordinal].kind == DefinitionKind::ResultPlace
                    })
                {
                    continue;
                }
                return Some(keys.clone());
            }
        }
        None
    }
    fn uncertain_name(&self, name: &str, call: bool) -> bool {
        for (index, scope) in self.scopes.iter().enumerate().rev() {
            if self.uncertain[index].contains(name) {
                return true;
            }
            if let Some(keys) = scope.get(name) {
                if call
                    && keys.iter().all(|k| {
                        self.facts.definitions[k.ordinal].kind == DefinitionKind::ResultPlace
                    })
                {
                    continue;
                }
                return false;
            }
        }
        false
    }
    fn local(
        &mut self,
        node: NodeId,
        name: &str,
        span: Span,
        type_ref: &TypeRef,
        value: Option<&Expr>,
        kind: DefinitionKind,
    ) {
        let name_span = self.declaration_span(name, span, true);
        let non_callable = crate::ordinary_callable::closed_non_callable_type(type_ref)
            || value.is_some_and(|v| matches!(v.kind, ExprKind::Literal(_)));
        let key = self.add(name, kind, span, name_span, non_callable);
        let mut issue = None;
        if self.self_name.as_deref() == Some(name) {
            issue = Some(Issue::SelfNameShadow);
        }
        if let Some(previous) = self
            .scopes
            .last()
            .unwrap()
            .get(name)
            .filter(|_| issue.is_none())
        {
            issue = Some(
                if self.scopes.len() == 1
                    && previous
                        .iter()
                        .any(|k| self.facts.definitions[k.ordinal].kind == DefinitionKind::Callable)
                {
                    Issue::RootValueCallableConflict
                } else {
                    Issue::DuplicateBinding
                },
            );
        }
        if let Some(issue) = issue {
            let message = match issue {
                Issue::RootValueCallableConflict => {
                    "같은 루트 범위의 값과 호출 정의 이름이 충돌합니다"
                }
                Issue::SelfNameShadow => "자기 결과 자리 이름을 지역 결속으로 가릴 수 없습니다",
                _ => "같은 지역 범위에 같은 이름을 중복 결속할 수 없습니다",
            };
            self.facts.errors.push((name_span.unwrap_or(span), message));
            if let Some(at) = name_span {
                self.note(node, at, BindingState::Error(issue));
            }
        } else {
            self.scopes
                .last_mut()
                .unwrap()
                .insert(name.into(), vec![key]);
            if let Some(at) = name_span {
                self.note(node, at, BindingState::Final(key));
            }
        }
    }
    fn var(&mut self, expr: &Expr, name: &str) {
        let Some(at) = self.declaration_span(name, expr.span, true) else {
            return;
        };
        let state = if self.uncertain_name(name, false) {
            BindingState::Unresolved(Issue::OwnerSpecificScope)
        } else {
            match self.lookup(name, false) {
                Some(keys) if keys.len() == 1 => {
                    let key = keys[0];
                    if (self.facts.definitions[key.ordinal].kind == DefinitionKind::Callable
                        && self.unknown_global.contains(name))
                        || (self.facts.definitions[key.ordinal].owner_scope == DefinitionOwnerScope::RelocatedTopLevelSetup
                            && !self.phase_scope)
                    {
                        BindingState::Unresolved(Issue::OwnerSpecificScope)
                    } else {
                        BindingState::Final(key)
                    }
                }
                Some(keys) => BindingState::Candidates(keys),
                None => BindingState::Unresolved(Issue::UnknownName),
            }
        };
        self.note(expr.id, at, state);
    }
    fn body(&mut self, body: &Body) {
        self.scopes.push(Scope::new());
        self.uncertain.push(HashSet::new());
        self.statements(&body.stmts);
        self.scopes.pop();
        self.uncertain.pop();
    }
    fn statements(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.statement(stmt);
        }
    }
    fn opaque(&mut self, span: Span) {
        let tokens = self
            .tokens_in(span)
            .filter(|t| {
                span.start <= t.span.start && t.span.end <= span.end && self.source_name_token(t)
            })
            .map(|t| Span::new(t.span.start, t.span.end))
            .collect::<Vec<_>>();
        for at in tokens {
            self.note(0, at, BindingState::Unresolved(Issue::OwnerSpecificScope));
        }
    }
    fn statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Definition {
                id,
                name,
                span,
                type_ref,
                value,
                ..
            } => {
                self.expr(value);
                self.local(
                    *id,
                    name,
                    *span,
                    type_ref,
                    Some(value),
                    DefinitionKind::Local,
                );
            }
            // DeclBlock participates in the existing world/storage owner, not an
            // ordinary lexical Definition. Do not invent a resource binding ID.
            Stmt::DeclBlock { items, .. }
                if !items.is_empty() && items.iter().all(|i| self.setup_resources.contains_key(&i.id)) => {
                // The same SeedDef owner proved these top-level setup sites.
                // Their source declarations were noted once in the root scope.
                for item in items {
                    if let Some(value) = &item.value { self.expr(value); }
                }
            }
            Stmt::DeclBlock { items, span, .. } => {
                for i in items {
                    self.unknown_global.insert(i.name.clone());
                    self.uncertain.last_mut().unwrap().insert(i.name.clone());
                }
                self.opaque(*span);
            }
            Stmt::Mutate { target, value, .. } => {
                self.expr(target);
                self.expr(value);
            }
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => {
                self.expr(producer);
                self.expr(destination);
            }
            Stmt::Expr { expr, .. }
            | Stmt::Show { expr, .. }
            | Stmt::Inspect { expr, .. }
            | Stmt::Return { value: expr, .. } => self.expr(expr),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr(condition);
                self.body(then_body);
                if let Some(b) = else_body {
                    self.body(b);
                }
            }
            Stmt::While {
                condition, body, ..
            }
            | Stmt::Guard {
                condition, body, ..
            } => {
                self.expr(condition);
                self.body(body);
            }
            Stmt::Repeat { body, .. } => self.body(body),
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for b in branches {
                    self.expr(&b.condition);
                    self.body(&b.body);
                }
                self.body(else_body);
            }
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.expr(condition);
                if let Some(b) = then_body {
                    self.body(b);
                }
                self.body(else_body);
            }
            Stmt::PublicObservation { fields, .. } => {
                for f in fields {
                    self.expr(&f.value);
                }
            }
            // These scopes have their own owner. Their raw tokens cannot become
            // exact ordinary lexical bindings merely because the AST is present.
            Stmt::ForEach { iterable, body, .. } => {
                self.expr(iterable);
                self.opaque(body.span);
            }
            Stmt::Hook { kind: HookKind::EveryMadi, body, .. }
                if !self.setup_resources.is_empty() && body.stmts.iter().all(|stmt|
                    matches!(stmt, Stmt::Mutate { .. } | Stmt::Expr { .. }
                        | Stmt::Show { .. } | Stmt::Inspect { .. })) => {
                let previous = self.phase_scope;
                self.phase_scope = true;
                self.body(body);
                self.phase_scope = previous;
            }
            Stmt::BeatBlock { body, .. }
            | Stmt::Hook { body, .. }
            | Stmt::HookWhenBecomes { body, .. }
            | Stmt::HookWhile { body, .. }
            | Stmt::Quantifier { body, .. } => self.opaque(body.span),
            Stmt::SumDefinition { name, span, .. } | Stmt::DerivedBinding { name, span, .. } => {
                self.uncertain.last_mut().unwrap().insert(name.clone());
                self.opaque(*span);
            }
            Stmt::Receive { span, .. }
            | Stmt::Send { span, .. }
            | Stmt::Try { span, .. }
            | Stmt::SumMatch { span, .. } => self.opaque(*span),
            Stmt::MetaBlock { .. }
            | Stmt::Pragma { .. }
            | Stmt::Break { .. }
            | Stmt::ContinueLoop { .. } => {}
        }
    }
    fn expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Var(name) => self.var(expr, name),
            ExprKind::Call { args, .. } => {
                // Injected/default expressions belong to their original source
                // context; do not bind a copied default in the caller's scope.
                for arg in args {
                    if expr.span.start <= arg.expr.span.start && arg.expr.span.end <= expr.span.end
                    {
                        self.expr(&arg.expr);
                    }
                }
                let name_span = self
                    .program
                    .origin
                    .authored_call_name_spans
                    .get(&expr.id)
                    .copied();
                let diagnostic_span = name_span.unwrap_or(expr.span);
                let state = if let Some(nodes) =
                    self.program.origin.authored_call_candidates.get(&expr.id)
                {
                    let candidates = nodes
                        .iter()
                        .filter_map(|id| self.callables.get(id).copied())
                        .collect::<Vec<_>>();
                    let mut shadow = None;
                    for key in &candidates {
                        let name = &self.facts.definitions[key.ordinal].name;
                        if let Some(local) = self.lookup(name, true) {
                            if local.iter().any(|k| {
                                self.facts.definitions[k.ordinal].kind != DefinitionKind::Callable
                            }) {
                                shadow = Some(local);
                                break;
                            }
                        }
                    }
                    if candidates.iter().any(|k| {
                        self.uncertain_name(&self.facts.definitions[k.ordinal].name, true)
                            || self
                                .unknown_global
                                .contains(&self.facts.definitions[k.ordinal].name)
                    }) {
                        BindingState::Unresolved(Issue::OwnerSpecificScope)
                    } else if let Some(local) = shadow {
                        if local
                            .iter()
                            .all(|k| self.facts.definitions[k.ordinal].non_callable)
                        {
                            self.facts.errors.push((
                                diagnostic_span,
                                "지역 값에 가려진 호출을 바깥 정의로 대신 실행할 수 없습니다",
                            ));
                            BindingState::Error(Issue::ShadowedNonCallable)
                        } else {
                            // A missing local-call admission must not execute the
                            // hidden global definition. Do not invent a callable
                            // value/alias rule from a Function type or an AST shape.
                            self.facts.errors.push((diagnostic_span,
                            "가려진 지역 호출의 형식이 확인되지 않아 바깥 정의로 실행할 수 없습니다"));
                            BindingState::Unresolved(Issue::CallableValueTypeNotSupplied)
                        }
                    } else if candidates.is_empty() {
                        BindingState::Unresolved(Issue::CallOwnerNotSupplied)
                    } else {
                        BindingState::Candidates(candidates)
                    }
                } else {
                    BindingState::Unresolved(Issue::CallOwnerNotSupplied)
                };
                if let Some(span) = name_span {
                    self.note(expr.id, span, state);
                }
            }
            ExprKind::Infix { left, right, .. } => {
                self.expr(left);
                self.expr(right);
            }
            ExprKind::Suffix { value, .. } | ExprKind::Nuance { expr: value, .. } => {
                self.expr(value)
            }
            ExprKind::FieldAccess { target, .. } => self.expr(target),
            ExprKind::SelectionPlace { target, index } => {
                self.expr(target);
                self.expr(index);
            }
            ExprKind::ProductConstruction { fields, .. } => {
                for f in fields {
                    self.expr(&f.expr);
                }
            }
            ExprKind::CollectionConstructor { elements }
            | ExprKind::TupleConstructor { elements } => {
                for e in elements {
                    self.expr(e);
                }
            }
            ExprKind::Pack { fields } => {
                for (_, e) in fields {
                    self.expr(e);
                }
            }
            ExprKind::Pipe { stages } => {
                for e in stages {
                    self.expr(e);
                }
            }
            ExprKind::Eval { thunk, .. } => self.expr(thunk),
            ExprKind::Thunk(body) => self.body(body),
            ExprKind::Literal(_) => {}
            _ => self.opaque(expr.span),
        }
    }
}
pub(crate) fn resolve(program: &CanonProgram, tokens: &[Token]) -> NameResolutionFacts {
    let source_hash = *blake3::hash(program.origin.source.as_bytes()).as_bytes();
    let mut hash = blake3::Hasher::new();
    hash.update(&(program.origin.file_path.len() as u64).to_be_bytes());
    hash.update(program.origin.file_path.as_bytes());
    hash.update(&source_hash);
    let facts = NameResolutionFacts {
        revision: *hash.finalize().as_bytes(),
        source_hash,
        path: program.origin.file_path.clone(),
        definitions: vec![],
        uses: vec![],
        errors: vec![],
    };
    let mut r = Resolver {
        program,
        tokens: {
            let mut names = tokens
                .iter()
                .filter(|t| {
                    matches!(t.kind, TokenKind::Ident(_))
                        && program.origin.source.get(t.span.start..t.span.end)
                            == Some(t.raw.as_str())
                })
                .collect::<Vec<_>>();
            names.sort_by_key(|t| (t.span.start, t.span.end));
            names.dedup_by_key(|t| (t.span.start, t.span.end));
            names
        },
        use_index: HashMap::new(),
        facts,
        scopes: vec![Scope::new()],
        uncertain: vec![HashSet::new()],
        callables: HashMap::new(),
        unknown_global: HashSet::new(),
        self_name: None,
        setup_resources: HashMap::new(),
        phase_scope: false,
    };
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if !crate::ordinary_callable::is_v1_ordinary_callable(seed) {
            continue;
        }
        let end = seed.body.as_ref().map_or(seed.span.end, |b| b.span.start);
        let start = seed
            .params
            .iter()
            .map(|p| p.span.end)
            .max()
            .unwrap_or(seed.span.start)
            .max(seed.span.start);
        let at = r.declaration_span(&seed.canonical_name, Span::new(start, end), false);
        let key = r.add(
            &seed.canonical_name,
            DefinitionKind::Callable,
            seed.span,
            at,
            false,
        );
        r.callables.insert(seed.id, key);
        r.scopes[0]
            .entry(seed.canonical_name.clone())
            .or_default()
            .push(key);
        if let Some(at) = at {
            r.note(seed.id, at, BindingState::Final(key));
        }
    }
    // A declaration owned by another scope subsystem cannot silently turn
    // a same-spelling callable reference into an exact ordinary binding.
    fn collect(stmt: &Stmt, names: &mut HashSet<String>) {
        match stmt {
            Stmt::DeclBlock { items, .. } => {
                for i in items {
                    names.insert(i.name.clone());
                }
            }
            Stmt::DerivedBinding { name, .. } | Stmt::SumDefinition { name, .. } => {
                names.insert(name.clone());
            }
            Stmt::ForEach { item, body, .. } => {
                names.insert(item.clone());
                for s in &body.stmts {
                    collect(s, names);
                }
            }
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                for s in &then_body.stmts {
                    collect(s, names);
                }
                if let Some(b) = else_body {
                    for s in &b.stmts {
                        collect(s, names);
                    }
                }
            }
            _ => {}
        }
    }
    let mut setup_blocks = Vec::new();
    let mut setup_counts: HashMap<String, usize> = HashMap::new();
    for TopLevelItem::SeedDef(seed) in &program.items {
        if let Some(body) = &seed.body {
            for statement in &body.stmts {
                if seed.is_relocated_top_level_setup(statement) {
                    if let Stmt::DeclBlock { id, items, .. } = statement {
                        if !items.is_empty() && items.iter().all(|item|
                            crate::ordinary_callable::closed_non_callable_type(&item.type_ref)
                            && matches!(item.value.as_ref().map(|v| &v.kind), Some(ExprKind::Literal(_)))
                            && r.declaration_span(&item.name, item.span, true).is_some()) {
                            for item in items { *setup_counts.entry(item.name.clone()).or_default() += 1; }
                            setup_blocks.push((*id, items));
                        }
                    }
                }
            }
        }
    }
    for s in &program.root_stmts {
        collect(s, &mut r.unknown_global);
    }
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(b) = &seed.body {
            for s in &b.stmts {
                let candidate = matches!(s, Stmt::DeclBlock { id, .. }
                    if setup_blocks.iter().any(|(candidate, _)| candidate == id));
                if !candidate { collect(s, &mut r.unknown_global); }
            }
        }
    }
    for (_, items) in setup_blocks {
        let blocked = items.iter().any(|item|
            setup_counts.get(&item.name) != Some(&1)
            || r.unknown_global.contains(&item.name)
            || r.scopes[0].contains_key(&item.name)
            || program.root_stmts.iter().any(|s|
                matches!(s, Stmt::Definition { name, .. } if name == &item.name)));
        if blocked {
            r.unknown_global.extend(items.iter().map(|item| item.name.clone()));
            continue;
        }
        for item in items {
            let at = r.declaration_span(&item.name, item.span, true).unwrap();
            let key = r.add(&item.name, DefinitionKind::Local, item.span, Some(at), true);
            r.facts.definitions[key.ordinal].owner_scope = DefinitionOwnerScope::RelocatedTopLevelSetup;
            r.scopes[0].insert(item.name.clone(), vec![key]);
            r.setup_resources.insert(item.id, key);
            r.note(item.id, at, BindingState::Final(key));
        }
    }
    r.statements(&program.root_stmts);
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if !r.callables.contains_key(&seed.id) {
            continue;
        }
        r.scopes.push(Scope::new());
        r.uncertain.push(HashSet::new());
        r.self_name = Some(seed.canonical_name.clone());
        // Result-place identity is local to this invocation, not the callable.
        let callable = r.callables[&seed.id];
        let at = r.facts.definitions[callable.ordinal].name_span;
        let result = r.add(
            &seed.canonical_name,
            DefinitionKind::ResultPlace,
            seed.span,
            at,
            false,
        );
        r.facts.definitions[result.ordinal].owner_callable = Some(callable);
        r.scopes
            .last_mut()
            .unwrap()
            .insert(seed.canonical_name.clone(), vec![result]);
        for param in &seed.params {
            r.local(
                param.id,
                &param.pin_name,
                param.span,
                &param.type_ref,
                None,
                DefinitionKind::Parameter,
            );
        }
        if let Some(body) = &seed.body {
            r.statements(&body.stmts);
        }
        r.self_name = None;
        r.scopes.pop();
        r.uncertain.pop();
    }
    // Type/field names and owner-specific syntax remain explicitly unresolved,
    // rather than being omitted and later guessed by an editor consumer.
    let uncovered = r
        .tokens
        .iter()
        .filter(|t| {
            r.source_name_token(t) && !r.use_index.contains_key(&(t.span.start, t.span.end))
        })
        .map(|t| Span::new(t.span.start, t.span.end))
        .collect::<Vec<_>>();
    for span in uncovered {
        r.note(0, span, BindingState::Unresolved(Issue::OwnerSpecificScope));
    }
    r.facts.uses.sort_by_key(|u| (u.span.start, u.span.end));
    r.facts
}
