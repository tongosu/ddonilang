use ddonirang_core::Fixed64;

/// A generic JSON number can already have been represented as binary floating
/// point by its parser.  Fixed64 carriers therefore accept decimal text or a
/// raw integer object, never a JSON number value.
pub(crate) fn reject_binary_json_number(_number: &serde_json::Number) -> Option<Fixed64> {
    None
}

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
    use serde_json::Value as JsonValue;

    #[test]
    fn boundary_preserves_legacy_raw_conversion() {
        assert_eq!(Fixed64::from_f64_lossy(3.14).raw_i64(), 13_486_197_309);
        assert_eq!(Fixed64::from_f64_lossy(-0.8).raw_i64(), -3_435_973_836);
        assert_eq!(Fixed64::from_f64_lossy(f64::NAN), Fixed64::ZERO);
        assert_eq!(Fixed64::from_f64_lossy(f64::INFINITY), Fixed64::MAX);
        assert_eq!(Fixed64::from_f64_lossy(f64::NEG_INFINITY), Fixed64::MIN);
    }

    #[test]
    fn deterministic_json_numbers_are_rejected_before_the_fixed64_textual_boundary() {
        let half_ulp: JsonValue = serde_json::from_str("0.000000000116415321826934814453125")
            .expect("half ULP JSON number");
        let three_halves_ulp: JsonValue =
            serde_json::from_str("0.000000000349245965480804443359375")
                .expect("three halves ULP JSON number");
        let overflow: JsonValue = serde_json::from_str("2147483648")
            .expect("out-of-range JSON number");

        for number in [&half_ulp, &three_halves_ulp, &overflow] {
            assert_eq!(
                reject_binary_json_number(number.as_number().expect("number")),
                None,
                "binary JSON numbers must not become Fixed64 authority"
            );
        }
        assert_eq!(
            Fixed64::parse_decimal("0.000000000116415321826934814453125"),
            Some(Fixed64::ZERO),
            "the exact textual path retains ties-to-even"
        );
        assert_eq!(
            Fixed64::parse_decimal("0.000000000349245965480804443359375"),
            Some(Fixed64::from_raw_i64(2)),
            "the exact textual path retains the next even raw value"
        );
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn native_and_wasm_carriers_share_the_strict_fixed64_boundary() {
        let binary_number: JsonValue = serde_json::from_str("0.5")
            .expect("binary JSON number");
        let exact_text = serde_json::json!("0.000000000349245965480804443359375");

        assert_eq!(crate::unit_wire::json_to_fixed64(&binary_number), None);
        assert_eq!(crate::wasm_api::json_to_fixed64(&binary_number), None);
        assert_eq!(
            crate::unit_wire::json_to_fixed64(&exact_text),
            Some(Fixed64::from_raw_i64(2))
        );
        assert_eq!(
            crate::wasm_api::json_to_fixed64(&exact_text),
            Some(Fixed64::from_raw_i64(2))
        );
    }
}
