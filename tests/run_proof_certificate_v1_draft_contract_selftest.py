#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/proof_certificate_v1_draft_contract/README.md")
PACK_README = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/README.md")
PACK_MANIFEST = Path("pack/age4_proof_certificate_v1_draft_candidate_v1/draft_pack.detjson")
ARTIFACT_README = Path("tests/proof_certificate_v1_draft_artifact/README.md")
LAYERS_README = Path("tests/proof_certificate_v1_draft_artifact_layers/README.md")
PROMOTION_README = Path("tests/proof_certificate_v1_promotion_candidate/README.md")
CLEAN_ARTIFACT = Path("tests/proof_certificate_v1_draft_artifact/fixtures/clean.proof_certificate_v1_draft_artifact.detjson")
ABORT_ARTIFACT = Path("tests/proof_certificate_v1_draft_artifact/fixtures/abort.proof_certificate_v1_draft_artifact.detjson")
HEADER_FIXTURE = Path("tests/proof_certificate_v1_draft_artifact_layers/fixtures/shared.proof_certificate_v1_draft_artifact_header.detjson")
CLEAN_PROFILE_FIXTURE = Path("tests/proof_certificate_v1_draft_artifact_layers/fixtures/clean.proof_certificate_v1_draft_artifact_profile.detjson")
ABORT_PROFILE_FIXTURE = Path("tests/proof_certificate_v1_draft_artifact_layers/fixtures/abort.proof_certificate_v1_draft_artifact_profile.detjson")

README_SNIPPETS = (
    "## Stable Contract",
    "`pack/age4_proof_certificate_v1_draft_candidate_v1/README.md`",
    "`pack/age4_proof_certificate_v1_draft_candidate_v1/draft_pack.detjson`",
    "`tests/proof_certificate_v1_draft_artifact/README.md`",
    "`tests/proof_certificate_v1_draft_artifact_layers/README.md`",
    "`python tests/run_proof_certificate_v1_draft_contract_selftest.py`",
    "`proof_certificate_v1_draft_contract_selftest`",
    "`ddn.proof_certificate_v1_draft_pack.v1`",
    "`ddn.proof_certificate_v1_draft_artifact.v1`",
    "`shared header(3)` + `profile body(6)`",
)
POINTERS = (
    "`tests/proof_certificate_v1_draft_contract/README.md`",
    "`python tests/run_proof_certificate_v1_draft_contract_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-v1-draft-contract-selftest] fail: {message}")
    return 1


def load_json(path: Path) -> object:
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
        ensure_pointers(PACK_README)
        ensure_pointers(ARTIFACT_README)
        ensure_pointers(LAYERS_README)
        ensure_pointers(PROMOTION_README)

        pack = load_json(PACK_MANIFEST)
        if not isinstance(pack, dict):
            raise ValueError("pack manifest should be object")
        if str(pack.get("schema", "")).strip() != "ddn.proof_certificate_v1_draft_pack.v1":
            raise ValueError("pack schema mismatch")
        profiles = pack.get("profiles")
        if not isinstance(profiles, list) or len(profiles) != 2:
            raise ValueError("pack profile count mismatch")

        header = load_json(HEADER_FIXTURE)
        clean_profile = load_json(CLEAN_PROFILE_FIXTURE)
        abort_profile = load_json(ABORT_PROFILE_FIXTURE)
        clean_artifact = load_json(CLEAN_ARTIFACT)
        abort_artifact = load_json(ABORT_ARTIFACT)
        if not isinstance(header, dict) or not isinstance(clean_profile, dict) or not isinstance(abort_profile, dict):
            raise ValueError("layer fixtures should be objects")
        if not isinstance(clean_artifact, dict) or not isinstance(abort_artifact, dict):
            raise ValueError("artifact fixtures should be objects")

        if build_artifact(header, clean_profile) != clean_artifact:
            raise ValueError("clean draft contract reconstruction mismatch")
        if build_artifact(header, abort_profile) != abort_artifact:
            raise ValueError("abort draft contract reconstruction mismatch")
        if str(clean_artifact.get("schema", "")).strip() != "ddn.proof_certificate_v1_draft_artifact.v1":
            raise ValueError("clean artifact schema mismatch")
        if str(abort_artifact.get("schema", "")).strip() != "ddn.proof_certificate_v1_draft_artifact.v1":
            raise ValueError("abort artifact schema mismatch")
    except ValueError as exc:
        return fail(str(exc))

    print("[proof-certificate-v1-draft-contract-selftest] ok layers=3 profiles=2")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
