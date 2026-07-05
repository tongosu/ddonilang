# GAJI_SCAFFOLD_SURVEY_V1

작성일: 2026-07-06
브랜치: `codex/queue-20260706`
범위: `gaji/` 실제 30개 디렉터리, `docs/ssot/gaji/` 스켈레톤 비교, `tools/teul-cli/src/cli/gaji.rs::collect_packages`/`parse_gaji_toml` 확인.
성격: 조사 전용. `gaji new` 설계나 구현은 하지 않았다.

## 요약

- 실제 `gaji/` 최상위 디렉터리는 30개다.
- 실제 `gaji.toml`은 재귀 기준 13개다. 이 중 현재 `collect_packages`가 바로 발견하는 최상위 `gaji/<name>/gaji.toml`은 11개이고, `gaji/bogae/space2d`, `gaji/phys/pendulum` 2개는 중첩 패키지라 현재 최상위 스캔에는 잡히지 않는다.
- 실제 `gaji.toml` 13개는 모두 같은 키 집합을 쓴다: `id`, `name`, `version`, `ssot_requires`, `det_tier`, `openness`, `description`.
- 제품 CLI의 `parse_gaji_toml`은 현재 `id`, `name`, `version`만 읽는다. `ssot_requires`, `det_tier`, `openness`, `description`, SSOT 일부 파일의 `[requires]`, `age_target`은 읽지 않는다.
- 관측된 기존 최소 뼈대는 `gaji.toml` + `README.md` 2파일이다(`ddn.story.v0`, `ddn.timeline.v0`). 표준 API 뼈대 계열은 여기에 `ddn/exports.ddn`을 더한 3파일 구조를 쓴다.

## 조사 명령

```powershell
Get-ChildItem -LiteralPath gaji -Directory
Get-ChildItem -LiteralPath gaji -Recurse -Filter gaji.toml -File
Get-ChildItem -LiteralPath docs/ssot/gaji -Recurse -Filter gaji.toml -File
rg -n "fn collect_packages|fn parse_gaji_toml|match key" tools/teul-cli/src/cli/gaji.rs
rg -n "\[requires\]|age_target|det_tier|openness|ssot_requires|description" docs/ssot/gaji -g "gaji.toml"
```

## 제품 코드 근거

`tools/teul-cli/src/cli/gaji.rs:528`의 `collect_packages`는 `gaji_root`의 직계 자식만 순회한다. 각 자식 디렉터리 바로 아래 `gaji.toml`이 없으면 `continue`한다(`:537`-`:540`). 그래서 `gaji/bogae/space2d/gaji.toml`, `gaji/phys/pendulum/gaji.toml` 같은 중첩 패키지는 현재 스캔 대상이 아니다.

`tools/teul-cli/src/cli/gaji.rs:557`의 `parse_gaji_toml`은 줄 단위로 `key = value`를 읽고, `:575`-`:579`에서 `id`, `name`, `version`만 반영한다. `:587`은 `id`가 없으면 `name`, 그것도 없으면 `gaji/<dir_name>`으로 대체한다. `:588`-`:592`는 `version`이 없으면 `E_GAJI_TOML_VERSION`으로 실패시킨다.

## gaji/ 30개 전수 표

| 패키지명 | 공통 파일 목록 | gaji.toml 필드 실사용 | 비고 |
|---|---|---|---|
| `30_nurigym_core` | `ddn/`, `gaji.toml`, `README.md` | `id`, `name`, `version`, `ssot_requires`, `det_tier`, `openness`, `description` | 최상위 `gaji.toml`; 현재 CLI 스캔 대상 |
| `bogae` | `space2d/` | `space2d/gaji.toml`: 위 7개 필드 | 중첩 패키지. 현재 `collect_packages` 최상위 스캔에는 잡히지 않음 |
| `ddn.nuance.v0` | `gaji.toml`, `nuance.detjson`, `README.md` | 위 7개 필드 | 최상위 `gaji.toml`; 데이터 파일 포함 |
| `ddn.story.v0` | `gaji.toml`, `README.md` | 위 7개 필드 | 최상위 `gaji.toml`; 관측된 2파일 최소 뼈대 |
| `ddn.timeline.v0` | `gaji.toml`, `README.md` | 위 7개 필드 | 최상위 `gaji.toml`; 관측된 2파일 최소 뼈대 |
| `element_swap` | `ddn/`, `gaji.toml`, `overrides/`, `README.md` | 위 7개 필드 | 최상위 `gaji.toml`; overrides 포함 |
| `phys` | `pendulum/` | `pendulum/gaji.toml`: 위 7개 필드 | 중첩 패키지. 현재 `collect_packages` 최상위 스캔에는 잡히지 않음 |
| `std_block_piece` | `minimum.ddn`, `README.md` | 없음 | `gaji.toml` 없음 |
| `std_charim` | `ddn/`, `gaji.toml`, `README.md` | 위 7개 필드 | 최상위 `gaji.toml`; 표준 API 뼈대 |
| `std_grid` | `ddn/`, `README.md` | 없음 | `gaji.toml` 없음 |
| `std_grid_game_bogae_bridge` | `example.ddn`, `README.md` | 없음 | 예제형 디렉터리 |
| `std_grid_game_bogae_browser_dom_smoke` | `example.ddn`, `README.md` | 없음 | 예제형 디렉터리 |
| `std_grid_game_bogae_browser_input_delivery` | `example.ddn`, `README.md` | 없음 | 예제형 디렉터리 |
| `std_grid_game_bogae_finite_live_loop` | `example.ddn`, `README.md` | 없음 | 예제형 디렉터리 |
| `std_grid_game_bogae_live_bridge` | `example.ddn`, `README.md` | 없음 | 예제형 디렉터리 |
| `std_grid_game_bogae_viewer_js_dom` | `example.ddn`, `README.md` | 없음 | 예제형 디렉터리 |
| `std_grid_game_bogae_web_playback` | `example.ddn`, `README.md` | 없음 | 예제형 디렉터리 |
| `std_grid_game_bogae_web_showcase` | `example.ddn`, `README.md` | 없음 | 예제형 디렉터리 |
| `std_grid_game_playable` | `minimum.ddn`, `README.md` | 없음 | `gaji.toml` 없음 |
| `std_grid_game_playable_view` | `example.ddn`, `README.md` | 없음 | 예제형 디렉터리 |
| `std_grid_game_rules_minimum` | `example.ddn`, `README.md` | 없음 | 예제형 디렉터리 |
| `std_grid_game_state` | `minimum.ddn`, `README.md` | 없음 | `gaji.toml` 없음 |
| `std_input_map` | `ddn/`, `README.md` | 없음 | `gaji.toml` 없음 |
| `std_logic` | `ddn/`, `gaji.toml`, `README.md` | 위 7개 필드 | 최상위 `gaji.toml`; 표준 API 뼈대 |
| `std_map` | `ddn/`, `gaji.toml`, `README.md` | 위 7개 필드 | 최상위 `gaji.toml`; 표준 API 뼈대 |
| `std_math` | `ddn/`, `gaji.toml`, `README.md` | 위 7개 필드 | 최상위 `gaji.toml`; 표준 API 뼈대 |
| `std_physics_1d` | `ddn/`, `README.md` | 없음 | `gaji.toml` 없음 |
| `std_random_bag` | `minimum.ddn`, `README.md` | 없음 | `gaji.toml` 없음 |
| `std_text` | `ddn/`, `gaji.toml`, `README.md` | 위 7개 필드 | 최상위 `gaji.toml`; 표준 API 뼈대 |
| `time` | `ddn/`, `gaji.toml`, `README.md` | 위 7개 필드 | 최상위 `gaji.toml`; 실제 `exports.ddn` 구현 있음 |

## 필드 사용 현황

실제 `gaji/`의 `gaji.toml` 13개는 모두 아래 7개 키를 사용한다.

| 필드 | 실제 사용 수 | 제품 파서 반영 여부 | 비고 |
|---|---:|---|---|
| `id` | 13 | 읽음 | 없으면 `name` 또는 디렉터리명 기반 대체 |
| `name` | 13 | 읽음 | `id`가 없을 때 대체값으로만 쓰임 |
| `version` | 13 | 읽음 | 없으면 `E_GAJI_TOML_VERSION` |
| `ssot_requires` | 13 | 읽지 않음 | 실제 파일에는 최상위 키 |
| `det_tier` | 13 | 읽지 않음 | 실제 파일에는 최상위 키 |
| `openness` | 13 | 읽지 않음 | 실제 파일에는 최상위 키 |
| `description` | 13 | 읽지 않음 | 실제 파일에는 최상위 키 |

SSOT 쪽 `docs/ssot/gaji`의 일부 파일(`ddn.bogae`, `ddn.nuance.v0`, `ddn.std.colors`, `ddn.story.v0`, `ddn.timeline.v0`)은 `[requires]` 섹션 아래 `ssot_requires`, `age_target`, `det_tier`, `openness`를 둔다. 현재 제품 파서는 섹션 줄을 건너뛰고 키 이름만 보는 단순 파서지만, 위 4개 키 자체를 반영하지 않으므로 이 섹션 구조도 제품 동작에는 반영되지 않는다.

## SSOT 스켈레톤과 실제 gaji/ 차이

| 구분 | 값 |
|---|---|
| 실제 `gaji/` 최상위 디렉터리 수 | 30 |
| `docs/ssot/gaji/` 최상위 디렉터리 수 | 19 |
| 실제 `gaji/` 재귀 `gaji.toml` 수 | 13 |
| `docs/ssot/gaji/` 재귀 `gaji.toml` 수 | 10 |
| 실제에만 있는 최상위 디렉터리 | `30_nurigym_core`, `element_swap`, `std_grid_game_bogae_bridge`, `std_grid_game_bogae_browser_dom_smoke`, `std_grid_game_bogae_browser_input_delivery`, `std_grid_game_bogae_finite_live_loop`, `std_grid_game_bogae_live_bridge`, `std_grid_game_bogae_viewer_js_dom`, `std_grid_game_bogae_web_playback`, `std_grid_game_bogae_web_showcase`, `std_grid_game_playable`, `std_grid_game_playable_view`, `std_grid_game_rules_minimum`, `std_physics_1d` |
| SSOT에만 있는 최상위 디렉터리 | `40_jojo_studio_econ`, `ddn.bogae`, `ddn.std.colors` |
| 양쪽에 있는 최상위 디렉터리 | `bogae`, `ddn.nuance.v0`, `ddn.story.v0`, `ddn.timeline.v0`, `phys`, `std_block_piece`, `std_charim`, `std_grid`, `std_grid_game_state`, `std_input_map`, `std_logic`, `std_map`, `std_math`, `std_random_bag`, `std_text`, `time` |

구조상 가장 큰 차이는 실제 `gaji/`에는 게임/보개 예제형 디렉터리와 런타임 실험 디렉터리가 많이 추가되어 있지만, 그중 다수는 `gaji.toml`이 없어 현재 CLI의 패키지 스캔 대상이 아니라는 점이다. 반대로 SSOT 쪽에는 `ddn.bogae`, `ddn.std.colors`처럼 실제 `gaji/` 최상위에는 없는 스켈레톤이 남아 있다.

## 최소 뼈대 예시 1: `gaji/ddn.story.v0`

파일 목록:

```text
gaji/ddn.story.v0/gaji.toml
gaji/ddn.story.v0/README.md
```

`gaji/ddn.story.v0/gaji.toml`:

```toml
id = "gaji/ddn.story.v0"
name = "서사 스키마 v0"
version = "0.1.0"
ssot_requires = ">=20.1.24"

# 결정성/공개성(초기값)
det_tier = "D-STRICT"
openness = "closed"

# 메타
description = "시트콤/서사 결과 스키마"
```

`gaji/ddn.story.v0/README.md`:

```markdown
# 서사 스키마 v0 (gaji/ddn.story.v0)

시트콤 엔진이 생성하는 story.detjson 스키마의 최소 골격입니다.
```

## 최소 뼈대 예시 2: `gaji/std_logic`

파일 목록:

```text
gaji/std_logic/ddn/exports.ddn
gaji/std_logic/gaji.toml
gaji/std_logic/README.md
```

`gaji/std_logic/gaji.toml`:

```toml
id = "gaji/std_logic"
name = "표준 논리 가지"
version = "0.1.0"
ssot_requires = ">=20.1.22"

# 결정성/공개성(초기값)
det_tier = "D-STRICT"
openness = "closed"

# 메타
description = "논리/비교 처리"
```

`gaji/std_logic/README.md`:

```markdown
# 표준 논리 가지 (gaji/std_logic)

이 패키지는 SSOT v20.1.22의 DR-089에 따라 stdlib를 가지(gaji)로 분할한 모듈입니다.

## 공개 API
- `ddn/exports.ddn`에 공개 API 이름/시그니처를 나열합니다.
- 문서/pack/예제에서는 **정본 이름만 사용**합니다.
- 기존 구현 명칭은 alias 테이블(`docs/status/STDLIB_ALIAS_TABLE.md`)로 관리합니다.

## 구현 상태
- 실제 구현/별칭/팩은 Codex 작업 티켓을 따릅니다.
```

`gaji/std_logic/ddn/exports.ddn`:

```ddn
// exports.ddn — 공개 API 선언(스켈레톤)
// TODO: SSOT_LANG/SSOT_DECISIONS의 표준 이름/시그니처에 맞춰 채우기.
```

## 요약 결론

현재 구현 기준으로 CLI 패키지 스캔에 들어가려면 `gaji/<패키지명>/gaji.toml`이 최상위 직계 위치에 있어야 하고, 제품 파서가 실제로 요구하는 필드는 `version`뿐이다. 다만 기존 저장소 관례 기준의 최소 뼈대는 `gaji.toml` + `README.md` 2파일이며, 표준 API를 드러내는 가지는 여기에 `ddn/exports.ddn`을 추가한 3파일 구조를 쓴다. 실제 파일에 널리 쓰이는 `ssot_requires`, `det_tier`, `openness`, `description`은 현재 제품 파서가 읽지 않는 문서성 메타데이터다.
