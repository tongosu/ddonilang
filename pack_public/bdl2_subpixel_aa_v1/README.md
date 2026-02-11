# bdl2_subpixel_aa_v1

BDL2 Q24.8 + AA 플래그 최소 구현 확인 pack.

## 포함
- `--bogae-codec bdl2` 출력이 BDL2 detbin/bogae_hash로 결정적임을 확인
- 서브픽셀/AA 정책은 W22 detbin 고정 테스트로 검증(아래 참조)

## 실행
- `python tests/run_pack_golden.py bdl2_subpixel_aa_v1`
