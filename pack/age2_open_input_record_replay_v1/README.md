# age2_open_input_record_replay_v1

- 목적: `open.input` record/replay로 키보드 입력 스냅샷이 틱별로 재주입되고 `state_hash`가 재현되는지 고정한다.
- 범위:
  - `실행정책 { 열림허용: 입력. }.` 정규화
  - 런타임 단위 테스트에서 `open.input` record/replay roundtrip
  - pack에서는 `--open replay`만으로 동일한 틱 상태 재현
- 기대:
  - tick 0/1/2의 `state_hash`가 replay에서 고정된다.
  - `점수 보여주기` 출력이 `10`, `11`, `12` 순서로 유지된다.

## 수동 갱신

```powershell
python tests/run_pack_golden.py --update age2_open_input_record_replay_v1
```
