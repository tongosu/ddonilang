from __future__ import annotations

from dataclasses import dataclass

from fastapi import Header, HTTPException


@dataclass(frozen=True)
class Actor:
    actor_id: str
    roles: frozenset[str]

    def has_role(self, role: str) -> bool:
        return role in self.roles


DEV_TOKENS: dict[str, Actor] = {
    "dev-owner-token": Actor("local-owner", frozenset({"owner", "publisher"})),
    "dev-viewer-token": Actor("local-viewer", frozenset({"viewer"})),
}


def _parse_bearer(value: str | None) -> str:
    text = (value or "").strip()
    if not text:
        raise HTTPException(status_code=401, detail={"code": "E_AUTH_REQUIRED"})
    prefix = "Bearer "
    if not text.startswith(prefix):
        raise HTTPException(status_code=401, detail={"code": "E_AUTH_INVALID"})
    token = text[len(prefix) :].strip()
    if not token:
        raise HTTPException(status_code=401, detail={"code": "E_AUTH_INVALID"})
    return token


def actor_from_authorization(authorization: str | None) -> Actor:
    token = _parse_bearer(authorization)
    actor = DEV_TOKENS.get(token)
    if actor is None:
        raise HTTPException(status_code=401, detail={"code": "E_AUTH_INVALID"})
    return actor


def require_actor(authorization: str | None = Header(default=None)) -> Actor:
    return actor_from_authorization(authorization)


def optional_actor(authorization: str | None = Header(default=None)) -> Actor | None:
    if authorization is None or not str(authorization).strip():
        return None
    return actor_from_authorization(authorization)


def require_role(actor: Actor, *roles: str) -> None:
    if not any(actor.has_role(role) for role in roles):
        raise HTTPException(status_code=403, detail={"code": "E_PERMISSION_DENIED"})


def require_owner(actor: Actor, owner_id: str) -> None:
    if actor.actor_id != owner_id:
        raise HTTPException(status_code=403, detail={"code": "E_PERMISSION_DENIED"})

