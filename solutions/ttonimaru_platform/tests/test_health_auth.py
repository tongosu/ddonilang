from __future__ import annotations

from conftest import OWNER_HEADERS, VIEWER_HEADERS, make_client


def test_healthz() -> None:
    client = make_client()
    res = client.get("/healthz")
    assert res.status_code == 200
    assert res.json()["status"] == "ok"


def test_internal_requires_token() -> None:
    client = make_client()
    res = client.post("/internal/v0/projects", json={"name": "x"})
    assert res.status_code == 401


def test_invalid_token_fails() -> None:
    client = make_client()
    res = client.post(
        "/internal/v0/projects",
        json={"name": "x"},
        headers={"Authorization": "Bearer wrong-token"},
    )
    assert res.status_code == 401


def test_viewer_cannot_mutate() -> None:
    client = make_client()
    res = client.post("/internal/v0/projects", json={"name": "x"}, headers=VIEWER_HEADERS)
    assert res.status_code == 403


def test_owner_can_create_private_project() -> None:
    client = make_client()
    res = client.post("/internal/v0/projects", json={"name": "x"}, headers=OWNER_HEADERS)
    assert res.status_code == 200
    body = res.json()
    assert body["owner_id"] == "local-owner"
    assert body["visibility"] == "private"
