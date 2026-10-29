    #[test]
    fn source_first_cli_uses_existing_native_owners_and_strict_arguments(){
        use super::*;
        let parse=|args:Vec<&str>|parse_args(args.into_iter().map(str::to_owned).collect());
        assert_eq!(parse(vec!["작품자료","원문저장","project.json","--판","01","--저장소","store"]).unwrap(),
            Command::NativeSourceSave{input:PathBuf::from("project.json"),revision:vec![1],root:PathBuf::from("store")});
        assert_eq!(parse(vec!["작품자료","첫실행","source.json","--판","02","--저장소","store","--마디수","1"]).unwrap(),
            Command::NativeSourceRun{input:PathBuf::from("source.json"),revision:vec![2],root:PathBuf::from("store"),madi:1,prepare_output:None,record_original_private:false});
        assert_eq!(parse(vec!["작품자료","첫실행","source.json","--판","02","--저장소","store","--마디수","3","--저장파일","first.json","--record-original-private"]).unwrap(),
            Command::NativeSourceRun{input:PathBuf::from("source.json"),revision:vec![2],root:PathBuf::from("store"),madi:3,prepare_output:Some(PathBuf::from("first.json")),record_original_private:true});
        assert!(parse(vec!["작품자료","첫실행","source.json","--판","02","--저장소","store","--마디수","3","--record-original-private","--record-original-private"]).is_err());
        assert_eq!(parse(vec!["작품자료","원문이어실행","saved.json","--부모","01","--판","02","--저장소","store","--허가","locator","--마디수","1"]).unwrap(),
            Command::NativeSourceResume{input:PathBuf::from("saved.json"),parent:vec![1],revision:vec![2],root:PathBuf::from("store"),locator:"locator".into(),madi:1,save_revision:None,prepare_output:None,evidence_output:None});
        assert_eq!(parse(vec!["작품자료","원문이어실행","saved.json","--부모","01","--판","02","--저장소","store","--허가","locator","--마디수","1","--저장판","03","--저장파일","next.json"]).unwrap(),
            Command::NativeSourceResume{input:PathBuf::from("saved.json"),parent:vec![1],revision:vec![2],root:PathBuf::from("store"),locator:"locator".into(),madi:1,save_revision:Some(vec![3]),prepare_output:Some(PathBuf::from("next.json")),evidence_output:None});
        assert_eq!(parse(vec!["작품자료","원문이어실행","saved.json","--부모","01","--판","02","--저장소","store","--허가","locator","--마디수","0","--recording-evidence-private","original.bundle"]).unwrap(),
            Command::NativeSourceResume{input:PathBuf::from("saved.json"),parent:vec![1],revision:vec![2],root:PathBuf::from("store"),locator:"locator".into(),madi:0,save_revision:None,prepare_output:None,evidence_output:Some(PathBuf::from("original.bundle"))});
        assert!(parse(vec!["작품자료","원문이어실행","saved.json","--부모","01","--판","02","--저장소","store","--허가","locator","--마디수","1","--recording-evidence-private","original.bundle"]).is_err());
        assert!(parse(vec!["작품자료","원문이어실행","saved.json","--부모","01","--판","02","--저장소","store","--허가","locator","--마디수","0"]).is_err());
        for args in [vec!["작품자료","첫실행","source.json","--판","02","--저장소","store","--마디수","0"],
            vec!["작품자료","원문저장","project.json","--판","01","--저장소","store","--허가","fake"],
            vec!["작품자료","첫실행","source.json","--판","02","--저장소","store","--허가","fake"]] {
            assert!(parse(args).is_err());
        }
        assert!(serde_json::from_str::<NativeSourceLocatorReport>(r#"{"project_identity_hex":"a","source_revision_hex":"01","source_integrity_hex":"b","grant_locator":"c","permit":true}"#).is_err());
    }
    #[test]
    fn workbench_enter_exit_does_not_treat_eof_as_an_exit_request() {
        for (bytes, expected) in [
            (&b""[..], false), (&b"not-a-command\n"[..], false),
            (&b"not-a-command"[..], false), (&b" "[..], false),
            (&b"\t"[..], false), (&b"\r"[..], false),
            (&b"not-a-command\n \t"[..], false),
            (&b"\n"[..], true), (&b"\r\n"[..], true),
            (&b" \n"[..], true), (&b"\t\n"[..], true),
            (&b" \t\r\n"[..], true), (&b"not-a-command\n\n"[..], true),
        ] {
            assert_eq!(super::read_workbench_exit(&mut std::io::Cursor::new(bytes)).unwrap(), expected);
        }
        assert!(super::read_workbench_exit(&mut std::io::Cursor::new([0xff])).is_err());
    }

    #[test]
    fn native_custody_requires_explicit_initialization_request_token_and_bounded_count() {
        use super::*;
        let save:Vec<String>=["작품자료","내구기록","authored.json","--마디수","3","--판","09","--저장소","new-store","--저장소-초기화","--복구자료","prepared"]
            .into_iter().map(str::to_string).collect();
        assert_eq!(parse_args(save.clone()).unwrap(),Command::NativeCustodyRecord{input:PathBuf::from("authored.json"),input_mode:NativeCustodyInput::General(3),revision:vec![9],root:PathBuf::from("new-store"),recovery:PathBuf::from("prepared")});
        let mut logical=save.clone();logical[3]="--입력".into();logical[4]="logical.json".into();
        assert!(matches!(parse_args(logical).unwrap(),Command::NativeCustodyRecord{input_mode:NativeCustodyInput::Logical(_),..}));
        let mut missing=save.clone();missing.pop();assert!(parse_args(missing).is_err());
        let mut zero=save;zero[4]="0".into();assert!(parse_args(zero).is_err());
        let mut token=[0u8;33];token[0]=1;token[1..].fill(9);
        let recover:Vec<String>=["작품자료","복구","saved.json","--판","09","--저장소","store","--허가","locator",
            "--요청","request.bin","--토큰",&hex::encode(token),"--마디수","4"].into_iter().map(str::to_string).collect();
        assert_eq!(parse_args(recover.clone()).unwrap(),Command::NativeCustodyResume{input:PathBuf::from("saved.json"),input_mode:NativeCustodyInput::General(4),revision:vec![9],root:PathBuf::from("store"),
            locator:"locator".into(),request:PathBuf::from("request.bin"),token});
        for invalid in ["00".repeat(33),"01".into(),hex::encode(token).to_uppercase()] {
            // The numeric-only positive fixture has no upper-case variant.
            if invalid==hex::encode(token) {continue;}
            let mut changed=recover.clone();changed[12]=invalid;assert!(parse_args(changed).is_err());
        }
        let mut extra=recover;extra.push("--저장소-초기화".into());assert!(parse_args(extra).is_err());
    }

    #[test]
    fn general_native_uses_existing_project_commands_and_explicit_storage_lineage() {
        use super::*;
        let args:Vec<String>=["작품자료","실행","authored.json","--마디수","2","--저장소","store","--부모","처음","--판","09"]
            .into_iter().map(str::to_string).collect();
        assert_eq!(parse_args(args.clone()).unwrap(),Command::AuthoredRun {input:PathBuf::from("authored.json"),madi:Some(2),
            native:Some(NativeGeneralSave {root:PathBuf::from("store"),parent:vec![],revision:vec![9],initialize:false,expected_receipt:None})});
        for suffix in [vec!["--저장소-초기화","--저장소-초기화"],vec!["--이전-영수증"],vec!["--입력","logical.json"]] {
            let mut changed=args.clone();changed.extend(suffix.into_iter().map(str::to_string));
            assert!(parse_args(changed).is_err());
        }
        let resume:Vec<String>=["작품자료","이어실행","saved.json","--부모","처음","--판","09","--마디수","2",
            "--저장소","store","--거래",&"00".repeat(32),"--일반"].into_iter().map(str::to_string).collect();
        assert_eq!(parse_args(resume.clone()).unwrap(),Command::AuthoredResume {input:PathBuf::from("saved.json"),parent:vec![],revision:vec![9],madi:2,
            native:Some(NativeResume {root:PathBuf::from("store"),digest:[0;32],input:None,save:None})});
        let mut resave=resume.clone();resave.extend(["--저장-판","0a","--이전-영수증","receipt.hex"].into_iter().map(str::to_string));
        assert_eq!(parse_args(resave.clone()).unwrap(),Command::AuthoredResume {input:PathBuf::from("saved.json"),parent:vec![],revision:vec![9],madi:2,
            native:Some(NativeResume {root:PathBuf::from("store"),digest:[0;32],input:None,
                save:Some(NativeResumeSave {revision:vec![10],receipt:PathBuf::from("receipt.hex")})})});
        resave[14]="--입력".into();assert!(parse_args(resave).is_err());
        assert_eq!(parse_args(["작품자료","실행","authored.json","--마디수","2"].into_iter().map(str::to_string).collect()).unwrap(),
            Command::AuthoredRun {input:PathBuf::from("authored.json"),madi:Some(2),native:None});
    }
    #[test]
    fn cli_pretty_output_preserves_bytes_and_owns_deep_response_until_drop() {
        use ddonirang_tool::serde_json::{json, Value};
        use ddonirang_tool::runtime_surface::RuntimeJsonOutput;
        for value in [json!(null), json!([]), json!({"a":[true,"한글\n😀"],"b":{}}), json!({"type":"bytes_v1"})] {
            let expected = ddonirang_tool::serde_json::to_string_pretty(&value).unwrap() + "\n";
            assert_eq!(super::pretty_json(&RuntimeJsonOutput::new(value)).unwrap(), expected);
        }
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut root = Value::Bool(true);
            for _ in 0..512 { root = Value::Array(vec![root]); }
            let response = RuntimeJsonOutput::new(root);
            let text = super::pretty_json(&response).unwrap();
            assert_eq!(text.matches("[\n").count(), 512);
            assert!(text.ends_with("]\n"));
            drop(response);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn saved_resume_requires_explicit_canonical_lineage_and_allows_restore_only() {
        let args=|parent:&str,revision:&str,madi:&str|vec!["작품자료","이어실행","saved.json","--부모",parent,"--판",revision,"--마디수",madi].into_iter().map(str::to_string).collect();
        assert_eq!(super::parse_args(args("처음","01","0")).unwrap(),super::Command::AuthoredResume{
            input:std::path::PathBuf::from("saved.json"),parent:vec![],revision:vec![1],madi:0,native:None,
        });
        for (parent,revision,madi) in [("", "01","1"),("처음","0","1"),("처음","AB","1"),("처음","01","-1")]{
            assert!(super::parse_args(args(parent,revision,madi)).is_err());
        }
        assert!(super::parse_args(vec!["작품자료".into(),"이어실행".into(),"saved.json".into()]).is_err());
        let mut native_args=args("처음","01","0");
        native_args.extend(["--저장소","native-store","--거래",&"00".repeat(32),"--입력","logical.json"].into_iter().map(str::to_string));
        assert_eq!(super::parse_args(native_args.clone()).unwrap(),super::Command::AuthoredResume{
            input:std::path::PathBuf::from("saved.json"),parent:vec![],revision:vec![1],madi:0,
            native:Some(super::NativeResume{root:std::path::PathBuf::from("native-store"),digest:[0;32],input:Some(std::path::PathBuf::from("logical.json")),save:None}),
        });
        native_args[11]="00".into();
        assert!(super::parse_args(native_args).is_err());
    }
    #[test]
    fn native_record_requires_explicit_lineage_and_never_implicitly_initializes(){
        let args:Vec<String>=["작품자료","논리기록","authored.json","--입력","logical.json","--부모","처음","--판","02","--저장소","store"].into_iter().map(str::to_string).collect();
        assert_eq!(super::parse_args(args.clone()).unwrap(),super::Command::AuthoredLogicalRecord(super::NativeLogicalRecord{
            authored:"authored.json".into(),input:"logical.json".into(),root:"store".into(),parent:vec![],revision:vec![2],
            initialize:false,expected_receipt:None,
        }));
        let mut explicit=args.clone();explicit.extend(["--저장소-초기화".into(),"--이전-영수증".into(),"receipt.hex".into()]);
        assert!(super::parse_args(explicit.clone()).is_ok());explicit.push("--저장소-초기화".into());
        assert!(super::parse_args(explicit).is_err());
        let mut missing=args.clone();missing.push("--이전-영수증".into());assert!(super::parse_args(missing).is_err());
        let mut invalid=args;invalid[8]="AB".into();assert!(super::parse_args(invalid).is_err());
    }
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn candidate_surface_is_small_and_does_not_name_legacy_cli() {
        let text = usage();
        assert!(text.contains("정본 실행파일: ddonilang, 또니랑; 호환 별칭: ddonirang"));
        for command in [
            "실행",
            "검사",
            "짓기",
            "정본",
            "다듬기",
            "작품정리",
            "작품찾기",
            "작품관계",
            "작품이름",
            "작품잠금",
            "작품실행",
            "한줄실행",
        ] {
            assert!(text.contains(command));
        }
        assert!(!text.contains("teul-cli"));
        assert!(!text.contains("run-maze"));
    }

    #[test]
    fn korean_core_spellings_reuse_existing_command_identities() {
        let source_args = |command: &str| {
            vec![
                command.into(),
                "input.ddn".into(),
                "--source-profile".into(),
                "v1-core-v25".into(),
            ]
        };

        assert_eq!(
            parse_args(source_args("실행")),
            parse_args(source_args("run"))
        );
        assert_eq!(
            parse_args(source_args("검사")),
            parse_args(source_args("check"))
        );
        assert_eq!(
            parse_args(source_args("짓기")),
            parse_args(source_args("build"))
        );
        assert_eq!(
            parse_args(vec![
                "개발".into(),
                "정본".into(),
                "input.ddn".into(),
                "--source-profile".into(),
                "v1-core-v25".into(),
            ]),
            parse_args(source_args("canon"))
        );
        assert_eq!(parse_args(vec!["도움".into()]), Ok(Command::Help));
        assert_eq!(parse_args(vec!["--도움".into()]), Ok(Command::Help));
        assert_eq!(parse_args(vec!["판본".into()]), Ok(Command::Version));
        assert_eq!(parse_args(vec!["--판본".into()]), Ok(Command::Version));

        let english_missing_profile = parse_args(vec!["run".into(), "input.ddn".into()]);
        let korean_missing_profile = parse_args(vec!["실행".into(), "input.ddn".into()]);
        assert_eq!(korean_missing_profile, english_missing_profile);
    }

    #[test]
    fn control_input_official_aliases_and_duplicate_options() {
        fn command(flags: &[&str]) -> Command {
            let mut args = vec!["실행".into(), "game.ddn".into(), "--source-profile=v1-core-v25".into()];
            args.extend(flags.iter().map(|value| value.to_string()));
            parse_args(args).expect("control option parsing")
        }
        assert_eq!(command(&["--마디수=3", "--실시간-입력", "터미널", "--초당-마디=60"]),
            command(&["--madi=3", "--sam-live", "console", "--madi-hz=60"]));
        assert_eq!(command(&["--마디수", "3", "--실시간-입력", "웹"]),
            command(&["--madi", "3", "--sam-live", "web"]));
        for unapproved in ["--입력-영수증=record.json", "--input-receipts=record.json"] {
            assert!(parse_args(vec!["실행".into(), "game.ddn".into(),
                "--source-profile=v1-core-v25".into(), unapproved.into()])
                .unwrap_err().contains("E_CLI_OPTION_UNKNOWN"));
        }
        for duplicate in [
            ["--마디수=3", "--madi=4"],
            ["--실시간-입력=터미널", "--sam-live=console"],
            ["--입력-테이프=a", "--sam=b"],
        ] {
            let mut args = vec!["실행".into(), "game.ddn".into(), "--source-profile=v1-core-v25".into()];
            args.extend(duplicate.into_iter().map(String::from));
            assert!(parse_args(args).unwrap_err().contains("OPTION_DUPLICATE"));
        }
    }

    #[test]
    fn control_input_official_ddn_replay_and_corruption_no_publication() {
        use ddonirang_tool::input_tape::{InputTape, InputRecord, InputEvent, InputEventKind, mask_to_bytes, encode_input_tape};
        struct Scratch(PathBuf);
        impl Drop for Scratch { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
        let directory = std::env::temp_dir().join(format!("ddn-control-cli-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&directory).unwrap();
        let scratch = Scratch(directory);
        let tape_path = scratch.0.join("input.ddntape");
        let summary_path = scratch.0.join("summary.detjson");
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tools/teul-cli/tests/fixtures/cli_control_input_r1.ddn");
        let tape = InputTape { version: 2, madi_hz: 60, records: vec![
            InputRecord { madi: 0, held_mask: mask_to_bytes(2), events: vec![InputEvent { sequence: 0, key_bit: 2, kind: InputEventKind::Press }] },
            InputRecord { madi: 1, held_mask: mask_to_bytes(0), events: vec![InputEvent { sequence: 1, key_bit: 2, kind: InputEventKind::Release }] },
        ] };
        let mut bytes = encode_input_tape(&tape).unwrap();
        fs::write(&tape_path, &bytes).unwrap();
        let run = |flag: &str| {
            let args = vec!["실행".into(), source.display().to_string(), "--source-profile=v1-core-v25".into(),
                "--마디수=2".into(), flag.into(), tape_path.display().to_string(), "--json".into(),
                "--summary-json".into(), summary_path.display().to_string()];
            execute(parse_args(args).unwrap())
        };
        let (korean, code) = run("--입력-테이프").unwrap();
        assert_eq!(code, 0);
        let (english, code) = run("--sam").unwrap();
        assert_eq!(code, 0);
        assert_eq!(korean, english);
        let summary: ddonirang_tool::serde_json::Value = ddonirang_tool::serde_json::from_str(&korean).unwrap();
        assert_eq!(summary["output_log_texts"], ddonirang_tool::serde_json::json!(["참", "거짓"]));
        let before = fs::read(&summary_path).unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        fs::write(&tape_path, bytes).unwrap();
        let error = run("--입력-테이프").err().unwrap();
        assert!(error.message.contains("E_SAM_TAPE_DIGEST_MISMATCH"));
        assert_eq!(fs::read(summary_path).unwrap(), before);
    }

    #[test]
    fn official_run_accepts_explicit_json_and_atomic_summary_without_changing_human_default() {
        let parsed = parse_args(vec![
            "실행".into(),
            "input.ddn".into(),
            "--source-profile".into(),
            "v1-core-v25".into(),
            "--madi=3".into(),
            "--json".into(),
            "--summary-json=summary.detjson".into(),
        ])
        .expect("run options");
        assert_eq!(
            parsed,
            Command::Run(SourceCommand {
                path: PathBuf::from("input.ddn"),
                source_profile: "v1-core-v25".into(),
                requested_madi: Some(3),
                json_stdout: true,
                summary_json: Some(PathBuf::from("summary.detjson")),
                host_resources: HostResourceOptions::default(),
                control_input: ControlInputOptions::default(),
                bogae_entry: None,
            })
        );

        let human = parse_args(vec![
            "run".into(),
            "input.ddn".into(),
            "--source-profile=v1-core-v25".into(),
            "--summary-json".into(),
            "summary.detjson".into(),
        ])
        .expect("human stdout mode");
        assert!(matches!(
            human,
            Command::Run(SourceCommand {
                json_stdout: false,
                summary_json: Some(_),
                ..
            })
        ));
    }

    #[test]
    fn official_run_rejects_implicit_or_duplicate_json_modes() {
        for args in [
            vec![
                "run".into(),
                "input.ddn".into(),
                "--source-profile=v1-core-v25".into(),
                "--json=true".into(),
            ],
            vec![
                "run".into(),
                "input.ddn".into(),
                "--source-profile=v1-core-v25".into(),
                "--json".into(),
                "--json".into(),
            ],
            vec![
                "run".into(),
                "input.ddn".into(),
                "--source-profile=v1-core-v25".into(),
                "--summary-json".into(),
            ],
        ] {
            assert!(parse_args(args).is_err());
        }
    }

    #[test]
    fn official_run_human_output_preserves_program_order_and_empty_output() {
        let summary = ddonirang_tool::serde_json::json!({
            "output_log_texts": ["첫 출력", "둘째 출력"],
            "state_hash": "not-human-output",
        });
        assert_eq!(
            human_run_output(&summary).expect("human output"),
            "첫 출력\n둘째 출력\n"
        );
        assert_eq!(
            human_run_output(&ddonirang_tool::serde_json::json!({"output_log_texts": []}))
                .expect("empty output"),
            ""
        );
    }

    #[test]
    fn official_run_rejects_malformed_output_log_before_stdout_projection() {
        let malformed = ddonirang_tool::serde_json::json!({"output_log_texts": ["ok", 3]});
        let error = human_run_output(&malformed).expect_err("malformed output log");
        assert_eq!(error.exit_code, 1);
        assert_eq!(error.message, "E_CLI_OUTPUT_LOG_INVALID");
    }

    #[test]
    fn a3_p1_human_output_uses_typed_projection_without_mutating_log() {
        let summary = ddonirang_tool::serde_json::json!({
            "output_log_texts": ["19.019999999", "19.019999999"],
            "output_display": {"schema":"ddn.output-display.v1", "texts":["19.02", "19.019999999"]}
        });
        assert_eq!(human_run_output(&summary).unwrap(), "19.02\n19.019999999\n");
        assert_eq!(summary["output_log_texts"][0], "19.019999999");
    }

    #[test]
    fn official_run_rejects_summary_output_alias_of_existing_input() {
        let root = std::env::temp_dir().join(format!(
            "ddn-cli-summary-alias-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("test root");
        let input = root.join("input.ddn");
        std::fs::write(&input, "값:셈씨 := 1.").expect("input");
        assert!(existing_input_output_alias(&input, &input));
        assert!(existing_input_output_alias(
            &input,
            &root.join(".").join("input.ddn")
        ));
        std::fs::remove_dir_all(&root).expect("task-owned test root cleanup");
    }

    #[test]
    fn general_run_help_describes_fresh_identity_and_recorded_context(){
        for command in ["run","실행"] {for flag in ["--help","--도움","-h"] {
            assert_eq!(parse_args(vec![command.into(),flag.into()]),Ok(Command::Help));
        }}
        for expected in ["등록 파일 없이 실행마다 임시 정체", "이름씨·원자·임자", "실행 간 state_hash가 다를 수",
            "초기 정체·입력·난수 상태", "입력 기록 하나만으로", "저장한 작품은 보존된 정체", "임시 정체를 저장·내보내는 권한은 생기지"] {
            assert!(usage().contains(expected),"missing identity boundary: {expected}");
        }
    }
    #[test]
    fn every_existing_candidate_command_accepts_existing_help_flags() {
        for command in [
            "run",
            "check",
            "build",
            "canon",
            "lint",
            "project-normalize",
            "project-discover",
            "project-graph",
            "project-symbols",
            "project-lock",
            "project-run",
            "currentline-run",
        ] {
            for flag in ["--help", "--도움", "-h"] {
                assert_eq!(
                    parse_args(vec![command.into(), flag.into()]),
                    Ok(Command::Help),
                    "command={command} flag={flag}",
                );
            }
        }
        for command in ["실행", "검사", "짓기"] {
            assert_eq!(
                parse_args(vec![command.into(), "--help".into()]),
                Ok(Command::Help),
            );
        }
        assert_eq!(
            parse_args(vec!["개발".into(), "정본".into(), "--help".into()]),
            Ok(Command::Help),
        );

        let unknown = parse_args(vec!["unknown".into(), "--help".into()])
            .expect_err("unknown command must remain fail-closed");
        assert!(unknown.starts_with("E_CLI_COMMAND_UNKNOWN"));
    }

    #[test]
    fn source_profile_is_explicit_and_unknown_options_fail_closed() {
        let missing = parse_args(vec!["run".into(), "input.ddn".into()])
            .expect_err("profile must be explicit");
        assert!(missing.starts_with("E_SOURCE_PROFILE_REQUIRED"));

        let unknown = parse_args(vec![
            "run".into(),
            "input.ddn".into(),
            "--source-profile".into(),
            "v1-core-v25".into(),
            "--legacy-fallback".into(),
        ])
        .expect_err("unknown option must fail closed");
        assert!(unknown.starts_with("E_CLI_OPTION_UNKNOWN"));
    }

    #[test]
    fn project_normalize_has_one_explicit_input_and_rejects_options() {
        assert_eq!(
            parse_args(vec!["project-normalize".into(), "project.json".into()]),
            Ok(Command::ProjectNormalize(ProjectCommand {
                path: PathBuf::from("project.json"),
            }))
        );
        let missing = parse_args(vec!["project-normalize".into()])
            .expect_err("project input must be explicit");
        assert!(missing.starts_with("E_CLI_PROJECT_INPUT_REQUIRED"));
        let option = parse_args(vec!["project-normalize".into(), "--legacy".into()])
            .expect_err("unknown options must fail closed");
        assert!(option.starts_with("E_CLI_OPTION_UNKNOWN"));
    }

    #[test]
    fn project_inspection_commands_bind_shared_inputs_and_fail_closed() {
        let lines=usage().lines().filter(|line|line.trim_start().starts_with("ddonilang 작품찾기 ")).collect::<Vec<_>>();
        assert_eq!(lines.len(),2);
        assert!(lines.iter().any(|line|line.contains("작품찾기 --가상입력 <요청.json>")));
        assert!(lines.iter().all(|line|!(line.contains("<작품머리.ddn>")&&line.contains("--가상입력"))));
        assert_eq!(
            parse_args(vec![
                "project-discover".into(),
                "--virtual".into(),
                "request.json".into(),
            ]),
            Ok(Command::ProjectDiscover(ProjectDiscoverCommand {
                start: None,
                project: None,
                virtual_request: Some(PathBuf::from("request.json")),
            }))
        );
        assert_eq!(
            parse_args(vec!["project-graph".into(), "project.json".into()]),
            Ok(Command::ProjectGraph(ProjectCommand {
                path: PathBuf::from("project.json"),
            }))
        );
        assert_eq!(
            parse_args(vec![
                "project-lock".into(),
                "--frozen".into(),
                "project.json".into(),
            ]),
            Ok(Command::ProjectLock(ProjectLockCommand {
                path: PathBuf::from("project.json"),
                frozen: true,
            }))
        );
        let conflicting = parse_args(vec![
            "project-discover".into(),
            "source.ddn".into(),
            "--virtual".into(),
            "request.json".into(),
        ])
        .expect_err("virtual discovery must not silently ignore disk inputs");
        assert!(conflicting.starts_with("E_CLI_OPTION_CONFLICT"));
        let output_option = parse_args(vec![
            "project-symbols".into(),
            "project.json".into(),
            "--out".into(),
        ])
        .expect_err("unowned output materialization must fail closed");
        assert!(output_option.starts_with("E_CLI_OPTION_UNKNOWN --out"));
    }

    #[test]
    fn project_run_binds_neutral_identity_inputs_and_rejects_virtual_conflicts() {
        let lines=usage().lines().filter(|line|line.trim_start().starts_with("ddonilang 작품실행 ")).collect::<Vec<_>>();
        assert_eq!(lines.len(),2);
        assert!(lines.iter().any(|line|line.contains("작품실행 --가상입력 <작품요청.json>")));
        assert!(lines.iter().all(|line|!(line.contains("<작품머리.ddn>")&&line.contains("--가상입력"))));
        assert!(parse_args(vec!["project-run".into()]).unwrap_err().starts_with("E_PROJECT_HEAD_REQUIRED"));
        let explicit = parse_args(vec!["project-run".into(), "임의이름.ddn".into()]).unwrap();
        let Command::ProjectRun(explicit) = explicit else { panic!("project run"); };
        assert_eq!(explicit.start, Some(PathBuf::from("임의이름.ddn")));
        assert!(explicit.target.is_empty(), "no hidden app target");
        assert_eq!(
            parse_args(vec![
                "project-run".into(),
                "--virtual".into(),
                "project.json".into(),
                "--target".into(),
                "앱".into(),
                "--inspect-graph".into(),
                "--inspect-lock".into(),
                "--out".into(),
                "result.detjson".into(),
            ]),
            Ok(Command::ProjectRun(ProjectRunCommand {
                start: None,
                project: None,
                virtual_source: Some(PathBuf::from("project.json")),
                target: "앱".into(),
                inspect_graph: true,
                inspect_lock: true,
                out: Some(PathBuf::from("result.detjson")),
                host_resources: HostResourceOptions::default(),
            }))
        );
        for args in [
            vec![
                "project-run".into(),
                "source.ddn".into(),
                "--virtual".into(),
                "project.json".into(),
            ],
            vec![
                "project-run".into(),
                "--project".into(),
                "project.ddnproj".into(),
                "--virtual".into(),
                "project.json".into(),
            ],
        ] {
            let error = parse_args(args).expect_err("virtual inputs must be exclusive");
            assert!(error.starts_with("E_CLI_OPTION_CONFLICT command=project-run --virtual"));
        }
    }

    #[test]
    fn currentline_requires_cell_and_explicit_supported_profile() {
        assert_eq!(
            parse_args(vec![
                "currentline-run".into(),
                "--cell".into(),
                "cell.ddn".into(),
                "--source-profile".into(),
                "v1-core-v25".into(),
                "--context-json".into(),
                "before.detjson".into(),
                "--context-out".into(),
                "after.detjson".into(),
                "--summary-json".into(),
                "summary.detjson".into(),
            ]),
            Ok(Command::CurrentlineRun(CurrentlineCommand {
                cell: PathBuf::from("cell.ddn"),
                context_json: Some(PathBuf::from("before.detjson")),
                context_out: Some(PathBuf::from("after.detjson")),
                summary_json: Some(PathBuf::from("summary.detjson")),
                source_profile: "v1-core-v25".into(),
            }))
        );
        let missing_profile = parse_args(vec![
            "currentline-run".into(),
            "--cell".into(),
            "cell.ddn".into(),
        ])
        .expect_err("profile must be explicit");
        assert!(missing_profile.starts_with("E_SOURCE_PROFILE_REQUIRED"));
    }

    #[test]
    fn currentline_v1_core_compiles_into_supported_runtime_source() {
        let currentline = ddonirang_lang::apply_currentline_cell_v25(
            "제목 <- \"콘솔 보개 예제\".\nx <- 15.\ny <- 8.\n합 <- x + y.\n합 보여주기.",
            None,
        )
        .expect("currentline compile");
        assert!(!currentline.project_source.contains("움직씨 ="));
        let summary = ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile(
            &currentline.project_source,
            "currentline-v1-core.ddn",
            None,
            Some("v1-core-v25"),
        )
        .expect("supported-profile currentline execution");
        assert_eq!(summary["output_log_texts"][0], "23");
    }

    #[test]
    fn official_run_records_then_replays_declared_resource_without_its_locator() {
        let root = std::env::temp_dir().join(format!(
            "ddn-host-resource-cli-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("test root");
        let source_path = root.join("input.ddn");
        let resource_path = root.join("caption.txt");
        let manifest_path = root.join("resources.json");
        let log_path = root.join("record.detjson");
        std::fs::write(
            &source_path,
            "매마디:움직씨 := {\n    값:수 := 1.\n    값 보여주기.\n}\n",
        )
        .expect("source");
        let resource_text = "선언 입력\n";
        std::fs::write(&resource_path, resource_text).expect("resource");
        let mut hasher = Sha256::new();
        hasher.update(resource_text.as_bytes());
        let manifest = serde_json::json!({
            "schema": ddonirang_tool::host_resource::HOST_RESOURCE_MANIFEST_SCHEMA,
            "resources": [{
                "logical_resource_id": "caption",
                "locator": "caption.txt",
                "sha256": format!("{:x}", hasher.finalize()),
                "byte_length": resource_text.len(),
                "media_type": ddonirang_tool::host_resource::V1_TEXT_MEDIA_TYPE,
            }],
        });
        std::fs::write(&manifest_path, manifest.to_string()).expect("manifest");

        let source_arg = source_path.to_string_lossy().to_string();
        let log_arg = log_path.to_string_lossy().to_string();
        let manifest_arg = manifest_path.to_string_lossy().to_string();
        let record = parse_args(vec![
            "run".into(), source_arg.clone(), "--source-profile".into(),
            "v1-core-v25".into(), "--json".into(), "--open".into(), "record".into(),
            "--open-log".into(), log_arg.clone(), "--open-resources".into(), manifest_arg,
        ])
        .expect("record command");
        let (record_output, _) = execute(record).expect("record execution");
        assert!(record_output.contains("host_resource_receipt"));
        assert!(log_path.is_file());

        std::fs::remove_file(&resource_path).expect("remove only task-owned declared input");
        let replay = parse_args(vec![
            "run".into(), source_arg, "--source-profile".into(), "v1-core-v25".into(),
            "--json".into(), "--open".into(), "replay".into(), "--open-log".into(), log_arg,
        ])
        .expect("replay command");
        let (replay_output, _) = execute(replay).expect("replay without declared input locator");
        assert_eq!(record_output, replay_output);

        std::fs::remove_dir_all(&root).expect("remove only task-owned test root");
    }

    #[test]
    fn record_log_and_bundle_publish_atomically_on_target_failure() {
        let root = std::env::temp_dir().join(format!(
            "ddn-host-resource-cli-atomic-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("test root");
        let source_path = root.join("input.ddn");
        let resource_path = root.join("caption.txt");
        let manifest_path = root.join("resources.json");
        let log_path = root.join("record.detjson");
        let bundle_path = root.join("bundle.detjson");
        std::fs::write(
            &source_path,
            "매마디:움직씨 := {\n    값:수 := 1.\n    값 보여주기.\n}\n",
        )
        .expect("source");
        let resource_text = "선언 입력\n";
        std::fs::write(&resource_path, resource_text).expect("resource");
        let mut hasher = Sha256::new();
        hasher.update(resource_text.as_bytes());
        let manifest = serde_json::json!({
            "schema": ddonirang_tool::host_resource::HOST_RESOURCE_MANIFEST_SCHEMA,
            "resources": [{
                "logical_resource_id": "caption",
                "locator": "caption.txt",
                "sha256": format!("{:x}", hasher.finalize()),
                "byte_length": resource_text.len(),
                "media_type": ddonirang_tool::host_resource::V1_TEXT_MEDIA_TYPE,
            }],
        });
        std::fs::write(&manifest_path, manifest.to_string()).expect("manifest");
        std::fs::create_dir(&bundle_path).expect("invalid bundle directory");

        let record = parse_args(vec![
            "run".into(),
            source_path.to_string_lossy().to_string(),
            "--source-profile".into(),
            "v1-core-v25".into(),
            "--open".into(),
            "record".into(),
            "--open-log".into(),
            log_path.to_string_lossy().to_string(),
            "--open-bundle".into(),
            bundle_path.to_string_lossy().to_string(),
            "--open-resources".into(),
            manifest_path.to_string_lossy().to_string(),
        ])
        .expect("record command");
        let error = execute(record).expect_err("directory target must fail before publish");
        assert!(
            error.message.contains("E_SHARED_ARTIFACT_OUTPUT_TARGET_NOT_FILE"),
            "{}",
            error.message
        );
        assert!(!log_path.exists(), "first artifact must not leak on preflight failure");
        assert!(bundle_path.is_dir());

        std::fs::remove_dir_all(&root).expect("remove only task-owned test root");
    }

    #[test]
    fn lint_requires_profile_and_rejects_output_without_patch_mode() {
        let missing = parse_args(vec!["lint".into(), "lesson.ddn".into()])
            .expect_err("lint profile must be explicit");
        assert!(missing.starts_with("E_SOURCE_PROFILE_REQUIRED"));
        let conflict = parse_args(vec![
            "lint".into(),
            "lesson.ddn".into(),
            "--source-profile".into(),
            "v1-core-v25".into(),
            "--out".into(),
            "patch.json".into(),
        ])
        .expect_err("out requires patch mode");
        assert!(conflict.starts_with("E_CLI_OPTION_CONFLICT"));
    }
