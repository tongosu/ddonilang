# gogae9_w98_release_gate

- 상태: 제품 release gate 검증 팩 (SSOT v20.3.1)
- 기준: `docs/ssot/walks/gogae9/w98_release_v14/README.md`
- Pack ID: `pack/gogae9_w98_release_gate`

## 범위
- `tools/release/gogae9_release_gate.py`가 W89~W99 고개9 suite를 실행한다.
- 산출물은 `pack_results.detjson`, `release_manifest.detjson`, `pack_results.cert.json`이다.
- release manifest에는 SSOT reference, workspace bundle hash, pack results hash, cert proof가 포함된다.
