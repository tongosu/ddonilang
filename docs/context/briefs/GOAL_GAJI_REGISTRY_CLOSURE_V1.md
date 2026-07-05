# GOAL: 가지(gaji) 로컬 레지스트리 실제 닫힘 V1 (Codex Goal 모드용, 1-2주 규모)

> 작성: Claude (2026-07-06) / 실행: Codex(Goal 자율 루프) / 리뷰: Claude
> 근거: `docs/context/reports/LOCAL_REGISTRY_LANDING_AUDIT_V1.md`(Q21), `docs/context/reports/GAJI_SCAFFOLD_SURVEY_V1.md`(Q22)
> 범위: **언어 커널이 아니라 툴체인**이다 — Q13~18 커널 게이트와 무관, 지금 바로 착수 가능.
> 성격: 실제 구현 + 감사 병행. 각 마일스톤마다 실행(구현) → 검증(테스트) → 감사(실측 재확인) 사이클을 스스로 반복하라.

## 배경 — 정확히 뭐가 비어 있는지 (Q21 실측)

- `gaji lock/install/update/vendor`, `gaji registry publish/search/versions/entry/verify/audit-verify/download`는 전부 **존재하고 더미 환경에서 PASS 확인됨**.
- 하지만 세 가지가 실제로 안 이어진다:
  1. **스캐너가 재귀적이지 않다**(`tools/teul-cli/src/cli/gaji.rs:528` `collect_packages`) — `gaji/<name>/gaji.toml`만 보고 그 아래는 안 본다. 실제로 `gaji/bogae/space2d/gaji.toml`, `gaji/phys/pendulum/gaji.toml` 2개가 이 때문에 스캔에서 누락된다.
  2. **`registry download`(`tools/teul-cli/src/cli/gaji_registry.rs:1485` `run_download`)가 index entry의 아카이브를 파일로 받기만 하고, 그걸 unpack해서 `vendor/gaji`에 놓는 install 경로와 안 이어진다.**
  3. **`registry publish`(`tools/teul-cli/src/cli/gaji_registry.rs:1633` `run_publish`)가 실제 `gaji/<name>` 디렉터리를 패키징하는 게 아니라 index JSON에 entry만 추가한다.**
- `gaji/` top-level 30개 중 direct `gaji.toml`이 있는 건 11개, 나머지 19개는 메타데이터 자체가 없다(정적 배치 자료인지 진짜 패키지인지 미확인).

## 최종 목표(Outcome)

저장소의 실제 `gaji/` 패키지 하나를 **실제로 publish → registry에 등록 → download → unpack → vendor 배치**까지 전 과정이 진짜로 연결된 상태로 만든다. "더미 환경에서 각 명령이 따로 도는" 수준을 넘어, **저장소 자산으로 end-to-end 파이프라인이 실제로 닫히는 것**을 목표로 한다.

## 마일스톤 (순서대로, 각각 커밋 1개 이상)

### M1 — 재귀 스캐너

- `collect_packages`를 재귀로 바꿔 `gaji/bogae/space2d/gaji.toml`, `gaji/phys/pendulum/gaji.toml`을 포함해 전체 13개를 발견하게 한다.
- 무한 재귀/심볼릭 링크 순환 방지(깊이 제한 또는 방문 집합).
- **Verification**: `gaji lock` 실행 후 `ddn.lock`에 13개 패키지 전부 등재 확인. 기존 11개 패키지의 hash/내용이 하나도 안 바뀌었는지 확인(회귀 없음).

### M2 — 19개 메타데이터 없는 디렉터리 분류

- `gaji/` top-level 19개(`gaji/bogae`, `gaji/phys`, `gaji/std_grid_game_*` 등)를 하나씩 확인: (a) 진짜 패키지인데 `gaji.toml`이 없는 것, (b) 패키지가 아니라 공유 코드/문서/중첩 부모 디렉터리인 것.
- 분류 결과를 표로 보고서에 남긴다. **(a)로 분류된 것에 `gaji.toml`을 임의로 만들지 마라** — 그건 설계 판단이 필요하다. 표로만 보고.

### M3 — publish가 실제로 패키지를 포장하게

- `run_publish`(또는 그 상위 CLI 경로)가 로컬 `gaji/<name>` 디렉터리를 실제로 아카이브(tar 또는 zip, 기존 `collect_files`/`package_hash` 로직 재사용)로 만들고, 그 아카이브를 index에 등록하도록 확장.
- 기존 index entry 스키마와 호환 유지(추가 필드는 additive).

### M4 — download가 실제로 install까지 이어지게

- `run_download`가 받은 아카이브를 unpack해서 `vendor/gaji/<name>`에 배치하는 경로를 추가(또는 `gaji install --from-registry <name>` 같은 새 서브커맨드로 명시적으로 연결 — 어느 쪽이든 M3에서 만든 아카이브 포맷과 짝이 맞아야 한다).
- 기존 `vendor` 로직(로컬 lock 기반)과 이 registry 기반 경로가 공존 가능해야 한다(하나가 다른 걸 깨면 안 됨).

### M5 — 실제 패키지로 end-to-end 재현

- 더미가 아니라 **저장소의 실제 패키지 하나**(예: `gaji/std_math` 또는 재귀로 새로 발견된 `gaji/phys/pendulum`)를 로컬 더미 registry에 실제로 publish하고, 다른 위치(scratch 디렉터리)에서 실제로 download+install해서 원본과 내용이 동일한지(`package_hash` 일치) 확인하는 회귀 테스트를 작성.

### M6 — 감사 재확인 (병행)

- M1~M5가 끝나면, `docs/context/roadmap/GANADA_MATRIX_CORRECTED_20260706.md`의 가-4/나-4(로컬 레지스트리) 칸을 다시 실측해서, "부분 착지"에서 "실제동작"으로 판정이 바뀌는지 확인하고 짧은 갱신 보고를 남긴다(로드맵 문서 자체는 수정하지 말고 별도 보고서로).

## 공통 제약(Constraints)

- 기존 `gaji lock/install/update/vendor`의 로컬(비-registry) 경로 동작을 깨지 마라 — 회귀 테스트로 매 마일스톤마다 확인.
- `gaji.toml` 스키마의 기존 필드(`id/name/version/ssot_requires/det_tier/openness/description`) 의미를 바꾸지 마라.
- 네트워크 실제 호출 금지 — 전부 로컬 파일시스템 기반 더미 registry로 검증.
- M2에서 발견한 "패키지인데 메타데이터 없음" 후보에 임의로 `gaji.toml`을 만들지 마라(설계 판단 필요, 보고만).
- 각 마일스톤 완료 시 `codex/queue-20260706` 브랜치에 커밋(`[GOAL-GAJI-M1]` ~ `[GOAL-GAJI-M6]`), main 직접 커밋/push 금지.
- 판단이 막히는 지점(예: M3/M4의 아카이브 포맷 선택)에서는 스스로 합리적 기본값을 정해 진행하되, 그 근거를 보고서에 명시하라 — 완전히 막히면 그 마일스톤만 부분 보고하고 다음으로 넘어가라.

## 산출물

- `docs/context/reports/GAJI_REGISTRY_CLOSURE_V1.md` — M1~M6 전체 실행 요약, 각 마일스톤 검증 결과, 실제 명령/출력 근거.
- 이 브리프 파일 하단에 마일스톤별 `## 실행 보고 M1` ~ `## 실행 보고 M6`, 마지막에 `## Goal 종료 보고`.

## 진행 규칙

1. M1 → M6 순서. 하나가 완전히 막히면 부분 보고 후 다음으로.
2. 매 마일스톤 커밋 전 자기 검증을 실제로 실행하고 출력을 인용(주장 금지).
3. `cargo test --manifest-path tools/teul-cli/Cargo.toml` 매 마일스톤 후 PASS 확인(회귀 없음).
4. `python tests/run_ci_sanity_gate.py --profile core_lang` 최종 PASS 확인.

## 실행 보고 M1

- 완료: `collect_packages`를 재귀 스캔으로 바꿔 `gaji/bogae/space2d/gaji.toml`, `gaji/phys/pendulum/gaji.toml`을 포함하게 했다.
- 순환 방지: `MAX_GAJI_SCAN_DEPTH = 16` 깊이 제한을 두고, `DirEntry::file_type().is_dir()` 기준으로 디렉터리를 판별해 심볼릭 링크 디렉터리를 따라가지 않게 했다.
- 회귀 확인: 직전 HEAD direct-only lock과 현재 recursive lock을 비교했다. `pre_count=11`, `post_count=13`, `changed_existing=`(기존 11개 `version/path/hash/files` 변경 0건).
- 새로 발견된 패키지: `물리 진자 가지`(`phys/pendulum`), `보개 공간2d 가지`(`bogae/space2d`).
- 실행:
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml run_lock_recursively_finds_nested_packages` PASS — `1 passed; 0 failed`.
  - `cargo run --manifest-path tools/teul-cli/Cargo.toml -- gaji lock --root . --out out/gaji-registry-closure/m1/ddn.lock.post` PASS — `gaji_lock_hash=blake3:e3f182d383cf8237f2bbddc79beccaa4a60bd43dd52058c324318f8535667965`.
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml` PASS — `1094 passed; 0 failed`.

## 실행 보고 M2

- 완료: direct `gaji.toml`이 없는 `gaji/` 최상위 디렉터리 19개를 전수 분류했다.
- 분류 결과: 중첩 부모 2개(`bogae`, `phys`), metadata 누락 패키지 후보 3개(`std_grid`, `std_input_map`, `std_physics_1d`), 문서/예제 skeleton 14개.
- 제약 준수: metadata 누락 후보에 `gaji.toml`을 만들지 않았고, 코드도 수정하지 않았다.
- 산출물: `docs/context/reports/GAJI_REGISTRY_CLOSURE_V1.md`의 M2 표.
- 실행:
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml` PASS — `1094 passed; 0 failed`.

## 실행 보고 M3

- 완료: `gaji registry publish`에 `--package-dir`/`--archive-out` 선택 인자를 추가해 실제 gaji package 디렉터리를 deterministic zip archive로 포장하고 index에 등록하게 했다.
- 호환성: 기존 수동 `--archive-sha256`/`--download-url` 경로는 유지했다. index entry schema는 바꾸지 않고 기존 `archive_sha256`, `download_url`만 채운다.
- 포맷: 기존 `zip` 의존성과 `universe` pack 패턴을 따라 `Stored` zip + 고정 timestamp + 정렬된 파일 순서를 사용했다.
- 제약 준수: `--package-dir`에는 `gaji.toml`을 요구하므로 metadata 없는 M2 후보를 임의 발행하지 않는다.
- 실제 실측: `gaji/std_math`를 `out/gaji-registry-closure/m3/registry.index.json`에 발행했고, archive `archives/gaji__std_math__0.1.0.zip` 생성 및 `archive_sha256=sha256:73943e7c3c814cfeb28f9231854d29fb4b4a9bd81c7e34ea7669f5cd983a0ac0` 확인.
- 실행:
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml run_cli_publish_package_dir_writes_archive_and_index` PASS — `1 passed; 0 failed`.
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml run_cli_publish_requires_archive_sha_or_package_dir` PASS — `1 passed; 0 failed`.
  - `cargo run --manifest-path tools/teul-cli/Cargo.toml -- gaji registry publish --index out/gaji-registry-closure/m3/registry.index.json --scope gaji --name std_math --version 0.1.0 --package-dir gaji/std_math --token token1 --role publisher --at 2026-02-19T00:00:00Z` PASS.
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml` PASS — `1096 passed; 0 failed`.

## 실행 보고 M4

- 완료: `gaji registry download`에 `--vendor-out <dir>` 선택 인자를 추가해 archive 다운로드 후 `vendor-out/<name>`으로 해제하게 했다.
- 호환성: 기존 `--out` archive 파일 출력은 유지한다. `--vendor-out`을 주지 않으면 기존 download 동작만 수행한다.
- vendor 배치: 일반 사용은 `--vendor-out vendor/gaji`이며 결과는 `vendor/gaji/<name>`이다. 기존 vendor 전체가 아니라 해당 package 디렉터리만 교체한다.
- 안전성: zip entry는 상대 경로만 허용하고, 해제 후 `gaji.toml` 존재를 확인한다.
- 실제 실측: `gaji/std_math`를 M4 로컬 registry에 publish한 뒤 `download --vendor-out out/gaji-registry-closure/m4/vendor/gaji` 실행. `registry_download_vendor_files=3`, `vendor_gaji_toml_exists=true`, `vendor_exports_exists=true`.
- 실행:
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml run_cli_download_vendor_out_unpacks_package_archive` PASS — `1 passed; 0 failed`.
  - `cargo run --manifest-path tools/teul-cli/Cargo.toml -- gaji registry download --index out/gaji-registry-closure/m4/registry.index.json --scope gaji --name std_math --version 0.1.0 --out out/gaji-registry-closure/m4/download/std_math.zip --vendor-out out/gaji-registry-closure/m4/vendor/gaji` PASS.
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml` PASS — `1097 passed; 0 failed`.

## 실행 보고 M5

- 완료: 실제 repository package를 쓰는 회귀 테스트 `run_cli_real_gaji_package_publish_download_vendor_matches_source`를 추가했다.
- 테스트 내용: `gaji/std_math` 원본 package를 publish → download --vendor-out으로 scratch vendor에 설치하고, 원본/벤더 상대경로별 bytes map을 비교한다.
- 추가 실측: 중첩 실제 패키지 `gaji/phys/pendulum`도 scratch registry에서 publish → download --vendor-out 실행했고 `content_match=true`, `vendor_file_count=3`을 확인했다.
- 실행:
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml run_cli_real_gaji_package_publish_download_vendor_matches_source` PASS — `1 passed; 0 failed`.
  - `cargo run --manifest-path tools/teul-cli/Cargo.toml -- gaji registry publish --index out/gaji-registry-closure/m5/registry.index.json --scope gaji --name pendulum --version 0.1.0 --package-dir gaji/phys/pendulum --token token1 --role publisher --at 2026-02-19T00:00:00Z` PASS.
  - `cargo run --manifest-path tools/teul-cli/Cargo.toml -- gaji registry download --index out/gaji-registry-closure/m5/registry.index.json --scope gaji --name pendulum --version 0.1.0 --out out/gaji-registry-closure/m5/download/pendulum.zip --vendor-out out/gaji-registry-closure/m5/vendor/gaji` PASS — `content_match=true`.
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml` PASS — `1098 passed; 0 failed`.

## 실행 보고 M6

- 완료: `docs/context/roadmap/GANADA_MATRIX_CORRECTED_20260706.md`의 가-4/나-4 로컬 레지스트리 칸을 별도 보고서에서 재감사했다. 로드맵 문서 자체는 수정하지 않았다.
- 재실행:
  - `python tests/run_roadmap_v2_ga4_package_gaji_reconciliation_check.py` PASS — `[roadmap-v2-ga4-package-gaji-reconciliation] OK`.
  - `python tests/run_pack_golden.py roadmap_v2_ga4_package_gaji_reconciliation_v1` PASS — `pack golden ok`.
  - `python tests/run_roadmap_v2_na4_stdlib_registry_reconciliation_check.py` PASS — `[roadmap-v2-na4-stdlib-registry-reconciliation] OK`.
  - `python tests/run_pack_golden.py roadmap_v2_na4_stdlib_registry_reconciliation_v1` PASS — `pack golden ok`.
- 재판정: M1~M5로 scanner 재귀화, package archive publish, download vendor unpack, 실제 repository package e2e가 닫혔다. 따라서 가-4/나-4의 local filesystem registry minimum은 `부분 착지`에서 `실제동작`으로 상향 가능하다.
- 제한: public registry final, network/cloud sync, trust signing, full install/update/remove UX, 제품 UI install execution, top-level `gaji/` 30개 전체 package closure는 여전히 주장하지 않는다.
- 실행:
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml` PASS — `1098 passed; 0 failed`.
  - `python tests/run_ci_sanity_gate.py --profile core_lang` PASS — `ci_sanity_status=pass code=OK step=all msg="-" profile=core_lang`.

## Goal 종료 보고

- M1~M6 순서 실행 완료.
- 최종 산출물: `docs/context/reports/GAJI_REGISTRY_CLOSURE_V1.md`.
- 구현 닫힘 범위: metadata-bearing 실제 gaji package의 local registry publish -> registry index 등록 -> download -> vendor unpack 경로.
- 미주장 범위: public registry, network/cloud, trust signing, 제품 UI install 실행, metadata 없는 후보 package의 스키마 결정, `gaji/` 30개 전체 closure.
- 최종 검증:
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml` PASS.
  - `python tests/run_ci_sanity_gate.py --profile core_lang` PASS.
