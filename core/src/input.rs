use crate::platform::{KEY_A, KEY_D, KEY_S, KEY_W};
use std::collections::BTreeMap;

/// In-memory read-only projection of one logical InputSnapshot.
/// This does not admit input, mint channel identity, encode a tape or execute
/// reactions. The ingress owner must supply resolved typed channels and the
/// snapshot's settled held state. No key/device name is resolved here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogicalInputQuery<C> {
    madi: u64,
    channels: BTreeMap<C, LogicalChannelKind>,
    events: Vec<LogicalInputEvent<C>>,
    held: BTreeMap<C, bool>,
    initial_held: Option<BTreeMap<C, bool>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicalChannelKind {
    Control,
    Text,
    Number,
    Coordinate,
}

/// Provisional Alpha4 value transaction phase. No host time or pixel authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueEventPhase { Set, Begin, Move, End }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogicalInputEventKind {
    Press,
    Repeat,
    Release,
    CommittedText(String),
    Paste(String),
    Number(i64, ValueEventPhase),
    Coordinate([i64; 2], ValueEventPhase),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogicalInputEvent<C> {
    pub receive_ordinal: u64,
    pub channel: C,
    pub kind: LogicalInputEventKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicalInputQueryError {
    NonIncreasingOrdinal,
    UnknownChannel,
    ChannelKindMismatch,
    MissingHeldState,
    InvalidEventPrefix,
    InconsistentHeldState,
}

impl<C: Ord> LogicalInputQuery<C> {
    /// Validate a projection, not a sealed execution/input contract. Ordinals
    /// are retained exactly; gaps are not filled and events are never sorted.
    /// The settled state is supplied, not guessed from a partial event list.
    pub fn from_projection(
        madi: u64,
        channels: BTreeMap<C, LogicalChannelKind>,
        events: Vec<LogicalInputEvent<C>>,
        held: BTreeMap<C, bool>,
    ) -> Result<Self, LogicalInputQueryError> {
        let mut previous = None;
        for event in &events {
            if previous.is_some_and(|ordinal| event.receive_ordinal <= ordinal) {
                return Err(LogicalInputQueryError::NonIncreasingOrdinal);
            }
            previous = Some(event.receive_ordinal);
            let expected = match &event.kind {
                LogicalInputEventKind::Press
                | LogicalInputEventKind::Repeat
                | LogicalInputEventKind::Release => LogicalChannelKind::Control,
                LogicalInputEventKind::CommittedText(_) | LogicalInputEventKind::Paste(_) => {
                    LogicalChannelKind::Text
                }
                LogicalInputEventKind::Number(_, _) => LogicalChannelKind::Number,
                LogicalInputEventKind::Coordinate(_, _) => LogicalChannelKind::Coordinate,
            };
            match channels.get(&event.channel) {
                None => return Err(LogicalInputQueryError::UnknownChannel),
                Some(actual) if *actual != expected => {
                    return Err(LogicalInputQueryError::ChannelKindMismatch);
                }
                _ => {}
            }
        }
        for channel in held.keys() {
            match channels.get(channel) {
                None => return Err(LogicalInputQueryError::UnknownChannel),
                Some(kind) if *kind != LogicalChannelKind::Control => {
                    return Err(LogicalInputQueryError::ChannelKindMismatch);
                }
                _ => {}
            }
        }
        if channels.iter().any(|(channel, kind)| {
            *kind == LogicalChannelKind::Control && !held.contains_key(channel)
        }) {
            return Err(LogicalInputQueryError::MissingHeldState);
        }
        Ok(Self {
            madi,
            channels,
            events,
            held,
            initial_held: None,
        })
    }

    pub fn madi(&self) -> u64 {
        self.madi
    }

    pub fn is_held(&self, channel: &C) -> Result<bool, LogicalInputQueryError> {
        match self.channels.get(channel) {
            None => Err(LogicalInputQueryError::UnknownChannel),
            Some(kind) if *kind != LogicalChannelKind::Control => Err(LogicalInputQueryError::ChannelKindMismatch),
            Some(LogicalChannelKind::Control) => self
                .held
                .get(channel)
                .copied()
                .ok_or(LogicalInputQueryError::MissingHeldState),
            _ => Err(LogicalInputQueryError::ChannelKindMismatch),
        }
    }

    pub fn events(&self) -> &[LogicalInputEvent<C>] {
        &self.events
    }

    pub fn events_for<'a>(
        &'a self,
        channel: &'a C,
    ) -> Result<impl Iterator<Item = &'a LogicalInputEvent<C>>, LogicalInputQueryError> {
        if !self.channels.contains_key(channel) {
            return Err(LogicalInputQueryError::UnknownChannel);
        }
        Ok(self
            .events
            .iter()
            .filter(move |event| &event.channel == channel))
    }
}

impl<C: Ord + Clone> LogicalInputQuery<C> {
    /// A3-P4: the ingress supplies the previous committed state. Never infer it
    /// backwards from Press/Release (repeated Press need not invert a state).
    pub fn with_initial_held(mut self, initial: BTreeMap<C, bool>) -> Result<Self, LogicalInputQueryError> {
        if initial.keys().ne(self.held.keys()) {
            return Err(LogicalInputQueryError::MissingHeldState);
        }
        let mut settled = initial.clone();
        for event in &self.events {
            let state = settled.get_mut(&event.channel);
            match &event.kind {
                LogicalInputEventKind::Press => *state.ok_or(LogicalInputQueryError::MissingHeldState)? = true,
                LogicalInputEventKind::Release => *state.ok_or(LogicalInputQueryError::MissingHeldState)? = false,
                LogicalInputEventKind::Repeat if state.copied() != Some(true) => return Err(LogicalInputQueryError::InconsistentHeldState),
                _ => {}
            }
        }
        if settled != self.held { return Err(LogicalInputQueryError::InconsistentHeldState); }
        self.initial_held = Some(initial);
        Ok(self)
    }

    /// Keep the non-consuming complete event list; only held-state observation
    /// advances with the event being dispatched. Final madi sees final held.
    pub fn at_event_prefix(&self, count: usize) -> Result<Self, LogicalInputQueryError> {
        if count > self.events.len() { return Err(LogicalInputQueryError::InvalidEventPrefix); }
        let mut result = self.clone();
        result.held = self.initial_held.clone().ok_or(LogicalInputQueryError::MissingHeldState)?;
        for event in self.events.iter().take(count) {
            match event.kind {
                LogicalInputEventKind::Press => { result.held.insert(event.channel.clone(), true); }
                LogicalInputEventKind::Release => { result.held.insert(event.channel.clone(), false); }
                _ => {}
            }
        }
        Ok(result)
    }
}

pub fn key_bit_from_name(name: &str) -> Option<u64> {
    match name.to_ascii_lowercase().as_str() {
        "w" | "i" | "up" | "arrowup" => Some(KEY_W),
        "a" | "j" | "left" | "arrowleft" => Some(KEY_A),
        "s" | "k" | "down" | "arrowdown" => Some(KEY_S),
        "d" | "l" | "right" | "arrowright" => Some(KEY_D),
        _ => None,
    }
}

pub fn is_key_pressed(keys_pressed: u64, name: &str) -> bool {
    key_bit_from_name(name)
        .map(|bit| keys_pressed & bit != 0)
        .unwrap_or(false)
}

pub fn is_key_just_pressed(prev_keys: u64, keys_pressed: u64, name: &str) -> bool {
    key_bit_from_name(name)
        .map(|bit| (keys_pressed & bit != 0) && (prev_keys & bit == 0))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_bit_mapping_supports_wasd_and_arrows() {
        assert_eq!(key_bit_from_name("W"), Some(KEY_W));
        assert_eq!(key_bit_from_name("arrowdown"), Some(KEY_S));
        assert_eq!(key_bit_from_name("left"), Some(KEY_A));
        assert_eq!(key_bit_from_name("right"), Some(KEY_D));
        assert_eq!(key_bit_from_name("i"), Some(KEY_W));
        assert_eq!(key_bit_from_name("j"), Some(KEY_A));
        assert_eq!(key_bit_from_name("k"), Some(KEY_S));
        assert_eq!(key_bit_from_name("l"), Some(KEY_D));
        assert_eq!(key_bit_from_name("unknown"), None);
    }

    #[test]
    fn key_pressed_checks_bitmask() {
        let keys = KEY_W | KEY_D;
        assert!(is_key_pressed(keys, "w"));
        assert!(is_key_pressed(keys, "right"));
        assert!(!is_key_pressed(keys, "s"));
    }

    #[test]
    fn key_just_pressed_checks_edge() {
        let prev = KEY_W;
        let now = KEY_W | KEY_D;
        assert!(!is_key_just_pressed(prev, now, "w"));
        assert!(is_key_just_pressed(prev, now, "d"));
    }

    // Typed resolved slots in tests, not a new canonical channel ID encoding.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Channel(u8);

    fn control_event(ordinal: u64, kind: LogicalInputEventKind) -> LogicalInputEvent<Channel> {
        LogicalInputEvent {
            receive_ordinal: ordinal,
            channel: Channel(0),
            kind,
        }
    }

    fn controls() -> BTreeMap<Channel, LogicalChannelKind> {
        BTreeMap::from([(Channel(0), LogicalChannelKind::Control)])
    }

    #[test]
    fn logical_query_preserves_events_separately_from_settled_held() {
        let events = vec![
            control_event(7, LogicalInputEventKind::Press),
            control_event(8, LogicalInputEventKind::Repeat),
            control_event(10, LogicalInputEventKind::Release),
        ];
        let query = LogicalInputQuery::from_projection(
            3,
            controls(),
            events.clone(),
            BTreeMap::from([(Channel(0), false)]),
        )
        .unwrap();
        assert_eq!(query.madi(), 3);
        assert!(!query.is_held(&Channel(0)).unwrap());
        assert_eq!(query.events(), events);
        assert_eq!(query.events_for(&Channel(0)).unwrap().count(), 3);
        assert_eq!(query.events(), events); // Query does not consume input.
        assert_eq!(query.events_for(&Channel(0)).unwrap().count(), 3);
        assert!(matches!(
            query.events()[1].kind,
            LogicalInputEventKind::Repeat
        ));
    }

    #[test]
    fn a3_p4_event_prefix_uses_explicit_initial_state_and_preserves_events() {
        let query = LogicalInputQuery::from_projection(3, controls(), vec![
            control_event(0, LogicalInputEventKind::Press),
            control_event(1, LogicalInputEventKind::Press),
            control_event(2, LogicalInputEventKind::Release),
        ], BTreeMap::from([(Channel(0), false)])).unwrap();
        assert_eq!(query.at_event_prefix(0).unwrap_err(), LogicalInputQueryError::MissingHeldState);
        let query = query.with_initial_held(BTreeMap::from([(Channel(0), true)])).unwrap();
        for (count, held) in [(0,true),(1,true),(2,true),(3,false)] {
            let prefix = query.at_event_prefix(count).unwrap();
            assert_eq!(prefix.is_held(&Channel(0)).unwrap(), held);
            assert_eq!(prefix.events(), query.events());
        }
        assert_eq!(query.at_event_prefix(4).unwrap_err(), LogicalInputQueryError::InvalidEventPrefix);
    }

    #[test]
    fn logical_query_does_not_repair_duplicate_or_reordered_events() {
        for second in [6, 7] {
            let result = LogicalInputQuery::from_projection(
                3,
                controls(),
                vec![
                    control_event(7, LogicalInputEventKind::Press),
                    control_event(second, LogicalInputEventKind::Release),
                ],
                BTreeMap::from([(Channel(0), false)]),
            );
            assert_eq!(
                result.unwrap_err(),
                LogicalInputQueryError::NonIncreasingOrdinal
            );
        }
    }

    #[test]
    fn logical_query_requires_known_typed_channel_and_explicit_settled_state() {
        assert_eq!(
            LogicalInputQuery::from_projection(0, controls(), vec![], BTreeMap::new()).unwrap_err(),
            LogicalInputQueryError::MissingHeldState
        );
        assert_eq!(
            LogicalInputQuery::from_projection(
                0,
                controls(),
                vec![LogicalInputEvent {
                    receive_ordinal: 0,
                    channel: Channel(1),
                    kind: LogicalInputEventKind::Press
                }],
                BTreeMap::from([(Channel(0), false)])
            )
            .unwrap_err(),
            LogicalInputQueryError::UnknownChannel
        );
        let query = LogicalInputQuery::from_projection(
            0,
            controls(),
            vec![],
            BTreeMap::from([(Channel(0), true)]),
        )
        .unwrap();
        assert!(query.is_held(&Channel(0)).unwrap()); // No synthetic Press.
        assert!(query.events().is_empty());
        assert_eq!(
            query.is_held(&Channel(9)),
            Err(LogicalInputQueryError::UnknownChannel)
        );
        assert_eq!(
            query.events_for(&Channel(9)).err(),
            Some(LogicalInputQueryError::UnknownChannel)
        );
    }

    #[test]
    fn logical_query_preserves_committed_text_and_paste_exactly() {
        let channels = BTreeMap::from([(Channel(1), LogicalChannelKind::Text)]);
        let events = vec![
            LogicalInputEvent {
                receive_ordinal: 1,
                channel: Channel(1),
                kind: LogicalInputEventKind::CommittedText("e\u{301}".into()),
            },
            LogicalInputEvent {
                receive_ordinal: 2,
                channel: Channel(1),
                kind: LogicalInputEventKind::Paste("한글\n".into()),
            },
        ];
        let query =
            LogicalInputQuery::from_projection(2, channels, events.clone(), BTreeMap::new())
                .unwrap();
        assert_eq!(query.events(), events); // No implicit Unicode normalization.
        assert_eq!(
            query.is_held(&Channel(1)),
            Err(LogicalInputQueryError::ChannelKindMismatch)
        );
    }

    #[test]
    fn logical_query_rejects_control_text_kind_confusion() {
        assert_eq!(
            LogicalInputQuery::from_projection(
                0,
                controls(),
                vec![control_event(
                    0,
                    LogicalInputEventKind::CommittedText("x".into())
                )],
                BTreeMap::from([(Channel(0), false)])
            )
            .unwrap_err(),
            LogicalInputQueryError::ChannelKindMismatch
        );
        assert_eq!(
            LogicalInputQuery::from_projection(
                0,
                BTreeMap::from([(Channel(0), LogicalChannelKind::Text)]),
                vec![control_event(0, LogicalInputEventKind::Press)],
                BTreeMap::new()
            )
            .unwrap_err(),
            LogicalInputQueryError::ChannelKindMismatch
        );
    }
}
