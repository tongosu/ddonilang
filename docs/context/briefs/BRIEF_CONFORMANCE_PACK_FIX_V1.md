# BRIEF: 적합성 팩 그룹1 케이스 3건 정정 (Q18 진단 반영)

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 근거: Q18 진단 결과 — bare `이름꼬리.` 호출은 `Expr::Path`로 파싱되어 `살림.이름꼬리` 경로 참조가 되고, 씨앗 dispatch는 `Expr::Call`(괄호 있는 호출)에서만 일어난다. 저장소 실사례 28건도 전부 `()` 또는 `(인자=값)` 접두 호출 형태였다(bare 형태 없음).
> 성격: 커널 기능 구현 아님. 브리프 예시가 잘못된 호출 문법을 썼던 것을 바로잡는 것뿐.

## 수정 대상 (3개 파일, 호출부만 수정 — 선언부는 그대로)

### `pack/lang_kernel_v1_conformance/cases/stem_alias_dop_dou.ddn`
9행 `돕기.` → `() 돕기.`로 변경.

### `pack/lang_kernel_v1_conformance/cases/tail_equiv_gi_hagi.ddn`
5행 `회복기.` → `() 회복기.`, 6행 `회복하기.` → `() 회복하기.`로 변경.

### `pack/lang_kernel_v1_conformance/cases/stem_alias_ambiguous.ddn`
9행 `계산하기.` → `() 계산하기.`로 변경. (이 케이스는 여전히 `E_CALL_TAIL_AMBIGUOUS` 기대 — 호출 형태만 고치고 기대 오류 코드는 유지)

## 작업

1. 위 3개 파일을 수정한다.
2. 각각 실제로 재실행해 결과를 캡처한다:
   - `stem_alias_dop_dou.ddn`: exit_code=0, stdout에 "도움" 포함 기대(그린)
   - `tail_equiv_gi_hagi.ddn`: exit_code=0, 두 호출 모두 "회복" 출력, 두 결과가 동일한지 확인(그린 — CALL-TAIL-EQUIV-01 검증)
   - `stem_alias_ambiguous.ddn`: `E_CALL_TAIL_AMBIGUOUS` 오류 기대(원래 의도대로 그린)
3. 세 케이스 모두 브리프 원래 의도(그룹1 = 그린/회귀 고정)대로 실제 결과가 나오면 `golden.jsonl`을 그 실제값으로 갱신한다. **여전히 예상과 다르면 golden을 임의로 맞추지 말고 그대로 보고**(이 경우 추가 진단 필요).
4. `README.md`에 이번 수정 이력(bare 호출 → 접두 호출 정정, Q18 근거)을 한 줄 추가한다.

## 수용 기준

- [ ] 3개 파일 수정, 실제 CLI 재실행 결과로 golden.jsonl 갱신(추정 금지)
- [ ] `python tests/run_pack_golden.py lang_kernel_v1_conformance` PASS
- [ ] `python tests/run_ci_sanity_gate.py --profile core_lang` PASS
- [ ] `git diff --check` PASS
- [ ] 위 3파일과 golden.jsonl, README.md 외 변경 없음

## 금지 사항

main 커밋(브랜치에 커밋) / 이 3파일 외 케이스 수정 / 커널 기능 구현 / 범위 밖 수정.

## 보고 형식

이 파일 하단 `## 실행 보고`: 3케이스 각각의 실제 재실행 결과.

## 실행 보고

작성: Codex (2026-07-06)

### 변경

- `pack/lang_kernel_v1_conformance/cases/stem_alias_dop_dou.ddn`: `돕기.` -> `() 돕기.`
- `pack/lang_kernel_v1_conformance/cases/tail_equiv_gi_hagi.ddn`: `회복기.`/`회복하기.` -> `() 회복기.`/`() 회복하기.`
- `pack/lang_kernel_v1_conformance/cases/stem_alias_ambiguous.ddn`: `계산하기.` -> `() 계산하기.`
- `pack/lang_kernel_v1_conformance/golden.jsonl`: 기대와 일치한 `stem_alias_dop_dou`, `tail_equiv_gi_hagi` 2건만 실제 CLI 결과로 갱신.
- `pack/lang_kernel_v1_conformance/README.md`: Q18 근거와 bare 호출 -> 접두 호출 정정 이력, `stem_alias_ambiguous` 불일치 결과를 기록.

### 3케이스 재실행 결과

실행 로그 위치: `I:\home\urihanl\ddn\codex\out\queue-20260706\conformance-fix\`

| 케이스 | 실제 exit_code | stdout 요약 | stderr 요약 | 판정 |
|---|---:|---|---|---|
| `stem_alias_dop_dou` | 0 | `도움` | 없음 | 기대대로 PASS. golden 갱신함. |
| `tail_equiv_gi_hagi` | 0 | `회복`, `회복` | 없음 | 기대대로 PASS. 두 호출 출력 동일. golden 갱신함. |
| `stem_alias_ambiguous` | 0 | state/trace hash만 출력 | 없음 | 기대와 다름. `E_CALL_TAIL_AMBIGUOUS`가 발생하지 않음. golden 임의 갱신하지 않음. |

`stem_alias_ambiguous`의 실제 결과:

```text
cmd=I:\home\urihanl\ddn\codex\target\debug\teul-cli.exe run pack\lang_kernel_v1_conformance\cases\stem_alias_ambiguous.ddn
exit_code=0
stdout:
state_hash=blake3:f83311d1881e4ab122c25dd8c3a9e30349858a9cac5f6066f609c52c4ed9db45
trace_hash=blake3:87696537a1552c09a4cd26b2cc45298e8a2952224f974f1692dfdc8f9663c438
stderr: <empty>
```

### 검증

- `python tests/run_pack_golden.py lang_kernel_v1_conformance`: **FAIL**
  - 실패 1건: `stem_alias_ambiguous`, expected exit_code=1, got exit_code=0.
  - 브리프의 "여전히 예상과 다르면 golden을 임의로 맞추지 말고 그대로 보고" 조건에 따라 이 케이스는 golden을 실제 PASS로 맞추지 않았다.
- `python tests/run_ci_sanity_gate.py --profile core_lang`: **PASS**
- `git diff --check`: **PASS**
- core_lang 실행 중 갱신된 `pack/open_bundle_artifact/run_bundle/geoul.diag.jsonl`, `pack/open_end_to_end/open.log.jsonl`는 검증 부산물이라 원래 상태로 되돌렸다.

### 결론

그린 예상 2건은 Q18 진단대로 접두 호출로 정정하면 제품 CLI에서 PASS한다. 그러나 ambiguous 케이스는 접두 호출로 바꿔도 현재 제품 실행 경로에서 `E_CALL_TAIL_AMBIGUOUS`를 내지 않고 정상 실행된다. 따라서 브리프 수용 기준 중 `lang_kernel_v1_conformance` pack golden PASS는 현재 제품 동작 기준으로 미충족이며, 추가 진단 또는 제품 파서/런타임의 모호성 정책 결정이 필요하다.
