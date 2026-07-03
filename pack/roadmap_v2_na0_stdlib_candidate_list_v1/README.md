# roadmap_v2_na0_stdlib_candidate_list_v1

ROADMAP_V2 `나-0` 표준가지 후보 목록 확정 정합화 pack이다.

이 pack은 새 stdlib surface, parser/runtime 변경, 제품 UI 변경을 주장하지 않는다. 기존 stdlib catalog와 `stdlib_1_v1` 계열 evidence를 `나-0` 후보 목록 확정 좌표에 연결한다.

대표 검증:

```sh
python tests/run_stdlib_catalog_check.py
python tests/run_stdlib_1_check.py
python tests/run_pack_golden.py roadmap_v2_na0_stdlib_candidate_list_v1
python tests/run_roadmap_v2_na0_stdlib_candidate_list_check.py
```
