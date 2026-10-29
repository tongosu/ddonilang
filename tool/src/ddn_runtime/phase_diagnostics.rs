//! C33: diagnostic-only provenance. Never evaluates source or mutates canon.
use std::collections::{BTreeMap, BTreeSet};
use ddonirang_lang::ast::{CanonProgram, DeclKind, Expr, ExprKind, Literal, Stmt, TopLevelItem};
use super::DdnParseWarning;

#[derive(Clone, PartialEq, Eq)]
enum Domain { Phase, Text(BTreeSet<String>), Unknown }
#[derive(Clone)]
struct Binding { domain: Domain, mutable: bool }
type Env = BTreeMap<String, Binding>;

fn invalidate(env: &mut Env) {
    for binding in env.values_mut().filter(|b| b.mutable) { binding.domain = Domain::Unknown; }
}

fn joined(a: &Domain, b: &Domain) -> Domain {
    match (a, b) {
        (Domain::Phase, Domain::Phase) => Domain::Phase,
        (Domain::Text(a), Domain::Text(b)) => Domain::Text(a.union(b).cloned().collect()),
        _ => Domain::Unknown,
    }
}

struct Collector<'a> { program: &'a CanonProgram, warnings: Vec<DdnParseWarning> }
impl Collector<'_> {
    fn expression(&mut self, expr: &Expr, env: &mut Env) -> Domain {
        match &expr.kind {
            ExprKind::Literal(Literal::String(text)) => Domain::Text([text.clone()].into_iter().collect()),
            ExprKind::Var(name) => env.get(name).map(|b| b.domain.clone()).unwrap_or(Domain::Unknown),
            ExprKind::Call { args, .. } => {
                for arg in args { self.expression(&arg.expr, env); }
                let admitted = self.program.origin.manipulation_phase_calls.get(&expr.id)
                    .is_some_and(|b| b.definition == "B1.DEF.INPUT_PHASE.R1"
                        && b.ordinal_pin == "B1.PIN.INPUT_PHASE.ORDINAL.R1");
                if admitted { Domain::Phase } else { invalidate(env); Domain::Unknown }
            }
            ExprKind::Infix { left, op, right } => {
                let a = self.expression(left, env);
                let b = self.expression(right, env);
                if matches!(op.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") {
                    let bad = match (&a, &b) {
                        (Domain::Phase, Domain::Text(texts)) if outside(texts) => Some(right.span),
                        (Domain::Text(texts), Domain::Phase) if outside(texts) => Some(left.span),
                        _ => None,
                    };
                    if let Some(span) = bad {
                        self.warnings.push(DdnParseWarning {
                            code: "W_INPUT_PHASE_COMPARE_OUTSIDE_DOMAIN".into(),
                            message: "조작단계의 네 반환값(설정·시작·이동·끝) 밖의 글과 비교합니다.".into(),
                            span_start: span.start, span_end: span.end,
                        });
                    }
                }
                Domain::Unknown
            }
            ExprKind::Thunk(body) => {
                let mut local = env.clone(); invalidate(&mut local);
                self.statements(&body.stmts, &mut local); Domain::Unknown
            }
            ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements }
                | ExprKind::Pipe { stages: elements } => {
                for item in elements { self.expression(item, env); } Domain::Unknown
            }
            ExprKind::ProductConstruction { fields, .. } => {
                for field in fields { self.expression(&field.expr, env); } Domain::Unknown
            }
            ExprKind::FieldAccess { target, .. } => { self.expression(target, env); Domain::Unknown }
            ExprKind::MultiAxisSelection(selection) => {
                for child in selection.children() { self.expression(child, env); } Domain::Unknown
            }
            ExprKind::SelectionPlace { target, index } => {
                self.expression(target, env); self.expression(index, env); Domain::Unknown
            }
            ExprKind::Suffix { value, .. } | ExprKind::Nuance { expr: value, .. } => self.expression(value, env),
            ExprKind::Eval { thunk, .. } => { self.expression(thunk, env); invalidate(env); Domain::Unknown }
            ExprKind::SeedLiteral { param, body } => {
                let mut local = env.clone(); invalidate(&mut local); local.remove(param);
                self.expression(body, &mut local); Domain::Unknown
            }
            ExprKind::Pack { fields } => {
                for (_, field) in fields { self.expression(field, env); } Domain::Unknown
            }
            _ => Domain::Unknown,
        }
    }

    fn statements(&mut self, stmts: &[Stmt], env: &mut Env) {
        for stmt in stmts {
            match stmt {
                Stmt::Definition { name, value, .. } => {
                    let domain = self.expression(value, env);
                    env.insert(name.clone(), Binding { domain, mutable: false });
                }
                Stmt::DeclBlock { items, .. } => for item in items {
                    let domain = item.value.as_ref().map(|e| self.expression(e, env)).unwrap_or(Domain::Unknown);
                    env.insert(item.name.clone(), Binding { domain, mutable: item.kind == DeclKind::Gureut });
                },
                Stmt::Mutate { target, value, .. } => {
                    self.expression(value, env); self.expression(target, env);
                    // Reassignment deliberately drops refinement, even for a phase RHS.
                    if let ExprKind::Var(name) = &target.kind {
                        env.insert(name.clone(), Binding { domain: Domain::Unknown, mutable: true });
                    } else { invalidate(env); }
                }
                Stmt::ResultFlow { producer, destination, .. } => {
                    self.expression(producer, env); self.expression(destination, env); invalidate(env);
                }
                Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. }
                    | Stmt::Return { value: expr, .. } => { self.expression(expr, env); }
                Stmt::If { condition, then_body, else_body, .. } => {
                    self.expression(condition, env);
                    let mut a = env.clone(); let mut b = env.clone();
                    self.statements(&then_body.stmts, &mut a);
                    if let Some(body) = else_body { self.statements(&body.stmts, &mut b); }
                    for (name, binding) in env.iter_mut() {
                        binding.domain = match (a.get(name), b.get(name)) {
                            (Some(a), Some(b)) => joined(&a.domain, &b.domain), _ => Domain::Unknown,
                        };
                    }
                }
                Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. }
                    | Stmt::Repeat { body, .. } | Stmt::Quantifier { body, .. } => {
                    let mut local = env.clone(); invalidate(&mut local);
                    self.statements(&body.stmts, &mut local); invalidate(env);
                }
                Stmt::ForEach { item, iterable, body, .. } => {
                    self.expression(iterable, env);
                    let mut local = env.clone(); invalidate(&mut local); local.remove(item);
                    self.statements(&body.stmts, &mut local); invalidate(env);
                }
                Stmt::While { condition, body, .. } | Stmt::HookWhile { condition, body, .. }
                    | Stmt::HookWhenBecomes { condition, body, .. } | Stmt::Guard { condition, body, .. } => {
                    let mut local = env.clone(); invalidate(&mut local);
                    self.expression(condition, &mut local); self.statements(&body.stmts, &mut local); invalidate(env);
                }
                Stmt::Choose { branches, else_body, .. } => {
                    for branch in branches {
                        let mut local = env.clone(); invalidate(&mut local);
                        self.expression(&branch.condition, &mut local); self.statements(&branch.body.stmts, &mut local);
                    }
                    let mut local = env.clone(); invalidate(&mut local);
                    self.statements(&else_body.stmts, &mut local); invalidate(env);
                }
                Stmt::Try { action, body, .. } => {
                    self.expression(action, env); let mut local = env.clone(); invalidate(&mut local);
                    self.statements(&body.stmts, &mut local); invalidate(env);
                }
                Stmt::Receive { sender, condition, binding, body, .. } => {
                    if let Some(expr) = sender { self.expression(expr, env); }
                    let mut local = env.clone(); invalidate(&mut local);
                    if let Some(name) = binding { local.remove(name); }
                    if let Some(expr) = condition { self.expression(expr, &mut local); }
                    self.statements(&body.stmts, &mut local); invalidate(env);
                }
                Stmt::Send { sender, payload, receiver, .. } => {
                    if let Some(expr) = sender { self.expression(expr, env); }
                    self.expression(payload, env); self.expression(receiver, env); invalidate(env);
                }
                Stmt::PublicObservation { fields, .. } => for field in fields { self.expression(&field.value, env); },
                _ => { invalidate(env); }
            }
        }
    }
}

fn outside(texts: &BTreeSet<String>) -> bool {
    texts.iter().any(|s| !matches!(s.as_str(), "설정" | "시작" | "이동" | "끝"))
}

pub(super) fn collect(program: &CanonProgram) -> Vec<DdnParseWarning> {
    let mut collector = Collector { program, warnings: Vec::new() };
    let mut root = Env::new();
    collector.statements(&program.root_stmts, &mut root);
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            let mut local = root.clone(); invalidate(&mut local);
            for param in &seed.params { local.remove(&param.pin_name); }
            collector.statements(&body.stmts, &mut local);
        }
    }
    collector.warnings.sort_by_key(|w| (w.span_start, w.span_end));
    collector.warnings.dedup_by(|a, b| a.span_start == b.span_start && a.span_end == b.span_end);
    collector.warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    fn warnings(source: &str) -> Vec<DdnParseWarning> {
        let program = ddonirang_lang::parse_v25(source, "phase.ddn").unwrap();
        collect(&program)
    }
    #[test]
    fn phase_warning_tracks_binding_and_exact_opposite_span() {
        for source in [
            "((순서=0) 조작단계 == \"완료\") 보여주기.",
            "(\"완료\" != (순서=0) 조작단계) 보여주기.",
            "국면 := (순서=0) 조작단계. 복사 := 국면. (복사 == \"끝 \") 보여주기.",
        ] {
            let found = warnings(source); assert_eq!(found.len(), 1, "{source}");
            assert_eq!(found[0].code, "W_INPUT_PHASE_COMPARE_OUTSIDE_DOMAIN");
            assert!(source[found[0].span_start..found[0].span_end].contains('"'));
        }
    }
    #[test]
    fn phase_warning_does_not_certify_unknown_or_reassigned_values() {
        for source in [
            "((순서=0) 조작단계 == \"끝\") 보여주기.",
            "채비 { 국면:글 := (순서=0) 조작단계. }. 국면 <- \"완료\". (국면 == \"완료\") 보여주기.",
            "(\"일반 글\" == \"완료\") 보여주기.",
        ] { assert!(warnings(source).is_empty(), "{source}"); }
    }
    #[test]
    fn phase_warning_is_advisory_in_shared_program_admission() {
        let source="(매마디)마다 { ((순서=0) 조작단계 == \"중간\") 보여주기. }.";
        let program=super::super::DdnProgram::from_v25_source(source,"phase-warning.ddn").unwrap();
        let found=program.parse_warnings().iter().filter(|w|w.code=="W_INPUT_PHASE_COMPARE_OUTSIDE_DOMAIN").collect::<Vec<_>>();
        assert_eq!(found.len(),1);
        assert_eq!(&source[found[0].span_start..found[0].span_end],"\"중간\"");
    }
    #[test]
    fn phase_warning_does_not_bind_an_authored_homonym() {
        let source="(순서:바른수) 조작단계:글:셈씨 := { 조작단계 <- \"중간\". }. ((순서=0) 조작단계 == \"중간\") 보여주기.";
        assert!(warnings(source).is_empty());
    }
}
