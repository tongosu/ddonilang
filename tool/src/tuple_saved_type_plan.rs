//! Private compilation-local Tuple persistence evidence. No stored NodeId,
//! ambient registry lookup, runtime value-shape inference or JSON import.
use std::collections::{BTreeMap, BTreeSet, HashSet};
use ddonirang_core::platform::RecordFieldType;
use ddonirang_lang::{CanonProgram, Expr, ExprKind, Literal, NodeId, Stmt, TensorAxisSelector, TypeRef};
use ddonirang_lang::runtime::{record_type_identity, RecordSchemas};

fn resolve_nominal_type(kind:&RecordFieldType,nominals:&BTreeMap<String,(u8,[u8;32])>)->RecordFieldType {
    enum Job<'a>{Type(&'a RecordFieldType),Tuple(usize),Applied(&'a str,usize),Record(Vec<String>)}
    let mut jobs=vec![Job::Type(kind)];let mut built=Vec::new();
    while let Some(job)=jobs.pop(){match job {
        Job::Type(RecordFieldType::Named{name})=>built.push(if let Some((category,id))=nominals.get(name){
            RecordFieldType::ResolvedNominalV1{category:*category,name_type_identity:*id}
        }else{RecordFieldType::Named{name:name.clone()}}),
        Job::Type(RecordFieldType::ResolvedNominalV1{category,name_type_identity})=>built.push(RecordFieldType::ResolvedNominalV1{category:*category,name_type_identity:*name_type_identity}),
        Job::Type(RecordFieldType::Tuple{items})=>{jobs.push(Job::Tuple(items.len()));jobs.extend(items.iter().rev().map(Job::Type));},
        Job::Type(RecordFieldType::Applied{name,args})=>{jobs.push(Job::Applied(name,args.len()));jobs.extend(args.iter().rev().map(Job::Type));},
        Job::Type(RecordFieldType::Record{fields})=>{jobs.push(Job::Record(fields.keys().cloned().collect()));jobs.extend(fields.values().rev().map(Job::Type));},
        Job::Tuple(n)=>{let items=built.split_off(built.len()-n);built.push(RecordFieldType::Tuple{items});},
        Job::Applied(name,n)=>{let args=built.split_off(built.len()-n);built.push(RecordFieldType::Applied{name:name.into(),args});},
        Job::Record(keys)=>{let values=built.split_off(built.len()-keys.len());built.push(RecordFieldType::Record{fields:keys.into_iter().zip(values).collect()});},
    }}
    built.pop().expect("complete declaration type")
}

#[derive(Clone, Debug)]
struct Binding { root: NodeId, kind: Option<RecordFieldType> }

#[derive(Clone, Default, Debug)]
pub(crate) struct TupleSavedTypePlan {
    rows: BTreeMap<String, Binding>,
    writes: BTreeMap<NodeId, (String, NodeId)>,
    rejected: BTreeSet<String>,
}

impl TupleSavedTypePlan {
    pub(crate) fn compile(program: &CanonProgram, entry: Option<&str>, entry_first: bool, world_decls: &HashSet<String>, records: &RecordSchemas,
        numeric: Option<&crate::numeric_constraint_plan::NumericConstraintPlan>,nominals:&BTreeMap<String,(u8,[u8;32])>) -> Self {
        let mut result = Self::default();
        let mut bindings = BTreeMap::<String, Binding>::new();
        // One admitted numeric producer feeds execution and persistence. A
        // missing position owner is not re-inferred from spelling or a value.
        let expression_type = |expr: &Expr, bindings: &BTreeMap<String, Binding>, calls: &BTreeMap<String, Option<RecordFieldType>>| {
            let kind=if let Some(plan) = numeric { plan.position_types.get(&expr.id).cloned() }
                else { expression_type(expr, bindings, calls) };
            kind.map(|kind|resolve_nominal_type(&kind,nominals))
        };
        // Same admitted compilation, declaration-owned result TypeRef only.
        // A successful call validates its pins/result in the common evaluator;
        // neither runtime shape nor a coarse Number label issues this type.
        let mut call_results = BTreeMap::<String, Option<RecordFieldType>>::new();
        for item in &program.items {
            let ddonirang_lang::TopLevelItem::SeedDef(seed) = item;
            let kind = seed.result_type.as_ref()
                .and_then(|kind| record_type_identity(kind, records).ok()).map(|kind|resolve_nominal_type(&kind,nominals));
            for name in seed.callable_surface_names() {
                // eval_call dispatches sealed builtins before ordinary source
                // definitions. A same-spelled declaration cannot certify the
                // result of a different callable owner.
                let canonical = ddonirang_lang::stdlib::canonicalize_stdlib_alias(&name);
                let ordinary = crate::work_cost::builtin_work_cost_rule(canonical)
                    .is_ok_and(|rule| rule.is_none());
                let kind = if ordinary { kind.clone() } else { None };
                if call_results.contains_key(&name) { call_results.insert(name, None); }
                else { call_results.insert(name, kind); }
            }
        }
        let entry_body = program.items.iter().find_map(|item| {
            let ddonirang_lang::TopLevelItem::SeedDef(seed) = item;
            (Some(seed.canonical_name.as_str()) == entry).then_some(seed.body.as_ref()).flatten()
        });
        let mut statements = Vec::new();
        // Reuse the parser's exact relocation ownership, not a seed-name or
        // value-shape guess. Named entries still initialize authored 채비.
        for item in &program.items {
            let ddonirang_lang::TopLevelItem::SeedDef(seed)=item;
            if Some(seed.canonical_name.as_str())==entry {continue;}
            if let Some(body)=&seed.body {
                statements.extend(body.stmts.iter().filter(|stmt|seed.is_relocated_top_level_setup(stmt)));
            }
        }
        if entry_first { statements.extend(entry_body.into_iter().flat_map(|body| body.stmts.iter())); }
        statements.extend(program.root_stmts.iter());
        if !entry_first { statements.extend(entry_body.into_iter().flat_map(|body| body.stmts.iter())); }
        for stmt in statements {
            match stmt {
                Stmt::DeclBlock { items, .. } => for item in items {
                    let kind = if matches!(item.type_ref, TypeRef::Infer) {
                        item.value.as_ref().and_then(|expr| expression_type(expr, &bindings, &call_results))
                    } else { record_type_identity(&item.type_ref, records).ok().map(|kind|resolve_nominal_type(&kind,nominals)) };
                    if bindings.contains_key(&item.name) { result.rejected.insert(item.name.clone()); }
                    let binding = Binding { root: item.id, kind };
                    bindings.insert(item.name.clone(), binding.clone());
                    // The existing DeclBlock evaluator sends every initialized
                    // item through set_resource, including the grouped items
                    // whose names are not in the lifecycle-prefix name set.
                    if item.value.is_some() { result.write(item.id, &item.name, &binding); }
                    if let Some(expr) = &item.value { reject_nested_writes(expr, &mut result.rejected); }
                },
                Stmt::Definition { id, name, type_ref, value, .. } => {
                    let kind = if matches!(type_ref, TypeRef::Infer) { expression_type(value, &bindings, &call_results) }
                        else { record_type_identity(type_ref, records).ok().map(|kind|resolve_nominal_type(&kind,nominals)) };
                    if bindings.contains_key(name) { result.rejected.insert(name.clone()); }
                    let binding = Binding { root: *id, kind };
                    bindings.insert(name.clone(), binding.clone());
                    if world_decls.contains(name) { result.write(*id, name, &binding); }
                    reject_nested_writes(value, &mut result.rejected);
                }
                Stmt::Mutate { id, target, value, .. } => {
                    let Some(name) = root_name(target) else { continue; };
                    let expression = expression_type(value, &bindings, &call_results);
                    if !bindings.contains_key(name) {
                        bindings.insert(name.to_owned(), Binding { root: *id, kind: expression.clone() });
                    }
                    let binding = &bindings[name];
                    // A replacement preserves the root descriptor only when
                    // the selected positions have exactly the RHS descriptor.
                    let expected = expression_type(target, &bindings, &call_results);
                    if expression.is_none() || expected != expression { result.rejected.insert(name.to_owned()); }
                    result.write(*id, name, binding);
                    reject_nested_writes(value, &mut result.rejected);
                }
                _ => reject_statement_writes(stmt, &mut result.rejected),
            }
        }
        // This first source supplier admits straight-line root bindings only.
        // A nested/dynamic write cannot be ignored when choosing a root row.
        for item in &program.items {
            let ddonirang_lang::TopLevelItem::SeedDef(seed) = item;
            if Some(seed.canonical_name.as_str()) == entry { continue; }
            if let Some(body) = &seed.body {
                for stmt in &body.stmts {
                    if !seed.is_relocated_top_level_setup(stmt){reject_statement_writes(stmt, &mut result.rejected);}
                }
            }
        }
        result
    }

    fn write(&mut self, site: NodeId, name: &str, binding: &Binding) {
        if let Some(previous) = self.rows.get(name) {
            if previous.root != binding.root || previous.kind != binding.kind { self.rejected.insert(name.to_owned()); }
        } else { self.rows.insert(name.to_owned(), binding.clone()); }
        if let Some(previous) = self.writes.insert(site, (name.to_owned(), binding.root)) {
            if previous != (name.to_owned(), binding.root) { self.rejected.insert(name.to_owned()); }
        }
    }

    pub(crate) fn expected(&self, name: &str) -> Option<&RecordFieldType> {
        if self.rejected.contains(name) || self.rejected.contains("*") { return None; }
        self.rows.get(name)?.kind.as_ref()
    }

    pub(crate) fn for_write(&self, site: NodeId, name: &str) -> Option<&RecordFieldType> {
        let (tag, root) = self.writes.get(&site)?;
        let binding = self.rows.get(name)?;
        if tag != name || *root != binding.root { return None; }
        self.expected(name)
    }
}

fn root_name(mut expr: &Expr) -> Option<&str> {
    loop { match &expr.kind {
        ExprKind::Var(name) => return Some(name),
        ExprKind::SelectionPlace { target, .. } => expr = target,
        ExprKind::MultiAxisSelection(selection) => expr = &selection.target,
        _ => return None,
    } }
}

pub(crate) fn expression_type_from_owners(expr: &Expr, bindings: BTreeMap<String, RecordFieldType>, call_results: &BTreeMap<String, Option<RecordFieldType>>) -> Option<RecordFieldType> {
    let bindings = bindings.into_iter().map(|(name, kind)| (name, Binding { root: 0, kind: Some(kind) })).collect();
    expression_type_inner(expr, &bindings, call_results, true)
}

fn expression_type(expr: &Expr, bindings: &BTreeMap<String, Binding>, call_results: &BTreeMap<String, Option<RecordFieldType>>) -> Option<RecordFieldType> {
    expression_type_inner(expr, bindings, call_results, false)
}

fn expression_type_inner(expr: &Expr, bindings: &BTreeMap<String, Binding>, call_results: &BTreeMap<String, Option<RecordFieldType>>, preserve_negated_owner: bool) -> Option<RecordFieldType> {
    enum Job<'a> { Expr(&'a Expr), Tuple(usize), Infix(&'a str), Select(&'a [TensorAxisSelector]), Index(&'a Expr) }
    let mut pending = vec![Job::Expr(expr)];
    let mut built = Vec::<RecordFieldType>::new();
    while let Some(job) = pending.pop() {
        match job {
            Job::Expr(expr) => {
                if preserve_negated_owner {
                    if let Some(operand) = expr.source_negated_operand() {
                        pending.push(Job::Expr(operand));
                        continue;
                    }
                }
                match &expr.kind {
                ExprKind::Literal(Literal::Fixed64(_)) => {
                    // The parser owns spelling and default type. Equal raw
                    // values (1 and 1.0) do not issue the same child identity.
                    let TypeRef::Named(name) = expr.numeric_literal_source.as_ref()?.default_type() else { return None; };
                    built.push(RecordFieldType::Named { name });
                }
                ExprKind::Literal(Literal::String(_)) => built.push(RecordFieldType::Named { name: "글".into() }),
                ExprKind::Literal(Literal::Bool(_)) => built.push(RecordFieldType::Named { name: "참거짓".into() }),
                ExprKind::Var(name) if matches!(name.as_str(), "참" | "거짓") && !bindings.contains_key(name) =>
                    built.push(RecordFieldType::Named { name: "참거짓".into() }),
                ExprKind::Var(name) => built.push(bindings.get(name)?.kind.clone()?),
                ExprKind::Infix { left, op, right } if matches!(op.as_str(), "+" | "-" | "*" | "/") => {
                    pending.push(Job::Infix(op));
                    pending.push(Job::Expr(right)); pending.push(Job::Expr(left));
                }
                ExprKind::Call { func, .. } if !bindings.contains_key(func) => {
                    let kind=crate::numeric_constraint_plan::sealed_numeric_constructor_result(func)
                        .and_then(|kind|record_type_identity(&kind,&RecordSchemas::new()).ok())
                        .or_else(||call_results.get(func).cloned().flatten())?;
                    built.push(kind);
                }
                ExprKind::TupleConstructor { elements } => {
                    pending.push(Job::Tuple(elements.len()));
                    pending.extend(elements.iter().rev().map(Job::Expr));
                }
                ExprKind::MultiAxisSelection(selection) if selection.selectors.len() == 1 => {
                    pending.push(Job::Select(&selection.selectors));
                    pending.push(Job::Expr(&selection.target));
                }
                ExprKind::SelectionPlace { target, index } => {
                    pending.push(Job::Index(index)); pending.push(Job::Expr(target));
                }
                _ => return None,
                }
            }
            Job::Tuple(n) => {
                let items = built.split_off(built.len().checked_sub(n)?);
                built.push(RecordFieldType::Tuple { items });
            }
            Job::Infix(op) => {
                let right = built.pop()?; let left = built.pop()?;
                if let (RecordFieldType::Named{name:a},RecordFieldType::Named{name:b})=(&left,&right) {
                    if let Some(TypeRef::Named(name))=crate::numeric_constraint_plan::numeric_operation_result_type(&TypeRef::Named(a.clone()),&TypeRef::Named(b.clone()),op) {
                        built.push(RecordFieldType::Named{name});continue;
                    }
                }
                // Mixed literals / integer-to-approximate admission remain
                // pending. Do not prohibit existing execution or certify an
                // unresolved result for persistence with a guessed type.
                if left != right { return None; }
                let RecordFieldType::Named { name } = &left else { return None; };
                if !matches!(name.as_str(), "바른수" | "셈수" | "나눔수")
                    && !(op == "+" && name == "글") { return None; }
                built.push(left);
            }
            Job::Select(selectors) => {
                let mut kind = built.pop()?;
                let RecordFieldType::Tuple { items } = &mut kind else { return None; };
                match &selectors[0] {
                    TensorAxisSelector::Range { start, end } => {
                        let (start, end) = (usize::try_from(*start).ok()?, usize::try_from(*end).ok()?);
                        if start > end { return None; }
                        built.push(RecordFieldType::Tuple { items: items.get(start..end)?.to_vec() });
                    }
                    TensorAxisSelector::Index(index) => {
                        let position = crate::source_analysis::static_tuple_index(index)?;
                        built.push(items.get(position)?.clone());
                    }
                    TensorAxisSelector::Full => return None,
                }
            }
            Job::Index(index) => {
                let kind = built.pop()?;
                let RecordFieldType::Tuple { items } = &kind else { return None; };
                let position = crate::source_analysis::static_tuple_index(index)?;
                built.push(items.get(position)?.clone());
            }
        }
    }
    if built.len() != 1 { return None; }
    built.pop()
}

enum Scan<'a> { Expr(&'a Expr), Stmt(&'a Stmt) }
fn reject_nested_writes(expr: &Expr, rejected: &mut BTreeSet<String>) { reject_writes(Scan::Expr(expr), rejected); }
fn reject_statement_writes(stmt: &Stmt, rejected: &mut BTreeSet<String>) { reject_writes(Scan::Stmt(stmt), rejected); }
fn reject_writes(root: Scan<'_>, rejected: &mut BTreeSet<String>) {
    let mut pending = vec![root];
    while let Some(job) = pending.pop() {
        let stmt = match job {
            Scan::Expr(expr) => {
                match &expr.kind {
                    ExprKind::Thunk(body) => pending.extend(body.stmts.iter().map(Scan::Stmt)),
                    ExprKind::TupleConstructor { elements } | ExprKind::CollectionConstructor { elements } | ExprKind::Pipe { stages: elements } => pending.extend(elements.iter().map(Scan::Expr)),
                    ExprKind::Call { args, .. } | ExprKind::ProductConstruction { fields: args, .. } => pending.extend(args.iter().map(|arg| Scan::Expr(&arg.expr))),
                    ExprKind::Infix { left, right, .. } => { pending.push(Scan::Expr(left)); pending.push(Scan::Expr(right)); }
                    ExprKind::Eval { thunk, .. } | ExprKind::Suffix { value: thunk, .. } | ExprKind::Nuance { expr: thunk, .. } | ExprKind::SeedLiteral { body: thunk, .. } => pending.push(Scan::Expr(thunk)),
                    ExprKind::MultiAxisSelection(selection) => pending.extend(selection.children().map(Scan::Expr)),
                    ExprKind::SelectionPlace { target, index } => { pending.push(Scan::Expr(target)); pending.push(Scan::Expr(index)); }
                    ExprKind::FieldAccess { target, .. } => pending.push(Scan::Expr(target)),
                    ExprKind::Pack { fields } => pending.extend(fields.iter().map(|(_, expr)| Scan::Expr(expr))),
                    ExprKind::Tensor(provider) => pending.push(Scan::Expr(provider.input())),
                    ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => pending.extend(inject.iter().map(|(_, expr)| Scan::Expr(expr))),
                    ExprKind::Template(_) | ExprKind::Formula(_) => {},
                    ExprKind::Literal(_) | ExprKind::Var(_) | ExprKind::FlowValue => {},
                    // Unmapped executable expressions cannot certify that
                    // another resource's Tuple write set is complete.
                    _ => { rejected.insert("*".into()); },
                }
                continue;
            }
            Scan::Stmt(stmt) => stmt,
        };
        match stmt {
            Stmt::Mutate { target, value, .. } => {
                if let Some(name) = root_name(target) { rejected.insert(name.to_owned()); }
                pending.push(Scan::Expr(value));
            }
            Stmt::ResultFlow { destination, producer, .. } => {
                if let Some(name) = root_name(destination) { rejected.insert(name.to_owned()); }
                pending.push(Scan::Expr(producer));
            }
            Stmt::If { condition, then_body, else_body, .. } => {
                pending.push(Scan::Expr(condition));
                pending.extend(then_body.stmts.iter().map(Scan::Stmt));
                if let Some(body) = else_body { pending.extend(body.stmts.iter().map(Scan::Stmt)); }
            }
            Stmt::Repeat { body, .. } | Stmt::While { body, .. } | Stmt::ForEach { body, .. }
            | Stmt::Quantifier { body, .. } | Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. }
            | Stmt::HookWhenBecomes { body, .. } | Stmt::HookWhile { body, .. } | Stmt::Guard { body, .. }
            | Stmt::Try { body, .. } | Stmt::Receive { body, .. } => pending.extend(body.stmts.iter().map(Scan::Stmt)),
            Stmt::Choose { branches, else_body, .. } => {
                pending.extend(else_body.stmts.iter().map(Scan::Stmt));
                for branch in branches { pending.extend(branch.body.stmts.iter().map(Scan::Stmt)); }
            }
            Stmt::Contract { then_body, else_body, .. } => {
                pending.extend(else_body.stmts.iter().map(Scan::Stmt));
                if let Some(body) = then_body { pending.extend(body.stmts.iter().map(Scan::Stmt)); }
            }
            Stmt::Definition { value, .. } | Stmt::Expr { expr: value, .. }
            | Stmt::Show { expr: value, .. } | Stmt::Inspect { expr: value, .. }
            | Stmt::Return { value, .. } => pending.push(Scan::Expr(value)),
            Stmt::DeclBlock { items, .. } => for item in items {
                if let Some(value) = &item.value {
                    rejected.insert(item.name.clone());
                    pending.push(Scan::Expr(value));
                }
            },
            Stmt::PublicObservation { fields, .. } => pending.extend(fields.iter().map(|field| Scan::Expr(&field.value))),
            Stmt::Break { .. } | Stmt::ContinueLoop { .. } | Stmt::Pragma { .. } | Stmt::MetaBlock { .. } => {},
            _ => { rejected.insert("*".into()); },
        }
    }
}
