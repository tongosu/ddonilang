use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use blake3;

use crate::core::detbin::encode_state;
use crate::core::state::State;

const AUDIT_MAGIC: &[u8; 4] = b"DDNI";
const AUDIT_VERSION: u16 = 1;
const SNAPSHOT_MAGIC: &[u8; 11] = b"DDN_SAM_V1\n";

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

    #[allow(dead_code)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct NetEventV1 {
    pub sender: String,
    pub seq: u64,
    pub order_key: String,
    pub payload: String,
}

#[derive(Clone, Debug)]
pub struct InputSnapshotV1 {
    pub madi: u64,
    pub held_mask: u16,
    pub pressed_mask: u16,
    pub released_mask: u16,
    pub rng_seed: u64,
    pub net_events: Vec<NetEventV1>,
}

pub fn encode_input_snapshot(snapshot: &InputSnapshotV1) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(SNAPSHOT_MAGIC);
    out.extend_from_slice(&snapshot.madi.to_le_bytes());
    out.extend_from_slice(&snapshot.held_mask.to_le_bytes());
    out.extend_from_slice(&snapshot.pressed_mask.to_le_bytes());
    out.extend_from_slice(&snapshot.released_mask.to_le_bytes());
    out.extend_from_slice(&snapshot.rng_seed.to_le_bytes());
    out.extend_from_slice(&(snapshot.net_events.len() as u32).to_le_bytes());
    for event in &snapshot.net_events {
        push_str(&mut out, &event.sender);
        out.extend_from_slice(&event.seq.to_le_bytes());
        push_str(&mut out, &event.order_key);
        push_str(&mut out, &event.payload);
    }
    out
}

pub fn decode_input_snapshot(bytes: &[u8]) -> Result<InputSnapshotV1, String> {
    let mut idx = 0usize;
    if bytes.len() < SNAPSHOT_MAGIC.len() {
        return Err("snapshot detbin 길이가 너무 짧습니다".to_string());
    }
    let magic = &bytes[..SNAPSHOT_MAGIC.len()];
    if magic != SNAPSHOT_MAGIC {
        return Err("snapshot detbin magic 불일치".to_string());
    }
    idx += SNAPSHOT_MAGIC.len();
    let madi = read_u64_slice(bytes, &mut idx)?;
    let held_mask = read_u16_slice(bytes, &mut idx)?;
    let pressed_mask = read_u16_slice(bytes, &mut idx)?;
    let released_mask = read_u16_slice(bytes, &mut idx)?;
    let rng_seed = read_u64_slice(bytes, &mut idx)?;
    let event_count = read_u32_slice(bytes, &mut idx)? as usize;
    let mut net_events = Vec::with_capacity(event_count);
    for _ in 0..event_count {
        let sender = read_str_slice(bytes, &mut idx)?;
        let seq = read_u64_slice(bytes, &mut idx)?;
        let order_key = read_str_slice(bytes, &mut idx)?;
        let payload = read_str_slice(bytes, &mut idx)?;
        net_events.push(NetEventV1 {
            sender,
            seq,
            order_key,
            payload,
        });
    }
    if idx != bytes.len() {
        return Err("snapshot detbin에 여분 바이트가 있습니다".to_string());
    }
    Ok(InputSnapshotV1 {
        madi,
        held_mask,
        pressed_mask,
        released_mask,
        rng_seed,
        net_events,
    })
}

pub fn encode_state_for_geoul(state: &State) -> Vec<u8> {
    let mut cleaned = state.clone();
    clear_sam_keys(&mut cleaned);
    encode_state(&cleaned)
}

pub fn geoul_state_hash_bytes(state: &State) -> [u8; 32] {
    let detbin = encode_state_for_geoul(state);
    *blake3::hash(&detbin).as_bytes()
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
        let header_bytes = encode_header(&header);
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
        })
    }

    pub fn audit_path(&self) -> &Path {
        &self.audit_path
    }

    pub fn set_entry(&mut self, entry_file: &str, entry_hash: &str) {
        self.entry_file = Some(entry_file.to_string());
        self.entry_hash = Some(entry_hash.to_string());
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
        let name = format!("cp_{:06}.detbin", madi);
        let path = self.checkpoint_dir.join(name);
        fs::write(path, state_detbin).map_err(|e| e.to_string())
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

    #[allow(dead_code)]
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
        self.file.seek(SeekFrom::Start(offset)).map_err(|e| e.to_string())?;
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

fn encode_header(header: &AuditHeader) -> Vec<u8> {
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
    file.read_exact(&mut state_hash).map_err(|e| e.to_string())?;
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
            chunk[0], chunk[1], chunk[2], chunk[3],
            chunk[4], chunk[5], chunk[6], chunk[7],
        ]));
    }
    Ok(offsets)
}

fn write_idx_file(path: &Path, offsets: &[u64]) -> Result<(), String> {
    let mut out = Vec::with_capacity(offsets.len() * 8);
    for offset in offsets {
        out.extend_from_slice(&offset.to_le_bytes());
    }
    fs::write(path, out).map_err(|e| e.to_string())
}

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
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"kind\": \"geoul_bundle_v1\",\n");
    out.push_str(&format!("  \"ssot_version\": \"{}\",\n", escape_json(ssot_version)));
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
    out.push_str("  \"audit_file\": \"audit.ddni\",\n");
    out.push_str("  \"index_file\": \"audit.idx\"\n");
    out.push_str("}\n");
    out
}

fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}

fn push_str(out: &mut Vec<u8>, text: &str) {
    let bytes = text.as_bytes();
    let len = bytes.len() as u64;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(bytes);
}

fn clear_sam_keys(state: &mut State) {
    let keys = state
        .resources
        .keys()
        .filter(|key| {
            let name = key.as_str();
            name.starts_with("샘.") || name.starts_with("입력상태.")
        })
        .cloned()
        .collect::<Vec<_>>();
    for key in keys {
        state.resources.remove(&key);
    }
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

fn read_u16_slice(bytes: &[u8], idx: &mut usize) -> Result<u16, String> {
    let end = idx.saturating_add(2);
    if end > bytes.len() {
        return Err("snapshot detbin EOF".to_string());
    }
    let out = u16::from_le_bytes([bytes[*idx], bytes[*idx + 1]]);
    *idx = end;
    Ok(out)
}

fn read_u32_slice(bytes: &[u8], idx: &mut usize) -> Result<u32, String> {
    let end = idx.saturating_add(4);
    if end > bytes.len() {
        return Err("snapshot detbin EOF".to_string());
    }
    let out = u32::from_le_bytes([bytes[*idx], bytes[*idx + 1], bytes[*idx + 2], bytes[*idx + 3]]);
    *idx = end;
    Ok(out)
}

fn read_u64_slice(bytes: &[u8], idx: &mut usize) -> Result<u64, String> {
    let end = idx.saturating_add(8);
    if end > bytes.len() {
        return Err("snapshot detbin EOF".to_string());
    }
    let out = u64::from_le_bytes([
        bytes[*idx],
        bytes[*idx + 1],
        bytes[*idx + 2],
        bytes[*idx + 3],
        bytes[*idx + 4],
        bytes[*idx + 5],
        bytes[*idx + 6],
        bytes[*idx + 7],
    ]);
    *idx = end;
    Ok(out)
}

fn read_str_slice(bytes: &[u8], idx: &mut usize) -> Result<String, String> {
    let len = read_u64_slice(bytes, idx)? as usize;
    let end = idx.saturating_add(len);
    if end > bytes.len() {
        return Err("snapshot detbin 문자열 EOF".to_string());
    }
    let text =
        std::str::from_utf8(&bytes[*idx..end]).map_err(|_| "snapshot detbin UTF-8 오류".to_string())?;
    *idx = end;
    Ok(text.to_string())
}
