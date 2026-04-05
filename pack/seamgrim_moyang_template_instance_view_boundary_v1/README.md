# seamgrim_moyang_template_instance_view_boundary_v1

`모양 {}`를 템플릿/인스턴스 패턴으로 사용할 때, 인스턴스 파라미터(반지름/색) 변경이
상태 전이(`state_hash`)를 오염시키지 않는지 확인하는 경계 팩.

검증 포인트:

- 템플릿: `(중심x, 중심y, 반지름, 채움색) 원틀:움직씨 = { 모양 { ... } }`
- 인스턴스: `(x, 0, 0.10, "#38bdf8") 원틀.` vs `(x, 0, 0.22, "#f97316") 원틀.`
- 색 단독 변경 인스턴스: `(x, 0, 0.10, "#f97316") 원틀.`
- 두 입력의 `state_hash`는 동일해야 한다. (view 전용 차이)
- 두 입력의 `bogae_hash`는 달라야 한다. (모양 파라미터 차이 반영)

검증:

- `python tests/run_pack_golden.py seamgrim_moyang_template_instance_view_boundary_v1`
- parent family:
  - `tests/seamgrim_view_hash_family/README.md`
  - `python tests/run_seamgrim_view_hash_family_selftest.py`
