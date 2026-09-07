use crate::platform::{NetEvent, Origin};
use crate::signals::DiagEvent;
use crate::{
    analyze_patch_effects, canonical_commit_receipt_hash, canonical_execution_binding_hash,
    canonical_input_snapshot_hash, canonical_input_snapshot_text, canonical_patch_ops_hash,
    canonical_patch_ops_text, canonical_signal_list_hash, canonicalize_candidate_set,
    commit_patch_candidates_atomically,
    commit_patch_candidates_atomically_with_capability_preflight, ArithmeticFaultKind,
    AtomicCapabilityPreflight, AtomicCommitError, AtomicCommitInputKind, AtomicCommitStatus,
    AuthorizedCapabilityDispatch, CandidateBatch, CandidateDisposition, CandidateIdentityScope,
    CandidateOrigin, CanonicalExecutionBindingRecord, CapabilityAuthorization,
    CapabilityAuthorizationRequest, CapabilityBinding, CapabilityRef, CapabilityScope,
    CapabilityValue, ComponentTag, EntityId, ExprTrace, FaultContext, Fixed64, InputSnapshot,
    InputSource, LegacySinglePatchEnvelope, LogicalPatchKey, NuriWorld, Patch, PatchCandidate,
    PatchOp, ProcessLocalCapabilityStore, ResourceHandle, ResourceStorageKind, ResourceValue,
    Signal, SourceSpan, UnitDim, VecSignalSink,
};

fn patch(ops: Vec<PatchOp>) -> Patch {
    Patch {
        ops,
        origin: Origin::system("atomic-vertical-slice"),
    }
}

fn candidate(id: &str, task: &str, ops: Vec<PatchOp>) -> PatchCandidate {
    let mut candidate = PatchCandidate::new(id, 11, patch(ops));
    candidate.origin_kind = "iyagi".to_string();
    candidate.origin_id = "1".to_string();
    candidate.task_group_id = "group-1".to_string();
    candidate.task_id = task.to_string();
    candidate
}

fn div_zero(tag: &str) -> PatchOp {
    PatchOp::DivAssignResourceFixed64 {
        tag: tag.to_string(),
        rhs: Fixed64::ZERO,
        tick_id: 11,
        location: "test:atomic-vertical-slice",
        source_span: None,
        expr: None,
    }
}

fn diag(reason: &str) -> DiagEvent {
    DiagEvent {
        madi: 0,
        seq: 999,
        fault_id: reason.to_string(),
        rule_id: String::new(),
        reason: reason.to_string(),
        sub_reason: None,
        mode: None,
        contract_kind: None,
        origin: "test".to_string(),
        targets: Vec::new(),
        sam_hash: None,
        source_span: None,
        expr: None,
        message: None,
    }
}

#[derive(Default)]
struct DispatchCounter(usize);

impl AuthorizedCapabilityDispatch for DispatchCounter {
    fn dispatch_authorized(&mut self, _request: &CapabilityAuthorizationRequest) {
        self.0 += 1;
    }
}

fn handle_candidate(tag: &str) -> PatchCandidate {
    candidate(
        "capability-handle",
        "capability-task",
        vec![PatchOp::SetResourceHandle {
            tag: tag.to_string(),
            handle: ResourceHandle::from_raw(42),
        }],
    )
}

fn capability_binding() -> CapabilityBinding {
    CapabilityBinding::new("owner-a", "world-a", 7, 3)
}

fn capability_store(binding: &CapabilityBinding, tag: &str) -> ProcessLocalCapabilityStore {
    let mut store = ProcessLocalCapabilityStore::new(binding.process_id);
    assert!(store.insert(CapabilityValue::new(
        CapabilityRef::from_process_local(99),
        binding.clone(),
        [CapabilityScope::ResourceHandleWrite { tag: tag.to_string() }],
    )));
    store
}

#[test]
fn cap_p01_authorized_handle_write_dispatches_then_publishes_once() {
    let binding = capability_binding();
    let store = capability_store(&binding, "asset");
    let mut dispatch = DispatchCounter::default();
    let mut preflight = AtomicCapabilityPreflight::new(
        &store,
        binding,
        Some(CapabilityRef::from_process_local(99)),
        &mut dispatch,
    );
    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();
    let execution = commit_patch_candidates_atomically_with_capability_preflight(
        &mut world,
        &[handle_candidate("asset")],
        11,
        &mut sink,
        &mut preflight,
    );
    assert_eq!(execution.receipt.status, AtomicCommitStatus::Committed);
    assert_eq!(execution.capability_authorization, Some(CapabilityAuthorization::Authorized { capability: CapabilityRef::from_process_local(99) }));
    assert_eq!(dispatch.0, 1);
    assert_eq!(world.get_resource_handle("asset"), Some(ResourceHandle::from_raw(42)));
}

#[test]
fn cap_p02_missing_capability_rejects_without_dispatch_or_publish() {
    let binding = capability_binding();
    let store = capability_store(&binding, "asset");
    let mut dispatch = DispatchCounter::default();
    let mut preflight = AtomicCapabilityPreflight::new(&store, binding, None, &mut dispatch);
    let mut world = NuriWorld::new();
    let before = world.state_hash();
    let mut sink = VecSignalSink::default();
    let execution = commit_patch_candidates_atomically_with_capability_preflight(&mut world, &[handle_candidate("asset")], 11, &mut sink, &mut preflight);
    assert_eq!(execution.receipt.error, Some(AtomicCommitError::CapabilityPreflightUnavailable));
    assert_eq!(execution.capability_authorization, Some(CapabilityAuthorization::MissingCapability));
    assert_eq!(dispatch.0, 0);
    assert_eq!(world.state_hash(), before);
    assert_eq!(world.get_resource_handle("asset"), None);
}

#[test]
fn cap_p03_wrong_capability_is_distinct_and_fail_closed() {
    let binding = capability_binding();
    let store = capability_store(&binding, "asset");
    let mut dispatch = DispatchCounter::default();
    let mut preflight = AtomicCapabilityPreflight::new(&store, binding, Some(CapabilityRef::from_process_local(100)), &mut dispatch);
    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();
    let execution = commit_patch_candidates_atomically_with_capability_preflight(&mut world, &[handle_candidate("asset")], 11, &mut sink, &mut preflight);
    assert_eq!(execution.receipt.error, Some(AtomicCommitError::CapabilityPreflightDenied));
    assert_eq!(execution.capability_authorization, Some(CapabilityAuthorization::WrongCapability));
    assert_eq!(dispatch.0, 0);
}

#[test]
fn cap_p04_wrong_scope_is_distinct_and_fail_closed() {
    let binding = capability_binding();
    let store = capability_store(&binding, "other");
    let mut dispatch = DispatchCounter::default();
    let mut preflight = AtomicCapabilityPreflight::new(&store, binding, Some(CapabilityRef::from_process_local(99)), &mut dispatch);
    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();
    let execution = commit_patch_candidates_atomically_with_capability_preflight(&mut world, &[handle_candidate("asset")], 11, &mut sink, &mut preflight);
    assert_eq!(execution.capability_authorization, Some(CapabilityAuthorization::ScopeDenied));
    assert_eq!(dispatch.0, 0);
    assert_eq!(world.get_resource_handle("asset"), None);
}

#[test]
fn cap_p05_wrong_owner_is_distinct_and_fail_closed() {
    let binding = capability_binding();
    let store = capability_store(&binding, "asset");
    let request_binding = CapabilityBinding::new("owner-b", "world-a", 7, 3);
    let mut dispatch = DispatchCounter::default();
    let mut preflight = AtomicCapabilityPreflight::new(&store, request_binding, Some(CapabilityRef::from_process_local(99)), &mut dispatch);
    let mut world = NuriWorld::new(); let mut sink = VecSignalSink::default();
    let execution = commit_patch_candidates_atomically_with_capability_preflight(&mut world, &[handle_candidate("asset")], 11, &mut sink, &mut preflight);
    assert_eq!(execution.capability_authorization, Some(CapabilityAuthorization::WrongOwner));
    assert_eq!(dispatch.0, 0);
}

#[test]
fn cap_p06_wrong_world_is_distinct_and_fail_closed() {
    let binding = capability_binding(); let store = capability_store(&binding, "asset");
    let request_binding = CapabilityBinding::new("owner-a", "world-b", 7, 3);
    let mut dispatch = DispatchCounter::default();
    let mut preflight = AtomicCapabilityPreflight::new(&store, request_binding, Some(CapabilityRef::from_process_local(99)), &mut dispatch);
    let mut world = NuriWorld::new(); let mut sink = VecSignalSink::default();
    let execution = commit_patch_candidates_atomically_with_capability_preflight(&mut world, &[handle_candidate("asset")], 11, &mut sink, &mut preflight);
    assert_eq!(execution.capability_authorization, Some(CapabilityAuthorization::WrongWorld)); assert_eq!(dispatch.0, 0);
}

#[test]
fn cap_p07_generation_mismatch_is_distinct_and_fail_closed() {
    let binding = capability_binding(); let store = capability_store(&binding, "asset");
    let request_binding = CapabilityBinding::new("owner-a", "world-a", 7, 4);
    let mut dispatch = DispatchCounter::default();
    let mut preflight = AtomicCapabilityPreflight::new(&store, request_binding, Some(CapabilityRef::from_process_local(99)), &mut dispatch);
    let mut world = NuriWorld::new(); let mut sink = VecSignalSink::default();
    let execution = commit_patch_candidates_atomically_with_capability_preflight(&mut world, &[handle_candidate("asset")], 11, &mut sink, &mut preflight);
    assert_eq!(execution.capability_authorization, Some(CapabilityAuthorization::GenerationMismatch)); assert_eq!(dispatch.0, 0);
}

#[test]
fn cap_p08_resource_handle_is_not_a_capability_token() {
    let binding = capability_binding(); let store = capability_store(&binding, "asset");
    assert_ne!(CapabilityRef::from_process_local(ResourceHandle::from_raw(42).raw()), CapabilityRef::from_process_local(99));
    let request = CapabilityAuthorizationRequest { presented: Some(CapabilityRef::from_process_local(ResourceHandle::from_raw(42).raw())), binding, operation: crate::CapabilityOperation::ResourceHandleWrite { tags: vec!["asset".to_string()] } };
    assert_eq!(store.authorize(&request), CapabilityAuthorization::WrongCapability);
}

#[test]
fn cap_p09_non_handle_candidate_needs_no_capability_or_dispatch() {
    let binding = capability_binding(); let store = capability_store(&binding, "asset");
    let mut dispatch = DispatchCounter::default();
    let mut preflight = AtomicCapabilityPreflight::new(&store, binding, None, &mut dispatch);
    let ordinary = candidate("ordinary", "ordinary-task", vec![PatchOp::SetResourceFixed64 { tag: "safe".to_string(), value: Fixed64::ONE }]);
    let mut world = NuriWorld::new(); let mut sink = VecSignalSink::default();
    let execution = commit_patch_candidates_atomically_with_capability_preflight(&mut world, &[ordinary], 11, &mut sink, &mut preflight);
    assert_eq!(execution.receipt.status, AtomicCommitStatus::Committed); assert_eq!(execution.capability_authorization, None); assert_eq!(dispatch.0, 0);
}

#[test]
fn analyzer_maps_all_current_patch_ops_to_logical_effects() {
    let entity = EntityId(7);
    let patch = patch(vec![
        PatchOp::SetComponentJson {
            entity,
            tag: ComponentTag("위치".to_string()),
            json: "1".to_string(),
        },
        PatchOp::RemoveComponent {
            entity,
            tag: ComponentTag("이전".to_string()),
        },
        PatchOp::SetResourceJson {
            tag: "json".to_string(),
            json: "{}".to_string(),
        },
        PatchOp::SetResourceFixed64 {
            tag: "fixed".to_string(),
            value: Fixed64::ONE,
        },
        PatchOp::SetResourceHandle {
            tag: "handle".to_string(),
            handle: ResourceHandle::from_raw(3),
        },
        PatchOp::SetResourceValue {
            tag: "value".to_string(),
            value: ResourceValue::String("값".to_string()),
        },
        PatchOp::DivAssignResourceFixed64 {
            tag: "fixed".to_string(),
            rhs: Fixed64::ONE,
            tick_id: 11,
            location: "test:effect-analysis",
            source_span: Some(SourceSpan {
                file: "sample.ddn".to_string(),
                start_line: 1,
                start_col: Some(1),
                end_line: 1,
                end_col: Some(2),
            }),
            expr: Some(ExprTrace {
                tag: "divide".to_string(),
                text: None,
            }),
        },
        PatchOp::EmitSignal {
            signal: Signal::Alrim { name: "준비" },
            targets: Vec::new(),
        },
        PatchOp::EmitSignal {
            signal: Signal::Diag {
                event: diag("NOTICE"),
            },
            targets: vec!["world".to_string()],
        },
        PatchOp::EmitSignal {
            signal: Signal::ArithmeticFault {
                ctx: FaultContext {
                    tick_id: 11,
                    location: "test:explicit-fault",
                    source_span: None,
                    expr: None,
                },
                kind: ArithmeticFaultKind::DivByZero,
            },
            targets: vec!["fixed".to_string()],
        },
        PatchOp::GuardViolation {
            entity,
            rule_id: "RULE-1".to_string(),
        },
    ]);

    let effects = analyze_patch_effects(&patch);

    assert_eq!(
        effects.reads,
        vec![LogicalPatchKey::Resource {
            tag: "fixed".to_string()
        }]
    );
    assert!(!effects.writes.contains(&LogicalPatchKey::Component {
        entity,
        tag: "#규칙위반".to_string()
    }));
    assert!(!effects.writes.contains(&LogicalPatchKey::Component {
        entity,
        tag: "#휴면".to_string()
    }));
    assert_eq!(
        effects.resource_writes,
        vec![
            ("fixed".to_string(), ResourceStorageKind::Fixed64),
            ("handle".to_string(), ResourceStorageKind::Handle),
            ("json".to_string(), ResourceStorageKind::Json),
            ("value".to_string(), ResourceStorageKind::Value),
        ]
    );
    assert_eq!(effects.domain_emission_count, 1);
    assert_eq!(effects.diagnostic_intent_count, 3);
    assert!(effects.may_arithmetic_fail);
    assert!(effects.has_guard_violation);
    assert_eq!(effects.guard_violation_entities, vec![entity]);
    assert!(effects.requires_capability_preflight);

    let canonical_ops = canonical_patch_ops_text(&patch);
    assert!(!canonical_ops.contains('\n'));
    assert!(canonical_ops.contains("div_assign_fixed64"));
    assert!(canonical_ops.contains("guard_violation"));
    assert_eq!(canonical_patch_ops_hash(&patch).len(), 71);
}

#[test]
fn arithmetic_fault_invalidates_only_its_assignment_statement() {
    let entity = EntityId(8);
    let old_tag = ComponentTag("old".to_string());
    let new_tag = ComponentTag("new".to_string());
    let mut world = NuriWorld::new();
    world.set_component_json(entity, old_tag.clone(), "old-value".to_string());
    world.set_resource_fixed64("faulting".to_string(), Fixed64::from_i64(4));
    let before = world.state_hash();
    let candidate = candidate(
        "fault-candidate",
        "task-a",
        vec![
            PatchOp::SetComponentJson {
                entity,
                tag: new_tag.clone(),
                json: "new-value".to_string(),
            },
            PatchOp::RemoveComponent {
                entity,
                tag: old_tag.clone(),
            },
            PatchOp::EmitSignal {
                signal: Signal::Alrim { name: "SURVIVES" },
                targets: vec!["world".to_string()],
            },
            div_zero("faulting"),
        ],
    );
    let mut sink = VecSignalSink::default();
    let expected_candidate_set = canonicalize_candidate_set(std::slice::from_ref(&candidate), 11);

    let receipt = commit_patch_candidates_atomically(
        &mut world,
        std::slice::from_ref(&candidate),
        11,
        &mut sink,
    );

    assert_eq!(receipt.status, AtomicCommitStatus::Committed);
    assert_eq!(receipt.error, None);
    assert_ne!(receipt.initial_state_hash, receipt.committed_state_hash);
    assert_ne!(world.state_hash(), before);
    assert_eq!(
        world.get_component_json(entity, &new_tag).as_deref(),
        Some("new-value")
    );
    assert_eq!(world.get_component_json(entity, &old_tag), None);
    assert_eq!(
        world.get_resource_fixed64("faulting"),
        Some(Fixed64::from_i64(4))
    );
    assert_eq!(receipt.released_domain_emissions, 1);
    assert_eq!(
        receipt.failure_consequence_ids,
        vec!["arith:fault-candidate:3:div0"]
    );
    assert_eq!(
        receipt.candidate_decisions[0].disposition,
        CandidateDisposition::Committed
    );
    assert!(receipt.canonical_hashes_verified);
    assert_eq!(receipt.candidate_set_hash, expected_candidate_set.hash);
    let released = sink
        .signals
        .iter()
        .cloned()
        .chain(
            sink.diag_events
                .iter()
                .cloned()
                .map(|event| Signal::Diag { event }),
        )
        .collect::<Vec<_>>();
    assert_eq!(
        receipt.emission_digest,
        canonical_signal_list_hash(&released)
    );
    assert_eq!(
        receipt.receipt_hash,
        canonical_commit_receipt_hash(&receipt.canonical_record())
    );
    for hash in [
        &receipt.candidate_set_hash,
        &receipt.emission_digest,
        &receipt.receipt_hash,
    ] {
        assert!(hash.starts_with("blake3:"));
        assert_eq!(hash.len(), 71);
    }
    assert!(sink
        .signals
        .iter()
        .any(|signal| matches!(signal, Signal::Alrim { name: "SURVIVES" })));
    assert!(sink
        .signals
        .iter()
        .any(|signal| matches!(signal, Signal::ArithmeticFault { .. })));
    assert_eq!(
        sink.diag_events
            .iter()
            .map(|event| (event.madi, event.seq))
            .collect::<Vec<_>>(),
        vec![(11, 0)]
    );
}

#[test]
fn six_arrival_permutations_of_disjoint_candidates_converge() {
    let base = [
        candidate(
            "candidate-a",
            "task-a",
            vec![
                PatchOp::SetResourceFixed64 {
                    tag: "a".to_string(),
                    value: Fixed64::from_i64(1),
                },
                PatchOp::EmitSignal {
                    signal: Signal::Alrim { name: "ALRIM_A" },
                    targets: Vec::new(),
                },
            ],
        ),
        candidate(
            "candidate-b",
            "task-b",
            vec![
                PatchOp::SetResourceFixed64 {
                    tag: "b".to_string(),
                    value: Fixed64::from_i64(2),
                },
                PatchOp::EmitSignal {
                    signal: Signal::Alrim { name: "ALRIM_B" },
                    targets: Vec::new(),
                },
            ],
        ),
        candidate(
            "candidate-c",
            "task-c",
            vec![
                PatchOp::SetResourceFixed64 {
                    tag: "c".to_string(),
                    value: Fixed64::from_i64(3),
                },
                PatchOp::EmitSignal {
                    signal: Signal::Alrim { name: "ALRIM_C" },
                    targets: Vec::new(),
                },
            ],
        ),
    ];
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let mut baseline = None;

    for order in orders {
        let candidates = order.map(|index| base[index].clone());
        let expected_candidate_set = canonicalize_candidate_set(&candidates, 11);
        let mut world = NuriWorld::new();
        let mut sink = VecSignalSink::default();
        let receipt = commit_patch_candidates_atomically(&mut world, &candidates, 11, &mut sink);
        let signal_names = sink.signals.iter().map(Signal::name).collect::<Vec<_>>();
        let observation = (world.state_hash(), receipt.clone(), signal_names);

        assert_eq!(receipt.status, AtomicCommitStatus::Committed);
        assert_eq!(
            receipt.ordered_candidate_ids,
            vec!["candidate-a", "candidate-b", "candidate-c"]
        );
        assert_eq!(receipt.released_domain_emissions, 3);
        assert_eq!(receipt.candidate_set_hash, expected_candidate_set.hash);
        assert_eq!(
            receipt.emission_digest,
            canonical_signal_list_hash(&sink.signals)
        );
        assert_eq!(
            receipt.receipt_hash,
            canonical_commit_receipt_hash(&receipt.canonical_record())
        );
        assert!(receipt
            .candidate_decisions
            .iter()
            .all(|row| row.disposition == CandidateDisposition::Committed));
        assert_eq!(observation.2, vec!["ALRIM_A", "ALRIM_B", "ALRIM_C"]);
        if let Some(expected) = &baseline {
            assert_eq!(&observation, expected);
        } else {
            baseline = Some(observation);
        }
    }
}

#[test]
fn equal_strict_writes_from_two_candidates_are_rejected() {
    let candidates = [
        candidate(
            "candidate-a",
            "task-a",
            vec![PatchOp::SetResourceFixed64 {
                tag: "same".to_string(),
                value: Fixed64::from_i64(7),
            }],
        ),
        candidate(
            "candidate-b",
            "task-b",
            vec![PatchOp::SetResourceFixed64 {
                tag: "same".to_string(),
                value: Fixed64::from_i64(7),
            }],
        ),
    ];
    let mut baseline = None;
    for order in [[0, 1], [1, 0]] {
        let arrival = order.map(|index| candidates[index].clone());
        let mut world = NuriWorld::new();
        let before = world.state_hash();
        let mut sink = VecSignalSink::default();

        let receipt = commit_patch_candidates_atomically(&mut world, &arrival, 11, &mut sink);

        assert_eq!(receipt.status, AtomicCommitStatus::Rejected);
        assert_eq!(receipt.error, Some(AtomicCommitError::WriteConflict));
        assert_eq!(receipt.conflict_keys, vec!["resource/same"]);
        assert_eq!(world.state_hash(), before);
        assert_eq!(sink.diag_events[0].reason, "E_MADI_WRITE_CONFLICT");
        let observation = (receipt, sink.diag_events);
        if let Some(expected) = &baseline {
            assert_eq!(&observation, expected);
        } else {
            baseline = Some(observation);
        }
    }
}

#[test]
fn one_resource_name_cannot_cross_storage_kinds_in_atomic_batch() {
    let candidate = candidate(
        "candidate-a",
        "task-a",
        vec![
            PatchOp::SetResourceJson {
                tag: "same".to_string(),
                json: "7".to_string(),
            },
            PatchOp::SetResourceFixed64 {
                tag: "same".to_string(),
                value: Fixed64::from_i64(7),
            },
        ],
    );
    let mut world = NuriWorld::new();
    let before = world.state_hash();
    let mut sink = VecSignalSink::default();

    let receipt = commit_patch_candidates_atomically(&mut world, &[candidate], 11, &mut sink);

    assert_eq!(receipt.status, AtomicCommitStatus::Rejected);
    assert_eq!(
        receipt.error,
        Some(AtomicCommitError::ResourceNameTypeConflict)
    );
    assert_eq!(receipt.conflict_keys, vec!["resource/same"]);
    assert_eq!(world.state_hash(), before);
    assert_eq!(world.get_resource_json("same"), None);
    assert_eq!(world.get_resource_fixed64("same"), None);
}

#[test]
fn nfc_equivalent_resource_names_share_one_logical_conflict_key() {
    let candidates = [
        candidate(
            "candidate-a",
            "task-a",
            vec![PatchOp::SetResourceFixed64 {
                tag: "가".to_string(),
                value: Fixed64::from_i64(1),
            }],
        ),
        candidate(
            "candidate-b",
            "task-b",
            vec![PatchOp::SetResourceFixed64 {
                tag: "\u{1100}\u{1161}".to_string(),
                value: Fixed64::from_i64(2),
            }],
        ),
    ];
    let mut world = NuriWorld::new();
    let before = world.state_hash();
    let mut sink = VecSignalSink::default();

    let receipt = commit_patch_candidates_atomically(&mut world, &candidates, 11, &mut sink);

    assert_eq!(receipt.error, Some(AtomicCommitError::WriteConflict));
    assert_eq!(receipt.conflict_keys, vec!["resource/가"]);
    assert_eq!(world.state_hash(), before);
}

#[test]
fn nfc_equivalent_candidate_ids_are_duplicates() {
    let candidates = [
        candidate(
            "후보-가",
            "task-a",
            vec![PatchOp::SetResourceFixed64 {
                tag: "a".to_string(),
                value: Fixed64::from_i64(1),
            }],
        ),
        candidate(
            "후보-\u{1100}\u{1161}",
            "task-b",
            vec![PatchOp::SetResourceFixed64 {
                tag: "b".to_string(),
                value: Fixed64::from_i64(2),
            }],
        ),
    ];
    let mut world = NuriWorld::new();
    let before = world.state_hash();
    let mut sink = VecSignalSink::default();

    let receipt = commit_patch_candidates_atomically(&mut world, &candidates, 11, &mut sink);

    assert_eq!(receipt.error, Some(AtomicCommitError::DuplicateCandidate));
    assert_eq!(world.state_hash(), before);
}

#[test]
fn duplicate_candidates_with_equal_order_keys_reject_deterministically() {
    let candidates = [
        candidate(
            "duplicate",
            "same-task",
            vec![PatchOp::SetResourceFixed64 {
                tag: "a".to_string(),
                value: Fixed64::from_i64(1),
            }],
        ),
        candidate(
            "duplicate",
            "same-task",
            vec![PatchOp::SetResourceFixed64 {
                tag: "b".to_string(),
                value: Fixed64::from_i64(2),
            }],
        ),
    ];
    let mut baseline = None;

    for order in [[0, 1], [1, 0]] {
        let arrival = order.map(|index| candidates[index].clone());
        let mut world = NuriWorld::new();
        let before = world.state_hash();
        let mut sink = VecSignalSink::default();

        let receipt = commit_patch_candidates_atomically(&mut world, &arrival, 11, &mut sink);

        assert_eq!(receipt.error, Some(AtomicCommitError::DuplicateCandidate));
        assert_eq!(world.state_hash(), before);
        let observation = (receipt, sink.diag_events);
        if let Some(expected) = &baseline {
            assert_eq!(&observation, expected);
        } else {
            baseline = Some(observation);
        }
    }
}

#[test]
fn patch_origin_is_not_a_hidden_input_outside_the_candidate_envelope() {
    let first = candidate("fault", "task-a", vec![div_zero("faulting")]);
    let mut second = first.clone();
    second.patch.origin = Origin::system("different-hidden-origin");
    let mut observations = Vec::new();

    for candidate in [first, second] {
        let mut world = NuriWorld::new();
        world.set_resource_fixed64("faulting".to_string(), Fixed64::from_i64(4));
        let mut sink = VecSignalSink::default();

        let receipt = commit_patch_candidates_atomically(
            &mut world,
            std::slice::from_ref(&candidate),
            11,
            &mut sink,
        );

        assert_eq!(receipt.status, AtomicCommitStatus::Committed);
        assert_eq!(receipt.error, None);
        assert_eq!(receipt.failure_consequence_ids, vec!["arith:fault:0:div0"]);
        assert_eq!(sink.diag_events[0].origin, candidate.origin_id);
        observations.push((receipt, sink.signals, sink.diag_events));
    }

    assert_eq!(observations[0], observations[1]);
}

#[test]
fn patch_candidate_constructor_derives_one_canonical_typed_origin() {
    let entity_candidate = PatchCandidate::new(
        "entity",
        11,
        Patch {
            ops: Vec::new(),
            origin: Origin::Entity(EntityId(9)),
        },
    );
    assert_eq!(entity_candidate.origin_kind, "iyagi");
    assert_eq!(entity_candidate.origin_id, "9");
    assert_eq!(
        entity_candidate.canonical_origin(),
        Some(CandidateOrigin::Entity(EntityId(9)))
    );

    let system_candidate = PatchCandidate::new("system", 11, patch(Vec::new()));
    assert_eq!(system_candidate.origin_kind, "system");
    assert_eq!(system_candidate.origin_id, "#system:atomic-vertical-slice");
    assert_eq!(
        system_candidate.canonical_origin(),
        Some(CandidateOrigin::System(
            "#system:atomic-vertical-slice".to_string()
        ))
    );
}

#[test]
fn legacy_single_patch_envelope_identity_scope_is_explicit_and_deterministic() {
    let base_patch = patch(vec![PatchOp::SetResourceFixed64 {
        tag: "x".to_string(),
        value: Fixed64::ONE,
    }]);
    let first = LegacySinglePatchEnvelope::new(11, base_patch.clone());
    let same = LegacySinglePatchEnvelope::new(11, base_patch.clone());
    let different_madi = LegacySinglePatchEnvelope::new(12, base_patch.clone());
    let different_patch = LegacySinglePatchEnvelope::new(
        11,
        patch(vec![PatchOp::SetResourceFixed64 {
            tag: "x".to_string(),
            value: Fixed64::from_i64(2),
        }]),
    );
    let different_origin = LegacySinglePatchEnvelope::new(
        11,
        Patch {
            ops: base_patch.ops,
            origin: Origin::Entity(EntityId(7)),
        },
    );

    assert_eq!(first, same);
    assert_ne!(
        first.candidate.candidate_id,
        different_madi.candidate.candidate_id
    );
    assert_ne!(
        first.candidate.candidate_id,
        different_patch.candidate.candidate_id
    );
    assert_ne!(
        first.candidate.candidate_id,
        different_origin.candidate.candidate_id
    );
    assert!(first
        .candidate
        .candidate_id
        .contains(&canonical_patch_ops_hash(&first.candidate.patch)));
    let record = first.input_record();
    assert_eq!(record.kind, AtomicCommitInputKind::LegacySinglePatchCompat);
    assert_eq!(
        record.identity_scope,
        CandidateIdentityScope::PatchContentAndMadiCompat
    );
    assert_eq!(record.madi, 11);
    assert_eq!(record.candidate_ids, vec![first.candidate.candidate_id]);
}

#[test]
fn candidate_batch_preserves_declared_identities_without_legacy_rewrite() {
    let candidate_b = candidate(
        "declared-b",
        "task-b",
        vec![PatchOp::SetResourceFixed64 {
            tag: "b".to_string(),
            value: Fixed64::from_i64(2),
        }],
    );
    let candidate_a = candidate(
        "declared-a",
        "task-a",
        vec![PatchOp::SetResourceFixed64 {
            tag: "a".to_string(),
            value: Fixed64::ONE,
        }],
    );
    let batch = CandidateBatch::new(11, vec![candidate_b, candidate_a]);
    let record = batch.input_record();

    assert_eq!(record.kind, AtomicCommitInputKind::CandidateBatch);
    assert_eq!(
        record.identity_scope,
        CandidateIdentityScope::DeclaredCandidateEnvelope
    );
    assert_eq!(record.candidate_ids, vec!["declared-a", "declared-b"]);
    assert!(batch
        .candidates
        .iter()
        .all(|candidate| !candidate.candidate_id.starts_with("legacy-single-patch:")));

    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();
    let receipt = batch.commit(&mut world, &mut sink);

    assert_eq!(receipt.status, AtomicCommitStatus::Committed);
    assert_eq!(receipt.ordered_candidate_ids, record.candidate_ids);
}

#[test]
fn invalid_candidate_origin_fails_closed_as_patch_schema() {
    let mut invalid = candidate("invalid", "task-a", Vec::new());
    invalid.origin_id = "entity:9".to_string();
    let mut world = NuriWorld::new();
    let before = world.state_hash();
    let mut sink = VecSignalSink::default();

    let receipt = commit_patch_candidates_atomically(&mut world, &[invalid], 11, &mut sink);

    assert_eq!(receipt.error, Some(AtomicCommitError::PatchSchema));
    assert_eq!(world.state_hash(), before);
}

#[test]
fn guard_discards_only_violating_origin_and_converges_for_six_arrivals() {
    let entity = EntityId(9);
    let mut offender = candidate(
        "offender",
        "task-offender",
        vec![
            PatchOp::SetResourceFixed64 {
                tag: "x".to_string(),
                value: Fixed64::from_i64(99),
            },
            PatchOp::EmitSignal {
                signal: Signal::Alrim {
                    name: "OFFENDER_MUST_NOT_ESCAPE",
                },
                targets: Vec::new(),
            },
        ],
    );
    offender.origin_id = "9".to_string();
    let mut survivor = candidate(
        "survivor",
        "task-survivor",
        vec![
            PatchOp::SetResourceFixed64 {
                tag: "y".to_string(),
                value: Fixed64::from_i64(2),
            },
            PatchOp::EmitSignal {
                signal: Signal::Alrim { name: "SURVIVOR" },
                targets: Vec::new(),
            },
        ],
    );
    survivor.origin_id = "10".to_string();
    let mut guard = candidate(
        "guard",
        "task-guard",
        vec![PatchOp::GuardViolation {
            entity,
            rule_id: "RULE-X".to_string(),
        }],
    );
    guard.origin_kind = "system".to_string();
    guard.origin_id = "#system:guard".to_string();
    let base = [offender, survivor, guard];
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let mut baseline = None;

    for order in orders {
        let arrival = order.map(|index| base[index].clone());
        let mut world = NuriWorld::new();
        world.set_resource_fixed64("x".to_string(), Fixed64::ONE);
        let mut sink = VecSignalSink::default();

        let receipt = commit_patch_candidates_atomically(&mut world, &arrival, 11, &mut sink);

        assert_eq!(receipt.status, AtomicCommitStatus::Committed);
        assert_eq!(receipt.error, None);
        assert_eq!(world.get_resource_fixed64("x"), Some(Fixed64::ONE));
        assert_eq!(world.get_resource_fixed64("y"), Some(Fixed64::from_i64(2)));
        assert_eq!(
            world
                .get_component_json(entity, &ComponentTag("#규칙위반".to_string()))
                .as_deref(),
            Some("참")
        );
        assert_eq!(
            world
                .get_component_json(entity, &ComponentTag("#휴면".to_string()))
                .as_deref(),
            Some("참")
        );
        assert_eq!(receipt.failure_consequence_ids, vec!["guard:9:isolation"]);
        assert_eq!(receipt.released_domain_emissions, 1);
        assert_eq!(sink.signals, vec![Signal::Alrim { name: "SURVIVOR" }]);
        assert_eq!(sink.diag_events.len(), 1);
        assert_eq!(sink.diag_events[0].origin, "entity:9");
        assert_eq!(sink.diag_events[0].rule_id, "RULE-X");
        assert_eq!(
            receipt
                .candidate_decisions
                .iter()
                .find(|decision| decision.candidate_id == "offender")
                .map(|decision| decision.disposition),
            Some(CandidateDisposition::Rejected)
        );
        assert!(receipt
            .candidate_decisions
            .iter()
            .filter(|decision| decision.candidate_id != "offender")
            .all(|decision| decision.disposition == CandidateDisposition::Committed));
        let observation = (world.state_hash(), receipt, sink.signals, sink.diag_events);
        if let Some(expected) = &baseline {
            assert_eq!(&observation, expected);
        } else {
            baseline = Some(observation);
        }
    }
}

#[test]
fn writes_from_guard_violator_do_not_create_surviving_conflicts() {
    let entity = EntityId(9);
    let mut offender = candidate(
        "offender",
        "task-offender",
        vec![PatchOp::SetResourceFixed64 {
            tag: "same".to_string(),
            value: Fixed64::from_i64(99),
        }],
    );
    offender.origin_id = "9".to_string();
    let mut survivor = candidate(
        "survivor",
        "task-survivor",
        vec![PatchOp::SetResourceFixed64 {
            tag: "same".to_string(),
            value: Fixed64::from_i64(2),
        }],
    );
    survivor.origin_id = "10".to_string();
    let mut guard = candidate(
        "guard",
        "task-guard",
        vec![PatchOp::GuardViolation {
            entity,
            rule_id: "RULE-X".to_string(),
        }],
    );
    guard.origin_kind = "system".to_string();
    guard.origin_id = "#system:guard".to_string();
    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();

    let receipt =
        commit_patch_candidates_atomically(&mut world, &[offender, survivor, guard], 11, &mut sink);

    assert_eq!(receipt.status, AtomicCommitStatus::Committed);
    assert!(receipt.conflict_keys.is_empty());
    assert_eq!(
        world.get_resource_fixed64("same"),
        Some(Fixed64::from_i64(2))
    );
}

#[test]
fn multiple_guard_violators_are_isolated_in_entity_order() {
    let mut offender_11 = candidate(
        "offender-11",
        "task-offender-11",
        vec![PatchOp::SetResourceFixed64 {
            tag: "z".to_string(),
            value: Fixed64::from_i64(11),
        }],
    );
    offender_11.origin_id = "11".to_string();
    let mut offender_9 = candidate(
        "offender-9",
        "task-offender-9",
        vec![PatchOp::SetResourceFixed64 {
            tag: "x".to_string(),
            value: Fixed64::from_i64(9),
        }],
    );
    offender_9.origin_id = "9".to_string();
    let mut guard = candidate(
        "guard",
        "task-guard",
        vec![
            PatchOp::GuardViolation {
                entity: EntityId(11),
                rule_id: "RULE-11".to_string(),
            },
            PatchOp::GuardViolation {
                entity: EntityId(9),
                rule_id: "RULE-9".to_string(),
            },
        ],
    );
    guard.origin_kind = "system".to_string();
    guard.origin_id = "#system:guard".to_string();
    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();

    let receipt = commit_patch_candidates_atomically(
        &mut world,
        &[offender_11, guard, offender_9],
        11,
        &mut sink,
    );

    assert_eq!(receipt.status, AtomicCommitStatus::Committed);
    assert_eq!(
        receipt.failure_consequence_ids,
        vec!["guard:9:isolation", "guard:11:isolation"]
    );
    assert_eq!(world.get_resource_fixed64("x"), None);
    assert_eq!(world.get_resource_fixed64("z"), None);
    for entity in [EntityId(9), EntityId(11)] {
        assert_eq!(
            world
                .get_component_json(entity, &ComponentTag("#규칙위반".to_string()))
                .as_deref(),
            Some("참")
        );
        assert_eq!(
            world
                .get_component_json(entity, &ComponentTag("#휴면".to_string()))
                .as_deref(),
            Some("참")
        );
    }
    assert_eq!(
        sink.diag_events
            .iter()
            .map(|event| (event.seq, event.origin.as_str()))
            .collect::<Vec<_>>(),
        vec![(0, "entity:11"), (1, "entity:9")]
    );
}

#[test]
fn discarded_handle_assignment_does_not_require_capability_preflight() {
    let entity = EntityId(9);
    let mut offender = candidate(
        "offender",
        "task-offender",
        vec![PatchOp::SetResourceHandle {
            tag: "asset".to_string(),
            handle: ResourceHandle::from_raw(42),
        }],
    );
    offender.origin_id = "9".to_string();
    let mut guard = candidate(
        "guard",
        "task-guard",
        vec![PatchOp::GuardViolation {
            entity,
            rule_id: "RULE-X".to_string(),
        }],
    );
    guard.origin_kind = "system".to_string();
    guard.origin_id = "#system:guard".to_string();
    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();

    let receipt = commit_patch_candidates_atomically(&mut world, &[offender, guard], 11, &mut sink);

    assert_eq!(receipt.status, AtomicCommitStatus::Committed);
    assert_eq!(world.get_resource_handle("asset"), None);
    assert_eq!(receipt.failure_consequence_ids, vec!["guard:9:isolation"]);
}

#[test]
fn conflict_between_survivors_rejects_batch_and_isolation_markers() {
    let entity = EntityId(9);
    let mut offender = candidate("offender", "task-offender", Vec::new());
    offender.origin_id = "9".to_string();
    let mut survivor_a = candidate(
        "survivor-a",
        "task-survivor-a",
        vec![PatchOp::SetResourceFixed64 {
            tag: "same".to_string(),
            value: Fixed64::from_i64(2),
        }],
    );
    survivor_a.origin_id = "10".to_string();
    let mut survivor_b = candidate(
        "survivor-b",
        "task-survivor-b",
        vec![PatchOp::SetResourceFixed64 {
            tag: "same".to_string(),
            value: Fixed64::from_i64(3),
        }],
    );
    survivor_b.origin_id = "12".to_string();
    let mut guard = candidate(
        "guard",
        "task-guard",
        vec![PatchOp::GuardViolation {
            entity,
            rule_id: "RULE-X".to_string(),
        }],
    );
    guard.origin_kind = "system".to_string();
    guard.origin_id = "#system:guard".to_string();
    let mut world = NuriWorld::new();
    let before = world.state_hash();
    let mut sink = VecSignalSink::default();

    let receipt = commit_patch_candidates_atomically(
        &mut world,
        &[offender, survivor_a, survivor_b, guard],
        11,
        &mut sink,
    );

    assert_eq!(receipt.error, Some(AtomicCommitError::WriteConflict));
    assert_eq!(world.state_hash(), before);
    assert_eq!(
        world.get_component_json(entity, &ComponentTag("#규칙위반".to_string())),
        None
    );
    assert!(receipt.failure_consequence_ids.is_empty());
}

#[test]
fn handle_assignment_stays_fail_closed_without_capability_preflight() {
    let handle = candidate(
        "handle",
        "task-a",
        vec![PatchOp::SetResourceHandle {
            tag: "asset".to_string(),
            handle: ResourceHandle::from_raw(42),
        }],
    );
    let mut handle_world = NuriWorld::new();
    let handle_before = handle_world.state_hash();
    let mut handle_sink = VecSignalSink::default();
    let handle_receipt =
        commit_patch_candidates_atomically(&mut handle_world, &[handle], 11, &mut handle_sink);

    assert_eq!(
        handle_receipt.error,
        Some(AtomicCommitError::CapabilityPreflightUnavailable)
    );
    assert_eq!(handle_world.state_hash(), handle_before);
    assert_eq!(handle_world.get_resource_handle("asset"), None);
    assert!(handle_receipt.canonical_hashes_verified);
    assert_eq!(
        handle_receipt.receipt_hash,
        canonical_commit_receipt_hash(&handle_receipt.canonical_record())
    );
}

#[test]
fn statement_local_arithmetic_fault_does_not_hide_write_conflict() {
    let candidates = [
        candidate(
            "candidate-a",
            "task-a",
            vec![
                PatchOp::SetResourceFixed64 {
                    tag: "same".to_string(),
                    value: Fixed64::from_i64(1),
                },
                div_zero("faulting"),
            ],
        ),
        candidate(
            "candidate-b",
            "task-b",
            vec![PatchOp::SetResourceFixed64 {
                tag: "same".to_string(),
                value: Fixed64::from_i64(2),
            }],
        ),
    ];
    let mut world = NuriWorld::new();
    world.set_resource_fixed64("faulting".to_string(), Fixed64::from_i64(4));
    let before = world.state_hash();
    let mut sink = VecSignalSink::default();

    let receipt = commit_patch_candidates_atomically(&mut world, &candidates, 11, &mut sink);

    assert_eq!(receipt.error, Some(AtomicCommitError::WriteConflict));
    assert_eq!(receipt.conflict_keys, vec!["resource/same"]);
    assert_eq!(
        receipt.failure_consequence_ids,
        vec!["arith:candidate-a:1:div0"]
    );
    assert_eq!(world.state_hash(), before);
    assert_eq!(sink.diag_events[0].reason, "ARITH_FAULT");
    assert_eq!(sink.diag_events[1].reason, "E_MADI_WRITE_CONFLICT");
}

#[test]
fn faulting_assignment_does_not_create_a_write_conflict() {
    let candidates = [
        candidate("faulting", "task-faulting", vec![div_zero("same")]),
        candidate(
            "survivor",
            "task-survivor",
            vec![PatchOp::SetResourceFixed64 {
                tag: "same".to_string(),
                value: Fixed64::from_i64(2),
            }],
        ),
    ];
    let mut world = NuriWorld::new();
    world.set_resource_fixed64("same".to_string(), Fixed64::from_i64(4));
    let mut sink = VecSignalSink::default();

    let receipt = commit_patch_candidates_atomically(&mut world, &candidates, 11, &mut sink);

    assert_eq!(receipt.status, AtomicCommitStatus::Committed);
    assert!(receipt.conflict_keys.is_empty());
    assert_eq!(
        world.get_resource_fixed64("same"),
        Some(Fixed64::from_i64(2))
    );
    assert_eq!(
        receipt.failure_consequence_ids,
        vec!["arith:faulting:0:div0"]
    );
}

#[test]
fn faulting_division_does_not_create_a_resource_kind_conflict() {
    let candidate = candidate("faulting", "task-faulting", vec![div_zero("same")]);
    let mut world = NuriWorld::new();
    world.set_resource_json("same".to_string(), "{}".to_string());
    let before = world.state_hash();
    let mut sink = VecSignalSink::default();

    let receipt = commit_patch_candidates_atomically(&mut world, &[candidate], 11, &mut sink);

    assert_eq!(receipt.status, AtomicCommitStatus::Committed);
    assert_eq!(receipt.error, None);
    assert_eq!(world.state_hash(), before);
    assert_eq!(world.get_resource_json("same").as_deref(), Some("{}"));
    assert_eq!(world.get_resource_fixed64("same"), None);
}

#[test]
fn explicit_dimension_fault_signal_keeps_neighboring_statements() {
    let candidate = candidate(
        "explicit-fault",
        "task-explicit-fault",
        vec![
            PatchOp::SetResourceFixed64 {
                tag: "a".to_string(),
                value: Fixed64::from_i64(10),
            },
            PatchOp::EmitSignal {
                signal: Signal::ArithmeticFault {
                    ctx: FaultContext {
                        tick_id: 11,
                        location: "test:dimension-mismatch",
                        source_span: None,
                        expr: None,
                    },
                    kind: ArithmeticFaultKind::DimensionMismatch {
                        left: UnitDim::LENGTH,
                        right: UnitDim::TIME,
                    },
                },
                targets: vec!["resource:b".to_string()],
            },
            PatchOp::SetResourceFixed64 {
                tag: "c".to_string(),
                value: Fixed64::from_i64(30),
            },
        ],
    );
    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();

    let receipt = commit_patch_candidates_atomically(&mut world, &[candidate], 11, &mut sink);

    assert_eq!(receipt.status, AtomicCommitStatus::Committed);
    assert_eq!(world.get_resource_fixed64("a"), Some(Fixed64::from_i64(10)));
    assert_eq!(world.get_resource_fixed64("c"), Some(Fixed64::from_i64(30)));
    assert_eq!(
        receipt.failure_consequence_ids,
        vec!["arith:explicit-fault:1:dim_mismatch"]
    );
    assert_eq!(sink.diag_events[0].reason, "UNIT_MISMATCH");
    assert!(sink.signals.iter().any(|signal| matches!(
        signal,
        Signal::ArithmeticFault {
            kind: ArithmeticFaultKind::DimensionMismatch { .. },
            ..
        }
    )));
}

#[test]
fn guard_isolation_and_statement_local_arithmetic_fault_compose() {
    let entity = EntityId(9);
    let mut offender = candidate(
        "offender",
        "task-offender",
        vec![
            PatchOp::SetResourceFixed64 {
                tag: "x".to_string(),
                value: Fixed64::from_i64(99),
            },
            div_zero("faulting"),
            PatchOp::EmitSignal {
                signal: Signal::Alrim { name: "OFFENDER" },
                targets: Vec::new(),
            },
        ],
    );
    offender.origin_id = "9".to_string();
    let mut survivor = candidate(
        "survivor",
        "task-survivor",
        vec![PatchOp::SetResourceFixed64 {
            tag: "y".to_string(),
            value: Fixed64::from_i64(2),
        }],
    );
    survivor.origin_id = "10".to_string();
    let mut guard = candidate(
        "guard",
        "task-guard",
        vec![PatchOp::GuardViolation {
            entity,
            rule_id: "RULE-X".to_string(),
        }],
    );
    guard.origin_kind = "system".to_string();
    guard.origin_id = "#system:guard".to_string();
    let mut world = NuriWorld::new();
    world.set_resource_fixed64("x".to_string(), Fixed64::ONE);
    world.set_resource_fixed64("faulting".to_string(), Fixed64::from_i64(4));
    let mut sink = VecSignalSink::default();

    let receipt =
        commit_patch_candidates_atomically(&mut world, &[offender, survivor, guard], 11, &mut sink);

    assert_eq!(receipt.status, AtomicCommitStatus::Committed);
    assert_eq!(world.get_resource_fixed64("x"), Some(Fixed64::ONE));
    assert_eq!(world.get_resource_fixed64("y"), Some(Fixed64::from_i64(2)));
    assert_eq!(
        receipt.failure_consequence_ids,
        vec!["arith:offender:1:div0", "guard:9:isolation"]
    );
    assert!(sink
        .signals
        .iter()
        .all(|signal| !matches!(signal, Signal::Alrim { name: "OFFENDER" })));
    assert!(sink
        .signals
        .iter()
        .any(|signal| matches!(signal, Signal::ArithmeticFault { .. })));
}

#[test]
fn two_faulting_candidates_share_one_unique_batch_diagnostic_sequence() {
    let candidates = [
        candidate("candidate-a", "task-a", vec![div_zero("a")]),
        candidate("candidate-b", "task-b", vec![div_zero("b")]),
    ];
    let mut world = NuriWorld::new();
    world.set_resource_fixed64("a".to_string(), Fixed64::from_i64(4));
    world.set_resource_fixed64("b".to_string(), Fixed64::from_i64(8));
    let before = world.state_hash();
    let mut sink = VecSignalSink::default();

    let receipt = commit_patch_candidates_atomically(&mut world, &candidates, 11, &mut sink);

    assert_eq!(receipt.status, AtomicCommitStatus::Committed);
    assert_eq!(receipt.error, None);
    assert_eq!(world.state_hash(), before);
    assert_eq!(receipt.initial_state_hash, receipt.committed_state_hash);
    assert_eq!(
        receipt.failure_consequence_ids,
        vec!["arith:candidate-a:0:div0", "arith:candidate-b:0:div0"]
    );
    assert!(receipt
        .candidate_decisions
        .iter()
        .all(|decision| decision.disposition == CandidateDisposition::Committed));
    assert_eq!(receipt.diagnostic_sequence_range, Some((0, 2)));
    assert_eq!(
        sink.diag_events
            .iter()
            .map(|event| (event.madi, event.seq))
            .collect::<Vec<_>>(),
        vec![(11, 0), (11, 1)]
    );
}

#[test]
fn canonical_input_snapshot_binds_schedule_payload_and_normalizes_text() {
    let base = InputSnapshot {
        tick_id: 4,
        dt: Fixed64::ONE,
        keys_pressed: 0,
        last_key_name: "\u{1100}\u{1161}".to_string(),
        pointer_x_i32: 0,
        pointer_y_i32: 0,
        ai_injections: Vec::new(),
        net_events: Vec::new(),
        frame_source: InputSource::Person,
        rng_seed: 42,
    };
    let mut scheduled = base.clone();
    scheduled.frame_source = InputSource::Schedule;
    scheduled.net_events.push(NetEvent {
        sender: "eco_runner".to_string(),
        seq: 0,
        order_key: "eco_shock".to_string(),
        payload_detjson: r#"{"delta_raw":429496730,"target":"세율"}"#.to_string(),
        source: InputSource::Schedule,
    });

    let base_text = canonical_input_snapshot_text(&base);
    assert!(base_text.contains(r#""last_key_name":"가""#));
    assert!(!base_text.contains("\u{1100}\u{1161}"));
    assert_ne!(
        canonical_input_snapshot_hash(&base),
        canonical_input_snapshot_hash(&scheduled)
    );
    let mut changed_payload = scheduled.clone();
    changed_payload.net_events[0].payload_detjson =
        r#"{"delta_raw":858993459,"target":"세율"}"#.to_string();
    assert_ne!(
        canonical_input_snapshot_hash(&scheduled),
        canonical_input_snapshot_hash(&changed_payload)
    );
}

#[test]
fn bound_legacy_envelope_separates_identical_patch_by_execution_binding() {
    let patch = patch(vec![PatchOp::SetResourceFixed64 {
        tag: "세율".to_string(),
        value: Fixed64::from_ratio(1, 10),
    }]);
    let person = CanonicalExecutionBindingRecord {
        route: "eco/macro".to_string(),
        source_revision_digest: "sha256:source".to_string(),
        input_snapshot_digest: "blake3:person".to_string(),
    };
    let schedule = CanonicalExecutionBindingRecord {
        input_snapshot_digest: "blake3:schedule".to_string(),
        ..person.clone()
    };
    let person_digest = canonical_execution_binding_hash(&person);
    let schedule_digest = canonical_execution_binding_hash(&schedule);
    let person_envelope = LegacySinglePatchEnvelope::new_with_execution_binding(
        4,
        patch.clone(),
        person_digest.clone(),
    );
    let schedule_envelope =
        LegacySinglePatchEnvelope::new_with_execution_binding(4, patch, schedule_digest.clone());

    assert_ne!(person_digest, schedule_digest);
    assert_ne!(
        person_envelope.candidate.candidate_id,
        schedule_envelope.candidate.candidate_id
    );
    assert_eq!(
        person_envelope.input_record().identity_scope,
        CandidateIdentityScope::PatchContentMadiAndExecutionBinding
    );
    assert_eq!(
        person_envelope.input_record().execution_binding_digest,
        Some(person_digest)
    );
    let mut first_world = NuriWorld::new();
    let mut second_world = NuriWorld::new();
    let mut first_sink = VecSignalSink::default();
    let mut second_sink = VecSignalSink::default();
    let first = person_envelope.commit(&mut first_world, &mut first_sink);
    let second = schedule_envelope.commit(&mut second_world, &mut second_sink);
    assert_ne!(first.candidate_set_hash, second.candidate_set_hash);
    assert_ne!(first.receipt_hash, second.receipt_hash);
    assert_eq!(first.committed_state_hash, second.committed_state_hash);
}

#[derive(Default)]
struct CapabilityDispatchCounter {
    requests: Vec<CapabilityAuthorizationRequest>,
}

impl AuthorizedCapabilityDispatch for CapabilityDispatchCounter {
    fn dispatch_authorized(&mut self, request: &CapabilityAuthorizationRequest) {
        self.requests.push(request.clone());
    }
}

fn capability_fixture(tag: &str) -> (ProcessLocalCapabilityStore, CapabilityBinding, CapabilityRef) {
    let binding = CapabilityBinding::new("owner-a", "world-a", 41, 7);
    let reference = CapabilityRef::from_process_local(42);
    let capability = CapabilityValue::new(
        reference,
        binding.clone(),
        [CapabilityScope::ResourceHandleWrite {
            tag: tag.to_string(),
        }],
    );
    let mut store = ProcessLocalCapabilityStore::new(binding.process_id);
    assert!(store.insert(capability));
    (store, binding, reference)
}

fn capability_handle_candidate(tag: &str) -> PatchCandidate {
    candidate(
        "capability-handle",
        "capability-task",
        vec![PatchOp::SetResourceHandle {
            tag: tag.to_string(),
            handle: ResourceHandle::from_raw(42),
        }],
    )
}

#[test]
fn capability_physical_v0_p01_valid_authorization_dispatches_once() {
    let (store, binding, reference) = capability_fixture("asset");
    let candidate = capability_handle_candidate("asset");
    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();
    let mut dispatch = CapabilityDispatchCounter::default();
    let mut preflight =
        AtomicCapabilityPreflight::new(&store, binding, Some(reference), &mut dispatch);

    let execution = commit_patch_candidates_atomically_with_capability_preflight(
        &mut world,
        &[candidate],
        11,
        &mut sink,
        &mut preflight,
    );

    assert_eq!(
        execution.capability_authorization,
        Some(CapabilityAuthorization::Authorized {
            capability: reference
        })
    );
    assert_eq!(execution.receipt.status, AtomicCommitStatus::Committed);
    assert_eq!(dispatch.requests.len(), 1);
    assert_eq!(world.get_resource_handle("asset"), Some(ResourceHandle::from_raw(42)));
}

#[test]
fn capability_physical_v0_p02_missing_rejects_before_dispatch_and_publish() {
    let (store, binding, _) = capability_fixture("asset");
    let candidate = capability_handle_candidate("asset");
    let mut world = NuriWorld::new();
    let before = world.state_hash();
    let mut sink = VecSignalSink::default();
    let mut dispatch = CapabilityDispatchCounter::default();
    let mut preflight = AtomicCapabilityPreflight::new(&store, binding, None, &mut dispatch);

    let execution = commit_patch_candidates_atomically_with_capability_preflight(
        &mut world,
        &[candidate],
        11,
        &mut sink,
        &mut preflight,
    );

    assert_eq!(
        execution.capability_authorization,
        Some(CapabilityAuthorization::MissingCapability)
    );
    assert_eq!(
        execution.receipt.error,
        Some(AtomicCommitError::CapabilityPreflightUnavailable)
    );
    assert_eq!(dispatch.requests.len(), 0);
    assert_eq!(execution.receipt.released_domain_emissions, 0);
    assert_eq!(world.state_hash(), before);
}

#[test]
fn capability_physical_v0_p03_wrong_scope_rejects_before_dispatch() {
    let (store, binding, reference) = capability_fixture("allowed");
    let candidate = capability_handle_candidate("denied");
    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();
    let mut dispatch = CapabilityDispatchCounter::default();
    let mut preflight =
        AtomicCapabilityPreflight::new(&store, binding, Some(reference), &mut dispatch);

    let execution = commit_patch_candidates_atomically_with_capability_preflight(
        &mut world,
        &[candidate],
        11,
        &mut sink,
        &mut preflight,
    );

    assert_eq!(
        execution.capability_authorization,
        Some(CapabilityAuthorization::ScopeDenied)
    );
    assert_eq!(
        execution.receipt.error,
        Some(AtomicCommitError::CapabilityPreflightDenied)
    );
    assert_eq!(dispatch.requests.len(), 0);
    assert_eq!(world.get_resource_handle("denied"), None);
}

#[test]
fn capability_physical_v0_p04_wrong_owner_and_world_are_distinct_denials() {
    let (store, binding, reference) = capability_fixture("asset");
    for (request_binding, expected) in [
        (
            CapabilityBinding::new("owner-b", "world-a", 41, 7),
            CapabilityAuthorization::WrongOwner,
        ),
        (
            CapabilityBinding::new("owner-a", "world-b", 41, 7),
            CapabilityAuthorization::WrongWorld,
        ),
        (
            CapabilityBinding::new("owner-a", "world-a", 99, 7),
            CapabilityAuthorization::WrongProcess,
        ),
    ] {
        let mut world = NuriWorld::new();
        let mut sink = VecSignalSink::default();
        let mut dispatch = CapabilityDispatchCounter::default();
        let mut preflight = AtomicCapabilityPreflight::new(
            &store,
            request_binding,
            Some(reference),
            &mut dispatch,
        );
        let execution = commit_patch_candidates_atomically_with_capability_preflight(
            &mut world,
            &[capability_handle_candidate("asset")],
            11,
            &mut sink,
            &mut preflight,
        );
        assert_eq!(execution.capability_authorization, Some(expected));
        assert_eq!(
            execution.receipt.error,
            Some(AtomicCommitError::CapabilityPreflightDenied)
        );
        assert_eq!(dispatch.requests.len(), 0);
        assert_eq!(world.get_resource_handle("asset"), None);
    }
    assert_eq!(binding.owner_id, "owner-a");
}

#[test]
fn capability_physical_v0_p05_generation_mismatch_rejects_before_dispatch() {
    let (store, mut binding, reference) = capability_fixture("asset");
    binding.generation += 1;
    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();
    let mut dispatch = CapabilityDispatchCounter::default();
    let mut preflight =
        AtomicCapabilityPreflight::new(&store, binding, Some(reference), &mut dispatch);
    let execution = commit_patch_candidates_atomically_with_capability_preflight(
        &mut world,
        &[capability_handle_candidate("asset")],
        11,
        &mut sink,
        &mut preflight,
    );

    assert_eq!(
        execution.capability_authorization,
        Some(CapabilityAuthorization::GenerationMismatch)
    );
    assert_eq!(
        execution.receipt.error,
        Some(AtomicCommitError::CapabilityPreflightDenied)
    );
    assert_eq!(dispatch.requests.len(), 0);
    assert_eq!(world.get_resource_handle("asset"), None);
}

#[test]
fn capability_physical_v0_p06_valid_path_publishes_one_atomic_batch() {
    let (store, binding, reference) = capability_fixture("asset");
    let candidate = candidate(
        "capability-publish",
        "capability-task",
        vec![
            PatchOp::SetResourceHandle {
                tag: "asset".to_string(),
                handle: ResourceHandle::from_raw(42),
            },
            PatchOp::EmitSignal {
                signal: Signal::Alrim { name: "authorized" },
                targets: Vec::new(),
            },
        ],
    );
    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();
    let mut dispatch = CapabilityDispatchCounter::default();
    let mut preflight =
        AtomicCapabilityPreflight::new(&store, binding, Some(reference), &mut dispatch);
    let execution = commit_patch_candidates_atomically_with_capability_preflight(
        &mut world,
        &[candidate],
        11,
        &mut sink,
        &mut preflight,
    );

    assert_eq!(execution.receipt.status, AtomicCommitStatus::Committed);
    assert_eq!(execution.receipt.candidate_decisions.len(), 1);
    assert_eq!(execution.receipt.released_domain_emissions, 1);
    assert_eq!(dispatch.requests.len(), 1);
}

#[test]
fn capability_physical_v0_p07_denial_has_no_partial_state_receipt_or_publish() {
    let (store, binding, _) = capability_fixture("asset");
    let candidate = candidate(
        "capability-denied-atomicity",
        "capability-task",
        vec![
            PatchOp::SetResourceHandle {
                tag: "asset".to_string(),
                handle: ResourceHandle::from_raw(42),
            },
            PatchOp::SetResourceJson {
                tag: "must-not-exist".to_string(),
                json: "true".to_string(),
            },
            PatchOp::EmitSignal {
                signal: Signal::Alrim { name: "must-not-publish" },
                targets: Vec::new(),
            },
        ],
    );
    let mut world = NuriWorld::new();
    let before = world.state_hash();
    let mut sink = VecSignalSink::default();
    let mut dispatch = CapabilityDispatchCounter::default();
    let mut preflight = AtomicCapabilityPreflight::new(&store, binding, None, &mut dispatch);
    let execution = commit_patch_candidates_atomically_with_capability_preflight(
        &mut world,
        &[candidate],
        11,
        &mut sink,
        &mut preflight,
    );

    assert_eq!(execution.receipt.status, AtomicCommitStatus::Rejected);
    assert_eq!(execution.receipt.released_domain_emissions, 0);
    assert_eq!(world.state_hash(), before);
    assert_eq!(world.get_resource_json("must-not-exist"), None);
    assert_eq!(dispatch.requests.len(), 0);
    assert!(execution.receipt.receipt_hash.len() > 0);
}

#[test]
fn capability_physical_v0_resource_handle_is_not_authorization_authority() {
    let (store, binding, _) = capability_fixture("asset");
    let handle = ResourceHandle::from_raw(42);
    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();
    let mut dispatch = CapabilityDispatchCounter::default();
    let mut preflight = AtomicCapabilityPreflight::new(&store, binding, None, &mut dispatch);
    let execution = commit_patch_candidates_atomically_with_capability_preflight(
        &mut world,
        &[candidate(
            "resource-handle-only",
            "capability-task",
            vec![PatchOp::SetResourceHandle {
                tag: "asset".to_string(),
                handle,
            }],
        )],
        11,
        &mut sink,
        &mut preflight,
    );

    assert_eq!(handle.raw(), 42);
    assert_eq!(
        execution.capability_authorization,
        Some(CapabilityAuthorization::MissingCapability)
    );
    assert_eq!(world.get_resource_handle("asset"), None);
    assert_eq!(dispatch.requests.len(), 0);
}

#[test]
fn capability_physical_v0_wrong_capability_reference_is_rejected() {
    let (store, binding, _) = capability_fixture("asset");
    let wrong_reference = CapabilityRef::from_process_local(777);
    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();
    let mut dispatch = CapabilityDispatchCounter::default();
    let mut preflight =
        AtomicCapabilityPreflight::new(&store, binding, Some(wrong_reference), &mut dispatch);
    let execution = commit_patch_candidates_atomically_with_capability_preflight(
        &mut world,
        &[capability_handle_candidate("asset")],
        11,
        &mut sink,
        &mut preflight,
    );

    assert_eq!(
        execution.capability_authorization,
        Some(CapabilityAuthorization::WrongCapability)
    );
    assert_eq!(
        execution.receipt.error,
        Some(AtomicCommitError::CapabilityPreflightDenied)
    );
    assert_eq!(dispatch.requests.len(), 0);
    assert_eq!(world.get_resource_handle("asset"), None);
}

#[test]
fn capability_physical_v0_inactive_capability_is_rejected() {
    let binding = CapabilityBinding::new("owner-a", "world-a", 41, 7);
    let reference = CapabilityRef::from_process_local(42);
    let capability = CapabilityValue::new(
        reference,
        binding.clone(),
        [CapabilityScope::ResourceHandleWrite {
            tag: "asset".to_string(),
        }],
    )
    .with_validity(false);
    let mut store = ProcessLocalCapabilityStore::new(binding.process_id);
    assert!(store.insert(capability));
    let mut world = NuriWorld::new();
    let mut sink = VecSignalSink::default();
    let mut dispatch = CapabilityDispatchCounter::default();
    let mut preflight =
        AtomicCapabilityPreflight::new(&store, binding, Some(reference), &mut dispatch);
    let execution = commit_patch_candidates_atomically_with_capability_preflight(
        &mut world,
        &[capability_handle_candidate("asset")],
        11,
        &mut sink,
        &mut preflight,
    );

    assert_eq!(
        execution.capability_authorization,
        Some(CapabilityAuthorization::InactiveCapability)
    );
    assert_eq!(dispatch.requests.len(), 0);
    assert_eq!(world.get_resource_handle("asset"), None);
}
