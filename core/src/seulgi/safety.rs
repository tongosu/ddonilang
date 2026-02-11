#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SafetyMode {
    AllowList,
    DenyList,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafetyRule {
    pub mode: SafetyMode,
    pub intents: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafetyDecision {
    pub allowed: bool,
    pub reason: String,
}

pub fn check(rule: &SafetyRule, intent_kind: &str) -> SafetyDecision {
    let listed = rule.intents.iter().any(|item| item == intent_kind);
    match rule.mode {
        SafetyMode::AllowList => {
            if listed {
                SafetyDecision { allowed: true, reason: "allowlist hit".to_string() }
            } else {
                SafetyDecision { allowed: false, reason: "allowlist miss".to_string() }
            }
        }
        SafetyMode::DenyList => {
            if listed {
                SafetyDecision { allowed: false, reason: "denylist hit".to_string() }
            } else {
                SafetyDecision { allowed: true, reason: "denylist miss".to_string() }
            }
        }
    }
}
