use std::path::{Path, PathBuf};

use crate::cli::paths;

use clap::ValueEnum;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum BogaeMode {
    Web,
    Console,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum BogaeCodec {
    Bdl1,
    Bdl2,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum BogaeCmdPolicy {
    None,
    Cap,
    Summary,
}

#[derive(Clone, Copy, Debug)]
pub struct OverlayConfig {
    pub grid: bool,
    pub bounds: bool,
    pub delta: bool,
}

impl OverlayConfig {
    pub fn empty() -> Self {
        Self {
            grid: false,
            bounds: false,
            delta: false,
        }
    }

    pub fn from_csv(input: &str) -> Result<Self, String> {
        let mut config = Self::empty();
        let trimmed = input.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("off") {
            return Ok(config);
        }
        if trimmed.eq_ignore_ascii_case("all") {
            config.grid = true;
            config.bounds = true;
            config.delta = true;
            return Ok(config);
        }
        for token in trimmed.split(',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            match token {
                "grid" => config.grid = true,
                "bounds" => config.bounds = true,
                "delta" => config.delta = true,
                "off" => {}
                other => {
                    return Err(format!("알 수 없는 overlay 토큰: {}", other));
                }
            }
        }
        Ok(config)
    }

    pub fn to_detjson(&self) -> String {
        format!(
            "{{\n  \"kind\": \"bogae_overlay_v1\",\n  \"grid\": {},\n  \"bounds\": {},\n  \"delta\": {}\n}}\n",
            if self.grid { "true" } else { "false" },
            if self.bounds { "true" } else { "false" },
            if self.delta { "true" } else { "false" },
        )
    }
}

pub fn default_bogae_out_dir() -> PathBuf {
    paths::build_dir().join("bogae")
}

pub fn is_bogae_out_dir(path: &Path) -> bool {
    path.is_dir() || path.extension().is_none()
}

pub fn resolve_bogae_out_dir(bogae_out: Option<&Path>) -> PathBuf {
    match bogae_out {
        Some(path) if is_bogae_out_dir(path) => path.to_path_buf(),
        Some(path) => path
            .parent()
            .map(|dir| dir.to_path_buf())
            .unwrap_or_else(default_bogae_out_dir),
        None => default_bogae_out_dir(),
    }
}
