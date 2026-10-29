#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SocialBusMessage {
    pub agent_id: u64,
    pub payload: String,
}

#[derive(Default, Clone, Debug)]
pub struct SocialBus {
    pub queue: Vec<SocialBusMessage>,
}

impl SocialBus {
    pub fn push(&mut self, agent_id: u64, payload: String) {
        self.queue.push(SocialBusMessage { agent_id, payload });
    }

    pub fn drain(&mut self) -> Vec<SocialBusMessage> {
        let mut out = Vec::new();
        std::mem::swap(&mut out, &mut self.queue);
        out
    }
}
