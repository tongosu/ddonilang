use std::path::PathBuf;

use crate::cli::run::RunEmitSink;
use clap::{Parser, Subcommand};
use std::env;

mod ai_prompt;
mod canon;
mod cli;
mod core;
mod file_meta;
mod lang;
mod runtime;

#[derive(Subcommand)]
enum WarpCommands {
    Bench {
        file: PathBuf,
        #[arg(long, value_enum, default_value_t = WarpBackendArg::Cpu)]
        backend: WarpBackendArg,
        #[arg(long, value_enum, default_value_t = WarpPolicyArg::Strict)]
        policy: WarpPolicyArg,
        #[arg(long, default_value_t = 1)]
        threads: usize,
        #[arg(long)]
        measure: bool,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum WarpBackendArg {
    Off,
    Cpu,
    Gpu,
}

impl WarpBackendArg {
    fn to_core(self) -> ddonirang_core::WarpBackend {
        match self {
            WarpBackendArg::Off => ddonirang_core::WarpBackend::Off,
            WarpBackendArg::Cpu => ddonirang_core::WarpBackend::Cpu,
            WarpBackendArg::Gpu => ddonirang_core::WarpBackend::Gpu,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum WarpPolicyArg {
    Strict,
    Fast,
}

impl WarpPolicyArg {
    fn to_core(self) -> ddonirang_core::WarpPolicy {
        match self {
            WarpPolicyArg::Strict => ddonirang_core::WarpPolicy::Strict,
            WarpPolicyArg::Fast => ddonirang_core::WarpPolicy::Fast,
        }
    }
}

#[derive(Parser)]
#[command(name = "teul-cli")]
#[command(about = "또니랑 실행 도구 (WALK02)")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    Run {
        file: PathBuf,
        #[arg(long, aliases = ["ticks", "max-madi"], value_name = "N|infinite")]
        madi: Option<String>,
        #[arg(long, default_value = "0x0")]
        seed: String,
        #[arg(long = "age-target")]
        age_target: Option<String>,
        #[arg(long = "state")]
        state: Vec<String>,
        #[arg(long = "state-file")]
        state_file: Vec<PathBuf>,
        #[arg(long = "diag-jsonl", alias = "diag")]
        diag_jsonl: Option<PathBuf>,
        #[arg(long = "diag-report-out")]
        diag_report_out: Option<PathBuf>,
        #[arg(long = "enable-repro")]
        enable_repro: bool,
        #[arg(long = "repro-json")]
        repro_json: Option<PathBuf>,
        #[arg(long = "run-manifest")]
        run_manifest: Option<PathBuf>,
        #[arg(long = "artifact")]
        artifact: Vec<String>,
        #[arg(long = "trace-json")]
        trace_json: Option<PathBuf>,
        #[arg(long = "geoul-out")]
        geoul_out: Option<PathBuf>,
        #[arg(long = "geoul-record-out")]
        geoul_record_out: Option<PathBuf>,
        #[arg(long = "trace-tier", value_enum, default_value_t = cli::trace_tier::TraceTierArg::TOff)]
        trace_tier: cli::trace_tier::TraceTierArg,
        #[arg(long = "lang-mode", value_enum)]
        lang_mode: Option<cli::lang_mode::LangModeArg>,
        #[arg(long = "bogae", value_enum)]
        bogae: Option<cli::bogae::BogaeMode>,
        #[arg(long = "bogae-codec", value_enum, default_value_t = cli::bogae::BogaeCodec::Bdl1)]
        bogae_codec: cli::bogae::BogaeCodec,
        #[arg(long = "bogae-out")]
        bogae_out: Option<PathBuf>,
        #[arg(long = "bogae-skin")]
        bogae_skin: Option<PathBuf>,
        #[arg(long = "bogae-overlay")]
        bogae_overlay: Option<String>,
        #[arg(long = "bogae-cmd-policy", value_enum, default_value_t = cli::bogae::BogaeCmdPolicy::None)]
        bogae_cmd_policy: cli::bogae::BogaeCmdPolicy,
        #[arg(long = "bogae-cmd-cap")]
        bogae_cmd_cap: Option<u32>,
        #[arg(long = "bogae-cache-log")]
        bogae_cache_log: bool,
        #[arg(long = "bogae-live")]
        bogae_live: bool,
        #[arg(
            long = "console-cell-aspect",
            value_enum,
            default_value_t = cli::bogae_console::ConsoleCellAspect::Auto
        )]
        console_cell_aspect: cli::bogae_console::ConsoleCellAspect,
        #[arg(long = "until-gameover")]
        until_gameover: bool,
        #[arg(long = "gameover-key", default_value = "게임오버")]
        gameover_key: String,
        #[arg(long = "console-grid", value_name = "COLSxROWS")]
        console_grid: Option<String>,
        #[arg(long = "console-panel-cols", default_value_t = 0)]
        console_panel_cols: usize,
        #[arg(long = "sam")]
        sam: Option<PathBuf>,
        #[arg(long = "record-sam")]
        record_sam: Option<PathBuf>,
        #[arg(long = "sam-live", value_enum)]
        sam_live: Option<cli::sam_live::SamLiveMode>,
        #[arg(long = "sam-live-host", default_value = "127.0.0.1")]
        sam_live_host: String,
        #[arg(long = "sam-live-port", default_value_t = 5001)]
        sam_live_port: u16,
        #[arg(long = "madi-hz")]
        madi_hz: Option<u32>,
        #[arg(long = "open", value_enum)]
        open_mode: Option<cli::open::OpenModeArg>,
        #[arg(long = "open-log")]
        open_log: Option<PathBuf>,
        #[arg(long = "open-bundle")]
        open_bundle: Option<PathBuf>,
        #[arg(long = "no-open")]
        no_open: bool,
        #[arg(long = "unsafe-open")]
        unsafe_open: bool,
    },
    View {
        file: PathBuf,
        #[arg(long = "bogae", value_enum, default_value_t = cli::bogae::BogaeMode::Web)]
        bogae: cli::bogae::BogaeMode,
        #[arg(long = "bogae-codec", value_enum)]
        bogae_codec: Option<cli::bogae::BogaeCodec>,
        #[arg(long = "bogae-out")]
        bogae_out: Option<PathBuf>,
        #[arg(long = "bogae-skin")]
        bogae_skin: Option<PathBuf>,
        #[arg(long = "bogae-overlay")]
        bogae_overlay: Option<String>,
        #[arg(
            long = "console-cell-aspect",
            value_enum,
            default_value_t = cli::bogae_console::ConsoleCellAspect::Auto
        )]
        console_cell_aspect: cli::bogae_console::ConsoleCellAspect,
        #[arg(long = "console-grid", value_name = "COLSxROWS")]
        console_grid: Option<String>,
        #[arg(long = "console-panel-cols", default_value_t = 0)]
        console_panel_cols: usize,
        #[arg(long = "no-open")]
        no_open: bool,
    },
    Check {
        file: PathBuf,
    },
    Test {
        file: Option<PathBuf>,
        #[arg(long, default_value_t = 1)]
        threads: usize,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        smoke: bool,
        #[arg(long)]
        golden: bool,
        #[arg(long = "pack")]
        pack: Vec<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        record: bool,
        #[arg(long)]
        update: bool,
        #[arg(long = "skip-ui-common")]
        skip_ui_common: bool,
        #[arg(long = "skip-wrapper")]
        skip_wrapper: bool,
    },
    Warp {
        #[command(subcommand)]
        command: WarpCommands,
    },
    Build {
        file: PathBuf,
    },
    Lint {
        file: PathBuf,
        #[arg(long = "suggest-patch")]
        suggest_patch: bool,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Repl,
    Worker,
    Canon {
        file: PathBuf,
        #[arg(long, value_enum, default_value_t = cli::canon::EmitKind::Ddn)]
        emit: cli::canon::EmitKind,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long = "fixits-json")]
        fixits_json: Option<PathBuf>,
        #[arg(long = "diag-jsonl", alias = "diag")]
        diag_jsonl: Option<PathBuf>,
        #[arg(long = "meta-out")]
        meta_out: Option<PathBuf>,
        #[arg(long)]
        bridge: Option<cli::canon::BridgeKind>,
        #[arg(long)]
        check: bool,
    },
    Asset {
        #[command(subcommand)]
        command: AssetCommands,
    },
    Bogae {
        #[command(subcommand)]
        command: BogaeCommands,
    },
    Dotbogi {
        #[command(subcommand)]
        command: DotbogiCommands,
    },
    Eco {
        #[command(subcommand)]
        command: EcoCommands,
    },
    Ai {
        #[command(subcommand)]
        command: AiCommands,
    },
    Bdl {
        #[command(subcommand)]
        command: BdlCommands,
    },
    Replay {
        #[command(subcommand)]
        command: ReplayCommands,
    },
    Geoul {
        #[command(subcommand)]
        command: GeoulCommands,
    },
    Patch {
        #[command(subcommand)]
        command: PatchCommands,
    },
    Scan {
        #[arg(long)]
        root: Option<PathBuf>,
    },
    Gaji {
        #[command(subcommand)]
        command: GajiCommands,
    },
    Intent {
        #[command(subcommand)]
        command: IntentCommands,
    },
    Goal {
        #[command(subcommand)]
        command: GoalCommands,
    },
    Goap {
        #[command(subcommand)]
        command: GoapCommands,
    },
    Observation {
        #[command(subcommand)]
        command: ObservationCommands,
    },
    #[command(name = "nurigym", alias = "nuri-gym")]
    NuriGym {
        #[command(subcommand)]
        command: NuriGymCommands,
    },
    Gateway {
        #[command(subcommand)]
        command: GatewayCommands,
    },
    Latency {
        #[command(subcommand)]
        command: LatencyCommands,
    },
    Safety {
        #[command(subcommand)]
        command: SafetyCommands,
    },
    Dataset {
        #[command(subcommand)]
        command: DatasetCommands,
    },
    Doc {
        #[command(subcommand)]
        command: DocCommands,
    },
    Reward {
        #[command(subcommand)]
        command: RewardCommands,
    },
    Train {
        config: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Imitation {
        config: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Eval {
        config: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Bundle {
        #[command(subcommand)]
        command: BundleCommands,
    },
    Edu {
        #[command(subcommand)]
        command: EduCommands,
    },
    Swarm {
        #[command(subcommand)]
        command: SwarmCommands,
    },
    Infer {
        #[command(subcommand)]
        command: InferCommands,
    },
    Alrim {
        #[command(subcommand)]
        command: AlrimCommands,
    },
    Story {
        #[command(subcommand)]
        command: StoryCommands,
    },
    Timeline {
        #[command(subcommand)]
        command: TimelineCommands,
    },
    Workshop {
        #[command(subcommand)]
        command: WorkshopCommands,
    },
}

#[derive(Subcommand)]
enum AiCommands {
    Extract {
        file: PathBuf,
        #[arg(long, default_value = "ai.request.json")]
        out: PathBuf,
    },
    Prompt {
        #[arg(long, default_value = "lean")]
        profile: String,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        bundle: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum AssetCommands {
    Manifest {
        root: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum BogaeCommands {
    Edit {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 0)]
        dx: i32,
        #[arg(long, default_value_t = 0)]
        dy: i32,
        #[arg(long)]
        color: Option<String>,
    },
    Bundle {
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        mapping: Option<PathBuf>,
        #[arg(long)]
        scene: Option<PathBuf>,
        #[arg(long)]
        assets: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum DotbogiCommands {
    Case {
        input: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long = "after-state-out")]
        after_state_out: Option<PathBuf>,
        #[arg(long = "report-out")]
        report_out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum EcoCommands {
    MacroMicro {
        input: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    NetworkFlow {
        input: PathBuf,
        #[arg(long = "madi", default_value_t = 1)]
        madi: u64,
        #[arg(long, default_value = "0x0")]
        seed: String,
        #[arg(long, default_value = "0.01")]
        threshold: String,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    AbmSpatial {
        input: PathBuf,
        #[arg(long = "madi", default_value_t = 1)]
        madi: u64,
        #[arg(long, default_value = "0x0")]
        seed: String,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum GajiCommands {
    Lock {
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long = "registry-index")]
        registry_index: Option<PathBuf>,
        #[arg(long = "snapshot-id")]
        snapshot_id: Option<String>,
        #[arg(long = "index-root-hash")]
        index_root_hash: Option<String>,
        #[arg(long = "trust-root-hash")]
        trust_root_hash: Option<String>,
        #[arg(long = "trust-root-source")]
        trust_root_source: Option<String>,
        #[arg(long = "audit-last-hash")]
        audit_last_hash: Option<String>,
    },
    Install {
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        lock: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long = "registry-index")]
        registry_index: Option<PathBuf>,
        #[arg(long = "verify-registry")]
        verify_registry: bool,
        #[arg(long = "strict-registry")]
        strict_registry: bool,
        #[arg(long = "registry-verify-out")]
        registry_verify_out: Option<PathBuf>,
        #[arg(long = "registry-audit-log")]
        registry_audit_log: Option<PathBuf>,
        #[arg(long = "verify-registry-audit")]
        verify_registry_audit: bool,
        #[arg(long = "registry-audit-verify-out")]
        registry_audit_verify_out: Option<PathBuf>,
        #[arg(long = "expect-audit-last-hash")]
        expect_audit_last_hash: Option<String>,
        #[arg(long = "frozen-lockfile")]
        frozen_lockfile: bool,
        #[arg(long = "expect-snapshot-id")]
        expect_snapshot_id: Option<String>,
        #[arg(long = "expect-index-root-hash")]
        expect_index_root_hash: Option<String>,
        #[arg(long = "expect-trust-root-hash")]
        expect_trust_root_hash: Option<String>,
        #[arg(long = "require-trust-root")]
        require_trust_root: bool,
        #[arg(long = "deny-yanked-locked")]
        deny_yanked_locked: bool,
    },
    Update {
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        lock: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long = "registry-index")]
        registry_index: Option<PathBuf>,
        #[arg(long = "verify-registry")]
        verify_registry: bool,
        #[arg(long = "strict-registry")]
        strict_registry: bool,
        #[arg(long = "registry-verify-out")]
        registry_verify_out: Option<PathBuf>,
        #[arg(long = "registry-audit-log")]
        registry_audit_log: Option<PathBuf>,
        #[arg(long = "verify-registry-audit")]
        verify_registry_audit: bool,
        #[arg(long = "registry-audit-verify-out")]
        registry_audit_verify_out: Option<PathBuf>,
        #[arg(long = "expect-audit-last-hash")]
        expect_audit_last_hash: Option<String>,
        #[arg(long = "snapshot-id")]
        snapshot_id: Option<String>,
        #[arg(long = "index-root-hash")]
        index_root_hash: Option<String>,
        #[arg(long = "trust-root-hash")]
        trust_root_hash: Option<String>,
        #[arg(long = "trust-root-source")]
        trust_root_source: Option<String>,
        #[arg(long = "audit-last-hash")]
        audit_last_hash: Option<String>,
        #[arg(long = "frozen-lockfile")]
        frozen_lockfile: bool,
        #[arg(long = "expect-snapshot-id")]
        expect_snapshot_id: Option<String>,
        #[arg(long = "expect-index-root-hash")]
        expect_index_root_hash: Option<String>,
        #[arg(long = "expect-trust-root-hash")]
        expect_trust_root_hash: Option<String>,
        #[arg(long = "require-trust-root")]
        require_trust_root: bool,
        #[arg(long = "deny-yanked-locked")]
        deny_yanked_locked: bool,
    },
    Vendor {
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        lock: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long = "registry-index")]
        registry_index: Option<PathBuf>,
        #[arg(long = "verify-registry")]
        verify_registry: bool,
        #[arg(long = "strict-registry")]
        strict_registry: bool,
        #[arg(long = "registry-verify-out")]
        registry_verify_out: Option<PathBuf>,
        #[arg(long = "registry-audit-log")]
        registry_audit_log: Option<PathBuf>,
        #[arg(long = "verify-registry-audit")]
        verify_registry_audit: bool,
        #[arg(long = "registry-audit-verify-out")]
        registry_audit_verify_out: Option<PathBuf>,
        #[arg(long = "expect-audit-last-hash")]
        expect_audit_last_hash: Option<String>,
        #[arg(long = "frozen-lockfile")]
        frozen_lockfile: bool,
        #[arg(long = "expect-snapshot-id")]
        expect_snapshot_id: Option<String>,
        #[arg(long = "expect-index-root-hash")]
        expect_index_root_hash: Option<String>,
        #[arg(long = "expect-trust-root-hash")]
        expect_trust_root_hash: Option<String>,
        #[arg(long = "require-trust-root")]
        require_trust_root: bool,
        #[arg(long = "deny-yanked-locked")]
        deny_yanked_locked: bool,
    },
    #[command(disable_help_flag = true)]
    Registry {
        #[arg(
            value_name = "REGISTRY_ARGS",
            num_args = 0..,
            allow_hyphen_values = true,
            trailing_var_arg = true
        )]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
enum BdlCommands {
    Packet {
        #[command(subcommand)]
        command: BdlPacketCommands,
    },
}

#[derive(Subcommand)]
enum BdlPacketCommands {
    Wrap {
        input: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    Unwrap {
        input: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Subcommand)]
enum BundleCommands {
    Parity {
        bundle_in: PathBuf,
        inputs: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long = "wasm-hash")]
        wasm_hash: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ReplayCommands {
    Diff {
        #[arg(long)]
        a: PathBuf,
        #[arg(long)]
        b: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        no_summary: bool,
    },
    Verify {
        #[arg(long = "geoul")]
        geoul: PathBuf,
        #[arg(long = "until")]
        until: Option<u64>,
        #[arg(long = "seek")]
        seek: Option<u64>,
        #[arg(long = "entry")]
        entry: Option<PathBuf>,
    },
    Branch {
        #[arg(long = "geoul")]
        geoul: PathBuf,
        #[arg(long = "at")]
        at: u64,
        #[arg(long = "inject-sam")]
        inject_sam: PathBuf,
        #[arg(long = "out")]
        out: PathBuf,
        #[arg(long = "entry")]
        entry: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum GeoulCommands {
    Hash {
        #[arg(long = "geoul")]
        geoul: PathBuf,
    },
    Seek {
        #[arg(long = "geoul")]
        geoul: PathBuf,
        #[arg(long = "madi")]
        madi: u64,
    },
    Query {
        #[arg(long = "geoul")]
        geoul: PathBuf,
        #[arg(long = "madi")]
        madi: u64,
        #[arg(long = "key")]
        key: String,
        #[arg(long = "entry")]
        entry: Option<PathBuf>,
    },
    Backtrace {
        #[arg(long = "geoul")]
        geoul: PathBuf,
        #[arg(long = "key")]
        key: String,
        #[arg(long = "from")]
        from: u64,
        #[arg(long = "to")]
        to: u64,
        #[arg(long = "entry")]
        entry: Option<PathBuf>,
    },
    Record {
        #[command(subcommand)]
        command: GeoulRecordCommands,
    },
}

#[derive(Subcommand)]
enum GeoulRecordCommands {
    Make {
        input: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Check {
        input: PathBuf,
    },
}

#[derive(Subcommand)]
enum PatchCommands {
    Propose {
        file: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Preview {
        patch: PathBuf,
        #[arg(long, value_enum, default_value_t = cli::patch::PreviewFormat::Diff)]
        format: cli::patch::PreviewFormat,
    },
    Approve {
        patch: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        notes: Option<String>,
    },
    Apply {
        patch: PathBuf,
        #[arg(long)]
        approval: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long = "in-place")]
        in_place: bool,
    },
    Verify {
        patch: PathBuf,
        #[arg(long)]
        approval: PathBuf,
        #[arg(long)]
        tests: Option<PathBuf>,
        #[arg(long)]
        walk: Option<String>,
    },
}

#[derive(Subcommand)]
enum IntentCommands {
    Inspect {
        #[arg(long = "geoul")]
        geoul: PathBuf,
        #[arg(long = "madi")]
        madi: Option<u64>,
        #[arg(long = "agent")]
        agent: Option<u64>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Mock {
        input: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 0)]
        agent_id: u64,
        #[arg(long, default_value_t = 0)]
        madi: u64,
        #[arg(long, default_value_t = 0)]
        recv_seq: u64,
    },
    Merge {
        #[arg(long = "in")]
        inputs: Vec<PathBuf>,
        #[arg(long = "madi")]
        madi: Option<u64>,
        #[arg(long = "agent")]
        agent: Option<u64>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum GoalCommands {
    Parse {
        input: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Plan {
        #[arg(long)]
        actions: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum GoapCommands {
    Plan {
        input: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ObservationCommands {
    Canon {
        input: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum NuriGymCommands {
    Spec {
        #[arg(long = "from")]
        from: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        slots: Option<u32>,
    },
    View {
        #[arg(long = "spec")]
        spec: PathBuf,
    },
    Run {
        input: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum GatewayCommands {
    Serve {
        #[arg(long)]
        world: PathBuf,
        #[arg(long, default_value_t = 1)]
        threads: usize,
        #[arg(long)]
        input: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        realms: Option<u64>,
        #[arg(long, value_enum, default_value_t = GatewayInputFormatArg::Auto)]
        input_format: GatewayInputFormatArg,
        #[arg(long)]
        listen: Option<String>,
        #[arg(long, value_enum, default_value_t = GatewayListenProtoArg::Tcp)]
        listen_proto: GatewayListenProtoArg,
        #[arg(long)]
        listen_max_events: Option<u64>,
        #[arg(long)]
        listen_timeout_ms: Option<u64>,
        #[arg(long)]
        send: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = GatewayInputFormatArg::Auto)]
        send_format: GatewayInputFormatArg,
    },
    #[command(name = "load-sim")]
    LoadSim {
        #[arg(long, default_value_t = 100)]
        clients: u64,
        #[arg(long, default_value_t = 60)]
        ticks: u64,
        #[arg(long, default_value_t = 0)]
        seed: u64,
        #[arg(long, default_value_t = 1)]
        realms: u64,
        #[arg(long = "tick-hz", default_value_t = 60)]
        tick_hz: u64,
        #[arg(long, default_value_t = 1)]
        threads: usize,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum LatencyCommands {
    Simulate {
        #[arg(long = "L", default_value_t = 3)]
        l_madi: u64,
        #[arg(long, value_enum, default_value_t = LatencyModeArg::Fixed)]
        mode: LatencyModeArg,
        #[arg(long, default_value_t = 10)]
        count: u64,
        #[arg(long, default_value_t = 0)]
        seed: u64,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum GatewayInputFormatArg {
    Auto,
    Detjson,
    Jsonl,
    Sam,
}

impl GatewayInputFormatArg {
    fn to_core(self) -> cli::gateway::InputFormat {
        match self {
            GatewayInputFormatArg::Auto => cli::gateway::InputFormat::Auto,
            GatewayInputFormatArg::Detjson => cli::gateway::InputFormat::DetJson,
            GatewayInputFormatArg::Jsonl => cli::gateway::InputFormat::Jsonl,
            GatewayInputFormatArg::Sam => cli::gateway::InputFormat::Sam,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum GatewayListenProtoArg {
    Tcp,
    Udp,
}

impl GatewayListenProtoArg {
    fn to_core(self) -> cli::gateway::ListenProtocol {
        match self {
            GatewayListenProtoArg::Tcp => cli::gateway::ListenProtocol::Tcp,
            GatewayListenProtoArg::Udp => cli::gateway::ListenProtocol::Udp,
        }
    }
}

#[derive(Subcommand)]
enum SafetyCommands {
    Check {
        #[arg(long)]
        rules: PathBuf,
        #[arg(long)]
        intent: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum DatasetCommands {
    Export {
        #[arg(long = "geoul")]
        geoul: PathBuf,
        #[arg(long, default_value = "nurigym_v0")]
        format: String,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value = "nurigym.unknown")]
        env_id: String,
    },
}

#[derive(Subcommand)]
enum DocCommands {
    Build {
        #[arg(long, default_value = "dist/malmoi")]
        out: PathBuf,
    },
    Verify {
        #[arg(long)]
        pack: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum RewardCommands {
    Check {
        input: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum EduCommands {
    Accuracy {
        input: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum SwarmCommands {
    Collision {
        input: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum InferCommands {
    Mlp {
        model: PathBuf,
        input: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum AlrimCommands {
    Registry {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum WorkshopCommands {
    Gen {
        #[arg(long = "geoul")]
        geoul: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    Apply {
        #[arg(long)]
        workshop: PathBuf,
        #[arg(long)]
        patch: PathBuf,
    },
    Open {
        #[arg(long)]
        workshop: PathBuf,
    },
}

#[derive(Subcommand)]
enum StoryCommands {
    Make {
        #[arg(long = "geoul")]
        geoul: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Subcommand)]
enum TimelineCommands {
    Make {
        #[arg(long = "geoul")]
        geoul: PathBuf,
        #[arg(long = "story")]
        story: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum LatencyModeArg {
    Fixed,
    Jitter,
}

impl LatencyModeArg {
    fn to_core(self) -> ddonirang_core::seulgi::latency::LatencyMode {
        match self {
            LatencyModeArg::Fixed => ddonirang_core::seulgi::latency::LatencyMode::Fixed,
            LatencyModeArg::Jitter => ddonirang_core::seulgi::latency::LatencyMode::Jitter,
        }
    }
}

pub(crate) struct RunCommandArgs {
    pub(crate) file: PathBuf,
    pub(crate) madi: Option<String>,
    pub(crate) seed: String,
    pub(crate) age_target: Option<String>,
    pub(crate) state: Vec<String>,
    pub(crate) state_file: Vec<PathBuf>,
    pub(crate) diag_jsonl: Option<PathBuf>,
    pub(crate) diag_report_out: Option<PathBuf>,
    pub(crate) enable_repro: bool,
    pub(crate) repro_json: Option<PathBuf>,
    pub(crate) run_manifest: Option<PathBuf>,
    pub(crate) artifact: Vec<String>,
    pub(crate) trace_json: Option<PathBuf>,
    pub(crate) geoul_out: Option<PathBuf>,
    pub(crate) geoul_record_out: Option<PathBuf>,
    pub(crate) trace_tier: cli::trace_tier::TraceTierArg,
    pub(crate) lang_mode: Option<cli::lang_mode::LangModeArg>,
    pub(crate) bogae: Option<cli::bogae::BogaeMode>,
    pub(crate) bogae_codec: cli::bogae::BogaeCodec,
    pub(crate) bogae_out: Option<PathBuf>,
    pub(crate) bogae_skin: Option<PathBuf>,
    pub(crate) bogae_overlay: Option<String>,
    pub(crate) bogae_cmd_policy: cli::bogae::BogaeCmdPolicy,
    pub(crate) bogae_cmd_cap: Option<u32>,
    pub(crate) bogae_cache_log: bool,
    pub(crate) bogae_live: bool,
    pub(crate) console_cell_aspect: cli::bogae_console::ConsoleCellAspect,
    pub(crate) console_grid: Option<String>,
    pub(crate) console_panel_cols: usize,
    pub(crate) until_gameover: bool,
    pub(crate) gameover_key: String,
    pub(crate) sam: Option<PathBuf>,
    pub(crate) record_sam: Option<PathBuf>,
    pub(crate) sam_live: Option<cli::sam_live::SamLiveMode>,
    pub(crate) sam_live_host: String,
    pub(crate) sam_live_port: u16,
    pub(crate) madi_hz: Option<u32>,
    pub(crate) open_mode: Option<cli::open::OpenModeArg>,
    pub(crate) open_log: Option<PathBuf>,
    pub(crate) open_bundle: Option<PathBuf>,
    pub(crate) no_open: bool,
    pub(crate) unsafe_open: bool,
    pub(crate) run_command_override: Option<String>,
}

pub(crate) fn execute_run_command(
    args: RunCommandArgs,
    emit: &mut dyn cli::run::RunEmitSink,
) -> Result<(), String> {
    let RunCommandArgs {
        file,
        madi,
        seed,
        age_target,
        state,
        state_file,
        diag_jsonl,
        diag_report_out,
        enable_repro,
        repro_json,
        run_manifest,
        artifact,
        trace_json,
        geoul_out,
        geoul_record_out,
        trace_tier,
        lang_mode,
        bogae,
        bogae_codec,
        bogae_out,
        bogae_skin,
        bogae_overlay,
        bogae_cmd_policy,
        bogae_cmd_cap,
        bogae_cache_log,
        bogae_live,
        console_cell_aspect,
        console_grid,
        console_panel_cols,
        until_gameover,
        gameover_key,
        sam,
        record_sam,
        sam_live,
        sam_live_host,
        sam_live_port,
        madi_hz,
        open_mode,
        open_log,
        open_bundle,
        no_open,
        unsafe_open,
        run_command_override,
    } = args;

    let seed = parse_seed(&seed).map_err(|message| format!("E_CLI_BAD_SEED {}", message))?;
    let madi =
        parse_madi_arg(madi.as_deref()).map_err(|message| format!("E_CLI_MADI {}", message))?;
    let console_grid = parse_console_grid(console_grid.as_deref())
        .map_err(|message| format!("E_CLI_CONSOLE_GRID {}", message))?;
    if console_panel_cols > 0 && console_grid.is_none() {
        return Err("E_CLI_CONSOLE_PANEL console-panel-cols requires --console-grid".to_string());
    }
    let repro_json = match (enable_repro, repro_json) {
        (true, Some(path)) => Some(path),
        (true, None) => Some(
            cli::paths::build_dir()
                .join("repro")
                .join("ddn.repro.last.json"),
        ),
        (false, Some(path)) => Some(path),
        (false, None) => None,
    };
    let artifact_pins = cli::run::parse_artifact_pins(&artifact)
        .map_err(|message| format!("E_CLI_ARTIFACT {}", message))?;
    let cmd_policy = match bogae_cmd_policy {
        cli::bogae::BogaeCmdPolicy::None => {
            if bogae_cmd_cap.is_some() {
                return Err(
                    "E_CLI_BOGAE_CMD_POLICY bogae-cmd-cap은 policy=none에서 사용할 수 없습니다."
                        .to_string(),
                );
            }
            crate::core::bogae::CmdPolicyConfig::none()
        }
        cli::bogae::BogaeCmdPolicy::Cap => {
            let cap = match bogae_cmd_cap {
                Some(value) if value > 0 => value,
                _ => {
                    return Err(
                        "E_CLI_BOGAE_CMD_POLICY bogae-cmd-cap(>0)이 필요합니다.".to_string()
                    );
                }
            };
            crate::core::bogae::CmdPolicyConfig {
                mode: crate::core::bogae::CmdPolicyMode::Cap,
                cap,
            }
        }
        cli::bogae::BogaeCmdPolicy::Summary => {
            let cap = match bogae_cmd_cap {
                Some(value) if value > 0 => value,
                _ => {
                    return Err(
                        "E_CLI_BOGAE_CMD_POLICY bogae-cmd-cap(>0)이 필요합니다.".to_string()
                    );
                }
            };
            crate::core::bogae::CmdPolicyConfig {
                mode: crate::core::bogae::CmdPolicyMode::Summary,
                cap,
            }
        }
    };
    let overlay = match bogae_overlay {
        Some(value) => cli::bogae::OverlayConfig::from_csv(&value)
            .map_err(|message| format!("E_CLI_BOGAE_OVERLAY {}", message))?,
        None => cli::bogae::OverlayConfig::empty(),
    };
    let bogae_codec = match bogae_codec {
        cli::bogae::BogaeCodec::Bdl1 => crate::core::bogae::BogaeCodec::Bdl1,
        cli::bogae::BogaeCodec::Bdl2 => crate::core::bogae::BogaeCodec::Bdl2,
    };
    let mut console_config =
        cli::bogae_console::ConsoleRenderConfig::with_cell_aspect(console_cell_aspect);
    console_config.panel_cols = console_panel_cols;
    if let Some((cols, rows)) = console_grid {
        console_config.grid_cols = Some(cols);
        console_config.grid_rows = Some(rows);
    }
    let run_command = run_command_override.or_else(|| Some(build_command_string()));
    let options = cli::run::RunOptions {
        diag_jsonl,
        diag_report_out,
        repro_json,
        trace_json,
        geoul_out,
        geoul_record_out,
        trace_tier: trace_tier.to_core(),
        age_target,
        lang_mode,
        bogae_mode: bogae,
        bogae_codec,
        bogae_out,
        bogae_skin,
        overlay,
        cmd_policy,
        bogae_cache_log,
        bogae_live,
        console_config,
        until_gameover,
        gameover_key,
        sam_path: sam,
        record_sam_path: record_sam,
        sam_live,
        sam_live_host,
        sam_live_port,
        madi_hz,
        open_mode: open_mode.map(|mode| mode.to_runtime()),
        open_log,
        open_bundle,
        no_open,
        unsafe_open,
        run_manifest,
        artifact_pins,
        run_command,
        init_state: state,
        init_state_files: state_file,
    };
    cli::run::run_file_with_emitter(&file, madi, seed, options, emit)
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run {
            file,
            madi,
            seed,
            age_target,
            state,
            state_file,
            diag_jsonl,
            diag_report_out,
            enable_repro,
            repro_json,
            run_manifest,
            artifact,
            trace_json,
            geoul_out,
            geoul_record_out,
            trace_tier,
            lang_mode,
            bogae,
            bogae_codec,
            bogae_out,
            bogae_skin,
            bogae_overlay,
            bogae_cmd_policy,
            bogae_cmd_cap,
            bogae_cache_log,
            bogae_live,
            console_cell_aspect,
            console_grid,
            console_panel_cols,
            until_gameover,
            gameover_key,
            sam,
            record_sam,
            sam_live,
            sam_live_host,
            sam_live_port,
            madi_hz,
            open_mode,
            open_log,
            open_bundle,
            no_open,
            unsafe_open,
        } => {
            let mut emitter = cli::run::StdoutRunEmitter;
            let run_args = RunCommandArgs {
                file,
                madi,
                seed,
                age_target,
                state,
                state_file,
                diag_jsonl,
                diag_report_out,
                enable_repro,
                repro_json,
                run_manifest,
                artifact,
                trace_json,
                geoul_out,
                geoul_record_out,
                trace_tier,
                lang_mode,
                bogae,
                bogae_codec,
                bogae_out,
                bogae_skin,
                bogae_overlay,
                bogae_cmd_policy,
                bogae_cmd_cap,
                bogae_cache_log,
                bogae_live,
                console_cell_aspect,
                console_grid,
                console_panel_cols,
                until_gameover,
                gameover_key,
                sam,
                record_sam,
                sam_live,
                sam_live_host,
                sam_live_port,
                madi_hz,
                open_mode,
                open_log,
                open_bundle,
                no_open,
                unsafe_open,
                run_command_override: None,
            };
            if let Err(err) = execute_run_command(run_args, &mut emitter) {
                emitter.err(&err);
                std::process::exit(1);
            }
        }
        Commands::View {
            file,
            bogae,
            bogae_codec,
            bogae_out,
            bogae_skin,
            bogae_overlay,
            console_cell_aspect,
            console_grid,
            console_panel_cols,
            no_open,
        } => {
            let overlay = match bogae_overlay {
                Some(value) => match cli::bogae::OverlayConfig::from_csv(&value) {
                    Ok(config) => config,
                    Err(message) => {
                        eprintln!("E_CLI_BOGAE_OVERLAY {}", message);
                        std::process::exit(1);
                    }
                },
                None => cli::bogae::OverlayConfig::empty(),
            };
            let bogae_codec = bogae_codec.map(|codec| match codec {
                cli::bogae::BogaeCodec::Bdl1 => crate::core::bogae::BogaeCodec::Bdl1,
                cli::bogae::BogaeCodec::Bdl2 => crate::core::bogae::BogaeCodec::Bdl2,
            });
            let console_grid = match parse_console_grid(console_grid.as_deref()) {
                Ok(value) => value,
                Err(message) => {
                    eprintln!("E_CLI_CONSOLE_GRID {}", message);
                    std::process::exit(1);
                }
            };
            if console_panel_cols > 0 && console_grid.is_none() {
                eprintln!("E_CLI_CONSOLE_PANEL console-panel-cols requires --console-grid");
                std::process::exit(1);
            }
            let mut console_config =
                cli::bogae_console::ConsoleRenderConfig::with_cell_aspect(console_cell_aspect);
            console_config.panel_cols = console_panel_cols;
            if let Some((cols, rows)) = console_grid {
                console_config.grid_cols = Some(cols);
                console_config.grid_rows = Some(rows);
            }
            let options = cli::view::ViewOptions {
                bogae,
                bogae_codec,
                bogae_out,
                bogae_skin,
                overlay,
                console_config,
                no_open,
            };
            if let Err(err) = cli::view::run_view(&file, options) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        Commands::Check { file } => {
            let args = cli::check::CheckArgs { emit_schema: true };
            if let Err(err) = cli::check::run(&file, args) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        Commands::Test {
            file,
            threads,
            out,
            smoke,
            golden,
            pack,
            all,
            record,
            update,
            skip_ui_common,
            skip_wrapper,
        } => {
            if smoke && golden {
                eprintln!("E_TEST_MODE_CONFLICT --smoke 와 --golden 은 동시에 사용할 수 없습니다.");
                std::process::exit(1);
            }
            if smoke || golden {
                if file.is_some() {
                    eprintln!("E_TEST_FILE_CONFLICT --smoke/--golden 모드에서는 file 위치 인자를 사용하지 않습니다.");
                    std::process::exit(1);
                }
                if smoke {
                    if all || record {
                        eprintln!("E_TEST_SMOKE_OPTION --smoke 모드에서는 --all/--record 를 사용할 수 없습니다.");
                        std::process::exit(1);
                    }
                    let options = cli::test::SmokeRunnerOptions {
                        packs: pack,
                        update,
                        skip_ui_common,
                        skip_wrapper,
                    };
                    if let Err(err) = cli::test::run_wasm_smoke_runner(options) {
                        eprintln!("{}", err);
                        std::process::exit(1);
                    }
                } else {
                    if skip_ui_common || skip_wrapper {
                        eprintln!("E_TEST_GOLDEN_OPTION --golden 모드에서는 --skip-ui-common/--skip-wrapper 를 사용할 수 없습니다.");
                        std::process::exit(1);
                    }
                    let options = cli::test::GoldenRunnerOptions {
                        packs: pack,
                        all,
                        record,
                        update,
                    };
                    if let Err(err) = cli::test::run_pack_golden_runner(options) {
                        eprintln!("{}", err);
                        std::process::exit(1);
                    }
                }
            } else {
                let Some(file) = file else {
                    eprintln!(
                        "E_TEST_FILE_REQUIRED 기본 test 모드에서는 file 위치 인자가 필요합니다."
                    );
                    std::process::exit(1);
                };
                if all || record || update || skip_ui_common || skip_wrapper || !pack.is_empty() {
                    eprintln!("E_TEST_OPTION_INVALID 기본 test 모드에서는 --all/--record/--update/--pack/--skip-* 옵션을 사용할 수 없습니다.");
                    std::process::exit(1);
                }
                if let Err(err) = cli::test::run_realms_test(&file, threads, out.as_deref()) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        }
        Commands::Warp { command } => match command {
            WarpCommands::Bench {
                file,
                backend,
                policy,
                threads,
                measure,
                out,
            } => {
                if let Err(err) = cli::warp::run_bench(
                    &file,
                    backend.to_core(),
                    policy.to_core(),
                    threads,
                    measure,
                    out.as_deref(),
                ) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Build { file } => {
            let args = cli::check::CheckArgs { emit_schema: true };
            if let Err(err) = cli::check::run(&file, args) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        Commands::Lint {
            file,
            suggest_patch,
            out,
        } => {
            if let Err(err) = cli::lint::run(&file, suggest_patch, out.as_deref()) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        Commands::Repl => {
            if let Err(err) = cli::repl::repl() {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        Commands::Worker => {
            if let Err(err) = cli::worker::run() {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        Commands::Canon {
            file,
            emit,
            out,
            fixits_json,
            diag_jsonl,
            meta_out,
            bridge,
            check,
        } => {
            let args = cli::canon::CanonArgs {
                emit,
                out_dir: out,
                bridge,
                fixits_json,
                diag_jsonl,
                meta_out,
                check,
            };
            if let Err(err) = cli::canon::run(&file, args) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        Commands::Asset { command } => match command {
            AssetCommands::Manifest { root, out } => {
                if let Err(err) = cli::asset::run_manifest(&root, out.as_deref()) {
                    eprintln!("E_ASSET_MANIFEST {}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Bogae { command } => match command {
            BogaeCommands::Edit {
                input,
                out,
                dx,
                dy,
                color,
            } => {
                let options = cli::bogae_edit::BogaeEditOptions {
                    input: &input,
                    output: &out,
                    dx,
                    dy,
                    color: color.as_deref(),
                };
                if let Err(err) = cli::bogae_edit::run_edit(options) {
                    eprintln!("E_BOGAE_EDIT {}", err);
                    std::process::exit(1);
                }
            }
            BogaeCommands::Bundle {
                out,
                mapping,
                scene,
                assets,
            } => {
                let options = cli::bogae_bundle::BogaeBundleOptions {
                    out_dir: &out,
                    mapping: mapping.as_deref(),
                    scene: scene.as_deref(),
                    assets_dir: assets.as_deref(),
                };
                if let Err(err) = cli::bogae_bundle::run_bundle(options) {
                    eprintln!("E_BOGAE_BUNDLE {}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Dotbogi { command } => match command {
            DotbogiCommands::Case {
                input,
                out,
                after_state_out,
                report_out,
            } => {
                let options = cli::dotbogi::DotbogiCaseOptions {
                    input: &input,
                    out: out.as_deref(),
                    after_state_out: after_state_out.as_deref(),
                    report_out: report_out.as_deref(),
                };
                if let Err(err) = cli::dotbogi::run_case(options) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Eco { command } => match command {
            EcoCommands::MacroMicro { input, out } => {
                if let Err(err) = cli::eco::run_macro_micro(&input, out.as_deref()) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            EcoCommands::NetworkFlow {
                input,
                madi,
                seed,
                threshold,
                out,
            } => {
                let parsed_seed = match parse_seed(&seed) {
                    Ok(value) => value,
                    Err(err) => {
                        eprintln!("E_ECO_NETWORK_FLOW_SEED {}", err);
                        std::process::exit(1);
                    }
                };
                let parsed_threshold =
                    match crate::core::fixed64::Fixed64::parse_literal(&threshold) {
                        Some(value) => value,
                        None => {
                            eprintln!(
                                "E_ECO_NETWORK_FLOW_THRESHOLD threshold 파싱 실패: {}",
                                threshold
                            );
                            std::process::exit(1);
                        }
                    };
                if let Err(err) = cli::eco::run_network_flow(
                    &input,
                    madi,
                    parsed_seed,
                    parsed_threshold,
                    out.as_deref(),
                ) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            EcoCommands::AbmSpatial {
                input,
                madi,
                seed,
                out,
            } => {
                let parsed_seed = match parse_seed(&seed) {
                    Ok(value) => value,
                    Err(err) => {
                        eprintln!("E_ECO_ABM_SPATIAL_SEED {}", err);
                        std::process::exit(1);
                    }
                };
                if let Err(err) =
                    cli::eco::run_abm_spatial(&input, madi, parsed_seed, out.as_deref())
                {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Ai { command } => match command {
            AiCommands::Extract { file, out } => {
                if let Err(err) = cli::ai::extract(&file, &out) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            AiCommands::Prompt {
                profile,
                out,
                bundle,
            } => {
                let args = ai_prompt::AiPromptArgs {
                    profile,
                    out_path: out,
                    bundle_path: bundle,
                };
                if let Err(err) = ai_prompt::run_ai_prompt(args) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Bdl { command } => match command {
            BdlCommands::Packet { command } => match command {
                BdlPacketCommands::Wrap { input, out } => {
                    if let Err(err) = cli::bdl_packet::wrap_packet(&input, &out) {
                        eprintln!("{}", err);
                        std::process::exit(1);
                    }
                }
                BdlPacketCommands::Unwrap { input, out } => {
                    if let Err(err) = cli::bdl_packet::unwrap_packet(&input, &out) {
                        eprintln!("{}", err);
                        std::process::exit(1);
                    }
                }
            },
        },
        Commands::Replay { command } => match command {
            ReplayCommands::Diff {
                a,
                b,
                out,
                no_summary,
            } => {
                let options = cli::replay_diff::ReplayDiffOptions {
                    a,
                    b,
                    out,
                    write_summary: !no_summary,
                };
                if let Err(err) = cli::replay_diff::run_diff(options) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            ReplayCommands::Verify {
                geoul,
                until,
                seek,
                entry,
            } => {
                if let Err(err) =
                    cli::replay::run_replay_verify(&geoul, entry.as_deref(), until, seek)
                {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            ReplayCommands::Branch {
                geoul,
                at,
                inject_sam,
                out,
                entry,
            } => {
                if let Err(err) = cli::replay_branch::run_replay_branch(
                    &geoul,
                    at,
                    &inject_sam,
                    &out,
                    entry.as_deref(),
                ) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Geoul { command } => match command {
            GeoulCommands::Hash { geoul } => {
                if let Err(err) = cli::geoul::run_geoul_hash(&geoul) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            GeoulCommands::Seek { geoul, madi } => {
                if let Err(err) = cli::geoul::run_geoul_seek(&geoul, madi) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            GeoulCommands::Query {
                geoul,
                madi,
                key,
                entry,
            } => {
                if let Err(err) = cli::geoul::run_geoul_query(&geoul, madi, &key, entry.as_deref())
                {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            GeoulCommands::Backtrace {
                geoul,
                key,
                from,
                to,
                entry,
            } => {
                if let Err(err) =
                    cli::geoul::run_geoul_backtrace(&geoul, &key, from, to, entry.as_deref())
                {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            GeoulCommands::Record { command } => match command {
                GeoulRecordCommands::Make { input, out } => {
                    if let Err(err) = cli::geoul::run_geoul_record_make(&input, out.as_deref()) {
                        eprintln!("{}", err);
                        std::process::exit(1);
                    }
                }
                GeoulRecordCommands::Check { input } => {
                    if let Err(err) = cli::geoul::run_geoul_record_check(&input) {
                        eprintln!("{}", err);
                        std::process::exit(1);
                    }
                }
            },
        },
        Commands::Patch { command } => match command {
            PatchCommands::Propose { file, out } => {
                if let Err(err) = cli::patch::run_propose(&file, out.as_deref()) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            PatchCommands::Preview { patch, format } => {
                if let Err(err) = cli::patch::run_preview(&patch, format) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            PatchCommands::Approve {
                patch,
                out,
                yes,
                notes,
            } => {
                let out =
                    out.unwrap_or_else(|| cli::paths::build_dir().join("ddn.patch.approval.json"));
                if let Err(err) = cli::patch::run_approve(&patch, &out, yes, notes) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            PatchCommands::Apply {
                patch,
                approval,
                out,
                in_place,
            } => {
                if let Err(err) = cli::patch::run_apply(&patch, &approval, out.as_deref(), in_place)
                {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            PatchCommands::Verify {
                patch,
                approval,
                tests,
                walk,
            } => {
                if let Err(err) =
                    cli::patch::run_verify(&patch, &approval, tests.as_deref(), walk.as_deref())
                {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Scan { root } => {
            let root = root.unwrap_or_else(|| PathBuf::from("."));
            if let Err(err) = cli::scan::run(&root) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        Commands::Gaji { command } => {
            if let Err(err) = run_gaji_command(command) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        Commands::Intent { command } => match command {
            IntentCommands::Inspect {
                geoul,
                madi,
                agent,
                out,
            } => {
                if let Err(err) = cli::intent::run_inspect(&geoul, madi, agent, out.as_deref()) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            IntentCommands::Mock {
                input,
                out,
                agent_id,
                madi,
                recv_seq,
            } => {
                if let Err(err) = cli::intent::run_mock(&input, &out, agent_id, madi, recv_seq) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            IntentCommands::Merge {
                inputs,
                madi,
                agent,
                out,
            } => {
                if inputs.is_empty() {
                    eprintln!("E_INTENT_MERGE inputs가 비었습니다");
                    std::process::exit(1);
                }
                if let Err(err) = cli::intent::run_merge(&inputs, madi, agent, out.as_deref()) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Goal { command } => match command {
            GoalCommands::Parse { input, out } => {
                if let Err(err) = cli::goal::run_parse(&input, out.as_deref()) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            GoalCommands::Plan { actions, out } => {
                if let Err(err) = cli::goal::run_plan(&actions, out.as_deref()) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Goap { command } => match command {
            GoapCommands::Plan { input, out } => {
                if let Err(err) = cli::goap::run_plan(&input, out.as_deref()) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Observation { command } => match command {
            ObservationCommands::Canon { input, out } => {
                if let Err(err) = cli::observation::run_canon(&input, out.as_deref()) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::NuriGym { command } => match command {
            NuriGymCommands::Spec { from, out, slots } => {
                let out = out.unwrap_or_else(|| cli::paths::build_dir().join("nurigym"));
                if let Err(err) = cli::nurigym::run_spec(&from, &out, slots) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            NuriGymCommands::View { spec } => {
                if let Err(err) = cli::nurigym::run_view(&spec) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            NuriGymCommands::Run { input, out } => {
                let out = out.unwrap_or_else(|| cli::paths::build_dir().join("nurigym"));
                if let Err(err) = cli::nurigym::run_episode_file(&input, &out) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Gateway { command } => match command {
            GatewayCommands::Serve {
                world,
                threads,
                input,
                out,
                realms,
                input_format,
                listen,
                listen_proto,
                listen_max_events,
                listen_timeout_ms,
                send,
                send_format,
            } => {
                let options = cli::gateway::ServeOptions {
                    world,
                    threads,
                    input,
                    out,
                    realms,
                    input_format: input_format.to_core(),
                    listen_addr: listen,
                    listen_proto: listen_proto.to_core(),
                    listen_max_events,
                    listen_timeout_ms,
                    send_path: send,
                    send_format: send_format.to_core(),
                };
                if let Err(err) = cli::gateway::run_serve(options) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            GatewayCommands::LoadSim {
                clients,
                ticks,
                seed,
                realms,
                tick_hz,
                threads,
                out,
            } => {
                let options = cli::gateway::LoadSimOptions {
                    clients,
                    ticks,
                    seed,
                    realms,
                    tick_hz,
                    threads,
                    out,
                };
                if let Err(err) = cli::gateway::run_load_sim(options) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Latency { command } => match command {
            LatencyCommands::Simulate {
                l_madi,
                mode,
                count,
                seed,
                out,
            } => {
                if let Err(err) =
                    cli::latency::run_simulate(l_madi, mode.to_core(), count, seed, out.as_deref())
                {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Safety { command } => match command {
            SafetyCommands::Check { rules, intent, out } => {
                if let Err(err) = cli::safety::run_check(&rules, &intent, out.as_deref()) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Dataset { command } => match command {
            DatasetCommands::Export {
                geoul,
                format,
                out,
                env_id,
            } => {
                if let Err(err) = cli::dataset::run_export(&geoul, &format, &out, &env_id) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Doc { command } => match command {
            DocCommands::Build { out } => {
                let options = cli::docset::DocBuildOptions { out };
                if let Err(err) = cli::docset::run_build(options) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            DocCommands::Verify { pack, out } => {
                let options = cli::docset::DocVerifyOptions { pack, out };
                if let Err(err) = cli::docset::run_verify(options) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Reward { command } => match command {
            RewardCommands::Check { input, out } => {
                if let Err(err) = cli::reward::run_reward_check(&input, out.as_deref()) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Train { config, out } => {
            if let Err(err) = cli::train::run_train(&config, out.as_deref()) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        Commands::Imitation { config, out } => {
            if let Err(err) = cli::imitation::run_imitation(&config, out.as_deref()) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        Commands::Eval { config, out } => {
            if let Err(err) = cli::eval::run_eval(&config, out.as_deref()) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        Commands::Bundle { command } => match command {
            BundleCommands::Parity {
                bundle_in,
                inputs,
                out,
                wasm_hash,
            } => {
                if let Err(err) = cli::seulgi_bundle::run_parity(
                    &bundle_in,
                    &inputs,
                    out.as_deref(),
                    wasm_hash.as_deref(),
                ) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Edu { command } => match command {
            EduCommands::Accuracy { input, out } => {
                if let Err(err) = cli::edu::run_accuracy(&input, out.as_deref()) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Swarm { command } => match command {
            SwarmCommands::Collision { input, out } => {
                if let Err(err) = cli::swarm::run_collision(&input, out.as_deref()) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Infer { command } => match command {
            InferCommands::Mlp { model, input, out } => {
                if let Err(err) = cli::infer::run_mlp(&model, &input, out.as_deref()) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Alrim { command } => match command {
            AlrimCommands::Registry { root, out } => {
                let out = out.unwrap_or_else(|| cli::paths::build_dir().join("registry"));
                if let Err(err) = cli::alrim::run_registry(&root, &out) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Story { command } => match command {
            StoryCommands::Make { geoul, out } => {
                if let Err(err) = cli::story::run_make(&geoul, &out) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Timeline { command } => match command {
            TimelineCommands::Make { geoul, story, out } => {
                if let Err(err) = cli::timeline::run_make(&geoul, &story, &out) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        Commands::Workshop { command } => match command {
            WorkshopCommands::Gen { geoul, out } => {
                if let Err(err) = cli::workshop::run_gen(&geoul, &out) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            WorkshopCommands::Apply { workshop, patch } => {
                if let Err(err) = cli::workshop::run_apply(&workshop, &patch) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            WorkshopCommands::Open { workshop } => {
                if let Err(err) = cli::workshop::run_open(&workshop) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
    }
}

fn parse_seed(input: &str) -> Result<u64, String> {
    let trimmed = input.trim();
    if let Some(hex) = trimmed.strip_prefix("0x") {
        u64::from_str_radix(hex, 16).map_err(|e| e.to_string())
    } else {
        trimmed.parse::<u64>().map_err(|e| e.to_string())
    }
}

fn run_gaji_command(command: GajiCommands) -> Result<(), String> {
    match command {
        GajiCommands::Lock {
            root,
            out,
            registry_index,
            snapshot_id,
            index_root_hash,
            trust_root_hash,
            trust_root_source,
            audit_last_hash,
        } => {
            let root = root.unwrap_or_else(|| PathBuf::from("."));
            let out = out.unwrap_or_else(|| root.join("ddn.lock"));
            let mut options = cli::gaji::LockWriteOptions {
                snapshot_id,
                index_root_hash,
                trust_root_hash,
                trust_root_source,
                audit_last_hash,
            };
            if let Some(index) = registry_index.as_deref() {
                cli::gaji::apply_registry_meta_from_index(&mut options, index)?;
            }
            cli::gaji::run_lock_with_options(&root, &out, &options)
        }
        GajiCommands::Install {
            root,
            lock,
            out,
            registry_index,
            verify_registry,
            strict_registry,
            registry_verify_out,
            registry_audit_log,
            verify_registry_audit,
            registry_audit_verify_out,
            expect_audit_last_hash,
            frozen_lockfile,
            expect_snapshot_id,
            expect_index_root_hash,
            expect_trust_root_hash,
            require_trust_root,
            deny_yanked_locked,
        } => {
            let root = root.unwrap_or_else(|| PathBuf::from("."));
            let lock = lock.unwrap_or_else(|| root.join("ddn.lock"));
            let out = out.unwrap_or_else(|| root.join("vendor").join("gaji"));
            let frozen = cli::gaji::FrozenLockOptions {
                frozen_lockfile,
                expect_snapshot_id,
                expect_index_root_hash,
                expect_trust_root_hash,
                require_trust_root,
                deny_yanked_locked,
                registry_index,
                verify_registry,
                registry_verify_out,
                registry_audit_log,
                verify_registry_audit,
                registry_audit_verify_out,
                expect_audit_last_hash,
                strict_registry,
            };
            cli::gaji::run_install_with_options(&root, &lock, &out, &frozen)
        }
        GajiCommands::Update {
            root,
            lock,
            out,
            registry_index,
            verify_registry,
            strict_registry,
            registry_verify_out,
            registry_audit_log,
            verify_registry_audit,
            registry_audit_verify_out,
            expect_audit_last_hash,
            snapshot_id,
            index_root_hash,
            trust_root_hash,
            trust_root_source,
            audit_last_hash,
            frozen_lockfile,
            expect_snapshot_id,
            expect_index_root_hash,
            expect_trust_root_hash,
            require_trust_root,
            deny_yanked_locked,
        } => {
            let root = root.unwrap_or_else(|| PathBuf::from("."));
            let lock = lock.unwrap_or_else(|| root.join("ddn.lock"));
            let out = out.unwrap_or_else(|| root.join("vendor").join("gaji"));
            let lock_options = cli::gaji::LockWriteOptions {
                snapshot_id,
                index_root_hash,
                trust_root_hash,
                trust_root_source,
                audit_last_hash,
            };
            let frozen = cli::gaji::FrozenLockOptions {
                frozen_lockfile,
                expect_snapshot_id,
                expect_index_root_hash,
                expect_trust_root_hash,
                require_trust_root,
                deny_yanked_locked,
                registry_index,
                verify_registry,
                registry_verify_out,
                registry_audit_log,
                verify_registry_audit,
                registry_audit_verify_out,
                expect_audit_last_hash,
                strict_registry,
            };
            cli::gaji::run_update_with_options(&root, &lock, &out, &lock_options, &frozen)
        }
        GajiCommands::Vendor {
            root,
            lock,
            out,
            registry_index,
            verify_registry,
            strict_registry,
            registry_verify_out,
            registry_audit_log,
            verify_registry_audit,
            registry_audit_verify_out,
            expect_audit_last_hash,
            frozen_lockfile,
            expect_snapshot_id,
            expect_index_root_hash,
            expect_trust_root_hash,
            require_trust_root,
            deny_yanked_locked,
        } => {
            let root = root.unwrap_or_else(|| PathBuf::from("."));
            let lock = lock.unwrap_or_else(|| root.join("ddn.lock"));
            let out = out.unwrap_or_else(|| root.join("vendor").join("gaji"));
            let frozen = cli::gaji::FrozenLockOptions {
                frozen_lockfile,
                expect_snapshot_id,
                expect_index_root_hash,
                expect_trust_root_hash,
                require_trust_root,
                deny_yanked_locked,
                registry_index,
                verify_registry,
                registry_verify_out,
                registry_audit_log,
                verify_registry_audit,
                registry_audit_verify_out,
                expect_audit_last_hash,
                strict_registry,
            };
            cli::gaji::run_vendor_with_options(&root, &lock, &out, &frozen)
        }
        GajiCommands::Registry { args } => cli::gaji_registry::run_cli(&args),
    }
}

fn parse_madi_arg(input: Option<&str>) -> Result<Option<cli::run::MadiLimit>, String> {
    let Some(raw) = input else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.eq_ignore_ascii_case("infinite") {
        return Ok(Some(cli::run::MadiLimit::Infinite));
    }
    let value = trimmed
        .parse::<u64>()
        .map_err(|_| "madi는 정수 또는 infinite 여야 합니다.".to_string())?;
    Ok(Some(cli::run::MadiLimit::Finite(value)))
}

fn parse_console_grid(input: Option<&str>) -> Result<Option<(usize, usize)>, String> {
    let Some(raw) = input else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let normalized = trimmed.to_ascii_lowercase();
    let (cols, rows) = normalized
        .split_once('x')
        .ok_or_else(|| "grid must be formatted as COLSxROWS (ex: 12x24)".to_string())?;
    let cols = cols
        .trim()
        .parse::<usize>()
        .map_err(|_| "grid cols must be a positive integer".to_string())?;
    let rows = rows
        .trim()
        .parse::<usize>()
        .map_err(|_| "grid rows must be a positive integer".to_string())?;
    if cols == 0 || rows == 0 {
        return Err("grid cols/rows must be greater than 0".to_string());
    }
    Ok(Some((cols, rows)))
}

fn build_command_string() -> String {
    let parts = env::args().collect::<Vec<_>>();
    build_command_string_from_parts(&parts)
}

pub(crate) fn build_command_string_from_parts(parts: &[String]) -> String {
    parts
        .iter()
        .map(|arg| shell_quote_arg(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote_arg(arg: &str) -> String {
    if arg.chars().any(|ch| ch.is_whitespace()) {
        let escaped = arg.replace('"', "\\\"");
        format!("\"{}\"", escaped)
    } else {
        arg.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::{Path, PathBuf};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ddn_main_{}_{}", name, stamp));
        fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    fn write_demo_pkg(root: &Path, id: &str, version: &str) {
        let pkg = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg).expect("pkg mkdir");
        fs::write(
            pkg.join("gaji.toml"),
            format!("id = \"{}\"\nversion = \"{}\"\n", id, version),
        )
        .expect("write toml");
        fs::write(pkg.join("main.ddn"), "값 <- 1.\n").expect("write src");
    }

    fn sha256_hex_prefixed(bytes: &[u8]) -> String {
        let digest = Sha256::digest(bytes);
        format!("sha256:{}", hex::encode(digest))
    }

    fn start_http_fixture(bytes: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind http fixture");
        let addr = listener.local_addr().expect("local addr");
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                bytes.len()
            );
            stream.write_all(headers.as_bytes()).expect("write headers");
            stream.write_all(bytes).expect("write body");
            stream.flush().expect("flush");
        });
        format!("http://{}/archive.ddn.tar.gz", addr)
    }

    fn write_registry_index_with_entry(
        path: &Path,
        snapshot_id: &str,
        index_root_hash: &str,
        scope: &str,
        name: &str,
        version: &str,
    ) {
        let root = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": snapshot_id,
            "index_root_hash": index_root_hash,
            "trust_root": {
                "hash": "sha256:trust",
                "source": "registry"
            },
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": scope,
                "name": name,
                "version": version,
                "yanked": false
            }]
        });
        fs::write(
            path,
            serde_json::to_string_pretty(&root).expect("index json"),
        )
        .expect("write");
    }

    fn merge_object_fields(dst: &mut Value, patch: &Value) {
        let dst_map = dst.as_object_mut().expect("dst object");
        let patch_map = patch.as_object().expect("patch object");
        for (k, v) in patch_map {
            dst_map.insert(k.clone(), v.clone());
        }
    }

    fn write_main_verify_fixture(
        index: &Path,
        lock: &Path,
        snapshot_id: &str,
        index_root_hash: &str,
        index_entry_patch: Value,
        lock_package_patch: Value,
    ) {
        let mut entry = json!({
            "schema": "ddn.registry.index_entry.v1",
            "scope": "표준",
            "name": "역학",
            "version": "20.6.30",
            "yanked": false
        });
        merge_object_fields(&mut entry, &index_entry_patch);
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": snapshot_id,
            "index_root_hash": index_root_hash,
            "entries": [entry]
        });
        fs::write(
            index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let mut package = json!({
            "id": "표준/역학",
            "version": "20.6.30",
            "path": "x",
            "hash": "blake3:x",
            "yanked": false
        });
        merge_object_fields(&mut package, &lock_package_patch);
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": snapshot_id,
                "index_root_hash": index_root_hash
            },
            "packages": [package]
        });
        fs::write(
            lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");
    }

    fn main_registry_verify_args(index: &Path, lock: &Path) -> Vec<String> {
        vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ]
    }

    fn write_audit_log_and_last_hash(audit_log: &Path) -> String {
        let index = audit_log.with_file_name("audit_index.json");
        cli::gaji_registry::run_publish(
            &index,
            &cli::gaji_registry::PublishOptions {
                audit_log: Some(audit_log.to_string_lossy().to_string()),
                scope: "표준".to_string(),
                name: "역학".to_string(),
                version: "0.1.0".to_string(),
                archive_sha256: "sha256:a".to_string(),
                contract: None,
                detmath_seal_hash: None,
                min_runtime: None,
                download_url: None,
                summary: None,
                token: "token1".to_string(),
                role: "publisher".to_string(),
                at: Some("2026-02-20T00:00:00Z".to_string()),
            },
        )
        .expect("publish for audit log");
        let report = cli::gaji_registry::run_audit_verify(audit_log).expect("audit verify");
        report.last_hash().expect("last hash").to_string()
    }

    fn run_parsed_gaji(args: &[String]) -> Result<(), String> {
        let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;
        match cli.command {
            Commands::Gaji { command } => run_gaji_command(command),
            _ => Err("expected gaji command".to_string()),
        }
    }

    fn run_parsed_dotbogi(args: &[String]) -> Result<(), String> {
        let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;
        match cli.command {
            Commands::Dotbogi { command } => match command {
                DotbogiCommands::Case {
                    input,
                    out,
                    after_state_out,
                    report_out,
                } => {
                    let options = cli::dotbogi::DotbogiCaseOptions {
                        input: &input,
                        out: out.as_deref(),
                        after_state_out: after_state_out.as_deref(),
                        report_out: report_out.as_deref(),
                    };
                    cli::dotbogi::run_case(options)
                }
            },
            _ => Err("expected dotbogi command".to_string()),
        }
    }

    fn run_parsed_eco(args: &[String]) -> Result<(), String> {
        let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;
        match cli.command {
            Commands::Eco { command } => match command {
                EcoCommands::MacroMicro { input, out } => {
                    cli::eco::run_macro_micro(&input, out.as_deref())
                }
                EcoCommands::NetworkFlow {
                    input,
                    madi,
                    seed,
                    threshold,
                    out,
                } => {
                    let parsed_seed = parse_seed(&seed)
                        .map_err(|err| format!("E_ECO_NETWORK_FLOW_SEED {}", err))?;
                    let parsed_threshold = crate::core::fixed64::Fixed64::parse_literal(&threshold)
                        .ok_or_else(|| {
                            format!(
                                "E_ECO_NETWORK_FLOW_THRESHOLD threshold 파싱 실패: {}",
                                threshold
                            )
                        })?;
                    cli::eco::run_network_flow(
                        &input,
                        madi,
                        parsed_seed,
                        parsed_threshold,
                        out.as_deref(),
                    )
                }
                EcoCommands::AbmSpatial {
                    input,
                    madi,
                    seed,
                    out,
                } => {
                    let parsed_seed = parse_seed(&seed)
                        .map_err(|err| format!("E_ECO_ABM_SPATIAL_SEED {}", err))?;
                    cli::eco::run_abm_spatial(&input, madi, parsed_seed, out.as_deref())
                }
            },
            _ => Err("expected eco command".to_string()),
        }
    }

    fn read_audit_rows(path: &Path) -> Vec<Value> {
        let text = fs::read_to_string(path).expect("read audit rows");
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect("parse audit row"))
            .collect()
    }

    fn assert_verify_report_contract(report: &Value, packages: u64, matched: u64) {
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.verify_report.v1")
        );
        assert_eq!(report.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            report.get("packages").and_then(|v| v.as_u64()),
            Some(packages)
        );
        assert_eq!(
            report.get("matched").and_then(|v| v.as_u64()),
            Some(matched)
        );
        assert_eq!(
            report
                .get("duplicate_resolution_policy")
                .and_then(|v| v.as_str()),
            Some(crate::cli::gaji_registry::VERIFY_DUPLICATE_RESOLUTION_POLICY)
        );
    }

    fn assert_audit_verify_report_contract(report: &Value, rows: u64) {
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.audit_verify_report.v1")
        );
        assert_eq!(report.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(report.get("rows").and_then(|v| v.as_u64()), Some(rows));
        assert!(report
            .get("last_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .starts_with("blake3:"));
    }

    fn assert_diag_with_fix(err: &str, code: &str) {
        assert!(err.contains(code), "missing code {code} in {err}");
        assert!(err.contains("fix="), "missing fix= in {err}");
    }

    fn assert_audit_last_hash_diag(err: &str) {
        assert_diag_with_fix(err, "E_REG_AUDIT_LAST_HASH_MISMATCH");
        assert!(err.contains("hint="), "missing hint= in {err}");
    }

    #[test]
    fn dotbogi_case_main_cli_runs_and_writes_outputs() {
        let root = temp_dir("main_dotbogi_case_ok");
        let input = root.join("case.detjson");
        let out = root.join("dotbogi.output.detjson");
        let after_state_out = root.join("after_state.detjson");
        let report_out = root.join("dotbogi.report.detjson");
        let doc = json!({
            "schema": "ddn.dotbogi.case.v1",
            "input": {
                "schema": "dotbogi.input.v1",
                "state": {
                    "player": {
                        "hp": 10,
                        "potion": 1
                    }
                }
            },
            "dotbogi": {
                "view_meta": {
                    "inventory": ["potion"]
                },
                "events": [{
                    "type": "아이템사용",
                    "id": "potion"
                }]
            },
            "roundtrip": {
                "event_rules": [{
                    "event_type": "아이템사용",
                    "ops": [
                        {"op": "add", "path": "player.hp", "value": 5},
                        {"op": "add", "path": "player.potion", "value": -1}
                    ]
                }]
            }
        });
        fs::write(&input, serde_json::to_string_pretty(&doc).expect("json")).expect("write input");

        let args = vec![
            "teul-cli".to_string(),
            "dotbogi".to_string(),
            "case".to_string(),
            input.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--after-state-out".to_string(),
            after_state_out.to_string_lossy().to_string(),
            "--report-out".to_string(),
            report_out.to_string_lossy().to_string(),
        ];
        run_parsed_dotbogi(&args).expect("dotbogi case should pass");

        assert!(out.exists());
        assert!(after_state_out.exists());
        assert!(report_out.exists());

        let report: Value =
            serde_json::from_str(&fs::read_to_string(&report_out).expect("read report"))
                .expect("parse report");
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.dotbogi.case.report.v1")
        );
        assert_eq!(
            report
                .get("after_state")
                .and_then(|v| v.get("player"))
                .and_then(|v| v.get("hp"))
                .and_then(|v| v.as_i64()),
            Some(15)
        );
    }

    #[test]
    fn dotbogi_case_main_cli_write_forbidden_returns_error() {
        let root = temp_dir("main_dotbogi_case_forbidden");
        let input = root.join("case.detjson");
        let doc = json!({
            "schema": "ddn.dotbogi.case.v1",
            "input": {
                "schema": "dotbogi.input.v1",
                "state": {"hp": 1}
            },
            "dotbogi": {
                "view_meta": {},
                "events": [],
                "state_write": {"hp": 999}
            }
        });
        fs::write(&input, serde_json::to_string_pretty(&doc).expect("json")).expect("write input");
        let args = vec![
            "teul-cli".to_string(),
            "dotbogi".to_string(),
            "case".to_string(),
            input.to_string_lossy().to_string(),
        ];
        let err = run_parsed_dotbogi(&args).expect_err("write forbidden must fail");
        assert!(err.contains("E_DOTBOGI_STATE_WRITE_FORBIDDEN"));
    }

    #[test]
    fn eco_macro_micro_main_cli_runs_and_writes_report() {
        let root = temp_dir("main_eco_macro_micro_ok");
        let macro_model = root.join("macro.ddn");
        let micro_model = root.join("micro.ddn");
        let runner = root.join("runner.json");
        let report_out = root.join("runner.report.detjson");
        fs::write(&macro_model, "(매마디)마다 { 값 <- 1. }.\n").expect("write macro");
        fs::write(&micro_model, "(매마디)마다 { 값 <- 2. }.\n").expect("write micro");
        let spec = serde_json::json!({
            "schema": "ddn.macro_micro_runner.v0",
            "seed": 42,
            "ticks": 3,
            "models": {
                "거시": macro_model.to_string_lossy(),
                "미시": micro_model.to_string_lossy(),
            },
            "diagnostics": [
                { "name": "거시↔미시 값", "lhs": "거시.값", "rhs": "미시.값", "threshold": 0.1 }
            ]
        });
        fs::write(
            &runner,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&spec).expect("runner json")
            ),
        )
        .expect("write runner");

        run_parsed_eco(&[
            "teul-cli".to_string(),
            "eco".to_string(),
            "macro-micro".to_string(),
            runner.to_string_lossy().to_string(),
            "--out".to_string(),
            report_out.to_string_lossy().to_string(),
        ])
        .expect("eco run");

        let report: Value =
            serde_json::from_str(&fs::read_to_string(&report_out).expect("read report"))
                .expect("parse report");
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.runner_report.v0")
        );
        let rows = report
            .get("results")
            .and_then(|v| v.as_array())
            .expect("results");
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].get("divergence_tick").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            rows[0].get("error_code").and_then(|v| v.as_str()),
            Some("E_ECO_DIVERGENCE_DETECTED")
        );
    }

    #[test]
    fn eco_macro_micro_main_cli_rejects_invalid_scope() {
        let root = temp_dir("main_eco_macro_micro_invalid_scope");
        let macro_model = root.join("macro.ddn");
        let micro_model = root.join("micro.ddn");
        let runner = root.join("runner.json");
        let report_out = root.join("runner.report.detjson");
        fs::write(&macro_model, "(매마디)마다 { 값 <- 1. }.\n").expect("write macro");
        fs::write(&micro_model, "(매마디)마다 { 값 <- 1. }.\n").expect("write micro");
        let spec = serde_json::json!({
            "schema": "ddn.macro_micro_runner.v0",
            "seed": 42,
            "ticks": 3,
            "shock": {
                "type": "세율_인상",
                "target": "세율",
                "delta": 0.1,
                "at_tick": 2,
                "scope": "invalid_scope"
            },
            "models": {
                "거시": macro_model.to_string_lossy(),
                "미시": micro_model.to_string_lossy(),
            },
            "diagnostics": [
                { "name": "거시↔미시 값", "lhs": "거시.값", "rhs": "미시.값", "threshold": 0.1 }
            ]
        });
        fs::write(
            &runner,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&spec).expect("runner json")
            ),
        )
        .expect("write runner");

        let err = run_parsed_eco(&[
            "teul-cli".to_string(),
            "eco".to_string(),
            "macro-micro".to_string(),
            runner.to_string_lossy().to_string(),
            "--out".to_string(),
            report_out.to_string_lossy().to_string(),
        ])
        .expect_err("invalid scope must fail");
        assert!(err.contains("E_ECO_RUNNER_SHOCK"));
    }

    #[test]
    fn eco_macro_micro_main_cli_accepts_macro_and_micro_scope() {
        let root = temp_dir("main_eco_macro_micro_scope_ok");
        let macro_model = root.join("macro.ddn");
        let micro_model = root.join("micro.ddn");
        fs::write(
            &macro_model,
            "(시작)할때 { 세율 <- 0. }.\n(매마디)마다 { 균형가격 <- (100 + 세율 * 50). }.\n",
        )
        .expect("write macro");
        fs::write(
            &micro_model,
            "(시작)할때 { 세율 <- 0. }.\n(매마디)마다 { 평균가격 <- (100 + 세율 * 200). }.\n",
        )
        .expect("write micro");

        for (scope, expected_scope) in [("거시", "거시"), ("미시", "미시")] {
            let runner = root.join(format!("runner_{}.json", expected_scope));
            let report_out = root.join(format!("report_{}.detjson", expected_scope));
            let spec = serde_json::json!({
                "schema": "ddn.macro_micro_runner.v0",
                "seed": 42,
                "ticks": 3,
                "shock": {
                    "type": "세율_인상",
                    "target": "세율",
                    "delta": 0.1,
                    "at_tick": 2,
                    "scope": scope
                },
                "models": {
                    "거시": macro_model.to_string_lossy(),
                    "미시": micro_model.to_string_lossy(),
                },
                "diagnostics": [
                    { "name": "거시↔미시 값", "lhs": "거시.균형가격", "rhs": "미시.평균가격", "threshold": 0.1 }
                ]
            });
            fs::write(
                &runner,
                format!(
                    "{}\n",
                    serde_json::to_string_pretty(&spec).expect("runner json")
                ),
            )
            .expect("write runner");

            run_parsed_eco(&[
                "teul-cli".to_string(),
                "eco".to_string(),
                "macro-micro".to_string(),
                runner.to_string_lossy().to_string(),
                "--out".to_string(),
                report_out.to_string_lossy().to_string(),
            ])
            .expect("scope run");

            let report: Value =
                serde_json::from_str(&fs::read_to_string(&report_out).expect("read report"))
                    .expect("parse report");
            assert_eq!(
                report.get("shock_scope").and_then(|v| v.as_str()),
                Some(expected_scope)
            );
        }
    }

    #[test]
    fn eco_macro_micro_main_cli_rejects_zero_ticks_and_empty_diagnostics() {
        let root = temp_dir("main_eco_macro_micro_invalid_contract");
        let macro_model = root.join("macro.ddn");
        let micro_model = root.join("micro.ddn");
        fs::write(&macro_model, "(매마디)마다 { 값 <- 1. }.\n").expect("write macro");
        fs::write(&micro_model, "(매마디)마다 { 값 <- 1. }.\n").expect("write micro");

        let runner_ticks = root.join("runner_ticks.json");
        let spec_ticks = serde_json::json!({
            "schema": "ddn.macro_micro_runner.v0",
            "seed": 42,
            "ticks": 0,
            "models": {
                "거시": macro_model.to_string_lossy(),
                "미시": micro_model.to_string_lossy(),
            },
            "diagnostics": [
                { "name": "거시↔미시 값", "lhs": "거시.값", "rhs": "미시.값", "threshold": 0.1 }
            ]
        });
        fs::write(
            &runner_ticks,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&spec_ticks).expect("runner json")
            ),
        )
        .expect("write runner ticks");
        let err_ticks = run_parsed_eco(&[
            "teul-cli".to_string(),
            "eco".to_string(),
            "macro-micro".to_string(),
            runner_ticks.to_string_lossy().to_string(),
        ])
        .expect_err("ticks=0 must fail");
        assert!(err_ticks.contains("E_ECO_RUNNER_TICKS"));

        let runner_diag = root.join("runner_diag.json");
        let spec_diag = serde_json::json!({
            "schema": "ddn.macro_micro_runner.v0",
            "seed": 42,
            "ticks": 3,
            "models": {
                "거시": macro_model.to_string_lossy(),
                "미시": micro_model.to_string_lossy(),
            },
            "diagnostics": []
        });
        fs::write(
            &runner_diag,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&spec_diag).expect("runner json")
            ),
        )
        .expect("write runner diag");
        let err_diag = run_parsed_eco(&[
            "teul-cli".to_string(),
            "eco".to_string(),
            "macro-micro".to_string(),
            runner_diag.to_string_lossy().to_string(),
        ])
        .expect_err("empty diagnostics must fail");
        assert!(err_diag.contains("E_ECO_RUNNER_DIAG"));
    }

    #[test]
    fn eco_network_flow_main_cli_returns_fixed_error() {
        let root = temp_dir("main_eco_network_flow_fail");
        let model = root.join("network.ddn");
        let report_out = root.join("network.report.detjson");
        fs::write(
            &model,
            "(매마디)마다 {\n  임금 <- 70.\n  이전소득 <- 10.\n  소비 <- 65.\n  세금 <- 10.\n}.\n",
        )
        .expect("write model");

        let err = run_parsed_eco(&[
            "teul-cli".to_string(),
            "eco".to_string(),
            "network-flow".to_string(),
            model.to_string_lossy().to_string(),
            "--madi".to_string(),
            "1".to_string(),
            "--seed".to_string(),
            "0x2a".to_string(),
            "--threshold".to_string(),
            "0.01".to_string(),
            "--out".to_string(),
            report_out.to_string_lossy().to_string(),
        ])
        .expect_err("must fail");
        assert_eq!(err, "E_SFC_IDENTITY_VIOLATION");

        let report: Value =
            serde_json::from_str(&fs::read_to_string(&report_out).expect("read report"))
                .expect("parse report");
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.eco.network_flow_report.v0")
        );
        assert_eq!(
            report.get("error_code").and_then(|v| v.as_str()),
            Some("E_SFC_IDENTITY_VIOLATION")
        );
    }

    #[test]
    fn eco_abm_spatial_main_cli_runs_and_writes_report() {
        let root = temp_dir("main_eco_abm_spatial_ok");
        let model = root.join("abm.ddn");
        let report_out = root.join("abm.report.detjson");
        fs::write(&model, "(매마디)마다 { 부목록 <- [1, 1, 1, 1]. }.\n").expect("write model");

        run_parsed_eco(&[
            "teul-cli".to_string(),
            "eco".to_string(),
            "abm-spatial".to_string(),
            model.to_string_lossy().to_string(),
            "--madi".to_string(),
            "1".to_string(),
            "--seed".to_string(),
            "0x1".to_string(),
            "--out".to_string(),
            report_out.to_string_lossy().to_string(),
        ])
        .expect("abm command");

        let report: Value =
            serde_json::from_str(&fs::read_to_string(&report_out).expect("read report"))
                .expect("parse report");
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.eco.abm_spatial_report.v0")
        );
        assert_eq!(report.get("gini").and_then(|v| v.as_str()), Some("0"));
        assert_eq!(report.get("agent_count").and_then(|v| v.as_u64()), Some(4));
    }

    #[test]
    fn eco_network_flow_main_cli_rejects_invalid_threshold() {
        let root = temp_dir("main_eco_network_flow_bad_threshold");
        let model = root.join("network.ddn");
        fs::write(
            &model,
            "(매마디)마다 {\n  총수입 <- 100.\n  총지출 <- 100.\n}.\n",
        )
        .expect("write model");

        let err = run_parsed_eco(&[
            "teul-cli".to_string(),
            "eco".to_string(),
            "network-flow".to_string(),
            model.to_string_lossy().to_string(),
            "--threshold".to_string(),
            "abc".to_string(),
        ])
        .expect_err("must fail");
        assert!(err.contains("E_ECO_NETWORK_FLOW_THRESHOLD"));
    }

    #[test]
    fn eco_abm_spatial_main_cli_rejects_invalid_seed() {
        let root = temp_dir("main_eco_abm_spatial_bad_seed");
        let model = root.join("abm.ddn");
        fs::write(&model, "(매마디)마다 { 부목록 <- [1, 2]. }.\n").expect("write model");

        let err = run_parsed_eco(&[
            "teul-cli".to_string(),
            "eco".to_string(),
            "abm-spatial".to_string(),
            model.to_string_lossy().to_string(),
            "--seed".to_string(),
            "bad_seed".to_string(),
        ])
        .expect_err("must fail");
        assert!(err.contains("E_ECO_ABM_SPATIAL_SEED"));
    }

    #[test]
    fn gaji_registry_diag_assertions_use_helpers_only() {
        let src = include_str!("main.rs");
        assert!(
            !src.contains("assert!(err.contains(\"E_REG_"),
            "raw E_REG assertion found; use assert_diag_with_fix helper"
        );
    }

    #[test]
    fn gaji_lock_main_cli_registry_index_applies_meta() {
        let root = temp_dir("main_gaji_lock_registry_index_meta");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-lock-index",
            "sha256:index-main-lock-index",
            "표준",
            "역학",
            "0.1.0",
        );
        let lock = root.join("custom.lock.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("lock with registry index meta");

        let lock_json: Value = serde_json::from_str(&fs::read_to_string(&lock).expect("read lock"))
            .expect("lock json");
        let snapshot = lock_json
            .get("registry_snapshot")
            .expect("registry_snapshot");
        assert_eq!(
            snapshot.get("snapshot_id").and_then(|v| v.as_str()),
            Some("snap-main-lock-index")
        );
        assert_eq!(
            snapshot.get("index_root_hash").and_then(|v| v.as_str()),
            Some("sha256:index-main-lock-index")
        );
        let trust = lock_json.get("trust_root").expect("trust_root");
        assert_eq!(
            trust.get("hash").and_then(|v| v.as_str()),
            Some("sha256:trust")
        );
        assert_eq!(
            trust.get("source").and_then(|v| v.as_str()),
            Some("registry")
        );
    }

    #[test]
    fn gaji_lock_main_cli_registry_index_missing_file_fails() {
        let root = temp_dir("main_gaji_lock_registry_index_missing_file");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let index = root.join("missing.registry.index.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing registry index must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_READ");
    }

    #[test]
    fn gaji_lock_main_cli_registry_index_invalid_json_fails() {
        let root = temp_dir("main_gaji_lock_registry_index_invalid_json");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let index = root.join("registry.index.json");
        fs::write(&index, "{ invalid json").expect("write bad index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("invalid registry index json must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_PARSE");
    }

    #[test]
    fn gaji_lock_main_cli_registry_index_missing_snapshot_meta_fails() {
        let root = temp_dir("main_gaji_lock_registry_index_missing_snapshot_meta");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "entries": [{
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "0.1.0",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing snapshot meta must fail");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISSING");
    }

    #[test]
    fn gaji_lock_main_cli_invalid_trust_root_source_fails() {
        let root = temp_dir("main_gaji_lock_invalid_trust_root_source");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
            "--snapshot-id".to_string(),
            "snap-main-lock-invalid-source".to_string(),
            "--index-root-hash".to_string(),
            "sha256:index-main-lock-invalid-source".to_string(),
            "--trust-root-hash".to_string(),
            "sha256:trust".to_string(),
            "--trust-root-source".to_string(),
            "invalid".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("invalid trust_root_source must fail");
        assert!(err.contains("E_GAJI_LOCK_META"));
    }

    #[test]
    fn gaji_lock_main_cli_snapshot_id_without_index_root_hash_fails() {
        let root = temp_dir("main_gaji_lock_snapshot_without_index_hash");
        write_demo_pkg(&root, "표준/역학", "0.1.0");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--snapshot-id".to_string(),
            "snap-main-lock-only-snapshot".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("snapshot only must fail");
        assert!(err.contains("E_GAJI_LOCK_META"));
        assert!(err.contains("snapshot_id/index_root_hash"));
    }

    #[test]
    fn gaji_lock_main_cli_index_root_hash_without_snapshot_id_fails() {
        let root = temp_dir("main_gaji_lock_index_hash_without_snapshot");
        write_demo_pkg(&root, "표준/역학", "0.1.0");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--index-root-hash".to_string(),
            "sha256:index-main-lock-only-hash".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("index hash only must fail");
        assert!(err.contains("E_GAJI_LOCK_META"));
        assert!(err.contains("snapshot_id/index_root_hash"));
    }

    #[test]
    fn gaji_lock_main_cli_trust_root_hash_without_source_fails() {
        let root = temp_dir("main_gaji_lock_trust_hash_without_source");
        write_demo_pkg(&root, "표준/역학", "0.1.0");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--trust-root-hash".to_string(),
            "sha256:trust".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("trust hash only must fail");
        assert!(err.contains("E_GAJI_LOCK_META"));
        assert!(err.contains("trust_root_hash/trust_root_source"));
    }

    #[test]
    fn gaji_lock_main_cli_trust_root_source_without_hash_fails() {
        let root = temp_dir("main_gaji_lock_trust_source_without_hash");
        write_demo_pkg(&root, "표준/역학", "0.1.0");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--trust-root-source".to_string(),
            "registry".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("trust source only must fail");
        assert!(err.contains("E_GAJI_LOCK_META"));
        assert!(err.contains("trust_root_hash/trust_root_source"));
    }

    #[test]
    fn gaji_lock_main_cli_audit_last_hash_writes_registry_audit_meta() {
        let root = temp_dir("main_gaji_lock_audit_last_hash_meta");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
            "--audit-last-hash".to_string(),
            "blake3:last-main-lock".to_string(),
        ];
        run_parsed_gaji(&args).expect("lock with audit-last-hash");

        let lock_json: Value = serde_json::from_str(&fs::read_to_string(&lock).expect("read lock"))
            .expect("lock json");
        assert_eq!(
            lock_json
                .get("registry_audit")
                .and_then(|v| v.get("last_hash"))
                .and_then(|v| v.as_str()),
            Some("blake3:last-main-lock")
        );
    }

    #[test]
    fn gaji_lock_main_cli_writes_registry_snapshot_and_trust_root_meta() {
        let root = temp_dir("main_gaji_lock_snapshot_trust_meta");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
            "--snapshot-id".to_string(),
            "snap-main-lock-meta".to_string(),
            "--index-root-hash".to_string(),
            "sha256:index-main-lock-meta".to_string(),
            "--trust-root-hash".to_string(),
            "sha256:trust-main-lock-meta".to_string(),
            "--trust-root-source".to_string(),
            "registry".to_string(),
        ];
        run_parsed_gaji(&args).expect("lock with snapshot/trust meta");

        let lock_json: Value = serde_json::from_str(&fs::read_to_string(&lock).expect("read lock"))
            .expect("lock json");
        assert_eq!(
            lock_json
                .get("registry_snapshot")
                .and_then(|v| v.get("snapshot_id"))
                .and_then(|v| v.as_str()),
            Some("snap-main-lock-meta")
        );
        assert_eq!(
            lock_json
                .get("registry_snapshot")
                .and_then(|v| v.get("index_root_hash"))
                .and_then(|v| v.as_str()),
            Some("sha256:index-main-lock-meta")
        );
        assert_eq!(
            lock_json
                .get("trust_root")
                .and_then(|v| v.get("hash"))
                .and_then(|v| v.as_str()),
            Some("sha256:trust-main-lock-meta")
        );
        assert_eq!(
            lock_json
                .get("trust_root")
                .and_then(|v| v.get("source"))
                .and_then(|v| v.as_str()),
            Some("registry")
        );
    }

    #[test]
    fn gaji_lock_main_cli_default_out_writes_ddn_lock() {
        let root = temp_dir("main_gaji_lock_default_out");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        assert!(!lock.exists());

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("lock default out");
        assert!(lock.exists());

        let lock_json: Value = serde_json::from_str(&fs::read_to_string(&lock).expect("read lock"))
            .expect("lock json");
        assert_eq!(
            lock_json.get("schema_version").and_then(|v| v.as_str()),
            Some("v1")
        );
    }

    #[test]
    fn gaji_lock_main_cli_out_parent_not_directory_fails() {
        let root = temp_dir("main_gaji_lock_out_parent_file");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let blocker = root.join("not_dir");
        fs::write(&blocker, "block").expect("write blocker file");
        let out = blocker.join("ddn.lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("lock out parent file should fail");
        assert!(err.contains("E_GAJI_WRITE"));
    }

    #[test]
    fn gaji_lock_main_cli_missing_gaji_dir_fails() {
        let root = temp_dir("main_gaji_lock_missing_gaji_dir");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing gaji dir must fail");
        assert!(err.contains("E_GAJI_SCAN"));
    }

    #[test]
    fn gaji_lock_main_cli_missing_version_in_gaji_toml_fails() {
        let root = temp_dir("main_gaji_lock_missing_toml_version");
        let pkg = root.join("gaji").join("demo");
        fs::create_dir_all(&pkg).expect("pkg mkdir");
        fs::write(pkg.join("gaji.toml"), "id = \"표준/역학\"\n").expect("write toml");
        fs::write(pkg.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing version must fail");
        assert!(err.contains("E_GAJI_TOML_VERSION"));
    }

    #[test]
    fn gaji_lock_main_cli_gaji_toml_is_directory_fails_with_read_error() {
        let root = temp_dir("main_gaji_lock_toml_dir_read_error");
        let pkg = root.join("gaji").join("demo");
        fs::create_dir_all(pkg.join("gaji.toml")).expect("mkdir toml dir");
        fs::write(pkg.join("main.ddn"), "값 <- 1.\n").expect("write src");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("directory toml should fail");
        assert!(err.contains("E_GAJI_READ"));
    }

    #[test]
    fn gaji_install_main_cli_strict_audit_requires_last_hash_pin() {
        let root = temp_dir("main_gaji_install_need_pin");
        write_demo_pkg(&root, "표준/역학", "0.1.0");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-install",
            "sha256:index-main-install",
            "표준",
            "역학",
            "0.1.0",
        );
        let audit = root.join("registry.audit.jsonl");
        fs::write(&audit, "").expect("touch audit");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must require last hash pin");
        assert_diag_with_fix(&err, "E_REG_AUDIT_LAST_HASH_REQUIRED");
    }

    #[test]
    fn gaji_install_main_cli_bootstraps_lock_when_missing() {
        let root = temp_dir("main_gaji_install_bootstrap_lock");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        assert!(!lock.exists());

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("install should bootstrap lock");
        assert!(lock.exists());
        assert!(root
            .join("vendor")
            .join("gaji")
            .join("demo")
            .join("main.ddn")
            .exists());
        let vendor_index = root
            .join("vendor")
            .join("gaji")
            .join("ddn.vendor.index.json");
        assert!(vendor_index.exists());
        let index_json: Value =
            serde_json::from_str(&fs::read_to_string(vendor_index).expect("read vendor index"))
                .expect("parse vendor index");
        assert_eq!(
            index_json.get("schema_version").and_then(|v| v.as_str()),
            Some("ddn.vendor.index.v1")
        );
    }

    #[test]
    fn gaji_install_main_cli_missing_gaji_dir_fails() {
        let root = temp_dir("main_gaji_install_missing_gaji_dir");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing gaji dir must fail");
        assert!(err.contains("E_GAJI_SCAN"));
    }

    #[test]
    fn gaji_install_main_cli_invalid_existing_lock_json_fails() {
        let root = temp_dir("main_gaji_install_invalid_existing_lock_json");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        fs::write(&lock, "{ invalid json").expect("write bad lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("invalid existing lock should fail");
        assert!(err.contains("E_GAJI_LOCK_PARSE"));
    }

    #[test]
    fn gaji_install_main_cli_existing_lock_schema_mismatch_fails() {
        let root = temp_dir("main_gaji_install_existing_lock_schema_mismatch");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        fs::write(
            &lock,
            serde_json::to_string_pretty(&json!({
                "schema_version": "v2",
                "lock_hash": "blake3:dummy",
                "packages": []
            }))
            .expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("schema mismatch should fail");
        assert!(err.contains("E_GAJI_LOCK_SCHEMA"));
    }

    #[test]
    fn gaji_install_main_cli_existing_lock_packages_missing_fails() {
        let root = temp_dir("main_gaji_install_existing_lock_packages_missing");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        fs::write(
            &lock,
            serde_json::to_string_pretty(&json!({
                "schema_version": "v1",
                "lock_hash": "blake3:dummy"
            }))
            .expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing packages should fail");
        assert!(err.contains("E_GAJI_LOCK_PACKAGES"));
    }

    #[test]
    fn gaji_install_main_cli_existing_lock_field_missing_id_fails() {
        let root = temp_dir("main_gaji_install_existing_lock_field_missing_id");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        fs::write(
            &lock,
            serde_json::to_string_pretty(&json!({
                "schema_version": "v1",
                "lock_hash": "blake3:dummy",
                "packages": [{
                    "version": "0.1.0",
                    "path": "demo",
                    "hash": "blake3:dummy"
                }]
            }))
            .expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing id should fail");
        assert!(err.contains("E_GAJI_LOCK_FIELD"));
        assert!(err.contains("id"));
    }

    #[test]
    fn gaji_install_main_cli_existing_lock_field_missing_hash_fails() {
        let root = temp_dir("main_gaji_install_existing_lock_field_missing_hash");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        fs::write(
            &lock,
            serde_json::to_string_pretty(&json!({
                "schema_version": "v1",
                "lock_hash": "blake3:dummy",
                "packages": [{
                    "id": "표준/역학",
                    "version": "0.1.0",
                    "path": "demo"
                }]
            }))
            .expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing hash should fail");
        assert!(err.contains("E_GAJI_LOCK_FIELD"));
        assert!(err.contains("hash"));
    }

    #[test]
    fn gaji_install_main_cli_out_exists_as_file_fails_with_clean_error() {
        let root = temp_dir("main_gaji_install_out_file_clean_error");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let out = root.join("vendor").join("gaji");
        fs::create_dir_all(out.parent().expect("parent")).expect("mkdir parent");
        fs::write(&out, "not directory").expect("write out file");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("existing out file should fail");
        assert!(err.contains("E_GAJI_VENDOR_CLEAN"));
    }

    #[test]
    fn gaji_install_main_cli_out_parent_not_directory_fails_with_write_error() {
        let root = temp_dir("main_gaji_install_out_parent_file_write_error");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let blocker = root.join("not_dir");
        fs::write(&blocker, "block").expect("write blocker file");
        let out = blocker.join("vendor");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("out parent file should fail");
        assert!(err.contains("E_GAJI_VENDOR_WRITE"));
    }

    #[test]
    fn gaji_install_main_cli_strict_registry_requires_index_before_bootstrap() {
        let root = temp_dir("main_gaji_install_strict_requires_index");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        assert!(!lock.exists());

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("strict install requires index");
        assert_diag_with_fix(&err, "E_REG_VERIFY_INDEX_REQUIRED");
    }

    #[test]
    fn gaji_install_main_cli_strict_registry_bootstraps_lock_from_index() {
        let root = temp_dir("main_gaji_install_bootstrap_from_index");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        assert!(!lock.exists());

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-install-bootstrap",
            "sha256:index-main-install-bootstrap",
            "표준",
            "역학",
            "0.1.0",
        );

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("strict install with index should pass");
        let lock_json: Value = serde_json::from_str(&fs::read_to_string(&lock).expect("read lock"))
            .expect("parse lock");
        assert_eq!(
            lock_json
                .get("registry_snapshot")
                .and_then(|v| v.get("snapshot_id"))
                .and_then(|v| v.as_str()),
            Some("snap-main-install-bootstrap")
        );
        assert_eq!(
            lock_json
                .get("registry_snapshot")
                .and_then(|v| v.get("index_root_hash"))
                .and_then(|v| v.as_str()),
            Some("sha256:index-main-install-bootstrap")
        );
    }

    #[test]
    fn gaji_install_main_cli_strict_registry_index_missing_snapshot_meta_fails() {
        let root = temp_dir("main_gaji_install_missing_snapshot_meta");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "entries": [{
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "0.1.0",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing snapshot meta must fail");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISSING");
    }

    #[test]
    fn gaji_install_main_cli_strict_registry_missing_index_file_fails() {
        let root = temp_dir("main_gaji_install_missing_index");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let index = root.join("missing.registry.index.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on missing registry index");
        assert_diag_with_fix(&err, "E_REG_INDEX_READ");
    }

    #[test]
    fn gaji_install_main_cli_strict_registry_invalid_index_json_fails() {
        let root = temp_dir("main_gaji_install_invalid_index_json");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let index = root.join("registry.index.json");
        fs::write(&index, "{ invalid json").expect("write bad index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on invalid registry index json");
        assert_diag_with_fix(&err, "E_REG_INDEX_PARSE");
    }

    #[test]
    fn gaji_install_main_cli_verify_registry_requires_index() {
        let root = temp_dir("main_gaji_install_verify_registry_requires_index");
        write_demo_pkg(&root, "표준/역학", "0.1.0");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--verify-registry".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("verify-registry requires index");
        assert_diag_with_fix(&err, "E_REG_VERIFY_INDEX_REQUIRED");
    }

    #[test]
    fn gaji_install_main_cli_verify_registry_audit_requires_log() {
        let root = temp_dir("main_gaji_install_verify_registry_audit_requires_log");
        write_demo_pkg(&root, "표준/역학", "0.1.0");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--verify-registry-audit".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("verify-registry-audit requires log");
        assert_diag_with_fix(&err, "E_REG_AUDIT_VERIFY_LOG_REQUIRED");
    }

    #[test]
    fn gaji_install_main_cli_verify_registry_writes_default_report_path() {
        let root = temp_dir("main_gaji_install_verify_registry_default_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-install-verify-report",
            "sha256:index-main-install-verify-report",
            "표준",
            "역학",
            "0.1.0",
        );

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--verify-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("install verify-registry");

        let report = root.join("vendor").join("registry.verify.json");
        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_eq!(
            report_json.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.verify_report.v1")
        );
        assert_eq!(
            report_json.get("packages").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            report_json
                .get("duplicate_resolution_policy")
                .and_then(|v| v.as_str()),
            Some(crate::cli::gaji_registry::VERIFY_DUPLICATE_RESOLUTION_POLICY)
        );
    }

    #[test]
    fn gaji_install_main_cli_verify_registry_writes_explicit_report_path() {
        let root = temp_dir("main_gaji_install_verify_registry_explicit_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-install-verify-explicit-report",
            "sha256:index-main-install-verify-explicit-report",
            "표준",
            "역학",
            "0.1.0",
        );
        let report = root
            .join("reports")
            .join("install.registry.verify.explicit.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--verify-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
            "--registry-verify-out".to_string(),
            report.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("install verify-registry with explicit report");

        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_eq!(
            report_json.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.verify_report.v1")
        );
        assert_eq!(
            report_json.get("packages").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(report_json.get("matched").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(
            report_json
                .get("duplicate_resolution_policy")
                .and_then(|v| v.as_str()),
            Some(crate::cli::gaji_registry::VERIFY_DUPLICATE_RESOLUTION_POLICY)
        );
    }

    #[test]
    fn gaji_install_main_cli_verify_registry_audit_writes_default_report_path() {
        let root = temp_dir("main_gaji_install_verify_registry_audit_default_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let audit = root.join("registry.audit.jsonl");
        let _ = write_audit_log_and_last_hash(&audit);

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--verify-registry-audit".to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("install verify-registry-audit");

        let report = root.join("vendor").join("registry.audit.verify.json");
        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_audit_verify_report_contract(&report_json, 1);
    }

    #[test]
    fn gaji_install_main_cli_verify_registry_audit_writes_explicit_report_path() {
        let root = temp_dir("main_gaji_install_verify_registry_audit_explicit_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let audit = root.join("registry.audit.jsonl");
        let _ = write_audit_log_and_last_hash(&audit);
        let report = root
            .join("reports")
            .join("install.registry.audit.verify.explicit.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--verify-registry-audit".to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--registry-audit-verify-out".to_string(),
            report.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("install verify-registry-audit with explicit report");

        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_audit_verify_report_contract(&report_json, 1);
    }

    #[test]
    fn gaji_install_main_cli_verify_registry_custom_out_writes_report_next_to_out() {
        let root = temp_dir("main_gaji_install_verify_registry_custom_out_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-install-verify-custom-out",
            "sha256:index-main-install-verify-custom-out",
            "표준",
            "역학",
            "0.1.0",
        );
        let out = root.join("custom_vendor").join("bundle");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("install verify-registry with custom out");

        let report = root.join("custom_vendor").join("registry.verify.json");
        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_verify_report_contract(&report_json, 1, 1);
    }

    #[test]
    fn gaji_install_main_cli_verify_registry_audit_custom_out_writes_report_next_to_out() {
        let root = temp_dir("main_gaji_install_verify_registry_audit_custom_out_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let audit = root.join("registry.audit.jsonl");
        let _ = write_audit_log_and_last_hash(&audit);
        let out = root.join("custom_vendor").join("bundle");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-registry-audit".to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("install verify-registry-audit with custom out");

        let report = root
            .join("custom_vendor")
            .join("registry.audit.verify.json");
        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_eq!(report_json.get("rows").and_then(|v| v.as_u64()), Some(1));
    }

    #[test]
    fn gaji_install_main_cli_strict_audit_accepts_expect_last_hash() {
        let root = temp_dir("main_gaji_install_with_pin");
        write_demo_pkg(&root, "표준/역학", "0.1.0");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-install-ok",
            "sha256:index-main-install-ok",
            "표준",
            "역학",
            "0.1.0",
        );
        let audit = root.join("registry.audit.jsonl");
        let expected_hash = write_audit_log_and_last_hash(&audit);

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            expected_hash,
        ];
        run_parsed_gaji(&args).expect("install should pass with expected hash pin");
        assert!(root
            .join("vendor")
            .join("gaji")
            .join("demo")
            .join("main.ddn")
            .exists());
    }

    #[test]
    fn gaji_install_main_cli_strict_audit_uses_lock_last_hash_pin() {
        let root = temp_dir("main_gaji_install_lock_pin");
        write_demo_pkg(&root, "표준/역학", "0.1.0");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-install-lock-pin",
            "sha256:index-main-install-lock-pin",
            "표준",
            "역학",
            "0.1.0",
        );
        let audit = root.join("registry.audit.jsonl");
        let expected_hash = write_audit_log_and_last_hash(&audit);

        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-install-lock-pin".to_string()),
                index_root_hash: Some("sha256:index-main-install-lock-pin".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: Some(expected_hash),
            },
        )
        .expect("lock with audit pin");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "install".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("install should pass with lock audit pin");
        assert!(root
            .join("vendor")
            .join("gaji")
            .join("demo")
            .join("main.ddn")
            .exists());
    }

    #[test]
    fn gaji_update_main_cli_strict_audit_requires_last_hash_pin() {
        let root = temp_dir("main_gaji_update_need_pin");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-update".to_string()),
                index_root_hash: Some("sha256:index-main-update".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-update",
            "sha256:index-main-update",
            "표준",
            "역학",
            "0.1.0",
        );
        let audit = root.join("registry.audit.jsonl");
        fs::write(&audit, "").expect("touch audit");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must require last hash pin");
        assert_diag_with_fix(&err, "E_REG_AUDIT_LAST_HASH_REQUIRED");
    }

    #[test]
    fn gaji_update_main_cli_strict_registry_requires_index() {
        let root = temp_dir("main_gaji_update_strict_requires_index");
        write_demo_pkg(&root, "표준/역학", "0.1.0");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("strict update requires index");
        assert_diag_with_fix(&err, "E_REG_VERIFY_INDEX_REQUIRED");
    }

    #[test]
    fn gaji_update_main_cli_strict_registry_missing_index_file_fails() {
        let root = temp_dir("main_gaji_update_missing_index");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let index = root.join("missing.registry.index.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on missing registry index");
        assert_diag_with_fix(&err, "E_REG_INDEX_READ");
    }

    #[test]
    fn gaji_update_main_cli_strict_registry_invalid_index_json_fails() {
        let root = temp_dir("main_gaji_update_invalid_index_json");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let index = root.join("registry.index.json");
        fs::write(&index, "{ invalid json").expect("write bad index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on invalid registry index json");
        assert_diag_with_fix(&err, "E_REG_INDEX_PARSE");
    }

    #[test]
    fn gaji_update_main_cli_verify_registry_requires_index() {
        let root = temp_dir("main_gaji_update_verify_registry_requires_index");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");

        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-registry".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("verify-registry requires index");
        assert_diag_with_fix(&err, "E_REG_VERIFY_INDEX_REQUIRED");
    }

    #[test]
    fn gaji_update_main_cli_verify_registry_audit_requires_log() {
        let root = temp_dir("main_gaji_update_verify_registry_audit_requires_log");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");

        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-registry-audit".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("verify-registry-audit requires log");
        assert_diag_with_fix(&err, "E_REG_AUDIT_VERIFY_LOG_REQUIRED");
    }

    #[test]
    fn gaji_update_main_cli_verify_registry_writes_default_report_path() {
        let root = temp_dir("main_gaji_update_verify_registry_default_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-update-verify-report",
            "sha256:index-main-update-verify-report",
            "표준",
            "역학",
            "0.1.0",
        );

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("update verify-registry");

        let report = root.join("vendor").join("registry.verify.json");
        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_eq!(
            report_json.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.verify_report.v1")
        );
        assert_eq!(
            report_json.get("packages").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(report_json.get("matched").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(
            report_json
                .get("duplicate_resolution_policy")
                .and_then(|v| v.as_str()),
            Some(crate::cli::gaji_registry::VERIFY_DUPLICATE_RESOLUTION_POLICY)
        );
    }

    #[test]
    fn gaji_update_main_cli_verify_registry_writes_explicit_report_path() {
        let root = temp_dir("main_gaji_update_verify_registry_explicit_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-update-verify-explicit-report",
            "sha256:index-main-update-verify-explicit-report",
            "표준",
            "역학",
            "0.1.0",
        );
        let report = root
            .join("reports")
            .join("update.registry.verify.explicit.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
            "--registry-verify-out".to_string(),
            report.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("update verify-registry with explicit report");

        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_eq!(
            report_json.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.verify_report.v1")
        );
        assert_eq!(
            report_json.get("packages").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(report_json.get("matched").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(
            report_json
                .get("duplicate_resolution_policy")
                .and_then(|v| v.as_str()),
            Some(crate::cli::gaji_registry::VERIFY_DUPLICATE_RESOLUTION_POLICY)
        );
    }

    #[test]
    fn gaji_update_main_cli_verify_registry_audit_writes_default_report_path() {
        let root = temp_dir("main_gaji_update_verify_registry_audit_default_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let audit = root.join("registry.audit.jsonl");
        let _ = write_audit_log_and_last_hash(&audit);

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-registry-audit".to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("update verify-registry-audit");

        let report = root.join("vendor").join("registry.audit.verify.json");
        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_audit_verify_report_contract(&report_json, 1);
    }

    #[test]
    fn gaji_update_main_cli_verify_registry_audit_writes_explicit_report_path() {
        let root = temp_dir("main_gaji_update_verify_registry_audit_explicit_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let audit = root.join("registry.audit.jsonl");
        let _ = write_audit_log_and_last_hash(&audit);
        let report = root
            .join("reports")
            .join("update.registry.audit.verify.explicit.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-registry-audit".to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--registry-audit-verify-out".to_string(),
            report.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("update verify-registry-audit with explicit report");

        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_audit_verify_report_contract(&report_json, 1);
    }

    #[test]
    fn gaji_update_main_cli_verify_registry_custom_out_writes_report_next_to_out() {
        let root = temp_dir("main_gaji_update_verify_registry_custom_out_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-update-verify-custom-out",
            "sha256:index-main-update-verify-custom-out",
            "표준",
            "역학",
            "0.1.0",
        );
        let out = root.join("custom_vendor").join("bundle");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("update verify-registry with custom out");

        let report = root.join("custom_vendor").join("registry.verify.json");
        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_verify_report_contract(&report_json, 1, 1);
    }

    #[test]
    fn gaji_update_main_cli_verify_registry_audit_custom_out_writes_report_next_to_out() {
        let root = temp_dir("main_gaji_update_verify_registry_audit_custom_out_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let audit = root.join("registry.audit.jsonl");
        let _ = write_audit_log_and_last_hash(&audit);
        let out = root.join("custom_vendor").join("bundle");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-registry-audit".to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("update verify-registry-audit with custom out");

        let report = root
            .join("custom_vendor")
            .join("registry.audit.verify.json");
        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_eq!(report_json.get("rows").and_then(|v| v.as_u64()), Some(1));
    }

    #[test]
    fn gaji_update_main_cli_strict_audit_lock_last_hash_mismatch_fails() {
        let root = temp_dir("main_gaji_update_lock_hash_mismatch");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-update-lock-bad".to_string()),
                index_root_hash: Some("sha256:index-main-update-lock-bad".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: Some("blake3:not-match".to_string()),
            },
        )
        .expect("lock");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-update-lock-bad",
            "sha256:index-main-update-lock-bad",
            "표준",
            "역학",
            "0.1.0",
        );
        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on lock audit hash mismatch");
        assert_audit_last_hash_diag(&err);
    }

    #[test]
    fn gaji_update_main_cli_strict_registry_applies_index_meta_to_existing_lock() {
        let root = temp_dir("main_gaji_update_apply_index_meta");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock(&root, &lock).expect("lock");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-update-apply",
            "sha256:index-main-update-apply",
            "표준",
            "역학",
            "0.1.0",
        );

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("strict update with index should pass");

        let lock_json: Value = serde_json::from_str(&fs::read_to_string(&lock).expect("read lock"))
            .expect("parse lock");
        assert_eq!(
            lock_json
                .get("registry_snapshot")
                .and_then(|v| v.get("snapshot_id"))
                .and_then(|v| v.as_str()),
            Some("snap-main-update-apply")
        );
        assert_eq!(
            lock_json
                .get("registry_snapshot")
                .and_then(|v| v.get("index_root_hash"))
                .and_then(|v| v.as_str()),
            Some("sha256:index-main-update-apply")
        );
    }

    #[test]
    fn gaji_update_main_cli_preserves_existing_lock_meta() {
        let root = temp_dir("main_gaji_update_preserve_meta");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("2026-02-20T00:00:00Z#000001".to_string()),
                index_root_hash: Some("sha256:index".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("mirror".to_string()),
                audit_last_hash: Some("blake3:audit_last".to_string()),
            },
        )
        .expect("lock with meta");
        fs::write(
            root.join("gaji").join("demo").join("main.ddn"),
            "값 <- 2.\n",
        )
        .expect("update src");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("update should preserve lock meta");

        let lock_json: Value = serde_json::from_str(&fs::read_to_string(&lock).expect("read lock"))
            .expect("parse lock");
        assert_eq!(
            lock_json
                .get("registry_snapshot")
                .and_then(|v| v.get("snapshot_id"))
                .and_then(|v| v.as_str()),
            Some("2026-02-20T00:00:00Z#000001")
        );
        assert_eq!(
            lock_json
                .get("registry_snapshot")
                .and_then(|v| v.get("index_root_hash"))
                .and_then(|v| v.as_str()),
            Some("sha256:index")
        );
        assert_eq!(
            lock_json
                .get("trust_root")
                .and_then(|v| v.get("hash"))
                .and_then(|v| v.as_str()),
            Some("sha256:trust")
        );
        assert_eq!(
            lock_json
                .get("trust_root")
                .and_then(|v| v.get("source"))
                .and_then(|v| v.as_str()),
            Some("mirror")
        );
        assert_eq!(
            lock_json
                .get("registry_audit")
                .and_then(|v| v.get("last_hash"))
                .and_then(|v| v.as_str()),
            Some("blake3:audit_last")
        );
    }

    #[test]
    fn gaji_update_main_cli_default_paths_write_vendor_index() {
        let root = temp_dir("main_gaji_update_default_paths");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        assert!(!lock.exists());

        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare default lock");
        assert!(lock.exists());

        fs::write(
            root.join("gaji").join("demo").join("main.ddn"),
            "값 <- 2.\n",
        )
        .expect("update source");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("update default paths");

        let vendor_index = root
            .join("vendor")
            .join("gaji")
            .join("ddn.vendor.index.json");
        assert!(vendor_index.exists());
        let index_json: Value =
            serde_json::from_str(&fs::read_to_string(vendor_index).expect("read vendor index"))
                .expect("parse vendor index");
        assert_eq!(
            index_json.get("schema_version").and_then(|v| v.as_str()),
            Some("ddn.vendor.index.v1")
        );
    }

    #[test]
    fn gaji_update_main_cli_refreshes_package_hash_and_vendor_index() {
        let root = temp_dir("main_gaji_update_refresh_hash");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");

        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let before: Value =
            serde_json::from_str(&fs::read_to_string(&lock).expect("read lock before"))
                .expect("lock json before");
        let before_hash = before
            .get("packages")
            .and_then(|v| v.as_array())
            .and_then(|rows| rows.first())
            .and_then(|row| row.get("hash"))
            .and_then(|v| v.as_str())
            .expect("before package hash")
            .to_string();

        fs::write(
            root.join("gaji").join("demo").join("main.ddn"),
            "값 <- 999.\n",
        )
        .expect("update source");

        let out = root.join("vendor").join("gaji");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("update should refresh hash");

        let after: Value =
            serde_json::from_str(&fs::read_to_string(&lock).expect("read lock after"))
                .expect("lock json after");
        let after_hash = after
            .get("packages")
            .and_then(|v| v.as_array())
            .and_then(|rows| rows.first())
            .and_then(|row| row.get("hash"))
            .and_then(|v| v.as_str())
            .expect("after package hash")
            .to_string();
        assert_ne!(before_hash, after_hash);

        let vendor_index: Value = serde_json::from_str(
            &fs::read_to_string(out.join("ddn.vendor.index.json")).expect("read vendor index"),
        )
        .expect("parse vendor index");
        let vendor_hash = vendor_index
            .get("packages")
            .and_then(|v| v.as_array())
            .and_then(|rows| rows.first())
            .and_then(|row| row.get("hash"))
            .and_then(|v| v.as_str())
            .expect("vendor package hash");
        assert_eq!(vendor_hash, after_hash);
    }

    #[test]
    fn gaji_update_main_cli_missing_gaji_dir_fails() {
        let root = temp_dir("main_gaji_update_missing_gaji_dir");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing gaji dir must fail");
        assert!(err.contains("E_GAJI_SCAN"));
    }

    #[test]
    fn gaji_update_main_cli_missing_lock_bootstraps_and_passes() {
        let root = temp_dir("main_gaji_update_missing_lock");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("missing.ddn.lock");
        assert!(!lock.exists());

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("missing lock should bootstrap");
        assert!(lock.exists());
        assert!(root
            .join("vendor")
            .join("gaji")
            .join("ddn.vendor.index.json")
            .exists());
    }

    #[test]
    fn gaji_update_main_cli_invalid_lock_json_fails() {
        let root = temp_dir("main_gaji_update_invalid_lock_json");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        fs::write(&lock, "{ invalid json").expect("write bad lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("invalid lock json should fail");
        assert!(err.contains("E_GAJI_LOCK_PARSE"));
    }

    #[test]
    fn gaji_update_main_cli_lock_schema_mismatch_fails() {
        let root = temp_dir("main_gaji_update_lock_schema_mismatch");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        fs::write(
            &lock,
            serde_json::to_string_pretty(&json!({
                "schema_version": "v2",
                "lock_hash": "blake3:dummy",
                "packages": []
            }))
            .expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("schema mismatch should fail");
        assert!(err.contains("E_GAJI_LOCK_SCHEMA"));
    }

    #[test]
    fn gaji_update_main_cli_out_exists_as_file_fails_with_clean_error() {
        let root = temp_dir("main_gaji_update_out_file_clean_error");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let out = root.join("vendor").join("gaji");
        fs::create_dir_all(out.parent().expect("parent")).expect("mkdir parent");
        fs::write(&out, "not directory").expect("write out file");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("existing out file should fail");
        assert!(err.contains("E_GAJI_VENDOR_CLEAN"));
    }

    #[test]
    fn gaji_update_main_cli_out_parent_not_directory_fails_with_write_error() {
        let root = temp_dir("main_gaji_update_out_parent_file_write_error");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let blocker = root.join("not_dir");
        fs::write(&blocker, "block").expect("write blocker file");
        let out = blocker.join("vendor");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "update".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("out parent file should fail");
        assert!(err.contains("E_GAJI_VENDOR_WRITE"));
    }

    #[test]
    fn gaji_vendor_main_cli_strict_audit_requires_last_hash_pin() {
        let root = temp_dir("main_gaji_vendor_need_pin");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-vendor".to_string()),
                index_root_hash: Some("sha256:index-main-vendor".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-vendor",
            "sha256:index-main-vendor",
            "표준",
            "역학",
            "0.1.0",
        );
        let audit = root.join("registry.audit.jsonl");
        fs::write(&audit, "").expect("touch audit");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must require last hash pin");
        assert_diag_with_fix(&err, "E_REG_AUDIT_LAST_HASH_REQUIRED");
    }

    #[test]
    fn gaji_vendor_main_cli_strict_registry_requires_index() {
        let root = temp_dir("main_gaji_vendor_strict_requires_index");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");

        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
            "--snapshot-id".to_string(),
            "snap-main-vendor-strict".to_string(),
            "--index-root-hash".to_string(),
            "sha256:index-main-vendor-strict".to_string(),
            "--trust-root-hash".to_string(),
            "sha256:trust".to_string(),
            "--trust-root-source".to_string(),
            "registry".to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("lock with strict-ready meta");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("strict vendor requires index");
        assert_diag_with_fix(&err, "E_REG_VERIFY_INDEX_REQUIRED");
    }

    #[test]
    fn gaji_vendor_main_cli_strict_audit_accepts_expect_last_hash() {
        let root = temp_dir("main_gaji_vendor_with_pin");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-vendor-pin".to_string()),
                index_root_hash: Some("sha256:index-main-vendor-pin".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-vendor-pin",
            "sha256:index-main-vendor-pin",
            "표준",
            "역학",
            "0.1.0",
        );
        let audit = root.join("registry.audit.jsonl");
        let expected_hash = write_audit_log_and_last_hash(&audit);

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            expected_hash,
        ];
        run_parsed_gaji(&args).expect("vendor strict+audit with expected hash should pass");
    }

    #[test]
    fn gaji_vendor_main_cli_strict_registry_missing_index_file_fails() {
        let root = temp_dir("main_gaji_vendor_missing_index");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-vendor-missing-index".to_string()),
                index_root_hash: Some("sha256:index-main-vendor-missing-index".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock");
        let index = root.join("missing.registry.index.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on missing registry index");
        assert_diag_with_fix(&err, "E_REG_INDEX_READ");
    }

    #[test]
    fn gaji_vendor_main_cli_missing_lock_fails() {
        let root = temp_dir("main_gaji_vendor_missing_lock");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("missing.ddn.lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing lock should fail");
        assert!(err.contains("E_GAJI_LOCK_READ"));
    }

    #[test]
    fn gaji_vendor_main_cli_invalid_lock_json_fails() {
        let root = temp_dir("main_gaji_vendor_invalid_lock_json");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        fs::write(&lock, "{ invalid json").expect("write bad lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("invalid lock json should fail");
        assert!(err.contains("E_GAJI_LOCK_PARSE"));
    }

    #[test]
    fn gaji_vendor_main_cli_lock_packages_missing_fails() {
        let root = temp_dir("main_gaji_vendor_lock_packages_missing");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        fs::write(
            &lock,
            serde_json::to_string_pretty(&json!({
                "schema_version": "v1",
                "lock_hash": "blake3:dummy"
            }))
            .expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing packages should fail");
        assert!(err.contains("E_GAJI_LOCK_PACKAGES"));
    }

    #[test]
    fn gaji_vendor_main_cli_lock_field_missing_id_fails() {
        let root = temp_dir("main_gaji_vendor_lock_field_missing_id");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        fs::write(
            &lock,
            serde_json::to_string_pretty(&json!({
                "schema_version": "v1",
                "lock_hash": "blake3:dummy",
                "packages": [{
                    "version": "0.1.0",
                    "path": "demo",
                    "hash": "blake3:dummy"
                }]
            }))
            .expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing id field should fail");
        assert!(err.contains("E_GAJI_LOCK_FIELD"));
        assert!(err.contains("id"));
    }

    #[test]
    fn gaji_vendor_main_cli_strict_registry_invalid_index_json_fails() {
        let root = temp_dir("main_gaji_vendor_invalid_index_json");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-vendor-invalid-index".to_string()),
                index_root_hash: Some("sha256:index-main-vendor-invalid-index".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock");
        let index = root.join("registry.index.json");
        fs::write(&index, "{ invalid json").expect("write bad index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on invalid registry index json");
        assert_diag_with_fix(&err, "E_REG_INDEX_PARSE");
    }

    #[test]
    fn gaji_vendor_main_cli_strict_registry_enforces_trust_root() {
        let root = temp_dir("main_gaji_vendor_strict_trust_root");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-vendor-trust".to_string()),
                index_root_hash: Some("sha256:index-main-vendor-trust".to_string()),
                trust_root_hash: None,
                trust_root_source: None,
                audit_last_hash: None,
            },
        )
        .expect("lock with snapshot only");

        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-vendor-trust",
                "index_root_hash": "sha256:index-main-vendor-trust",
                "entries": [{
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "0.1.0",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("strict must require trust_root");
        assert_diag_with_fix(&err, "E_REG_TRUST_ROOT_INVALID");
    }

    #[test]
    fn gaji_vendor_main_cli_deny_yanked_locked_rejects() {
        let root = temp_dir("main_gaji_vendor_deny_yanked_locked");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock(&root, &lock).expect("lock");

        let mut lock_json: Value =
            serde_json::from_str(&fs::read_to_string(&lock).expect("read lock"))
                .expect("lock json");
        lock_json
            .get_mut("packages")
            .and_then(|v| v.as_array_mut())
            .and_then(|rows| rows.get_mut(0))
            .and_then(|row| row.as_object_mut())
            .expect("package row")
            .insert("yanked".to_string(), Value::Bool(true));
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json text"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--deny-yanked-locked".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("deny yanked should reject");
        assert_diag_with_fix(&err, "E_REG_YANKED_LOCKED");
    }

    #[test]
    fn gaji_vendor_main_cli_strict_registry_enforces_deny_yanked_locked() {
        let root = temp_dir("main_gaji_vendor_strict_yanked_locked");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-vendor-yanked".to_string()),
                index_root_hash: Some("sha256:index-main-vendor-yanked".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock with strict-ready meta");

        let mut lock_json: Value =
            serde_json::from_str(&fs::read_to_string(&lock).expect("read lock"))
                .expect("lock json");
        lock_json
            .get_mut("packages")
            .and_then(|v| v.as_array_mut())
            .and_then(|rows| rows.get_mut(0))
            .and_then(|row| row.as_object_mut())
            .expect("package row")
            .insert("yanked".to_string(), Value::Bool(true));
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json text"),
        )
        .expect("write lock");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-vendor-yanked",
            "sha256:index-main-vendor-yanked",
            "표준",
            "역학",
            "0.1.0",
        );

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--frozen-lockfile".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("strict must reject yanked lock row");
        assert_diag_with_fix(&err, "E_REG_YANKED_LOCKED");
    }

    #[test]
    fn gaji_vendor_main_cli_strict_registry_with_index_passes_without_verify_flag() {
        let root = temp_dir("main_gaji_vendor_strict_index_pass");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-vendor-strict-pass".to_string()),
                index_root_hash: Some("sha256:index-main-vendor-strict-pass".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock with strict-ready meta");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-vendor-strict-pass",
            "sha256:index-main-vendor-strict-pass",
            "표준",
            "역학",
            "0.1.0",
        );

        let out = root.join("vendor").join("gaji");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--frozen-lockfile".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("strict registry should pass without explicit verify flag");
        assert!(out.join("demo").join("gaji.toml").exists());
    }

    #[test]
    fn gaji_vendor_main_cli_strict_registry_auto_verifies_audit_when_log_given() {
        let root = temp_dir("main_gaji_vendor_strict_auto_audit_verify");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-vendor-strict-audit".to_string()),
                index_root_hash: Some("sha256:index-main-vendor-strict-audit".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: Some("blake3:strict-audit-pin".to_string()),
            },
        )
        .expect("lock with audit pin");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-vendor-strict-audit",
            "sha256:index-main-vendor-strict-audit",
            "표준",
            "역학",
            "0.1.0",
        );

        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);
        let mut rows = read_audit_rows(&audit);
        rows.last_mut()
            .and_then(|row| row.as_object_mut())
            .expect("last audit row")
            .insert(
                "row_hash".to_string(),
                Value::String("blake3:tampered".to_string()),
            );
        let tampered = rows
            .iter()
            .map(|row| serde_json::to_string(row).expect("row json"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&audit, format!("{tampered}\n")).expect("rewrite tampered audit");

        let out = root.join("vendor").join("gaji");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--strict-registry".to_string(),
            "--frozen-lockfile".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("strict should auto-verify audit and fail");
        assert_diag_with_fix(&err, "E_REG_AUDIT_ROW_HASH_MISMATCH");
    }

    #[test]
    fn gaji_vendor_main_cli_frozen_requires_snapshot_meta() {
        let root = temp_dir("main_gaji_vendor_frozen_requires_snapshot_meta");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock(&root, &lock).expect("lock without registry meta");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--frozen-lockfile".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("frozen should require snapshot meta");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISSING");
    }

    #[test]
    fn gaji_vendor_main_cli_verify_registry_writes_default_report_path() {
        let root = temp_dir("main_gaji_vendor_verify_registry_default_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");

        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-vendor-report".to_string()),
                index_root_hash: Some("sha256:index-main-vendor-report".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-vendor-report",
            "sha256:index-main-vendor-report",
            "표준",
            "역학",
            "0.1.0",
        );

        let out = root.join("vendor").join("gaji");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("vendor verify-registry pass");

        let report = root.join("vendor").join("registry.verify.json");
        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_eq!(
            report_json.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.verify_report.v1")
        );
        assert_eq!(
            report_json.get("packages").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(report_json.get("matched").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(
            report_json
                .get("duplicate_resolution_policy")
                .and_then(|v| v.as_str()),
            Some(crate::cli::gaji_registry::VERIFY_DUPLICATE_RESOLUTION_POLICY)
        );
    }

    #[test]
    fn gaji_vendor_main_cli_verify_registry_audit_writes_default_report_path() {
        let root = temp_dir("main_gaji_vendor_verify_registry_audit_default_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");

        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: None,
                index_root_hash: None,
                trust_root_hash: None,
                trust_root_source: None,
                audit_last_hash: None,
            },
        )
        .expect("lock");

        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let out = root.join("vendor").join("gaji");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-registry-audit".to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("vendor verify-registry-audit pass");

        let report = root.join("vendor").join("registry.audit.verify.json");
        assert!(report.exists());
    }

    #[test]
    fn gaji_vendor_main_cli_verify_registry_custom_out_writes_report_next_to_out() {
        let root = temp_dir("main_gaji_vendor_verify_registry_custom_out_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-vendor-verify-custom-out".to_string()),
                index_root_hash: Some("sha256:index-main-vendor-verify-custom-out".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-vendor-verify-custom-out",
            "sha256:index-main-vendor-verify-custom-out",
            "표준",
            "역학",
            "0.1.0",
        );
        let out = root.join("custom_vendor").join("bundle");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("vendor verify-registry with custom out");

        let report = root.join("custom_vendor").join("registry.verify.json");
        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_verify_report_contract(&report_json, 1, 1);
    }

    #[test]
    fn gaji_vendor_main_cli_verify_registry_audit_custom_out_writes_report_next_to_out() {
        let root = temp_dir("main_gaji_vendor_verify_registry_audit_custom_out_report");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock(&root, &lock).expect("lock");
        let audit = root.join("registry.audit.jsonl");
        let _ = write_audit_log_and_last_hash(&audit);
        let out = root.join("custom_vendor").join("bundle");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-registry-audit".to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("vendor verify-registry-audit with custom out");

        let report = root
            .join("custom_vendor")
            .join("registry.audit.verify.json");
        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_eq!(report_json.get("rows").and_then(|v| v.as_u64()), Some(1));
    }

    #[test]
    fn gaji_vendor_main_cli_verify_registry_requires_index() {
        let root = temp_dir("main_gaji_vendor_verify_registry_requires_index");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock(&root, &lock).expect("lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-registry".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("verify-registry requires index");
        assert_diag_with_fix(&err, "E_REG_VERIFY_INDEX_REQUIRED");
    }

    #[test]
    fn gaji_vendor_main_cli_verify_registry_snapshot_mismatch_fails() {
        let root = temp_dir("main_gaji_vendor_verify_registry_snapshot_mismatch");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-vendor-lock".to_string()),
                index_root_hash: Some("sha256:index-main-vendor".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-vendor-index",
            "sha256:index-main-vendor",
            "표준",
            "역학",
            "0.1.0",
        );

        let out = root.join("vendor").join("gaji");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("snapshot mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISMATCH");
    }

    #[test]
    fn gaji_vendor_main_cli_verify_registry_writes_report_json() {
        let root = temp_dir("main_gaji_vendor_verify_registry_report_json");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: Some("snap-main-vendor-report-json".to_string()),
                index_root_hash: Some("sha256:index-main-vendor-report-json".to_string()),
                trust_root_hash: Some("sha256:trust".to_string()),
                trust_root_source: Some("registry".to_string()),
                audit_last_hash: None,
            },
        )
        .expect("lock");

        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-vendor-report-json",
            "sha256:index-main-vendor-report-json",
            "표준",
            "역학",
            "0.1.0",
        );

        let out = root.join("vendor").join("gaji");
        let report_out = root.join("vendor").join("registry.verify.custom.json");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-registry".to_string(),
            "--registry-index".to_string(),
            index.to_string_lossy().to_string(),
            "--registry-verify-out".to_string(),
            report_out.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("vendor verify-registry with explicit report");

        let text = fs::read_to_string(&report_out).expect("read report");
        let report: Value = serde_json::from_str(&text).expect("parse report");
        assert_verify_report_contract(&report, 1, 1);
    }

    #[test]
    fn gaji_vendor_main_cli_verify_registry_audit_requires_log() {
        let root = temp_dir("main_gaji_vendor_verify_registry_audit_requires_log");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock(&root, &lock).expect("lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-registry-audit".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("verify-registry-audit requires log");
        assert_diag_with_fix(&err, "E_REG_AUDIT_VERIFY_LOG_REQUIRED");
    }

    #[test]
    fn gaji_vendor_main_cli_verify_registry_audit_writes_report_json() {
        let root = temp_dir("main_gaji_vendor_verify_registry_audit_report_json");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock(&root, &lock).expect("lock");

        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let out = root.join("vendor").join("gaji");
        let audit_out = root
            .join("vendor")
            .join("registry.audit.verify.custom.json");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-registry-audit".to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--registry-audit-verify-out".to_string(),
            audit_out.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("vendor verify-registry-audit with explicit report");

        let text = fs::read_to_string(&audit_out).expect("read audit report");
        let report: Value = serde_json::from_str(&text).expect("parse audit report");
        assert_audit_verify_report_contract(&report, 1);
    }

    #[test]
    fn gaji_vendor_main_cli_verify_registry_audit_expect_last_hash_mismatch_fails() {
        let root = temp_dir("main_gaji_vendor_verify_registry_audit_hash_mismatch");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock(&root, &lock).expect("lock");

        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let out = root.join("vendor").join("gaji");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-registry-audit".to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            "blake3:not-match".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on expected hash mismatch");
        assert_audit_last_hash_diag(&err);
    }

    #[test]
    fn gaji_vendor_main_cli_verify_registry_audit_uses_lock_last_hash_by_default() {
        let root = temp_dir("main_gaji_vendor_audit_uses_lock_last_hash");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock_with_options(
            &root,
            &lock,
            &cli::gaji::LockWriteOptions {
                snapshot_id: None,
                index_root_hash: None,
                trust_root_hash: None,
                trust_root_source: None,
                audit_last_hash: Some("blake3:not-match".to_string()),
            },
        )
        .expect("lock with audit hash");

        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let out = root.join("vendor").join("gaji");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-registry-audit".to_string(),
            "--registry-audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("lock last hash mismatch must fail");
        assert_audit_last_hash_diag(&err);
    }

    #[test]
    fn gaji_vendor_main_cli_index_keeps_registry_meta_fields() {
        let root = temp_dir("main_gaji_vendor_index_meta");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        cli::gaji::run_lock(&root, &lock).expect("lock");

        let mut lock_json: Value =
            serde_json::from_str(&fs::read_to_string(&lock).expect("read lock"))
                .expect("parse lock");
        let row = lock_json
            .get_mut("packages")
            .and_then(|v| v.as_array_mut())
            .and_then(|rows| rows.get_mut(0))
            .and_then(|v| v.as_object_mut())
            .expect("pkg row");
        row.insert(
            "archive_sha256".to_string(),
            Value::String("sha256:archive-a".to_string()),
        );
        row.insert(
            "download_url".to_string(),
            Value::String("https://registry/a".to_string()),
        );
        row.insert("dependencies".to_string(), json!({"표준/수학": "1.0.0"}));
        row.insert(
            "contract".to_string(),
            Value::String("D-STRICT".to_string()),
        );
        row.insert(
            "min_runtime".to_string(),
            Value::String("20.6.29".to_string()),
        );
        row.insert(
            "detmath_seal_hash".to_string(),
            Value::String("sha256:seal-a".to_string()),
        );
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let out = root.join("vendor").join("gaji");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("vendor should pass");

        let index: Value = serde_json::from_str(
            &fs::read_to_string(out.join("ddn.vendor.index.json")).expect("read vendor index"),
        )
        .expect("parse vendor index");
        let pkg = index
            .get("packages")
            .and_then(|v| v.as_array())
            .and_then(|rows| rows.first())
            .expect("pkg");
        assert_eq!(
            pkg.get("archive_sha256").and_then(|v| v.as_str()),
            Some("sha256:archive-a")
        );
        assert_eq!(
            pkg.get("download_url").and_then(|v| v.as_str()),
            Some("https://registry/a")
        );
        assert_eq!(
            pkg.get("dependencies")
                .and_then(|v| v.get("표준/수학"))
                .and_then(|v| v.as_str()),
            Some("1.0.0")
        );
        assert_eq!(
            pkg.get("contract").and_then(|v| v.as_str()),
            Some("D-STRICT")
        );
        assert_eq!(
            pkg.get("min_runtime").and_then(|v| v.as_str()),
            Some("20.6.29")
        );
        assert_eq!(
            pkg.get("detmath_seal_hash").and_then(|v| v.as_str()),
            Some("sha256:seal-a")
        );
    }

    #[test]
    fn gaji_vendor_main_cli_default_out_writes_vendor_index() {
        let root = temp_dir("main_gaji_vendor_default_out");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");

        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("vendor default out");

        let vendor_index = root
            .join("vendor")
            .join("gaji")
            .join("ddn.vendor.index.json");
        assert!(vendor_index.exists());
        let index_json: Value =
            serde_json::from_str(&fs::read_to_string(vendor_index).expect("read vendor index"))
                .expect("parse vendor index");
        assert_eq!(
            index_json.get("schema_version").and_then(|v| v.as_str()),
            Some("ddn.vendor.index.v1")
        );
    }

    #[test]
    fn gaji_vendor_main_cli_out_exists_as_file_fails_with_clean_error() {
        let root = temp_dir("main_gaji_vendor_out_file_clean_error");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let out = root.join("vendor").join("gaji");
        fs::create_dir_all(out.parent().expect("parent")).expect("mkdir parent");
        fs::write(&out, "not directory").expect("write out file");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("existing out file should fail");
        assert!(err.contains("E_GAJI_VENDOR_CLEAN"));
    }

    #[test]
    fn gaji_vendor_main_cli_out_parent_not_directory_fails_with_write_error() {
        let root = temp_dir("main_gaji_vendor_out_parent_file_write_error");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        let blocker = root.join("not_dir");
        fs::write(&blocker, "block").expect("write blocker file");
        let out = blocker.join("vendor");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("out parent file should fail");
        assert!(err.contains("E_GAJI_VENDOR_WRITE"));
    }

    #[test]
    fn gaji_vendor_main_cli_missing_vendor_source_fails() {
        let root = temp_dir("main_gaji_vendor_missing_source");
        let lock = root.join("ddn.lock");
        fs::write(
            &lock,
            serde_json::to_string_pretty(&json!({
                "schema_version": "v1",
                "packages": [{
                    "id": "표준/역학",
                    "version": "0.1.0",
                    "path": "missing",
                    "hash": "blake3:x",
                    "yanked": false
                }]
            }))
            .expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing vendor source must fail");
        assert!(err.contains("E_GAJI_VENDOR_SRC"));
    }

    #[test]
    fn gaji_vendor_main_cli_hash_mismatch_fails() {
        let root = temp_dir("main_gaji_vendor_hash_mismatch");
        write_demo_pkg(&root, "표준/역학", "0.1.0");
        let lock = root.join("ddn.lock");
        let lock_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "lock".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--out".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&lock_args).expect("prepare lock");

        fs::write(
            root.join("gaji").join("demo").join("main.ddn"),
            "값 <- 999.\n",
        )
        .expect("mutate source");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "vendor".to_string(),
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("hash mismatch must fail");
        assert!(err.contains("E_GAJI_VENDOR_HASH_MISMATCH"));
    }

    #[test]
    fn gaji_registry_main_cli_versions_missing_index_file_fails() {
        let root = temp_dir("main_gaji_registry_versions_missing_index");
        let index = root.join("missing.registry.index.json");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on missing index");
        assert_diag_with_fix(&err, "E_REG_INDEX_READ");
    }

    #[test]
    fn gaji_registry_main_cli_versions_executes() {
        let root = temp_dir("main_gaji_registry_versions_executes");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-versions",
            "sha256:index-main-reg-versions",
            "표준",
            "역학",
            "20.6.30",
        );
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        run_parsed_gaji(&args).expect("versions should pass");
    }

    #[test]
    fn gaji_registry_main_cli_versions_package_versions_schema_passes() {
        let root = temp_dir("main_gaji_registry_versions_package_versions_schema");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "scope": "표준",
                "name": "역학",
                "versions": [{
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        run_parsed_gaji(&args).expect("package_versions schema should pass");
    }

    #[test]
    fn gaji_registry_main_cli_versions_package_versions_yanked_only_requires_include_yanked() {
        let root = temp_dir("main_gaji_registry_versions_package_versions_yanked_only");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "scope": "표준",
                "name": "역학",
                "versions": [{
                    "version": "20.6.30",
                    "yanked": true
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let no_include_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&no_include_args)
            .expect_err("yanked-only package_versions should fail by default");
        assert_diag_with_fix(&err, "E_REG_INDEX_NOT_FOUND");

        let include_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--include-yanked".to_string(),
        ];
        run_parsed_gaji(&include_args).expect("include-yanked should allow yanked-only versions");
    }

    #[test]
    fn gaji_registry_main_cli_versions_package_versions_missing_versions_array_fails() {
        let root = temp_dir("main_gaji_registry_versions_package_versions_missing_versions");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "scope": "표준",
                "name": "역학"
            }))
            .expect("index json"),
        )
        .expect("write index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing versions array must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_SCHEMA");
    }

    #[test]
    fn gaji_registry_main_cli_versions_package_versions_missing_scope_fails() {
        let root = temp_dir("main_gaji_registry_versions_package_versions_missing_scope");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "name": "역학",
                "versions": [{
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing scope must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("scope"));
    }

    #[test]
    fn gaji_registry_main_cli_versions_package_versions_missing_name_fails() {
        let root = temp_dir("main_gaji_registry_versions_package_versions_missing_name");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "scope": "표준",
                "versions": [{
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing name must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("name"));
    }

    #[test]
    fn gaji_registry_main_cli_versions_package_versions_missing_version_field_fails() {
        let root = temp_dir("main_gaji_registry_versions_package_versions_missing_version");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "scope": "표준",
                "name": "역학",
                "versions": [{
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing version field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("version"));
    }

    #[test]
    fn gaji_registry_main_cli_versions_index_entry_missing_scope_field_fails() {
        let root = temp_dir("main_gaji_registry_versions_index_entry_missing_scope");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing scope field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("scope"));
    }

    #[test]
    fn gaji_registry_main_cli_versions_index_entry_missing_name_field_fails() {
        let root = temp_dir("main_gaji_registry_versions_index_entry_missing_name");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "version": "20.6.30",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing name field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("name"));
    }

    #[test]
    fn gaji_registry_main_cli_versions_index_entry_missing_version_field_fails() {
        let root = temp_dir("main_gaji_registry_versions_index_entry_missing_version");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing version field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("version"));
    }

    #[test]
    fn gaji_registry_main_cli_versions_index_schema_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_versions_index_schema_mismatch");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.unknown.v1"
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on index schema mismatch");
        assert_diag_with_fix(&err, "E_REG_INDEX_SCHEMA");
    }

    #[test]
    fn gaji_registry_main_cli_versions_invalid_index_json_fails() {
        let root = temp_dir("main_gaji_registry_versions_invalid_index_json");
        let index = root.join("registry.index.json");
        fs::write(&index, "{ invalid json").expect("write bad index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on invalid index json");
        assert_diag_with_fix(&err, "E_REG_INDEX_PARSE");
    }

    #[test]
    fn gaji_registry_main_cli_versions_guard_with_snapshot_and_trust_passes() {
        let root = temp_dir("main_gaji_registry_versions_guard_ok");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-guard-ok",
            "sha256:index-main-reg-guard-ok",
            "표준",
            "역학",
            "20.6.30",
        );
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--expect-snapshot-id".to_string(),
            "snap-main-reg-guard-ok".to_string(),
            "--expect-index-root-hash".to_string(),
            "sha256:index-main-reg-guard-ok".to_string(),
            "--require-trust-root".to_string(),
            "--expect-trust-root-hash".to_string(),
            "sha256:trust".to_string(),
        ];
        run_parsed_gaji(&args).expect("guard pass");
    }

    #[test]
    fn gaji_registry_main_cli_versions_guard_from_lock_passes() {
        let root = temp_dir("main_gaji_registry_versions_guard_lock_ok");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-lock-ok",
            "sha256:index-main-reg-lock-ok",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-lock-ok",
                "index_root_hash": "sha256:index-main-reg-lock-ok"
            },
            "trust_root": {
                "hash": "sha256:trust",
                "source": "registry"
            },
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--require-trust-root".to_string(),
        ];
        run_parsed_gaji(&args).expect("lock guard pass");
    }

    #[test]
    fn gaji_registry_main_cli_versions_frozen_requires_snapshot_meta() {
        let root = temp_dir("main_gaji_registry_versions_frozen");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "entries": [{
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--frozen-lockfile".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail without snapshot pins");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISSING");
    }

    #[test]
    fn gaji_registry_main_cli_versions_guard_index_hash_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_versions_guard_bad_hash");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-guard-bad-hash",
            "sha256:index-main-reg-guard-bad-hash",
            "표준",
            "역학",
            "20.6.30",
        );
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--expect-index-root-hash".to_string(),
            "sha256:not-match".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_ROOT_HASH_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_versions_guard_from_lock_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_versions_guard_lock_mismatch");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-lock-good",
            "sha256:index-main-reg-lock-good",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-lock-bad",
                "index_root_hash": "sha256:index-main-reg-lock-good"
            },
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("lock snapshot mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_versions_frozen_lock_requires_snapshot_meta_in_lock() {
        let root = temp_dir("main_gaji_registry_versions_frozen_lock_missing_meta");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-lock-frozen",
            "sha256:index-main-reg-lock-frozen",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--frozen-lockfile".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("frozen lock must require snapshot pins");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISSING");
    }

    #[test]
    fn gaji_registry_main_cli_versions_expect_trust_root_hash_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_versions_expect_trust_root_hash_mismatch");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-trust-mismatch",
            "sha256:index-main-reg-trust-mismatch",
            "표준",
            "역학",
            "20.6.30",
        );

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--expect-trust-root-hash".to_string(),
            "sha256:not-match".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("trust_root hash mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_TRUST_ROOT_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_versions_require_trust_root_missing_fails() {
        let root = temp_dir("main_gaji_registry_versions_require_trust_root_missing");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-versions-no-trust",
                "index_root_hash": "sha256:index-main-reg-versions-no-trust",
                "entries": [{
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--require-trust-root".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail when trust_root is missing");
        assert_diag_with_fix(&err, "E_REG_TRUST_ROOT_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_versions_expect_snapshot_id_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_versions_expect_snapshot_id_mismatch");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-versions-snapshot-good",
            "sha256:index-main-reg-versions-snapshot-good",
            "표준",
            "역학",
            "20.6.30",
        );

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--expect-snapshot-id".to_string(),
            "snap-main-reg-versions-snapshot-bad".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("snapshot mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_versions_include_yanked_allows_yanked_only_entry() {
        let root = temp_dir("main_gaji_registry_versions_include_yanked_toggle");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-versions-yanked-only",
                "index_root_hash": "sha256:index-main-reg-versions-yanked-only",
                "entries": [{
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "yanked": true
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let no_include_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
        ];
        let err =
            run_parsed_gaji(&no_include_args).expect_err("yanked-only should fail by default");
        assert_diag_with_fix(&err, "E_REG_INDEX_NOT_FOUND");

        let include_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "versions".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--include-yanked".to_string(),
        ];
        run_parsed_gaji(&include_args).expect("include-yanked should allow yanked-only entry");
    }

    #[test]
    fn gaji_registry_main_cli_entry_missing_index_file_fails() {
        let root = temp_dir("main_gaji_registry_entry_missing_index");
        let index = root.join("missing.registry.index.json");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on missing index");
        assert_diag_with_fix(&err, "E_REG_INDEX_READ");
    }

    #[test]
    fn gaji_registry_main_cli_entry_missing_version_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_entry_missing_version");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-entry-missing",
            "sha256:index-main-reg-entry-missing",
            "표준",
            "역학",
            "20.6.30",
        );
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "99.9.99".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing entry must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_NOT_FOUND");
    }

    #[test]
    fn gaji_registry_main_cli_entry_invalid_index_json_fails() {
        let root = temp_dir("main_gaji_registry_entry_invalid_index_json");
        let index = root.join("registry.index.json");
        fs::write(&index, "{ invalid json").expect("write bad index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on invalid index json");
        assert_diag_with_fix(&err, "E_REG_INDEX_PARSE");
    }

    #[test]
    fn gaji_registry_main_cli_entry_index_schema_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_entry_index_schema_mismatch");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v0",
                "snapshot_id": "snap-main-reg-entry-schema-bad",
                "index_root_hash": "sha256:index-main-reg-entry-schema-bad"
            }))
            .expect("index json"),
        )
        .expect("write index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("index schema mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_SCHEMA");
    }

    #[test]
    fn gaji_registry_main_cli_entry_index_entry_schema_passes() {
        let root = temp_dir("main_gaji_registry_entry_index_entry_schema");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
        ];
        run_parsed_gaji(&args).expect("index_entry schema should pass");
    }

    #[test]
    fn gaji_registry_main_cli_entry_package_versions_schema_passes() {
        let root = temp_dir("main_gaji_registry_entry_package_versions_schema");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "scope": "표준",
                "name": "역학",
                "versions": [{
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
        ];
        run_parsed_gaji(&args).expect("package_versions schema entry should pass");
    }

    #[test]
    fn gaji_registry_main_cli_entry_package_versions_missing_scope_fails() {
        let root = temp_dir("main_gaji_registry_entry_package_versions_missing_scope");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "name": "역학",
                "versions": [{
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing scope must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("scope"));
    }

    #[test]
    fn gaji_registry_main_cli_entry_package_versions_missing_name_fails() {
        let root = temp_dir("main_gaji_registry_entry_package_versions_missing_name");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "scope": "표준",
                "versions": [{
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing name must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("name"));
    }

    #[test]
    fn gaji_registry_main_cli_entry_package_versions_missing_version_field_fails() {
        let root = temp_dir("main_gaji_registry_entry_package_versions_missing_version");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "scope": "표준",
                "name": "역학",
                "versions": [{
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing version field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("version"));
    }

    #[test]
    fn gaji_registry_main_cli_entry_index_entry_missing_scope_field_fails() {
        let root = temp_dir("main_gaji_registry_entry_index_entry_missing_scope");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing scope field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("scope"));
    }

    #[test]
    fn gaji_registry_main_cli_entry_index_entry_missing_name_field_fails() {
        let root = temp_dir("main_gaji_registry_entry_index_entry_missing_name");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "version": "20.6.30",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing name field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("name"));
    }

    #[test]
    fn gaji_registry_main_cli_entry_index_entry_missing_version_field_fails() {
        let root = temp_dir("main_gaji_registry_entry_index_entry_missing_version");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing version field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("version"));
    }

    #[test]
    fn gaji_registry_main_cli_entry_guard_from_lock_passes() {
        let root = temp_dir("main_gaji_registry_entry_guard_lock_ok");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-entry-lock-ok",
            "sha256:index-main-reg-entry-lock-ok",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-entry-lock-ok",
                "index_root_hash": "sha256:index-main-reg-entry-lock-ok"
            },
            "trust_root": {
                "hash": "sha256:trust",
                "source": "registry"
            },
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--require-trust-root".to_string(),
        ];
        run_parsed_gaji(&args).expect("entry lock guard pass");
    }

    #[test]
    fn gaji_registry_main_cli_entry_guard_from_lock_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_entry_guard_lock_mismatch");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-entry-lock-good",
            "sha256:index-main-reg-entry-lock-good",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-entry-lock-bad",
                "index_root_hash": "sha256:index-main-reg-entry-lock-good"
            },
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("entry lock snapshot mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_entry_frozen_lock_requires_snapshot_meta_in_lock() {
        let root = temp_dir("main_gaji_registry_entry_frozen_lock_missing_meta");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-entry-frozen",
            "sha256:index-main-reg-entry-frozen",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--frozen-lockfile".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("frozen lock must require snapshot pins");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISSING");
    }

    #[test]
    fn gaji_registry_main_cli_entry_require_trust_root_missing_fails() {
        let root = temp_dir("main_gaji_registry_entry_require_trust_root_missing");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-entry-no-trust",
                "index_root_hash": "sha256:index-main-reg-entry-no-trust",
                "entries": [{
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--require-trust-root".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail when trust_root is missing");
        assert_diag_with_fix(&err, "E_REG_TRUST_ROOT_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_entry_expect_trust_root_hash_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_entry_expect_trust_root_hash_mismatch");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-entry-trust-mismatch",
            "sha256:index-main-reg-entry-trust-mismatch",
            "표준",
            "역학",
            "20.6.30",
        );

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--expect-trust-root-hash".to_string(),
            "sha256:not-match".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("trust_root hash mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_TRUST_ROOT_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_entry_expect_snapshot_id_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_entry_expect_snapshot_id_mismatch");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-entry-snapshot-good",
            "sha256:index-main-reg-entry-snapshot-good",
            "표준",
            "역학",
            "20.6.30",
        );

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "entry".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--expect-snapshot-id".to_string(),
            "snap-main-reg-entry-snapshot-bad".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("snapshot mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_search_empty_query_fails() {
        let root = temp_dir("main_gaji_registry_search_empty_query");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-search",
            "sha256:index-main-reg-search",
            "표준",
            "역학",
            "20.6.30",
        );

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "   ".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on empty query");
        assert_diag_with_fix(&err, "E_REG_SEARCH_QUERY");
    }

    #[test]
    fn gaji_registry_main_cli_search_missing_index_file_fails() {
        let root = temp_dir("main_gaji_registry_search_missing_index");
        let index = root.join("missing.registry.index.json");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on missing index");
        assert_diag_with_fix(&err, "E_REG_INDEX_READ");
    }

    #[test]
    fn gaji_registry_main_cli_search_invalid_index_json_fails() {
        let root = temp_dir("main_gaji_registry_search_invalid_index_json");
        let index = root.join("registry.index.json");
        fs::write(&index, "{ invalid json").expect("write bad index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on invalid index json");
        assert_diag_with_fix(&err, "E_REG_INDEX_PARSE");
    }

    #[test]
    fn gaji_registry_main_cli_search_package_versions_schema_passes() {
        let root = temp_dir("main_gaji_registry_search_package_versions_schema");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "scope": "표준",
                "name": "역학",
                "versions": [{
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
        ];
        run_parsed_gaji(&args).expect("package_versions schema search should pass");
    }

    #[test]
    fn gaji_registry_main_cli_search_index_entry_schema_passes() {
        let root = temp_dir("main_gaji_registry_search_index_entry_schema");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
        ];
        run_parsed_gaji(&args).expect("index_entry schema search should pass");
    }

    #[test]
    fn gaji_registry_main_cli_search_index_schema_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_search_index_schema_mismatch");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.unknown.v1"
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on index schema mismatch");
        assert_diag_with_fix(&err, "E_REG_INDEX_SCHEMA");
    }

    #[test]
    fn gaji_registry_main_cli_search_package_versions_missing_scope_fails() {
        let root = temp_dir("main_gaji_registry_search_package_versions_missing_scope");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "name": "역학",
                "versions": [{
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing scope must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("scope"));
    }

    #[test]
    fn gaji_registry_main_cli_search_package_versions_missing_name_fails() {
        let root = temp_dir("main_gaji_registry_search_package_versions_missing_name");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "scope": "표준",
                "versions": [{
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing name must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("name"));
    }

    #[test]
    fn gaji_registry_main_cli_search_package_versions_missing_version_field_fails() {
        let root = temp_dir("main_gaji_registry_search_package_versions_missing_version");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "scope": "표준",
                "name": "역학",
                "versions": [{
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing version field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("version"));
    }

    #[test]
    fn gaji_registry_main_cli_search_index_entry_missing_scope_field_fails() {
        let root = temp_dir("main_gaji_registry_search_index_entry_missing_scope");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing scope field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("scope"));
    }

    #[test]
    fn gaji_registry_main_cli_search_index_entry_missing_name_field_fails() {
        let root = temp_dir("main_gaji_registry_search_index_entry_missing_name");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "version": "20.6.30",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing name field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("name"));
    }

    #[test]
    fn gaji_registry_main_cli_search_index_entry_missing_version_field_fails() {
        let root = temp_dir("main_gaji_registry_search_index_entry_missing_version");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing version field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("version"));
    }

    #[test]
    fn gaji_registry_main_cli_search_guard_from_lock_passes() {
        let root = temp_dir("main_gaji_registry_search_guard_lock_ok");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-search-lock-ok",
            "sha256:index-main-reg-search-lock-ok",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-search-lock-ok",
                "index_root_hash": "sha256:index-main-reg-search-lock-ok"
            },
            "trust_root": {
                "hash": "sha256:trust",
                "source": "registry"
            },
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
            "--require-trust-root".to_string(),
        ];
        run_parsed_gaji(&args).expect("search lock guard pass");
    }

    #[test]
    fn gaji_registry_main_cli_search_guard_from_lock_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_search_guard_lock_mismatch");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-search-lock-good",
            "sha256:index-main-reg-search-lock-good",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-search-lock-good",
                "index_root_hash": "sha256:index-main-reg-search-lock-bad"
            },
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("search lock hash mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_ROOT_HASH_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_search_require_trust_root_missing_fails() {
        let root = temp_dir("main_gaji_registry_search_require_trust_root_missing");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-search-no-trust",
                "index_root_hash": "sha256:index-main-reg-search-no-trust",
                "entries": [{
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
            "--require-trust-root".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail when trust_root is missing");
        assert_diag_with_fix(&err, "E_REG_TRUST_ROOT_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_search_expect_trust_root_hash_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_search_expect_trust_root_hash_mismatch");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-search-trust-mismatch",
            "sha256:index-main-reg-search-trust-mismatch",
            "표준",
            "역학",
            "20.6.30",
        );

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
            "--expect-trust-root-hash".to_string(),
            "sha256:not-match".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("trust_root hash mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_TRUST_ROOT_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_search_expect_snapshot_id_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_search_expect_snapshot_id_mismatch");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-search-snapshot-good",
            "sha256:index-main-reg-search-snapshot-good",
            "표준",
            "역학",
            "20.6.30",
        );

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "search".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--query".to_string(),
            "역학".to_string(),
            "--expect-snapshot-id".to_string(),
            "snap-main-reg-search-snapshot-bad".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("snapshot mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_publish_missing_auth_policy_fails() {
        let root = temp_dir("main_gaji_registry_publish_missing_auth_policy");
        let index = root.join("registry.index.json");
        let policy = root.join("missing.auth_policy.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on missing auth policy");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_READ");
    }

    #[test]
    fn gaji_registry_main_cli_publish_and_yank_executes() {
        let root = temp_dir("main_gaji_registry_publish_and_yank");
        let index = root.join("registry.index.json");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        run_parsed_gaji(&yank_args).expect("yank");
    }

    #[test]
    fn gaji_registry_main_cli_publish_with_at_preserves_trust_root_and_sets_published_at() {
        let root = temp_dir("main_gaji_registry_publish_with_at_meta");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-old",
                "index_root_hash": "sha256:old-index",
                "trust_root": {
                    "hash": "sha256:trust-mirror",
                    "source": "mirror"
                },
                "entries": [{
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "수학",
                    "version": "1.0.0",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let publish_ts = "2026-02-20T12:34:56Z";
        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
            "--at".to_string(),
            publish_ts.to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish with --at");

        let snapshot: Value =
            serde_json::from_str(&fs::read_to_string(&index).expect("read index"))
                .expect("parse index");
        assert_eq!(
            snapshot.get("snapshot_id").and_then(|v| v.as_str()),
            Some(publish_ts)
        );
        assert_eq!(
            snapshot
                .get("trust_root")
                .and_then(|v| v.get("hash"))
                .and_then(|v| v.as_str()),
            Some("sha256:trust-mirror")
        );
        assert_eq!(
            snapshot
                .get("trust_root")
                .and_then(|v| v.get("source"))
                .and_then(|v| v.as_str()),
            Some("mirror")
        );
        assert!(snapshot
            .get("index_root_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .starts_with("blake3:"));

        let entries = snapshot
            .get("entries")
            .and_then(|v| v.as_array())
            .expect("entries");
        let published = entries
            .iter()
            .find(|row| {
                row.get("scope").and_then(|v| v.as_str()) == Some("표준")
                    && row.get("name").and_then(|v| v.as_str()) == Some("역학")
                    && row.get("version").and_then(|v| v.as_str()) == Some("20.6.30")
            })
            .expect("published entry");
        assert_eq!(
            published.get("published_at").and_then(|v| v.as_str()),
            Some(publish_ts)
        );
        assert_eq!(
            published.get("yanked").and_then(|v| v.as_bool()),
            Some(false)
        );
    }

    #[test]
    fn gaji_registry_main_cli_publish_writes_optional_metadata_fields() {
        let root = temp_dir("main_gaji_registry_publish_optional_metadata");
        let index = root.join("registry.index.json");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:archive-a".to_string(),
            "--contract".to_string(),
            "D-STRICT".to_string(),
            "--detmath-seal-hash".to_string(),
            "sha256:seal-a".to_string(),
            "--min-runtime".to_string(),
            "20.6.29".to_string(),
            "--download-url".to_string(),
            "https://registry.example/physics-20.6.30.tgz".to_string(),
            "--summary".to_string(),
            "역학 요약".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
            "--at".to_string(),
            "2026-02-20T12:00:00Z".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish with optional metadata");

        let snapshot: Value =
            serde_json::from_str(&fs::read_to_string(&index).expect("read index"))
                .expect("parse index");
        let entries = snapshot
            .get("entries")
            .and_then(|v| v.as_array())
            .expect("entries");
        let target = entries
            .iter()
            .find(|row| {
                row.get("scope").and_then(|v| v.as_str()) == Some("표준")
                    && row.get("name").and_then(|v| v.as_str()) == Some("역학")
                    && row.get("version").and_then(|v| v.as_str()) == Some("20.6.30")
            })
            .expect("target entry");
        assert_eq!(
            target.get("archive_sha256").and_then(|v| v.as_str()),
            Some("sha256:archive-a")
        );
        assert_eq!(
            target.get("contract").and_then(|v| v.as_str()),
            Some("D-STRICT")
        );
        assert_eq!(
            target.get("detmath_seal_hash").and_then(|v| v.as_str()),
            Some("sha256:seal-a")
        );
        assert_eq!(
            target.get("min_runtime").and_then(|v| v.as_str()),
            Some("20.6.29")
        );
        assert_eq!(
            target.get("download_url").and_then(|v| v.as_str()),
            Some("https://registry.example/physics-20.6.30.tgz")
        );
        assert_eq!(
            target.get("summary").and_then(|v| v.as_str()),
            Some("역학 요약")
        );
        assert_eq!(
            target
                .get("dependencies")
                .and_then(|v| v.as_object())
                .map(|m| m.len()),
            Some(0)
        );
        assert_eq!(target.get("yanked").and_then(|v| v.as_bool()), Some(false));
    }

    #[test]
    fn gaji_registry_main_cli_yank_preserves_optional_metadata_fields() {
        let root = temp_dir("main_gaji_registry_yank_preserve_optional_metadata");
        let index = root.join("registry.index.json");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:archive-a".to_string(),
            "--contract".to_string(),
            "D-STRICT".to_string(),
            "--detmath-seal-hash".to_string(),
            "sha256:seal-a".to_string(),
            "--min-runtime".to_string(),
            "20.6.29".to_string(),
            "--download-url".to_string(),
            "https://registry.example/physics-20.6.30.tgz".to_string(),
            "--summary".to_string(),
            "역학 요약".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
            "--at".to_string(),
            "2026-02-20T12:00:00Z".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish with optional metadata");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
            "--at".to_string(),
            "2026-02-20T13:00:00Z".to_string(),
        ];
        run_parsed_gaji(&yank_args).expect("yank");

        let snapshot: Value =
            serde_json::from_str(&fs::read_to_string(&index).expect("read index"))
                .expect("parse index");
        let entries = snapshot
            .get("entries")
            .and_then(|v| v.as_array())
            .expect("entries");
        let target = entries
            .iter()
            .find(|row| {
                row.get("scope").and_then(|v| v.as_str()) == Some("표준")
                    && row.get("name").and_then(|v| v.as_str()) == Some("역학")
                    && row.get("version").and_then(|v| v.as_str()) == Some("20.6.30")
            })
            .expect("target entry");
        assert_eq!(
            target.get("archive_sha256").and_then(|v| v.as_str()),
            Some("sha256:archive-a")
        );
        assert_eq!(
            target.get("contract").and_then(|v| v.as_str()),
            Some("D-STRICT")
        );
        assert_eq!(
            target.get("detmath_seal_hash").and_then(|v| v.as_str()),
            Some("sha256:seal-a")
        );
        assert_eq!(
            target.get("min_runtime").and_then(|v| v.as_str()),
            Some("20.6.29")
        );
        assert_eq!(
            target.get("download_url").and_then(|v| v.as_str()),
            Some("https://registry.example/physics-20.6.30.tgz")
        );
        assert_eq!(
            target.get("summary").and_then(|v| v.as_str()),
            Some("역학 요약")
        );
        assert_eq!(
            target
                .get("dependencies")
                .and_then(|v| v.as_object())
                .map(|m| m.len()),
            Some(0)
        );
        assert_eq!(target.get("yanked").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            target.get("yank_reason_code").and_then(|v| v.as_str()),
            Some("policy")
        );
    }

    #[test]
    fn gaji_registry_main_cli_publish_yank_with_at_writes_index_and_audit_timestamps() {
        let root = temp_dir("main_gaji_registry_publish_yank_with_at_timestamps");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-old",
                "index_root_hash": "sha256:old-index",
                "trust_root": {
                    "hash": "sha256:trust",
                    "source": "registry"
                },
                "entries": []
            }))
            .expect("index json"),
        )
        .expect("write index");
        let audit = root.join("registry.audit.jsonl");

        let publish_ts = "2026-02-20T10:00:00Z";
        let yank_ts = "2026-02-20T11:00:00Z";
        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
            "--at".to_string(),
            publish_ts.to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish with audit");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--note".to_string(),
            "safety".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
            "--at".to_string(),
            yank_ts.to_string(),
        ];
        run_parsed_gaji(&yank_args).expect("yank with audit");

        let snapshot: Value =
            serde_json::from_str(&fs::read_to_string(&index).expect("read index"))
                .expect("parse index");
        assert_eq!(
            snapshot.get("snapshot_id").and_then(|v| v.as_str()),
            Some(yank_ts)
        );
        let entries = snapshot
            .get("entries")
            .and_then(|v| v.as_array())
            .expect("entries");
        let target = entries
            .iter()
            .find(|row| {
                row.get("scope").and_then(|v| v.as_str()) == Some("표준")
                    && row.get("name").and_then(|v| v.as_str()) == Some("역학")
                    && row.get("version").and_then(|v| v.as_str()) == Some("20.6.30")
            })
            .expect("yanked entry");
        assert_eq!(
            target.get("published_at").and_then(|v| v.as_str()),
            Some(publish_ts)
        );
        assert_eq!(target.get("yanked").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            target.get("yanked_at").and_then(|v| v.as_str()),
            Some(yank_ts)
        );
        assert_eq!(
            target.get("yank_reason_code").and_then(|v| v.as_str()),
            Some("policy")
        );
        assert_eq!(
            target.get("yank_note").and_then(|v| v.as_str()),
            Some("safety")
        );

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("action"))
                .and_then(|v| v.as_str()),
            Some("publish")
        );
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("ts"))
                .and_then(|v| v.as_str()),
            Some(publish_ts)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("action"))
                .and_then(|v| v.as_str()),
            Some("yank")
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("ts"))
                .and_then(|v| v.as_str()),
            Some(yank_ts)
        );
    }

    #[test]
    fn gaji_registry_main_cli_publish_writes_default_audit_log() {
        let root = temp_dir("main_gaji_registry_publish_default_audit");
        let index = root.join("registry.index.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&args).expect("publish without explicit audit-log");

        let audit = index.with_extension("audit.jsonl");
        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("action"))
                .and_then(|v| v.as_str()),
            Some("publish")
        );
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn gaji_registry_main_cli_yank_writes_default_audit_log() {
        let root = temp_dir("main_gaji_registry_yank_default_audit");
        let index = root.join("registry.index.json");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        run_parsed_gaji(&yank_args).expect("yank without explicit audit-log");

        let audit = index.with_extension("audit.jsonl");
        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("action"))
                .and_then(|v| v.as_str()),
            Some("publish")
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("action"))
                .and_then(|v| v.as_str()),
            Some("yank")
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn gaji_registry_main_cli_yank_missing_entry_writes_default_audit_denied() {
        let root = temp_dir("main_gaji_registry_yank_missing_entry_default_audit");
        let index = root.join("registry.index.json");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "99.9.99".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_parsed_gaji(&yank_args).expect_err("missing yank target must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_NOT_FOUND");

        let audit = index.with_extension("audit.jsonl");
        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("action"))
                .and_then(|v| v.as_str()),
            Some("yank")
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_INDEX_NOT_FOUND")
        );
    }

    #[test]
    fn gaji_registry_main_cli_publish_missing_auth_policy_writes_audit_denied() {
        let root = temp_dir("main_gaji_registry_publish_missing_auth_policy_audit");
        let index = root.join("registry.index.json");
        let audit = root.join("registry.audit.jsonl");
        let policy = root.join("missing.auth_policy.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on missing auth policy");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_READ");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_POLICY_READ")
        );
    }

    #[test]
    fn gaji_registry_main_cli_publish_immutable_exists_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_publish_immutable_exists");
        let index = root.join("registry.index.json");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("first publish");
        let err = run_parsed_gaji(&publish_args).expect_err("duplicate publish must fail");
        assert_diag_with_fix(&err, "E_REG_IMMUTABLE_EXISTS");
    }

    #[test]
    fn gaji_registry_main_cli_publish_immutable_exists_writes_audit_denied() {
        let root = temp_dir("main_gaji_registry_publish_immutable_exists_audit");
        let index = root.join("registry.index.json");
        let audit = root.join("registry.audit.jsonl");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("first publish");

        let err = run_parsed_gaji(&publish_args).expect_err("duplicate publish must fail");
        assert_diag_with_fix(&err, "E_REG_IMMUTABLE_EXISTS");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_IMMUTABLE_EXISTS")
        );
    }

    #[test]
    fn gaji_registry_main_cli_publish_role_forbidden_writes_audit_denied() {
        let root = temp_dir("main_gaji_registry_publish_role_denied_audit");
        let index = root.join("registry.index.json");
        let audit = root.join("registry.audit.jsonl");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "viewer".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("role forbidden");
        assert_diag_with_fix(&err, "E_REG_SCOPE_FORBIDDEN");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_SCOPE_FORBIDDEN")
        );
    }

    #[test]
    fn gaji_registry_main_cli_publish_auth_policy_invalid_json_fails() {
        let root = temp_dir("main_gaji_registry_publish_auth_policy_invalid_json");
        let index = root.join("registry.index.json");
        let policy = root.join("auth_policy.json");
        fs::write(&policy, "{ invalid json").expect("write bad auth policy");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("invalid auth policy json must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_PARSE");
    }

    #[test]
    fn gaji_registry_main_cli_publish_auth_policy_token_hash_passes() {
        let root = temp_dir("main_gaji_registry_publish_auth_hash");
        let index = root.join("registry.index.json");
        let policy = root.join("auth_policy.json");
        let token = "token_hash_only";
        let token_hash = format!("blake3:{}", blake3::hash(token.as_bytes()).to_hex());
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v1",
                "tokens": [{
                    "token_hash": token_hash,
                    "roles": ["publisher"],
                    "scopes": ["표준"]
                }]
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            token.to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&args).expect("publish with token_hash policy");
    }

    #[test]
    fn gaji_registry_main_cli_publish_auth_policy_unknown_token_fails_and_audits() {
        let root = temp_dir("main_gaji_registry_publish_auth_unknown");
        let index = root.join("registry.index.json");
        let audit = root.join("registry.audit.jsonl");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v1",
                "tokens": [{
                    "token": "token1",
                    "role": "publisher",
                    "scopes": ["표준"]
                }]
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "unknown".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must reject unknown token");
        assert_diag_with_fix(&err, "E_REG_AUTH_TOKEN_UNKNOWN");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_TOKEN_UNKNOWN")
        );
    }

    #[test]
    fn gaji_registry_main_cli_yank_invalid_auth_policy_json_fails() {
        let root = temp_dir("main_gaji_registry_yank_invalid_auth_policy_json");
        let index = root.join("registry.index.json");
        let policy = root.join("auth_policy.json");
        fs::write(&policy, "{ invalid json").expect("write bad auth policy");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on invalid auth policy json");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_PARSE");
    }

    #[test]
    fn gaji_registry_main_cli_yank_invalid_auth_policy_json_writes_audit_denied() {
        let root = temp_dir("main_gaji_registry_yank_invalid_auth_policy_json_audit");
        let index = root.join("registry.index.json");
        let audit = root.join("registry.audit.jsonl");
        let policy = root.join("auth_policy.json");
        fs::write(&policy, "{ invalid json").expect("write bad auth policy");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_parsed_gaji(&yank_args).expect_err("must fail on invalid auth policy json");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_PARSE");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_POLICY_PARSE")
        );
    }

    #[test]
    fn gaji_registry_main_cli_yank_missing_entry_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_yank_missing_entry");
        let index = root.join("registry.index.json");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "99.9.99".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_parsed_gaji(&yank_args).expect_err("missing yank target must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_NOT_FOUND");
    }

    #[test]
    fn gaji_registry_main_cli_yank_missing_entry_writes_audit_denied() {
        let root = temp_dir("main_gaji_registry_yank_missing_entry_audit");
        let index = root.join("registry.index.json");
        let audit = root.join("registry.audit.jsonl");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "99.9.99".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_parsed_gaji(&yank_args).expect_err("missing yank target must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_NOT_FOUND");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_INDEX_NOT_FOUND")
        );
    }

    #[test]
    fn gaji_registry_main_cli_yank_missing_auth_policy_writes_audit_denied() {
        let root = temp_dir("main_gaji_registry_yank_missing_auth_policy_audit");
        let index = root.join("registry.index.json");
        let audit = root.join("registry.audit.jsonl");
        let policy = root.join("missing.auth_policy.json");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_parsed_gaji(&yank_args).expect_err("must fail on missing auth policy");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_READ");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_POLICY_READ")
        );
    }

    #[test]
    fn gaji_registry_main_cli_yank_missing_token_writes_audit_denied() {
        let root = temp_dir("main_gaji_registry_yank_token_denied_audit");
        let index = root.join("registry.index.json");
        let audit = root.join("registry.audit.jsonl");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token_ok".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_parsed_gaji(&yank_args).expect_err("missing token");
        assert_diag_with_fix(&err, "E_REG_AUTH_REQUIRED");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_REQUIRED")
        );
    }

    #[test]
    fn gaji_registry_main_cli_yank_auth_policy_scope_forbidden_fails() {
        let root = temp_dir("main_gaji_registry_yank_auth_scope");
        let index = root.join("registry.index.json");
        let audit = root.join("registry.audit.jsonl");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v1",
                "tokens": [
                    {
                        "token": "token1",
                        "role": "publisher",
                        "scopes": ["표준"]
                    },
                    {
                        "token": "token2",
                        "role": "scope_admin",
                        "scopes": ["나눔"]
                    }
                ]
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_parsed_gaji(&yank_args).expect_err("scope must be denied");
        assert_diag_with_fix(&err, "E_REG_AUTH_SCOPE_FORBIDDEN");
    }

    #[test]
    fn gaji_registry_main_cli_yank_auth_policy_schema_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_yank_auth_policy_schema_mismatch");
        let index = root.join("registry.index.json");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v0",
                "tokens": []
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_parsed_gaji(&yank_args).expect_err("must fail on schema mismatch");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");
    }

    #[test]
    fn gaji_registry_main_cli_yank_auth_policy_schema_mismatch_writes_audit_denied() {
        let root = temp_dir("main_gaji_registry_yank_auth_policy_schema_mismatch_audit");
        let index = root.join("registry.index.json");
        let audit = root.join("registry.audit.jsonl");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v0",
                "tokens": []
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_parsed_gaji(&yank_args).expect_err("must fail on schema mismatch");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_POLICY_SCHEMA")
        );
    }

    #[test]
    fn gaji_registry_main_cli_yank_auth_policy_missing_tokens_fails() {
        let root = temp_dir("main_gaji_registry_yank_auth_policy_missing_tokens");
        let index = root.join("registry.index.json");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v1"
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_parsed_gaji(&yank_args).expect_err("missing tokens must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");
    }

    #[test]
    fn gaji_registry_main_cli_yank_auth_policy_missing_tokens_writes_audit_denied() {
        let root = temp_dir("main_gaji_registry_yank_auth_policy_missing_tokens_audit");
        let index = root.join("registry.index.json");
        let audit = root.join("registry.audit.jsonl");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v1"
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let publish_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        run_parsed_gaji(&publish_args).expect("publish");

        let yank_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "yank".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--reason-code".to_string(),
            "policy".to_string(),
            "--token".to_string(),
            "token2".to_string(),
            "--role".to_string(),
            "scope_admin".to_string(),
        ];
        let err = run_parsed_gaji(&yank_args).expect_err("missing tokens must fail");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[1]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_POLICY_SCHEMA")
        );
    }

    #[test]
    fn gaji_registry_main_cli_publish_auth_policy_schema_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_publish_auth_policy_schema_mismatch");
        let index = root.join("registry.index.json");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v0",
                "tokens": []
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on schema mismatch");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");
    }

    #[test]
    fn gaji_registry_main_cli_publish_auth_policy_schema_mismatch_writes_audit_denied() {
        let root = temp_dir("main_gaji_registry_publish_auth_policy_schema_mismatch_audit");
        let index = root.join("registry.index.json");
        let audit = root.join("registry.audit.jsonl");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v0",
                "tokens": []
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on schema mismatch");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_POLICY_SCHEMA")
        );
    }

    #[test]
    fn gaji_registry_main_cli_publish_auth_policy_missing_tokens_writes_audit_denied() {
        let root = temp_dir("main_gaji_registry_publish_auth_policy_missing_tokens_audit");
        let index = root.join("registry.index.json");
        let audit = root.join("registry.audit.jsonl");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v1"
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on missing tokens");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");

        let rows = read_audit_rows(&audit);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("allowed"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            rows[0]
                .get("body")
                .and_then(|b| b.get("error_code"))
                .and_then(|v| v.as_str()),
            Some("E_REG_AUTH_POLICY_SCHEMA")
        );
    }

    #[test]
    fn gaji_registry_main_cli_publish_auth_policy_missing_tokens_fails() {
        let root = temp_dir("main_gaji_registry_publish_auth_policy_missing_tokens");
        let index = root.join("registry.index.json");
        let policy = root.join("auth_policy.json");
        fs::write(
            &policy,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.auth_policy.v1"
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "publish".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--auth-policy".to_string(),
            policy.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--archive-sha256".to_string(),
            "sha256:a".to_string(),
            "--token".to_string(),
            "token1".to_string(),
            "--role".to_string(),
            "publisher".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on missing tokens");
        assert_diag_with_fix(&err, "E_REG_AUTH_POLICY_SCHEMA");
    }

    #[test]
    fn gaji_registry_main_cli_verify_passes() {
        let root = temp_dir("main_gaji_registry_verify_pass");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-verify-pass",
            "sha256:index-main-reg-verify-pass",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-verify-pass",
                "index_root_hash": "sha256:index-main-reg-verify-pass"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("verify pass");
    }

    #[test]
    fn gaji_registry_main_cli_verify_missing_index_file_fails() {
        let root = temp_dir("main_gaji_registry_verify_missing_index");
        let index = root.join("missing.registry.index.json");
        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing index file must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_READ");
    }

    #[test]
    fn gaji_registry_main_cli_verify_invalid_index_json_fails() {
        let root = temp_dir("main_gaji_registry_verify_invalid_index_json");
        let index = root.join("registry.index.json");
        fs::write(&index, "{ invalid json").expect("write bad index");
        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("invalid index json must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_PARSE");
    }

    #[test]
    fn gaji_registry_main_cli_verify_index_schema_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_verify_index_schema_mismatch");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v0",
                "snapshot_id": "snap-main-reg-verify-schema-bad",
                "index_root_hash": "sha256:index-main-reg-verify-schema-bad"
            }))
            .expect("index json"),
        )
        .expect("write index");
        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("index schema mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_SCHEMA");
    }

    #[test]
    fn gaji_registry_main_cli_verify_index_entry_schema_passes() {
        let root = temp_dir("main_gaji_registry_verify_index_entry_schema");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");
        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("index_entry schema verify should pass");
    }

    #[test]
    fn gaji_registry_main_cli_verify_package_versions_schema_passes() {
        let root = temp_dir("main_gaji_registry_verify_package_versions_schema");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "scope": "표준",
                "name": "역학",
                "versions": [{
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");
        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("package_versions schema verify should pass");
    }

    #[test]
    fn gaji_registry_main_cli_verify_prefers_non_yanked_when_duplicate_pin_exists() {
        let root = temp_dir("main_gaji_registry_verify_prefers_non_yanked_duplicate");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-dup-pin",
                "index_root_hash": "sha256:index-main-reg-dup-pin",
                "entries": [
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "yanked": true
                    },
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "yanked": false
                    }
                ]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-dup-pin",
                "index_root_hash": "sha256:index-main-reg-dup-pin"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--deny-yanked-locked".to_string(),
        ];
        run_parsed_gaji(&args).expect("must prefer non-yanked duplicate and pass");
    }

    #[test]
    fn gaji_registry_main_cli_verify_duplicate_pin_reports_yanked_index_zero() {
        let root = temp_dir("main_gaji_registry_verify_dup_pin_report_counts");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-dup-pin-count",
                "index_root_hash": "sha256:index-main-reg-dup-pin-count",
                "entries": [
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "yanked": true
                    },
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "yanked": false
                    }
                ]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-dup-pin-count",
                "index_root_hash": "sha256:index-main-reg-dup-pin-count"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let out = root.join("verify.report.json");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("verify should pass");

        let report_text = fs::read_to_string(&out).expect("read report");
        let report: Value = serde_json::from_str(&report_text).expect("parse report");
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.verify_report.v1")
        );
        assert_eq!(report.get("packages").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(report.get("matched").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(report.get("yanked_lock").and_then(|v| v.as_u64()), Some(0));
        assert_eq!(report.get("yanked_index").and_then(|v| v.as_u64()), Some(0));
        assert_eq!(
            report
                .get("duplicate_resolution_policy")
                .and_then(|v| v.as_str()),
            Some(crate::cli::gaji_registry::VERIFY_DUPLICATE_RESOLUTION_POLICY)
        );
    }

    #[test]
    fn gaji_registry_main_cli_verify_duplicate_pin_same_state_archive_pin_is_order_independent() {
        let root = temp_dir("main_gaji_registry_verify_dup_pin_same_state_archive_order");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-dup-pin-archive",
                "index_root_hash": "sha256:index-main-reg-dup-pin-archive",
                "entries": [
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "archive_sha256": "sha256:b",
                        "download_url": "https://registry/b",
                        "summary": "나",
                        "yanked": false
                    },
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "archive_sha256": "sha256:a",
                        "download_url": "https://registry/a",
                        "summary": "가",
                        "yanked": false
                    }
                ]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-dup-pin-archive",
                "index_root_hash": "sha256:index-main-reg-dup-pin-archive"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "archive_sha256": "sha256:a",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("archive pin should match deterministic duplicate choice");
    }

    #[test]
    fn gaji_registry_main_cli_verify_duplicate_pin_prefers_contract_matched_entry() {
        let root = temp_dir("main_gaji_registry_verify_dup_pin_prefers_contract_match");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-dup-pin-contract",
                "index_root_hash": "sha256:index-main-reg-dup-pin-contract",
                "entries": [
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "contract": "D-APPROX",
                        "yanked": false
                    },
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "contract": "D-STRICT",
                        "yanked": false
                    }
                ]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-dup-pin-contract",
                "index_root_hash": "sha256:index-main-reg-dup-pin-contract"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "contract": "D-STRICT",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("contract pin should prefer matched duplicate entry");
    }

    #[test]
    fn gaji_registry_main_cli_verify_duplicate_pin_prefers_dependencies_matched_entry() {
        let root = temp_dir("main_gaji_registry_verify_dup_pin_prefers_dependencies_match");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-dup-pin-deps",
                "index_root_hash": "sha256:index-main-reg-dup-pin-deps",
                "entries": [
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "dependencies": {
                            "표준/벡터": "20.6.0"
                        },
                        "yanked": false
                    },
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "dependencies": {
                            "표준/힘": "20.6.0",
                            "표준/벡터": "20.6.0"
                        },
                        "yanked": false
                    }
                ]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-dup-pin-deps",
                "index_root_hash": "sha256:index-main-reg-dup-pin-deps"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "dependencies": {
                    "표준/벡터": "20.6.0",
                    "표준/힘": "20.6.0"
                },
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("dependencies pin should prefer matched duplicate entry");
    }

    #[test]
    fn gaji_registry_main_cli_verify_duplicate_pin_prefers_higher_pin_match_score() {
        let root = temp_dir("main_gaji_registry_verify_dup_pin_prefers_higher_pin_match_score");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-dup-pin-score",
                "index_root_hash": "sha256:index-main-reg-dup-pin-score",
                "entries": [
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "contract": "D-STRICT",
                        "min_runtime": "v20.6.29",
                        "yanked": false
                    },
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "contract": "D-STRICT",
                        "min_runtime": "v20.6.30",
                        "yanked": false
                    }
                ]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-dup-pin-score",
                "index_root_hash": "sha256:index-main-reg-dup-pin-score"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "contract": "D-STRICT",
                "min_runtime": "v20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args)
            .expect("verify should prefer duplicate entry with higher pin-match score");
    }

    #[test]
    fn gaji_registry_main_cli_verify_duplicate_pin_prioritizes_non_yanked_over_higher_score() {
        let root = temp_dir("main_gaji_registry_verify_dup_pin_non_yanked_over_higher_score");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-dup-pin-non-yanked-first",
                "index_root_hash": "sha256:index-main-reg-dup-pin-non-yanked-first",
                "entries": [
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "contract": "D-STRICT",
                        "yanked": true
                    },
                    {
                        "schema": "ddn.registry.index_entry.v1",
                        "scope": "표준",
                        "name": "역학",
                        "version": "20.6.30",
                        "contract": "D-APPROX",
                        "yanked": false
                    }
                ]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-dup-pin-non-yanked-first",
                "index_root_hash": "sha256:index-main-reg-dup-pin-non-yanked-first"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "contract": "D-STRICT",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("non-yanked must be selected before score");
        assert!(
            err.contains("E_REG_CONTRACT_MISMATCH"),
            "unexpected err: {}",
            err
        );
    }

    #[test]
    fn gaji_registry_main_cli_verify_duplicate_pin_score_tie_is_order_independent_for_diag() {
        let root = temp_dir("main_gaji_registry_verify_dup_pin_score_tie_diag_order_independent");
        let index_a = root.join("registry_a.index.json");
        let index_b = root.join("registry_b.index.json");
        let entry_1 = json!({
            "schema": "ddn.registry.index_entry.v1",
            "scope": "표준",
            "name": "역학",
            "version": "20.6.30",
            "min_runtime": "v20.6.29",
            "yanked": false
        });
        let entry_2 = json!({
            "schema": "ddn.registry.index_entry.v1",
            "scope": "표준",
            "name": "역학",
            "version": "20.6.30",
            "min_runtime": "v20.6.30",
            "yanked": false
        });
        fs::write(
            &index_a,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-dup-pin-score-tie",
                "index_root_hash": "sha256:index-main-reg-dup-pin-score-tie",
                "entries": [entry_1.clone(), entry_2.clone()]
            }))
            .expect("index json"),
        )
        .expect("write index a");
        fs::write(
            &index_b,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-dup-pin-score-tie",
                "index_root_hash": "sha256:index-main-reg-dup-pin-score-tie",
                "entries": [entry_2, entry_1]
            }))
            .expect("index json"),
        )
        .expect("write index b");

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-dup-pin-score-tie",
                "index_root_hash": "sha256:index-main-reg-dup-pin-score-tie"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "min_runtime": "v99.0.0",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args_a = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index_a.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let args_b = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index_b.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err_a = run_parsed_gaji(&args_a).expect_err("min_runtime mismatch expected");
        let err_b = run_parsed_gaji(&args_b).expect_err("min_runtime mismatch expected");
        assert_eq!(
            err_a, err_b,
            "score-tie diagnostics must be order independent"
        );
    }

    #[test]
    fn gaji_registry_main_cli_verify_package_versions_missing_scope_fails() {
        let root = temp_dir("main_gaji_registry_verify_package_versions_missing_scope");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "name": "역학",
                "versions": [{
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");
        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing scope must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("scope"));
    }

    #[test]
    fn gaji_registry_main_cli_verify_package_versions_missing_name_fails() {
        let root = temp_dir("main_gaji_registry_verify_package_versions_missing_name");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "scope": "표준",
                "versions": [{
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");
        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing name must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("name"));
    }

    #[test]
    fn gaji_registry_main_cli_verify_package_versions_missing_version_field_fails() {
        let root = temp_dir("main_gaji_registry_verify_package_versions_missing_version");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.package_versions.v1",
                "scope": "표준",
                "name": "역학",
                "versions": [{
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");
        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing version field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("version"));
    }

    #[test]
    fn gaji_registry_main_cli_verify_index_entry_missing_scope_field_fails() {
        let root = temp_dir("main_gaji_registry_verify_index_entry_missing_scope");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "name": "역학",
                "version": "20.6.30",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");
        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing scope field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("scope"));
    }

    #[test]
    fn gaji_registry_main_cli_verify_index_entry_missing_name_field_fails() {
        let root = temp_dir("main_gaji_registry_verify_index_entry_missing_name");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "version": "20.6.30",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");
        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing name field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("name"));
    }

    #[test]
    fn gaji_registry_main_cli_verify_index_entry_missing_version_field_fails() {
        let root = temp_dir("main_gaji_registry_verify_index_entry_missing_version");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "yanked": false
            }))
            .expect("index json"),
        )
        .expect("write index");
        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing version field must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("version"));
    }

    #[test]
    fn gaji_registry_main_cli_verify_frozen_lock_requires_snapshot_meta_in_lock() {
        let root = temp_dir("main_gaji_registry_verify_frozen_lock_missing_meta");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-verify-frozen",
            "sha256:index-main-reg-verify-frozen",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--frozen-lockfile".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("frozen lock must require snapshot pins");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISSING");
    }

    #[test]
    fn gaji_registry_main_cli_verify_expect_snapshot_id_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_verify_expect_snapshot_id_mismatch");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-verify-snapshot-good",
            "sha256:index-main-reg-verify-snapshot-good",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--expect-snapshot-id".to_string(),
            "snap-main-reg-verify-snapshot-bad".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("snapshot id mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_SNAPSHOT_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_verify_expect_index_root_hash_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_verify_expect_index_root_hash_mismatch");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-verify-index-good",
            "sha256:index-main-reg-verify-index-good",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--expect-index-root-hash".to_string(),
            "sha256:index-main-reg-verify-index-bad".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("index root hash mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_ROOT_HASH_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_verify_require_trust_root_missing_fails() {
        let root = temp_dir("main_gaji_registry_verify_require_trust_root_missing");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-verify-no-trust",
                "index_root_hash": "sha256:index-main-reg-verify-no-trust",
                "entries": [{
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "yanked": false
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--require-trust-root".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail when trust_root is missing");
        assert_diag_with_fix(&err, "E_REG_TRUST_ROOT_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_verify_expect_trust_root_hash_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_verify_expect_trust_root_hash_mismatch");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-verify-trust-good",
            "sha256:index-main-reg-verify-trust-good",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--expect-trust-root-hash".to_string(),
            "sha256:trust-not-match".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("trust_root hash mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_TRUST_ROOT_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_package_id_invalid_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_bad_id");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-bad-id",
            "sha256:index-main-reg-bad-id",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-bad-id",
                "index_root_hash": "sha256:index-main-reg-bad-id"
            },
            "packages": [{
                "id": "표준역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("invalid lock package id must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_package_id_extra_slash_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_bad_id_extra_slash");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-bad-id-2",
            "sha256:index-main-reg-bad-id-2",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-bad-id-2",
                "index_root_hash": "sha256:index-main-reg-bad-id-2"
            },
            "packages": [{
                "id": "표준/역학/추가",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err =
            run_parsed_gaji(&args).expect_err("invalid lock package id extra slash must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_package_id_empty_scope_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_bad_id_empty_scope");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-bad-id-empty-scope",
            "sha256:index-main-reg-bad-id-empty-scope",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-bad-id-empty-scope",
                "index_root_hash": "sha256:index-main-reg-bad-id-empty-scope"
            },
            "packages": [{
                "id": "/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("empty scope in lock package id must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_package_id_empty_name_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_bad_id_empty_name");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-bad-id-empty-name",
            "sha256:index-main-reg-bad-id-empty-name",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-bad-id-empty-name",
                "index_root_hash": "sha256:index-main-reg-bad-id-empty-name"
            },
            "packages": [{
                "id": "표준/",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("empty name in lock package id must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_package_id_scope_with_space_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_bad_id_scope_space");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-bad-id-scope-space",
            "sha256:index-main-reg-bad-id-scope-space",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-bad-id-scope-space",
                "index_root_hash": "sha256:index-main-reg-bad-id-scope-space"
            },
            "packages": [{
                "id": " 표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args)
            .expect_err("scope with leading space in lock package id must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_package_id_name_with_space_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_bad_id_name_space");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-bad-id-name-space",
            "sha256:index-main-reg-bad-id-name-space",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-bad-id-name-space",
                "index_root_hash": "sha256:index-main-reg-bad-id-name-space"
            },
            "packages": [{
                "id": "표준/ 역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args)
            .expect_err("name with leading space in lock package id must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_package_id_scope_inner_tab_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_bad_id_scope_inner_tab");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-bad-id-scope-inner-tab",
            "sha256:index-main-reg-bad-id-scope-inner-tab",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-bad-id-scope-inner-tab",
                "index_root_hash": "sha256:index-main-reg-bad-id-scope-inner-tab"
            },
            "packages": [{
                "id": "표\t준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("inner tab in scope must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_package_id_name_inner_newline_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_bad_id_name_inner_newline");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-bad-id-name-inner-newline",
            "sha256:index-main-reg-bad-id-name-inner-newline",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-bad-id-name-inner-newline",
                "index_root_hash": "sha256:index-main-reg-bad-id-name-inner-newline"
            },
            "packages": [{
                "id": "표준/역\n학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("inner newline in name must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGE_ID_INVALID");
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_package_id_non_string_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_bad_id_non_string");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-bad-id-non-string",
            "sha256:index-main-reg-bad-id-non-string",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-bad-id-non-string",
                "index_root_hash": "sha256:index-main-reg-bad-id-non-string"
            },
            "packages": [{
                "id": 123,
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("non-string id in lock package must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("id"));
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_package_version_non_string_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_bad_version_non_string");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-bad-version-non-string",
            "sha256:index-main-reg-bad-version-non-string",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-bad-version-non-string",
                "index_root_hash": "sha256:index-main-reg-bad-version-non-string"
            },
            "packages": [{
                "id": "표준/역학",
                "version": 20630,
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("non-string version in lock package must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("version"));
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_package_version_empty_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_bad_version_empty");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-bad-version-empty",
            "sha256:index-main-reg-bad-version-empty",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-bad-version-empty",
                "index_root_hash": "sha256:index-main-reg-bad-version-empty"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("empty version in lock package must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("version"));
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_package_version_with_space_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_bad_version_space");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-bad-version-space",
            "sha256:index-main-reg-bad-version-space",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-bad-version-space",
                "index_root_hash": "sha256:index-main-reg-bad-version-space"
            },
            "packages": [{
                "id": "표준/역학",
                "version": " 20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("spaced version in lock package must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_FIELD");
        assert!(err.contains("version"));
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_schema_invalid_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_schema_bad");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-bad-schema",
            "sha256:index-main-reg-bad-schema",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v0",
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("invalid lock schema must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_SCHEMA");
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_packages_missing_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_packages_missing");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-missing-packages",
            "sha256:index-main-reg-missing-packages",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1"
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing lock packages must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGES");
    }

    #[test]
    fn gaji_registry_main_cli_verify_lock_packages_not_array_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_lock_packages_not_array");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-packages-not-array",
            "sha256:index-main-reg-packages-not-array",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": {}
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("non-array lock packages must fail");
        assert_diag_with_fix(&err, "E_REG_LOCK_PACKAGES");
    }

    #[test]
    fn gaji_registry_main_cli_verify_missing_index_entry_fails_with_fix() {
        let root = temp_dir("main_gaji_registry_verify_missing_index_entry");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-verify-missing",
                "index_root_hash": "sha256:index-main-reg-verify-missing",
                "entries": []
            }))
            .expect("index json"),
        )
        .expect("write index");

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-verify-missing",
                "index_root_hash": "sha256:index-main-reg-verify-missing"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing index entry must fail");
        assert_diag_with_fix(&err, "E_REG_INDEX_NOT_FOUND");
    }

    #[test]
    fn gaji_registry_main_cli_verify_deny_yanked_locked_fails() {
        let root = temp_dir("main_gaji_registry_verify_deny_yanked");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-yanked",
                "index_root_hash": "sha256:index-main-reg-yanked",
                "entries": [{
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "yanked": true
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-yanked",
                "index_root_hash": "sha256:index-main-reg-yanked"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--deny-yanked-locked".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("deny yanked must fail");
        assert_diag_with_fix(&err, "E_REG_YANKED_LOCKED");
    }

    #[test]
    fn gaji_registry_main_cli_verify_writes_report_json() {
        let root = temp_dir("main_gaji_registry_verify_report_json");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-report",
            "sha256:index-main-reg-report",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-report",
                "index_root_hash": "sha256:index-main-reg-report"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let out = root.join("verify.report.json");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("verify with out");

        let report_text = fs::read_to_string(&out).expect("read report");
        let report: Value = serde_json::from_str(&report_text).expect("parse report");
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.verify_report.v1")
        );
        assert_eq!(report.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            report.get("index_path").and_then(|v| v.as_str()),
            Some(index.to_string_lossy().as_ref())
        );
        assert_eq!(
            report.get("lock_path").and_then(|v| v.as_str()),
            Some(lock.to_string_lossy().as_ref())
        );
        assert_eq!(report.get("packages").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(report.get("matched").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(report.get("yanked_lock").and_then(|v| v.as_u64()), Some(0));
        assert_eq!(report.get("yanked_index").and_then(|v| v.as_u64()), Some(0));
    }

    #[test]
    fn gaji_registry_main_cli_verify_out_parent_file_fails_with_report_write() {
        let root = temp_dir("main_gaji_registry_verify_out_parent_file");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-report-write-fail",
            "sha256:index-main-reg-report-write-fail",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let parent_file = root.join("blocked");
        fs::write(&parent_file, "file blocks directory").expect("write parent file");
        let out = parent_file.join("verify.report.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("report out parent file must fail");
        assert_diag_with_fix(&err, "E_REG_REPORT_WRITE");
        assert!(!out.exists());
    }

    #[test]
    fn gaji_registry_main_cli_verify_with_audit_verify_out_write_failure_skips_audit_step() {
        let root = temp_dir("main_gaji_registry_verify_with_audit_verify_out_write_fail");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-verify-out-write-fail",
            "sha256:index-main-reg-verify-out-write-fail",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-verify-out-write-fail",
                "index_root_hash": "sha256:index-main-reg-verify-out-write-fail"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let verify_out_parent = root.join("blocked_verify_out");
        fs::write(&verify_out_parent, "file blocks directory").expect("write parent file");
        let verify_out = verify_out_parent.join("verify.report.json");
        let audit_out = root.join("audit.verify.report.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            verify_out.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--audit-out".to_string(),
            audit_out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("verify out write failure must fail");
        assert_diag_with_fix(&err, "E_REG_REPORT_WRITE");
        assert!(!verify_out.exists());
        assert!(
            !audit_out.exists(),
            "audit step should be skipped when verify report write already failed"
        );
    }

    #[test]
    fn gaji_registry_main_cli_verify_writes_default_report_json() {
        let root = temp_dir("main_gaji_registry_verify_report_default");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-report-default",
            "sha256:index-main-reg-report-default",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-report-default",
                "index_root_hash": "sha256:index-main-reg-report-default"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("verify default out");

        let default_report = lock.with_extension("verify.report.json");
        let report_text = fs::read_to_string(default_report).expect("read report");
        let report: Value = serde_json::from_str(&report_text).expect("parse report");
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.registry.verify_report.v1")
        );
        assert_eq!(report.get("ok").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(report.get("packages").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(report.get("matched").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(
            report
                .get("duplicate_resolution_policy")
                .and_then(|v| v.as_str()),
            Some(crate::cli::gaji_registry::VERIFY_DUPLICATE_RESOLUTION_POLICY)
        );
    }

    #[test]
    fn gaji_registry_main_cli_verify_empty_packages_writes_zero_counts_report() {
        let root = temp_dir("main_gaji_registry_verify_empty_packages_report");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-empty-packages",
            "sha256:index-main-reg-empty-packages",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "packages": []
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("verify with empty packages should pass");

        let report_path = lock.with_extension("verify.report.json");
        let report_text = fs::read_to_string(&report_path).expect("read report");
        let report: Value = serde_json::from_str(&report_text).expect("parse report");
        assert_verify_report_contract(&report, 0, 0);
        assert_eq!(report.get("yanked_lock").and_then(|v| v.as_u64()), Some(0));
        assert_eq!(report.get("yanked_index").and_then(|v| v.as_u64()), Some(0));
    }

    #[test]
    fn gaji_registry_main_cli_verify_report_counts_yanked_values() {
        let root = temp_dir("main_gaji_registry_verify_report_yanked_counts");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-report-yanked",
                "index_root_hash": "sha256:index-main-reg-report-yanked",
                "entries": [{
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "yanked": true
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-report-yanked",
                "index_root_hash": "sha256:index-main-reg-report-yanked"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": true
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let out = root.join("verify.report.json");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("verify with yanked counts");

        let report_text = fs::read_to_string(&out).expect("read report");
        let report: Value = serde_json::from_str(&report_text).expect("parse report");
        assert_verify_report_contract(&report, 1, 1);
        assert_eq!(report.get("yanked_lock").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(report.get("yanked_index").and_then(|v| v.as_u64()), Some(1));
    }

    #[test]
    fn gaji_registry_main_cli_verify_failure_does_not_write_report() {
        let root = temp_dir("main_gaji_registry_verify_fail_no_report");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-fail-no-report",
                "index_root_hash": "sha256:index-main-reg-fail-no-report",
                "entries": [{
                    "schema": "ddn.registry.index_entry.v1",
                    "scope": "표준",
                    "name": "역학",
                    "version": "20.6.30",
                    "yanked": true
                }]
            }))
            .expect("index json"),
        )
        .expect("write index");

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-fail-no-report",
                "index_root_hash": "sha256:index-main-reg-fail-no-report"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let out = root.join("verify.report.json");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--deny-yanked-locked".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("deny-yanked verify must fail");
        assert_diag_with_fix(&err, "E_REG_YANKED_LOCKED");
        assert!(!out.exists(), "report file must not be written on failure");
    }

    #[test]
    fn gaji_registry_main_cli_verify_with_audit_requires_log() {
        let root = temp_dir("main_gaji_registry_verify_need_audit_log");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-need-audit-log",
            "sha256:index-main-reg-need-audit-log",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-need-audit-log",
                "index_root_hash": "sha256:index-main-reg-need-audit-log"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("verify-audit needs audit log");
        assert_diag_with_fix(&err, "E_REG_AUDIT_VERIFY_LOG_REQUIRED");
    }

    #[test]
    fn gaji_registry_main_cli_verify_with_audit_parse_failure_writes_only_verify_report() {
        let root = temp_dir("main_gaji_registry_verify_with_audit_parse_failure_reports");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-with-audit-parse-fail",
            "sha256:index-main-reg-with-audit-parse-fail",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-with-audit-parse-fail",
                "index_root_hash": "sha256:index-main-reg-with-audit-parse-fail"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let audit = root.join("registry.audit.jsonl");
        fs::write(&audit, "{ not-json").expect("write bad audit");

        let verify_out = root.join("verify.report.json");
        let audit_out = root.join("audit.verify.report.json");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            verify_out.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--audit-out".to_string(),
            audit_out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("invalid audit json must fail");
        assert_diag_with_fix(&err, "E_REG_AUDIT_PARSE");

        assert!(
            verify_out.exists(),
            "verify report should be written before audit step fails"
        );
        assert!(
            !audit_out.exists(),
            "audit report must not be written on audit parse failure"
        );

        let verify_report_text = fs::read_to_string(&verify_out).expect("read verify report");
        let verify_report: Value =
            serde_json::from_str(&verify_report_text).expect("parse verify report");
        assert_verify_report_contract(&verify_report, 1, 1);
    }

    #[test]
    fn gaji_registry_main_cli_verify_with_audit_verify_failure_writes_no_reports() {
        let root = temp_dir("main_gaji_registry_verify_with_audit_verify_failure_no_reports");
        let index = root.join("registry.index.json");
        fs::write(
            &index,
            serde_json::to_string_pretty(&json!({
                "schema": "ddn.registry.snapshot.v1",
                "snapshot_id": "snap-main-reg-with-audit-verify-fail",
                "index_root_hash": "sha256:index-main-reg-with-audit-verify-fail",
                "entries": []
            }))
            .expect("index json"),
        )
        .expect("write index");

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-with-audit-verify-fail",
                "index_root_hash": "sha256:index-main-reg-with-audit-verify-fail"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let verify_out = root.join("verify.report.json");
        let audit_out = root.join("audit.verify.report.json");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            verify_out.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--audit-out".to_string(),
            audit_out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("verify stage must fail before audit");
        assert_diag_with_fix(&err, "E_REG_INDEX_NOT_FOUND");

        assert!(
            !verify_out.exists(),
            "verify report must not be written when verify stage fails"
        );
        assert!(
            !audit_out.exists(),
            "audit report must not be written when verify stage fails"
        );
    }

    #[test]
    fn gaji_registry_main_cli_verify_contract_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_verify_contract_mismatch");
        let index = root.join("registry.index.json");
        let lock = root.join("ddn.lock");
        write_main_verify_fixture(
            &index,
            &lock,
            "snap-main-reg-contract",
            "sha256:index-main-reg-contract",
            json!({
                "contract": "D-STRICT",
                "min_runtime": "20.6.29",
                "detmath_seal_hash": "sha256:seal-a"
            }),
            json!({
                "contract": "D-SEALED"
            }),
        );
        let args = main_registry_verify_args(&index, &lock);
        let err = run_parsed_gaji(&args).expect_err("contract mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_CONTRACT_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_verify_min_runtime_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_verify_min_runtime_mismatch");
        let index = root.join("registry.index.json");
        let lock = root.join("ddn.lock");
        write_main_verify_fixture(
            &index,
            &lock,
            "snap-main-reg-min-runtime",
            "sha256:index-main-reg-min-runtime",
            json!({
                "contract": "D-STRICT",
                "min_runtime": "20.6.29",
                "detmath_seal_hash": "sha256:seal-a"
            }),
            json!({
                "min_runtime": "20.6.31"
            }),
        );
        let args = main_registry_verify_args(&index, &lock);
        let err = run_parsed_gaji(&args).expect_err("min_runtime mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_MIN_RUNTIME_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_verify_detmath_seal_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_verify_detmath_seal_mismatch");
        let index = root.join("registry.index.json");
        let lock = root.join("ddn.lock");
        write_main_verify_fixture(
            &index,
            &lock,
            "snap-main-reg-seal",
            "sha256:index-main-reg-seal",
            json!({
                "contract": "D-STRICT",
                "min_runtime": "20.6.29",
                "detmath_seal_hash": "sha256:seal-a"
            }),
            json!({
                "detmath_seal_hash": "sha256:seal-b"
            }),
        );
        let args = main_registry_verify_args(&index, &lock);
        let err = run_parsed_gaji(&args).expect_err("detmath seal mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_DETMATH_SEAL_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_verify_archive_sha256_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_verify_archive_sha_mismatch");
        let index = root.join("registry.index.json");
        let lock = root.join("ddn.lock");
        write_main_verify_fixture(
            &index,
            &lock,
            "snap-main-reg-archive",
            "sha256:index-main-reg-archive",
            json!({
                "archive_sha256": "sha256:archive-a"
            }),
            json!({
                "archive_sha256": "sha256:archive-b"
            }),
        );
        let args = main_registry_verify_args(&index, &lock);
        let err = run_parsed_gaji(&args).expect_err("archive sha mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_ARCHIVE_SHA256_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_verify_archive_sha256_match_passes() {
        let root = temp_dir("main_gaji_registry_verify_archive_sha_match");
        let index = root.join("registry.index.json");
        let lock = root.join("ddn.lock");
        write_main_verify_fixture(
            &index,
            &lock,
            "snap-main-reg-archive-match",
            "sha256:index-main-reg-archive-match",
            json!({
                "archive_sha256": "sha256:archive-a"
            }),
            json!({
                "archive_sha256": "sha256:archive-a"
            }),
        );
        let args = main_registry_verify_args(&index, &lock);
        run_parsed_gaji(&args).expect("archive sha match pass");
    }

    #[test]
    fn gaji_registry_main_cli_verify_download_url_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_verify_download_url_mismatch");
        let index = root.join("registry.index.json");
        let lock = root.join("ddn.lock");
        write_main_verify_fixture(
            &index,
            &lock,
            "snap-main-reg-download-url",
            "sha256:index-main-reg-download-url",
            json!({
                "download_url": "https://registry/a"
            }),
            json!({
                "download_url": "https://registry/b"
            }),
        );
        let args = main_registry_verify_args(&index, &lock);
        let err = run_parsed_gaji(&args).expect_err("download_url mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_DOWNLOAD_URL_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_download_passes() {
        let root = temp_dir("main_gaji_registry_download_pass");
        let index = root.join("registry.index.json");
        let source = root.join("archives").join("physics-20.6.30.ddn.tar.gz");
        fs::create_dir_all(source.parent().expect("parent")).expect("mkdir");
        let bytes = b"archive-main-pass";
        fs::write(&source, bytes).expect("write source");
        let expected_sha = sha256_hex_prefixed(bytes);

        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-main-reg-download-pass",
            "index_root_hash": "sha256:index-main-reg-download-pass",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "archive_sha256": expected_sha,
                "download_url": source.to_string_lossy(),
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let out = root.join("download").join("physics-20.6.30.ddn.tar.gz");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "download".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("download pass");

        let copied = fs::read(&out).expect("read copied");
        assert_eq!(copied, bytes);
    }

    #[test]
    fn gaji_registry_main_cli_download_sha_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_download_sha_mismatch");
        let index = root.join("registry.index.json");
        let source = root.join("archives").join("physics-20.6.30.ddn.tar.gz");
        fs::create_dir_all(source.parent().expect("parent")).expect("mkdir");
        fs::write(&source, b"archive-main-mismatch").expect("write source");

        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-main-reg-download-sha",
            "index_root_hash": "sha256:index-main-reg-download-sha",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "archive_sha256": "sha256:not-match",
                "download_url": source.to_string_lossy(),
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let out = root.join("download").join("physics-20.6.30.ddn.tar.gz");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "download".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("sha mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_ARCHIVE_SHA256_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_download_http_requires_allow_http() {
        let root = temp_dir("main_gaji_registry_download_http_requires_flag");
        let index = root.join("registry.index.json");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-main-reg-download-http-no-flag",
            "index_root_hash": "sha256:index-main-reg-download-http-no-flag",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "archive_sha256": "sha256:any",
                "download_url": "https://registry.example/physics-20.6.30.ddn.tar.gz",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let out = root.join("download").join("physics-20.6.30.ddn.tar.gz");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "download".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("http download must require allow-http");
        assert_diag_with_fix(&err, "E_REG_DOWNLOAD_HTTP_DISABLED");
    }

    #[test]
    fn gaji_registry_main_cli_download_http_allow_http_passes() {
        let root = temp_dir("main_gaji_registry_download_http_allow");
        let index = root.join("registry.index.json");
        let bytes = b"archive-main-http-ok";
        let expected_sha = sha256_hex_prefixed(bytes);
        let download_url = start_http_fixture(bytes);
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-main-reg-download-http-ok",
            "index_root_hash": "sha256:index-main-reg-download-http-ok",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "archive_sha256": expected_sha,
                "download_url": download_url,
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let out = root.join("download").join("physics-20.6.30.ddn.tar.gz");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "download".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--allow-http".to_string(),
        ];
        run_parsed_gaji(&args).expect("http download should pass with allow-http");
        let copied = fs::read(&out).expect("read copied");
        assert_eq!(copied, bytes);
    }

    #[test]
    fn gaji_registry_main_cli_download_offline_cache_miss_fails() {
        let root = temp_dir("main_gaji_registry_download_offline_cache_miss");
        let index = root.join("registry.index.json");
        let cache_dir = root.join("cache");
        let expected_sha = sha256_hex_prefixed(b"archive-main-cache-miss");
        let snapshot = json!({
            "schema": "ddn.registry.snapshot.v1",
            "snapshot_id": "snap-main-reg-download-offline-miss",
            "index_root_hash": "sha256:index-main-reg-download-offline-miss",
            "entries": [{
                "schema": "ddn.registry.index_entry.v1",
                "scope": "표준",
                "name": "역학",
                "version": "20.6.30",
                "archive_sha256": expected_sha,
                "download_url": "https://registry.example/physics-20.6.30.ddn.tar.gz",
                "yanked": false
            }]
        });
        fs::write(
            &index,
            serde_json::to_string_pretty(&snapshot).expect("json"),
        )
        .expect("write");

        let out = root.join("download").join("physics-20.6.30.ddn.tar.gz");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "download".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--scope".to_string(),
            "표준".to_string(),
            "--name".to_string(),
            "역학".to_string(),
            "--version".to_string(),
            "20.6.30".to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--cache-dir".to_string(),
            cache_dir.to_string_lossy().to_string(),
            "--offline".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("offline cache miss must fail");
        assert_diag_with_fix(&err, "E_CACHE_UNAVAILABLE_OFFLINE");
    }

    #[test]
    fn gaji_registry_main_cli_verify_dependencies_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_verify_dependencies_mismatch");
        let index = root.join("registry.index.json");
        let lock = root.join("ddn.lock");
        write_main_verify_fixture(
            &index,
            &lock,
            "snap-main-reg-deps",
            "sha256:index-main-reg-deps",
            json!({
                "dependencies": {"표준/수학": "1.0.0"}
            }),
            json!({
                "dependencies": {"표준/수학": "2.0.0"}
            }),
        );
        let args = main_registry_verify_args(&index, &lock);
        let err = run_parsed_gaji(&args).expect_err("dependencies mismatch must fail");
        assert_diag_with_fix(&err, "E_REG_DEPENDENCIES_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_verify_dependencies_match_with_different_key_order() {
        let root = temp_dir("main_gaji_registry_verify_dependencies_order");
        let index = root.join("registry.index.json");
        let lock = root.join("ddn.lock");
        write_main_verify_fixture(
            &index,
            &lock,
            "snap-main-reg-deps-order",
            "sha256:index-main-reg-deps-order",
            json!({
                "dependencies": {"a": 1, "b": 2}
            }),
            json!({
                "dependencies": {"b": 2, "a": 1}
            }),
        );
        let args = main_registry_verify_args(&index, &lock);
        run_parsed_gaji(&args).expect("dependencies order-insensitive match pass");
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_writes_report_json() {
        let root = temp_dir("main_gaji_registry_audit_verify_report_json");
        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let out = root.join("audit.verify.report.json");
        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("audit verify with out");

        let report_text = fs::read_to_string(&out).expect("read report");
        let report: Value = serde_json::from_str(&report_text).expect("parse report");
        assert_audit_verify_report_contract(&report, 1);
        assert_eq!(
            report.get("audit_log_path").and_then(|v| v.as_str()),
            Some(audit.to_string_lossy().as_ref())
        );
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_missing_log_file_fails() {
        let root = temp_dir("main_gaji_registry_audit_verify_missing_log");
        let audit = root.join("missing.registry.audit.jsonl");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing audit log must fail");
        assert_diag_with_fix(&err, "E_REG_AUDIT_READ");
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_invalid_log_json_fails() {
        let root = temp_dir("main_gaji_registry_audit_verify_invalid_json");
        let audit = root.join("registry.audit.jsonl");
        fs::write(&audit, "{ invalid json\n").expect("write bad audit log");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("invalid audit log json must fail");
        assert_diag_with_fix(&err, "E_REG_AUDIT_PARSE");
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_empty_log_fails() {
        let root = temp_dir("main_gaji_registry_audit_verify_empty_log");
        let audit = root.join("registry.audit.jsonl");
        fs::write(&audit, "").expect("write empty audit log");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("empty audit log must fail");
        assert_diag_with_fix(&err, "E_REG_AUDIT_EMPTY");
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_row_hash_tamper_fails() {
        let root = temp_dir("main_gaji_registry_audit_verify_row_hash_tamper");
        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let mut rows = read_audit_rows(&audit);
        rows.get_mut(0)
            .and_then(|row| row.as_object_mut())
            .expect("row 0")
            .insert(
                "row_hash".to_string(),
                Value::String("blake3:tampered".to_string()),
            );
        let text = rows
            .iter()
            .map(|row| serde_json::to_string(row).expect("row json"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&audit, format!("{text}\n")).expect("rewrite audit");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("tampered row hash must fail");
        assert_diag_with_fix(&err, "E_REG_AUDIT_ROW_HASH_MISMATCH");
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_chain_broken_fails() {
        let root = temp_dir("main_gaji_registry_audit_verify_chain_broken");
        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let mut rows = read_audit_rows(&audit);
        let row0 = rows.get_mut(0).expect("row 0");
        let body = row0.get_mut("body").expect("body");
        body.as_object_mut().expect("body object").insert(
            "prev_hash".to_string(),
            Value::String("blake3:unexpected".to_string()),
        );
        let body_text = serde_json::to_string(body).expect("body json");
        let row_hash = format!("blake3:{}", blake3::hash(body_text.as_bytes()).to_hex());
        row0.as_object_mut()
            .expect("row object")
            .insert("row_hash".to_string(), Value::String(row_hash));
        let text = rows
            .iter()
            .map(|row| serde_json::to_string(row).expect("row json"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&audit, format!("{text}\n")).expect("rewrite audit");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("broken chain must fail");
        assert_diag_with_fix(&err, "E_REG_AUDIT_CHAIN_BROKEN");
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_missing_body_field_fails() {
        let root = temp_dir("main_gaji_registry_audit_verify_missing_body");
        let audit = root.join("registry.audit.jsonl");
        fs::write(
            &audit,
            serde_json::to_string(&json!({
                "row_hash": "blake3:any"
            }))
            .expect("row json"),
        )
        .expect("write audit");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing body should fail");
        assert_diag_with_fix(&err, "E_REG_AUDIT_ROW_FIELD");
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_schema_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_audit_verify_schema_mismatch");
        let audit = root.join("registry.audit.jsonl");
        let body = json!({
            "schema": "ddn.registry.audit.v0",
            "ts": "2026-02-20T00:00:00Z",
            "actor": {"token_hash": "blake3:t", "role": "publisher"},
            "action": "publish",
            "package_id": "표준/역학@0.1.0",
            "prev_hash": Value::Null
        });
        let body_text = serde_json::to_string(&body).expect("body json");
        let row_hash = format!("blake3:{}", blake3::hash(body_text.as_bytes()).to_hex());
        fs::write(
            &audit,
            serde_json::to_string(&json!({
                "body": body,
                "row_hash": row_hash
            }))
            .expect("row json"),
        )
        .expect("write audit");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("schema mismatch should fail");
        assert_diag_with_fix(&err, "E_REG_AUDIT_SCHEMA");
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_missing_row_hash_field_fails() {
        let root = temp_dir("main_gaji_registry_audit_verify_missing_row_hash");
        let audit = root.join("registry.audit.jsonl");
        fs::write(
            &audit,
            serde_json::to_string(&json!({
                "body": {
                    "schema": "ddn.registry.audit.v1",
                    "ts": "2026-02-20T00:00:00Z",
                    "actor": {"token_hash": "blake3:t", "role": "publisher"},
                    "action": "publish",
                    "package_id": "표준/역학@0.1.0",
                    "prev_hash": Value::Null
                }
            }))
            .expect("row json"),
        )
        .expect("write audit");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("missing row_hash should fail");
        assert_diag_with_fix(&err, "E_REG_AUDIT_ROW_FIELD");
    }

    #[test]
    fn gaji_registry_main_cli_verify_with_audit_expect_last_hash_recovery() {
        let root = temp_dir("main_gaji_registry_verify_audit_recovery");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-verify",
            "sha256:index-main-reg-verify",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-verify",
                "index_root_hash": "sha256:index-main-reg-verify"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let audit = root.join("registry.audit.jsonl");
        let expected_hash = write_audit_log_and_last_hash(&audit);

        let bad_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            "blake3:not-match".to_string(),
        ];
        let err = run_parsed_gaji(&bad_args).expect_err("must fail on bad expected hash");
        assert_audit_last_hash_diag(&err);

        let good_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            expected_hash,
        ];
        run_parsed_gaji(&good_args).expect("recovery verify should pass");
    }

    #[test]
    fn gaji_registry_main_cli_verify_with_audit_expect_last_hash_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_verify_audit_mismatch_only");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-audit-mismatch",
            "sha256:index-main-reg-audit-mismatch",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-audit-mismatch",
                "index_root_hash": "sha256:index-main-reg-audit-mismatch"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            "blake3:not-match".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on expected hash mismatch");
        assert_audit_last_hash_diag(&err);
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_expect_last_hash_recovery() {
        let root = temp_dir("main_gaji_registry_audit_verify_recovery");
        let audit = root.join("registry.audit.jsonl");
        let expected_hash = write_audit_log_and_last_hash(&audit);

        let bad_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            "blake3:not-match".to_string(),
        ];
        let err = run_parsed_gaji(&bad_args).expect_err("must fail on bad expected hash");
        assert_audit_last_hash_diag(&err);

        let good_args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            expected_hash,
        ];
        run_parsed_gaji(&good_args).expect("recovery audit-verify should pass");
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_executes() {
        let root = temp_dir("main_gaji_registry_audit_verify_executes");
        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("audit-verify should pass");
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_expect_last_hash_mismatch_fails() {
        let root = temp_dir("main_gaji_registry_audit_verify_expect_last_hash_bad");
        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            "blake3:not-match".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("must fail on expected hash mismatch");
        assert_audit_last_hash_diag(&err);
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_expect_last_hash_mismatch_does_not_write_report() {
        let root = temp_dir("main_gaji_registry_audit_verify_hash_mismatch_no_report");
        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);
        let out = root.join("audit.verify.custom.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--expect-audit-last-hash".to_string(),
            "blake3:not-match".to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("expected hash mismatch must fail");
        assert_audit_last_hash_diag(&err);
        assert!(
            !out.exists(),
            "audit verify report must not be written on mismatch failure"
        );
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_parse_failure_does_not_write_default_report() {
        let root = temp_dir("main_gaji_registry_audit_verify_parse_no_default_report");
        let audit = root.join("registry.audit.jsonl");
        fs::write(&audit, "{ invalid json").expect("write bad audit");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("invalid audit json must fail");
        assert_diag_with_fix(&err, "E_REG_AUDIT_PARSE");

        let default_report = audit.with_extension("verify.report.json");
        assert!(
            !default_report.exists(),
            "default audit verify report must not be written on parse failure"
        );
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_out_parent_file_fails_with_report_write() {
        let root = temp_dir("main_gaji_registry_audit_verify_out_parent_file");
        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let out_parent = root.join("blocked");
        fs::write(&out_parent, "file blocks directory").expect("write parent file");
        let out = out_parent.join("audit.verify.report.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("audit verify out parent file must fail");
        assert_diag_with_fix(&err, "E_REG_REPORT_WRITE");
        assert!(!out.exists());
    }

    #[test]
    fn gaji_registry_main_cli_verify_with_audit_writes_audit_report_json() {
        let root = temp_dir("main_gaji_registry_verify_with_audit_report");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-audit-out",
            "sha256:index-main-reg-audit-out",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-audit-out",
                "index_root_hash": "sha256:index-main-reg-audit-out"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);
        let out = root.join("verify.report.json");
        let audit_out = root.join("audit.verify.report.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            out.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--audit-out".to_string(),
            audit_out.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("verify + audit with explicit out");

        let report_text = fs::read_to_string(&out).expect("read verify report");
        let report: Value = serde_json::from_str(&report_text).expect("parse verify report");
        assert_verify_report_contract(&report, 1, 1);
        let audit_report_text = fs::read_to_string(&audit_out).expect("read audit report");
        let audit_report: Value =
            serde_json::from_str(&audit_report_text).expect("parse audit report");
        assert_audit_verify_report_contract(&audit_report, 1);
    }

    #[test]
    fn gaji_registry_main_cli_verify_with_audit_out_parent_file_fails_with_report_write() {
        let root = temp_dir("main_gaji_registry_verify_with_audit_out_parent_file");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-audit-out-write-fail",
            "sha256:index-main-reg-audit-out-write-fail",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-audit-out-write-fail",
                "index_root_hash": "sha256:index-main-reg-audit-out-write-fail"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let verify_out = root.join("verify.report.json");
        let audit_out_parent = root.join("blocked_audit_out");
        fs::write(&audit_out_parent, "file blocks directory").expect("write parent file");
        let audit_out = audit_out_parent.join("audit.verify.report.json");

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--out".to_string(),
            verify_out.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
            "--audit-out".to_string(),
            audit_out.to_string_lossy().to_string(),
        ];
        let err = run_parsed_gaji(&args).expect_err("audit out parent file must fail");
        assert_diag_with_fix(&err, "E_REG_REPORT_WRITE");
        assert!(
            verify_out.exists(),
            "verify report should be written before audit report write"
        );
        assert!(!audit_out.exists());
        let verify_report_text = fs::read_to_string(&verify_out).expect("read verify report");
        let verify_report: Value =
            serde_json::from_str(&verify_report_text).expect("parse verify report");
        assert_verify_report_contract(&verify_report, 1, 1);
    }

    #[test]
    fn gaji_registry_main_cli_verify_with_audit_writes_default_reports() {
        let root = temp_dir("main_gaji_registry_verify_default_reports");
        let index = root.join("registry.index.json");
        write_registry_index_with_entry(
            &index,
            "snap-main-reg-default-report",
            "sha256:index-main-reg-default-report",
            "표준",
            "역학",
            "20.6.30",
        );

        let lock = root.join("ddn.lock");
        let lock_json = json!({
            "schema_version": "v1",
            "registry_snapshot": {
                "snapshot_id": "snap-main-reg-default-report",
                "index_root_hash": "sha256:index-main-reg-default-report"
            },
            "packages": [{
                "id": "표준/역학",
                "version": "20.6.30",
                "path": "x",
                "hash": "blake3:x",
                "yanked": false
            }]
        });
        fs::write(
            &lock,
            serde_json::to_string_pretty(&lock_json).expect("lock json"),
        )
        .expect("write lock");

        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "verify".to_string(),
            "--index".to_string(),
            index.to_string_lossy().to_string(),
            "--lock".to_string(),
            lock.to_string_lossy().to_string(),
            "--verify-audit".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("verify with audit should pass");

        let verify_report = lock.with_extension("verify.report.json");
        let audit_report = lock.with_extension("audit.verify.report.json");
        assert!(verify_report.exists());
        assert!(audit_report.exists());

        let verify_json: Value =
            serde_json::from_str(&fs::read_to_string(&verify_report).expect("read verify report"))
                .expect("parse verify report");
        assert_verify_report_contract(&verify_json, 1, 1);

        let audit_json: Value =
            serde_json::from_str(&fs::read_to_string(&audit_report).expect("read audit report"))
                .expect("parse audit report");
        assert_audit_verify_report_contract(&audit_json, 1);
    }

    #[test]
    fn gaji_registry_main_cli_audit_verify_writes_default_report() {
        let root = temp_dir("main_gaji_registry_audit_verify_default_report");
        let audit = root.join("registry.audit.jsonl");
        write_audit_log_and_last_hash(&audit);

        let args = vec![
            "teul-cli".to_string(),
            "gaji".to_string(),
            "registry".to_string(),
            "audit-verify".to_string(),
            "--audit-log".to_string(),
            audit.to_string_lossy().to_string(),
        ];
        run_parsed_gaji(&args).expect("audit-verify should pass");
        let report = audit.with_extension("verify.report.json");
        assert!(report.exists());
        let report_json: Value =
            serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
                .expect("parse report");
        assert_audit_verify_report_contract(&report_json, 1);
    }
}
