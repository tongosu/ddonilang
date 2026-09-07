use std::collections::BTreeMap;

pub const LEGACY_RUNTIME_SSOT_VERSION: &str = "20.6.6";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateKey(pub String);

impl StateKey {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct DeterministicState<K, V> {
    pub resources: BTreeMap<K, V>,
}

impl<K: Ord, V> DeterministicState<K, V> {
    pub fn new() -> Self {
        Self {
            resources: BTreeMap::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.resources.get(key)
    }

    pub fn set(&mut self, key: K, value: V) {
        self.resources.insert(key, value);
    }

    pub fn len(&self) -> usize {
        self.resources.len()
    }

    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputTraceEvent {
    Log(String),
}

#[derive(Clone, Debug, Default)]
pub struct OutputTrace {
    pub events: Vec<OutputTraceEvent>,
}

impl OutputTrace {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log(&mut self, text: String) {
        self.events.push(OutputTraceEvent::Log(text));
    }

    pub fn log_lines(&self) -> Vec<&str> {
        self.events
            .iter()
            .map(|event| match event {
                OutputTraceEvent::Log(text) => text.as_str(),
            })
            .collect()
    }
}

pub struct TraceWireMeta<'a> {
    pub ssot_version: &'a str,
    pub seed: u64,
    pub madi: u64,
}

pub fn blake3_wire_identity(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

pub fn escape_json_string_contents(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}

pub fn encode_state_canon_entries<I, K, V>(entries: I) -> Vec<u8>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut out = Vec::new();
    out.extend_from_slice(b"DDN_STATE_V1\n");
    for (key, value) in entries {
        out.extend_from_slice(key.as_ref().as_bytes());
        out.push(b'\t');
        out.extend_from_slice(value.as_ref().as_bytes());
        out.push(b'\n');
    }
    out
}

pub fn encode_trace_output_bundle<I, S>(
    source: &str,
    output_lines: I,
    state_hash: &str,
    meta: &TraceWireMeta<'_>,
) -> Vec<u8>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut out = Vec::new();
    out.extend_from_slice(b"DDN_TRACE_V1\n");
    write_kv(&mut out, "ssot", meta.ssot_version);
    write_kv(&mut out, "seed", &format!("0x{:016x}", meta.seed));
    write_kv(&mut out, "madi", &meta.madi.to_string());
    write_kv(&mut out, "state_hash", state_hash);
    write_kv(&mut out, "source", &escape_field(source));
    for line in output_lines {
        write_kv(&mut out, "out", &escape_field(line.as_ref()));
    }
    out
}

fn write_kv(out: &mut Vec<u8>, key: &str, value: &str) {
    out.extend_from_slice(key.as_bytes());
    out.push(b'\t');
    out.extend_from_slice(value.as_bytes());
    out.push(b'\n');
}

fn escape_field(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        blake3_wire_identity, encode_state_canon_entries, encode_trace_output_bundle,
        escape_json_string_contents, DeterministicState, OutputTrace, StateKey, TraceWireMeta,
        LEGACY_RUNTIME_SSOT_VERSION,
    };

    #[test]
    fn state_and_trace_wire_preserve_exact_v1_bytes() {
        let state = encode_state_canon_entries([("가", "1"), ("나", "\"둘\"")]);
        assert_eq!(state, "DDN_STATE_V1\n가\t1\n나\t\"둘\"\n".as_bytes());
        assert_eq!(LEGACY_RUNTIME_SSOT_VERSION, "20.6.6");
        assert_eq!(
            blake3_wire_identity(&state),
            format!("blake3:{}", blake3::hash(&state).to_hex())
        );

        let trace = encode_trace_output_bundle(
            "첫째\n둘째",
            ["탭\t역슬래시\\"],
            "blake3:abc",
            &TraceWireMeta {
                ssot_version: "25.47.0",
                seed: 7,
                madi: 3,
            },
        );
        assert_eq!(
            trace,
            concat!(
                "DDN_TRACE_V1\n",
                "ssot\t25.47.0\n",
                "seed\t0x0000000000000007\n",
                "madi\t3\n",
                "state_hash\tblake3:abc\n",
                "source\t첫째\\n둘째\n",
                "out\t탭\\t역슬래시\\\\\n"
            )
            .as_bytes()
        );
    }

    #[test]
    fn shared_state_and_trace_models_preserve_deterministic_order_and_lines() {
        let mut state = DeterministicState::new();
        state.set(StateKey::new("나"), "2".to_string());
        state.set(StateKey::new("가"), "1".to_string());
        assert_eq!(state.len(), 2);
        assert_eq!(
            state.get(&StateKey::new("가")).map(String::as_str),
            Some("1")
        );
        assert_eq!(
            state
                .resources
                .keys()
                .map(StateKey::as_str)
                .collect::<Vec<_>>(),
            vec!["가", "나"]
        );

        let mut trace = OutputTrace::new();
        trace.log("첫째".to_string());
        trace.log("둘째".to_string());
        assert_eq!(trace.log_lines(), vec!["첫째", "둘째"]);
    }

    #[test]
    fn shared_json_string_escape_preserves_cli_artifact_bytes() {
        assert_eq!(
            escape_json_string_contents("따옴표\" 역슬래시\\ 줄\n탭\t귀환\r"),
            "따옴표\\\" 역슬래시\\\\ 줄\\n탭\\t귀환\\r"
        );
    }
}
