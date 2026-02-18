# pack/bogae_asset_manifest_v1

AssetManifestV1 최소 스모크.

- 목표:
  - asset_id=blake3(bytes) 규칙을 검증한다.
  - manifest 정본(해시) 출력을 고정한다.

## 입력
- `assets/` 디렉터리 안의 파일

## 골든
- `golden.jsonl`