use crate::signals::{Signal, SignalSink, TickId, VecSignalSink};

pub const ALRIM_MAX_PASSES: u8 = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlrimLogEntry {
    pub tick_id: TickId,
    pub pass_index: u8,
    pub name: &'static str,
    pub carried: bool,
}

pub trait AlrimHandler {
    fn on_signal(&mut self, signal: &Signal, out: &mut dyn SignalSink);
}

pub trait AlrimLogger {
    fn on_event(&mut self, entry: AlrimLogEntry);
}

#[derive(Default)]
pub struct VecAlrimLogger {
    pub entries: Vec<AlrimLogEntry>,
}

impl AlrimLogger for VecAlrimLogger {
    fn on_event(&mut self, entry: AlrimLogEntry) {
        self.entries.push(entry);
    }
}

#[derive(Default)]
pub struct AlrimLoop {
    carryover: Vec<Signal>,
}

impl AlrimLoop {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn carryover_len(&self) -> usize {
        self.carryover.len()
    }

    pub fn take_carryover(&mut self) -> Vec<Signal> {
        core::mem::take(&mut self.carryover)
    }

    pub fn run_tick(
        &mut self,
        tick_id: TickId,
        mut initial: Vec<Signal>,
        handler: &mut dyn AlrimHandler,
        logger: &mut dyn AlrimLogger,
    ) {
        let mut queue = Vec::new();
        if !self.carryover.is_empty() {
            queue.extend(self.carryover.drain(..));
        }
        if !initial.is_empty() {
            queue.append(&mut initial);
        }

        let mut pass_index: u8 = 0;
        while !queue.is_empty() && pass_index < ALRIM_MAX_PASSES {
            let mut sink = VecSignalSink::default();
            for signal in &queue {
                logger.on_event(AlrimLogEntry {
                    tick_id,
                    pass_index,
                    name: signal.name(),
                    carried: false,
                });
                handler.on_signal(signal, &mut sink);
            }
            queue = sink.signals;
            pass_index += 1;
        }

        if !queue.is_empty() {
            for signal in &queue {
                logger.on_event(AlrimLogEntry {
                    tick_id,
                    pass_index,
                    name: signal.name(),
                    carried: true,
                });
            }
            self.carryover = queue;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoHandler;

    impl AlrimHandler for EchoHandler {
        fn on_signal(&mut self, signal: &Signal, out: &mut dyn SignalSink) {
            out.emit(signal.clone());
        }
    }

    #[test]
    fn alrim_pass_limit_carries_over() {
        let mut loop_ = AlrimLoop::new();
        let mut logger = VecAlrimLogger::default();
        let mut handler = EchoHandler;

        loop_.run_tick(1, vec![Signal::Alrim { name: "연쇄" }], &mut handler, &mut logger);

        let processed = logger.entries.iter().filter(|e| !e.carried).count();
        let carried = logger.entries.iter().filter(|e| e.carried).count();

        assert_eq!(processed, ALRIM_MAX_PASSES as usize);
        assert_eq!(carried, 1);
        assert_eq!(loop_.carryover_len(), 1);
    }
}
