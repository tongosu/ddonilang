use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use ddonirang_tool::artifact_output::existing_input_output_alias;

const SOURCE_PROFILE_FLAG: &str = "--source-profile";

fn usage() -> &'static str {
    "또니랑 V1 명령줄 (정본 실행파일: ddonilang, 또니랑; 호환 별칭: ddonirang)\n\
사용법:\n\
  ddonilang 실행 <파일.ddn> --source-profile <profile> [--마디수 <N>] [--json] [--summary-json <summary.detjson>] [--open deny|record|replay --open-log <transcript.json> --open-resources <manifest.json>]  (technical: run)\n\
  ddonilang 검사 <파일.ddn> --source-profile <profile>  (technical: check)\n\
  ddonilang 짓기 <파일.ddn> --source-profile <profile>  (technical: build)\n\
  ddonilang 개발 정본 <파일.ddn> --source-profile <profile>  (technical: canon)\n\
  ddonilang lint <파일.ddn> --source-profile <profile> [--suggest-patch] [--out <ddn.patch.json>]\n\
  ddonilang project-normalize <virtual-project.json>\n\
  ddonilang project-discover <작품머리.ddn> [--project <작품머리.ddn>] [--virtual <request.json>]\n\
  ddonilang project-graph <virtual-project.json>\n\
  ddonilang project-symbols <virtual-project.json>\n\
  ddonilang project-lock <virtual-project.json> [--frozen]\n\
  ddonilang project-run <작품머리.ddn> [--project <작품머리.ddn>] [--virtual <request.json>] [--inspect-graph] [--inspect-lock] [--open deny|record|replay --open-log <transcript.json> --open-resources <manifest.json>] [--out <artifact.detjson>]\n\
  ddonilang currentline-run --cell <파일.ddn> --source-profile <profile> [--context-json <context.detjson>] [--context-out <context.detjson>] [--summary-json <summary.detjson>]\n\
  ddonilang workbench [--no-open]\n\
  ddonilang 논리입력기록 <파일.ddn> <입력요청.json> <기록파일>  (technical: logical-record)\n\
  ddonilang 논리입력재생 <파일.ddn> <기록파일> <입력계약.json>  (technical: logical-replay)\n\
  ddonilang 작품자료 <만들기|열기|복제|갱신|보개등록|이름변경|경로변경> <파일.json>  (technical: authored-project)\n\
  ddonilang 작품자료 실행 <파일.json> [--마디수 N]: 저장된 보개 정체로 실행\n\
  ddonilang 도움  (technical: help)\n\
  ddonilang 판본  (technical: version)\n\n\
제어입력 옵션 (괄호 안은 영어 호환 별칭):\n\
  --마디수 N (--madi): 실행할 마디 수\n\
  --보개-시작 <정의이름>: 보기값·보개짜임값의 시작을 명시 (임시 실행 정체, 영구 ID 생성 없음)\n\
  --실시간-입력 터미널|웹 (--sam-live console|web)\n\
  --실시간-입력-호스트 (--sam-live-host), --실시간-입력-포트 (--sam-live-port)\n\
  --초당-마디 N (--madi-hz): 입력 마디 속도\n\
  --입력-테이프 파일 (--sam): 명시적 테이프 재생\n\
  --입력-테이프-기록 파일 (--record-sam): 실시간 입력 기록\n\
문법 판 v25의 알파 실행 프로필: --source-profile v1-core-v25\n"
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Version,
    Run(SourceCommand),
    Check(SourceCommand),
    Build(SourceCommand),
    Canon(SourceCommand),
    Lint(LintCommand),
    ProjectNormalize(ProjectCommand),
    ProjectDiscover(ProjectDiscoverCommand),
    ProjectGraph(ProjectCommand),
    ProjectSymbols(ProjectCommand),
    ProjectLock(ProjectLockCommand),
    ProjectRun(ProjectRunCommand),
    CurrentlineRun(CurrentlineCommand),
    Workbench(WorkbenchCommand),
    LogicalRecord { source: PathBuf, request: PathBuf, output: PathBuf },
    LogicalReplay { source: PathBuf, tape: PathBuf, binding: PathBuf },
    ManipulationRecord { source: PathBuf, request: PathBuf, output: PathBuf },
    ManipulationReplay { source: PathBuf, tape: PathBuf, binding: PathBuf },
    AuthoredProject { operation: String, input: PathBuf },
    AuthoredRun { input: PathBuf, madi: Option<u32> },
}

#[derive(Debug, PartialEq, Eq)]
struct SourceCommand {
    path: PathBuf,
    source_profile: String,
    requested_madi: Option<u32>,
    json_stdout: bool,
    summary_json: Option<PathBuf>,
    host_resources: HostResourceOptions,
    control_input: ControlInputOptions,
    bogae_entry: Option<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct ControlInputOptions {
    mode: Option<String>,
    host: Option<String>,
    port: Option<String>,
    hz: Option<String>,
    tape: Option<String>,
    record: Option<String>,
}

impl ControlInputOptions {
    fn active(&self) -> bool { self.mode.is_some() || self.tape.is_some() }

    fn parse_option(&mut self, args: &[String], index: &mut usize) -> Result<bool, String> {
        let (flag, inline) = args[*index].split_once('=').map_or((args[*index].as_str(), None), |(flag, value)| (flag, Some(value)));
        let (slot, korean) = match flag {
            "--실시간-입력" | "--sam-live" => (&mut self.mode, "--실시간-입력"),
            "--실시간-입력-호스트" | "--sam-live-host" => (&mut self.host, "--실시간-입력-호스트"),
            "--실시간-입력-포트" | "--sam-live-port" => (&mut self.port, "--실시간-입력-포트"),
            "--초당-마디" | "--madi-hz" => (&mut self.hz, "--초당-마디"),
            "--입력-테이프" | "--sam" => (&mut self.tape, "--입력-테이프"),
            "--입력-테이프-기록" | "--record-sam" => (&mut self.record, "--입력-테이프-기록"),
            _ => return Ok(false),
        };
        if slot.is_some() { return Err(format!("E_CLI_OPTION_DUPLICATE {korean}: 같은 옵션을 두 번 지정할 수 없습니다")); }
        let value = match inline {
            Some("") => return Err(format!("E_CLI_OPTION_VALUE_REQUIRED {korean}: 값이 필요합니다")),
            Some(value) => value.to_string(),
            None => next_cli_value(args, index, korean)?,
        };
        *slot = Some(value);
        if korean == "--실시간-입력" {
            self.mode = Some(match self.mode.as_deref() {
                Some("터미널" | "console") => "console",
                Some("웹" | "web") => "web",
                _ => return Err("E_CLI_OPTION_VALUE_INVALID --실시간-입력: 터미널(console) 또는 웹(web)을 지정하세요".into()),
            }.into());
        }
        Ok(true)
    }

    fn validate(&self, requested_madi: Option<u32>) -> Result<(), String> {
        if self.mode.is_some() && self.tape.is_some() {
            return Err("E_CLI_OPTION_CONFLICT 실시간 입력과 입력 테이프를 동시에 지정할 수 없습니다".into());
        }
        if self.mode.is_some() && requested_madi.is_none() {
            return Err("E_SAM_LIVE_MADI_REQUIRED --마디수(--madi)로 유한 실행 길이를 지정하세요".into());
        }
        if self.mode.is_none() && (self.host.is_some() || self.port.is_some() || self.hz.is_some() || self.record.is_some()) {
            return Err("E_CLI_OPTION_CONFLICT 입력 호스트·속도·기록 옵션에는 --실시간-입력(--sam-live)이 필요합니다".into());
        }
        if let Some(port) = &self.port {
            port.parse::<u16>().map_err(|_| "E_CLI_OPTION_VALUE_INVALID --실시간-입력-포트: 포트 범위를 확인하세요".to_string())?;
        }
        if let Some(hz) = &self.hz {
            if hz.parse::<u32>().ok().filter(|hz| *hz > 0).is_none() {
                return Err("E_SAM_BAD_MADI_HZ --초당-마디(--madi-hz)는 양의 정수여야 합니다".into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct HostResourceOptions {
    mode: Option<String>,
    log: Option<PathBuf>,
    bundle: Option<PathBuf>,
    resources: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
struct ProjectCommand {
    path: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
struct ProjectDiscoverCommand {
    start: Option<PathBuf>,
    project: Option<PathBuf>,
    virtual_request: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
struct ProjectLockCommand {
    path: PathBuf,
    frozen: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct ProjectRunCommand {
    start: Option<PathBuf>,
    project: Option<PathBuf>,
    virtual_source: Option<PathBuf>,
    target: String,
    inspect_graph: bool,
    inspect_lock: bool,
    out: Option<PathBuf>,
    host_resources: HostResourceOptions,
}

#[derive(Debug, PartialEq, Eq)]
struct CurrentlineCommand {
    cell: PathBuf,
    context_json: Option<PathBuf>,
    context_out: Option<PathBuf>,
    summary_json: Option<PathBuf>,
    source_profile: String,
}

#[derive(Debug, PartialEq, Eq)]
struct WorkbenchCommand {
    no_open: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct LintCommand {
    path: PathBuf,
    source_profile: String,
    suggest_patch: bool,
    out: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
struct ExecutionError {
    message: String,
    exit_code: i32,
}

impl ExecutionError {
    fn runtime(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 1,
        }
    }

    fn input(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 2,
        }
    }
}

impl From<ddonirang_tool::source_frontdoor_workflow::SourceFrontdoorError>
    for ExecutionError
{
    fn from(error: ddonirang_tool::source_frontdoor_workflow::SourceFrontdoorError) -> Self {
        Self {
            message: error.message().to_string(),
            exit_code: error.exit_code(),
        }
    }
}

fn next_cli_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    let value = args.get(*index).filter(|value| !value.starts_with("--"));
    value
        .cloned()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("E_CLI_OPTION_VALUE_REQUIRED {flag}\n{}", usage()))
}

fn set_host_resource_mode(options: &mut HostResourceOptions, value: String) -> Result<(), String> {
    if options.mode.is_some() {
        return Err(format!("E_CLI_OPTION_DUPLICATE --open\n{}", usage()));
    }
    if !matches!(value.as_str(), "deny" | "record" | "replay") {
        return Err(format!("E_CLI_OPTION_VALUE_INVALID --open={value}\n{}", usage()));
    }
    options.mode = Some(value);
    Ok(())
}

fn set_host_resource_path(
    slot: &mut Option<PathBuf>,
    value: String,
    flag: &str,
) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!("E_CLI_OPTION_DUPLICATE {flag}\n{}", usage()));
    }
    if value.is_empty() {
        return Err(format!("E_CLI_OPTION_VALUE_REQUIRED {flag}\n{}", usage()));
    }
    *slot = Some(PathBuf::from(value));
    Ok(())
}

fn validate_host_resource_options(options: &HostResourceOptions) -> Result<(), String> {
    match options.mode.as_deref() {
        None => {
            if options.log.is_some() || options.bundle.is_some() || options.resources.is_some() {
                Err("E_HOST_RESOURCE_OPEN_MODE_REQUIRED".to_string())
            } else {
                Ok(())
            }
        }
        Some("deny") => {
            if options.log.is_some() || options.bundle.is_some() || options.resources.is_some() {
                Err("E_HOST_RESOURCE_DENY_INPUT_FORBIDDEN".to_string())
            } else {
                Ok(())
            }
        }
        Some("record") => {
            if options.resources.is_none() {
                Err("E_HOST_RESOURCE_MANIFEST_REQUIRED".to_string())
            } else if options.log.is_none() && options.bundle.is_none() {
                Err("E_HOST_RESOURCE_RECORD_ARTIFACT_REQUIRED".to_string())
            } else {
                Ok(())
            }
        }
        Some("replay") => {
            if options.resources.is_some() {
                Err("E_HOST_RESOURCE_REPLAY_RESOURCES_FORBIDDEN".to_string())
            } else if options.log.is_some() == options.bundle.is_some() {
                Err("E_HOST_RESOURCE_REPLAY_ARTIFACT_EXACTLY_ONE".to_string())
            } else {
                Ok(())
            }
        }
        Some(_) => unreachable!("host resource mode is parsed before validation"),
    }
}

fn prepare_host_resources_for_cli(
    options: &HostResourceOptions,
) -> Result<Option<ddonirang_tool::host_resource::PreparedHostResources>, ExecutionError> {
    match options.mode.as_deref() {
        None | Some("deny") => Ok(None),
        Some("record") => ddonirang_tool::host_resource::prepare_record_from_manifest_path(
            options.resources.as_deref().expect("validated resource manifest"),
        )
        .map(Some)
        .map_err(ExecutionError::input),
        Some("replay") => {
            let (path, bundle) = match (&options.log, &options.bundle) {
                (Some(path), None) => (path, false),
                (None, Some(path)) => (path, true),
                _ => unreachable!("validated replay artifact"),
            };
            ddonirang_tool::host_resource::read_replay_artifact(path, bundle)
                .map(Some)
                .map_err(ExecutionError::input)
        }
        Some(_) => unreachable!("validated host resource mode"),
    }
}

fn preflight_invocation_artifact_targets(
    options: &HostResourceOptions,
    extra_output: Option<&Path>,
) -> Result<(), ExecutionError> {
    let mut targets = Vec::new();
    if options.mode.as_deref() == Some("record") {
        if let Some(path) = options.log.as_deref() {
            targets.push(path);
        }
        if let Some(path) = options.bundle.as_deref() {
            targets.push(path);
        }
    }
    if let Some(path) = extra_output {
        targets.push(path);
    }
    ddonirang_tool::artifact_output::preflight_artifact_targets(&targets)
        .map_err(ExecutionError::runtime)
}

fn publish_invocation_artifacts(
    options: &HostResourceOptions,
    prepared: Option<&ddonirang_tool::host_resource::PreparedHostResources>,
    project_output: Option<(&Path, &str)>,
) -> Result<(), ExecutionError> {
    let mut owned = Vec::<(PathBuf, String)>::new();
    if options.mode.as_deref() == Some("record") {
        let prepared = prepared.expect("record has prepared resources");
        if let Some(path) = &options.log {
            owned.push((
                path.clone(),
                format!(
                    "{}\n",
                    ddonirang_tool::host_resource::transcript_json(prepared)
                        .map_err(ExecutionError::runtime)?
                ),
            ));
        }
        if let Some(path) = &options.bundle {
            owned.push((
                path.clone(),
                format!(
                    "{}\n",
                    ddonirang_tool::host_resource::bundle_json(prepared)
                        .map_err(ExecutionError::runtime)?
                ),
            ));
        }
    }
    if let Some((path, text)) = project_output {
        owned.push((path.to_path_buf(), text.to_string()));
    }
    if owned.is_empty() {
        return Ok(());
    }
    let payloads = owned
        .iter()
        .map(|(path, text)| ddonirang_tool::artifact_output::ArtifactPayload {
            target: path.as_path(),
            bytes: text.as_bytes(),
        })
        .collect::<Vec<_>>();
    ddonirang_tool::artifact_output::write_artifact_set_atomic(&payloads)
        .map_err(ExecutionError::runtime)
}

fn parse_source_command(
    command: &str,
    args: Vec<String>,
    allow_madi: bool,
) -> Result<SourceCommand, String> {
    let mut path = None;
    let mut source_profile = None;
    let mut requested_madi = None;
    let mut json_stdout = false;
    let mut summary_json = None;
    let mut host_resources = HostResourceOptions::default();
    let mut control_input = ControlInputOptions::default();
    let mut bogae_entry = None;
    let mut index = 0usize;

    while index < args.len() {
        if allow_madi && control_input.parse_option(&args, &mut index)? {
            index += 1;
            continue;
        }
        let argument = &args[index];
        match argument.as_str() {
            "--보개-시작" => {
                if !allow_madi || bogae_entry.is_some() {
                    return Err("E_CLI_BOGAE_ENTRY: 실행에 보개 시작 하나만 명시하세요".into());
                }
                index += 1;
                let value=args.get(index).filter(|value|!value.is_empty() && !value.starts_with('-'))
                    .ok_or("E_CLI_BOGAE_ENTRY: 보개 시작 정의가 필요합니다")?;
                bogae_entry=Some(value.clone());
            }
            SOURCE_PROFILE_FLAG => {
                if source_profile.is_some() {
                    return Err(format!(
                        "E_CLI_OPTION_DUPLICATE {SOURCE_PROFILE_FLAG}\n{}",
                        usage()
                    ));
                }
                index += 1;
                let value = args.get(index).filter(|value| !value.starts_with("--"));
                let Some(value) = value else {
                    return Err(format!(
                        "E_CLI_OPTION_VALUE_REQUIRED {SOURCE_PROFILE_FLAG}\n{}",
                        usage()
                    ));
                };
                source_profile = Some(value.clone());
            }
            "--마디수" | "--madi" if allow_madi => {
                if requested_madi.is_some() {
                    return Err(format!("E_CLI_OPTION_DUPLICATE --마디수(--madi): 중복 입력입니다\n{}", usage()));
                }
                index += 1;
                let value = args.get(index).filter(|value| !value.starts_with("--"));
                let Some(value) = value else {
                    return Err(format!("E_CLI_OPTION_VALUE_REQUIRED --마디수(--madi): 값이 필요합니다\n{}", usage()));
                };
                requested_madi = Some(value.parse::<u32>().map_err(|_| {
                    format!("E_CLI_OPTION_VALUE_INVALID --마디수(--madi)={value}: 양의 정수를 지정하세요\n{}", usage())
                })?);
                if requested_madi == Some(0) {
                    return Err(format!("E_CLI_OPTION_VALUE_INVALID --마디수(--madi)=0: 양의 정수를 지정하세요\n{}", usage()));
                }
            }
            "--json" if allow_madi => {
                if json_stdout {
                    return Err(format!("E_CLI_OPTION_DUPLICATE --json\n{}", usage()));
                }
                json_stdout = true;
            }
            "--summary-json" if allow_madi => {
                set_host_resource_path(
                    &mut summary_json,
                    next_cli_value(&args, &mut index, "--summary-json")?,
                    "--summary-json",
                )?;
            }
            "--open" if allow_madi => {
                set_host_resource_mode(&mut host_resources, next_cli_value(&args, &mut index, "--open")?)?;
            }
            "--open-log" if allow_madi => {
                set_host_resource_path(&mut host_resources.log, next_cli_value(&args, &mut index, "--open-log")?, "--open-log")?;
            }
            "--open-bundle" if allow_madi => {
                set_host_resource_path(&mut host_resources.bundle, next_cli_value(&args, &mut index, "--open-bundle")?, "--open-bundle")?;
            }
            "--open-resources" if allow_madi => {
                set_host_resource_path(&mut host_resources.resources, next_cli_value(&args, &mut index, "--open-resources")?, "--open-resources")?;
            }
            _ if argument.starts_with("--source-profile=") => {
                if source_profile.is_some() {
                    return Err(format!(
                        "E_CLI_OPTION_DUPLICATE {SOURCE_PROFILE_FLAG}\n{}",
                        usage()
                    ));
                }
                let value = argument.trim_start_matches("--source-profile=");
                if value.is_empty() {
                    return Err(format!(
                        "E_CLI_OPTION_VALUE_REQUIRED {SOURCE_PROFILE_FLAG}\n{}",
                        usage()
                    ));
                }
                source_profile = Some(value.to_string());
            }
            _ if allow_madi && (argument.starts_with("--마디수=") || argument.starts_with("--madi=")) => {
                if requested_madi.is_some() {
                    return Err(format!("E_CLI_OPTION_DUPLICATE --마디수(--madi): 중복 입력입니다\n{}", usage()));
                }
                let value = argument.split_once('=').expect("matched assignment").1;
                let parsed = value.parse::<u32>().map_err(|_| {
                    format!("E_CLI_OPTION_VALUE_INVALID --마디수(--madi)={value}: 양의 정수를 지정하세요\n{}", usage())
                })?;
                if parsed == 0 {
                    return Err(format!("E_CLI_OPTION_VALUE_INVALID --마디수(--madi)=0: 양의 정수를 지정하세요\n{}", usage()));
                }
                requested_madi = Some(parsed);
            }
            _ if allow_madi && argument.starts_with("--json=") => {
                return Err(format!("E_CLI_OPTION_VALUE_INVALID {argument}\n{}", usage()));
            }
            _ if allow_madi && argument.starts_with("--summary-json=") => {
                set_host_resource_path(
                    &mut summary_json,
                    argument.trim_start_matches("--summary-json=").to_string(),
                    "--summary-json",
                )?;
            }
            _ if allow_madi && argument.starts_with("--open=") => {
                set_host_resource_mode(&mut host_resources, argument.trim_start_matches("--open=").to_string())?;
            }
            _ if allow_madi && argument.starts_with("--open-log=") => {
                set_host_resource_path(&mut host_resources.log, argument.trim_start_matches("--open-log=").to_string(), "--open-log")?;
            }
            _ if allow_madi && argument.starts_with("--open-bundle=") => {
                set_host_resource_path(&mut host_resources.bundle, argument.trim_start_matches("--open-bundle=").to_string(), "--open-bundle")?;
            }
            _ if allow_madi && argument.starts_with("--open-resources=") => {
                set_host_resource_path(&mut host_resources.resources, argument.trim_start_matches("--open-resources=").to_string(), "--open-resources")?;
            }
            _ if argument.starts_with('-') => {
                return Err(format!("E_CLI_OPTION_UNKNOWN {argument}\n{}", usage()));
            }
            _ => {
                if path.is_some() {
                    return Err(format!(
                        "E_CLI_POSITIONAL_UNEXPECTED command={command} value={argument}\n{}",
                        usage()
                    ));
                }
                path = Some(PathBuf::from(argument));
            }
        }
        index += 1;
    }

    let path =
        path.ok_or_else(|| format!("E_CLI_SOURCE_REQUIRED command={command}\n{}", usage()))?;
    let source_profile = source_profile.ok_or_else(|| {
        format!(
            "E_SOURCE_PROFILE_REQUIRED {SOURCE_PROFILE_FLAG}\n{}",
            usage()
        )
    })?;
    validate_host_resource_options(&host_resources)?;
    control_input.validate(requested_madi)?;
    if control_input.active() && !matches!(host_resources.mode.as_deref(), None | Some("deny")) {
        return Err("E_CLI_OPTION_CONFLICT 제어입력과 --open 결합 실행은 지원하지 않습니다".into());
    }
    if bogae_entry.is_some() && (control_input.active() || !matches!(host_resources.mode.as_deref(), None|Some("deny"))) {
        return Err("E_CLI_OPTION_CONFLICT: 보개 시작과 제어입력·외부 자원 결합은 아직 지원하지 않습니다".into());
    }
    Ok(SourceCommand {
        path,
        source_profile,
        requested_madi,
        json_stdout,
        summary_json,
        host_resources,
        control_input,
        bogae_entry,
    })
}

fn parse_project_input(command: &str, args: &[String]) -> Result<ProjectCommand, String> {
    if let Some(option) = args.iter().find(|argument| argument.starts_with('-')) {
        return Err(format!("E_CLI_OPTION_UNKNOWN {option}\n{}", usage()));
    }
    if args.len() != 1 {
        return Err(format!(
            "E_CLI_PROJECT_INPUT_REQUIRED command={command}\n{}",
            usage()
        ));
    }
    Ok(ProjectCommand {
        path: PathBuf::from(&args[0]),
    })
}

fn parse_project_discover(args: &[String]) -> Result<ProjectDiscoverCommand, String> {
    let mut start = None;
    let mut project = None;
    let mut virtual_request = None;
    let mut index = 0usize;
    while index < args.len() {
        let argument = &args[index];
        let (slot, flag) = match argument.as_str() {
            "--project" => (Some(&mut project), "--project"),
            "--virtual" => (Some(&mut virtual_request), "--virtual"),
            _ => (None, ""),
        };
        if let Some(slot) = slot {
            if slot.is_some() {
                return Err(format!("E_CLI_OPTION_DUPLICATE {flag}\n{}", usage()));
            }
            index += 1;
            let value = args.get(index).filter(|value| !value.starts_with('-'));
            let Some(value) = value else {
                return Err(format!("E_CLI_OPTION_VALUE_REQUIRED {flag}\n{}", usage()));
            };
            *slot = Some(PathBuf::from(value));
        } else if let Some(value) = argument.strip_prefix("--project=") {
            if project.is_some() {
                return Err(format!("E_CLI_OPTION_DUPLICATE --project\n{}", usage()));
            }
            if value.is_empty() {
                return Err(format!(
                    "E_CLI_OPTION_VALUE_REQUIRED --project\n{}",
                    usage()
                ));
            }
            project = Some(PathBuf::from(value));
        } else if let Some(value) = argument.strip_prefix("--virtual=") {
            if virtual_request.is_some() {
                return Err(format!("E_CLI_OPTION_DUPLICATE --virtual\n{}", usage()));
            }
            if value.is_empty() {
                return Err(format!(
                    "E_CLI_OPTION_VALUE_REQUIRED --virtual\n{}",
                    usage()
                ));
            }
            virtual_request = Some(PathBuf::from(value));
        } else if argument.starts_with('-') {
            return Err(format!("E_CLI_OPTION_UNKNOWN {argument}\n{}", usage()));
        } else if start.is_some() {
            return Err(format!(
                "E_CLI_POSITIONAL_UNEXPECTED command=project-discover value={argument}\n{}",
                usage()
            ));
        } else {
            start = Some(PathBuf::from(argument));
        }
        index += 1;
    }
    if virtual_request.is_some() && (start.is_some() || project.is_some()) {
        return Err(format!(
            "E_CLI_OPTION_CONFLICT command=project-discover --virtual\n{}",
            usage()
        ));
    }
    Ok(ProjectDiscoverCommand {
        start,
        project,
        virtual_request,
    })
}

fn parse_project_lock(args: &[String]) -> Result<ProjectLockCommand, String> {
    let mut path = None;
    let mut frozen = false;
    for argument in args {
        if argument == "--frozen" {
            if frozen {
                return Err(format!("E_CLI_OPTION_DUPLICATE --frozen\n{}", usage()));
            }
            frozen = true;
        } else if argument.starts_with('-') {
            return Err(format!("E_CLI_OPTION_UNKNOWN {argument}\n{}", usage()));
        } else if path.is_some() {
            return Err(format!(
                "E_CLI_POSITIONAL_UNEXPECTED command=project-lock value={argument}\n{}",
                usage()
            ));
        } else {
            path = Some(PathBuf::from(argument));
        }
    }
    let path = path.ok_or_else(|| {
        format!(
            "E_CLI_PROJECT_INPUT_REQUIRED command=project-lock\n{}",
            usage()
        )
    })?;
    Ok(ProjectLockCommand { path, frozen })
}

fn parse_project_run(args: &[String]) -> Result<ProjectRunCommand, String> {
    let mut start = None;
    let mut project = None;
    let mut virtual_source = None;
    let mut target = None;
    let mut inspect_graph = false;
    let mut inspect_lock = false;
    let mut out = None;
    let mut host_resources = HostResourceOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        let argument = &args[index];
        let value_slot = match argument.as_str() {
            "--project" => Some((&mut project, "--project")),
            "--virtual" => Some((&mut virtual_source, "--virtual")),
            "--out" => Some((&mut out, "--out")),
            _ => None,
        };
        if let Some((slot, flag)) = value_slot {
            if slot.is_some() {
                return Err(format!("E_CLI_OPTION_DUPLICATE {flag}\n{}", usage()));
            }
            index += 1;
            let value = args.get(index).filter(|value| !value.starts_with('-'));
            let Some(value) = value else {
                return Err(format!("E_CLI_OPTION_VALUE_REQUIRED {flag}\n{}", usage()));
            };
            *slot = Some(PathBuf::from(value));
        } else if argument == "--target" {
            if target.is_some() {
                return Err(format!("E_CLI_OPTION_DUPLICATE --target\n{}", usage()));
            }
            index += 1;
            let value = args.get(index).filter(|value| !value.starts_with('-'));
            let Some(value) = value else {
                return Err(format!("E_CLI_OPTION_VALUE_REQUIRED --target\n{}", usage()));
            };
            target = Some(value.clone());
        } else if argument == "--inspect-graph" {
            if inspect_graph {
                return Err(format!(
                    "E_CLI_OPTION_DUPLICATE --inspect-graph\n{}",
                    usage()
                ));
            }
            inspect_graph = true;
        } else if argument == "--inspect-lock" {
            if inspect_lock {
                return Err(format!(
                    "E_CLI_OPTION_DUPLICATE --inspect-lock\n{}",
                    usage()
                ));
            }
            inspect_lock = true;
        } else if argument == "--open" {
            set_host_resource_mode(
                &mut host_resources,
                next_cli_value(args, &mut index, "--open")?,
            )?;
        } else if argument == "--open-log" {
            set_host_resource_path(
                &mut host_resources.log,
                next_cli_value(args, &mut index, "--open-log")?,
                "--open-log",
            )?;
        } else if argument == "--open-bundle" {
            set_host_resource_path(
                &mut host_resources.bundle,
                next_cli_value(args, &mut index, "--open-bundle")?,
                "--open-bundle",
            )?;
        } else if argument == "--open-resources" {
            set_host_resource_path(
                &mut host_resources.resources,
                next_cli_value(args, &mut index, "--open-resources")?,
                "--open-resources",
            )?;
        } else if let Some(value) = argument.strip_prefix("--project=") {
            if project.is_some() {
                return Err(format!("E_CLI_OPTION_DUPLICATE --project\n{}", usage()));
            }
            if value.is_empty() {
                return Err(format!(
                    "E_CLI_OPTION_VALUE_REQUIRED --project\n{}",
                    usage()
                ));
            }
            project = Some(PathBuf::from(value));
        } else if let Some(value) = argument.strip_prefix("--virtual=") {
            if virtual_source.is_some() {
                return Err(format!("E_CLI_OPTION_DUPLICATE --virtual\n{}", usage()));
            }
            if value.is_empty() {
                return Err(format!(
                    "E_CLI_OPTION_VALUE_REQUIRED --virtual\n{}",
                    usage()
                ));
            }
            virtual_source = Some(PathBuf::from(value));
        } else if let Some(value) = argument.strip_prefix("--target=") {
            if target.is_some() {
                return Err(format!("E_CLI_OPTION_DUPLICATE --target\n{}", usage()));
            }
            if value.is_empty() {
                return Err(format!("E_CLI_OPTION_VALUE_REQUIRED --target\n{}", usage()));
            }
            target = Some(value.to_string());
        } else if let Some(value) = argument.strip_prefix("--out=") {
            if out.is_some() {
                return Err(format!("E_CLI_OPTION_DUPLICATE --out\n{}", usage()));
            }
            if value.is_empty() {
                return Err(format!("E_CLI_OPTION_VALUE_REQUIRED --out\n{}", usage()));
            }
            out = Some(PathBuf::from(value));
        } else if let Some(value) = argument.strip_prefix("--open=") {
            set_host_resource_mode(&mut host_resources, value.to_string())?;
        } else if let Some(value) = argument.strip_prefix("--open-log=") {
            set_host_resource_path(&mut host_resources.log, value.to_string(), "--open-log")?;
        } else if let Some(value) = argument.strip_prefix("--open-bundle=") {
            set_host_resource_path(
                &mut host_resources.bundle,
                value.to_string(),
                "--open-bundle",
            )?;
        } else if let Some(value) = argument.strip_prefix("--open-resources=") {
            set_host_resource_path(
                &mut host_resources.resources,
                value.to_string(),
                "--open-resources",
            )?;
        } else if argument.starts_with('-') {
            return Err(format!("E_CLI_OPTION_UNKNOWN {argument}\n{}", usage()));
        } else if start.is_some() {
            return Err(format!(
                "E_CLI_POSITIONAL_UNEXPECTED command=project-run value={argument}\n{}",
                usage()
            ));
        } else {
            start = Some(PathBuf::from(argument));
        }
        index += 1;
    }
    if virtual_source.is_some() && (start.is_some() || project.is_some()) {
        return Err(format!(
            "E_CLI_OPTION_CONFLICT command=project-run --virtual\n{}",
            usage()
        ));
    }
    if virtual_source.is_none() && start.is_none() && project.is_none() {
        return Err("E_PROJECT_HEAD_REQUIRED: 실행할 작품 머리 파일을 지정해 주세요. 예: ddonilang project-run 작품.ddn".to_string());
    }
    validate_host_resource_options(&host_resources)?;
    Ok(ProjectRunCommand {
        start,
        project,
        virtual_source,
        // D9: only the authored root chooses its start definition.
        target: target.unwrap_or_default(),
        inspect_graph,
        inspect_lock,
        out,
        host_resources,
    })
}

fn parse_currentline_command(args: &[String]) -> Result<CurrentlineCommand, String> {
    let mut cell = None;
    let mut context_json = None;
    let mut context_out = None;
    let mut summary_json = None;
    let mut source_profile = None;
    let mut index = 0usize;
    while index < args.len() {
        let argument = &args[index];
        let (slot, flag): (Option<&mut Option<PathBuf>>, &str) = match argument.as_str() {
            "--cell" => (Some(&mut cell), "--cell"),
            "--context-json" => (Some(&mut context_json), "--context-json"),
            "--context-out" => (Some(&mut context_out), "--context-out"),
            "--summary-json" => (Some(&mut summary_json), "--summary-json"),
            _ => (None, ""),
        };
        if let Some(slot) = slot {
            if slot.is_some() {
                return Err(format!("E_CLI_OPTION_DUPLICATE {flag}\n{}", usage()));
            }
            index += 1;
            let value = args.get(index).filter(|value| !value.starts_with('-'));
            let Some(value) = value else {
                return Err(format!("E_CLI_OPTION_VALUE_REQUIRED {flag}\n{}", usage()));
            };
            *slot = Some(PathBuf::from(value));
        } else if argument == SOURCE_PROFILE_FLAG {
            if source_profile.is_some() {
                return Err(format!(
                    "E_CLI_OPTION_DUPLICATE {SOURCE_PROFILE_FLAG}\n{}",
                    usage()
                ));
            }
            index += 1;
            let value = args.get(index).filter(|value| !value.starts_with("--"));
            let Some(value) = value else {
                return Err(format!(
                    "E_CLI_OPTION_VALUE_REQUIRED {SOURCE_PROFILE_FLAG}\n{}",
                    usage()
                ));
            };
            source_profile = Some(value.clone());
        } else if let Some(value) = argument.strip_prefix("--source-profile=") {
            if source_profile.is_some() {
                return Err(format!(
                    "E_CLI_OPTION_DUPLICATE {SOURCE_PROFILE_FLAG}\n{}",
                    usage()
                ));
            }
            if value.is_empty() {
                return Err(format!(
                    "E_CLI_OPTION_VALUE_REQUIRED {SOURCE_PROFILE_FLAG}\n{}",
                    usage()
                ));
            }
            source_profile = Some(value.to_string());
        } else if argument.starts_with('-') {
            return Err(format!("E_CLI_OPTION_UNKNOWN {argument}\n{}", usage()));
        } else {
            return Err(format!(
                "E_CLI_POSITIONAL_UNEXPECTED command=currentline-run value={argument}\n{}",
                usage()
            ));
        }
        index += 1;
    }
    let cell = cell.ok_or_else(|| format!("E_CURRENTLINE_CELL_REQUIRED --cell\n{}", usage()))?;
    let source_profile = source_profile.ok_or_else(|| {
        format!(
            "E_SOURCE_PROFILE_REQUIRED {SOURCE_PROFILE_FLAG}\n{}",
            usage()
        )
    })?;
    Ok(CurrentlineCommand {
        cell,
        context_json,
        context_out,
        summary_json,
        source_profile,
    })
}

fn parse_lint_command(args: &[String]) -> Result<LintCommand, String> {
    let mut path = None;
    let mut source_profile = None;
    let mut suggest_patch = false;
    let mut out = None;
    let mut index = 0usize;
    while index < args.len() {
        let argument = &args[index];
        if argument == SOURCE_PROFILE_FLAG {
            if source_profile.is_some() {
                return Err(format!(
                    "E_CLI_OPTION_DUPLICATE {SOURCE_PROFILE_FLAG}\n{}",
                    usage()
                ));
            }
            index += 1;
            let value = args.get(index).filter(|value| !value.starts_with("--"));
            let Some(value) = value else {
                return Err(format!(
                    "E_CLI_OPTION_VALUE_REQUIRED {SOURCE_PROFILE_FLAG}\n{}",
                    usage()
                ));
            };
            source_profile = Some(value.clone());
        } else if let Some(value) = argument.strip_prefix("--source-profile=") {
            if source_profile.is_some() {
                return Err(format!(
                    "E_CLI_OPTION_DUPLICATE {SOURCE_PROFILE_FLAG}\n{}",
                    usage()
                ));
            }
            if value.is_empty() {
                return Err(format!(
                    "E_CLI_OPTION_VALUE_REQUIRED {SOURCE_PROFILE_FLAG}\n{}",
                    usage()
                ));
            }
            source_profile = Some(value.to_string());
        } else if argument == "--suggest-patch" {
            if suggest_patch {
                return Err(format!(
                    "E_CLI_OPTION_DUPLICATE --suggest-patch\n{}",
                    usage()
                ));
            }
            suggest_patch = true;
        } else if argument == "--out" {
            if out.is_some() {
                return Err(format!("E_CLI_OPTION_DUPLICATE --out\n{}", usage()));
            }
            index += 1;
            let value = args.get(index).filter(|value| !value.starts_with('-'));
            let Some(value) = value else {
                return Err(format!("E_CLI_OPTION_VALUE_REQUIRED --out\n{}", usage()));
            };
            out = Some(PathBuf::from(value));
        } else if argument.starts_with('-') {
            return Err(format!("E_CLI_OPTION_UNKNOWN {argument}\n{}", usage()));
        } else if path.is_some() {
            return Err(format!(
                "E_CLI_POSITIONAL_UNEXPECTED command=lint value={argument}\n{}",
                usage()
            ));
        } else {
            path = Some(PathBuf::from(argument));
        }
        index += 1;
    }
    if out.is_some() && !suggest_patch {
        return Err(format!(
            "E_CLI_OPTION_CONFLICT command=lint --out requires --suggest-patch\n{}",
            usage()
        ));
    }
    let path = path.ok_or_else(|| format!("E_CLI_SOURCE_REQUIRED command=lint\n{}", usage()))?;
    let source_profile = source_profile.ok_or_else(|| {
        format!(
            "E_SOURCE_PROFILE_REQUIRED {SOURCE_PROFILE_FLAG}\n{}",
            usage()
        )
    })?;
    Ok(LintCommand {
        path,
        source_profile,
        suggest_patch,
        out,
    })
}

fn command_help_requested(command: &str, rest: &[String]) -> bool {
    let [flag] = rest else {
        return false;
    };
    let help_flag = matches!(flag.as_str(), "--help" | "--도움" | "-h");
    help_flag
        && matches!(
            command,
            "help"
                | "도움"
                | "version"
                | "판본"
                | "run"
                | "실행"
                | "check"
                | "검사"
                | "build"
                | "짓기"
                | "canon"
                | "개발"
                | "lint"
                | "project-normalize"
                | "project-discover"
                | "project-graph"
                | "project-symbols"
                | "project-lock"
                | "project-run"
                | "currentline-run"
                | "workbench"
                | "logical-record" | "논리입력기록"
                | "logical-replay" | "논리입력재생"
                | "manipulation-record" | "조작입력기록"
                | "manipulation-replay" | "조작입력재생"
        )
}

fn parse_args(args: Vec<String>) -> Result<Command, String> {
    let Some((command, rest)) = args.split_first() else {
        return Ok(Command::Help);
    };
    if command_help_requested(command, rest) {
        return Ok(Command::Help);
    }
    match command.as_str() {
        "작품자료" | "authored-project" => {
            if matches!(rest.first().map(String::as_str), Some("실행" | "run")) {
                let madi = match rest.len() {
                    2 => None,
                    4 if rest[2] == "--마디수" => {
                        let value = rest[3].parse::<u32>().map_err(|_| "마디 수는 양의 정수여야 합니다")?;
                        if value == 0 { return Err("마디 수는 0일 수 없습니다".into()); }
                        Some(value)
                    }
                    _ => return Err("작품자료 실행 <파일.json> [--마디수 N]".into()),
                };
                return Ok(Command::AuthoredRun { input: PathBuf::from(&rest[1]), madi });
            }
            if rest.len() != 2 { return Err("작품자료 <만들기|열기|복제|갱신|보개등록|이름변경|경로변경> <파일.json>".into()); }
            let operation = match rest[0].as_str() {
                "만들기" | "create" => "create",
                "열기" | "open" => "open",
                "복제" | "clone" => "clone",
                "갱신" | "update" => "update",
                "보개등록" | "register_bogae" => "register_bogae",
                "이름변경" | "rename" => "rename",
                "경로변경" | "relocate" => "relocate",
                _ => return Err("지원하지 않는 작품자료 작업입니다".into()),
            };
            Ok(Command::AuthoredProject { operation: operation.into(), input: PathBuf::from(&rest[1]) })
        }
        "manipulation-record" | "조작입력기록" => {
            if rest.len()!=3 || rest.iter().any(|value|value.starts_with("--")) { return Err("E_CLI_MANIPULATION_ARGUMENTS: 원문·입력요청·출력기록 경로가 필요합니다".into()); }
            Ok(Command::ManipulationRecord {source:PathBuf::from(&rest[0]),request:PathBuf::from(&rest[1]),output:PathBuf::from(&rest[2])})
        }
        "manipulation-replay" | "조작입력재생" => {
            if rest.len()!=3 || rest.iter().any(|value|value.starts_with("--")) { return Err("E_CLI_MANIPULATION_ARGUMENTS: 원문·기록·입력계약 경로가 필요합니다".into()); }
            Ok(Command::ManipulationReplay {source:PathBuf::from(&rest[0]),tape:PathBuf::from(&rest[1]),binding:PathBuf::from(&rest[2])})
        }
        "logical-record" | "논리입력기록" => {
            if rest.len() != 3 || rest.iter().any(|value| value.starts_with("--")) {
                return Err("E_CLI_LOGICAL_ARGUMENTS: 원문·입력요청·출력기록 경로가 필요합니다".into());
            }
            Ok(Command::LogicalRecord { source: PathBuf::from(&rest[0]), request: PathBuf::from(&rest[1]), output: PathBuf::from(&rest[2]) })
        }
        "logical-replay" | "논리입력재생" => {
            if rest.len() != 3 || rest.iter().any(|value| value.starts_with("--")) {
                return Err("E_CLI_LOGICAL_ARGUMENTS: 원문·기록·입력계약 경로가 필요합니다".into());
            }
            Ok(Command::LogicalReplay { source: PathBuf::from(&rest[0]), tape: PathBuf::from(&rest[1]), binding: PathBuf::from(&rest[2]) })
        }
        "help" | "도움" | "--help" | "--도움" | "-h" => {
            if rest.is_empty() {
                Ok(Command::Help)
            } else {
                Err(format!(
                    "E_CLI_POSITIONAL_UNEXPECTED value={}\n{}",
                    rest[0],
                    usage()
                ))
            }
        }
        "version" | "판본" | "--version" | "--판본" | "-V" => {
            if rest.is_empty() {
                Ok(Command::Version)
            } else {
                Err(format!(
                    "E_CLI_POSITIONAL_UNEXPECTED value={}\n{}",
                    rest[0],
                    usage()
                ))
            }
        }
        "run" | "실행" => parse_source_command("run", rest.to_vec(), true).map(Command::Run),
        "check" | "검사" => {
            parse_source_command("check", rest.to_vec(), false).map(Command::Check)
        }
        "build" | "짓기" => {
            parse_source_command("build", rest.to_vec(), false).map(Command::Build)
        }
        "canon" => parse_source_command(command, rest.to_vec(), false).map(Command::Canon),
        "개발" => {
            let Some((developer_command, developer_rest)) = rest.split_first() else {
                return Err(format!(
                    "E_CLI_COMMAND_REQUIRED_AFTER_DEVELOPER\n{}",
                    usage()
                ));
            };
            match developer_command.as_str() {
                "정본"
                    if developer_rest.len() == 1
                        && matches!(developer_rest[0].as_str(), "--help" | "--도움" | "-h") =>
                {
                    Ok(Command::Help)
                }
                "정본" => parse_source_command("canon", developer_rest.to_vec(), false)
                    .map(Command::Canon),
                _ => Err(format!(
                    "E_CLI_COMMAND_UNKNOWN 개발 {developer_command}\n{}",
                    usage()
                )),
            }
        }
        "lint" => parse_lint_command(rest).map(Command::Lint),
        "project-normalize" => parse_project_input(command, rest).map(Command::ProjectNormalize),
        "project-discover" => parse_project_discover(rest).map(Command::ProjectDiscover),
        "project-graph" => parse_project_input(command, rest).map(Command::ProjectGraph),
        "project-symbols" => parse_project_input(command, rest).map(Command::ProjectSymbols),
        "project-lock" => parse_project_lock(rest).map(Command::ProjectLock),
        "project-run" => parse_project_run(rest).map(Command::ProjectRun),
        "currentline-run" => parse_currentline_command(rest).map(Command::CurrentlineRun),
        "workbench" => parse_workbench_command(rest).map(Command::Workbench),
        _ => Err(format!("E_CLI_COMMAND_UNKNOWN {command}\n{}", usage())),
    }
}

fn parse_workbench_command(args: &[String]) -> Result<WorkbenchCommand, String> {
    match args {
        [] => Ok(WorkbenchCommand { no_open: false }),
        [flag] if flag == "--no-open" => Ok(WorkbenchCommand { no_open: true }),
        [flag] if flag.starts_with("--") => Err(format!("E_CLI_OPTION_UNKNOWN {flag}\n{}", usage())),
        [value, ..] => Err(format!("E_CLI_POSITIONAL_UNEXPECTED value={value}\n{}", usage())),
    }
}

fn read_source(command: &SourceCommand) -> Result<String, ExecutionError> {
    let bytes = fs::read(&command.path).map_err(|error| {
        ExecutionError::runtime(format!(
            "E_CLI_SOURCE_READ {} {error}",
            command.path.display()
        ))
    })?;
    String::from_utf8(bytes).map_err(|error| {
        ExecutionError::runtime(format!(
            "E_CLI_SOURCE_UTF8 {} {error}",
            command.path.display()
        ))
    })
}

fn pretty_json(value: &ddonirang_tool::serde_json::Value) -> Result<String, ExecutionError> {
    ddonirang_tool::serde_json::to_string_pretty(value)
        .map(|text| format!("{text}\n"))
        .map_err(|error| ExecutionError::runtime(format!("E_CLI_JSON_SERIALIZE {error}")))
}

fn human_run_output(summary: &ddonirang_tool::serde_json::Value) -> Result<String, ExecutionError> {
    let display = summary.get("output_display").filter(|projection|
        projection["schema"] == "ddn.output-display.v1");
    let output_log = summary
        .get("output_log_texts")
        .and_then(ddonirang_tool::serde_json::Value::as_array)
        .ok_or_else(|| ExecutionError::runtime("E_CLI_OUTPUT_LOG_INVALID"))?;
    let mut output = String::new();
    for (index, value) in output_log.iter().enumerate() {
        let original = value
            .as_str()
            .ok_or_else(|| ExecutionError::runtime("E_CLI_OUTPUT_LOG_INVALID"))?;
        let text = display.and_then(|projection| projection["texts"].get(index))
            .and_then(ddonirang_tool::serde_json::Value::as_str).unwrap_or(original);
        output.push_str(text);
        output.push('\n');
    }
    Ok(output)
}

fn read_project_json(path: &PathBuf, error_code: &str) -> Result<String, ExecutionError> {
    fs::read_to_string(path).map_err(|error| {
        ExecutionError::input(format!(
            "{error_code} 입력 파일을 읽을 수 없습니다: {} ({error})",
            path.display()
        ))
    })
}

fn project_result(
    result: Result<String, ddonirang_tool::local_project::LocalProjectError>,
) -> Result<(String, i32), ExecutionError> {
    result
        .map(|output| (output, 0))
        .map_err(|error| ExecutionError::input(error.to_json_value().to_string()))
}

fn read_utf8_input(path: &PathBuf, error_code: &str) -> Result<String, ExecutionError> {
    let bytes = fs::read(path).map_err(|error| {
        ExecutionError::input(format!("{error_code} {} {error}", path.display()))
    })?;
    String::from_utf8(bytes).map_err(|error| {
        ExecutionError::input(format!("{error_code}_UTF8 {} {error}", path.display()))
    })
}

fn execute_control_input(command: &SourceCommand, source: &str) -> Result<(String, i32), ExecutionError> {
    use ddonirang_tool::artifact_output::{preflight_artifact_targets, write_artifact_set_atomic, ArtifactPayload};
    use ddonirang_tool::live_input::{LiveInput, LiveInputMode};
    let options = &command.control_input;
    let record_path = options.record.as_ref().map(PathBuf::from);
    let mut targets = Vec::new();
    for path in [record_path.as_deref(), command.summary_json.as_deref()].into_iter().flatten() {
        if existing_input_output_alias(&command.path, path)
            || options.tape.as_deref().is_some_and(|input| existing_input_output_alias(Path::new(input), path)) {
            return Err(ExecutionError::input("E_CLI_OUTPUT_ALIASES_INPUT 결과 경로가 입력 파일과 같습니다"));
        }
        targets.push(path);
    }
    preflight_artifact_targets(&targets).map_err(ExecutionError::runtime)?;
    let path = command.path.display().to_string();
    let (summary, tape_candidate) = if let Some(mode) = &options.mode {
        let hz = options.hz.as_deref().unwrap_or("60").parse::<u32>().map_err(|_| ExecutionError::input("E_SAM_BAD_MADI_HZ"))?;
        let port = options.port.as_deref().unwrap_or("8765").parse::<u16>().map_err(|_| ExecutionError::input("E_CLI_OPTION_VALUE_INVALID"))?;
        let mut live = LiveInput::new(
            if mode == "console" { LiveInputMode::Console } else { LiveInputMode::Web },
            options.host.clone().unwrap_or_else(|| "127.0.0.1".into()), port, hz, record_path,
        ).map_err(ExecutionError::runtime)?;
        let result = ddonirang_tool::runtime_surface::run_summary_from_source_with_live_input(
            source, &path, command.requested_madi, 0, Some(&command.source_profile), hz,
            |madi| live.sample_tick(madi)?.into_record(madi),
        );
        // Preserve the existing ordered-event V2 recording. Alpha1's
        // Workbench V1 compatibility path must not downgrade CLI evidence.
        live.seal_result(result).map_err(ExecutionError::runtime)?
    } else {
        let tape_path = options.tape.as_deref().expect("active validated tape source");
        let tape = ddonirang_tool::input_tape::read_input_tape(Path::new(tape_path)).map_err(ExecutionError::input)?;
        let summary = ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile_with_input_tape(
            source, &path, command.requested_madi, Some(&command.source_profile), &tape,
        ).map_err(ExecutionError::runtime)?;
        (summary, None)
    };
    let summary_json = pretty_json(&summary)?;
    let stdout = if command.json_stdout { summary_json.clone() } else { human_run_output(&summary)? };
    let mut payloads = Vec::new();
    if let Some((path, bytes)) = &tape_candidate {
        payloads.push(ArtifactPayload { target: path, bytes });
    }
    if let Some(path) = &command.summary_json {
        payloads.push(ArtifactPayload { target: path, bytes: summary_json.as_bytes() });
    }
    write_artifact_set_atomic(&payloads).map_err(ExecutionError::runtime)?;
    Ok((stdout, 0))
}

fn execute(command: Command) -> Result<(String, i32), ExecutionError> {
    match command {
        Command::AuthoredRun { input, madi } => {
            let text = String::from_utf8(read_logical_input(&input)?)
                .map_err(|_| ExecutionError::input("E_SOURCE_UTF8_INVALID: 작품자료가 UTF-8이 아닙니다"))?;
            let artifact = ddonirang_tool::authored_project::open(text.as_bytes())
                .map_err(ExecutionError::input)?;
            let (source, source_path, _) = artifact.execution_source()
                .map_err(ExecutionError::input)?;
            let summary = ddonirang_tool::runtime_surface::run_summary_with_authored_bogae(
                &source, &source_path, madi, &artifact.source_profile, &text, None, None,
            ).map_err(ExecutionError::runtime)?;
            Ok((pretty_json(&summary)? + "\n", 0))
        }
        Command::AuthoredProject { operation, input } => {
            let text = String::from_utf8(read_logical_input(&input)?)
                .map_err(|_| ExecutionError::input("E_SOURCE_UTF8_INVALID: 작품자료가 UTF-8이 아닙니다"))?;
            let result = ddonirang_tool::authored_project::operate(&operation, &text)
                .map_err(ExecutionError::input)?;
            Ok((result + "\n", 0))
        }
        Command::ManipulationRecord {source,request,output} => {
            if existing_input_output_alias(&source,&output)||existing_input_output_alias(&request,&output) {
                return Err(ExecutionError::input("E_CLI_OUTPUT_ALIASES_INPUT: 기록으로 입력을 덮어쓸 수 없습니다"));
            }
            ddonirang_tool::artifact_output::preflight_artifact_targets(&[output.as_path()]).map_err(ExecutionError::runtime)?;
            let source=String::from_utf8(read_logical_input(&source)?).map_err(|_|ExecutionError::input("E_SOURCE_UTF8_INVALID: 원문이 UTF-8이 아닙니다"))?;
            let request=ddonirang_tool::serde_json::from_slice(&read_logical_input(&request)?).map_err(|_|ExecutionError::input("E_MANIPULATION_REQUEST_JSON: 입력 요청이 올바르지 않습니다"))?;
            let (bytes,summary)=ddonirang_tool::manipulation_input::record_source(&source,&request).map_err(ExecutionError::runtime)?;
            ddonirang_tool::artifact_output::write_binary_artifact_atomic(&output,&bytes).map_err(ExecutionError::runtime)?;
            Ok((pretty_json(&summary)?+"\n",0))
        }
        Command::ManipulationReplay {source,tape,binding} => {
            let source=String::from_utf8(read_logical_input(&source)?).map_err(|_|ExecutionError::input("E_SOURCE_UTF8_INVALID: 원문이 UTF-8이 아닙니다"))?;
            let binding=ddonirang_tool::serde_json::from_slice(&read_logical_input(&binding)?).map_err(|_|ExecutionError::input("E_MANIPULATION_BINDING_JSON: 입력 계약이 올바르지 않습니다"))?;
            let summary=ddonirang_tool::manipulation_input::replay_source(&source,&read_logical_input(&tape)?,&binding).map_err(ExecutionError::runtime)?;
            Ok((pretty_json(&summary)?+"\n",0))
        }
        Command::LogicalRecord { source, request, output } => {
            if existing_input_output_alias(&source, &output) || existing_input_output_alias(&request, &output) {
                return Err(ExecutionError::input("E_CLI_OUTPUT_ALIASES_INPUT: 기록으로 입력을 덮어쓸 수 없습니다"));
            }
            ddonirang_tool::artifact_output::preflight_artifact_targets(&[output.as_path()])
                .map_err(ExecutionError::runtime)?;
            let source = read_logical_input(&source)?;
            let source = String::from_utf8(source).map_err(|_| ExecutionError::input("E_SOURCE_UTF8_INVALID: 원문이 UTF-8이 아닙니다"))?;
            let request = ddonirang_tool::serde_json::from_slice(&read_logical_input(&request)?)
                .map_err(|_| ExecutionError::input("E_LOGICAL_INPUT_REQUEST: 입력 요청 필드가 올바르지 않습니다"))?;
            let (bytes, summary) = ddonirang_tool::logical_input_tape::record_source(&source, &request)
                .map_err(ExecutionError::runtime)?;
            ddonirang_tool::artifact_output::write_binary_artifact_atomic(&output, &bytes)
                .map_err(ExecutionError::runtime)?;
            Ok((pretty_json(&summary)? + "\n", 0))
        }
        Command::LogicalReplay { source, tape, binding } => {
            let source = String::from_utf8(read_logical_input(&source)?)
                .map_err(|_| ExecutionError::input("E_SOURCE_UTF8_INVALID: 원문이 UTF-8이 아닙니다"))?;
            let expected = ddonirang_tool::serde_json::from_slice(&read_logical_input(&binding)?)
                .map_err(|_| ExecutionError::input("E_LOGICAL_INPUT_BINDING: 입력 계약 필드가 올바르지 않습니다"))?;
            let summary = ddonirang_tool::logical_input_tape::replay_source(&source, &read_logical_input(&tape)?, &expected)
                .map_err(ExecutionError::runtime)?;
            Ok((pretty_json(&summary)? + "\n", 0))
        }
        Command::Help => Ok((usage().to_string(), 0)),
        Command::Version => Ok((
            format!(
                "ddonirang {}\nsupported-source-profile: {}\n",
                env!("CARGO_PKG_VERSION"),
                ddonirang_tool::ddn_runtime::V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY
            ),
            0,
        )),
        Command::Run(command) => {
            let source = read_source(&command)?;
            let path = command.path.display().to_string();
            if command
                .summary_json
                .as_deref()
                .is_some_and(|output| existing_input_output_alias(&command.path, output))
            {
                return Err(ExecutionError::input("E_CLI_OUTPUT_ALIASES_INPUT"));
            }
            preflight_invocation_artifact_targets(
                &command.host_resources,
                command.summary_json.as_deref(),
            )?;
            ddonirang_tool::source_frontdoor_workflow::validate_supported_source_for_execution(
                &source,
                &command.path,
                &command.source_profile,
            )
            .map_err(ExecutionError::from)?;
            if command.control_input.active() {
                return execute_control_input(&command, &source);
            }
            let prepared = prepare_host_resources_for_cli(&command.host_resources)?;
            let summary = if let Some(entry) = &command.bogae_entry {
                ddonirang_tool::runtime_surface::run_summary_with_bogae_entry(
                    &source,&path,command.requested_madi,&command.source_profile,entry,
                )
            } else if let Some(prepared) = &prepared {
                ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile_with_host_resources(
                    &source,
                    &path,
                    command.requested_madi,
                    Some(&command.source_profile),
                    prepared,
                )
            } else {
                ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile(
                    &source,
                    &path,
                    command.requested_madi,
                    Some(&command.source_profile),
                )
            }
            .map_err(ExecutionError::runtime)?;
            let summary_json = pretty_json(&summary)?;
            let output = if command.json_stdout {
                summary_json.clone()
            } else {
                human_run_output(&summary)?
            };
            publish_invocation_artifacts(
                &command.host_resources,
                prepared.as_ref(),
                command
                    .summary_json
                    .as_deref()
                    .map(|path| (path, summary_json.as_str())),
            )?;
            Ok((output, 0))
        }
        Command::Check(command) | Command::Build(command) => {
            ddonirang_tool::source_frontdoor_workflow::run_check_or_build_supported_profile(
                &command.path,
                &command.source_profile,
            )
            .map_err(ExecutionError::from)
        }
        Command::Canon(command) => ddonirang_tool::source_frontdoor_workflow::run_canon_supported_profile(
            &command.path,
            &command.source_profile,
            None,
            false,
        )
        .map(|output| (output, 0))
        .map_err(ExecutionError::from),
        Command::Lint(command) => {
            let output = ddonirang_tool::source_frontdoor_workflow::run_lint_supported_profile(
                &command.path,
                &command.source_profile,
                command.suggest_patch,
                command.out.as_deref(),
            )
            .map_err(ExecutionError::from)?;
            for warning in output.warnings {
                eprintln!("{warning}");
            }
            Ok((output.stdout, 0))
        }
        Command::ProjectNormalize(command) => {
            let source_json = read_project_json(&command.path, "E_PROJECT_NORMALIZE_READ")?;
            let output =
                ddonirang_tool::local_project::normalize_virtual_project_source_json(&source_json)
                    .map_err(|error| ExecutionError::input(error.to_json_value().to_string()))?;
            Ok((output, 0))
        }
        Command::ProjectDiscover(command) => {
            if let Some(path) = command.virtual_request {
                let source_json = read_project_json(&path, "E_PROJECT_DISCOVER_READ")?;
                project_result(
                    ddonirang_tool::local_project::discover_virtual_project_root_json(&source_json),
                )
            } else {
                let start = command.start.unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                });
                project_result(
                    ddonirang_tool::local_project::discover_disk_project_root_json(
                        &start,
                        command.project.as_deref(),
                    ),
                )
            }
        }
        Command::ProjectGraph(command) => {
            let source_json = read_project_json(&command.path, "E_PROJECT_GRAPH_READ")?;
            project_result(ddonirang_tool::local_project::build_project_graph_json(
                &source_json,
            ))
        }
        Command::ProjectSymbols(command) => {
            let source_json = read_project_json(&command.path, "E_PROJECT_SYMBOLS_READ")?;
            project_result(
                ddonirang_tool::local_project::build_project_symbol_boundary_json(&source_json),
            )
        }
        Command::ProjectLock(command) => {
            let source_json = read_project_json(&command.path, "E_PROJECT_LOCK_READ")?;
            let result = if command.frozen {
                ddonirang_tool::local_project::verify_project_gaji_lock_json(&source_json)
            } else {
                ddonirang_tool::local_project::build_project_gaji_lock_json(&source_json)
            };
            project_result(result)
        }
        Command::ProjectRun(command) => {
            preflight_invocation_artifact_targets(
                &command.host_resources,
                command.out.as_deref(),
            )?;
            let prepared = prepare_host_resources_for_cli(&command.host_resources)?;
            let result = if let Some(path) = command.virtual_source {
                let source_json = read_project_json(&path, "E_PROJECT_RUN_READ")?;
                if let Some(prepared) = &prepared {
                    ddonirang_tool::local_project::build_project_run_summary_with_supported_profile_prepared_host_resources_json(
                        &source_json,
                        &command.target,
                        command.inspect_graph,
                        command.inspect_lock,
                        prepared,
                    )
                } else {
                    ddonirang_tool::local_project::build_project_run_summary_with_supported_profile_json(
                        &source_json,
                        &command.target,
                        command.inspect_graph,
                        command.inspect_lock,
                    )
                }
            } else {
                let start = command.start.unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                });
                if let Some(prepared) = &prepared {
                    ddonirang_tool::local_project::build_disk_project_run_summary_with_supported_profile_prepared_host_resources_json(
                        &start,
                        command.project.as_deref(),
                        &command.target,
                        command.inspect_graph,
                        command.inspect_lock,
                        prepared,
                    )
                } else {
                    ddonirang_tool::local_project::build_disk_project_run_summary_with_supported_profile_json(
                        &start,
                        command.project.as_deref(),
                        &command.target,
                        command.inspect_graph,
                        command.inspect_lock,
                    )
                }
            };
            let output =
                result.map_err(|error| ExecutionError::input(error.to_json_value().to_string()))?;
            if let Some(path) = command.out {
                if command.host_resources.mode.as_deref() == Some("record") {
                    publish_invocation_artifacts(
                        &command.host_resources,
                        prepared.as_ref(),
                        Some((&path, &output)),
                    )?;
                } else {
                    ddonirang_tool::local_project::write_project_run_artifact_atomic(&path, &output)
                        .map_err(|error| ExecutionError::runtime(error.to_json_value().to_string()))?;
                }
                Ok((String::new(), 0))
            } else {
                publish_invocation_artifacts(&command.host_resources, prepared.as_ref(), None)?;
                Ok((output, 0))
            }
        }
        Command::CurrentlineRun(command) => {
            let cell_source = read_utf8_input(&command.cell, "E_CURRENTLINE_CELL_READ")?;
            let context_text = command
                .context_json
                .as_ref()
                .map(|path| read_utf8_input(path, "E_CURRENTLINE_CONTEXT_READ"))
                .transpose()?;
            let currentline =
                ddonirang_tool::currentline_workflow::execute_currentline_with_supported_profile(
                    &cell_source,
                    &command.cell.display().to_string(),
                    context_text.as_deref(),
                    &command.source_profile,
                )
                .map_err(|error| match error.kind {
                    ddonirang_tool::currentline_workflow::CurrentlineWorkflowErrorKind::Input => {
                        ExecutionError::input(error.message)
                    }
                    ddonirang_tool::currentline_workflow::CurrentlineWorkflowErrorKind::Runtime => {
                        ExecutionError::runtime(error.message)
                    }
                })?;
            let summary_text = pretty_json(&currentline.summary)?;
            let mut outputs = Vec::new();
            if let Some(path) = command.summary_json {
                outputs.push((path, summary_text));
            }
            if let Some(path) = command.context_out {
                outputs.push((path, format!("{}\n", currentline.context_json)));
            }
            ddonirang_tool::currentline_workflow::write_currentline_outputs_atomic(&outputs)
                .map_err(|error| match error.kind {
                    ddonirang_tool::currentline_workflow::CurrentlineWorkflowErrorKind::Input => {
                        ExecutionError::input(error.message)
                    }
                    ddonirang_tool::currentline_workflow::CurrentlineWorkflowErrorKind::Runtime => {
                        ExecutionError::runtime(error.message)
                    }
                })?;
            Ok((currentline.stdout, 0))
        }
        Command::Workbench(_) => Err(ExecutionError::runtime("E_WORKBENCH_MAIN_DISPATCH_REQUIRED")),
    }
}

fn run_workbench(command: WorkbenchCommand) -> Result<(), ExecutionError> {
    let executable = std::env::current_exe()
        .map_err(|error| ExecutionError::runtime(format!("E_WORKBENCH_EXECUTABLE_PATH {error}")))?;
    let bundle_root = executable
        .parent()
        .ok_or_else(|| ExecutionError::runtime("E_WORKBENCH_EXECUTABLE_PARENT"))?
        .join("workbench");
    let host = ddonirang_tool::workbench_host::WorkbenchHost::start(
        &bundle_root,
        &bundle_root.join("asset_manifest.detjson"),
    )
    .map_err(ExecutionError::runtime)?;
    let mut record = serde_json::json!({
        "schema": "ddonirang.workbench_startup.v1",
        "ok": true,
        "profile": ddonirang_tool::ddn_runtime::V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
        "bind_address": "127.0.0.1",
        "startup_url": host.startup_url(),
        "port": host.port(),
        "asset_manifest_identity": host.manifest_sha256(),
        "browser_open_requested": !command.no_open,
    });
    if command.no_open {
        record["automation_token"] = serde_json::Value::String(host.token().to_string());
    }
    write_stdout(&(record.to_string() + "\n")).map_err(ExecutionError::runtime)?;
    if !command.no_open {
        #[cfg(target_os = "windows")]
        let opened = std::process::Command::new("explorer.exe")
            .arg(host.url())
            .spawn();
        #[cfg(not(target_os = "windows"))]
        let opened = std::process::Command::new("xdg-open").arg(host.url()).spawn();
        if let Err(error) = opened {
            eprintln!("E_WORKBENCH_BROWSER_OPEN {error}");
        }
    }
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn write_stdout(text: &str) -> Result<(), String> {
    io::stdout()
        .write_all(text.as_bytes())
        .map_err(|error| format!("E_CLI_STDOUT_WRITE {error}"))
}

fn read_logical_input(path: &Path) -> Result<Vec<u8>, ExecutionError> {
    let size = fs::metadata(path).map_err(|error| ExecutionError::input(format!("E_LOGICAL_INPUT_READ: {error}")))?.len();
    if size > 67_108_864 + 64 {
        return Err(ExecutionError::input("E_LOGICAL_INPUT_SIZE: 입력 크기 한도를 넘었습니다"));
    }
    let bytes = fs::read(path).map_err(|error| ExecutionError::input(format!("E_LOGICAL_INPUT_READ: {error}")))?;
    if bytes.len() > 67_108_864 + 64 {
        return Err(ExecutionError::input("E_LOGICAL_INPUT_SIZE: 입력 크기 한도를 넘었습니다"));
    }
    Ok(bytes)
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let command = match parse_args(args) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    if let Command::Workbench(command) = command {
        if let Err(error) = run_workbench(command) {
            eprintln!("{}", error.message);
            std::process::exit(error.exit_code);
        }
        return;
    }
    match execute(command) {
        Ok((output, exit_code)) => {
            if let Err(error) = write_stdout(&output) {
                eprintln!("{error}");
                std::process::exit(1);
            }
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
        }
        Err(error) => {
            eprintln!("{}", error.message);
            std::process::exit(error.exit_code);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn candidate_surface_is_small_and_does_not_name_legacy_cli() {
        let text = usage();
        assert!(text.contains("정본 실행파일: ddonilang, 또니랑; 호환 별칭: ddonirang"));
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
}
