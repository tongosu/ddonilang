# D-PACK: compound_update_basics

## 목적
- `+<-`/`-<-` 복합 갱신 설탕이 `x <- x ± y`로 정본화되는지 확인한다.
- `+=`/`-=`는 진단으로 거부되는지 확인한다.

## 구성
- `input.ddn`: 기본 동작 샘플
- `input_plus_equal.ddn`: `+=` 거부 샘플
- `input_minus_equal.ddn`: `-=` 거부 샘플
- `golden.jsonl`: canon/run 성공/거부 경로를 같이 고정하는 pack golden
- `tests/README.md`: 수동 실행 가이드
- `tests/compound_update_reject_contract/README.md`: `+=`/`-=` 거부 계약면

## DoD(최소)
- `teul-cli canon`에서 `+<-`/`-<-`가 `<-` 전개로 출력된다.
- `teul-cli run`에서 기대 출력이 재현된다.
- `+=`/`-=` 입력은 canon에서 `E_CANON_UNSUPPORTED_COMPOUND_UPDATE`, run에서 `E_PARSE_UNEXPECTED_TOKEN`으로 거부된다.
