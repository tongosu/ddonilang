use ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile;

#[test]
fn bytes_parser_failures_keep_registered_codes_at_the_shared_frontdoor() {
    for (source, code) in [
        ("(바이트열=\"가\") UTF8NFC글로 보여주기.", "E_TYPE_MISMATCH"),
        ("바이트열{0x100} 보여주기.", "E_BYTES_LITERAL_INVALID"),
        ("(다른핀=\"A\") UTF8바이트열로 보여주기.", "E_BYTES_ARGUMENT_BINDING"),
    ] {
        let error = run_summary_from_supported_source_profile(
            source, "bytes-parse.ddn", Some(1), Some("v1-core-v25"),
        ).unwrap_err();
        let diagnostic: serde_json::Value = serde_json::from_str(&error).unwrap();
        assert_eq!(diagnostic["code"], code, "{error}");
        assert!(diagnostic["source_span"].is_object());
        assert!(diagnostic.get("output_log_texts").is_none());
    }
}

#[test]
fn bytes_and_text_overloads_are_disjoint_in_both_authored_orders() {
    for source in [
        include_str!("../../pack/v1_core_supported_ingress_w1/fixtures/vc06_overload.ddn"),
        include_str!("../../pack/v1_core_supported_ingress_w1/fixtures/vc06_overload_reversed.ddn"),
    ] {
        let result = run_summary_from_supported_source_profile(
            source, "overload.ddn", Some(1), Some("v1-core-v25"),
        ).unwrap();
        assert_eq!(result["output_log_texts"], serde_json::json!(["바이트", "글"]));
    }
}

#[test]
fn invalid_utf8_retains_the_owner_diagnostic_and_source_location() {
    let source = "\n(바이트열=바이트열{0xff}) UTF8NFC글로 보여주기.";
    let error = run_summary_from_supported_source_profile(
        source,
        "utf8.ddn", Some(1), Some("v1-core-v25"),
    ).unwrap_err();
    let diagnostic: serde_json::Value = serde_json::from_str(&error).unwrap();
    assert_eq!(diagnostic["code"], "E_BYTES_UTF8_INVALID");
    assert!(diagnostic.get("state_hash").is_none());
    assert_eq!(diagnostic["source_path"], "utf8.ddn");
    let start = diagnostic["source_span"]["start"].as_u64().unwrap() as usize;
    let end = diagnostic["source_span"]["end"].as_u64().unwrap() as usize;
    assert!(start >= 1 && source[start..end].contains("UTF8NFC글로"));
}

#[test]
fn explicit_utf8_bytes_use_the_shared_source_frontdoor() {
    for (source, expected) in [
        (
            "(바이트열=바이트열{0xe1, 0x84, 0x80, 0xe1, 0x85, 0xa1}) UTF8NFC글로 보여주기.",
            "가",
        ),
        ("(글=\"A\") UTF8바이트열로 보여주기.", "바이트열{0x41}"),
        (
            "(글=\"가\") UTF8바이트열로 보여주기.",
            "바이트열{0xe1, 0x84, 0x80, 0xe1, 0x85, 0xa1}",
        ),
        ("(글=\"\") UTF8바이트열로 보여주기.", "바이트열{}"),
        ("(바이트열=바이트열{}) UTF8NFC글로 보여주기.", ""),
        (
            "(바이트열=바이트열{0xef, 0xbb, 0xbf}) UTF8NFC글로 보여주기.",
            "\u{feff}",
        ),
        ("(바이트열{0x41} == 바이트열{0x41}) 보여주기.", "참"),
        ("(바이트열{0x41} == 바이트열{0x42}) 보여주기.", "거짓"),
        ("(바이트열{} == \"\") 보여주기.", "거짓"),
    ] {
        let result = run_summary_from_supported_source_profile(
            source,
            "bytes.ddn",
            Some(1),
            Some("v1-core-v25"),
        )
        .unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(result["output_log_texts"], serde_json::json!([expected]));
        let second = run_summary_from_supported_source_profile(
            source,
            "bytes.ddn",
            Some(1),
            Some("v1-core-v25"),
        )
        .unwrap();
        assert_eq!(result, second);
        let mut program = ddonirang_lang::parse_v25(source, "bytes.ddn").unwrap();
        ddonirang_lang::canonicalize(&mut program).unwrap();
        let canon = ddonirang_lang::normalize(&program, ddonirang_lang::NormalizationLevel::N1);
        let reparsed = ddonirang_lang::parse_v25(&canon, "bytes.ddn").unwrap();
        assert_eq!(
            canon,
            ddonirang_lang::normalize(&reparsed, ddonirang_lang::NormalizationLevel::N1)
        );
        let replay = run_summary_from_supported_source_profile(
            &canon,
            "bytes.ddn",
            Some(1),
            Some("v1-core-v25"),
        )
        .unwrap();
        assert_eq!(result["output_log_texts"], replay["output_log_texts"]);
        assert_eq!(result["state_hash"], replay["state_hash"]);
    }
}

#[test]
fn bytes_reject_malformed_values_types_and_guessed_bindings_atomically() {
    for source in [
        "(바이트열=바이트열{0xff}) UTF8NFC글로 보여주기.",
        "(바이트열=바이트열{0xc0,0xaf}) UTF8NFC글로 보여주기.",
        "(바이트열=바이트열{0xed,0xa0,0x80}) UTF8NFC글로 보여주기.",
        "(바이트열=바이트열{0xe1,0x84}) UTF8NFC글로 보여주기.",
        "(바이트열=\"가\") UTF8NFC글로 보여주기.",
        "(글=바이트열{0x41}) UTF8바이트열로 보여주기.",
        "(글=65) UTF8바이트열로 보여주기.",
        "(다른핀=\"A\") UTF8바이트열로 보여주기.",
        "(글=\"A\", 글=\"B\") UTF8바이트열로 보여주기.",
        "(글=\"A\") UTF8바이트열 보여주기.",
        "(글=\"A\") UTF8바이트열로기 보여주기.",
        "UTF8바이트열로 (글:글 --> 글) := 글.\n(글=\"A\") UTF8바이트열로 보여주기.",
        "바이트열{0x100} 보여주기.",
        "바이트열{0x1} 보여주기.",
        "바이트열{0X41} 보여주기.",
        "바이트열{65} 보여주기.",
        "바이트열{0x41,} 보여주기.",
    ] {
        let source = format!("\"게시되면 안 됨\" 보여주기.\n{source}");
        let run = || {
            run_summary_from_supported_source_profile(
                &source,
                "한글😀.ddn",
                Some(1),
                Some("v1-core-v25"),
            )
        };
        let error = run().expect_err(&source);
        assert_eq!(Err(error.clone()), run());
        let failure: serde_json::Value = serde_json::from_str(&error).unwrap();
        assert!(failure["code"].is_string(), "{source}: {error}");
        for key in [
            "output_log_texts",
            "state_hash",
            "trace_hash",
            "work_cost_receipt",
            "result",
        ] {
            assert!(failure.get(key).is_none(), "{source}: partial {key}");
        }
    }
}

#[test]
fn bytes_are_not_implicitly_text_list_or_number_at_callable_boundaries() {
    for (parameter_type, argument) in [
        ("글", "바이트열{0x41}"),
        ("수", "바이트열{0x41}"),
        ("차림", "바이트열{0x41}"),
        ("바이트열", "\"A\""),
        ("바이트열", "65"),
        ("바이트열", "(65) 차림"),
    ] {
        let source = format!(
            "확인 (값:{parameter_type} --> 글) := \"성공 아님\".\n\"게시 금지\" 보여주기.\n(값={argument}) 확인 보여주기."
        );
        let failure = run_summary_from_supported_source_profile(
            &source, "한글😀.ddn", Some(1), Some("v1-core-v25"),
        ).expect_err(&source);
        assert!(failure.contains("TYPE_MISMATCH"), "{source}: {failure}");
        let failure: serde_json::Value = serde_json::from_str(&failure).unwrap();
        assert!(failure.get("state_hash").is_none());
        assert!(failure.get("output_log_texts").is_none());
    }
}
