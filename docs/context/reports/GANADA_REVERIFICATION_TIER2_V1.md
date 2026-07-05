# GANADA_REVERIFICATION_TIER2_V1

작성일: 2026-07-06
브랜치: `codex/queue-20260706`
대상: `ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md` §5 중 다/마/하/라 줄기 24칸
성격: 진단 전용. golden 갱신, 코드 수정, 삭제 없음.

## 실행 로그

- 체커 실행 로그: `I:/home/urihanl/ddn/codex/out/queue-20260706/q24-ganada-tier2/`
- 보조 pack golden/템플릿 로그: `I:/home/urihanl/ddn/codex/out/queue-20260706/q24-ganada-tier2/pack_golden/`
- 직접 lesson 실행 로그: `I:/home/urihanl/ddn/codex/out/queue-20260706/q24-ganada-tier2/lesson_run/`
- 체커 요약: 35개 체커 실행 중 PASS 6개(`ma5_lts_boundary`, `ha0`, `ha1`, `ha2`, `ha3`, `la0_docs`), FAIL 29개.
- 보조 pack golden 요약: 존재하는 roadmap_v2 marker/rebase/reconciliation 팩 20개는 모두 PASS. 교과 대표 lesson pack 4개는 `golden.jsonl`이 없어 `run_pack_golden.py` FAIL.

## 24칸 재검증 표

| 줄기-마루 | 원문 완료근거 | 존재확인 | 실행결과 | 표본 내용확인 | 재판정 상태 |
|---|---|---|---|---|---|
| 다-0 | `math library proposal` | `tests/run_roadmap_v2_da0_math_proof_scope_check.py`, `pack/roadmap_v2_da0_math_proof_scope_v1` | 체커 FAIL: matrix counts mismatch `rows=90 behavior=90 docs=0`. 보조 pack golden PASS. | roadmap_v2 pack은 범위/상태 marker. 수학 라이브러리 동작 닫힘 증거는 아님. | 존재하나FAIL |
| 다-1 | `math smoke pack` | `tests/run_roadmap_v2_da1_*`, `pack/roadmap_v2_da1_math_first_run_frontier_rebase_v1` | `da1_rebase`, `da1_first_run`, `da1_closure`, `da1_final` 모두 FAIL. 누락: `ROADMAP_V2_DA1_*`, `MATH_VECTOR_MINIMUM_FIRST_RUN_V1.md`, `MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1.md` 등. 보조 pack golden PASS. | pack은 frontier/rebase marker이며 exact/vector/function smoke 통합 닫힘을 증명하지 못한다. | 존재하나FAIL |
| 다-2 | `symbolic/proof pack PASS` | `tests/run_roadmap_v2_da2_symbolic_solve_proof_frontier_rebase_check.py`, `pack/roadmap_v2_da2_symbolic_solve_proof_frontier_rebase_v1` | 체커 FAIL: `ROADMAP_V2_DA2_SYMBOLIC_SOLVE_PROOF_FRONTIER_REBASE_V1.md` 누락. 보조 pack golden PASS. | symbolic/solve/proof 닫힘 pack이 아니라 frontier marker. | 존재하나FAIL |
| 다-3 | `seamgrim math view pack` | `tests/run_roadmap_v2_da3_seamgrim_math_view_frontier_rebase_check.py`, `pack/roadmap_v2_da3_seamgrim_math_view_frontier_rebase_v1` | 체커 FAIL: `ROADMAP_V2_DA3_SEAMGRIM_MATH_VIEW_FRONTIER_REBASE_V1.md` 누락. 보조 pack golden PASS. | 셈그림 수학 보개 연결 동작 검증이 아니라 frontier marker. | 존재하나FAIL |
| 다-4 | `registry pack` | `tests/run_roadmap_v2_da4_math_package_share_frontier_rebase_check.py`, `pack/roadmap_v2_da4_math_package_share_frontier_rebase_v1` | 체커 FAIL: `ROADMAP_V2_DA4_MATH_PACKAGE_SHARE_FRONTIER_REBASE_V1.md` 누락. 보조 pack golden PASS. | 수학가지 공유/registry 닫힘 증거 없음. | 존재하나FAIL |
| 다-5 | `math LTS suite` | `tests/run_roadmap_v2_da5_math_lts_frontier_rebase_check.py`, `pack/roadmap_v2_da5_math_lts_frontier_rebase_v1` | 체커 FAIL: `ROADMAP_V2_DA5_MATH_LTS_FRONTIER_REBASE_V1.md` 누락. 보조 pack golden PASS. | LTS suite가 아니라 frontier marker. | 존재하나FAIL |
| 마-0 | `curriculum catalog` | `tests/run_roadmap_v2_ma0_curriculum_catalog_check.py`, `pack/roadmap_v2_ma0_curriculum_catalog_v1` | 체커 FAIL: matrix counts mismatch `rows=90 behavior=90 docs=0`. 보조 pack golden PASS. | catalog marker 성격. 교과 카탈로그 닫힘 동작으로 볼 증거 부족. | 존재하나FAIL |
| 마-1 | `lesson smoke` | `tests/run_roadmap_v2_ma1_lesson_first_run_reconciliation_check.py`, `pack/roadmap_v2_ma1_lesson_first_run_reconciliation_v1`, 대표 lesson 후보 | 체커 FAIL: 하위 `studio_classroom_report_workflow_runner.mjs`에서 `failed export row missing`. 보조 roadmap pack golden PASS. | `pack/edu_p1_constant_accel/lesson.ddn`와 `pack/edu_e1_supply_demand_tax/lesson.ddn`는 각각 `보개로 그려.` 한 줄 placeholder. 새 대표 `edu_seamgrim_rep_phys_projectile_xy_v1`, `edu_seamgrim_rep_econ_supply_demand_tax_v1`는 실질 모델이 있으나 `golden.jsonl` 없음. `physics_pendulum_seed_v1`, `econ_tax_shock_supply_demand_seed_v1` 직접 실행은 `E_LEGACY_RANGE_SYNTAX` FAIL. | 존재하나FAIL |
| 마-2 | `pack/checker PASS` | `tests/run_roadmap_v2_ma2_*`, 교과 lesson 후보 | `ma2_pack` FAIL: `MA2_SEAMGRIM_CURRICULUM_2_PACK_CLOSURE_V1.md` 누락. `ma2_unlock` FAIL: `MA2_STUDIO_PREREQ_UNLOCK_V1.md` 누락. | 마-1과 같은 대표 구형 lesson 2개가 placeholder이고, 대표 pack 4개는 golden pack으로 닫히지 않는다. | 존재하나FAIL |
| 마-3 | `classroom UI pack` | `tests/run_roadmap_v2_ma3_*`, `pack/roadmap_v2_ma3_studio_classroom_workbench_prereq_rebase_v1` | `ma3_pack`, `ma3_prereq` 모두 FAIL: `MA3_*` 문서 누락. 보조 pack golden PASS. | 수업용 작업실 닫힘이 아니라 prereq/rebase marker. | 존재하나FAIL |
| 마-4 | `publication pack` | `tests/run_roadmap_v2_ma4_*`, `pack/roadmap_v2_ma4_public_lesson_publication_prereq_rebase_v1` | `ma4_pack`, `ma4_prereq` 모두 FAIL: `MA4_*` 문서 누락. 보조 pack golden PASS. | 공개 차시 publication 닫힘 증거가 재실행되지 않는다. | 존재하나FAIL |
| 마-5 | `curriculum LTS` | `tests/run_roadmap_v2_ma5_*`, `pack/roadmap_v2_ma5_curriculum_lts_prereq_rebase_v1` | `ma5_lts_boundary`만 PASS. `ma5_pack` FAIL: `MA5_SEAMGRIM_CURRICULUM_5_LTS_PACK_CLOSURE_V1.md` 누락. `ma5_prereq`도 하위 closure FAIL. | PASS한 boundary 체커는 후보 진행률 경계와 false claim 방지만 확인한다. LTS 닫힘이 아니다. | 존재하나FAIL |
| 하-0 | `textbook map` | `tests/run_roadmap_v2_ha0_education_curriculum_template_matrix_reconciliation_check.py`, `pack/roadmap_v2_ha0_education_curriculum_template_matrix_reconciliation_v1` | 체커 PASS. 보조 pack golden PASS. | contract가 `runtime_claim=false`, `product_code_change=false`, `product_ui_change=false`; curriculum metadata/template evidence를 matrix에 반영하는 reconciliation. | 존재+PASS이나형식뿐 |
| 하-1 | `teaching smoke` | `tests/run_roadmap_v2_ha1_representative_teaching_smoke_check.py`, `pack/education_curriculum_1_v1`, 대표 3과목 pack | 체커 PASS. `education_curriculum_1_v1` golden PASS. | 체커는 artifact 존재/meta/view_spec을 확인하지만 `pack/edu_s1_function_graph/lesson.ddn`, `pack/edu_p1_constant_accel/lesson.ddn`, `pack/edu_e1_supply_demand_tax/lesson.ddn`가 모두 `보개로 그려.` 한 줄이다. 대표 교재 첫실행의 실질 내용은 placeholder. | 존재+PASS이나형식뿐 |
| 하-2 | `lesson pack PASS` | `tests/run_roadmap_v2_ha2_education_assessment_pack_check.py`, `pack/education_curriculum_2_v1` | 체커 PASS. pack golden PASS, node runner PASS. | local evidence only. contract는 gradebook/live submission/remote sync/state_hash/parser/grammar를 false로 둔다. 또한 이 행이 요구하는 마-2 교과 pack은 위에서 FAIL. | 존재+PASS이나형식뿐 |
| 하-3 | `classroom UI pack` | `tests/run_roadmap_v2_ha3_classroom_ui_pack_check.py`, `pack/education_curriculum_3_v1` | 체커 PASS. pack golden PASS, 하-2 체커 PASS, node runner PASS. | local classroom UI evidence는 있으나 contract가 live submission/remote sync/gradebook/state_hash/parser/grammar를 false로 둔다. 마-3 전제 체커는 FAIL. | 존재+PASS이나형식뿐 |
| 하-4 | `publication pack` | `tests/run_roadmap_v2_ha4_public_course_publication_pack_check.py` | 체커 FAIL: `solutions/seamgrim_ui_mvp/ui/styles.css`에 `.education-publication-pack`, `.education-publication-artifacts`, `.education-publication-preview` 누락. | 공개 강좌/교재 publication UI 닫힘 증거가 현재 스타일/표면과 맞지 않는다. | 존재하나FAIL |
| 하-5 | `education LTS` | `tests/run_roadmap_v2_ha5_education_operations_lts_check.py` | 체커 FAIL: `styles.css`에 `.education-operations-lts`, `.education-operations-artifacts`, `.education-operations-preview` 누락. | 교육 운영 LTS UI 닫힘 증거가 현재 스타일/표면과 맞지 않는다. | 존재하나FAIL |
| 라-0 | `proposal` | `tests/run_roadmap_v2_la0_malblock_design_behavior_reassessment_check.py`, `tests/run_roadmap_v2_la0_pa0_docs_closed_reconciliation_check.py`, `pack/roadmap_v2_la0_pa0_docs_closed_reconciliation_v1` | docs-closed reconciliation PASS. behavior reassessment FAIL: `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1.md` 누락. | docs pack contract가 `behavior_closed_coordinates=[]`, `roadmap_behavior_increment=false`라고 명시한다. 원문은 닫힘-동작이므로 불충분. | 존재하나FAIL |
| 라-1 | `generated DDN check/run` | `tests/run_roadmap_v2_la1_malblock_intro_rail_matrix_reconciliation_check.py`, `pack/roadmap_v2_la1_malblock_intro_rail_matrix_reconciliation_v1` | 체커 FAIL: 하위 `run_roadmap_v2_na1_post_matrix_frontier_rebase_check.py`가 `seamgrim_editor_run_transaction` 오류로 실패. 보조 pack golden PASS. | block->DDN 첫실행을 닫는 현재 체커가 PASS하지 않는다. | 존재하나FAIL |
| 라-2 | `roundtrip pack` | `tests/run_roadmap_v2_la2_*`, `pack/roadmap_v2_la2_matrix_status_reconciliation_v1` | `la2_matrix`, `la2_rebase`, `la2_final` 모두 FAIL. 누락: `LA2_MATRIX_STATUS_RECONCILIATION_V1.md`, `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1.md`, `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1.md` 등. 보조 pack golden PASS. | subset roundtrip 닫힘 증거가 재실행되지 않는다. | 존재하나FAIL |
| 라-3 | `three-mode layout pack` | `tests/run_roadmap_v2_la3_workbench_integration_reconciliation_check.py`, `pack/roadmap_v2_la3_workbench_integration_reconciliation_v1` | 체커 FAIL: `MALBLOCK_AUTHORING_UI_V1.md`, `SEAMGRIM_LESSON_AUTHORING_FLOW_V1.md` 누락. 보조 pack golden PASS. | 3모드 작업실 통합 닫힘 증거가 누락 문서에 막힌다. | 존재하나FAIL |
| 라-4 | `lesson package` | `tests/run_roadmap_v2_la4_lesson_package_reconciliation_check.py`, `pack/roadmap_v2_la4_lesson_package_reconciliation_v1` | 체커 FAIL: 하위 라-3 체커 FAIL. 보조 pack golden PASS. | 라-3 미닫힘 위에 lesson package를 닫을 수 없다. | 존재하나FAIL |
| 라-5 | `editor LTS suite` | `tests/run_roadmap_v2_la5_editor_lts_reconciliation_check.py`, `pack/roadmap_v2_la5_editor_lts_reconciliation_v1` | 체커 FAIL: 하위 `studio_classroom_report_workflow_runner.mjs`에서 `failed export row missing`. 보조 pack golden PASS. | editor LTS suite가 현재 제품/러너 상태로 PASS하지 않는다. | 존재하나FAIL |

## 마-1/마-2 lesson placeholder 확인

| 파일 | 직접 확인 | 직접 실행 |
|---|---|---|
| `pack/edu_p1_constant_accel/lesson.ddn` | `보개로 그려.` 한 줄. placeholder. | `teul-cli run` exit 0, state/trace/bogae hash만 출력. |
| `pack/edu_e1_supply_demand_tax/lesson.ddn` | `보개로 그려.` 한 줄. placeholder. | `teul-cli run` exit 0, state/trace/bogae hash만 출력. |
| `pack/edu_seamgrim_rep_phys_projectile_xy_v1/lesson.ddn` | 채비/시작/매마디/포물선 계산이 있는 실질 lesson. | `teul-cli run` exit 0, state/trace/bogae hash 출력. `golden.jsonl`은 없음. |
| `pack/edu_seamgrim_rep_econ_supply_demand_tax_v1/lesson.ddn` | 채비/시작/매마디/수요·공급·세금 계산이 있는 실질 lesson. | `teul-cli run` exit 0, state/trace/bogae hash 출력. `golden.jsonl`은 없음. |
| `solutions/seamgrim_ui_mvp/seed_lessons_v1/physics_pendulum_seed_v1/lesson.ddn` | 진자 상태전이 모델이 있으나 구형 range 주석/표면 포함. | `teul-cli run` FAIL: `E_LEGACY_RANGE_SYNTAX`. |
| `solutions/seamgrim_ui_mvp/seed_lessons_v1/econ_tax_shock_supply_demand_seed_v1/lesson.ddn` | 수요·공급 세금 충격 상태전이 모델이 있으나 구형 range 주석/표면 포함. | `teul-cli run` FAIL: `E_LEGACY_RANGE_SYNTAX`. |

## 결론

Q24 대상 24칸 중 `진짜닫힘`으로 재판정한 칸은 없다. 20칸은 대응 체커가 FAIL했고, 4칸(하-0~하-3)은 PASS 증거가 있으나 marker/docs/local UI evidence 또는 placeholder lesson 문제 때문에 원문 `닫힘-동작`을 그대로 신뢰하기 어렵다. 라-0은 docs-closed 보조 체커만 PASS했고 behavior reassessment 체커가 FAIL했으므로 `존재하나FAIL`로 판정했다. 특히 마-1/마-2의 구형 대표 교과 lesson 2개는 실제로 한 줄 placeholder다.
