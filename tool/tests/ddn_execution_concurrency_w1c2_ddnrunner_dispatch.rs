// Cargo discovers this path as an integration target, but the cases are expanded
// only inside ddn_runtime's unit-test module. That keeps the typed injection
// helpers crate-confined while still exercising the real DdnRunner entrypoint.
macro_rules! w1c2_i2_ddnrunner_dispatch_tests {
() => {
use ddonirang_core::platform::NuriWorld;
use ddonirang_core::{Fixed64, InputSnapshot, InputSource};
use ddonirang_lang::runtime::Value as RuntimeValue;
use ddonirang_lang::{
    ExecutionInstanceIdentity, InternalDeliveryEnvelopeSemantic, ProviderRegistrationIdentity,
    ReactionProviderDescriptor, ReactionProviderPredicate, ReactionProviderRegistry,
    SenderIdentityPredicate,
};
use super::{
    DdnProgram, DdnRunner, InternalEffectIntent, InternalReactionOperation as Op,
    InternalReactionProgram, InternalTypedReactionRuntime,
};
use std::collections::{BTreeMap, HashMap};

const ONE_ROUTE: &str = r#"
채비 { 받은:글 <- "". }.
(값:수) 첫알림:알림씨 = { 없음. }
관제탑:임자 = {
  (받은알림:첫알림)를 받으면 { 받은 <- "route". }.
}
매틱:움직씨 = { (철수)의 ((값=1) 첫알림) ~> 관제탑. }
"#;

const ZERO_ROUTE: &str = r#"
(값:수) 첫알림:알림씨 = { 없음. }
(값:수) 둘알림:알림씨 = { 없음. }
관제탑:임자 = { (받은알림:둘알림)를 받으면 { 없음. }. }
매틱:움직씨 = { (철수)의 ((값=1) 첫알림) ~> 관제탑. }
"#;

const TWO_ROUTES: &str = r#"
(값:수) 첫알림:알림씨 = { 없음. }
관제탑:임자 = {
  (받은알림:첫알림)를 받으면 { 없음. }.
  (받은알림:첫알림)를 받으면 { 없음. }.
}
매틱:움직씨 = { (철수)의 ((값=1) 첫알림) ~> 관제탑. }
"#;

fn empty_input() -> InputSnapshot {
    InputSnapshot {
        tick_id: 0,
        dt: Fixed64::from_i64(1),
        keys_pressed: 0,
        last_key_name: String::new(),
        pointer_x_i32: 0,
        pointer_y_i32: 0,
        ai_injections: Vec::new(),
        net_events: Vec::new(),
        frame_source: InputSource::Person,
        rng_seed: 0,
    }
}

fn defaults() -> HashMap<String, RuntimeValue> {
    HashMap::from([("받은".to_string(), RuntimeValue::String(String::new()))])
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
        InternalReactionProgram::new(operations),
    )
}

fn registry_with(
    descriptors: Vec<ReactionProviderDescriptor<InternalReactionProgram>>,
) -> (
    ReactionProviderRegistry<InternalReactionProgram>,
    Vec<ProviderRegistrationIdentity>,
) {
    let mut registry = ReactionProviderRegistry::default();
    let mut keys = Vec::new();
    for descriptor in descriptors {
        keys.push(descriptor.registration_identity.clone());
        registry
            .register_descriptor(descriptor)
            .expect("unique internal provider descriptor");
    }
    (registry, keys)
}

fn runner_with(
    source: &str,
    registry: ReactionProviderRegistry<InternalReactionProgram>,
    execution: &str,
) -> DdnRunner {
    let program = DdnProgram::from_source(source, "w1c2_i2.ddn").expect("source parses");
    DdnRunner::new(program, "매틱").with_internal_reaction_provider_registry(
        registry,
        ExecutionInstanceIdentity::derive(execution),
    )
}

#[test]
fn w1c2_i2_case_01_ddnrunner_route_zero_provider_callback_zero() {
    let descriptor = provider(
        "관찰",
        "body",
        "owner",
        "reg",
        vec![Op::StageEffect(InternalEffectIntent::new(
            "must-not-release",
        ))],
    );
    let unknown_key = descriptor.registration_identity.clone();
    let mut runner = runner_with(
        ZERO_ROUTE,
        ReactionProviderRegistry::default(),
        "exec-zero",
    )
    .with_internal_provider_reachability(vec![unknown_key]);
    let error = match runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()) {
        Ok(_) => panic!("zero route must fail"),
        Err(error) => error,
    };
    assert!(error.contains("E_ASYNC_ENDPOINT_UNHANDLED_CANDIDATE"));
    assert_eq!(runner.internal_reaction_runtime().revision(), 0);
    assert!(runner
        .internal_reaction_runtime()
        .released_effects()
        .is_empty());
}

#[test]
fn w1c2_i2_case_02_ddnrunner_route_two_provider_callback_zero() {
    let descriptor = provider(
        "관찰",
        "body",
        "owner",
        "reg",
        vec![Op::StageEffect(InternalEffectIntent::new(
            "must-not-release",
        ))],
    );
    let unknown_key = descriptor.registration_identity.clone();
    let mut runner = runner_with(
        TWO_ROUTES,
        ReactionProviderRegistry::default(),
        "exec-two",
    )
    .with_internal_provider_reachability(vec![unknown_key]);
    let error = match runner.run_update(&NuriWorld::new(), &empty_input(), &HashMap::new()) {
        Ok(_) => panic!("ambiguous route must fail"),
        Err(error) => error,
    };
    assert!(error.contains("E_ASYNC_ENDPOINT_AMBIGUOUS_CANDIDATE"));
    assert_eq!(runner.internal_reaction_runtime().revision(), 0);
    assert!(runner
        .internal_reaction_runtime()
        .released_effects()
        .is_empty());
}

#[test]
fn w1c2_i2_case_03_ddnrunner_route_one_empty_registry_preserves_result() {
    let program = DdnProgram::from_source(ONE_ROUTE, "w1c2_i2.ddn").expect("source parses");
    let mut runner = DdnRunner::new(program, "매틱");
    assert_eq!(runner.internal_provider_registry_len(), 0);
    let output = runner
        .run_update(&NuriWorld::new(), &empty_input(), &defaults())
        .expect("one route with empty registry succeeds");
    assert_eq!(
        output.resources.get("받은"),
        Some(&RuntimeValue::String("route".to_string()))
    );
    assert!(runner
        .last_internal_reaction_success()
        .expect("shared dispatch success")
        .provider_registration_identities
        .is_empty());
}

#[test]
fn w1c2_i2_case_04_ddnrunner_route_one_provider_candidate_commits() {
    let descriptor = provider(
        "관찰-a",
        "body-a",
        "owner",
        "reg-a",
        vec![Op::Set {
            key: "provider-a".into(),
            value: "seen".into(),
        }],
    );
    let (registry, _) = registry_with(vec![descriptor]);
    let mut runner = runner_with(ONE_ROUTE, registry, "exec-one-provider");
    runner
        .run_update(&NuriWorld::new(), &empty_input(), &defaults())
        .expect("provider candidate commits through DdnRunner");
    assert_eq!(
        runner
            .internal_reaction_runtime()
            .world()
            .get("provider-a")
            .map(String::as_str),
        Some("seen")
    );
}

#[test]
fn w1c2_i2_case_05_ddnrunner_diamond_reachability_executes_once() {
    let descriptor = provider(
        "관찰-a",
        "body-a",
        "owner",
        "reg-a",
        vec![Op::EmitResult("provider-a".into())],
    );
    let key = descriptor.registration_identity.clone();
    let (registry, _) = registry_with(vec![descriptor]);
    let mut runner = runner_with(ONE_ROUTE, registry, "exec-diamond")
        .with_internal_provider_reachability(vec![key.clone(), key.clone(), key]);
    runner
        .run_update(&NuriWorld::new(), &empty_input(), &defaults())
        .expect("diamond reachability succeeds");
    assert_eq!(
        runner.internal_reaction_runtime().published_results(),
        &["provider-a"]
    );
    assert_eq!(
        runner
            .last_internal_reaction_success()
            .unwrap()
            .provider_registration_identities
            .len(),
        1
    );
}

#[test]
fn w1c2_i2_case_06_ddnrunner_distinct_providers_both_execute() {
    let a = provider(
        "관찰-a",
        "body-a",
        "owner",
        "reg-a",
        vec![Op::Set {
            key: "a".into(),
            value: "1".into(),
        }],
    );
    let b = provider(
        "관찰-b",
        "body-b",
        "owner",
        "reg-b",
        vec![Op::Set {
            key: "b".into(),
            value: "2".into(),
        }],
    );
    let (registry, _) = registry_with(vec![a, b]);
    let mut runner = runner_with(ONE_ROUTE, registry, "exec-distinct");
    runner
        .run_update(&NuriWorld::new(), &empty_input(), &defaults())
        .expect("distinct providers commit");
    let world = runner.internal_reaction_runtime().world();
    assert_eq!(world.get("a").map(String::as_str), Some("1"));
    assert_eq!(world.get("b").map(String::as_str), Some("2"));
}

#[test]
fn w1c2_i2_case_07_ddnrunner_registration_and_worker_permutation_deterministic() {
    let a = provider(
        "관찰-a",
        "body-a",
        "owner",
        "reg-a",
        vec![Op::Set {
            key: "a".into(),
            value: "1".into(),
        }],
    );
    let b = provider(
        "관찰-b",
        "body-b",
        "owner",
        "reg-b",
        vec![Op::Set {
            key: "b".into(),
            value: "2".into(),
        }],
    );
    let a_key = a.registration_identity.clone();
    let b_key = b.registration_identity.clone();
    let (left_registry, _) = registry_with(vec![a.clone(), b.clone()]);
    let (right_registry, _) = registry_with(vec![b, a]);
    let mut left = runner_with(ONE_ROUTE, left_registry, "exec-permutation")
        .with_internal_provider_reachability(vec![a_key.clone(), b_key.clone()]);
    let mut right = runner_with(ONE_ROUTE, right_registry, "exec-permutation")
        .with_internal_provider_reachability(vec![b_key, a_key]);
    left.run_update(&NuriWorld::new(), &empty_input(), &defaults())
        .unwrap();
    right
        .run_update(&NuriWorld::new(), &empty_input(), &defaults())
        .unwrap();
    assert_eq!(
        left.internal_reaction_runtime().world(),
        right.internal_reaction_runtime().world()
    );
    assert_eq!(
        left.last_internal_reaction_success()
            .unwrap()
            .reaction_digest,
        right
            .last_internal_reaction_success()
            .unwrap()
            .reaction_digest
    );
}

#[test]
fn w1c2_i2_case_08_ddnrunner_providers_share_one_settled_snapshot() {
    let writer = provider(
        "writer",
        "body-writer",
        "owner",
        "reg-writer",
        vec![Op::Set {
            key: "x".into(),
            value: "new".into(),
        }],
    );
    let observer = provider(
        "observer",
        "body-observer",
        "owner",
        "reg-observer",
        vec![Op::RequireSettledValue {
            key: "x".into(),
            expected: Some("old".into()),
        }],
    );
    let (registry, _) = registry_with(vec![writer, observer]);
    let mut world = BTreeMap::new();
    world.insert("x".to_string(), "old".to_string());
    let runtime = InternalTypedReactionRuntime::with_world_and_queue_capacity(world, 8);
    let mut runner =
        runner_with(ONE_ROUTE, registry, "exec-settled").with_internal_reaction_runtime(runtime);
    runner
        .run_update(&NuriWorld::new(), &empty_input(), &defaults())
        .unwrap();
    assert_eq!(
        runner
            .internal_reaction_runtime()
            .world()
            .get("x")
            .map(String::as_str),
        Some("new")
    );
}

#[test]
fn w1c2_i2_case_09_ddnrunner_provider_failure_aborts_route_publish() {
    let descriptor = provider(
        "failure",
        "body-failure",
        "owner",
        "reg-failure",
        vec![
            Op::Set {
                key: "must-not-commit".into(),
                value: "provider-failure".into(),
            },
            Op::StageSend(InternalDeliveryEnvelopeSemantic::new(
                "내부수신자",
                "내부알림",
                "관제탑",
                "ANY_PAYLOAD",
            )),
            Op::EmitReceipt("must-not-publish".into()),
            Op::StageEffect(InternalEffectIntent::new("must-not-release")),
            Op::Fail {
                code: "E_PROVIDER_TEST".into(),
            },
        ],
    );
    let (registry, _) = registry_with(vec![descriptor]);
    let mut runner = runner_with(ONE_ROUTE, registry, "exec-failure");
    let error = match runner.run_update(&NuriWorld::new(), &empty_input(), &defaults()) {
        Ok(_) => panic!("provider failure rejects complete route group"),
        Err(error) => error,
    };
    assert!(error.contains("E_ASYNC_PROVIDER_BODY_CANDIDATE"));
    assert_eq!(runner.internal_reaction_runtime().revision(), 0);
    assert!(runner.internal_reaction_runtime().world().is_empty());
    assert_eq!(runner.internal_reaction_runtime().pending_delivery_count(), 0);
    assert_eq!(runner.internal_reaction_runtime().waiting_delivery_count(), 0);
    assert!(runner.internal_reaction_runtime().published_receipts().is_empty());
    assert!(runner.internal_reaction_runtime().released_effects().is_empty());
    assert!(runner.internal_effect_sink_observations().is_empty());
    assert!(runner.internal_reaction_commit_events().is_empty());
    assert!(runner.last_internal_reaction_success().is_none());
}

#[test]
fn w1c2_i2_case_10_ddnrunner_provider_conflict_is_deterministic_failure() {
    let a = provider(
        "a",
        "body-a",
        "owner",
        "reg-a",
        vec![
            Op::Set {
                key: "x".into(),
                value: "a".into(),
            },
            Op::StageSend(InternalDeliveryEnvelopeSemantic::new(
                "내부수신자",
                "내부알림",
                "관제탑",
                "ANY_PAYLOAD",
            )),
            Op::EmitReceipt("must-not-publish".into()),
            Op::StageEffect(InternalEffectIntent::new("must-not-release")),
        ],
    );
    let b = provider(
        "b",
        "body-b",
        "owner",
        "reg-b",
        vec![Op::Set {
            key: "x".into(),
            value: "b".into(),
        }],
    );
    let (registry, _) = registry_with(vec![a, b]);
    let mut runner = runner_with(ONE_ROUTE, registry, "exec-conflict");
    let error = match runner.run_update(&NuriWorld::new(), &empty_input(), &defaults()) {
        Ok(_) => panic!("conflicting provider patches reject group"),
        Err(error) => error,
    };
    assert!(error.contains("E_ASYNC_PROVIDER_PATCH_CONFLICT_CANDIDATE"));
    assert_eq!(runner.internal_reaction_runtime().revision(), 0);
    assert!(runner.internal_reaction_runtime().world().is_empty());
    assert_eq!(runner.internal_reaction_runtime().pending_delivery_count(), 0);
    assert_eq!(runner.internal_reaction_runtime().waiting_delivery_count(), 0);
    assert!(runner.internal_reaction_runtime().published_receipts().is_empty());
    assert!(runner.internal_reaction_runtime().released_effects().is_empty());
    assert!(runner.internal_effect_sink_observations().is_empty());
    assert!(runner.internal_reaction_commit_events().is_empty());
    assert!(runner.last_internal_reaction_success().is_none());
}

#[test]
fn w1c2_i2_case_11_ddnrunner_effect_intent_releases_after_outer_commit() {
    let descriptor = provider(
        "effect",
        "body-effect",
        "owner",
        "reg-effect",
        vec![
            Op::Set {
                key: "effect-visible".into(),
                value: "committed".into(),
            },
            Op::StageEffect(InternalEffectIntent::new("external-a")),
        ],
    );
    let (registry, _) = registry_with(vec![descriptor]);
    let mut runner = runner_with(ONE_ROUTE, registry, "exec-effect");
    runner
        .run_update(&NuriWorld::new(), &empty_input(), &defaults())
        .unwrap();
    assert_eq!(
        runner.internal_reaction_commit_events(),
        &["ddn_route_frame_committed", "effect_released:external-a"]
    );
    assert_eq!(runner.internal_reaction_runtime().revision(), 1);
    assert_eq!(runner.internal_reaction_runtime().world().len(), 1);
    assert_eq!(
        runner.internal_reaction_runtime().world().get("effect-visible"),
        Some(&"committed".to_string())
    );
    assert_eq!(runner.internal_reaction_runtime().released_effects(), &["external-a"]);
    let observations = runner.internal_effect_sink_observations();
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0].identity, "external-a");
    assert_eq!(
        observations[0].committed_world.get("effect-visible"),
        Some(&"committed".to_string())
    );
    assert_eq!(observations[0].committed_revision, 1);
    assert!(observations[0].route_frame_committed);
}

#[test]
fn w1c2_i2_case_12_ddnrunner_wait_resume_preserves_provider_outbox() {
    let source = r#"
(값:수) 첫알림:알림씨 = { 없음. }
관제탑:임자 = { (받은알림:첫알림)를 받으면 { 없음. }. }
매틱:움직씨 = {
  (철수)의 ((값=1) 첫알림) ~> 관제탑.
  (철수)의 ((값=2) 첫알림) ~> 관제탑.
}
"#;
    let descriptor = provider(
        "outbox",
        "body-outbox",
        "owner",
        "reg-outbox",
        vec![Op::StageSend(InternalDeliveryEnvelopeSemantic::new(
            "내부수신자",
            "내부알림",
            "관제탑",
            "ANY_PAYLOAD",
        ))],
    );
    let (registry, _) = registry_with(vec![descriptor]);
    let runtime = InternalTypedReactionRuntime::with_world_and_queue_capacity(BTreeMap::new(), 1);
    let mut runner =
        runner_with(source, registry, "exec-wait").with_internal_reaction_runtime(runtime);
    runner
        .run_update(&NuriWorld::new(), &empty_input(), &HashMap::new())
        .unwrap();
    assert_eq!(
        runner.internal_reaction_runtime().pending_delivery_count(),
        1
    );
    assert_eq!(
        runner.internal_reaction_runtime().waiting_delivery_count(),
        1
    );
    assert_eq!(
        runner
            .last_internal_reaction_success()
            .unwrap()
            .wait_receipts
            .len(),
        1
    );
}

#[test]
fn w1c2_i2_case_13_ddnrunner_nested_delivery_preserves_fifo_no_reentry() {
    let source = r#"
채비 { 순서:수 <- 0. }.
(값:수) 첫알림:알림씨 = { 없음. }
(값:수) 둘알림:알림씨 = { 없음. }
관제탑:임자 = {
  (받은알림:첫알림)를 받으면 {
    순서 <- 순서 * 10 + 1.
    ((값=0) 둘알림) ~> 제.
    순서 <- 순서 * 10 + 2.
  }.
  (받은알림:둘알림)를 받으면 { 순서 <- 순서 * 10 + 3. }.
}
매틱:움직씨 = { (철수)의 ((값=1) 첫알림) ~> 관제탑. }
"#;
    let descriptor = provider(
        "first-only",
        "body-first",
        "owner",
        "reg-first",
        vec![Op::EmitResult("first-provider".into())],
    );
    let (registry, _) = registry_with(vec![descriptor]);
    let mut runner = runner_with(source, registry, "exec-nested");
    let defaults = HashMap::from([(
        "순서".to_string(),
        RuntimeValue::Fixed64(Fixed64::from_i64(0)),
    )]);
    let output = runner
        .run_update(&NuriWorld::new(), &empty_input(), &defaults)
        .unwrap();
    assert_eq!(
        output.resources.get("순서"),
        Some(&RuntimeValue::Fixed64(Fixed64::from_i64(123)))
    );
    assert_eq!(
        runner.internal_reaction_runtime().published_results(),
        &["first-provider"]
    );
}

#[test]
fn w1c2_i2_case_14_ddnrunner_execution_instance_changes_occurrence_identity() {
    let descriptor = provider("observer", "body", "owner", "reg", vec![]);
    let (left_registry, _) = registry_with(vec![descriptor.clone()]);
    let (right_registry, _) = registry_with(vec![descriptor]);
    let mut left = runner_with(ONE_ROUTE, left_registry, "execution-a");
    let mut right = runner_with(ONE_ROUTE, right_registry, "execution-b");
    left.run_update(&NuriWorld::new(), &empty_input(), &defaults())
        .unwrap();
    right
        .run_update(&NuriWorld::new(), &empty_input(), &defaults())
        .unwrap();
    let left_success = left.last_internal_reaction_success().unwrap();
    let right_success = right.last_internal_reaction_success().unwrap();
    assert_ne!(
        left_success.execution_identity,
        right_success.execution_identity
    );
    assert_ne!(
        left_success.occurrence_identity,
        right_success.occurrence_identity
    );
}

#[test]
fn w1c2_i2_case_15_ddnrunner_occurrence_duplicate_attack_applies_once() {
    let descriptor = provider(
        "once",
        "body-once",
        "owner",
        "reg-once",
        vec![Op::EmitReceipt("once".into())],
    );
    let key = descriptor.registration_identity.clone();
    let (registry, _) = registry_with(vec![descriptor]);
    let mut runner = runner_with(ONE_ROUTE, registry, "exec-duplicate")
        .with_internal_provider_reachability(vec![key.clone(), key.clone(), key.clone(), key]);
    runner
        .run_update(&NuriWorld::new(), &empty_input(), &defaults())
        .unwrap();
    assert_eq!(
        runner.internal_reaction_runtime().published_receipts(),
        &["once"]
    );
    assert_eq!(
        runner
            .last_internal_reaction_success()
            .unwrap()
            .provider_registration_identities
            .len(),
        1
    );
}
};
}
