use ddonirang_lang::{ast::*, parse_v25_with_mode, ParseMode};
use ddonirang_lang::normalizer::{Normalizer, NormalizationLevel};

fn source(left: &str) -> String {
    format!("// 한글 🍎\r\n(미지수:나눔수~을, 근거:({left} =:= 7)보임~로)\r\n같음보이다{{같음보이기}}:({left} =:= 7)보임:셈씨 := {{ 같음보이다 <- 근거. }}.")
}

#[test]
fn algebra_goal_uses_shared_parser_and_preserves_term_ranges() {
    let raw=source("미지수 + 3");
    let program=parse_v25_with_mode(&raw,"대수.ddn",ParseMode::Strict).unwrap();
    let TopLevelItem::SeedDef(seed)=&program.items[0];
    let TypeRef::Applied {name,args}=&seed.params[1].type_ref else {panic!("보임 타입 필요")};
    assert_eq!(name,"보임");
    let [TypeRef::AlgebraEquality{left,right}]=args.as_slice() else {panic!("정적 등식 필요")};
    assert_eq!(&raw[left.span.start..left.span.end],"미지수 + 3");
    assert_eq!(&raw[right.span.start..right.span.end],"7");
    assert!(seed.proof_header.is_some());
    // Syntactic goals do not create a checked result or a fake proposition name.
    assert!(ddonirang_lang::proof_authoring::resolve_assumption_source(&raw,"대수.ddn").is_err());
}

#[test]
fn algebra_goal_canonical_form_is_reparseable_and_idempotent() {
    let raw=source("(미지수 * 미지수 + 1 / 2) / -2");
    let first=parse_v25_with_mode(&raw,"대수.ddn",ParseMode::Strict).unwrap();
    let text=Normalizer::new(NormalizationLevel::N1).normalize_program(&first);
    let second=parse_v25_with_mode(&text,"대수.ddn",ParseMode::Strict).unwrap();
    assert_eq!(text,Normalizer::new(NormalizationLevel::N1).normalize_program(&second));
}

#[test]
fn algebra_goal_rejects_variable_zero_and_approximate_denominators() {
    for denominator in ["미지수","0","2.0"] {
        let raw=source(&format!("미지수 / {denominator}"));
        let error=parse_v25_with_mode(&raw,"대수.ddn",ParseMode::Strict).unwrap_err();
        assert!(error.message.starts_with("E_PROOF_ALGEBRA_TERM:"),"{error:?}");
        assert_eq!(&raw[error.span.start..error.span.end],denominator);
    }
}

#[test]
fn existing_named_proof_and_structure_headers_keep_their_bindings() {
    let named="(P:세움) 두고\n(공개증거=본문증거:(P)보임~을)\n가정쓰다:(P)보임:셈씨 := { 가정쓰다 <- 본문증거. }.";
    for raw in [named,include_str!("structure_law_ko_surface_r1.ddn")] {
        let first=ddonirang_lang::proof_authoring::resolve_assumption_source(raw,"기존.ddn").unwrap();
        let program=parse_v25_with_mode(raw,"기존.ddn",ParseMode::Strict).unwrap();
        let normalized=Normalizer::new(NormalizationLevel::N1).normalize_program(&program);
        let reparsed=parse_v25_with_mode(&normalized,"기존.ddn",ParseMode::Strict).unwrap();
        assert_eq!(program.items.len(),reparsed.items.len());
        for (TopLevelItem::SeedDef(a),TopLevelItem::SeedDef(b)) in program.items.iter().zip(&reparsed.items) {
            assert_eq!(a.params.len(),b.params.len());
            assert_eq!(a.proof_header.as_ref().map(|h|h.propositions.len()),b.proof_header.as_ref().map(|h|h.propositions.len()));
        }
        let second=ddonirang_lang::proof_authoring::resolve_assumption_source(&normalized,"기존.ddn").unwrap();
        assert_eq!(first.len(),second.len());
        for (before,after) in first.iter().zip(&second) {
            assert_eq!(before.assumptions().len(),after.assumptions().len());
        }

    }
}

#[test]
fn unsupported_algebra_statements_point_to_the_authored_statement() {
    let base=include_str!("../../docs/EXAMPLES/대수보임_첫수업_R1/01_같은항빼기.ddn");
    for newline in ["\n","\r\n"] {
        for (raw,code) in [
            (format!("1 보여주기.\n{base}"),"E_PROOF_ROOT_UNSUPPORTED"),
            (format!("{base}\n1 보여주기."),"E_PROOF_ROOT_UNSUPPORTED"),
            (base.replace("    빼기 :=", "    1 보여주기.\n    빼기 :="),"E_PROOF_RULE_UNSUPPORTED"),
            (base.replace("    일차방정식보이다 <-", "    1 보여주기.\n    일차방정식보이다 <-"),"E_PROOF_RULE_UNSUPPORTED"),
        ] {
            let raw=format!("// 한글 🍎\n{raw}").replace("\r\n","\n").replace("\n",newline);
            let prepared=ddonirang_lang::proof_authoring::prepare_source(&raw,"진단.ddn").unwrap();
            let error=prepared.resolve_algebra().unwrap_err();
            assert!(error.message.starts_with(code),"{error:?}");
            let fragment=&raw[error.span.start..error.span.end];
            assert!(fragment=="1 보여주기." || fragment=="1 보여주기", "{fragment:?}");
        }
    }
}

#[test]
fn v4_division_requires_exact_static_name_pins_and_unshadowed_binding() {
    use ddonirang_lang::proof_authoring::prepare_source;
    let raw=include_str!("../../tool/tests/fixtures/v4_linear_steps/정상.ddn");
    assert!(prepare_source(raw,"수업.ddn").unwrap().resolve_algebra().is_ok());
    for source in [raw.replace("양변나누기","양쪽나누기"),
        raw.replace("근거=정리, 계수=3 / 2","근거=정리"),
        raw.replace("계수=3 / 2","다른=3 / 2"),
        raw.replace("계수=3 / 2","계수=3 / 2, 계수=3 / 2"),
        raw.replace("나누기 :=", "양변나누기 := 근거.\n나누기 :="),
        raw.replace("근거=정리, 계수", "근거=나누기, 계수")] {
        assert!(prepare_source(&source,"거부.ddn").and_then(|p|p.resolve_algebra()).is_err(),"{source}");
    }
    assert!(prepare_source("(근거=참, 계수=3) 양변나누기.","밖.ddn").is_err());
}
