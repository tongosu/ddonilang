use std::fs;
use std::path::Path;

use blake3;

use crate::core::bdl_packet::{
    decode_bdl1_packet, encode_bdl1_packet, payload_hash_string, BdlPacketError,
};

pub fn wrap_packet(input: &Path, out: &Path) -> Result<(), String> {
    let payload = fs::read(input).map_err(|e| format_error(input, BdlPacketError::Truncated, e))?;
    if payload.len() < 4 || &payload[0..4] != b"BDL1" {
        return Err(format!(
            "{} {}:1:1 입력이 BDL1 detbin이 아닙니다.",
            BdlPacketError::InvalidMagic.code(),
            input.display()
        ));
    }
    let packet = encode_bdl1_packet(&payload).map_err(|err| format_error(input, err, ""))?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(out, packet).map_err(|e| e.to_string())?;
    let hash = blake3::hash(&payload);
    println!("payload_hash={}", payload_hash_string(hash.as_bytes()));
    Ok(())
}

pub fn unwrap_packet(input: &Path, out: &Path) -> Result<(), String> {
    let bytes = fs::read(input).map_err(|e| format_error(input, BdlPacketError::Truncated, e))?;
    let (payload, info) = decode_bdl1_packet(&bytes).map_err(|err| format_error(input, err, ""))?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(out, payload).map_err(|e| e.to_string())?;
    println!("payload_hash={}", payload_hash_string(&info.payload_hash));
    Ok(())
}

fn format_error(path: &Path, err: BdlPacketError, suffix: impl std::fmt::Display) -> String {
    if suffix.to_string().is_empty() {
        format!("{} {}:1:1 {}", err.code(), path.display(), err.message())
    } else {
        format!(
            "{} {}:1:1 {} {}",
            err.code(),
            path.display(),
            err.message(),
            suffix
        )
    }
}
