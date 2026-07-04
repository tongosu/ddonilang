# Fixed64 수치 봉투 실측 V1

작성일: 2026-07-06

범위: `tools/teul-cli/src/core/fixed64.rs`의 `Fixed64` Q31.32 구현과 `tools/teul-cli/src/runtime/eval.rs` 제품 경로에서 호출되는 초등함수 구현을 기준으로 실측했다. 코드 수정, golden 갱신, 수리 작업은 하지 않았다.

## 요약

- `Fixed64::SCALE_BITS = 32`, `SCALE = 2^32 = 4,294,967,296`.
- raw 저장소는 `i64`이므로 표현 가능 범위는 raw `[-9223372036854775808, 9223372036854775807]`.
- 값 범위는 `[-2147483648, 2147483647.99999999976716935634613037109375]`.
- 최소 눈금은 `2^-32 = 0.00000000023283064365386962890625`.
- `sqrt`는 `int_sqrt((raw as i128) << 32)` 기반이며, 샘플 실측 최대 절대오차는 `2.3283064365386963e-10`이었다.
- `sin/cos`는 `runtime/detmath.rs`의 32단계 CORDIC 구현이며, 샘플 실측 최대 절대오차는 `sin 3.026642630965526e-09`, `cos 5.869545516290486e-09`였다.
- `exp/log`는 `lang/src/stdlib.rs`, `tools/teul-cli/src/runtime/eval.rs`, `tools/teul-cli/src/runtime/detmath.rs`에서 제품 구현을 찾지 못해 미구현으로 기록한다.

## 측정 표

| 함수/연산 | 정의역 구간 | 측정 방법 | 오차 상한(측정값) | 비고 |
|---|---|---|---:|---|
| 표현 범위 | raw i64 전체 | `SCALE_BITS=32`, `i64::MIN/MAX`에서 Decimal 계산 | 0 | 최소 `-2147483648`, 최대 `2147483647.99999999976716935634613037109375` |
| 최소 눈금 | raw 1 | `1 / 2^32` Decimal 계산 | 0 | `0.00000000023283064365386962890625` |
| `sqrt` | `[0,1)` | `Fixed64::sqrt` 정수 알고리즘 재현, 20002개 결정적 샘플, f64 `sqrt`와 비교 | `2.328195969347746e-10` | 최대 지점 raw `824418972`, 값 `0.19194999989122152` |
| `sqrt` | `[1,100)` | 동일 | `2.3282820116321545e-10` | 최대 지점 raw `16179356552`, 값 `3.767049999907613` |
| `sqrt` | `[100,1e6)` | 동일 | `2.3283064365386963e-10` | 최대 지점 raw `696788800881295`, 값 `162233.78499999992` |
| `sin` | `[-pi/2, pi/2]` | `detmath::sin_cos` CORDIC 재현, 40003개 결정적 샘플, f64 `sin`과 비교 | `3.026642630965526e-09` | 최대 지점 raw `-545456050`, 값 `-0.1269988832063973` |
| `sin` | `[-2pi, 2pi]` | 동일, `wrap_angle` 포함 | `2.8268479224102805e-09` | 최대 지점 raw `-1165798459`, 값 `-0.27143360557965934` |
| `cos` | `[-pi/2, pi/2]` | `detmath::sin_cos` CORDIC 재현, 40003개 결정적 샘플, f64 `cos`와 비교 | `5.5042279345496326e-09` | 최대 지점 raw `-5908601212`, 값 `-1.3757034232839942` |
| `cos` | `[-2pi, 2pi]` | 동일, `wrap_angle` 포함 | `5.869545516290486e-09` | 최대 지점 raw `-7584436495`, 값 `-1.765889230882749` |
| `exp` | - | `eval.rs`/`detmath.rs`/`stdlib.rs` 검색 | - | 미구현 |
| `log` | - | `eval.rs`/`detmath.rs`/`stdlib.rs` 검색 | - | 미구현 |

주의: 위 오차 상한은 하네스 샘플에서의 측정값이다. 형식 검증으로 증명한 전역 수학 상한은 아니다.

## 오버플로/포화 경계 재확인

| 연산 | 결과 raw | 결과 값 | saturation delta | 비고 |
|---|---:|---:|---:|---|
| `from_int(i64::MAX)` | `9223372036854775807` | 최대값으로 포화 | 1 | `checked_mul(SCALE)` overflow |
| `from_int(i64::MIN)` | `-9223372036854775808` | `-2147483648` | 1 | `checked_mul(SCALE)` overflow |
| `from_ratio(1,0)` | `9223372036854775807` | 최대값으로 포화 | 1 | 0 나눗셈 입력은 `i64::MAX` 반환 |
| `MAX_RAW + 1raw` | `9223372036854775807` | 최대값으로 포화 | 1 | `saturating_add` |
| `MIN_RAW - 1raw` | `-9223372036854775808` | `-2147483648` | 1 | `saturating_sub` |
| `MAX_RAW - (-1raw)` | `9223372036854775807` | 최대값으로 포화 | 1 | `saturating_sub` |
| `MIN_RAW + (-1raw)` | `-9223372036854775808` | `-2147483648` | 1 | `saturating_add` |
| `MAX_RAW * 2.0` | `9223372036854775807` | 최대값으로 포화 | 1 | `mul_raw` -> `saturate_i128` |
| `MIN_RAW * 2.0` | `-9223372036854775808` | `-2147483648` | 1 | `mul_raw` -> `saturate_i128` |
| `1.0 / 0raw` | 없음 | 없음 | 0 | `checked_div`는 `None`, 포화 카운트 증가 없음 |

## 근거 파일

- `tools/teul-cli/src/core/fixed64.rs`
- `tools/teul-cli/src/runtime/detmath.rs`
- `tools/teul-cli/src/runtime/eval.rs`
- `lang/src/stdlib.rs`

## 실행한 하네스

아래 Python 하네스를 저장소 루트에서 `@' ... '@ | python -` 형태로 실행했다. 하네스는 Rust 파일에서 상수를 읽어 검산하고, `Fixed64::sqrt`, `detmath::sin_cos`, 포화 연산을 같은 정수 규칙으로 재현한다.

```python
import json, math, re
from pathlib import Path
from decimal import Decimal, getcontext

ROOT = Path('.').resolve()
fixed_src = (ROOT / 'tools/teul-cli/src/core/fixed64.rs').read_text(encoding='utf-8')
detmath_src = (ROOT / 'tools/teul-cli/src/runtime/detmath.rs').read_text(encoding='utf-8')
eval_src = (ROOT / 'tools/teul-cli/src/runtime/eval.rs').read_text(encoding='utf-8')
stdlib_src = (ROOT / 'lang/src/stdlib.rs').read_text(encoding='utf-8')

SCALE_BITS = int(re.search(r'pub const SCALE_BITS: u32 = (\d+);', fixed_src).group(1))
SCALE = 1 << SCALE_BITS
I64_MAX = (1 << 63) - 1
I64_MIN = -(1 << 63)

PI_RAW = int(re.search(r'const PI_RAW: i64 = ([\d_]+);', detmath_src).group(1).replace('_',''))
PI_HALF_RAW = int(re.search(r'const PI_HALF_RAW: i64 = ([\d_]+);', detmath_src).group(1).replace('_',''))
TWO_PI_RAW = int(re.search(r'const TWO_PI_RAW: i64 = ([\d_]+);', detmath_src).group(1).replace('_',''))
K_INV_RAW = int(re.search(r'const K_INV_RAW: i64 = ([\d_]+);', detmath_src).group(1).replace('_',''))
atan_body = re.search(r'const ATAN_TABLE: \[i64; 32\] = \[(.*?)\];', detmath_src, re.S).group(1)
ATAN_TABLE = [int(x.replace('_','')) for x in re.findall(r'\d[\d_]*', atan_body)]

def clamp_i64(v):
    return I64_MAX if v > I64_MAX else I64_MIN if v < I64_MIN else v

def sat_add(a, b):
    return clamp_i64(a + b)

def sat_sub(a, b):
    return clamp_i64(a - b)

def sat_neg(a):
    return I64_MAX if a == I64_MIN else -a

def rust_rem(a, b):
    q = abs(a) // abs(b)
    if (a < 0) != (b < 0):
        q = -q
    return a - q * b

def raw_to_float(raw):
    return raw / SCALE

def fixed_sqrt_raw(raw):
    if raw < 0:
        return None
    return clamp_i64(math.isqrt(raw << SCALE_BITS))

def wrap_angle(raw):
    r = rust_rem(raw, TWO_PI_RAW)
    if r > PI_RAW:
        r -= TWO_PI_RAW
    elif r < -PI_RAW:
        r += TWO_PI_RAW
    return r

def cordic(theta_raw):
    x = K_INV_RAW
    y = 0
    z = theta_raw
    for i, atan_raw in enumerate(ATAN_TABLE):
        shift = min(i, SCALE_BITS)
        x_shift = x >> shift
        y_shift = y >> shift
        if z >= 0:
            x = sat_sub(x, y_shift)
            y = sat_add(y, x_shift)
            z = sat_sub(z, atan_raw)
        else:
            x = sat_add(x, y_shift)
            y = sat_sub(y, x_shift)
            z = sat_add(z, atan_raw)
    return x, y

def sin_cos_raw(angle_raw):
    theta = wrap_angle(angle_raw)
    cos_sign = 1
    if theta > PI_HALF_RAW:
        theta = PI_RAW - theta
        cos_sign = -1
    elif theta < -PI_HALF_RAW:
        theta = -PI_RAW - theta
        cos_sign = -1
    cos_raw, sin_raw = cordic(theta)
    if cos_sign < 0:
        cos_raw = sat_neg(cos_raw)
    return cos_raw, sin_raw

def sample_raw_range(start_real, end_real, n):
    start = int(math.floor(start_real * SCALE))
    end = int(math.floor(end_real * SCALE))
    vals = [start + ((end - start) * i) // (n - 1) for i in range(n)]
    vals.extend([start, end - 1 if end > start else end, 0, 1, SCALE - 1, SCALE, 100*SCALE - 1])
    return sorted(set(v for v in vals if start <= v < end and 0 <= v <= I64_MAX))

def measure_sqrt(label, start_real, end_real, n=20001):
    max_abs = -1.0
    max_raw = None
    for raw in sample_raw_range(start_real, end_real, n):
        got = raw_to_float(fixed_sqrt_raw(raw))
        ref = math.sqrt(raw_to_float(raw))
        err = abs(got - ref)
        if err > max_abs:
            max_abs = err
            max_raw = raw
    return label, max_abs, max_raw, raw_to_float(max_raw)

def measure_trig(fn_name, start_real, end_real, n=40001):
    max_abs = -1.0
    max_raw = None
    start = int(math.floor(start_real * SCALE))
    end = int(math.floor(end_real * SCALE))
    vals = [start + ((end - start) * i) // (n - 1) for i in range(n)]
    vals.extend([0, PI_HALF_RAW, -PI_HALF_RAW, PI_RAW, -PI_RAW, TWO_PI_RAW, -TWO_PI_RAW])
    for raw in sorted(set(v for v in vals if start <= v <= end)):
        cos_raw, sin_raw = sin_cos_raw(raw)
        got = raw_to_float(sin_raw if fn_name == 'sin' else cos_raw)
        ref = math.sin(raw_to_float(raw)) if fn_name == 'sin' else math.cos(raw_to_float(raw))
        err = abs(got - ref)
        if err > max_abs:
            max_abs = err
            max_raw = raw
    return fn_name, start_real, end_real, max_abs, max_raw, raw_to_float(max_raw)

sat_count = 0

def saturate_i128(v):
    global sat_count
    if v > I64_MAX:
        sat_count += 1
        return I64_MAX
    if v < I64_MIN:
        sat_count += 1
        return I64_MIN
    return v

def from_int(v):
    global sat_count
    raw = v * SCALE
    if raw < I64_MIN or raw > I64_MAX:
        sat_count += 1
        return I64_MIN if v < 0 else I64_MAX
    return raw

def from_ratio(num, den):
    global sat_count
    if den == 0:
        sat_count += 1
        return I64_MAX
    return saturate_i128((num << SCALE_BITS) // den)

def add(a,b):
    global sat_count
    v = a + b
    if v < I64_MIN or v > I64_MAX:
        sat_count += 1
        return I64_MIN if a < 0 else I64_MAX
    return v

def sub(a,b):
    global sat_count
    v = a - b
    if v < I64_MIN or v > I64_MAX:
        sat_count += 1
        return I64_MIN if a < 0 else I64_MAX
    return v

def mul(a,b):
    return saturate_i128((a * b) >> SCALE_BITS)

def div(a,b):
    if b == 0:
        return None
    return saturate_i128((a << SCALE_BITS) // b)

getcontext().prec = 80
print('range.max', Decimal(I64_MAX) / Decimal(SCALE))
print('range.min', Decimal(I64_MIN) / Decimal(SCALE))
print('quantum', Decimal(1) / Decimal(SCALE))
print('sqrt', measure_sqrt('[0,1)', 0, 1))
print('sqrt', measure_sqrt('[1,100)', 1, 100))
print('sqrt', measure_sqrt('[100,1e6)', 100, 1_000_000))
print('trig', measure_trig('sin', -math.pi/2, math.pi/2))
print('trig', measure_trig('sin', -2*math.pi, 2*math.pi))
print('trig', measure_trig('cos', -math.pi/2, math.pi/2))
print('trig', measure_trig('cos', -2*math.pi, 2*math.pi))
print('implemented', {
    'sqrt': '"sqrt" =>' in eval_src,
    'sin': '"sin" =>' in eval_src and 'pub fn sin' in detmath_src,
    'cos': '"cos" =>' in eval_src and 'pub fn cos' in detmath_src,
    'exp': '"exp" =>' in eval_src or 'pub fn exp' in detmath_src,
    'log': '"log" =>' in eval_src or 'pub fn log' in detmath_src,
})
```

## 검증

- `cargo test --manifest-path tools/teul-cli/Cargo.toml` PASS
