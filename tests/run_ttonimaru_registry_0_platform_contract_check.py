#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
CHARTER = ROOT / "pack" / "ttonimaru_registry_0_v1" / "valid" / "valid_platform_charter.detjson"
PLATFORM_CONTRACT = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "platform_contract.js"


def load_charter() -> dict[str, Any]:
    return json.loads(CHARTER.read_text(encoding="utf-8"))


def load_platform_contract() -> dict[str, Any]:
    script = f"""
import {{ pathToFileURL }} from 'url';
const mod = await import(pathToFileURL({json.dumps(str(PLATFORM_CONTRACT))}).href);
const values = (obj) => Object.values(obj);
console.log(JSON.stringify({{
  ObjectKind: values(mod.ObjectKind),
  ShareKind: values(mod.ShareKind),
  Visibility: values(mod.Visibility),
  Role: values(mod.Role),
  CatalogKind: values(mod.CatalogKind),
  PublicationPolicy: mod.PublicationPolicy
}}));
"""
    result = subprocess.run(
        ["node", "--input-type=module", "-e", script],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or result.stdout.strip() or "node failed")
    return json.loads(result.stdout)


def require_equal(name: str, actual: Any, expected: Any) -> list[str]:
    if actual != expected:
        return [f"{name}: expected {expected!r}, got {actual!r}"]
    return []


def main() -> int:
    failures: list[str] = []
    try:
        charter = load_charter()
        contract = load_platform_contract()
    except Exception as exc:  # noqa: BLE001 - checker should print concise failure.
        print(f"platform contract load failed: {exc}", file=sys.stderr)
        return 1

    object_values = set(contract["ObjectKind"])
    charter_objects = set(charter["objects"])
    if not charter_objects.issubset(object_values):
        failures.append(f"objects: charter has values missing in ObjectKind: {sorted(charter_objects - object_values)}")
    failures.extend(require_equal("objects scope", sorted(charter_objects), ["artifact", "lesson", "package", "project"]))
    failures.extend(require_equal("share_kinds", sorted(charter["share_kinds"]), sorted(contract["ShareKind"])))
    failures.extend(require_equal("visibility", sorted(charter["visibility"]), sorted(contract["Visibility"])))
    failures.extend(require_equal("roles", sorted(charter["roles"]), sorted(contract["Role"])))
    failures.extend(require_equal("catalog kinds", sorted(item["kind"] for item in charter["catalogs"]), sorted(contract["CatalogKind"])))

    publication = charter["publication_policy"]
    contract_publication = contract["PublicationPolicy"]
    failures.extend(require_equal("publication.revision_pin_required", publication["revision_pin_required"], contract_publication["PINNED_REVISION_REQUIRED"]))
    failures.extend(require_equal("publication.published_artifact_immutable", publication["published_artifact_immutable"], contract_publication["SNAPSHOT_IMMUTABLE"]))
    failures.extend(require_equal("publication.public_link_target", "artifact", contract_publication["PUBLIC_LINK_TARGET_DEFAULT"]))
    failures.extend(require_equal("publication.republish_append_only", publication["republish_mode"] == "new_artifact", contract_publication["REPUBLISH_APPEND_ONLY"]))

    if failures:
        for failure in failures:
            print(failure, file=sys.stderr)
        return 1

    print("platform_contract=PASS objects=4 share_kinds=3 visibility=4 roles=4 catalogs=3 publication=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
