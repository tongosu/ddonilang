#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/README.md")
PACK_PATH = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/draft_pack.detjson")
PROMOTION_README = Path("tests/proof_certificate_v1_promotion_candidate/README.md")
CANDIDATE_README = Path("tests/proof_certificate_candidate_manifest/README.md")
LAYERS_README = Path("tests/proof_certificate_candidate_layers/README.md")
TEST_CLEAN_CANDIDATE = Path("tests/proof_certificate_candidate_manifest/fixtures/clean.proof_certificate_candidate.detjson")
TEST_ABORT_CANDIDATE = Path("tests/proof_certificate_candidate_manifest/fixtures/abort.proof_certificate_candidate.detjson")
TEST_SHARED_SHELL = Path("tests/proof_certificate_candidate_layers/fixtures/shared.proof_certificate_candidate_shell.detjson")
TEST_CLEAN_STATE = Path("tests/proof_certificate_candidate_layers/fixtures/clean.proof_certificate_candidate_state.detjson")
TEST_ABORT_STATE = Path("tests/proof_certificate_candidate_layers/fixtures/abort.proof_certificate_candidate_state.detjson")
PACK_SHARED_SHELL = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/shared.proof_certificate_candidate_shell.detjson")
PACK_CLEAN_STATE = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/clean.proof_certificate_candidate_state.detjson")
PACK_ABORT_STATE = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/abort.proof_certificate_candidate_state.detjson")
PACK_CLEAN_CANDIDATE = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/clean.proof_certificate_candidate.detjson")
PACK_ABORT_CANDIDATE = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/abort.proof_certificate_candidate.detjson")

README_SNIPPETS = (
    "Pack ID: `pack/age4_proof_certificate_v1_draft_candidate_v1`",
    "`proof_certificate_v1` 정본화 직전의 승격 후보를 self-contained bundle로 고정한다.",
    "`draft_pack.detjson`",
    "`fixtures/shared.proof_certificate_candidate_shell.detjson`",
    "`fixtures/clean.proof_certificate_candidate_state.detjson`",
    "`fixtures/abort.proof_certificate_candidate_state.detjson`",
    "`fixtures/clean.proof_certificate_candidate.detjson`",
    "`fixtures/abort.proof_certificate_candidate.detjson`",
    "`tests/proof_certificate_v1_promotion_candidate/README.md`",
    "`python tests/run_proof_certificate_v1_promotion_candidate_selftest.py`",
)
POINTERS = (
    "`pack/age4_proof_certificate_v1_draft_candidate_v1/README.md`",
    "`python tests/run_proof_certificate_v1_draft_pack_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-v1-draft-pack-selftest] fail: {message}")
    return 1


def load_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def build_candidate(shell: dict, state: dict) -> dict:
    merged = dict(shell)
    merged.update(state)
    return merged


def main() -> int:
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")

    try:
        ensure_pointers(PROMOTION_README)
        ensure_pointers(CANDIDATE_README)
        ensure_pointers(LAYERS_README)

        pack = load_json(PACK_PATH)
        if not isinstance(pack, dict):
            raise ValueError("pack root should be object")
        if str(pack.get("schema", "")).strip() != "ddn.proof_certificate_v1_draft_pack.v1":
            raise ValueError("pack schema mismatch")
        if str(pack.get("artifact_kind", "")).strip() != "proof_certificate_v1_promotion_candidate":
            raise ValueError("artifact_kind mismatch")
        if str(pack.get("shared_shell", "")).strip() != "fixtures/shared.proof_certificate_candidate_shell.detjson":
            raise ValueError("shared_shell path mismatch")

        profiles = pack.get("profiles")
        if not isinstance(profiles, list) or len(profiles) != 2:
            raise ValueError("profiles count mismatch")
        expected_profiles = {
            "clean": (
                "fixtures/clean.proof_certificate_candidate.detjson",
                "fixtures/clean.proof_certificate_candidate_state.detjson",
            ),
            "abort": (
                "fixtures/abort.proof_certificate_candidate.detjson",
                "fixtures/abort.proof_certificate_candidate_state.detjson",
            ),
        }
        for row in profiles:
            if not isinstance(row, dict):
                raise ValueError("profile row should be object")
            name = str(row.get("name", "")).strip()
            if name not in expected_profiles:
                raise ValueError(f"unexpected profile name: {name}")
            expected_candidate, expected_state = expected_profiles[name]
            if str(row.get("candidate", "")).strip() != expected_candidate:
                raise ValueError(f"{name} candidate path mismatch")
            if str(row.get("state_delta", "")).strip() != expected_state:
                raise ValueError(f"{name} state_delta path mismatch")

        pack_shared = load_json(PACK_SHARED_SHELL)
        pack_clean_state = load_json(PACK_CLEAN_STATE)
        pack_abort_state = load_json(PACK_ABORT_STATE)
        pack_clean_candidate = load_json(PACK_CLEAN_CANDIDATE)
        pack_abort_candidate = load_json(PACK_ABORT_CANDIDATE)
        test_shared = load_json(TEST_SHARED_SHELL)
        test_clean_state = load_json(TEST_CLEAN_STATE)
        test_abort_state = load_json(TEST_ABORT_STATE)
        test_clean_candidate = load_json(TEST_CLEAN_CANDIDATE)
        test_abort_candidate = load_json(TEST_ABORT_CANDIDATE)

        if pack_shared != test_shared:
            raise ValueError("pack shared shell mismatch")
        if pack_clean_state != test_clean_state:
            raise ValueError("pack clean state mismatch")
        if pack_abort_state != test_abort_state:
            raise ValueError("pack abort state mismatch")
        if pack_clean_candidate != test_clean_candidate:
            raise ValueError("pack clean candidate mismatch")
        if pack_abort_candidate != test_abort_candidate:
            raise ValueError("pack abort candidate mismatch")

        if build_candidate(pack_shared, pack_clean_state) != pack_clean_candidate:
            raise ValueError("pack clean reconstruction mismatch")
        if build_candidate(pack_shared, pack_abort_state) != pack_abort_candidate:
            raise ValueError("pack abort reconstruction mismatch")
    except ValueError as exc:
        return fail(str(exc))

    print("[proof-certificate-v1-draft-pack-selftest] ok profiles=2 fixtures=5")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
