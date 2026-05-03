# lang_pragmatism_pack_v1

현재 current-line에서 바로 실행 가능한 대표 입력을 묶은 pragmatism golden pack이다.

- evidence_tier: `golden_closed`
- closure_claim: `yes`
- focus:
  - 범위/리스트/문자열 기본 subset이 실제 runner에서 PASS하는지 확인
  - wording mismatch 없이 representative runnable evidence를 고정

실행:

```bash
python tests/run_lang_pragmatism_pack_check.py
python tests/run_pack_golden.py lang_pragmatism_pack_v1
```
