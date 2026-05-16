from __future__ import annotations

from fastapi import APIRouter, Depends, HTTPException
from fastapi.responses import RedirectResponse, Response

from solutions.ttonimaru_platform.api.auth import Actor, optional_actor
from solutions.ttonimaru_platform.storage.sqlite_store import TtonimaruStore, stable_json_text


def _can_read_publication(publication: dict[str, object], actor: Actor | None) -> bool:
    visibility = str(publication.get("visibility") or "")
    if visibility == "public":
        return True
    if actor is None:
        raise HTTPException(status_code=401, detail={"code": "E_AUTH_REQUIRED"})
    return actor.actor_id == str(publication.get("owner_id") or "")


def _publication_response(publication: dict[str, object], actor: Actor | None) -> Response:
    if not _can_read_publication(publication, actor):
        raise HTTPException(status_code=403, detail={"code": "E_PERMISSION_DENIED"})
    return Response(content=stable_json_text(publication), media_type="application/json")


def _package_stub(scope: str, name: str, version: str = "latest") -> dict[str, object]:
    return {
        "schema": "ddn.ttonimaru.package_metadata_stub.v1",
        "scope": scope,
        "name": name,
        "version": version or "latest",
        "status": "stub",
        "install_supported": False,
        "update_supported": False,
        "remove_supported": False,
        "public_registry_final": False,
    }


def make_public_router(store: TtonimaruStore) -> APIRouter:
    router = APIRouter()

    @router.get("/api/v1/publications/{publication_id}")
    def get_publication(
        publication_id: str,
        actor: Actor | None = Depends(optional_actor),
    ) -> Response:
        publication = store.get_publication(publication_id)
        if publication is None:
            raise HTTPException(status_code=404, detail={"code": "E_PUBLICATION_NOT_FOUND"})
        return _publication_response(publication, actor)

    @router.get("/api/v1/publications/{publication_id}/manifest")
    def get_publication_manifest(
        publication_id: str,
        actor: Actor | None = Depends(optional_actor),
    ) -> Response:
        publication = store.get_publication(publication_id)
        if publication is None:
            raise HTTPException(status_code=404, detail={"code": "E_PUBLICATION_NOT_FOUND"})
        return _publication_response(publication, actor)

    @router.get("/u/{owner}/{slug}")
    def get_alias(owner: str, slug: str) -> RedirectResponse:
        publication = store.find_publication_by_alias(owner, slug)
        if publication is None:
            raise HTTPException(status_code=404, detail={"code": "E_PUBLICATION_ALIAS_NOT_FOUND"})
        return RedirectResponse(
            url=f"/api/v1/publications/{publication['publication_id']}",
            status_code=302,
        )

    @router.get("/api/v1/registry/packages/{scope}/{name}")
    def get_package(scope: str, name: str) -> dict[str, object]:
        return _package_stub(scope=scope, name=name)

    @router.get("/api/v1/registry/packages/{scope}/{name}/{version}")
    def get_package_version(scope: str, name: str, version: str) -> dict[str, object]:
        return _package_stub(scope=scope, name=name, version=version)

    return router

