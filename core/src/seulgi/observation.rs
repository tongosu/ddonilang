use crate::fixed64::Fixed64;
use crate::seulgi::vision::VisionCone;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub agent_id: u64,
    pub madi: u64,
    pub timestamp_ms: u64,
    pub self_state: AgentState,
    pub visible_objects: Vec<Object>,
    pub visible_agents: Vec<Agent>,
    pub world_state: WorldState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentState {
    pub position: (Fixed64, Fixed64),
    pub velocity: (Fixed64, Fixed64),
    pub health: u32,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Object {
    pub id: u64,
    pub position: (Fixed64, Fixed64),
    pub object_type: String,
    pub color: String,
    pub size: Fixed64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agent {
    pub id: u64,
    pub position: (Fixed64, Fixed64),
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldState {
    pub gravity: Fixed64,
    pub time_of_day: u32,
    pub weather: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameAgent {
    pub id: u64,
    pub position: (Fixed64, Fixed64),
    pub velocity: (Fixed64, Fixed64),
    pub health: u32,
    pub status: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub current_madi: u64,
    pub elapsed_ms: u64,
    pub agents: Vec<GameAgent>,
    pub objects: Vec<Object>,
    pub world: WorldState,
}

impl GameState {
    pub fn get_agent(&self, agent_id: u64) -> Option<&GameAgent> {
        self.agents.iter().find(|agent| agent.id == agent_id)
    }
}

pub trait ObservationSpec {
    fn capture(&self, state: &GameState, agent_id: u64) -> Observation;

    fn to_detjson(&self, obs: &Observation) -> Result<String, String> {
        Ok(observation_detjson(obs))
    }

    fn from_detjson(&self, json: &str) -> Result<Observation, String> {
        observation_from_detjson(json)
    }

    fn hash(&self, obs: &Observation) -> String {
        let json = self.to_detjson(obs).unwrap_or_default();
        format!("blake3:{}", blake3::hash(json.as_bytes()).to_hex())
    }

    fn equals(&self, obs1: &Observation, obs2: &Observation) -> bool {
        self.hash(obs1) == self.hash(obs2)
    }
}

pub struct DefaultObserver {
    vision: VisionCone,
}

impl DefaultObserver {
    pub fn new(max_distance: Fixed64, fov_degrees: u16) -> Self {
        Self {
            vision: VisionCone {
                max_distance,
                fov_degrees,
            },
        }
    }
}

impl ObservationSpec for DefaultObserver {
    fn capture(&self, state: &GameState, agent_id: u64) -> Observation {
        let agent = state
            .get_agent(agent_id)
            .expect("E_OBS_AGENT_NOT_FOUND");

        let self_state = AgentState {
            position: agent.position,
            velocity: agent.velocity,
            health: agent.health,
            status: agent.status.clone(),
        };

        let mut visible_objects = self
            .vision
            .get_visible_objects(agent.position, &state.objects);
        visible_objects.sort_by_key(|obj| obj.id);

        let mut visible_agents: Vec<Agent> = state
            .agents
            .iter()
            .filter(|other| other.id != agent_id)
            .filter(|other| self.vision.can_see(agent.position, other.position))
            .map(|other| Agent {
                id: other.id,
                position: other.position,
                name: other.name.clone(),
            })
            .collect();
        visible_agents.sort_by_key(|agent| agent.id);

        Observation {
            agent_id,
            madi: state.current_madi,
            timestamp_ms: state.elapsed_ms,
            self_state,
            visible_objects,
            visible_agents,
            world_state: state.world.clone(),
        }
    }
}

pub fn observation_detjson(obs: &Observation) -> String {
    let mut out = String::new();
    out.push('{');
    push_kv_str(&mut out, "schema", "seulgi.observation.v1", true);
    push_kv_num(&mut out, "agent_id", obs.agent_id, false);
    push_kv_num(&mut out, "madi", obs.madi, false);
    push_kv_num(&mut out, "timestamp_ms", obs.timestamp_ms, false);
    out.push_str(",\"self_state\":");
    out.push_str(&agent_state_detjson(&obs.self_state));

    out.push_str(",\"visible_objects\":[");
    let mut objects = obs.visible_objects.clone();
    objects.sort_by_key(|item| item.id);
    for (idx, item) in objects.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&object_detjson(item));
    }
    out.push(']');

    out.push_str(",\"visible_agents\":[");
    let mut agents = obs.visible_agents.clone();
    agents.sort_by_key(|item| item.id);
    for (idx, item) in agents.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&agent_detjson(item));
    }
    out.push(']');

    out.push_str(",\"world_state\":");
    out.push_str(&world_state_detjson(&obs.world_state));
    out.push('}');
    out
}

pub fn observation_from_detjson(json: &str) -> Result<Observation, String> {
    let mut reader = DetJsonReader::new(json);
    let obs = parse_observation(&mut reader)?;
    reader.skip_ws();
    if reader.peek().is_some() {
        return Err("E_OBS_JSON_TRAILING".to_string());
    }
    Ok(obs)
}

fn agent_state_detjson(state: &AgentState) -> String {
    let mut out = String::new();
    out.push('{');
    push_kv_xy(&mut out, "position", state.position, true);
    push_kv_xy(&mut out, "velocity", state.velocity, false);
    push_kv_num(&mut out, "health", state.health as u64, false);
    push_kv_str(&mut out, "status", &state.status, false);
    out.push('}');
    out
}

fn object_detjson(object: &Object) -> String {
    let mut out = String::new();
    out.push('{');
    push_kv_num(&mut out, "id", object.id, true);
    push_kv_xy(&mut out, "position", object.position, false);
    push_kv_str(&mut out, "object_type", &object.object_type, false);
    push_kv_str(&mut out, "color", &object.color, false);
    push_kv_fixed(&mut out, "size", object.size, false);
    out.push('}');
    out
}

fn agent_detjson(agent: &Agent) -> String {
    let mut out = String::new();
    out.push('{');
    push_kv_num(&mut out, "id", agent.id, true);
    push_kv_xy(&mut out, "position", agent.position, false);
    push_kv_str(&mut out, "name", &agent.name, false);
    out.push('}');
    out
}

fn world_state_detjson(world: &WorldState) -> String {
    let mut out = String::new();
    out.push('{');
    push_kv_fixed(&mut out, "gravity", world.gravity, true);
    push_kv_num(&mut out, "time_of_day", world.time_of_day as u64, false);
    push_kv_str(&mut out, "weather", &world.weather, false);
    out.push('}');
    out
}

fn push_kv_str(out: &mut String, key: &str, value: &str, first: bool) {
    if !first {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":\"");
    out.push_str(&escape_json(value));
    out.push('"');
}

fn push_kv_num(out: &mut String, key: &str, value: u64, first: bool) {
    if !first {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":");
    out.push_str(&value.to_string());
}

fn push_kv_fixed(out: &mut String, key: &str, value: Fixed64, first: bool) {
    if !first {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":");
    out.push_str(&fixed64_to_string(value));
}

fn push_kv_xy(out: &mut String, key: &str, value: (Fixed64, Fixed64), first: bool) {
    if !first {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":");
    out.push('{');
    push_kv_fixed(out, "x", value.0, true);
    push_kv_fixed(out, "y", value.1, false);
    out.push('}');
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

fn fixed64_to_string(value: Fixed64) -> String {
    let raw = value.raw_i64();
    if raw == 0 {
        return "0".to_string();
    }
    let negative = raw < 0;
    let abs_raw = if raw == i64::MIN {
        (i64::MAX as i128) + 1
    } else {
        raw.abs() as i128
    };
    let int_part = abs_raw >> 32;
    let frac_part = abs_raw & 0xFFFF_FFFF;
    if frac_part == 0 {
        if negative {
            format!("-{}", int_part)
        } else {
            int_part.to_string()
        }
    } else {
        let frac_decimal = ((frac_part * 1_000_000) >> 32) as u32;
        let mut out = if negative {
            format!("-{}.{:06}", int_part, frac_decimal)
        } else {
            format!("{}.{:06}", int_part, frac_decimal)
        };
        while out.ends_with('0') {
            out.pop();
        }
        if out.ends_with('.') {
            out.pop();
        }
        out
    }
}

struct DetJsonReader<'a> {
    input: &'a str,
    idx: usize,
}

impl<'a> DetJsonReader<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, idx: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.idx..].chars().next()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.idx += ch.len_utf8();
        Some(ch)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ' | '\n' | '\r' | '\t')) {
            self.next();
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        self.skip_ws();
        match self.next() {
            Some(ch) if ch == expected => Ok(()),
            _ => Err(format!("E_OBS_JSON_EXPECT {}", expected)),
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.skip_ws();
        if self.next() != Some('"') {
            return Err("E_OBS_JSON_STRING".to_string());
        }
        let mut out = String::new();
        loop {
            let ch = self.next().ok_or_else(|| "E_OBS_JSON_STRING_END".to_string())?;
            match ch {
                '"' => return Ok(out),
                '\\' => {
                    let esc = self.next().ok_or_else(|| "E_OBS_JSON_ESCAPE".to_string())?;
                    match esc {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            let mut code = 0u32;
                            for _ in 0..4 {
                                let hex = self.next().ok_or_else(|| "E_OBS_JSON_ESCAPE_U".to_string())?;
                                code = (code << 4)
                                    + hex
                                        .to_digit(16)
                                        .ok_or_else(|| "E_OBS_JSON_ESCAPE_U".to_string())?;
                            }
                            if let Some(decoded) = char::from_u32(code) {
                                out.push(decoded);
                            } else {
                                return Err("E_OBS_JSON_ESCAPE_U".to_string());
                            }
                        }
                        _ => return Err("E_OBS_JSON_ESCAPE".to_string()),
                    }
                }
                other => out.push(other),
            }
        }
    }

    fn parse_number(&mut self) -> Result<String, String> {
        self.skip_ws();
        let start = self.idx;
        let mut saw = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '-' || ch == '+' || ch == '.' {
                saw = true;
                self.next();
            } else {
                break;
            }
        }
        if !saw {
            return Err("E_OBS_JSON_NUMBER".to_string());
        }
        Ok(self.input[start..self.idx].to_string())
    }

    fn parse_key(&mut self, expected: &str) -> Result<(), String> {
        let key = self.parse_string()?;
        if key != expected {
            return Err(format!("E_OBS_JSON_KEY {}", expected));
        }
        self.expect_char(':')?;
        Ok(())
    }
}

fn parse_observation(reader: &mut DetJsonReader) -> Result<Observation, String> {
    reader.expect_char('{')?;
    reader.parse_key("schema")?;
    let schema = reader.parse_string()?;
    if schema != "seulgi.observation.v1" && schema != "observation.v1" {
        return Err("E_OBS_JSON_SCHEMA".to_string());
    }
    reader.expect_char(',')?;
    reader.parse_key("agent_id")?;
    let agent_id = parse_u64(reader)?;
    reader.expect_char(',')?;
    reader.parse_key("madi")?;
    let madi = parse_u64(reader)?;
    reader.expect_char(',')?;
    reader.parse_key("timestamp_ms")?;
    let timestamp_ms = parse_u64(reader)?;
    reader.expect_char(',')?;
    reader.parse_key("self_state")?;
    let self_state = parse_agent_state(reader)?;
    reader.expect_char(',')?;
    reader.parse_key("visible_objects")?;
    let visible_objects = parse_array(reader, parse_object)?;
    reader.expect_char(',')?;
    reader.parse_key("visible_agents")?;
    let visible_agents = parse_array(reader, parse_agent)?;
    reader.expect_char(',')?;
    reader.parse_key("world_state")?;
    let world_state = parse_world_state(reader)?;
    reader.expect_char('}')?;

    Ok(Observation {
        agent_id,
        madi,
        timestamp_ms,
        self_state,
        visible_objects,
        visible_agents,
        world_state,
    })
}

fn parse_agent_state(reader: &mut DetJsonReader) -> Result<AgentState, String> {
    reader.expect_char('{')?;
    reader.parse_key("position")?;
    let position = parse_xy(reader)?;
    reader.expect_char(',')?;
    reader.parse_key("velocity")?;
    let velocity = parse_xy(reader)?;
    reader.expect_char(',')?;
    reader.parse_key("health")?;
    let health = parse_u64(reader)? as u32;
    reader.expect_char(',')?;
    reader.parse_key("status")?;
    let status = reader.parse_string()?;
    reader.expect_char('}')?;
    Ok(AgentState {
        position,
        velocity,
        health,
        status,
    })
}

fn parse_object(reader: &mut DetJsonReader) -> Result<Object, String> {
    reader.expect_char('{')?;
    reader.parse_key("id")?;
    let id = parse_u64(reader)?;
    reader.expect_char(',')?;
    reader.parse_key("position")?;
    let position = parse_xy(reader)?;
    reader.expect_char(',')?;
    reader.parse_key("object_type")?;
    let object_type = reader.parse_string()?;
    reader.expect_char(',')?;
    reader.parse_key("color")?;
    let color = reader.parse_string()?;
    reader.expect_char(',')?;
    reader.parse_key("size")?;
    let size = parse_fixed64(reader)?;
    reader.expect_char('}')?;
    Ok(Object {
        id,
        position,
        object_type,
        color,
        size,
    })
}

fn parse_agent(reader: &mut DetJsonReader) -> Result<Agent, String> {
    reader.expect_char('{')?;
    reader.parse_key("id")?;
    let id = parse_u64(reader)?;
    reader.expect_char(',')?;
    reader.parse_key("position")?;
    let position = parse_xy(reader)?;
    reader.expect_char(',')?;
    reader.parse_key("name")?;
    let name = reader.parse_string()?;
    reader.expect_char('}')?;
    Ok(Agent { id, position, name })
}

fn parse_world_state(reader: &mut DetJsonReader) -> Result<WorldState, String> {
    reader.expect_char('{')?;
    reader.parse_key("gravity")?;
    let gravity = parse_fixed64(reader)?;
    reader.expect_char(',')?;
    reader.parse_key("time_of_day")?;
    let time_of_day = parse_u64(reader)? as u32;
    reader.expect_char(',')?;
    reader.parse_key("weather")?;
    let weather = reader.parse_string()?;
    reader.expect_char('}')?;
    Ok(WorldState {
        gravity,
        time_of_day,
        weather,
    })
}

fn parse_xy(reader: &mut DetJsonReader) -> Result<(Fixed64, Fixed64), String> {
    reader.expect_char('{')?;
    reader.parse_key("x")?;
    let x = parse_fixed64(reader)?;
    reader.expect_char(',')?;
    reader.parse_key("y")?;
    let y = parse_fixed64(reader)?;
    reader.expect_char('}')?;
    Ok((x, y))
}

fn parse_array<T>(
    reader: &mut DetJsonReader,
    mut parse_item: impl FnMut(&mut DetJsonReader) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    reader.expect_char('[')?;
    reader.skip_ws();
    if reader.peek() == Some(']') {
        reader.next();
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    loop {
        let item = parse_item(reader)?;
        items.push(item);
        reader.skip_ws();
        match reader.peek() {
            Some(',') => {
                reader.next();
            }
            Some(']') => {
                reader.next();
                break;
            }
            _ => return Err("E_OBS_JSON_ARRAY".to_string()),
        }
    }
    Ok(items)
}

fn parse_u64(reader: &mut DetJsonReader) -> Result<u64, String> {
    let raw = reader.parse_number()?;
    if raw.contains('.') {
        return Err("E_OBS_JSON_INT".to_string());
    }
    raw.parse::<u64>().map_err(|_| "E_OBS_JSON_INT".to_string())
}

fn parse_fixed64(reader: &mut DetJsonReader) -> Result<Fixed64, String> {
    let raw = reader.parse_number()?;
    parse_fixed64_string(&raw)
}

fn parse_fixed64_string(input: &str) -> Result<Fixed64, String> {
    let text = input.trim();
    if text.is_empty() {
        return Err("E_OBS_FIXED64_EMPTY".to_string());
    }
    let mut sign = 1i128;
    let mut raw_text = text;
    if let Some(rest) = raw_text.strip_prefix('-') {
        sign = -1;
        raw_text = rest;
    } else if let Some(rest) = raw_text.strip_prefix('+') {
        raw_text = rest;
    }
    let mut parts = raw_text.splitn(2, '.');
    let int_part = parts.next().unwrap_or("");
    let frac_part = parts.next().unwrap_or("");

    let int_value = if int_part.is_empty() {
        0i128
    } else {
        if !int_part.chars().all(|c| c.is_ascii_digit()) {
            return Err("E_OBS_FIXED64_INT".to_string());
        }
        int_part
            .parse::<i128>()
            .map_err(|_| "E_OBS_FIXED64_INT".to_string())?
    };

    let frac_value = if frac_part.is_empty() {
        0i128
    } else {
        if !frac_part.chars().all(|c| c.is_ascii_digit()) {
            return Err("E_OBS_FIXED64_FRAC".to_string());
        }
        frac_part
            .parse::<i128>()
            .map_err(|_| "E_OBS_FIXED64_FRAC".to_string())?
    };

    let scale = 10i128.pow(frac_part.len() as u32);
    let frac_raw = if frac_part.is_empty() {
        0i128
    } else {
        (frac_value * (1i128 << 32)) / scale
    };

    let raw = (int_value << 32) + frac_raw;
    let signed = raw.saturating_mul(sign);
    let clamped = clamp_i128_to_i64(signed);
    Ok(Fixed64::from_raw_i64(clamped))
}

fn clamp_i128_to_i64(value: i128) -> i64 {
    if value > i64::MAX as i128 {
        i64::MAX
    } else if value < i64::MIN as i128 {
        i64::MIN
    } else {
        value as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_roundtrip_detjson() {
        let obs = Observation {
            agent_id: 1,
            madi: 100,
            timestamp_ms: 1000,
            self_state: AgentState {
                position: (Fixed64::from_i64(10), Fixed64::from_i64(20)),
                velocity: (Fixed64::from_i64(1), Fixed64::from_i64(0)),
                health: 100,
                status: "idle".to_string(),
            },
            visible_objects: vec![
                Object {
                    id: 2,
                    position: (Fixed64::from_i64(2), Fixed64::from_i64(2)),
                    object_type: "box".to_string(),
                    color: "yellow".to_string(),
                    size: Fixed64::from_i64(1),
                },
                Object {
                    id: 1,
                    position: (Fixed64::from_i64(1), Fixed64::from_i64(1)),
                    object_type: "ball".to_string(),
                    color: "blue".to_string(),
                    size: Fixed64::from_i64(1),
                },
            ],
            visible_agents: vec![Agent {
                id: 3,
                position: (Fixed64::from_i64(3), Fixed64::from_i64(3)),
                name: "ally".to_string(),
            }],
            world_state: WorldState {
                gravity: Fixed64::from_i64(-9),
                time_of_day: 12000,
                weather: "clear".to_string(),
            },
        };

        let json = observation_detjson(&obs);
        let parsed = observation_from_detjson(&json).unwrap();
        let observer = DefaultObserver::new(Fixed64::from_i64(100), 360);

        assert_eq!(observer.hash(&obs), observer.hash(&parsed));
        assert!(observer.equals(&obs, &parsed));
    }

    #[test]
    fn vision_cone_distance_check() {
        let vision = VisionCone {
            max_distance: Fixed64::from_i64(10),
            fov_degrees: 360,
        };
        let origin = (Fixed64::from_i64(0), Fixed64::from_i64(0));
        let near = (Fixed64::from_i64(3), Fixed64::from_i64(4));
        let far = (Fixed64::from_i64(11), Fixed64::from_i64(0));

        assert!(vision.can_see(origin, near));
        assert!(!vision.can_see(origin, far));
    }
}
