use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cli::bogae::{resolve_bogae_out_dir, BogaeMode, OverlayConfig};
use crate::cli::bogae_console::{render_drawlist_ascii, ConsoleRenderConfig};
use crate::cli::bogae_web::write_web_assets;
use crate::core::bogae::{
    decode_drawlist_detbin_any, hash_drawlist_detbin, BogaeCodec, BogaeError,
};

pub struct ViewOptions {
    pub bogae: BogaeMode,
    pub bogae_codec: Option<BogaeCodec>,
    pub bogae_out: Option<PathBuf>,
    pub bogae_skin: Option<PathBuf>,
    pub overlay: OverlayConfig,
    pub console_config: ConsoleRenderConfig,
    pub no_open: bool,
}

pub fn run_view(path: &Path, options: ViewOptions) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let (drawlist, codec) =
        decode_drawlist_detbin_any(&bytes).map_err(|err| format_error(path, err))?;
    if let Some(expected) = options.bogae_codec {
        if expected != codec {
            return Err(format!(
                "E_BOGAE_CODEC_MISMATCH {}:1:1 codec가 일치하지 않습니다 (expected={} actual={})",
                path.display(),
                expected.tag(),
                codec.tag()
            ));
        }
    }
    let hash = hash_drawlist_detbin(&bytes);
    let cmd_count = drawlist.cmds.len() as u32;

    match options.bogae {
        BogaeMode::Web => {
            let out_dir = resolve_bogae_out_dir(options.bogae_out.as_deref());
            let index_path = write_web_assets(
                &out_dir,
                &drawlist,
                &bytes,
                codec,
                options.bogae_skin.as_deref(),
                options.overlay,
            )?;
            if options.no_open {
                println!("bogae_hash={} cmd_count={} codec={}", hash, cmd_count, codec.tag());
                return Ok(());
            }
            open_in_browser(&index_path)?;
        }
        BogaeMode::Console => {
            println!("bogae_hash={} cmd_count={} codec={}", hash, cmd_count, codec.tag());
            println!(
                "{}",
                render_drawlist_ascii(&drawlist, options.console_config)
            );
        }
    }

    Ok(())
}

fn format_error(path: &Path, err: BogaeError) -> String {
    format!("{} {}:1:1 {}", err.code(), path.display(), err.message())
}

fn open_in_browser(path: &Path) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", "", &path.display().to_string()])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
