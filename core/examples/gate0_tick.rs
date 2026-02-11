use ddonirang_core::{
    EngineLoop, Fixed64, Nuri,
    platform::{Bogae, DetNuri, DetSam, InMemoryGeoul, Iyagi, Origin, Patch, PatchOp},
    signals::{TickId, VecSignalSink},
};

struct DemoIyagi;

impl Iyagi for DemoIyagi {
    fn run_startup(&mut self, _world: &ddonirang_core::platform::NuriWorld) -> Patch {
        Patch::default()
    }

    fn run_update(
        &mut self,
        _world: &ddonirang_core::platform::NuriWorld,
        _input: &ddonirang_core::platform::InputSnapshot,
    ) -> Patch {
        Patch {
            ops: vec![PatchOp::SetResourceFixed64 {
                tag: "x".to_string(),
                value: Fixed64::from_i64(5),
            }],
            origin: Origin::system("example"),
        }
    }
}

struct NoopBogae;

impl Bogae for NoopBogae {
    fn render(&mut self, _world: &ddonirang_core::platform::NuriWorld, _tick_id: TickId) {}
}

fn main() {
    let sam = DetSam::new(Fixed64::from_i64(1));
    let iyagi = DemoIyagi;
    let nuri = DetNuri::new();
    let geoul = InMemoryGeoul::new();
    let bogae = NoopBogae;

    let mut loop_ = EngineLoop::new(sam, iyagi, nuri, geoul, bogae);
    let mut sink = VecSignalSink::default();

    let frame = loop_.tick_once(0, &mut sink);
    let x = loop_
        .nuri
        .world()
        .get_resource_fixed64("x")
        .unwrap_or(Fixed64::ZERO);

    println!("tick: {}", frame.snapshot.tick_id);
    println!("x: {}", x);
    println!("state_hash: {}", frame.state_hash.to_hex());
    println!("signals: {}", sink.signals.len());
}
