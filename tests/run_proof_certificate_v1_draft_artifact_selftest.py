#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/proof_certificate_v1_draft_artifact/README.md")
PACK_README = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/README.md")
PACK_MANIFEST = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/draft_pack.detjson")
PROMOTION_README = Path("tests/proof_certificate_v1_promotion_candidate/README.md")
PACK_SHARED = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/shared.proof_certificate_candidate_shell.detjson")
PACK_CLEAN_STATE = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/clean.proof_certificate_candidate_state.detjson")
PACK_ABORT_STATE = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/abort.proof_certificate_candidate_state.detjson")
PACK_CLEAN_CANDIDATE = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/clean.proof_certificate_candidate.detjson")
PACK_ABORT_CANDIDATE = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/abort.proof_certificate_candidate.detjson")
FIXTURE_CLEAN = Path("tests/proof_certificate_v1_draft_artifact/fixtures/clean.proof_certificate_v1_draft_artifact.detjson")
FIXTURE_ABORT = Path("tests/proof_certificate_v1_draft_artifact/fixtures/abort.proof_certificate_v1_draft_artifact.detjson")

README_SNIPPETS = (
    "## Stable Contract",
    "`pack/age4_proof_certificate_v1_draft_candidate_v1/README.md`",
    "`pack/age4_proof_certificate_v1_draft_candidate_v1/draft_pack.detjson`",
    "`tests/proof_certificate_v1_draft_artifact/fixtures/clean.proof_certificate_v1_draft_artifact.detjson`",
    "`tests/proof_certificate_v1_draft_artifact/fixtures/abort.proof_certificate_v1_draft_artifact.detjson`",
    "`python tests/run_proof_certificate_v1_draft_artifact_selftest.py`",
    "`proof_certificate_v1_draft_artifact_selftest`",
    "`schema`",
    "`source_pack_schema`",
    "`source_pack_id`",
    "`profile`",
    "`candidate_manifest`",
    "`shared_shell`",
    "`state_delta`",
)
POINTERS = (
    "`tests/proof_certificate_v1_draft_artifact/README.md`",
    "`python tests/run_proof_certificate_v1_draft_artifact_selftest.py`",
)
EXPECTED_KEYS = (
    "schema",
    "source_pack_schema",
    "source_pack_id",
    "profile",
    "shared_shell_key_count",
    "state_delta_key_count",
    "candidate_manifest",
    "shared_shell",
    "state_delta",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-v1-draft-artifact-selftest] fail: {message}")
    return 1


def load_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def build_candidate(shell: dict, state: dict) -> dict:
    out = dict(shell)
    out.update(state)
    return out


def validate_fixture(
    fixture_path: Path,
    *,
    profile: str,
    pack_candidate_path: Path,
    pack_state_path: Path,
    shared_shell: dict,
) -> None:
    fixture = load_json(fixture_path)
    if not isinstance(fixture, dict):
        raise ValueError(f"{fixture_path}: root should be object")
    if sorted(fixture.keys()) != sorted(EXPECTED_KEYS):
        raise ValueError(f"{fixture_path}: keys mismatch")
    if str(fixture.get("schema", "")).strip() != "ddn.proof_certificate_v1_draft_artifact.v1":
        raise ValueError(f"{fixture_path}: schema mismatch")
    if str(fixture.get("source_pack_schema", "")).strip() != "ddn.proof_certificate_v1_draft_pack.v1":
        raise ValueError(f"{fixture_path}: source_pack_schema mismatch")
    if str(fixture.get("source_pack_id", "")).strip() != "pack/age4_proof_certificate_v1_draft_candidate_v1":
        raise ValueError(f"{fixture_path}: source_pack_id mismatch")
    if str(fixture.get("profile", "")).strip() != profile:
        raise ValueError(f"{fixture_path}: profile mismatch")
    if int(fixture.get("shared_shell_key_count", -1)) != 7:
        raise ValueError(f"{fixture_path}: shared_shell_key_count mismatch")
    if int(fixture.get("state_delta_key_count", -1)) != 6:
        raise ValueError(f"{fixture_path}: state_delta_key_count mismatch")

    pack_candidate = load_json(pack_candidate_path)
    pack_state = load_json(pack_state_path)
    if fixture.get("candidate_manifest") != pack_candidate:
        raise ValueError(f"{fixture_path}: candidate_manifest mismatch")
    if fixture.get("shared_shell") != shared_shell:
        raise ValueError(f"{fixture_path}: shared_shell mismatch")
    if fixture.get("state_delta") != pack_state:
        raise ValueError(f"{fixture_path}: state_delta mismatch")
    if build_candidate(shared_shell, pack_state) != pack_candidate:
        raise ValueError(f"{fixture_path}: reconstruction mismatch")


def main() -> int:
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")

    try:
        ensure_pointers(PACK_README)
        ensure_pointers(PROMOTION_README)
        pack = load_json(PACK_MANIFEST)
        if not isinstance(pack, dict):
            raise ValueError("pack manifest should be object")
        profiles = pack.get("profiles")
        if not isinstance(profiles, list) or len(profiles) != 2:
            raise ValueError("pack profile count mismatch")

        shared_shell = load_json(PACK_SHARED)
        if not isinstance(shared_shell, dict):
            raise ValueError("pack shared shell should be object")

        validate_fixture(
            FIXTURE_CLEAN,
            profile="clean",
            pack_candidate_path=PACK_CLEAN_CANDIDATE,
            pack_state_path=PACK_CLEAN_STATE,
            shared_shell=shared_shell,
        )
        validate_fixture(
            FIXTURE_ABORT,
            profile="abort",
            pack_candidate_path=PACK_ABORT_CANDIDATE,
            pack_state_path=PACK_ABORT_STATE,
            shared_shell=shared_shell,
        )
    except ValueError as exc:
        return fail(str(exc))

    print("[proof-certificate-v1-draft-artifact-selftest] ok profiles=2 fields=9")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
