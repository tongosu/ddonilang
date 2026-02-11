# D-PACK: type_runtime_typecheck

## 목적
- DR-067 PinSpec/TypeRef 런타임 타입검사 동작을 확인한다.
- 수/정수/단위/옵션/컨테이너 타입 불일치 사례를 재현한다.

## 구성
- input_num_mismatch.ddn: `수`에 글 입력 (실패)
- input_num_unit_mismatch.ddn: `수`에 `1@m` 입력 (실패)
- input_int_mismatch.ddn: `정수`에 소수 입력 (실패)
- input_unit_mismatch.ddn: `수@m`에 `수@s` 입력 (실패)
- input_optional_ok.ddn: `글?` 생략 (성공)
- input_optional_mismatch.ddn: `글?`에 수 입력 (실패)
- input_list_mismatch.ddn: `(수)차림`에 글 원소 (실패)
- input_set_mismatch.ddn: `(글)모음`에 수 원소 (실패)
- input_map_mismatch.ddn: `(글, 수)짝맞춤`에 수 키 (실패)
- input_infer_ok.ddn: `_` 타입은 검사 생략 (성공)

## DoD(최소)
- *_mismatch.ddn 실행 시 `E_RUNTIME_TYPE_MISMATCH`가 출력된다.
- *_ok.ddn 실행 시 오류 없이 종료한다.
