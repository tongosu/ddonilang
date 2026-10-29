use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::state_trace_wire::escape_json_string_contents as escape_json;

const AUDIT_MAGIC: &[u8; 4] = b"DDNI";
const AUDIT_VERSION: u16 = 1;

pub const DEFAULT_CHECKPOINT_STRIDE: u64 = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceTier {
    Off,
    Patch,
    Alrim,
    Full,
}

impl TraceTier {
    pub fn as_u32(self) -> u32 {
        match self {
            TraceTier::Off => 0,
            TraceTier::Patch => 1,
            TraceTier::Alrim => 2,
            TraceTier::Full => 3,
        }
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(TraceTier::Off),
            1 => Some(TraceTier::Patch),
            2 => Some(TraceTier::Alrim),
            3 => Some(TraceTier::Full),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditHeader {
    pub started_at: u64,
    pub det_tier: u32,
    pub num_backend: u32,
    pub trace_tier: u32,
    pub commit_policy: u32,
}

impl AuditHeader {
    pub fn new(det_tier: u32, trace_tier: u32, num_backend: u32, commit_policy: u32) -> Self {
        Self {
            started_at: 0,
            det_tier,
            num_backend,
            trace_tier,
            commit_policy,
        }
    }
}

pub struct GeoulFramePayload<'a> {
    pub patch: Option<&'a [u8]>,
    pub alrim: Option<&'a [u8]>,
    pub full: Option<&'a [u8]>,
}

#[allow(dead_code)]
pub struct GeoulSummary {
    pub audit_hash: String,
    pub start_madi: u64,
    pub end_madi: u64,
    pub frame_count: u64,
}

pub struct GeoulBundleWriter {
    out_dir: PathBuf,
    audit_path: PathBuf,
    idx_path: PathBuf,
    checkpoint_dir: PathBuf,
    file: File,
    offsets: Vec<u64>,
    hasher: blake3::Hasher,
    bytes_written: u64,
    checkpoint_stride: u64,
    start_madi: Option<u64>,
    end_madi: Option<u64>,
    header: AuditHeader,
    ssot_version: String,
    toolchain_version: String,
    entry_file: Option<String>,
    entry_hash: Option<String>,
    age_target_source: Option<String>,
    age_target_value: Option<String>,
    seulgi_latency_madi: Option<u64>,
    seulgi_latency_drop_policy: Option<String>,
}

impl GeoulBundleWriter {
    pub fn create(
        out_dir: &Path,
        header: AuditHeader,
        checkpoint_stride: u64,
        ssot_version: &str,
        toolchain_version: &str,
    ) -> Result<Self, String> {
        fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
        let checkpoint_dir = out_dir.join("checkpoints");
        fs::create_dir_all(&checkpoint_dir).map_err(|e| e.to_string())?;
        let audit_path = out_dir.join("audit.ddni");
        let idx_path = out_dir.join("audit.idx");
        let mut file = File::create(&audit_path).map_err(|e| e.to_string())?;
        let mut hasher = blake3::Hasher::new();
        let header_bytes = encode_audit_header(&header);
        file.write_all(&header_bytes).map_err(|e| e.to_string())?;
        hasher.update(&header_bytes);
        Ok(Self {
            out_dir: out_dir.to_path_buf(),
            audit_path,
            idx_path,
            checkpoint_dir,
            file,
            offsets: Vec::new(),
            hasher,
            bytes_written: header_bytes.len() as u64,
            checkpoint_stride: checkpoint_stride.max(1),
            start_madi: None,
            end_madi: None,
            header,
            ssot_version: ssot_version.to_string(),
            toolchain_version: toolchain_version.to_string(),
            entry_file: None,
            entry_hash: None,
            age_target_source: None,
            age_target_value: None,
            seulgi_latency_madi: None,
            seulgi_latency_drop_policy: None,
        })
    }

    pub fn audit_path(&self) -> &Path {
        &self.audit_path
    }

    pub fn set_entry(&mut self, entry_file: &str, entry_hash: &str) {
        self.entry_file = Some(entry_file.to_string());
        self.entry_hash = Some(entry_hash.to_string());
    }

    pub fn set_age_target(&mut self, age_target_source: &str, age_target_value: &str) {
        self.age_target_source = Some(age_target_source.to_string());
        self.age_target_value = Some(age_target_value.to_string());
    }

    pub fn set_seulgi_latency_madi(&mut self, seulgi_latency_madi: u64) {
        self.seulgi_latency_madi = Some(seulgi_latency_madi);
    }

    pub fn set_seulgi_latency_drop_policy(&mut self, policy: &str) {
        self.seulgi_latency_drop_policy = Some(policy.to_string());
    }

    pub fn record_frame(
        &mut self,
        madi: u64,
        snapshot_detbin: &[u8],
        state_detbin: &[u8],
        payload: GeoulFramePayload<'_>,
    ) -> Result<(), String> {
        let snapshot_len = u32::try_from(snapshot_detbin.len())
            .map_err(|_| "스냅샷 detbin이 너무 큽니다".to_string())?;
        let patch_len = u32::try_from(payload.patch.map_or(0, |data| data.len()))
            .map_err(|_| "patch blob이 너무 큽니다".to_string())?;
        let alrim_len = u32::try_from(payload.alrim.map_or(0, |data| data.len()))
            .map_err(|_| "alrim blob이 너무 큽니다".to_string())?;
        let full_len = u32::try_from(payload.full.map_or(0, |data| data.len()))
            .map_err(|_| "full blob이 너무 큽니다".to_string())?;
        let state_hash = blake3::hash(state_detbin);
        let mut header = Vec::with_capacity(64);
        header.extend_from_slice(&madi.to_le_bytes());
        header.extend_from_slice(state_hash.as_bytes());
        header.extend_from_slice(&snapshot_len.to_le_bytes());
        header.extend_from_slice(&patch_len.to_le_bytes());
        header.extend_from_slice(&alrim_len.to_le_bytes());
        header.extend_from_slice(&full_len.to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes());

        self.offsets.push(self.bytes_written);
        self.write_all(&header)?;
        self.write_all(snapshot_detbin)?;
        if let Some(patch) = payload.patch {
            self.write_all(patch)?;
        }
        if let Some(alrim) = payload.alrim {
            self.write_all(alrim)?;
        }
        if let Some(full) = payload.full {
            self.write_all(full)?;
        }

        if madi % self.checkpoint_stride == 0 {
            self.write_checkpoint(madi, state_detbin)?;
        }
        if self.start_madi.is_none() {
            self.start_madi = Some(madi);
        }
        self.end_madi = Some(madi);
        Ok(())
    }

    pub fn finish(mut self) -> Result<GeoulSummary, String> {
        self.file.flush().map_err(|e| e.to_string())?;
        let audit_hash = format!("blake3:{}", self.hasher.finalize().to_hex());
        write_idx_file(&self.idx_path, &self.offsets)?;
        let start_madi = self.start_madi.unwrap_or(0);
        let end_madi = self.end_madi.map(|m| m + 1).unwrap_or(0);
        let frame_count = self.offsets.len() as u64;
        let manifest_text = build_manifest_text(
            &self.header,
            &self.ssot_version,
            &self.toolchain_version,
            self.checkpoint_stride,
            start_madi,
            end_madi,
            frame_count,
            self.bytes_written,
            &audit_hash,
            self.entry_file.as_deref(),
            self.entry_hash.as_deref(),
            self.age_target_source.as_deref(),
            self.age_target_value.as_deref(),
            self.seulgi_latency_madi,
            self.seulgi_latency_drop_policy.as_deref(),
        );
        fs::write(self.out_dir.join("manifest.detjson"), manifest_text)
            .map_err(|e| e.to_string())?;
        Ok(GeoulSummary {
            audit_hash,
            start_madi,
            end_madi,
            frame_count,
        })
    }

    fn write_checkpoint(&self, madi: u64, state_detbin: &[u8]) -> Result<(), String> {
        fs::write(
            self.checkpoint_dir.join(format!("cp_{:06}.detbin", madi)),
            state_detbin,
        )
        .map_err(|e| e.to_string())
    }

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.file.write_all(bytes).map_err(|e| e.to_string())?;
        self.hasher.update(bytes);
        self.bytes_written = self.bytes_written.saturating_add(bytes.len() as u64);
        Ok(())
    }
}

#[allow(dead_code)]
pub struct AuditFrameHeader {
    pub madi: u64,
    pub state_hash: [u8; 32],
    pub snapshot_bytes: u32,
    pub patch_bytes: u32,
    pub alrim_bytes: u32,
    pub full_bytes: u32,
}

pub struct GeoulFrame {
    pub header: AuditFrameHeader,
    pub snapshot_detbin: Vec<u8>,
    pub patch_blob: Vec<u8>,
    #[allow(dead_code)]
    pub alrim_blob: Vec<u8>,
    #[allow(dead_code)]
    pub full_blob: Vec<u8>,
}

pub struct GeoulBundleReader {
    file: File,
    offsets: Vec<u64>,
    #[allow(dead_code)]
    header: AuditHeader,
}

impl GeoulBundleReader {
    pub fn open(out_dir: &Path) -> Result<Self, String> {
        let audit_path = out_dir.join("audit.ddni");
        let idx_path = out_dir.join("audit.idx");
        let mut file = File::open(&audit_path).map_err(|e| e.to_string())?;
        let header = read_header(&mut file)?;
        let offsets = read_idx_file(&idx_path)?;
        Ok(Self {
            file,
            offsets,
            header,
        })
    }

    pub fn header(&self) -> &AuditHeader {
        &self.header
    }

    pub fn frame_count(&self) -> u64 {
        self.offsets.len() as u64
    }

    pub fn read_frame_header(&mut self, madi: u64) -> Result<AuditFrameHeader, String> {
        let idx = usize::try_from(madi).map_err(|_| "madi 범위 오류".to_string())?;
        let offset = *self
            .offsets
            .get(idx)
            .ok_or_else(|| "madi 범위가 idx를 벗어났습니다".to_string())?;
        self.file
            .seek(SeekFrom::Start(offset))
            .map_err(|e| e.to_string())?;
        read_frame_header(&mut self.file)
    }

    pub fn read_frame(&mut self, madi: u64) -> Result<GeoulFrame, String> {
        let header = self.read_frame_header(madi)?;
        let snapshot_detbin = read_payload(&mut self.file, header.snapshot_bytes)?;
        let patch_blob = read_payload(&mut self.file, header.patch_bytes)?;
        let alrim_blob = read_payload(&mut self.file, header.alrim_bytes)?;
        let full_blob = read_payload(&mut self.file, header.full_bytes)?;
        Ok(GeoulFrame {
            header,
            snapshot_detbin,
            patch_blob,
            alrim_blob,
            full_blob,
        })
    }
}

pub fn audit_hash(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    let digest = blake3::hash(&buf);
    Ok(format!("blake3:{}", digest.to_hex()))
}

pub fn encode_audit_header(header: &AuditHeader) -> Vec<u8> {
    let mut out = Vec::with_capacity(32);
    out.extend_from_slice(AUDIT_MAGIC);
    out.extend_from_slice(&AUDIT_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&header.started_at.to_le_bytes());
    out.extend_from_slice(&header.det_tier.to_le_bytes());
    out.extend_from_slice(&header.num_backend.to_le_bytes());
    out.extend_from_slice(&header.trace_tier.to_le_bytes());
    out.extend_from_slice(&header.commit_policy.to_le_bytes());
    out
}

fn write_idx_file(path: &Path, offsets: &[u64]) -> Result<(), String> {
    let mut out = Vec::with_capacity(offsets.len() * 8);
    for offset in offsets {
        out.extend_from_slice(&offset.to_le_bytes());
    }
    fs::write(path, out).map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
fn build_manifest_text(
    header: &AuditHeader,
    ssot_version: &str,
    toolchain_version: &str,
    checkpoint_stride: u64,
    start_madi: u64,
    end_madi: u64,
    frame_count: u64,
    audit_size: u64,
    audit_hash: &str,
    entry_file: Option<&str>,
    entry_hash: Option<&str>,
    age_target_source: Option<&str>,
    age_target_value: Option<&str>,
    seulgi_latency_madi: Option<u64>,
    seulgi_latency_drop_policy: Option<&str>,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"kind\": \"geoul_bundle_v1\",\n");
    out.push_str(&format!(
        "  \"ssot_version\": \"{}\",\n",
        escape_json(ssot_version)
    ));
    out.push_str(&format!(
        "  \"toolchain_version\": \"{}\",\n",
        escape_json(toolchain_version)
    ));
    out.push_str(&format!("  \"det_tier\": {},\n", header.det_tier));
    out.push_str(&format!("  \"trace_tier\": {},\n", header.trace_tier));
    out.push_str(&format!("  \"num_backend\": {},\n", header.num_backend));
    out.push_str(&format!(
        "  \"checkpoint_stride\": {},\n",
        checkpoint_stride
    ));
    out.push_str(&format!("  \"start_madi\": {},\n", start_madi));
    out.push_str(&format!("  \"end_madi\": {},\n", end_madi));
    out.push_str(&format!("  \"frame_count\": {},\n", frame_count));
    out.push_str(&format!("  \"audit_size\": {},\n", audit_size));
    out.push_str(&format!(
        "  \"audit_hash\": \"{}\",\n",
        escape_json(audit_hash)
    ));
    if let Some(file) = entry_file {
        out.push_str(&format!("  \"entry_file\": \"{}\",\n", escape_json(file)));
        if let Some(hash) = entry_hash {
            out.push_str(&format!("  \"entry_hash\": \"{}\",\n", escape_json(hash)));
        }
    }
    if let Some(source) = age_target_source {
        out.push_str(&format!(
            "  \"age_target_source\": \"{}\",\n",
            escape_json(source)
        ));
    }
    if let Some(value) = age_target_value {
        out.push_str(&format!(
            "  \"age_target_value\": \"{}\",\n",
            escape_json(value)
        ));
    }
    if let Some(value) = seulgi_latency_madi {
        out.push_str(&format!("  \"seulgi_latency_madi\": {},\n", value));
    }
    if let Some(policy) = seulgi_latency_drop_policy {
        out.push_str(&format!(
            "  \"seulgi_latency_drop_policy\": \"{}\",\n",
            escape_json(policy)
        ));
    }
    out.push_str("  \"audit_file\": \"audit.ddni\",\n");
    out.push_str("  \"index_file\": \"audit.idx\"\n");
    out.push_str("}\n");
    out
}

fn read_header(file: &mut File) -> Result<AuditHeader, String> {
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).map_err(|e| e.to_string())?;
    if &magic != AUDIT_MAGIC {
        return Err("audit.ddni magic 불일치".to_string());
    }
    let version = read_u16(file)?;
    if version != AUDIT_VERSION {
        return Err(format!("audit.ddni version 불일치: {}", version));
    }
    let _reserved = read_u16(file)?;
    let started_at = read_u64(file)?;
    let det_tier = read_u32(file)?;
    let num_backend = read_u32(file)?;
    let trace_tier = read_u32(file)?;
    let commit_policy = read_u32(file)?;
    Ok(AuditHeader {
        started_at,
        det_tier,
        num_backend,
        trace_tier,
        commit_policy,
    })
}

fn read_frame_header(file: &mut File) -> Result<AuditFrameHeader, String> {
    let madi = read_u64(file)?;
    let mut state_hash = [0u8; 32];
    file.read_exact(&mut state_hash)
        .map_err(|e| e.to_string())?;
    let snapshot_bytes = read_u32(file)?;
    let patch_bytes = read_u32(file)?;
    let alrim_bytes = read_u32(file)?;
    let full_bytes = read_u32(file)?;
    let _reserved = read_u32(file)?;
    Ok(AuditFrameHeader {
        madi,
        state_hash,
        snapshot_bytes,
        patch_bytes,
        alrim_bytes,
        full_bytes,
    })
}

fn read_idx_file(path: &Path) -> Result<Vec<u64>, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    if buf.len() % 8 != 0 {
        return Err("audit.idx 길이가 8의 배수가 아닙니다".to_string());
    }
    let mut offsets = Vec::with_capacity(buf.len() / 8);
    for chunk in buf.chunks_exact(8) {
        offsets.push(u64::from_le_bytes([
            chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
        ]));
    }
    Ok(offsets)
}

fn read_payload(file: &mut File, len: u32) -> Result<Vec<u8>, String> {
    if len == 0 {
        return Ok(Vec::new());
    }
    let mut buf = vec![0u8; len as usize];
    file.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

fn read_u16(file: &mut File) -> Result<u16, String> {
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u32(file: &mut File) -> Result<u32, String> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64(file: &mut File) -> Result<u64, String> {
    let mut buf = [0u8; 8];
    file.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(u64::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::{
        build_manifest_text, encode_audit_header, AuditHeader, GeoulBundleReader,
        GeoulBundleWriter, GeoulFramePayload,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NONCE: AtomicU64 = AtomicU64::new(0);

    fn test_dir(label: &str) -> PathBuf {
        let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "ddn_geoul_bundle_{label}_{}_{}",
            process::id(),
            nonce
        ))
    }

    #[test]
    fn reader_decodes_shared_header_and_frame_payloads() {
        let dir = test_dir("roundtrip");
        let header = AuditHeader::new(1, 2, 3, 4);
        let state = b"state";
        let mut writer = GeoulBundleWriter::create(&dir, header.clone(), 256, "25.47.0", "0.1.0")
            .expect("create bundle");
        writer
            .record_frame(
                7,
                b"sam",
                state,
                GeoulFramePayload {
                    patch: Some(b"pa"),
                    alrim: Some(b"a"),
                    full: None,
                },
            )
            .expect("record frame");
        let summary = writer.finish().expect("finish bundle");
        assert_eq!(summary.frame_count, 1);

        let mut reader = GeoulBundleReader::open(&dir).expect("open bundle");
        assert_eq!(reader.header(), &header);
        assert_eq!(reader.frame_count(), 1);
        let frame = reader.read_frame(0).expect("read frame");
        assert_eq!(frame.header.madi, 7);
        assert_eq!(frame.header.state_hash, *blake3::hash(state).as_bytes());
        assert_eq!(frame.snapshot_detbin, b"sam");
        assert_eq!(frame.patch_blob, b"pa");
        assert_eq!(frame.alrim_blob, b"a");
        assert!(frame.full_blob.is_empty());

        fs::remove_dir_all(&dir).expect("remove test dir");
    }

    #[test]
    fn malformed_index_fails_closed() {
        let dir = test_dir("bad_index");
        fs::create_dir_all(&dir).expect("create test dir");
        fs::write(
            dir.join("audit.ddni"),
            encode_audit_header(&AuditHeader::new(0, 0, 1, 0)),
        )
        .expect("write audit");
        fs::write(dir.join("audit.idx"), [0u8]).expect("write malformed index");

        let error = GeoulBundleReader::open(&dir)
            .err()
            .expect("malformed index must fail closed");
        assert_eq!(error, "audit.idx 길이가 8의 배수가 아닙니다");

        fs::remove_dir_all(&dir).expect("remove test dir");
    }

    #[test]
    fn manifest_includes_seulgi_latency_madi_when_set() {
        let header = AuditHeader::new(0, 0, 1, 0);
        let text = build_manifest_text(
            &header,
            "21.0.0",
            "0.1.0",
            256,
            0,
            3,
            3,
            1234,
            "blake3:abc",
            Some("entry.ddn"),
            Some("blake3:def"),
            Some("flag"),
            Some("age3"),
            Some(5),
            Some("late_drop"),
        );
        assert!(text.contains("\"seulgi_latency_madi\": 5"));
        assert!(text.contains("\"seulgi_latency_drop_policy\": \"late_drop\""));
    }

    #[test]
    fn manifest_omits_seulgi_latency_madi_when_unset() {
        let header = AuditHeader::new(0, 0, 1, 0);
        let text = build_manifest_text(
            &header,
            "21.0.0",
            "0.1.0",
            256,
            0,
            3,
            3,
            1234,
            "blake3:abc",
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(!text.contains("\"seulgi_latency_madi\""));
        assert!(!text.contains("\"seulgi_latency_drop_policy\""));
    }
}
