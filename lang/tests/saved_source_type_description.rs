use ddonirang_lang::{ast::*, parse_v25_with_mode, ParseMode};
use ddonirang_lang::normalizer::{Normalizer, NormalizationLevel};

fn assert_descriptions_in_current_n1(raw: &str) {
    let program = parse_v25_with_mode(raw, "타입원문.ddn", ParseMode::Strict).unwrap();
    let n1 = Normalizer::new(NormalizationLevel::N1).normalize_program(&program);
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        for param in &seed.params {
            let description = Normalizer::source_type_description(&param.type_ref, program.source_epoch);
            assert!(!description.is_empty());
            assert!(n1.contains(&description), "{description} not in {n1}");
            assert!(!description.ends_with('\n'));
        }
        if let Some(result) = &seed.result_type {
            let description = Normalizer::source_type_description(result, program.source_epoch);
            assert!(n1.contains(&description), "{description} not in {n1}");
        }
    }
    let again = Normalizer::new(NormalizationLevel::N1).normalize_program(&program);
    assert_eq!(n1, again, "read-only description must not alter program normalization");
}

#[test]
fn description_uses_same_n1_owner_for_authored_algebra_goal() {
    let raw = "// 한글 🍎\r\n(미지수:나눔수~을, 근거:((미지수 * 미지수 + 1 / 2) / -2 =:= 7)보임~로) 같음보이다{같음보이기}:((미지수 * 미지수 + 1 / 2) / -2 =:= 7)보임:셈씨 := { 같음보이다 <- 근거. }.";
    assert_descriptions_in_current_n1(raw);
}

#[test]
fn description_uses_same_n1_owner_for_authored_structure_law() {
    assert_descriptions_in_current_n1(include_str!("structure_law_ko_surface_r1.ddn"));
}

#[test]
fn description_is_context_free_text_not_a_source_admission_token() {
    let invalid = TypeRef::StructureLaw { structure: "미해소구조".into(), member: "없는법칙".into() };
    assert_eq!(Normalizer::source_type_description(&invalid, SourceEpoch::V25), "미해소구조.없는법칙");
    // Producing text is intentionally possible for unresolved syntax. It must
    // never replace the separately required source-owned signature admission.
    let types = [TypeRef::Named("나눔수".into()), TypeRef::Tuple(vec![TypeRef::Named("바른수".into()), TypeRef::Infer]),
        TypeRef::Function { params: vec![TypeRef::Named("글".into())], result: Box::new(TypeRef::Named("참거짓".into())) }];
    for kind in types {
        assert_eq!(Normalizer::source_type_description(&kind, SourceEpoch::V25),
            Normalizer::source_type_description(&kind, SourceEpoch::PreV25));
    }
}
