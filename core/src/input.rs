use crate::platform::{KEY_A, KEY_D, KEY_S, KEY_W};

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
}
