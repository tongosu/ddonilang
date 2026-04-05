#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/proof_certificate_v1_draft_artifact_layers/README.md")
ARTIFACT_README = Path("tests/proof_certificate_v1_draft_artifact/README.md")
PROMOTION_README = Path("tests/proof_certificate_v1_promotion_candidate/README.md")
HEADER_FIXTURE = Path("tests/proof_certificate_v1_draft_artifact_layers/fixtures/shared.proof_certificate_v1_draft_artifact_header.detjson")
CLEAN_PROFILE_FIXTURE = Path("tests/proof_certificate_v1_draft_artifact_layers/fixtures/clean.proof_certificate_v1_draft_artifact_profile.detjson")
ABORT_PROFILE_FIXTURE = Path("tests/proof_certificate_v1_draft_artifact_layers/fixtures/abort.proof_certificate_v1_draft_artifact_profile.detjson")
CLEAN_ARTIFACT = Path("tests/proof_certificate_v1_draft_artifact/fixtures/clean.proof_certificate_v1_draft_artifact.detjson")
ABORT_ARTIFACT = Path("tests/proof_certificate_v1_draft_artifact/fixtures/abort.proof_certificate_v1_draft_artifact.detjson")

README_SNIPPETS = (
    "## Stable Contract",
    "`tests/proof_certificate_v1_draft_artifact/fixtures/clean.proof_certificate_v1_draft_artifact.detjson`",
    "`tests/proof_certificate_v1_draft_artifact/fixtures/abort.proof_certificate_v1_draft_artifact.detjson`",
    "`tests/proof_certificate_v1_draft_artifact_layers/fixtures/shared.proof_certificate_v1_draft_artifact_header.detjson`",
    "`tests/proof_certificate_v1_draft_artifact_layers/fixtures/clean.proof_certificate_v1_draft_artifact_profile.detjson`",
    "`tests/proof_certificate_v1_draft_artifact_layers/fixtures/abort.proof_certificate_v1_draft_artifact_profile.detjson`",
    "`python tests/run_proof_certificate_v1_draft_artifact_layers_selftest.py`",
    "`proof_certificate_v1_draft_artifact_layers_selftest`",
)
POINTERS = (
    "`tests/proof_certificate_v1_draft_artifact_layers/README.md`",
    "`python tests/run_proof_certificate_v1_draft_artifact_layers_selftest.py`",
)
HEADER_KEYS = ("schema", "source_pack_schema", "source_pack_id")
PROFILE_KEYS = (
    "profile",
    "shared_shell_key_count",
    "state_delta_key_count",
    "candidate_manifest",
    "shared_shell",
    "state_delta",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-v1-draft-artifact-layers-selftest] fail: {message}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def build_artifact(header: dict, profile: dict) -> dict:
    merged = dict(header)
    merged.update(profile)
    return merged


def main() -> int:
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")

    try:
        ensure_pointers(ARTIFACT_README)
        ensure_pointers(PROMOTION_README)
        header = load_json(HEADER_FIXTURE)
        clean_profile = load_json(CLEAN_PROFILE_FIXTURE)
        abort_profile = load_json(ABORT_PROFILE_FIXTURE)
        clean_artifact = load_json(CLEAN_ARTIFACT)
        abort_artifact = load_json(ABORT_ARTIFACT)

        if sorted(header.keys()) != sorted(HEADER_KEYS):
            raise ValueError("header keys mismatch")
        if sorted(clean_profile.keys()) != sorted(PROFILE_KEYS):
            raise ValueError("clean profile keys mismatch")
        if sorted(abort_profile.keys()) != sorted(PROFILE_KEYS):
            raise ValueError("abort profile keys mismatch")

        if build_artifact(header, clean_profile) != clean_artifact:
            raise ValueError("clean artifact reconstruction mismatch")
        if build_artifact(header, abort_profile) != abort_artifact:
            raise ValueError("abort artifact reconstruction mismatch")
    except ValueError as exc:
        return fail(str(exc))

    print("[proof-certificate-v1-draft-artifact-layers-selftest] ok header=3 profile=6")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
