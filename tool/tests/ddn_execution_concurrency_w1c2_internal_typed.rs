use ddonirang_lang::{
    EndpointRoute, EndpointRouteSemantic, ExecutionInstanceIdentity,
    InternalDeliveryEnvelopeSemantic, InternalProviderIdentityFailure, ReactionProviderDescriptor,
    ReactionProviderPredicate, ReactionProviderRegistry, SenderIdentityPredicate,
};
use ddonirang_tool::ddn_runtime::{
    DdnProgram, DdnRunner, InternalEffectIntent, InternalReactionFailure,
    InternalReactionOperation as Op, InternalReactionProgram, InternalTypedReactionRuntime,
};
use std::collections::BTreeMap;

fn envelope(kind: &str) -> InternalDeliveryEnvelopeSemantic {
    InternalDeliveryEnvelopeSemantic::new("관제탑", kind, "철수", "ANY_PAYLOAD")
}

fn program(operations: Vec<Op>) -> InternalReactionProgram {
    InternalReactionProgram::new(operations)
}

fn route(operations: Vec<Op>) -> EndpointRoute<InternalReactionProgram> {
    EndpointRoute::new(
        EndpointRouteSemantic::new(
            "관제탑",
            "첫알림",
            SenderIdentityPredicate::AnySender,
            "ANY_PAYLOAD",
            "기본::관제탑",
        ),
        program(operations),
    )
}

fn provider(
    declaration: &str,
    body_identity: &str,
    owner_instance: &str,
    registration: &str,
    operations: Vec<Op>,
) -> ReactionProviderDescriptor<InternalReactionProgram> {
    ReactionProviderDescriptor::new(
        "기본::관제탑",
        declaration,
        body_identity,
        "기본::관제탑",
        owner_instance,
        registration,
        ReactionProviderPredicate::new(
            "관제탑",
            "첫알림",
            SenderIdentityPredicate::AnySender,
            "ANY_PAYLOAD",
        ),
        program(operations),
    )
}

fn registry_with(
    descriptors: Vec<ReactionProviderDescriptor<InternalReactionProgram>>,
) -> (
    ReactionProviderRegistry<InternalReactionProgram>,
    Vec<ddonirang_lang::ProviderRegistrationIdentity>,
) {
    let mut registry = ReactionProviderRegistry::default();
    let mut keys = Vec::new();
    for descriptor in descriptors {
        keys.push(descriptor.registration_identity.clone());
        registry
            .register_descriptor(descriptor)
            .expect("unique internal provider registration");
    }
    (registry, keys)
}

fn execute_ok(
    runtime: &mut InternalTypedReactionRuntime,
    routes: &[EndpointRoute<InternalReactionProgram>],
    registry: &ReactionProviderRegistry<InternalReactionProgram>,
    reached: Vec<ddonirang_lang::ProviderRegistrationIdentity>,
    execution: &str,
    occurrence: &str,
) -> ddonirang_tool::ddn_runtime::InternalReactionSuccess {
    runtime
        .execute(
            routes,
            registry,
            reached,
            &envelope("첫알림"),
            ExecutionInstanceIdentity::derive(execution),
            occurrence,
            |_, _, _| Ok(()),
        )
        .expect("internal typed reaction must commit")
}

#[test]
fn w1c2_i1_case_01_route_zero_means_provider_zero() {
    let descriptor = provider("관찰", "body-a", "owner-a", "reg-a", vec![]);
    let unknown_reached = descriptor.registration_identity.clone();
    let registry = ReactionProviderRegistry::default();
    let mut runtime = InternalTypedReactionRuntime::default();
    let failure = runtime
        .execute(
            &[],
            &registry,
            vec![unknown_reached],
            &envelope("첫알림"),
            ExecutionInstanceIdentity::derive("exec-1"),
            "delivery-1",
            |_, _, _| panic!("effect release must not run"),
        )
        .expect_err("zero route must fail before provider resolution");
    assert_eq!(failure, InternalReactionFailure::RouteUnhandled);
    assert_eq!(runtime.revision(), 0);
}

#[test]
fn w1c2_i1_case_02_route_two_means_provider_zero() {
    let descriptor = provider("관찰", "body-a", "owner-a", "reg-a", vec![]);
    let unknown_reached = descriptor.registration_identity.clone();
    let registry = ReactionProviderRegistry::default();
    let routes = vec![route(vec![]), route(vec![])];
    let mut runtime = InternalTypedReactionRuntime::default();
    let failure = runtime
        .execute(
            &routes,
            &registry,
            vec![unknown_reached],
            &envelope("첫알림"),
            ExecutionInstanceIdentity::derive("exec-1"),
            "delivery-1",
            |_, _, _| panic!("effect release must not run"),
        )
        .expect_err("ambiguous route must fail before provider resolution");
    assert_eq!(
        failure,
        InternalReactionFailure::RouteAmbiguous { count: 2 }
    );
}

#[test]
fn w1c2_i1_case_03_route_one_provider_zero() {
    let routes = vec![route(vec![Op::Set {
        key: "route".into(),
        value: "done".into(),
    }])];
    let registry = ReactionProviderRegistry::default();
    let mut runtime = InternalTypedReactionRuntime::default();
    let success = execute_ok(&mut runtime, &routes, &registry, vec![], "exec-1", "d1");
    assert!(success.provider_registration_identities.is_empty());
    assert_eq!(
        runtime.world().get("route").map(String::as_str),
        Some("done")
    );
}

#[test]
fn w1c2_i1_case_04_route_one_provider_one() {
    let descriptor = provider(
        "관찰-a",
        "body-a",
        "owner-a",
        "reg-a",
        vec![Op::Set {
            key: "provider-a".into(),
            value: "seen".into(),
        }],
    );
    let (registry, keys) = registry_with(vec![descriptor]);
    let mut runtime = InternalTypedReactionRuntime::default();
    let success = execute_ok(
        &mut runtime,
        &[route(vec![])],
        &registry,
        keys,
        "exec-1",
        "d1",
    );
    assert_eq!(success.provider_registration_identities.len(), 1);
    assert_eq!(
        runtime.world().get("provider-a").map(String::as_str),
        Some("seen")
    );
}

#[test]
fn w1c2_i1_case_05_distinct_provider_two_fanout() {
    let a = provider(
        "관찰-a",
        "body-a",
        "owner-a",
        "reg-a",
        vec![Op::Set {
            key: "a".into(),
            value: "1".into(),
        }],
    );
    let b = provider(
        "관찰-b",
        "body-b",
        "owner-a",
        "reg-b",
        vec![Op::Set {
            key: "b".into(),
            value: "1".into(),
        }],
    );
    let (registry, keys) = registry_with(vec![a, b]);
    let mut runtime = InternalTypedReactionRuntime::default();
    let success = execute_ok(
        &mut runtime,
        &[route(vec![])],
        &registry,
        keys,
        "exec-1",
        "d1",
    );
    assert_eq!(success.provider_registration_identities.len(), 2);
    assert_eq!(runtime.world().len(), 2);
}

#[test]
fn w1c2_i1_case_06_same_identity_diamond_once() {
    let descriptor = provider(
        "관찰-a",
        "body-a",
        "owner-a",
        "reg-a",
        vec![Op::EmitResult("provider-a".into())],
    );
    let key = descriptor.registration_identity.clone();
    let (registry, _) = registry_with(vec![descriptor]);
    let mut runtime = InternalTypedReactionRuntime::default();
    let success = execute_ok(
        &mut runtime,
        &[route(vec![])],
        &registry,
        vec![key.clone(), key.clone(), key],
        "exec-1",
        "d1",
    );
    assert_eq!(success.provider_registration_identities.len(), 1);
    assert_eq!(runtime.published_results(), &["provider-a"]);
}

#[test]
fn w1c2_i1_case_07_duplicate_definition_is_error() {
    let descriptor = provider("관찰-a", "body-a", "owner-a", "reg-a", vec![]);
    let duplicate = descriptor.clone();
    let mut registry = ReactionProviderRegistry::default();
    registry
        .register_descriptor(descriptor)
        .expect("first definition");
    assert!(matches!(
        registry.register_descriptor(duplicate),
        Err(InternalProviderIdentityFailure::DuplicateDefinition { .. })
    ));
}

#[test]
fn w1c2_i1_case_08_owner_instances_are_distinct() {
    let a = provider("관찰", "body", "owner-a", "reg", vec![]);
    let b = provider("관찰", "body", "owner-b", "reg", vec![]);
    assert_ne!(a.owner_identity, b.owner_identity);
    assert_ne!(a.registration_identity, b.registration_identity);
}

#[test]
fn w1c2_i1_case_09_execution_instances_are_distinct() {
    let registry = ReactionProviderRegistry::default();
    let routes = vec![route(vec![])];
    let mut a = InternalTypedReactionRuntime::default();
    let mut b = InternalTypedReactionRuntime::default();
    let first = execute_ok(&mut a, &routes, &registry, vec![], "exec-1", "d1");
    let second = execute_ok(&mut b, &routes, &registry, vec![], "exec-2", "d1");
    assert_ne!(first.execution_identity, second.execution_identity);
    assert_ne!(first.occurrence_identity, second.occurrence_identity);
}

#[test]
fn w1c2_i1_case_10_all_participants_read_same_settled_snapshot() {
    let descriptor = provider(
        "관찰",
        "body",
        "owner-a",
        "reg-a",
        vec![Op::RequireSettledValue {
            key: "x".into(),
            expected: Some("old".into()),
        }],
    );
    let (registry, keys) = registry_with(vec![descriptor]);
    let mut world = BTreeMap::new();
    world.insert("x".into(), "old".into());
    let mut runtime = InternalTypedReactionRuntime::with_world_and_queue_capacity(world, 4);
    execute_ok(
        &mut runtime,
        &[route(vec![Op::Set {
            key: "x".into(),
            value: "new".into(),
        }])],
        &registry,
        keys,
        "exec-1",
        "d1",
    );
    assert_eq!(runtime.world().get("x").map(String::as_str), Some("new"));
}

#[test]
fn w1c2_i1_case_11_worker_permutation_is_deterministic() {
    let a = provider(
        "관찰-a",
        "body-a",
        "owner-a",
        "reg-a",
        vec![Op::Set {
            key: "a".into(),
            value: "1".into(),
        }],
    );
    let b = provider(
        "관찰-b",
        "body-b",
        "owner-a",
        "reg-b",
        vec![Op::Set {
            key: "b".into(),
            value: "2".into(),
        }],
    );
    let a_key = a.registration_identity.clone();
    let b_key = b.registration_identity.clone();
    let (registry, _) = registry_with(vec![a, b]);
    let routes = vec![route(vec![Op::EmitTrace("route".into())])];
    let mut left = InternalTypedReactionRuntime::default();
    let mut right = InternalTypedReactionRuntime::default();
    let left_success = execute_ok(
        &mut left,
        &routes,
        &registry,
        vec![a_key.clone(), b_key.clone()],
        "exec-1",
        "d1",
    );
    let right_success = execute_ok(
        &mut right,
        &routes,
        &registry,
        vec![b_key, a_key],
        "exec-1",
        "d1",
    );
    assert_eq!(left.world(), right.world());
    assert_eq!(left_success.reaction_digest, right_success.reaction_digest);
}

#[test]
fn w1c2_i1_case_12_provider_patch_conflict_aborts() {
    let descriptor = provider(
        "관찰",
        "body",
        "owner-a",
        "reg-a",
        vec![Op::Set {
            key: "x".into(),
            value: "provider".into(),
        }],
    );
    let (registry, keys) = registry_with(vec![descriptor]);
    let mut runtime = InternalTypedReactionRuntime::default();
    let failure = runtime
        .execute(
            &[route(vec![Op::Set {
                key: "x".into(),
                value: "route".into(),
            }])],
            &registry,
            keys,
            &envelope("첫알림"),
            ExecutionInstanceIdentity::derive("exec-1"),
            "d1",
            |_, _, _| Ok(()),
        )
        .expect_err("conflicting candidates must abort");
    assert!(matches!(
        failure,
        InternalReactionFailure::PatchConflict { .. }
    ));
    assert!(runtime.world().is_empty());
    assert_eq!(runtime.revision(), 0);
}

#[test]
fn w1c2_i1_case_13_provider_body_failure_aborts() {
    let descriptor = provider(
        "관찰",
        "body",
        "owner-a",
        "reg-a",
        vec![Op::Fail {
            code: "E_PROVIDER_BODY_TEST".into(),
        }],
    );
    let (registry, keys) = registry_with(vec![descriptor]);
    let mut runtime = InternalTypedReactionRuntime::default();
    let failure = runtime
        .execute(
            &[route(vec![Op::Set {
                key: "route".into(),
                value: "candidate".into(),
            }])],
            &registry,
            keys,
            &envelope("첫알림"),
            ExecutionInstanceIdentity::derive("exec-1"),
            "d1",
            |_, _, _| Ok(()),
        )
        .expect_err("provider body failure must abort group");
    assert!(matches!(
        failure,
        InternalReactionFailure::BodyFailure { .. }
    ));
    assert!(runtime.world().is_empty());
}

#[test]
fn w1c2_i1_case_14_failure_publishes_nothing() {
    let descriptor = provider(
        "관찰",
        "body",
        "owner-a",
        "reg-a",
        vec![Op::Fail {
            code: "E_PROVIDER_BODY_TEST".into(),
        }],
    );
    let (registry, keys) = registry_with(vec![descriptor]);
    let mut runtime = InternalTypedReactionRuntime::default();
    let mut releases = 0usize;
    let _ = runtime.execute(
        &[route(vec![
            Op::EmitResult("result".into()),
            Op::EmitTrace("trace".into()),
            Op::EmitReceipt("receipt".into()),
            Op::StageSend(envelope("nested")),
            Op::StageEffect(InternalEffectIntent::new("external")),
        ])],
        &registry,
        keys,
        &envelope("첫알림"),
        ExecutionInstanceIdentity::derive("exec-1"),
        "d1",
        |_, _, _| {
            releases += 1;
            Ok(())
        },
    );
    assert_eq!(runtime.revision(), 0);
    assert!(runtime.published_results().is_empty());
    assert!(runtime.published_traces().is_empty());
    assert!(runtime.published_receipts().is_empty());
    assert_eq!(runtime.pending_delivery_count(), 0);
    assert_eq!(releases, 0);
}

#[test]
fn w1c2_i1_case_15_nested_send_is_staged_outbox() {
    let registry = ReactionProviderRegistry::default();
    let mut runtime = InternalTypedReactionRuntime::default();
    execute_ok(
        &mut runtime,
        &[route(vec![Op::StageSend(envelope("nested"))])],
        &registry,
        vec![],
        "exec-1",
        "d1",
    );
    assert_eq!(runtime.pending_delivery_count(), 1);
}

#[test]
fn w1c2_i1_case_16_nested_send_does_not_reenter() {
    let registry = ReactionProviderRegistry::default();
    let mut runtime = InternalTypedReactionRuntime::default();
    execute_ok(
        &mut runtime,
        &[route(vec![Op::StageSend(envelope("nested"))])],
        &registry,
        vec![],
        "exec-1",
        "d1",
    );
    assert!(runtime.world().is_empty());
    assert_eq!(runtime.pop_next_delivery().unwrap().message_type, "nested");
    assert!(runtime.world().is_empty());
}

#[test]
fn w1c2_i1_case_17_bounded_full_is_wait_and_fifo_resume() {
    let registry = ReactionProviderRegistry::default();
    let routes = vec![route(vec![Op::StageSend(envelope("nested"))])];
    let mut runtime =
        InternalTypedReactionRuntime::with_world_and_queue_capacity(BTreeMap::new(), 1);
    let first = execute_ok(&mut runtime, &routes, &registry, vec![], "exec-1", "d1");
    let second = execute_ok(&mut runtime, &routes, &registry, vec![], "exec-1", "d2");
    assert!(first.wait_receipts.is_empty());
    assert_eq!(second.wait_receipts.len(), 1);
    assert_eq!(runtime.pending_delivery_count(), 1);
    assert_eq!(runtime.waiting_delivery_count(), 1);
    runtime.pop_next_delivery();
    assert_eq!(runtime.pending_delivery_count(), 1);
    assert_eq!(runtime.waiting_delivery_count(), 0);
    assert_eq!(runtime.drain_resumed_waits().len(), 1);
}

#[test]
fn w1c2_i1_case_18_abort_discards_nested_outbox() {
    let descriptor = provider(
        "관찰",
        "body",
        "owner-a",
        "reg-a",
        vec![Op::Fail {
            code: "E_PROVIDER_BODY_TEST".into(),
        }],
    );
    let (registry, keys) = registry_with(vec![descriptor]);
    let mut runtime = InternalTypedReactionRuntime::default();
    let _ = runtime.execute(
        &[route(vec![Op::StageSend(envelope("nested"))])],
        &registry,
        keys,
        &envelope("첫알림"),
        ExecutionInstanceIdentity::derive("exec-1"),
        "d1",
        |_, _, _| Ok(()),
    );
    assert_eq!(runtime.pending_delivery_count(), 0);
    assert_eq!(runtime.waiting_delivery_count(), 0);
}

#[test]
fn w1c2_i1_case_19_external_effect_release_is_after_commit() {
    let registry = ReactionProviderRegistry::default();
    let mut runtime = InternalTypedReactionRuntime::default();
    let mut observed_revision = 0u64;
    let success = runtime
        .execute(
            &[route(vec![
                Op::Set {
                    key: "x".into(),
                    value: "committed".into(),
                },
                Op::StageEffect(InternalEffectIntent::new("external")),
            ])],
            &registry,
            vec![],
            &envelope("첫알림"),
            ExecutionInstanceIdentity::derive("exec-1"),
            "d1",
            |_, committed_world, committed_revision| {
                assert_eq!(
                    committed_world.get("x").map(String::as_str),
                    Some("committed")
                );
                observed_revision = committed_revision;
                Ok(())
            },
        )
        .expect("commit then release");
    assert_eq!(observed_revision, 1);
    assert_eq!(success.released_effect_identities, &["external"]);
    assert_eq!(
        runtime.world().get("x").map(String::as_str),
        Some("committed")
    );
}

#[test]
fn w1c2_i1_case_20_historical_hook_is_not_auto_registered() {
    let source = r#"
(값:수) 첫알림:알림씨 = { 없음. }
관제탑:임자 = { 첫알림을 받으면 { 없음. }. }
매틱:움직씨 = { (철수)의 ((값=1) 첫알림) ~> 관제탑. }
"#;
    let program = DdnProgram::from_source(source, "historical.ddn").expect("historical parses");
    let runner = DdnRunner::new(program, "매틱");
    assert_eq!(runner.public_provider_source_ingress_count(), 0);
}

#[test]
fn w1c2_i1_case_21_receive_is_not_reinterpreted_as_provider() {
    let source = r#"
(값:수) 첫알림:알림씨 = { 없음. }
관제탑:임자 = { (받은수:수)를 받으면 { 없음. }. }
매틱:움직씨 = { 없음. }
"#;
    let program = DdnProgram::from_source(source, "receive.ddn").expect("receive parses");
    let runner = DdnRunner::new(program, "매틱");
    assert_eq!(runner.public_provider_source_ingress_count(), 0);
}

#[test]
fn w1c2_i1_case_22_tool_ddnrunner_public_provider_ingress_is_unsupported() {
    let source = r#"
(값:수) 첫알림:알림씨 = { 없음. }
관제탑:임자 = { 첫알림을 받으면 { 없음. }. }
매틱:움직씨 = { 없음. }
"#;
    let program = DdnProgram::from_source(source, "teul-unsupported.ddn").expect("source parses");
    let runner = DdnRunner::new(program, "매틱");
    assert_eq!(runner.public_provider_source_ingress_count(), 0);
}

#[test]
fn w1c2_i1_case_23_external_release_failure_does_not_rewind_world() {
    let registry = ReactionProviderRegistry::default();
    let mut runtime = InternalTypedReactionRuntime::default();
    let success = runtime
        .execute(
            &[route(vec![
                Op::Set {
                    key: "x".into(),
                    value: "committed".into(),
                },
                Op::StageEffect(InternalEffectIntent::new("external")),
            ])],
            &registry,
            vec![],
            &envelope("첫알림"),
            ExecutionInstanceIdentity::derive("exec-1"),
            "d1",
            |_, _, _| Err("host unavailable".into()),
        )
        .expect("internal commit remains successful");
    assert_eq!(runtime.revision(), 1);
    assert_eq!(
        runtime.world().get("x").map(String::as_str),
        Some("committed")
    );
    assert_eq!(success.external_release_failures.len(), 1);
    assert!(runtime.released_effects().is_empty());
}
