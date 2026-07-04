# SSOT_AMENDMENT_IMPACT_SCAN_V1

## 범위

- Q17 산출물이다.
- 대상 제안은 텐서 정본화, 명사형 `음/ㅁ` 트리거+값참조 꼬리, D36 `세계영향`→`누리바꿈`이다.
- `docs/ssot/**`를 포함해 읽기 전용으로 스캔했으며, SSOT/제안서/코드/팩 수정은 하지 않았다.
- 검색 범위는 `docs/context`, `docs/status`, `docs/reports`, `docs/steps`, `docs/studio`, `docs/guides`, `docs/ssot`, `publish`, `lang`, `tools`, `tool`, `tests`, `pack`, `solutions`, `core`이다.
- 제외: `.git`, `node_modules`, `target`, `build`, `out`, `__pycache__`, `.pytest_cache`, 바이너리/이미지/압축 산출물, 그리고 이 보고서 자신.

## 재현 명령

- 텐서: `rg -n "겹차림" <검색범위> --glob !<산출물/바이너리>`
- 누리바꿈: `rg -n "세계영향|world_affecting" <검색범위> --glob !<산출물/바이너리>`
- 음/ㅁ 직접 표기: `rg -n "음/ㅁ|~음|~ㅁ|~함|값참조" docs/context/proposals docs/context/reports docs/context/briefs lang tests pack solutions`
- DDN 예시 파생어: `rg -n "계산함|도움|먹음|들음|걸음|잘됨|안됨|맺음" pack solutions --glob *.ddn`
- 구현 근거: `git grep -n -e "choose_variant" -e "surface_form" -e "음/ㅁ" -- lang/src`

## 요약

- 텐서 `겹차림` 참조: 104행 / 51파일
- D36 `세계영향|world_affecting` 참조: 712행 / 92파일
- 음/ㅁ 직접 표기(`음/ㅁ`, `~음`, `~ㅁ`, `~함`, `값참조`): 36행 / 6파일
- DDN 예시 파생어 후보: 20행 / 4파일
- `lang/src` surface 구현 근거: 8행 / 2파일

### 범주별 집계

| 항목 | docs/ssot | docs/context | docs/other | lang | tools | tests | pack | solutions | other |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 텐서 겹차림 | 9 | 70 | 0 | 10 | 12 | 3 | 0 | 0 | 0 |
| 세계영향/world_affecting | 183 | 503 | 3 | 0 | 0 | 15 | 8 | 0 | 0 |
| 음/ㅁ 직접 표기 | 0 | 36 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| DDN 예시 파생어 | 3 | 0 | 0 | 0 | 0 | 0 | 10 | 7 | 0 |
| surface 구현 | 0 | 0 | 0 | 8 | 0 | 0 | 0 | 0 | 0 |

### DDN 예시 파생어 후보 집계

| 토큰 | DDN 행 수 | 해석 |
|---|---:|---|
| `계산함` | 0 | 값참조 꼬리 사용 증거 없음 |
| `도움` | 20 | 모두 도움말 계열 식별자/문자열로 보이며 값참조 꼬리 사용 증거 아님 |
| `먹음` | 0 | 값참조 꼬리 사용 증거 없음 |
| `들음` | 0 | 값참조 꼬리 사용 증거 없음 |
| `걸음` | 0 | 값참조 꼬리 사용 증거 없음 |
| `잘됨` | 0 | 값참조 꼬리 사용 증거 없음 |
| `안됨` | 0 | 값참조 꼬리 사용 증거 없음 |
| `맺음` | 0 | 값참조 꼬리 사용 증거 없음 |

## 1. 텐서 정본화 영향

| 파일 | 줄 | 현재 내용 | 영향 예상 |
|---|---:|---|---|
| `docs/context/all/ssot_context_bundle_ALL.md` | 83327 | > - \`겹차림\`(정본) ↔ \`텐서\`(별칭) | 문서 참조 문구 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 84504 | \| TERM-LEGACY-013 \| 텐서 \| 겹차림 \| tensor 용어 정본화(입력 별칭) \| | 문서 참조 문구 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 84807 | "canonical": "겹차림" | 문서 참조 문구 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 107998 | \| TERM-LEGACY-013 \| 텐서 \| 겹차림 \| tensor 용어 정본화(입력 별칭) \| | 문서 참조 문구 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 108301 | "canonical": "겹차림" | 문서 참조 문구 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 127122 | > - \`겹차림\`(정본) ↔ \`텐서\`(별칭) | 문서 참조 문구 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 1712 | > - \`겹차림\`(정본) ↔ \`텐서\`(별칭) | 문서 참조 문구 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 2889 | \| TERM-LEGACY-013 \| 텐서 \| 겹차림 \| tensor 용어 정본화(입력 별칭) \| | 문서 참조 문구 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 3192 | "canonical": "겹차림" | 문서 참조 문구 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 26383 | \| TERM-LEGACY-013 \| 텐서 \| 겹차림 \| tensor 용어 정본화(입력 별칭) \| | 문서 참조 문구 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 26686 | "canonical": "겹차림" | 문서 참조 문구 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 45507 | > - \`겹차림\`(정본) ↔ \`텐서\`(별칭) | 문서 참조 문구 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 2760 | \| TERM-LEGACY-013 \| 텐서 \| 겹차림 \| tensor 용어 정본화(입력 별칭) \| | 문서 참조 문구 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 3063 | "canonical": "겹차림" | 문서 참조 문구 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 25695 | > - \`겹차림\`(정본) ↔ \`텐서\`(별칭) | 문서 참조 문구 영향 후보. |
| `docs/context/briefs/MISSION_20260705_LONG_TRACK_CONSOLIDATION_V1.md` | 21 | 3. 상태 판정 규칙: SSOT 문면의 명시 표기(normative/docs-first/이월/superseded)를 따르고, 코드 근거는 lang/src·tools/teul-cli/src에서 확인한다. **SSOT와 코드가 모순이면 상태 칸에 \`⚠️모순\`으로 표기하고 양쪽 근거를 남긴다** (예: 겹차림↔텐서 정... | 문서 참조 문구 영향 후보. |
| `docs/context/briefs/MISSION_20260705_LONG_TRACK_CONSOLIDATION_V1.md` | 89 | - 모순 표기: 1건 (\`겹차림\` 정본 / \`텐서\` 별칭 문면 vs \`tensor.v0\` 코드 근거) | 문서 참조 문구 영향 후보. |
| `docs/context/briefs/QUEUE_CODEX_20260706.md` | 124 | 1. \`PROPOSAL_SSOT_TERMS_TENSOR_CANONICAL_20260704.md\`(텐서 정본화): \`겹차림\`을 참조하는 모든 코드/팩/문서 위치 전수. | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_multilang_draft_v1_20260214.detjson` | 402 | "ko_canon": "겹차림", | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_multilang_draft_v1_20260214.detjson` | 416 | "notes": "정본=겹차림, legacy=텐서" | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_multilang_draft_v1_20260214.tsv` | 24 | type 겹차림 텐서 tensor テンソル тензор tensor kikinchay kimsa_tupu tensor piNavu TenSar TenSar tentsorea  정본=겹차림, legacy=텐서 | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_multilang_draft_v3_20260215.detjson` | 1 | {"generated_at":"2026-02-15 (Asia/Seoul)","rows":[{"kind":"decl_kind","ko_canon":"이름씨","ko_alias":"","en":"noun","ja":"名詞","mn":"нэр","tr":"isim","qu":"suti","ay":"suti","ne":"n... | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_multilang_draft_v3_20260215.md` | 43 | \| 겹차림 \| 텐서 \| tensor \| テンソル \| тензор \| tensor \| kikinchay \| kimsa_tupu \| tensor \| piNavu \| TenSar \| TenSar \| tentsorea \|  \| | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_multilang_draft_v3_20260215.tsv` | 22 | type 겹차림 텐서 tensor テンソル тензор tensor kikinchay kimsa_tupu tensor piNavu TenSar TenSar tentsorea  정본=겹차림, legacy=텐서 | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_multilang_draft_v3_20260215_sectionview_20260215.md` | 43 | \| 겹차림 \| 텐서 \| tensor \| テンソル \| тензор \| tensor \| kikinchay \| kimsa_tupu \| tensor \| piNavu \| TenSar \| TenSar \| tentsorea \|  \| | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_multilang_draft_v3_20260215_sectionview_20260215.tsv` | 22 | 1-2 겹차림 텐서 tensor テンソル тензор tensor kikinchay kimsa_tupu tensor piNavu TenSar TenSar tentsorea | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v2_20260213.detjson` | 251 | "ko_canon": "겹차림", | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v2_20260213.detjson` | 253 | "notes": "정본=겹차림, legacy=텐서", | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v2_20260213.tsv` | 24 | type 겹차림 텐서 tensor テンソル тензор tensor  정본=겹차림, legacy=텐서 | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v3_20260213.detjson` | 251 | "ko_canon": "겹차림", | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v3_20260213.detjson` | 253 | "notes": "정본=겹차림, legacy=텐서", | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v3_20260213.tsv` | 24 | type 겹차림 텐서 tensor テンソル тензор tensor  정본=겹차림, legacy=텐서 | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v4_20260215.detjson` | 402 | "ko_canon": "겹차림", | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v4_20260215.detjson` | 416 | "notes": "정본=겹차림, legacy=텐서" | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v4_20260215.tsv` | 24 | type 겹차림 텐서 tensor テンソル тензор tensor kikinchay kimsa_tupu tensor piNavu TenSar TenSar tentsorea  정본=겹차림, legacy=텐서 | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v5_20260215.detjson` | 402 | "ko_canon": "겹차림", | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v5_20260215.detjson` | 416 | "notes": "정본=겹차림, legacy=텐서" | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v5_20260215.md` | 214 | \| 겹차림 \| 텐서 \| tensor \| テンソル \| тензор \| tensor \| kikinchay \| kimsa_tupu \| tensor \| piNavu \| TenSar \| TenSar \| tentsorea \|  \| 정본=겹차림, legacy=텐서 \| | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v5_20260215.tsv` | 24 | type 겹차림 텐서 tensor テンソル тензор tensor kikinchay kimsa_tupu tensor piNavu TenSar TenSar tentsorea  정본=겹차림, legacy=텐서 | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v6_20260215.detjson` | 402 | "ko_canon": "겹차림", | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v6_20260215.detjson` | 416 | "notes": "정본=겹차림, legacy=텐서" | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v6_20260215.md` | 214 | \| 겹차림 \| 텐서 \| tensor \| テンソル \| тензор \| tensor \| kikinchay \| kimsa_tupu \| टेन्सर/tensor \| piNavu \| TenSar \| TenSar \| tentsorea \|  \| 정본=겹차림, legacy=텐서 \| | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v6_20260215.tsv` | 24 | type 겹차림 텐서 tensor テンソル тензор tensor kikinchay kimsa_tupu टेन्सर/tensor piNavu TenSar TenSar tentsorea  정본=겹차림, legacy=텐서 | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v7_20260215.detjson` | 402 | "ko_canon": "겹차림", | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v7_20260215.detjson` | 416 | "notes": "정본=겹차림, legacy=텐서" | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v7_20260215.md` | 209 | \| 겹차림 \| 텐서 \| tensor \| テンソル \| тензор \| tensor \| kikinchay \| kimsa_tupu \| टेन्सर/tensor \| piNavu \| TenSar \| TenSar \| tentsorea \|  \| 정본=겹차림, legacy=텐서 \| | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v7_20260215.tsv` | 24 | type 겹차림 텐서 tensor テンソル тензор tensor kikinchay kimsa_tupu टेन्सर/tensor piNavu TenSar TenSar tentsorea  정본=겹차림, legacy=텐서 | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v8_20260215.detjson` | 402 | "ko_canon":  "겹차림", | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v8_20260215.detjson` | 416 | "notes":  "정본=겹차림, legacy=텐서" | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v8_20260215.md` | 52 | \| 겹차림 \| 텐서 \| tensor \| テンソル \| тензор \| tensor \| kikinchay \| kimsa_tupu \| टेन्सर/tensor \| piNavu \| TenSar \| TenSar \| tentsorea \|  \| 정본=겹차림, legacy=텐서 \| | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v8_20260215.tsv` | 24 | type 겹차림 텐서 tensor テンソル тензор tensor kikinchay kimsa_tupu टेन्सर/tensor piNavu TenSar TenSar tentsorea  정본=겹차림, legacy=텐서 | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v9_20260301.detjson` | 402 | "ko_canon":  "겹차림", | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v9_20260301.detjson` | 416 | "notes":  "정본=겹차림, legacy=텐서" | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v9_20260301.md` | 53 | \| 겹차림 \| 텐서 \| tensor \| テンソル \| тензор \| tensor \| kikinchay \| kimsa_tupu \| टेन्सर/tensor \| piNavu \| TenSar \| TenSar \| tentsorea \|  \| 정본=겹차림, legacy=텐서 \| | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v9_20260301.tsv` | 24 | type 겹차림 텐서 tensor テンソル тензор tensor kikinchay kimsa_tupu टेन्सर/tensor piNavu TenSar TenSar tentsorea  정본=겹차림, legacy=텐서 | 문서 참조 문구 영향 후보. |
| `docs/context/notes/dialect/DDONIRANG_terms_sym_multilang_table_20260213_v2.tsv` | 6 | 겹차림 텐서 tensor テンソル тензор tensör tensor tensor(텐서) 순우리말 정본 | 문서 참조 문구 영향 후보. |
| `docs/context/patches/SSOT_LANG_DIALECT_KEYWORDS_FULL_BLOCK_v2_20260213.md` | 29 | \|type\|겹차림\|텐서\|tensor\|テンソル\|тензор\|tensor\|\|정본=겹차림, legacy=텐서\| | 문서 참조 문구 영향 후보. |
| `docs/context/patches/SSOT_LANG_DIALECT_KEYWORDS_FULL_BLOCK_v3_20260213.md` | 29 | \|type\|겹차림\|텐서\|tensor\|テンソル\|тензор\|tensor\|\|정본=겹차림, legacy=텐서\| | 문서 참조 문구 영향 후보. |
| `docs/context/patches/SSOT_PATCH_LANG_PRETTY_SYM3_V3_20260213.md` | 110 | - \`겹차림\`(정본) ← alias: \`텐서\` | 문서 참조 문구 영향 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_SPEC_20260704.md` | 79 | **정정 확인**: 텐서는 이미 존재한다(stdlib \`텐서.형상/자료/배치/값/바꾼값\`, 입력 별칭 \`겹차림.*\`, CLI \`teul-cli tensor validate/hash/canon\`). **텐서 정본 확정**: \`텐서\`가 정본, \`겹차림\`은 입력 별칭 — SSOT_TERMS 현행(겹차림 ... | 제안/스펙 문서 근거 문구. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_SPEC_20260704.md` | 400 | \| \`PROPOSAL_SSOT_TERMS_TENSOR_CANONICAL_20260704.md\` \| 텐서 정본화(겹차림↔텐서 방향 반전) \| | 제안/스펙 문서 근거 문구. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_APPLY_ALL_PRETTY_SYM3_V3_20260213.md` | 34 | - \`되풀이/되돌림/톺아보기/해봄줄/바른꼴/차림칸/겹차림\` 등: 순우리말 정본을 기본으로 하고, | 제안/스펙 문서 근거 문구. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_APPLY_ALL_PRETTY_SYM3_V3_20260213.md` | 48 | - **\`겹차림(텐서)\`는 텐서 전용**으로 둔다. | 제안/스펙 문서 근거 문구. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SSOT_TERMS_TENSOR_CANONICAL_20260704.md` | 1 | # PROPOSAL: SSOT_TERMS 개정 — 텐서 정본화 (겹차림과 정본-별칭 방향 반전) | 제안/스펙 문서 근거 문구. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SSOT_TERMS_TENSOR_CANONICAL_20260704.md` | 11 | > - \`겹차림\`(정본) ↔ \`텐서\`(별칭) | 제안/스펙 문서 근거 문구. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SSOT_TERMS_TENSOR_CANONICAL_20260704.md` | 17 | > - \`텐서\`(정본) ↔ \`겹차림\`(별칭) | 제안/스펙 문서 근거 문구. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SSOT_TERMS_TENSOR_CANONICAL_20260704.md` | 23 | 2. **교육 목적 정합:** 이 언어의 목적은 교과 이해다. 학습자가 교과서·대학 교재에서 만나는 용어(텐서)와 언어 정본이 일치해야 하고, \`겹차림\`은 아름다운 순우리말이므로 입력 별칭으로 영구 보존한다. | 제안/스펙 문서 근거 문구. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SSOT_TERMS_TENSOR_CANONICAL_20260704.md` | 24 | 3. **코드 현행과 일치:** \`lang/src/stdlib.rs\`는 이미 \`겹차림.*\` → \`텐서.*\`로 정본화한다 (stdlib.rs:68~72). 본 개정으로 SSOT-코드 모순(인벤토리 ⚠️ 1건)이 해소된다. | 제안/스펙 문서 근거 문구. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SSOT_TERMS_TENSOR_CANONICAL_20260704.md` | 29 | 2. TERMS/LANG 내 "겹차림"을 정본으로 전제한 서술이 있으면 동일 원칙으로 정리 (예: TERMS:494 텐서 정의부는 이미 \`텐서\` 명칭 사용 — 변경 불요). | 제안/스펙 문서 근거 문구. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/reports/SSOT_LANG_INVENTORY_V1.md` | 39 | \| 겹차림/텐서 \| 타입 \| 겹차림 \| 텐서 \| ⚠️모순 \| docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:444; docs/ssot/ssot/SSOT_TERMS_v24.12.9.md:494 \| tools/teul-cli/src/cli/tensor.rs:67 \| | 기존 감사/인벤토리 문서의 모순 표기 갱신 후보. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 1278 | > - \`겹차림\`(정본) ↔ \`텐서\`(별칭) | SSOT 본문 개정 대상: 텐서 정본/겹차림 별칭 방향 반전. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 2455 | \| TERM-LEGACY-013 \| 텐서 \| 겹차림 \| tensor 용어 정본화(입력 별칭) \| | SSOT 본문 개정 대상: 텐서 정본/겹차림 별칭 방향 반전. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 2758 | "canonical": "겹차림" | SSOT 본문 개정 대상: 텐서 정본/겹차림 별칭 방향 반전. |
| `docs/ssot/releases/v23.0.6/SSOT_LANG_v23.0.6.md` | 314 | \| TERM-LEGACY-013 \| 텐서 \| 겹차림 \| tensor 용어 정본화(입력 별칭) \| | SSOT 본문 개정 대상: 텐서 정본/겹차림 별칭 방향 반전. |
| `docs/ssot/releases/v23.0.6/SSOT_LANG_v23.0.6.md` | 617 | "canonical": "겹차림" | SSOT 본문 개정 대상: 텐서 정본/겹차림 별칭 방향 반전. |
| `docs/ssot/releases/v23.0.6/SSOT_TERMS_v23.0.6.md` | 81 | > - \`겹차림\`(정본) ↔ \`텐서\`(별칭) | SSOT 본문 개정 대상: 텐서 정본/겹차림 별칭 방향 반전. |
| `docs/ssot/ssot/SSOT_LANG_v24.12.9.md` | 988 | \| TERM-LEGACY-013 \| 텐서 \| 겹차림 \| tensor 용어 정본화(입력 별칭) \| | SSOT 본문 개정 대상: 텐서 정본/겹차림 별칭 방향 반전. |
| `docs/ssot/ssot/SSOT_LANG_v24.12.9.md` | 1291 | "canonical": "겹차림" | SSOT 본문 개정 대상: 텐서 정본/겹차림 별칭 방향 반전. |
| `docs/ssot/ssot/SSOT_TERMS_v24.12.9.md` | 444 | > - \`겹차림\`(정본) ↔ \`텐서\`(별칭) | SSOT 본문 개정 대상: 텐서 정본/겹차림 별칭 방향 반전. |
| `lang/src/stdlib.rs` | 68 | "겹차림.형상" => "텐서.형상", | 코드는 이미 겹차림 입력을 텐서 정본으로 canonicalize; SSOT 반영 시 코드 방향과 정합. |
| `lang/src/stdlib.rs` | 69 | "겹차림.자료" => "텐서.자료", | 코드는 이미 겹차림 입력을 텐서 정본으로 canonicalize; SSOT 반영 시 코드 방향과 정합. |
| `lang/src/stdlib.rs` | 70 | "겹차림.배치" => "텐서.배치", | 코드는 이미 겹차림 입력을 텐서 정본으로 canonicalize; SSOT 반영 시 코드 방향과 정합. |
| `lang/src/stdlib.rs` | 71 | "겹차림.값" => "텐서.값", | 코드는 이미 겹차림 입력을 텐서 정본으로 canonicalize; SSOT 반영 시 코드 방향과 정합. |
| `lang/src/stdlib.rs` | 72 | "겹차림.바꾼값" => "텐서.바꾼값", | 코드는 이미 겹차림 입력을 텐서 정본으로 canonicalize; SSOT 반영 시 코드 방향과 정합. |
| `lang/src/stdlib.rs` | 2318 | assert_eq!(canonicalize_stdlib_alias("겹차림.형상"), "텐서.형상"); | 코드는 이미 겹차림 입력을 텐서 정본으로 canonicalize; SSOT 반영 시 코드 방향과 정합. |
| `lang/src/stdlib.rs` | 2319 | assert_eq!(canonicalize_stdlib_alias("겹차림.자료"), "텐서.자료"); | 코드는 이미 겹차림 입력을 텐서 정본으로 canonicalize; SSOT 반영 시 코드 방향과 정합. |
| `lang/src/stdlib.rs` | 2320 | assert_eq!(canonicalize_stdlib_alias("겹차림.배치"), "텐서.배치"); | 코드는 이미 겹차림 입력을 텐서 정본으로 canonicalize; SSOT 반영 시 코드 방향과 정합. |
| `lang/src/stdlib.rs` | 2321 | assert_eq!(canonicalize_stdlib_alias("겹차림.값"), "텐서.값"); | 코드는 이미 겹차림 입력을 텐서 정본으로 canonicalize; SSOT 반영 시 코드 방향과 정합. |
| `lang/src/stdlib.rs` | 2322 | assert_eq!(canonicalize_stdlib_alias("겹차림.바꾼값"), "텐서.바꾼값"); | 코드는 이미 겹차림 입력을 텐서 정본으로 canonicalize; SSOT 반영 시 코드 방향과 정합. |
| `tests/toolchain_golden/ai_prompt_lean.txt` | 550 | > - \`겹차림\`(정본) ↔ \`텐서\`(별칭) | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tests/toolchain_golden/ai_prompt_lean.txt` | 3022 | \| TERM-LEGACY-013 \| 텐서 \| 겹차림 \| tensor 용어 정본화(입력 별칭) \| | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tests/toolchain_golden/ai_prompt_lean.txt` | 3325 | "canonical": "겹차림" | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tools/teul-cli/tests/golden/W91/W91_G01_ai_prompt_invariants/ai.prompt.txt` | 179 | > - \`겹차림\`(정본) ↔ \`텐서\`(별칭) | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tools/teul-cli/tests/golden/W91/W91_G01_ai_prompt_invariants/ai.prompt.txt` | 4326 | - \`되풀이/되돌림/톺아보기/맞물림씨/겹차림/바른꼴/해봄줄/차림판\`을 정본(우선)으로 두고, | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tools/teul-cli/tests/golden/W91/W91_G01_ai_prompt_invariants/ai.prompt.txt` | 5450 | \| TERM-LEGACY-013 \| 텐서 \| 겹차림 \| tensor 용어 정본화(입력 별칭) \| | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tools/teul-cli/tests/golden/W91/W91_G01_ai_prompt_invariants/ai.prompt.txt` | 5754 | "canonical": "겹차림" | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tools/teul-cli/tests/golden/W91/W91_G02_ai_prompt_snapshot/ai.prompt.golden.txt` | 148 | > - \`겹차림\`(정본) ↔ \`텐서\`(별칭) | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tools/teul-cli/tests/golden/W91/W91_G02_ai_prompt_snapshot/ai.prompt.golden.txt` | 4228 | - \`되풀이/되돌림/톺아보기/맞물림씨/겹차림/바른꼴/해봄줄/차림판\`을 정본(우선)으로 두고, | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tools/teul-cli/tests/golden/W91/W91_G02_ai_prompt_snapshot/ai.prompt.golden.txt` | 5020 | \| TERM-LEGACY-013 \| 텐서 \| 겹차림 \| tensor 용어 정본화(입력 별칭) \| | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tools/teul-cli/tests/golden/W91/W91_G02_ai_prompt_snapshot/ai.prompt.golden.txt` | 5324 | "canonical": "겹차림" | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tools/teul-cli/tests/golden/W91/W91_G02_ai_prompt_snapshot/ai.prompt.txt` | 179 | > - \`겹차림\`(정본) ↔ \`텐서\`(별칭) | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tools/teul-cli/tests/golden/W91/W91_G02_ai_prompt_snapshot/ai.prompt.txt` | 4326 | - \`되풀이/되돌림/톺아보기/맞물림씨/겹차림/바른꼴/해봄줄/차림판\`을 정본(우선)으로 두고, | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tools/teul-cli/tests/golden/W91/W91_G02_ai_prompt_snapshot/ai.prompt.txt` | 5450 | \| TERM-LEGACY-013 \| 텐서 \| 겹차림 \| tensor 용어 정본화(입력 별칭) \| | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tools/teul-cli/tests/golden/W91/W91_G02_ai_prompt_snapshot/ai.prompt.txt` | 5754 | "canonical": "겹차림" | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |

## 2. 명사형 음/ㅁ 트리거+값참조 꼬리 영향

### 2.1 직접 표기/문서 영향

| 파일 | 줄 | 현재 내용 | 영향 예상 |
|---|---:|---|---|
| `docs/context/briefs/BRIEF_LANG_KERNEL_CONFORMANCE_PACK_V1.md` | 56 | **주의**: \`PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md\`(음/ㅁ 트리거+값참조 꼬리)가 아직 SSOT 미반영이므로, \`계산함.\` 형태의 값참조 호출도 현재는 실패할 것 — 이것도 레드 케이스로 별도 기록(\`value_ref_tail_undefined\`), 단 이 케이스... | 적합성 팩 레드 케이스 기준. SSOT 반영 전에는 실패 기대 유지. |
| `docs/context/briefs/QUEUE_CODEX_20260706.md` | 125 | 2. \`PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md\`(음/ㅁ 트리거+값참조 꼬리): 이 SSOT 변경이 반영되면 \`~음/~ㅁ/~함\` 형태를 이미 쓰고 있는 기존 코드/팩이 있는지(있다면 어떻게 처리되던 것인지). | 명사형 꼬리 논의 문서 영향 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 173 | - **D21-보강 (Claude 제안, 확정 대기) — 명사형 이형(二形) 규칙**: \`~기\` = **행동**(호출·인용·조합 가능: 만들기/풀기/잇기/멈추기), \`~음\` = **상태·결과·구조**(값·블록: 보임/매김/세움/없음/잘됨/맺음). 현행 정본이 이미 이 규칙을 거의 완벽히 따름 — 명문화만 필요... | 논의 기록/방향 문서. 최종 채택 시 역사 문맥 또는 최신 상태 표기 정리 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 174 | - **D21-보강2 (2026-07-04 사용자 형태론 지적 반영) — 생산성 비대칭**: \`~음\`은 불규칙 활용(걷다→걸음, 돕다→도움)이라 기계적 파생 불가, \`~기\`와 \`되다\` 축(되기/됨)은 완전 규칙 — 사용자 지적. 해법: **\`~기\`=생산적 형(사용자 파생 자유, 100% 규칙)**, *... | 논의 기록/방향 문서. 최종 채택 시 역사 문맥 또는 최신 상태 표기 정리 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 266 | **2026-07-05 추가 (Codex \`:움직어근\` 제안 검토 후 기각, 대안 채택):** Codex가 \`계산:움직어근\` 같은 신규 씨(품사) 구성물을 제안했으나 기각한다 — 씨 체계를 늘리지 않고도 §V18-00C의 **동치 꼬리 집합에 4번째 항목("값참조": 음/ㅁ/함)을 추가**하면 동일 목표(계산... | 논의 기록/방향 문서. 최종 채택 시 역사 문맥 또는 최신 상태 표기 정리 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 273 | 2. **D21-보강 최종 확정 (규범 문안)**: \`~기\`형은 생산적 문법 — 행동/과정/행동 인용, 사용자 파생 허용. \`~음\`형은 **생산적 문법이 아니다** — 의미 분류 표지이자 어휘 설계 원칙일 뿐. 닫힌 커널 어휘(보임/매김/세움/맺음/없음/잘됨/안됨/드러냄/맡음 — 단순 토큰으로 처리)와 사용자... | 논의 기록/방향 문서. 최종 채택 시 역사 문맥 또는 최신 상태 표기 정리 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 288 | **실제 빈 곳 (진짜 신규 기여):** §11.3.3의 짝어미계열 목록에 **\`음/ㅁ\`(명사형 나눔형) 없음**. \`surface.rs\`/GATE0 체크리스트 어디에도 음/ㅁ 처리 없음(확인함). 보수 규칙대로면 현재 SSOT 문면 기준 \`돕+음\`의 결과(도움 vs 돕음)가 미지정 상태 — 이것이 D18... | 논의 기록/방향 문서. 최종 채택 시 역사 문맥 또는 최신 상태 표기 정리 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 290 | **D18/D21 재정정:** "닫힌 커널 어휘 30개만 \`~음\` 허용"이라는 이전 Claude 프레임은 SSOT 실제 모델보다 좁음. **SSOT 실제 원칙: \`~\`로 제대로 선언된 동사는(커널 소유든 사용자 선언이든) 모든 활용형이 안전 — 폐쇄 목록이 아니라 선언 기반 개방.** 사용자가 제시한 \`계산... | 논의 기록/방향 문서. 최종 채택 시 역사 문맥 또는 최신 상태 표기 정리 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 292 | **후속 조치:** ①SSOT_LANG §11.3.3에 \`음/ㅁ\`을 짝어미계열로 추가하는 개정 제안서 작성 (별도 파일) ②\`GATE0_IMPLEMENTATION_CHECKLIST §6.6\` 골든 데이터 절 신설 제안(돕~도우+음/ㅁ→도움 등 최소셋 포함) ③커널 스펙 v0.3은 이 영역 전체를 **SSOT ... | 논의 기록/방향 문서. 최종 채택 시 역사 문맥 또는 최신 상태 표기 정리 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 298 | **D21 최종 결정에 대한 영향 — 없음(재확인, 정밀화됨):** 생성(known stem→surface)과 인식(surface→stem 역산)은 다른 문제다. \`surface.rs\`는 전자만 푼다(불규칙 교체를 호출자가 미리 명시해야 함). \`~음/~ㅁ\`의 어미 선택(자음끝=\`~음\`/모음끝=\`~ㅁ\`... | 논의 기록/방향 문서. 최종 채택 시 역사 문맥 또는 최신 상태 표기 정리 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 300 | **백로그 (v1 비블로커):** \`surface_form\`을 canon.rs/normalizer.rs의 **닫힌 어휘 생성 도구**로 배선 — 커널에 새 B층 낱말(예: 향후 추가될 \`~음\`형 정본)을 추가할 때 불규칙 활용을 손으로 틀리지 않게 검증하는 용도. 사용자 입력 파싱 경로에는 영향 없음. | 논의 기록/방향 문서. 최종 채택 시 역사 문맥 또는 최신 상태 표기 정리 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_SPEC_20260704.md` | 26 | 6. **씨앗 이름의 품사를 추정하지 않는다.** [SSOT 인용 — §V18-00/00C] 형태소 분석(하다류 판별·불규칙 활용)을 하지 않는다. 호출 표면형의 "꼬리"만으로 실행/연쇄/조건/값참조를 표현한다(§4 상세). | 커널 스펙의 SSOT 개정 대기 표기. SSOT 반영 후 상태 문구 갱신 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_SPEC_20260704.md` | 181 | - (b) 모음 끼움 짝어미 계열: 니/으니, 면/으면, 니까/으니까, 며/으며, 면서/으면서 (**+ 음/ㅁ [SSOT 개정 대기]** — \`PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md\`) | 커널 스펙의 SSOT 개정 대기 표기. SSOT 반영 후 상태 문구 갱신 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_SPEC_20260704.md` | 199 | \| **값참조** (신규) \| \`~음/~ㅁ\` ≡ \`~함\` \| **[SSOT 개정 대기]** — 명사화 결과/상태 (\`PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md\`) \| | 커널 스펙의 SSOT 개정 대기 표기. SSOT 반영 후 상태 문구 갱신 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_SPEC_20260704.md` | 208 | - **\`~음/~ㅁ/~함\` = 생산적 문법이 아니다**: 의미 분류 표지이자 어휘 설계 원칙일 뿐. **파서는 ~음 파생을 계산하지 않으며 불규칙 활용을 처리하지 않는다**(파서 ≠ 형태소 분석기). 허용 대상: ①닫힌 커널 어휘(보임/매김/세움/맺음/없음/잘됨/안됨/드러냄/맡음 — 단순 토큰) ②사용자가 **명... | 커널 스펙의 SSOT 개정 대기 표기. SSOT 반영 후 상태 문구 갱신 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_SPEC_20260704.md` | 209 | - 임의 \`~음\` 파생은 \`E_NOMINALIZER_EUM_NOT_PRODUCTIVE\` + fix-it("행동이면 \`~기\`, 상태·결과면 명시 선언"). | 커널 스펙의 SSOT 개정 대기 표기. SSOT 반영 후 상태 문구 갱신 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_SPEC_20260704.md` | 211 | - **언어심리학적 근거**: \`~기\`는 화자에게 실시간 생산적 파생으로 처리되나(걷기/돕기 모두 즉시 자연스러움), \`~음\`(특히 불규칙, "도움")은 어휘화된 고정 낱말로 처리되는 경향 — B층 설계와 정합. | 커널 스펙의 SSOT 개정 대기 표기. SSOT 반영 후 상태 문구 갱신 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_SPEC_20260704.md` | 216 | \`lang/src/surface.rs::surface_form/choose_variant\`(어간 교체 생성 엔진, \`"음/ㅁ"\` 토큰 지원은 코드 수정 없이 즉시 동작 — 범용 함수 확인됨)를 canon.rs/normalizer.rs에 배선. 현재 미배선이나 사용자 입력 파싱 경로에는 영향 없음. | 커널 스펙의 SSOT 개정 대기 표기. SSOT 반영 후 상태 문구 갱신 후보. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_SPEC_20260704.md` | 401 | \| \`PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md\` \| §11.3.3에 음/ㅁ 트리거 추가 + §V18-00C 4번째 꼬리 집합(값참조) 신설 \| | 커널 스펙의 SSOT 개정 대기 표기. SSOT 반영 후 상태 문구 갱신 후보. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 1 | # PROPOSAL: SSOT_LANG §11.3.3 개정 + §V18-00C 확장 — 명사형 \`음/ㅁ\` 교대 트리거 및 값참조 꼬리 집합 추가 | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 13 | **명사형 \`음/ㅁ\`이 없다.** §11.3.6이 참조하는 \`GATE0_IMPLEMENTATION_CHECKLIST §6.6\`도 실제로 존재하지 않는 절이며, \`lang/src/surface.rs\` 구현에도 \`음/ㅁ\` 처리가 없다(코드 확인 완료). | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 22 | - 예: 니/으니, 면/으면, 니까/으니까, 며/으며, 면서/으면서, 음/ㅁ   ← 추가 | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 27 | (a) 모음 끼움 짝어미 선택(으-끼움 쌍): 받침 유무에 따라 니/으니, 면/으면, 니까/으니까, 음/ㅁ 등을 선택한다. | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 35 | 돕~도우 + 음/ㅁ → 도움 | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 36 | 듣~들 + 음/ㅁ → 들음 | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 37 | 걷~걸 + 음/ㅁ → 걸음 | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 38 | 먹 + 음/ㅁ → 먹음      (규칙, 별칭 불필요 확인) | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 39 | 가 + 음/ㅁ → 감        (규칙, 모음 끝 → ㅁ 직접 결합) | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 40 | 하 + 음/ㅁ → 함        (규칙, N+하다 복합동사 전체에 적용 — 계산하다→계산함 등) | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 46 | 2. \`음/ㅁ\`은 언어학적으로 \`니/으니\`, \`면/으면\`과 같은 부류(받침 유무에 따른 모음 끼움 짝어미)이므로, 기존 트리거 판정 로직(§11.3.3 구현 메모: "트리거 판정은 표면 문자열의 첫 글자가 아니라 어미 토큰의 정본 분류에 의해 이뤄진다")을 그대로 재사용 가능 — 신규 구현 로직 불필요, ... | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 47 | 3. 이 보강이 있어야 언어의 명사형 표면(\`문서/예시에서 ~하기를 권장\` — §10.5 인근 927행)의 짝인 \`~음/~ㅁ\` 결과 표면(맺음/결과 타입의 \`잘됨/안됨\`, 상태 표현 등 — LANG_KERNEL_V1 D18 논의)이 정본 파이프라인 위에서 안전하게 성립한다. | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 52 | \`계산:셈씨\`(또는 \`:움직씨\`)처럼 N+하다 표제어를 별칭 없이 선언했을 때, \`계산하기\`(실행)는 기존 CALL-TAIL-EQUIV-01의 실행 집합(\`기\`≡\`하기\`)으로 이미 커버되지만, **\`계산함\`(값/상태/결과 참조)에 대응하는 꼬리 집합이 없다.** 이는 §11.3.3의 \`음/ㅁ\... | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 62 | 값참조(명사화 결과/상태): ~음/~ㅁ 및 별칭 ~함(하다형 전용) | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 63 | 예: 도움(돕~도우), 먹음(먹), 감(가), 계산함(계산, 하다형이므로 항상 ~ㅁ→함) | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/proposals/PROPOSAL_SSOT_LANG_EUM_TRIGGER_20260705.md` | 72 | \`lang/src/surface.rs::choose_variant\`는 이미 임의의 \`"X/Y"\` 토큰 쌍을 지원하는 범용 함수이므로(\`"니/으니"\`와 동일 경로), \`"음/ㅁ"\` 토큰을 트리거 목록에 추가하는 것 외에 **함수 로직 변경 불필요**. 실제 호출부 배선(canon.rs/normalizer... | 제안서 원문. 반영 시 SSOT §11.3.3/§V18-00C 개정 근거. |
| `docs/context/roadmap/PLAN_WEEK_20260706.md` | 8 | - 사용자 대기: SSOT 개정 제안 3건(텐서 정본화/음·ㅁ 트리거+값참조/누리바꿈) 반영 여부 — Q17 조사 완료 후 한 번에 검토 권장 | 명사형 꼬리 논의 문서 영향 후보. |

### 2.2 DDN 입력 예시 파생어 후보

| 파일 | 줄 | 현재 내용 | 영향 예상 |
|---|---:|---|---|
| `docs/ssot/pack/game_maker_tetris_slice0/input.ddn` | 680 | 도움말.모양.트레잇 <- #보개/2D.Text. | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `docs/ssot/pack/game_maker_tetris_slice0/input.ddn` | 681 | 도움말.모양.글씨 <- (x=살림.패널_텍스트_x, y=살림.패널_텍스트_y + (살림.패널_줄간격 * 3), size=10, text="LEFT/RIGHT 이동 DOWN 소프트 UP 회전 Z 반회전 X 홀드 SPACE 하드"). | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `docs/ssot/pack/game_maker_tetris_slice0/input.ddn` | 682 | 도움말.모양.색 <- "#88c0d0ff". | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `pack/game_maker_tetris_full/input.ddn` | 56 | 도움말_텍스트:글 <- "왼/오 이동 아래 소프트 위 회전 Z 반회전 X 홀드 스페이스 하드". | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `pack/game_maker_tetris_full/input.ddn` | 82 | 도움말:묶음. | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `pack/game_maker_tetris_full/input.ddn` | 179 | 도움말.생김새.결 <- #보개/2D.Text. | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `pack/game_maker_tetris_full/input.ddn` | 180 | 도움말.생김새.글씨 <- (x=패널_텍스트_x, y=패널_텍스트_y + (패널_줄간격 * 6), size=10, text=도움말_텍스트). | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `pack/game_maker_tetris_full/input.ddn` | 181 | 도움말.생김새.색 <- "#88c0d0ff". | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `pack/game_maker_tetris_slice0/input.ddn` | 50 | 도움말_텍스트:글 <- "왼쪽/오른쪽 이동 아래쪽 소프트 위쪽 회전 Z키 반회전 X키 홀드 스페이스 하드". | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `pack/game_maker_tetris_slice0/input.ddn` | 59 | 도움말:묶음. | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `pack/game_maker_tetris_slice0/input.ddn` | 112 | 도움말.생김새.결 <- #보개/2D.Text. | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `pack/game_maker_tetris_slice0/input.ddn` | 113 | 도움말.생김새.글씨 <- (x=패널_텍스트_x, y=패널_텍스트_y + (패널_줄간격 * 3), size=10, text=도움말_텍스트). | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `pack/game_maker_tetris_slice0/input.ddn` | 114 | 도움말.생김새.색 <- "#88c0d0ff". | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `solutions/seamgrim_ui_mvp/samples/11_tetris_full_playable.ddn` | 55 | 도움말_텍스트:글 <- "←/→ 이동 ↓ 소프트 ↑ 회전 Z 반회전 X 홀드 Space 하드". | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `solutions/seamgrim_ui_mvp/samples/11_tetris_full_playable.ddn` | 81 | 도움말:묶음. | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `solutions/seamgrim_ui_mvp/samples/11_tetris_full_playable.ddn` | 198 | 도움말 <- () 짝맞춤. | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `solutions/seamgrim_ui_mvp/samples/11_tetris_full_playable.ddn` | 199 | 도움말.생김새 <- () 짝맞춤. | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `solutions/seamgrim_ui_mvp/samples/11_tetris_full_playable.ddn` | 200 | 도움말.생김새.결 <- "#보개/2D.Text". | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `solutions/seamgrim_ui_mvp/samples/11_tetris_full_playable.ddn` | 201 | 도움말.생김새.글씨 <- ("x", 패널_텍스트_x, "y", 패널_텍스트_y + (패널_줄간격 * 6), "size", 10, "text", 도움말_텍스트) 짝맞춤. | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |
| `solutions/seamgrim_ui_mvp/samples/11_tetris_full_playable.ddn` | 202 | 도움말.생김새.색 <- "#88c0d0ff". | DDN 식별자/문자열의 도움말 계열. 값참조 꼬리 사용 증거가 아니라 동형 일반 명사로 보임. |

### 2.3 구현 영향 근거

| 파일 | 줄 | 현재 내용 | 영향 예상 |
|---|---:|---|---|
| `lang/src/lib.rs` | 50 | pub use surface::{surface_form, SurfaceError}; | 표면 생성 API/어간 별칭 처리. 값참조 꼬리 배선 시 호출부 영향 후보. |
| `lang/src/surface.rs` | 14 | pub fn surface_form(stem: &str, morphemes: &[&str]) -> Result<String, SurfaceError> { | 표면 생성 API/어간 별칭 처리. 값참조 꼬리 배선 시 호출부 영향 후보. |
| `lang/src/surface.rs` | 18 | let (base, alt) = split_stem(stem); | 표면 생성 API/어간 별칭 처리. 값참조 꼬리 배선 시 호출부 영향 후보. |
| `lang/src/surface.rs` | 24 | let first = choose_variant(&out_stem, morphemes[0]); | 범용 X/Y 어미 선택 로직. 제안서 주장처럼 음/ㅁ 토큰 추가 시 재사용 가능. |
| `lang/src/surface.rs` | 32 | fn split_stem(stem: &str) -> (&str, Option<&str>) { | 표면 생성 API/어간 별칭 처리. 값참조 꼬리 배선 시 호출부 영향 후보. |
| `lang/src/surface.rs` | 41 | fn choose_variant(stem: &str, token: &str) -> String { | 범용 X/Y 어미 선택 로직. 제안서 주장처럼 음/ㅁ 토큰 추가 시 재사용 가능. |
| `lang/src/surface.rs` | 42 | if let Some(idx) = token.find('/') { | 범용 X/Y 어미 선택 로직. 제안서 주장처럼 음/ㅁ 토큰 추가 시 재사용 가능. |
| `lang/src/surface.rs` | 138 | let out = surface_form(stem, &morphemes).expect("surface"); | 표면 생성 API/어간 별칭 처리. 값참조 꼬리 배선 시 호출부 영향 후보. |

## 3. 누리바꿈(D36) 영향

| 파일 | 줄 | 현재 내용 | 영향 예상 |
|---|---:|---|---|
| `docs/context/all/DEV_SUMMARY.md` | 14821 | - \`행동갈래_목록\`: \`보기만/세계영향\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/DEV_SUMMARY.md` | 15097 | - \`세계영향\` = world_affecting 정본 (\`누리영향\` 아님) | 문서/검증 문자열 영향 후보. |
| `docs/context/all/archive/ssot_pack_ALL__20260405T055430Z.md` | 714 | 4. **view_only / world_affecting 분리** | 문서/검증 문자열 영향 후보. |
| `docs/context/all/archive/ssot_pack_ALL__20260405T055430Z.md` | 718 | - 행동/제출 버튼은 \`world_affecting\` 입력 사건으로 처리함을 문서화 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/archive/ssot_pack_ALL__20260405T055430Z.md` | 721 | - \`world_affecting_submit\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/archive/ssot_pack_ALL__20260405T055430Z.md` | 176436 | ===== FILE: docs\ssot\pack\world_affecting_view_switch_v1\expected\PLACEHOLDER.md ===== | 문서/검증 문자열 영향 후보. |
| `docs/context/all/archive/ssot_pack_ALL__20260405T055430Z.md` | 176437 | # world_affecting_view_switch_v1 expected placeholder | 문서/검증 문자열 영향 후보. |
| `docs/context/all/archive/ssot_pack_ALL__20260405T055430Z.md` | 176442 | - action.kind = world_affecting | 문서/검증 문자열 영향 후보. |
| `docs/context/all/archive/ssot_pack_ALL__20260405T055430Z.md` | 176446 | ===== FILE: docs\ssot\pack\world_affecting_view_switch_v1\meta.toml ===== | 문서/검증 문자열 영향 후보. |
| `docs/context/all/archive/ssot_pack_ALL__20260405T055430Z.md` | 176447 | id = "world_affecting_view_switch_v1" | 문서/검증 문자열 영향 후보. |
| `docs/context/all/archive/ssot_pack_ALL__20260405T055430Z.md` | 176450 | ===== FILE: docs\ssot\pack\world_affecting_view_switch_v1\README.md ===== | 문서/검증 문자열 영향 후보. |
| `docs/context/all/archive/ssot_pack_ALL__20260405T055430Z.md` | 176451 | # world_affecting_view_switch_v1 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/archive/ssot_pack_ALL__20260405T055430Z.md` | 176458 | 1. action.kind = world_affecting | 문서/검증 문자열 영향 후보. |
| `docs/context/all/archive/ssot_pack_ALL__20260405T055430Z.md` | 176462 | ===== FILE: docs\ssot\pack\world_affecting_view_switch_v1\tests\README.md ===== | 문서/검증 문자열 영향 후보. |
| `docs/context/all/archive/ssot_pack_ALL__20260405T055430Z.md` | 176463 | # tests for world_affecting_view_switch_v1 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1350 | concept_term,world_affecting,ko,세계영향,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1351 | concept_term,world_affecting,ko_common,세계영향,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1352 | concept_term,world_affecting,sym3,,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1353 | concept_term,world_affecting,en,world_affecting,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1354 | concept_term,world_affecting,ja,世界影響,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1355 | concept_term,world_affecting,mn,ертөнцөд_нөлөөлөх,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1356 | concept_term,world_affecting,tr,dünya_etkili,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1357 | concept_term,world_affecting,qu,⟦TBD_NATIVE_REVIEW⟧,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1358 | concept_term,world_affecting,ay,⟦TBD_NATIVE_REVIEW⟧,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1359 | concept_term,world_affecting,ne,विश्व_प्रभाव,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1360 | concept_term,world_affecting,ta,உலக_பாதிப்பு,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1361 | concept_term,world_affecting,te,ప్రపంచ_ప్రభావ,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1362 | concept_term,world_affecting,kn,ಲೋಕ_ಪ್ರಭಾವ,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1363 | concept_term,world_affecting,eu,mundu_eragin,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_appendix_ALL.md` | 1434 | world_affecting,T0/T1 proposed; T2 draft,세계영향,세계영향,,world_affecting,世界影響,ертөнцөд_нөлөөлөх,dünya_etkili,⟦TBD_NATIVE_REVIEW⟧,⟦TBD_NATIVE_REVIEW⟧,विश्व_प्रभाव,உலக_பாதிப்பு,ప్రపంచ_... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 570 | - \`action.kind = view_only \| world_affecting\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 81673 | > - (AXIS LOCK) 외부개입은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 81674 | > - (REPLAY RULE) \`재연주입\`은 원천 재호출 없이 봉인 기록을 다시 넣는 경로다. replay run 에서 **\`세계영향\` 실주입은 금지**하고, \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 81692 | > - (AXES FIX) 실행축 canonical은 **\`#모두필요 / #빈칸채움\`**, **\`세계영향\`**, **\`한걸음(designed / STUB_ONLY)\`**, **\`살핌내주기갈래 = 설정.거울\`**, **\`놀이기본 = 없음보람 + 해당없음(N/A)\`** 로 정렬한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 81752 | > - (PLATFORM/RULE) 버튼/입력칸/토글/선택 surface는 보개/UI에 있을 수 있지만, **직접 상태 변경은 금지**한다. 세계 의미가 있는 입력은 반드시 \`ui.event -> InputSnapshot -> 이야기/누리\` 경계를 따라야 하며, \`view_only\` 전환은 \`state_ha... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 81754 | > - (PACK/SUPPORT) \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 82078 | > - (AXIS LOCK) 외부개입은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 82079 | > - (REPLAY RULE) \`재연주입\`은 원천 재호출 없이 봉인 기록을 다시 넣는 경로다. replay run 에서 **\`세계영향\` 실주입은 금지**하고, \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 82097 | > - (AXES FIX) 실행축 canonical은 **\`#모두필요 / #빈칸채움\`**, **\`세계영향\`**, **\`한걸음(designed / STUB_ONLY)\`**, **\`살핌내주기갈래 = 설정.거울\`**, **\`놀이기본 = 없음보람 + 해당없음(N/A)\`** 로 정렬한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 82157 | > - (PLATFORM/RULE) 버튼/입력칸/토글/선택 surface는 보개/UI에 있을 수 있지만, **직접 상태 변경은 금지**한다. 세계 의미가 있는 입력은 반드시 \`ui.event -> InputSnapshot -> 이야기/누리\` 경계를 따라야 하며, \`view_only\` 전환은 \`state_ha... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 82159 | > - (PACK/SUPPORT) \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 82453 | - 외부개입 정본 3축은 **\`입력원천 / 주입방식 / 행동갈래\`** 이다. \`입력원천\` 6값은 **\`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`**, \`주입방식\` 2값은 **\`실주입 / 재연주입\`**, \`행동갈래\` 2값은 **\`보기만 / 세계영향\`** 으로 잠근다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 82454 | - replay run 에서는 **\`세계영향\` 실주입을 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 82475 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 한국어 정본으로 정리한다. action kind 정본은 **\`보기만 / 세계영향\`** 이며, source 값의 \`schedule\` 한국어 label은 **\`일정\`** 으로 고정한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 82529 | - full export에는 \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`, \... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 82682 | - \`행동갈래\` = \`보기만 / 세계영향\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 82691 | 7. replay run 에서 **\`세계영향\` 실주입은 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 83268 | > **v23.0.5 용어 상태:** 외부개입 축은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`입력원천\` 6값은 \`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`, \`주입방식\` 2값은 \`실주입 / 재연주입\`, \`행동갈래\` 2값은 \`보기만 / 세계영향\` 이다. ... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 83477 | \| **세계영향** \| 세계 규칙/상태 전이에 영향을 주는 입력 사건 \| world_affecting \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 83610 | - canonical 값은 \`보기만 / 세계영향\` 이며, \`입력원천\` / \`주입방식\` 과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 84141 | \| **세계영향** \| 세계 상태/규칙에 영향을 주는 action kind \| world_affecting \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 91432 | - action kind 정본은 **\`보기만 / 세계영향\`** 이다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 91444 | \| \`세계영향\` \| action kind / input meaning \| AGE3 \| partial \| STUB_ONLY \| canonical world-affecting kind \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 91473 | - replay 경로에서는 **원천 재호출 0건**을 유지하고, \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 91523 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 93115 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 96147 | 4. replay run 에서 **\`세계영향\` 실주입은 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 96266 | - replay run 에서는 \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 96289 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 으로 정리하고, \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 96323 | - \`ui.event -> InputSnapshot\`, \`view_only\` / \`world_affecting\`, 보개 capability boundary는 \`v20.22.0\` 기준선을 그대로 유지한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 96453 | - \`행동갈래\` = 세계 의미 여부. 값은 \`보기만 / 세계영향\` 이다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 96455 | - \`보기만\` 과 \`세계영향\` 은 source/origin 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 96464 | - \`view_only / world_affecting\` 회귀팩 closure | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 96479 | - action kind는 \`view_only / world_affecting\` 2종만 둔다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 98433 | - \`행동갈래\` = \`보기만 / 세계영향\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 98447 | - replay run 에서 **\`세계영향\` 실주입은 금지**한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 98521 | - **world_affecting** | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 98529 | 2. \`world_affecting\` 액션은 반드시 replay/감사 경계 안으로 들어가는 입력 사건이어야 한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 98531 | 4. \`view_only\` 전환과 \`world_affecting\` 입력을 같은 버튼/화면 안에 공존시킬 수는 있지만, action kind 를 분리 기록해야 한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 98537 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 98586 | - replay run 에서 \`세계영향\` 실주입 금지 + \`recv_seq\` 정렬 규칙 명시 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 98608 | - \`view_only / world_affecting\` 경계와 overlay static-one-layer guard 등록 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 100716 | \| AGE3 \| REFERENCE \| \`ui.event -> InputSnapshot\` 경계와 \`view_only / world_affecting\` 구분을 platform/toolchain 규약으로 고정 \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 100718 | \| AGE3 \| REFERENCE \| \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 103967 | \| AGE3 \| ui.event boundary \| OPEN \| \`ui.event -> InputSnapshot -> 이야기/누리\`, \`view_only / world_affecting\` 분리 \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 103975 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 104133 | - \`view_only / world_affecting\` 2종 경계 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 105815 | - replay 중 \`세계영향\` 실주입 금지와 \`보기만\` toolchain 예외를 runner/golden 으로 검증 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 105963 | 2. \`action.kind = view_only \| world_affecting\` 기준 확정 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 105986 | - \`world_affecting\`은 입력 사건 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 105990 | 3. \`view_only_switch_no_statehash_change_v1\` / \`world_affecting_view_switch_v1\` pack closure | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 106076 | - replay run 에서 \`세계영향\` 실주입 금지, \`보기만\` toolchain 예외, \`recv_seq\` 기반 동일 마디 정렬 규칙 추가 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 106095 | - AGE4 = Refine/Settle, overlay AGE4 최소범위, UI \`view_only/world_affecting\` 2종, numeric partial-closure 표현 정리 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 107509 | - \`PROPOSAL_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328_v2\`를 반영해 \`view meaning vs composition\`, \`family/backend/profile/madang\`, \`2d -> space2d\`, \`3d -> space3d\`... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 107511 | - 신규 skeleton pack 6종(\`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 114926 | - action kind 정본은 **\`보기만 / 세계영향\`** 이다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 114938 | \| \`세계영향\` \| action kind / input meaning \| AGE3 \| partial \| STUB_ONLY \| canonical world-affecting kind \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 114962 | - 외부개입 정본 3축은 **\`입력원천 / 주입방식 / 행동갈래\`** 이다. \`입력원천\` 6값은 **\`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`**, \`주입방식\` 2값은 **\`실주입 / 재연주입\`**, \`행동갈래\` 2값은 **\`보기만 / 세계영향\`** 으로 잠근다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 114963 | - replay run 에서는 **\`세계영향\` 실주입을 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 114984 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 한국어 정본으로 정리한다. action kind 정본은 **\`보기만 / 세계영향\`** 이며, source 값의 \`schedule\` 한국어 label은 **\`일정\`** 으로 고정한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 115038 | - full export에는 \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`, \... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 115191 | - \`행동갈래\` = \`보기만 / 세계영향\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 115200 | 7. replay run 에서 **\`세계영향\` 실주입은 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 115852 | - \`view_only / world_affecting\` 2종 경계 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 117534 | - replay 중 \`세계영향\` 실주입 금지와 \`보기만\` toolchain 예외를 runner/golden 으로 검증 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 117682 | 2. \`action.kind = view_only \| world_affecting\` 기준 확정 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 117705 | - \`world_affecting\`은 입력 사건 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 117709 | 3. \`view_only_switch_no_statehash_change_v1\` / \`world_affecting_view_switch_v1\` pack closure | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 117790 | - replay run 에서 \`세계영향\` 실주입 금지, \`보기만\` toolchain 예외, \`recv_seq\` 기반 동일 마디 정렬 규칙 추가 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 117809 | - AGE4 = Refine/Settle, overlay AGE4 최소범위, UI \`view_only/world_affecting\` 2종, numeric partial-closure 표현 정리 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 119223 | - \`PROPOSAL_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328_v2\`를 반영해 \`view meaning vs composition\`, \`family/backend/profile/madang\`, \`2d -> space2d\`, \`3d -> space3d\`... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 119225 | - 신규 skeleton pack 6종(\`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 119280 | - replay run 에서 \`세계영향\` 실주입 금지 + \`recv_seq\` 정렬 규칙 명시 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 119302 | - \`view_only / world_affecting\` 경계와 overlay static-one-layer guard 등록 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 121410 | \| AGE3 \| REFERENCE \| \`ui.event -> InputSnapshot\` 경계와 \`view_only / world_affecting\` 구분을 platform/toolchain 규약으로 고정 \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 121412 | \| AGE3 \| REFERENCE \| \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 121534 | - replay run 에서는 \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 121557 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 으로 정리하고, \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 121591 | - \`ui.event -> InputSnapshot\`, \`view_only\` / \`world_affecting\`, 보개 capability boundary는 \`v20.22.0\` 기준선을 그대로 유지한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 121721 | - \`행동갈래\` = 세계 의미 여부. 값은 \`보기만 / 세계영향\` 이다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 121723 | - \`보기만\` 과 \`세계영향\` 은 source/origin 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 121732 | - \`view_only / world_affecting\` 회귀팩 closure | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 121747 | - action kind는 \`view_only / world_affecting\` 2종만 둔다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 123701 | - \`행동갈래\` = \`보기만 / 세계영향\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 123715 | - replay run 에서 **\`세계영향\` 실주입은 금지**한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 123789 | - **world_affecting** | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 123797 | 2. \`world_affecting\` 액션은 반드시 replay/감사 경계 안으로 들어가는 입력 사건이어야 한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 123799 | 4. \`view_only\` 전환과 \`world_affecting\` 입력을 같은 버튼/화면 안에 공존시킬 수는 있지만, action kind 를 분리 기록해야 한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 123805 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 126981 | \| AGE3 \| ui.event boundary \| OPEN \| \`ui.event -> InputSnapshot -> 이야기/누리\`, \`view_only / world_affecting\` 분리 \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 126989 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 127063 | > **v23.0.5 용어 상태:** 외부개입 축은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`입력원천\` 6값은 \`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`, \`주입방식\` 2값은 \`실주입 / 재연주입\`, \`행동갈래\` 2값은 \`보기만 / 세계영향\` 이다. ... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 127272 | \| **세계영향** \| 세계 규칙/상태 전이에 영향을 주는 입력 사건 \| world_affecting \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 127405 | - canonical 값은 \`보기만 / 세계영향\` 이며, \`입력원천\` / \`주입방식\` 과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 127936 | \| **세계영향** \| 세계 상태/규칙에 영향을 주는 action kind \| world_affecting \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 128000 | - replay 경로에서는 **원천 재호출 0건**을 유지하고, \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 128050 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 129642 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 132674 | 4. replay run 에서 **\`세계영향\` 실주입은 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143090 | concept_term,world_affecting,ko,세계영향,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143091 | concept_term,world_affecting,ko_common,세계영향,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143092 | concept_term,world_affecting,sym3,,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143093 | concept_term,world_affecting,en,world_affecting,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143094 | concept_term,world_affecting,ja,世界影響,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143095 | concept_term,world_affecting,mn,ертөнцөд_нөлөөлөх,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143096 | concept_term,world_affecting,tr,dünya_etkili,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143097 | concept_term,world_affecting,qu,⟦TBD_NATIVE_REVIEW⟧,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143098 | concept_term,world_affecting,ay,⟦TBD_NATIVE_REVIEW⟧,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143099 | concept_term,world_affecting,ne,विश्व_प्रभाव,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143100 | concept_term,world_affecting,ta,உலக_பாதிப்பு,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143101 | concept_term,world_affecting,te,ప్రపంచ_ప్రభావ,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143102 | concept_term,world_affecting,kn,ಲೋಕ_ಪ್ರಭಾವ,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143103 | concept_term,world_affecting,eu,mundu_eragin,T0/T1 proposed; T2 draft | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_context_bundle_ALL.md` | 143174 | world_affecting,T0/T1 proposed; T2 draft,세계영향,세계영향,,world_affecting,世界影響,ертөнцөд_нөлөөлөх,dünya_etkili,⟦TBD_NATIVE_REVIEW⟧,⟦TBD_NATIVE_REVIEW⟧,विश्व_प्रभाव,உலக_பாதிப்பு,ప్రపంచ_... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_pack_ALL.md` | 944 | 4. **view_only / world_affecting 분리** | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_pack_ALL.md` | 948 | - 행동/제출 버튼은 \`world_affecting\` 입력 사건으로 처리함을 문서화 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_pack_ALL.md` | 951 | - \`world_affecting_submit\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_pack_ALL.md` | 174469 | ===== FILE: docs\ssot\pack\world_affecting_view_switch_v1\expected\PLACEHOLDER.md ===== | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_pack_ALL.md` | 174470 | # world_affecting_view_switch_v1 expected placeholder | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_pack_ALL.md` | 174475 | - action.kind = world_affecting | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_pack_ALL.md` | 174479 | ===== FILE: docs\ssot\pack\world_affecting_view_switch_v1\meta.toml ===== | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_pack_ALL.md` | 174480 | id = "world_affecting_view_switch_v1" | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_pack_ALL.md` | 174483 | ===== FILE: docs\ssot\pack\world_affecting_view_switch_v1\README.md ===== | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_pack_ALL.md` | 174484 | # world_affecting_view_switch_v1 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_pack_ALL.md` | 174491 | 1. action.kind = world_affecting | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_pack_ALL.md` | 174495 | ===== FILE: docs\ssot\pack\world_affecting_view_switch_v1\tests\README.md ===== | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_pack_ALL.md` | 174496 | # tests for world_affecting_view_switch_v1 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 58 | > - (AXIS LOCK) 외부개입은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 59 | > - (REPLAY RULE) \`재연주입\`은 원천 재호출 없이 봉인 기록을 다시 넣는 경로다. replay run 에서 **\`세계영향\` 실주입은 금지**하고, \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 77 | > - (AXES FIX) 실행축 canonical은 **\`#모두필요 / #빈칸채움\`**, **\`세계영향\`**, **\`한걸음(designed / STUB_ONLY)\`**, **\`살핌내주기갈래 = 설정.거울\`**, **\`놀이기본 = 없음보람 + 해당없음(N/A)\`** 로 정렬한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 137 | > - (PLATFORM/RULE) 버튼/입력칸/토글/선택 surface는 보개/UI에 있을 수 있지만, **직접 상태 변경은 금지**한다. 세계 의미가 있는 입력은 반드시 \`ui.event -> InputSnapshot -> 이야기/누리\` 경계를 따라야 하며, \`view_only\` 전환은 \`state_ha... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 139 | > - (PACK/SUPPORT) \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 463 | > - (AXIS LOCK) 외부개입은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 464 | > - (REPLAY RULE) \`재연주입\`은 원천 재호출 없이 봉인 기록을 다시 넣는 경로다. replay run 에서 **\`세계영향\` 실주입은 금지**하고, \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 482 | > - (AXES FIX) 실행축 canonical은 **\`#모두필요 / #빈칸채움\`**, **\`세계영향\`**, **\`한걸음(designed / STUB_ONLY)\`**, **\`살핌내주기갈래 = 설정.거울\`**, **\`놀이기본 = 없음보람 + 해당없음(N/A)\`** 로 정렬한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 542 | > - (PLATFORM/RULE) 버튼/입력칸/토글/선택 surface는 보개/UI에 있을 수 있지만, **직접 상태 변경은 금지**한다. 세계 의미가 있는 입력은 반드시 \`ui.event -> InputSnapshot -> 이야기/누리\` 경계를 따라야 하며, \`view_only\` 전환은 \`state_ha... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 544 | > - (PACK/SUPPORT) \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 838 | - 외부개입 정본 3축은 **\`입력원천 / 주입방식 / 행동갈래\`** 이다. \`입력원천\` 6값은 **\`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`**, \`주입방식\` 2값은 **\`실주입 / 재연주입\`**, \`행동갈래\` 2값은 **\`보기만 / 세계영향\`** 으로 잠근다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 839 | - replay run 에서는 **\`세계영향\` 실주입을 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 860 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 한국어 정본으로 정리한다. action kind 정본은 **\`보기만 / 세계영향\`** 이며, source 값의 \`schedule\` 한국어 label은 **\`일정\`** 으로 고정한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 914 | - full export에는 \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`, \... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 1067 | - \`행동갈래\` = \`보기만 / 세계영향\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 1076 | 7. replay run 에서 **\`세계영향\` 실주입은 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 1653 | > **v23.0.5 용어 상태:** 외부개입 축은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`입력원천\` 6값은 \`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`, \`주입방식\` 2값은 \`실주입 / 재연주입\`, \`행동갈래\` 2값은 \`보기만 / 세계영향\` 이다. ... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 1862 | \| **세계영향** \| 세계 규칙/상태 전이에 영향을 주는 입력 사건 \| world_affecting \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 1995 | - canonical 값은 \`보기만 / 세계영향\` 이며, \`입력원천\` / \`주입방식\` 과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 2526 | \| **세계영향** \| 세계 상태/규칙에 영향을 주는 action kind \| world_affecting \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 9817 | - action kind 정본은 **\`보기만 / 세계영향\`** 이다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 9829 | \| \`세계영향\` \| action kind / input meaning \| AGE3 \| partial \| STUB_ONLY \| canonical world-affecting kind \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 9858 | - replay 경로에서는 **원천 재호출 0건**을 유지하고, \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 9908 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 11500 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 14532 | 4. replay run 에서 **\`세계영향\` 실주입은 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 14651 | - replay run 에서는 \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 14674 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 으로 정리하고, \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 14708 | - \`ui.event -> InputSnapshot\`, \`view_only\` / \`world_affecting\`, 보개 capability boundary는 \`v20.22.0\` 기준선을 그대로 유지한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 14838 | - \`행동갈래\` = 세계 의미 여부. 값은 \`보기만 / 세계영향\` 이다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 14840 | - \`보기만\` 과 \`세계영향\` 은 source/origin 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 14849 | - \`view_only / world_affecting\` 회귀팩 closure | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 14864 | - action kind는 \`view_only / world_affecting\` 2종만 둔다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 16818 | - \`행동갈래\` = \`보기만 / 세계영향\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 16832 | - replay run 에서 **\`세계영향\` 실주입은 금지**한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 16906 | - **world_affecting** | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 16914 | 2. \`world_affecting\` 액션은 반드시 replay/감사 경계 안으로 들어가는 입력 사건이어야 한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 16916 | 4. \`view_only\` 전환과 \`world_affecting\` 입력을 같은 버튼/화면 안에 공존시킬 수는 있지만, action kind 를 분리 기록해야 한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 16922 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 16971 | - replay run 에서 \`세계영향\` 실주입 금지 + \`recv_seq\` 정렬 규칙 명시 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 16993 | - \`view_only / world_affecting\` 경계와 overlay static-one-layer guard 등록 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 19101 | \| AGE3 \| REFERENCE \| \`ui.event -> InputSnapshot\` 경계와 \`view_only / world_affecting\` 구분을 platform/toolchain 규약으로 고정 \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 19103 | \| AGE3 \| REFERENCE \| \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 22352 | \| AGE3 \| ui.event boundary \| OPEN \| \`ui.event -> InputSnapshot -> 이야기/누리\`, \`view_only / world_affecting\` 분리 \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 22360 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 22518 | - \`view_only / world_affecting\` 2종 경계 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 24200 | - replay 중 \`세계영향\` 실주입 금지와 \`보기만\` toolchain 예외를 runner/golden 으로 검증 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 24348 | 2. \`action.kind = view_only \| world_affecting\` 기준 확정 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 24371 | - \`world_affecting\`은 입력 사건 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 24375 | 3. \`view_only_switch_no_statehash_change_v1\` / \`world_affecting_view_switch_v1\` pack closure | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 24461 | - replay run 에서 \`세계영향\` 실주입 금지, \`보기만\` toolchain 예외, \`recv_seq\` 기반 동일 마디 정렬 규칙 추가 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 24480 | - AGE4 = Refine/Settle, overlay AGE4 최소범위, UI \`view_only/world_affecting\` 2종, numeric partial-closure 표현 정리 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 25894 | - \`PROPOSAL_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328_v2\`를 반영해 \`view meaning vs composition\`, \`family/backend/profile/madang\`, \`2d -> space2d\`, \`3d -> space3d\`... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 25896 | - 신규 skeleton pack 6종(\`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 33311 | - action kind 정본은 **\`보기만 / 세계영향\`** 이다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 33323 | \| \`세계영향\` \| action kind / input meaning \| AGE3 \| partial \| STUB_ONLY \| canonical world-affecting kind \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 33347 | - 외부개입 정본 3축은 **\`입력원천 / 주입방식 / 행동갈래\`** 이다. \`입력원천\` 6값은 **\`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`**, \`주입방식\` 2값은 **\`실주입 / 재연주입\`**, \`행동갈래\` 2값은 **\`보기만 / 세계영향\`** 으로 잠근다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 33348 | - replay run 에서는 **\`세계영향\` 실주입을 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 33369 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 한국어 정본으로 정리한다. action kind 정본은 **\`보기만 / 세계영향\`** 이며, source 값의 \`schedule\` 한국어 label은 **\`일정\`** 으로 고정한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 33423 | - full export에는 \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`, \... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 33576 | - \`행동갈래\` = \`보기만 / 세계영향\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 33585 | 7. replay run 에서 **\`세계영향\` 실주입은 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 34237 | - \`view_only / world_affecting\` 2종 경계 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 35919 | - replay 중 \`세계영향\` 실주입 금지와 \`보기만\` toolchain 예외를 runner/golden 으로 검증 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 36067 | 2. \`action.kind = view_only \| world_affecting\` 기준 확정 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 36090 | - \`world_affecting\`은 입력 사건 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 36094 | 3. \`view_only_switch_no_statehash_change_v1\` / \`world_affecting_view_switch_v1\` pack closure | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 36175 | - replay run 에서 \`세계영향\` 실주입 금지, \`보기만\` toolchain 예외, \`recv_seq\` 기반 동일 마디 정렬 규칙 추가 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 36194 | - AGE4 = Refine/Settle, overlay AGE4 최소범위, UI \`view_only/world_affecting\` 2종, numeric partial-closure 표현 정리 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 37608 | - \`PROPOSAL_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328_v2\`를 반영해 \`view meaning vs composition\`, \`family/backend/profile/madang\`, \`2d -> space2d\`, \`3d -> space3d\`... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 37610 | - 신규 skeleton pack 6종(\`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 37665 | - replay run 에서 \`세계영향\` 실주입 금지 + \`recv_seq\` 정렬 규칙 명시 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 37687 | - \`view_only / world_affecting\` 경계와 overlay static-one-layer guard 등록 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 39795 | \| AGE3 \| REFERENCE \| \`ui.event -> InputSnapshot\` 경계와 \`view_only / world_affecting\` 구분을 platform/toolchain 규약으로 고정 \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 39797 | \| AGE3 \| REFERENCE \| \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 39919 | - replay run 에서는 \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 39942 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 으로 정리하고, \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 39976 | - \`ui.event -> InputSnapshot\`, \`view_only\` / \`world_affecting\`, 보개 capability boundary는 \`v20.22.0\` 기준선을 그대로 유지한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 40106 | - \`행동갈래\` = 세계 의미 여부. 값은 \`보기만 / 세계영향\` 이다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 40108 | - \`보기만\` 과 \`세계영향\` 은 source/origin 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 40117 | - \`view_only / world_affecting\` 회귀팩 closure | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 40132 | - action kind는 \`view_only / world_affecting\` 2종만 둔다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 42086 | - \`행동갈래\` = \`보기만 / 세계영향\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 42100 | - replay run 에서 **\`세계영향\` 실주입은 금지**한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 42174 | - **world_affecting** | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 42182 | 2. \`world_affecting\` 액션은 반드시 replay/감사 경계 안으로 들어가는 입력 사건이어야 한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 42184 | 4. \`view_only\` 전환과 \`world_affecting\` 입력을 같은 버튼/화면 안에 공존시킬 수는 있지만, action kind 를 분리 기록해야 한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 42190 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 45366 | \| AGE3 \| ui.event boundary \| OPEN \| \`ui.event -> InputSnapshot -> 이야기/누리\`, \`view_only / world_affecting\` 분리 \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 45374 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 45448 | > **v23.0.5 용어 상태:** 외부개입 축은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`입력원천\` 6값은 \`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`, \`주입방식\` 2값은 \`실주입 / 재연주입\`, \`행동갈래\` 2값은 \`보기만 / 세계영향\` 이다. ... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 45657 | \| **세계영향** \| 세계 규칙/상태 전이에 영향을 주는 입력 사건 \| world_affecting \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 45790 | - canonical 값은 \`보기만 / 세계영향\` 이며, \`입력원천\` / \`주입방식\` 과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 46321 | \| **세계영향** \| 세계 상태/규칙에 영향을 주는 action kind \| world_affecting \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 46385 | - replay 경로에서는 **원천 재호출 0건**을 유지하고, \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 46435 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 48027 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_releases_ALL.md` | 51059 | 4. replay run 에서 **\`세계영향\` 실주입은 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_solutions_ALL.md` | 565 | - \`action.kind = view_only \| world_affecting\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 412 | > - (SAM/BOUNDARY) 샘(Sam)은 더 이상 입력 장치 묶음이 아니라 **외부개입을 \`InputSnapshot\`으로 봉인·재연하는 실행 경계**다. \`ui.event -> InputSnapshot -> 이야기/누리\` 경계를 exact boundary로 적고, \`view_only / world_a... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 540 | > - (AXIS LOCK) 외부개입은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 541 | > - (REPLAY RULE) \`재연주입\`은 원천 재호출 없이 봉인 기록을 다시 넣는 경로다. replay run 에서 **\`세계영향\` 실주입은 금지**하고, \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 559 | > - (AXES FIX) 실행축 canonical은 **\`#모두필요 / #빈칸채움\`**, **\`세계영향\`**, **\`한걸음(designed / STUB_ONLY)\`**, **\`살핌내주기갈래 = 설정.거울\`**, **\`놀이기본 = 없음보람 + 해당없음(N/A)\`** 로 정렬한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 619 | > - (PLATFORM/RULE) 버튼/입력칸/토글/선택 surface는 보개/UI에 있을 수 있지만, **직접 상태 변경은 금지**한다. 세계 의미가 있는 입력은 반드시 \`ui.event -> InputSnapshot -> 이야기/누리\` 경계를 따라야 하며, \`view_only\` 전환은 \`state_ha... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 621 | > - (PACK/SUPPORT) \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 9624 | "행동갈래": "세계영향", | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 9848 | - action kind 정본은 **\`보기만 / 세계영향\`** 이다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 9867 | \| \`세계영향\` \| action kind / input meaning \| AGE3 \| partial \| STUB_ONLY \| canonical world-affecting kind \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 10533 | - 외부개입 정본 3축은 **\`입력원천 / 주입방식 / 행동갈래\`** 이다. \`입력원천\` 6값은 **\`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`**, \`주입방식\` 2값은 **\`실주입 / 재연주입\`**, \`행동갈래\` 2값은 **\`보기만 / 세계영향\`** 으로 잠근다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 10534 | - replay run 에서는 **\`세계영향\` 실주입을 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 10555 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 한국어 정본으로 정리한다. action kind 정본은 **\`보기만 / 세계영향\`** 이며, source 값의 \`schedule\` 한국어 label은 **\`일정\`** 으로 고정한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 10609 | - full export에는 \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`, \... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 10769 | - \`행동갈래\` = \`보기만 / 세계영향\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 10778 | 7. replay run 에서 **\`세계영향\` 실주입은 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 11844 | - world_affecting / view_only / analysis_only / DEFERRED 경계를 한 장부로 정리한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 12818 | - \`view_only / world_affecting\` 2종 경계 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 14504 | - replay 중 \`세계영향\` 실주입 금지와 \`보기만\` toolchain 예외를 runner/golden 으로 검증 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 14710 | 2. \`action.kind = view_only \| world_affecting\` 기준 확정 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 14733 | - \`world_affecting\`은 입력 사건 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 14737 | 3. \`view_only_switch_no_statehash_change_v1\` / \`world_affecting_view_switch_v1\` pack closure | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 15936 | ### PLN-20260407-EXEC-CLOSURE-P0B-V1: \`ui.event -> InputSnapshot\` + \`view_only/world_affecting\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 15941 | - \`action.kind = view_only \| world_affecting\` 경계를 exact lock 하고 replay 재주입 형식을 닫는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 15944 | - \`ui_event_to_inputsnapshot_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`world_affecting_view_switch_v1\` pack closure | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 15946 | - \`world_affecting\` 입력의 snapshot 기록 + replay 재주입 사례 1종 이상 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 16182 | - replay run 에서 \`세계영향\` 실주입 금지 + \`recv_seq\` 정렬 규칙 명시 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 16204 | - \`view_only / world_affecting\` 경계와 overlay static-one-layer guard 등록 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 18227 | - replay 중 \`세계영향\` 실주입 금지, \`보기만\` 실주입은 toolchain 예외로만 허용 | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 18332 | \| AGE3 \| REFERENCE \| \`ui.event -> InputSnapshot\` 경계와 \`view_only / world_affecting\` 구분을 platform/toolchain 규약으로 고정 \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 18334 | \| AGE3 \| REFERENCE \| \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 18925 | - execution ordering \`(agent_id, recv_seq)\`, replay/source recall 금지, \`ui.event -> InputSnapshot -> 이야기/누리\`, \`view_only / world_affecting\`, raw prompt/response/debug trace... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 18933 | - \`view_only\` 와 \`world_affecting\` 분리는 그대로 유지하고, 3D consumer 경로에서 생기는 \`camera / asset / animation / HUD / layout\` 변화도 **adapter payload / view_meta** 쪽에 남긴다. \`engine3d.god... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 18953 | - \`view_only\` 와 \`world_affecting\` 은 계속 분리하고, \`view_only\` 전환은 \`state_hash\` 밖, \`world_affecting\` 입력은 replay/audit 경계 안으로 둔다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 19062 | - \`ui.event\`는 UI/도구 표면이고, world 의미를 바꾸는 입력은 여전히 **\`InputSnapshot\` 봉인 경계** 를 통과해야 한다. \`view_only\` 조작은 \`state_hash\` 밖, \`world_affecting\`은 snapshot/replay 경로 안에 있다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 19166 | - replay run 에서는 \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 19189 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 으로 정리하고, \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 19223 | - \`ui.event -> InputSnapshot\`, \`view_only\` / \`world_affecting\`, 보개 capability boundary는 \`v20.22.0\` 기준선을 그대로 유지한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 19392 | - \`행동갈래\` = 세계 의미 여부. 값은 \`보기만 / 세계영향\` 이다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 19394 | - \`보기만\` 과 \`세계영향\` 은 source/origin 축과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 19403 | - \`view_only / world_affecting\` 회귀팩 closure | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 19418 | - action kind는 \`view_only / world_affecting\` 2종만 둔다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 21400 | - \`행동갈래\` = \`보기만 / 세계영향\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 21415 | - replay run 에서 **\`세계영향\` 실주입은 금지**한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 21493 | - **world_affecting** | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 21501 | 2. \`world_affecting\` 액션은 반드시 replay/감사 경계 안으로 들어가는 입력 사건이어야 한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 21503 | 4. \`view_only\` 전환과 \`world_affecting\` 입력을 같은 버튼/화면 안에 공존시킬 수는 있지만, action kind 를 분리 기록해야 한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 21509 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 21878 | - P0-B: \`ui_event_to_inputsnapshot_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 25174 | \| AGE3 \| ui.event boundary \| OPEN \| \`ui.event -> InputSnapshot -> 이야기/누리\`, \`view_only / world_affecting\` 분리 \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 25182 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 25531 | \| \`world_affecting\` \| 세계영향 \| 이야기/누리 의미를 바꾸는 입력 사건. \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 25576 | \| \`world_affecting\` \| 세계영향 입력 \| world state 의미를 바꾸는 입력. 반드시 \`InputSnapshot\` 경유로 들어간다. \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 25629 | > **v23.0.5 용어 상태:** 외부개입 축은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`입력원천\` 6값은 \`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`, \`주입방식\` 2값은 \`실주입 / 재연주입\`, \`행동갈래\` 2값은 \`보기만 / 세계영향\` 이다. ... | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 25847 | \| **세계영향** \| 세계 규칙/상태 전이에 영향을 주는 입력 사건 \| world_affecting \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 25989 | - canonical 값은 \`보기만 / 세계영향\` 이며, \`입력원천\` / \`주입방식\` 과 섞지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 26513 | \| **세계영향** \| 세계 상태/규칙에 영향을 주는 action kind \| world_affecting \| | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 26970 | - \`ui.event\` 최소 필드와 \`action.kind = view_only \| world_affecting\` exact lock, replay 재주입 형식은 \`InputSnapshot\` 봉인 규칙과 함께 다룬다. toolchain 은 direct state mutation UI 를 허용하지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 27040 | - replay 경로에서는 **원천 재호출 0건**을 유지하고, \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 27185 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/all/ssot_ssot_ALL.md` | 28790 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/briefs/QUEUE_CODEX_20260706.md` | 126 | 3. D36 누리바꿈(세계영향 → 누리바꿈): \`세계영향\`/\`world_affecting\` 문자열을 참조하는 모든 코드/팩/문서 위치 전수. | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/CODEX_HANDOFF_STORY_TECH_BUNDLE_INDEX_V1_20260412.md` | 19 | - \`state_hash\` / replay / InputSnapshot / view_only vs world_affecting 경계는 흐리지 않는다. | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/CODEX_PROMPT_V23_1_EXECUTION_CLOSURE_TRIAD_V1_20260407.md` | 11 | 2. \`ui.event -> InputSnapshot\` exact schema + \`view_only/world_affecting\` 경계 | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/CODEX_PROMPT_V23_1_EXECUTION_CLOSURE_TRIAD_V1_20260407.md` | 33 | - \`action.kind = view_only \| world_affecting\` exact schema | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/CODEX_PROMPT_V23_1_EXECUTION_CLOSURE_TRIAD_V1_20260407.md` | 38 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/CODEX_PROMPT_V23_1_EXECUTION_CLOSURE_WITH_V23_0_14_AND_DEV_SUMMARY_V2_20260407.md` | 15 | 3. \`ui.event -> InputSnapshot\` exact schema + \`view_only/world_affecting\` 경계 closure | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/CODEX_PROMPT_V23_1_EXECUTION_CLOSURE_WITH_V23_0_14_AND_DEV_SUMMARY_V2_20260407.md` | 55 | - \`world_affecting_view_switch_v1\` | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/CODEX_PROMPT_V23_1_EXECUTION_CLOSURE_WITH_V23_0_14_AND_DEV_SUMMARY_V2_20260407.md` | 59 | - \`world_affecting\` 입력이 snapshot/replay 경로로만 들어감 | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/CODEX_TICKET_BOGAE_VIEW_BOUNDARY_V1_20260328.md` | 34 | - \`docs/ssot/pack/world_affecting_view_switch_v1/\` | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/CODEX_TICKET_BOGAE_VIEW_BOUNDARY_V1_20260328.md` | 43 | 6. \`world_affecting\` action은 반드시 \`ui.event -> InputSnapshot -> 이야기/누리\` 경계를 따른다. | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328.md` | 37 | * \`view_only\` 전환과 \`world_affecting\` 전환을 구분하는 상위 문장 추가 | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328.md` | 101 | * \`view_only\` 액션과 \`world_affecting\` 액션 구분 문장 추가 | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328.md` | 105 | * 턴 소비/전투 진입/행동 선택 = world_affecting | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328.md` | 106 | * \`view_only\` 전환은 \`state_hash\` 밖, \`world_affecting\` 입력은 replay/감사 경계 안으로 들어간다는 문장 추가 | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328.md` | 110 | * \`world_affecting\` | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328.md` | 172 | * \`action.kind = view_only \| world_affecting\` 예시 추가 | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328.md` | 173 | * \`event.kind = view_only \| world_affecting\` 예시 추가 | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328.md` | 212 | * \`world_affecting_submit\` | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328.md` | 249 | ### 9-5. \`docs/ssot/pack/world_affecting_view_switch_v1/\` | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328_v2.md` | 66 | - \`view_only\` 액션과 \`world_affecting\` 액션 구분 문장 추가 | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328_v2.md` | 69 | - 턴 소비/전투 진입/행동 선택 = world_affecting | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328_v2.md` | 70 | - \`view_only\` 전환은 \`state_hash\` 밖, \`world_affecting\` 입력은 replay/감사 경계 안으로 들어간다는 문장 추가 | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328_v2.md` | 117 | - \`action.kind = view_only \| world_affecting\` 예시 추가 | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328_v2.md` | 150 | - \`world_affecting_submit\` | 문서/검증 문자열 영향 후보. |
| `docs/context/codex_tasks/PATCH_BULLETS_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328_v2.md` | 178 | ### 9-5. \`docs/ssot/pack/world_affecting_view_switch_v1/\` | 문서/검증 문자열 영향 후보. |
| `docs/context/proposals/DDONIRANG_DISCUSSION_BUNDLE_ALL_20260406.md` | 481 | - \`view_only\` vs \`world_affecting\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/DDONIRANG_DISCUSSION_BUNDLE_ALL_20260406.md` | 1911 | - \`세계영향\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/DDONIRANG_DISCUSSION_BUNDLE_ALL_20260406.md` | 2014 | - \`view_only\` vs \`world_affecting\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/DECISION_LANGUAGE_FREEZE_BOUNDARY_V1_20260405.md` | 27 | - \`view_only\` vs \`world_affecting\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/IMPACT_MULTI_AGENT_ACTION_FILTER_BEFORE_MERGE_V1_20260404.md` | 187 | \`world_affecting\` action은 결국 InputSnapshot 봉인 안으로 들어오므로, | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/MEMO_ACTIVATE_R0A_R0E_AND_FIRST_3_PACKS_V1_20260405.md` | 22 | \| G3 \| \`설정.거울 / 설정.실행\` 하위 표면 AGE gate 명시 (\`뒤살핌 / 없음보람 / 늘보람 / 한걸음 / 세계영향\`) \| designed+STUB_ONLY → parser 경계 반영 \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/NURIGYM_LANGUAGE_PROFILE_V1_20260406.md` | 62 | - \`세계영향\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328.md` | 344 | * \`world_affecting\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328.md` | 350 | * \`world_affecting\`은 \`InputSnapshot\`에 봉인해 replay/감사 경계 안으로 보낸다 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328.md` | 445 | * \`world_affecting_action\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328.md` | 456 | 5. \`docs/ssot/pack/world_affecting_view_switch_v1/\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328.md` | 476 | * \`ui.action.kind = view_only \| world_affecting\` 최소 분류 반영 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_DDONIRANG_DECLARATIVE_RELATION_ALRIM_IEUM_ALIGNMENT_20260606.md` | 658 | world_affecting event는 InputSnapshot 없이는 적용 불가하다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_DDONIRANG_DETERMINISTIC_SIM_TESTING_TRACK_V1_20260524.md` | 16 | - world_affecting event는 InputSnapshot 없이 적용 불가. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_DDONIRANG_MASTER_EXECUTION_V2_1_SSOT_CONFORMANT_20260410.md` | 169 | - \`world_affecting\` = 반드시 \`InputSnapshot\` 경유 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_DDONIRANG_MASTER_EXECUTION_V2_1_SSOT_CONFORMANT_20260410.md` | 186 | \| \`docs/ssot/solutions/05_schemas_catalog_v0/schema/ui.v0.md\` \| placeholder 제거 \| \`ui.event\`, \`view_only\`, \`world_affecting\` 경계 반영 \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_DDONIRANG_MASTER_EXECUTION_V2_1_SSOT_CONFORMANT_20260410.md` | 230 | - \`ui_event_to_inputsnapshot_v1\` 에 \`view_only_switch\` / \`world_affecting_submit\` representative case 가 들어감 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_DDONIRANG_MASTER_EXECUTION_V2_1_SSOT_CONFORMANT_20260410.md` | 260 | 4) ui.event direct mutation 금지, world_affecting은 InputSnapshot 경유 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_DDONIRANG_MASTER_EXECUTION_V2_1_SSOT_CONFORMANT_20260410.md` | 621 | - \`world_affecting_view_switch_v1\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXECUTION_BRANCH_GOGAE_GEOREUM_V2_20260405.md` | 39 | - 대상: \`뒤살핌 / 없음보람 / 늘보람 / 한걸음 / 세계영향\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXECUTION_BRANCH_GOGAE_GEOREUM_V2_20260405.md` | 57 | \| W6 \| InputSnapshot 경계 closure \| 세계영향 입력 봉인 pack \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXECUTION_BRANCH_GOGAE_GEOREUM_V2_20260405.md` | 107 | - \`가만히 / 거부 / 자르기 / 보기만 / 세계영향\` 의미축은 SSOT 본문에 있음 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_BOUNDARY_CONTRACT_V1_20260404.md` | 83 | \| \`세계영향\` \| world_affecting \| 세계 상태 변경, replay/감사 경계 안 \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_BOUNDARY_CONTRACT_V1_20260404.md` | 89 | \| \`사람\` \| \`실주입\` \| \`세계영향\` \| 사람이 지금 직접 게임/시뮬 조작 \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_BOUNDARY_CONTRACT_V1_20260404.md` | 90 | \| \`사람\` \| \`재연주입\` \| \`세계영향\` \| 과거 사람 입력을 replay \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_BOUNDARY_CONTRACT_V1_20260404.md` | 91 | \| \`펼침실행\` \| \`실주입\` \| \`세계영향\` \| RL policy가 지금 step 생성 중 \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_BOUNDARY_CONTRACT_V1_20260404.md` | 92 | \| \`펼침실행\` \| \`재연주입\` \| \`세계영향\` \| 과거 rollout 기록을 replay \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_BOUNDARY_CONTRACT_V1_20260404.md` | 94 | \| \`이어전달\` \| \`재연주입\` \| \`세계영향\` \| relay가 자체 canonical payload를 생성한 뒤 재전달 replay \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_BOUNDARY_CONTRACT_V1_20260404.md` | 150 | ### 기본 정책: replay 중 세계영향 실주입 전면 금지 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_BOUNDARY_CONTRACT_V1_20260404.md` | 152 | > **replay run에서 \`행동갈래: 세계영향\` 실주입은 금지한다.** | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_BOUNDARY_CONTRACT_V1_20260404.md` | 211 | \`보기만\`과 \`세계영향\`을 동시에 가지는 사건은 허용하지 않는다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_BOUNDARY_CONTRACT_V1_20260404.md` | 277 | - \`행동갈래\` 확장 시 2값(\`보기만/세계영향\`) 위에 open-ended reserved로 추가한다. 기존 2값은 변경하지 않는다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_BOUNDARY_CONTRACT_V1_20260404.md` | 300 | - replay 중 \`세계영향\` 실주입 전면 금지가 SSOT_MASTER에 명시된다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_SOURCE_SPLIT_V1_20260404.md` | 18 | - view_only / world_affecting 분리 원칙 변경 (기존 확정 유지) | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_SOURCE_SPLIT_V1_20260404.md` | 54 | ### 1-2. view_only / world_affecting 분리 (확정 유지) | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_SOURCE_SPLIT_V1_20260404.md` | 57 | - \`world_affecting\` 입력은 replay/감사 경계 안 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_SOURCE_SPLIT_V1_20260404.md` | 158 | \`view_only\` / \`world_affecting\` (= \`세계영향\` canonical, D-003)은 source가 아니다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_SOURCE_SPLIT_V1_20260404.md` | 165 | \| \`action_kind\` \| 세계에 영향이 있는가 \| \`보기만 / 세계영향\` \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_SOURCE_SPLIT_V1_20260404.md` | 169 | - rollout이 만든 \`세계영향\` 입력 → source=rollout, action_kind=세계영향, inject_mode=live | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_EXTERNAL_INTERVENTION_SOURCE_SPLIT_V1_20260404.md` | 212 | - \`action_kind\` (\`보기만 / 세계영향\`)는 source와 별도 축임이 SSOT에 명시된다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_GRAMMAR_STATUS_TABLE_V1_20260329.md` | 89 | - \`view_only / world_affecting\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_HARDCUT_HASH_HEADERS_AND_SETTING_ROOT_KRNAME_V1_20260418.md` | 226 | - \`행동갈래: #세계영향\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_HARDCUT_HASH_HEADERS_AND_SETTING_ROOT_KRNAME_V1_20260418.md` | 326 | 행동갈래: #세계영향. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 185 | 5. **효과/권한 축** — \`누리바꿈\`/열림(기존 \`world_affecting\` 축의 한국어 정본 갱신)의 확장으로 H2~H3. 커널 V1은 코어 순수성 헌법으로 충분. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 198 | **D26-용어 보강 (2026-07-05 사용자 확정):** 행동갈래의 한국어 정본은 **\`보기만 / 누리바꿈\`** 으로 둔다. \`누리바꿈\`은 누리 상태와 \`state_hash\`를 바꾸는 입력 사건이며 반드시 \`InputSnapshot\` 경유로 들어간다. 기존 \`world_affecting\`은 s... | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 203 | **기존 자산 매핑 (Claude):** ADT=고름씨+맺음+?+알림+상태머신의 내부 통일 문제(전부 존재, 모델 통합만 필요) / 이벤트소싱="상태+로그 하이브리드"는 Nuri(상태)+Geoul(기록) 그 자체 — 이미 아키텍처 / 효과·권한=D26 기결정(\`누리바꿈\`/열림 축, 기존 \`world_affect... | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 218 | \| D31 \| 효과 표기 위치 \| V1 manifest(기존 \`world_affecting\`/\`누리바꿈\` 축)만, 씨앗 서명 표기는 H2 \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 233 | - **D31 ✅** V1은 manifest 세계영향 축, 씨앗 서명 효과 표기는 H2. 효과 축: 상태바꿈/시간읽기/난수쓰기/파일읽기/파일쓰기/외부입력/외부출력/외부망/AI호출/기록쓰기 — ⚠️ D26과 통일 필요 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_DIRECTION_20260704.md` | 245 | - **D36 (신규) ⚠️ SSOT 절차 필요**: \`세계영향\` → **\`누리바꿈\`** 정본 교체 (사용자·Codex 합의). 단 기존 SSOT **D-003이 "세계영향 = world_affecting 정본"으로 확정**한 사항이므로 supersede 기록이 필요 — Claude가 SSOT 개정 제안서 작... | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_SPEC_20260704.md` | 313 | - **누리바꿈**: 누리 상태와 \`state_hash\`를 바꾸는 입력 사건. 반드시 \`InputSnapshot\` 경유. [SSOT 개정 대기 — \`세계영향\`(D-003 기존 정본) → \`누리바꿈\` 교체, \`PROPOSAL_SSOT_DECISIONS_NURI_BAKKUM_*\` 별도 작성 필요] | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LANG_KERNEL_V1_SPEC_20260704.md` | 402 | \| (미작성) 누리바꿈 supersede \| D-003 "세계영향=world_affecting 정본" supersede — D36, 별도 제안서 필요 \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LIFECYCLE_SCOPELOCK_UI_OVERLAY_NUMERIC_V2_20260404.md` | 58 | - \`docs/ssot/pack/world_affecting_view_switch_v1/\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LIFECYCLE_SCOPELOCK_UI_OVERLAY_NUMERIC_V2_20260404.md` | 188 | - \`view_only / world_affecting\` 회귀팩 closure | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LIFECYCLE_SCOPELOCK_UI_OVERLAY_NUMERIC_V2_20260404.md` | 217 | \| UI 경계 \| \`view_only_switch_no_statehash_change_v1\`, \`world_affecting_view_switch_v1\` \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LIFECYCLE_SCOPELOCK_UI_OVERLAY_NUMERIC_V2_20260404.md` | 233 | - \`world_affecting\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LIFECYCLE_SCOPELOCK_UI_OVERLAY_NUMERIC_V2_20260404.md` | 246 | - 2종(\`보기만\` / \`세계영향\`)으로 충분하다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LIFECYCLE_SCOPELOCK_UI_OVERLAY_NUMERIC_V2_20260404.md` | 373 | - \`world_affecting_view_switch_v1\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LIFECYCLE_SCOPELOCK_UI_OVERLAY_NUMERIC_V2_20260404.md` | 414 | 5. UI 입력은 \`view_only / world_affecting\` 2종만 허용한다 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_LIFECYCLE_SCOPELOCK_UI_OVERLAY_NUMERIC_V2_20260404.md` | 434 | 10. UI 입력 경계의 action kind는 \`view_only / world_affecting\` 2종만 둔다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_EXECUTION_TAXONOMY_V0_20260425_r2.md` | 108 | 모든 \`world_affecting\` 작용은 직접 세계를 바꾸지 않고, resolved event가 된 뒤 \`InputSnapshot\` 경계를 탄다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_EXECUTION_TAXONOMY_V0_20260425_r2.md` | 247 | 실제 금지/허용 규칙 = world_affecting | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_EXECUTION_TAXONOMY_V0_20260425_r2.md` | 392 | 실험 파라미터가 world state를 바꾸면 world_affecting | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_EXECUTION_TAXONOMY_V0_20260425_r2.md` | 505 | - 승인 전 world_affecting event가 되지 않는다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_EXECUTION_TAXONOMY_V0_20260425_r2.md` | 567 | "state_effect": "world_affecting", | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_EXECUTION_TAXONOMY_V0_20260425_r2.md` | 624 | - resolved world_affecting event만 InputSnapshot에 봉인된다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_EXECUTION_TAXONOMY_V0_20260425_r2.md` | 710 | - world_affecting | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_EXECUTION_TAXONOMY_V0_20260425_r2.md` | 720 | #### 8.1.1 \`world_affecting\` 결 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_EXECUTION_TAXONOMY_V0_20260425_r2.md` | 722 | \`world_affecting\` 결은 바람장, 중력장, 마력장, 지형 경사처럼 실제 실행 결과에 영향을 주는 장이다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_EXECUTION_TAXONOMY_V0_20260425_r2.md` | 817 | - world_affecting은 InputSnapshot 경계. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_EXECUTION_TAXONOMY_V0_20260425_r2.md` | 821 | - \`field_grain\`은 \`state_effect = world_affecting \| view_only\`만 허용하며, \`by_context\` 류 표현은 금지한다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_EXECUTION_TAXONOMY_V0_20260425_r2.md` | 840 | - world_affecting 작용은 InputSnapshot 경계를 따른다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_EXECUTION_TAXONOMY_V0_20260425_r2.md` | 847 | - 결/거스름 field_grain은 \`state_effect = world_affecting \| view_only\`만 허용하고 by_context류 표현을 금지한다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_TOTAL_CATALOG_V0_20260425.md` | 35 | \| \`world_affecting\` \| 실제 상태를 바꾸며 \`InputSnapshot\` 경유 \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_TOTAL_CATALOG_V0_20260425.md` | 56 | \| 말계 \| 말주문 \| \`immediate_world_script\` \| \`world_affecting\` \| 자연어형 표면은 \`phrase_pattern\` \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_TOTAL_CATALOG_V0_20260425.md` | 57 | \| 글계 \| 룬/새김 \| \`immediate_world_script\` \| \`world_affecting\` \| 지속 trigger \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_TOTAL_CATALOG_V0_20260425.md` | 58 | \| 꼴계 \| 마법진/결계 \| \`immediate_world_script\` \| \`world_affecting\` \| overlay와 규칙을 분리 \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_TOTAL_CATALOG_V0_20260425.md` | 59 | \| 몸계 \| 무예/춤 \| \`state_machine_resource_flow\` \| \`world_affecting\` \| gesture/combo \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_TOTAL_CATALOG_V0_20260425.md` | 60 | \| 물계 \| 연금술 \| \`recipe_pipeline\` \| \`world_affecting\` \| 재료/순서/품질 \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_TOTAL_CATALOG_V0_20260425.md` | 61 | \| 물계 \| 요리/약선 \| \`recipe_pipeline\` \| \`world_affecting\` \| 섭취 후 지연 효과 \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_ACTION_METHOD_TOTAL_CATALOG_V0_20260425.md` | 62 | \| 쇠계 \| 기계술 \| \`state_machine_resource_flow\` \| \`world_affecting\` \| 압력/밸브/연료 \| | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_PHRASE_TO_ACTION_METHOD_MODEL_V0_20260425.md` | 29 | 3. \`view_only\`와 \`world_affecting\`을 구분한다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_PHRASE_TO_ACTION_METHOD_MODEL_V0_20260425.md` | 94 | "state_effect": "world_affecting" | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_NURI_WORLD_SCRIPT_BINDING_MODEL_V0_20260425.md` | 46 | "state_effect": "world_affecting" | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_SEULGI_ASSISTED_MATCHING_BOUNDARY_V0.md` | 36 | - world_affecting이면 InputSnapshot에 봉인 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MALHIM_SEULGI_ASSISTED_MATCHING_BOUNDARY_V0.md` | 91 | "state_effect": "world_affecting" | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_MULTI_AGENT_TIME_ACTION_RUNTIME_V1_20260401.md` | 423 | - \`view_only\` / \`world_affecting\` 분리 유지 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_PROJECT_NURI_NONCORE_MAP_V1_20260404.md` | 93 | - \`보기만 / 세계영향\` 축 분리 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_PROJECT_NURI_NONCORE_MAP_V1_20260404.md` | 97 | - replay run에서 \`세계영향\` 실주입 금지 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_PROJECT_NURI_NONCORE_MAP_V1_20260404.md` | 378 | - \`보기만 / 세계영향\` 분리 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_PROJECT_NURI_NONCORE_MAP_V1_20260404.md` | 380 | - replay에서 \`세계영향\` 실주입 금지 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_PR_BATCH_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410.md` | 144 | - \`view_only\` / \`world_affecting\` 분리 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_PR_BATCH_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410.md` | 219 | - \`effect_kind = view_only \| world_affecting\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_PR_BATCH_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410.md` | 222 | - \`world_affecting\` 은 반드시 \`InputSnapshot\` 경유 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_PR_BATCH_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410.md` | 286 | 6. \`docs/ssot/pack/ui_event_to_inputsnapshot_v1/fixtures/world_affecting_submit.detjson\` *(신규)* | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_PR_BATCH_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410.md` | 304 | 20. \`docs/ssot/pack/sam_inputsnapshot_contract_v1/fixtures/ui_world_affecting.detjson\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_PR_BATCH_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410.md` | 332 | - \`u02_world_affecting_submit_snapshotted\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_PR_BATCH_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410.md` | 337 | - \`world_affecting\` action 은 \`InputSnapshot\` 기록이 남음 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_PR_BATCH_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410.md` | 346 | - \`행동갈래_목록 = 보기만 / 세계영향\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_ROADMAP_V2_GANADA_15_JULGI_6_MARU_MASTER_20260426_FULL_R2.md` | 1036 | - 보개 조작은 view_only / world_affecting으로 분리한다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_RUNTIME_GOLDEN_CLOSURE_EXTEND_V1_20260411.md` | 67 | - \`view_only / world_affecting\` 분리 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_RUNTIME_GOLDEN_CLOSURE_EXTEND_V1_20260411.md` | 73 | - \`world_affecting\` 입력은 반드시 \`InputSnapshot\` 경유 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 18 | 3. **\`ui.event -> InputSnapshot\`** exact schema 를 닫아, \`view_only / world_affecting\` 경계를 pack으로 증명한다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 33 | - \`view_only\` 와 \`world_affecting\` 은 구분되어 있으나, 실제 입력 포맷과 replay 재주입 형식은 아직 하드컷되지 않았다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 134 | - 세계영향 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 138 | - \`보기만/세계영향\`은 **입력원천 축과 섞지 않는다.** | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 140 | - replay 중 \`세계영향\` 실주입은 금지한다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 158 | - \`world_affecting\` 은 replay/감사 경계 안의 입력 사건 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 172 | "kind": "world_affecting", | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 187 | - \`action.kind\` 는 \`view_only \| world_affecting\` 둘만 허용 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 340 | 15. \`docs/ssot/pack/world_affecting_view_switch_v1/README.md\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 389 | - \`view_only / world_affecting\` 분리 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 466 | - \`action.kind = view_only \| world_affecting\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 479 | - \`view_only/world_affecting\` lowering 차이 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 488 | 2. \`world_affecting_move.detjson\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 508 | - \`c02_human_live_world_affecting\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 509 | - \`c03_seulgi_live_world_affecting\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 516 | - replay 중 \`세계영향\` 실주입 금지 검증 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_SAM_SEULGI_CAPABILITY_GATE_MATRIX_V1_20260410 (1).md` | 705 | - \`world_affecting\` 입력은 snapshot 기록이 남음 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_STORY_TECH_SYNTHESIS_V1_20260412.md` | 110 | - \`view_only / world_affecting\` 분리를 존중함 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_0_0_INTEGRATED_TIME_ACTION_RUNTIME_AND_NATIVE_TERM_LOCK_20260403.md` | 170 | - \`view_only\` 와 \`world_affecting\` 은 source와 섞지 않는다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_0_0_INTEGRATED_TIME_ACTION_RUNTIME_AND_NATIVE_TERM_LOCK_20260403.md` | 269 | - \`누리영향\` = world_affecting | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_0_0_INTEGRATED_TIME_ACTION_RUNTIME_AND_NATIVE_TERM_LOCK_20260403.md` | 374 | - \`docs/ssot/pack/view_only_vs_world_affecting_boundary_v1/\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_0_1_CONFLICT_FIXES_20260404.md` | 140 | ## 5. \`세계영향\`으로 정렬 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_0_1_CONFLICT_FIXES_20260404.md` | 143 | v22/v21 본문은 \`세계영향\`을 썼고, | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_0_1_CONFLICT_FIXES_20260404.md` | 147 | 이번 정정에서는 canonical 을 **\`세계영향\`** 으로 둔다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_0_1_CONFLICT_FIXES_20260404.md` | 150 | - \`세계영향\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_0_1_CONFLICT_FIXES_20260404.md` | 215 | - \`세계영향\` canonical 정렬 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_1_UNREFLECTED_DISCUSSIONS_SWEEP_V1_20260408.md` | 210 | - \`행동갈래\`: \`보기만 / 세계영향\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_1_UNREFLECTED_DISCUSSIONS_SWEEP_V1_20260408.md` | 223 | - \`docs/ssot/pack/world_affecting_view_switch_v1/**\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_1_UNREFLECTED_DISCUSSIONS_SWEEP_V1_20260408.md` | 229 | - \`world_affecting_view_switch_v1\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_EXECUTION_CLOSURE_TRIAD_AND_SURFACE_GAPS_V1_20260407.md` | 18 | 2. \`ui.event -> InputSnapshot\` exact schema + \`view_only/world_affecting\` 경계 closure | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_EXECUTION_CLOSURE_TRIAD_AND_SURFACE_GAPS_V1_20260407.md` | 31 | - \`view_only\`와 \`world_affecting\` 구분은 state_hash/replay 신뢰의 핵심이다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_EXECUTION_CLOSURE_TRIAD_AND_SURFACE_GAPS_V1_20260407.md` | 95 | - \`action.kind = view_only \| world_affecting\` exact lock | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_EXECUTION_CLOSURE_TRIAD_AND_SURFACE_GAPS_V1_20260407.md` | 97 | - \`view_only\` 조작은 \`state_hash\` 밖, \`world_affecting\`은 \`InputSnapshot\` 경유 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_EXECUTION_CLOSURE_TRIAD_AND_SURFACE_GAPS_V1_20260407.md` | 103 | - \`world_affecting_view_switch_v1\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_EXECUTION_CLOSURE_TRIAD_AND_SURFACE_GAPS_V1_20260407.md` | 109 | - \`view_only\`와 \`world_affecting\` 혼합 해석 금지 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_EXECUTION_CLOSURE_TRIAD_AND_SURFACE_GAPS_V1_20260407.md` | 219 | - \`world_affecting_view_switch_v1\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_EXECUTION_CLOSURE_WITH_V23_0_14_AND_DEV_SUMMARY_V2_20260407.md` | 19 | 2. **P0-B** \`ui.event -> InputSnapshot\` + \`view_only/world_affecting\` 경계 closure | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_EXECUTION_CLOSURE_WITH_V23_0_14_AND_DEV_SUMMARY_V2_20260407.md` | 123 | - \`action.kind = view_only \| world_affecting\` exact lock | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_EXECUTION_CLOSURE_WITH_V23_0_14_AND_DEV_SUMMARY_V2_20260407.md` | 125 | - \`view_only\`는 \`state_hash\` 밖, \`world_affecting\`은 \`InputSnapshot\` 경유 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_EXECUTION_CLOSURE_WITH_V23_0_14_AND_DEV_SUMMARY_V2_20260407.md` | 130 | - \`world_affecting_view_switch_v1\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_EXECUTION_CLOSURE_WITH_V23_0_14_AND_DEV_SUMMARY_V2_20260407.md` | 132 | - \`world_affecting\` 입력은 snapshot 기록 + replay 재주입 사례 1종 이상 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_1_EXECUTION_CLOSURE_WITH_V23_0_14_AND_DEV_SUMMARY_V2_20260407.md` | 136 | - \`view_only\`와 \`world_affecting\`의 이중 해석 금지 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_2_EXECUTION_DOCS_CONSISTENCY_BUNDLE_V1_20260409.md` | 207 | - \`world_affecting_view_switch_v1\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_2_SEULGI_INTENT_PACK_AND_DDN_SURFACE_V1_20260409.md` | 65 | - 모든 세계영향은 샘(InputSnapshot) 경유 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_2_SEULGI_INTENT_PACK_AND_DDN_SURFACE_V1_20260409.md` | 206 | "행동갈래": "세계영향", | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_2_SEULGI_INTENT_PACK_AND_DDN_SURFACE_V1_20260409.md` | 221 | - \`행동갈래\` : \`보기만 \| 세계영향\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_2_SEULGI_INTENT_PACK_AND_DDN_SURFACE_V1_20260409.md` | 252 | - \`행동갈래\`: \`보기만 / 세계영향\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_2_SEULGI_INTENT_PACK_AND_DDN_SURFACE_V1_20260409.md` | 265 | - replay에서 \`세계영향\` 실주입 금지 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V23_2_SEULGI_INTENT_PACK_AND_DDN_SURFACE_V1_20260409.md` | 389 | - [ ] replay 중 \`세계영향\` live inject 금지 검증이 golden/checker에 포함됨 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/PROPOSAL_V24_RUNTIME_PROMOTION_AND_DET_CAPABILITY_MATRIX_V1_20260410 (1).md` | 286 | - Gatekeeper 가 검증 가능한 세계영향만 runtime 으로 허용한다. | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/SEAMGRIM_PUZZLE_GAME_READINESS_REVIEW_V1_20260406.md` | 25 | - \`view_only\` vs \`world_affecting\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/drafts/malhim_phrase_action_method_v0/malhim.action_method.v0.json` | 51 | "world_affecting", | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/drafts/malhim_phrase_action_method_v0/malhim.action_method.v0.json` | 78 | "state_effect": "world_affecting" | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/drafts/malhim_phrase_action_method_v0/malhim.phrase_pattern.v0.json` | 40 | "world_affecting", | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/drafts/malhim_phrase_action_method_v0/malhim.phrase_pattern.v0.json` | 70 | "state_effect_hint": "world_affecting" | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/p04_external_replay_20260401.md` | 46 | - \`world_affecting\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/p04_external_replay_20260401.md` | 57 | - \`view_only_vs_world_affecting_boundary_v1\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/proposal_execution_axes_v_3_20260404.md` | 174 | - \`세계영향\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/proposal_execution_axes_v_3_20260404.md` | 523 | ### \`세계영향\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/proposal_execution_axes_v_3_20260404.md` | 544 | - “보기만 vs 세계영향” 개념 설명 | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/context/proposals/proposal_malhim_seulgi_assisted_matching_boundary_v0_v2_20260425.md` | 39 | - \`world_affecting\`이면 \`InputSnapshot\` | 커널 방향/제안 문서 용어 영향. 제안 상태 유지, 직접 수정 없음. |
| `docs/reports/audit/REPORT_KEEP_MOVE_ARCHIVE_DELETE_CANDIDATES_V1.md` | 2950 | \| docs\ssot\pack\world_affecting_view_switch_v1 \| ssot_pack_spec \| internal_reference \| keep \| active \| - \| - \| - \| SSOT pack 정본/예시 \| docs/ssot/pack/** \| low \| - \| | 문서/검증 문자열 영향 후보. |
| `docs/reports/audit/REPORT_KEEP_MOVE_ARCHIVE_DELETE_CANDIDATES_V1.md` | 2951 | \| docs\ssot\pack\world_affecting_view_switch_v1\expected \| ssot_pack_spec \| internal_reference \| keep \| active \| - \| - \| - \| SSOT pack 정본/예시 \| docs/ssot/pack/** \| low... | 문서/검증 문자열 영향 후보. |
| `docs/reports/audit/REPORT_KEEP_MOVE_ARCHIVE_DELETE_CANDIDATES_V1.md` | 2952 | \| docs\ssot\pack\world_affecting_view_switch_v1\tests \| ssot_pack_spec \| internal_reference \| keep \| active \| - \| - \| - \| SSOT pack 정본/예시 \| docs/ssot/pack/** \| low \|... | 문서/검증 문자열 영향 후보. |
| `docs/ssot/appendix/csv/combined_term_ledger_long.csv` | 1248 | concept_term,world_affecting,ko,세계영향,T0/T1 proposed; T2 draft | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/appendix/csv/combined_term_ledger_long.csv` | 1249 | concept_term,world_affecting,ko_common,세계영향,T0/T1 proposed; T2 draft | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/appendix/csv/combined_term_ledger_long.csv` | 1250 | concept_term,world_affecting,sym3,,T0/T1 proposed; T2 draft | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/appendix/csv/combined_term_ledger_long.csv` | 1251 | concept_term,world_affecting,en,world_affecting,T0/T1 proposed; T2 draft | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/appendix/csv/combined_term_ledger_long.csv` | 1252 | concept_term,world_affecting,ja,世界影響,T0/T1 proposed; T2 draft | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/appendix/csv/combined_term_ledger_long.csv` | 1253 | concept_term,world_affecting,mn,ертөнцөд_нөлөөлөх,T0/T1 proposed; T2 draft | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/appendix/csv/combined_term_ledger_long.csv` | 1254 | concept_term,world_affecting,tr,dünya_etkili,T0/T1 proposed; T2 draft | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/appendix/csv/combined_term_ledger_long.csv` | 1255 | concept_term,world_affecting,qu,⟦TBD_NATIVE_REVIEW⟧,T0/T1 proposed; T2 draft | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/appendix/csv/combined_term_ledger_long.csv` | 1256 | concept_term,world_affecting,ay,⟦TBD_NATIVE_REVIEW⟧,T0/T1 proposed; T2 draft | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/appendix/csv/combined_term_ledger_long.csv` | 1257 | concept_term,world_affecting,ne,विश्व_प्रभाव,T0/T1 proposed; T2 draft | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/appendix/csv/combined_term_ledger_long.csv` | 1258 | concept_term,world_affecting,ta,உலக_பாதிப்பு,T0/T1 proposed; T2 draft | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/appendix/csv/combined_term_ledger_long.csv` | 1259 | concept_term,world_affecting,te,ప్రపంచ_ప్రభావ,T0/T1 proposed; T2 draft | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/appendix/csv/combined_term_ledger_long.csv` | 1260 | concept_term,world_affecting,kn,ಲೋಕ_ಪ್ರಭಾವ,T0/T1 proposed; T2 draft | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/appendix/csv/combined_term_ledger_long.csv` | 1261 | concept_term,world_affecting,eu,mundu_eragin,T0/T1 proposed; T2 draft | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/appendix/csv/concept_term_matrix_full.csv` | 13 | world_affecting,T0/T1 proposed; T2 draft,세계영향,세계영향,,world_affecting,世界影響,ертөнцөд_нөлөөлөх,dünya_etkili,⟦TBD_NATIVE_REVIEW⟧,⟦TBD_NATIVE_REVIEW⟧,विश्व_प्रभाव,உலக_பாதிப்பு,ప్రపంచ_... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/pack/bogae_script_game_ui_v0/README.md` | 13 | 4. **view_only / world_affecting 분리** | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/pack/bogae_script_game_ui_v0/README.md` | 17 | - 행동/제출 버튼은 \`world_affecting\` 입력 사건으로 처리함을 문서화 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/pack/bogae_script_game_ui_v0/README.md` | 20 | - \`world_affecting_submit\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/pack/world_affecting_view_switch_v1/README.md` | 1 | # world_affecting_view_switch_v1 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/pack/world_affecting_view_switch_v1/README.md` | 8 | 1. action.kind = world_affecting | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/pack/world_affecting_view_switch_v1/expected/PLACEHOLDER.md` | 1 | # world_affecting_view_switch_v1 expected placeholder | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/pack/world_affecting_view_switch_v1/expected/PLACEHOLDER.md` | 6 | - action.kind = world_affecting | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/pack/world_affecting_view_switch_v1/meta.toml` | 1 | id = "world_affecting_view_switch_v1" | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/pack/world_affecting_view_switch_v1/tests/README.md` | 1 | # tests for world_affecting_view_switch_v1 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/README_v23.0.6.md` | 20 | > - (AXIS LOCK) 외부개입은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/README_v23.0.6.md` | 21 | > - (REPLAY RULE) \`재연주입\`은 원천 재호출 없이 봉인 기록을 다시 넣는 경로다. replay run 에서 **\`세계영향\` 실주입은 금지**하고, \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/README_v23.0.6.md` | 39 | > - (AXES FIX) 실행축 canonical은 **\`#모두필요 / #빈칸채움\`**, **\`세계영향\`**, **\`한걸음(designed / STUB_ONLY)\`**, **\`살핌내주기갈래 = 설정.거울\`**, **\`놀이기본 = 없음보람 + 해당없음(N/A)\`** 로 정렬한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/README_v23.0.6.md` | 99 | > - (PLATFORM/RULE) 버튼/입력칸/토글/선택 surface는 보개/UI에 있을 수 있지만, **직접 상태 변경은 금지**한다. 세계 의미가 있는 입력은 반드시 \`ui.event -> InputSnapshot -> 이야기/누리\` 경계를 따라야 하며, \`view_only\` 전환은 \`state_ha... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/README_v23.0.6.md` | 101 | > - (PACK/SUPPORT) \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 29 | > - (AXIS LOCK) 외부개입은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 30 | > - (REPLAY RULE) \`재연주입\`은 원천 재호출 없이 봉인 기록을 다시 넣는 경로다. replay run 에서 **\`세계영향\` 실주입은 금지**하고, \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 48 | > - (AXES FIX) 실행축 canonical은 **\`#모두필요 / #빈칸채움\`**, **\`세계영향\`**, **\`한걸음(designed / STUB_ONLY)\`**, **\`살핌내주기갈래 = 설정.거울\`**, **\`놀이기본 = 없음보람 + 해당없음(N/A)\`** 로 정렬한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 108 | > - (PLATFORM/RULE) 버튼/입력칸/토글/선택 surface는 보개/UI에 있을 수 있지만, **직접 상태 변경은 금지**한다. 세계 의미가 있는 입력은 반드시 \`ui.event -> InputSnapshot -> 이야기/누리\` 경계를 따라야 하며, \`view_only\` 전환은 \`state_ha... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 110 | > - (PACK/SUPPORT) \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 404 | - 외부개입 정본 3축은 **\`입력원천 / 주입방식 / 행동갈래\`** 이다. \`입력원천\` 6값은 **\`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`**, \`주입방식\` 2값은 **\`실주입 / 재연주입\`**, \`행동갈래\` 2값은 **\`보기만 / 세계영향\`** 으로 잠근다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 405 | - replay run 에서는 **\`세계영향\` 실주입을 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 426 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 한국어 정본으로 정리한다. action kind 정본은 **\`보기만 / 세계영향\`** 이며, source 값의 \`schedule\` 한국어 label은 **\`일정\`** 으로 고정한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 480 | - full export에는 \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`, \... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 633 | - \`행동갈래\` = \`보기만 / 세계영향\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 642 | 7. replay run 에서 **\`세계영향\` 실주입은 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 1219 | > **v23.0.5 용어 상태:** 외부개입 축은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`입력원천\` 6값은 \`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`, \`주입방식\` 2값은 \`실주입 / 재연주입\`, \`행동갈래\` 2값은 \`보기만 / 세계영향\` 이다. ... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 1428 | \| **세계영향** \| 세계 규칙/상태 전이에 영향을 주는 입력 사건 \| world_affecting \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 1561 | - canonical 값은 \`보기만 / 세계영향\` 이며, \`입력원천\` / \`주입방식\` 과 섞지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 2092 | \| **세계영향** \| 세계 상태/규칙에 영향을 주는 action kind \| world_affecting \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 9383 | - action kind 정본은 **\`보기만 / 세계영향\`** 이다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 9395 | \| \`세계영향\` \| action kind / input meaning \| AGE3 \| partial \| STUB_ONLY \| canonical world-affecting kind \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 9424 | - replay 경로에서는 **원천 재호출 0건**을 유지하고, \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 9474 | - \`world_affecting_view_switch_v1\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 11066 | - \`world_affecting_view_switch_v1\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 14098 | 4. replay run 에서 **\`세계영향\` 실주입은 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 14217 | - replay run 에서는 \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 14240 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 으로 정리하고, \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 14274 | - \`ui.event -> InputSnapshot\`, \`view_only\` / \`world_affecting\`, 보개 capability boundary는 \`v20.22.0\` 기준선을 그대로 유지한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 14404 | - \`행동갈래\` = 세계 의미 여부. 값은 \`보기만 / 세계영향\` 이다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 14406 | - \`보기만\` 과 \`세계영향\` 은 source/origin 축과 섞지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 14415 | - \`view_only / world_affecting\` 회귀팩 closure | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 14430 | - action kind는 \`view_only / world_affecting\` 2종만 둔다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 16384 | - \`행동갈래\` = \`보기만 / 세계영향\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 16398 | - replay run 에서 **\`세계영향\` 실주입은 금지**한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 16472 | - **world_affecting** | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 16480 | 2. \`world_affecting\` 액션은 반드시 replay/감사 경계 안으로 들어가는 입력 사건이어야 한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 16482 | 4. \`view_only\` 전환과 \`world_affecting\` 입력을 같은 버튼/화면 안에 공존시킬 수는 있지만, action kind 를 분리 기록해야 한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 16488 | - \`world_affecting_view_switch_v1\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 16537 | - replay run 에서 \`세계영향\` 실주입 금지 + \`recv_seq\` 정렬 규칙 명시 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 16559 | - \`view_only / world_affecting\` 경계와 overlay static-one-layer guard 등록 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 18667 | \| AGE3 \| REFERENCE \| \`ui.event -> InputSnapshot\` 경계와 \`view_only / world_affecting\` 구분을 platform/toolchain 규약으로 고정 \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 18669 | \| AGE3 \| REFERENCE \| \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 21918 | \| AGE3 \| ui.event boundary \| OPEN \| \`ui.event -> InputSnapshot -> 이야기/누리\`, \`view_only / world_affecting\` 분리 \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 21926 | - \`world_affecting_view_switch_v1\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 22084 | - \`view_only / world_affecting\` 2종 경계 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 23766 | - replay 중 \`세계영향\` 실주입 금지와 \`보기만\` toolchain 예외를 runner/golden 으로 검증 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 23914 | 2. \`action.kind = view_only \| world_affecting\` 기준 확정 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 23937 | - \`world_affecting\`은 입력 사건 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 23941 | 3. \`view_only_switch_no_statehash_change_v1\` / \`world_affecting_view_switch_v1\` pack closure | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 24027 | - replay run 에서 \`세계영향\` 실주입 금지, \`보기만\` toolchain 예외, \`recv_seq\` 기반 동일 마디 정렬 규칙 추가 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 24046 | - AGE4 = Refine/Settle, overlay AGE4 최소범위, UI \`view_only/world_affecting\` 2종, numeric partial-closure 표현 정리 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 25460 | - \`PROPOSAL_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328_v2\`를 반영해 \`view meaning vs composition\`, \`family/backend/profile/madang\`, \`2d -> space2d\`, \`3d -> space3d\`... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ALL_v23.0.6.md` | 25462 | - 신규 skeleton pack 6종(\`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_LANG_v23.0.6.md` | 7242 | - action kind 정본은 **\`보기만 / 세계영향\`** 이다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_LANG_v23.0.6.md` | 7254 | \| \`세계영향\` \| action kind / input meaning \| AGE3 \| partial \| STUB_ONLY \| canonical world-affecting kind \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_MASTER_v23.0.6.md` | 20 | - 외부개입 정본 3축은 **\`입력원천 / 주입방식 / 행동갈래\`** 이다. \`입력원천\` 6값은 **\`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`**, \`주입방식\` 2값은 **\`실주입 / 재연주입\`**, \`행동갈래\` 2값은 **\`보기만 / 세계영향\`** 으로 잠근다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_MASTER_v23.0.6.md` | 21 | - replay run 에서는 **\`세계영향\` 실주입을 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_MASTER_v23.0.6.md` | 42 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 한국어 정본으로 정리한다. action kind 정본은 **\`보기만 / 세계영향\`** 이며, source 값의 \`schedule\` 한국어 label은 **\`일정\`** 으로 고정한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_MASTER_v23.0.6.md` | 96 | - full export에는 \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`, \... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_MASTER_v23.0.6.md` | 249 | - \`행동갈래\` = \`보기만 / 세계영향\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_MASTER_v23.0.6.md` | 258 | 7. replay run 에서 **\`세계영향\` 실주입은 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_OPEN_ISSUES_v23.0.6.md` | 102 | - \`view_only / world_affecting\` 2종 경계 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_OPEN_ISSUES_v23.0.6.md` | 1784 | - replay 중 \`세계영향\` 실주입 금지와 \`보기만\` toolchain 예외를 runner/golden 으로 검증 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_OPEN_ISSUES_v23.0.6.md` | 1932 | 2. \`action.kind = view_only \| world_affecting\` 기준 확정 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_OPEN_ISSUES_v23.0.6.md` | 1955 | - \`world_affecting\`은 입력 사건 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_OPEN_ISSUES_v23.0.6.md` | 1959 | 3. \`view_only_switch_no_statehash_change_v1\` / \`world_affecting_view_switch_v1\` pack closure | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PENDING_v23.0.6.md` | 23 | - replay run 에서 \`세계영향\` 실주입 금지, \`보기만\` toolchain 예외, \`recv_seq\` 기반 동일 마디 정렬 규칙 추가 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PENDING_v23.0.6.md` | 42 | - AGE4 = Refine/Settle, overlay AGE4 최소범위, UI \`view_only/world_affecting\` 2종, numeric partial-closure 표현 정리 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PENDING_v23.0.6.md` | 1456 | - \`PROPOSAL_BOGAE_VIEW_MEANING_AND_INPUT_BOUNDARY_V1_20260328_v2\`를 반영해 \`view meaning vs composition\`, \`family/backend/profile/madang\`, \`2d -> space2d\`, \`3d -> space3d\`... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PENDING_v23.0.6.md` | 1458 | - 신규 skeleton pack 6종(\`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLANS_v23.0.6.md` | 29 | - replay run 에서 \`세계영향\` 실주입 금지 + \`recv_seq\` 정렬 규칙 명시 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLANS_v23.0.6.md` | 51 | - \`view_only / world_affecting\` 경계와 overlay static-one-layer guard 등록 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLANS_v23.0.6.md` | 2159 | \| AGE3 \| REFERENCE \| \`ui.event -> InputSnapshot\` 경계와 \`view_only / world_affecting\` 구분을 platform/toolchain 규약으로 고정 \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLANS_v23.0.6.md` | 2161 | \| AGE3 \| REFERENCE \| \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLATFORM_v23.0.6.md` | 23 | - replay run 에서는 \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLATFORM_v23.0.6.md` | 46 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 으로 정리하고, \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLATFORM_v23.0.6.md` | 80 | - \`ui.event -> InputSnapshot\`, \`view_only\` / \`world_affecting\`, 보개 capability boundary는 \`v20.22.0\` 기준선을 그대로 유지한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLATFORM_v23.0.6.md` | 210 | - \`행동갈래\` = 세계 의미 여부. 값은 \`보기만 / 세계영향\` 이다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLATFORM_v23.0.6.md` | 212 | - \`보기만\` 과 \`세계영향\` 은 source/origin 축과 섞지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLATFORM_v23.0.6.md` | 221 | - \`view_only / world_affecting\` 회귀팩 closure | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLATFORM_v23.0.6.md` | 236 | - action kind는 \`view_only / world_affecting\` 2종만 둔다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLATFORM_v23.0.6.md` | 2190 | - \`행동갈래\` = \`보기만 / 세계영향\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLATFORM_v23.0.6.md` | 2204 | - replay run 에서 **\`세계영향\` 실주입은 금지**한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLATFORM_v23.0.6.md` | 2278 | - **world_affecting** | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLATFORM_v23.0.6.md` | 2286 | 2. \`world_affecting\` 액션은 반드시 replay/감사 경계 안으로 들어가는 입력 사건이어야 한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLATFORM_v23.0.6.md` | 2288 | 4. \`view_only\` 전환과 \`world_affecting\` 입력을 같은 버튼/화면 안에 공존시킬 수는 있지만, action kind 를 분리 기록해야 한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_PLATFORM_v23.0.6.md` | 2294 | - \`world_affecting_view_switch_v1\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ROADMAP_CATALOG_v23.0.6.md` | 3146 | \| AGE3 \| ui.event boundary \| OPEN \| \`ui.event -> InputSnapshot -> 이야기/누리\`, \`view_only / world_affecting\` 분리 \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_ROADMAP_CATALOG_v23.0.6.md` | 3154 | - \`world_affecting_view_switch_v1\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_TERMS_v23.0.6.md` | 22 | > **v23.0.5 용어 상태:** 외부개입 축은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`입력원천\` 6값은 \`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`, \`주입방식\` 2값은 \`실주입 / 재연주입\`, \`행동갈래\` 2값은 \`보기만 / 세계영향\` 이다. ... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_TERMS_v23.0.6.md` | 231 | \| **세계영향** \| 세계 규칙/상태 전이에 영향을 주는 입력 사건 \| world_affecting \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_TERMS_v23.0.6.md` | 364 | - canonical 값은 \`보기만 / 세계영향\` 이며, \`입력원천\` / \`주입방식\` 과 섞지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_TERMS_v23.0.6.md` | 895 | \| **세계영향** \| 세계 상태/규칙에 영향을 주는 action kind \| world_affecting \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_TOOLCHAIN_v23.0.6.md` | 20 | - replay 경로에서는 **원천 재호출 0건**을 유지하고, \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_TOOLCHAIN_v23.0.6.md` | 70 | - \`world_affecting_view_switch_v1\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_TOOLCHAIN_v23.0.6.md` | 1662 | - \`world_affecting_view_switch_v1\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/releases/v23.0.6/SSOT_TOOLCHAIN_v23.0.6.md` | 4694 | 4. replay run 에서 **\`세계영향\` 실주입은 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/solutions/05_schemas_catalog_v0/schema/ui.v0.md` | 52 | - \`action.kind = view_only \| world_affecting\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/README_v24.12.9.md` | 222 | > - (SAM/BOUNDARY) 샘(Sam)은 더 이상 입력 장치 묶음이 아니라 **외부개입을 \`InputSnapshot\`으로 봉인·재연하는 실행 경계**다. \`ui.event -> InputSnapshot -> 이야기/누리\` 경계를 exact boundary로 적고, \`view_only / world_a... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/README_v24.12.9.md` | 350 | > - (AXIS LOCK) 외부개입은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/README_v24.12.9.md` | 351 | > - (REPLAY RULE) \`재연주입\`은 원천 재호출 없이 봉인 기록을 다시 넣는 경로다. replay run 에서 **\`세계영향\` 실주입은 금지**하고, \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/README_v24.12.9.md` | 369 | > - (AXES FIX) 실행축 canonical은 **\`#모두필요 / #빈칸채움\`**, **\`세계영향\`**, **\`한걸음(designed / STUB_ONLY)\`**, **\`살핌내주기갈래 = 설정.거울\`**, **\`놀이기본 = 없음보람 + 해당없음(N/A)\`** 로 정렬한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/README_v24.12.9.md` | 429 | > - (PLATFORM/RULE) 버튼/입력칸/토글/선택 surface는 보개/UI에 있을 수 있지만, **직접 상태 변경은 금지**한다. 세계 의미가 있는 입력은 반드시 \`ui.event -> InputSnapshot -> 이야기/누리\` 경계를 따라야 하며, \`view_only\` 전환은 \`state_ha... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/README_v24.12.9.md` | 431 | > - (PACK/SUPPORT) \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_LANG_v24.12.9.md` | 7852 | "행동갈래": "세계영향", | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_LANG_v24.12.9.md` | 8076 | - action kind 정본은 **\`보기만 / 세계영향\`** 이다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_LANG_v24.12.9.md` | 8095 | \| \`세계영향\` \| action kind / input meaning \| AGE3 \| partial \| STUB_ONLY \| canonical world-affecting kind \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_MASTER_v24.12.9.md` | 608 | - 외부개입 정본 3축은 **\`입력원천 / 주입방식 / 행동갈래\`** 이다. \`입력원천\` 6값은 **\`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`**, \`주입방식\` 2값은 **\`실주입 / 재연주입\`**, \`행동갈래\` 2값은 **\`보기만 / 세계영향\`** 으로 잠근다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_MASTER_v24.12.9.md` | 609 | - replay run 에서는 **\`세계영향\` 실주입을 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_MASTER_v24.12.9.md` | 630 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 한국어 정본으로 정리한다. action kind 정본은 **\`보기만 / 세계영향\`** 이며, source 값의 \`schedule\` 한국어 label은 **\`일정\`** 으로 고정한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_MASTER_v24.12.9.md` | 684 | - full export에는 \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot_v1\`, \... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_MASTER_v24.12.9.md` | 844 | - \`행동갈래\` = \`보기만 / 세계영향\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_MASTER_v24.12.9.md` | 853 | 7. replay run 에서 **\`세계영향\` 실주입은 금지**한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_OPEN_ISSUES_v24.12.9.md` | 454 | - world_affecting / view_only / analysis_only / DEFERRED 경계를 한 장부로 정리한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_OPEN_ISSUES_v24.12.9.md` | 1428 | - \`view_only / world_affecting\` 2종 경계 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_OPEN_ISSUES_v24.12.9.md` | 3114 | - replay 중 \`세계영향\` 실주입 금지와 \`보기만\` toolchain 예외를 runner/golden 으로 검증 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_OPEN_ISSUES_v24.12.9.md` | 3320 | 2. \`action.kind = view_only \| world_affecting\` 기준 확정 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_OPEN_ISSUES_v24.12.9.md` | 3343 | - \`world_affecting\`은 입력 사건 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_OPEN_ISSUES_v24.12.9.md` | 3347 | 3. \`view_only_switch_no_statehash_change_v1\` / \`world_affecting_view_switch_v1\` pack closure | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLANS_v24.12.9.md` | 770 | ### PLN-20260407-EXEC-CLOSURE-P0B-V1: \`ui.event -> InputSnapshot\` + \`view_only/world_affecting\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLANS_v24.12.9.md` | 775 | - \`action.kind = view_only \| world_affecting\` 경계를 exact lock 하고 replay 재주입 형식을 닫는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLANS_v24.12.9.md` | 778 | - \`ui_event_to_inputsnapshot_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`world_affecting_view_switch_v1\` pack closure | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLANS_v24.12.9.md` | 780 | - \`world_affecting\` 입력의 snapshot 기록 + replay 재주입 사례 1종 이상 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLANS_v24.12.9.md` | 1016 | - replay run 에서 \`세계영향\` 실주입 금지 + \`recv_seq\` 정렬 규칙 명시 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLANS_v24.12.9.md` | 1038 | - \`view_only / world_affecting\` 경계와 overlay static-one-layer guard 등록 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLANS_v24.12.9.md` | 3061 | - replay 중 \`세계영향\` 실주입 금지, \`보기만\` 실주입은 toolchain 예외로만 허용 | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLANS_v24.12.9.md` | 3166 | \| AGE3 \| REFERENCE \| \`ui.event -> InputSnapshot\` 경계와 \`view_only / world_affecting\` 구분을 platform/toolchain 규약으로 고정 \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLANS_v24.12.9.md` | 3168 | \| AGE3 \| REFERENCE \| \`view_family_capability_gate_v1\`, \`view_stack_primary_secondary_overlay_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`ui_event_to_inputsnapshot... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 431 | - execution ordering \`(agent_id, recv_seq)\`, replay/source recall 금지, \`ui.event -> InputSnapshot -> 이야기/누리\`, \`view_only / world_affecting\`, raw prompt/response/debug trace... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 439 | - \`view_only\` 와 \`world_affecting\` 분리는 그대로 유지하고, 3D consumer 경로에서 생기는 \`camera / asset / animation / HUD / layout\` 변화도 **adapter payload / view_meta** 쪽에 남긴다. \`engine3d.god... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 459 | - \`view_only\` 와 \`world_affecting\` 은 계속 분리하고, \`view_only\` 전환은 \`state_hash\` 밖, \`world_affecting\` 입력은 replay/audit 경계 안으로 둔다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 568 | - \`ui.event\`는 UI/도구 표면이고, world 의미를 바꾸는 입력은 여전히 **\`InputSnapshot\` 봉인 경계** 를 통과해야 한다. \`view_only\` 조작은 \`state_hash\` 밖, \`world_affecting\`은 snapshot/replay 경로 안에 있다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 672 | - replay run 에서는 \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용하며 \`state_hash\`를 바꾸지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 695 | - 외부개입 메타는 **\`입력원천 / 주입방식 / 실주입 / 재연주입\`** 으로 정리하고, \`보기만 / 세계영향\` 은 source 축과 섞지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 729 | - \`ui.event -> InputSnapshot\`, \`view_only\` / \`world_affecting\`, 보개 capability boundary는 \`v20.22.0\` 기준선을 그대로 유지한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 898 | - \`행동갈래\` = 세계 의미 여부. 값은 \`보기만 / 세계영향\` 이다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 900 | - \`보기만\` 과 \`세계영향\` 은 source/origin 축과 섞지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 909 | - \`view_only / world_affecting\` 회귀팩 closure | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 924 | - action kind는 \`view_only / world_affecting\` 2종만 둔다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 2906 | - \`행동갈래\` = \`보기만 / 세계영향\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 2921 | - replay run 에서 **\`세계영향\` 실주입은 금지**한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 2999 | - **world_affecting** | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 3007 | 2. \`world_affecting\` 액션은 반드시 replay/감사 경계 안으로 들어가는 입력 사건이어야 한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 3009 | 4. \`view_only\` 전환과 \`world_affecting\` 입력을 같은 버튼/화면 안에 공존시킬 수는 있지만, action kind 를 분리 기록해야 한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_PLATFORM_v24.12.9.md` | 3015 | - \`world_affecting_view_switch_v1\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_ROADMAP_CATALOG_v24.12.9.md` | 262 | - P0-B: \`ui_event_to_inputsnapshot_v1\`, \`view_only_switch_no_statehash_change_v1\`, \`world_affecting_view_switch_v1\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_ROADMAP_CATALOG_v24.12.9.md` | 3558 | \| AGE3 \| ui.event boundary \| OPEN \| \`ui.event -> InputSnapshot -> 이야기/누리\`, \`view_only / world_affecting\` 분리 \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_ROADMAP_CATALOG_v24.12.9.md` | 3566 | - \`world_affecting_view_switch_v1\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_TERMS_v24.12.9.md` | 280 | \| \`world_affecting\` \| 세계영향 \| 이야기/누리 의미를 바꾸는 입력 사건. \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_TERMS_v24.12.9.md` | 325 | \| \`world_affecting\` \| 세계영향 입력 \| world state 의미를 바꾸는 입력. 반드시 \`InputSnapshot\` 경유로 들어간다. \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_TERMS_v24.12.9.md` | 378 | > **v23.0.5 용어 상태:** 외부개입 축은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`입력원천\` 6값은 \`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`, \`주입방식\` 2값은 \`실주입 / 재연주입\`, \`행동갈래\` 2값은 \`보기만 / 세계영향\` 이다. ... | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_TERMS_v24.12.9.md` | 596 | \| **세계영향** \| 세계 규칙/상태 전이에 영향을 주는 입력 사건 \| world_affecting \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_TERMS_v24.12.9.md` | 738 | - canonical 값은 \`보기만 / 세계영향\` 이며, \`입력원천\` / \`주입방식\` 과 섞지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_TERMS_v24.12.9.md` | 1262 | \| **세계영향** \| 세계 상태/규칙에 영향을 주는 action kind \| world_affecting \| | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_TOOLCHAIN_v24.12.9.md` | 365 | - \`ui.event\` 최소 필드와 \`action.kind = view_only \| world_affecting\` exact lock, replay 재주입 형식은 \`InputSnapshot\` 봉인 규칙과 함께 다룬다. toolchain 은 direct state mutation UI 를 허용하지 않는다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_TOOLCHAIN_v24.12.9.md` | 435 | - replay 경로에서는 **원천 재호출 0건**을 유지하고, \`세계영향\` 실주입을 금지한다. \`보기만\` 실주입은 toolchain/view-dock 예외로만 허용한다. | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_TOOLCHAIN_v24.12.9.md` | 580 | - \`world_affecting_view_switch_v1\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `docs/ssot/ssot/SSOT_TOOLCHAIN_v24.12.9.md` | 2185 | - \`world_affecting_view_switch_v1\` | SSOT/정본 문서의 행동갈래 용어 영향. Codex 직접 수정 금지. |
| `pack/external_intent_boundary_v1/README.md` | 10 | - \`행동갈래\` 2값 (\`보기만\`, \`세계영향\`) | 팩 데이터/문서의 행동갈래 값. 누리바꿈 채택 시 alias 또는 fixture migration 필요. |
| `pack/external_intent_boundary_v1/boundary_contract.detjson` | 18 | "세계영향" | 팩 데이터/문서의 행동갈래 값. 누리바꿈 채택 시 alias 또는 fixture migration 필요. |
| `pack/external_intent_boundary_v1/boundary_contract.detjson` | 51 | "행동갈래": "세계영향", | 팩 데이터/문서의 행동갈래 값. 누리바꿈 채택 시 alias 또는 fixture migration 필요. |
| `pack/external_intent_boundary_v1/boundary_contract.detjson` | 59 | "행동갈래": "세계영향", | 팩 데이터/문서의 행동갈래 값. 누리바꿈 채택 시 alias 또는 fixture migration 필요. |
| `pack/external_intent_boundary_v1/boundary_contract.detjson` | 67 | "행동갈래": "세계영향", | 팩 데이터/문서의 행동갈래 값. 누리바꿈 채택 시 alias 또는 fixture migration 필요. |
| `pack/external_intent_boundary_v1/boundary_contract.detjson` | 75 | "행동갈래": "세계영향", | 팩 데이터/문서의 행동갈래 값. 누리바꿈 채택 시 alias 또는 fixture migration 필요. |
| `pack/external_intent_boundary_v1/boundary_contract.detjson` | 83 | "행동갈래": "세계영향", | 팩 데이터/문서의 행동갈래 값. 누리바꿈 채택 시 alias 또는 fixture migration 필요. |
| `pack/external_intent_boundary_v1/intent.md` | 16 | - \`행동갈래\`: \`보기만\` / \`세계영향\` | 팩 데이터/문서의 행동갈래 값. 누리바꿈 채택 시 alias 또는 fixture migration 필요. |
| `tests/run_external_intent_boundary_pack_check.py` | 11 | "c01_human_input_replay": ("사람", "실주입", "세계영향"), | 체크 기대 문자열. 누리바꿈 채택 시 테스트 기대값/호환 별칭 정책 결정 필요. |
| `tests/run_external_intent_boundary_pack_check.py` | 12 | "c02_seulgi_injection_replay": ("슬기", "실주입", "세계영향"), | 체크 기대 문자열. 누리바꿈 채택 시 테스트 기대값/호환 별칭 정책 결정 필요. |
| `tests/run_external_intent_boundary_pack_check.py` | 13 | "c03_gatekeeper_reject": ("슬기", "실주입", "세계영향"), | 체크 기대 문자열. 누리바꿈 채택 시 테스트 기대값/호환 별칭 정책 결정 필요. |
| `tests/run_external_intent_boundary_pack_check.py` | 14 | "c04_schedule_event_same_boundary": ("일정", "실주입", "세계영향"), | 체크 기대 문자열. 누리바꿈 채택 시 테스트 기대값/호환 별칭 정책 결정 필요. |
| `tests/run_external_intent_boundary_pack_check.py` | 15 | "c05_relay_event_replay": ("이어전달", "재연주입", "세계영향"), | 체크 기대 문자열. 누리바꿈 채택 시 테스트 기대값/호환 별칭 정책 결정 필요. |
| `tests/run_external_intent_boundary_pack_check.py` | 22 | REQUIRED_ACTION_KINDS = ["보기만", "세계영향"] | 체크 기대 문자열. 누리바꿈 채택 시 테스트 기대값/호환 별칭 정책 결정 필요. |
| `tests/toolchain_golden/ai_prompt_lean.txt` | 386 | \| \`world_affecting\` \| 세계영향 \| 이야기/누리 의미를 바꾸는 입력 사건. \| | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tests/toolchain_golden/ai_prompt_lean.txt` | 431 | \| \`world_affecting\` \| 세계영향 입력 \| world state 의미를 바꾸는 입력. 반드시 \`InputSnapshot\` 경유로 들어간다. \| | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tests/toolchain_golden/ai_prompt_lean.txt` | 484 | > **v23.0.5 용어 상태:** 외부개입 축은 **\`입력원천 / 주입방식 / 행동갈래\`** 3축으로 읽는다. \`입력원천\` 6값은 \`사람 / 슬기 / 밖일 / 일정 / 이어전달 / 펼침실행\`, \`주입방식\` 2값은 \`실주입 / 재연주입\`, \`행동갈래\` 2값은 \`보기만 / 세계영향\` 이다. ... | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tests/toolchain_golden/ai_prompt_lean.txt` | 702 | \| **세계영향** \| 세계 규칙/상태 전이에 영향을 주는 입력 사건 \| world_affecting \| | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tests/toolchain_golden/ai_prompt_lean.txt` | 844 | - canonical 값은 \`보기만 / 세계영향\` 이며, \`입력원천\` / \`주입방식\` 과 섞지 않는다. | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tests/toolchain_golden/ai_prompt_lean.txt` | 1368 | \| **세계영향** \| 세계 상태/규칙에 영향을 주는 action kind \| world_affecting \| | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tests/toolchain_golden/ai_prompt_lean.txt` | 9886 | "행동갈래": "세계영향", | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tests/toolchain_golden/ai_prompt_lean.txt` | 10110 | - action kind 정본은 **\`보기만 / 세계영향\`** 이다. | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |
| `tests/toolchain_golden/ai_prompt_lean.txt` | 10129 | \| \`세계영향\` \| action kind / input meaning \| AGE3 \| partial \| STUB_ONLY \| canonical world-affecting kind \| | AI prompt/golden 스냅샷 문구 영향. golden --update 금지, 후속 승인 필요. |

## 결론

- 텐서 정본화는 `lang/src/stdlib.rs` 코드가 이미 `겹차림.*`을 `텐서.*`로 canonicalize하므로 SSOT/문서/golden 정합화가 주 영향이다.
- `음/ㅁ` 제안은 현재 제품 DDN 입력에서 값참조 꼬리로 쓰인 증거가 없다. 구현상 `choose_variant`는 X/Y 토큰을 처리할 수 있지만 호출부에 `음/ㅁ` 배선은 아직 보이지 않는다.
- `세계영향`→`누리바꿈`은 `pack/external_intent_boundary_v1` 데이터와 `tests/run_external_intent_boundary_pack_check.py`의 기대 문자열에 직접 영향이 있다. golden/prompt 문서는 후속 승인 없이는 갱신하지 않는다.
- 본 작업은 조사만 수행했으며 SSOT 제안서·코드·팩은 수정하지 않았다.

## 자기 검증

- 스캔 파일 수: 22522개
- 텐서 행 수: 104
- 누리바꿈 행 수: 712
- 음/ㅁ 직접 표기 행 수: 36
- DDN 예시 파생어 후보 행 수: 20
- 수정 범위: 신규 보고서 1개
