# age1_patch_approval

## 목적
- patch preview/approve/apply/verify 흐름을 간단히 확인한다.

## 사용
```bash
cd pack/age1_patch_approval
teul-cli patch preview ddn.patch.json
teul-cli patch approve ddn.patch.json --out build/ddn.patch.approval.json --yes
teul-cli patch apply ddn.patch.json --approval build/ddn.patch.approval.json --out build/patched
teul-cli patch verify ddn.patch.json --approval build/ddn.patch.approval.json --tests ../../tools/teul-cli/tests/golden --walk 90
```

## 참고
- patch 파일은 현재 디렉터리 기준 상대 경로를 사용한다.
