//! Product CLI spellings only. Never translate source text, paths, JSON keys,
//! stored bytes or private evidence switches. English input remains supported.
pub(super) const COMMANDS: &[(&str, &str)] = &[
    ("정본", "canon"), ("다듬기", "lint"),
    ("작품정리", "project-normalize"), ("작품찾기", "project-discover"),
    ("작품관계", "project-graph"), ("작품이름", "project-symbols"),
    ("작품잠금", "project-lock"), ("작품실행", "project-run"),
    ("한줄실행", "currentline-run"), ("작업대", "workbench"),
];

// Existing Korean options are deliberately not rewritten. In particular,
// Native source operations already require their exact --마디수 spelling.
pub(super) const OPTIONS: &[(&str, &str)] = &[
    ("--글판", "--source-profile"), ("--구조출력", "--json"),
    ("--요약파일", "--summary-json"), ("--외부접근", "--open"),
    ("--외부기록", "--open-log"), ("--외부묶음", "--open-bundle"),
    ("--외부자원", "--open-resources"), ("--수정제안", "--suggest-patch"),
    ("--출력", "--out"), ("--작품", "--project"),
    ("--가상입력", "--virtual"), ("--잠금유지", "--frozen"),
    ("--관계검사", "--inspect-graph"), ("--잠금검사", "--inspect-lock"),
    ("--대상", "--target"), ("--한줄", "--cell"),
    ("--문맥입력", "--context-json"), ("--문맥출력", "--context-out"),
    ("--브라우저없이", "--no-open"), ("--포트", "--port"),
    ("--작품자료", "--authored"), ("--형식", "--format"),
];

pub(super) fn normalize_args(mut args: Vec<String>) -> Vec<String> {
    // This existing frontdoor has its own closed Korean options and positional
    // request paths. Never reinterpret those data as public option aliases.
    if matches!(args.first().map(String::as_str), Some("작품자료" | "authored-project")) {
        return args;
    }
    for (index, argument) in args.iter_mut().enumerate() {
        if index == 0 {
            if let Some((_, existing)) = COMMANDS.iter().find(|(korean, _)| argument.as_str() == *korean) {
                *argument = (*existing).to_owned();
                continue;
            }
        }
        // Only flag names are rewritten; the complete value after '=' is kept.
        // Positional paths such as "작업대" or "기록" are never command-mapped.
        let (name, value) = argument.split_once('=').map_or((argument.as_str(), None),
            |(name, value)| (name, Some(value)));
        if let Some((_, existing)) = OPTIONS.iter().find(|(korean, _)| name == *korean) {
            *argument = match value { Some(value) => format!("{existing}={value}"), None => (*existing).to_owned() };
        }
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_args, Command};
    fn parse(words: &[&str]) -> Result<Command, String> {
        parse_args(words.iter().map(|word| (*word).to_owned()).collect())
    }

    #[test]
    fn korean_cli_public_commands_keep_existing_parser_semantics() {
        for (korean, english) in COMMANDS {
            // Compare both success and rejection, not an untyped token list.
            let tail: &[&str] = if *english == "workbench" { &[] } else { &["입력.json"] };
            let mut ko = vec![*korean]; ko.extend(tail);
            let mut en = vec![*english]; en.extend(tail);
            assert_eq!(parse(&ko), parse(&en), "{korean}");
            assert_eq!(parse(&[*korean, "--도움"]), parse(&[*english, "--help"]));
        }
    }

    #[test]
    fn korean_cli_source_options_preserve_paths_and_detect_mixed_duplicates() {
        assert_eq!(parse(&["실행","작업대","--글판","v1-core-v25","--마디수","2","--구조출력","--요약파일=기록=둘.json"]),
            parse(&["run","작업대","--source-profile","v1-core-v25","--madi","2","--json","--summary-json=기록=둘.json"]));
        for words in [
            vec!["실행","원문.ddn","--글판","v1-core-v25","--source-profile","v1-core-v25"],
            vec!["실행","원문.ddn","--글판"],
            vec!["실행","원문.ddn","--글판","v1-core-v25","--요약파일","a","--summary-json","b"],
            vec!["작업대","--포트","8787","--port","8788"],
        ] { assert!(parse(&words).is_err(), "{words:?}"); }
        assert_eq!(normalize_args(vec!["실행".into(),"작업대".into(),"--글판=v1-core-v25".into()]),
            vec!["실행","작업대","--source-profile=v1-core-v25"]);
    }

    #[test]
    fn korean_cli_transport_options_and_private_boundary_are_preserved() {
        assert!(crate::usage().contains("ddonilang 조작입력기록 <파일.ddn>"));
        assert!(crate::usage().contains("ddonilang 조작입력재생 <파일.ddn>"));
        assert_eq!(parse(&["논리입력기록","작업대","기록","출력","--작품자료","--형식=v3"]),
            parse(&["logical-record","작업대","기록","출력","--authored","--format=v3"]));
        assert!(parse(&["논리입력재생","a","b","c","--형식=v3","--format=v3"]).is_err());
        assert!(OPTIONS.iter().all(|(ko,en)| !ko.contains("private") && !en.contains("private")));
        for private in ["--finish-live-private","--record-original-private","--recording-evidence-private"] {
            let words=vec!["작품자료".into(), private.into()];
            assert_eq!(normalize_args(words.clone()), words);
            assert!(!crate::usage().contains(private));
        }
    }

    #[test]
    fn korean_cli_native_request_paths_are_not_option_aliases() {
        for command in ["작품자료", "authored-project"] {
            for (spelling, _) in OPTIONS {
                let original = vec![command.to_owned(), "원문전이".to_owned(), (*spelling).to_owned()];
                assert_eq!(normalize_args(original.clone()), original);
                assert_eq!(parse(&[command, "원문전이", spelling]).unwrap(),
                    Command::NativeSourceOperation { input: std::path::PathBuf::from(spelling) });
            }
        }
    }

    #[test]
    fn korean_cli_project_currentline_lint_and_external_access_match_existing_commands() {
        for (ko,en) in [
            (vec!["작업대","--브라우저없이","--포트","8788"],vec!["workbench","--no-open","--port","8788"]),
            (vec!["작품찾기","--가상입력","요청.json"],vec!["project-discover","--virtual","요청.json"]),
            (vec!["작품잠금","요청.json","--잠금유지"],vec!["project-lock","요청.json","--frozen"]),
            (vec!["다듬기","원문.ddn","--글판","v1-core-v25","--수정제안","--출력","수정.json"],
             vec!["lint","원문.ddn","--source-profile","v1-core-v25","--suggest-patch","--out","수정.json"]),
            (vec!["한줄실행","--한줄","원문.ddn","--글판","v1-core-v25","--문맥입력","a.json","--문맥출력","b.json"],
             vec!["currentline-run","--cell","원문.ddn","--source-profile","v1-core-v25","--context-json","a.json","--context-out","b.json"]),
            (vec!["실행","원문.ddn","--글판","v1-core-v25","--외부접근","기록","--외부자원","자원.json","--외부기록","기록.json"],
             vec!["run","원문.ddn","--source-profile","v1-core-v25","--open","record","--open-resources","자원.json","--open-log","기록.json"]),
        ] {
            let korean=parse(&ko).expect("valid Korean public CLI");
            let english=parse(&en).expect("existing equivalent CLI");
            assert_eq!(korean,english,"{ko:?}");
        }
        for (korean,english) in OPTIONS {
            assert_eq!(normalize_args(vec!["실행".into(),format!("{korean}=값=둘")]),
                vec!["실행".into(),format!("{english}=값=둘")]);
        }
    }
}
