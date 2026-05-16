from __future__ import annotations

from conftest import OWNER_HEADERS, VIEWER_HEADERS, create_project, make_client, save_revision


def publish(client, revision_id: str, *, visibility: str = "private", slug: str = "demo"):
    return client.post(
        f"/internal/v0/revisions/{revision_id}/publish",
        json={"slug": slug, "visibility": visibility},
        headers=OWNER_HEADERS,
    )


def test_publication_requires_revision() -> None:
    client = make_client()
    res = publish(client, "rev_missing")
    assert res.status_code == 404


def test_private_publication_owner_only_and_stable_json() -> None:
    client = make_client()
    project = create_project(client)
    revision = save_revision(client, project["id"], state_hash="hash-before")
    created = publish(client, revision["id"], visibility="private", slug="private-demo")
    assert created.status_code == 200, created.text
    publication_id = created.json()["publication_id"]

    first = client.get(f"/api/v1/publications/{publication_id}", headers=OWNER_HEADERS)
    second = client.get(f"/api/v1/publications/{publication_id}", headers=OWNER_HEADERS)
    assert first.status_code == 200
    assert first.content == second.content
    assert b"hash-before" in first.content

    no_auth = client.get(f"/api/v1/publications/{publication_id}")
    assert no_auth.status_code == 401
    viewer = client.get(f"/api/v1/publications/{publication_id}", headers=VIEWER_HEADERS)
    assert viewer.status_code == 403


def test_public_publication_is_unauthenticated_readable_and_alias_redirects() -> None:
    client = make_client()
    project = create_project(client)
    revision = save_revision(client, project["id"], state_hash="public-hash")
    created = publish(client, revision["id"], visibility="public", slug="public-demo")
    assert created.status_code == 200, created.text
    publication_id = created.json()["publication_id"]

    public_read = client.get(f"/api/v1/publications/{publication_id}")
    assert public_read.status_code == 200
    assert public_read.json()["publication_id"] == publication_id

    alias = client.get("/u/local-owner/public-demo", follow_redirects=False)
    assert alias.status_code == 302
    assert alias.headers["location"] == f"/api/v1/publications/{publication_id}"
