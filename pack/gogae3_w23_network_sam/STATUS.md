# W23: 네트워크 샘 - 구현 상태

**최종 업데이트**: 2026-01-18 20:30
**담당**: Codex
**리뷰어**: Claude (설계), Gemini (검증)

---

## 진행 상황

- [x] README.md 작성
- [x] input.ddn 템플릿 작성
- [x] expect/README.md 가이드 추가
- [ ] 실제 데모 시나리오 구현 (악수하는 솜씨)
- [ ] inputs/ 네트워크 입력샘 샘플 생성
- [ ] expect/state_hashes.json 기대값 생성
- [ ] tests/ 골든 테스트 작성
- [ ] Rust 런타임: InputSnapshot 네트워크 채널 구현
- [ ] Rust 런타임: Sync Master 구현

**완료율**: 33% (3/9)

---

## DoD (완료 정의)

- [ ] 지연 상황에서도 모든 클라이언트의 `state_hash`가 100% 일치
- [ ] 데모: "멀리서 악수하는 솜씨" (서울/부산) 동작
- [ ] 리플레이 시 네트워크 입력샘 재주입으로 동일 결과

---

## 블로커 (막힌 부분)

### 1. ❌ **InputSnapshot 네트워크 채널 스키마 미확정**
- **문제**: SSOT_PLATFORM에 `sender/seq/order_key` 필드 명세 없음
- **영향**: 입력샘 샘플 생성 불가, Rust 구조체 정의 불가
- **담당**: ChatGPT (SSOT 업데이트 필요)
- **참조**: PROPOSAL_GOGAE3_PLAN_v20.1.10.md 1.5절
- **기한**: 2026-01-20

### 2. ⚠️ **안정 정렬 규칙 구체화 필요**
- **문제**: (마디, 발신자, seq) 우선순위 명확히 필요
- **영향**: Sync Master 정렬 로직 구현 불가
- **담당**: Claude (SPEC.md 분석 후 제안)
- **기한**: 2026-01-19

### 3. ⚠️ **중복/유실 정책 미정**
- **문제**: 동일 seq 재전송 시 처리 방법 불명확
- **영향**: 결정성 보장 불가
- **담당**: ChatGPT (SSOT 또는 SPEC 명확화)
- **기한**: 2026-01-21

---

## 다음 작업

### ChatGPT
1. SSOT_PLATFORM에 InputSnapshot 네트워크 채널 스키마 추가
2. 중복/유실 정책 결정성 규칙 명시

### Claude
1. SPEC.md 기반 안정 정렬 규칙 명확화
2. Sync Master 설계안 작성

### Codex
1. 블로커 해결 후 실제 데모 시나리오 구현
2. inputs/ 샘플 생성 (지연/순서뒤섞임 포함)
3. Rust InputSnapshot 구조체 구현

### Gemini
1. (대기) 구현 완료 후 리플레이 검증

---

## 참고 문서

- [SPEC.md](../../docs/ssot/walks/gogae3/w23_network_sam/SPEC.md)
- [IMPL_GUIDE.md](../../docs/ssot/walks/gogae3/w23_network_sam/IMPL_GUIDE.md)
- [PROPOSAL v20.1.10](../../docs/context/roadmap/gogae3/PROPOSAL_GOGAE3_PLAN_v20.1.10.md)

---

## 변경 이력

- 2026-01-18 20:30: STATUS.md 생성 (Claude)
- 2026-01-18 03:14: 팩 디렉토리 생성 (자동)
- 2026-01-18 20:12: 템플릿 통합 (Claude)

