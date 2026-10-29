use ddonirang_lang::{Lexer, TokenKind};
use ddonirang_tool::name_binding::{bind_document, ByteRange, DefinitionKind, Resolution, UnsupportedReason};
use ddonirang_tool::name_binding::token_ranges::{
    analyze_name_tokens, DeclarationNameRange, DeclarationSyntaxKind, NameTokenOutcome,
};

const SOURCE: &str = "(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\n결과 := 21을 두배하기. 결과 보여주기.\n";
const PATH: &str = "한글-이름범위.ddn";

fn located(value: &DeclarationNameRange) -> ByteRange {
    match value { DeclarationNameRange::Located(range) => *range, other => panic!("{other:?}") }
}

#[test]
fn parameter_name_is_finer_than_unchanged_a1_whole_pin_range() {
    let report=analyze_name_tokens(SOURCE,PATH);
    assert_eq!(report.outcome(),&NameTokenOutcome::Available);
    let param=report.declarations().iter().find(|d|d.kind==DeclarationSyntaxKind::ParameterPin&&d.name=="값").unwrap();
    let name=located(&param.name_range);
    assert_eq!(&SOURCE[name.start..name.end],"값");
    assert_eq!(name.start,SOURCE.find("값:수").unwrap());
    assert!(name.end<param.syntax_range.end);
    let old=bind_document(SOURCE,PATH);
    let old_param=old.definitions.iter().find(|d|d.kind==DefinitionKind::Parameter&&d.name=="값").unwrap();
    assert_eq!(old_param.declaration,param.syntax_range);
    assert!(matches!(old_param.resolution,Resolution::Unsupported(_)));
    let seed=report.declarations().iter().find(|d|d.kind==DeclarationSyntaxKind::Seed&&d.name=="두배").unwrap();
    let seed_name=located(&seed.name_range);
    assert_eq!(&SOURCE[seed_name.start..seed_name.end],"두배");
    assert_eq!(seed_name.start,SOURCE.find("두배:수:셈씨").unwrap());
    assert!(seed.node_id.is_some()&&param.node_id.is_some());
}

#[test]
fn same_spelling_local_stays_separate_syntax_and_a1_refusal_is_preserved() {
    let source="(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\n(값:수~을) 세배:수:셈씨 := { 두배:수 := 값 * 3. 세배 <- 두배. }.\n결과 := 7을 세배하기. 결과 보여주기.\n";
    let report=analyze_name_tokens(source,PATH);
    assert_eq!(report.outcome(),&NameTokenOutcome::Available);
    let names:Vec<_>=report.declarations().iter().filter(|d|d.name=="두배").collect();
    assert!(names.iter().any(|d|d.kind==DeclarationSyntaxKind::Seed));
    assert!(names.iter().any(|d|d.kind==DeclarationSyntaxKind::StatementBinder));
    let local=names.iter().find(|d|d.kind==DeclarationSyntaxKind::StatementBinder).unwrap();
    let local_range=located(&local.name_range);
    assert_eq!(&source[local_range.start..local_range.end],"두배");
    assert_eq!(local_range.start,source.find("두배:수 :=").unwrap());
    let binding=bind_document(source,PATH);
    let callable=binding.definitions.iter().find(|d|d.kind==DefinitionKind::TopLevelCallable&&d.name=="두배").unwrap();
    let Resolution::Unsupported(sites)=&callable.resolution else {panic!("syntax ranges granted a binding")};
    assert!(sites.iter().any(|s|s.reason==UnsupportedReason::SameSpellingLocalBinding));
}

#[test]
fn unicode_crlf_comments_and_strings_do_not_become_identifier_occurrences() {
    let prefix="// 😀 값 두배\r\n";
    let source=format!("{prefix}{}\"두배 값\" 보여주기.\r\n",SOURCE.replace('\n',"\r\n"));
    let report=analyze_name_tokens(&source,PATH);
    assert_eq!(report.outcome(),&NameTokenOutcome::Available);
    let param=report.declarations().iter().find(|d|d.kind==DeclarationSyntaxKind::ParameterPin&&d.name=="값").unwrap();
    assert_eq!(located(&param.name_range).start,prefix.len()+1);
    let string_start=source.find('"').unwrap();
    let string_end=string_start+"\"두배 값\"".len();
    for token in report.tokens() {
        assert!(token.range.start>=prefix.len());
        assert!(!(token.range.start<string_end&&string_start<token.range.end));
        assert!(source.is_char_boundary(token.range.start)&&source.is_char_boundary(token.range.end));
        assert_eq!(&source[token.range.start..token.range.end],token.spelling);
    }
    assert_eq!(report.source_revision_identity(),format!("blake3:{}",blake3::hash(source.as_bytes()).to_hex()));
}

#[test]
fn byte_lookup_is_half_open_character_aligned_and_source_revision_bound() {
    let report=analyze_name_tokens(SOURCE,PATH);
    assert_eq!(report.outcome(),&NameTokenOutcome::Available);
    assert!(report.is_for_source(SOURCE));
    assert!(!report.is_for_source(&SOURCE.replace("21","22")));
    let start=SOURCE.find("두배:수:셈씨").unwrap();
    assert_eq!(report.token_at(start).unwrap().spelling,"두배");
    assert!(report.token_at(start+1).is_none());
    assert!(report.token_at(start+"두".len()).is_some());
    assert!(report.token_at(start+"두배".len()).is_none());
    assert!(report.token_at(SOURCE.len()).is_none());
    assert!(report.token_at(usize::MAX).is_none());
    let original=report.source_revision_identity().to_string();
    let changed=analyze_name_tokens(&SOURCE.replace("21","22"),PATH);
    assert_ne!(changed.source_revision_identity(),original);
    assert_eq!(report.source_revision_identity(),original);
}

#[test]
fn rejected_or_unfinished_document_never_publishes_valid_prefix_ranges() {
    for source in ["값 := 1.\n다른 := (", "값 := \"닫지 않은 글"] {
        let report=analyze_name_tokens(source,PATH);
        assert_eq!(report.outcome(),&NameTokenOutcome::SourceRejected);
        assert!(report.tokens().is_empty()&&report.declarations().is_empty());
        assert!(report.token_at(0).is_none());
        assert!(report.is_for_source(source));
    }
}

#[test]
fn shared_lexer_whole_call_tokens_are_not_invented_call_stem_bindings() {
    let report=analyze_name_tokens(SOURCE,PATH);
    assert_eq!(report.outcome(),&NameTokenOutcome::Available);
    let raw:Vec<_>=Lexer::new(SOURCE).tokenize().unwrap().into_iter().filter(|t|matches!(t.kind,TokenKind::Ident(_)))
        .map(|t|(t.raw,t.span.start,t.span.end)).collect();
    let supplied:Vec<_>=report.tokens().iter().map(|t|(t.spelling.clone(),t.range.start,t.range.end)).collect();
    assert_eq!(supplied,raw);
    let call=report.tokens().iter().find(|t|t.spelling=="두배하기").unwrap();
    assert_eq!(&SOURCE[call.range.start..call.range.end],"두배하기");
    assert_eq!(call.range.end-call.range.start,"두배하기".len());
    let binding=bind_document(SOURCE,PATH);
    let callable=binding.definitions.iter().find(|d|d.kind==DefinitionKind::TopLevelCallable&&d.name=="두배").unwrap();
    assert!(matches!(callable.resolution,Resolution::Exact(_)),"A1 existing exact scope changed");
}
