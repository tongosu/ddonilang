# module_system_smoke_v1

Phase 1 모듈 표면(`쓰임 {}`, `드러냄 {}`, `공개 {}`)의 파서/진단 스모크 팩.

- 성공:
  - `쓰임 + 드러냄` 파싱/실행
  - `공개`(드러냄 입력 별칭) 파싱/실행
  - `모듈별명.이름` 호출(`수학.길이`) 해석/실행
- 실패:
  - `E_IMPORT_ALIAS_DUPLICATE`
  - `E_IMPORT_ALIAS_RESERVED`
  - `E_IMPORT_PATH_INVALID`
  - `E_IMPORT_VERSION_CONFLICT`
  - `E_EXPORT_BLOCK_DUPLICATE`
