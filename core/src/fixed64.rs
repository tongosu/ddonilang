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
        let q = num / (rhs.raw as i128);
        Ok(Self {
            raw: clamp_i128_to_i64(q),
        })
    }

    /// SSOT: 0으로 나누기 발생 시 "대입 무효화" + "산술고장 신호"
    /// - rhs==0 이면 self는 그대로 유지
    /// - sink로 산술고장 emit
    #[inline]
    pub fn div_assign_det(
        &mut self,
        rhs: Self,
        ctx: FaultContext,
        sink: &mut dyn SignalSink,
    ) {
        if rhs.raw == 0 {
            sink.emit(Signal::ArithmeticFault {
                ctx,
                kind: ArithmeticFaultKind::DivByZero,
            });
            // 대입 무효화: self 변경 없음
            return;
        }
        // 정상: self <- self / rhs (포화 포함)
        let num = (self.raw as i128) << (Self::FRAC_BITS as i128);
        let q = num / (rhs.raw as i128);
        self.raw = clamp_i128_to_i64(q);
    }

    // 추가해야 할 상수
    pub const SCALE_F64: f64 = 4294967296.0; // FIXED64_LINT_ALLOW

    // 추가해야 할 메서드
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

    pub fn from_f64_lossy(value: f64) -> Self { // FIXED64_LINT_ALLOW
        Self::from_raw_i64((value * Self::SCALE_F64) as i64)
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
        Fixed64 { raw: self.raw.saturating_neg() }
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

impl fmt::Display for Fixed64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let int_part = self.int_part();
        let frac_part = self.frac_part();
        
        if frac_part == 0 {
            // 정수만 있는 경우
            write!(f, "{}", int_part)
        } else {
            // 1. 소수점 아래 6자리 정밀도로 계산 (반올림 보정 포함)
            // frac_part / 2^32 * 1,000,000
            let frac_decimal = ((frac_part as u128 * 1_000_000) >> 32) as u32;
            
            // 2. 임시 문자열 생성 후 0 제거 (한 번만 출력하기 위함)
            let s = format!("{}.{:06}", int_part, frac_decimal);
            let trimmed = s.trim_end_matches('0').trim_end_matches('.');
            
            write!(f, "{}", trimmed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signals::{FaultContext, VecSignalSink};

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
            Signal::ArithmeticFault { ctx, kind: ArithmeticFaultKind::DivByZero }
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
        
        let display_val = format!("{}", Fixed64::from_f64_lossy(3.14)); // FIXED64_LINT_ALLOW
        
        // 이진법 오차로 인해 3.14 또는 3.139999로 출력될 수 있습니다.
        assert!(
            display_val == "3.14" || display_val == "3.139999",
            "실제 출력값: '{}' (예상값과 다름)", display_val
        );
    }
}
