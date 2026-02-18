# PROJECT_STATUS.md

## 목적
이 문서는 관문 0 작업에서 추가/수정된 코드와 결정 사항을 한곳에 모아 이후 협업 시 빠르게 맥락을 복원할 수 있도록 정리한 기록이다.

## 최근 문서 갱신
- 셈그림 UI MVP/stdlib 범위 상세 보고서를 확장하고, 교과과정 통합 마스터코스 제안서를 추가했다.
- stdlib_range_basics에 단위 불일치 오류 골든 케이스를 추가했다.
- stdlib 범위 pack에 단위 범위 케이스를 추가하고, 브리지 체크에서 UI 마크업 확인을 포함했다.
- stdlib_range_basics에 범위 간격 0 오류 골든 케이스를 추가했다.
- stdlib 범위 기본 pack(stdlib_range_basics)을 추가하고 골든/문서 커버리지를 보강했다.
- 셈그림 UI에 DDN 생성 버튼을 추가하고 브리지 자동 점검 스크립트(bridge_check.py), stdlib 범위 문서 표(impl/alias) 및 ddonirang_mastercourse 계획을 보강했다.
- AGENTS 지침에 build/out 경로를 시작 시 생성하고 I: 경로 우선, 없으면 C: fallback하도록 명시했다.
- stdlib `범위`(2~3인자, 단위 정합/증가·감소/포함 범위) 함수를 teul-cli/tool 런타임과 lang stdlib 시그니처에 추가하고, 셈그림 샘플/자동 DDN 생성에 반영했다.
- 샘/거울 최소 스키마를 SSOT에 반영하기 위한 관리자 요청 노트(`docs/notes/SSOT_SAM_GEOUL_SCHEMA_REQUEST_20260130.md`)를 추가했다.
- 셈그림 UI에서 수식/변수와 DDN 편집을 탭으로 분리하고, DDN 샘플을 에 대해/차림 기반으로 간소화했다.
- 셈그림 UI MVP에 수식→DDN 자동 생성→런타임 브리지 실행을 연결하고, 샘/거울 스키마(v0) 이름 정합 및 UI 검증/요약 로직을 보강했다.
- 범위 리터럴/범위 생성 최소 스펙 제안서를 추가하고 SSOT ALL 합본(`docs/context/all/ssot_*_ALL.md`)을 갱신했다.
- 셈그림 UI MVP 제안/설계 문서, UI 스켈레톤/그래프·스냅샷 JSON 및 샘/거울 로드(검증 요약/요약 카드) + 스냅샷 다중 오버레이(자동 색상/라벨 규칙) + DDN 브리지 실행, seamgrim_line_graph D-PACK(2케이스), seamgrim_ui_mvp 솔루션/변환툴/스냅샷·샘·거울 스키마와 SSOT 제안서를 추가했다.
- W77 gogae7 대표 데모 D-PACK을 추가하고 bogae_hash 골든을 고정했다.
- W76 detbin 캐시(동일 프레임)와 cache hit/miss 로그를 추가했다.
- W75 docs/ssot/walks 갱신은 SSOT 관리자 작업으로 요청만 기록했다.
- W74 pack golden 러너에 bogae_hash 1급 필드 검증을 추가하고 bogae_runner_bogae_hash D-PACK을 추가했다.
- W73 bogae bundle v1 CLI와 bundle 스모크 D-PACK을 추가했다.
- W72 bogae hash determinism v1 스모크 D-PACK을 추가했다.
- W71 bogae editor v0(detbin 이동/색상 변경) CLI와 editor 스모크 D-PACK을 추가했다.
- W70 bogae web viewer v1 스모크 D-PACK을 추가했다.
- W69 bogae mapping v1(태그→Rect/Text/Sprite) 스모크 D-PACK과 매핑 처리(보개_태그_목록/보개_매핑_목록)를 추가했다.
- W68 bogae asset manifest v1 스모크 D-PACK과 `teul-cli asset manifest` 명령을 추가했다.
- W67 bogae adapter v1 smoke D-PACK을 추가하고 bogae_hash 골든을 고정했다.
- W66 open end-to-end(deny/record/replay) D-PACK을 추가했다.
- W65 open bundle 아티팩트(--open-bundle)와 open_bundle_artifact D-PACK을 추가했다.
- W64 open.rand(외부 난수) 봉인과 open_rand_record_replay D-PACK을 추가했다.
- W63 replay mismatch 진단 코드를 표준화(E_OPEN_LOG_MISSING/PARSE/REPLAY_MISS/LOG_TAMPER)하고 D-PACK을 추가했다.
- W62 open site_id 정본화(blake3 해시)와 open_site_id_canon D-PACK을 추가했다.
- W61 open.policy allowlist 파싱/기본 모드 적용과 open_policy_allowlist D-PACK을 추가했다.
- open.policy 탐색을 입력 파일 기준 상위(프로젝트 루트까지)로 확장했다.
- W46 목표 문법(-도록) 파서/TargetState/GoalCondition 구현과 D-PACK/골든 테스트를 추가했다.
- AGE2(Open) W59 `#열림 허용(...)` 지시문 파싱과 미선언 open 경고, open_decl_policy D-PACK/골든을 추가했다.
- open_deny_policy D-PACK과 stderr/exit_code golden 검증을 추가했다.
- open_decl_policy_warn D-PACK으로 미선언 open_kind 경고를 검증했다.
- open_replay_missing D-PACK으로 replay 로그 누락 오류를 검증했다.
- open_replay_invalid D-PACK 추가와 open_deny_policy record 케이스 확장을 반영했다.
- open_replay_hash_mismatch D-PACK 추가와 replay log detjson_hash 검증을 반영했다.
- open_replay_site_mismatch/open_replay_schema_mismatch D-PACK 추가와 replay schema 검증을 반영했다.
- open_file_read_key_mismatch D-PACK과 stderr 대안 매칭을 반영했다.
- open_replay_schema_v2_accept/open_file_read_path_normalize D-PACK과 open.log v2 fallback 및 경로 정규화를 반영했다.
- open_file_read_abs_path D-PACK과 golden runner의 {{CWD}} 치환을 추가했다.
- open_file_read_abs_case/open_replay_schema_v2_extra D-PACK과 {{CWD_UPPER}} 치환을 추가했다.
- open_file_read_unc_case/open_replay_schema_v2_missing D-PACK과 OPEN_LOG_V2_RULES 문서를 추가했다.
- open_file_read_unc_dot/open_replay_schema_v2_nested_meta D-PACK과 v2 중첩 meta 규칙을 추가했다.
- open_file_read_unc_mix/open_replay_schema_v2_meta_list D-PACK과 v2 meta scalar/list 규칙을 추가했다.
- open_file_read_unc_dot_mix/open_replay_schema_v2_meta_scalar D-PACK과 v2 meta 문자열/숫자 규칙을 추가했다.
- open_file_read_unc_space_special/open_replay_schema_v2_meta_bool_null D-PACK과 v2 meta 불리언/널 규칙을 추가했다.
- AGE2(Open) W56 기본(deny/record/replay + open.log.jsonl)을 구현하고 open_clock/open_file_read pack goldens를 추가했다.
- 제5고개 pack에 golden.jsonl을 추가하고 golden 러너를 확장해 run 이외 teul-cli 명령도 검증 가능하게 했다.
- 라이브 콘솔 커서를 숨겨 하단 깜빡임을 줄이고 웹 뷰어 기본 배율을 1.25로 조정했다.
- 테트리스 Full D-PACK(pack/game_maker_tetris_full)과 스킨/샘 입력/예시를 추가했다.
- 테트리스 Full 콘솔/웹 정렬 가이드와 웹 뷰어 배율(scale) 옵션을 반영했다.
- W45 SeulgiIntent/SeulgiPacket를 SSOT v20.2.0 규격으로 정합하고 intent bundle/팩/골든을 갱신했다.
- SSOT 기준을 v20.2.0으로 갱신하고 CURRENT/ai prompt/W91/toolchain golden 및 SSOT_bundle_v20.2.0_codex.zip을 정합했다.
- W44 time travel 통합 D-PACK/골든 추가를 반영했다.
- W43 audit inspector(geoul query/backtrace) CLI/팩/골든 추가를 반영했다.
- W42 branching manager(replay branch) CLI/팩/골든 추가를 반영했다.
- W41 타임라인 CLI와 gaji/ddn.timeline.v0, gogae4_w41 D-PACK 및 골든 테스트를 반영했다.
- W40 시트콤 엔진에서 suggested_intents를 추가하고 골든 기대값을 갱신했다.
- W40 시트콤 엔진 CLI와 gaji/ddn.story.v0, gogae4_w40 D-PACK 및 골든 테스트를 반영했다.
- W39 말결값 런타임과 gaji/ddn.nuance.v0, gogae4_w39 D-PACK 및 골든 테스트를 반영했다.
- W37 기억씨/ W38 감정씨 런타임 추가와 gogae4_w37/w38 D-PACK 및 골든 테스트를 반영했다.
- AGE1 stdlib 완료 선언(2026-01-26)과 stdlib_missing_coverage_* pack 골든 고정(시드 명시 포함)을 반영했다.
- teul-cli 런타임에 `풀기`/입력/RNG/자원 호출과 호출식 단독 문장 실행을 추가하고 stdlib_missing_coverage_pure/io pack 골든을 갱신했다.
- `teul-cli canon`에 fix-it in-place 자동 반영 금지 가드를 추가하고 ddn.ai.policy.json에 rails/training 경계를 명시했다.
- `teul-cli run`에 `--run-manifest`/`--artifact`를 추가해 쓸감 핀 경로를 마련했다.
- `gaji/element_swap` 골격을 추가해 샘/보개 등 원소 구현 교체 경로를 예약했다.
- AGE1 pending(C2/C3/C4) 항목을 DONE으로 갱신했다.
- WORKSPACE_MANIFEST/CURRENT patch_version을 v20.2.0_20260127_292dad7d로 갱신했다.
- 정본화기에 씨앗 정의(인자/기본값)와 호출식 문장 지원을 추가했다.
- 정본화기에 `@단위`/`@"자원"` 접미, 원자 리터럴(`#...`), 훅 문장(`(시작)할때`, `(매마디)마다`), `?`/`!` 종결 처리에 더해 `??` 프롬프트 구문, `해서/하고` 파이프, `(핀묶음)인 호출`, 소수점 숫자, 다중 단어 호출명, 블록/개행 종결 허용을 추가하고 docs/EXAMPLES CANON:SKIP을 0건으로 정리했다.
- `teul-cli canon --check`를 추가하고 pack/차림 값 호출용 pack 인자 구문을 정본화기가 인식하도록 보완했다.
- `tests/check_canon_inputs.py`에 docs/EXAMPLES까지 포함해 정본 체크를 확장하고 guides 예제 01~04를 최신 정본 출력으로 갱신했으며 CANON:SKIP 잔여를 0건으로 정리했다.
- `docs/WORKSPACE_MANIFEST.json` 해시/바이트를 최신 변경분으로 갱신했다.
- `teul-cli gaji lock`(build/lock/gaji.lock.json)과 `teul-cli scan`(W_SKIP_NON_GAJI_DIR 경고)을 추가했다.
- patch apply의 canon 체크를 strict로 전환하고 W90 patch 골든 입력을 정본화 가능 형태로 갱신했다.
- 꼬리 없는 호출 진단(E_CALL_TAIL_MISSING_*)을 LSP/정본화기에 추가하고 QuickFix(…기/…하기)를 제공했다.
- `gaji/30_nurigym_core` 골격을 추가하고 `pack/nuri_gym_cartpole`를 DR-075 최소 스키마 DetJson 샘플로 갱신했다.
- `teul-cli alrim registry`를 추가해 `build/registry/alrim.registry.json` 생성/해시/충돌 진단 경로를 마련했다.
- `ddn.ai.policy.json` 기본 파일을 추가했다(정책 해시 메타 포함 경로 활성화).
- `ddn.patch.json`에 before/after 스키마를 추가하고 patch apply의 canon 체크(실패 시 경고 스킵) 및 verify 기본 tests 경로를 보강했다.
- 제5고개 W45~W55 슬기/누리짐/GOAP/latency/safety/dataset/workshop 경로를 core/teul-cli에 추가하고 관련 pack 11종과 골든 테스트를 구성했다.
- stdlib gaji 분할 v1을 위해 `gaji/std_{text,charim,math,map,logic}` 골격과 exports 스켈레톤을 추가하고 WORKSPACE_MANIFEST에 반영했다.
- AI 친화 Phase0 산출물(stdlib_examples 10종, errors.jsonl 120개/12패턴)을 확인하고 AGE1 pending 항목을 갱신했다.
- `pack/math_calculus_v1`, `pack/bdl2_subpixel_aa_v1`, `pack/age1_charim_index`를 추가하고 pack/W22 골든을 재검증했다.
- 기본 pack golden 러너 목록에 math_calculus_v1/bdl2_subpixel_aa_v1/age1_charim_index를 포함했다.
- pack/stdlib_examples 일부 입력/기대값을 정정하고 전체 pack golden(--all)을 재검증했다.
- 키보드 한국어 별칭/보개 배경색 키/보개 목록 키를 DONE 처리하고 관련 pack goldens를 추가했다.
- 텐서 stdlib P0, stdlib 정합, 모두{} 설탕을 DONE 처리하고 관련 pack 골든/W108~W109를 재검증했다.
- AGE1 런타임 타입검사/계약 진단/슬기 훅 항목을 DONE으로 갱신하고 관련 W106/W107 골든을 재검증했다.
- 복합 갱신/계약 모드 pack golden.jsonl을 추가하고 기본 pack 골든 러너 목록에 포함했다.
- AGE1 pending 정리에서 복합 갱신/계약 모드/fixits-json을 DONE으로 갱신하고 관련 D-PACK(복합 갱신/계약 모드)을 추가했다.
- PLN-20260125-COMPOUND-UPDATE-01 정합 방침을 기록했다: `+<-`/`-<-`만 채택, `+=`/`-=`는 거부.
- ai prompt lean 골든을 SSOT v20.1.18 기준으로 재생성하고 테스트를 재검증했다.
- WORKSPACE_MANIFEST patch_version을 v20.1.18_20260125_a0a63b00로 갱신했다.
- CURRENT.md를 SSOT v20.1.18 및 최신 WORKSPACE_MANIFEST 기준으로 갱신했다.
- ddn_runtime에 수학 stdlib(abs/min/max/clamp/sqrt/powi)를 추가해 teul-cli와 정합을 맞췄다.
- WORKSPACE_MANIFEST.json을 갱신했다.
- stdlib 정본/별칭 상태판(STDLIB_IMPL_MATRIX/ALIAS_TABLE)과 stdlib 기본 pack 4종(text/charim/math/map)을 추가했다.
- stdlib 별칭 정본화를 반영하고 `묶음.키` 필드 접근 문법/정본/런타임 지원과 W05 골든 테스트를 추가했다.
- teul-cli ai prompt SSOT_VERSION을 v20.1.17로 갱신하고 SSOT_bundle_v20.1.17_codex.zip 및 W91 골든을 재생성했다.
- ai prompt 번들 해시 벡터(AI-PROMPT-GOLDEN-01) 단위 테스트를 teul-cli에 추가했다.
- teul-cli canon Parser::peek 누락과 check SeedLiteral 매치 누락을 보완했다.
- W91 ai prompt 스냅샷 골든을 최신 SSOT 기준으로 갱신했다.
- teul-cli ai prompt SSOT_VERSION을 v20.1.16으로 갱신하고 W91 골든을 재생성했다.
- SSOT_bundle_v20.1.16_codex.zip을 생성하고 W91 골든을 zip 기준으로 고정했다.
- teul-cli 골든 테스트(W01~W109)를 재검증했다.
- stdlib 확장 제안 대비 구현 매핑 표를 정리했다.
- stdlib 명칭 정합(정본/별칭) 제안서를 추가했다.
- stdlib 명칭 정합(정본/별칭) 결정(DEC-20260125-STDLIB-NAMES-01)을 SSOT 변경 기록에 반영했다.
- 씨앗 리터럴 `{x | ...}` 문법/정본화/런타임을 추가하고 차림 고차 함수(정렬/거르기/변환/합치기) 및 문자열 포함/시작/끝/숫자로 표준 시그니처를 SSOT v20.1.15 기준으로 정리했다.
- 초심자 학습 자료를 docs/guides/에 추가했다(BEGINNER_GUIDE, examples/01~04.ddn, README, LEARNING_RESOURCES_INDEX).
- AI 학습 데이터 구조를 docs/context/ai_learning/에 정리하고 Self-Documented 예제와 오류 코퍼스(errors.jsonl)를 추가했다.
- AGENTS.md의 SSOT 버전 참조를 v20.1.15로 갱신하고 학습 자료 색인 링크를 추가했다.
- teul-cli lint --suggest-patch로 레거시 용어 교정 ddn.patch.json 생성 경로를 추가했다.
- teul-cli canon --emit fixits-json을 구현하고 pack/cli_fixits_json_basics + W90_G06 골든 테스트를 추가했다.
- teul-cli patch propose 명령과 W90_G07 골든 테스트를 추가했다.
- teul-cli ai prompt 명령을 추가하고 lean/runtime/full 프로파일 + zip/dir 번들 입력을 지원한다.
- ai prompt W91 골든(구조 검증/스냅샷)을 추가했다.
- 계약 위반 diag에 DR-069 표준 필드/문구와 fault_id를 적용하고 슬기 훅 기록 이벤트를 추가했다(W107 post 골든 포함).
- AI 친화 Phase0 데이터( pack/stdlib_examples, errors.jsonl, index.json )를 정리했다.
- 호출 문법을 `(인자) 이름`으로 고정하고 prefix `이름(인자)` 호출을 제거했다(예제/팩/골든 갱신).
- 미로 트랙을 `docs/EXAMPLES/tracks/`, 데모를 `docs/EXAMPLES/demos/`, 관문0 기록을 `docs/steps/000/`으로 통합하고 리뷰/템플릿 폴더를 기존 폴더로 흡수했다.
- docs/legacy(archive_root 포함)를 제거하고 관련 문서 참조를 정리했다.
- 설계 문서를 `docs/context/design/`으로 이동하고 참조를 갱신했다.
- SSOT 정본 내 `docs/ssot/ssot/SSOT_PENDING.md`를 제거하고 pending을 `docs/decisions/SSOT_PENDING.md`로 일원화했다.
- 로드맵 문서를 `docs/context/roadmap/`으로 이동하고 참조를 갱신했다.
- docs 구조를 통합 정리하고 보고서를 `docs/reports/(impl|audit)`로 이관, `docs/context/reports/`는 리다이렉트로 유지했다.
- 논리 연산자 `&&`/`||`와 `그리고`/`또는`, 부정 `아님` 설탕을 lang/tool에 반영했다.
- 계약에 `바탕으로(알림)`/`다짐하고(알림)` 모드를 추가하고, 위반 시 알림 기록 후 진행하도록 확장했다.
- teul-cli 계약(바탕으로/다짐하고/맞으면) 파싱/정본화/실행과 계약 위반 diag/W107 골든 테스트를 추가했다.
- 계약 위반 메시지/diag 표준과 `슬기.계약위반` 훅 결정을 SSOT 변경 기록에 반영했다.
- teul-cli 런타임 타입 불일치 메시지에 기대/실제 타입 표시를 추가하고 type_runtime_typecheck D-PACK을 보강했다.
- SSOT에 AGE1 런타임 타입검사 의미론(DR-067)을 반영했다.
- ddn_runtime에 PinSpec 런타임 타입검사와 `E_RUNTIME_TYPE_MISMATCH` 메시지/단위 테스트를 추가했다.
- 런타임 타입검사 D-PACK과 teul-cli W106 골든 테스트를 추가했다.
- 차림 인덱싱 설탕(a[i]/a[i] <- v)과 차림.값/차림.바꾼값 런타임, W103 골든 테스트를 추가했다.
- teul-cli에 차림 리터럴 `[]` 파싱/정본 출력과 W102 골든 테스트를 추가했다.
- `아니고 (조건) 일때` else-if 설탕과 `+<-`/`-<-` 복합 갱신 설탕을 추가하고 W104 골든 테스트를 보강했다.
- 중첩 차림 텐서 설탕(`[[...],[...]]` → 형상/자료/배치 묶음)을 추가하고 W105 골든 테스트를 추가했다.
- `(임자) 대상차림 모두 { ... }.` 설탕과 텐서 표준 함수(형상/자료/배치/값/바꾼값), W108/W109 골든 테스트, 관련 D-PACK을 추가했다.
- 차림 리터럴 `[]`/`[v1,...]` 파서/정본 전개 지원을 추가했다.
- 컨테이너 자원 저장/조회용 pack/age1_container_resource 골든/테스트를 추가했다.
- teul-cli ai prompt SSOT_VERSION을 v20.1.13로 갱신하고 골든을 재생성했다.
- ddn 런타임 자원에 차림/모음/짝맞춤 저장과 결정적 상태 해시(ResourceValue 직렬화)를 추가했다.
- ddn 런타임/teul-cli 순회 대상에 차림/모음/짝맞춤을 포함하고, 짝맞춤은 차림[열쇠, 값]으로 순회하도록 정리했다.
- teul-cli 반복/동안/에 대해/멈추기 문장과 예제를 추가했다.
- teul-cli 반복/순회 W97 골든 테스트를 추가했다.
- lang/tool 반복/순회/멈추기 지원을 추가했다.
- 컨테이너 타입 표기 `(T)차림` 형태를 파서/정본화에서 지원하도록 확장했다.
- `미분하기`/`적분하기`를 ddn.math/ext 표기 변환으로 지원했다.
- teul-cli 글무늬 맞추기(패턴 매칭)와 AGE1 템플릿 매칭 예제/팩을 추가했다.
- ddn.math/ext 예약 호출(sum/prod/diff/int) 파싱 허용과 Gate0 풀기 FATAL 처리, 예제/팩을 추가했다.
- 보개 관련 팩의 `?_??_?` 플레이스홀더를 `보개_그림판_가로/세로`로 정리했다.
- 보개 관련 팩 입력을 `보개_바탕색`으로 통일하고 W35 리플레이 하네스 키 표기를 한국어 별칭으로 갱신했다.
- teul-cli 샘 키보드 한국어 별칭과 보개 배경색 키(`보개_바탕색`)를 추가하고 테트리스 Slice0 키 표기를 한국어로 갱신했다.
- teul-cli patch 파이프라인 골든 테스트(W90)와 patch 예제 팩을 추가하고, 보개 목록 drawlist + tetris_board_drawlist로 테트리스 Slice0 보드 렌더링을 간소화했다.
- teul-cli patch preview/approve/apply/verify 파이프라인과 차림/모음/짝맞춤 컨테이너 연산(문자열 포함) 지원, AGE1 예제/팩을 반영했다.
- W35 replay verify(geoul 재주입 검증)와 W36 trace tier(T-OFF~T-FULL) 기록 확장을 반영했다.
- W34 geoul 블랙박스 로그(audit.ddni/idx/manifest) 기록과 teul-cli geoul seek/hash, D-PACK/골든을 반영했다.
- W01~W33 teul-cli 골든과 W23~W33 tool state_hash 스모크를 재검증했다.
- W33 teul-cli 골든 테스트 추가를 반영했다.
- W32 teul-cli 골든 테스트 추가와 W33 통합 시장 state_hash 기대값을 반영했다.
- W31 teul-cli 골든 테스트 추가와 W32 차분 뷰어 state_hash 기대값을 반영했다.
- W30 teul-cli 골든 테스트 추가와 W31 협업 마당 state_hash 기대값을 반영했다.
- W29 teul-cli 골든 테스트 추가와 W30 Q-Chain 승인 state_hash 기대값을 반영했다.
- W28 teul-cli 골든 테스트 추가와 W29 리액티브 상한 state_hash 기대값을 반영했다.
- W27 teul-cli 골든 테스트 추가와 W28 지표 스트림 state_hash 기대값을 반영했다.
- W27 불변 훅(늘지켜보고) state_hash 계산 경로와 팩 기대값을 반영했다.
- W26 경제 씨앗/거래 기록 state_hash 계산 경로와 팩 기대값을 반영했다.
- W25 쿼리 배치 state_hash 계산 경로(core gogae3)와 기대값 갱신을 반영했다.
- teul-cli W24/W25 state_hash를 core 계산 기준으로 동기화하고 골든 기대값을 갱신했다.
- core ECS 아키타입/컬럼 저장 전환과 W24 state_hash 테스트 갱신을 반영했다.
- W24/W25 입력샘 스키마/샘플과 sam 골든 테스트를 추가했다.
- W24/W25 D-PACK state_hash 기대값과 골든 테스트를 추가했다.
- W23 테스트 재검증을 완료하고 DoD 충족을 확인했다.
- W24/W25 D-PACK 현황을 점검하고 미완료로 기록했다.
- W22 골든 테스트 재검증을 완료하고 DoD 충족을 확인했다.
- teul-cli --sam에 detjson 입력샘 파싱/정렬과 샘 자원 요약 기록을 추가했다.
- CI에서 python3 설치/확인 단계를 보강했다.
- SSOT_PENDING_20260118 제안 문서의 깨진 구간을 복구했다.
- W23 입력샘 state_hash 기대값을 확정하고 정렬 기반 해시 테스트를 추가했다.
- UTF-8 검사 유틸을 CI에 연결하고 잔여 BOM 문서를 정리했다.
- UTF-8 검사 유틸을 추가하고 W23 입력샘 detjson 파서에 net_events 정렬을 적용했다.
- 비UTF-8 문서를 UTF-8로 정리하고 SSOT_PENDING(노트/결정) 깨진 구간을 복구했다.
- 저장소 내 UTF-8 BOM을 제거해 텍스트 파일 인코딩을 통일했다.
- BDL2 Q24.8 소수부 허용과 AA 플래그 렌더링 정책(AA=true 소수 유지, AA=false 정수 스냅)을 적용하고 W22 서브픽셀 골든 테스트를 추가했다.
- W22 BDL2 Circle/Arc/Curve 커맨드 인코더/디코더/뷰어 지원과 골든 테스트를 추가했다.
- golden ??? DDN? docs/ssot/pack ?? ?? ???? ???? W19/W21 stdout ??? ????.
- 보개 예제/팩 DDN을 생김새/결/보개_그림판_* 정본으로 갱신했다.
- 보개 drawlist 키 용어를 생김새/결/보개_그림판_* 정본으로 추가하고 기존 모양/트레잇/bogae_canvas_* 별칭을 허용했다.
- 보고서/예시/골든 정리: `golden/`과 `해보기/`를 정리하고 `docs/steps/000/artifacts/golden`, `docs/EXAMPLES/haebogi`로 이동했으며 `tools/ddonirang-vscode`는 `tools/vscode-ddn/legacy`로 옮겼다.
- W23 입력샘 DetJson v1 net_events 파서와 replay log net_events 기록/복원 경로를 추가했다.
- W23 착수: InputSnapshot net_events 필드/정렬 처리와 테스트를 추가했다.
- W26~W33 D-PACK 스켈레톤(pack/gogae3_w26~w33)을 생성했다.
- W23 입력샘(DetJson v1) 스키마/샘플과 W24/W25 D-PACK 스켈레톤을 추가했다.
- W23 D-PACK 스켈레톤(pack/gogae3_w23_network_sam) 착수 기록을 추가했다.
- W14/W15 골든 테스트 러너의 stdout 해시 라인 처리 규칙을 보정하고 재검증했다.
- W22 BDL2 제안서에 승인 준비 체크리스트를 추가했다.
- 관문0 기록 위치를 `docs/steps/000/`으로 통일했다.
- Slice0 보개 캔버스 높이를 352로 늘려 하단 줄 표시 문제를 수정했다.
- web/live 및 playback viewer에서 알파 0 스프라이트를 건너뛰어 빈 셀 덮임을 방지했다.
- sam-live 콘솔에서 Ctrl+C/Esc 종료 처리와 스프라이트 문자 매핑(@ → 기호)을 반영한다.
- 테트리스 Slice0 현재 조각 4블록 렌더링, 이동 DAS/ARR, 회전 킥, 게임오버 처리 반영을 기록한다.
- 테트리스 Slice0 보드 렌더링(잠금 블록 스프라이트)과 라인 삭제 점수/연출 텍스트를 반영한다.
- teul-cli `tetris_*` 보드 함수와 `}아닌것` 예제를 기록했다.
- teul-cli canon 분기 지원과 Slice0 낙하/고정 추가를 기록했다.
- teul-cli `고르기:` 분기 문장 및 Slice0 입력 갱신을 기록했다.
- teul-cli `일때/아니면` 분기 문장 추가 및 예제 갱신을 기록했다.
- 테트리스 Slice0 입력 스켈레톤(키 매핑/상태 모델) 갱신을 기록했다.
- teul-cli `--sam-live` 실시간 입력(콘솔/웹)과 viewer input 전달을 기록했다.
- BDL2 codec/CLI 플래그/뷰어 파서 추가를 기록했다.
- W22 구현 단계 계획과 BDL2 pack 테스트 정의를 보강했다.
- teul-cli `--bogae-live` 라이브 보개(콘솔/웹)과 viewer/live.html 폴링을 추가했다.
- W18 replay diff 명령과 state_hash 포함 playback manifest/pack 반영.
- W19 cmd_count cap/summary 정책 + diag 기록 + pack 반영.
- W20 BDL1 packet wrap/unwrap + roundtrip pack 반영.
- W21 overlay.detjson/뷰어 오버레이 + hash 불변 pack 반영.
- W22 BDL2 detbin 스키마/커맨드 테이블 확정 및 pack 문서 정비.
- W14~W17 재검증 로그를 반영했다.
- `TokenKind::Hash` 미사용 경고를 제거하고 수식 태그 처리를 Atom 기반으로 정리했다.
- W06 수식 주입 파싱 수정 및 골든 재검증 PASS 기록.
- W13 웹 산출물 골든 테스트(stdout/hash 기준 정리) PASS 기록.
- W14 헤드리스(stdout 1줄) 출력 포맷 및 골든 테스트 추가 기록.
- W13 웹(Canvas) 보개 뷰어/CLI 옵션/골든 테스트 추가 기록.
- W12 보개 drawlist(사각형/색상), 원자/묶음 리터럴, D-PACK 추가 기록.
- W12 보개 drawlist/BDL1 해시 출력과 `보개로 그려.` 문법 추가 기록.
- W01~W17 골든 재검증 기록 추가.
- W07~W11 골든 재검증 기록 추가.
- 로드맵은 설명용, 구현/검증은 WALK 스펙/골든 우선 원칙 반영.
- W06 골든 재검증 기록 추가.
- W05 골든 재검증 기록 추가.
- W04 골든 재검증 기록 추가.
- AGE 경로를 age0 기준으로 정정.
- SSOT 구조 변경(walks/age) 경로를 AGENTS/steps/status 문서에 반영.
- W03 골든 재검증 기록 추가.
- gaji/pack 입력 호환(`=` 대입, `(묶음)인 식 풀기`, bare 경로 보정) 기록 추가.
- step01 지시문 라인(`#...`) 무시 처리와 W01 골든 재검증 기록 추가.
- 걸음별 데모(docs/EXAMPLES/demos/001~011) 추가 및 EXAMPLES 안내 갱신.
- SSOT v20.0.3 기준으로 000 단계 문서 참조를 갱신(PLAN/RESULT/TESTS/HANDOFF).
- SYNTAX_BRIDGE/AI 코드블록 규칙/해시 표기 통일을 반영한 도구줄 문서 갱신.
- 99걸음(AGE0=1~33) 기획을 `docs/ssot/walks/gogae1/`와 `docs/ssot/age/age0/`에 유지하는 예외를 SSOT 변경 기록에 반영.
- AGE01 WALK01용 `tools/teul-cli` 독립 프로젝트와 W01 골든 테스트 러너를 추가.
- W03~W11 단계 문서와 INDEX 상태를 갱신.
- W12 글무늬 렌더링 단계 문서/예제 추가.
- W13~W14 단계 문서 추가(수식 거듭제곱, sin/cos 결정적 구현).

## 다음 협업에 필요한 문서(권장)
- AGENTS.md: 작업 규칙, 승인 필요 작업, 파일 위치 규칙.
- docs/README.md: 프로젝트 개요, 목표, 폴더 구조.
- docs/steps/INDEX.md: 단계별 진행 상태.
- docs/ARCHITECTURE.md: core/lang/tool 책임 분리와 데이터 흐름.
- docs/CLI_USAGE.md: 실행 명령 모음.
- docs/decisions/: 합의된 결정 기록(SSOT 변경 포함).
- docs/notes/: 합의 전 제안/질문 기록.
- docs/context/design/UNITS_v0.md: 단위 시스템 설계(정적 타입/차원 검증 제안).
- docs/context/design/RESOURCE_REGISTRY_v0.md: ResourceHandle/레지스트리 규격 초안.
- docs/status/LANG_STATUS.md: 또니랑 문법/런타임 지원 범위(지원/미지원/계획).
- docs/DECISIONS.md: 선택 이유(왜 그렇게 했는지) 기록.
- docs/KNOWN_ISSUES.md: 조사 처리, @ 접착자, 루프/필드 접근 미지원 등 제약.

## 레이어별 구조와 역할

### core/
- core/src/platform.rs
  - 역할: 엔진 공통 타입(입력 스냅샷, 누리/보개/거울 인터페이스 등).
  - 핵심: InputSnapshot에 last_key_name 추가(마지막 키 문자열).
  - 변경: PatchOp::EmitSignal 추가(차원고장/산술고장 기록).
  - 추가: Patch Origin 기록/Guard 위반 집행/diag 이벤트 생성.
- core/src/units.rs
  - 역할: UnitDim/UnitValue 및 단위 레지스트리.
- core/src/resource.rs
  - 역할: ResourceHandle 정의 및 핸들 산출 규칙.
- core/src/input.rs
  - 역할: 키 문자열을 엔진 입력 비트로 변환.
  - 변경: i/j/k/l 키 매핑 추가.
- core/src/signals.rs
  - 역할: 산술고장/차원고장 신호 정의 및 sink.

### lang/
- lang/src/lexer.rs
  - 역할: 소스 → 토큰화.
  - 변경: 문자열 이스케이프 처리(\n, \t, \r, \", \\), Gate0 키워드 추가(해보고/고르기/바탕으로/다짐하고/전제하에/보장하고/해서).
- lang/src/parser.rs
  - 역할: 토큰 → AST 파싱.
  - 변경: 일때/아니면(조건문), 비교/동등 연산, 셈씨 인식, <- 대입 파싱 안정화.
  - 변경: 호출 꼬리 동치/해석(~기/하기, ~고/하고, ~면/하면) 및 X/X하 충돌 오류 추가.
  - 변경: Thunk `{}` + 평가 표지, 파이프 `~기 해서` 호출식 제한/흐름값 주입, 해보고/고르기/계약 파싱 추가.
  - 변경: v19.4.19 글무늬/수식 주입 표기(`(<키=값>) 글무늬{...}`, `(키=값)인 식 풀기`) 파싱 및 금지 패턴 차단.
- lang/src/ast.rs
  - 역할: AST 정의.
  - 변경: Stmt::If/Try/Choose/Contract, ExprKind::Thunk/Eval/Pipe/FlowValue 추가.
- lang/src/normalizer.rs
  - 역할: 정본화 출력.
  - 변경: 값함수 → 셈씨로 정규화, Thunk/파이프/계약 출력 추가.
  - 변경: v19.4.19 주입 표기 정본화(`(<키=값>) 글무늬{...}`, `(키=값)인 식 풀기`).
- lang/src/runtime.rs
  - 역할: 런타임 값(Value)과 표준 함수 실행 지원.
- lang/src/stdlib.rs
  - 역할: 표준 라이브러리 시그니처 목록.
  - 변경: 글바꾸기, RNG(무작위/무작위정수/무작위선택) 추가.
- lang/src/lib.rs
  - 역할: parse/normalize 편의 API와 통합 테스트.

### tool/
- tool/src/ddn_runtime.rs
  - 역할: ddn 인터프리터(표현식 평가, 함수 호출, 자원 갱신).
  - 핵심 함수:
    - DdnProgram::from_source: 소스 파싱 + 오류 메시지 포맷.
    - DdnRunner::run_update: 입력/월드 기반 ddn 업데이트 실행.
    - EvalContext::eval_expr/eval_stmt/eval_call: 핵심 평가 루틴.
    - value_to_index/value_to_string: 값 변환.
  - 추가: Thunk 평가, 파이프 실행, 해보고/고르기/계약, RNG 시드 기반 함수 처리.
- tool/src/preprocess.rs
  - ??: `??()`/`??{}` ??? ? `!!{}` ?? ??/??.
- tool/src/schema.rs
  - 역할: `ddn.schema.json` 생성(씨앗/핀/타입/단위/쓸감 요약).
- tool/src/gate0_registry.rs
  - 역할: 쓸감/단위 곳간 로더 및 Gate0 Strict 검증(레거시 ddn.resource.json 경고 포함).
- tool/src/main.rs
  - 역할: CLI 진입점, 데모 실행, 렌더링.
  - 핵심 함수:
    - run_maze_ddn / run_maze_ddn_live: ddn 기반 미로 실행.
    - DdnMazeIyagi::run_update: ddn 스크립트와 엔진 연결.
    - map_text_from_data: 초기 맵 텍스트 생성(G/C 포함).
    - read_text_from_path: UTF-8/UTF-16 자동 디코드.
  - 추가: preprocess-ai/build-schema 명령.
  - 추가: diag 이벤트를 `geoul.diag.jsonl`로 기록.
  - 추가: `teul-cli test`(`golden/*.test.json`) 실행 및 repro 생성, `geoul query` 지원.
- tools/vscode-ddn
  - 역할: VS Code용 ddn 문법 하이라이팅(TextMate grammar).
  - 구성: package.json, syntaxes/ddn.tmLanguage.json, language-configuration.json.

## 함수 상세(관문0 범위)

### core/src/platform.rs
- NuriWorld::new: 빈 월드 생성.
- NuriWorld::spawn: 엔티티 ID 발급.
- NuriWorld::set_component_json/remove_component: 엔티티 컴포넌트 업데이트/삭제.
- NuriWorld::get_component_json: 엔티티 컴포넌트 조회.
- NuriWorld::set_resource_json/get_resource_json: 문자열 자원 설정/조회.
- NuriWorld::set_resource_fixed64/get_resource_fixed64: 수치 자원 설정/조회.
- NuriWorld::set_resource_handle/get_resource_handle: 자원핸들 저장/조회.
- NuriWorld::state_hash: 월드 상태의 결정적 해시(BLAKE3).
- Signal::kind_u32/stable_payload_hash/stable_sort_key: 신호 정렬/결정성 보장.
- InputSnapshot::is_key_pressed: 키 비트로 눌림 여부 확인.
- Sam/Iyagi/Nuri/Bogae/Geoul/Seulgi 트레이트 메서드: 엔진 루프의 역할 경계를 정의.
- DetSam::new/begin_tick: 입력 스냅샷 생성(키 비트, last_key_name 포함).
- DetNuri::new/world/apply_patch: 패치 적용과 월드 접근.
- InMemoryGeoul::record/replay_next: 틱 프레임 기록/재생.

### core/src/input.rs
- key_bit_from_name: 키 이름 문자열 → 엔진 키 비트(방향키/wasd/ijkl/arrow).
- is_key_pressed: 키 비트마스크에서 눌림 확인.
- is_key_just_pressed: 직전 틱 대비 새 눌림 확인.

### lang/src/lexer.rs
- Lexer::tokenize: 전체 소스를 토큰 스트림으로 변환.
- Lexer::next_token: 문자에 따라 토큰 분기.
- Lexer::read_hangul/read_ascii: 식별자/키워드 인식, 단순 조사 분리.
- Lexer::read_formula_block: `수식{...}` 특수블록 본문 추출.
- Lexer::read_string: 문자열 리터럴(이스케이프 포함) 처리.
- Lexer::read_number: 정수/실수 판별.
- Lexer::read_atom/read_variable: #원자, ?변수 토큰 처리.
- LexError: 렉서 오류 타입.

### lang/src/parser.rs
- Parser::parse_program: 소스 → CanonProgram.
- Parser::parse_seed_def/parse_params/parse_seed_kind: 씨앗 정의/파라미터/종류 파싱.
- Parser::parse_body/parse_stmt: 문장 블록과 문장 파싱(대입/반환/조건).
- Parser::parse_expr + comparison chain: 연산자 우선순위 파싱.
- Parser::parse_formula_block: `수식{...}` 특수블록 파싱 + 태그/비ASCII 검증.
- Parser::parse_primary: 기본 표현식/후위 호출 파싱.
- Parser::consume_arrow: '<-' 대입 처리(분리 토큰 대응 포함).
- ParseError: 파서 오류 타입.

### lang/src/ast.rs
- Stmt::If: 조건/분기 구조를 표현.
- SeedDef/Expr/ExprKind 등: 문법 트리의 기본 구성 요소(Formula 포함).

### lang/src/normalizer.rs
- Normalizer::normalize_program: 표준 표기 문자열로 출력.
- 셈씨 표준화: 값함수/셈씨 용어 통일.

### lang/src/runtime.rs
- Value: 런타임 값 타입(수/단위값/글/참거짓/목록/없음/자원핸들).
- InputState: 키 입력 스냅샷.
- input_pressed/input_just_pressed: 키 입력 검사.
- list_len/list_nth/list_add/list_remove: 목록 유틸리티.
- string_len/string_concat/string_split/string_join: 문자열 유틸리티.
- RuntimeError: 타입 오류 등 런타임 오류.

### lang/src/stdlib.rs
- string_function_sigs/list_function_sigs/input_function_sigs: 표준 함수 시그니처 정의.
- minimal_stdlib_sigs: 사용 가능한 표준 함수 목록 합성.
- resource_function_sigs: 자원 핸들 생성((글) 자원) 시그니처 추가.
- transform_function_sigs: `채우기`/`풀기` 변환 함수 시그니처 포함.

### lang/src/lib.rs
- parse: 소스 → AST 파이프라인.
- parse_and_normalize: 소스 → 정본화 문자열.

### tool/src/ddn_runtime.rs
- DdnProgram::from_source: ddn 소스 파싱 + 오류 위치 표시.
- DdnRunner::run_update: 입력/월드/기본값을 넣고 ddn 업데이트 실행.
- EvalContext::eval_seed/body/stmt/expr: 실행기 핵심.
- EvalContext::eval_call: 표준 함수 호출 디스패치(글바꾸기 포함).
- EvalContext::eval_call: `풀기` 수식 평가 + 엄격 주입 검증 + `#latex` FATAL 처리.
- EvalContext::eval_pipe: 파이프 실행과 흐름값 주입 처리.
- RNG: InputSnapshot 시드 기반 무작위/무작위정수/무작위선택 지원.
- 차원 불일치: 대입 무효화 + 차원고장 신호(EmitSignal) 처리.
- EvalContext::resource_exists/get_resource/set_resource: 자원 조회/갱신.
- @단위/자원핸들 처리: UnitValue/ResourceHandle 생성과 단위 태그 자원 변환.
- value_to_index/value_to_string: 타입 변환.
- is_truthy: 조건식 평가.
- format_parse_error: 라인/컬럼 표시 파싱 오류 포맷.

### tool/src/main.rs
- main: CLI 커맨드 분기(run-...).
- parse_maze: 텍스트 맵 → 타일/시작점/목표/코인 파싱.
- map_text_from_tiles/map_text_from_data: 타일/코인/목표를 포함한 맵 문자열 생성.
- read_text_from_path/read_text_from_stdin: 파일/표준입력 텍스트 디코딩(UTF-8/UTF-16).
- decode_stdin/decode_utf16_le/looks_like_utf16le: 인코딩 판별 유틸.
- read_key_for_tick: 라이브 입력 폴링 및 키/종료 신호 추출.
- move_to_key_bits/move_to_key_name: 이동 문자 → 키 비트/문자열.
- DdnMazeIyagi::run_update: ddn 결과로 월드 패치 + 맵 렌더.
- run_maze_ddn/run_maze_ddn_live: ddn 미로 실행(파일/라이브 입력).
- run_maze_file/run_maze_replay/run_maze_record/run_maze_replay_file: 기존 미로 실행/리플레이.
- ConsoleBogae/LiveBogae/LiveTerminalGuard: 화면 출력과 터미널 모드 전환.

## 최근 작업 요약(핵심 변경)
- `(임자) 대상차림 모두 { ... }.` 설탕과 텐서 표준 함수(형상/자료/배치/값/바꾼값), W108/W109 골든 테스트, 관련 D-PACK을 추가했다.
- teul-cli에 `반복:`/`동안:`/`에 대해:` 문장과 `멈추기`를 추가하고 반복 예제를 보강했다.
- teul-cli 반복/순회 W97 골든 테스트를 추가했다.
- lang/tool에 반복/순회/멈추기 실행 경로를 추가했다.
- Slice0 보개 캔버스 높이를 조정해 하단 줄이 잘리지 않도록 개선.
- web/live 및 playback viewer에서 알파 0 스프라이트 렌더링을 건너뛰도록 수정.
- sam-live 콘솔 입력에서 Ctrl+C/Esc로 중지할 수 있도록 stop 플래그를 연결하고, 콘솔 렌더러가 스프라이트 URI에서 문자(glyph)를 추출하도록 개선했다.
- 테트리스 Slice0에 현재 조각 4블록 렌더링, 이동 DAS/ARR, 회전 킥, 게임오버 처리를 추가했다.
- teul-cli `tetris_board_cell` 내장 함수 추가와 Slice0 보드 잠금 블록 스프라이트/라인 삭제 점수·연출을 반영한다.
- teul-cli에 `tetris_*` 보드 헬퍼와 `묶음값`을 추가해 Slice0 충돌/라인삭제 스켈레톤을 구성.
- `}아닌것` 조건 예제를 추가.
- teul-cli canon에 `일때/고르기` 분기 정본화와 `{...}인것` 조건 처리를 추가.
- Slice0 입력에 기본 낙하 카운트/잠금 흐름을 추가.
- teul-cli에 `고르기:` 분기 파싱/실행을 추가하고 Slice0 입력을 `일때/고르기`로 갱신.
- teul-cli에 `일때/아니면` 분기 파싱/실행과 조건 평가 규칙(차원 없는 수만 허용)을 추가.
- pack/game_maker_tetris_slice0 입력 스켈레톤에 키 매핑/상태 모델/홀드/회전 로직을 추가.
- `--sam-live console|web`로 실시간 입력을 연결하고 웹 뷰어 input= 쿼리 전달을 추가.
- BDL2 detbin 코덱, `--bogae-codec` 플래그, viewer BDL2 파서 추가.
- W22 구현 단계 계획과 BDL2 pack 테스트/진단 정의를 보강.
- `--bogae-live`로 실행 중 프레임을 콘솔/웹에 실시간 표시하고 live.html 폴링 뷰어를 추가.
- W18 replay diff 명령 + playback manifest state_hash 포함 + pack 추가.
- W19 cmd_count cap/summary 정책 + diag 기록 + pack 추가.
- W20 BDL1 packet wrap/unwrap + roundtrip pack 추가.
- W21 overlay.detjson/뷰어 오버레이 + hash 불변 pack 추가.
- W22 BDL2 detbin 스키마/커맨드 테이블 확정 및 pack 문서 정비.
- `--no-open` 헤드리스 모드에서 stdout 1줄 포맷(`bogae_hash`, `cmd_count`, `codec=BDL1`)을 고정.
- teul-cli에 웹(Canvas) 보개 뷰어(ddn view)와 `--bogae web/--bogae-out` 아티팩트 생성 경로를 추가.
- 보개 drawlist를 상태(`모양.트레잇`/`모양.네모`/`모양.채움색`) 기반으로 생성하고, 원자(#...)·묶음((x=..., y=...)) 리터럴과 W12 D-PACK/골든을 추가.
- W12 보개 drawlist/BDL1 해시 산출(`bogae_hash`)과 `--bogae-out` 출력 경로 추가.
- gaji/pack 입력을 위한 `=` 대입, `(묶음)인 식 풀기`, bare 경로 `살림.` 보정 추가.
- teul-cli lexer/canon에서 줄 시작 `#` 지시문 라인을 주석처럼 무시.
- teul-cli 글무늬 템플릿 렌더링(자리표시자/포맷/단위 변환) 추가.
- 수식(MathIR) 파서/평가 추가(#ascii/#ascii1) 및 math stdlib(abs/min/max/clamp/sqrt/powi) 연결.
- Fixed64 sqrt 구현 추가.
- 훅 실행 엔진 추가: (시작)할때, (매마디)마다 + --ticks/--madi 지원.
- diag JSONL 옵션 별칭(--diag) 정리 및 상태 해시 영향 없음 확인.
- repro 자동 기록 `--enable-repro` 추가, repro JSON에 ssot_version/repro_command 기록.
- `teul-cli ai extract` 서브커맨드 추가(??() 구멍 추출).
- canon `--out` 경로 반영 및 run_golden multi 파이프라인 지원.
- W03~W11 골든 테스트 추가/통과.
- SSOT 기준을 v19.4.19로 상향하고(꺾쇠 금지, `$` 뉘앙스, `글무늬` 템플릿, 계약 정본 `바탕으로/다짐하고`) 변경점을 문서에 반영.
- v19.4.19 Gate0 규칙(`}해서`/`-서` FATAL, 뉘앙스 `$`, `글무늬` 템플릿, `늘지켜보고`, `핀=값`, 계약 정본 `바탕으로/다짐하고`) 구현 반영.
- 기본 단위 목록(mm/us/min/h/g) 보강.
- OneDrive 환경 대응: cargo `target-dir`을 `C:\dev\cargo-target\ddonilang`로 고정.
- state_hash(BLAKE3)로 전환하고 world_hash 기반 로그/골든 포맷을 정합.
- 단계별 기록을 `docs/steps/〈단계〉/` 구조로 전환하고 관문 0을 000 단계로 분리.
- 문서/예시의 제네릭 꺾쇠 표기를 〈〉로 정리.
- 관문 0 개발 참조 체크리스트(v19.4.19)를 단계 000 참고로 정리.
- SSOT v18 기준 문서 참조를 정렬하고 LANG_STATUS에 미구현 항목을 명시.
- 골든/리플레이 산출물을 `docs/steps/000/artifacts/`로 이동하고 도구 기본 경로를 맞춤.
- 11단계 데모 트랙 문서를 `docs/EXAMPLES/tracks/11단계/`로 정리.
- DetMath LUT 매니페스트/로더 추가 및 구동 전 해시 검증 연결.
- DetMath 매니페스트 누락 항목 부정 테스트 추가.
- DivAssignResourceFixed64: div0 시 대입 커밋 생략(미존재 자원 생성 방지).
- 알림 16패스 제한/이월 루프 최소 구현 및 데모 로그 추가.
- 한국어 활용/표면화 골든 테스트 추가.
- 접미 체인 파싱/바인딩 골든 테스트 추가.
- 선택 인자 기본값 5종 테스트 추가.
- CLI run-once: 수/글/참거짓 선언 리터럴을 자원으로 저장.
- ddn 인터프리터 도입: tool/src/ddn_runtime.rs 신규.
- ddn 미로 실행 커맨드 추가: run-maze-ddn, run-maze-ddn-live.
- 입력키 전달: InputSnapshot.last_key_name 및 ddn에서 입력키 리소스 제공.
- 미로 렌더링 개선: DdnMazeIyagi가 맵원본 기반으로 맵을 그려 코인 제거가 반영됨.
- 문자열 이스케이프 지원: \n 처리로 줄목록 분리 정확화.
- 문법 확장: 조건문(일때/아니면), 비교 연산, 셈씨 표준화.
- 글바꾸기 내장 추가: ddn만으로 맵 문자열의 특정 위치 치환 가능.
- 조사 토큰/접착자(`@`/`:`/`~`) 정본화 경로 추가, 선택 인자 기본값/없음 주입, 조사 바인딩 우선 처리 및 `:핀` 고정, 문장 종결 어조(`?`/`!`) + 어미 접미 기반 어조 추정 반영.
- `@단위`/`@"자원"` 접미를 표현식에서 파싱하도록 확장(단위 레지스트리 기반 검증 포함).
- 단위 레지스트리 확장 + UnitDim/UnitValue 런타임 검증, 자원핸들(ResourceHandle) 저장/조회 경로 추가.
- SSOT v19.4.19 세트 기준 문서 참조를 최신으로 동기화.
- 정적 차원 불일치 검증 추가 및 단위 레지스트리 확장(평/inch/ft/kmh/mps/KRW/USD).
- 쓸감/단위 곳간 로더 및 기본 레지스트리(ddn.asset.json/ddn.units.json) 추가.
- `??()`/`??{}` ??? ? `!!{}` ?? ??(preprocess-ai) ??.
- `수식{...}` 특수블록/`풀기` 평가 경로 추가(#ascii/#ascii1 평가, 엄격 주입 검증, `#latex` FATAL).
- `ddn.schema.json` 생성기/`build-schema` 명령 추가.
- Patch Origin 기록/Guard 위반 집행/diag 이벤트(`geoul.diag.jsonl`) 기록 추가.
- `teul-cli test`/`golden/*.test.json` 테스트 러너 + repro 생성 + `geoul query` 추가.
- VS Code 문법 하이라이팅: TextMate 기반 ddn 하이라이터 추가.
- 하이라이터 확장: @ 접착자/단위/자원 리터럴 패턴 강화.
- 호출 꼬리 동치/해석 규칙 추가 및 X/X하 씨앗 이름 충돌 오류 처리.
- 레거시 ddn.resource.json 입력 별칭 경고 지원.
- Gate0 Thunk/eval 표지, 파이프 call-only + 흐름값 주입, 해보고/고르기/계약 파싱 추가.
- 결정적 RNG 함수(무작위/무작위정수/무작위선택) 및 검증 영역 RNG 금지 규칙 반영.
- 차원 불일치 런타임 안전망(대입 무효화 + 차원고장 신호) 추가.
- Gate0 골든 시나리오(G0-PIPE/UNIT/FAULT) 및 전용 스크립트 추가.
- Fixed64 raw_i64 결정성 벡터 테스트 추가.
- NAME-LINT-01/TERM-LINT-01: 파서 즉시 오류 + 정본화기 LEGACY 경고/추천(자동 치환 제거).
- 산술 고장 diag에 source_span/expr.tag(arith:*) 기록 추가.
- 계약 위반을 Signal::Diag로 기록(CONTRACT_PRE/POST + sub_reason/mode/contract_kind/message).
- repro 보고서에 lint_version/term_map_version 기록 추가.

## ddn 표준 라이브러리(현재 지원)
- 문자열
  - (글) 길이 -> 정수
  - (글, 구분) 자르기 -> 목록〈글〉
  - (글, 글) 합치기 -> 글
  - (목록〈글〉, 구분) 붙이기 -> 글
  - (값) 글로 -> 글
  - (글, 인덱스, 새글) 글바꾸기 -> 글
    - 인덱스는 0-based.
    - 범위 밖이면 원본 글을 반환.
- 목록
  - (...요소) 목록 -> 목록
  - (목록, 인덱스) 번째 -> 요소
  - (차림, 인덱스) 차림.값 -> 값?
  - (차림, 인덱스, 값) 차림.바꾼값 -> 차림
  - (목록, 값) 추가 -> 목록
  - (목록, 인덱스) 제거 -> 목록
- 입력
  - (키) 눌렸나 -> 참/거짓
  - (키) 막눌렸나 -> 참/거짓
- RNG
  - () 무작위 -> 수
  - (최소, 최대) 무작위정수 -> 정수
  - (목록) 무작위선택 -> 요소
- 자원
  - (글) 자원 -> 자원핸들
- 수식
  - (수식값, 묶음) 풀기 -> 수

## 언어/문법 요약(현재 지원)
- 씨앗 정의: (파라미터들) 이름:셈씨 = { ... } 형태.
- 문장 종료: `.`, `?`, `!`로 끝난다(어미 접미 기반 어조 추정 포함).
- 대입: 변수 <- 값.
- 차림 인덱싱 설탕: a[i]/a[i] <- v는 차림.값/차림.바꾼값 호출로 정본화.
- 조건: (조건) 일때 { ... } 아니면 { ... }
- 안은문장: `{ }` Thunk + `}한것/}인것/}아닌것/}하고` 평가 표지.
- 파이프: `~기 해서` 체인(호출식만 허용, 흐름값 주입).
- 상태 대조/분기: `해보고:` + `고르기:`(아니면 필수).
- 계약: `{...}인것 바탕으로/다짐하고` + 아니면/맞으면 블록(별칭: 전제하에/보장하고).
- 호출: (인자1, 인자2) 함수이름 (prefix `이름(인자)` 금지)
- 비교: ==, !=, <, <=, >, >=

## ddn 미로 스크립트 흐름(docs/EXAMPLES/tracks/11단계/artifacts/scripts/미로.ddn, docs/EXAMPLES/tracks/11단계/artifacts/scripts/maze_v0.ddn)
- 입력키를 읽어 이동 방향 결정.
- 벽인지로 충돌 검사 후 플레이어 좌표 갱신.
- 코인(C) 처리:
  - 너비 = 첫 줄 길이
  - 인덱스 = 플레이어_y * (너비 + 1) + 플레이어_x
  - 맵원본 <- (맵원본, 인덱스, ".") 글바꾸기
  - 점수 +<- 1
  - 줄바꿈을 포함한 문자열 인덱싱이라 +1 보정이 필요.

## 예제 스크립트

### 입력키를 메시지로 보여주기
```ddn
매틱:움직씨 = {
    키 <- (입력키).
    메시지 <- (키) 글로.
}
```

### 코인 줍기(맵 문자열 갱신)
```ddn
(((맵원본, 플레이어_x, 플레이어_y) 칸) == "C") 일때 {
    너비 <- (((맵원본) 줄목록, 0) 번째) 길이.
    인덱스 <- (플레이어_y * (너비 + 1) + 플레이어_x).
    맵원본 <- (맵원본, 인덱스, ".") 글바꾸기.
    점수 <- 점수 + 1.
}
```

### 글바꾸기 기본 예제
```ddn
문장 <- "hello".
문장 <- (문장, 0, "H") 글바꾸기.
메시지 <- 문장.
```

## 주요 결정/선택 이유
- 입력키(last_key_name) 추가:
  - 입력 시스템을 키 비트에 고정하지 않고, ddn에서 임의 키를 다루기 위해 필요.
- 글바꾸기 도입:
  - 목록 자원을 저장할 수 없는 제약 때문에 문자열 치환 방식이 가장 작고 안전.
- 맵원본 기반 렌더링:
  - ddn에서 맵을 변경하면 화면에도 즉시 반영되어야 함.
- 문자열 이스케이프 지원:
  - "\n"을 실제 줄바꿈으로 처리해야 줄목록/맵 파싱이 정상 동작.
- 셈씨 표준화:
  - 용어 일관성과 향후 문서 정합성을 위해 값함수 대신 셈씨로 통일.

## 현재 제약/미지원
- 조사 처리: 단순 접미 분리 + `:핀`/`~조사`만 지원, 오탐 가능.
- 단위 레지스트리는 평/inch/ft/kmh/mps/KRW/USD까지 확장(경계 입력 런타임 안전망 구현).
- ddn 런타임 순회는 차림/모음/짝맞춤을 지원(짝맞춤은 차림[열쇠, 값] 반환).
- 필드 접근(a.b) 미지원.
- 묶음/수식/글무늬 값 자원 저장 미지원.

## 실행 방법
- ddn 미로(라이브 키 입력)
  - cargo run -p ddonirang-tool -- run-maze-ddn-live docs/EXAMPLES/tracks/11단계/artifacts/maps/maze.txt docs/EXAMPLES/tracks/11단계/artifacts/scripts/미로.ddn
- ddn 미로(이동 문자열)
  - cargo run -p ddonirang-tool -- run-maze-ddn docs/EXAMPLES/tracks/11단계/artifacts/maps/maze.txt docs/EXAMPLES/tracks/11단계/artifacts/scripts/미로.ddn ijkl

- ai prompt(컨텍스트 출력)
  - cargo run -p ddonirang-tool -- ai prompt --profile lean --out build/ai.prompt.txt
  - cargo run -p ddonirang-tool -- ai prompt --profile lean --bundle SSOT_bundle_v19.4.19_codex.zip --out build/ai.prompt.txt
## 다음 단계 후보(요약)
- SSOT v19.4.19 보완 항목(`}해서`, `-서` FATAL, `$` 뉘앙스, `글무늬`, 정의/호출 분리, `핀=값` 정본, 계약 정본 `바탕으로/다짐하고`) 반영.
- teul-cli test + `golden/*.test.json`(DetTest/TraceTest) ?? ?? ??.
- 필드 접근 지원.
- 묶음/수식/글무늬 자원 저장 범위 정리.

## 추가로 넣으면 좋은 것
- 필요 시 보완: 설계/결정 문서의 최신화 및 누락된 한계 사항 추가.



