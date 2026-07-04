# BRIEF: 커널 적합성 팩 골격 작성 — `pack/lang_kernel_v1_conformance/`

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 근거: `PROPOSAL_LANG_KERNEL_V1_SPEC_20260704.md` §14
> 목적: 이 팩의 리뷰 통과가 **Q13~Q18(0705 큐, 커널 구현 시리즈)의 게이트 해제 조건**이다.
> 성격: 이 브리프는 "무엇을 테스트할지·왜·기대 동작"만 명세한다. pack 디렉터리 생성, golden.jsonl 작성, CLI 실행·값 캡처, red/green 판정은 전부 Codex 실행분이다.

## 배경

커널 구현 파이프라인(스펙 확정 → 적합성 케이스 → Codex 구현 → Claude 리뷰)에서, 이 팩은 **레드(현재 실패) 케이스**와 **그린(현재 통과, 회귀 방지 고정) 케이스**를 함께 담는다. 레드 케이스는 Q13~Q18 구현이 끝나면 통과해야 하는 목표이고, 그린 케이스는 이미 SSOT/코드에 있는 동작을 지금 고정해두는 것이다.

**중요**: 이 브리프의 케이스들은 실제로 실행해서 얻은 값이 아니라 **의도된 동작 명세**다. Codex는 각 케이스를 실제로 CLI에 돌려서 ①레드는 정말 실패하는지(그리고 어떻게 실패하는지 — 구현 안 됨 오류인지, 다른 오류인지) ②그린은 정말 통과하는지 확인하고, 실제 캡처된 출력으로 golden.jsonl을 채운다. 의도와 실제가 다르면(예: 그린으로 예상했는데 실패) 그 사실을 보고에 남긴다 — 조용히 값을 맞추지 않는다.

## pack 구조 (기존 관례 참고: `pack/lang_core_2_v1/`, `pack/numeric_root_finding_bisection_v1/`)

```
pack/lang_kernel_v1_conformance/
  README.md              ← 이 팩의 목적, 커널 스펙 링크
  golden.jsonl           ← 케이스 전체 (아래 스키마)
  cases/
    <case_id>.ddn         ← 케이스별 입력 파일
```

golden.jsonl 한 줄 스키마(기존 관례): `{"id": "...", "cmd": ["run", "pack/lang_kernel_v1_conformance/cases/<file>.ddn"], "stdout": [...], "exit_code": N}` 또는 오류 케이스는 `{"id": "...", "cmd": [...], "expected_error_code": "E_...", "exit_code": 1}`.

## 케이스 그룹 (그룹별로 순서대로 실행, 그룹 하나 = 커밋 하나 권장)

### 그룹 1 — 형태론 회귀 고정 [그린 예상, SSOT §11.3/§V18-00C 이미 구현]

목적: 이미 존재하는 동작을 지금 고정한다. 커널 스펙 §4 인용.

1. `stem_alias_돕도우`: `돕~도우:움직씨 = { (제) -> 참거짓 = { 참. } }.` 선언 후 `돕기.` 호출 — 정상 실행 확인.
2. `stem_alias_ambiguous`: 같은 스코프에 `계산:셈씨 = {...}.`와 `계산하:셈씨 = {...}.`를 **함께** 선언하고 `계산하기.` 호출 — `E_CALL_TAIL_AMBIGUOUS` 기대(스펙 §4.4).
3. `tail_equiv_기하기`: 같은 씨앗을 `기` 꼬리와 `하기` 꼬리 양쪽으로 호출 — 두 호출의 정본 AST/실행 결과가 동일한지 확인(스펙 §4.4 CALL-TAIL-EQUIV-01).
4. `condition_new_surface`: `만약 3 > 1 이면 { "참" 보여주기. } 아니면 { "거짓" 보여주기. }.` — 정상 실행, "참" 출력 확인(스펙 §3.2, 이미 normative).

### 그룹 2 — 시간 [레드 예상, D1/D22 커널 신규]

목적: `마디사이`/`지금`이 아직 없음을 확인(레드), 구현 후 그린이 될 목표.

5. `time_madisai_undefined`: `(매마디)마다 { 보임 { dt: 마디사이. } }.` 실행 — 현재는 `마디사이`가 정의되지 않은 식별자이므로 오류 기대. **실제 오류 코드를 캡처**(추측 말 것 — 미정의 식별자 오류의 정확한 코드/메시지를 실행해서 확인).
6. `time_jigeum_undefined`: 위와 동일 패턴으로 `지금` 확인.

이 두 케이스가 Q13(0705, 시간 구현) 완료 후 통과해야 할 목표다. 구현 후 기대값: `마디사이 <- 0.02@s.` 채비 선언 시 매 틱 `지금`이 `0.02@s`씩 누적.

### 그룹 3 — 벡터2/텐서 [레드 예상, D3 커널 신규]

7. `vector2_construct_undefined`: `위치 <- 벡터2(3@m, 4@m).` — 현재 `벡터2`가 정의되지 않았으므로 오류 기대. 실제 오류 캡처.
8. `tensor_existing_baseline`: 기존 `텐서.형상`/`텐서.자료` 등이 실제로 동작하는지 최소 1케이스로 확인(그린 예상 — 스펙 §2.2가 "이미 존재"라 주장한 것의 회귀 고정).

### 그룹 4 — Option/Result [레드 예상, D18 커널 신규 — SSOT 개정 대기와 연동]

9. `option_syntax_undefined`: `x: 수? <- 없음.` — 현재 `?` 타입 접미가 파서에 없다면 파싱 오류 기대. 실제 캡처.
10. `result_matum_undefined`: `풀이: 맺음<수> <- 다항식.풀기(수식{ x = 1 }, "x").` — `맺음<T>` 타입이 없다면 오류 기대. 실제 캡처.

**주의**: `PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md`(음/ㅁ 트리거+값참조 꼬리)가 아직 SSOT 미반영이므로, `계산함.` 형태의 값참조 호출도 현재는 실패할 것 — 이것도 레드 케이스로 별도 기록(`value_ref_tail_undefined`), 단 이 케이스는 **SSOT 반영 후에만 구현 대상**이라고 주석에 명시(Q13~18과 다른 선행조건).

### 그룹 5 — 오버플로 [레드 예상, D4 커널 신규 — Q7 데이터로 안전 확인됨]

11. `overflow_saturate_current`: `x <- 2000000000 * 2000000000.` 실행 — 현재는 포화(saturate)되어 조용히 최대값 반환. **현재 동작을 그대로 캡처**(이게 지금의 "그린" — 전환 전 기준선). 전환 후에는 이 케이스가 `E_NUM_OVERFLOW`로 바뀌어야 하므로, 전환 시 이 케이스는 의도적으로 깨지고 갱신되어야 함을 README에 명시.

### 그룹 6 — 포함절 [레드 예상, D28 커널 신규]

12. `relative_clause_undefined`: `(3보다 큰 것) 목록에서 고르기.` 유사 패턴(스펙 §3.3 닫힌 패턴 목록 중 하나 선택) — 현재 파서에 없다면 오류 기대. 실제 캡처.

## 수용 기준

- [ ] 12개 케이스 전부 실제 CLI로 실행되고 캡처된 실제 값으로 golden.jsonl 작성(추정치 금지)
- [ ] 그룹 1, 그룹8(텐서 baseline)은 실제로 통과(그린) 확인 — 실패하면 "예상과 다름"으로 보고, golden 조작 금지
- [ ] 그룹 2~6(그룹8 제외)은 실제로 실패(레드) 확인 — 우연히 통과하면 "예상과 다름"으로 보고(스펙에 이미 구현된 것일 수 있음 — 재조사 필요)
- [ ] 그룹5의 "전환 시 의도적으로 깨짐" 성격을 README에 명시
- [ ] `python tests/run_pack_golden.py lang_kernel_v1_conformance` 실행 결과(전체 PASS든 일부 의도된 결과든)를 보고에 첨부
- [ ] `python tests/run_ci_sanity_gate.py --profile core_lang` PASS(회귀 없음 확인)

## 금지 사항

- main 커밋(브랜치에) / golden 값 임의 조작(레드를 억지로 그린으로 맞추기 금지 — 이게 이 팩의 핵심 목적을 무너뜨림) / 새 커널 기능 구현(이 브리프는 테스트 작성만, Q13~18에서 구현) / 범위 밖 수정

## 보고 형식

이 파일 하단 `## 실행 보고`: 그룹별 실제 결과(예상대로 레드/그린이었는지, 다르면 무엇이 달랐는지) + 실행 로그 경로.

## 실행 보고

- 실행일: 2026-07-06
- 생성 팩: `pack/lang_kernel_v1_conformance/`
- 케이스 수: 13개(기본 12개 + `value_ref_tail_undefined` 별도 레드 케이스)
- 캡처 로그: `I:/home/urihanl/ddn/codex/out/queue-20260706/q-conformance/case_results.json`
- `python tests/run_pack_golden.py lang_kernel_v1_conformance`: PASS
- `python tests/run_ci_sanity_gate.py --profile core_lang`: PASS

| 그룹 | 결과 요약 |
|---|---|
| 형태론 회귀 | `stem_alias_dop_dou`=FAIL(E_RUNTIME_UNDEFINED); `stem_alias_ambiguous`=FAIL(E_RUNTIME_UNDEFINED); `tail_equiv_gi_hagi`=FAIL(E_RUNTIME_UNDEFINED); `condition_new_surface`=PASS |
| 시간 | `time_madisai_undefined`=FAIL(E_RUNTIME_UNDEFINED); `time_jigeum_undefined`=FAIL(E_RUNTIME_UNDEFINED) |
| 벡터2/텐서 | `vector2_construct_undefined`=FAIL(E_PARSE_UNEXPECTED_TOKEN); `tensor_existing_baseline`=PASS |
| Option/Result | `option_syntax_undefined`=FAIL(E_LEX_UNEXPECTED_CHAR); `result_matum_undefined`=FAIL(E_PARSE_UNEXPECTED_TOKEN); `value_ref_tail_undefined`=FAIL(E_RUNTIME_UNDEFINED) |
| 오버플로 | `overflow_saturate_current`=PASS |
| 포함절 | `relative_clause_undefined`=FAIL(E_LEX_BAD_IDENT_START) |

예상과 다른 항목:
- `stem_alias_dop_dou`: 그린 예상이었으나 `E_RUNTIME_UNDEFINED` (`살림.돕기`)
- `tail_equiv_gi_hagi`: 그린 예상이었으나 `E_RUNTIME_UNDEFINED` (`살림.회복기`)
- `stem_alias_ambiguous`: `E_CALL_TAIL_AMBIGUOUS` 기대였으나 제품 run 경로 실제값은 `E_RUNTIME_UNDEFINED` (`살림.계산하기`)

비고: `overflow_saturate_current`는 현재 포화 기준선으로 PASS이며, D4 `E_NUM_OVERFLOW` 전환 시 의도적으로 깨져야 함을 README에 명시했다.
