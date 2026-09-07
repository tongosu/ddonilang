fn domain_hash(domain: &str, canonical: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(&[0]);
    hasher.update(canonical.as_bytes());
    format!("blake3:{}", hasher.finalize().to_hex())
}

#[test]
fn patch_candidate_hash_vectors_are_domain_separated_utf8_and_newline_free() {
    let ops = r#"[{"kind":"assign","local_seq":0,"target":"resource/점수","value":7},{"kind":"emit_alrim","local_seq":1,"name":"준비"}]"#;
    let ops_hash = domain_hash("ddn.patch_ops.v1", ops);

    let candidate_template = r#"{"candidate_id":"후보-1","emission_intents":[{"kind":"Alrim","name":"준비"}],"local_candidate_seq":0,"madi":9,"ops":[{"kind":"assign","local_seq":0,"target":"resource/점수","value":7},{"kind":"emit_alrim","local_seq":1,"name":"준비"}],"ops_canonical_hash":"OPS_HASH","origin_id":"1","origin_kind":"iyagi","phase":20,"read_set":[],"status":"ok","task_group_id":"group-1","task_id":"task-a","write_set":["resource/점수"]}"#;
    let candidate = candidate_template.replace("OPS_HASH", &ops_hash);
    let candidate_hash = domain_hash("ddn.patch_candidate.v1", &candidate);

    let candidate_set_template = r#"{"madi":9,"ordered_candidates":[{"candidate_id":"후보-1","emission_intents":[{"kind":"Alrim","name":"준비"}],"local_candidate_seq":0,"madi":9,"ops":[{"kind":"assign","local_seq":0,"target":"resource/점수","value":7},{"kind":"emit_alrim","local_seq":1,"name":"준비"}],"ops_canonical_hash":"OPS_HASH","origin_id":"1","origin_kind":"iyagi","phase":20,"read_set":[],"status":"ok","task_group_id":"group-1","task_id":"task-a","write_set":["resource/점수"]}]}"#;
    let candidate_set = candidate_set_template.replace("OPS_HASH", &ops_hash);
    let candidate_set_hash = domain_hash("ddn.patch_candidate_set.v1", &candidate_set);

    let receipt_template = r#"{"candidate_dispositions":[{"candidate_id":"후보-1","disposition":"committed"}],"candidate_set_hash":"SET_HASH","commit_id":"commit-9","committed_state_hash":"blake3:state-after","conflict_keys":[],"diagnostic_sequence_range":null,"emission_digest":"blake3:emission","error_code":null,"failure_consequence_ids":[],"initial_state_hash":"blake3:state-before","madi":9,"ordered_candidate_ids":["후보-1"],"status":"committed"}"#;
    let receipt = receipt_template.replace("SET_HASH", &candidate_set_hash);
    let receipt_hash = domain_hash("ddn.commit_receipt.v1", &receipt);

    for canonical in [
        ops,
        candidate.as_str(),
        candidate_set.as_str(),
        receipt.as_str(),
    ] {
        assert!(!canonical.starts_with('\u{feff}'));
        assert!(!canonical.ends_with('\n'));
        assert!(!canonical.contains('\u{fffd}'));
    }
    assert!(ops.contains("점수") && ops.contains("준비"));
    for hash in [
        &ops_hash,
        &candidate_hash,
        &candidate_set_hash,
        &receipt_hash,
    ] {
        assert!(hash.starts_with("blake3:"));
        assert_eq!(hash.len(), "blake3:".len() + 64);
    }
    assert_ne!(ops_hash, domain_hash("ddn.patch_candidate.v1", ops));

    assert_eq!(
        ops_hash,
        "blake3:bdd85bfea46f7200a02f072861a345303033ef505a7db4f100c9f516c0b42143"
    );
    assert_eq!(
        candidate_hash,
        "blake3:d0e1dc813ba7111dd85fd2d1b981f746e9a7158f0551466a67de58a70a9a4d43"
    );
    assert_eq!(
        candidate_set_hash,
        "blake3:e1de913a7ccb11605a8e10f9d58fd6848248ff3918a103ac4ca80d17b05514e7"
    );
    assert_eq!(
        receipt_hash,
        "blake3:7060186ccd25b7dd16d4ff4d069f0cc23c5953d5e0f3664a31da10634082d429"
    );

    println!("ops={ops_hash}");
    println!("candidate={candidate_hash}");
    println!("candidate_set={candidate_set_hash}");
    println!("receipt={receipt_hash}");
}

#[test]
fn product_canonical_serializer_reproduces_all_locked_vectors_byte_for_byte() {
    use crate::platform::Origin;
    use crate::{
        canonical_commit_receipt_hash, canonical_commit_receipt_text, canonicalize_candidate,
        canonicalize_candidate_set, CanonicalCommitReceiptRecord, Fixed64, Patch, PatchCandidate,
        PatchOp, Signal,
    };

    let patch = Patch {
        ops: vec![
            PatchOp::SetResourceFixed64 {
                tag: "점수".to_string(),
                value: Fixed64::from_i64(7),
            },
            PatchOp::EmitSignal {
                signal: Signal::Alrim { name: "준비" },
                targets: Vec::new(),
            },
        ],
        origin: Origin::system("vector"),
    };
    let mut candidate = PatchCandidate::new("후보-1", 9, patch);
    candidate.phase = 20;
    candidate.origin_kind = "iyagi".to_string();
    candidate.origin_id = "1".to_string();
    candidate.task_group_id = "group-1".to_string();
    candidate.task_id = "task-a".to_string();
    candidate.local_candidate_seq = 0;

    let canonical_candidate = canonicalize_candidate(&candidate);
    assert_eq!(
        canonical_candidate.ops_text,
        r#"[{"kind":"assign","local_seq":0,"target":"resource/점수","value":7},{"kind":"emit_alrim","local_seq":1,"name":"준비"}]"#
    );
    assert_eq!(
        canonical_candidate.ops_hash,
        "blake3:bdd85bfea46f7200a02f072861a345303033ef505a7db4f100c9f516c0b42143"
    );
    assert_eq!(
        canonical_candidate.candidate_text,
        r#"{"candidate_id":"후보-1","emission_intents":[{"kind":"Alrim","name":"준비"}],"local_candidate_seq":0,"madi":9,"ops":[{"kind":"assign","local_seq":0,"target":"resource/점수","value":7},{"kind":"emit_alrim","local_seq":1,"name":"준비"}],"ops_canonical_hash":"blake3:bdd85bfea46f7200a02f072861a345303033ef505a7db4f100c9f516c0b42143","origin_id":"1","origin_kind":"iyagi","phase":20,"read_set":[],"status":"ok","task_group_id":"group-1","task_id":"task-a","write_set":["resource/점수"]}"#
    );
    assert_eq!(
        canonical_candidate.candidate_hash,
        "blake3:d0e1dc813ba7111dd85fd2d1b981f746e9a7158f0551466a67de58a70a9a4d43"
    );

    let candidate_set = canonicalize_candidate_set(&[candidate], 9);
    assert_eq!(
        candidate_set.canonical_text,
        r#"{"madi":9,"ordered_candidates":[{"candidate_id":"후보-1","emission_intents":[{"kind":"Alrim","name":"준비"}],"local_candidate_seq":0,"madi":9,"ops":[{"kind":"assign","local_seq":0,"target":"resource/점수","value":7},{"kind":"emit_alrim","local_seq":1,"name":"준비"}],"ops_canonical_hash":"blake3:bdd85bfea46f7200a02f072861a345303033ef505a7db4f100c9f516c0b42143","origin_id":"1","origin_kind":"iyagi","phase":20,"read_set":[],"status":"ok","task_group_id":"group-1","task_id":"task-a","write_set":["resource/점수"]}]}"#
    );
    assert_eq!(
        candidate_set.hash,
        "blake3:e1de913a7ccb11605a8e10f9d58fd6848248ff3918a103ac4ca80d17b05514e7"
    );

    let receipt = CanonicalCommitReceiptRecord {
        candidate_dispositions: vec![("후보-1".to_string(), "committed".to_string())],
        candidate_set_hash: candidate_set.hash,
        commit_id: "commit-9".to_string(),
        committed_state_hash: "blake3:state-after".to_string(),
        conflict_keys: Vec::new(),
        diagnostic_sequence_range: None,
        emission_digest: "blake3:emission".to_string(),
        error_code: None,
        failure_consequence_ids: Vec::new(),
        initial_state_hash: "blake3:state-before".to_string(),
        madi: 9,
        ordered_candidate_ids: vec!["후보-1".to_string()],
        status: "committed".to_string(),
    };
    assert_eq!(
        canonical_commit_receipt_text(&receipt),
        r#"{"candidate_dispositions":[{"candidate_id":"후보-1","disposition":"committed"}],"candidate_set_hash":"blake3:e1de913a7ccb11605a8e10f9d58fd6848248ff3918a103ac4ca80d17b05514e7","commit_id":"commit-9","committed_state_hash":"blake3:state-after","conflict_keys":[],"diagnostic_sequence_range":null,"emission_digest":"blake3:emission","error_code":null,"failure_consequence_ids":[],"initial_state_hash":"blake3:state-before","madi":9,"ordered_candidate_ids":["후보-1"],"status":"committed"}"#
    );
    assert_eq!(
        canonical_commit_receipt_hash(&receipt),
        "blake3:7060186ccd25b7dd16d4ff4d069f0cc23c5953d5e0f3664a31da10634082d429"
    );
}

#[test]
fn product_canonical_serializer_normalizes_nfc_and_escapes_json_controls() {
    use crate::platform::Origin;
    use crate::{canonical_patch_ops_text, Patch, PatchOp};

    let patch = Patch {
        ops: vec![PatchOp::SetResourceJson {
            tag: "\u{1100}\u{1161}".to_string(),
            json: "줄\n\"끝\"".to_string(),
        }],
        origin: Origin::system("nfc-vector"),
    };

    let canonical = canonical_patch_ops_text(&patch);
    assert_eq!(
        canonical,
        r#"[{"kind":"assign","local_seq":0,"target":"resource/가","value":{"kind":"json","text":"줄\n\"끝\""}}]"#
    );
    assert!(!canonical.contains("\u{1100}\u{1161}"));
    assert!(!canonical.contains('\n'));
    assert!(!canonical.starts_with('\u{feff}'));
}
