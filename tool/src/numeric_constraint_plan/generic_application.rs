//! First-rank static applications for the direct parameter-transfer body.
//! Inputs come from source analysis, never from runtime values. No proof is issued.
use super::*;
use ddonirang_lang::{Span, SeedKind};

/// Diagnostic facts crossing the existing String-returning compiler boundary.
/// This decodes only our fixed diagnostic suffix, never authored DDN or types.
pub(crate) struct GenericSourceDiagnostic {
    pub(crate) code: &'static str,
    pub(crate) spans: Vec<Span>,
}

pub(crate) fn source_diagnostic(source: &str, message: &str) -> Option<GenericSourceDiagnostic> {
    let code = ["E_GENERIC_BODY_UNSUPPORTED", "E_GENERIC_RESULT_TYPE",
        "E_GENERIC_REQUIRED_INPUT", "E_GENERIC_INPUT_UNRESOLVED", "E_GENERIC_UNIFY_CONFLICT"]
        .into_iter().find(|code|message.strip_prefix(code).is_some_and(|rest|rest.starts_with(':')))?;
    let (_, locations) = message.rsplit_once("; 원문 ")?;
    let mut spans = Vec::new();
    for location in locations.split(" / ") {
        if spans.len() == 2 { return None; }
        let (start, end) = location.split_once("..")?;
        if !start.bytes().all(|byte|byte.is_ascii_digit()) || !end.bytes().all(|byte|byte.is_ascii_digit()) { return None; }
        let (start,end) = (start.parse::<usize>().ok()?,end.parse::<usize>().ok()?);
        if start>end || end>source.len() || !source.is_char_boundary(start) || !source.is_char_boundary(end) { return None; }
        spans.push(Span::new(start,end));
    }
    Some(GenericSourceDiagnostic { code, spans })
}

fn failure(seed: &SeedDef, code: &str, detail: &str) -> String {
    format!("{code}: {detail}; 원문 {}..{}", seed.span.start, seed.span.end)
}

pub(super) fn validate_body(seed: &SeedDef) -> Result<(), String> {
    let unsupported = || failure(seed, "E_GENERIC_BODY_UNSUPPORTED", "현재 두루쓰기 본문은 필수 입력의 직접 결과 전달을 지원합니다");
    if seed.seed_kind != SeedKind::Semssi || !seed.modifiers.is_empty()
        || seed.params.iter().any(|p| p.optional || p.default_value.is_some()) {
        return Err(unsupported());
    }
    fn mentions(kind: &TypeRef, name: &str) -> bool {
        match kind {
            TypeRef::Named(found) => found == name,
            TypeRef::Tuple(items) => items.iter().any(|item|mentions(item,name)),
            _ => false,
        }
    }
    if seed.type_params.iter().any(|param|!seed.params.iter().any(|pin|mentions(&pin.type_ref,&param.name))) {
        return Err(failure(seed,"E_GENERIC_INPUT_UNRESOLVED", "씨자리는 필수 입력에서 추론 가능해야 합니다"));
    }
    let Some(body) = &seed.body else { return Err(unsupported()); };
    let [Stmt::Mutate { target, value, .. }] = body.stmts.as_slice() else { return Err(unsupported()); };
    let (ExprKind::Var(result), ExprKind::Var(input)) = (&target.kind, &value.kind) else { return Err(unsupported()); };
    let Some(pin) = seed.params.iter().find(|pin| &pin.pin_name == input) else { return Err(unsupported()); };
    // Validate parametrically, before substitution. Coincident actual types
    // must not turn a body returning U into a declaration promising T.
    if result != &seed.canonical_name || seed.result_type.as_ref() != Some(&pin.type_ref) {
        return Err(failure(seed, "E_GENERIC_RESULT_TYPE", "결과 전달의 선언 씨가 일치하지 않습니다"));
    }
    Ok(())
}

fn intrinsic(kind: &TypeRef) -> bool {
    match kind {
        TypeRef::Named(name) => matches!(ddonirang_lang::stdlib::canonicalize_type_alias(name),
            "바른수" | "셈수" | "나눔수" | "큰바른수" | "곱수" | "글" | "참거짓" | "바이트열"),
        TypeRef::Tuple(items) => items.iter().all(intrinsic),
        _ => false,
    }
}

fn bind(seed: &SeedDef, formal: &TypeRef, actual: &TypeRef, span: Span,
    bindings: &mut BTreeMap<NodeId, (TypeRef, Span)>) -> Result<(), String> {
    if let TypeRef::Named(name) = formal {
        if let Some(param) = seed.type_params.iter().find(|param| &param.name == name) {
            if let Some((previous, previous_span)) = bindings.get(&param.id) {
                if previous != actual {
                    return Err(format!("E_GENERIC_UNIFY_CONFLICT: 씨자리 {}의 필수 입력이 충돌합니다; 원문 {}..{} / {}..{}",
                        param.name, previous_span.start, previous_span.end, span.start, span.end));
                }
            } else { bindings.insert(param.id, (actual.clone(), span)); }
            return Ok(());
        }
    }
    if let (TypeRef::Tuple(formals), TypeRef::Tuple(actuals)) = (formal, actual) {
        if formals.len() == actuals.len() {
            for (formal, actual) in formals.iter().zip(actuals) { bind(seed, formal, actual, span, bindings)?; }
            return Ok(());
        }
    }
    if formal == actual { Ok(()) }
    else { Err(failure(seed, "E_GENERIC_UNIFY_CONFLICT", "필수 입력의 선언 씨와 실제 씨가 다릅니다")) }
}

fn substitute(seed: &SeedDef, kind: &TypeRef, bindings: &BTreeMap<NodeId, (TypeRef, Span)>) -> TypeRef {
    match kind {
        TypeRef::Named(name) => seed.type_params.iter().find(|param| &param.name == name)
            .and_then(|param| bindings.get(&param.id)).map(|(kind, _)| kind.clone()).unwrap_or_else(||kind.clone()),
        TypeRef::Tuple(items) => TypeRef::Tuple(items.iter().map(|item|substitute(seed,item,bindings)).collect()),
        _ => kind.clone(),
    }
}

pub(super) fn instantiate(seed: &SeedDef, inputs: &[(Span, TypeRef)]) -> Result<SeedDef, String> {
    validate_body(seed)?;
    if inputs.len() != seed.params.len() { return Err(failure(seed,"E_GENERIC_REQUIRED_INPUT", "필수 입력이 누락됐습니다")); }
    let mut bindings = BTreeMap::new();
    for (pin, (span, actual)) in seed.params.iter().zip(inputs) {
        if !intrinsic(actual) { return Err(failure(seed,"E_GENERIC_INPUT_UNRESOLVED", "지원되는 정적 입력 씨의 공급이 필요합니다")); }
        bind(seed, &pin.type_ref, actual, *span, &mut bindings)?;
    }
    if seed.type_params.iter().any(|param|!bindings.contains_key(&param.id)) {
        return Err(failure(seed,"E_GENERIC_INPUT_UNRESOLVED", "모든 씨자리가 필수 입력에서 결정되어야 합니다"));
    }
    let mut resolved = seed.clone();
    for pin in &mut resolved.params { pin.type_ref = substitute(seed,&pin.type_ref,&bindings); }
    resolved.result_type = seed.result_type.as_ref().map(|kind|substitute(seed,kind,&bindings));
    resolved.type_params.clear();
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    const ID: &str = "(값씨:씨자리) 두고 (값:값씨~을) 그대로:값씨:셈씨 := { 그대로 <- 값. }.";
    #[test]
    fn generic_diagnostic_keeps_two_utf8_ranges_and_rejects_invalid_ranges() {
        let source="한😀3";
        let good=source_diagnostic(source,"E_GENERIC_UNIFY_CONFLICT: 충돌; 원문 0..3 / 3..7").unwrap();
        assert_eq!(good.code,"E_GENERIC_UNIFY_CONFLICT");
        assert_eq!(good.spans.len(),2);
        assert_eq!(&source[good.spans[1].start..good.spans[1].end],"😀");
        for message in ["E_GENERIC_UNIFY_CONFLICT: 충돌; 원문 1..3", "E_GENERIC_UNIFY_CONFLICT: 충돌; 원문 0..99",
            "E_GENERIC_UNIFY_CONFLICT: 충돌; 원문 0..3 / 3..7 / 7..8", "E_PARSE: 원문 0..3"] {
            assert!(source_diagnostic(source,message).is_none());
        }
    }
    fn compile(raw: &str) -> Result<NumericConstraintPlan, String> {
        let mut source = ddonirang_lang::parse_v25(raw,"제네릭.ddn").map_err(|error|format!("{error:?}"))?;
        ddonirang_lang::canonicalize(&mut source).map_err(|error|format!("{error:?}"))?;
        NumericConstraintPlan::compile(&source,&HashSet::new())
    }
    #[test]
    fn generic_calls_bind_per_application_not_first_call_cache() {
        let plan = compile(&format!("{ID}\n첫 <- (값=3) 그대로. 둘 <- (값=\"글\") 그대로.")).unwrap();
        assert_eq!(plan.generic_calls.len(),2);
        let kinds = plan.generic_calls.values().map(|seed|seed.result_type.clone().unwrap()).collect::<Vec<_>>();
        assert!(kinds.contains(&TypeRef::Named("바른수".into())));
        assert!(kinds.contains(&TypeRef::Named("글".into())));
        let definitions = plan.generic_calls.values().map(|seed|seed.id).collect::<BTreeSet<_>>();
        assert_eq!(definitions.len(),1);
    }
    #[test]
    fn generic_conflict_reports_both_required_input_spans() {
        let source = "(T:씨자리) 두고 (앞:T~을, 뒤:T~로) 첫째:T:셈씨 := { 첫째 <- 앞. }.\n결과 <- (앞=3, 뒤=\"글\") 첫째.";
        let error = compile(source).unwrap_err();
        assert!(error.starts_with("E_GENERIC_UNIFY_CONFLICT:"),"{error}");
        let number=source.find("3").unwrap(); let text=source.find("\"글\"").unwrap();
        assert!(error.contains(&format!("{}..{}",number,number+1)),"{error}");
        assert!(error.contains(&format!("{}..{}",text,text+"\"글\"".len())),"{error}");
    }
    #[test]
    fn generic_body_is_checked_before_actual_type_coincidence() {
        let source="(T:씨자리, U:씨자리) 두고 (앞:T~을, 뒤:U~로) 첫째:T:셈씨 := { 첫째 <- 뒤. }.\n결과 <- (앞=3, 뒤=4) 첫째.";
        assert!(compile(source).unwrap_err().starts_with("E_GENERIC_RESULT_TYPE:"));
    }
    #[test]
    fn generic_unresolved_and_nonparametric_bodies_do_not_execute() {
        for source in [ID.replace("값씨:씨자리", "값씨:씨자리, 미결:씨자리")+"\n결과 <- (값=3) 그대로.",
            ID.replace("그대로 <- 값.", "그대로 <- 3.")+"\n결과 <- (값=3) 그대로."] {
            assert!(compile(&source).is_err());
        }
    }
}
