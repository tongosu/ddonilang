# 发布说明 2026-02-11 (中文)

> 历史发布说明。当前公开入口是 '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'。

## 摘要

本次发布聚焦 AGE2 Open 策略强化，以及 open.net/open.ffi/open.gpu 的最小 schema/runtime API。

## 重点

- 当 age_target < AGE2 时，'open=record|replay' 会被阻止。
- '--unsafe-open' 已作为显式绕过选项加入。
- open log schema:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- pack:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## 行为变更

open=record|replay 仅在 age_target >= AGE2 时允许；--unsafe-open 是显式绕过。

## 历史测试命令

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## 当前状态指引

当前 Seamgrim/WASM/current-line 状态请使用本文件夹的 QUICKSTART.md 和 DEV_STRUCTURE.md。
