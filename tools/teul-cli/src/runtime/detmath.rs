use crate::core::fixed64::Fixed64;

const SCALE_BITS: u32 = 32;
const PI_RAW: i64 = 13_493_037_704;
const PI_HALF_RAW: i64 = 6_746_518_852;
const TWO_PI_RAW: i64 = 26_986_075_409;
const K_INV_RAW: i64 = 2_608_131_496;

const ATAN_TABLE: [i64; 32] = [
    3_373_259_426,
    1_991_351_317,
    1_052_175_346,
    534_100_634,
    268_086_747,
    134_174_062,
    67_103_403,
    33_553_749,
    16_777_130,
    8_388_597,
    4_194_302,
    2_097_151,
    1_048_575,
    524_287,
    262_143,
    131_071,
    65_535,
    32_767,
    16_383,
    8_191,
    4_095,
    2_047,
    1_023,
    511,
    255,
    127,
    63,
    32,
    16,
    8,
    4,
    2,
];

pub fn sin(angle: Fixed64) -> Fixed64 {
    let (_, sin_val) = sin_cos(angle);
    sin_val
}

pub fn cos(angle: Fixed64) -> Fixed64 {
    let (cos_val, _) = sin_cos(angle);
    cos_val
}

pub fn sin_cos(angle: Fixed64) -> (Fixed64, Fixed64) {
    let mut theta = wrap_angle(angle.raw());
    let mut cos_sign: i64 = 1;

    if theta > PI_HALF_RAW {
        theta = PI_RAW - theta;
        cos_sign = -1;
    } else if theta < -PI_HALF_RAW {
        theta = -PI_RAW - theta;
        cos_sign = -1;
    }

    let (mut cos_raw, sin_raw) = cordic(theta);
    if cos_sign < 0 {
        cos_raw = cos_raw.saturating_neg();
    }
    (Fixed64::from_raw(cos_raw), Fixed64::from_raw(sin_raw))
}

fn wrap_angle(raw: i64) -> i64 {
    let mut r = raw % TWO_PI_RAW;
    if r > PI_RAW {
        r -= TWO_PI_RAW;
    } else if r < -PI_RAW {
        r += TWO_PI_RAW;
    }
    r
}

fn cordic(theta_raw: i64) -> (i64, i64) {
    let mut x = K_INV_RAW;
    let mut y = 0_i64;
    let mut z = theta_raw;

    for (i, atan_raw) in ATAN_TABLE.iter().enumerate() {
        let shift = i.min(SCALE_BITS as usize);
        let x_shift = x >> shift;
        let y_shift = y >> shift;
        if z >= 0 {
            let x_new = x.saturating_sub(y_shift);
            let y_new = y.saturating_add(x_shift);
            x = x_new;
            y = y_new;
            z = z.saturating_sub(*atan_raw);
        } else {
            let x_new = x.saturating_add(y_shift);
            let y_new = y.saturating_sub(x_shift);
            x = x_new;
            y = y_new;
            z = z.saturating_add(*atan_raw);
        }
    }

    (x, y)
}
