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
mod native_source;

/// CLI and the same-origin workbench share the exact closed Native request
/// frontdoor. A JSON request supplies locators only, never a run permit.
pub fn native_source_operation(request:&str)->Result<String,String>{
    native_source::NativeSourceController::start().call(request.into())
}

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
        Self::start_on_port(asset_root, manifest_path, 0)
    }

    pub fn start_on_port(asset_root: &Path, manifest_path: &Path, port: u16) -> Result<Self, String> {
        let (assets, manifest_sha256) = verified_assets(asset_root, manifest_path)?;
        // Choose before binding: never transiently occupy a user server port
        // (in particular 8790) or a browser-restricted low port.
        if port == 8790 { return Err("E_WORKBENCH_PORT_RESERVED".into()); }
        let listener = if port == 0 { bind_loopback()? } else {
            TcpListener::bind(("127.0.0.1", port))
                .map_err(|error| format!("E_WORKBENCH_LOOPBACK_BIND {error}"))?
        };
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
        let native_source = native_source::NativeSourceController::start();
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
                        let native_source = native_source.clone();
                        thread::spawn(move || {
                            let _ = handle_request_with_native(stream, &request_root, &request_assets, &request_token,Some(&native_source));
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

fn unavailable_automatic_port(error:&std::io::Error)->bool {
    error.kind()==std::io::ErrorKind::AddrInUse
        // Windows excludes some otherwise valid dynamic ports. Only the
        // automatic allocator retries another candidate; an explicitly
        // requested port still returns its original permission failure.
        || cfg!(windows) && error.raw_os_error()==Some(10013)
}

fn bind_loopback() -> Result<TcpListener, String> {
    for _ in 0..128 {
        let mut bytes = [0u8; 2];
        getrandom::fill(&mut bytes).map_err(|error| format!("E_WORKBENCH_TOKEN_RANDOM {error}"))?;
        let port = 20000 + u16::from_le_bytes(bytes) % 40000;
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(listener) => return Ok(listener),
            Err(error) if unavailable_automatic_port(&error) => continue,
            Err(error) => return Err(format!("E_WORKBENCH_LOOPBACK_BIND {error}")),
        }
    }
    Err("E_WORKBENCH_LOOPBACK_BIND: 사용 가능한 포트가 없습니다".into())
}

// Decode a URL once, then validate the resulting path before manifest lookup.
// In particular, encoded separators cannot bypass the manifest boundary.
fn decode_asset_path(path: &str) -> Result<String, String> {
    let mut decoded = Vec::with_capacity(path.len());
    let bytes = path.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'%' {
            let pair = bytes.get(at + 1..at + 3).ok_or("E_WORKBENCH_ROUTE_DENIED")?;
            let digit = |value: u8| match value {
                b'0'..=b'9' => Some(value - b'0'),
                b'a'..=b'f' => Some(value - b'a' + 10),
                b'A'..=b'F' => Some(value - b'A' + 10),
                _ => None,
            };
            let value = digit(pair[0]).zip(digit(pair[1]))
                .map(|(high, low)| high * 16 + low).ok_or("E_WORKBENCH_ROUTE_DENIED")?;
            if matches!(value, 0 | b'/' | b'\\' | b'%') { return Err("E_WORKBENCH_ROUTE_DENIED".into()); }
            decoded.push(value);
            at += 3;
        } else {
            decoded.push(bytes[at]);
            at += 1;
        }
    }
    String::from_utf8(decoded).map_err(|_| "E_WORKBENCH_ROUTE_DENIED".into())
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

fn handle_request(stream: TcpStream, root: &Path, assets: &std::collections::BTreeMap<String, WorkbenchAsset>, token: &str) -> Result<(), String> {
    handle_request_with_native(stream,root,assets,token,None)
}
fn single_header<'a>(headers:&'a str,wanted:&str)->Result<Option<&'a str>,String>{
    let mut values=headers.lines().skip(1).filter_map(|line|line.split_once(':'))
        .filter(|(name,_)|name.eq_ignore_ascii_case(wanted)).map(|(_,value)|value.trim());
    let value=values.next();if values.next().is_some(){return Err("E_WORKBENCH_DUPLICATE_HEADER".into());}Ok(value)
}
fn handle_request_with_native(mut stream: TcpStream, root: &Path, assets: &std::collections::BTreeMap<String, WorkbenchAsset>, token: &str,
    native:Option<&native_source::NativeSourceController>) -> Result<(), String> {
    // Windows accept inherits the nonblocking listener mode. This handler
    // uses blocking read/write_all, so restore that mode on this connection
    // before a fragmented header or a backpressured response can WouldBlock.
    stream
        .set_nonblocking(false)
        .map_err(|error| format!("E_WORKBENCH_REQUEST_MODE {error}"))?;
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
    let header_end=request.windows(4).position(|window|window==b"\r\n\r\n").ok_or("E_WORKBENCH_REQUEST_EMPTY")?+4;
    let headers = String::from_utf8(request[..header_end].to_vec()).map_err(|_| "E_WORKBENCH_REQUEST_UTF8".to_string())?;
    let line = headers.lines().next().ok_or("E_WORKBENCH_REQUEST_EMPTY")?;
    let mut fields = line.split_whitespace();
    let method = fields.next().unwrap_or("");
    let target = fields.next().unwrap_or("");
    if !matches!(method,"GET"|"POST") || fields.next()!=Some("HTTP/1.1")||fields.next().is_some() { return respond(&mut stream, 405, "text/plain", b"method denied", None); }
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let query_authorized = query.split('&').any(|field| field == format!("token={token}"));
    let cookie_authorized = headers.lines().skip(1).any(|line| {
        line.split_once(':').is_some_and(|(name, value)| {
            name.trim().eq_ignore_ascii_case("cookie")
                && value.split(';').any(|field| {
                    field.trim() == format!("ddn_workbench_token={token}")
                })
        })
    });
    if method=="POST" {
        let port=stream.local_addr().map_err(|_|"E_WORKBENCH_LOCAL_ADDRESS")?.port();
        let expected_host=format!("127.0.0.1:{port}");let expected_origin=format!("http://{expected_host}");
        // Mutating transport never accepts a query token or cross-origin form.
        if !cookie_authorized||single_header(&headers,"cookie")?.is_none()
            ||single_header(&headers,"host")?!=Some(expected_host.as_str())
            ||single_header(&headers,"origin")?!=Some(expected_origin.as_str()) {
            return respond(&mut stream,403,"text/plain",b"same-origin cookie required",None);
        }
        if path!="/__ddn/vc04/source"||!query.is_empty(){return respond(&mut stream,404,"text/plain",b"route denied",None);}
        if single_header(&headers,"transfer-encoding")?.is_some()
            ||single_header(&headers,"content-type")?!=Some("application/json"){
            return respond(&mut stream,400,"text/plain",b"body framing denied",None);
        }
        let length=single_header(&headers,"content-length")?.and_then(|v|v.parse::<usize>().ok())
            .ok_or("E_WORKBENCH_BODY_LENGTH")?;
        if length>8*1024*1024{return respond(&mut stream,413,"text/plain",b"body too large",None);}
        while request.len()-header_end<length {
            let count=stream.read(&mut chunk).map_err(|_|"E_WORKBENCH_BODY_READ")?;
            if count==0{return Err("E_WORKBENCH_BODY_TRUNCATED".into());}request.extend_from_slice(&chunk[..count]);
        }
        if request.len()-header_end!=length{return respond(&mut stream,400,"text/plain",b"body framing denied",None);}
        let body=String::from_utf8(request[header_end..].to_vec()).map_err(|_|"E_WORKBENCH_BODY_UTF8")?;
        let result=native.ok_or("E_NATIVE_SOURCE_CONTROLLER_UNAVAILABLE")?.call(body);
        return match result {
            Ok(body)=>respond(&mut stream,200,"application/json; charset=utf-8",body.as_bytes(),None),
            Err(error)=>respond(&mut stream,409,"application/json; charset=utf-8",serde_json::json!({"error":error}).to_string().as_bytes(),None),
        };
    }
    if !query_authorized && !cookie_authorized { return respond(&mut stream, 403, "text/plain", b"token denied", None); }
    if path=="/__ddn/vc04/source"&&native.is_some(){
        return respond(&mut stream,200,"application/json",b"{\"native_source\":true}",None);
    }
    let decoded = match decode_asset_path(path) {
        Ok(path) => path,
        Err(_) => return respond(&mut stream, 404, "text/plain", b"asset denied", None),
    };
    let relative = match canonical_asset_path(&decoded) {
        Ok(path) => path,
        Err(_) => return respond(&mut stream, 404, "text/plain", b"asset denied", None),
    };
    let Some(asset) = assets.get(relative) else { return respond(&mut stream, 404, "text/plain", b"asset denied", None); };
    let body = fs::read(root.join(&asset.path)).map_err(|_| "E_WORKBENCH_ASSET_READ".to_string())?;
    let set_cookie = if query_authorized && path == "/" { Some(format!("ddn_workbench_token={token}; HttpOnly; SameSite=Strict; Path=/")) } else { None };
    respond_with_native_marker(&mut stream, 200, &asset.content_type, &body, set_cookie.as_deref(),native.is_some()&&cfg!(windows))
}

fn respond(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8], set_cookie: Option<&str>) -> Result<(), String> {
    respond_with_native_marker(stream,status,content_type,body,set_cookie,false)
}
fn respond_with_native_marker(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8], set_cookie: Option<&str>,native:bool) -> Result<(), String> {
    let phrase = match status { 200 => "OK",400=>"Bad Request",409=>"Conflict", 403 => "Forbidden", 404 => "Not Found", 405 => "Method Not Allowed", 413 => "Payload Too Large", _ => "Error" };
    let cookie = set_cookie.map(|value| format!("Set-Cookie: {value}\r\n")).unwrap_or_default();
    let marker=if native{"X-DDN-Native-Source: v1\r\n"}else{""};
    write!(stream, "HTTP/1.1 {status} {phrase}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\n{cookie}{marker}Connection: close\r\n\r\n", body.len()).map_err(|error| format!("E_WORKBENCH_RESPONSE_WRITE {error}"))?;
    stream.write_all(body).map_err(|error| format!("E_WORKBENCH_RESPONSE_WRITE {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn native_mutation_requires_cookie_origin_and_unambiguous_framing(){
        for (origin,cookie,extra) in [
            ("http://foreign.invalid","ddn_workbench_token=actual",""),
            ("SELF","wrong=actual",""),
            ("SELF","ddn_workbench_token=actual","Transfer-Encoding: chunked\r\n"),
        ] {
            let listener=bind_loopback().unwrap();let address=listener.local_addr().unwrap();
            let mut client=TcpStream::connect(address).unwrap();
            let (server,_)=listener.accept().unwrap();
            let worker=thread::spawn(move||handle_request_with_native(server,Path::new("."),&Default::default(),"actual",None));
            let origin=if origin=="SELF"{format!("http://{address}")}else{origin.into()};
            write!(client,"POST /__ddn/vc04/source HTTP/1.1\r\nHost: {address}\r\nOrigin: {origin}\r\nCookie: {cookie}\r\nContent-Type: application/json\r\nContent-Length: 2\r\n{extra}\r\n{{}}").unwrap();
            let mut response=String::new();client.read_to_string(&mut response).unwrap();worker.join().unwrap().unwrap();
            assert!(!response.starts_with("HTTP/1.1 200"));
        }
        assert!(single_header("POST / HTTP/1.1\r\nContent-Length: 2\r\nContent-Length: 3\r\n","content-length").is_err());
    }

    #[test]
    fn automatic_port_retries_only_unavailable_candidates() {
        use std::io::{Error,ErrorKind};
        assert!(unavailable_automatic_port(&Error::from(ErrorKind::AddrInUse)));
        assert_eq!(unavailable_automatic_port(&Error::from_raw_os_error(10013)),cfg!(windows));
        for kind in [ErrorKind::InvalidInput,ErrorKind::AddrNotAvailable,ErrorKind::Interrupted] {
            assert!(!unavailable_automatic_port(&Error::from(kind)));
        }
    }

    #[test]
    fn accepted_socket_completes_fragmented_headers_and_backpressured_body() {
        let root = std::env::temp_dir().join(format!("ddn-workbench-fragmented-{}-{}",
            std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&root).unwrap();
        let body = vec![b'x'; 2 * 1024 * 1024];
        fs::write(root.join("index.html"), &body).unwrap();
        let manifest = serde_json::json!({"schema":"ddonirang.workbench_asset_manifest.v1","files":[{
            "path":"index.html","bytes":body.len(),"sha256":hex::encode(Sha256::digest(&body)),
            "content_type":"text/html","role":"entry"}]});
        let manifest_path = root.join("asset_manifest.detjson");
        fs::write(&manifest_path, manifest.to_string()).unwrap();
        let (assets, _) = verified_assets(&root, &manifest_path).unwrap();
        let listener = bind_loopback().unwrap();
        listener.set_nonblocking(true).unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        client.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        client.write_all(b"GET /?token=fragmented HTTP/1.1\r\nHost: localhost\r\n").unwrap();
        let (server, _) = listener.accept().unwrap();
        let worker = thread::spawn(move || handle_request(server, &root, &assets, "fragmented"));
        // No fixture changes the host's stream mode. A partial header must
        // wait for the rest, and the large response must tolerate a slow reader.
        thread::sleep(Duration::from_millis(50));
        let tail = client.write_all(b"Accept: */*\r\n\r\n");
        thread::sleep(Duration::from_millis(50));
        let mut response = Vec::new();
        let read = client.read_to_end(&mut response);
        let served = worker.join().unwrap();
        assert!(served.is_ok(), "fragmented request failed: {served:?}; tail={tail:?}; read={read:?}");
        tail.unwrap(); read.unwrap();
        assert!(response.starts_with(b"HTTP/1.1 200"));
        assert!(response.ends_with(&body), "complete response body required");
        // Keep this owned fixture for failure inspection; no broad cleanup.
    }

    #[test]
    fn korean_asset_urls_decode_once_without_path_escape() {
        assert_eq!(decode_asset_path("/first_steps/%EA%B2%80%EC%A6%9D.json").unwrap(), "/first_steps/검증.json");
        assert_eq!(decode_asset_path("/a+b.ddn").unwrap(), "/a+b.ddn");
        for path in ["/%", "/%GG", "/%FF", "/%00", "/%2fsecret", "/%5csecret", "/%252e%252e/secret"] {
            assert!(decode_asset_path(path).is_err(), "{path}");
        }
        let escaped = decode_asset_path("/%2e%2e/secret").unwrap();
        assert!(canonical_asset_path(&escaped).is_err());
        let listener = bind_loopback().unwrap();
        assert!((20000..60000).contains(&listener.local_addr().unwrap().port()));
    }

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
        fs::write(root.join("예제.ddn"), "1 보여주기.").unwrap();
        let body = fs::read(root.join("index.html")).unwrap();
        let styles = fs::read(root.join("styles.css")).unwrap();
        let korean = fs::read(root.join("예제.ddn")).unwrap();
        let manifest = serde_json::json!({
            "schema": "ddonirang.workbench_asset_manifest.v1",
            "files": [
                {"path":"index.html","bytes":body.len(),"sha256":hex::encode(Sha256::digest(&body)),"content_type":"text/html","role":"entry"},
                {"path":"styles.css","bytes":styles.len(),"sha256":hex::encode(Sha256::digest(&styles)),"content_type":"text/css","role":"runtime"},
                {"path":"예제.ddn","bytes":korean.len(),"sha256":hex::encode(Sha256::digest(&korean)),"content_type":"text/plain; charset=utf-8","role":"runtime"}
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
        for (url, expected) in [("/%EC%98%88%EC%A0%9C.ddn", 200), ("/%2e%2e/secret", 404), ("/%FF", 404), ("/not-in-manifest", 404)] {
            let mut stream = TcpStream::connect(("127.0.0.1", host.port())).unwrap();
            stream.write_all(format!("GET {url} HTTP/1.1\r\nHost: localhost\r\nCookie: ddn_workbench_token={}\r\n\r\n", host.token()).as_bytes()).unwrap();
            let mut response = String::new(); stream.read_to_string(&mut response).unwrap();
            assert!(response.starts_with(&format!("HTTP/1.1 {expected}")), "{url}: {response}");
            if expected == 200 { assert!(response.ends_with("1 보여주기.")); }
        }
        drop(host);
        let _ = fs::remove_dir_all(root);
    }
}
