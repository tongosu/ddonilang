use crate::signals::{ArithmeticFaultKind, FaultContext, Signal, SignalSink};
use std::fmt;

/// SSOT: Fixed64(Q32.32), raw_value = value * 2^32
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fixed64 {
    raw: i64,
}

impl Fixed64 {
    pub const FRAC_BITS: u32 = 32;
    pub const ONE_RAW: i64 = 1i64 << 32;

    pub const MIN: Fixed64 = Fixed64 { raw: i64::MIN };
    pub const MAX: Fixed64 = Fixed64 { raw: i64::MAX };
    pub const ZERO: Fixed64 = Fixed64 { raw: 0 };
    pub const ONE: Fixed64 = Fixed64 { raw: Self::ONE_RAW };

    #[inline]
    pub const fn from_raw_i64(raw: i64) -> Self {
        Self { raw }
    }

    #[inline]
    pub const fn raw_i64(self) -> i64 {
        self.raw
    }

    /// Current fixed-point arithmetic contract: overflow is a typed failure,
    /// never a saturated value.
    #[inline]
    pub fn checked_add(self, rhs: Self) -> Result<Self, ArithmeticFaultKind> {
        self.raw
            .checked_add(rhs.raw)
            .map(Self::from_raw_i64)
            .ok_or(ArithmeticFaultKind::Overflow)
    }

    #[inline]
    pub fn checked_sub(self, rhs: Self) -> Result<Self, ArithmeticFaultKind> {
        self.raw
            .checked_sub(rhs.raw)
            .map(Self::from_raw_i64)
            .ok_or(ArithmeticFaultKind::Overflow)
    }

    /// Q31.32 multiplication with exact integer ties-to-even rounding.
    #[inline]
    pub fn checked_mul(self, rhs: Self) -> Result<Self, ArithmeticFaultKind> {
        let product = (self.raw as i128) * (rhs.raw as i128);
        let raw = round_div_ties_to_even(product, 1i128 << Self::FRAC_BITS)
            .ok_or(ArithmeticFaultKind::Overflow)?;
        i64::try_from(raw)
            .map(Self::from_raw_i64)
            .map_err(|_| ArithmeticFaultKind::Overflow)
    }

    /// Exact Q31.32 weighted blend with a dimensionless weight in `[0, 1]`.
    /// The two products are accumulated in one widened intermediate so a
    /// representable endpoint is never lost to component-wise rounding.
    #[inline]
    pub fn checked_weighted_blend(
        self,
        next: Self,
        weight: Self,
    ) -> Result<Self, ArithmeticFaultKind> {
        if weight.raw < 0 || weight.raw > Self::ONE_RAW {
            return Err(ArithmeticFaultKind::Overflow);
        }
        let keep = Self::ONE_RAW - weight.raw;
        let numerator = (self.raw as i128) * (keep as i128)
            + (next.raw as i128) * (weight.raw as i128);
        let raw = round_div_ties_to_even(numerator, Self::ONE_RAW as i128)
            .ok_or(ArithmeticFaultKind::Overflow)?;
        i64::try_from(raw)
            .map(Self::from_raw_i64)
            .map_err(|_| ArithmeticFaultKind::Overflow)
    }

    /// 정수 -> Fixed64 (value << 32), 범위 밖은 포화
    #[inline]
    pub fn from_i64(value: i64) -> Self {
        let x = (value as i128) << (Self::FRAC_BITS as i128);
        Self {
            raw: clamp_i128_to_i64(x),
        }
    }

    /// i32 -> Fixed64
    #[inline]
    pub fn from_i32(value: i32) -> Self {
        Self::from_i64(value as i64)
    }

    /// 유리수 -> Fixed64. 몫은 0 방향으로 버리고 범위 밖은 포화한다.
    #[inline]
    pub fn from_ratio(num: i64, den: i64) -> Self {
        if den == 0 {
            return match num.cmp(&0) {
                core::cmp::Ordering::Less => Self::MIN,
                core::cmp::Ordering::Equal => Self::ZERO,
                core::cmp::Ordering::Greater => Self::MAX,
            };
        }
        let scaled = (num as i128) << Self::FRAC_BITS;
        Self::from_raw_i64(clamp_i128_to_i64(scaled / den as i128))
    }

    /// Current textual numeric admission.  Values outside Q31.32 are rejected;
    /// this path never converts overflow into a saturated numeric value.
    pub fn parse_decimal(input: &str) -> Option<Self> {
        Self::parse_decimal_exact(input)
    }

    /// 사용자 원문 10진 문자열 -> Fixed64. 표현 범위를 넘는 값은 포화하지 않고 거부한다.
    pub fn parse_decimal_exact(input: &str) -> Option<Self> {
        parse_decimal_raw_checked(input).and_then(|(raw, overflowed)| {
            (!overflowed).then(|| Self::from_raw_i64(raw))
        })
    }

    /// 덧셈: SSOT 요구대로 포화(saturating)
    #[inline]
    pub fn saturating_add(self, rhs: Self) -> Self {
        Self {
            raw: self.raw.saturating_add(rhs.raw),
        }
    }

    /// 뺄셈: SSOT 요구대로 포화(saturating)
    #[inline]
    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self {
            raw: self.raw.saturating_sub(rhs.raw),
        }
    }

    /// 곱셈: i128 중간값으로 결정성 보장, 오버플로는 포화
    /// Q32.32: (a.raw * b.raw) >> 32
    #[inline]
    pub fn saturating_mul(self, rhs: Self) -> Self {
        let prod = (self.raw as i128) * (rhs.raw as i128);
        let shifted = prod >> (Self::FRAC_BITS as i128);
        Self {
            raw: clamp_i128_to_i64(shifted),
        }
    }

    /// 나눗셈(결정적): 0으로 나누기면 Err(산술고장)
    /// Q32.32: (a.raw << 32) / b.raw
    #[inline]
    pub fn try_div(self, rhs: Self) -> Result<Self, ArithmeticFaultKind> {
        if rhs.raw == 0 {
            return Err(ArithmeticFaultKind::DivByZero);
        }
        let num = (self.raw as i128) << (Self::FRAC_BITS as i128);
        let raw = round_div_ties_to_even(num, rhs.raw as i128)
            .ok_or(ArithmeticFaultKind::Overflow)?;
        i64::try_from(raw)
            .map(Self::from_raw_i64)
            .map_err(|_| ArithmeticFaultKind::Overflow)
    }

    /// SSOT: 0으로 나누기 발생 시 "대입 무효화" + "산술고장 신호"
    /// - rhs==0 이면 self는 그대로 유지
    /// - sink로 산술고장 emit
    #[inline]
    pub fn div_assign_det(&mut self, rhs: Self, ctx: FaultContext, sink: &mut dyn SignalSink) {
        if rhs.raw == 0 {
            sink.emit(Signal::ArithmeticFault {
                ctx,
                kind: ArithmeticFaultKind::DivByZero,
            });
            // 대입 무효화: self 변경 없음
            return;
        }
        match self.try_div(rhs) {
            Ok(value) => *self = value,
            Err(kind) => sink.emit(Signal::ArithmeticFault { ctx, kind }),
        }
    }

    #[inline]
    pub const fn int_part(self) -> i64 {
        self.raw >> 32
    }

    #[inline]
    pub const fn frac_part(self) -> i64 {
        self.raw & 0xFFFF_FFFF
    }

    pub const NEG_ONE: Self = Self::from_raw_i64(-1 << 32);

    pub const fn to_raw(self) -> i64 {
        self.raw
    }

    // SSOT/Gate0 raw_i64 결정성 벡터 기준값(v1)
    pub const DETERMINISM_VECTOR_V1_EXPECTED: [i64; 7] = [
        0x0000_0001_8000_0000,
        0x0000_0000_8000_0000,
        -0x0000_0000_8000_0000i64,
        0x0000_0000_8000_0000,
        0x0000_0002_0000_0000,
        0x0000_0000_8000_0000,
        0x0000_0000_0000_0000,
    ];

    // 크로스 플랫폼 비교용 raw_i64 결정성 벡터 생성(v1)
    pub fn determinism_vector_v1() -> [i64; 7] {
        let a = Fixed64::from_raw_i64(0x0000_0001_0000_0000); // 1.0
        let b = Fixed64::from_raw_i64(0x0000_0000_8000_0000); // 0.5
        let c = Fixed64::NEG_ONE; // -1.0
        // F5-GA[FIXED-VECTOR]: 두 try_div의 분모는 이 함수가 직접 만든 0이 아닌 1과 0.5이며 결과도 표현 범위 안이다.
        [
            (a + b).raw_i64(),
            (a - b).raw_i64(),
            (b - a).raw_i64(),
            (a * b).raw_i64(),
            a.try_div(b).expect("determinism_vector_v1 a/b").raw_i64(),
            b.try_div(a).expect("determinism_vector_v1 b/a").raw_i64(),
            (c + a).raw_i64(),
        ]
    }
}

/// Integer-only round-to-nearest with exact ties-to-even.  `denominator` must
/// be non-zero; `None` represents an impossible arithmetic domain.
fn round_div_ties_to_even(numerator: i128, denominator: i128) -> Option<i128> {
    if denominator == 0 {
        return None;
    }
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    if remainder == 0 {
        return Some(quotient);
    }
    let abs_remainder = remainder.unsigned_abs();
    let abs_denominator = denominator.unsigned_abs();
    let twice_remainder = abs_remainder.checked_mul(2)?;
    let direction = if (numerator < 0) == (denominator < 0) { 1 } else { -1 };
    if twice_remainder < abs_denominator {
        Some(quotient)
    } else if twice_remainder > abs_denominator {
        quotient.checked_add(direction)
    } else if quotient & 1 == 0 {
        Some(quotient)
    } else {
        quotient.checked_add(direction)
    }
}

#[inline]
fn clamp_i128_to_i64(x: i128) -> i64 {
    if x > (i64::MAX as i128) {
        i64::MAX
    } else if x < (i64::MIN as i128) {
        i64::MIN
    } else {
        x as i64
    }
}

fn parse_decimal_raw_checked(input: &str) -> Option<(i64, bool)> {
    let text = input.trim();
    if text.is_empty() {
        return None;
    }
    let (negative, unsigned) = if let Some(rest) = text.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = text.strip_prefix('+') {
        (false, rest)
    } else {
        (false, text)
    };
    if unsigned.is_empty() {
        return None;
    }

    let mut exponent_index = None;
    for (index, ch) in unsigned.char_indices() {
        if matches!(ch, 'e' | 'E') {
            if exponent_index.is_some() {
                return None;
            }
            exponent_index = Some(index);
        }
    }
    let (mantissa, exponent) = if let Some(index) = exponent_index {
        let exponent_text = &unsigned[index + 1..];
        if exponent_text.is_empty() {
            return None;
        }
        (&unsigned[..index], exponent_text.parse::<i64>().ok()?)
    } else {
        (unsigned, 0)
    };

    let mut decimal_index = None;
    for (index, ch) in mantissa.char_indices() {
        if ch == '.' {
            if decimal_index.is_some() {
                return None;
            }
            decimal_index = Some(index);
        } else if !ch.is_ascii_digit() {
            return None;
        }
    }
    let integer_digit_count = decimal_index.unwrap_or(mantissa.len());
    let mut digits = mantissa
        .bytes()
        .filter(|byte| *byte != b'.')
        .collect::<Vec<_>>();
    if digits.is_empty() || digits.iter().any(|byte| !byte.is_ascii_digit()) {
        return None;
    }

    let leading_zero_count = digits.iter().take_while(|byte| **byte == b'0').count();
    if leading_zero_count == digits.len() {
        return Some((0, false));
    }
    digits.drain(..leading_zero_count);
    let decimal_point = (integer_digit_count as i128)
        .saturating_add(exponent as i128)
        .saturating_sub(leading_zero_count as i128);
    if decimal_point >= 11 {
        return Some((if negative { i64::MIN } else { i64::MAX }, true));
    }
    if decimal_point < -9 {
        return Some((0, false));
    }

    let mut integer_digits = Vec::new();
    let mut fractional_digits = Vec::new();
    if decimal_point <= 0 {
        fractional_digits.resize((-decimal_point) as usize, b'0');
        fractional_digits.extend_from_slice(&digits);
    } else {
        let point = decimal_point as usize;
        if point >= digits.len() {
            integer_digits.extend_from_slice(&digits);
            integer_digits.resize(point, b'0');
        } else {
            // F5-GA[DECIMAL-SPLIT]: 양수 point<digits.len인 이 분기에서만 같은 바이트 Vec을 앞뒤로 나눈다.
            integer_digits.extend_from_slice(&digits[..point]);
            fractional_digits.extend_from_slice(&digits[point..]);
        }
    }

    let mut integer_value = 0i128;
    for digit in integer_digits {
        integer_value = integer_value
            .saturating_mul(10)
            .saturating_add((digit - b'0') as i128);
    }
    let mut fractional_raw = 0u64;
    for _ in 0..Fixed64::FRAC_BITS {
        let mut carry = 0u8;
        for digit in fractional_digits.iter_mut().rev() {
            let product = (*digit - b'0') * 2 + carry;
            *digit = b'0' + product % 10;
            carry = product / 10;
        }
        fractional_raw = (fractional_raw << 1) | carry as u64;
    }

    // The next bit and residue decide the Q31.32 rounding boundary exactly.
    let mut carry = 0u8;
    for digit in fractional_digits.iter_mut().rev() {
        let product = (*digit - b'0') * 2 + carry;
        *digit = b'0' + product % 10;
        carry = product / 10;
    }
    if carry == 1
        && (fractional_digits.iter().any(|digit| *digit != b'0') || fractional_raw & 1 == 1)
    {
        fractional_raw = fractional_raw.checked_add(1)?;
    }

    let absolute_raw = integer_value
        .saturating_mul(Fixed64::ONE_RAW as i128)
        .saturating_add(fractional_raw as i128);
    let signed_raw = if negative {
        absolute_raw.saturating_neg()
    } else {
        absolute_raw
    };
    let overflowed = signed_raw > i64::MAX as i128 || signed_raw < i64::MIN as i128;
    Some((clamp_i128_to_i64(signed_raw), overflowed))
}

// ---- 연산자 오버로드: 덧/뺄/곱은 포화로 고정, 나눗셈은 try_div로만 ----

impl core::ops::Add for Fixed64 {
    type Output = Fixed64;
    #[inline]
    fn add(self, rhs: Fixed64) -> Fixed64 {
        self.saturating_add(rhs)
    }
}

impl core::ops::Sub for Fixed64 {
    type Output = Fixed64;
    #[inline]
    fn sub(self, rhs: Fixed64) -> Fixed64 {
        self.saturating_sub(rhs)
    }
}

impl core::ops::Mul for Fixed64 {
    type Output = Fixed64;
    #[inline]
    fn mul(self, rhs: Fixed64) -> Fixed64 {
        self.saturating_mul(rhs)
    }
}

impl core::ops::Neg for Fixed64 {
    type Output = Fixed64;
    #[inline]
    fn neg(self) -> Fixed64 {
        Fixed64 {
            raw: self.raw.saturating_neg(),
        }
    }
}

impl core::ops::Div for Fixed64 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.raw == 0 {
            panic!("0으로 나눌 수 없습니다.");
        }
        // 정밀도 유지를 위해 i128로 확장하여 연산
        let res = (self.raw as i128 * (1i128 << 32)) / rhs.raw as i128;
        Self::from_raw_i64(res as i64)
    }
}

// ============================================================================
// 포맷팅
// ============================================================================

impl Fixed64 {
    /// Q32.32 값 정체성·정본화용 정확한 십진 문자열. 화면 반올림과 분리한다.
    pub fn canonical_decimal_text(self) -> String {
        let raw = i128::from(self.raw);
        let sign = if raw < 0 { "-" } else { "" };
        let magnitude = raw.unsigned_abs();
        let whole = magnitude >> Self::FRAC_BITS;
        let fraction = magnitude & ((1u128 << Self::FRAC_BITS) - 1);
        if fraction == 0 { return format!("{sign}{whole}"); }
        // 2^32 분모의 십진 전개는 32자리 안에서 정확히 끝난다.
        let decimal = fraction * 5u128.pow(Self::FRAC_BITS);
        let digits = format!("{decimal:032}");
        format!("{sign}{whole}.{}", digits.trim_end_matches('0'))
    }

    /// 세계 정체성·정본화가 아닌 화면/요약 자원 표시 전용 6자리 문자열.
    pub fn display_rounded_six(self) -> String {
        // WO-09 A: 세계 raw 값은 그대로 두고 기존 6자리 표시만 반올림한다.
        // i64::MIN도 안전하게 표현하도록 절댓값 계산은 넓은 정수에서 한다.
        let magnitude = (self.raw as i128).unsigned_abs();
        let scaled = magnitude * 1_000_000;
        let mut micros = scaled >> Self::FRAC_BITS;
        let remainder = scaled & ((1u128 << Self::FRAC_BITS) - 1);
        let half = 1u128 << (Self::FRAC_BITS - 1);
        if remainder > half || (remainder == half && micros & 1 == 1) {
            micros += 1;
        }
        let sign = if self.raw < 0 && micros != 0 { "-" } else { "" };
        let integer = micros / 1_000_000;
        let fraction = micros % 1_000_000;
        if fraction == 0 {
            format!("{sign}{integer}")
        } else {
            let digits = format!("{fraction:06}");
            format!("{sign}{integer}.{}", digits.trim_end_matches('0'))
        }
    }
}

// 사용자 표시는 6자리 반올림이다. 정체성·정본화는 명시적 정확 formatter를 쓴다.
impl fmt::Display for Fixed64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.display_rounded_six())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signals::{FaultContext, VecSignalSink};

    #[test]
    fn exact_canonical_decimal_and_display_are_separate() {
        for (raw, exact, display) in [
            (-(1i64 << 31), "-0.5", "-0.5"),
            (-1, "-0.00000000023283064365386962890625", "0"),
            (i64::MIN, "-2147483648", "-2147483648"),
            (i64::MAX, "2147483647.99999999976716935634613037109375", "2147483648"),
            (1i64 << 25, "0.0078125", "0.007812"),
            (3i64 << 25, "0.0234375", "0.023438"),
            (-(1i64 << 25), "-0.0078125", "-0.007812"),
            (-(3i64 << 25), "-0.0234375", "-0.023438"),
        ] {
            let value = Fixed64::from_raw_i64(raw);
            assert_eq!(value.canonical_decimal_text(), exact);
            assert_eq!(value.to_string(), display);
            assert_eq!(Fixed64::parse_decimal_exact(exact).unwrap().raw_i64(), raw);
            assert_eq!(value.raw_i64(), raw);
        }
    }

    #[test]
    fn exact_canonical_keys_preserve_adjacent_raw_and_raw_canon_bytes() {
        use crate::platform::{ResourceMapEntry, ResourceValue};
        for raw in [i64::MIN, -4294967297, -1, 0, 1, 4294967296, i64::MAX - 1] {
            let a = Fixed64::from_raw_i64(raw);
            let b = Fixed64::from_raw_i64(raw + 1);
            assert_ne!(a.canonical_decimal_text(), b.canonical_decimal_text());
            let av = ResourceValue::Fixed64(a);
            let bv = ResourceValue::Fixed64(b);
            assert_ne!(av.canon_key(), bv.canon_key());
            let set = ResourceValue::set_from_values(vec![av.clone(), bv.clone()]);
            assert!(matches!(set, ResourceValue::Set(ref items) if items.len() == 2));
            let map = ResourceValue::map_from_entries(vec![
                ResourceMapEntry { key: av.clone(), value: ResourceValue::None },
                ResourceMapEntry { key: bv, value: ResourceValue::None },
            ]);
            assert!(matches!(map, ResourceValue::Map(ref entries) if entries.len() == 2));
            let mut encoded = Vec::new();
            av.encode_canon(&mut encoded);
            let mut expected = vec![0x02];
            expected.extend_from_slice(&raw.to_le_bytes());
            assert_eq!(encoded, expected);
        }
    }

    #[test]
    fn q32_32_raw_is_exact() {
        let one = Fixed64::from_i64(1);
        assert_eq!(one.raw_i64(), 1i64 << 32);

        let three = Fixed64::from_i64(3);
        let two = Fixed64::from_i64(2);

        // 3/2 = 1.5 => raw = 1.5 * 2^32 = 6442450944
        let v = three.try_div(two).unwrap();
        assert_eq!(v.raw_i64(), 6_442_450_944);
    }

    #[test]
    fn saturating_add_sub() {
        let max = Fixed64::MAX;
        let one = Fixed64::ONE;
        assert_eq!((max + one).raw_i64(), i64::MAX);

        let min = Fixed64::MIN;
        assert_eq!((min - one).raw_i64(), i64::MIN);
    }

    #[test]
    fn checked_arithmetic_rejects_overflow_instead_of_saturating() {
        assert_eq!(Fixed64::MAX.checked_add(Fixed64::ONE), Err(ArithmeticFaultKind::Overflow));
        assert_eq!(Fixed64::MIN.checked_sub(Fixed64::ONE), Err(ArithmeticFaultKind::Overflow));
        assert_eq!(Fixed64::MAX.checked_mul(Fixed64::from_i64(2)), Err(ArithmeticFaultKind::Overflow));
        assert_eq!(Fixed64::MAX.try_div(Fixed64::from_raw_i64(1)), Err(ArithmeticFaultKind::Overflow));
    }

    #[test]
    fn checked_mul_and_div_round_ties_to_even() {
        let half_ulp = Fixed64::from_raw_i64(1i64 << 31);
        let one_ulp = Fixed64::from_raw_i64(1);
        let three_ulp = Fixed64::from_raw_i64(3);
        assert_eq!(one_ulp.checked_mul(half_ulp), Ok(Fixed64::ZERO));
        assert_eq!(three_ulp.checked_mul(half_ulp), Ok(Fixed64::from_raw_i64(2)));
        assert_eq!(Fixed64::ONE.try_div(Fixed64::from_i64(2)), Ok(half_ulp));
        assert_eq!(Fixed64::from_raw_i64(3).try_div(Fixed64::from_i64(2)), Ok(Fixed64::from_raw_i64(2)));
    }

    #[test]
    fn checked_weighted_blend_preserves_exact_endpoint_and_ties_to_even() {
        let half = Fixed64::from_raw_i64(1i64 << 31);
        assert_eq!(
            Fixed64::MAX.checked_weighted_blend(Fixed64::MAX, half),
            Ok(Fixed64::MAX)
        );
        assert_eq!(
            Fixed64::from_raw_i64(1).checked_weighted_blend(Fixed64::from_raw_i64(3), half),
            Ok(Fixed64::from_raw_i64(2))
        );
        assert_eq!(
            Fixed64::from_raw_i64(1).checked_weighted_blend(Fixed64::from_raw_i64(2), half),
            Ok(Fixed64::from_raw_i64(2))
        );
        assert_eq!(
            Fixed64::ONE.checked_weighted_blend(Fixed64::ZERO, Fixed64::from_raw_i64(-1)),
            Err(ArithmeticFaultKind::Overflow)
        );
    }

    #[test]
    fn div_by_zero_assignment_is_invalid_and_emits_fault() {
        let mut sink = VecSignalSink::default();
        let ctx = FaultContext {
            tick_id: 42,
            location: "test:div0",
            source_span: None,
            expr: None,
        };

        let mut x = Fixed64::from_i64(5);
        let before = x;

        x.div_assign_det(Fixed64::ZERO, ctx.clone(), &mut sink);

        // 대입 무효화
        assert_eq!(x, before);

        // 산술고장 신호
        assert_eq!(sink.signals.len(), 1);
        assert_eq!(
            sink.signals[0],
            Signal::ArithmeticFault {
                ctx,
                kind: ArithmeticFaultKind::DivByZero
            }
        );
    }

    #[test]
    fn determinism_is_integer_only() {
        // 이 테스트는 "플랫폼별 부동소수점 차이"를 원천 배제했음을 검증하는 성격:
        // - i128 정수 산술 + 고정 시프트만 사용
        // - 결과 raw가 상수로 귀결되는지 확인
        let a = Fixed64::from_raw_i64(0x0000_0001_0000_0000); // 1.0
        let b = Fixed64::from_raw_i64(0x0000_0000_8000_0000); // 0.5
        let c = a * b; // 0.5
        assert_eq!(c.raw_i64(), 0x0000_0000_8000_0000);
    }

    #[test]
    fn determinism_vector_matches() {
        let results = Fixed64::determinism_vector_v1();
        let expected = Fixed64::DETERMINISM_VECTOR_V1_EXPECTED;
        assert_eq!(results, expected);
    }

    #[test]
    fn test_from_i64() {
        assert_eq!(Fixed64::from_i64(0), Fixed64::ZERO);
        assert_eq!(Fixed64::from_i64(1), Fixed64::ONE);
        assert_eq!(Fixed64::from_i64(-1), Fixed64::NEG_ONE);
        assert_eq!(Fixed64::from_i64(10).to_raw(), 10 << 32);
    }

    #[test]
    fn ratio_conversion_is_integer_only_and_truncates_toward_zero() {
        assert_eq!(Fixed64::from_ratio(1, 2).raw_i64(), 1i64 << 31);
        assert_eq!(Fixed64::from_ratio(-5, 9).raw_i64(), -2_386_092_942);
        assert_eq!(
            Fixed64::from_ratio(27_315, 100).raw_i64(),
            1_173_170_316_902
        );
        assert_eq!(Fixed64::from_ratio(1, 0), Fixed64::MAX);
        assert_eq!(Fixed64::from_ratio(-1, 0), Fixed64::MIN);
        assert_eq!(Fixed64::from_ratio(0, 0), Fixed64::ZERO);
    }

    #[test]
    fn decimal_conversion_is_integer_only_and_exact_at_raw_precision() {
        let cases = [
            ("0.5", 1i64 << 31),
            ("-0.8", -3_435_973_837),
            ("1.25e2", 125i64 << 32),
            ("3.14", 13_486_197_309),
            ("0.00000000023283064365386962890625", 1),
            ("1e-10", 0),
        ];
        for (text, expected_raw) in cases {
            assert_eq!(
                Fixed64::parse_decimal(text).map(Fixed64::raw_i64),
                Some(expected_raw),
                "decimal input: {text}"
            );
        }
        assert_eq!(Fixed64::parse_decimal("2147483648"), None);
        assert_eq!(Fixed64::parse_decimal("-2147483648"), Some(Fixed64::MIN));
        assert_eq!(
            Fixed64::parse_decimal_exact("2147483647"),
            Some(Fixed64::from_i64(2_147_483_647))
        );
        assert_eq!(Fixed64::parse_decimal_exact("2147483648"), None);
        assert_eq!(Fixed64::parse_decimal_exact("-2147483648"), Some(Fixed64::MIN));
        assert_eq!(
            Fixed64::parse_decimal_exact("602214076000000000000000"),
            None
        );
        for invalid in ["", ".", "+", "1.2.3", "1e", "1e2e3", "nan"] {
            assert_eq!(
                Fixed64::parse_decimal(invalid),
                None,
                "invalid input: {invalid}"
            );
        }
    }

    #[test]
    fn test_addition() {
        let a = Fixed64::from_i64(3);
        let b = Fixed64::from_i64(7);
        assert_eq!(a + b, Fixed64::from_i64(10));
    }

    #[test]
    fn test_subtraction() {
        let a = Fixed64::from_i64(10);
        let b = Fixed64::from_i64(3);
        assert_eq!(a - b, Fixed64::from_i64(7));
    }

    #[test]
    fn test_multiplication() {
        let a = Fixed64::from_i64(3);
        let b = Fixed64::from_i64(4);
        assert_eq!(a * b, Fixed64::from_i64(12));
    }

    #[test]
    fn test_division() {
        let a = Fixed64::from_i64(12);
        let b = Fixed64::from_i64(4);
        assert_eq!(a / b, Fixed64::from_i64(3));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Fixed64::from_i64(10)), "10");

        let display_val = format!("{}", Fixed64::parse_decimal("3.14").expect("valid decimal"));
        assert_eq!(display_val, "3.14");
        assert_eq!(Fixed64::parse_decimal("3.14").unwrap().display_rounded_six(), "3.14");
    }

    #[test]
    fn display_signed_decimals_round_without_changing_raw_value() {
        for (input, expected) in [
            ("-0.08", "-0.08"), ("-1.5", "-1.5"),
            ("-2", "-2"), ("0.25", "0.25"),
            ("0.9999999", "1"), ("-0.9999999", "-1"),
        ] {
            let value = Fixed64::parse_decimal(input).unwrap();
            let raw = value.raw_i64();
            assert_eq!(value.display_rounded_six(), expected, "{input}");
            assert_eq!(value.raw_i64(), raw);
        }
        assert_eq!(Fixed64::MIN.display_rounded_six(), "-2147483648");
        assert_eq!(Fixed64::MAX.display_rounded_six(), "2147483648");
        assert_eq!(Fixed64::from_raw_i64(-1).display_rounded_six(), "0");
        // 정확한 반: 1/128 = .0078125 (짝수 아래), 3/128 = .0234375 (짝수 위).
        for (raw, expected) in [
            (1i64 << 25, "0.007812"), (3i64 << 25, "0.023438"),
            (-(1i64 << 25), "-0.007812"), (-(3i64 << 25), "-0.023438"),
        ] {
            assert_eq!(Fixed64::from_raw_i64(raw).display_rounded_six(), expected);
        }
    }
}
