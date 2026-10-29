#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AgeTarget {
    Age0,
    Age1,
    Age2,
    Age3,
    Age4,
    Age5,
    Age6,
    Age7,
}

impl AgeTarget {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_uppercase().as_str() {
            "AGE0" => Some(Self::Age0),
            "AGE1" => Some(Self::Age1),
            "AGE2" => Some(Self::Age2),
            "AGE3" => Some(Self::Age3),
            "AGE4" => Some(Self::Age4),
            "AGE5" => Some(Self::Age5),
            "AGE6" => Some(Self::Age6),
            "AGE7" => Some(Self::Age7),
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Age0 => "AGE0",
            Self::Age1 => "AGE1",
            Self::Age2 => "AGE2",
            Self::Age3 => "AGE3",
            Self::Age4 => "AGE4",
            Self::Age5 => "AGE5",
            Self::Age6 => "AGE6",
            Self::Age7 => "AGE7",
        }
    }
}

pub fn age_not_available_error(feature: &str, need: AgeTarget, current: AgeTarget) -> String {
    format!(
        "E_AGE_NOT_AVAILABLE 요청 기능은 현재 AGE에서 사용할 수 없습니다: {} (need {}, current {})",
        feature,
        need.label(),
        current.label()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_age_target_accepts_age0_to_age7() {
        for label in [
            "AGE0", "AGE1", "AGE2", "AGE3", "AGE4", "AGE5", "AGE6", "AGE7",
        ] {
            assert!(AgeTarget::parse(label).is_some(), "must parse {}", label);
        }
    }

    #[test]
    fn age_not_available_error_has_expected_format() {
        let err = age_not_available_error("open_mode", AgeTarget::Age2, AgeTarget::Age1);
        assert!(err.contains("E_AGE_NOT_AVAILABLE"));
        assert!(err.contains("open_mode"));
        assert!(err.contains("need AGE2"));
        assert!(err.contains("current AGE1"));
    }
}
