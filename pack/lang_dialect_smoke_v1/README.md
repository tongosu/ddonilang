# lang_dialect_smoke_v1

#말씨 헤더 활성화와 영어 키워드 스코프를 최소로 확인하는 스모크 팩이다.

## 목표
- #말씨: en에서 ko + sym3 + en 키워드만 활성화
- 영어 if 키워드가 일때로 인식되는지 확인
- sym3 조건 토큰(?=>)이 말씨와 무관하게 활성인지 확인
- 기본/미지원 말씨에서 `if`가 키워드가 아니라 식별자로 남는지 확인

## 실행
python tests/run_pack_golden.py lang_dialect_smoke_v1
