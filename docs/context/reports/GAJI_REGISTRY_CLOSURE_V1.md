# GAJI_REGISTRY_CLOSURE_V1

## M1 — 재귀 스캐너

### 변경

- `tools/teul-cli/src/cli/gaji.rs`의 `collect_packages`를 재귀 스캔으로 변경했다.
- 중첩 `gaji.toml`을 찾되, 패키지 루트(`gaji.toml`이 있는 디렉터리)를 만나면 그 하위로 더 내려가지 않게 했다. 기존 direct 패키지의 파일 수집 의미를 유지하기 위한 처리다.
- `MAX_GAJI_SCAN_DEPTH = 16` 깊이 제한을 추가했다.
- `DirEntry::file_type().is_dir()`로 디렉터리를 판별해 심볼릭 링크 디렉터리를 따라가지 않게 했다.
- `run_lock_recursively_finds_nested_packages` 회귀 테스트를 추가했다.

### 실측

`cargo run --manifest-path tools/teul-cli/Cargo.toml -- gaji lock --root . --out out/gaji-registry-closure/m1/ddn.lock.post`

```text
gaji_lock_written=I:\dev\ddonilang\codex\out\gaji-registry-closure\m1\ddn.lock.post
gaji_lock_hash=blake3:e3f182d383cf8237f2bbddc79beccaa4a60bd43dd52058c324318f8535667965
```

직전 HEAD의 direct-only 스캐너와 현재 recursive 스캐너 비교:

```text
pre_count=11
post_count=13
missing_from_post=
new_in_post=물리 진자 가지,보개 공간2d 가지
changed_existing=
new_pkg id=물리 진자 가지 path=phys/pendulum version=0.1.0 hash=blake3:b21ea55f280eb836aaaaf48a5184b04df9083e3de212a4117ce0ac15528cec49
new_pkg id=보개 공간2d 가지 path=bogae/space2d version=0.1.0 hash=blake3:2c440378c0c24e9d78326cefc4cf62e40788ca74dbb049006fc3f3c20e10fe71
```

판정:

- `gaji lock` 결과가 11개에서 13개로 늘었다.
- 새로 발견된 패키지는 `gaji/phys/pendulum`, `gaji/bogae/space2d` 두 개다.
- 기존 11개 패키지는 `version/path/hash/files` 변경이 0건이다.

### 검증

```text
cargo test --manifest-path tools/teul-cli/Cargo.toml run_lock_recursively_finds_nested_packages
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1093 filtered out
```

```text
cargo test --manifest-path tools/teul-cli/Cargo.toml
test result: ok. 1094 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## M2 — 메타데이터 없는 top-level 디렉터리 분류

### 범위

- `gaji/` 최상위 디렉터리: 30개.
- direct `gaji.toml` 보유: 11개.
- direct `gaji.toml` 없음: 19개.
- 재귀 `gaji.toml`: 13개. 이 중 2개는 `gaji/bogae/space2d`, `gaji/phys/pendulum` 중첩 패키지다.

### 분류 기준

- `중첩 부모`: 최상위 디렉터리 자체에는 `gaji.toml`이 없고, 하위 디렉터리에 실제 패키지 `gaji.toml`이 있다.
- `패키지 후보`: README가 `가지`/공개 API surface를 설명하고 `ddn/exports.ddn` 같은 package surface 파일이 있지만, metadata가 없다.
- `문서/예제 skeleton`: README가 `Documentation-only skeleton`, `not loaded by product runtime code`, `does not define a module/import mechanism` 등을 명시하거나, 예제 파일이 pack/제품 경로의 증거를 가리키는 문서용 자산이다.

### 19개 분류표

| top-level | 파일 근거 | 분류 | 메모 |
|---|---|---|---|
| `bogae` | `space2d/gaji.toml`, `space2d/ddn/exports.ddn` | 중첩 부모 | M1 이후 실제 패키지는 `gaji/bogae/space2d`로 스캔됨 |
| `phys` | `pendulum/gaji.toml`, `pendulum/ddn/exports.ddn` | 중첩 부모 | M1 이후 실제 패키지는 `gaji/phys/pendulum`로 스캔됨 |
| `std_grid` | `ddn/exports.ddn`, README 공개 API | 패키지 후보 | `격자.*` 공개 surface가 있으나 metadata 없음 |
| `std_input_map` | `ddn/exports.ddn`, README 공개 API | 패키지 후보 | `입력사상.*` 공개 surface가 있으나 metadata 없음 |
| `std_physics_1d` | `ddn/exports.ddn`, README 공개 API | 패키지 후보 | `물리1d.*` 공개 surface가 있으나 metadata 없음 |
| `std_block_piece` | `minimum.ddn`, README | 문서/예제 skeleton | README가 product code 미로드와 module/import 부재 명시 |
| `std_grid_game_bogae_bridge` | `example.ddn`, README | 문서/예제 skeleton | README가 documentation-only skeleton 명시 |
| `std_grid_game_bogae_browser_dom_smoke` | `example.ddn`, README | 문서/예제 skeleton | README가 runtime 미로드 명시 |
| `std_grid_game_bogae_browser_input_delivery` | `example.ddn`, README | 문서/예제 skeleton | README가 runtime 미로드 명시 |
| `std_grid_game_bogae_finite_live_loop` | `example.ddn`, README | 문서/예제 skeleton | README가 runtime 미로드 명시 |
| `std_grid_game_bogae_live_bridge` | `example.ddn`, README | 문서/예제 skeleton | 예제가 pack 경로를 가리키는 documentation-only pointer |
| `std_grid_game_bogae_viewer_js_dom` | `example.ddn`, README | 문서/예제 skeleton | 예제가 pack 경로를 가리키는 documentation-only pointer |
| `std_grid_game_bogae_web_playback` | `example.ddn`, README | 문서/예제 skeleton | 예제가 pack 입력을 실행 증거로 가리킴 |
| `std_grid_game_bogae_web_showcase` | `example.ddn`, README | 문서/예제 skeleton | README가 runtime 미로드와 module/import 부재 명시 |
| `std_grid_game_playable` | `minimum.ddn`, README | 문서/예제 skeleton | README가 product code 미로드 명시 |
| `std_grid_game_playable_view` | `example.ddn`, README | 문서/예제 skeleton | README가 runtime 미로드 명시 |
| `std_grid_game_rules_minimum` | `example.ddn`, README | 문서/예제 skeleton | README가 runtime 미로드 명시 |
| `std_grid_game_state` | `minimum.ddn`, README | 문서/예제 skeleton | README가 product code 미로드와 module/import 부재 명시 |
| `std_random_bag` | `minimum.ddn`, README | 문서/예제 skeleton | README가 product code 미로드와 module/import 부재 명시 |

### 판정 요약

- metadata 없는 19개 중 `bogae`, `phys` 2개는 중첩 패키지 부모라 top-level `gaji.toml`을 만들 대상이 아니다.
- `std_grid`, `std_input_map`, `std_physics_1d` 3개는 실제 패키지 후보지만, 브리프 제약에 따라 `gaji.toml`을 만들지 않고 후보로만 보고한다.
- 나머지 14개는 README/예제 기준 문서 또는 예제 skeleton이며, 현 시점에서 install/discover 대상 패키지로 분류하지 않는다.
- M2에서는 `gaji.toml`을 추가하지 않았다.

### 검증

```text
cargo test --manifest-path tools/teul-cli/Cargo.toml
test result: ok. 1094 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## M3 — publish 실제 패키징

### 변경

- `teul-cli gaji registry publish`에 선택형 `--package-dir <path>`, `--archive-out <path>`를 추가했다.
- 기존 수동 경로(`--archive-sha256`, `--download-url`)는 그대로 유지했다.
- `--package-dir`가 주어지면 해당 gaji package 디렉터리를 deterministic zip archive로 작성한다.
- archive 기본 위치는 index 파일 옆 `archives/<scope>__<name>__<version>.zip`이다.
- `archive_sha256`은 작성된 archive bytes의 실제 sha256으로 계산해 index entry에 넣는다.
- `download_url`이 명시되지 않으면 archive의 index-relative path를 넣는다.
- package-dir에는 `gaji.toml`이 있어야 한다. metadata 없는 디렉터리는 M2 제약대로 임의 발행하지 못하게 했다.

### 포맷 선택

- 기존 의존성에 `zip`이 이미 있고 `tools/teul-cli/src/cli/universe.rs`가 deterministic zip 패턴을 사용하고 있어 같은 방식을 택했다.
- zip 옵션은 `CompressionMethod::Stored`, 고정 timestamp `1980-01-01T00:00:00`, 파일명 정렬이다.
- index schema에는 새 필드를 넣지 않았다. 기존 `archive_sha256`, `download_url`만 채운다.

### 실제 저장소 패키지 실측

`gaji/std_math`를 로컬 registry에 발행했다.

```text
cargo run --manifest-path tools/teul-cli/Cargo.toml -- gaji registry publish --index out/gaji-registry-closure/m3/registry.index.json --scope gaji --name std_math --version 0.1.0 --package-dir gaji/std_math --token token1 --role publisher --at 2026-02-19T00:00:00Z
registry_publish_ok=gaji/std_math@0.1.0
registry_publish_archive_out=out/gaji-registry-closure/m3\archives\gaji__std_math__0.1.0.zip
registry_publish_archive_sha256=sha256:73943e7c3c814cfeb28f9231854d29fb4b4a9bd81c7e34ea7669f5cd983a0ac0
registry_publish_download_url=archives/gaji__std_math__0.1.0.zip
```

Index/파일 확인:

```text
entry_scope=gaji
entry_name=std_math
entry_version=0.1.0
entry_archive_sha256=sha256:73943e7c3c814cfeb28f9231854d29fb4b4a9bd81c7e34ea7669f5cd983a0ac0
entry_download_url=archives/gaji__std_math__0.1.0.zip
archive_exists=true
archive_bytes=1175
```

### 검증

```text
cargo test --manifest-path tools/teul-cli/Cargo.toml run_cli_publish_package_dir_writes_archive_and_index
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1095 filtered out
```

```text
cargo test --manifest-path tools/teul-cli/Cargo.toml run_cli_publish_requires_archive_sha_or_package_dir
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1095 filtered out
```

```text
cargo test --manifest-path tools/teul-cli/Cargo.toml
test result: ok. 1096 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
