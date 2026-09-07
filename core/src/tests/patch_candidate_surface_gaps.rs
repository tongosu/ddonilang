use crate::{
    platform::{ComponentTag, DetNuri, EntityId, Nuri, Origin, Patch, PatchOp},
    signals::{Signal, VecSignalSink},
    Fixed64,
};

fn div_zero_patch(tag: &str, tick_id: u64) -> Patch {
    Patch {
        ops: vec![PatchOp::DivAssignResourceFixed64 {
            tag: tag.to_string(),
            rhs: Fixed64::ZERO,
            tick_id,
            location: "research:patch_candidate_surface_gap",
            source_span: None,
            expr: None,
        }],
        origin: Origin::system("patch-candidate-surface-gap"),
    }
}

#[test]
fn component_set_and_remove_before_later_fault_remain_current_gap() {
    let mut nuri = DetNuri::new();
    let entity = EntityId(11);
    let old_tag = ComponentTag("old".to_string());
    let new_tag = ComponentTag("new".to_string());
    nuri.world_mut()
        .set_component_json(entity, old_tag.clone(), "old-value".to_string());
    nuri.world_mut()
        .set_resource_fixed64("faulting".to_string(), Fixed64::from_i64(4));
    let before_hash = nuri.state_hash();
    let patch = Patch {
        ops: vec![
            PatchOp::SetComponentJson {
                entity,
                tag: new_tag.clone(),
                json: "new-value".to_string(),
            },
            PatchOp::RemoveComponent {
                entity,
                tag: old_tag.clone(),
            },
            PatchOp::DivAssignResourceFixed64 {
                tag: "faulting".to_string(),
                rhs: Fixed64::ZERO,
                tick_id: 3,
                location: "research:component_mutation_before_fault",
                source_span: None,
                expr: None,
            },
        ],
        origin: Origin::system("patch-candidate-surface-gap"),
    };
    let mut sink = VecSignalSink::default();

    nuri.apply_patch(&patch, 3, &mut sink);

    assert_eq!(
        nuri.world().get_component_json(entity, &new_tag).as_deref(),
        Some("new-value")
    );
    assert_eq!(nuri.world().get_component_json(entity, &old_tag), None);
    assert_eq!(
        nuri.world()
            .get_resource_fixed64("faulting")
            .expect("faulting")
            .int_part(),
        4
    );
    assert_ne!(nuri.state_hash(), before_hash);
    assert!(sink
        .signals
        .iter()
        .any(|signal| matches!(signal, Signal::ArithmeticFault { .. })));
}

#[test]
fn domain_alrim_before_later_fault_escapes_current_gap() {
    let mut nuri = DetNuri::new();
    nuri.world_mut()
        .set_resource_fixed64("faulting".to_string(), Fixed64::from_i64(4));
    let patch = Patch {
        ops: vec![
            PatchOp::EmitSignal {
                signal: Signal::Alrim {
                    name: "BEFORE_FAULT",
                },
                targets: vec!["research".to_string()],
            },
            PatchOp::DivAssignResourceFixed64 {
                tag: "faulting".to_string(),
                rhs: Fixed64::ZERO,
                tick_id: 4,
                location: "research:signal_before_fault",
                source_span: None,
                expr: None,
            },
        ],
        origin: Origin::system("patch-candidate-surface-gap"),
    };
    let mut sink = VecSignalSink::default();

    nuri.apply_patch(&patch, 4, &mut sink);

    assert!(matches!(
        sink.signals.as_slice(),
        [
            Signal::Alrim {
                name: "BEFORE_FAULT"
            },
            Signal::ArithmeticFault { .. }
        ]
    ));
    assert_eq!(sink.diag_events.len(), 1);
}

#[test]
fn guard_rejected_origin_still_emits_domain_alrim_current_gap() {
    let mut nuri = DetNuri::new();
    let entity = EntityId(12);
    nuri.world_mut()
        .set_resource_fixed64("x".to_string(), Fixed64::from_i64(1));
    let patch = Patch {
        ops: vec![
            PatchOp::SetResourceFixed64 {
                tag: "x".to_string(),
                value: Fixed64::from_i64(99),
            },
            PatchOp::EmitSignal {
                signal: Signal::Alrim {
                    name: "REJECTED_DOMAIN_EVENT",
                },
                targets: vec!["entity:12".to_string()],
            },
            PatchOp::GuardViolation {
                entity,
                rule_id: "RULE_REJECT".to_string(),
            },
        ],
        origin: Origin::Entity(entity),
    };
    let mut sink = VecSignalSink::default();

    nuri.apply_patch(&patch, 5, &mut sink);

    assert_eq!(
        nuri.world()
            .get_resource_fixed64("x")
            .expect("x")
            .int_part(),
        1,
        "guard는 같은 임자 origin의 대입을 거부한다"
    );
    assert!(sink.signals.iter().any(|signal| matches!(
        signal,
        Signal::Alrim {
            name: "REJECTED_DOMAIN_EVENT"
        }
    )));
    assert!(sink
        .diag_events
        .iter()
        .any(|event| event.reason == "GUARD_VIOLATION"));
}

#[test]
fn separate_patches_reuse_same_madi_diag_sequence_zero_current_gap() {
    let mut nuri = DetNuri::new();
    nuri.world_mut()
        .set_resource_fixed64("a".to_string(), Fixed64::from_i64(4));
    nuri.world_mut()
        .set_resource_fixed64("b".to_string(), Fixed64::from_i64(8));
    let mut sink = VecSignalSink::default();

    nuri.apply_patch(&div_zero_patch("a", 6), 6, &mut sink);
    nuri.apply_patch(&div_zero_patch("b", 6), 6, &mut sink);

    assert_eq!(sink.diag_events.len(), 2);
    assert_eq!(
        sink.diag_events
            .iter()
            .map(|event| (event.madi, event.seq))
            .collect::<Vec<_>>(),
        vec![(6, 0), (6, 0)]
    );
}

#[test]
fn component_set_remove_conflict_is_arrival_order_dependent_current_gap() {
    let entity = EntityId(13);
    let tag = ComponentTag("flag".to_string());
    let set = Patch {
        ops: vec![PatchOp::SetComponentJson {
            entity,
            tag: tag.clone(),
            json: "set".to_string(),
        }],
        origin: Origin::system("patch-candidate-surface-gap"),
    };
    let remove = Patch {
        ops: vec![PatchOp::RemoveComponent {
            entity,
            tag: tag.clone(),
        }],
        origin: Origin::system("patch-candidate-surface-gap"),
    };

    let mut set_then_remove = DetNuri::new();
    let mut sink_sr = VecSignalSink::default();
    set_then_remove.apply_patch(&set, 7, &mut sink_sr);
    set_then_remove.apply_patch(&remove, 7, &mut sink_sr);

    let mut remove_then_set = DetNuri::new();
    let mut sink_rs = VecSignalSink::default();
    remove_then_set.apply_patch(&remove, 7, &mut sink_rs);
    remove_then_set.apply_patch(&set, 7, &mut sink_rs);

    assert_eq!(
        set_then_remove.world().get_component_json(entity, &tag),
        None
    );
    assert_eq!(
        remove_then_set
            .world()
            .get_component_json(entity, &tag)
            .as_deref(),
        Some("set")
    );
    assert_ne!(set_then_remove.state_hash(), remove_then_set.state_hash());
    assert!(sink_sr.diag_events.is_empty() && sink_rs.diag_events.is_empty());
}

#[test]
fn same_resource_name_can_exist_in_multiple_storage_kinds_current_gap() {
    let mut nuri = DetNuri::new();
    let patch = Patch {
        ops: vec![
            PatchOp::SetResourceJson {
                tag: "same-name".to_string(),
                json: "json-value".to_string(),
            },
            PatchOp::SetResourceFixed64 {
                tag: "same-name".to_string(),
                value: Fixed64::from_i64(7),
            },
        ],
        origin: Origin::system("patch-candidate-surface-gap"),
    };
    let mut sink = VecSignalSink::default();

    nuri.apply_patch(&patch, 8, &mut sink);

    assert_eq!(
        nuri.world().get_resource_json("same-name").as_deref(),
        Some("json-value")
    );
    assert_eq!(
        nuri.world()
            .get_resource_fixed64("same-name")
            .expect("same-name fixed64")
            .int_part(),
        7
    );
    assert!(sink.signals.is_empty());
    assert!(sink.diag_events.is_empty());
}
