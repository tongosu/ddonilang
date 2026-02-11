use crate::core::fixed64::Fixed64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitExpr {
    pub factors: Vec<UnitFactor>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitFactor {
    pub name: String,
    pub exp: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UnitDim {
    pub length: i32,
    pub time: i32,
    pub mass: i32,
    pub angle: i32,
    pub pixel: i32,
}

impl UnitDim {
    pub fn zero() -> Self {
        Self {
            length: 0,
            time: 0,
            mass: 0,
            angle: 0,
            pixel: 0,
        }
    }

    pub fn is_dimensionless(self) -> bool {
        self.length == 0
            && self.time == 0
            && self.mass == 0
            && self.angle == 0
            && self.pixel == 0
    }

    pub fn add(self, other: Self) -> Self {
        Self {
            length: self.length + other.length,
            time: self.time + other.time,
            mass: self.mass + other.mass,
            angle: self.angle + other.angle,
            pixel: self.pixel + other.pixel,
        }
    }

    pub fn scale(self, factor: i32) -> Self {
        Self {
            length: self.length * factor,
            time: self.time * factor,
            mass: self.mass * factor,
            angle: self.angle * factor,
            pixel: self.pixel * factor,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct UnitDef {
    dim: UnitDim,
    scale: UnitScale,
}

#[derive(Clone, Copy, Debug)]
pub struct UnitScale {
    num: i128,
    den: i128,
}

impl UnitScale {
    pub fn one() -> Self {
        Self { num: 1, den: 1 }
    }

    pub fn from_int(value: i64) -> Self {
        Self {
            num: value as i128,
            den: 1,
        }
    }

    pub fn from_ratio(num: i64, den: i64) -> Self {
        let den = if den == 0 { 1 } else { den };
        Self {
            num: num as i128,
            den: den as i128,
        }
    }

    pub fn mul(self, other: Self) -> Self {
        Self {
            num: self.num.saturating_mul(other.num),
            den: self.den.saturating_mul(other.den),
        }
    }

    pub fn div(self, other: Self) -> Self {
        Self {
            num: self.num.saturating_mul(other.den),
            den: self.den.saturating_mul(other.num),
        }
    }

    pub fn powi(self, exp: i32) -> Self {
        if exp == 0 {
            return Self::one();
        }
        let mut acc = Self::one();
        for _ in 0..exp.abs() {
            acc = acc.mul(self);
        }
        if exp < 0 {
            acc = Self {
                num: acc.den,
                den: acc.num,
            };
        }
        acc
    }

    pub fn apply(self, value: Fixed64) -> Fixed64 {
        if self.den == 0 {
            return Fixed64::from_raw(i64::MAX);
        }
        let raw = value.raw() as i128;
        let scaled = raw.saturating_mul(self.num) / self.den;
        Fixed64::from_raw(saturate_i128(scaled))
    }

    pub fn apply_inverse(self, value: Fixed64) -> Fixed64 {
        if self.num == 0 {
            return Fixed64::from_raw(i64::MAX);
        }
        let raw = value.raw() as i128;
        let scaled = raw.saturating_mul(self.den) / self.num;
        Fixed64::from_raw(saturate_i128(scaled))
    }
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

#[derive(Debug)]
pub enum UnitError {
    Unknown(String),
    Overflow,
}

pub fn eval_unit_expr(expr: &UnitExpr) -> Result<(UnitDim, UnitScale), UnitError> {
    let mut dim = UnitDim::zero();
    let mut scale = UnitScale::one();

    for factor in &expr.factors {
        let def = unit_def(&factor.name).ok_or_else(|| UnitError::Unknown(factor.name.clone()))?;
        let exp = factor.exp;
        if exp == 0 {
            continue;
        }
        let dim_part = def.dim.scale(exp);
        dim = dim.add(dim_part);
        let pow = def.scale.powi(exp.abs());
        if exp > 0 {
            scale = scale.mul(pow);
        } else {
            if pow.num == 0 {
                return Err(UnitError::Overflow);
            }
            scale = scale.div(pow);
        }
    }

    Ok((dim, scale))
}

pub fn format_dim(dim: UnitDim) -> String {
    if dim.is_dimensionless() {
        return "".to_string();
    }

    let mut numer = Vec::new();
    let mut denom = Vec::new();
    push_dim(&mut numer, &mut denom, "m", dim.length);
    push_dim(&mut numer, &mut denom, "s", dim.time);
    push_dim(&mut numer, &mut denom, "kg", dim.mass);
    push_dim(&mut numer, &mut denom, "rad", dim.angle);
    push_dim(&mut numer, &mut denom, "px", dim.pixel);

    let numer_str = if numer.is_empty() {
        "1".to_string()
    } else {
        numer.join("*")
    };

    if denom.is_empty() {
        numer_str
    } else {
        format!("{}/{}", numer_str, denom.join("*"))
    }
}

fn push_dim(numer: &mut Vec<String>, denom: &mut Vec<String>, name: &str, exp: i32) {
    if exp == 0 {
        return;
    }
    let target = if exp > 0 { numer } else { denom };
    let power = exp.abs();
    if power == 1 {
        target.push(name.to_string());
    } else {
        target.push(format!("{}^{}", name, power));
    }
}

fn unit_def(name: &str) -> Option<UnitDef> {
    match name {
        "m" => Some(UnitDef {
            dim: UnitDim {
                length: 1,
                time: 0,
                mass: 0,
                angle: 0,
                pixel: 0,
            },
            scale: UnitScale::one(),
        }),
        "mm" => Some(UnitDef {
            dim: UnitDim {
                length: 1,
                time: 0,
                mass: 0,
                angle: 0,
                pixel: 0,
            },
            scale: UnitScale::from_ratio(1, 1000),
        }),
        "cm" => Some(UnitDef {
            dim: UnitDim {
                length: 1,
                time: 0,
                mass: 0,
                angle: 0,
                pixel: 0,
            },
            scale: UnitScale::from_ratio(1, 100),
        }),
        "km" => Some(UnitDef {
            dim: UnitDim {
                length: 1,
                time: 0,
                mass: 0,
                angle: 0,
                pixel: 0,
            },
            scale: UnitScale::from_int(1000),
        }),
        "s" => Some(UnitDef {
            dim: UnitDim {
                length: 0,
                time: 1,
                mass: 0,
                angle: 0,
                pixel: 0,
            },
            scale: UnitScale::one(),
        }),
        "us" => Some(UnitDef {
            dim: UnitDim {
                length: 0,
                time: 1,
                mass: 0,
                angle: 0,
                pixel: 0,
            },
            scale: UnitScale::from_ratio(1, 1_000_000),
        }),
        "ms" => Some(UnitDef {
            dim: UnitDim {
                length: 0,
                time: 1,
                mass: 0,
                angle: 0,
                pixel: 0,
            },
            scale: UnitScale::from_ratio(1, 1000),
        }),
        "min" => Some(UnitDef {
            dim: UnitDim {
                length: 0,
                time: 1,
                mass: 0,
                angle: 0,
                pixel: 0,
            },
            scale: UnitScale::from_int(60),
        }),
        "h" => Some(UnitDef {
            dim: UnitDim {
                length: 0,
                time: 1,
                mass: 0,
                angle: 0,
                pixel: 0,
            },
            scale: UnitScale::from_int(3600),
        }),
        "kg" => Some(UnitDef {
            dim: UnitDim {
                length: 0,
                time: 0,
                mass: 1,
                angle: 0,
                pixel: 0,
            },
            scale: UnitScale::one(),
        }),
        "g" => Some(UnitDef {
            dim: UnitDim {
                length: 0,
                time: 0,
                mass: 1,
                angle: 0,
                pixel: 0,
            },
            scale: UnitScale::from_ratio(1, 1000),
        }),
        "rad" => Some(UnitDef {
            dim: UnitDim {
                length: 0,
                time: 0,
                mass: 0,
                angle: 1,
                pixel: 0,
            },
            scale: UnitScale::one(),
        }),
        "px" => Some(UnitDef {
            dim: UnitDim {
                length: 0,
                time: 0,
                mass: 0,
                angle: 0,
                pixel: 1,
            },
            scale: UnitScale::one(),
        }),
        "N" => Some(UnitDef {
            dim: UnitDim {
                length: 1,
                time: -2,
                mass: 1,
                angle: 0,
                pixel: 0,
            },
            scale: UnitScale::one(),
        }),
        _ => None,
    }
}
