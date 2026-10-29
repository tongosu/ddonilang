//! Test-build rendezvous only. Never an on-disk recovery capability or an
//! environment-controlled hook in a distributed Native executable.
use std::{cell::RefCell, path::PathBuf};

std::thread_local! {
    static PROBE: RefCell<Option<(String, PathBuf)>> = const { RefCell::new(None) };
}

pub(crate) fn arm(stage: &str, marker: PathBuf) {
    assert!(matches!(stage, "before_prepared" | "after_prepared" | "before_cas"
        | "after_cas" | "after_confirmation" | "after_receipt"
        | "authoring_after_source_cas" | "authoring_after_consume" | "authoring_invalid_transition"));
    PROBE.with(|slot| {
        assert!(slot.borrow().is_none());
        *slot.borrow_mut() = Some((stage.to_owned(), marker));
    });
}

pub(crate) fn checkpoint(stage: &str) {
    let mut marker = PROBE.with(|slot| slot.borrow().as_ref()
        .filter(|(selected, _)| selected == stage).map(|(_, marker)| marker.clone()));
    #[cfg(feature="beta2-crash-probe")]
    if marker.is_none() && std::env::var("DDN_B2_CRASH_STAGE").ok().as_deref()==Some(stage) {
        let path=PathBuf::from(std::env::var_os("DDN_B2_CRASH_MARKER").expect("explicit crash marker"));
        assert_eq!(path.file_name().and_then(|s|s.to_str()),Some("reached"));
        marker=Some(path);
        eprintln!("BETA2_CRASH_PROBE_DEVELOPMENT_BUILD: {stage}");
    }
    if let Some(marker) = marker {
        use std::io::Write;
        let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(marker)
            .expect("new owned marker; never overwrite");
        file.write_all(stage.as_bytes()).expect("kill rendezvous marker");
        drop(file);
        // The parent owns Child and terminates only this process. A missing
        // parent times out as a test failure, never as simulated crash PASS.
        std::thread::sleep(std::time::Duration::from_secs(30));
        panic!("parent did not terminate process at {stage}");
    }
}

/// Development observer data only. The cold consumer still requires the
/// actual SQL payload, registration, source owner, Pending/Witness and grant.
#[cfg(feature="beta2-crash-probe")]
pub(crate) fn capture_candidate(artifact:&str,summary:&serde_json::Value,reference:&serde_json::Value)->Result<(),String>{
    use std::io::Write;
    let Some(path)=std::env::var_os("DDN_B2_CRASH_CAPTURE") else{return Ok(());};
    let path=PathBuf::from(path);
    if path.file_name().and_then(|s|s.to_str())!=Some("candidate.json"){
        return Err("E_B2_PROBE_PATH: 시험 자료 파일명이 다릅니다".into());
    }
    let bytes=serde_json::to_vec(&serde_json::json!({"artifact":artifact,"summary":summary,"continuous_reference":reference,
        "claim":"UNTRUSTED_TEST_OBSERVATION_NOT_RECOVERY_AUTHORITY"})).map_err(|e|e.to_string())?;
    let mut file=std::fs::OpenOptions::new().create_new(true).write(true).open(path).map_err(|e|e.to_string())?;
    file.write_all(&bytes).map_err(|e|e.to_string())
}
