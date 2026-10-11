//! Loopback-only static host for the sealed offline Workbench bundle.

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Component, Path};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const MAX_REQUEST_BYTES: usize = 16 * 1024;

#[derive(Debug, Deserialize)]
pub struct WorkbenchAssetManifest {
    pub schema: String,
    pub files: Vec<WorkbenchAsset>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WorkbenchAsset {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
    pub content_type: String,
    pub role: String,
}

#[derive(Debug)]
pub struct WorkbenchHost {
    port: u16,
    token: String,
    manifest_sha256: String,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl WorkbenchHost {
    pub fn start(asset_root: &Path, manifest_path: &Path) -> Result<Self, String> {
        let (assets, manifest_sha256) = verified_assets(asset_root, manifest_path)?;
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|error| format!("E_WORKBENCH_LOOPBACK_BIND {error}"))?;
        listener
            .set_nonblocking(true)
            .map_err(|error| format!("E_WORKBENCH_LOOPBACK_NONBLOCKING {error}"))?;
        let port = listener
            .local_addr()
            .map_err(|error| format!("E_WORKBENCH_LOOPBACK_PORT {error}"))?
            .port();
        let token = session_token()?;
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let thread_token = token.clone();
        let root = asset_root.to_path_buf();
        let thread = thread::spawn(move || {
            while !thread_stop.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, address)) => {
                        if !address.ip().is_loopback() {
                            let _ = stream.shutdown(Shutdown::Both);
                            continue;
                        }
                        let request_root = root.clone();
                        let request_assets = assets.clone();
                        let request_token = thread_token.clone();
                        thread::spawn(move || {
                            let _ = handle_request(stream, &request_root, &request_assets, &request_token);
                        });
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self { port, token, manifest_sha256, stop, thread: Some(thread) })
    }

    pub fn port(&self) -> u16 { self.port }
    pub fn token(&self) -> &str { &self.token }
    pub fn url(&self) -> String { format!("http://127.0.0.1:{}/?token={}", self.port, self.token) }
    pub fn startup_url(&self) -> String { format!("http://127.0.0.1:{}/", self.port) }
    pub fn manifest_sha256(&self) -> &str { &self.manifest_sha256 }
}

impl Drop for WorkbenchHost {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = TcpStream::connect(("127.0.0.1", self.port));
        if let Some(thread) = self.thread.take() { let _ = thread.join(); }
    }
}

fn session_token() -> Result<String, String> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|error| format!("E_WORKBENCH_TOKEN_RANDOM {error}"))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn verified_assets(root: &Path, manifest_path: &Path) -> Result<(std::collections::BTreeMap<String, WorkbenchAsset>, String), String> {
    let raw_bytes = fs::read(manifest_path).map_err(|error| format!("E_WORKBENCH_ASSET_MANIFEST_READ {error}"))?;
    let raw = String::from_utf8(raw_bytes.clone()).map_err(|_| "E_WORKBENCH_ASSET_MANIFEST_UTF8".to_string())?;
    let manifest: WorkbenchAssetManifest = serde_json::from_str(&raw).map_err(|error| format!("E_WORKBENCH_ASSET_MANIFEST_PARSE {error}"))?;
    if manifest.schema != "ddonirang.workbench_asset_manifest.v1" { return Err("E_WORKBENCH_ASSET_MANIFEST_SCHEMA".into()); }
    let mut assets = std::collections::BTreeMap::new();
    for asset in manifest.files {
        if !valid_asset_path(&asset.path) || assets.contains_key(&asset.path) { return Err("E_WORKBENCH_ASSET_PATH_INVALID".into()); }
        let path = root.join(&asset.path);
        let metadata = fs::symlink_metadata(&path).map_err(|_| "E_WORKBENCH_ASSET_MISSING".to_string())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() { return Err("E_WORKBENCH_ASSET_PATH_INVALID".into()); }
        let bytes = fs::read(&path).map_err(|_| "E_WORKBENCH_ASSET_READ".to_string())?;
        if bytes.len() as u64 != asset.bytes || hex::encode(Sha256::digest(&bytes)) != asset.sha256 { return Err("E_WORKBENCH_ASSET_HASH_MISMATCH".into()); }
        assets.insert(asset.path.clone(), asset);
    }
    let mut observed = std::collections::BTreeSet::new();
    collect_asset_paths(root, root, &mut observed)?;
    let declared = assets.keys().cloned().collect::<std::collections::BTreeSet<_>>();
    if observed != declared { return Err("E_WORKBENCH_ASSET_MANIFEST_COVERAGE".into()); }
    if !assets.contains_key("index.html") { return Err("E_WORKBENCH_INDEX_MISSING".into()); }
    Ok((assets, hex::encode(Sha256::digest(&raw_bytes))))
}

fn collect_asset_paths(root: &Path, current: &Path, observed: &mut std::collections::BTreeSet<String>) -> Result<(), String> {
    for entry in fs::read_dir(current).map_err(|_| "E_WORKBENCH_ASSET_READ".to_string())? {
        let entry = entry.map_err(|_| "E_WORKBENCH_ASSET_READ".to_string())?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|_| "E_WORKBENCH_ASSET_READ".to_string())?;
        if file_type.is_symlink() { return Err("E_WORKBENCH_ASSET_PATH_INVALID".into()); }
        if file_type.is_dir() {
            collect_asset_paths(root, &path, observed)?;
        } else if file_type.is_file() {
            let relative = path.strip_prefix(root).map_err(|_| "E_WORKBENCH_ASSET_PATH_INVALID".to_string())?.to_string_lossy().replace('\\', "/");
            if relative != "asset_manifest.detjson" && valid_asset_path(&relative) { observed.insert(relative); } else if relative != "asset_manifest.detjson" { return Err("E_WORKBENCH_ASSET_PATH_INVALID".into()); }
        } else {
            return Err("E_WORKBENCH_ASSET_PATH_INVALID".into());
        }
    }
    Ok(())
}

fn valid_asset_path(value: &str) -> bool {
    !value.is_empty() && Path::new(value).components().all(|part| matches!(part, Component::Normal(_))) && !value.contains('\\')
}

fn canonical_asset_path(path: &str) -> Result<&str, String> {
    let relative = if path == "/" {
        "index.html"
    } else {
        path.strip_prefix('/').ok_or("E_WORKBENCH_ROUTE_DENIED")?
    };
    // The tracked Workbench retains repository-relative fallback URLs.  In an
    // extracted candidate those URLs name the same manifest-owned assets, not
    // a second source tree or a host filesystem path.
    let relative = relative.strip_prefix("solutions/seamgrim_ui_mvp/").unwrap_or(relative);
    if !valid_asset_path(relative) {
        return Err("E_WORKBENCH_ROUTE_DENIED".into());
    }
    Ok(relative)
}

fn handle_request(mut stream: TcpStream, root: &Path, assets: &std::collections::BTreeMap<String, WorkbenchAsset>, token: &str) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("E_WORKBENCH_REQUEST_TIMEOUT {error}"))?;
    let mut request = Vec::with_capacity(1024);
    let mut chunk = [0_u8; 1024];
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        let count = stream
            .read(&mut chunk)
            .map_err(|error| format!("E_WORKBENCH_REQUEST_READ {error}"))?;
        if count == 0 { return Err("E_WORKBENCH_REQUEST_EMPTY".into()); }
        if request.len().saturating_add(count) > MAX_REQUEST_BYTES {
            return respond(&mut stream, 413, "text/plain", b"request too large", None);
        }
        request.extend_from_slice(&chunk[..count]);
    }
    let request = String::from_utf8(request).map_err(|_| "E_WORKBENCH_REQUEST_UTF8".to_string())?;
    let line = request.lines().next().ok_or("E_WORKBENCH_REQUEST_EMPTY")?;
    let mut fields = line.split_whitespace();
    let method = fields.next().unwrap_or("");
    let target = fields.next().unwrap_or("");
    if method != "GET" || fields.next().is_none() { return respond(&mut stream, 405, "text/plain", b"method denied", None); }
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let query_authorized = query.split('&').any(|field| field == format!("token={token}"));
    let cookie_authorized = request.lines().skip(1).any(|line| {
        line.split_once(':').is_some_and(|(name, value)| {
            name.trim().eq_ignore_ascii_case("cookie")
                && value.split(';').any(|field| {
                    field.trim() == format!("ddn_workbench_token={token}")
                })
        })
    });
    if !query_authorized && !cookie_authorized { return respond(&mut stream, 403, "text/plain", b"token denied", None); }
    let relative = canonical_asset_path(path)?;
    let Some(asset) = assets.get(relative) else { return respond(&mut stream, 404, "text/plain", b"asset denied", None); };
    let body = fs::read(root.join(&asset.path)).map_err(|_| "E_WORKBENCH_ASSET_READ".to_string())?;
    let set_cookie = if query_authorized && path == "/" { Some(format!("ddn_workbench_token={token}; HttpOnly; SameSite=Strict; Path=/")) } else { None };
    respond(&mut stream, 200, &asset.content_type, &body, set_cookie.as_deref())
}

fn respond(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8], set_cookie: Option<&str>) -> Result<(), String> {
    let phrase = match status { 200 => "OK", 403 => "Forbidden", 404 => "Not Found", 405 => "Method Not Allowed", 413 => "Payload Too Large", _ => "Error" };
    let cookie = set_cookie.map(|value| format!("Set-Cookie: {value}\r\n")).unwrap_or_default();
    write!(stream, "HTTP/1.1 {status} {phrase}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\n{cookie}Connection: close\r\n\r\n", body.len()).map_err(|error| format!("E_WORKBENCH_RESPONSE_WRITE {error}"))?;
    stream.write_all(body).map_err(|error| format!("E_WORKBENCH_RESPONSE_WRITE {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loopback_host_requires_token_and_serves_only_manifest_assets() {
        assert_eq!(canonical_asset_path("/solutions/seamgrim_ui_mvp/styles.css").unwrap(), "styles.css");
        assert!(canonical_asset_path("/solutions/seamgrim_ui_mvp/../secret").is_err());
        let root = std::env::temp_dir().join(format!(
            "ddonirang-workbench-host-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("index.html"), "<main>offline</main>").unwrap();
        fs::write(root.join("styles.css"), "main{display:block}").unwrap();
        let body = fs::read(root.join("index.html")).unwrap();
        let styles = fs::read(root.join("styles.css")).unwrap();
        let manifest = serde_json::json!({
            "schema": "ddonirang.workbench_asset_manifest.v1",
            "files": [
                {"path":"index.html","bytes":body.len(),"sha256":hex::encode(Sha256::digest(&body)),"content_type":"text/html","role":"entry"},
                {"path":"styles.css","bytes":styles.len(),"sha256":hex::encode(Sha256::digest(&styles)),"content_type":"text/css","role":"runtime"}
            ]
        });
        let manifest_path = root.join("asset_manifest.detjson");
        fs::write(&manifest_path, manifest.to_string()).unwrap();
        let host = WorkbenchHost::start(&root, &manifest_path).unwrap();
        let mut denied = TcpStream::connect(("127.0.0.1", host.port())).unwrap();
        denied.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();
        let mut denied_text = String::new(); denied.read_to_string(&mut denied_text).unwrap();
        assert!(denied_text.starts_with("HTTP/1.1 403"));
        let mut accepted = TcpStream::connect(("127.0.0.1", host.port())).unwrap();
        accepted.write_all(format!("GET /?token={} HTTP/1.1\r\nHost: localhost\r\n\r\n", host.token()).as_bytes()).unwrap();
        let mut accepted_text = String::new(); accepted.read_to_string(&mut accepted_text).unwrap();
        assert!(accepted_text.starts_with("HTTP/1.1 200"));
        assert!(accepted_text.contains("Cache-Control: no-store"));
        assert!(accepted_text.contains("Set-Cookie: ddn_workbench_token="));
        let mut cookie_asset = TcpStream::connect(("127.0.0.1", host.port())).unwrap();
        cookie_asset
            .write_all(
                format!(
                    "GET /styles.css HTTP/1.1\r\nHost: localhost\r\nCookie: ddn_workbench_token={}\r\n\r\n",
                    host.token()
                )
                .as_bytes(),
            )
            .unwrap();
        let mut cookie_text = String::new();
        cookie_asset.read_to_string(&mut cookie_text).unwrap();
        assert!(cookie_text.starts_with("HTTP/1.1 200"));
        let mut lowercase_cookie_asset = TcpStream::connect(("127.0.0.1", host.port())).unwrap();
        lowercase_cookie_asset
            .write_all(
                format!(
                    "GET /styles.css HTTP/1.1\r\nHost: localhost\r\ncookie: ddn_workbench_token={}\r\n\r\n",
                    host.token()
                )
                .as_bytes(),
            )
            .unwrap();
        lowercase_cookie_asset
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut lowercase_cookie_text = String::new();
        lowercase_cookie_asset.read_to_string(&mut lowercase_cookie_text).unwrap();
        assert!(lowercase_cookie_text.starts_with("HTTP/1.1 200"));
        drop(host);
        let _ = fs::remove_dir_all(root);
    }
}
