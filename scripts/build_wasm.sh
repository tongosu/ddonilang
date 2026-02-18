#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

BUILD_ROOT="/mnt/i/home/urihanl/ddn/codex/build"
if [[ ! -d "${BUILD_ROOT}" ]]; then
  BUILD_ROOT="/mnt/c/ddn/codex/build"
fi
mkdir -p "${BUILD_ROOT}"

OUT_DIR="${BUILD_ROOT}/wasm/ddonirang_tool"
mkdir -p "${OUT_DIR}"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack이 없습니다. 설치: cargo install wasm-pack"
  exit 1
fi

pushd "${REPO_ROOT}/tool" >/dev/null
wasm-pack build . \
  --target web \
  --release \
  --out-dir "${OUT_DIR}" \
  --out-name ddonirang_tool \
  -- --features wasm
popd >/dev/null

if [[ -f "${REPO_ROOT}/scripts/copy_wasm_tool_to_ui.ps1" ]]; then
  echo "WASM 산출물 복사(Windows PowerShell 스크립트)는 필요 시 build_wasm_tool.ps1을 사용하세요."
fi

echo "WASM build complete: ${OUT_DIR}"
ls -lh "${OUT_DIR}/ddonirang_tool_bg.wasm"
