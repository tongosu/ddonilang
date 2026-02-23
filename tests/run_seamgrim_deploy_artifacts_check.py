#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


def has_all_patterns(text: str, patterns: list[str]) -> tuple[bool, str]:
    for pattern in patterns:
        if pattern not in text:
            return False, pattern
    return True, ""


def main() -> int:
    root = Path(__file__).resolve().parent.parent

    required_files = [
        root / ".dockerignore",
        root / "solutions" / "seamgrim_ui_mvp" / "deploy" / "Dockerfile",
        root / "solutions" / "seamgrim_ui_mvp" / "deploy" / "docker-compose.yml",
        root / "solutions" / "seamgrim_ui_mvp" / "deploy" / "README.md",
        root / "solutions" / "seamgrim_ui_mvp" / "deploy" / "nginx" / "seamgrim.conf",
    ]
    for path in required_files:
        if not path.exists():
            rel = path.relative_to(root).as_posix()
            print(f"missing deploy file: {rel}")
            return 1

    dockerfile = (root / "solutions" / "seamgrim_ui_mvp" / "deploy" / "Dockerfile").read_text(
        encoding="utf-8"
    )
    compose = (
        root / "solutions" / "seamgrim_ui_mvp" / "deploy" / "docker-compose.yml"
    ).read_text(encoding="utf-8")
    nginx_conf = (
        root / "solutions" / "seamgrim_ui_mvp" / "deploy" / "nginx" / "seamgrim.conf"
    ).read_text(encoding="utf-8")
    deploy_readme = (root / "solutions" / "seamgrim_ui_mvp" / "deploy" / "README.md").read_text(
        encoding="utf-8"
    )
    tools_readme = (root / "solutions" / "seamgrim_ui_mvp" / "tools" / "README.md").read_text(
        encoding="utf-8"
    )
    ui_readme = (root / "solutions" / "seamgrim_ui_mvp" / "ui" / "README.md").read_text(
        encoding="utf-8"
    )

    ok, missing = has_all_patterns(
        dockerfile,
        [
            "FROM rust:",
            "FROM python:",
            "ddn_exec_server.py",
            "EXPOSE 8787",
            "/api/health",
        ],
    )
    if not ok:
        print(f"check=dockerfile_required_tokens missing={missing}")
        return 1

    ok, missing = has_all_patterns(
        compose,
        [
            "ddn-exec-server",
            "dockerfile: solutions/seamgrim_ui_mvp/deploy/Dockerfile",
            "DDN_EXEC_SERVER_PORT",
            "restart: unless-stopped",
        ],
    )
    if not ok:
        print(f"check=compose_required_tokens missing={missing}")
        return 1

    ok, missing = has_all_patterns(
        nginx_conf,
        [
            "application/wasm",
            "location /wasm/",
            "default_type application/wasm",
            "proxy_pass http://ddn-exec-server:8787",
        ],
    )
    if not ok:
        print(f"check=nginx_conf_required_tokens missing={missing}")
        return 1

    ok, missing = has_all_patterns(
        deploy_readme,
        [
            "Nginx Reverse Proxy",
            "application/wasm",
            "proxy_pass http://ddn-exec-server:8787",
        ],
    )
    if not ok:
        print(f"check=deploy_readme_nginx_guide missing={missing}")
        return 1

    ok, missing = has_all_patterns(
        tools_readme,
        [
            "docker compose -f solutions/seamgrim_ui_mvp/deploy/docker-compose.yml up --build -d",
            "docker compose -f solutions/seamgrim_ui_mvp/deploy/docker-compose.yml down",
        ],
    )
    if not ok:
        print(f"check=tools_readme_deploy_guide missing={missing}")
        return 1

    ok, missing = has_all_patterns(
        ui_readme,
        [
            "docker compose -f solutions/seamgrim_ui_mvp/deploy/docker-compose.yml up --build -d",
        ],
    )
    if not ok:
        print(f"check=ui_readme_deploy_guide missing={missing}")
        return 1

    print("seamgrim deploy artifacts check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
