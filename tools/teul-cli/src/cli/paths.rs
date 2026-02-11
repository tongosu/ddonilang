use std::path::{Path, PathBuf};

const PREFERRED_BUILD_DIR: &str = "I:/home/urihanl/ddn/codex/build";
const FALLBACK_BUILD_DIR: &str = "C:/ddn/codex/build";
const PREFERRED_OUT_DIR: &str = "I:/home/urihanl/ddn/codex/out";
const FALLBACK_OUT_DIR: &str = "C:/ddn/codex/out";

fn pick_dir(preferred: &str, fallback: &str) -> PathBuf {
    let preferred_path = Path::new(preferred);
    if preferred_path.is_dir() {
        preferred_path.to_path_buf()
    } else {
        PathBuf::from(fallback)
    }
}

pub fn build_dir() -> PathBuf {
    pick_dir(PREFERRED_BUILD_DIR, FALLBACK_BUILD_DIR)
}

#[allow(dead_code)]
pub fn out_dir() -> PathBuf {
    pick_dir(PREFERRED_OUT_DIR, FALLBACK_OUT_DIR)
}
