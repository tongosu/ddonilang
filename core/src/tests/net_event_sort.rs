use crate::fixed64::Fixed64;
use crate::platform::{DetSam, Sam};

#[test]
fn net_events_sorted_by_sender_and_seq() {
    let mut sam = DetSam::new(Fixed64::from_i64(1));
    sam.push_net_event("peer-b", 2, "peer-b#2", "{\"kind\":\"k\"}");
    sam.push_net_event("peer-a", 2, "peer-a#2", "{\"kind\":\"k\"}");
    sam.push_net_event("peer-a", 1, "peer-a#1", "{\"kind\":\"k\"}");

    let snapshot = sam.begin_tick(0);
    assert_eq!(snapshot.net_events.len(), 3);
    assert_eq!(snapshot.net_events[0].sender, "peer-a");
    assert_eq!(snapshot.net_events[0].seq, 1);
    assert_eq!(snapshot.net_events[1].sender, "peer-a");
    assert_eq!(snapshot.net_events[1].seq, 2);
    assert_eq!(snapshot.net_events[2].sender, "peer-b");
    assert_eq!(snapshot.net_events[2].seq, 2);
}
