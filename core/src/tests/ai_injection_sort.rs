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

#[test]
fn ai_injections_execution_order_uses_agent_and_recv_seq_only() {
    let mut sam = DetSam::new(Fixed64::from_i64(1));

    sam.push_async_ai(
        7,
        2,
        100,
        105,
        SeulgiIntent::MoveTo {
            x: Fixed64::from_i64(10),
            y: Fixed64::from_i64(20),
        },
    );
    sam.push_async_ai(
        7,
        1,
        1,
        999,
        SeulgiIntent::Say {
            text: "payload-order-must-not-win".to_string(),
        },
    );

    let snapshot = sam.begin_tick(1);
    assert_eq!(snapshot.ai_injections.len(), 2);
    assert_eq!(
        snapshot
            .ai_injections
            .iter()
            .map(|packet| (packet.agent_id, packet.recv_seq))
            .collect::<Vec<(u64, u64)>>(),
        vec![(7, 1), (7, 2)]
    );
}
