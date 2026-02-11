# gogae5_w55_smart_errand_integration

통합 DoD: run → geoul → replay verify → dataset export.

## 실행
- `teul-cli run pack/gogae5_w55_smart_errand_integration/input.ddn --madi 2 --geoul-out build/geoul_w55`
- `teul-cli replay verify --geoul build/geoul_w55`
- `teul-cli dataset export --geoul build/geoul_w55 --format nurigym_v0 --out build/dataset_w55`
