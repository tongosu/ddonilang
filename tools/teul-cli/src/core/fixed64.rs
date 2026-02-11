#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fixed64 {
    raw: i64,
}

impl Fixed64 {
    pub const SCALE_BITS: u32 = 32;
    pub const SCALE: i64 = 1_i64 << Self::SCALE_BITS;

    pub fn from_raw(raw: i64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> i64 {
        self.raw
    }

    #[allow(dead_code)]
    pub fn zero() -> Self {
        Self::from_raw(0)
    }

    pub fn one() -> Self {
        Self::from_raw(Self::SCALE)
    }

    #[allow(dead_code)]
    pub fn from_int(value: i64) -> Self {
        Self::from_raw(value.saturating_mul(Self::SCALE))
    }

    #[allow(dead_code)]
    pub fn from_ratio(num: i64, den: i64) -> Self {
        if den == 0 {
            return Self::from_raw(i64::MAX);
        }
        let value = ((num as i128) << Self::SCALE_BITS) / (den as i128);
        Self::from_raw(saturate_i128(value))
    }

    pub fn saturating_add(self, other: Self) -> Self {
        Self::from_raw(self.raw.saturating_add(other.raw))
    }

    pub fn saturating_sub(self, other: Self) -> Self {
        Self::from_raw(self.raw.saturating_sub(other.raw))
    }

    pub fn saturating_mul(self, other: Self) -> Self {
        let raw = mul_raw(self.raw, other.raw);
        Self::from_raw(raw)
    }

    pub fn checked_div(self, other: Self) -> Option<Self> {
        if other.raw == 0 {
            return None;
        }
        let raw = div_raw(self.raw, other.raw);
        Some(Self::from_raw(raw))
    }

    pub fn powi(self, exp: i32) -> Self {
        if exp == 0 {
            return Self::one();
        }
        let mut acc = Self::one();
        for _ in 0..exp.abs() {
            acc = acc.saturating_mul(self);
        }
        if exp < 0 {
            match Self::one().checked_div(acc) {
                Some(value) => value,
                None => Self::from_raw(i64::MAX),
            }
        } else {
            acc
        }
    }

    pub fn format(self) -> String {
        format_fixed64(self.raw)
    }

    pub fn parse_literal(text: &str) -> Option<Self> {
        parse_fixed64(text).map(Self::from_raw)
    }

    pub fn sqrt(self) -> Option<Self> {
        if self.raw < 0 {
            return None;
        }
        let scaled = (self.raw as i128) << Self::SCALE_BITS;
        let root = int_sqrt(scaled);
        Some(Self::from_raw(saturate_i128(root)))
    }
}

fn mul_raw(a: i64, b: i64) -> i64 {
    let prod = (a as i128) * (b as i128);
    let shifted = prod >> Fixed64::SCALE_BITS;
    saturate_i128(shifted)
}

fn div_raw(a: i64, b: i64) -> i64 {
    let num = (a as i128) << Fixed64::SCALE_BITS;
    let quot = num / (b as i128);
    saturate_i128(quot)
}

fn saturate_i128(value: i128) -> i64 {
    if value > i64::MAX as i128 {
        i64::MAX
    } else if value < i64::MIN as i128 {
        i64::MIN
    } else {
        value as i64
    }
}

fn parse_fixed64(text: &str) -> Option<i64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let negative = trimmed.starts_with('-');
    let digits = trimmed.strip_prefix('-').unwrap_or(trimmed);
    let mut parts = digits.splitn(2, '.');
    let int_part = parts.next().unwrap_or("");
    let frac_part = parts.next().unwrap_or("");

    let int_value = int_part.parse::<i128>().ok()?;
    let mut raw = int_value.saturating_mul(Fixed64::SCALE as i128);

    if !frac_part.is_empty() {
        let mut frac_value: i128 = 0;
        let mut denom: i128 = 1;
        for ch in frac_part.chars() {
            if !ch.is_ascii_digit() {
                return None;
            }
            frac_value = frac_value.saturating_mul(10).saturating_add((ch as u8 - b'0') as i128);
            denom = denom.saturating_mul(10);
        }
        let frac_scaled = (frac_value << Fixed64::SCALE_BITS) / denom;
        raw = raw.saturating_add(frac_scaled);
    }

    let signed = if negative { -raw } else { raw };
    Some(saturate_i128(signed))
}

fn format_fixed64(raw: i64) -> String {
    if raw == 0 {
        return "0".to_string();
    }

    let negative = raw < 0;
    let abs = if raw == i64::MIN {
        (i64::MAX as i128 + 1) as i128
    } else {
        raw.abs() as i128
    };

    let int_part = abs >> Fixed64::SCALE_BITS;
    let mut frac = abs & ((1_i128 << Fixed64::SCALE_BITS) - 1);

    let mut out = int_part.to_string();
    if frac != 0 {
        let mut digits = String::new();
        for _ in 0..10 {
            frac *= 10;
            let digit = (frac >> Fixed64::SCALE_BITS) as u8;
            frac &= (1_i128 << Fixed64::SCALE_BITS) - 1;
            digits.push((b'0' + digit) as char);
        }
        while digits.ends_with('0') {
            digits.pop();
        }
        if !digits.is_empty() {
            out.push('.');
            out.push_str(&digits);
        }
    }

    if negative {
        out.insert(0, '-');
    }
    out
}

fn int_sqrt(value: i128) -> i128 {
    if value <= 0 {
        return 0;
    }
    let mut x = value;
    let mut y = (x + 1) >> 1;
    while y < x {
        x = y;
        y = (x + value / x) >> 1;
    }
    x
}
