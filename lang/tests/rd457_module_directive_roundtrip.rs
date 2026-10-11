use ddonirang_lang::{
    canonicalize_module_directive_document, parse_module_directive_document,
    parse_module_directive_frontdoor_with_mode, ModuleDirectiveContext, ModuleDirectiveDocument,
    ModuleDirectiveKind, ParseMode,
};

fn context() -> ModuleDirectiveContext {
    ModuleDirectiveContext::default()
        .with_known_imports(["./lib/운동", "./lib/😀"])
        .with_public_definitions(["속도", "값"])
}

fn error(source: &str) -> String {
    parse_module_directive_document(source, "module.ddn", &context())
        .expect_err("case must fail closed")
        .message
}

#[test]
fn rd457_import_export_typed_ast_and_canon_roundtrip() {
    let source = "(운동=\"./lib/운동\") 쓰임.\n(속도=속도) 드러냄.\n값:셈수 := 1.\n";
    let document = parse_module_directive_document(source, "module.ddn", &context()).unwrap();
    assert_eq!(
        document.imports.as_ref().unwrap().kind,
        ModuleDirectiveKind::Import
    );
    assert_eq!(
        document.exports.as_ref().unwrap().kind,
        ModuleDirectiveKind::Export
    );
    assert_eq!(
        document.imports.as_ref().unwrap().entries[0].target,
        "./lib/운동"
    );
    assert_eq!(document.exports.as_ref().unwrap().entries[0].target, "속도");
    let first = canonicalize_module_directive_document(&document);
    let second = canonicalize_module_directive_document(
        &parse_module_directive_document(&first, "module.ddn", &context()).unwrap(),
    );
    assert_eq!(first, second);
    assert_eq!(first, source);
}

#[test]
fn rd457_shared_frontdoor_preserves_metadata_and_parses_v25_declaration_tail() {
    let source = "(운동=\"./lib/😀\") 쓰임.\n(속도=속도) 드러냄.\n값:셈수 := 1.\n";
    let result = parse_module_directive_frontdoor_with_mode(
        source,
        "frontdoor_module.ddn",
        ParseMode::Strict,
        &context(),
    )
    .expect("shared frontdoor must preserve directives and parse the declaration tail");

    assert_eq!(
        result.directives.imports.as_ref().unwrap().entries[0].target,
        "./lib/😀"
    );
    assert_eq!(
        result.directives.exports.as_ref().unwrap().entries[0].target,
        "속도"
    );
    // dae9389628f8 represents ordinary definitions as root statements, not seeds.
    let program = result.declaration_program.as_ref().unwrap();
    assert!(program.items.is_empty());
    assert_eq!(program.root_stmts.len(), 1);
    assert!(matches!(&program.root_stmts[0], ddonirang_lang::Stmt::Definition { name, .. } if name == "값"));
    assert_eq!(result.canonical_source, source);
}

#[test]
fn rd457_shared_frontdoor_fails_before_metadata_publication_on_invalid_v25_tail() {
    let error = parse_module_directive_frontdoor_with_mode(
        "(운동=\"./lib/운동\") 쓰임.\n값:셈수 := .\n",
        "invalid_tail.ddn",
        ParseMode::Strict,
        &context(),
    )
    .expect_err("invalid declaration tail must fail before a frontdoor result exists");

    assert!(!error.message.is_empty());
}

#[test]
fn rd457_shared_frontdoor_rejects_legacy_brace_without_conversion() {
    let error = parse_module_directive_frontdoor_with_mode(
        "쓰임 { 운동: \"./lib/운동\". }",
        "legacy_brace.ddn",
        ParseMode::Strict,
        &context(),
    )
    .expect_err("legacy brace source must not be converted by the shared frontdoor");

    assert!(error.message.starts_with("E_RD457_LEGACY_BRACE_DIRECTIVE"));
}

#[test]
fn rd457_preserves_utf8_byte_spans_when_import_target_contains_emoji() {
    let source = "(운동=\"./lib/😀\") 쓰임.\n(속도=속도) 드러냄.\n";
    let document = parse_module_directive_document(source, "emoji_target.ddn", &context()).unwrap();
    let import = document.imports.unwrap();
    let export = document.exports.unwrap();
    assert_eq!(import.entries[0].local_name, "운동");
    assert_eq!(import.entries[0].target, "./lib/😀");
    assert_eq!(
        &source[import.span.start..import.span.end],
        "(운동=\"./lib/😀\") 쓰임."
    );
    assert_eq!(export.span.start, source.find("(속도=속도)").unwrap());
    assert_eq!(
        &source[export.span.start..export.span.end],
        "(속도=속도) 드러냄."
    );
    assert_eq!(
        canonicalize_module_directive_document(&ModuleDirectiveDocument {
            imports: Some(import),
            exports: Some(export),
            declaration_source: String::new()
        }),
        source
    );
}

#[test]
fn rd457_rejects_legacy_duplicate_unknown_private_and_malformed_surfaces() {
    for legacy in [
        "쓰임 { 운동: \"./lib/운동\". }",
        "드러냄 { 속도. }",
        "공개 { 속도. }",
    ] {
        assert!(error(legacy).starts_with("E_RD457_LEGACY"), "{legacy}");
    }
    assert!(
        error("(운동=\"./lib/운동\") 쓰임. (힘=\"./lib/운동\") 쓰임.")
            .starts_with("E_RD457_IMPORT_DUPLICATE")
    );
    assert!(error("(속도=속도) 드러냄. (값=값) 드러냄.").starts_with("E_RD457_EXPORT_DUPLICATE"));
    assert!(error("(미지=\"./lib/missing\") 쓰임.").starts_with("E_RD457_IMPORT_TARGET_UNKNOWN"));
    assert!(
        error("(비공개=비공개) 드러냄.").starts_with("E_RD457_EXPORT_TARGET_PRIVATE_OR_UNKNOWN")
    );
    assert!(error("(*=\"./lib/운동\") 쓰임.").starts_with("E_RD457_ENTRY_NAME"));
    assert!(error("값 <- (운동=\"./lib/운동\") 쓰임.").starts_with("E_RD457_DIRECTIVE_EXPECTED"));
    assert!(error("(운동 \"./lib/운동\") 쓰임.").starts_with("E_RD457_ENTRY_EQUALS"));
    assert!(error("(운동=\"./lib/운동\") 쓰임").starts_with("E_RD457_DIRECTIVE_TERMINATOR"));
    assert!(error("(운동=\"./lib/운동\")").starts_with("E_RD457_DIRECTIVE_KEYWORD"));
}
