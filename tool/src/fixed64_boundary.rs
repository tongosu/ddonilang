use ddonirang_core::Fixed64;

/// 제품의 이진 부동소수 입력과 결정 코어 사이의 명시적 변환 경계.
///
/// 기존 공개 동작과 raw 값을 보존하기 위해 Rust의 포화 캐스트 규칙을 그대로 사용한다.
pub(crate) trait Fixed64FloatBoundary {
    const SCALE_F64: f64 = 4_294_967_296.0;

    fn from_f64_lossy(value: f64) -> Self;
}

impl Fixed64FloatBoundary for Fixed64 {
    #[inline]
    fn from_f64_lossy(value: f64) -> Self {
        Self::from_raw_i64((value * Self::SCALE_F64) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_preserves_legacy_raw_conversion() {
        assert_eq!(Fixed64::from_f64_lossy(3.14).raw_i64(), 13_486_197_309);
        assert_eq!(Fixed64::from_f64_lossy(-0.8).raw_i64(), -3_435_973_836);
        assert_eq!(Fixed64::from_f64_lossy(f64::NAN), Fixed64::ZERO);
        assert_eq!(Fixed64::from_f64_lossy(f64::INFINITY), Fixed64::MAX);
        assert_eq!(Fixed64::from_f64_lossy(f64::NEG_INFINITY), Fixed64::MIN);
    }
}
