# LOCAL_REGISTRY_LANDING_AUDIT_V1

## 결론

`gaji.toml` 기반 로컬 레지스트리는 **부분 착지** 상태다.

- `teul-cli gaji lock/install/update/vendor`는 존재하며, 로컬 `gaji/` 바로 아래의 `gaji.toml` 패키지를 `ddn.lock`과 `vendor/gaji`로 복사하는 경로가 있다.
- `teul-cli gaji registry ...`도 존재하며, JSON index 기준 `versions/entry/search/federated-search/download/publish/yank/verify/audit-verify` 표면을 제공한다.
- 그러나 저장소의 `gaji/` 패키지 집합이 실제로 registry publish/discover/install 경로를 거쳐 쓰인다는 증거는 없다. 현 검증은 fixture/temp index 중심이고, UI도 install execution을 후속으로 명시한다.
- `discover`라는 직접 서브커맨드는 없다. 가장 가까운 표면은 `registry search`와 `registry federated-search`다.
- `gaji/` 아래 top-level 디렉터리는 30개지만, current local install 스캐너가 발견 가능한 direct `gaji.toml` 패키지는 11개뿐이다. recursive `gaji.toml`은 13개이며, 그중 2개는 현재 1-depth 스캐너에서 누락된다.

따라서 SSOT가 말하는 `immediate_dev_track`의 **local registry minimum**은 CLI와 fixture 수준에서 일부 구현되어 있으나, “gaji/ 30개 패키지가 실제 install/publish/discover 경로를 통해 쓰이는 제품 닫힘”은 아직 아니다. 현재 `gaji/`의 상당수는 레지스트리 경로로 설치되는 패키지라기보다 코드베이스에 직접 배치된 정적 후보/자료에 가깝다.

## SSOT 기준

| 기준 | 근거 | 감사 판정 |
|---|---|---|
| local install / local publish / current workflow minimum은 immediate_dev_track | `docs/ssot/ssot/SSOT_TOOLCHAIN_v24.12.9.md:136`, `:137`, `:138`, `:139`, `:140`, `:228`, `:229`, `:230`, `:231` | 일부 착지. CLI 표면은 있으나 저장소 패키지 30개 전체의 닫힌 흐름은 아님 |
| public registry ecosystem / full install-update-remove UX는 follow-on | `docs/ssot/ssot/SSOT_TOOLCHAIN_v24.12.9.md:141`, `:142`, `:143`, `:144`, `:232`, `:248` | 구현/문서 모두 public final 미주장 |
| 모듬 선택/다운로드/버전/핀/잠금/검증은 toolchain 담당 | `docs/ssot/ssot/SSOT_TOOLCHAIN_v24.12.9.md:1452`, `:1453`, `:1454`, `:1455` | `gaji`/`gaji_registry` CLI에 일부 있음 |
| `ddn.lock`은 의존성 최종 id/pin/resolved_source와 재현 SSOT | `docs/ssot/ssot/SSOT_TOOLCHAIN_v24.12.9.md:1526`, `:1528`, `:1529`, `:1530`, `:1531` | `gaji lock/install/vendor`는 lock을 만들고 읽지만 registry source까지 닫힌 해소 모델은 제한적 |
| `gaji.toml` 정본 metadata | `docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7014`, `:7028`, `:7029`, `:7030` | 파일은 존재하나 top-level 30개 중 direct discover 가능은 11개 |
| MOD1 다운로드/버전/잠금/검증은 toolchain 경계 | `docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7536`, `:7538`, `:7539`, `:7540` | 언어 표면과 별도 CLI로 일부 구현 |
| PROJECT1 구조 의미와 lock/meta 경계 | `docs/ssot/ssot/SSOT_LANG_v24.12.9.md:7633`, `:7636`, `:7637`, `:7651`, `:7654` | registry/install은 runtime core가 아니라 metadata/toolchain 층으로 보는 것이 맞음 |

## CLI 구현 표면

| 표면 | 구현 근거 | 실제 의미 |
|---|---|---|
| `gaji lock` | `tools/teul-cli/src/main.rs:640`, `:641`, `:646`, `:647`, `:2991`, `:2992`, `:2993`, `:3004` | `root/gaji`를 스캔해 `ddn.lock` 작성 |
| `gaji install` | `tools/teul-cli/src/main.rs:659`, `:666`, `:668`, `:670`, `:682`, `:3006`, `:3025`, `:3026`, `:3027`, `:3044` | lock이 없으면 만들고, 이후 vendor 실행 |
| `gaji update` | `tools/teul-cli/src/main.rs:695`, `:702`, `:704`, `:706`, `:728`, `:3046`, `:3070`, `:3071`, `:3072`, `:3096` | lock 재생성 후 vendor 실행 |
| `gaji vendor` | `tools/teul-cli/src/main.rs:741`, `:748`, `:750`, `:752`, `:764`, `:3098`, `:3117`, `:3118`, `:3119`, `:3136` | lock의 패키지를 `vendor/gaji`로 복사 |
| `gaji registry ...` | `tools/teul-cli/src/main.rs:777`, `:778`, `:785`, `:3138` | 별도 registry CLI로 raw args 전달 |
| registry read/search | `tools/teul-cli/src/cli/gaji_registry.rs:132`, `:133`, `:155`, `:177`, `:199`, `:1355`, `:1386`, `:1422`, `:1443` | JSON index 기준 조회/검색 |
| registry download | `tools/teul-cli/src/cli/gaji_registry.rs:211`, `:230`, `:1530`, `:1556`, `:1564`, `:1572`, `:1586`, `:1596` | index entry의 `download_url` 아카이브를 받아 파일로 씀. install과 연결된 unpack 경로는 아님 |
| registry publish | `tools/teul-cli/src/cli/gaji_registry.rs:243`, `:256`, `:264`, `:1637`, `:1642`, `:1678`, `:1708`, `:1712`, `:1717`, `:1727`, `:1739` | index JSON에 entry 추가 + audit append. `gaji/` 디렉터리를 포장하는 publish는 아님 |

구현 파일 자체도 범위를 “local registry minimum”으로 설명한다. `tools/teul-cli/src/cli/gaji_registry.rs:3`부터 `:5`는 local install/local publish와 `gaji.toml`/`ddn.lock`/publication snapshot guard를 current line으로 두고, full public registry ecosystem과 hosted governance를 out of scope로 둔다.

## 로컬 `gaji/` 인벤토리

감사 명령:

```powershell
Get-ChildItem -Path gaji -Directory
Get-ChildItem -Path gaji -Recurse -Filter gaji.toml
```

결과:

| 구분 | 수 | 의미 |
|---|---:|---|
| `gaji/` top-level 디렉터리 | 30 | 사용자가 지칭한 “gaji/ 30개”에 해당 |
| direct `gaji/<name>/gaji.toml` | 11 | 현 `collect_packages()`가 발견 가능한 패키지 |
| recursive `gaji.toml` 전체 | 13 | 중첩 2개 포함 |
| direct 디렉터리 중 `gaji.toml` 없음 | 19 | 현 `gaji lock/install`에서는 기본 스캔상 무시 |

현 스캐너는 recursive가 아니다. `tools/teul-cli/src/cli/gaji.rs:528`부터 `:540`은 `fs::read_dir(gaji_root)`의 direct child만 보고, 그 child 바로 아래 `gaji.toml`이 없으면 건너뛴다. 따라서 아래 2개 recursive 패키지는 파일은 있으나 current local install 스캔에는 걸리지 않는다.

- `gaji/bogae/space2d/gaji.toml`
- `gaji/phys/pendulum/gaji.toml`

direct `gaji.toml` 패키지는 11개다.

- `gaji/30_nurigym_core`
- `gaji/ddn.nuance.v0`
- `gaji/ddn.story.v0`
- `gaji/ddn.timeline.v0`
- `gaji/element_swap`
- `gaji/std_charim`
- `gaji/std_logic`
- `gaji/std_map`
- `gaji/std_math`
- `gaji/std_text`
- `gaji/time`

top-level direct 디렉터리 중 `gaji.toml`이 없는 항목은 19개다.

- `gaji/bogae`
- `gaji/phys`
- `gaji/std_block_piece`
- `gaji/std_grid`
- `gaji/std_grid_game_bogae_bridge`
- `gaji/std_grid_game_bogae_browser_dom_smoke`
- `gaji/std_grid_game_bogae_browser_input_delivery`
- `gaji/std_grid_game_bogae_finite_live_loop`
- `gaji/std_grid_game_bogae_live_bridge`
- `gaji/std_grid_game_bogae_viewer_js_dom`
- `gaji/std_grid_game_bogae_web_playback`
- `gaji/std_grid_game_bogae_web_showcase`
- `gaji/std_grid_game_playable`
- `gaji/std_grid_game_playable_view`
- `gaji/std_grid_game_rules_minimum`
- `gaji/std_grid_game_state`
- `gaji/std_input_map`
- `gaji/std_physics_1d`
- `gaji/std_random_bag`

## 설치/발행/발견 경로 판정

| 질문 | 판정 | 근거 |
|---|---|---|
| teul-cli에 관련 서브커맨드가 있는가 | 있음 | `gaji lock/install/update/vendor`와 `gaji registry ...`가 있다 |
| `discover`가 서브커맨드명으로 있는가 | 없음 | `rg "discover"`에서 registry package discovery 명령은 발견되지 않았다. 대체 표면은 `registry search`/`federated-search` |
| `gaji/` 30개 top-level이 install 경로를 통과하는가 | 아니오 | direct `gaji.toml` 11개만 current scanner 대상. 19개 top-level은 무시, nested 2개도 무시 |
| `gaji/` 패키지가 publish로 포장되는가 | 아니오 | `registry publish`는 CLI 인자로 받은 scope/name/version/archive hash를 index entry에 추가한다. 소스 `gaji.toml`/디렉터리 포장 단계는 없다 |
| registry download가 install로 이어지는가 | 아니오 | `registry download`는 아카이브 bytes를 `out` 파일에 쓴다. `gaji install`은 local `root/gaji`와 `ddn.lock` 기반 vendor 복사다 |
| pack/check가 실제 저장소 `gaji/`를 registry 경로로 설치하는가 | 확인 안 됨 | 확인된 pack은 fixture/temp index 중심이다 |
| UI가 install 실행을 닫았다고 주장하는가 | 아니오 | `solutions/seamgrim_ui_mvp/ui/app.js:1482`부터 `:1504`는 mock/server adapter와 “준비 중” toast이며, `toolchain_registry_verification.js:224`는 install execution/network/trust signing을 후속으로 둔다 |

## 테스트/팩 증거

| 증거 | 근거 | 의미 |
|---|---|---|
| `gaji_registry_report_provenance_v1` | `pack/gaji_registry_report_provenance_v1/README.md:3`, `:6`, `:7`, `:8`, `:9` | registry verify/audit-verify report provenance 고정 |
| `run_gaji_registry_pack_check.py` | `tests/run_gaji_registry_pack_check.py:66`, `:68`, `:71`, `:74`, `:75`, `:76`, `:77`, `:80`, `:94`, `:97`, `:99`, `:100`, `:110`, `:112`, `:114` | fixture index/lock을 temp dir로 복사해 verify/publish/audit-verify 실행. 저장소 `gaji/` 패키지 설치는 아님 |
| `run_gaji_registry_report_provenance_check.py` | `tests/run_gaji_registry_report_provenance_check.py:44`, `:46`, `:47`, `:89`, `:96`, `:97`, `:98`, `:118`, `:125`, `:126`, `:127`, `:146`, `:153`, `:154`, `:155` | temp index/lock 생성 후 cargo run으로 registry report 검증 |
| `seamgrim_registry_publish_install_shell_v1` | `pack/seamgrim_registry_publish_install_shell_v1/contract.detjson:4`, `:5`, `:6`, `:7`, `:8`, `:9` | runner-backed shell 경계 팩이며 `closure_claim`은 `no` |
| capability matrix sample | `tests/run_ai_det_tier_capability_matrix_pack_check.py:122`, `:123`, `:124`, `:127` | 일부 `gaji.toml` 존재 여부만 확인. registry install/discover 경로 검증은 아님 |

## 미착지/불일치

1. “30개 패키지”와 current scanner의 실제 대상이 다르다. top-level 30개 중 direct `gaji.toml`은 11개뿐이고, 현 scanner는 recursive `gaji.toml` 2개도 발견하지 않는다.
2. `publish`는 `gaji.toml` 패키지 publish가 아니라 registry index entry append다.
3. `install`은 registry에서 discover/download한 패키지를 설치하는 경로가 아니라, local `root/gaji` + `ddn.lock` 기반 vendor copy다.
4. `download`는 archive 파일 저장까지만 한다. `vendor/gaji`로 unpack하거나 project lock/install과 결합하는 경로는 확인되지 않았다.
5. `discover` 명령은 없다. search/federated-search는 있으나 “패키지 발견 후 install” workflow로 닫혀 있지 않다.
6. 제품 UI는 install/publish를 mock/dry-run/준비 중으로 다룬다. 공개 registry publish, install execution, network IO는 후속으로 명시되어 있다.

## 최종 판정

현재 상태는 다음처럼 부르는 것이 정확하다.

- **착지됨:** local `gaji.toml` 스캔 기반 lock/vendor 최소, registry JSON index 조회/검증/발행/audit 최소, fixture 기반 provenance pack.
- **부분 착지:** registry metadata를 lock/install/vendor 검증 옵션에 연결하는 guard.
- **미착지:** 저장소 `gaji/` 30개 top-level 전체의 package registry discover/install/publish 닫힌 workflow, `gaji.toml` 디렉터리 포장 publish, registry download-to-install, 제품 UI install 실행 닫힘.

이번 감사에서는 구현·수정·네트워크 실행을 하지 않았다. 파일/코드/문서 근거만 확인했다.
