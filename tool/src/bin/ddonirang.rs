use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const SOURCE_PROFILE_FLAG: &str = "--source-profile";

fn usage() -> &'static str {
    "또니랑 V1 명령줄 (정본 실행파일: ddonilang, 또니랑; 호환 별칭: ddonirang)\n\
사용법:\n\
  ddonilang 실행 <파일.ddn> --source-profile <profile> [--madi <N>] [--open deny|record|replay --open-log <transcript.json> --open-resources <manifest.json>]  (technical: run)\n\
  ddonilang 검사 <파일.ddn> --source-profile <profile>  (technical: check)\n\
  ddonilang 짓기 <파일.ddn> --source-profile <profile>  (technical: build)\n\
  ddonilang 개발 정본 <파일.ddn> --source-profile <profile>  (technical: canon)\n\
  ddonilang lint <파일.ddn> --source-profile <profile> [--suggest-patch] [--out <ddn.patch.json>]\n\
  ddonilang project-normalize <virtual-project.json>\n\
  ddonilang project-discover [시작경로] [--project <project.ddnproj>] [--virtual <request.json>]\n\
  ddonilang project-graph <virtual-project.json>\n\
  ddonilang project-symbols <virtual-project.json>\n\
  ddonilang project-lock <virtual-project.json> [--frozen]\n\
  ddonilang project-run [시작경로] [--project <project.ddnproj>] [--virtual <request.json>] [--target <이름>] [--inspect-graph] [--inspect-lock] [--open deny|record|replay --open-log <transcript.json> --open-resources <manifest.json>] [--out <artifact.detjson>]\n\
  ddonilang currentline-run --cell <파일.ddn> --source-profile <profile> [--context-json <context.detjson>] [--context-out <context.detjson>] [--summary-json <summary.detjson>]\n\
  ddonilang 도움  (technical: help)\n\
  ddonilang 판본  (technical: version)\n\n\
V1 지원 profile: v1-core-v25\n"
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
}

#[derive(Debug, PartialEq, Eq)]
struct SourceCommand {
    path: PathBuf,
    source_profile: String,
    requested_madi: Option<u32>,
    host_resources: HostResourceOptions,
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
    let mut host_resources = HostResourceOptions::default();
    let mut index = 0usize;

    while index < args.len() {
        let argument = &args[index];
        match argument.as_str() {
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
            "--madi" if allow_madi => {
                if requested_madi.is_some() {
                    return Err(format!("E_CLI_OPTION_DUPLICATE --madi\n{}", usage()));
                }
                index += 1;
                let value = args.get(index).filter(|value| !value.starts_with("--"));
                let Some(value) = value else {
                    return Err(format!("E_CLI_OPTION_VALUE_REQUIRED --madi\n{}", usage()));
                };
                requested_madi = Some(value.parse::<u32>().map_err(|_| {
                    format!("E_CLI_OPTION_VALUE_INVALID --madi={value}\n{}", usage())
                })?);
                if requested_madi == Some(0) {
                    return Err(format!("E_CLI_OPTION_VALUE_INVALID --madi=0\n{}", usage()));
                }
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
            _ if allow_madi && argument.starts_with("--madi=") => {
                if requested_madi.is_some() {
                    return Err(format!("E_CLI_OPTION_DUPLICATE --madi\n{}", usage()));
                }
                let value = argument.trim_start_matches("--madi=");
                let parsed = value.parse::<u32>().map_err(|_| {
                    format!("E_CLI_OPTION_VALUE_INVALID --madi={value}\n{}", usage())
                })?;
                if parsed == 0 {
                    return Err(format!("E_CLI_OPTION_VALUE_INVALID --madi=0\n{}", usage()));
                }
                requested_madi = Some(parsed);
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
    Ok(SourceCommand {
        path,
        source_profile,
        requested_madi,
        host_resources,
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
    validate_host_resource_options(&host_resources)?;
    Ok(ProjectRunCommand {
        start,
        project,
        virtual_source,
        target: target.unwrap_or_else(|| "앱".to_string()),
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
        _ => Err(format!("E_CLI_COMMAND_UNKNOWN {command}\n{}", usage())),
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

fn execute(command: Command) -> Result<(String, i32), ExecutionError> {
    match command {
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
            preflight_invocation_artifact_targets(&command.host_resources, None)?;
            let prepared = prepare_host_resources_for_cli(&command.host_resources)?;
            let summary = if let Some(prepared) = &prepared {
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
            let output = pretty_json(&summary)?;
            publish_invocation_artifacts(&command.host_resources, prepared.as_ref(), None)?;
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
    }
}

fn write_stdout(text: &str) -> Result<(), String> {
    io::stdout()
        .write_all(text.as_bytes())
        .map_err(|error| format!("E_CLI_STDOUT_WRITE {error}"))
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
            "v1-core-v25".into(), "--open".into(), "record".into(),
            "--open-log".into(), log_arg.clone(), "--open-resources".into(), manifest_arg,
        ])
        .expect("record command");
        let (record_output, _) = execute(record).expect("record execution");
        assert!(record_output.contains("host_resource_receipt"));
        assert!(log_path.is_file());

        std::fs::remove_file(&resource_path).expect("remove only task-owned declared input");
        let replay = parse_args(vec![
            "run".into(), source_arg, "--source-profile".into(), "v1-core-v25".into(),
            "--open".into(), "replay".into(), "--open-log".into(), log_arg,
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
