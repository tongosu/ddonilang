//! CLI presentation preference only. Never forwarded to source/profile,
//! execution, machine schemas, identities, replay, or storage admission.
use std::ffi::{OsStr, OsString};
use std::path::Path;

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
    select_for_executable(args, preference, None)
}

/// The executable name supplies only the last presentation default. Exact
/// comparison avoids normalization, fuzzy aliases, or executable authority.
pub(super) fn select_for_executable(
    args: Vec<String>, preference: Option<OsString>, executable: Option<&Path>,
) -> Result<Selection, String> {
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
            None => if executable.and_then(Path::file_name) == Some(OsStr::new("トニラン.exe")) {
                Language::Ja
            } else { Language::Ko },
        },
    };
    Ok(Selection { language, args: args[cursor..].to_vec() })
}


#[derive(serde::Deserialize)]
struct KoreanWords { id:String, codes:Vec<String>, contains:Vec<String>, category:String, what:String, why:String, how:String }
fn capture(pattern:&str,text:&str)->Option<String>{regex::Regex::new(pattern).ok()?.captures(text)?.get(1).map(|m|m.as_str().to_owned())}
fn named(value:Option<&str>)->Option<String>{let value=value?;if let Some(v)=value.strip_prefix("Named(\"").and_then(|s|s.strip_suffix("\")")){return Some(v.into());}if value.contains('(')||value.contains('{'){None}else{Some(value.into())}}
fn korean_words_details(code:&str,message:&str,expected:Option<&str>,actual:Option<&str>,pin:Option<&str>,fragment:Option<&str>)->Option<String>{
 let rows:Vec<KoreanWords>=serde_json::from_str(include_str!("diagnostic_words_ko.json")).ok()?;
 let row=rows.iter().find(|r|r.codes.iter().any(|c|c==code)&&r.contains.iter().all(|s|message.contains(s)))?;
 let mut what=row.what.clone();let mut facts:Vec<(&str,String)>=Vec::new();let m=message.lines().next().unwrap_or("");
 let mut expected=named(expected);let mut actual=named(actual);let mut pin=pin.map(str::to_owned);
 if row.id=="17"{expected=expected.or_else(||capture(r#"기대=Named\("([^"]+)"\)"#,m));actual=actual.or_else(||capture(r#"실제=Named\("([^"]+)"\)"#,m));pin=pin.or_else(||capture(r"핀=([^\s]+)",m));}
 if row.id=="30"{
  if let Some(pair)=regex::Regex::new(r"\(기대: ([^,]+), 실제: ([^)]+)\)").ok()?.captures(m){expected=expected.or_else(||Some(pair[1].into()));actual=actual.or_else(||Some(pair[2].into()));}
  if let Some(name)=capture(r"'([^']+)'의 재결속",m){facts.push(("이름",name));}
 }
 if row.id=="34"{expected=Some("수".into());actual=capture(r"차림 위치는 수여야 합니다: ([^\s]+)",m);pin=Some("자리 번호".into());}
 if matches!(row.id.as_str(),"17"|"30"|"34"){if let (Some(a),Some(b))=(&expected,&actual){what=format!("기대한 씨는 ‘{a}’이고 들어온 씨는 ‘{b}’입니다.");}}
 if row.id=="37"{pin=pin.or_else(||capture(r"알 수 없는 풀기 고정 핀: ([^\s]+)",m));}
 if let Some(p)=&pin{facts.push(("받는자리",p.clone()));}
 if row.id=="09"{if let Some(v)=capture(r"피연산자 형식이 맞지 않습니다: (.+?)(?: \(|$)",m){facts.push(("들어온 씨",v));}}
 if row.id=="07"{if let Some(name)=capture(r"정의되지 않은 이름입니다: ([^\s]+)",m){what=format!("‘{name}’라는 이름을 찾지 못했습니다.");}}
 if matches!(row.id.as_str(),"15"|"21"|"22"){
  let p=pin.clone().or_else(||capture(if row.id=="15"{r"필수 인자 '([^']+)'"}else{r"핀 '([^']+)'"},m));
  if let Some(p)=p{if pin.is_none(){facts.push(("받는자리",p.clone()));}if row.id=="21"{what=format!("받는자리 ‘{p}’를 찾지 못했습니다.");}if row.id=="22"{what=format!("받는자리 ‘{p}’에 값이 두 번 있습니다.");}}
 }
 if row.id=="16"{if let Some(v)=capture(r"조사 '([^']+)'",m){facts.push(("확인할 조사",v));}}
 if row.id=="25"{if let Some(v)=capture(r"`([^`]+)` 갈래가 선언되지",m){what=format!("선언에 없는 갈래입니다: ‘{v}’.");}}
 if row.id=="31"{if let Some(v)=capture(r"단위 '([^']+)'",m){facts.push(("확인할 단위",v));}}
 if row.id=="05"{if let Some(v)=fragment.filter(|s|!s.contains('\n')){what=format!("또니랑에서 쓰지 않는 기호입니다: ‘{v}’.");}}
 if row.id=="14"{if let Some(v)=capture(r"resource=runtime_builtin:([^,;\s]+)",m){facts.push(("확인할 호출",v));}}
 if row.id=="19"{if let Some(v)=capture(r"선언씨 ([^,\s)]+)",m){facts.push(("확인할 씨",v));}}
 if row.id=="33"{if let Some(v)=capture(r"위치 ([^\s]+)가 범위 밖",m){what=format!("자리 번호 ‘{v}’가 차림 범위를 벗어났습니다.");}}
 let mut lines=vec![if row.category.is_empty(){what}else{format!("{} · {what}",row.category)}];
 if !row.why.is_empty(){lines.push(format!("이유 · {}",row.why));}
 for (label,value) in facts{lines.push(format!("{label} · {value}"));}
 if !row.how.is_empty(){lines.push(format!("고침 제안 · {}",row.how));}
 if let Some(fragment)=fragment.filter(|s|!s.is_empty()){lines.push(format!("이 자리\n{fragment}\n^"));}
 Some(lines.join("\n"))
}
pub(super) fn korean_words(code:&str,message:&str)->Option<String>{korean_words_details(code,message,None,None,None,None)}
fn korean_value(value:&serde_json::Value)->Option<String>{
 let code=value.get("code")?.as_str()?;let message=value.get("message")?.as_str()?;
 let words=korean_words_details(code,message,value.get("expected").and_then(|v|v.as_str()),value.get("actual").and_then(|v|v.as_str()),value.get("offending_field").and_then(|v|v.as_str()),None)?;
 let path=value.get("source_path").and_then(|v|v.as_str()).unwrap_or("출처 미공급");
 let position=match value.get("source_span").and_then(|s|Some((s.get("start")?.as_u64()?,s.get("end")?.as_u64()?))){Some((a,b))=>format!("UTF-8 바이트 {a}..{b}"),None=>"위치 정보 없음".into()};
 Some(format!("{words}\n어디 · {path} · {position}\n진단 번호 · {code}"))
}
pub(super) fn korean_diagnostic(d:&ddonirang_tool::preprocess::ExecutionDiagnostic)->Option<String>{korean_value(&d.to_json_value())}

#[derive(Clone, Copy)]
pub(super) enum ErrorPhase { Command, Operation, Output }

/// A redirected stderr can contain an existing JSON error. Keep those bytes
/// intact, without guessing the protocol by parsing the error message text.
pub(super) fn error_text(language: Language, terminal: bool, phase: ErrorPhase, original: &str) -> String {
    if !terminal { return original.into(); }
    if language == Language::Ko {
        return serde_json::from_str::<serde_json::Value>(original).ok().and_then(|value|korean_value(&value)).unwrap_or_else(||original.into());
    }
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
    if !terminal || snapshot.outcome != "REJECTED" { return None; }
    if language == Language::Ko {
        let mut lines=Vec::new();
        for d in snapshot.diagnostics.iter().filter(|d|d.severity=="error") {
            // A pathname is not source authority. Read an excerpt only when its
            // actual bytes still match the producer's captured source identity.
            let source=std::fs::read_to_string(&d.path).ok().filter(|s|format!("blake3:{}",blake3::hash(s.as_bytes()).to_hex())==snapshot.source_revision_identity);
            let fragment=source.as_deref().and_then(|s|s.get(d.span.start..d.span.end)).filter(|s|!s.is_empty());
            if let Some(mut words)=korean_words_details(&d.code,&d.message,None,None,None,fragment) {
                if fragment.is_some() {
                    if let Some(prefix)=source.as_deref().and_then(|s|s.get(..d.span.start))
                        .map(|s|s.rsplit('\n').next().unwrap_or("")).filter(|s|!s.is_empty()) {
                        words=words.replacen("이 자리\n",&format!("앞 원문\n{prefix}\n이 자리\n"),1);
                    }
                }
                lines.push(format!("{words}\n어디 · {} · UTF-8 바이트 {}..{}\n진단 번호 · {}",d.path,d.span.start,d.span.end,d.code));
            }
        }
        return (!lines.is_empty()).then(||lines.join("\n"));
    }
    let mut lines = Vec::new();
    for diagnostic in snapshot.diagnostics.iter().filter(|item| item.severity == "error") {
        let reason = match diagnostic.code.as_str() {
            "E_PARSE" => "構文を解析できません。括弧・演算子・文末の書き方を、元の診断の位置で確認してください。",
            "E_SOURCE_PROFILE_REQUIRED" => "ソースのプロファイルを明示してください。表示言語の設定とは別です。",
            "E_SOURCE_PROFILE_UNSUPPORTED" => "指定されたソースのプロファイルには対応していません。表示言語を変更しても有効にはなりません。",
            "E_CALL_REQUIRED_PIN_MISSING" => "必須の引数が不足しています。呼び出した関数の引数名と値を確認してください。",
            "E_CALL_TAIL_MISSING_STMT" => "呼び出し文の末尾を、現在の構文例と照合してください。",
            "E_BOIM_LEGACY_REMOVED" => "この旧式の書き方には対応していません。現在の例題の書式を確認してください。",
            code => registered_source_guidance(code)
                .unwrap_or("[ko] この診断の理由は未翻訳です。下にある元の診断を確認してください。"),
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
        if language == Language::Ko { guidance } else { format!("{guidance}\n{original}") }
    } else { error_text(language, terminal, phase, original) }
}

fn registered_source_guidance(code: &str) -> Option<&'static str> {
    Some(match code {
        "E_MULTILANG_CALL_UNREGISTERED" => "翻訳宣言のモジュール別名と、その版に登録された関数名を確認してください。",
        "E_MULTILANG_PIN_UNREGISTERED" => "呼び出した関数の登録内容で引数名を確認してください。別の関数で使える名前が、この関数でも使えるとは限りません。",
        "E_LOGIC_CALL_PIN_OWNER" => "呼び出した論理関数で使える引数名を確認してください。",
        "E_J1_UNREGISTERED_SURFACE" => "指定したソースのプロファイルに登録された表記を使ってください。",
        "E_J1_ROLE_AMBIGUOUS" => "この位置で表記の役割を一つに決められません。宣言や参照が区別できるように修正してください。",
        _ => return None,
    })
}

/// The producer's same-attempt code/span selects presentation only. Unknown
/// custody errors get no invented source range or function-specific advice.
pub(super) fn error_text_with_diagnostic(language: Language, terminal: bool, original: &str,
    diagnostic: &ddonirang_tool::preprocess::ExecutionDiagnostic) -> String {
    if language != Language::Ja || !terminal { return original.into(); }
    let reason = match diagnostic.code {
        "E_MULTILANG_FULLWIDTH_SYNTAX" => "全角の構文記号・数字は使えません。示された位置の記号を半角に直してください。",
        "E_CALL_REQUIRED_PIN_MISSING" => "必須の引数が不足しています。呼び出した関数の引数名と値を確認してください。",
        "E_LOGIC_CALL_ARITY" => "引数の数が一致しません。登録された引数の不足・余分を確認してください。",
        "E_DDL_ACTIVATION_REQUIRED" => "ソースに明示的な翻訳の結束が必要です。表示言語を変えても有効にはなりません。",
        "E_DDL_PUBLIC_BODY_SCOPE" => "公開関数の本体では、必須の有理数引数・加算・乗算・正確な有理数定数を使えます。",
        code => match registered_source_guidance(code) {
            Some(reason) => reason,
            None => return error_text(language, terminal, ErrorPhase::Operation, original),
        },
    };
    let location = match diagnostic.span {
        Some((start, end)) if !diagnostic.source_path.is_empty() =>
            format!("（{}、開始位置 {start}、UTF-8バイト範囲 {start}..{end}）", diagnostic.source_path),
        _ => String::new(),
    };
    format!("{}: {reason}{location}\n{original}", diagnostic.code)
}

pub(super) fn help_text(language: Language, original: &str) -> String {
    if language == Language::Ko { return original.into(); }
    format!("또니랑 コマンドライン\n\
表示言語はソースの言語・実行結果・保存データを変更しません。\n\
表示言語: コマンドの前に --표시말씨 ko|ja または --display-language ko|ja を指定します。\n\
現在のCLIの優先順位: 明示オプション → DDN_DISPLAY_LANGUAGE → 登録実行ファイル名の既定値 → 韓国語。\n\
保存済みの表示設定はCLIにはまだ接続されていません。\n\
例:\n\
  ddonilang --display-language ja check source.ddn --source-profile v1-core-v25\n\
  ddonilang --display-language ja run source.ddn --source-profile v1-core-v25\n\
日本語J1の単一ソースは、先頭行 #!日本語 と明示プロファイル v1-core-v25-ja-base-r1 を組み合わせます。\n\
  ddonilang --display-language ja check 日本語.ddn --source-profile v1-core-v25-ja-base-r1\n\
  ddonilang --display-language ja run 日本語.ddn --source-profile v1-core-v25-ja-base-r1\n\
J1の名前は現在の構文パックが受け付ける範囲に限られます。複数ファイルの作品は準備済み作品データのプロファイルで実行します。\n\
  ddonilang --display-language ja 작품자료 실행 <準備済み作品.json> --마디수 1\n\
非公開検証: ddonilang authored-project linked_callable_source_run <要求.json>\n\
  検証済みのユーザーDDLとB1を同じ作品で明示的に実行します。保存は行いません。\n\
作業台を開く: ddonilang --display-language ja workbench --port 8787\n\
ポートを明示的に選べます。--port 0 では空いているポートを選びます。\n\
同じブラウザーの「自分の作品」を開き直すときは、保存時と同じアドレスを使います。\n\
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
    fn r20_help_matches_current_selection_and_discloses_saved_setting_gap() {
        let original="기존 한국어 도움말";
        assert_eq!(help_text(Language::Ko,original),original);
        let shown=help_text(Language::Ja,original);
        assert!(shown.contains("DDN_DISPLAY_LANGUAGE → 登録実行ファイル名の既定値 → 韓国語"));
        assert!(shown.contains("保存済みの表示設定はCLIにはまだ接続されていません"));
        assert!(shown.ends_with(original));
    }
    #[test]
    fn r18_exact_filename_default_and_unknown_fallback() {
        assert_eq!(select_for_executable(vec![],None,Some(Path::new("folder/トニラン.exe"))).unwrap().language,Language::Ja);
        for name in ["ddonilang.exe","또니랑.exe","내도구.exe","トニラン.EXE","トニラン",
            "myトニラン.exe","トニラン.exe.bak","ﾄﾆﾗﾝ.exe","トニラン.ｅｘｅ","トニラン.exe/ddonilang.exe"] {
            let selected=select_for_executable(vec!["run".into()],None,Some(Path::new(name))).unwrap();
            assert_eq!(selected.language,Language::Ko,"{name}");
            assert_eq!(selected.args,vec!["run"]);
        }
        assert_eq!(select_for_executable(vec![],None,None).unwrap().language,Language::Ko);
    }
    #[test]
    fn r18_existing_option_and_environment_priority_over_filename() {
        let japanese=Some(Path::new("トニラン.exe"));
        assert_eq!(select_for_executable(vec![],Some("ko".into()),japanese).unwrap().language,Language::Ko);
        assert_eq!(select_for_executable(vec![],Some("ja".into()),Some(Path::new("unknown.exe"))).unwrap().language,Language::Ja);
        assert_eq!(select_for_executable(vec!["--표시말씨".into(),"ko".into()],Some("ja".into()),japanese).unwrap().language,Language::Ko);
        assert_eq!(select_for_executable(vec!["--display-language=ja".into()],Some("invalid".into()),japanese).unwrap().language,Language::Ja);
        assert!(select_for_executable(vec![],Some("invalid".into()),japanese).is_err());
    }
    #[test]
    fn r18_duplicate_option_refusal_and_command_tail_preserved() {
        let japanese=Some(Path::new("トニラン.exe"));
        let duplicate=vec!["--표시말씨".into(),"ko".into(),"--display-language=ja".into()];
        assert_eq!(select_for_executable(duplicate.clone(),None,japanese).unwrap_err(),select(duplicate,None).unwrap_err());
        let tail=vec!["run".into(),"--display-language=ko".into(),"トニラン.exe".into()];
        let selected=select_for_executable(tail.clone(),None,japanese).unwrap();
        assert_eq!(selected.language,Language::Ja);assert_eq!(selected.args,tail);
    }
    #[cfg(windows)]
    #[test]
    fn r18_non_unicode_filename_preserves_normal_admission() {
        use std::os::windows::ffi::OsStringExt;
        let invalid=OsString::from_wide(&[0xd800]);
        let selected=select_for_executable(vec!["run".into()],None,Some(Path::new(&invalid))).unwrap();
        assert_eq!(selected.language,Language::Ko);assert_eq!(selected.args,vec!["run"]);
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

    #[test]
    fn r31_cli_same_attempt_diagnostic_preserves_original_and_redirects() {
        let diagnostic = ddonirang_tool::preprocess::ExecutionDiagnostic {
            code: "E_MULTILANG_FULLWIDTH_SYNTAX", message: "original failure".into(),
            source_path: "main.ddn".into(), span: Some((36,39)), phase: "lang_parse",
            cause: None, offending_field: None, expected: None, actual: None,
        };
        let original = "original failure";
        let shown = error_text_with_diagnostic(Language::Ja, true, original, &diagnostic);
        assert!(shown.contains("36..39")); assert!(shown.contains("main.ddn"));
        assert!(shown.ends_with("\noriginal failure"));
        for (language, terminal) in [(Language::Ko,true),(Language::Ko,false),(Language::Ja,false)] {
            assert_eq!(error_text_with_diagnostic(language,terminal,original,&diagnostic),original);
        }
        let mut unlocated = diagnostic.clone(); unlocated.span=None; unlocated.source_path.clear();
        assert!(!error_text_with_diagnostic(Language::Ja,true,original,&unlocated).contains("36..39"));
        unlocated.code="E_SOURCE_SAVE_BEFORE_COMMIT";
        assert_eq!(error_text_with_diagnostic(Language::Ja,true,original,&unlocated),
            error_text(Language::Ja,true,ErrorPhase::Operation,original));
        unlocated.code="E_CALL_REQUIRED_PIN_MISSING";
        let generic = error_text_with_diagnostic(Language::Ja,true,original,&unlocated);
        assert!(!generic.contains("金額")); assert!(!generic.contains("税額"));
        unlocated.code="E_LOGIC_CALL_ARITY";
        let arity=error_text_with_diagnostic(Language::Ja,true,original,&unlocated);
        assert!(arity.contains("引数の数")); assert!(!arity.contains("不足しています"));
    }

}


#[cfg(test)]
mod authoring_diagnostic_tests {
    use super::*;
    use ddonirang_tool::{authored_project as authored,serde_json::{self,json,Value}};
    fn create_request()->Value{json!({
        "source_profile":"v1-core-v25-ja-base-r1","module_name":"",
        "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
            {"path":"root.ddn","text":"#!日本語\r\n(版=\"v25\", モジュール群=列[(名前=アプリ, 見出し=\"main.ddn\", 原文=\"main.ddn\")], 開始=(モジュール=アプリ, 定義=実行)) 構成.\r\n"},
            {"path":"main.ddn","text":"#!日本語\r\n実行:手続き := {\r\n  (真) 表示.\r\n}.\r\n"}
        ]}})}
    fn bad_snapshot(request:&Value)->Value{
        let mut snapshot=request["source_snapshot"].clone();
        let text=snapshot["files"][1]["text"].as_str().unwrap().replace("表示.","表示。");
        snapshot["files"][1]["text"]=text.into();snapshot
    }
    fn check_failure(operation:&str,request:&Value,source:&str){
        let raw=serde_json::to_string(request).unwrap();
        let original=authored::operate(operation,&raw).unwrap_err();
        let (message,diagnostic)=authored::operate_for_cli(operation,&raw).unwrap_err();
        assert_eq!(message,original);let diagnostic=diagnostic.unwrap();
        let start=source.find('。').unwrap();
        assert_eq!(diagnostic.code,"E_MULTILANG_FULLWIDTH_SYNTAX");
        assert_eq!(diagnostic.source_path,"main.ddn");assert_eq!(diagnostic.span,Some((start,start+3)));
        let shown=error_text_with_diagnostic(Language::Ja,true,&message,&diagnostic);
        assert!(shown.contains("全角"));assert!(shown.contains(&format!("{start}..{}",start+3)));
        assert!(shown.ends_with(&message));
        assert_eq!(error_text_with_diagnostic(Language::Ja,false,&message,&diagnostic),message);
    }
    #[test]
    fn r31_cli_authoring_create_uses_actual_utf8_crlf_failure(){
        let mut request=create_request();request["source_snapshot"]=bad_snapshot(&request);
        let source=request["source_snapshot"]["files"][1]["text"].as_str().unwrap();
        check_failure("create",&request,source);
    }
    #[test]
    fn r31_cli_authoring_update_keeps_owner_and_edited_source_range(){
        let create=create_request();let original=authored::operate_for_cli("create",&create.to_string()).unwrap();
        let mut bad=bad_snapshot(&create);
        let text=bad["files"][1]["text"].as_str().unwrap().replace("  (真)","\r\n    (偽)");
        bad["files"][1]["text"]=text.clone().into();
        let request=json!({"original_json":original,"source_snapshot":bad});check_failure("update",&request,&text);
        let mut corrected=request;corrected["source_snapshot"]["files"][1]["text"]=text.replace('。',".").into();
        let after=authored::operate_for_cli("update",&corrected.to_string()).unwrap();
        assert_eq!(after,authored::operate("update",&corrected.to_string()).unwrap());
        let before:Value=serde_json::from_str(corrected["original_json"].as_str().unwrap()).unwrap();
        let after:Value=serde_json::from_str(&after).unwrap();
        assert_eq!(before["authored_identity"],after["authored_identity"]);
        assert_eq!(after["source_snapshot"],corrected["source_snapshot"]);
        assert_eq!(after["schema"],before["schema"]);
    }
    #[test]
    fn r31_cli_authoring_unverified_original_never_borrows_new_source_span(){
        let create=create_request();let original=authored::operate("create",&create.to_string()).unwrap();
        let mut broken:Value=serde_json::from_str(&original).unwrap();broken["content_sha256"]="00".repeat(32).into();
        let request=json!({"original_json":broken.to_string(),"source_snapshot":bad_snapshot(&create)});
        let (message,diagnostic)=authored::operate_for_cli("update",&request.to_string()).unwrap_err();
        assert_eq!(message,authored::operate("update",&request.to_string()).unwrap_err());assert!(diagnostic.is_none());
        let mut extra=create;extra["trusted"]=true.into();
        let (message,diagnostic)=authored::operate_for_cli("create",&extra.to_string()).unwrap_err();
        assert_eq!(message,authored::operate("create",&extra.to_string()).unwrap_err());assert!(diagnostic.is_none());
    }
}
