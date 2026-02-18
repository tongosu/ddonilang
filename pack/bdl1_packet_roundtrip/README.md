# pack/bdl1_packet_roundtrip

W20 BDL1 패킷 roundtrip 검증용 D-PACK이다.

## 목표
- BDL1 detbin을 패킷으로 감싸고 다시 복원했을 때 payload_hash가 동일하다.
- 손상된 패킷은 결정적으로 실패한다.

## 실행 예시
```bash
teul-cli bdl packet wrap input.bdl1.detbin --out out/sample.packet
teul-cli bdl packet unwrap out/sample.packet --out out/unwrapped.bdl1.detbin
```
