//! 원문 분석 facts만 공급한다. 의미 정체·PURE·수학 참여·Proof 발급기가 아니다.
use super::super::RegistrationDraft;
use super::{fail, original_site, typeproof::StaticTypeOwner, Site, Source};
use ddonirang_lang::{ExprKind, Literal, SeedKind, Span, Stmt, TopLevelItem};

/// 위치는 선택용일 뿐 DefinitionIdentity가 아니다. 외부 생성/필드 변경/serde 없음.
#[derive(Debug, PartialEq, Eq)]
pub(in crate::authored_project::registration) struct SourceFacts {
    source_scope: String,
    source_mode: &'static str,
    captures: Vec<(String, [u8; 32])>,
    definition_site: Site,
    pin_site: Site,
    pin_spelling: String,
    roles: Vec<String>,
    source_fragment: String,
    input_type: Vec<u8>,
    result_type: Vec<u8>,
    unresolved_operations: Vec<(usize, usize)>,
}

impl SourceFacts {
    /// 같은 원문 owner와 선택을 다시 검사한다. 성공해도 분석 facts 재검증일 뿐이다.
    pub(in crate::authored_project::registration) fn verify(
        &self,
        source: &Source,
        nominals: &RegistrationDraft,
        selected: Span,
    ) -> Result<(), String> {
        let current = analyze(source, nominals, selected)?;
        if self != &current {
            return Err(fail("함수 분석의 원문·정의·핀 결속이 달라졌습니다"));
        }
        Ok(())
    }
}

pub(super) fn analyze(
    source: &Source,
    nominals: &RegistrationDraft,
    selected: Span,
) -> Result<SourceFacts, String> {
    // 실제 공통 owner가 nominal/source와 정적 프로그램을 검사한다.
    // Sum 형식의 정체가 필요한 source는 빈 임시 row로 만들지 않고 기존 owner가 거부한다.
    let owner = StaticTypeOwner::from_source(source, nominals, &[])?;
    let program = owner.checked_program();
    let mut selected_seeds = program
        .items
        .iter()
        .filter_map(|TopLevelItem::SeedDef(seed)| (seed.span == selected).then_some(seed));
    let seed = selected_seeds
        .next()
        .ok_or_else(|| fail("선택한 실제 함수 원문이 없습니다"))?;
    if selected_seeds.next().is_some() {
        return Err(fail("선택한 함수 원문이 유일하지 않습니다"));
    }
    if !matches!(seed.seed_kind, SeedKind::Semssi | SeedKind::ValueFunc)
        || !seed.modifiers.is_empty()
        || seed.params.len() != 1
    {
        return Err(fail("명시된 단항 순수 본문 분석 범위가 아닙니다"));
    }
    let pin = &seed.params[0];
    if pin.optional || pin.default_value.is_some() || pin.pin_name == seed.canonical_name {
        return Err(fail("필수 단항 입력과 결과 자리가 구별되어야 합니다"));
    }
    let input = owner.resolve(&pin.type_ref, pin.span)?;
    let result = owner.resolve(
        seed.result_type
            .as_ref()
            .ok_or_else(|| fail("명시 결과씨가 없습니다"))?,
        seed.span,
    )?;
    if !input.is_rational_scalar() || !result.is_rational_scalar() {
        return Err(fail("입력과 결과의 실제 나눔수 형식이 필요합니다"));
    }
    let body = seed
        .body
        .as_ref()
        .ok_or_else(|| fail("실제 함수 본문이 없습니다"))?;
    let value = match body.stmts.as_slice() {
        [Stmt::Return { value, .. }] => value,
        [Stmt::Mutate {
            target,
            value,
            deferred: false,
            ..
        }] if matches!(&target.kind, ExprKind::Var(name) if name == &seed.canonical_name) => value,
        _ => {
            return Err(fail(
                "단일 결과식 외의 효과·선언·분기 본문은 지원하지 않습니다",
            ))
        }
    };
    let mut pending = vec![value];
    let mut operations = Vec::new();
    let mut visited = 0usize;
    while let Some(expr) = pending.pop() {
        visited += 1;
        if visited > 100_000 {
            return Err(fail("함수 분석 한도를 넘었습니다. 수학적 반례가 아닙니다"));
        }
        match &expr.kind {
            // 단일 입력뿐이고 local 선언/중첩 binder/call은 모두 거부한다.
            // 이 철자 검사는 AST 지역 참조의 범위 검사이며 semantic binder 발급이 아니다.
            ExprKind::Var(name) if name == &pin.pin_name => (),
            ExprKind::Literal(Literal::Int(_) | Literal::Fixed64(_))
                if expr.numeric_literal_source.is_some() =>
            {
                ()
            }
            ExprKind::Infix { left, op, right } if matches!(op.as_str(), "+" | "-" | "*" | "/") => {
                operations.push((expr.span.start, expr.span.end));
                pending.push(right);
                pending.push(left);
            }
            _ => return Err(fail("다른 변수·호출·효과 또는 미지원 본문 표현입니다")),
        }
    }
    // 연산 문자열만으로 타입/의미 해소 완료를 주장하지 않는다. 위치를 미해결 목록에 보존.
    Ok(SourceFacts {
        source_scope: source.scope().into(),
        source_mode: source.mode(),
        captures: source
            .captures()
            .iter()
            .map(|(path, capture)| (path.clone(), *capture.identity()))
            .collect(),
        definition_site: original_site(source, seed.span)?,
        pin_site: original_site(source, pin.span)?,
        pin_spelling: pin.pin_name.clone(),
        roles: pin.josa_list.clone(),
        source_fragment: program
            .origin
            .source
            .get(seed.span.start..seed.span.end)
            .ok_or_else(|| fail("함수 원문의 UTF-8 범위가 잘못됐습니다"))?
            .into(),
        input_type: input.encode()?,
        result_type: result.encode()?,
        unresolved_operations: operations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source(text: &str) -> Source {
        try_source(text).unwrap()
    }
    fn try_source(text: &str) -> Result<Source, String> {
        use crate::authored_project::{ProjectArtifact, SourceFile, SourceSnapshot};
        let root = "(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.";
        let project = ProjectArtifact::create(
            "v1-core-v25".into(),
            SourceSnapshot {
                root_path: Some("root.ddn".into()),
                entry_path: "main.ddn".into(),
                files: vec![
                    SourceFile {
                        path: "root.ddn".into(),
                        text: root.into(),
                    },
                    SourceFile {
                        path: "main.ddn".into(),
                        text: format!("{text}\n실행:움직씨 := {{}}."),
                    },
                ],
            },
            String::new(),
        )
        .map_err(|e| e.to_string())?;
        Ok(Source::saved(
            &crate::project_revision_envelope_supply::admit_fresh_precanon_authored_source(
                project.save().map_err(|e| e.to_string())?.as_bytes(),
            )?,
        ))
    }
    fn draft(source: &Source) -> RegistrationDraft {
        super::super::super::prepare_from_source_view(
            &super::super::super::SavedRegistrationSource::precanon(source.saved_owner()),
            None,
            &mut |raw| getrandom::fill(raw).map_err(|_| fail("등록 난수 실패")),
        )
        .unwrap()
    }
    fn select(source: &Source, index: usize) -> Span {
        let program = source.checked_program().unwrap();
        let TopLevelItem::SeedDef(seed) = &program.items[index];
        seed.span
    }
    const FEE: &str = "요금구하다 (사용량:나눔수 --> 나눔수) := 10 * 사용량.";
    #[test]
    fn explicit_public_pin_local_binder_surface_remains_unsupported() {
        let raw = "기본요금구하다 (사용량=양:나눔수~을 --> 나눔수) := 10 * 양.";
        let error = ddonirang_lang::parse_frontdoor_with_mode(
            raw,
            "alias.ddn",
            ddonirang_lang::ParseMode::SuccessorDefault,
        )
        .unwrap_err();
        assert!(raw.get(error.span.start..error.span.end).is_some());
        println!(
            "미지원 원문={raw} 범위={}..{} 진단={}",
            error.span.start, error.span.end, error.message
        );
    }
    #[test]
    fn actual_source_facts_revalidate_without_issuing_math_authority() {
        let source = source(FEE);
        let nominal = draft(&source);
        let at = select(&source, 0);
        let facts = super::super::analyze_math_callable_source(&source, &nominal, at).unwrap();
        facts.verify(&source, &nominal, at).unwrap();
        assert_eq!(facts.pin_spelling, "사용량");
        assert!(facts.roles.is_empty());
        assert!(!facts.unresolved_operations.is_empty());
    }
    #[test]
    fn rejects_source_substitution_and_other_definition() {
        let source_a = source(&format!(
            "{FEE}\n다른요금구하다 (사용량:나눔수 --> 나눔수) := 10 * 사용량."
        ));
        let nominal_a = draft(&source_a);
        let at = select(&source_a, 0);
        let facts = analyze(&source_a, &nominal_a, at).unwrap();
        assert!(facts
            .verify(&source_a, &nominal_a, select(&source_a, 1))
            .is_err());
        for text in [
            FEE.to_string(),
            FEE.replace("10", "20"),
            FEE.replace("사용량", "양"),
        ] {
            let changed = source(&text);
            let nominal = draft(&changed);
            assert!(facts
                .verify(&changed, &nominal, select(&changed, 0))
                .is_err());
        }
    }
    #[test]
    fn rejects_non_unary_wrong_type_default_and_external_reference() {
        for text in [
            "요금구하다 (사용량:나눔수, 더함:나눔수 --> 나눔수) := 사용량 + 더함.",
            "요금구하다 (사용량:글 --> 글) := 사용량.",
            "(사용량:나눔수~을 = 1) 요금구하다:나눔수:셈씨 := { 요금구하다 <- 사용량. }.",
            "요금구하다 (사용량:나눔수 --> 나눔수) := 외부값.",
        ] {
            let source = match try_source(text) {
                Ok(source) => source,
                Err(error) => {
                    println!("공통 원문 거부: {error}");
                    continue;
                }
            };
            let nominal = draft(&source);
            // Actual common checker may reject before this producer.
            if let Ok(program) = source.checked_program() {
                let TopLevelItem::SeedDef(seed) = &program.items[0];
                assert!(analyze(&source, &nominal, seed.span).is_err(), "{text}");
            }
        }
    }
}
