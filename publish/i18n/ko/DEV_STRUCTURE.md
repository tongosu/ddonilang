# 개발 구조 (한국어)

> 한국어 기준 공개 문서 묶음입니다.

이 문서는 공개용 다국어 요약입니다. 자세한 기준 문서는 '../../DDONIRANG_DEV_STRUCTURE.md'입니다.

## 핵심 레이어

| 레이어 | 경로 | 역할 |
| --- | --- | --- |
| core | 'core/' | 결정성 엔진 코어 |
| lang | 'lang/' | 문법, 파서, 정본화 |
| tool | 'tool/' | 런타임/도구 구현 |
| CLI | 'tools/teul-cli/' | CLI 실행과 검증 |
| packs | 'pack/' | 실행 가능한 pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | 웹 작업실과 보개 |
| tests | 'tests/' | 통합/제품 테스트 |
| publish | 'publish/' | 공개 문서 |

## 셈그림 작업실 V2

- 'ui/index.html': 단일 진입점
- 'ui/screens/run.js': 실행 화면과 current-line 실행
- 'ui/components/bogae.js': console/graph/space2d/grid 보개 렌더링
- 'ui/seamgrim_runtime_state.js': 마디, 런타임 상태, 거울 요약
- 'tools/ddn_exec_server.py': 로컬 정적 서버와 보조 API

## 런타임 원칙

- DDN 런타임, pack, state hash, 거울/replay 기록이 truth를 소유합니다.
- 보개는 보기 계층이며 runtime truth를 소유하지 않습니다.
- Python/JS는 orchestration과 UI를 맡을 수 있지만 언어 의미를 test-only lowering으로 대신하면 안 됩니다.

## 현재 evidence

- CLI/WASM runtime parity
- 4권 raw current-line bundle parity
- 셈그림 제품 smoke
- 보개 마디/그래프 UI check
