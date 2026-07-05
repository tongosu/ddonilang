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
