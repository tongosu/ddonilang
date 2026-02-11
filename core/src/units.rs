use crate::fixed64::Fixed64;
use std::collections::HashSet;
use std::sync::OnceLock;

static UNIT_SYMBOLS: OnceLock<HashSet<String>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnitDim {
    length: i8,
    time: i8,
    mass: i8,
    angle: i8,
    pixel: i8,
    krw: i8,
    usd: i8,
}

impl UnitDim {
    pub const NONE: UnitDim = UnitDim {
        length: 0,
        time: 0,
        mass: 0,
        angle: 0,
        pixel: 0,
        krw: 0,
        usd: 0,
    };
    pub const LENGTH: UnitDim = UnitDim {
        length: 1,
        time: 0,
        mass: 0,
        angle: 0,
        pixel: 0,
        krw: 0,
        usd: 0,
    };
    pub const AREA: UnitDim = UnitDim {
        length: 2,
        time: 0,
        mass: 0,
        angle: 0,
        pixel: 0,
        krw: 0,
        usd: 0,
    };
    pub const TIME: UnitDim = UnitDim {
        length: 0,
        time: 1,
        mass: 0,
        angle: 0,
        pixel: 0,
        krw: 0,
        usd: 0,
    };
    pub const MASS: UnitDim = UnitDim {
        length: 0,
        time: 0,
        mass: 1,
        angle: 0,
        pixel: 0,
        krw: 0,
        usd: 0,
    };
    pub const ANGLE: UnitDim = UnitDim {
        length: 0,
        time: 0,
        mass: 0,
        angle: 1,
        pixel: 0,
        krw: 0,
        usd: 0,
    };
    pub const PIXEL: UnitDim = UnitDim {
        length: 0,
        time: 0,
        mass: 0,
        angle: 0,
        pixel: 1,
        krw: 0,
        usd: 0,
    };
    pub const SPEED: UnitDim = UnitDim {
        length: 1,
        time: -1,
        mass: 0,
        angle: 0,
        pixel: 0,
        krw: 0,
        usd: 0,
    };
    pub const ACCELERATION: UnitDim = UnitDim {
        length: 1,
        time: -2,
        mass: 0,
        angle: 0,
        pixel: 0,
        krw: 0,
        usd: 0,
    };
    pub const FORCE: UnitDim = UnitDim {
        length: 1,
        time: -2,
        mass: 1,
        angle: 0,
        pixel: 0,
        krw: 0,
        usd: 0,
    };
    pub const KRW: UnitDim = UnitDim {
        length: 0,
        time: 0,
        mass: 0,
        angle: 0,
        pixel: 0,
        krw: 1,
        usd: 0,
    };
    pub const USD: UnitDim = UnitDim {
        length: 0,
        time: 0,
        mass: 0,
        angle: 0,
        pixel: 0,
        krw: 0,
        usd: 1,
    };

    pub fn add(self, other: UnitDim) -> UnitDim {
        UnitDim {
            length: self.length + other.length,
            time: self.time + other.time,
            mass: self.mass + other.mass,
            angle: self.angle + other.angle,
            pixel: self.pixel + other.pixel,
            krw: self.krw + other.krw,
            usd: self.usd + other.usd,
        }
    }

    pub fn sub(self, other: UnitDim) -> UnitDim {
        UnitDim {
            length: self.length - other.length,
            time: self.time - other.time,
            mass: self.mass - other.mass,
            angle: self.angle - other.angle,
            pixel: self.pixel - other.pixel,
            krw: self.krw - other.krw,
            usd: self.usd - other.usd,
        }
    }

    pub fn is_dimensionless(self) -> bool {
        self == UnitDim::NONE
    }

    pub fn sqrt(self) -> Option<UnitDim> {
        if self.length % 2 != 0
            || self.time % 2 != 0
            || self.mass % 2 != 0
            || self.angle % 2 != 0
            || self.pixel % 2 != 0
            || self.krw % 2 != 0
            || self.usd % 2 != 0
        {
            return None;
        }
        Some(UnitDim {
            length: self.length / 2,
            time: self.time / 2,
            mass: self.mass / 2,
            angle: self.angle / 2,
            pixel: self.pixel / 2,
            krw: self.krw / 2,
            usd: self.usd / 2,
        })
    }

    pub fn format(self) -> String {
        if self.is_dimensionless() {
            return "1".to_string();
        }

        let mut numerator = Vec::new();
        let mut denominator = Vec::new();
        push_unit(&mut numerator, &mut denominator, "kg", self.mass);
        push_unit(&mut numerator, &mut denominator, "m", self.length);
        push_unit(&mut numerator, &mut denominator, "s", self.time);
        push_unit(&mut numerator, &mut denominator, "rad", self.angle);
        push_unit(&mut numerator, &mut denominator, "px", self.pixel);
        push_unit(&mut numerator, &mut denominator, "KRW", self.krw);
        push_unit(&mut numerator, &mut denominator, "USD", self.usd);

        let num = if numerator.is_empty() {
            "1".to_string()
        } else {
            numerator.join("*")
        };
        if denominator.is_empty() {
            num
        } else {
            format!("{num}/{}", denominator.join("*"))
        }
    }
}

fn push_unit(target: &mut Vec<String>, denom: &mut Vec<String>, symbol: &str, exp: i8) {
    if exp == 0 {
        return;
    }
    let abs = exp.abs();
    let text = if abs == 1 {
        symbol.to_string()
    } else {
        format!("{symbol}^{abs}")
    };
    if exp > 0 {
        target.push(text);
    } else {
        denom.push(text);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitSpec {
    pub symbol: &'static str,
    pub dim: UnitDim,
    pub scale: Fixed64,
}

pub fn unit_spec_from_symbol(symbol: &str) -> Option<UnitSpec> {
    if let Some(symbols) = UNIT_SYMBOLS.get() {
        if !symbols.contains(symbol) {
            return None;
        }
    }
    match symbol {
        "m" => Some(Unit::Meter.spec()),
        "mm" => Some(Unit::Millimeter.spec()),
        "cm" => Some(Unit::Centimeter.spec()),
        "km" => Some(Unit::Kilometer.spec()),
        "inch" => Some(Unit::Inch.spec()),
        "ft" => Some(Unit::Foot.spec()),
        "\u{D3C9}" => Some(Unit::Pyeong.spec()),
        "s" | "\u{CD08}" => Some(Unit::Second.spec()),
        "us" => Some(Unit::Microsecond.spec()),
        "ms" => Some(Unit::Millisecond.spec()),
        "min" => Some(Unit::Minute.spec()),
        "h" => Some(Unit::Hour.spec()),
        "kg" => Some(Unit::Kilogram.spec()),
        "g" => Some(Unit::Gram.spec()),
        "rad" => Some(Unit::Radian.spec()),
        "px" => Some(Unit::Pixel.spec()),
        "m/s" => Some(Unit::MeterPerSecond.spec()),
        "mps" => Some(Unit::MeterPerSecond.spec()),
        "m/s^2" => Some(Unit::MeterPerSecondSquared.spec()),
        "kmh" => Some(Unit::KilometerPerHour.spec()),
        "N" => Some(Unit::Newton.spec()),
        "KRW" => Some(Unit::Krw.spec()),
        "USD" => Some(Unit::Usd.spec()),
        _ => None,
    }
}

pub fn set_unit_registry_symbols(symbols: HashSet<String>) -> Result<(), String> {
    if UNIT_SYMBOLS.get().is_some() {
        return Ok(());
    }
    UNIT_SYMBOLS
        .set(symbols)
        .map_err(|_| "단위 곳간은 한 번만 초기화할 수 있습니다".to_string())
}


pub fn is_known_unit(symbol: &str) -> bool {
    unit_spec_from_symbol(symbol).is_some()
}

pub fn canonical_unit_symbol(symbol: &str) -> Option<&'static str> {
    unit_spec_from_symbol(symbol).map(|spec| spec.symbol)
}

pub fn base_unit_symbol_for_dim(dim: UnitDim) -> Option<&'static str> {
    if dim == UnitDim::NONE {
        return None;
    }
    if dim == UnitDim::LENGTH {
        return Some("m");
    }
    if dim == UnitDim::AREA {
        return Some("m^2");
    }
    if dim == UnitDim::TIME {
        return Some("s");
    }
    if dim == UnitDim::MASS {
        return Some("kg");
    }
    if dim == UnitDim::ANGLE {
        return Some("rad");
    }
    if dim == UnitDim::PIXEL {
        return Some("px");
    }
    if dim == UnitDim::SPEED {
        return Some("m/s");
    }
    if dim == UnitDim::ACCELERATION {
        return Some("m/s^2");
    }
    if dim == UnitDim::FORCE {
        return Some("N");
    }
    if dim == UnitDim::KRW {
        return Some("KRW");
    }
    if dim == UnitDim::USD {
        return Some("USD");
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Meter,
    Millimeter,
    Centimeter,
    Kilometer,
    Inch,
    Foot,
    Pyeong,
    Second,
    Microsecond,
    Millisecond,
    Minute,
    Hour,
    Kilogram,
    Gram,
    Radian,
    Pixel,
    MeterPerSecond,
    MeterPerSecondSquared,
    KilometerPerHour,
    Newton,
    Krw,
    Usd,
}

impl Unit {
    pub fn spec(self) -> UnitSpec {
        match self {
            Unit::Meter => UnitSpec {
                symbol: "m",
                dim: UnitDim::LENGTH,
                scale: Fixed64::ONE,
            },
            Unit::Millimeter => UnitSpec {
                symbol: "mm",
                dim: UnitDim::LENGTH,
                scale: Fixed64::from_raw_i64((Fixed64::ONE_RAW + 500) / 1000),
            },
            Unit::Centimeter => UnitSpec {
                symbol: "cm",
                dim: UnitDim::LENGTH,
                scale: Fixed64::from_raw_i64((Fixed64::ONE_RAW + 50) / 100),
            },
            Unit::Kilometer => UnitSpec {
                symbol: "km",
                dim: UnitDim::LENGTH,
                scale: Fixed64::from_i64(1000),
            },
            Unit::Inch => UnitSpec {
                symbol: "inch",
                dim: UnitDim::LENGTH,
                scale: Fixed64::from_raw_i64((Fixed64::ONE_RAW * 254 + 5000) / 10000),
            },
            Unit::Foot => UnitSpec {
                symbol: "ft",
                dim: UnitDim::LENGTH,
                scale: Fixed64::from_raw_i64((Fixed64::ONE_RAW * 3048 + 5000) / 10000),
            },
            Unit::Pyeong => UnitSpec {
                symbol: "\u{D3C9}",
                dim: UnitDim::AREA,
                scale: Fixed64::from_raw_i64((Fixed64::ONE_RAW * 3305785 + 500000) / 1000000),
            },
            Unit::Second => UnitSpec {
                symbol: "s",
                dim: UnitDim::TIME,
                scale: Fixed64::ONE,
            },
            Unit::Microsecond => UnitSpec {
                symbol: "us",
                dim: UnitDim::TIME,
                scale: Fixed64::from_raw_i64((Fixed64::ONE_RAW + 500_000) / 1_000_000),
            },
            Unit::Millisecond => UnitSpec {
                symbol: "ms",
                dim: UnitDim::TIME,
                scale: Fixed64::from_raw_i64((Fixed64::ONE_RAW + 500) / 1000),
            },
            Unit::Minute => UnitSpec {
                symbol: "min",
                dim: UnitDim::TIME,
                scale: Fixed64::from_i64(60),
            },
            Unit::Hour => UnitSpec {
                symbol: "h",
                dim: UnitDim::TIME,
                scale: Fixed64::from_i64(3600),
            },
            Unit::Kilogram => UnitSpec {
                symbol: "kg",
                dim: UnitDim::MASS,
                scale: Fixed64::ONE,
            },
            Unit::Gram => UnitSpec {
                symbol: "g",
                dim: UnitDim::MASS,
                scale: Fixed64::from_raw_i64((Fixed64::ONE_RAW + 500) / 1000),
            },
            Unit::Radian => UnitSpec {
                symbol: "rad",
                dim: UnitDim::ANGLE,
                scale: Fixed64::ONE,
            },
            Unit::Pixel => UnitSpec {
                symbol: "px",
                dim: UnitDim::PIXEL,
                scale: Fixed64::ONE,
            },
            Unit::MeterPerSecond => UnitSpec {
                symbol: "m/s",
                dim: UnitDim::SPEED,
                scale: Fixed64::ONE,
            },
            Unit::MeterPerSecondSquared => UnitSpec {
                symbol: "m/s^2",
                dim: UnitDim::ACCELERATION,
                scale: Fixed64::ONE,
            },
            Unit::KilometerPerHour => UnitSpec {
                symbol: "kmh",
                dim: UnitDim::SPEED,
                scale: Fixed64::from_raw_i64((Fixed64::ONE_RAW * 5 + 9) / 18),
            },
            Unit::Newton => UnitSpec {
                symbol: "N",
                dim: UnitDim::FORCE,
                scale: Fixed64::ONE,
            },
            Unit::Krw => UnitSpec {
                symbol: "KRW",
                dim: UnitDim::KRW,
                scale: Fixed64::ONE,
            },
            Unit::Usd => UnitSpec {
                symbol: "USD",
                dim: UnitDim::USD,
                scale: Fixed64::ONE,
            },
        }
    }

    pub fn dim(self) -> UnitDim {
        self.spec().dim
    }

    pub fn symbol(self) -> &'static str {
        self.spec().symbol
    }

    pub fn scale_to_base(self) -> Fixed64 {
        self.spec().scale
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitError {
    DimensionMismatch { left: UnitDim, right: UnitDim },
    DivisionByZero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitValue {
    pub value: Fixed64,
    pub dim: UnitDim,
}

impl UnitValue {
    pub fn new(value: Fixed64, unit: Unit) -> Self {
        Self::from_spec(value, unit.spec())
    }

    pub fn from_spec(value: Fixed64, spec: UnitSpec) -> Self {
        Self {
            value: value * spec.scale,
            dim: spec.dim,
        }
    }

    pub fn is_dimensionless(self) -> bool {
        self.dim.is_dimensionless()
    }

    pub fn add(self, other: UnitValue) -> Result<UnitValue, UnitError> {
        if self.dim != other.dim {
            return Err(UnitError::DimensionMismatch {
                left: self.dim,
                right: other.dim,
            });
        }
        Ok(UnitValue {
            value: self.value + other.value,
            dim: self.dim,
        })
    }

    pub fn sub(self, other: UnitValue) -> Result<UnitValue, UnitError> {
        if self.dim != other.dim {
            return Err(UnitError::DimensionMismatch {
                left: self.dim,
                right: other.dim,
            });
        }
        Ok(UnitValue {
            value: self.value - other.value,
            dim: self.dim,
        })
    }

    pub fn mul(self, other: UnitValue) -> UnitValue {
        UnitValue {
            value: self.value * other.value,
            dim: self.dim.add(other.dim),
        }
    }

    pub fn div(self, other: UnitValue) -> Result<UnitValue, UnitError> {
        let value = self
            .value
            .try_div(other.value)
            .map_err(|_| UnitError::DivisionByZero)?;
        Ok(UnitValue {
            value,
            dim: self.dim.sub(other.dim),
        })
    }

    pub fn mul_scalar(self, scalar: Fixed64) -> UnitValue {
        UnitValue {
            value: self.value * scalar,
            dim: self.dim,
        }
    }

    pub fn div_scalar(self, scalar: Fixed64) -> Result<UnitValue, UnitError> {
        let value = self
            .value
            .try_div(scalar)
            .map_err(|_| UnitError::DivisionByZero)?;
        Ok(UnitValue {
            value,
            dim: self.dim,
        })
    }

    pub fn to_unit(self, spec: UnitSpec) -> Result<Fixed64, UnitError> {
        if self.dim != spec.dim {
            return Err(UnitError::DimensionMismatch {
                left: self.dim,
                right: spec.dim,
            });
        }
        self.value
            .try_div(spec.scale)
            .map_err(|_| UnitError::DivisionByZero)
    }

    pub fn display_symbol(self) -> Option<&'static str> {
        base_unit_symbol_for_dim(self.dim)
    }
}

pub fn resource_tag_with_unit(name: &str, unit: Unit) -> String {
    format!("{name}@{}", unit.symbol())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_add_cm_and_m_is_ok() {
        let a = UnitValue::new(Fixed64::from_i64(100), Unit::Centimeter);
        let b = UnitValue::new(Fixed64::from_i64(1), Unit::Meter);
        let sum = a.add(b).expect("unit add");
        assert_eq!(sum.dim, UnitDim::LENGTH);
        let expected = Fixed64::from_i64(2);
        let diff = (sum.value.raw_i64() - expected.raw_i64()).abs();
        assert!(diff <= 200, "raw diff too large: {diff}");
    }

    #[test]
    fn unit_add_length_and_time_is_error() {
        let a = UnitValue::new(Fixed64::from_i64(1), Unit::Meter);
        let b = UnitValue::new(Fixed64::from_i64(1), Unit::Second);
        let err = a.add(b).expect_err("dimension mismatch");
        assert_eq!(
            err,
            UnitError::DimensionMismatch {
                left: UnitDim::LENGTH,
                right: UnitDim::TIME
            }
        );
    }

    #[test]
    fn unit_division_builds_speed_dim() {
        let distance = UnitValue::new(Fixed64::from_i64(10), Unit::Meter);
        let time = UnitValue::new(Fixed64::from_i64(2), Unit::Second);
        let speed = distance.div(time).expect("div");
        assert_eq!(speed.dim, UnitDim::SPEED);
    }

    #[test]
    fn unit_kmh_scales_to_mps() {
        let speed = UnitValue::new(Fixed64::from_i64(36), Unit::KilometerPerHour);
        let expected = Fixed64::from_i64(10);
        let diff = (speed.value.raw_i64() - expected.raw_i64()).abs();
        assert!(diff <= 200, "raw diff too large: {diff}");
    }

    #[test]
    fn unit_currency_mismatch_is_error() {
        let a = UnitValue::new(Fixed64::from_i64(1), Unit::Krw);
        let b = UnitValue::new(Fixed64::from_i64(1), Unit::Usd);
        let err = a.add(b).expect_err("dimension mismatch");
        assert_eq!(
            err,
            UnitError::DimensionMismatch {
                left: UnitDim::KRW,
                right: UnitDim::USD
            }
        );
    }

    #[test]
    fn resource_tag_includes_unit_suffix() {
        let tag = resource_tag_with_unit("speed", Unit::Meter);
        assert_eq!(tag, "speed@m");
    }
}
