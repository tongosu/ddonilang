# 공개 문서 다국어 운영 기준

## 현재 범위

현재 `publish/`의 다국어 공개 문서는 `publish/i18n/<lang>/` 묶음을 기준으로 둔다. 기존 `publish/README_<lang>.md` 파일은 기존 링크 호환을 위해 유지한다.

| 언어/표기 | 파일 | 상태 |
| --- | --- | --- |
| 한국어 | `i18n/ko/` | 기준 문서 묶음 |
| 영어 | `i18n/en/` | 현행화 |
| 일본어 | `i18n/ja/` | starter 현행화 |
| 터키어 | `i18n/tr/` | starter 현행화 |
| 몽골어 | `i18n/mn/` | starter 현행화 |
| 아이마라어 | `i18n/ay/` | starter 현행화 |
| 바스크어 | `i18n/eu/` | starter 현행화 |
| 칸나다어 | `i18n/kn/` | starter 현행화 |
| 네팔어 | `i18n/ne/` | starter 현행화 |
| 케추아어 | `i18n/qu/` | starter 현행화 |
| sym3 | `i18n/sym3/` | compact 현행화 |
| 타밀어 | `i18n/ta/` | starter 현행화 |
| 텔루구어 | `i18n/te/` | starter 현행화 |
| 중국어 | `i18n/zh/` | starter 신규 |
| 스페인어 | `i18n/es/` | starter 신규 |
| 프랑스어 | `i18n/fr/` | starter 신규 |
| 독일어 | `i18n/de/` | starter 신규 |

## 기준 문서

- 빠른 시작: `QUICKSTART.md`, 다국어 starter: `i18n/<lang>/QUICKSTART.md`
- 개발 구조: `DDONIRANG_DEV_STRUCTURE.md`, 다국어 starter: `i18n/<lang>/DEV_STRUCTURE.md`
- 다운로드: `DOWNLOADS.md`, 다국어 starter: `i18n/<lang>/DOWNLOADS.md`
- 릴리즈 노트: `RELEASE_NOTES_20260211.md`, 다국어 starter: `i18n/<lang>/RELEASE_NOTES_20260211.md`
- 문법 요약: `ddonirang_grammar_summary.md`
- 문법 총정리: `ddonirang_grammar_full.md`

문법 요약/총정리는 아직 기준 문서 하나만 유지한다. 기능/명령/검증 경로가 자주 바뀌는 문서는 `tools/scripts/generate_publish_i18n_docs.ps1`로 starter 묶음을 재생성한다.

## 다국어 확장 원칙

- README는 각 언어 사용자에게 프로젝트가 무엇인지 알려주는 입구다.
- quickstart/dev structure/downloads/release notes는 `i18n/<lang>/`에 starter 묶음을 둔다.
- 언어/runtime 의미는 번역 문서가 소유하지 않는다. 실행 의미는 DDN runtime, pack/checker, mirror/replay truth가 소유한다.
- 다국어 이름/설명은 장기적으로 package/export symbol sidecar artifact에서 관리한다.
- 자동 번역 초안은 starter로 표시하고, native review가 끝난 문서만 완성 번역으로 본다.

## 폴더 구조 판단

지금은 두 구조를 함께 유지한다.

- 기존 링크 호환: `publish/README_<lang>.md`
- 새 다국어 문서 묶음: `publish/i18n/<lang>/`

새 언어는 우선 `i18n/<lang>/`에만 추가한다. native review가 끝나고 GitHub 루트 링크가 필요할 때 `README_<lang>.md`를 추가한다.
