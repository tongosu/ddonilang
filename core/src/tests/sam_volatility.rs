use std::collections::VecDeque;

use crate::{
    EngineLoop, Fixed64, TickId, KEY_W,
    platform::{
        Bogae, DetNuri, InMemoryGeoul, InputSnapshot, Iyagi, Nuri, NuriWorld, Origin, Patch,
        PatchOp, Sam, SeulgiIntent, SeulgiPacket,
    },
    signals::VecSignalSink,
};

struct SequenceSam {
    snapshots: VecDeque<InputSnapshot>,
    ai_queue: Vec<SeulgiPacket>,
}

impl SequenceSam {
    fn new(snapshots: Vec<InputSnapshot>) -> Self {
        Self {
            snapshots: VecDeque::from(snapshots),
            ai_queue: Vec::new(),
        }
    }
}

impl Sam for SequenceSam {
    fn begin_tick(&mut self, tick_id: TickId) -> InputSnapshot {
        let mut snapshot = self.snapshots.pop_front().expect("snapshot");
        self.ai_queue.sort();
        snapshot.ai_injections = core::mem::take(&mut self.ai_queue);
        snapshot.tick_id = tick_id;
        snapshot
    }

    fn push_async_ai(
        &mut self,
        agent_id: u64,
        recv_seq: u64,
        accepted_madi: u64,
        target_madi: u64,
        intent: SeulgiIntent,
    ) {
        self.ai_queue.push(SeulgiPacket {
            agent_id,
            recv_seq,
            accepted_madi,
            target_madi,
            intent,
        });
    }
}

struct InputScopeIyagi;

impl Iyagi for InputScopeIyagi {
    fn run_startup(&mut self, _world: &NuriWorld) -> Patch {
        Patch::default()
    }

    fn run_update(
        &mut self,
        world: &NuriWorld,
        input: &InputSnapshot,
    ) -> Patch {
        let mut ops = Vec::new();
        ops.push(PatchOp::SetResourceJson {
            tag: "current_key".to_string(),
            json: input.last_key_name.clone(),
        });

        if !input.last_key_name.is_empty() && world.get_resource_json("copied_key").is_none() {
            ops.push(PatchOp::SetResourceJson {
                tag: "copied_key".to_string(),
                json: input.last_key_name.clone(),
            });
        }

        Patch {
            ops,
            origin: Origin::system("test"),
        }
    }
}

struct NoopBogae;

impl Bogae for NoopBogae {
    fn render(&mut self, _world: &NuriWorld, _tick_id: TickId) {}
}

#[test]
fn sam_input_is_tick_scoped_and_requires_copy() {
    let snapshots = vec![
        InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: KEY_W,
            last_key_name: "w".to_string(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        },
        InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        },
    ];

    let sam = SequenceSam::new(snapshots);
    let iyagi = InputScopeIyagi;
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let frame1 = loop_.tick_once(1, &mut sink);
    assert_eq!(frame1.snapshot.last_key_name, "w");
    assert_eq!(
        loop_.nuri.world().get_resource_json("current_key").as_deref(),
        Some("w")
    );
    assert_eq!(
        loop_.nuri.world().get_resource_json("copied_key").as_deref(),
        Some("w")
    );

    let frame2 = loop_.tick_once(2, &mut sink);
    assert_eq!(frame2.snapshot.last_key_name, "");
    assert_eq!(
        loop_.nuri.world().get_resource_json("current_key").as_deref(),
        Some("")
    );
    assert_eq!(
        loop_.nuri.world().get_resource_json("copied_key").as_deref(),
        Some("w")
    );
}
