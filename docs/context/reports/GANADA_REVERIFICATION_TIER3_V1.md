# GANADA_REVERIFICATION_TIER3_V1

작성일: 2026-07-06
브랜치: `codex/queue-20260706`
대상: `ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md` §5 중 바/사/아/자/차/카/파/거 줄기 48칸
성격: 진단 전용. golden 갱신, 코드 수정, 삭제 없음.

## 실행 로그

- 체커 실행 로그: `I:/home/urihanl/ddn/codex/out/queue-20260706/q25-ganada-tier3/`
- 실패 tail 모음: `I:/home/urihanl/ddn/codex/out/queue-20260706/q25-ganada-tier3/failure_tails.txt`
- 보조 roadmap_v2 pack golden 로그: `I:/home/urihanl/ddn/codex/out/queue-20260706/q25-ganada-tier3/pack_golden/`
- 체커 요약: 56개 체커 실행 중 PASS 9개, FAIL 47개.
- 보조 pack golden 요약: 존재하는 roadmap_v2 대응 pack 22개 모두 PASS. 단, 이 PASS는 다수 칸에서 marker/reconciliation pack 자체의 golden 통과일 뿐 제품 닫힘을 보장하지 않는다.

## 48칸 재검증 표

| 줄기-마루 | 원문 완료근거 | 존재확인 | 실행결과 | 표본 내용확인 | 재판정 상태 |
|---|---|---|---|---|---|
| 바-0 | `free lab proposal` | `tests/run_roadmap_v2_ba0_*`, `pack/roadmap_v2_ba0_*` | `ba0_reassessment`, `ba0_rebase` 모두 FAIL: `BA0_FREE_LAB_SEED_REASSESSMENT_V1.md`, `BA0_FREE_LAB_SEED_REBASE_V1.md` 누락. 보조 pack golden PASS. | seed/rebase marker pack은 있으나 완료 문서 체커가 재현되지 않는다. | 존재하나FAIL |
| 바-1 | `free-lab smoke` | `tests/run_roadmap_v2_ba1_free_lab_first_run_check.py`, `pack/free_lab_1_v1` | 체커 PASS. 내부에서 `run_pack_golden.py free_lab_1_v1`, `node tests/free_lab_first_run_runner.mjs` PASS. | `free_lab_first_run.js`에 실제 DDN 템플릿, 새 실험/매김/기록 lane, UI builder가 있다. placeholder 아님. | 진짜닫힘 |
| 바-2 | `experiment_report pack` | `tests/run_roadmap_v2_ba2_free_lab_experiment_report_check.py`, `pack/free_lab_2_v1` | 체커 PASS. 내부에서 `free_lab_2_v1` golden, BA1 체커, node runner PASS. | 가설/레버/지표/결론 artifact와 `free_lab_experiment_report.js` UI builder가 있다. placeholder 아님. | 진짜닫힘 |
| 바-3 | `free-lab UI pack` | `tests/run_roadmap_v2_ba3_free_lab_ui_pack_check.py`, `pack/free_lab_3_v1` | 체커 PASS. 내부에서 `free_lab_3_v1` golden, BA2 체커, node runner PASS. | baseline/low/high 레버 sweep과 ghost 비교 UI builder가 있다. placeholder 아님. | 진짜닫힘 |
| 바-4 | `share pack` | `tests/run_roadmap_v2_ba4_free_lab_share_pack_check.py`, `pack/free_lab_4_v1` | 체커 PASS. 내부에서 `free_lab_4_v1` golden, BA3 체커, node runner PASS. | `seamgrim://free-lab/local/...` 로컬 snapshot/remix/handoff 링크를 만든다. public upload/registry/cloud는 false claim으로 막는다. | 진짜닫힘 |
| 바-5 | `research workflow suite` | `tests/run_roadmap_v2_ba5_free_lab_research_workflow_check.py`, `pack/free_lab_5_v1` | 체커 PASS. 내부에서 `free_lab_5_v1` golden, BA4 체커, node runner PASS. | batch/csv/notebook handoff, CSV text, 연구 workflow UI builder가 있다. 외부 notebook server claim은 false. | 진짜닫힘 |
| 사-0 | `bogae schema docs` | `tests/run_roadmap_v2_sa0_bogae_schema_boundary_check.py`, `pack/roadmap_v2_sa0_bogae_schema_boundary_v1` | 체커 FAIL: matrix counts mismatch `rows=90 behavior=90 docs=0`. 보조 pack golden PASS. | schema boundary pack은 있으나 닫힘-동작 재판정 체커가 현재 로드맵 상태와 맞지 않는다. | 존재하나FAIL |
| 사-1 | `bogae smoke pack` | `tests/run_roadmap_v2_sa1_*`, `pack/roadmap_v2_sa1_bogae_graph_space2d_matrix_reconciliation_v1` | `sa1_matrix`, `sa1_rebase` FAIL. 누락: `ROADMAP_V2_SA1_REBASE_V1.md`, `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md`. 보조 pack golden PASS. | graph/space2d 첫실행 닫힘 체커가 누락 문서에 막힌다. | 존재하나FAIL |
| 사-2 | `sprite/grid pack` | `tests/run_roadmap_v2_sa2_*`, `pack/roadmap_v2_sa2_*` | `sa2_final`, `sa2_rebase` FAIL. 누락: `SA2_SPRITE_GRID2D_FINAL_CLOSURE_V1.md`, `SA2_SPRITE_GRID2D_CLOSURE_REBASE_V1.md`. 보조 pack golden PASS. | sprite/grid2d 닫힘 문서와 final closure 체커가 재현되지 않는다. | 존재하나FAIL |
| 사-3 | `game preview pack` | `tests/run_roadmap_v2_sa3_game_preview_reconciliation_check.py`, `pack/roadmap_v2_sa3_game_preview_reconciliation_v1` | 체커 FAIL: 하위 `sa2_final` FAIL. 보조 pack golden PASS. | game preview는 사-2 final closure에 종속되어 현재 닫히지 않는다. | 존재하나FAIL |
| 사-4 | `asset registry pack` | `tests/run_roadmap_v2_sa4_asset_view_share_reconciliation_check.py`, `pack/roadmap_v2_sa4_asset_view_share_reconciliation_v1` | 체커 FAIL: 하위 `sa3`/`sa2_final` FAIL. 보조 pack golden PASS. | asset/view 공유도 이전 보개 닫힘 위에 얹힌 reconciliation이라 현재 PASS하지 않는다. | 존재하나FAIL |
| 사-5 | `renderer LTS` | `tests/run_roadmap_v2_sa5_renderer_hardening_reconciliation_check.py`, `pack/roadmap_v2_sa5_renderer_hardening_reconciliation_v1` | 체커 FAIL: matrix counts mismatch `rows=90 behavior=90 docs=0`. 보조 pack golden PASS. | renderer hardening LTS가 현재 행렬 상태와 맞지 않는다. | 존재하나FAIL |
| 아-0 | `nurigym schema docs` | `tests/run_roadmap_v2_a0_nurigym_schema_skeleton_check.py`, `pack/roadmap_v2_a0_nurigym_schema_skeleton_v1` | 체커 FAIL: matrix counts mismatch `rows=90 behavior=90 docs=0`. 보조 pack golden PASS. | schema skeleton marker는 있으나 닫힘-동작 재판정은 실패한다. | 존재하나FAIL |
| 아-1 | `smoke pack` | `tests/run_roadmap_v2_a1_*`, `pack/roadmap_v2_a1_nurigym_reset_step_matrix_reconciliation_v1` | `a1_final`, `a1_matrix` FAIL. 누락: `ROADMAP_V2_A1_NURIGYM_FINAL_CLOSURE_V1.md`, `ROADMAP_V2_A1_NURIGYM_REBASE_V1.md`, `NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1.md`. 보조 pack golden PASS. | 브리프에서 지적한 expected hash refresh 계열이 여전히 닫히지 않았다. | 존재하나FAIL |
| 아-2 | `dataset hash pack` | `tests/run_roadmap_v2_a2_*`, `pack/roadmap_v2_a2_nurigym_representative_environment_matrix_reconciliation_v1` | `a2_bandit_or`, `a2_final`, `a2_rebase`, `a2_matrix` 모두 FAIL. 누락: `NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1.md`, `NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1.md`, `NURIGYM_BANDIT_MINIMUM_PACK_V1.md` 등. 보조 pack golden PASS. | CartPole/Pendulum/GridWorld/Bandit 대표 환경의 expected refresh 문제가 남아 있어 닫힘으로 볼 수 없다. | 존재하나FAIL |
| 아-3 | `parity pack` | `tests/run_roadmap_v2_a3_nurigym_python_web_parity_check.py`, `pack/nurigym_python_web_parity_v1` | 체커 FAIL: matrix counts mismatch `rows=90 behavior=90 docs=0`. | Python/Web parity pack 존재와 별개로 로드맵 재판정 체커가 실패한다. | 존재하나FAIL |
| 아-4 | `dataset publish pack` | `tests/run_roadmap_v2_a4_dataset_registry_check.py`, `pack/nuri_gym_dataset_registry_v1` | 체커 FAIL: matrix counts mismatch `rows=90 behavior=90 docs=0`. | dataset registry 닫힘 체커가 현재 행렬 상태와 맞지 않는다. | 존재하나FAIL |
| 아-5 | `training workflow suite` | `tests/run_roadmap_v2_a5_nurigym_training_workflow_check.py`, `pack/nurigym_training_workflow_v1` | 체커 FAIL: matrix counts mismatch `rows=90 behavior=90 docs=0`. | training workflow suite가 현재 로드맵 닫힘 체커로 PASS하지 않는다. | 존재하나FAIL |
| 자-0 | `boundary docs` | `tests/run_roadmap_v2_ja0_ai_boundary_behavior_reassessment_check.py`, `pack/roadmap_v2_ja0_ai_boundary_behavior_reassessment_v1` | 체커 PASS, 보조 pack golden PASS. | pack은 자-0을 문서에서 동작으로 재평가하지만 `new_ai_call`, `model_training`, `auto_apply`, `file_write`, `runtime_ast_persistence`, `state_hash_ownership`, `production_ai_path`를 모두 false claim으로 둔다. | 존재+PASS이나형식뿐 |
| 자-1 | `intent pack` | `tests/run_roadmap_v2_ja_seulgi_boundary_reconciliation_check.py`, `pack/seulgi_v1`, `pack/roadmap_v2_ja_seulgi_boundary_reconciliation_v1` | 체커 PASS. 내부에서 `run_seulgi_v1_pack_check.py` 등 하위 Seulgi/SAM gate PASS. | SeulgiIntent/Gatekeeper skeleton의 하위 pack 검증이 실제로 돈다. production AI path는 claim하지 않는다. | 진짜닫힘 |
| 자-2 | `AI boundary pack` | `tests/run_roadmap_v2_ja_seulgi_boundary_reconciliation_check.py`, `pack/seulgi_gatekeeper_v1`, `pack/sam_ai_ordering_v1` | 체커 PASS. 내부에서 gatekeeper, SAM ordering, seamgrim CI gate step 체크 PASS. | Gatekeeper/InputSnapshot 경계 증거는 하위 pack으로 재현된다. production AI나 자동 적용은 claim하지 않는다. | 진짜닫힘 |
| 자-3 | `AI support UI pack` | `tests/run_roadmap_v2_ja3_seulgi_proposal_ui_check.py` | 체커 FAIL: `solutions/seamgrim_ui_mvp/ui/index.html`에 `id="seulgi-proposal-ui"`, `data-seulgi-proposal-ui` 누락. | 제안 UI의 제품 진입점 selector가 현재 HTML에 없다. | 존재하나FAIL |
| 자-4 | `model registry pack` | `tests/run_roadmap_v2_ja4_model_artifact_share_reconciliation_check.py`, `pack/model_artifact_*`, `pack/roadmap_v2_ja4_model_artifact_share_reconciliation_v1` | 체커 PASS. 내부에서 model artifact inference/eval/closure/provenance check와 golden PASS. | artifact hash/seal/eval pass-fail 증거가 실질적이다. 단 public model registry, production AI path, full training DSL은 false claim. | 진짜닫힘 |
| 자-5 | `AI LTS suite` | `tests/run_roadmap_v2_ja5_replay_safe_ai_workflow_check.py` | 체커 FAIL: `solutions/seamgrim_ui_mvp/ui/index.html`에 `id="seulgi-replay-safe-workflow"`, `data-seulgi-replay-safe-workflow` 누락. | replay-safe AI workflow 제품 진입점이 현재 HTML에 없다. | 존재하나FAIL |
| 차-0 | `proposal/schema` | `tests/run_roadmap_v2_cha0_*`, `pack/roadmap_v2_cha0_*` | `cha0_reassessment`, `cha0_rebase` 모두 FAIL. 누락: `CHA0_RPG_SEED_REASSESSMENT_V1.md`, `CHA0_RPG_SEED_REBASE_V1.md`. 보조 pack golden PASS. | RPG seed 문서/재평가 체커가 재현되지 않는다. | 존재하나FAIL |
| 차-1 | `smoke pack` | `tests/run_roadmap_v2_cha1_rpg_phrase_action_smoke_check.py` | 체커 FAIL: `CHA1_RPG_PHRASE_ACTION_SMOKE_V1.md` 누락. | phrase/action smoke 닫힘 문서가 없다. | 존재하나FAIL |
| 차-2 | `story pack` | `tests/run_roadmap_v2_cha2_rpg_story_pack_closure_check.py` | 체커 FAIL: `CHA2_RPG_STORY_PACK_CLOSURE_V1.md` 누락. | story/RPG pack closure 문서가 없다. | 존재하나FAIL |
| 차-3 | `authoring UI pack` | `tests/run_roadmap_v2_cha3_rpg_box_authoring_ui_check.py` | 체커 FAIL: `CHA3_RPG_BOX_NURIMAKER_AUTHORING_UI_V1.md` 누락. | RPG Box/누리메이커 authoring UI 닫힘 문서가 없다. | 존재하나FAIL |
| 차-4 | `package pack` | `tests/run_roadmap_v2_cha4_rpg_story_package_check.py` | 체커 FAIL: `CHA4_RPG_STORY_PACKAGE_V1.md` 누락. | story package 닫힘 문서가 없다. | 존재하나FAIL |
| 차-5 | `adapter LTS` | `tests/run_roadmap_v2_cha5_rpg_engine_adapter_lts_check.py` | 체커 FAIL: `CHA5_RPG_ENGINE_ADAPTER_LTS_V1.md` 누락. | engine adapter LTS 닫힘 문서가 없다. | 존재하나FAIL |
| 카-0 | `platform docs` | `tests/run_roadmap_v2_ka0_platform_charter_matrix_reconciliation_check.py`, `pack/roadmap_v2_ka0_platform_charter_matrix_reconciliation_v1` | 체커 FAIL: `ROADMAP_V2_KA0_PLATFORM_CHARTER_MATRIX_RECONCILIATION_V1.md` 누락. 보조 pack golden PASS. | platform charter reconciliation pack은 있으나 체커가 기대하는 문서가 없다. | 존재하나FAIL |
| 카-1 | `server smoke` | `tests/run_roadmap_v2_ka1_server_mvp_matrix_reconciliation_check.py`, `pack/roadmap_v2_ka1_server_mvp_matrix_reconciliation_v1` | 체커 FAIL: `ROADMAP_V2_KA1_SERVER_MVP_MATRIX_RECONCILIATION_V1.md` 누락. 보조 pack golden PASS. | server MVP smoke reconciliation 문서가 없다. | 존재하나FAIL |
| 카-2 | `API pack` | `tests/run_roadmap_v2_ka2_publication_read_api_check.py` | 체커 FAIL: `KA2_PUBLICATION_READ_API_CLOSURE_V1.md` 누락. | publication/read API closure 문서가 없다. | 존재하나FAIL |
| 카-3 | `share UI pack` | `tests/run_roadmap_v2_ka3_project_share_ui_check.py` | 체커 FAIL: `KA3_PROJECT_SHARE_UI_V1.md` 누락. | project/share UI 닫힘 문서가 없다. | 존재하나FAIL |
| 카-4 | `registry seed pack` | `tests/run_roadmap_v2_ka4_public_registry_seed_check.py` | 체커 FAIL: `KA4_PUBLIC_REGISTRY_SEED_V1.md` 누락. | public registry seed 닫힘 문서가 없다. | 존재하나FAIL |
| 카-5 | `platform LTS` | `tests/run_roadmap_v2_ka5_platform_hardening_check.py` | 체커 FAIL: `KA5_PLATFORM_HARDENING_V1.md` 누락. | auth/RBAC/audit/backup hardening 문서가 없다. | 존재하나FAIL |
| 파-0 | `case schema` | `tests/run_roadmap_v2_pa0_social_case_card_behavior_reassessment_check.py`, `pack/roadmap_v2_pa0_social_case_card_behavior_reassessment_v1` | 체커 PASS. 내부에서 case-card schema, AGE4, social-world pack/node runners PASS. 보조 pack golden PASS. | pack은 downstream `파-1`~`파-5` 증거를 근거로 재평가하지만 new product UI/code, real-world prediction, policy advice, public registry/network publish는 false claim으로 둔다. | 존재+PASS이나형식뿐 |
| 파-1 | `AGE4 case smoke` | `tests/run_roadmap_v2_pa1_baseline_market_first_run_matrix_reconciliation_check.py`, `pack/roadmap_v2_pa1_baseline_market_first_run_matrix_reconciliation_v1` | 체커 FAIL: 하위 `sa1` FAIL(`ROADMAP_V2_SA1_REBASE_V1.md` 누락). 보조 pack golden PASS. | 경제 첫실행은 보개 사-1 전제에 막혀 현재 닫힘 재현 불가. | 존재하나FAIL |
| 파-2 | `bridge report pack` | `tests/run_roadmap_v2_pa2_social_bridge_pack_check.py` | 체커 FAIL: `PA2_SOCIAL_BRIDGE_PACK_V1.md` 누락. | bridge/social pack 닫힘 문서가 없다. | 존재하나FAIL |
| 파-3 | `policy ghost pack` | `tests/run_roadmap_v2_pa3_policy_ghost_ui_check.py` | 체커 FAIL: `PA3_POLICY_GHOST_UI_V1.md` 누락. | policy ghost UI 닫힘 문서가 없다. | 존재하나FAIL |
| 파-4 | `social template pack` | `tests/run_roadmap_v2_pa4_social_template_registry_check.py` | 체커 FAIL: `PA4_SOCIAL_TEMPLATE_REGISTRY_V1.md` 누락. | social template registry 닫힘 문서가 없다. | 존재하나FAIL |
| 파-5 | `social-world LTS` | `tests/run_roadmap_v2_pa5_social_world_lts_check.py` | 체커 FAIL: `PA5_SOCIAL_WORLD_LTS_V1.md` 누락. | social-world LTS 닫힘 문서가 없다. | 존재하나FAIL |
| 거-0 | `question card schema` | `tests/run_roadmap_v2_geo0_*`, `pack/roadmap_v2_geo0_ai_question_card_seed_behavior_reassessment_v1` | `geo0_behavior` FAIL: matrix counts mismatch `rows=90 behavior=90 docs=0`. `geo0_reconciliation` FAIL: `GEO0_AI_QUESTION_CARD_SEED_RECONCILIATION_V1.md` 누락. 보조 pack golden PASS. | question card seed는 현재 닫힘-동작 체커로 재현되지 않는다. | 존재하나FAIL |
| 거-1 | `toolchain smoke` | `tests/run_roadmap_v2_geo1_question_card_smoke_check.py` | 체커 FAIL: `GEO1_QUESTION_CARD_SMOKE_V1.md` 누락. | question card smoke 닫힘 문서가 없다. | 존재하나FAIL |
| 거-2 | `AI validation pack` | `tests/run_roadmap_v2_geo2_ai_output_validation_pack_check.py` | 체커 FAIL: `GEO2_AI_OUTPUT_VALIDATION_PACK_V1.md` 누락. | AI output validation pack 닫힘 문서가 없다. | 존재하나FAIL |
| 거-3 | `authoring UI pack` | `tests/run_roadmap_v2_geo3_dev_assist_ui_check.py` | 체커 FAIL: `GEO3_DEV_ASSIST_UI_V1.md` 누락. | 개발보조 UI 닫힘 문서가 없다. | 존재하나FAIL |
| 거-4 | `tool registry` | `tests/run_roadmap_v2_geo4_author_tool_share_check.py` | 체커 FAIL: `GEO4_AUTHOR_TOOL_SHARE_V1.md` 누락. | author tool share 닫힘 문서가 없다. | 존재하나FAIL |
| 거-5 | `AI workflow LTS` | `tests/run_roadmap_v2_geo5_ai_workflow_hardening_check.py` | 체커 FAIL: `GEO5_AI_WORKFLOW_HARDENING_V1.md` 누락. | AI workflow hardening 닫힘 문서가 없다. | 존재하나FAIL |

## 아줄기 특기

브리프가 지목한 누리짐 상태는 현재 로드맵의 6마루 전부 `닫힘-동작` 표기와 맞지 않는다. 아-1은 `NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1.md`, 아-2는 `NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1.md`, `NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1.md`, `NURIGYM_BANDIT_MINIMUM_PACK_V1.md` 누락으로 모두 FAIL했다. 즉 "아-1 닫힘, 아-2 진행중/expected hash stale"라는 기억 쪽이 현재 재실행 결과와 더 가깝다.

## 결론

Q25 대상 48칸을 전수 확인했다. 재판정은 `진짜닫힘` 8칸, `존재+PASS이나형식뿐` 2칸, `존재하나FAIL` 38칸이다.

- `진짜닫힘`: 바-1~바-5, 자-1, 자-2, 자-4
- `존재+PASS이나형식뿐`: 자-0, 파-0
- `존재하나FAIL`: 나머지 38칸

특히 사/아/차/카/거 줄기는 이번 범위에서 `진짜닫힘`으로 재판정한 칸이 없다. 다수는 pack golden은 통과하지만 대응 체커가 요구하는 완료 문서, expected refresh, UI selector, 또는 이전 마루 전제가 현재 저장소에서 재현되지 않는다.
