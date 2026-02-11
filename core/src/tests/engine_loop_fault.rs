use crate::{
    ArithmeticFaultKind, EngineLoop, ExprTrace, Fixed64, SourceSpan, TickId,
    platform::{
        Bogae, ComponentTag, DetNuri, DetSam, EntityId, Geoul, InMemoryGeoul, Iyagi, Nuri,
        NuriWorld, Origin, Patch, PatchOp,
    },
    signals::{Signal, VecSignalSink},
};

struct Div0Iyagi;

impl Iyagi for Div0Iyagi {
    fn run_startup(&mut self, _world: &NuriWorld) -> Patch {
        Patch::default()
    }

    fn run_update(
        &mut self,
        _world: &NuriWorld,
        input: &crate::platform::InputSnapshot,
    ) -> Patch {
        // ✅ resource "x"에 대해 x /= 0 수행하도록 Patch 생성
        Patch {
            ops: vec![PatchOp::DivAssignResourceFixed64 {
                tag: "x".to_string(),
                rhs: Fixed64::ZERO,
                tick_id: input.tick_id,
                location: "iyagi:Div0Iyagi/x_div0",
                source_span: Some(SourceSpan {
                    file: "core/src/tests/engine_loop_fault.rs".to_string(),
                    start_line: 1,
                    start_col: Some(1),
                    end_line: 1,
                    end_col: Some(1),
                }),
                expr: Some(ExprTrace {
                    tag: "arith:DIV0".to_string(),
                    text: None,
                }),
            }],
            origin: Origin::system("test"),
        }
    }
}

#[derive(Default)]
struct SpyBogae {
    pub last_tick: Option<TickId>,
    pub renders: u64,
}

impl Bogae for SpyBogae {
    fn render(&mut self, _world: &NuriWorld, tick_id: TickId) {
        self.last_tick = Some(tick_id);
        self.renders += 1;
    }
}

#[test]
fn engine_loop_div0_fault_flows_to_nuri_signal_sink() {
    // 0) 준비
    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = Div0Iyagi;

    let mut nuri = DetNuri::new();
    // ✅ x = 5 (Fixed64)
    nuri.world_mut().set_resource_fixed64("x".to_string(), Fixed64::from_i64(5));
    let before = nuri.world().get_resource_fixed64("x").unwrap();

    let geoul = InMemoryGeoul::new();
    let bogae = SpyBogae::default();

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);

    let mut sink = VecSignalSink::default();

    // 1) 한 틱 실행
    let frame = loop_.tick_once(7, &mut sink);

    // 2) 산술고장 신호가 sink에 들어왔는지
    assert_eq!(sink.signals.len(), 1);
    match &sink.signals[0] {
        Signal::ArithmeticFault { ctx, kind } => {
            assert_eq!(ctx.tick_id, 7);
            assert_eq!(ctx.location, "iyagi:Div0Iyagi/x_div0");
            assert_eq!(*kind, ArithmeticFaultKind::DivByZero);
        }
        _ => panic!("예상하지 못한 알림"),
    }

    // 3) "대입 무효화" => x 값이 유지되는지
    let after = loop_.nuri.world().get_resource_fixed64("x").unwrap();
    assert_eq!(after, before);

    assert!(sink.diag_events.iter().any(|event| {
        event
            .expr
            .as_ref()
            .map(|expr| expr.tag.as_str() == "arith:DIV0")
            .unwrap_or(false)
            && event.source_span.is_some()
    }));

    // 4) Geoul 기록 확인 (1개 프레임)
    assert_eq!(loop_.geoul.len(), 1);
    assert_eq!(frame.snapshot.tick_id, 7);

    // 4-1) 리플레이 프레임 확인
    let replayed = loop_.geoul.replay_next().expect("replay frame");
    assert_eq!(replayed.state_hash, frame.state_hash);

    // 5) Bogae render 호출 확인
    assert_eq!(loop_.bogae.last_tick, Some(7));
    assert_eq!(loop_.bogae.renders, 1);
}

#[test]
fn engine_loop_div0_fault_on_missing_resource_does_not_create_resource() {
    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = Div0Iyagi;

    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = SpyBogae::default();

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let frame = loop_.tick_once(1, &mut sink);

    assert_eq!(sink.signals.len(), 1);
    let after = loop_.nuri.world().get_resource_fixed64("x");
    assert!(after.is_none());
    assert_eq!(frame.snapshot.tick_id, 1);
}

#[test]
fn guard_violation_drops_origin_assignments_and_marks_entity() {
    let mut nuri = DetNuri::new();
    nuri
        .world_mut()
        .set_resource_fixed64("x".to_string(), Fixed64::from_i64(1));

    let entity = EntityId(7);
    let patch = Patch {
        ops: vec![
            PatchOp::SetResourceFixed64 {
                tag: "x".to_string(),
                value: Fixed64::from_i64(99),
            },
            PatchOp::GuardViolation {
                entity,
                rule_id: "RULE_BOUNDARY".to_string(),
            },
        ],
        origin: Origin::Entity(entity),
    };

    let mut sink = VecSignalSink::default();
    nuri.apply_patch(&patch, 3, &mut sink);

    let x = nuri.world().get_resource_fixed64("x").unwrap();
    assert_eq!(x.int_part(), 1);

    let rule_tag = ComponentTag("#규칙위반".to_string());
    let disabled_tag = ComponentTag("#휴면".to_string());
    assert_eq!(
        nuri.world().get_component_json(entity, &rule_tag).as_deref(),
        Some("참")
    );
    assert_eq!(
        nuri.world().get_component_json(entity, &disabled_tag).as_deref(),
        Some("참")
    );

    assert!(sink
        .diag_events
        .iter()
        .any(|event| event.reason == "GUARD_VIOLATION" && event.rule_id == "RULE_BOUNDARY"));
}
