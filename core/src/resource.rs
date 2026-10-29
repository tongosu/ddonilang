use xxhash_rust::xxh3::{xxh3_64, xxh3_64_with_seed};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceHandle(u64);

impl ResourceHandle {
    pub fn from_path(path: &str) -> Self {
        Self(xxh3_64(path.as_bytes()))
    }

    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub fn raw(self) -> u64 {
        self.0
    }

    pub fn to_hex(self) -> String {
        format!("{:016x}", self.0)
    }
}

pub fn asset_handle_from_bundle_path(bundle_id: &str, path: &str) -> ResourceHandle {
    let key = format!("{bundle_id}::{path}");
    ResourceHandle(xxh3_64_with_seed(key.as_bytes(), 0))
}
