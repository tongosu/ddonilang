from __future__ import annotations

import hashlib
import json
import sqlite3
import uuid
from datetime import UTC, datetime
from pathlib import Path
from typing import Any


VISIBILITY = {"private", "team", "internal", "public"}


def utc_now() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def stable_json_bytes(value: dict[str, Any]) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def stable_json_text(value: dict[str, Any]) -> str:
    return stable_json_bytes(value).decode("utf-8")


def source_hash(source: str) -> str:
    return "sha256:" + hashlib.sha256(source.encode("utf-8")).hexdigest()


def new_id(prefix: str) -> str:
    return f"{prefix}_{uuid.uuid4().hex}"


class TtonimaruStore:
    def __init__(self, db_path: str | Path = ":memory:") -> None:
        self.db_path = str(db_path)
        self.conn = sqlite3.connect(self.db_path, check_same_thread=False)
        self.conn.row_factory = sqlite3.Row
        self.init_schema()

    def init_schema(self) -> None:
        self.conn.executescript(
            """
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                owner_id TEXT NOT NULL,
                name TEXT NOT NULL,
                visibility TEXT NOT NULL,
                source_lesson_id TEXT,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS revisions (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                owner_id TEXT NOT NULL,
                ddn_source TEXT NOT NULL,
                ddn_source_hash TEXT NOT NULL,
                state_hash TEXT NOT NULL,
                input_hash TEXT,
                source_lesson_id TEXT,
                saved_at TEXT NOT NULL,
                FOREIGN KEY(project_id) REFERENCES projects(id)
            );
            CREATE TABLE IF NOT EXISTS publications (
                id TEXT PRIMARY KEY,
                revision_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                owner_id TEXT NOT NULL,
                slug TEXT NOT NULL,
                visibility TEXT NOT NULL,
                manifest_json TEXT NOT NULL,
                published_at TEXT NOT NULL,
                UNIQUE(owner_id, slug),
                FOREIGN KEY(revision_id) REFERENCES revisions(id)
            );
            """
        )
        self.conn.commit()

    def create_project(
        self,
        *,
        owner_id: str,
        name: str,
        visibility: str = "private",
        source_lesson_id: str | None = None,
    ) -> dict[str, Any]:
        if visibility not in VISIBILITY:
            raise ValueError("E_VISIBILITY_INVALID")
        project_id = new_id("proj")
        row = {
            "id": project_id,
            "owner_id": owner_id,
            "name": name.strip() or "Untitled project",
            "visibility": visibility,
            "source_lesson_id": source_lesson_id or None,
            "created_at": utc_now(),
        }
        self.conn.execute(
            """
            INSERT INTO projects(id, owner_id, name, visibility, source_lesson_id, created_at)
            VALUES(:id, :owner_id, :name, :visibility, :source_lesson_id, :created_at)
            """,
            row,
        )
        self.conn.commit()
        return {"schema": "ddn.ttonimaru.project.v1", **row}

    def get_project(self, project_id: str) -> dict[str, Any] | None:
        row = self.conn.execute("SELECT * FROM projects WHERE id = ?", (project_id,)).fetchone()
        return None if row is None else {"schema": "ddn.ttonimaru.project.v1", **dict(row)}

    def create_revision(
        self,
        *,
        project_id: str,
        ddn_source: str,
        state_hash: str,
        input_hash: str | None = None,
        source_lesson_id: str | None = None,
    ) -> dict[str, Any]:
        project = self.get_project(project_id)
        if project is None:
            raise KeyError("E_PROJECT_NOT_FOUND")
        revision_id = new_id("rev")
        row = {
            "id": revision_id,
            "project_id": project_id,
            "owner_id": project["owner_id"],
            "ddn_source": ddn_source,
            "ddn_source_hash": source_hash(ddn_source),
            "state_hash": state_hash,
            "input_hash": input_hash or None,
            "source_lesson_id": source_lesson_id or project.get("source_lesson_id"),
            "saved_at": utc_now(),
        }
        self.conn.execute(
            """
            INSERT INTO revisions(
                id, project_id, owner_id, ddn_source, ddn_source_hash, state_hash,
                input_hash, source_lesson_id, saved_at
            )
            VALUES(
                :id, :project_id, :owner_id, :ddn_source, :ddn_source_hash, :state_hash,
                :input_hash, :source_lesson_id, :saved_at
            )
            """,
            row,
        )
        self.conn.commit()
        return {"schema": "ddn.ttonimaru.revision.v1", **row}

    def list_revisions(self, project_id: str) -> list[dict[str, Any]]:
        rows = self.conn.execute(
            "SELECT * FROM revisions WHERE project_id = ? ORDER BY rowid",
            (project_id,),
        ).fetchall()
        return [{"schema": "ddn.ttonimaru.revision.v1", **dict(row)} for row in rows]

    def get_revision(self, revision_id: str) -> dict[str, Any] | None:
        row = self.conn.execute("SELECT * FROM revisions WHERE id = ?", (revision_id,)).fetchone()
        return None if row is None else {"schema": "ddn.ttonimaru.revision.v1", **dict(row)}

    def create_publication(
        self,
        *,
        revision_id: str,
        owner: str,
        slug: str,
        visibility: str = "private",
    ) -> dict[str, Any]:
        revision = self.get_revision(revision_id)
        if revision is None:
            raise KeyError("E_REVISION_NOT_FOUND")
        if visibility not in VISIBILITY:
            raise ValueError("E_VISIBILITY_INVALID")
        publication_id = new_id("pub")
        published_at = utc_now()
        manifest = {
            "schema": "ddn.ttonimaru.publication_manifest.v1",
            "publication_id": publication_id,
            "revision_id": revision_id,
            "project_id": revision["project_id"],
            "owner_id": revision["owner_id"],
            "owner": owner,
            "slug": slug,
            "visibility": visibility,
            "ddn_source_hash": revision["ddn_source_hash"],
            "state_hash": revision["state_hash"],
            "source_lesson_id": revision.get("source_lesson_id"),
            "published_at": published_at,
            "immutable": True,
        }
        row = {
            "id": publication_id,
            "revision_id": revision_id,
            "project_id": revision["project_id"],
            "owner_id": revision["owner_id"],
            "slug": slug,
            "visibility": visibility,
            "manifest_json": stable_json_text(manifest),
            "published_at": published_at,
        }
        self.conn.execute(
            """
            INSERT INTO publications(
                id, revision_id, project_id, owner_id, slug, visibility, manifest_json, published_at
            )
            VALUES(
                :id, :revision_id, :project_id, :owner_id, :slug, :visibility, :manifest_json, :published_at
            )
            """,
            row,
        )
        self.conn.commit()
        return self.get_publication(publication_id) or manifest

    def get_publication(self, publication_id: str) -> dict[str, Any] | None:
        row = self.conn.execute("SELECT manifest_json FROM publications WHERE id = ?", (publication_id,)).fetchone()
        if row is None:
            return None
        parsed = json.loads(row["manifest_json"])
        assert isinstance(parsed, dict)
        return parsed

    def find_publication_by_alias(self, owner: str, slug: str) -> dict[str, Any] | None:
        row = self.conn.execute(
            """
            SELECT manifest_json FROM publications
            WHERE owner_id = ? AND slug = ?
            ORDER BY published_at DESC, id DESC LIMIT 1
            """,
            (owner, slug),
        ).fetchone()
        if row is None:
            return None
        parsed = json.loads(row["manifest_json"])
        assert isinstance(parsed, dict)
        return parsed
