#[derive(Clone, Debug)]
pub enum TraceEvent {
    Log(String),
}

#[derive(Clone, Debug, Default)]
pub struct Trace {
    pub events: Vec<TraceEvent>,
}

impl Trace {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log(&mut self, text: String) {
        self.events.push(TraceEvent::Log(text));
    }

    pub fn log_lines(&self) -> Vec<&str> {
        self.events
            .iter()
            .filter_map(|event| match event {
                TraceEvent::Log(text) => Some(text.as_str()),
            })
            .collect()
    }
}
