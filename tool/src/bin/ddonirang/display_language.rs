//! CLI presentation preference only. Never forwarded to source/profile,
//! execution, machine schemas, identities, replay, or storage admission.
use std::ffi::OsString;

pub(super) const PREFERENCE_ENV: &str = "DDN_DISPLAY_LANGUAGE";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Language { Ko, Ja }

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Selection {
    pub(super) language: Language,
    pub(super) args: Vec<String>,
}

fn parse_language(value: &str) -> Result<Language, String> {
    match value {
        "ko" => Ok(Language::Ko),
        "ja" => Ok(Language::Ja),
        _ => Err("표시 말씨는 ko 또는 ja만 지원합니다. 원문 글판은 바뀌지 않습니다.".into()),
    }
}

/// Global options are accepted only before the command. After that boundary,
/// option-looking paths and all private request values belong to their owner.
pub(super) fn select(args: Vec<String>, preference: Option<OsString>) -> Result<Selection, String> {
    let mut cursor = 0;
    let mut explicit = None;
    while let Some(argument) = args.get(cursor) {
        let (name, inline) = argument.split_once('=').map_or((argument.as_str(), None),
            |(name, value)| (name, Some(value)));
        if !matches!(name, "--표시말씨" | "--display-language") { break; }
        if explicit.is_some() { return Err("표시 말씨 옵션은 언어별 철자를 합쳐 한 번만 지정하세요.".into()); }
        cursor += 1;
        let value = if let Some(value) = inline { value } else {
            let value = args.get(cursor).filter(|value| !value.starts_with("--"))
                .ok_or("표시 말씨 옵션 뒤에 ko 또는 ja를 지정하세요.")?;
            cursor += 1;
            value
        };
        explicit = Some(parse_language(value)?);
    }
    let language = match explicit {
        Some(language) => language,
        None => match preference {
            Some(value) => parse_language(&value.into_string()
                .map_err(|_| "표시 말씨 환경 설정은 ko 또는 ja 문자열이어야 합니다.")?)?,
            None => Language::Ko,
        },
    };
    Ok(Selection { language, args: args[cursor..].to_vec() })
}

#[derive(Clone, Copy)]
pub(super) enum ErrorPhase { Command, Operation, Output }

/// A redirected stderr can contain an existing JSON error. Keep those bytes
/// intact, without guessing the protocol by parsing the error message text.
pub(super) fn error_text(language: Language, terminal: bool, phase: ErrorPhase, original: &str) -> String {
    if language != Language::Ja || !terminal { return original.into(); }
    let guidance = match phase {
        ErrorPhase::Command => "コマンドと指定したオプションを確認してください。元の診断は次のとおりです。",
        ErrorPhase::Operation => "指定した操作を完了できませんでした。元の診断のコード・理由・位置を確認してください。",
        ErrorPhase::Output => "出力を完了できませんでした。元の診断は次のとおりです。",
    };
    format!("{guidance}\n{original}")
}

/// Only structured producer fields select a resource. No parsing of the
/// original message/JSON and no executable or saved-state authority.
pub(super) fn analysis_guidance(language: Language, terminal: bool,
    snapshot: &ddonirang_tool::source_analysis::SourceAnalysisSnapshot) -> Option<String> {
    if language != Language::Ja || !terminal || snapshot.outcome != "REJECTED" { return None; }
    let mut lines = Vec::new();
    for diagnostic in snapshot.diagnostics.iter().filter(|item| item.severity == "error") {
        let reason = match diagnostic.code.as_str() {
            "E_PARSE" => "構文を解析できません。括弧・演算子・文末の書き方を、元の診断の位置で確認してください。",
            "E_SOURCE_PROFILE_REQUIRED" => "ソースのプロファイルを明示してください。表示言語の設定とは別です。",
            "E_SOURCE_PROFILE_UNSUPPORTED" => "指定されたソースのプロファイルには対応していません。表示言語を変更しても有効にはなりません。",
            "E_CALL_REQUIRED_PIN_MISSING" => "必須の引数が不足しています。呼び出した関数の引数名と値を確認してください。",
            "E_CALL_TAIL_MISSING_STMT" => "呼び出し文の末尾を、現在の構文例と照合してください。",
            "E_BOIM_LEGACY_REMOVED" => "この旧式の書き方には対応していません。現在の例題の書式を確認してください。",
            _ => "[ko] この診断の理由は未翻訳です。下にある元の診断を確認してください。",
        };
        let position = if diagnostic.span.encoding == "utf-8-byte" {
            format!("（開始位置 {}、UTF-8バイト範囲 {}..{}）", diagnostic.span.start, diagnostic.span.start, diagnostic.span.end)
        } else { "（位置の形式は元の診断を参照）".into() };
        lines.push(format!("{}: {reason}{position}", diagnostic.code));
    }
    (!lines.is_empty()).then(|| lines.join("\n"))
}

pub(super) fn error_text_with_analysis(language: Language, terminal: bool, phase: ErrorPhase, original: &str,
    analysis: Option<&ddonirang_tool::source_analysis::SourceAnalysisSnapshot>) -> String {
    if let Some(guidance) = analysis.and_then(|snapshot| analysis_guidance(language, terminal, snapshot)) {
        format!("{guidance}\n{original}")
    } else { error_text(language, terminal, phase, original) }
}

pub(super) fn help_text(language: Language, original: &str) -> String {
    if language == Language::Ko { return original.into(); }
    format!("또니랑 コマンドライン\n\
表示言語はソースの言語・実行結果・保存データを変更しません。\n\
表示言語: コマンドの前に --표시말씨 ko|ja または --display-language ko|ja を指定します。\n\
優先順位: 明示オプション → DDN_DISPLAY_LANGUAGE → 既存の韓国語表示。\n\
例:\n\
  ddonilang --display-language ja check source.ddn --source-profile v1-core-v25\n\
  ddonilang --display-language ja run source.ddn --source-profile v1-core-v25\n\
プログラムの出力とリダイレクトした診断は翻訳しません。端末では補助案内と元の診断を表示します。\n\
[ko] 以下は未翻訳の既存ヘルプです。コマンド・オプションの綴りと制限をそのまま示します。\n{original}")
}

pub(super) fn workbench_prompt(language: Language, terminal: bool) -> &'static str {
    if language == Language::Ja && terminal {
        "セルグリムのサーバーを終了するには、この画面で Enter を押してください。ブラウザーを閉じてもサーバーは動作し続けます。"
    } else {
        "셈그림 서버를 종료하려면 이 창에서 Enter를 누르세요. 브라우저 창을 닫아도 서버는 계속 실행됩니다."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn words(values: &[&str]) -> Vec<String> { values.iter().map(|value| (*value).into()).collect() }

    #[test]
    fn explicit_environment_and_existing_default_precedence() {
        assert_eq!(select(words(&["run", "학습.ddn"]), None).unwrap().language, Language::Ko);
        assert_eq!(select(words(&["run"]), Some("ja".into())).unwrap().language, Language::Ja);
        assert_eq!(select(words(&["--표시말씨", "ko", "run"]), Some("ja".into())).unwrap().language, Language::Ko);
        assert_eq!(select(words(&["--display-language=ja", "check"]), Some("unsupported".into())).unwrap(),
            Selection { language: Language::Ja, args: words(&["check"]) });
    }

    #[test]
    fn localized_spellings_are_one_singleton() {
        for values in [
            words(&["--표시말씨=ja", "--표시말씨=ja", "run"]),
            words(&["--표시말씨", "ja", "--display-language", "ko", "run"]),
            words(&["--display-language=ko", "--표시말씨=ko"]),
        ] { assert!(select(values, None).is_err()); }
    }

    #[test]
    fn global_options_never_scan_command_owned_paths_or_private_values() {
        for args in [words(&["run", "--표시말씨", "ja"]), words(&["작품자료", "원문전이", "--display-language=ja"]),
            words(&["check", "--display-language=ja.ddn"]), words(&["--", "--표시말씨", "ja"])] {
            assert_eq!(select(args.clone(), None).unwrap().args, args);
        }
        assert_eq!(select(words(&["--표시말씨=ja", "run", "--display-language=ko"]), None).unwrap().args,
            words(&["run", "--display-language=ko"]));
    }

    #[test]
    fn invalid_effective_preferences_never_fall_back_silently() {
        for args in [words(&["--표시말씨"]), words(&["--표시말씨", "--help"]),
            words(&["--표시말씨="]), words(&["--표시말씨=JA"]), words(&["--display-language=ja "])] {
            assert!(select(args, None).is_err());
        }
        assert!(select(words(&["run"]), Some("en".into())).is_err());
    }

    #[test]
    fn redirected_json_and_utf8_diagnostics_are_exact_originals() {
        for original in ["{\"code\":\"E_실패\",\"range\":{\"start\":21,\"end\":25}}", "E_PARSE 학습.ddn:2:9\r\n원문 <값>"] {
            assert_eq!(error_text(Language::Ja, false, ErrorPhase::Operation, original).as_bytes(), original.as_bytes());
            assert_eq!(error_text(Language::Ko, true, ErrorPhase::Command, original).as_bytes(), original.as_bytes());
        }
    }

    #[test]
    fn terminal_guidance_keeps_code_original_and_position_as_suffix() {
        let original = "E_PARSE 학습.ddn:2:9\r\n원문 <값>";
        let shown = error_text(Language::Ja, true, ErrorPhase::Operation, original);
        assert!(shown.starts_with("指定した操作"));
        assert!(shown.ends_with(original));
        assert_eq!(shown.matches("E_PARSE").count(), 1);
    }

    #[test]
    fn help_marks_untranslated_material_and_prompt_preserves_redirects() {
        let original = "사용법: ddonilang run --source-profile v1-core-v25\n";
        assert_eq!(help_text(Language::Ko, original), original);
        let shown = help_text(Language::Ja, original);
        assert!(shown.contains("[ko]")); assert!(shown.ends_with(original));
        assert!(workbench_prompt(Language::Ja, true).contains("Enter"));
        assert_eq!(workbench_prompt(Language::Ja, false), workbench_prompt(Language::Ko, true));
    }
}


#[cfg(test)]
mod typed_analysis_tests {
    use super::*;
    use ddonirang_tool::source_frontdoor_workflow::validate_supported_source_for_execution;
    use std::path::Path;

    #[test]
    fn typed_display_uses_actual_code_and_span_without_changing_original_json() {
        let error = validate_supported_source_for_execution("합 := (2 + ).", Path::new("한글.ddn"), "v1-core-v25").unwrap_err();
        let analysis = error.analysis().unwrap();
        let shown = error_text_with_analysis(Language::Ja, true, ErrorPhase::Operation, error.message(), Some(analysis));
        assert!(shown.contains("E_PARSE: 構文を解析できません"));
        assert!(shown.contains(&format!("開始位置 {}", analysis.diagnostics[0].span.start)));
        assert!(shown.contains(&format!("{}..{}", analysis.diagnostics[0].span.start, analysis.diagnostics[0].span.end)));
        assert!(shown.ends_with(error.message()));
        assert_eq!(error_text_with_analysis(Language::Ja, false, ErrorPhase::Operation, error.message(), Some(analysis)), error.message());
        assert_eq!(error_text_with_analysis(Language::Ko, true, ErrorPhase::Operation, error.message(), Some(analysis)), error.message());
    }

    #[test]
    fn opaque_message_never_selects_a_code_specific_translation() {
        let original = "{\"diagnostics\":[{\"code\":\"E_PARSE\"}]}";
        let shown = error_text_with_analysis(Language::Ja, true, ErrorPhase::Operation, original, None);
        assert!(shown.starts_with("指定した操作"));
        assert!(!shown.contains("構文を解析できません"));
        assert!(shown.ends_with(original));
    }

    #[test]
    fn unknown_codes_and_coordinates_are_explicit_and_accepted_is_not_an_error() {
        let error = validate_supported_source_for_execution("合 := (2 + ).", Path::new("sample.ddn"), "v1-core-v25").unwrap_err();
        let mut analysis = error.analysis().unwrap().clone();
        analysis.diagnostics[0].code = "E_FUTURE_DISPLAY_TEST".into();
        analysis.diagnostics[0].span.encoding = "unsupported-test-coordinate";
        let shown = analysis_guidance(Language::Ja, true, &analysis).unwrap();
        assert!(shown.contains("[ko]")); assert!(!shown.contains("UTF-8"));
        analysis.outcome = "ACCEPTED";
        assert!(analysis_guidance(Language::Ja, true, &analysis).is_none());
    }
    #[test]
    fn required_pin_guidance_never_invents_a_tax_specific_owner(){
        let error=validate_supported_source_for_execution("合 := (2 + ).",Path::new("other.ddn"),"v1-core-v25").unwrap_err();
        let mut analysis=error.analysis().unwrap().clone();
        // This common code carries no pin owner. The presentation resource
        // must remain valid for another function, not just the tax example.
        analysis.diagnostics[0].code="E_CALL_REQUIRED_PIN_MISSING".into();
        let original="E_CALL_REQUIRED_PIN_MISSING: 두배하다의 값 누락";
        let shown=error_text_with_analysis(Language::Ja,true,ErrorPhase::Operation,original,Some(&analysis));
        assert!(shown.contains("必須の引数"));assert!(!shown.contains("金額"));assert!(!shown.contains("税額"));assert!(shown.ends_with(original));
        assert_eq!(error_text_with_analysis(Language::Ko,true,ErrorPhase::Operation,original,Some(&analysis)),original);
        assert_eq!(error_text_with_analysis(Language::Ja,false,ErrorPhase::Operation,original,Some(&analysis)),original);
    }

}
