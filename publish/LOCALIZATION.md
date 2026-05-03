# 공개 문서 다국어 운영 기준

## 현재 범위

현재 `publish/`의 다국어 공개 문서는 README 계열을 기준으로 둔다.

| 언어/표기 | 파일 | 상태 |
| --- | --- | --- |
| 한국어 | `README.md` | 기준 문서 |
| 영어 | `README_en.md` | 현행화 |
| 아이마라어 | `README_ay.md` | starter 현행화 |
| 바스크어 | `README_eu.md` | starter 현행화 |
| 일본어 | `README_ja.md` | 현행화 |
| 칸나다어 | `README_kn.md` | starter 현행화 |
| 몽골어 | `README_mn.md` | 현행화 |
| 네팔어 | `README_ne.md` | starter 현행화 |
| 케추아어 | `README_qu.md` | starter 현행화 |
| sym3 | `README_sym3.md` | compact 현행화 |
| 타밀어 | `README_ta.md` | starter 현행화 |
| 텔루구어 | `README_te.md` | starter 현행화 |
| 터키어 | `README_tr.md` | 현행화 |

## 기준 문서

- 빠른 시작: `QUICKSTART.md`
- 개발 구조: `DDONIRANG_DEV_STRUCTURE.md`
- 다운로드: `DOWNLOADS.md`
- 문법 요약: `ddonirang_grammar_summary.md`
- 문법 총정리: `ddonirang_grammar_full.md`

이 문서들은 당분간 언어별 복제본을 만들지 않는다. 기능/명령/검증 경로가 자주 바뀌는 문서라서 여러 언어 복제본을 두면 쉽게 서로 어긋난다.

## 다국어 확장 원칙

- README는 각 언어 사용자에게 프로젝트가 무엇인지 알려주는 입구다.
- 실행 방법, 검증 명령, 개발 구조는 기준 문서 하나를 유지하고 README에서 연결한다.
- 언어/runtime 의미는 번역 문서가 소유하지 않는다. 실행 의미는 DDN runtime, pack/checker, mirror/replay truth가 소유한다.
- 다국어 이름/설명은 장기적으로 package/export symbol sidecar artifact에서 관리한다.
- 자동 번역 초안은 starter로 표시하고, native review가 끝난 문서만 완성 번역으로 본다.

## 폴더 구조 판단

지금은 `publish/README_<lang>.md` 파일명을 유지한다. GitHub 첫 화면과 기존 링크가 단순하고, 문서 수가 아직 작기 때문이다.

다음 조건 중 하나가 생기면 `publish/i18n/<lang>/README.md` 구조로 옮긴다.

- README 외 문서를 언어별로 3종 이상 유지해야 한다.
- 언어별 이미지/예제/튜토리얼이 생긴다.
- native review 상태, 번역 출처, 용어표를 언어별로 따로 관리해야 한다.

그 전까지는 루트 `publish/` 구조를 유지하고, `LOCALIZATION.md`가 다국어 운영 기준을 맡는다.
