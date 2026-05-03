# リリースノート 2026-02-11 (日本語)

> 過去のリリースノートです。現在の公開入口は '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md' です。

## 概要

このリリースは AGE2 Open ポリシー強化と open.net/open.ffi/open.gpu の最小 schema/runtime API に焦点を当てました。

## 主な変更

- age_target < AGE2 では 'open=record|replay' がブロックされます。
- '--unsafe-open' が明示的な迂回として追加されました。
- open log schema:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- pack:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## 動作変更

open=record|replay は age_target >= AGE2 の場合のみ許可されます。--unsafe-open で明示的に迂回します。

## 当時のテストコマンド

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## 現在状態への案内

現在の 셈그림/WASM/current-line 状態は、このフォルダの QUICKSTART.md と DEV_STRUCTURE.md を見てください。
