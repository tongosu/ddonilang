use std::collections::BTreeSet;

use crate::{
    platform::{DetNuri, Nuri, Origin, Patch, PatchOp},
    signals::{Signal, VecSignalSink},
    Fixed64, StateHash,
};

fn assign_patch(tag: &str, value: i64) -> Patch {
    Patch {
        ops: vec![PatchOp::SetResourceFixed64 {
            tag: tag.to_string(),
            value: Fixed64::from_i64(value),
        }],
        origin: Origin::system("multi-patch-product-gap"),
    }
}

fn apply_assign_sequence(sequence: &[(&str, i64)]) -> (StateHash, DetNuri, VecSignalSink) {
    let mut nuri = DetNuri::new();
    let mut sink = VecSignalSink::default();
    for (tag, value) in sequence {
        nuri.apply_patch(&assign_patch(tag, *value), 1, &mut sink);
    }
    (nuri.state_hash(), nuri, sink)
}

fn fixed64_int(nuri: &DetNuri, tag: &str) -> i64 {
    nuri.world()
        .get_resource_fixed64(tag)
        .unwrap_or_else(|| panic!("missing resource {tag}"))
        .int_part()
}

#[test]
fn disjoint_assign_patches_commute_in_current_product() {
    let (hash_ab, nuri_ab, sink_ab) = apply_assign_sequence(&[("left", 1), ("right", 2)]);
    let (hash_ba, nuri_ba, sink_ba) = apply_assign_sequence(&[("right", 2), ("left", 1)]);

    assert_eq!(hash_ab, hash_ba);
    assert_eq!(fixed64_int(&nuri_ab, "left"), 1);
    assert_eq!(fixed64_int(&nuri_ab, "right"), 2);
    assert_eq!(fixed64_int(&nuri_ba, "left"), 1);
    assert_eq!(fixed64_int(&nuri_ba, "right"), 2);
    assert!(sink_ab.diag_events.is_empty());
    assert!(sink_ba.diag_events.is_empty());
}

#[test]
fn identical_assign_patches_converge_in_current_product() {
    let (hash_ab, nuri_ab, sink_ab) = apply_assign_sequence(&[("x", 7), ("x", 7)]);
    let (hash_ba, nuri_ba, sink_ba) = apply_assign_sequence(&[("x", 7), ("x", 7)]);

    assert_eq!(hash_ab, hash_ba);
    assert_eq!(fixed64_int(&nuri_ab, "x"), 7);
    assert_eq!(fixed64_int(&nuri_ba, "x"), 7);
    assert!(sink_ab.diag_events.is_empty());
    assert!(sink_ba.diag_events.is_empty());
}

#[test]
fn conflicting_assign_patches_are_arrival_order_dependent_without_conflict_diag_current_gap() {
    let (hash_ab, nuri_ab, sink_ab) = apply_assign_sequence(&[("x", 1), ("x", 2)]);
    let (hash_ba, nuri_ba, sink_ba) = apply_assign_sequence(&[("x", 2), ("x", 1)]);

    assert_ne!(hash_ab, hash_ba);
    assert_eq!(fixed64_int(&nuri_ab, "x"), 2);
    assert_eq!(fixed64_int(&nuri_ba, "x"), 1);
    assert!(sink_ab.signals.is_empty());
    assert!(sink_ba.signals.is_empty());
    assert!(sink_ab.diag_events.is_empty());
    assert!(sink_ba.diag_events.is_empty());
}

#[test]
fn three_conflicting_assign_patches_have_three_final_hashes_across_six_permutations_current_gap() {
    let permutations = [
        [1, 2, 3],
        [1, 3, 2],
        [2, 1, 3],
        [2, 3, 1],
        [3, 1, 2],
        [3, 2, 1],
    ];
    let mut hashes = BTreeSet::new();

    for permutation in permutations {
        let sequence = permutation.map(|value| ("x", value));
        let (hash, nuri, sink) = apply_assign_sequence(&sequence);
        hashes.insert(hash.to_hex());
        assert_eq!(fixed64_int(&nuri, "x"), permutation[2]);
        assert!(sink.signals.is_empty());
        assert!(sink.diag_events.is_empty());
    }

    assert_eq!(hashes.len(), 3);
}

#[test]
fn failed_second_patch_does_not_roll_back_first_patch_as_a_group_current_gap() {
    let mut nuri = DetNuri::new();
    nuri.world_mut()
        .set_resource_fixed64("x".to_string(), Fixed64::from_i64(1));
    nuri.world_mut()
        .set_resource_fixed64("faulting".to_string(), Fixed64::from_i64(4));
    let first = assign_patch("x", 99);
    let second = Patch {
        ops: vec![PatchOp::DivAssignResourceFixed64 {
            tag: "faulting".to_string(),
            rhs: Fixed64::ZERO,
            tick_id: 1,
            location: "research:multi_patch_group_rollback",
            source_span: None,
            expr: None,
        }],
        origin: Origin::system("multi-patch-product-gap"),
    };
    let mut sink = VecSignalSink::default();

    nuri.apply_patch(&first, 1, &mut sink);
    nuri.apply_patch(&second, 1, &mut sink);

    assert_eq!(fixed64_int(&nuri, "x"), 99);
    assert_eq!(fixed64_int(&nuri, "faulting"), 4);
    assert!(sink
        .signals
        .iter()
        .any(|signal| matches!(signal, Signal::ArithmeticFault { .. })));
}
