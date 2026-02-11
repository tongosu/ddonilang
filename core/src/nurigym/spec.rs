#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationSpec {
    pub slot_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionSpec {
    pub actions: Vec<String>,
}

impl ObservationSpec {
    pub fn default_k64() -> Self {
        Self { slot_count: 64 }
    }

    pub fn to_detjson(&self) -> String {
        format!("{{\"schema\":\"nurigym.obs_spec.v1\",\"slot_count\":{}}}", self.slot_count)
    }
}

impl ActionSpec {
    pub fn empty() -> Self {
        Self { actions: Vec::new() }
    }

    pub fn to_detjson(&self) -> String {
        let mut out = String::new();
        out.push_str("{\"schema\":\"nurigym.action_spec.v1\",\"actions\":[");
        for (idx, item) in self.actions.iter().enumerate() {
            if idx > 0 {
                out.push(',');
            }
            out.push('"');
            out.push_str(&escape_json(item));
            out.push('"');
        }
        out.push_str("]}");
        out
    }
}

fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}