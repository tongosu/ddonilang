from __future__ import annotations

from fastapi.testclient import TestClient

from solutions.ttonimaru_platform.api.app import create_app


OWNER_HEADERS = {"Authorization": "Bearer dev-owner-token"}
VIEWER_HEADERS = {"Authorization": "Bearer dev-viewer-token"}


def make_client() -> TestClient:
    return TestClient(create_app(":memory:"))


def create_project(client: TestClient, *, source_lesson_id: str | None = None) -> dict:
    body = {"name": "Grid path project"}
    if source_lesson_id:
        body["source_lesson_id"] = source_lesson_id
    res = client.post("/internal/v0/projects", json=body, headers=OWNER_HEADERS)
    assert res.status_code == 200, res.text
    return res.json()


def save_revision(client: TestClient, project_id: str, *, state_hash: str = "sha256:" + "a" * 64) -> dict:
    res = client.post(
        f"/internal/v0/projects/{project_id}/save",
        json={
            "ddn_source": "보여주기 \"hello\".",
            "state_hash": state_hash,
            "input_hash": "sha256:" + "b" * 64,
            "source_lesson_id": "lesson-grid-path",
        },
        headers=OWNER_HEADERS,
    )
    assert res.status_code == 200, res.text
    return res.json()

