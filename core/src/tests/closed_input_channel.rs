use crate::{EngineLoop, Fixed64};
use crate::platform::{
    Bogae, DetNuri, DetSam, Geoul, InMemoryGeoul, InputSnapshot, Iyagi, Origin, Patch, PatchOp,
    Sam, SeulgiIntent,
};
use crate::signals::{TickId, VecSignalSink};

struct InputEchoIyagi;

impl Iyagi for InputEchoIyagi {
    fn run_startup(&mut self, _world: &crate::platform::NuriWorld) -> Patch {
        Patch::default()
    }

    fn run_update(
        &mut self,
        _world: &crate::platform::NuriWorld,
        input: &InputSnapshot,
    ) -> Patch {
        let mut ops = Vec::new();
        ops.push(PatchOp::SetResourceJson {
            tag: "last_key".to_string(),
            json: input.last_key_name.clone(),
        });
        ops.push(PatchOp::SetResourceJson {
            tag: "pointer".to_string(),
            json: format!("{},{}", input.pointer_x_i32, input.pointer_y_i32),
        });
        Patch {
            ops,
            origin: Origin::system("test"),
        }
    }
}

struct NoopBogae;

impl Bogae for NoopBogae {
    fn render(&mut self, _world: &crate::platform::NuriWorld, _tick_id: TickId) {}
}

#[test]
fn closed_input_is_snapshotted_and_replayable() {
    let mut sam = DetSam::new(Fixed64::from_i64(1));
    sam.last_key_name = "w".to_string();
    sam.pointer_x_i32 = 12;
    sam.pointer_y_i32 = -4;
    sam.push_async_ai(
        1,
        0,
        0,
        0,
        SeulgiIntent::Say {
            text: "echo".to_string(),
        },
    );

    let iyagi = InputEchoIyagi;
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;
    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let frame = loop_.tick_once(1, &mut sink);
    assert_eq!(frame.snapshot.last_key_name, "w");
    assert_eq!(frame.snapshot.pointer_x_i32, 12);
    assert_eq!(frame.snapshot.pointer_y_i32, -4);
    assert_eq!(frame.snapshot.ai_injections.len(), 1);

    let replayed = loop_.geoul.replay_next().expect("replay");
    assert_eq!(replayed.snapshot, frame.snapshot);
}
