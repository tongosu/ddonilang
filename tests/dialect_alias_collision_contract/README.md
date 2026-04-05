# Dialect Alias Collision Contract

## Stable Contract

- 목적:
  - `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v8_20260215.tsv`의 keyword/alias surface가 `lang/src/dialect.rs` 로더 기준에서 적어도 `ko` scope에서는 충돌 없이 1:1 canonical map을 이룬다는 점을 고정한다.
  - 동시에 현재 남아 있는 non-`ko` same-dialect collision inventory를 명시적으로 고정해서, 신규 collision drift를 바로 잡는다.
  - 특히 `샘 -> 샘입력`, `입력 -> 입력 블록` 분리가 다시 깨지지 않도록 직접 pin 한다.
- compared surface:
  - `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v8_20260215.tsv`
  - `docs/context/notes/dialect/DIALECT_KEYWORD_GAP_AUDIT_20260301.md`
  - `lang/src/dialect.rs`
- pinned rules:
  - `ko` scope keyword token은 하나의 canonical keyword에만 매핑된다.
  - non-`ko` same-dialect collision은 현재 inventory를 넘어서 늘어나지 않는다.
  - `샘` row의 ko alias는 `샘입력`을 포함하고 `입력`은 포함하지 않는다.
  - `입력` row는 별도 canonical keyword로 존재한다.
  - `구성 -> 짜임`, `효과/바깥 -> 너머`, `소식/사건/알람 -> 알림`, `보개장면 -> 보개마당`, `올때 -> 오면` alias는 계속 유지된다.
- selftest:
  - `python tests/run_dialect_alias_collision_contract_selftest.py`
  - `python tests/run_dialect_alias_collision_inventory_report_selftest.py`
  - `python tests/run_lang_surface_family_selftest.py`
  - `python tests/run_lang_surface_family_contract_selftest.py`
  - `python tests/run_lang_surface_family_contract_summary_selftest.py`
  - `python tests/run_ci_sanity_gate.py --profile core_lang`

## Checks

- ko no-collision:
  - `ko` scope의 `token -> canonical` map size는 항상 1이다.
- known collision inventory:
  - `ay`, `eu`, `kn`, `qu`, `ta`, `te`, `tr` scope의 collision set은 현재 inventory와 정확히 같다.
- pinned split:
  - `샘입력 -> 샘`
  - `입력 -> 입력`
- alias surface:
  - `구성 -> 짜임`
  - `효과 -> 너머`
  - `바깥 -> 너머`
  - `소식 -> 알림`
  - `사건 -> 알림`
  - `알람 -> 알림`
  - `보개장면 -> 보개마당`
- `올때 -> 오면`

## Stable Report Surface

- report schema:
  - `ddn.dialect_alias_collision_inventory.v1`
- report fields:
  - `ko_collision_count`
  - `non_ko_scope_count`
  - `non_ko_collision_count`
  - `known_inventory_match`
  - `non_ko_scopes`
- report selftest:
  - `python tests/run_dialect_alias_collision_inventory_report_selftest.py`
- example output:
  - `build/tmp/dialect_alias_collision_inventory_report.detjson`

## Consumer Surface

- upstream family:
  - `tests/lang_surface_family/README.md`
  - `python tests/run_lang_surface_family_selftest.py`
- direct selftest:
  - `python tests/run_dialect_alias_collision_contract_selftest.py`
  - `python tests/run_dialect_alias_collision_inventory_report_selftest.py`
