# pack/bdl2_spec_skeleton

W22 BDL2/V2 스펙 확정용 D-PACK이다.

## 목표
- BDL2 detbin 스키마/커맨드 테이블 확정 내용과 기대치를 기록한다.
- 픽셀/스크린샷 골든은 금지한다. (`bogae_hash`, `state_hash`만 사용)

## 구성
- input.ddn: BDL2 인코더 확인용 입력(기본 커맨드만; Circle/Arc/Curve는 detbin fixture로 검증)
- expect/: detbin/manifest/해시 기대치 문서
- diag/: BDL2 파싱 오류/검증 규칙 문서
- tests/: 골든 테스트 계획 및 실행 명령 문서
