use ddonirang_lang::ddl::{DdlRegistryEntry, DdlWave1Context};
use ddonirang_lang::{canonicalize_ddl_wave1, parse_ddl_wave1, parse_v25, OmittedPayload};

const WAVE1_LF: &str = "#!한국어\n원주율:셈수 :={...}.\n\n#!日本語\n円周率:数値 :={...}.\n";

fn error_code(source: &str, path: &str) -> &'static str {
    parse_ddl_wave1(source, path)
        .expect_err("expected Wave 1 DDL rejection")
        .code
}

fn wave1_entries() -> Vec<DdlRegistryEntry> {
    vec![
        DdlRegistryEntry {
            header: "#!\u{D55C}\u{AD6D}\u{C5B4}",
            profile_identity: "language-profile:ko",
            localized_name: "\u{C6D0}\u{C8FC}\u{C728}",
            localized_type: "\u{C148}\u{C218}",
            definition_identity: "definition:math.pi",
            type_identity: "type:number",
        },
        DdlRegistryEntry {
            header: "#!\u{65E5}\u{672C}\u{8A9E}",
            profile_identity: "language-profile:ja",
            localized_name: "\u{5186}\u{5468}\u{7387}",
            localized_type: "\u{6570}\u{5024}",
            definition_identity: "definition:math.pi",
            type_identity: "type:number",
        },
    ]
}

fn context_error(context: DdlWave1Context) -> &'static str {
    ddonirang_lang::ddl::parse_ddl_wave1_with_context(WAVE1_LF, "surface.ddl", &context)
        .expect_err("corrupt internal registry must reject the whole document")
        .code
}

#[test]
fn ddl_w1_positive_extensions_identity_spans_and_canon() {
    let ddl = parse_ddl_wave1(WAVE1_LF, "surface.ddl").expect(".ddl parses");
    let olgim = parse_ddl_wave1(WAVE1_LF, "surface.옮김").expect(".옮김 parses");
    assert_eq!(ddl.extension, ddonirang_lang::DdlExtension::Ddl);
    assert_eq!(olgim.extension, ddonirang_lang::DdlExtension::Olgim);
    assert_eq!(ddl.sections.len(), 2);
    assert_eq!(
        ddl.sections[0].language_profile_identity,
        "language-profile:ko"
    );
    assert_eq!(
        ddl.sections[1].language_profile_identity,
        "language-profile:ja"
    );
    assert_eq!(
        ddl.sections[0].projection.definition_identity,
        ddl.sections[1].projection.definition_identity
    );
    assert_eq!(
        ddl.sections[0].projection.type_identity,
        ddl.sections[1].projection.type_identity
    );
    assert_eq!(
        ddl.sections[0].projection.omitted_payload,
        OmittedPayload::ValueInitializer
    );
    assert_eq!(
        &WAVE1_LF[ddl.sections[0].header_span.start..ddl.sections[0].header_span.end],
        "#!한국어"
    );
    assert_eq!(
        &WAVE1_LF[ddl.sections[1].header_span.start..ddl.sections[1].header_span.end],
        "#!日本語"
    );
    assert_eq!(canonicalize_ddl_wave1(&ddl), canonicalize_ddl_wave1(&olgim));
}

#[test]
fn ddl_w1_canon_is_lf_and_byte_stable_for_crlf_input() {
    let crlf = WAVE1_LF.replace('\n', "\r\n");
    let first = canonicalize_ddl_wave1(&parse_ddl_wave1(&crlf, "surface.옮김").unwrap());
    let second = canonicalize_ddl_wave1(&parse_ddl_wave1(&first, "surface.옮김").unwrap());
    assert!(!first.contains('\r'));
    assert_eq!(first, WAVE1_LF);
    assert_eq!(first, second);
}

#[test]
fn ddl_w1_rejects_unregistered_duplicate_and_generic_profiles() {
    assert_eq!(
        error_code(
            "#!English\npi:number :={...}.\n\n#!日本語\n円周率:数値 :={...}.\n",
            "x.ddl"
        ),
        "E_DDL_PROFILE_UNKNOWN"
    );
    assert_eq!(
        error_code(
            "#!한국어\n원주율:셈수 :={...}.\n\n#!한국어\n원주율:셈수 :={...}.\n",
            "x.ddl"
        ),
        "E_DDL_PROFILE_DUPLICATE"
    );
    assert_eq!(
        error_code(
            "#!qu\nx:number :={...}.\n\n#!日本語\n円周率:数値 :={...}.\n",
            "x.ddl"
        ),
        "E_DDL_GENERIC_QUECHUA_FORBIDDEN"
    );
    assert_eq!(
        error_code(
            "#!Quechua\nx:number :={...}.\n\n#!日本語\n円周率:数値 :={...}.\n",
            "x.ddl"
        ),
        "E_DDL_GENERIC_QUECHUA_FORBIDDEN"
    );
}

#[test]
fn ddl_w1_rejects_header_position_and_section_cardinality_errors() {
    assert_eq!(
        error_code(
            "\n#!한국어\n원주율:셈수 :={...}.\n\n#!日本語\n円周率:数値 :={...}.\n",
            "x.ddl"
        ),
        "E_DDL_FIRST_HEADER_NOT_BYTE_ZERO"
    );
    assert_eq!(
        error_code(
            " #!한국어\n원주율:셈수 :={...}.\n\n#!日本語\n円周率:数値 :={...}.\n",
            "x.ddl"
        ),
        "E_DDL_FIRST_HEADER_NOT_BYTE_ZERO"
    );
    assert_eq!(
        error_code(
            "#! 한국어\n원주율:셈수 :={...}.\n\n#!日本語\n円周率:数値 :={...}.\n",
            "x.ddl"
        ),
        "E_DDL_PROFILE_UNKNOWN"
    );
    assert_eq!(
        error_code(
            "#!한국어 extra\n원주율:셈수 :={...}.\n\n#!日本語\n円周率:数値 :={...}.\n",
            "x.ddl"
        ),
        "E_DDL_PROFILE_UNKNOWN"
    );
    assert_eq!(
        error_code("#!한국어\n원주율:셈수 :={...}.\n", "x.ddl"),
        "E_DDL_SECTION_COUNT"
    );
    assert_eq!(error_code("#!한국어\n원주율:셈수 :={...}.\n\n#!日本語\n円周率:数値 :={...}.\n\n#!한국어\n원주율:셈수 :={...}.\n", "x.ddl"), "E_DDL_SECTION_COUNT");
}

#[test]
fn ddl_w1_does_not_promote_nested_string_or_comment_headers() {
    assert_eq!(
        error_code("#!한국어\n원주율:셈수 :={...}.\n// #!日本語\n", "x.ddl"),
        "E_DDL_SECTION_COUNT"
    );
    assert_eq!(
        error_code("#!한국어\n원주율:셈수 :={...}.\n\"#!日本語\"\n", "x.ddl"),
        "E_DDL_SECTION_COUNT"
    );
    assert_eq!(
        error_code("#!한국어\n{\n#!日本語\n}\n", "x.ddl"),
        "E_DDL_SECTION_COUNT"
    );
}

#[test]
fn ddl_w1_rejects_omission_aliases_execution_and_non_ddl_extension() {
    for malformed in [":={…}", ":={ }", ":={value}", ":={}", ":= {...}"] {
        let source =
            format!("#!한국어\n원주율:셈수 {malformed}.\n\n#!日本語\n円周率:数値 :={{...}}.\n");
        assert_eq!(error_code(&source, "x.ddl"), "E_DDL_OMITTED_VALUE_SHAPE");
    }
    assert_eq!(
        error_code(
            "#!한국어\n원주율:셈수 := 3.\n\n#!日本語\n円周率:数値 :={...}.\n",
            "x.ddl"
        ),
        "E_DDL_OMITTED_VALUE_SHAPE"
    );
    assert_eq!(error_code(WAVE1_LF, "x.ddn"), "E_DDL_EXTENSION_UNSUPPORTED");
}

#[test]
fn ddl_w1_rejects_registry_surface_and_type_mismatch_without_partial_document() {
    assert_eq!(
        error_code(
            "#!한국어\n원주율:다른씨 :={...}.\n\n#!日本語\n円周率:数値 :={...}.\n",
            "x.ddl"
        ),
        "E_DDL_OMITTED_VALUE_SHAPE"
    );
    assert_eq!(
        error_code(
            "#!한국어\n다른값:셈수 :={...}.\n\n#!日本語\n円周率:数値 :={...}.\n",
            "x.ddl"
        ),
        "E_DDL_OMITTED_VALUE_SHAPE"
    );
}

#[test]
fn ddl_w1_p06_uses_utf8_byte_spans_for_every_document_surface() {
    let document = parse_ddl_wave1(WAVE1_LF, "surface.ddl").unwrap();
    let korean_header = "#!\u{D55C}\u{AD6D}\u{C5B4}";
    let japanese_header = "#!\u{65E5}\u{672C}\u{8A9E}";
    let korean_declaration = "\u{C6D0}\u{C8FC}\u{C728}:\u{C148}\u{C218} :={...}.";
    let japanese_declaration = "\u{5186}\u{5468}\u{7387}:\u{6570}\u{5024} :={...}.";

    assert_eq!(
        &WAVE1_LF[document.source_span.start..document.source_span.end],
        WAVE1_LF
    );
    assert_eq!(
        &WAVE1_LF[document.sections[0].header_span.start..document.sections[0].header_span.end],
        korean_header
    );
    assert_eq!(
        &WAVE1_LF[document.sections[0].content_span.start..document.sections[0].content_span.end],
        format!("\n{korean_declaration}\n\n")
    );
    assert_eq!(
        &WAVE1_LF[document.sections[0].projection.declaration_span.start
            ..document.sections[0].projection.declaration_span.end],
        korean_declaration
    );
    assert_eq!(
        &WAVE1_LF[document.sections[1].header_span.start..document.sections[1].header_span.end],
        japanese_header
    );
    assert_eq!(
        &WAVE1_LF[document.sections[1].content_span.start..document.sections[1].content_span.end],
        format!("\n{japanese_declaration}\n")
    );
    assert_eq!(
        &WAVE1_LF[document.sections[1].projection.declaration_span.start
            ..document.sections[1].projection.declaration_span.end],
        japanese_declaration
    );
}

#[test]
fn ddl_w1_p10_retains_section_presentation_order_without_identity_drift() {
    let korean_header = "#!\u{D55C}\u{AD6D}\u{C5B4}";
    let japanese_header = "#!\u{65E5}\u{672C}\u{8A9E}";
    let korean_declaration = "\u{C6D0}\u{C8FC}\u{C728}:\u{C148}\u{C218} :={...}.";
    let japanese_declaration = "\u{5186}\u{5468}\u{7387}:\u{6570}\u{5024} :={...}.";
    let reversed = format!(
        "{japanese_header}\n{japanese_declaration}\n\n{korean_header}\n{korean_declaration}\n"
    );
    let standard = parse_ddl_wave1(WAVE1_LF, "standard.ddl").unwrap();
    let document = parse_ddl_wave1(&reversed, "reversed.ddl").unwrap();

    assert_eq!(
        document.sections[0].language_profile_identity,
        "language-profile:ja"
    );
    assert_eq!(
        document.sections[1].language_profile_identity,
        "language-profile:ko"
    );
    assert_eq!(
        document.sections[0].projection.definition_identity,
        document.sections[1].projection.definition_identity
    );
    assert_eq!(
        document.sections[0].projection.type_identity,
        document.sections[1].projection.type_identity
    );
    assert_eq!(
        document.sections[0].projection.definition_identity,
        standard.sections[0].projection.definition_identity
    );
    assert_eq!(
        document.sections[0].projection.type_identity,
        standard.sections[0].projection.type_identity
    );
    assert_eq!(canonicalize_ddl_wave1(&document), reversed);
}

#[test]
fn ddl_w1_rejects_missing_header_and_missing_or_inconsistent_translation_context() {
    let missing_header = "\u{C6D0}\u{C8FC}\u{C728}:\u{C148}\u{C218} :={...}.\n";
    assert_eq!(
        error_code(missing_header, "missing.ddl"),
        "E_DDL_FIRST_HEADER_REQUIRED"
    );

    let entries = wave1_entries();
    assert_eq!(
        context_error(DdlWave1Context::from_entries(vec![entries[0]])),
        "E_DDL_PROFILE_UNKNOWN"
    );

    let mut definition_mismatch = wave1_entries();
    definition_mismatch[1].definition_identity = "definition:math.tau";
    assert_eq!(
        context_error(DdlWave1Context::from_entries(definition_mismatch)),
        "E_DDL_DEFINITION_IDENTITY_MISMATCH"
    );

    let mut type_mismatch = wave1_entries();
    type_mismatch[1].type_identity = "type:integer";
    assert_eq!(
        context_error(DdlWave1Context::from_entries(type_mismatch)),
        "E_DDL_TYPE_IDENTITY_MISMATCH"
    );
}

#[test]
fn ddl_w1_rejects_mutable_private_local_and_omission_outside_value_projection() {
    for modifier in ["\u{BC14}\u{B014}", "\u{BE44}\u{AC1C}", "\u{C9C0}\u{C5ED}"] {
        let source = WAVE1_LF.replacen(
            "\u{C6D0}\u{C8FC}\u{C728}:\u{C148}\u{C218} :={...}.",
            &format!("{modifier} \u{C6D0}\u{C8FC}\u{C728}:\u{C148}\u{C218} :={{...}}."),
            1,
        );
        assert_eq!(error_code(&source, "x.ddl"), "E_DDL_OMITTED_VALUE_SHAPE");
    }

    assert!(parse_v25(WAVE1_LF, "ordinary.ddn").is_err());

    let type_schema = "#!\u{D55C}\u{AD6D}\u{C5B4}\n\u{ACF1}: {\n\u{C6D0}\u{C8FC}\u{C728}:\u{C148}\u{C218} :={...}.\n}\n\n#!\u{65E5}\u{672C}\u{8A9E}\n\u{5186}\u{5468}\u{7387}:\u{6570}\u{5024} :={...}.\n";
    assert_eq!(
        error_code(type_schema, "schema.ddl"),
        "E_DDL_PROJECTION_COUNT"
    );
}
