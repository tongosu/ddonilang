use std::fs;
use std::path::Path;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use blake3;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

const CERT_PRIVATE_SCHEMA: &str = "ddn.cert.private_key.v1";
const CERT_PUBLIC_SCHEMA: &str = "ddn.cert.public_key.v1";
const CERT_MANIFEST_SCHEMA: &str = "ddn.cert_manifest.v1";
const PROOF_CERT_VERIFY_REPORT_SCHEMA: &str = "ddn.proof_certificate_v1.verify_report.v1";
const CERT_ALGO: &str = "sha256-proto";

#[derive(Serialize, Deserialize)]
struct CertPrivateKey {
    schema: String,
    algo: String,
    secret_key: String,
    public_key: String,
}

#[derive(Serialize, Deserialize)]
struct CertPublicKey {
    schema: String,
    algo: String,
    public_key: String,
}

#[derive(Serialize, Deserialize)]
struct CertManifest {
    schema: String,
    algo: String,
    subject_path: String,
    subject_hash: String,
    pubkey: String,
    signature: String,
}

pub fn run_keygen(out_dir: &Path, seed: Option<&str>) -> Result<(), String> {
    fs::create_dir_all(out_dir).map_err(|e| {
        format!(
            "E_CERT_KEYGEN_OUT_DIR create out dir failed {} ({})",
            out_dir.display(),
            e
        )
    })?;

    let secret = derive_secret(seed, out_dir);
    let public = super::detjson::sha256_hex(secret.as_bytes());

    let private_doc = CertPrivateKey {
        schema: CERT_PRIVATE_SCHEMA.to_string(),
        algo: CERT_ALGO.to_string(),
        secret_key: secret,
        public_key: public.clone(),
    };
    let public_doc = CertPublicKey {
        schema: CERT_PUBLIC_SCHEMA.to_string(),
        algo: CERT_ALGO.to_string(),
        public_key: public.clone(),
    };

    let private_out = out_dir.join("cert_private.key");
    let public_out = out_dir.join("cert_public.key");
    write_json(&private_out, &private_doc)?;
    write_json(&public_out, &public_doc)?;

    println!("cert_keygen_out={}", out_dir.display());
    println!("cert_private_key={}", private_out.display());
    println!("cert_public_key={}", public_out.display());
    println!("cert_pubkey={}:{}", CERT_ALGO, public);
    Ok(())
}

pub fn run_sign(input: &Path, key: &Path, out: &Path) -> Result<(), String> {
    let source_bytes = fs::read(input).map_err(|e| {
        format!(
            "E_CERT_SIGN_INPUT_READ read input failed {} ({})",
            input.display(),
            e
        )
    })?;
    let manifest = build_signed_manifest(input, &source_bytes, key)?;
    write_json(out, &manifest)?;

    println!("cert_sign_out={}", out.display());
    println!("cert_subject_hash={}", manifest.subject_hash);
    println!("cert_pubkey={}", manifest.pubkey);
    Ok(())
}

pub fn build_signed_manifest_json(
    subject_path: &Path,
    subject_bytes: &[u8],
    key: &Path,
) -> Result<JsonValue, String> {
    let manifest = build_signed_manifest(subject_path, subject_bytes, key)?;
    serde_json::to_value(&manifest).map_err(|e| format!("E_CERT_JSON_SERIALIZE {}", e))
}

pub fn run_verify(input: &Path) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| {
        format!(
            "E_CERT_VERIFY_INPUT_READ read input failed {} ({})",
            input.display(),
            e
        )
    })?;
    let manifest: CertManifest =
        serde_json::from_str(&text).map_err(|e| format!("E_CERT_VERIFY_PARSE {}", e))?;
    verify_manifest(&manifest)?;

    println!("cert_verify=ok");
    println!("cert_subject_hash={}", manifest.subject_hash);
    println!("cert_pubkey={}", manifest.pubkey);
    Ok(())
}

pub fn run_verify_proof_certificate(input: &Path, out: Option<&Path>) -> Result<(), String> {
    let bundle_bytes = fs::read(input).map_err(|e| {
        format!(
            "E_PROOF_CERT_VERIFY_INPUT_READ read input failed {} ({})",
            input.display(),
            e
        )
    })?;
    let text = String::from_utf8(bundle_bytes.clone())
        .map_err(|_| format!("E_PROOF_CERT_VERIFY_INPUT_UTF8 {}", input.display()))?;
    let bundle: JsonValue =
        serde_json::from_str(&text).map_err(|e| format!("E_PROOF_CERT_VERIFY_PARSE {}", e))?;
    if required_json_string_field(&bundle, "schema")? != "ddn.proof_certificate_v1.v1" {
        return Err(format!(
            "E_PROOF_CERT_VERIFY_SCHEMA schema={}",
            required_json_string_field(&bundle, "schema")?
        ));
    }
    if required_json_string_field(&bundle, "cert_manifest_schema")? != CERT_MANIFEST_SCHEMA {
        return Err(format!(
            "E_PROOF_CERT_VERIFY_CERT_MANIFEST_SCHEMA schema={}",
            required_json_string_field(&bundle, "cert_manifest_schema")?
        ));
    }
    let cert_manifest = required_json_object_field(&bundle, "cert_manifest")?;
    let manifest = manifest_from_json(cert_manifest)?;
    verify_manifest(&manifest)?;

    let proof_subject_hash = required_json_string_field(&bundle, "proof_subject_hash")?;
    if proof_subject_hash != manifest.subject_hash {
        return Err("E_PROOF_CERT_VERIFY_SUBJECT_HASH_MISMATCH".to_string());
    }
    if required_json_string_field(&bundle, "cert_pubkey")? != manifest.pubkey {
        return Err("E_PROOF_CERT_VERIFY_PUBKEY_MISMATCH".to_string());
    }
    if required_json_string_field(&bundle, "cert_signature")? != manifest.signature {
        return Err("E_PROOF_CERT_VERIFY_SIGNATURE_MISMATCH".to_string());
    }

    let profile = required_json_string_field(&bundle, "profile")?;
    let verified = required_json_bool_field(&bundle, "verified")?;
    let contract_diag_count = required_json_u64_field(&bundle, "contract_diag_count")?;
    let runtime_candidate = required_json_object_field(&bundle, "runtime_candidate")?;
    let runtime_artifact = required_json_object_field(&bundle, "runtime_draft_artifact")?;
    if required_json_string_field(runtime_candidate, "schema")?
        != "ddn.proof_certificate_v1_runtime_candidate.v1"
    {
        return Err("E_PROOF_CERT_VERIFY_RUNTIME_CANDIDATE_SCHEMA".to_string());
    }
    if required_json_string_field(runtime_artifact, "schema")?
        != "ddn.proof_certificate_v1_runtime_draft_artifact.v1"
    {
        return Err("E_PROOF_CERT_VERIFY_RUNTIME_ARTIFACT_SCHEMA".to_string());
    }
    if required_json_string_field(runtime_candidate, "profile")? != profile {
        return Err("E_PROOF_CERT_VERIFY_RUNTIME_PROFILE_MISMATCH".to_string());
    }
    if required_json_bool_field(runtime_candidate, "verified")? != verified {
        return Err("E_PROOF_CERT_VERIFY_RUNTIME_VERIFIED_MISMATCH".to_string());
    }
    if required_json_u64_field(runtime_candidate, "contract_diag_count")? != contract_diag_count {
        return Err("E_PROOF_CERT_VERIFY_RUNTIME_DIAG_COUNT_MISMATCH".to_string());
    }
    if required_json_string_field(runtime_candidate, "proof_subject_hash")? != manifest.subject_hash
    {
        return Err("E_PROOF_CERT_VERIFY_RUNTIME_SUBJECT_HASH_MISMATCH".to_string());
    }
    let runtime_artifact_candidate =
        required_json_object_field(runtime_artifact, "candidate_manifest")?;
    if runtime_artifact_candidate != runtime_candidate {
        return Err("E_PROOF_CERT_VERIFY_RUNTIME_ARTIFACT_CANDIDATE_MISMATCH".to_string());
    }

    let source_proof_path = Path::new(required_json_string_field(&bundle, "source_proof_path")?);
    let source_proof_bytes = fs::read(source_proof_path).map_err(|e| {
        format!(
            "E_PROOF_CERT_VERIFY_SOURCE_PROOF_READ read source proof failed {} ({})",
            source_proof_path.display(),
            e
        )
    })?;
    let expected_subject_hash =
        format!("sha256:{}", super::detjson::sha256_hex(&source_proof_bytes));
    if expected_subject_hash != manifest.subject_hash {
        return Err("E_PROOF_CERT_VERIFY_SOURCE_PROOF_HASH_MISMATCH".to_string());
    }
    let source_proof_doc: JsonValue = serde_json::from_slice(&source_proof_bytes)
        .map_err(|e| format!("E_PROOF_CERT_VERIFY_SOURCE_PROOF_PARSE {}", e))?;
    if required_json_string_field(&source_proof_doc, "schema")?
        != required_json_string_field(&bundle, "source_proof_schema")?
    {
        return Err("E_PROOF_CERT_VERIFY_SOURCE_PROOF_SCHEMA_MISMATCH".to_string());
    }
    if required_json_string_field(&source_proof_doc, "kind")?
        != required_json_string_field(&bundle, "source_proof_kind")?
    {
        return Err("E_PROOF_CERT_VERIFY_SOURCE_PROOF_KIND_MISMATCH".to_string());
    }
    for key in [
        "canonical_body_hash",
        "proof_runtime_hash",
        "solver_translation_hash",
        "state_hash",
        "trace_hash",
    ] {
        if required_json_string_field(&source_proof_doc, key)?
            != required_json_string_field(&bundle, key)?
        {
            return Err(format!(
                "E_PROOF_CERT_VERIFY_SOURCE_PROOF_FIELD_MISMATCH {}",
                key
            ));
        }
    }

    let input_path_text = input.to_string_lossy().replace('\\', "/");
    let source_proof_path_text = source_proof_path.to_string_lossy().replace('\\', "/");
    let source_hash = format!("sha256:{}", super::detjson::sha256_hex(&bundle_bytes));
    let source_proof_hash = format!("sha256:{}", super::detjson::sha256_hex(&source_proof_bytes));
    let source_provenance = serde_json::json!({
        "schema": "ddn.proof_certificate_v1.verify_report_source_provenance.v1",
        "source_kind": "proof_certificate_bundle.v1",
        "input_bundle_file": input_path_text,
        "input_bundle_hash": source_hash,
        "source_proof_file": source_proof_path_text,
        "source_proof_hash": source_proof_hash,
    });

    let report = serde_json::json!({
        "schema": PROOF_CERT_VERIFY_REPORT_SCHEMA,
        "ok": true,
        "input_path": input_path_text,
        "source_hash": source_hash,
        "source_provenance": source_provenance,
        "profile": profile,
        "verified": verified,
        "contract_diag_count": contract_diag_count,
        "source_proof_path": source_proof_path_text,
        "source_proof_schema": required_json_string_field(&source_proof_doc, "schema")?,
        "source_proof_kind": required_json_string_field(&source_proof_doc, "kind")?,
        "proof_subject_hash": proof_subject_hash,
        "canonical_body_hash": required_json_string_field(&source_proof_doc, "canonical_body_hash")?,
        "proof_runtime_hash": required_json_string_field(&source_proof_doc, "proof_runtime_hash")?,
        "solver_translation_hash": required_json_string_field(&source_proof_doc, "solver_translation_hash")?,
        "state_hash": required_json_string_field(&source_proof_doc, "state_hash")?,
        "trace_hash": required_json_string_field(&source_proof_doc, "trace_hash")?,
        "cert_manifest_schema": CERT_MANIFEST_SCHEMA,
        "cert_algo": CERT_ALGO,
        "cert_subject_hash": manifest.subject_hash,
        "cert_pubkey": manifest.pubkey,
        "cert_signature": manifest.signature,
    });
    if let Some(path) = out {
        write_json(path, &report)?;
    }

    println!("proof_certificate_verify=ok");
    println!("proof_certificate_profile={}", profile);
    println!("cert_subject_hash={}", manifest.subject_hash);
    println!("cert_pubkey={}", manifest.pubkey);
    if let Some(path) = out {
        println!("proof_certificate_verify_report={}", path.display());
    }
    Ok(())
}

fn load_private_key(path: &Path) -> Result<CertPrivateKey, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("E_CERT_KEY_READ read key failed {} ({})", path.display(), e))?;
    let doc: CertPrivateKey =
        serde_json::from_str(&text).map_err(|e| format!("E_CERT_KEY_PARSE {}", e))?;
    if doc.schema.trim() != CERT_PRIVATE_SCHEMA {
        return Err(format!("E_CERT_KEY_SCHEMA schema={}", doc.schema));
    }
    if doc.algo.trim() != CERT_ALGO {
        return Err(format!("E_CERT_KEY_ALGO algo={}", doc.algo));
    }
    if !is_hex_64(&doc.public_key) || doc.secret_key.is_empty() {
        return Err("E_CERT_KEY_PARSE invalid key fields".to_string());
    }
    Ok(doc)
}

fn build_signed_manifest(
    subject_path: &Path,
    subject_bytes: &[u8],
    key: &Path,
) -> Result<CertManifest, String> {
    let key_doc = load_private_key(key)?;
    let public_from_secret = super::detjson::sha256_hex(key_doc.secret_key.as_bytes());
    if public_from_secret != key_doc.public_key {
        return Err("E_CERT_KEY_MISMATCH public key mismatch".to_string());
    }

    let subject_hash = format!("sha256:{}", super::detjson::sha256_hex(subject_bytes));
    let signature_raw = format!("{}:{}:{}", subject_hash, key_doc.public_key, CERT_ALGO);
    let signature = super::detjson::sha256_hex(signature_raw.as_bytes());
    Ok(CertManifest {
        schema: CERT_MANIFEST_SCHEMA.to_string(),
        algo: CERT_ALGO.to_string(),
        subject_path: subject_path.to_string_lossy().replace('\\', "/"),
        subject_hash,
        pubkey: format!("{}:{}", CERT_ALGO, key_doc.public_key),
        signature: format!("{}:{}", CERT_ALGO, signature),
    })
}

fn manifest_from_json(value: &JsonValue) -> Result<CertManifest, String> {
    serde_json::from_value(value.clone()).map_err(|e| format!("E_CERT_VERIFY_PARSE {}", e))
}

fn verify_manifest(manifest: &CertManifest) -> Result<(), String> {
    if manifest.schema.trim() != CERT_MANIFEST_SCHEMA {
        return Err(format!("E_CERT_VERIFY_SCHEMA schema={}", manifest.schema));
    }
    if manifest.algo.trim() != CERT_ALGO {
        return Err(format!("E_CERT_VERIFY_ALGO algo={}", manifest.algo));
    }
    validate_subject_hash(&manifest.subject_hash)?;
    let pubkey = parse_prefixed(&manifest.pubkey, CERT_ALGO, "E_CERT_VERIFY_PUBKEY_FORMAT")?;
    let signature = parse_prefixed(
        &manifest.signature,
        CERT_ALGO,
        "E_CERT_VERIFY_SIGNATURE_FORMAT",
    )?;
    if !is_hex_64(pubkey) {
        return Err(format!("E_CERT_VERIFY_PUBKEY_PARSE {}", pubkey));
    }
    if !is_hex_64(signature) {
        return Err(format!("E_CERT_VERIFY_SIGNATURE_PARSE {}", signature));
    }
    let expected_raw = format!("{}:{}:{}", manifest.subject_hash, pubkey, CERT_ALGO);
    let expected_signature = super::detjson::sha256_hex(expected_raw.as_bytes());
    if signature != expected_signature {
        return Err("E_CERT_VERIFY_FAIL signature mismatch".to_string());
    }
    Ok(())
}

fn required_json_object_field<'a>(
    value: &'a JsonValue,
    key: &str,
) -> Result<&'a JsonValue, String> {
    value
        .get(key)
        .filter(|v| v.is_object())
        .ok_or_else(|| format!("E_PROOF_CERT_VERIFY_FIELD_MISSING {}", key))
}

fn required_json_string_field<'a>(value: &'a JsonValue, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("E_PROOF_CERT_VERIFY_FIELD_MISSING {}", key))
}

fn required_json_bool_field(value: &JsonValue, key: &str) -> Result<bool, String> {
    value
        .get(key)
        .and_then(|v| v.as_bool())
        .ok_or_else(|| format!("E_PROOF_CERT_VERIFY_FIELD_MISSING {}", key))
}

fn required_json_u64_field(value: &JsonValue, key: &str) -> Result<u64, String> {
    value
        .get(key)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("E_PROOF_CERT_VERIFY_FIELD_MISSING {}", key))
}

fn validate_subject_hash(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    let Some(hash_hex) = trimmed.strip_prefix("sha256:") else {
        return Err(format!(
            "E_CERT_CANON_BYTES_DRIFT invalid subject_hash prefix: {}",
            trimmed
        ));
    };
    if !is_hex_64(hash_hex) {
        return Err(format!(
            "E_CERT_CANON_BYTES_DRIFT invalid subject_hash bytes: {}",
            trimmed
        ));
    }
    Ok(())
}

fn parse_prefixed<'a>(value: &'a str, algo: &str, code: &str) -> Result<&'a str, String> {
    let prefix = format!("{}:", algo);
    let trimmed = value.trim();
    let Some(rest) = trimmed.strip_prefix(&prefix) else {
        return Err(format!("{} {}", code, trimmed));
    };
    if rest.is_empty() {
        return Err(format!("{} {}", code, trimmed));
    }
    Ok(rest)
}

fn derive_secret(seed: Option<&str>, out_dir: &Path) -> String {
    let entropy = match seed {
        Some(raw) => format!("seed:{raw}"),
        None => {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|v| v.as_nanos())
                .unwrap_or(0);
            format!(
                "time:{} pid:{} out:{}",
                nanos,
                process::id(),
                out_dir.to_string_lossy()
            )
        }
    };
    blake3::hash(entropy.as_bytes()).to_hex().to_string()
}

fn is_hex_64(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit())
}

fn write_json<T: Serialize>(path: &Path, payload: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "E_CERT_OUT_DIR create parent failed {} ({})",
                parent.display(),
                e
            )
        })?;
    }
    let text = serde_json::to_string_pretty(payload)
        .map_err(|e| format!("E_CERT_JSON_SERIALIZE {}", e))?;
    fs::write(path, format!("{text}\n"))
        .map_err(|e| format!("E_CERT_OUT_WRITE write failed {} ({})", path.display(), e))
}
