#!/usr/bin/env python
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path


INT_RE = set("0123456789-")


def fail(code: str, msg: str) -> int:
    print(f"[tensor-v0-pack-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> object:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise ValueError(f"E_TENSOR_V0_FILE_MISSING::{path}")
    except Exception as exc:
        raise ValueError(f"E_TENSOR_V0_JSON_INVALID::{path} ({exc})")


def is_i64_raw_string(value: object) -> bool:
    if not isinstance(value, str):
        return False
    text = value.strip()
    if not text:
        return False
    if any(ch not in INT_RE for ch in text):
        return False
    if text in {"-", "+"}:
        return False
    try:
        int(text, 10)
    except Exception:
        return False
    return True


def canonical_json_text(data: object) -> str:
    return json.dumps(data, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def hash_sha256_canonical(data: object) -> str:
    payload = canonical_json_text(data).encode("utf-8")
    return f"sha256:{hashlib.sha256(payload).hexdigest()}"


def lex_lt(left: list[int], right: list[int]) -> bool:
    return tuple(left) < tuple(right)


def validate_shape(shape: object) -> tuple[bool, str]:
    if not isinstance(shape, list):
        return False, "E_TENSOR_V0_SHAPE"
    if len(shape) == 0:
        return False, "E_TENSOR_V0_SHAPE"
    for dim in shape:
        if not isinstance(dim, int):
            return False, "E_TENSOR_V0_SHAPE"
        if dim < 0:
            return False, "E_TENSOR_V0_SHAPE"
    return True, "OK"


def validate_tensor_v0(doc: object) -> tuple[bool, str]:
    if not isinstance(doc, dict):
        return False, "E_TENSOR_V0_ROOT_TYPE"
    if str(doc.get("schema", "")).strip() != "tensor.v0":
        return False, "E_TENSOR_V0_SCHEMA"
    kind = str(doc.get("kind", "")).strip()
    if kind not in {"dense", "sparse"}:
        return False, "E_TENSOR_V0_KIND"
    if str(doc.get("dtype", "")).strip() != "fixed64":
        return False, "E_TENSOR_V0_DTYPE"

    shape = doc.get("shape")
    ok_shape, shape_code = validate_shape(shape)
    if not ok_shape:
        return False, shape_code
    assert isinstance(shape, list)
    rank = len(shape)

    if kind == "dense":
        data = doc.get("data")
        if not isinstance(data, list):
            return False, "E_TENSOR_V0_DENSE_DATA_TYPE"
        expected_len = 1
        for dim in shape:
            expected_len *= int(dim)
        if len(data) != expected_len:
            return False, "E_TENSOR_V0_DENSE_DATA_LEN"
        for value in data:
            if not is_i64_raw_string(value):
                return False, "E_TENSOR_V0_DENSE_VALUE_TYPE"
        return True, "OK"

    items = doc.get("items")
    if not isinstance(items, list):
        return False, "E_TENSOR_V0_SPARSE_ITEMS_TYPE"
    prev_index: list[int] | None = None
    for row in items:
        if not isinstance(row, list) or len(row) != 2:
            return False, "E_TENSOR_V0_SPARSE_ITEM_TYPE"
        index, value = row
        if not isinstance(index, list):
            return False, "E_TENSOR_V0_SPARSE_INDEX_TYPE"
        if len(index) != rank:
            return False, "E_TENSOR_V0_SPARSE_INDEX_RANK"
        parsed_index: list[int] = []
        for axis, idx in enumerate(index):
            if not isinstance(idx, int):
                return False, "E_TENSOR_V0_SPARSE_INDEX_TYPE"
            if idx < 0 or idx >= int(shape[axis]):
                return False, "E_TENSOR_V0_SPARSE_INDEX_OOB"
            parsed_index.append(idx)
        if not is_i64_raw_string(value):
            return False, "E_TENSOR_V0_SPARSE_VALUE_TYPE"
        if prev_index is not None:
            if parsed_index == prev_index:
                return False, "E_TENSOR_V0_SPARSE_DUP"
            if not lex_lt(prev_index, parsed_index):
                return False, "E_TENSOR_V0_SPARSE_ORDER"
        prev_index = parsed_index
    return True, "OK"


def run_pack(repo_root: Path, pack_path: Path) -> tuple[bool, str, int]:
    golden_path = pack_path / "golden.jsonl"
    if not golden_path.exists():
        return False, "E_TENSOR_V0_GOLDEN_MISSING", 0
    lines = golden_path.read_text(encoding="utf-8").splitlines()
    total_cases = 0
    for line_no, raw in enumerate(lines, 1):
        text = raw.strip()
        if not text:
            continue
        total_cases += 1
        try:
            row = json.loads(text)
        except Exception as exc:
            return False, f"E_TENSOR_V0_GOLDEN_ROW_INVALID::{pack_path}:{line_no} ({exc})", total_cases
        if not isinstance(row, dict):
            return False, f"E_TENSOR_V0_GOLDEN_ROW_INVALID::{pack_path}:{line_no}", total_cases
        rel_case = str(row.get("case", "")).strip()
        if not rel_case:
            return False, f"E_TENSOR_V0_GOLDEN_ROW_CASE_MISSING::{pack_path}:{line_no}", total_cases
        case_path = (pack_path / rel_case).resolve()
        if not case_path.exists():
            return False, f"E_TENSOR_V0_FILE_MISSING::{case_path}", total_cases
        try:
            case_doc = load_json(case_path)
        except ValueError as exc:
            return False, str(exc), total_cases
        ok, code = validate_tensor_v0(case_doc)
        expect_ok = bool(row.get("expect_ok", True))
        if expect_ok:
            if not ok:
                return (
                    False,
                    f"E_TENSOR_V0_EXPECT_PASS_FAILED::{pack_path}:{line_no} case={rel_case} code={code}",
                    total_cases,
                )
            expected_hash = str(row.get("expected_hash", "")).strip()
            if not expected_hash:
                return False, f"E_TENSOR_V0_GOLDEN_HASH_MISSING::{pack_path}:{line_no}", total_cases
            actual_hash = hash_sha256_canonical(case_doc)
            if actual_hash != expected_hash:
                return (
                    False,
                    "E_TENSOR_V0_HASH_MISMATCH::"
                    f"{pack_path}:{line_no} case={rel_case} expected={expected_hash} actual={actual_hash}",
                    total_cases,
                )
        else:
            if ok:
                return (
                    False,
                    f"E_TENSOR_V0_EXPECT_FAIL_MISSED::{pack_path}:{line_no} case={rel_case}",
                    total_cases,
                )
            expected_error = str(row.get("expected_error", "")).strip()
            if not expected_error:
                return False, f"E_TENSOR_V0_GOLDEN_ERROR_MISSING::{pack_path}:{line_no}", total_cases
            if code != expected_error:
                return (
                    False,
                    "E_TENSOR_V0_EXPECT_CODE_MISMATCH::"
                    f"{pack_path}:{line_no} case={rel_case} expected={expected_error} actual={code}",
                    total_cases,
                )
    if total_cases == 0:
        return False, f"E_TENSOR_V0_GOLDEN_EMPTY::{pack_path}", total_cases
    return True, "OK", total_cases


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate tensor.v0 dense/sparse pack contracts")
    parser.add_argument(
        "--packs",
        nargs="*",
        default=["pack/tensor_v0_dense", "pack/tensor_v0_sparse"],
        help="tensor v0 pack paths",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    pack_paths = [(repo_root / Path(path)).resolve() for path in args.packs]
    missing = [str(path).replace("\\", "/") for path in pack_paths if not path.exists()]
    if missing:
        return fail("E_TENSOR_V0_PACK_MISSING", ",".join(missing))

    total_cases = 0
    for pack_path in pack_paths:
        ok, detail, case_count = run_pack(repo_root, pack_path)
        total_cases += case_count
        if not ok:
            code, msg = detail.split("::", 1) if "::" in detail else ("E_TENSOR_V0_PACK_INVALID", detail)
            return fail(code, msg)

    print("[tensor-v0-pack-check] ok")
    print("packs=" + ",".join(str(path).replace("\\", "/") for path in pack_paths))
    print(f"cases={total_cases}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
