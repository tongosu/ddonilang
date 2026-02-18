use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

const SSOT_VERSION: &str = "v20.6.33";
const AI_PROMPT_TEMPLATE: &str = include_str!("../assets/ai_prompt_template_v20.0.3.txt");

#[derive(Debug)]
pub struct AiPromptArgs {
    pub profile: String,
    pub out_path: Option<String>,
    pub bundle_path: Option<String>,
}

#[derive(Debug)]
struct PromptFile {
    name: String,
    bytes: Vec<u8>,
}

#[derive(Debug)]
enum BundleSource {
    Dir(PathBuf),
    Zip(PathBuf),
}

pub fn parse_ai_prompt_args<I>(args: &mut I) -> Result<AiPromptArgs, String>
where
    I: Iterator<Item = String>,
{
    let mut profile = None;
    let mut out_path = None;
    let mut bundle_path = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--profile" => {
                profile = Some(next_value(args, "--profile")?);
            }
            "--out" => {
                out_path = Some(next_value(args, "--out")?);
            }
            "--bundle" => {
                bundle_path = Some(next_value(args, "--bundle")?);
            }
            _ => return Err(format!("알 수 없는 옵션: {arg}")),
        }
    }
    let profile = profile.unwrap_or_else(|| "lean".to_string());
    Ok(AiPromptArgs {
        profile,
        out_path,
        bundle_path,
    })
}

fn next_value<I>(args: &mut I, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| format!("{flag} 값이 필요합니다."))
}

pub fn run_ai_prompt(args: AiPromptArgs) -> Result<(), String> {
    let output = build_ai_prompt(&args)?;
    if let Some(out_path) = &args.out_path {
        if let Some(parent) = Path::new(out_path).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("ai prompt 출력 경로 생성 실패: {e}"))?;
            }
        }
        fs::write(out_path, &output)
            .map_err(|e| format!("ai prompt 출력 실패: {e}"))?;
        println!("ai_prompt_written: {out_path}");
    } else {
        let mut stdout = io::stdout();
        stdout
            .write_all(&output)
            .map_err(|e| format!("ai prompt 출력 실패: {e}"))?;
    }
    Ok(())
}

fn build_ai_prompt(args: &AiPromptArgs) -> Result<Vec<u8>, String> {
    let profile = args.profile.to_ascii_lowercase();
    let (bundle_kind, source) = resolve_bundle_source(args.bundle_path.as_deref())?;
    let file_names = profile_file_names(&profile)?;
    ensure_version_matches(&file_names)?;
    let files = load_profile_files(&source, &file_names)?;
    let bundle_hash = compute_bundle_hash(&files);

    let mut out = Vec::new();
    append_template(&mut out)?;
    out.push(b'\n');
    write_context_header(
        &mut out,
        &profile,
        &bundle_kind,
        &bundle_hash,
        &file_names,
    );
    for file in &files {
        append_file_block(&mut out, file);
    }
    Ok(out)
}

fn resolve_bundle_source(bundle_path: Option<&str>) -> Result<(String, BundleSource), String> {
    if let Some(path) = bundle_path {
        let path = PathBuf::from(path);
        if path.is_dir() {
            return Ok(("dir".to_string(), BundleSource::Dir(path)));
        }
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("zip"))
            .unwrap_or(false)
        {
            let kind = if path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.contains("_codex"))
                .unwrap_or(false)
            {
                "codex"
            } else {
                "bundle"
            };
            return Ok((kind.to_string(), BundleSource::Zip(path)));
        }
        return Err(format!("번들 경로를 찾을 수 없습니다: {}", path.display()));
    }
    if let Some(default_zip) = default_bundle_zip_path() {
        return Ok(("codex".to_string(), BundleSource::Zip(default_zip)));
    }
    Ok(("dir".to_string(), BundleSource::Dir(default_ssot_dir()?)))
}

fn default_ssot_dir() -> Result<PathBuf, String> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .ok_or_else(|| "워크스페이스 루트를 찾을 수 없습니다.".to_string())?;
    let preferred = root.join("docs").join("ssot").join("ssot");
    if preferred.exists() {
        return Ok(preferred);
    }
    Ok(root.join("docs").join("ssot"))
}

fn default_bundle_zip_path() -> Option<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")));
    let name = format!("SSOT_bundle_{SSOT_VERSION}_codex.zip");
    let root_path = root.join(&name);
    if root_path.exists() {
        return Some(root_path);
    }
    let nested_docs_path = root.join("docs").join("ssot").join("ssot").join(&name);
    if nested_docs_path.exists() {
        return Some(nested_docs_path);
    }
    let docs_path = root.join("docs").join("ssot").join(&name);
    if docs_path.exists() {
        return Some(docs_path);
    }
    None
}

fn profile_file_names(profile: &str) -> Result<Vec<String>, String> {
    let names = match profile {
        "lean" => vec![
            ssot_file("SSOT_INDEX"),
            ssot_file("SSOT_TERMS"),
            ssot_file("SSOT_DECISIONS"),
            ssot_file("SSOT_LANG"),
            ssot_file("SSOT_DEMOS"),
            ssot_file("GATE0_IMPLEMENTATION_CHECKLIST"),
        ],
        "runtime" => vec![
            ssot_file("SSOT_INDEX"),
            ssot_file("SSOT_TERMS"),
            ssot_file("SSOT_DECISIONS"),
            ssot_file("SSOT_LANG"),
            ssot_file("SSOT_PLATFORM"),
            ssot_file("SSOT_DEMOS"),
            ssot_file("GATE0_IMPLEMENTATION_CHECKLIST"),
        ],
        "full" => vec![ssot_file("SSOT_ALL")],
        _ => return Err(format!("지원하지 않는 프로파일: {profile}")),
    };
    Ok(names)
}

fn ssot_file(base: &str) -> String {
    format!("{base}_{SSOT_VERSION}.md")
}

fn ensure_version_matches(file_names: &[String]) -> Result<(), String> {
    for name in file_names {
        if !name.contains(SSOT_VERSION) {
            return Err(format!("SSOT 버전 불일치: {name}"));
        }
    }
    Ok(())
}

fn load_profile_files(
    source: &BundleSource,
    file_names: &[String],
) -> Result<Vec<PromptFile>, String> {
    match source {
        BundleSource::Dir(dir) => load_profile_files_from_dir(dir, file_names),
        BundleSource::Zip(path) => load_profile_files_from_zip(path, file_names),
    }
}

fn load_profile_files_from_dir(
    base_dir: &Path,
    file_names: &[String],
) -> Result<Vec<PromptFile>, String> {
    let mut files = Vec::with_capacity(file_names.len());
    for name in file_names {
        let path = base_dir.join(name);
        let bytes = fs::read(&path)
            .map_err(|e| format!("SSOT 파일 읽기 실패: {} ({e})", path.display()))?;
        validate_prompt_bytes(name, &bytes)?;
        files.push(PromptFile {
            name: name.clone(),
            bytes,
        });
    }
    Ok(files)
}

fn load_profile_files_from_zip(
    zip_path: &Path,
    file_names: &[String],
) -> Result<Vec<PromptFile>, String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("SSOT 번들 열기 실패: {} ({e})", zip_path.display()))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("SSOT 번들 읽기 실패: {e}"))?;

    let mut exact_indices = HashMap::new();
    let mut suffix_indices: HashMap<String, Vec<usize>> = HashMap::new();
    for index in 0..archive.len() {
        let entry_name = archive
            .by_index(index)
            .map_err(|e| format!("SSOT 번들 엔트리 읽기 실패: {e}"))?
            .name()
            .to_string();
        for target in file_names {
            if entry_name == *target {
                if exact_indices.insert(target.clone(), index).is_some() {
                    return Err(format!("SSOT 번들에 동일한 파일이 중복됩니다: {target}"));
                }
            } else if entry_name.ends_with(&format!("/{}", target)) {
                suffix_indices
                    .entry(target.clone())
                    .or_default()
                    .push(index);
            }
        }
    }

    let mut files = Vec::with_capacity(file_names.len());
    for name in file_names {
        let index = if let Some(index) = exact_indices.get(name) {
            *index
        } else if let Some(matches) = suffix_indices.get(name) {
            if matches.len() == 1 {
                matches[0]
            } else {
                return Err(format!("SSOT 번들에 경로가 중복됩니다: {name}"));
            }
        } else {
            return Err(format!("SSOT 번들에서 파일을 찾을 수 없습니다: {name}"));
        };
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("SSOT 번들 엔트리 열기 실패: {e}"))?;
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|e| format!("SSOT 번들 읽기 실패: {name} ({e})"))?;
        validate_prompt_bytes(name, &bytes)?;
        files.push(PromptFile {
            name: name.clone(),
            bytes,
        });
    }
    Ok(files)
}

fn validate_prompt_bytes(name: &str, bytes: &[u8]) -> Result<(), String> {
    if bytes.contains(&b'\r') {
        return Err(format!("SSOT 파일에 CRLF가 포함되어 있습니다: {name}"));
    }
    std::str::from_utf8(bytes)
        .map_err(|_| format!("SSOT 파일이 UTF-8이 아닙니다: {name}"))?;
    Ok(())
}

fn compute_bundle_hash(files: &[PromptFile]) -> String {
    let mut hasher = Sha256::new();
    for file in files {
        hasher.update(format!("FILENAME {}\n", file.name).as_bytes());
        hasher.update(format!("BYTES {}\n", file.bytes.len()).as_bytes());
        hasher.update(&file.bytes);
        hasher.update(b"\n");
    }
    let digest = hasher.finalize();
    format!("sha256:{}", hex::encode(digest))
}

fn append_template(out: &mut Vec<u8>) -> Result<(), String> {
    if AI_PROMPT_TEMPLATE.contains('\r') {
        return Err("ai prompt 템플릿에 CRLF가 포함되어 있습니다.".to_string());
    }
    out.extend_from_slice(AI_PROMPT_TEMPLATE.as_bytes());
    if !AI_PROMPT_TEMPLATE.ends_with('\n') {
        out.push(b'\n');
    }
    Ok(())
}

fn push_line(out: &mut Vec<u8>, line: &str) {
    out.extend_from_slice(line.as_bytes());
    out.push(b'\n');
}

fn write_context_header(
    out: &mut Vec<u8>,
    profile: &str,
    bundle_kind: &str,
    bundle_hash: &str,
    file_names: &[String],
) {
    push_line(out, "[컨텍스트]");
    push_line(out, &format!("SSOT_VERSION = {SSOT_VERSION}"));
    push_line(out, &format!("BUNDLE_KIND = {bundle_kind}"));
    push_line(out, &format!("PROFILE = {profile}"));
    push_line(out, &format!("BUNDLE_HASH = {bundle_hash}"));
    push_line(out, "FILE_LIST =");
    for name in file_names {
        push_line(out, &format!("- {name}"));
    }
}

fn append_file_block(out: &mut Vec<u8>, file: &PromptFile) {
    push_line(out, &format!("===== BEGIN {} =====", file.name));
    out.extend_from_slice(&file.bytes);
    if !file.bytes.ends_with(b"\n") {
        out.push(b'\n');
    }
    push_line(out, &format!("===== END {} =====", file.name));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_prompt_bundle_hash_vector_matches() {
        let files = vec![
            PromptFile {
                name: "a.txt".to_string(),
                bytes: b"A".to_vec(),
            },
            PromptFile {
                name: "b.txt".to_string(),
                bytes: b"BC\n".to_vec(),
            },
        ];
        let hash = compute_bundle_hash(&files);
        assert_eq!(
            hash,
            "sha256:628d75b0daab4ea5006e09b27937d73c7a6d2ad4d10a5c4cc3482ad1c8dc4c5a"
        );
    }

    #[test]
    fn ai_prompt_output_invariants_hold() {
        let args = AiPromptArgs {
            profile: "lean".to_string(),
            out_path: None,
            bundle_path: None,
        };
        let output = build_ai_prompt(&args).expect("build ai prompt");
        let (bundle_kind, source) = resolve_bundle_source(None).expect("bundle source");
        assert!(!output.contains(&b'\r'));
        let output_str = String::from_utf8(output.clone()).expect("utf8");
        assert!(output_str.starts_with("[또니랑 코드 생성 규약]"));

        let ctx_index = output_str
            .find("[컨텍스트]\n")
            .expect("context header");
        let mut lines = output_str[ctx_index..].lines();
        assert_eq!(lines.next(), Some("[컨텍스트]"));
        let expected_version = format!("SSOT_VERSION = {SSOT_VERSION}");
        assert_eq!(lines.next(), Some(expected_version.as_str()));
        assert_eq!(
            lines.next(),
            Some(format!("BUNDLE_KIND = {bundle_kind}").as_str())
        );
        assert_eq!(lines.next(), Some("PROFILE = lean"));
        let bundle_hash_line = lines.next().expect("bundle hash");
        assert!(bundle_hash_line.starts_with("BUNDLE_HASH = sha256:"));
        assert_eq!(lines.next(), Some("FILE_LIST ="));

        let expected_files = profile_file_names("lean").expect("files");
        let mut actual_files = Vec::new();
        for _ in 0..expected_files.len() {
            let line = lines.next().expect("file list line");
            assert!(line.starts_with("- "));
            actual_files.push(line.trim_start_matches("- ").to_string());
        }
        assert_eq!(actual_files, expected_files);

        let files = load_profile_files(&source, &expected_files).expect("load files");
        let expected_hash = format!("BUNDLE_HASH = {}", compute_bundle_hash(&files));
        assert_eq!(bundle_hash_line, expected_hash);

        let mut cursor = output_str.as_str();
        for file in &files {
            let begin = format!("===== BEGIN {} =====\n", file.name);
            let end = format!("===== END {} =====\n", file.name);
            let start = cursor.find(&begin).expect("begin marker");
            cursor = &cursor[start + begin.len()..];
            let end_pos = cursor.find(&end).expect("end marker");
            let body = &cursor[..end_pos];
            let file_text = std::str::from_utf8(&file.bytes).expect("utf8 file");
            let mut expected_body = file_text.to_string();
            if !file_text.ends_with('\n') {
                expected_body.push('\n');
            }
            assert_eq!(body, expected_body);
            cursor = &cursor[end_pos + end.len()..];
        }
    }

    #[test]
    fn ai_prompt_output_matches_golden() {
        let args = AiPromptArgs {
            profile: "lean".to_string(),
            out_path: None,
            bundle_path: None,
        };
        let output = build_ai_prompt(&args).expect("build ai prompt");
        let golden_path = default_golden_prompt_path();
        let golden = fs::read(&golden_path)
            .map_err(|e| format!("golden 파일 읽기 실패: {} ({e})", golden_path.display()))
            .expect("read golden");
        assert_eq!(output, golden);
    }

    fn default_golden_prompt_path() -> PathBuf {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir
            .parent()
            .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")));
        root.join("tests")
            .join("toolchain_golden")
            .join("ai_prompt_lean.txt")
    }
}
