use std::path::PathBuf;

use clap::{Parser, Subcommand};
use std::env;

mod canon;
mod cli;
mod core;
mod file_meta;
mod lang;
mod runtime;
mod ai_prompt;

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
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        file: PathBuf,
        #[arg(long, aliases = ["ticks", "max-madi"], value_name = "N|infinite")]
        madi: Option<String>,
        #[arg(long, default_value = "0x0")]
        seed: String,
        #[arg(long = "state")]
        state: Vec<String>,
        #[arg(long = "state-file")]
        state_file: Vec<PathBuf>,
        #[arg(long = "diag-jsonl", alias = "diag")]
        diag_jsonl: Option<PathBuf>,
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
        file: PathBuf,
        #[arg(long, default_value_t = 1)]
        threads: usize,
        #[arg(long)]
        out: Option<PathBuf>,
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
enum GajiCommands {
    Lock {
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
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

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run {
            file,
            madi,
            seed,
            state,
            state_file,
            diag_jsonl,
            enable_repro,
            repro_json,
            run_manifest,
            artifact,
            trace_json,
            geoul_out,
            geoul_record_out,
            trace_tier,
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
            let seed = match parse_seed(&seed) {
                Ok(value) => value,
                Err(message) => {
                    eprintln!("E_CLI_BAD_SEED {}", message);
                    std::process::exit(1);
                }
            };
            let madi = match parse_madi_arg(madi.as_deref()) {
                Ok(value) => value,
                Err(message) => {
                    eprintln!("E_CLI_MADI {}", message);
                    std::process::exit(1);
                }
            };
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
            let repro_json = match (enable_repro, repro_json) {
                (true, Some(path)) => Some(path),
                (true, None) => Some(cli::paths::build_dir().join("repro").join("ddn.repro.last.json")),
                (false, Some(path)) => Some(path),
                (false, None) => None,
            };
            let artifact_pins = match cli::run::parse_artifact_pins(&artifact) {
                Ok(pins) => pins,
                Err(message) => {
                    eprintln!("E_CLI_ARTIFACT {}", message);
                    std::process::exit(1);
                }
            };
            let cmd_policy = match bogae_cmd_policy {
                cli::bogae::BogaeCmdPolicy::None => {
                    if bogae_cmd_cap.is_some() {
                        eprintln!("E_CLI_BOGAE_CMD_POLICY bogae-cmd-cap은 policy=none에서 사용할 수 없습니다.");
                        std::process::exit(1);
                    }
                    crate::core::bogae::CmdPolicyConfig::none()
                }
                cli::bogae::BogaeCmdPolicy::Cap => {
                    let cap = match bogae_cmd_cap {
                        Some(value) if value > 0 => value,
                        _ => {
                            eprintln!("E_CLI_BOGAE_CMD_POLICY bogae-cmd-cap(>0)이 필요합니다.");
                            std::process::exit(1);
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
                            eprintln!("E_CLI_BOGAE_CMD_POLICY bogae-cmd-cap(>0)이 필요합니다.");
                            std::process::exit(1);
                        }
                    };
                    crate::core::bogae::CmdPolicyConfig {
                        mode: crate::core::bogae::CmdPolicyMode::Summary,
                        cap,
                    }
                }
            };
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
            let run_command = Some(build_command_string());
            let options = cli::run::RunOptions {
                diag_jsonl,
                repro_json,
                trace_json,
                geoul_out,
                geoul_record_out,
                trace_tier: trace_tier.to_core(),
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
            if let Err(err) = cli::run::run_file(&file, madi, seed, options) {
                eprintln!("{}", err);
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
        Commands::Test { file, threads, out } => {
            if let Err(err) = cli::test::run_realms_test(&file, threads, out.as_deref()) {
                eprintln!("{}", err);
                std::process::exit(1);
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
            ReplayCommands::Diff { a, b, out, no_summary } => {
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
                if let Err(err) =
                    cli::geoul::run_geoul_query(&geoul, madi, &key, entry.as_deref())
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
                let out = out.unwrap_or_else(|| {
                    cli::paths::build_dir().join("ddn.patch.approval.json")
                });
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
                if let Err(err) = cli::patch::run_apply(&patch, &approval, out.as_deref(), in_place) {
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
                if let Err(err) = cli::patch::run_verify(
                    &patch,
                    &approval,
                    tests.as_deref(),
                    walk.as_deref(),
                ) {
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
        Commands::Gaji { command } => match command {
            GajiCommands::Lock { root, out } => {
                let root = root.unwrap_or_else(|| PathBuf::from("."));
                let out = out.unwrap_or_else(|| {
                    cli::paths::build_dir().join("lock").join("gaji.lock.json")
                });
                if let Err(err) = cli::gaji::run_lock(&root, &out) {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
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
                if let Err(err) = cli::latency::run_simulate(
                    l_madi,
                    mode.to_core(),
                    count,
                    seed,
                    out.as_deref(),
                ) {
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
    let cols = cols.trim().parse::<usize>().map_err(|_| "grid cols must be a positive integer".to_string())?;
    let rows = rows.trim().parse::<usize>().map_err(|_| "grid rows must be a positive integer".to_string())?;
    if cols == 0 || rows == 0 {
        return Err("grid cols/rows must be greater than 0".to_string());
    }
    Ok(Some((cols, rows)))
}

fn build_command_string() -> String {
    let mut parts = Vec::new();
    for arg in env::args() {
        parts.push(shell_quote_arg(&arg));
    }
    parts.join(" ")
}

fn shell_quote_arg(arg: &str) -> String {
    if arg.chars().any(|ch| ch.is_whitespace()) {
        let escaped = arg.replace('"', "\\\"");
        format!("\"{}\"", escaped)
    } else {
        arg.to_string()
    }
}
