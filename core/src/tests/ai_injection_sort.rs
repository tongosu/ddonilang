use crate::fixed64::Fixed64;
use crate::platform::{DetSam, Sam, SeulgiIntent};

#[test]
fn ai_injections_sorted_and_delayed_by_tick() {
    let mut sam = DetSam::new(Fixed64::from_i64(1));
    let intent = SeulgiIntent::Say {
        text: "hi".to_string(),
    };

    sam.push_async_ai(2, 2, 0, 0, intent.clone());
    sam.push_async_ai(1, 9, 0, 0, intent.clone());
    sam.push_async_ai(2, 1, 0, 0, intent.clone());

    let snapshot = sam.begin_tick(1);
    assert_eq!(snapshot.ai_injections.len(), 3);
    assert_eq!(snapshot.ai_injections[0].agent_id, 1);
    assert_eq!(snapshot.ai_injections[1].agent_id, 2);
    assert_eq!(snapshot.ai_injections[1].recv_seq, 1);
    assert_eq!(snapshot.ai_injections[2].recv_seq, 2);

    sam.push_async_ai(9, 0, 0, 0, intent.clone());
    let snapshot2 = sam.begin_tick(2);
    assert_eq!(snapshot2.ai_injections.len(), 1);
    assert_eq!(snapshot2.ai_injections[0].agent_id, 9);
}
