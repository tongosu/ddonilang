#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LatencyMode {
    Fixed,
    Jitter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LatencyPolicy {
    pub l_madi: u64,
    pub mode: LatencyMode,
    pub seed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LatencyEvent {
    pub madi: u64,
    pub deliver_madi: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayHeader {
    pub seulgi_latency_madi: u64,
    pub seulgi_latency_drop_policy: String,
    pub created_at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LatencySchedule {
    pub seulgi_latency_madi: u64,
    pub accept_madi: u64,
    pub target_madi: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduledPacket {
    pub accept_madi: u64,
    pub target_madi: u64,
    pub late: bool,
    pub dropped: bool,
}

pub const REPLAY_HEADER_DETERMINISTIC_CREATED_AT: &str = "1970-01-01T00:00:00Z";
pub const LATENCY_DROP_POLICY_LATE_DROP: &str = "late_drop";

impl LatencySchedule {
    pub fn new(seulgi_latency_madi: u64, accept_madi: u64) -> Self {
        Self {
            seulgi_latency_madi,
            accept_madi,
            target_madi: accept_madi.saturating_add(seulgi_latency_madi),
        }
    }
}

impl ScheduledPacket {
    pub fn from_schedule(schedule: LatencySchedule, current_madi: u64) -> Self {
        let late = current_madi > schedule.target_madi;
        Self {
            accept_madi: schedule.accept_madi,
            target_madi: schedule.target_madi,
            late,
            dropped: late,
        }
    }
}

pub fn build_replay_header(seulgi_latency_madi: u64) -> ReplayHeader {
    ReplayHeader {
        seulgi_latency_madi,
        seulgi_latency_drop_policy: LATENCY_DROP_POLICY_LATE_DROP.to_string(),
        created_at: REPLAY_HEADER_DETERMINISTIC_CREATED_AT.to_string(),
    }
}

pub fn simulate(policy: LatencyPolicy, count: u64) -> Vec<LatencyEvent> {
    let mut events = Vec::with_capacity(count as usize);
    let mut rng = policy.seed;
    for madi in 0..count {
        let delay = match policy.mode {
            LatencyMode::Fixed => policy.l_madi,
            LatencyMode::Jitter => {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                let jitter = if policy.l_madi == 0 {
                    0
                } else {
                    rng % (policy.l_madi + 1)
                };
                policy.l_madi.saturating_add(jitter)
            }
        };
        let schedule = LatencySchedule::new(delay, madi);
        events.push(LatencyEvent {
            madi: schedule.accept_madi,
            deliver_madi: schedule.target_madi,
        });
    }
    events
}

#[cfg(test)]
mod tests {
    use super::{
        build_replay_header, simulate, LatencyMode, LatencyPolicy, LatencySchedule,
        ScheduledPacket, LATENCY_DROP_POLICY_LATE_DROP, REPLAY_HEADER_DETERMINISTIC_CREATED_AT,
    };

    #[test]
    fn simulate_fixed_mode_schedule_is_linear() {
        let policy = LatencyPolicy {
            l_madi: 3,
            mode: LatencyMode::Fixed,
            seed: 0,
        };
        let events = simulate(policy, 3);
        let schedule: Vec<(u64, u64)> = events
            .iter()
            .map(|event| (event.madi, event.deliver_madi))
            .collect();
        assert_eq!(schedule, vec![(0, 3), (1, 4), (2, 5)]);
    }

    #[test]
    fn simulate_jitter_mode_is_deterministic_for_seed() {
        let policy = LatencyPolicy {
            l_madi: 3,
            mode: LatencyMode::Jitter,
            seed: 7,
        };
        let events = simulate(policy, 4);
        let schedule: Vec<(u64, u64)> = events
            .iter()
            .map(|event| (event.madi, event.deliver_madi))
            .collect();
        assert_eq!(schedule, vec![(0, 3), (1, 5), (2, 7), (3, 9)]);
    }

    #[test]
    fn simulate_saturates_delivery_madi_on_overflow() {
        let policy = LatencyPolicy {
            l_madi: u64::MAX,
            mode: LatencyMode::Fixed,
            seed: 0,
        };
        let events = simulate(policy, 2);
        let schedule: Vec<(u64, u64)> = events
            .iter()
            .map(|event| (event.madi, event.deliver_madi))
            .collect();
        assert_eq!(schedule, vec![(0, u64::MAX), (1, u64::MAX)]);
    }

    #[test]
    fn schedule_marks_late_packet_by_current_madi() {
        let schedule = LatencySchedule::new(3, 2);
        let on_time = ScheduledPacket::from_schedule(schedule, 5);
        let late = ScheduledPacket::from_schedule(schedule, 6);
        assert!(!on_time.late);
        assert!(!on_time.dropped);
        assert!(late.late);
        assert!(late.dropped);
    }

    #[test]
    fn replay_header_is_deterministic_for_goldens() {
        let header = build_replay_header(3);
        assert_eq!(header.seulgi_latency_madi, 3);
        assert_eq!(
            header.seulgi_latency_drop_policy,
            LATENCY_DROP_POLICY_LATE_DROP.to_string()
        );
        assert_eq!(
            header.created_at,
            REPLAY_HEADER_DETERMINISTIC_CREATED_AT.to_string()
        );
    }
}
