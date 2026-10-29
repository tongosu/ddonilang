use crate::ast::*;
use crate::lexer::{Lexer, TokenKind};
use crate::parser::ParseError;
use crate::stdlib::minimal_stdlib_sigs;
use crate::term_map;
use std::collections::{HashMap, HashSet};

/// Bind AST occurrences by their role and the selected public declaration.
/// Raw source/token bytes and local binder spelling remain untouched. This
/// neither creates a translated executable body nor issues persistent IDs.
pub(crate) fn bind_first_callable_registered_program(
    program: &mut CanonProgram, context: &crate::parser::RegisteredFirstCallableContext,
) -> Result<(), ParseError> {
    let registry = crate::ddl::FIRST_CALLABLE_JA_REGISTRATION;
    let binding = crate::ddl::bind_first_callable_ja_declaration(
        &context.original, registry.language_profile_identity, registry.registration_revision,
    ).map_err(|error| ParseError { span: error.span, message: format!("{}: {}", error.code, error.message) })?;
    let original = binding.callable(registry.callable_name).unwrap();
    let fail = |span, message: &str| ParseError { span, message: message.into() };
    let native_entry=context.activation.is_some() && program.root_stmts.is_empty() && program.items.len()==1;
    if !native_entry && (!program.items.is_empty() || program.root_stmts.len() != 1) {
        return Err(fail(Span::new(0, program.origin.source.len()),
            "E_MULTILANG_FIRST_SCOPE: 첫 범위는 원본 함수를 사용하는 식·값 정의·명시적 결과 기록 하나입니다"));
    }
    fn expression(
        expr: &mut Expr, binding: &crate::ddl::FirstCallableJaDeclarationBinding<'_>,
    ) -> Result<(), ParseError> {
        let registry = crate::ddl::FIRST_CALLABLE_JA_REGISTRATION;
        match &mut expr.kind {
            ExprKind::Call { func, args } => {
                for arg in args.iter_mut() { expression(&mut arg.expr, binding)?; }
                if func == registry.callable_name {
                    let original = binding.callable(func).unwrap();
                    *func = original.canonical_name.clone();
                    for arg in args.iter_mut() {
                        if let Some(pin) = &mut arg.resolved_pin {
                            let original_pin = binding.pin(pin).ok_or_else(|| ParseError {
                                span: arg.span, message: "E_MULTILANG_PIN_UNREGISTERED: 未登録の引数名です".into(),
                            })?;
                            *pin = original_pin.pin_name.clone();
                        }
                        if let Some(particle) = &mut arg.josa {
                            let index = registry.pin_particles.iter().position(|registered| *registered == particle)
                                .ok_or_else(|| ParseError { span: arg.span, message: "E_MULTILANG_PARTICLE_UNREGISTERED".into() })?;
                            *particle = original.params[index].josa_list[0].clone();
                        }
                    }
                } else if func == registry.rational_type_name {
                    let TypeRef::Named(original_type) = binding.result_type(func).unwrap() else {
                        return Err(ParseError { span: expr.span, message: "E_MULTILANG_TYPE_SUPPLY_REQUIRED".into() });
                    };
                    *func = original_type.clone();
                } else {
                    return Err(ParseError { span: expr.span, message: "E_MULTILANG_CALL_UNREGISTERED: 未登録の呼び出しです".into() });
                }
            }
            ExprKind::Literal(_) | ExprKind::Var(_) => {}
            ExprKind::Infix { left, right, .. } => {
                expression(left, binding)?; expression(right, binding)?;
            }
            _ => return Err(ParseError { span: expr.span, message: "E_MULTILANG_EXPRESSION_UNSUPPORTED".into() }),
        }
        Ok(())
    }
    if native_entry {
        let TopLevelItem::SeedDef(seed)=&mut program.items[0];
        if seed.seed_kind!=SeedKind::Umjikssi||!seed.params.is_empty()||!seed.type_params.is_empty()
            ||seed.canonical_name==original.canonical_name||seed.canonical_name==registry.callable_name
            ||!seed.alternate_forms.is_empty(){return Err(fail(seed.span,"E_MULTILANG_NATIVE_ENTRY_SCOPE"));}
        let body=seed.body.as_mut().ok_or_else(||fail(seed.span,"E_MULTILANG_NATIVE_ENTRY_BODY_REQUIRED"))?;
        if body.stmts.len()!=1{return Err(fail(seed.span,"E_MULTILANG_NATIVE_ENTRY_SCOPE"));}
        match &mut body.stmts[0]{
            Stmt::Show{expr,..}=>expression(expr,&binding)?,
            _=>return Err(fail(seed.span,"E_MULTILANG_NATIVE_ENTRY_SCOPE")),
        }
    }else {match &mut program.root_stmts[0] {
        Stmt::Definition { name, type_ref, value, reaction, span, .. } => {
            if !registry.pin_names.contains(&name.as_str()) || reaction.is_some() {
                return Err(fail(*span, "E_MULTILANG_FIRST_BINDER_UNSUPPORTED"));
            }
            match type_ref {
                TypeRef::Named(name) if name == registry.rational_type_name => {
                    *type_ref = original.result_type.as_ref().unwrap().clone();
                }
                TypeRef::Infer => {}
                _ => return Err(fail(*span, "E_MULTILANG_TYPE_UNREGISTERED")),
            }
            expression(value, &binding)?;
        }
        Stmt::Expr { expr, .. } => expression(expr, &binding)?,
        Stmt::Mutate { target, value, deferred, span, .. } => {
            if *deferred || !matches!(&target.kind, ExprKind::Var(name) if registry.pin_names.contains(&name.as_str())) {
                return Err(fail(*span, "E_MULTILANG_FIRST_BINDER_UNSUPPORTED"));
            }
            expression(value, &binding)?;
        }
        _ => return Err(fail(Span::new(0, program.origin.source.len()), "E_MULTILANG_STATEMENT_UNSUPPORTED")),
    }
    }
    // Only the actual original body is supplied. DDL has body=None. The
    // compiler context retains its distinct source owner for diagnostics.
    program.items.push(TopLevelItem::SeedDef(original.clone()));
    Ok(())
}

/// Format only the authored caller, through the existing N1 printer, and
/// re-resolve it against the same immutable context. Imported original bodies
/// are not emitted as Japanese definitions or mixed-language canonical text.
pub fn canonicalize_first_callable_registered_source(
    program: &CanonProgram, context: &std::sync::Arc<crate::parser::RegisteredFirstCallableContext>,
) -> Result<String, ParseError> {
    let registry = crate::ddl::FIRST_CALLABLE_JA_REGISTRATION;
    let binding = crate::ddl::bind_first_callable_ja_declaration(&context.original,
        registry.language_profile_identity, registry.registration_revision)
        .map_err(|e| ParseError { span: e.span, message: format!("{}: {}", e.code, e.message) })?;
    let original = binding.callable(registry.callable_name).unwrap();
    fn expression(expr: &mut Expr, original: &SeedDef) -> Result<(), ParseError> {
        let registry = crate::ddl::FIRST_CALLABLE_JA_REGISTRATION;
        match &mut expr.kind {
            ExprKind::Call { func, args } => {
                for arg in args.iter_mut() { expression(&mut arg.expr, original)?; }
                if func == &original.canonical_name {
                    *func = registry.callable_name.into();
                    for arg in args {
                        if let Some(pin) = &mut arg.resolved_pin {
                            let index = original.params.iter().position(|p| &p.pin_name == pin)
                                .ok_or_else(|| ParseError { span: arg.span, message: "E_MULTILANG_CANON_PIN_BINDING".into() })?;
                            *pin = registry.pin_names[index].into();
                        }
                        if let Some(particle) = &mut arg.josa {
                            let index = original.params.iter().position(|p| p.josa_list.contains(particle))
                                .ok_or_else(|| ParseError { span: arg.span, message: "E_MULTILANG_CANON_PARTICLE_BINDING".into() })?;
                            *particle = registry.pin_particles[index].into();
                        }
                    }
                } else if matches!(&original.result_type, Some(TypeRef::Named(name)) if name == func) {
                    *func = registry.rational_type_name.into();
                } else {
                    return Err(ParseError { span: expr.span, message: "E_MULTILANG_CANON_CALL_BINDING".into() });
                }
            }
            ExprKind::Infix { left, right, .. } => { expression(left, original)?; expression(right, original)?; }
            ExprKind::Literal(_) | ExprKind::Var(_) => {}
            _ => return Err(ParseError { span: expr.span, message: "E_MULTILANG_CANON_EXPRESSION".into() }),
        }
        Ok(())
    }
    fn shape(stmt: &Stmt) -> Result<String, ParseError> {
        let expr = crate::normalizer::normalize_expression_identity;
        Ok(match stmt {
            Stmt::Definition { name, type_ref, value, .. } => format!("definition:{name}:{type_ref:?}:{}", expr(value)),
            Stmt::Mutate { target, value, deferred, .. } => format!("record:{deferred}:{}:{}", expr(target), expr(value)),
            Stmt::Expr { expr: value, .. } => format!("expression:{}", expr(value)),
            _ => return Err(ParseError { span: Span::new(0, 0), message: "E_MULTILANG_CANON_STATEMENT".into() }),
        })
    }
    if program.root_stmts.is_empty() && context.activation.is_some() {
        let mut view=program.clone();
        view.items.retain(|TopLevelItem::SeedDef(seed)|seed.canonical_name!=original.canonical_name);
        if view.items.len()!=1{return Err(ParseError{span:Span::new(0,0),message:"E_MULTILANG_NATIVE_ENTRY_SCOPE".into()});}
        let TopLevelItem::SeedDef(seed)=&mut view.items[0];
        let body=seed.body.as_mut().ok_or_else(||ParseError{span:seed.span,message:"E_MULTILANG_NATIVE_ENTRY_BODY_REQUIRED".into()})?;
        if let [Stmt::Show{expr,..}]=body.stmts.as_mut_slice(){expression(expr,original)?;}else{return Err(ParseError{span:seed.span,message:"E_MULTILANG_NATIVE_ENTRY_SCOPE".into()});}
        let activation=context.activation.as_ref().unwrap();
        let directive_context=crate::module_directive::ModuleDirectiveContext::default().with_known_imports([activation.binding.target.clone()]);
        let (mut document,_)=crate::module_directive::first_callable_module_tokens(&program.origin.source,&program.origin.file_path,&directive_context,context)?;
        document.declaration_source.clear();
        if let Some(request)=document.imports.as_mut().and_then(|r|r.translation.as_mut()){
            request.target_language=activation.profile.canonical_name().into();request.target_version=activation.profile.version().into();
        }
        let text=format!("{}{}",crate::module_directive::canonicalize_module_directive_document(&document),crate::normalize(&view,crate::NormalizationLevel::N1));
        let reparsed=crate::parser::Parser::parse_with_first_callable_context(&text,&program.origin.file_path,context.clone())?;
        if crate::normalize(&reparsed,crate::NormalizationLevel::N1)!=crate::normalize(program,crate::NormalizationLevel::N1){return Err(ParseError{span:Span::new(0,text.len()),message:"E_MULTILANG_CANON_BINDING_CHANGED".into()});}
        return Ok(text);
    }
    let before = program.root_stmts.first().ok_or_else(|| ParseError { span: Span::new(0, 0), message: "E_MULTILANG_CANON_EMPTY".into() })?;
    let before = shape(before)?;
    let mut view = program.clone();
    view.items.clear();
    match &mut view.root_stmts[0] {
        Stmt::Definition { type_ref, value, .. } => {
            if *type_ref != TypeRef::Infer { *type_ref = TypeRef::Named(registry.rational_type_name.into()); }
            expression(value, original)?;
        }
        Stmt::Mutate { value, .. } => expression(value, original)?,
        Stmt::Expr { expr, .. } => expression(expr, original)?,
        _ => return Err(ParseError { span: Span::new(0, 0), message: "E_MULTILANG_CANON_STATEMENT".into() }),
    }
    let header=if let Some(activation)=&context.activation {
        let directive_context=crate::module_directive::ModuleDirectiveContext::default().with_known_imports([activation.binding.target.clone()]);
        let (mut document,_)=crate::module_directive::first_callable_module_tokens(&program.origin.source,&program.origin.file_path,&directive_context,context)?;
        document.declaration_source.clear();
        if let Some(request)=document.imports.as_mut().and_then(|record|record.translation.as_mut()){
            request.target_language=activation.profile.canonical_name().into();request.target_version=activation.profile.version().into();
        }
        crate::module_directive::canonicalize_module_directive_document(&document)
    } else {"#!日本語\n".into()};
    let text = format!("{}{}",header, crate::normalize(&view, crate::NormalizationLevel::N1));
    let reparsed = crate::parser::Parser::parse_with_first_callable_context(&text, &program.origin.file_path, context.clone())?;
    if reparsed.root_stmts.len() != 1 || shape(&reparsed.root_stmts[0])? != before {
        return Err(ParseError { span: Span::new(0, program.origin.source.len()), message: "E_MULTILANG_CANON_BINDING_CHANGED".into() });
    }
    Ok(text)
}

pub struct LintWarning {
    pub code: &'static str,
    pub span: Span,
    pub message: String,
}

pub struct CanonicalizeReport {
    pub warnings: Vec<LintWarning>,
}

#[derive(Clone)]
struct CanonicalSeedContext {
    signatures:HashMap<String,Vec<ParamPin>>,
    declared_types:HashSet<String>,
    su_alias:bool,
    registered_sum_pins:Option<std::collections::BTreeMap<NodeId,[u8;32]>>,
}
impl std::ops::Deref for CanonicalSeedContext {
    type Target=HashMap<String,Vec<ParamPin>>;
    fn deref(&self)->&Self::Target {&self.signatures}
}

/// Canonical DDL Wave 1 presentation.  Section order is retained for humans;
/// stable identities are already registry-bound in the DDL AST and are not
/// derived from this order.
pub fn canonicalize_ddl_wave1(document: &DdlDocument) -> String {
    let mut sections = Vec::with_capacity(document.sections.len());
    for section in &document.sections {
        let header = match section.language_profile_identity.as_str() {
            "language-profile:ko" => "#!한국어",
            "language-profile:ja" => "#!日本語",
            _ => unreachable!("DDL parser admits only the Wave 1 exact registry"),
        };
        let projection = &section.projection;
        debug_assert!(matches!(
            projection.omitted_payload,
            OmittedPayload::ValueInitializer
        ));
        sections.push(format!(
            "{header}\n{}:{} :={{...}}.",
            projection.localized_name, projection.localized_type
        ));
    }
    format!("{}\n", sections.join("\n\n"))
}

pub fn canonicalize(program: &mut CanonProgram) -> Result<CanonicalizeReport, ParseError> {
    let mut warnings = Vec::new();
    let signatures = collect_seed_signatures(program);
    for item in &mut program.items {
        canonicalize_top_level_item(item, &signatures, &mut warnings)?;
    }
    for stmt in &mut program.root_stmts {
        canonicalize_stmt(stmt, &signatures, &mut warnings)?;
    }
    let known_seeds = collect_known_seeds(program);
    let stdlib_names = collect_stdlib_names();
    lint_tailless_calls(program, &known_seeds, &stdlib_names, &mut warnings);
    lint_deprecated_block_header_colon(program, &mut warnings);
    lint_redundant_top_level_chaebi_reassign(program, &mut warnings);
    Ok(CanonicalizeReport { warnings })
}

fn lint_deprecated_block_header_colon(program: &CanonProgram, warnings: &mut Vec<LintWarning>) {
    let Ok(tokens) = Lexer::new(&program.origin.source).tokenize() else {
        return;
    };
    if tokens.len() < 3 {
        return;
    }
    for idx in 0..(tokens.len() - 2) {
        if !matches!(tokens[idx + 1].kind, TokenKind::Colon) {
            continue;
        }
        if !matches!(tokens[idx + 2].kind, TokenKind::LBrace) {
            continue;
        }
        let is_deprecated_header = match &tokens[idx].kind {
            TokenKind::KwBanbok | TokenKind::KwDongan | TokenKind::KwDaehae | TokenKind::KwBeat => {
                true
            }
            TokenKind::Ident(name) | TokenKind::Josa(name) => {
                matches!(name.as_str(), "채비" | "설정" | "보개" | "모양" | "슬기")
            }
            _ => false,
        };
        if !is_deprecated_header {
            continue;
        }
        let keyword = tokens[idx].raw.trim();
        warnings.push(LintWarning {
            code: "W_BLOCK_HEADER_COLON_DEPRECATED",
            span: Span {
                start: tokens[idx].span.start,
                end: tokens[idx + 2].span.end,
            },
            message: format!(
                "블록 헤더의 `{keyword}:` 표기는 예정된 비권장입니다. `{keyword} {{` 표기로 전환하세요"
            ),
        });
    }
}

fn lint_redundant_top_level_chaebi_reassign(
    program: &CanonProgram,
    warnings: &mut Vec<LintWarning>,
) {
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        let Some(body) = &seed.body else {
            continue;
        };
        let mut declared_names = HashSet::new();
        for stmt in &body.stmts {
            match stmt {
                Stmt::DeclBlock { items, .. } => {
                    for item in items {
                        declared_names.insert(item.name.clone());
                    }
                }
                Stmt::Mutate { target, value, .. } => {
                    let ExprKind::Var(name) = &target.kind else {
                        continue;
                    };
                    if !declared_names.contains(name) {
                        continue;
                    }
                    if !is_redundant_top_level_chaebi_reassign_value(value) {
                        continue;
                    }
                    warnings.push(LintWarning {
                        code: "W_CHAEBI_REDUNDANT_TOP_REASSIGN",
                        span: target.span,
                        message: format!(
                            "`채비 {{}}`에서 이미 준비한 `{name}`를 같은 최상위에서 다시 대입했습니다. 중복 준비가 아니면 일반 대입만 남기세요."
                        ),
                    });
                }
                _ => {}
            }
        }
    }
}

fn is_redundant_top_level_chaebi_reassign_value(value: &Expr) -> bool {
    matches!(value.kind, ExprKind::Literal(_))
}

fn canonicalize_top_level_item(
    item: &mut TopLevelItem,
    signatures: &CanonicalSeedContext,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    match item {
        TopLevelItem::SeedDef(seed) => canonicalize_seed_def(seed, signatures, warnings),
    }
}

fn canonicalize_seed_def(
    seed: &mut SeedDef,
    signatures: &CanonicalSeedContext,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    canonicalize_ident(&mut seed.canonical_name, seed.span, warnings)?;
    for form in &mut seed.alternate_forms {
        canonicalize_ident(form, seed.span, warnings)?;
    }
    seed.alternate_forms.sort();
    if let SeedKind::Named(name) = &mut seed.seed_kind {
        canonicalize_ident(name, seed.span, warnings)?;
    }
    for param in &mut seed.params {
        canonicalize_ident(&mut param.pin_name, param.span, warnings)?;
        canonicalize_type_ref(&mut param.type_ref, param.span, warnings, signatures)?;
        if let Some(default_value) = &mut param.default_value {
            canonicalize_expr(default_value, signatures, warnings)?;
        }
    }
    if let Some(result_type) = &mut seed.result_type {
        canonicalize_type_ref(result_type, seed.span, warnings, signatures)?;
    }
    if let Some(body) = &mut seed.body {
        canonicalize_body(body, signatures, warnings)?;
    }
    Ok(())
}

fn canonicalize_body(
    body: &mut Body,
    signatures: &CanonicalSeedContext,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    let mut scoped=signatures.clone();
    for statement in &body.stmts {
        if let Stmt::SumDefinition{name,..}=statement {scoped.declared_types.insert(name.clone());}
    }
    for stmt in &mut body.stmts {
        canonicalize_stmt(stmt, &scoped, warnings)?;
    }
    Ok(())
}

fn canonicalize_stmt(
    stmt: &mut Stmt,
    signatures: &CanonicalSeedContext,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    match stmt {
        Stmt::SumDefinition { name, branches, span, .. } => {
            canonicalize_ident(name, *span, warnings)?;
            for branch in branches {
                canonicalize_ident(&mut branch.name, branch.span, warnings)?;
                for payload in &mut branch.payload {
                    canonicalize_ident(&mut payload.binder, payload.binder_span, warnings)?;
                    canonicalize_type_ref(&mut payload.type_ref, payload.binder_span, warnings, signatures)?;
                }
            }
        }
        Stmt::SumMatch { subject, arms, default_body, .. } => {
            canonicalize_expr(subject, signatures, warnings)?;
            for arm in arms {
                if let Some(kind)=&mut arm.applied_owner {
                    canonicalize_type_ref(kind,arm.span,warnings,signatures)?;
                    // This lookup key belongs to the current compilation.
                    // Canonical type aliases must update both projections;
                    // saved owner bytes are not processed by this path.
                    arm.owner=crate::sum_owner::resolve_applied_sum_owner(kind,arm.span)
                        .map_err(|message|ParseError{span:arm.span,message})?.key;
                }
                else {canonicalize_ident(&mut arm.owner, arm.span, warnings)?;}
                canonicalize_ident(&mut arm.branch, arm.span, warnings)?;
                for binder in &mut arm.binder {
                    canonicalize_ident(&mut binder.pin, binder.span, warnings)?;
                    if let Some(order)=&signatures.registered_sum_pins {
                        let raw=order.get(&binder.id).filter(|raw|**raw!=[0;32]).ok_or_else(||ParseError{
                            span:binder.span,message:"E_SUM_PIN_IDENTITY_REQUIRED: 원문 받는자리의 검증된 등록 정체가 없습니다".into()})?;
                        binder.registered_pin_identity=Some(*raw);
                    }
                }
                canonicalize_body(&mut arm.body, signatures, warnings)?;
            }
            if let Some(body) = default_body {
                canonicalize_body(body, signatures, warnings)?;
            }
        }
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                canonicalize_ident(&mut item.name, item.span, warnings)?;
                canonicalize_type_ref(&mut item.type_ref, item.span, warnings, signatures)?;
                if let Some(value) = &mut item.value {
                    canonicalize_expr(value, signatures, warnings)?;
                }
            }
        }
        Stmt::Definition {
            name,
            type_ref,
            value,
            span,
            reaction,
            ..
        } => {
            canonicalize_ident(name, *span, warnings)?;
            canonicalize_type_ref(type_ref, *span, warnings, signatures)?;
            if let Some(registration)=reaction {
                canonicalize_ident(&mut registration.alert_type,*span,warnings)?;
                canonicalize_ident(&mut registration.payload.pin_name,registration.payload.span,warnings)?;
                canonicalize_type_ref(&mut registration.payload.type_ref,registration.payload.span,warnings,signatures)?;
            }
            canonicalize_expr(value, signatures, warnings)?;
        }
        Stmt::DerivedBinding {name,value,span,..} => {
            canonicalize_ident(name,*span,warnings)?;
            canonicalize_expr(value,signatures,warnings)?;
        }
        Stmt::Mutate { target, value, .. } => {
            canonicalize_expr(target, signatures, warnings)?;
            canonicalize_expr(value, signatures, warnings)?;
        }
        Stmt::ResultFlow {
            producer,
            destination,
            ..
        } => {
            canonicalize_expr(producer, signatures, warnings)?;
            canonicalize_expr(destination, signatures, warnings)?;
        }
        Stmt::Expr { expr, .. } => canonicalize_expr(expr, signatures, warnings)?,
        Stmt::Receive {
            condition, body, ..
        } => {
            if let Some(condition) = condition {
                canonicalize_expr(condition, signatures, warnings)?;
            }
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => {
            if let Some(sender) = sender {
                canonicalize_expr(sender, signatures, warnings)?;
            }
            canonicalize_expr(payload, signatures, warnings)?;
            canonicalize_expr(receiver, signatures, warnings)?;
        }
        Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
            canonicalize_expr(expr, signatures, warnings)?
        }
        Stmt::PublicObservation { fields, .. } => {
            for field in fields {
                canonicalize_expr(&mut field.value, signatures, warnings)?;
            }
        }
        Stmt::MetaBlock { .. } => {}
        Stmt::Pragma { .. } => {}
        Stmt::Return { value, .. } => canonicalize_expr(value, signatures, warnings)?,
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            canonicalize_expr(condition, signatures, warnings)?;
            canonicalize_body(then_body, signatures, warnings)?;
            if let Some(body) = else_body {
                canonicalize_body(body, signatures, warnings)?;
            }
        }
        Stmt::Try { action, body, .. } => {
            canonicalize_expr(action, signatures, warnings)?;
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                canonicalize_expr(&mut branch.condition, signatures, warnings)?;
                canonicalize_body(&mut branch.body, signatures, warnings)?;
            }
            canonicalize_body(else_body, signatures, warnings)?;
        }
        Stmt::Repeat { body, .. } => {
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::BeatBlock { body, .. } => {
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::Hook { body, .. } => {
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => {
            canonicalize_expr(condition, signatures, warnings)?;
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::While {
            condition, body, ..
        } => {
            canonicalize_expr(condition, signatures, warnings)?;
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::ForEach {
            item,
            item_type,
            iterable,
            body,
            span,
            ..
        } => {
            canonicalize_ident(item, *span, warnings)?;
            if let Some(type_ref) = item_type.as_mut() {
                canonicalize_type_ref(type_ref, *span, warnings, signatures)?;
            }
            canonicalize_expr(iterable, signatures, warnings)?;
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::Quantifier {
            variable,
            domain,
            body,
            span,
            ..
        } => {
            canonicalize_ident(variable, *span, warnings)?;
            canonicalize_type_ref(domain, *span, warnings, signatures)?;
            canonicalize_body(body, signatures, warnings)?;
        }
        Stmt::Break { .. } | Stmt::ContinueLoop { .. } => {}
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => {
            canonicalize_expr(condition, signatures, warnings)?;
            if let Some(body) = then_body {
                canonicalize_body(body, signatures, warnings)?;
            }
            canonicalize_body(else_body, signatures, warnings)?;
        }
        Stmt::Guard {
            condition, body, ..
        } => {
            canonicalize_expr(condition, signatures, warnings)?;
            canonicalize_body(body, signatures, warnings)?;
        }
    }
    Ok(())
}

fn canonicalize_expr(
    expr: &mut Expr,
    signatures: &CanonicalSeedContext,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    match &mut expr.kind {
        ExprKind::SumBranchValue { owner, applied_owner, branch, payload } => {
            if let Some(kind)=applied_owner {
                canonicalize_type_ref(kind,expr.span,warnings,signatures)?;
                *owner=crate::sum_owner::resolve_applied_sum_owner(kind,expr.span)
                    .map_err(|message|ParseError{span:expr.span,message})?.key;
            }
            else {canonicalize_ident(owner, expr.span, warnings)?;}
            canonicalize_ident(branch, expr.span, warnings)?;
            for payload in payload {
                canonicalize_ident(&mut payload.pin,payload.span,warnings)?;
                canonicalize_expr(&mut payload.value, signatures, warnings)?;
            }
        }
        ExprKind::Tensor(value) => {
            if let TensorProviderExpr::FromCollection{element_type,..}=value {
                canonicalize_type_ref(element_type,expr.span,warnings,signatures)?;
            }
            canonicalize_expr(value.input_mut(), signatures, warnings)?;
        },
        ExprKind::MultiAxisSelection(value) => {
            for child in value.children_mut() { canonicalize_expr(child, signatures, warnings)?; }
        }
        ExprKind::Var(name) => canonicalize_ident(name, expr.span, warnings)?,
        ExprKind::FieldAccess { target, field, .. } => {
            canonicalize_expr(target, signatures, warnings)?;
            canonicalize_ident(field, expr.span, warnings)?;
        }
        ExprKind::Call { args, func } => {
            canonicalize_ident(func, expr.span, warnings)?;
            for arg in args {
                canonicalize_expr(&mut arg.expr, signatures, warnings)?;
                if let Some(pin) = &mut arg.resolved_pin {
                    canonicalize_ident(pin, arg.span, warnings)?;
                }
                canonicalize_call_arg_josa(arg, func, signatures);
                lint_call_arg_josa_conflict(arg, func, signatures, warnings);
            }
        }
        ExprKind::ProductConstruction { owner, fields, .. } => {
            canonicalize_ident(owner, expr.span, warnings)?;
            for field in fields {
                if let Some(name) = &mut field.resolved_pin {
                    canonicalize_ident(name, field.span, warnings)?;
                }
                canonicalize_expr(&mut field.expr, signatures, warnings)?;
            }
        }
        ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
            for element in elements {
                canonicalize_expr(element, signatures, warnings)?;
            }
        }
        ExprKind::SelectionPlace { target, index } => {
            canonicalize_expr(target, signatures, warnings)?;
            canonicalize_expr(index, signatures, warnings)?;
        }
        ExprKind::Infix { left, right, .. } => {
            canonicalize_expr(left, signatures, warnings)?;
            canonicalize_expr(right, signatures, warnings)?;
        }
        ExprKind::Suffix { value, .. } => canonicalize_expr(value, signatures, warnings)?,
        ExprKind::Thunk(body) => canonicalize_body(body, signatures, warnings)?,
        ExprKind::Eval { thunk, .. } => canonicalize_expr(thunk, signatures, warnings)?,
        ExprKind::SeedLiteral { param, body } => {
            for part in param.split(',') {
                let candidate = part.trim();
                if !candidate.is_empty() {
                    let mut name = candidate.to_string();
                    canonicalize_ident(&mut name, expr.span, warnings)?;
                }
            }
            canonicalize_expr(body, signatures, warnings)?;
        }
        ExprKind::Pipe { stages } => {
            for stage in stages {
                canonicalize_expr(stage, signatures, warnings)?;
            }
        }
        ExprKind::Pack { fields } => {
            for (name, value) in fields {
                canonicalize_ident(name, expr.span, warnings)?;
                canonicalize_expr(value, signatures, warnings)?;
            }
        }
        ExprKind::Assertion(_) => {}
        ExprKind::Formula(_) => {}
        ExprKind::Template(_) => {}
        ExprKind::StateMachine(_) => {}
        ExprKind::TemplateRender { inject, .. } => {
            for (_, value) in inject {
                canonicalize_expr(value, signatures, warnings)?;
            }
        }
        ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                canonicalize_expr(value, signatures, warnings)?;
            }
        }
        ExprKind::Nuance { expr, .. } => canonicalize_expr(expr, signatures, warnings)?,
        ExprKind::Literal(_) | ExprKind::FlowValue => {}
    }
    Ok(())
}

fn canonicalize_call_arg_josa(
    arg: &mut ArgBinding,
    func: &str,
    signatures: &CanonicalSeedContext,
) {
    let Some(josa) = arg.josa.as_ref() else {
        return;
    };
    let Some(pin) = arg.resolved_pin.as_deref() else {
        return;
    };
    let Some(params) = signatures.get(func) else {
        return;
    };
    let Some(param) = params.iter().find(|param| param.pin_name == pin) else {
        return;
    };
    if !param.josa_list.iter().any(|candidate| candidate == josa) {
        return;
    }
    if let Some(primary_josa) = param.josa_list.first() {
        arg.josa = Some(primary_josa.clone());
    }
}

fn lint_call_arg_josa_conflict(
    arg: &ArgBinding,
    func: &str,
    signatures: &CanonicalSeedContext,
    warnings: &mut Vec<LintWarning>,
) {
    if !matches!(arg.binding_reason, BindingReason::UserFixed) {
        return;
    }
    let Some(josa) = arg.josa.as_deref() else {
        return;
    };
    let Some(pin) = arg.resolved_pin.as_deref() else {
        return;
    };
    let Some(params) = signatures.get(func) else {
        return;
    };
    let matches: Vec<&ParamPin> = params
        .iter()
        .filter(|param| param.josa_list.iter().any(|candidate| candidate == josa))
        .collect();
    if matches.len() < 2 {
        return;
    }
    if !matches.iter().any(|param| param.pin_name == pin) {
        return;
    }
    warnings.push(LintWarning {
        code: "W_CALL_JOSA_CONFLICT_FIXED",
        span: arg.span,
        message: format!(
            "호출 `{func}`에서 조사 `{josa}`는 여러 핀에 걸칩니다. 현재 인자는 `값:핀` 고정으로 해석했습니다"
        ),
    });
}

fn canonicalize_type_ref(
    type_ref: &mut TypeRef,
    span: Span,
    warnings: &mut Vec<LintWarning>,
    context:&CanonicalSeedContext,
) -> Result<(), ParseError> {
    match type_ref {
        TypeRef::AlgebraEquality {left,right} => {
            canonicalize_algebra_term(left,warnings)?;
            canonicalize_algebra_term(right,warnings)?;
        }
        TypeRef::StructureLaw { structure, member } => {
            canonicalize_ident(structure, span, warnings)?;
            canonicalize_ident(member, span, warnings)?;
        }
        TypeRef::Tuple(items) | TypeRef::PropositionConjunction(items) => {
            for item in items {
                canonicalize_type_ref(item, span, warnings, context)?;
            }
        }
        TypeRef::Tensor { element, .. } => canonicalize_type_ref(element, span, warnings, context)?,
        TypeRef::Named(name) => {
            canonicalize_ident(name, span, warnings)?;
            // M-A: only the builtin declaration alias. A schema owner in the
            // actual lexical type scope takes precedence; value identifiers,
            // function names and applied constructor names are not rewritten.
            *name=resolve_declared_type_spelling(name,context.su_alias,&context.declared_types,false);
        }
        TypeRef::Applied { name, args } => {
            canonicalize_ident(name, span, warnings)?;
            *name=resolve_declared_type_spelling(name,context.su_alias,&context.declared_types,true);
            for arg in args {
                canonicalize_type_ref(arg, span, warnings, context)?;
            }
        }
        TypeRef::Function { params, result } => {
            for param in params {
                canonicalize_type_ref(param, span, warnings, context)?;
            }
            canonicalize_type_ref(result, span, warnings, context)?;
        }
        TypeRef::Infer => {}
    }
    Ok(())
}

/// Common declaration-side alias resolution, not a TypeIdentity issuer.
/// Registration's static owner uses the same lexical shadow rule; it must
/// still resolve the resulting name to its actual declaration/builtin owner.
pub fn resolve_declared_type_spelling(name:&str,su_alias:bool,declared_types:&HashSet<String>,applied:bool)->String {
    if !applied&&su_alias&&name=="수"&&!declared_types.contains(name) {return "셈수".into();}
    canonical_type_alias(name).unwrap_or(name).to_owned()
}

fn canonical_type_alias(name: &str) -> Option<&'static str> {
    match name.trim() {
        "셈수" | "셈수2" | "셈수4" | "셈수8" | "fixed64" | "sim_num" => Some("셈수"),
        "바른수" | "바른수1" | "바른수2" | "바른수4" | "바른수8" | "정수" | "int" | "int64" => {
            Some("바른수")
        }
        "큰바른수" | "bigint" | "big_int" => Some("큰바른수"),
        "나눔수" | "유리수" | "rational" | "ratio" | "frac" => Some("나눔수"),
        "곱수" | "factor" | "factorized" | "primepow" => Some("곱수"),
        "논" | "bool" | "boolean" => Some("참거짓"),
        "목록" | "list" => Some("차림"),
        "모둠" | "set" => Some("모음"),
        "그림표" | "map" => Some("짝맞춤"),
        "값꾸러미" | "pack" => Some("묶음"),
        _ => None,
    }
}

fn canonicalize_ident(
    name: &mut String,
    span: Span,
    warnings: &mut Vec<LintWarning>,
) -> Result<(), ParseError> {
    if let Some(entry) = term_map::find_legacy_term(name.as_str()) {
        warnings.push(LintWarning {
            code: entry.code,
            span,
            message: format!(
                "TERM-LINT-01: 레거시 용어 '{}'는 '{}'를 권장합니다",
                entry.input, entry.canonical
            ),
        });
    }
    Ok(())
}

fn collect_known_seeds(program: &CanonProgram) -> HashSet<String> {
    let mut known = HashSet::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if seed.seed_kind.is_schema() { continue; }
        known.extend(seed.callable_surface_names());
    }
    for sig in minimal_stdlib_sigs() {
        known.insert(sig.name.to_string());
    }
    known
}

fn collect_seed_signatures(program: &CanonProgram) -> CanonicalSeedContext {
    let mut signatures = HashMap::new();
    let mut declared_types=HashSet::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if seed.seed_kind.is_schema() {declared_types.insert(seed.canonical_name.clone());continue;}
        for surface in seed.callable_surface_names() {
            signatures.insert(surface, seed.params.clone());
        }
    }
    for statement in &program.root_stmts {
        if let Stmt::SumDefinition{name,..}=statement {declared_types.insert(name.clone());}
    }
    CanonicalSeedContext {signatures,declared_types,su_alias:program.source_epoch==SourceEpoch::V25,registered_sum_pins:program.origin.registered_sum_pin_order.clone()}
}

fn collect_stdlib_names() -> HashSet<String> {
    let mut names = HashSet::new();
    for sig in minimal_stdlib_sigs() {
        names.insert(sig.name.to_string());
    }
    names
}

fn lint_tailless_calls(
    program: &CanonProgram,
    known_seeds: &HashSet<String>,
    stdlib_names: &HashSet<String>,
    warnings: &mut Vec<LintWarning>,
) {
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            lint_tailless_body(body, known_seeds, stdlib_names, warnings);
        }
    }
}

fn lint_tailless_body(
    body: &Body,
    known_seeds: &HashSet<String>,
    stdlib_names: &HashSet<String>,
    warnings: &mut Vec<LintWarning>,
) {
    for stmt in &body.stmts {
        match stmt {
            Stmt::SumDefinition { .. } => {}
            Stmt::SumMatch { subject, arms, default_body, .. } => {
                lint_tailless_expr(subject, known_seeds, stdlib_names, warnings);
                for arm in arms {
                    lint_tailless_body(&arm.body, known_seeds, stdlib_names, warnings);
                }
                if let Some(body) = default_body {
                    lint_tailless_body(body, known_seeds, stdlib_names, warnings);
                }
            }
            Stmt::DeclBlock { items, .. } => {
                for item in items {
                    if let Some(value) = &item.value {
                        lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
                    }
                }
            }
            Stmt::Expr { expr, .. } => {
                if let ExprKind::Var(name) = &expr.kind {
                    if known_seeds.contains(name) && !stdlib_names.contains(name) {
                        warnings.push(LintWarning {
                            code: "E_CALL_TAIL_MISSING_STMT",
                            span: expr.span,
                            message: format!(
                                "호출 꼬리가 필요합니다. 예: '{}하기.' 또는 '{}기.'",
                                name, name
                            ),
                        });
                    }
                }
                lint_tailless_expr(expr, known_seeds, stdlib_names, warnings);
            }
            Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                lint_tailless_expr(expr, known_seeds, stdlib_names, warnings);
            }
            Stmt::ResultFlow {
                producer,
                destination,
                ..
            } => {
                lint_tailless_expr(producer, known_seeds, stdlib_names, warnings);
                lint_tailless_expr(destination, known_seeds, stdlib_names, warnings);
            }
            Stmt::Receive {
                condition, body, ..
            } => {
                if let Some(condition) = condition {
                    lint_tailless_expr(condition, known_seeds, stdlib_names, warnings);
                }
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                if let Some(sender) = sender {
                    lint_tailless_expr(sender, known_seeds, stdlib_names, warnings);
                }
                lint_tailless_expr(payload, known_seeds, stdlib_names, warnings);
                lint_tailless_expr(receiver, known_seeds, stdlib_names, warnings);
            }
            Stmt::PublicObservation { fields, .. } => {
                for field in fields {
                    lint_tailless_expr(&field.value, known_seeds, stdlib_names, warnings);
                }
            }
            Stmt::MetaBlock { .. } => {}
            Stmt::Pragma { .. } => {}
            Stmt::Definition { value, .. } | Stmt::DerivedBinding {value,..} => {
                lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
            }
            Stmt::Mutate { target, value, .. } => {
                lint_tailless_expr(target, known_seeds, stdlib_names, warnings);
                lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
            }
            Stmt::Return { value, .. } => {
                lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                lint_tailless_expr(condition, known_seeds, stdlib_names, warnings);
                lint_tailless_body(then_body, known_seeds, stdlib_names, warnings);
                if let Some(body) = else_body {
                    lint_tailless_body(body, known_seeds, stdlib_names, warnings);
                }
            }
            Stmt::Try { action, body, .. } => {
                lint_tailless_expr(action, known_seeds, stdlib_names, warnings);
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    lint_tailless_expr(&branch.condition, known_seeds, stdlib_names, warnings);
                    lint_tailless_body(&branch.body, known_seeds, stdlib_names, warnings);
                }
                lint_tailless_body(else_body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Repeat { body, .. } => {
                lint_tailless_body(body, known_seeds, stdlib_names, warnings)
            }
            Stmt::BeatBlock { body, .. } => {
                lint_tailless_body(body, known_seeds, stdlib_names, warnings)
            }
            Stmt::Hook { body, .. } => {
                lint_tailless_body(body, known_seeds, stdlib_names, warnings)
            }
            Stmt::HookWhenBecomes {
                condition, body, ..
            }
            | Stmt::HookWhile {
                condition, body, ..
            } => {
                lint_tailless_expr(condition, known_seeds, stdlib_names, warnings);
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::While {
                condition, body, ..
            } => {
                lint_tailless_expr(condition, known_seeds, stdlib_names, warnings);
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::ForEach { iterable, body, .. } => {
                lint_tailless_expr(iterable, known_seeds, stdlib_names, warnings);
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Quantifier { body, .. } => {
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Contract {
                condition,
                then_body,
                else_body,
                ..
            } => {
                lint_tailless_expr(condition, known_seeds, stdlib_names, warnings);
                if let Some(body) = then_body {
                    lint_tailless_body(body, known_seeds, stdlib_names, warnings);
                }
                lint_tailless_body(else_body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Guard {
                condition, body, ..
            } => {
                lint_tailless_expr(condition, known_seeds, stdlib_names, warnings);
                lint_tailless_body(body, known_seeds, stdlib_names, warnings);
            }
            Stmt::Break { .. } | Stmt::ContinueLoop { .. } => {}
        }
    }
}

fn lint_tailless_expr(
    expr: &Expr,
    known_seeds: &HashSet<String>,
    stdlib_names: &HashSet<String>,
    warnings: &mut Vec<LintWarning>,
) {
    match &expr.kind {
        ExprKind::SumBranchValue { payload, .. } => {
            for payload in payload {
                lint_tailless_expr(&payload.value, known_seeds, stdlib_names, warnings);
            }
        }
        ExprKind::ProductConstruction { fields, .. } => {
            for field in fields {
                lint_tailless_expr(&field.expr, known_seeds, stdlib_names, warnings);
            }
        }
        ExprKind::Tensor(value) => lint_tailless_expr(value.input(), known_seeds, stdlib_names, warnings),
        ExprKind::MultiAxisSelection(value) => {
            for child in value.children() { lint_tailless_expr(child, known_seeds, stdlib_names, warnings); }
        }
        ExprKind::Call { args, func } => {
            if known_seeds.contains(func) && !stdlib_names.contains(func) && !has_call_tail(func) {
                warnings.push(LintWarning {
                    code: "E_CALL_TAIL_MISSING_AFTER_ARGS",
                    span: expr.span,
                    message: format!(
                        "호출 꼬리가 필요합니다. 예: '{}하기.' 또는 '{}기.'",
                        func, func
                    ),
                });
            }
            for arg in args {
                lint_tailless_expr(&arg.expr, known_seeds, stdlib_names, warnings);
            }
        }
        ExprKind::CollectionConstructor { elements } | ExprKind::TupleConstructor { elements } => {
            for element in elements {
                lint_tailless_expr(element, known_seeds, stdlib_names, warnings);
            }
        }
        ExprKind::SelectionPlace { target, index } => {
            lint_tailless_expr(target, known_seeds, stdlib_names, warnings);
            lint_tailless_expr(index, known_seeds, stdlib_names, warnings);
        }
        ExprKind::FieldAccess { target, .. } => {
            lint_tailless_expr(target, known_seeds, stdlib_names, warnings)
        }
        ExprKind::Infix { left, right, .. } => {
            lint_tailless_expr(left, known_seeds, stdlib_names, warnings);
            lint_tailless_expr(right, known_seeds, stdlib_names, warnings);
        }
        ExprKind::Suffix { value, .. } => {
            lint_tailless_expr(value, known_seeds, stdlib_names, warnings)
        }
        ExprKind::Thunk(body) => lint_tailless_body(body, known_seeds, stdlib_names, warnings),
        ExprKind::Eval { thunk, .. } => {
            lint_tailless_expr(thunk, known_seeds, stdlib_names, warnings)
        }
        ExprKind::SeedLiteral { body, .. } => {
            lint_tailless_expr(body, known_seeds, stdlib_names, warnings)
        }
        ExprKind::Pipe { stages } => {
            for stage in stages {
                lint_tailless_expr(stage, known_seeds, stdlib_names, warnings);
            }
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
            }
        }
        ExprKind::TemplateRender { inject, .. } => {
            for (_, value) in inject {
                lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
            }
        }
        ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                lint_tailless_expr(value, known_seeds, stdlib_names, warnings);
            }
        }
        ExprKind::Nuance { expr, .. } => {
            lint_tailless_expr(expr, known_seeds, stdlib_names, warnings)
        }
        ExprKind::Literal(_)
        | ExprKind::Var(_)
        | ExprKind::FlowValue
        | ExprKind::Assertion(_)
        | ExprKind::Formula(_)
        | ExprKind::Template(_)
        | ExprKind::StateMachine(_) => {}
    }
}

fn has_call_tail(name: &str) -> bool {
    let tails = ["기", "하기", "고", "하고", "면", "하면"];
    tails.iter().any(|tail| name.ends_with(tail))
}

#[cfg(test)]
mod declared_su_alias_tests {
    use super::*;

    #[test]
    fn declared_su_alias_is_scoped_to_type_children() {
        let source="(값:수) 값전달:수:셈씨 := { 값전달 <- 값. }. 자료:(수)차림 := 차림[1,2]. 수 := \"그대로\".";
        let mut program=crate::parse_v25(source,"declared-su.ddn").unwrap();
        canonicalize(&mut program).unwrap();
        let TopLevelItem::SeedDef(seed)=&program.items[0];
        assert_eq!(seed.params[0].type_ref,TypeRef::Named("셈수".into()));
        assert_eq!(seed.result_type,Some(TypeRef::Named("셈수".into())));
        let output=crate::parse_v25_and_normalize(source,"declared-su.ddn",crate::NormalizationLevel::N1).unwrap();
        assert!(output.contains("(셈수) 차림"),"{output}");
        assert!(output.contains("수 :="),"value identifier must remain: {output}");
        assert!(output.contains("\"그대로\""));
    }

    #[test]
    fn declared_su_alias_preserves_actual_user_type_owner() {
        let source="(내용:글) 수:묶음씨 := {}. (값:수) 그대로:수:셈씨 := { 그대로 <- 값. }.";
        let mut program=crate::parse_v25(source,"user-su.ddn").unwrap();
        canonicalize(&mut program).unwrap();
        let seed=program.items.iter().find_map(|item|match item {TopLevelItem::SeedDef(seed) if seed.canonical_name=="그대로"=>Some(seed),_=>None}).unwrap();
        assert_eq!(seed.params[0].type_ref,TypeRef::Named("수".into()));
        assert_eq!(seed.result_type,Some(TypeRef::Named("수".into())));
    }

    #[test]
    fn declared_su_alias_does_not_rewrite_legacy_or_constructor_names() {
        let mut context=CanonicalSeedContext {signatures:HashMap::new(),declared_types:HashSet::new(),su_alias:false,registered_sum_pins:None};
        let mut kind=TypeRef::Named("수".into());
        let span=Span{start:0,end:0};
        canonicalize_type_ref(&mut kind,span,&mut vec![],&context).unwrap();
        assert_eq!(kind,TypeRef::Named("수".into()));
        context.su_alias=true;
        let mut applied=TypeRef::Applied{name:"수".into(),args:vec![TypeRef::Named("수".into())]};
        canonicalize_type_ref(&mut applied,span,&mut vec![],&context).unwrap();
        assert_eq!(applied,TypeRef::Applied{name:"수".into(),args:vec![TypeRef::Named("셈수".into())]});
    }
}

fn canonicalize_algebra_term(term:&mut crate::ast::AlgebraTermSyntax,warnings:&mut Vec<LintWarning>) -> Result<(),ParseError> {
    use crate::ast::AlgebraTermKind;
    match &mut term.kind {
        AlgebraTermKind::Name(name)=>canonicalize_ident(name,term.span,warnings)?,
        AlgebraTermKind::Integer(_)=>{},
        AlgebraTermKind::Add(a,b)|AlgebraTermKind::Subtract(a,b)|AlgebraTermKind::Multiply(a,b)=>{
            canonicalize_algebra_term(a,warnings)?;canonicalize_algebra_term(b,warnings)?;
        }
        AlgebraTermKind::DivideByInteger(a,_)=>canonicalize_algebra_term(a,warnings)?,
    }
    Ok(())
}
