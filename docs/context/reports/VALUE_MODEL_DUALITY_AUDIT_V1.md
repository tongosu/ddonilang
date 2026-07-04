# 값 모델 이중화 감사 V1

작성일: 2026-07-06

범위: `lang/src/runtime.rs::Value`와 `tools/teul-cli/src/core/value.rs::Value`의 enum variant를 코드에서 직접 추출해 비교했다. 수정·통합 작업은 하지 않았다.

## 요약

- `lang/src/runtime.rs::Value`: 16개 variant.
- `tools/teul-cli/src/core/value.rs::Value`: 14개 variant.
- 완전 동일 모델이 아니다. `lang` 쪽은 `Fixed64`/`Unit`을 분리하고 `StateMachine`/`Regex`를 값 variant로 갖는다.
- `teul-cli` 쪽은 `Num(Quantity)`로 수와 단위를 합치고, `Math`와 `Dice`를 제품 런타임 값으로 갖는다.
- 두 경로가 같은 입력에서 같은 값 동작을 한다고 직접 비교하는 테스트는 찾지 못했다. 존재하는 parity 테스트는 주로 frontdoor parse 또는 CLI/WASM runtime parity이며, 두 `Value` enum 자체의 variant/display/canon 동등성 검사는 아니다.

## Variant 대응 표

| variant(lang) | variant(teul-cli) | 대응 상태(동일/이름다름/한쪽만존재) | 동작 동등성 테스트 존재 여부 |
|---|---|---|---|
| `None` | `None` | 동일 | 직접 동등성 테스트 부재. 각 경로 로컬 테스트만 존재: `lang/src/runtime.rs:437`, `tools/teul-cli/src/core/value.rs:681` |
| `Bool(bool)` | `Bool(bool)` | 동일 | 직접 동등성 테스트 부재. `lang/src/runtime.rs:392`, `tools/teul-cli/src/core/value.rs:697` 등 로컬 테스트만 존재 |
| `Fixed64(Fixed64)` | `Num(Quantity)` | 이름다름 | 직접 동등성 테스트 부재. `lang`은 숫자만 `Fixed64`, `teul-cli`는 `Quantity{raw, dim}` |
| `Unit(UnitValue)` | `Num(Quantity)` | 이름다름 | 직접 동등성 테스트 부재. `lang/src/runtime.rs:359`은 무차원 `Unit`을 index로 허용하고, `teul-cli/src/runtime/eval.rs:2118` 이후는 단위 리터럴을 `Value::Num(Quantity)`로 생성 |
| `String(String)` | `Str(String)` | 이름다름 | 직접 동등성 테스트 부재. `lang/src/runtime.rs:421`, `tools/teul-cli/src/core/value.rs:681` 등 로컬 테스트만 존재 |
| `ResourceHandle(ResourceHandle)` | `ResourceHandle(ResourceHandle)` | 동일 | 직접 동등성 테스트 부재 |
| `List(Vec<Value>)` | `List(ListValue)` | 동일 | 직접 동등성 테스트 부재. variant 이름은 같지만 `teul-cli`는 wrapper struct를 사용 |
| `Set(BTreeMap<String, Value>)` | `Set(SetValue)` | 동일 | 직접 동등성 테스트 부재. variant 이름은 같지만 `teul-cli`는 wrapper struct를 사용 |
| `Map(BTreeMap<String, MapEntry>)` | `Map(MapValue)` | 동일 | 직접 동등성 테스트 부재. 양쪽 모두 map get 로컬 테스트는 있음: `lang/src/runtime.rs:437`, `tools/teul-cli/src/core/value.rs:681` |
| `Pack(BTreeMap<String, Value>)` | `Pack(PackValue)` | 동일 | 직접 동등성 테스트 부재. variant 이름은 같지만 `teul-cli`는 exact numeric/relation special display를 포함 |
| `Assertion(Assertion)` | `Assertion(AssertionValue)` | 동일 | 직접 동등성 테스트 부재. payload 타입은 다름 |
| `StateMachine(StateMachine)` | - | 한쪽만존재(lang) | 직접 동등성 테스트 부재. `lang/src/parser.rs:2045`에서 `ExprKind::StateMachine`을 만들고 `lang/src/runtime.rs:218`에서 값 key 정본 문자열을 만든다 |
| `Formula(Formula)` | `Math(MathValue)` | 이름다름 | 직접 동등성 테스트 부재. `lang`은 AST `Formula`, `teul-cli`는 `MathValue{dialect, body}`를 제품 런타임에서 사용 |
| `Template(Template)` | `Template(TemplateValue)` | 동일 | 직접 동등성 테스트 부재. payload 타입은 다름 |
| `Regex(RegexLiteral)` | - | 한쪽만존재(lang) | 직접 동등성 테스트 부재. `lang/src/parser.rs:2165`에서 regex literal을 만들고 `lang/src/runtime.rs:252`에서 값 key 문자열을 만든다. `teul-cli`는 별도 `Regex` variant 없이 regex 값을 pack/문자열 필드로 다룬다(`tools/teul-cli/src/runtime/eval.rs:13042`) |
| `Lambda(LambdaValue)` | `Lambda(LambdaValue)` | 동일 | 직접 동등성 테스트 부재. 이름은 같지만 equality/canon 기준이 다름: `lang`은 id 비교, `teul-cli`는 capture 포함 canon hash |
| - | `Dice(DiceValue)` | 한쪽만존재(teul-cli) | 직접 동등성 테스트 부재. `tools/teul-cli/src/runtime/eval.rs:2266`, `2292`, `2313`에서 주사위 런타임 상태로 실제 사용 |

## 한쪽만 존재하는 variant의 실제 사용 근거

| variant | 쪽 | 실제 사용 근거 |
|---|---|---|
| `StateMachine` | `lang` | `lang/src/parser.rs:2045`에서 `ExprKind::StateMachine(machine)` 생성, `lang/src/runtime.rs:218`에서 `Value::StateMachine` 정본 문자열 생성 |
| `Regex` | `lang` | `lang/src/parser.rs:2165`에서 `Literal::Regex(regex)` 생성, `lang/src/runtime.rs:252`에서 `Value::Regex` 정본 문자열 생성 |
| `Dice` | `teul-cli` | `tools/teul-cli/src/runtime/eval.rs:2266`에서 `Value::Dice(DiceValue{...})` 생성, `2292`/`2313`에서 상태 갱신, `tools/teul-cli/src/cli/run.rs:4142`에서 JSON 출력 |

## 동등성 테스트 조사

검색 범위:

- `lang/src/runtime.rs`
- `tools/teul-cli/src/core/value.rs`
- `tools/teul-cli/src/runtime/eval.rs`
- `tools/teul-cli/src/cli/frontdoor_parse.rs`
- `tests/`

확인 결과:

- `lang/src/runtime.rs`에는 입력/list/string/map 로컬 단위 테스트가 있다.
- `tools/teul-cli/src/core/value.rs`에는 `MapValue::map_get`/`map_set` 로컬 단위 테스트가 있다.
- `tools/teul-cli/src/cli/frontdoor_parse.rs`에는 lang frontdoor parity 검사가 있지만, parser acceptance parity이며 두 `Value` enum의 런타임 값 동작 비교가 아니다.
- `tests/seamgrim_wasm_cli_runtime_parity_runner.mjs` 계열은 CLI/WASM 제품 경로 parity이며, `lang/src/runtime.rs::Value`와 `tools/teul-cli/src/core/value.rs::Value`를 직접 비교하지 않는다.

따라서 "같은 입력에 대해 두 Value 모델이 같은 동작을 한다"는 직접 회귀 테스트는 현재 부재로 분류한다.

## 추출 하네스

저장소 루트에서 아래 스크립트로 enum variant를 추출했다.

```python
import re
from pathlib import Path

for path in ['lang/src/runtime.rs', 'tools/teul-cli/src/core/value.rs']:
    text = Path(path).read_text(encoding='utf-8')
    body = re.search(r'pub enum Value \{(.*?)\n\}', text, re.S).group(1)
    variants = []
    for line in body.splitlines():
        line = line.strip()
        if not line or line.startswith('#') or line.startswith('//'):
            continue
        match = re.match(r'([A-Za-z_][A-Za-z0-9_]*)', line)
        if match:
            variants.append(match.group(1))
    print(path, len(variants), variants)
```

실행 결과:

```text
lang/src/runtime.rs 16 ['None', 'Bool', 'Fixed64', 'Unit', 'String', 'ResourceHandle', 'List', 'Set', 'Map', 'Pack', 'Assertion', 'StateMachine', 'Formula', 'Template', 'Regex', 'Lambda']
tools/teul-cli/src/core/value.rs 14 ['None', 'Bool', 'Num', 'Str', 'ResourceHandle', 'Math', 'Template', 'Assertion', 'Lambda', 'Dice', 'Pack', 'List', 'Set', 'Map']
```

## 검증

- enum 추출 하네스 실행 PASS.
- variant별 사용 근거 `rg` 확인 PASS.
- 수정·통합 실행 없음.
