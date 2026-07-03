# roadmap_v2_ma0_curriculum_catalog_v1

ROADMAP_V2 `마-0` 교과 카탈로그 정합화 pack이다.

이 pack은 새 lesson authoring UI, textbook rewrite, remote classroom, publication workflow, product UI 변경을 주장하지 않는다. 기존 Seamgrim lesson catalog, CurriculumMetaV1, BrowseScreen curriculum detail, and downstream `마-1`~`마-5` curriculum evidence를 `마-0` 교과 카탈로그 좌표에 연결한다.

대표 검증:

```sh
python tests/run_education_curriculum_template_check.py
python tests/run_seamgrim_education_curriculum_template_check.py
python tests/run_pack_golden.py education_curriculum_1_v1 seamgrim_curriculum_2_v1 seamgrim_curriculum_3_v1 seamgrim_curriculum_4_v1 seamgrim_curriculum_5_v1
python tests/run_pack_golden.py roadmap_v2_ma0_curriculum_catalog_v1
python tests/run_roadmap_v2_ma0_curriculum_catalog_check.py
```
