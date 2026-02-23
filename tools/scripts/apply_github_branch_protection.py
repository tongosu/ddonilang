#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
from pathlib import Path
from urllib import error, request


def load_config(path: Path) -> dict[str, object]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError(f"invalid config (must be object): {path}")
    return data


def parse_owner_repo(raw_remote: str) -> str:
    remote = raw_remote.strip()
    if remote.startswith("git@github.com:"):
        tail = remote.split("git@github.com:", 1)[1]
    elif remote.startswith("https://github.com/"):
        tail = remote.split("https://github.com/", 1)[1]
    elif remote.startswith("http://github.com/"):
        tail = remote.split("http://github.com/", 1)[1]
    else:
        raise ValueError(f"unsupported origin remote: {raw_remote}")

    if tail.endswith(".git"):
        tail = tail[:-4]
    parts = [item for item in tail.split("/") if item]
    if len(parts) < 2:
        raise ValueError(f"owner/repo parse failed from remote: {raw_remote}")
    return f"{parts[0]}/{parts[1]}"


def infer_repo_from_origin(root: Path) -> str:
    proc = subprocess.run(
        ["git", "remote", "get-url", "origin"],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        stderr = (proc.stderr or "").strip()
        raise RuntimeError(f"failed to read git origin remote: {stderr}")
    return parse_owner_repo(proc.stdout)


def normalize_payload(config: dict[str, object], branch: str) -> dict[str, object]:
    status = config.get("required_status_checks")
    if not isinstance(status, dict):
        raise ValueError("required_status_checks must be object")
    strict = bool(status.get("strict", True))
    contexts = status.get("contexts")
    if not isinstance(contexts, list) or not contexts:
        raise ValueError("required_status_checks.contexts must be non-empty list")
    context_values = [str(item).strip() for item in contexts if str(item).strip()]
    if not context_values:
        raise ValueError("required_status_checks.contexts must include non-empty value")

    payload: dict[str, object] = {
        "required_status_checks": {
            "strict": strict,
            "contexts": context_values,
        },
        "enforce_admins": bool(config.get("enforce_admins", False)),
        "required_pull_request_reviews": config.get("required_pull_request_reviews"),
        "restrictions": config.get("restrictions"),
        "required_conversation_resolution": bool(config.get("required_conversation_resolution", True)),
        "required_linear_history": bool(config.get("required_linear_history", False)),
        "allow_force_pushes": bool(config.get("allow_force_pushes", False)),
        "allow_deletions": bool(config.get("allow_deletions", False)),
        "block_creations": bool(config.get("block_creations", False)),
        "lock_branch": bool(config.get("lock_branch", False)),
        "allow_fork_syncing": bool(config.get("allow_fork_syncing", False)),
    }

    branch_from_config = str(config.get("branch", "")).strip()
    if branch_from_config and branch_from_config != branch:
        print(
            "warning: branch override differs from config "
            f"(config={branch_from_config}, arg={branch})"
        )
    return payload


def put_branch_protection(repo: str, branch: str, token: str, payload: dict[str, object]) -> dict[str, object]:
    url = f"https://api.github.com/repos/{repo}/branches/{branch}/protection"
    req = request.Request(
        url=url,
        method="PUT",
        data=json.dumps(payload).encode("utf-8"),
        headers={
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {token}",
            "X-GitHub-Api-Version": "2022-11-28",
            "Content-Type": "application/json",
            "User-Agent": "ddn-branch-protection-script",
        },
    )
    with request.urlopen(req, timeout=20) as resp:
        raw = resp.read().decode("utf-8", errors="replace")
    data = json.loads(raw) if raw.strip() else {}
    if not isinstance(data, dict):
        raise RuntimeError("unexpected GitHub API response type")
    return data


def main() -> int:
    parser = argparse.ArgumentParser(description="Apply GitHub branch protection from local json config")
    parser.add_argument(
        "--config",
        default=".github/branch-protection/main.required_checks.json",
        help="branch protection config json path",
    )
    parser.add_argument("--repo", default="", help="owner/repo (auto from origin if omitted)")
    parser.add_argument("--branch", default="", help="branch name override")
    parser.add_argument("--token-env", default="GITHUB_TOKEN", help="token env var name")
    parser.add_argument("--dry-run", action="store_true", help="print payload only, do not call API")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent.parent
    config_path = (root / args.config).resolve()
    if not config_path.exists():
        print(f"config not found: {config_path}")
        return 1

    try:
        config = load_config(config_path)
    except Exception as exc:
        print(f"config parse failed: {exc}")
        return 1

    branch = args.branch.strip() or str(config.get("branch", "")).strip() or "main"
    try:
        repo = args.repo.strip() or infer_repo_from_origin(root)
    except Exception as exc:
        print(f"repo resolve failed: {exc}")
        return 1

    try:
        payload = normalize_payload(config, branch)
    except Exception as exc:
        print(f"payload build failed: {exc}")
        return 1

    endpoint = f"https://api.github.com/repos/{repo}/branches/{branch}/protection"
    if args.dry_run:
        print(f"[dry-run] endpoint={endpoint}")
        print(json.dumps(payload, ensure_ascii=False, indent=2))
        print("branch protection dry-run ok")
        return 0

    token = os.environ.get(args.token_env, "").strip()
    if not token:
        print(f"missing token env: {args.token_env}")
        return 1

    try:
        resp = put_branch_protection(repo, branch, token, payload)
    except error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace").strip()
        print(f"github api error: status={exc.code} reason={exc.reason}")
        if body:
            print(body)
        return 1
    except Exception as exc:
        print(f"branch protection apply failed: {exc}")
        return 1

    contexts = payload["required_status_checks"]["contexts"]  # type: ignore[index]
    print(
        "branch protection apply ok: "
        f"repo={repo} branch={branch} strict={int(payload['required_status_checks']['strict'])} "
        f"contexts={contexts} required_conversation_resolution="
        f"{int(bool(payload.get('required_conversation_resolution', False)))}"
    )

    etag = str(resp.get("etag", "")).strip()
    if etag:
        print(f"etag={etag}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
