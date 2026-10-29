# Gate0 Surface Transport Family

## Stable Contract

- 목적:
  - 현재 `core_lang` gate에서 닫힌 Gate0/language transport line을 한 단계 위에서 함께 읽는 최상위 umbrella transport family contract를 고정한다.
  - 이 문서는 하위 transport line의 세부 field를 다시 정의하지 않고, `lang_surface_transport`, `lang_runtime_transport`, `gate0_runtime_transport`, `gate0_family_transport`, `gate0_transport_family_transport`, `gate0_surface_family_transport`가 같은 Gate0 surface transport를 이룬다는 점만 확인한다.
- 대상 surface:
  - `tests/lang_surface_family/README.md`
  - `tests/lang_runtime_family/README.md`
  - `tests/gate0_runtime_family/README.md`
  - `tests/gate0_family/README.md`
  - `tests/gate0_transport_family/README.md`
  - `tests/gate0_surface_family/README.md`
- selftest:
  - `python tests/run_lang_surface_family_transport_contract_selftest.py`
  - `python tests/run_lang_runtime_family_transport_contract_selftest.py`
  - `python tests/run_gate0_runtime_family_transport_contract_selftest.py`
  - `python tests/run_gate0_family_transport_contract_selftest.py`
  - `python tests/run_gate0_transport_family_transport_contract_selftest.py`
  - `python tests/run_gate0_surface_family_transport_contract_selftest.py`
  - `python tests/run_gate0_surface_transport_family_selftest.py`
  - `python tests/run_gate0_surface_transport_family_contract_selftest.py`
  - `python tests/run_gate0_surface_transport_family_contract_summary_selftest.py`
- sanity steps:
  - `lang_surface_family_transport_contract_selftest`
  - `lang_runtime_family_transport_contract_selftest`
  - `gate0_runtime_family_transport_contract_selftest`
  - `gate0_family_transport_contract_selftest`
  - `gate0_transport_family_transport_contract_selftest`
  - `gate0_surface_family_transport_contract_selftest`
  - `gate0_surface_transport_family_selftest`
  - `gate0_surface_transport_family_contract_selftest`
  - `gate0_surface_transport_family_contract_summary_selftest`

## Stable Bundle Contract

- bundle `checks_text`:
  - `lang_surface_family_transport,lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family_transport,gate0_surface_family_transport`
- progress schema:
  - `ddn.ci.gate0_surface_transport_family_contract_selftest.progress.v1`
- sanity steps:
  - `gate0_surface_transport_family_selftest`
  - `gate0_surface_transport_family_contract_selftest`
  - `gate0_surface_transport_family_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport bundle `checks_text`:
  - `family_contract,lang_surface_transport,lang_runtime_transport,gate0_runtime_transport,gate0_family_transport,gate0_transport_family_transport,gate0_surface_family_transport`
- progress schema:
  - `ddn.ci.gate0_surface_transport_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `gate0_surface_transport_family_transport_contract_selftest`
  - `gate0_surface_transport_family_transport_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`
- selftest:
  - `python tests/run_gate0_surface_transport_family_transport_contract_selftest.py`
  - `python tests/run_gate0_surface_transport_family_transport_contract_summary_selftest.py`

## Matrix

| surface line | summary | primary contract |
| --- | --- | --- |
| lang surface transport | `proof/bogae/compound update` downstream transport | `lang_surface_family`가 핵심 언어 표면 transport를 닫는다 |
| lang runtime transport | `lang surface + stdlib + tensor` downstream transport | `lang_runtime_family`가 언어/runtime transport를 닫는다 |
| gate0 runtime transport | `lang runtime + W95/W96/W97` downstream transport | `gate0_runtime_family`가 Gate0 runtime transport를 닫는다 |
| gate0 family transport | `gate0 runtime + W92/W93/W94` downstream transport | `gate0_family`가 Gate0 상위 transport를 닫는다 |
| gate0 transport umbrella | `lang/gate0 transport umbrella` | `gate0_transport_family`가 Gate0 transport umbrella를 닫는다 |
| gate0 surface umbrella transport | `Gate0/language 상위 umbrella transport` | `gate0_surface_family`가 Gate0 surface umbrella transport를 닫는다 |

## Consumer Surface

- `tests/lang_surface_family/README.md`
- `tests/lang_runtime_family/README.md`
- `tests/gate0_runtime_family/README.md`
- `tests/gate0_family/README.md`
- `tests/gate0_transport_family/README.md`
- `tests/gate0_surface_family/README.md`
- `python tests/run_lang_surface_family_transport_contract_selftest.py`
- `python tests/run_lang_runtime_family_transport_contract_selftest.py`
- `python tests/run_gate0_runtime_family_transport_contract_selftest.py`
- `python tests/run_gate0_family_transport_contract_selftest.py`
- `python tests/run_gate0_transport_family_transport_contract_selftest.py`
- `python tests/run_gate0_surface_family_transport_contract_selftest.py`
- `python tests/run_gate0_surface_transport_family_selftest.py`
- `python tests/run_gate0_surface_transport_family_contract_selftest.py`
- `python tests/run_gate0_surface_transport_family_contract_summary_selftest.py`
- `python tests/run_gate0_surface_transport_family_transport_contract_selftest.py`
- `python tests/run_gate0_surface_transport_family_transport_contract_summary_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
