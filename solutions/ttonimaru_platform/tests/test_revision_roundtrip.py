from __future__ import annotations

from conftest import OWNER_HEADERS, create_project, make_client, save_revision


def test_lesson_derived_save_creates_project_revision_not_lesson_mutation() -> None:
    client = make_client()
    project = create_project(client, source_lesson_id="seed_lesson_01")
    revision = save_revision(client, project["id"], state_hash="runtime-state-hash-1")

    assert project["schema"] == "ddn.ttonimaru.project.v1"
    assert project["source_lesson_id"] == "seed_lesson_01"
    assert revision["project_id"] == project["id"]
    assert revision["source_lesson_id"] == "lesson-grid-path"
    assert revision["state_hash"] == "runtime-state-hash-1"


def test_revision_append_only_list() -> None:
    client = make_client()
    project = create_project(client)
    first = save_revision(client, project["id"], state_hash="hash-1")
    second = save_revision(client, project["id"], state_hash="hash-2")

    res = client.get(f"/internal/v0/projects/{project['id']}/revisions", headers=OWNER_HEADERS)
    assert res.status_code == 200
    rows = res.json()["revisions"]
    assert [row["id"] for row in rows] == [first["id"], second["id"]]
    assert [row["state_hash"] for row in rows] == ["hash-1", "hash-2"]


def test_revision_read_preserves_source_and_hash_metadata() -> None:
    client = make_client()
    project = create_project(client)
    revision = save_revision(client, project["id"], state_hash="external-state-hash")

    res = client.get(f"/internal/v0/revisions/{revision['id']}", headers=OWNER_HEADERS)
    assert res.status_code == 200
    body = res.json()
    assert body["ddn_source"] == "보여주기 \"hello\"."
    assert body["state_hash"] == "external-state-hash"
    assert body["ddn_source_hash"].startswith("sha256:")
