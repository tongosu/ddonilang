from __future__ import annotations

from pathlib import Path

from fastapi import FastAPI

from solutions.ttonimaru_platform.api.routes_internal_v0 import make_internal_router
from solutions.ttonimaru_platform.api.routes_public_v1 import make_public_router
from solutions.ttonimaru_platform.storage.sqlite_store import TtonimaruStore


def create_app(db_path: str | Path = ":memory:") -> FastAPI:
    store = TtonimaruStore(db_path)
    app = FastAPI(title="Ttonimaru platform", version="0.1.0")
    app.state.store = store

    @app.get("/healthz")
    def healthz() -> dict[str, str]:
        return {"status": "ok", "service": "ttonimaru_platform"}

    app.include_router(make_internal_router(store))
    app.include_router(make_public_router(store))
    return app


app = create_app()

