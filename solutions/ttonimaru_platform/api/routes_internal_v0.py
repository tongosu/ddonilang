from __future__ import annotations

from typing import Any

from fastapi import APIRouter, Depends, HTTPException

from solutions.ttonimaru_platform.api.auth import Actor, require_actor, require_owner, require_role
from solutions.ttonimaru_platform.storage.sqlite_store import TtonimaruStore


def _body_string(body: dict[str, Any], key: str, default: str = "") -> str:
    value = body.get(key, default)
    return "" if value is None else str(value)


def _not_found(code: str) -> HTTPException:
    return HTTPException(status_code=404, detail={"code": code})


def make_internal_router(store: TtonimaruStore) -> APIRouter:
    router = APIRouter(prefix="/internal/v0")

    @router.post("/projects")
    def create_project(body: dict[str, Any], actor: Actor = Depends(require_actor)) -> dict[str, Any]:
        require_role(actor, "owner")
        try:
            return store.create_project(
                owner_id=actor.actor_id,
                name=_body_string(body, "name", "Untitled project"),
                visibility=_body_string(body, "visibility", "private") or "private",
                source_lesson_id=_body_string(body, "source_lesson_id", "") or None,
            )
        except ValueError as exc:
            raise HTTPException(status_code=422, detail={"code": str(exc)}) from exc

    @router.get("/projects/{project_id}")
    def get_project(project_id: str, actor: Actor = Depends(require_actor)) -> dict[str, Any]:
        project = store.get_project(project_id)
        if project is None:
            raise _not_found("E_PROJECT_NOT_FOUND")
        require_owner(actor, str(project["owner_id"]))
        return project

    @router.post("/projects/{project_id}/save")
    def save_project(project_id: str, body: dict[str, Any], actor: Actor = Depends(require_actor)) -> dict[str, Any]:
        require_role(actor, "owner")
        project = store.get_project(project_id)
        if project is None:
            raise _not_found("E_PROJECT_NOT_FOUND")
        require_owner(actor, str(project["owner_id"]))
        ddn_source = _body_string(body, "ddn_source", "")
        state_hash = _body_string(body, "state_hash", "")
        if not ddn_source.strip():
            raise HTTPException(status_code=422, detail={"code": "E_DDN_SOURCE_REQUIRED"})
        if not state_hash.strip():
            raise HTTPException(status_code=422, detail={"code": "E_STATE_HASH_REQUIRED"})
        return store.create_revision(
            project_id=project_id,
            ddn_source=ddn_source,
            state_hash=state_hash,
            input_hash=_body_string(body, "input_hash", "") or None,
            source_lesson_id=_body_string(body, "source_lesson_id", "") or None,
        )

    @router.get("/projects/{project_id}/revisions")
    def list_revisions(project_id: str, actor: Actor = Depends(require_actor)) -> dict[str, Any]:
        project = store.get_project(project_id)
        if project is None:
            raise _not_found("E_PROJECT_NOT_FOUND")
        require_owner(actor, str(project["owner_id"]))
        return {
            "schema": "ddn.ttonimaru.revision_list.v1",
            "project_id": project_id,
            "revisions": store.list_revisions(project_id),
        }

    @router.get("/revisions/{revision_id}")
    def get_revision(revision_id: str, actor: Actor = Depends(require_actor)) -> dict[str, Any]:
        revision = store.get_revision(revision_id)
        if revision is None:
            raise _not_found("E_REVISION_NOT_FOUND")
        require_owner(actor, str(revision["owner_id"]))
        return revision

    @router.post("/revisions/{revision_id}/publish")
    def publish_revision(revision_id: str, body: dict[str, Any], actor: Actor = Depends(require_actor)) -> dict[str, Any]:
        require_role(actor, "publisher")
        revision = store.get_revision(revision_id)
        if revision is None:
            raise _not_found("E_REVISION_NOT_FOUND")
        require_owner(actor, str(revision["owner_id"]))
        slug = _body_string(body, "slug", "")
        if not slug.strip():
            raise HTTPException(status_code=422, detail={"code": "E_SLUG_REQUIRED"})
        try:
            publication = store.create_publication(
                revision_id=revision_id,
                owner=actor.actor_id,
                slug=slug.strip(),
                visibility=_body_string(body, "visibility", "private") or "private",
            )
        except ValueError as exc:
            raise HTTPException(status_code=422, detail={"code": str(exc)}) from exc
        except Exception as exc:
            raise HTTPException(status_code=409, detail={"code": "E_PUBLICATION_ALIAS_CONFLICT"}) from exc
        return {
            "schema": "ddn.ttonimaru.publication_created.v1",
            "publication_id": publication["publication_id"],
            "canonical_url": f"/api/v1/publications/{publication['publication_id']}",
            "alias_url": f"/u/{actor.actor_id}/{publication['slug']}",
            "manifest": publication,
        }

    return router

