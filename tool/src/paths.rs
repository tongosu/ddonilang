use std::env;
use std::path::PathBuf;

fn configured_or_platform_dir(variable: &str, leaf: &str) -> PathBuf {
    if let Some(configured) = env::var_os(variable).filter(|value| !value.is_empty()) {
        return PathBuf::from(configured);
    }
    let platform_root = env::var_os("LOCALAPPDATA")
        .or_else(|| env::var_os("XDG_CACHE_HOME"))
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(env::temp_dir);
    platform_root.join("ddonirang").join(leaf)
}

pub fn build_dir() -> PathBuf {
    configured_or_platform_dir("DDN_BUILD_DIR", "build")
}

#[allow(dead_code)]
pub fn out_dir() -> PathBuf {
    configured_or_platform_dir("DDN_OUT_DIR", "out")
}
