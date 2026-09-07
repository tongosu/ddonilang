use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ddonirang_tool::ddn_runtime::DdnProgram;

fn collect_ddn_files(root: &Path, out: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("{}: {error}", root.display()))
        .map(|entry| entry.expect("directory entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_ddn_files(&path, out);
        } else if path.extension().and_then(|value| value.to_str()) == Some("ddn") {
            out.push(path);
        }
    }
}

fn error_family(message: &str) -> String {
    message
        .split(|ch: char| ch == ':' || ch.is_whitespace())
        .find(|part| part.starts_with("E_"))
        .unwrap_or("UNCLASSIFIED")
        .to_string()
}

#[test]
fn inventory_all_workbench_lessons_by_product_source_epoch() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("solutions")
        .join("seamgrim_ui_mvp")
        .join("lessons");
    let mut files = Vec::new();
    collect_ddn_files(&root, &mut files);
    assert!(!files.is_empty(), "lesson inventory must not be empty");

    let mut both = 0usize;
    let mut pre_v25_only = 0usize;
    let mut v25_only = 0usize;
    let mut neither = 0usize;
    let mut v25_error_families = BTreeMap::<String, usize>::new();
    let mut pre_v25_error_families = BTreeMap::<String, usize>::new();
    let mut v25_pass_paths = Vec::new();

    for path in &files {
        let source = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let label = path.to_string_lossy();
        let pre = DdnProgram::from_source(&source, &label);
        let current = DdnProgram::from_v25_source(&source, &label);
        match (&pre, &current) {
            (Ok(_), Ok(_)) => {
                both += 1;
                v25_pass_paths.push(path.strip_prefix(&root).unwrap().to_path_buf());
            }
            (Ok(_), Err(_)) => pre_v25_only += 1,
            (Err(_), Ok(_)) => {
                v25_only += 1;
                v25_pass_paths.push(path.strip_prefix(&root).unwrap().to_path_buf());
            }
            (Err(_), Err(_)) => neither += 1,
        }
        if let Err(error) = current {
            *v25_error_families.entry(error_family(&error)).or_default() += 1;
        }
        if let Err(error) = pre {
            *pre_v25_error_families.entry(error_family(&error)).or_default() += 1;
        }
    }

    println!("FD01_LESSON_TOTAL={}", files.len());
    println!("FD01_LESSON_BOTH={both}");
    println!("FD01_LESSON_PRE_V25_ONLY={pre_v25_only}");
    println!("FD01_LESSON_V25_ONLY={v25_only}");
    println!("FD01_LESSON_NEITHER={neither}");
    println!("FD01_V25_ERROR_FAMILIES={v25_error_families:?}");
    println!("FD01_PRE_V25_ERROR_FAMILIES={pre_v25_error_families:?}");
    println!("FD01_V25_PASS_PATHS={v25_pass_paths:?}");
    assert_eq!(both + pre_v25_only + v25_only + neither, files.len());
    assert_eq!(files.len(), 557, "sealed FD01 lesson membership drift");
    assert_eq!(both, 0, "unexpected dual-epoch lesson acceptance");
    assert_eq!(pre_v25_only, 127, "pre-V25-only membership drift");
    assert_eq!(v25_only, 0, "unexpected V25-ready lesson appeared");
    assert_eq!(neither, 430, "dual-rejection membership drift");
    assert!(v25_pass_paths.is_empty(), "global V25 switch gate must stay closed");
    assert_eq!(
        v25_error_families,
        BTreeMap::from([
            ("E_BOIM_LEGACY_REMOVED".to_string(), 414),
            ("E_V25_DEFINITION_BINDING_CATEGORY".to_string(), 8),
            ("E_V25_DEFINITION_MIGRATION_REQUIRED".to_string(), 135),
        ]),
        "V25 diagnostic-family inventory drift",
    );
}
