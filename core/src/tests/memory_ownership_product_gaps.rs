use crate::{
    platform::{DetNuri, DetSam, Nuri, Origin, Patch, PatchOp, Sam, SeulgiIntent},
    signals::{Signal, VecSignalSink},
    Fixed64, ResourceHandle,
};

#[test]
fn patch_fault_after_prior_write_leaves_partial_world_current_gap() {
    let mut nuri = DetNuri::new();
    nuri.world_mut()
        .set_resource_fixed64("written_before_fault".to_string(), Fixed64::from_i64(1));
    nuri.world_mut()
        .set_resource_fixed64("faulting".to_string(), Fixed64::from_i64(4));

    let patch = Patch {
        ops: vec![
            PatchOp::SetResourceFixed64 {
                tag: "written_before_fault".to_string(),
                value: Fixed64::from_i64(99),
            },
            PatchOp::DivAssignResourceFixed64 {
                tag: "faulting".to_string(),
                rhs: Fixed64::ZERO,
                tick_id: 7,
                location: "research:partial_commit_after_fault",
                source_span: None,
                expr: None,
            },
        ],
        origin: Origin::system("memory-ownership-product-gap"),
    };

    let mut sink = VecSignalSink::default();
    nuri.apply_patch(&patch, 7, &mut sink);

    assert_eq!(
        nuri.world()
            .get_resource_fixed64("written_before_fault")
            .expect("written_before_fault")
            .int_part(),
        99,
        "현재 apply_patch는 뒤 op가 실패해도 앞선 write를 되돌리지 않는다"
    );
    assert_eq!(
        nuri.world()
            .get_resource_fixed64("faulting")
            .expect("faulting")
            .int_part(),
        4
    );
    assert!(sink
        .signals
        .iter()
        .any(|signal| matches!(signal, Signal::ArithmeticFault { .. })));
}

#[test]
fn future_ai_packet_is_drained_before_target_madi_current_gap() {
    let mut sam = DetSam::new(Fixed64::from_i64(1));
    sam.push_async_ai(
        3,
        1,
        0,
        999,
        SeulgiIntent::Say {
            text: "future".to_string(),
        },
    );

    let snapshot = sam.begin_tick(1);
    assert_eq!(snapshot.ai_injections.len(), 1);
    assert_eq!(snapshot.ai_injections[0].target_madi, 999);
    assert_eq!(snapshot.tick_id, 1);
}

#[test]
fn arbitrary_raw_resource_handle_is_accepted_without_authority_lookup_current_gap() {
    let mut nuri = DetNuri::new();
    let arbitrary = ResourceHandle::from_raw(0xdead_beef_cafe_f00d);
    let patch = Patch {
        ops: vec![PatchOp::SetResourceHandle {
            tag: "research.arbitrary_handle".to_string(),
            handle: arbitrary,
        }],
        origin: Origin::system("memory-ownership-product-gap"),
    };

    let mut sink = VecSignalSink::default();
    nuri.apply_patch(&patch, 1, &mut sink);

    let restored = nuri
        .world()
        .get_resource_handle("research.arbitrary_handle")
        .expect("arbitrary handle accepted");
    assert_eq!(restored.raw(), 0xdead_beef_cafe_f00d);
    assert!(sink.signals.is_empty());
    assert!(sink.diag_events.is_empty());
}
