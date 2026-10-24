//! Exact numeric values and factor routes; unchanged implementations.
use super::{
    Integer,
    BTreeMap,
    BigInt,
    BigRational,
    EvalError,
    FACTOR_BIGINT_FACTOR_BITS_LIMIT,
    FACTOR_DECOMP_BITS_KEY,
    FACTOR_DECOMP_DEFERRED_REASON_BIT_LIMIT,
    FACTOR_DECOMP_DEFERRED_REASON_FACTOR_FAILED,
    FACTOR_DECOMP_DEFERRED_REASON_KEY,
    FACTOR_DECOMP_ROUTE_BIGINT,
    FACTOR_DECOMP_ROUTE_BIGINT_FALLBACK,
    FACTOR_DECOMP_ROUTE_BIGINT_MIXED,
    FACTOR_DECOMP_ROUTE_BIGINT_POLLARD,
    FACTOR_DECOMP_ROUTE_BIGINT_SMALL_PRIME,
    FACTOR_DECOMP_ROUTE_DEFERRED_BIT_LIMIT,
    FACTOR_DECOMP_ROUTE_DEFERRED_FACTOR_FAILED,
    FACTOR_DECOMP_ROUTE_I64,
    FACTOR_DECOMP_ROUTE_KEY,
    FACTOR_DECOMP_ROUTE_ZERO,
    FACTOR_DECOMP_STATUS_DEFERRED,
    FACTOR_DECOMP_STATUS_DONE,
    FACTOR_DECOMP_STATUS_KEY,
    FACTOR_POLLARD_C_SEED_LIMIT,
    FACTOR_POLLARD_MAX_ITERS,
    FACTOR_POLLARD_X0_SEEDS,
    FACTOR_SMALL_PRIMES,
    FACTOR_TRIAL_FALLBACK_LIMIT,
    Fixed64,
    NUMERIC_KIND_BIG_INT,
    NUMERIC_KIND_FACTOR,
    NUMERIC_KIND_RATIONAL,
    NUMERIC_PACK_APPROX_KEY,
    NUMERIC_PACK_KIND_KEY,
    Ordering,
    Sign,
    UnitError,
    Value,
    is_zero,
    unit_error,
    ToPrimitive,
    Zero,
};

#[cfg(test)]
use super::{FACTOR_POLLARD_MAX_ITERS_OVERRIDE, FACTOR_TRIAL_FALLBACK_LIMIT_OVERRIDE};

pub(super) fn make_numeric_pack(kind: &str, mut fields: BTreeMap<String, Value>) -> Value {
    fields.insert(
        NUMERIC_PACK_KIND_KEY.to_string(),
        Value::String(kind.to_string()),
    );
    Value::Pack(fields)
}

pub(super) fn numeric_pack_kind(value: &Value) -> Option<&str> {
    if matches!(value, Value::Rational(_)) { return Some(NUMERIC_KIND_RATIONAL); }
    let Value::Pack(fields) = value else {
        return None;
    };
    let Value::String(kind) = fields.get(NUMERIC_PACK_KIND_KEY)? else {
        return None;
    };
    (kind != NUMERIC_KIND_RATIONAL).then_some(kind.as_str())
}

pub(super) fn numeric_pack_approx(value: &Value) -> Option<Fixed64> {
    if let Value::Rational(value) = value {
        return Some(Fixed64::from_i64(value.ratio().numer().to_i64()?) / Fixed64::from_i64(value.ratio().denom().to_i64()?));
    }
    let Value::Pack(fields) = value else {
        return None;
    };
    match fields.get(NUMERIC_PACK_APPROX_KEY) {
        Some(Value::Fixed64(v)) => Some(*v),
        Some(Value::Unit(unit)) if unit.is_dimensionless() => Some(unit.value),
        _ => None,
    }
}

pub(super) fn value_to_fixed64_coerce(value: &Value) -> Result<Fixed64, EvalError> {
    match value {
        Value::Fixed64(v) => Ok(*v),
        Value::Unit(unit) if unit.is_dimensionless() => Ok(unit.value),
        Value::String(text) => Fixed64::parse_decimal(text)
            .ok_or_else(|| EvalError::from("수치 값이 필요합니다".to_string())),
        _ => numeric_pack_approx(value)
            .ok_or_else(|| EvalError::from("수치 값이 필요합니다".to_string())),
    }
}

pub(super) fn value_to_i64_strict(value: &Value) -> Result<i64, EvalError> {
    match value {
        Value::Fixed64(n) => {
            if n.frac_part() != 0 {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            Ok(n.int_part())
        }
        Value::Unit(unit) if unit.is_dimensionless() => {
            if unit.value.frac_part() != 0 {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            Ok(unit.value.int_part())
        }
        Value::String(text) => text
            .trim()
            .parse::<i64>()
            .map_err(|_| "정수 값이 필요합니다".to_string().into()),
        Value::Pack(_) | Value::Rational(_) => {
            let Some(exact) = exact_numeric_from_value(value)? else {
                return Err("정수 값이 필요합니다".to_string().into());
            };
            if !exact.is_integer() {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            let integer = exact.value.to_integer();
            integer
                .to_i64()
                .ok_or_else(|| EvalError::from("정수 값(i64 범위)이 필요합니다".to_string()))
        }
        _ => Err("정수 값이 필요합니다".to_string().into()),
    }
}

pub(super) fn parse_bigint_text(raw: &str, context: &str) -> Result<BigInt, EvalError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!("{context} 정수 문자열이 비었습니다")
            .to_string()
            .into());
    }
    let (negative, digits) = if let Some(rest) = trimmed.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = trimmed.strip_prefix('+') {
        (false, rest)
    } else {
        (false, trimmed)
    };
    let normalized = normalize_grouped_integer_digits(digits)
        .ok_or_else(|| EvalError::from(format!("{context} 정수 문자열 형식이 아닙니다")))?;
    if normalized.is_empty() {
        return Err(format!("{context} 정수 문자열 형식이 아닙니다")
            .to_string()
            .into());
    }
    let parsed = BigInt::parse_bytes(normalized.as_bytes(), 10)
        .ok_or_else(|| EvalError::from(format!("{context} 정수 문자열 형식이 아닙니다")))?;
    if negative {
        Ok(-parsed)
    } else {
        Ok(parsed)
    }
}

fn normalize_grouped_integer_digits(digits: &str) -> Option<String> {
    if digits.is_empty() {
        return None;
    }
    let mut out = String::with_capacity(digits.len());
    let mut prev_was_digit = false;
    let mut prev_was_sep = false;
    for ch in digits.chars() {
        if ch.is_ascii_digit() {
            out.push(ch);
            prev_was_digit = true;
            prev_was_sep = false;
            continue;
        }
        if ch == '_' {
            if !prev_was_digit || prev_was_sep {
                return None;
            }
            prev_was_digit = false;
            prev_was_sep = true;
            continue;
        }
        return None;
    }
    if prev_was_sep {
        return None;
    }
    if out.is_empty() {
        return None;
    }
    Some(out)
}

fn extract_bigint_string(value: &Value) -> Option<String> {
    let Value::Pack(fields) = value else {
        return None;
    };
    if numeric_pack_kind(value) != Some(NUMERIC_KIND_BIG_INT) {
        return None;
    }
    fields.get("값").and_then(|v| match v {
        Value::String(text) => Some(text.clone()),
        _ => None,
    })
}

fn bigint_from_value(value: &Value) -> Result<BigInt, EvalError> {
    match value {
        Value::Fixed64(n) => {
            if n.frac_part() != 0 {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            Ok(BigInt::from(n.int_part()))
        }
        Value::Unit(unit) if unit.is_dimensionless() => {
            if unit.value.frac_part() != 0 {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            Ok(BigInt::from(unit.value.int_part()))
        }
        Value::String(text) => parse_bigint_text(text, "정수"),
        Value::Pack(_) => {
            if let Some(raw) = extract_bigint_string(value) {
                return parse_bigint_text(&raw, "큰바른수");
            }
            if numeric_pack_kind(value) == Some(NUMERIC_KIND_RATIONAL) {
                let ratio = exact_numeric_from_value(value)?
                    .ok_or_else(|| EvalError::from("정수 값이 필요합니다".to_string()))?;
                if !ratio.value.is_integer() {
                    return Err("정수 값이 필요합니다".to_string().into());
                }
                return Ok(ratio.value.to_integer());
            }
            if let Some(v) = numeric_pack_approx(value) {
                if v.frac_part() == 0 {
                    return Ok(BigInt::from(v.int_part()));
                }
            }
            Err("정수 값이 필요합니다".to_string().into())
        }
        _ => Err("정수 값이 필요합니다".to_string().into()),
    }
}

fn factorize_i64(mut n: i64) -> Vec<(i64, i32)> {
    let mut out = Vec::new();
    if n < 0 {
        n = -n;
    }
    if n < 2 {
        return out;
    }
    let mut p = 2i64;
    while p.saturating_mul(p) <= n {
        let mut exp = 0i32;
        while n % p == 0 {
            n /= p;
            exp += 1;
        }
        if exp > 0 {
            out.push((p, exp));
        }
        p = if p == 2 { 3 } else { p + 2 };
    }
    if n > 1 {
        out.push((n, 1));
    }
    out
}

fn bigint_bit_len(value: &BigInt) -> usize {
    if value.is_zero() {
        0
    } else {
        value.to_str_radix(2).trim_start_matches('-').len()
    }
}

fn is_probable_prime_bigint(n: &BigInt) -> bool {
    let one = BigInt::from(1u8);
    let two = BigInt::from(2u8);
    let three = BigInt::from(3u8);

    if n < &two {
        return false;
    }
    if n == &two || n == &three {
        return true;
    }
    if (n % &two).is_zero() {
        return false;
    }

    for prime in FACTOR_SMALL_PRIMES {
        let p = BigInt::from(prime);
        if n == &p {
            return true;
        }
        if (n % &p).is_zero() {
            return false;
        }
    }

    let n_minus_one = n - &one;
    let mut d = n_minus_one.clone();
    let mut s = 0u32;
    while (&d % &two).is_zero() {
        d /= &two;
        s += 1;
    }

    let bases = [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    'witness: for base in bases {
        let a = BigInt::from(base);
        if &a >= n {
            continue;
        }
        let mut x = a.modpow(&d, n);
        if x == one || x == n_minus_one {
            continue;
        }
        for _ in 1..s {
            x = (&x * &x) % n;
            if x == n_minus_one {
                continue 'witness;
            }
        }
        return false;
    }
    true
}

fn pollard_rho_step(x: &BigInt, c: &BigInt, modulus: &BigInt) -> BigInt {
    ((x * x) + c) % modulus
}

fn factor_pollard_max_iters() -> usize {
    #[cfg(test)]
    {
        let override_iters = FACTOR_POLLARD_MAX_ITERS_OVERRIDE.with(|cell| cell.get());
        if let Some(value) = override_iters {
            return value;
        }
    }
    FACTOR_POLLARD_MAX_ITERS
}

#[cfg(test)]
pub(super) fn swap_factor_pollard_max_iters_for_test(value: usize) -> usize {
    FACTOR_POLLARD_MAX_ITERS_OVERRIDE.with(|cell| {
        let previous = cell.get().unwrap_or(usize::MAX);
        if value == usize::MAX {
            cell.set(None);
        } else {
            cell.set(Some(value));
        }
        previous
    })
}

fn factor_trial_fallback_limit() -> u64 {
    #[cfg(test)]
    {
        let override_limit = FACTOR_TRIAL_FALLBACK_LIMIT_OVERRIDE.with(|cell| cell.get());
        if let Some(value) = override_limit {
            return value;
        }
    }
    FACTOR_TRIAL_FALLBACK_LIMIT
}

fn factor_small_prime_max() -> u64 {
    FACTOR_SMALL_PRIMES.last().copied().unwrap_or(0)
}

pub(super) fn factor_policy_summary_text() -> String {
    format!(
        "bit_limit={};pollard_iters={};pollard_c_seeds={};pollard_x0_seeds={};fallback_limit={};small_prime_max={}",
        FACTOR_BIGINT_FACTOR_BITS_LIMIT,
        factor_pollard_max_iters(),
        FACTOR_POLLARD_C_SEED_LIMIT,
        FACTOR_POLLARD_X0_SEEDS.len(),
        factor_trial_fallback_limit(),
        factor_small_prime_max()
    )
}

#[cfg(test)]
pub(super) fn swap_factor_trial_fallback_limit_for_test(value: u64) -> u64 {
    FACTOR_TRIAL_FALLBACK_LIMIT_OVERRIDE.with(|cell| {
        let previous = cell.get().unwrap_or(u64::MAX);
        if value == u64::MAX {
            cell.set(None);
        } else {
            cell.set(Some(value));
        }
        previous
    })
}

fn fallback_trial_division_factor(n: &BigInt) -> Option<BigInt> {
    let mut candidate = 3u64;
    let limit = factor_trial_fallback_limit();
    while candidate <= limit {
        let factor = BigInt::from(candidate);
        if (&factor * &factor) > *n {
            break;
        }
        if (n % &factor).is_zero() {
            return Some(factor);
        }
        candidate += 2;
    }
    None
}

fn pollard_rho_factor(n: &BigInt) -> Option<BigInt> {
    let one = BigInt::from(1u8);
    let two = BigInt::from(2u8);

    if (n % &two).is_zero() {
        return Some(two);
    }
    if is_probable_prime_bigint(n) {
        return Some(n.clone());
    }

    for x0_seed in FACTOR_POLLARD_X0_SEEDS {
        for c_seed in 1u64..=FACTOR_POLLARD_C_SEED_LIMIT {
            let c = BigInt::from(c_seed);
            let mut x = BigInt::from(x0_seed);
            let mut y = BigInt::from(x0_seed);

            for _ in 0..factor_pollard_max_iters() {
                x = pollard_rho_step(&x, &c, n);
                y = pollard_rho_step(&y, &c, n);
                y = pollard_rho_step(&y, &c, n);
                let delta = if x >= y { &x - &y } else { &y - &x };
                let d = delta.gcd(n);
                if d == one {
                    continue;
                }
                if &d == n {
                    break;
                }
                return Some(d);
            }
        }
    }
    None
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct FactorizationStats {
    pub(super) used_small_prime: bool,
    pub(super) used_pollard: bool,
    pub(super) used_fallback: bool,
}

fn factor_route_for_bigint_done(stats: &FactorizationStats) -> &'static str {
    if stats.used_fallback && (stats.used_pollard || stats.used_small_prime) {
        return FACTOR_DECOMP_ROUTE_BIGINT_MIXED;
    }
    if stats.used_pollard && stats.used_small_prime {
        return FACTOR_DECOMP_ROUTE_BIGINT_MIXED;
    }
    if stats.used_fallback {
        return FACTOR_DECOMP_ROUTE_BIGINT_FALLBACK;
    }
    if stats.used_pollard {
        return FACTOR_DECOMP_ROUTE_BIGINT_POLLARD;
    }
    if stats.used_small_prime {
        return FACTOR_DECOMP_ROUTE_BIGINT_SMALL_PRIME;
    }
    FACTOR_DECOMP_ROUTE_BIGINT
}

fn factor_route_for_deferred(reason: &str) -> &'static str {
    match reason {
        FACTOR_DECOMP_DEFERRED_REASON_BIT_LIMIT => FACTOR_DECOMP_ROUTE_DEFERRED_BIT_LIMIT,
        FACTOR_DECOMP_DEFERRED_REASON_FACTOR_FAILED => FACTOR_DECOMP_ROUTE_DEFERRED_FACTOR_FAILED,
        _ => FACTOR_DECOMP_ROUTE_DEFERRED_FACTOR_FAILED,
    }
}

fn factorize_bigint_collect(
    n: BigInt,
    out: &mut Vec<BigInt>,
    stats: &mut FactorizationStats,
) -> bool {
    let one = BigInt::from(1u8);
    let two = BigInt::from(2u8);

    if n == one {
        return true;
    }
    if is_probable_prime_bigint(&n) {
        out.push(n);
        return true;
    }
    if (&n % &two).is_zero() {
        out.push(two.clone());
        return factorize_bigint_collect(n / &two, out, stats);
    }
    for prime in FACTOR_SMALL_PRIMES {
        let p = BigInt::from(prime);
        if (&n % &p).is_zero() {
            stats.used_small_prime = true;
            out.push(p.clone());
            return factorize_bigint_collect(n / &p, out, stats);
        }
    }

    let divisor = match pollard_rho_factor(&n) {
        Some(found) => {
            stats.used_pollard = true;
            found
        }
        None => match fallback_trial_division_factor(&n) {
            Some(found) => {
                stats.used_fallback = true;
                found
            }
            None => return false,
        },
    };
    if divisor == one || divisor == n {
        return false;
    }

    let quotient = &n / &divisor;
    factorize_bigint_collect(divisor, out, stats) && factorize_bigint_collect(quotient, out, stats)
}

fn factorize_bigint_full(
    n: BigInt,
) -> Result<(Vec<(BigInt, u32)>, FactorizationStats), &'static str> {
    let two = BigInt::from(2u8);
    if n < two {
        return Ok((Vec::new(), FactorizationStats::default()));
    }
    if bigint_bit_len(&n) > FACTOR_BIGINT_FACTOR_BITS_LIMIT {
        return Err(FACTOR_DECOMP_DEFERRED_REASON_BIT_LIMIT);
    }

    let mut primes = Vec::new();
    let mut stats = FactorizationStats::default();
    if !factorize_bigint_collect(n, &mut primes, &mut stats) {
        return Err(FACTOR_DECOMP_DEFERRED_REASON_FACTOR_FAILED);
    }
    primes.sort();

    let mut out: Vec<(BigInt, u32)> = Vec::new();
    for prime in primes {
        if let Some((last_prime, exp)) = out.last_mut() {
            if *last_prime == prime {
                *exp += 1;
                continue;
            }
        }
        out.push((prime, 1));
    }
    Ok((out, stats))
}

fn factor_canon(sign: i8, factors: &[(i64, i32)]) -> String {
    if factors.is_empty() {
        return if sign < 0 {
            "-1".to_string()
        } else {
            "1".to_string()
        };
    }
    let body = factors
        .iter()
        .map(|(prime, exp)| {
            if *exp == 1 {
                prime.to_string()
            } else {
                format!("{prime}^{exp}")
            }
        })
        .collect::<Vec<_>>()
        .join(" * ");
    if sign < 0 {
        format!("-{body}")
    } else {
        body
    }
}

fn factor_canon_bigint(sign: i8, factors: &[(BigInt, u32)]) -> String {
    if factors.is_empty() {
        return if sign < 0 {
            "-1".to_string()
        } else {
            "1".to_string()
        };
    }
    let body = factors
        .iter()
        .map(|(prime, exp)| {
            if *exp == 1 {
                prime.to_string()
            } else {
                format!("{prime}^{exp}")
            }
        })
        .collect::<Vec<_>>()
        .join(" * ");
    if sign < 0 {
        format!("-{body}")
    } else {
        body
    }
}

pub(super) fn make_big_int_value(value: &Value) -> Result<Value, EvalError> {
    let parsed = bigint_from_value(value)?;
    let text = parsed.to_string();
    let mut fields = BTreeMap::new();
    fields.insert("값".to_string(), Value::String(text));
    if let Some(v) = parsed.to_i64() {
        fields.insert(
            NUMERIC_PACK_APPROX_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(v)),
        );
    }
    Ok(make_numeric_pack(NUMERIC_KIND_BIG_INT, fields))
}

pub(super) fn make_rational_value(numerator: &Value, denominator: &Value) -> Result<Value, EvalError> {
    let n = bigint_from_value(numerator)?;
    let d = bigint_from_value(denominator)?;
    if d.is_zero() {
        return Err("나눔수의 분모는 0일 수 없습니다".to_string().into());
    }
    let ratio = BigRational::new(n, d);
    Ok(make_rational_pack_from_big_rational(&ratio))
}

pub(super) fn make_factor_value(value: &Value) -> Result<(Value, &'static str), EvalError> {
    let parsed = bigint_from_value(value)?;
    let sign: i8 = match parsed.sign() {
        Sign::Minus => -1,
        Sign::NoSign => 0,
        Sign::Plus => 1,
    };
    let (canon, status, deferred_reason, route, bit_len) = if parsed.is_zero() {
        (
            "0".to_string(),
            FACTOR_DECOMP_STATUS_DONE,
            None,
            FACTOR_DECOMP_ROUTE_ZERO,
            0usize,
        )
    } else if let Some(integer) = parsed.to_i64() {
        if integer == i64::MIN {
            let abs = -parsed.clone();
            let bit_len = bigint_bit_len(&abs);
            match factorize_bigint_full(abs) {
                Ok((factors, stats)) => (
                    factor_canon_bigint(sign, &factors),
                    FACTOR_DECOMP_STATUS_DONE,
                    None,
                    factor_route_for_bigint_done(&stats),
                    bit_len,
                ),
                Err(reason) => (
                    parsed.to_string(),
                    FACTOR_DECOMP_STATUS_DEFERRED,
                    Some(reason),
                    factor_route_for_deferred(reason),
                    bit_len,
                ),
            }
        } else {
            let factors = factorize_i64(integer);
            let abs = integer.unsigned_abs();
            let bit_len = if abs == 0 {
                0usize
            } else {
                (u64::BITS - abs.leading_zeros()) as usize
            };
            (
                factor_canon(sign, &factors),
                FACTOR_DECOMP_STATUS_DONE,
                None,
                FACTOR_DECOMP_ROUTE_I64,
                bit_len,
            )
        }
    } else {
        let abs = if sign < 0 {
            -parsed.clone()
        } else {
            parsed.clone()
        };
        let bit_len = bigint_bit_len(&abs);
        match factorize_bigint_full(abs) {
            Ok((factors, stats)) => (
                factor_canon_bigint(sign, &factors),
                FACTOR_DECOMP_STATUS_DONE,
                None,
                factor_route_for_bigint_done(&stats),
                bit_len,
            ),
            // bit 제한 또는 분해 실패 시에는 표현 안정성을 위해 지연으로 둔다.
            Err(reason) => (
                parsed.to_string(),
                FACTOR_DECOMP_STATUS_DEFERRED,
                Some(reason),
                factor_route_for_deferred(reason),
                bit_len,
            ),
        }
    };
    let mut fields = BTreeMap::new();
    fields.insert("값".to_string(), Value::String(parsed.to_string()));
    fields.insert("표기".to_string(), Value::String(canon));
    fields.insert(
        FACTOR_DECOMP_STATUS_KEY.to_string(),
        Value::String(status.to_string()),
    );
    fields.insert(
        "부호".to_string(),
        Value::Fixed64(Fixed64::from_i64(sign as i64)),
    );
    fields.insert(
        FACTOR_DECOMP_ROUTE_KEY.to_string(),
        Value::String(route.to_string()),
    );
    fields.insert(
        FACTOR_DECOMP_BITS_KEY.to_string(),
        Value::Fixed64(Fixed64::from_i64(bit_len as i64)),
    );
    if let Some(reason) = deferred_reason {
        fields.insert(
            FACTOR_DECOMP_DEFERRED_REASON_KEY.to_string(),
            Value::String(reason.to_string()),
        );
    }
    if let Some(v) = parsed.to_i64() {
        fields.insert(
            NUMERIC_PACK_APPROX_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(v)),
        );
    }
    Ok((make_numeric_pack(NUMERIC_KIND_FACTOR, fields), status))
}

#[derive(Clone, Debug)]
pub(super) struct ExactNumeric {
    pub(super) value: BigRational,
}

impl ExactNumeric {
    pub(super) fn from_bigint(value: BigInt) -> Self {
        Self {
            value: BigRational::from_integer(value),
        }
    }

    pub(super) fn from_fixed64(value: Fixed64) -> Self {
        let num = BigInt::from(value.raw_i64());
        let den = BigInt::from(1_i64) << Fixed64::FRAC_BITS;
        Self {
            value: BigRational::new(num, den),
        }
    }

    pub(super) fn is_integer(&self) -> bool {
        self.value.is_integer()
    }
}

pub(super) fn eval_exact_numeric_infix(
    op: &str,
    left: &Value,
    right: &Value,
) -> Result<Option<Value>, EvalError> {
    if !is_exact_numeric_family_value(left) && !is_exact_numeric_family_value(right) {
        return Ok(None);
    }
    let left_kind = numeric_pack_kind(left);
    let right_kind = numeric_pack_kind(right);
    let both_factor =
        left_kind == Some(NUMERIC_KIND_FACTOR) && right_kind == Some(NUMERIC_KIND_FACTOR);
    let Some(left_exact) = exact_numeric_from_value(left)? else {
        return Ok(None);
    };
    let Some(right_exact) = exact_numeric_from_value(right)? else {
        return Ok(None);
    };

    let out = match op {
        "+" => exact_add(&left_exact, &right_exact),
        "-" => exact_sub(&left_exact, &right_exact),
        "*" => exact_mul(&left_exact, &right_exact),
        "/" => exact_div(&left_exact, &right_exact)?,
        "%" => exact_rem(&left_exact, &right_exact)?,
        _ => return Ok(None),
    };
    if both_factor && matches!(op, "*" | "/") && out.is_integer() {
        let integer = out.value.to_integer();
        let (value, _) = make_factor_value(&make_big_int_pack_from_bigint(&integer))?;
        return Ok(Some(value));
    }
    let prefer_rational = op == "/"
        || left_kind == Some(NUMERIC_KIND_RATIONAL)
        || right_kind == Some(NUMERIC_KIND_RATIONAL)
        || both_factor && matches!(op, "+" | "-")
        || !left_exact.is_integer()
        || !right_exact.is_integer();
    Ok(Some(exact_numeric_to_value(out, prefer_rational)))
}

pub(super) fn eval_exact_numeric_compare_infix(
    op: &str,
    left: &Value,
    right: &Value,
) -> Result<Option<bool>, EvalError> {
    // The operator expectation must issue source literals before evaluation.
    // Do not reverse-convert an otherwise anonymous Fixed64 to a rational at
    // equality time, including recursive collection comparisons. Preserve the
    // existing legacy integer-family/i64 producer path; statically issued
    // cross-kind owners are still rejected by check_numeric_comparison_owners.
    // F5-GA[EXACT-KIND]: 같은 불변 값이 Some(나눔수)인 분기에서만 다시 읽으며 두 조회 사이 변경이 없다.
    let unresolved = if numeric_pack_kind(left) == Some(NUMERIC_KIND_RATIONAL) && matches!(right, Value::Fixed64(_)) {
        Some(("오른쪽", numeric_pack_kind(left).expect("exact kind")))
    } else if numeric_pack_kind(right) == Some(NUMERIC_KIND_RATIONAL) && matches!(left, Value::Fixed64(_)) {
        Some(("왼쪽", numeric_pack_kind(right).expect("exact kind")))
    } else { None };
    if let Some((side, exact_kind))=unresolved {
        return Err(format!("E_NUMERIC_COMPARISON_TYPE_UNRESOLVED: {op} 비교에서 {exact_kind}와 비교할 {side} 계산식의 수 종류를 확정할 수 없습니다. 미지원 수 혼합을 참·거짓으로 처리하지 않습니다. 두 값을 같은 수 종류로 명시해 주세요").into());
    }
    if let (Some(left), Some(right)) = (numeric_pack_kind(left), numeric_pack_kind(right)) {
        if left != right {
            return Err(format!("E_NUMERIC_COMPARISON_TYPE_MISMATCH: {op} 비교의 왼쪽은 {left}, 오른쪽은 {right}입니다. 형식이 정해진 서로 다른 수 종류는 자동 변환하여 비교하지 않습니다").into());
        }
    }
    if !is_exact_numeric_family_value(left) && !is_exact_numeric_family_value(right) {
        return Ok(None);
    }
    let Some(left_exact) = exact_numeric_from_value(left)? else {
        return Ok(None);
    };
    let Some(right_exact) = exact_numeric_from_value(right)? else {
        return Ok(None);
    };
    let ord = exact_cmp(&left_exact, &right_exact);
    let out = match op {
        "==" => ord == std::cmp::Ordering::Equal,
        "!=" => ord != std::cmp::Ordering::Equal,
        "<" => ord == std::cmp::Ordering::Less,
        "<=" => ord != std::cmp::Ordering::Greater,
        ">" => ord == std::cmp::Ordering::Greater,
        ">=" => ord != std::cmp::Ordering::Less,
        _ => return Ok(None),
    };
    Ok(Some(out))
}

pub(super) fn is_exact_numeric_family_value(value: &Value) -> bool {
    matches!(
        numeric_pack_kind(value),
        Some(NUMERIC_KIND_BIG_INT | NUMERIC_KIND_RATIONAL | NUMERIC_KIND_FACTOR)
    )
}

pub(super) fn exact_numeric_from_value(value: &Value) -> Result<Option<ExactNumeric>, EvalError> {
    match value {
        Value::Rational(value) => Ok(Some(ExactNumeric { value: value.ratio().clone() })),
        Value::Fixed64(n) => Ok(Some(ExactNumeric::from_fixed64(*n))),
        Value::Unit(unit) if unit.is_dimensionless() => {
            Ok(Some(ExactNumeric::from_fixed64(unit.value)))
        }
        Value::Pack(fields) => match numeric_pack_kind(value) {
            Some(NUMERIC_KIND_BIG_INT) => {
                let raw = fields
                    .get("값")
                    .and_then(|v| match v {
                        Value::String(text) => Some(text.as_str()),
                        _ => None,
                    })
                    .ok_or_else(|| EvalError::from("큰바른수.값이 없습니다".to_string()))?;
                let parsed = parse_bigint_text(raw, "큰바른수")?;
                Ok(Some(ExactNumeric::from_bigint(parsed)))
            }
            Some(NUMERIC_KIND_RATIONAL) => {
                let num_raw = fields
                    .get("분자")
                    .and_then(|v| match v {
                        Value::String(text) => Some(text.as_str()),
                        _ => None,
                    })
                    .ok_or_else(|| EvalError::from("나눔수.분자가 없습니다".to_string()))?;
                let den_raw = fields
                    .get("분모")
                    .and_then(|v| match v {
                        Value::String(text) => Some(text.as_str()),
                        _ => None,
                    })
                    .ok_or_else(|| EvalError::from("나눔수.분모가 없습니다".to_string()))?;
                let num = parse_bigint_text(num_raw, "나눔수 분자")?;
                let den = parse_bigint_text(den_raw, "나눔수 분모")?;
                if den.is_zero() {
                    return Err("나눔수의 분모는 0일 수 없습니다".to_string().into());
                }
                Ok(Some(ExactNumeric {
                    value: BigRational::new(num, den),
                }))
            }
            Some(NUMERIC_KIND_FACTOR) => {
                if let Some(raw) = fields.get("값").and_then(|v| match v {
                    Value::String(text) => Some(text.as_str()),
                    _ => None,
                }) {
                    let parsed = parse_bigint_text(raw, "곱수 값")?;
                    return Ok(Some(ExactNumeric::from_bigint(parsed)));
                }
                let approx = numeric_pack_approx(value)
                    .ok_or_else(|| EvalError::from("곱수 근사값이 없습니다".to_string()))?;
                if approx.frac_part() != 0 {
                    return Err("곱수 근사값은 정수여야 합니다".to_string().into());
                }
                Ok(Some(ExactNumeric::from_bigint(BigInt::from(
                    approx.int_part(),
                ))))
            }
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

fn exact_add(left: &ExactNumeric, right: &ExactNumeric) -> ExactNumeric {
    ExactNumeric {
        value: left.value.clone() + right.value.clone(),
    }
}

fn exact_sub(left: &ExactNumeric, right: &ExactNumeric) -> ExactNumeric {
    ExactNumeric {
        value: left.value.clone() - right.value.clone(),
    }
}

fn exact_mul(left: &ExactNumeric, right: &ExactNumeric) -> ExactNumeric {
    ExactNumeric {
        value: left.value.clone() * right.value.clone(),
    }
}

fn exact_div(left: &ExactNumeric, right: &ExactNumeric) -> Result<ExactNumeric, EvalError> {
    if right.value.is_zero() {
        return Err(unit_error(UnitError::DivisionByZero));
    }
    Ok(ExactNumeric {
        value: left.value.clone() / right.value.clone(),
    })
}

fn exact_rem(left: &ExactNumeric, right: &ExactNumeric) -> Result<ExactNumeric, EvalError> {
    if !left.is_integer() || !right.is_integer() {
        return Err("나머지 연산은 정수형(바른수/큰바른수/곱수)만 지원합니다"
            .to_string()
            .into());
    }
    let l = left.value.to_integer();
    let r = right.value.to_integer();
    if r.is_zero() {
        return Err(unit_error(UnitError::DivisionByZero));
    }
    Ok(ExactNumeric::from_bigint(l % r))
}

pub(super) fn exact_cmp(left: &ExactNumeric, right: &ExactNumeric) -> std::cmp::Ordering {
    left.value.cmp(&right.value)
}

pub(super) fn make_big_int_pack_from_bigint(value: &BigInt) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert("값".to_string(), Value::String(value.to_string()));
    if let Some(v) = value.to_i64() {
        fields.insert(
            NUMERIC_PACK_APPROX_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(v)),
        );
    }
    make_numeric_pack(NUMERIC_KIND_BIG_INT, fields)
}

pub(super) fn make_rational_pack_from_big_rational(value: &BigRational) -> Value {
    Value::Rational(ddonirang_core::saved_rational::NormalizedRationalValueV1::from_parts(
        value.numer().clone(), value.denom().clone(),
    ).expect("BigRational denominator is nonzero"))
}

fn exact_numeric_to_value(value: ExactNumeric, prefer_rational: bool) -> Value {
    if prefer_rational || !value.is_integer() {
        make_rational_pack_from_big_rational(&value.value)
    } else {
        let integer = value.value.to_integer();
        make_big_int_pack_from_bigint(&integer)
    }
}

