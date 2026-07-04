# CODEX 작업 큐 (2026-07-06) — 순차 자율 실행, 대용량

> 프로토콜은 `QUEUE_CODEX_20260705.md`와 동일: 브랜치 `codex/queue-20260706`에 작업 1건=커밋 1개, main 직접 커밋 금지, 신규 보고서는 `git add -f`(docs/ gitignore).
> Q13~Q18(커널 구현 시리즈)은 **여전히 게이트 잠김** — 이 큐에는 포함하지 않는다. 적합성 팩(Claude 작성)이 커밋되고 별도 게이트 해제 기록이 있어야 착수 가능.
> 이 큐는 순수 감사·실측·초안·정리 작업만 담는다 — 전부 언어 의미론 결정을 요구하지 않는다.

## 프로토콜 요약
1. 브랜치 `codex/queue-20260706` 생성(시작 전).
2. 작업 1건 완료 시 자기 검증 통과 확인 후 브랜치에 커밋 1개(`[Q<번호>] <요약>`).
3. 자기 검증 실패 시 원복 + 보고 기록 + 의존 없는 다음 작업으로.
4. 공통 금지: main 커밋 / push / golden --update / 파일 삭제 / allowlist 변경 / 네트워크 / 브리프 범위 밖 수정.
5. 각 작업의 산출물은 `docs/context/reports/`에 새 보고서로. 기존 보고서 수정은 명시된 것만.
6. 큐 소진 시 이 파일 하단에 완료 보고.

---

## Q12 — Fixed64 수치 봉투 실측 (커널 스펙 §6 요구사항)

**목표:** LANG_KERNEL_V1 스펙 §6("표현 범위·최소 눈금·초등함수 정밀도 보장 자릿수 [실측 작업 필요]")을 실제 숫자로 채운다.

**작업:**
1. `tools/teul-cli/src/core/fixed64.rs`의 `Fixed64`(Q31.32) 기준으로 다음을 코드에서 직접 계산/도출(추측 금지, 상수·구현에서 유도):
   - 표현 가능 범위(최대/최소값, 10진 근사)
   - 최소 눈금(2^-32의 10진 근사)
   - `sqrt`(`int_sqrt` 기반) 오차 상한을 정의역 몇 구간(예: [0,1), [1,100), [100,1e6))으로 나눠 측정 — 실제 값과 f64 기준값을 비교하는 테스트 하네스 작성
2. `lang/src/stdlib.rs` 또는 `tools/teul-cli/src/runtime/eval.rs`에 있는 sin/cos/exp/log 초등함수 구현을 찾아 동일 방식으로 정의역별 오차 측정 (구현이 없으면 "미구현"으로 명시 — 발명 금지)
3. 오버플로 경계값(`i64::MAX`/`MIN` 근접)에서의 현재 동작(포화)을 재확인하고 표로 정리.

**산출물:** `docs/context/reports/FIXED64_NUMERIC_ENVELOPE_V1.md`
스키마: `| 함수/연산 | 정의역 구간 | 측정 방법 | 오차 상한(측정값) | 비고 |`

**수용 기준:**
- [ ] 모든 수치가 실행 가능한 테스트 하네스(임시 Rust 테스트 또는 scratch 스크립트)로 측정됨, 추정치 금지
- [ ] 하네스 소스는 보고서에 인라인 또는 경로로 첨부
- [ ] `cargo test --manifest-path tools/teul-cli/Cargo.toml` PASS (기존 회귀 없음)
- [ ] 미구현 함수는 "미구현"으로 명시, 임의 수치 기입 금지

---

## Q13(신) — 값 모델 이중화 감사 (커널 스펙 §2.1 "부채 기록")

**목표:** `lang/src/runtime.rs::Value`(16종 추정)와 `tools/teul-cli/src/core/value.rs::Value`(14종 추정)의 정확한 diff를 만든다.

**작업:**
1. 두 enum의 전체 variant를 각각 나열(코드에서 직접 추출).
2. 각 variant가 반대편에 대응이 있는지, 이름이 다른지, 아예 없는지 분류.
3. 두 경로가 "같은 입력에 대해 같은 동작"을 하는지 확인할 수 있는 기존 테스트가 있는지 조사(없으면 "부재"로 명시).

**산출물:** `docs/context/reports/VALUE_MODEL_DUALITY_AUDIT_V1.md`
스키마: `| variant(lang) | variant(teul-cli) | 대응 상태(동일/이름다름/한쪽만존재) | 동작 동등성 테스트 존재 여부 |`

**수용 기준:**
- [ ] 두 enum 전체 variant 커버(빠짐없이)
- [ ] "한쪽만 존재" 항목은 어느 쪽 코드 경로에서 실제로 쓰이는지 근거(파일:행)
- [ ] 수정·통합 실행 금지 — 감사만

---

## Q14 — lesson 파일 전수 스타일 감사 (커널 §3.2 조건문 전환 준비)

**목표:** `{...}인것 일때` 등 구형 표면이 실제 lesson/pack에 얼마나 남아있는지, 전환 범위를 파악한다.

**작업:**
1. `solutions/seamgrim_ui_mvp/lessons/**/*.ddn`과 `pack/**/*.ddn` 전수를 정적 검색:
   - `인것 일때` 패턴 사용 빈도
   - `만약/이면/아니면` 신 표면 사용 빈도
   - `인 동안` 사용 빈도
2. 파일별 카운트 + 전체 합계.
3. 기본값 중복 재대입(`채비{}`에서 선언한 값을 본문에서 또 대입하는 패턴), 불필요한 이중 괄호 등 표면 어색함도 별도로 카운트(정규식 기반, 완벽할 필요 없음 — 대략적 규모 파악 목적).

**산출물:** `docs/context/reports/LESSON_STYLE_AUDIT_V1.md`
스키마: `| 파일 경로 | 인것일때 개수 | 만약/이면 개수 | 인동안 개수 | 비고(어색 패턴 발견 시) |` + 상단에 전체 합계 요약.

**수용 기준:**
- [ ] `.ddn` 파일 전수 커버(빠짐없음, 총 파일 수 명시)
- [ ] 수정 실행 금지 — 감사만, 이후 전환 작업 규모 산정용

---

## Q15 — 깨진 체크 173건 근본 원인 세분화 (Q3 후속)

**목표:** `BROKEN_CHECKS_AUDIT_V1.md`의 173건을 "이 문서가 애초에 이 프로젝트에 존재한 적이 있는지"까지 파고들어 분류를 정교화한다.

**작업:** 173건 각각에 대해:
1. 참조된 `.md` 문서명이 과거 어느 커밋에서도 존재한 적 있는지(`git log --all --diff-filter=A -- <파일명>` 유사 검색으로 생성 이력 자체가 있는지) — 기존 Q3는 "현재 워킹트리에 없음"만 확인했지 "생성 이력 자체가 없음"까지는 안 봤을 수 있음, 재확인.
2. 생성 이력이 있다면 어느 커밋에서 삭제됐는지 특정.
3. 생성 이력이 아예 없다면 "계획만 되고 실행 안 된 항목"으로 별도 분류.
4. 각 체크 파일이 어떤 미션/브리프/큐 항목과 연결되는지(파일명 패턴으로 추정 가능하면 기록).

**산출물:** `docs/context/reports/BROKEN_CHECKS_ROOT_CAUSE_V1.md` (기존 BROKEN_CHECKS_AUDIT_V1.md는 수정하지 않음, 신규 파일)
스키마: `| 체크 파일 | 참조 문서 | 생성 이력(있음/없음) | 삭제 커밋(있으면) | 분류(계획후미실행/생성후삭제) | 연관 추정 작업 |`

**수용 기준:**
- [ ] 173건 전수(Q3 목록 기준)
- [ ] "생성 이력" 판정은 `git log --all` 근거 제시
- [ ] 삭제·수리 실행 금지 — 분류만

---

## Q16 — UI 제품 연결 모듈 91개 런타임 로드 확인

**목표:** `UI_RUNNER_DEPENDENCY_MAP_V1.md`에서 "제품" 분류된 91개 모듈이 실제로 브라우저에서 에러 없이 로드되는지 정적 분석을 넘어 실행 확인한다.

**작업:**
1. 기존 프로젝트에 있는 브라우저 실행 도구(playwright 등, package.json에 playwright 존재 확인됨)를 사용해 `solutions/seamgrim_ui_mvp/ui/index.html`을 헤드리스로 로드.
2. 콘솔 에러/경고, 404(모듈 로드 실패) 수집.
3. "동적의심"으로 표기된 54개 항목이 실제로 로드되는 경로인지, 죽은 코드인지 실행 기반으로 재확인.

**산출물:** `docs/context/reports/UI_RUNTIME_LOAD_CHECK_V1.md`
스키마: `| 모듈 | 정적분류(기존) | 런타임 로드 확인(성공/실패/미도달) | 콘솔 에러 내용 |`

**수용 기준:**
- [ ] 91개 "제품" 분류 항목 + 54개 "동적의심" 항목 전수
- [ ] 실행 로그/스크린샷 등 증빙 경로 첨부
- [ ] UI 코드 수정 금지 — 확인만

---

## Q17 — SSOT 개정 제안 3건의 영향 범위 사전 조사

**목표:** 사용자가 SSOT 반영을 결정하기 전에, 반영 시 실제로 뭐가 바뀌는지 미리 조사해둔다(제안서 자체는 수정하지 않음).

**작업:** 아래 3개 제안서 각각에 대해 "반영되면 코드/팩 중 무엇이 영향받는지" 조사:
1. `PROPOSAL_SSOT_TERMS_TENSOR_CANONICAL_20260704.md`(텐서 정본화): `겹차림`을 참조하는 모든 코드/팩/문서 위치 전수.
2. `PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md`(음/ㅁ 트리거+값참조 꼬리): 이 SSOT 변경이 반영되면 `~음/~ㅁ/~함` 형태를 이미 쓰고 있는 기존 코드/팩이 있는지(있다면 어떻게 처리되던 것인지).
3. D36 누리바꿈(세계영향 → 누리바꿈): `세계영향`/`world_affecting` 문자열을 참조하는 모든 코드/팩/문서 위치 전수.

**산출물:** `docs/context/reports/SSOT_AMENDMENT_IMPACT_SCAN_V1.md`
스키마: 제안서별 절, 각 절에 `| 파일 | 줄 | 현재 내용 | 영향 예상 |`

**수용 기준:**
- [ ] 3개 제안서 모두 커버
- [ ] grep 등 재현 가능한 방법으로 전수 확인(누락 최소화)
- [ ] SSOT 제안서 자체나 코드 수정 금지 — 조사만

---

## Q-CONFORMANCE — 커널 적합성 팩 골격 작성 (게이트 해제 조건)

**목표:** `docs/context/briefs/BRIEF_LANG_KERNEL_CONFORMANCE_PACK_V1.md`를 그대로 실행. 이 작업의 Claude 리뷰 통과가 0705 큐 Q13~Q18(커널 구현)의 게이트 해제 조건이다.
**의존:** 없음 — Q12~Q17과 병렬 가능.
**주의:** 브리프 본문의 "레드 케이스를 억지로 그린으로 맞추지 말 것"을 반드시 지킬 것 — 이 팩의 존재 이유가 무너진다.

---

## 큐 완료 보고

- 실행일: 2026-07-06
- 브랜치: `codex/queue-20260706`
- main 직접 커밋/push/golden --update/파일 삭제/네트워크 실행 없음.

| 항목 | 커밋 | 산출물 | 자기 검증 |
|---|---|---|---|
| Q12 | `fc6a23d` | `docs/context/reports/FIXED64_NUMERIC_ENVELOPE_V1.md` | `cargo test --manifest-path tools/teul-cli/Cargo.toml` PASS |
| Q13 | `6033787` | `docs/context/reports/VALUE_MODEL_DUALITY_AUDIT_V1.md` | enum 추출/행 검증 PASS, `git diff --cached --check` PASS |
| Q14 | `554ed52` | `docs/context/reports/LESSON_STYLE_AUDIT_V1.md` | `.ddn` 1866개/표 행 검증 PASS, `git diff --cached --check` PASS |
| Q15 | `6431248` | `docs/context/reports/BROKEN_CHECKS_ROOT_CAUSE_V1.md` | Q3 체크 173개/문서 176개/참조행 225개 검증 PASS |
| Q16 | `4d7e010` | `docs/context/reports/UI_RUNTIME_LOAD_CHECK_V1.md` | Playwright 제품 91개/동적의심 54개 전부 성공, 콘솔/404 0건 |
| Q17 | `f0de276` | `docs/context/reports/SSOT_AMENDMENT_IMPACT_SCAN_V1.md` | 텐서 104행, 누리바꿈 712행, 음/ㅁ 직접 36행 검증 PASS |
| Q-CONFORMANCE | `2be61bd` | `pack/lang_kernel_v1_conformance/`, `docs/context/briefs/BRIEF_LANG_KERNEL_CONFORMANCE_PACK_V1.md` 실행 보고 | `python tests/run_pack_golden.py lang_kernel_v1_conformance` PASS; `python tests/run_ci_sanity_gate.py --profile core_lang` PASS |

Q-CONFORMANCE 특기:
- 기본 12개 케이스에 브리프가 별도 요구한 `value_ref_tail_undefined` 레드 케이스를 더해 총 13개를 캡처했다.
- `stem_alias_dop_dou`, `tail_equiv_gi_hagi`, `stem_alias_ambiguous`는 브리프 예상과 실제 제품 `teul-cli run` 결과가 달라 README와 브리프 실행 보고에 명시했다.
- 검증 실행 부산물로 갱신된 기존 open 로그 2개는 Q-CONFORMANCE 산출물이 아니므로 원래 상태로 되돌렸다.
