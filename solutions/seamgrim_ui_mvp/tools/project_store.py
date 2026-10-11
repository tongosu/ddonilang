from __future__ import annotations

import hashlib
import json
import os
import secrets
import shutil
import sqlite3
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


SCHEMA_VERSION = "seamgrim.project_store.sqlite.v0"
PROJECT_ID_PREFIX = "sgp_"
REVISION_ID_PREFIX = "sgr_"
RUN_ID_PREFIX = "sgrun_"
CAPABILITY_BYTES = 32
ROLE_VIEW = "view"
ROLE_EDIT = "edit"
MAX_PROJECT_TITLE_CHARS = 200
MAX_PROJECT_REVISIONS = 10_000
MAX_PROJECT_RUNS = 2_000
MAX_RUN_CHECKPOINTS = 2_000
MAX_CHECKPOINT_CANONICAL_BYTES = 256_000
SQLITE_SYNCHRONOUS_MODES = {"OFF", "NORMAL", "FULL", "EXTRA"}


class ProjectStoreError(RuntimeError):
    code = "E_PROJECT_STORE"
    http_status = 500

    def __init__(self, message: str = "") -> None:
        super().__init__(message or self.code)


class ProjectNotFoundError(ProjectStoreError):
    code = "E_PROJECT_NOT_FOUND"
    http_status = 404


class ProjectConflictError(ProjectStoreError):
    code = "E_PROJECT_REVISION_CONFLICT"
    http_status = 409


class ProjectDeletedError(ProjectStoreError):
    code = "E_PROJECT_DELETED"
    http_status = 410


class ProjectValidationError(ProjectStoreError):
    code = "E_PROJECT_INVALID"
    http_status = 400


class ProjectPermissionError(ProjectStoreError):
    code = "E_PROJECT_PERMISSION_DENIED"
    http_status = 403


class ProjectLimitError(ProjectStoreError):
    code = "E_PROJECT_LIMIT"
    http_status = 409


class ClosingSqliteConnection(sqlite3.Connection):
    """Commit or roll back a context-managed connection, then release its file handles."""

    def __exit__(self, exc_type: object, exc_value: object, traceback: object) -> bool:
        try:
            return bool(super().__exit__(exc_type, exc_value, traceback))
        finally:
            self.close()


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def stable_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def sha256_hex(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def new_id(prefix: str) -> str:
    return f"{prefix}{secrets.token_urlsafe(18).replace('-', '').replace('_', '')[:24]}"


def new_capability() -> str:
    return secrets.token_urlsafe(CAPABILITY_BYTES)


def as_object(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def normalize_project_title(value: Any, fallback: str = "이름 없는 셈그림") -> str:
    normalized_fallback = str(fallback or "이름 없는 셈그림").strip() or "이름 없는 셈그림"
    title = str(value or normalized_fallback).strip() or normalized_fallback
    if len(title) > MAX_PROJECT_TITLE_CHARS:
        raise ProjectValidationError(f"project title exceeds {MAX_PROJECT_TITLE_CHARS} characters")
    return title


def object_relative_path(content_sha256: str) -> str:
    digest = str(content_sha256 or "").strip().lower()
    if len(digest) != 64 or any(ch not in "0123456789abcdef" for ch in digest):
        raise ProjectValidationError("content sha256 must be 64 lowercase hex chars")
    return str(Path("objects") / digest[:2] / f"{digest}.json")


@dataclass(frozen=True)
class StoredRevision:
    project_id: str
    revision_id: str
    parent_revision_id: str | None
    content_sha256: str
    package_hash: str
    object_path: str
    created_at: str
    restore_of_revision_id: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "project_id": self.project_id,
            "revision_id": self.revision_id,
            "parent_revision_id": self.parent_revision_id,
            "content_sha256": self.content_sha256,
            "package_hash": self.package_hash,
            "object_path": self.object_path,
            "created_at": self.created_at,
            "restore_of_revision_id": self.restore_of_revision_id,
        }


class SeamgrimProjectStore:
    def __init__(
        self,
        data_dir: str | os.PathLike[str],
        *,
        sqlite_synchronous: str | None = None,
    ) -> None:
        self.data_dir = Path(data_dir).resolve()
        self.db_path = self.data_dir / "seamgrim_projects.sqlite3"
        self.objects_dir = self.data_dir / "objects"
        self.sqlite_synchronous = self._normalize_sqlite_synchronous(sqlite_synchronous or "NORMAL")
        self.data_dir.mkdir(parents=True, exist_ok=True)
        self.objects_dir.mkdir(parents=True, exist_ok=True)
        self._init_db()

    @staticmethod
    def _normalize_sqlite_synchronous(mode: str) -> str:
        normalized = str(mode or "NORMAL").strip().upper()
        if normalized not in SQLITE_SYNCHRONOUS_MODES:
            raise ProjectValidationError("invalid sqlite synchronous mode")
        return normalized

    def connect(self) -> sqlite3.Connection:
        conn = sqlite3.connect(self.db_path, factory=ClosingSqliteConnection)
        conn.row_factory = sqlite3.Row
        conn.execute("PRAGMA journal_mode=WAL")
        conn.execute("PRAGMA foreign_keys=ON")
        conn.execute(f"PRAGMA synchronous={self.sqlite_synchronous}")
        return conn

    def _init_db(self) -> None:
        with sqlite3.connect(self.db_path, factory=ClosingSqliteConnection) as conn:
            conn.execute("PRAGMA journal_mode=WAL")
            conn.execute("PRAGMA foreign_keys=ON")
            conn.execute(f"PRAGMA synchronous={self.sqlite_synchronous}")
            conn.executescript(
                """
                CREATE TABLE IF NOT EXISTS store_meta (
                  key TEXT PRIMARY KEY,
                  value TEXT NOT NULL
                );
                INSERT OR IGNORE INTO store_meta(key, value)
                  VALUES('schema', 'seamgrim.project_store.sqlite.v0');

                CREATE TABLE IF NOT EXISTS objects (
                  content_sha256 TEXT PRIMARY KEY,
                  byte_length INTEGER NOT NULL,
                  relative_path TEXT NOT NULL UNIQUE,
                  created_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS projects (
                  project_id TEXT PRIMARY KEY,
                  title TEXT NOT NULL,
                  source_lesson_id TEXT NOT NULL,
                  head_revision_id TEXT,
                  origin_project_id TEXT,
                  origin_revision_id TEXT,
                  created_at TEXT NOT NULL,
                  updated_at TEXT NOT NULL,
                  deleted_at TEXT,
                  FOREIGN KEY(head_revision_id) REFERENCES revisions(revision_id)
                );

                CREATE TABLE IF NOT EXISTS revisions (
                  revision_id TEXT PRIMARY KEY,
                  project_id TEXT NOT NULL,
                  parent_revision_id TEXT,
                  content_sha256 TEXT NOT NULL,
                  package_hash TEXT NOT NULL,
                  object_path TEXT NOT NULL,
                  created_at TEXT NOT NULL,
                  restore_of_revision_id TEXT,
                  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE RESTRICT,
                  FOREIGN KEY(parent_revision_id) REFERENCES revisions(revision_id),
                  FOREIGN KEY(content_sha256) REFERENCES objects(content_sha256),
                  FOREIGN KEY(restore_of_revision_id) REFERENCES revisions(revision_id)
                );

                CREATE INDEX IF NOT EXISTS idx_revisions_project_created
                  ON revisions(project_id, created_at, revision_id);

                CREATE TABLE IF NOT EXISTS capabilities (
                  project_id TEXT NOT NULL,
                  role TEXT NOT NULL CHECK(role IN ('edit', 'view')),
                  capability_sha256 TEXT NOT NULL UNIQUE,
                  created_at TEXT NOT NULL,
                  PRIMARY KEY(project_id, role),
                  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE RESTRICT
                );

                CREATE TABLE IF NOT EXISTS runs (
                  server_run_id TEXT PRIMARY KEY,
                  project_id TEXT NOT NULL,
                  source_revision_id TEXT NOT NULL,
                  source_content_sha256 TEXT NOT NULL,
                  recipe_hash TEXT NOT NULL,
                  seed TEXT NOT NULL,
                  parent_server_run_id TEXT,
                  branch_madi INTEGER,
                  branch_state_hash TEXT,
                  final_status TEXT NOT NULL,
                  created_at TEXT NOT NULL,
                  finalized_at TEXT,
                  latest_sequence INTEGER NOT NULL DEFAULT -1,
                  latest_committed_madi INTEGER NOT NULL DEFAULT -1,
                  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE RESTRICT,
                  FOREIGN KEY(source_revision_id) REFERENCES revisions(revision_id),
                  FOREIGN KEY(source_content_sha256) REFERENCES objects(content_sha256),
                  FOREIGN KEY(parent_server_run_id) REFERENCES runs(server_run_id)
                );

                CREATE TABLE IF NOT EXISTS run_checkpoints (
                  server_run_id TEXT NOT NULL,
                  sequence INTEGER NOT NULL,
                  project_id TEXT NOT NULL,
                  committed_madi INTEGER NOT NULL,
                  previous_checkpoint_sha256 TEXT,
                  checkpoint_sha256 TEXT NOT NULL UNIQUE,
                  object_path TEXT NOT NULL,
                  created_at TEXT NOT NULL,
                  PRIMARY KEY(server_run_id, sequence),
                  FOREIGN KEY(server_run_id) REFERENCES runs(server_run_id) ON DELETE RESTRICT,
                  FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE RESTRICT,
                  FOREIGN KEY(checkpoint_sha256) REFERENCES objects(content_sha256)
                );

                CREATE INDEX IF NOT EXISTS idx_runs_project_created
                  ON runs(project_id, created_at, server_run_id);

                CREATE INDEX IF NOT EXISTS idx_run_checkpoints_run_sequence
                  ON run_checkpoints(server_run_id, sequence);
                """
            )
            conn.commit()

    def runtime_pragmas(self) -> dict[str, Any]:
        with self.connect() as conn:
            journal_mode = str(conn.execute("PRAGMA journal_mode").fetchone()[0]).lower()
            synchronous_raw = int(conn.execute("PRAGMA synchronous").fetchone()[0])
        synchronous_name = {
            0: "OFF",
            1: "NORMAL",
            2: "FULL",
            3: "EXTRA",
        }.get(synchronous_raw, f"UNKNOWN:{synchronous_raw}")
        return {
            "schema": "seamgrim.project_store_runtime_pragmas.v1",
            "sqlite_version": sqlite3.sqlite_version,
            "journal_mode": journal_mode,
            "synchronous": synchronous_name,
            "synchronous_raw": synchronous_raw,
            "configured_synchronous": self.sqlite_synchronous,
        }

    def _write_object_atomic(self, content_sha256: str, canonical_json: str) -> tuple[str, int]:
        relative = object_relative_path(content_sha256)
        target = self.data_dir / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        encoded = canonical_json.encode("utf-8")
        if target.exists():
            existing = target.read_bytes()
            if hashlib.sha256(existing).hexdigest() != content_sha256:
                raise ProjectValidationError("existing object hash mismatch")
            return relative.replace("\\", "/"), len(existing)
        tmp = target.with_name(f"{target.name}.{os.getpid()}.{secrets.token_hex(6)}.tmp")
        with tmp.open("wb") as handle:
            handle.write(encoded)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(tmp, target)
        try:
            dir_fd = os.open(str(target.parent), os.O_RDONLY)
            try:
                os.fsync(dir_fd)
            finally:
                os.close(dir_fd)
        except OSError:
            pass
        return relative.replace("\\", "/"), len(encoded)

    def _artifact_parts(self, artifact: dict[str, Any]) -> tuple[dict[str, Any], str, str, str]:
        row = as_object(artifact)
        snapshot = as_object(row.get("project_snapshot"))
        if row.get("schema") != "seamgrim.project_snapshot_artifact.v0":
            raise ProjectValidationError("snapshot artifact schema mismatch")
        if snapshot.get("schema") != "seamgrim.project_snapshot.v0":
            raise ProjectValidationError("project snapshot schema mismatch")
        canonical_json = str(row.get("canonical_json") or stable_json(snapshot))
        expected_sha = sha256_hex(canonical_json)
        content_sha256 = str(row.get("content_sha256") or "").strip().lower()
        if content_sha256 != expected_sha:
            raise ProjectValidationError("project snapshot sha256 mismatch")
        package_hash = str(snapshot.get("package_hash_compat") or "").strip()
        return snapshot, canonical_json, content_sha256, package_hash

    def _require_project(self, conn: sqlite3.Connection, project_id: str, *, include_deleted: bool = False) -> sqlite3.Row:
        project = conn.execute("SELECT * FROM projects WHERE project_id = ?", (project_id,)).fetchone()
        if not project:
            raise ProjectNotFoundError("project not found")
        if project["deleted_at"] and not include_deleted:
            raise ProjectDeletedError("project is deleted")
        return project

    def create_project(
        self,
        artifact: dict[str, Any],
        *,
        title: str = "",
        project_id: str | None = None,
        revision_id: str | None = None,
        origin_project_id: str | None = None,
        origin_revision_id: str | None = None,
    ) -> dict[str, Any]:
        snapshot, canonical_json, content_sha256, package_hash = self._artifact_parts(artifact)
        now = utc_now_iso()
        pid = project_id or new_id(PROJECT_ID_PREFIX)
        rid = revision_id or new_id(REVISION_ID_PREFIX)
        snapshot_title = normalize_project_title(snapshot.get("project_name"))
        project_title = normalize_project_title(title, snapshot_title)
        source_lesson_id = str(snapshot.get("source_lesson_id") or "custom").strip() or "custom"
        object_path, byte_length = self._write_object_atomic(content_sha256, canonical_json)
        with self.connect() as conn:
            conn.execute("BEGIN IMMEDIATE")
            conn.execute(
                "INSERT OR IGNORE INTO objects(content_sha256, byte_length, relative_path, created_at) VALUES(?, ?, ?, ?)",
                (content_sha256, byte_length, object_path, now),
            )
            conn.execute(
                """
                INSERT INTO projects(
                  project_id, title, source_lesson_id, head_revision_id,
                  origin_project_id, origin_revision_id, created_at, updated_at, deleted_at
                ) VALUES(?, ?, ?, NULL, ?, ?, ?, ?, NULL)
                """,
                (pid, project_title, source_lesson_id, origin_project_id, origin_revision_id, now, now),
            )
            conn.execute(
                """
                INSERT INTO revisions(
                  revision_id, project_id, parent_revision_id, content_sha256,
                  package_hash, object_path, created_at, restore_of_revision_id
                ) VALUES(?, ?, NULL, ?, ?, ?, ?, NULL)
                """,
                (rid, pid, content_sha256, package_hash, object_path, now),
            )
            conn.execute(
                "UPDATE projects SET head_revision_id = ?, updated_at = ? WHERE project_id = ?",
                (rid, now, pid),
            )
            conn.commit()
        return self.get_project(pid)

    def add_revision(
        self,
        project_id: str,
        artifact: dict[str, Any],
        *,
        expected_parent_revision_id: str | None,
        revision_id: str | None = None,
        restore_of_revision_id: str | None = None,
    ) -> dict[str, Any]:
        snapshot, canonical_json, content_sha256, package_hash = self._artifact_parts(artifact)
        now = utc_now_iso()
        rid = revision_id or new_id(REVISION_ID_PREFIX)
        with self.connect() as conn:
            conn.execute("BEGIN IMMEDIATE")
            project = self._require_project(conn, project_id)
            head = project["head_revision_id"]
            if str(expected_parent_revision_id or "") != str(head or ""):
                raise ProjectConflictError("expected parent revision does not match head")
            revision_count = int(
                conn.execute("SELECT COUNT(*) FROM revisions WHERE project_id = ?", (project_id,)).fetchone()[0]
            )
            if revision_count >= MAX_PROJECT_REVISIONS:
                raise ProjectLimitError(f"project revision limit reached ({MAX_PROJECT_REVISIONS})")
            if restore_of_revision_id:
                self._require_revision(conn, project_id, restore_of_revision_id)
            next_title = normalize_project_title(snapshot.get("project_name"), str(project["title"]))
            object_path, byte_length = self._write_object_atomic(content_sha256, canonical_json)
            conn.execute(
                "INSERT OR IGNORE INTO objects(content_sha256, byte_length, relative_path, created_at) VALUES(?, ?, ?, ?)",
                (content_sha256, byte_length, object_path, now),
            )
            conn.execute(
                """
                INSERT INTO revisions(
                  revision_id, project_id, parent_revision_id, content_sha256,
                  package_hash, object_path, created_at, restore_of_revision_id
                ) VALUES(?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (rid, project_id, head, content_sha256, package_hash, object_path, now, restore_of_revision_id),
            )
            conn.execute(
                "UPDATE projects SET title = ?, head_revision_id = ?, updated_at = ? WHERE project_id = ?",
                (next_title, rid, now, project_id),
            )
            conn.commit()
        return self.get_project(project_id)

    def _require_revision(self, conn: sqlite3.Connection, project_id: str, revision_id: str) -> sqlite3.Row:
        row = conn.execute(
            "SELECT * FROM revisions WHERE project_id = ? AND revision_id = ?",
            (project_id, revision_id),
        ).fetchone()
        if not row:
            raise ProjectNotFoundError("revision not found")
        return row

    def read_object_json(self, content_sha256: str) -> dict[str, Any]:
        relative = object_relative_path(content_sha256)
        target = self.data_dir / relative
        if not target.exists():
            raise ProjectNotFoundError("object missing")
        text = target.read_text(encoding="utf-8")
        if sha256_hex(text) != content_sha256:
            raise ProjectValidationError("object sha256 mismatch")
        return json.loads(text)

    def get_project(self, project_id: str, *, include_deleted: bool = False) -> dict[str, Any]:
        with self.connect() as conn:
            project = self._require_project(conn, project_id, include_deleted=include_deleted)
            head_revision = None
            snapshot = None
            if project["head_revision_id"]:
                head_revision = conn.execute(
                    "SELECT * FROM revisions WHERE revision_id = ?",
                    (project["head_revision_id"],),
                ).fetchone()
                if head_revision:
                    snapshot = self.read_object_json(head_revision["content_sha256"])
            return {
                "schema": "seamgrim.project_record.v0",
                "project_id": project["project_id"],
                "title": project["title"],
                "source_lesson_id": project["source_lesson_id"],
                "head_revision_id": project["head_revision_id"],
                "origin_project_id": project["origin_project_id"],
                "origin_revision_id": project["origin_revision_id"],
                "created_at": project["created_at"],
                "updated_at": project["updated_at"],
                "deleted_at": project["deleted_at"],
                "content_sha256": head_revision["content_sha256"] if head_revision else "",
                "package_hash": head_revision["package_hash"] if head_revision else "",
                "project_snapshot": snapshot,
            }

    def list_revisions(self, project_id: str) -> list[dict[str, Any]]:
        with self.connect() as conn:
            self._require_project(conn, project_id)
            rows = conn.execute(
                """
                SELECT * FROM revisions
                WHERE project_id = ?
                ORDER BY created_at ASC, revision_id ASC
                """,
                (project_id,),
            ).fetchall()
            return [
                StoredRevision(
                    project_id=row["project_id"],
                    revision_id=row["revision_id"],
                    parent_revision_id=row["parent_revision_id"],
                    content_sha256=row["content_sha256"],
                    package_hash=row["package_hash"],
                    object_path=row["object_path"],
                    created_at=row["created_at"],
                    restore_of_revision_id=row["restore_of_revision_id"],
                ).to_dict()
                for row in rows
            ]

    def _require_run(self, conn: sqlite3.Connection, project_id: str, server_run_id: str) -> sqlite3.Row:
        row = conn.execute(
            "SELECT * FROM runs WHERE project_id = ? AND server_run_id = ?",
            (project_id, server_run_id),
        ).fetchone()
        if not row:
            raise ProjectNotFoundError("run not found")
        return row

    def _latest_run_checkpoint(self, conn: sqlite3.Connection, server_run_id: str) -> sqlite3.Row | None:
        return conn.execute(
            """
            SELECT * FROM run_checkpoints
            WHERE server_run_id = ?
            ORDER BY sequence DESC
            LIMIT 1
            """,
            (server_run_id,),
        ).fetchone()

    def create_run(
        self,
        project_id: str,
        *,
        source_revision_id: str,
        recipe_hash: str,
        expected_head_revision_id: str | None = None,
        seed: str = "",
        parent_server_run_id: str | None = None,
        branch_madi: int | None = None,
        branch_state_hash: str | None = None,
        server_run_id: str | None = None,
    ) -> dict[str, Any]:
        now = utc_now_iso()
        rid = server_run_id or new_id(RUN_ID_PREFIX)
        normalized_recipe = str(recipe_hash or "").strip()
        if not normalized_recipe:
            raise ProjectValidationError("recipe_hash is required")
        with self.connect() as conn:
            conn.execute("BEGIN IMMEDIATE")
            project = self._require_project(conn, project_id)
            normalized_expected_head = str(expected_head_revision_id or "").strip()
            if normalized_expected_head and normalized_expected_head != str(project["head_revision_id"] or ""):
                raise ProjectConflictError("expected run head revision does not match current head")
            source = self._require_revision(conn, project_id, source_revision_id)
            run_count = int(conn.execute("SELECT COUNT(*) FROM runs WHERE project_id = ?", (project_id,)).fetchone()[0])
            if run_count >= MAX_PROJECT_RUNS:
                raise ProjectLimitError(f"project run limit reached ({MAX_PROJECT_RUNS})")
            normalized_parent = str(parent_server_run_id or "").strip() or None
            normalized_branch_hash = str(branch_state_hash or "").strip() or None
            normalized_branch_madi = None if branch_madi is None else int(branch_madi)
            if normalized_parent:
                parent = self._require_run(conn, project_id, normalized_parent)
                if parent["finalized_at"] is None:
                    raise ProjectConflictError("parent run must be finalized before branching")
                parent_checkpoint = conn.execute(
                    """
                    SELECT * FROM run_checkpoints
                    WHERE server_run_id = ? AND committed_madi = ?
                    ORDER BY sequence DESC
                    LIMIT 1
                    """,
                    (normalized_parent, normalized_branch_madi),
                ).fetchone()
                if not parent_checkpoint:
                    raise ProjectValidationError("branch madi does not exist in parent run")
                checkpoint_json = self.read_object_json(parent_checkpoint["checkpoint_sha256"])
                state_hashes = checkpoint_json.get("state_hashes") if isinstance(checkpoint_json, dict) else []
                if normalized_branch_hash and normalized_branch_hash not in [str(item) for item in state_hashes or []]:
                    raise ProjectConflictError("branch_state_hash does not match parent checkpoint")
            elif normalized_branch_madi is not None or normalized_branch_hash:
                raise ProjectValidationError("branch metadata requires parent_server_run_id")
            conn.execute(
                """
                INSERT INTO runs(
                  server_run_id, project_id, source_revision_id, source_content_sha256,
                  recipe_hash, seed, parent_server_run_id, branch_madi, branch_state_hash,
                  final_status, created_at, finalized_at, latest_sequence, latest_committed_madi
                ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, 'running', ?, NULL, -1, -1)
                """,
                (
                    rid,
                    project_id,
                    source_revision_id,
                    source["content_sha256"],
                    normalized_recipe,
                    str(seed or ""),
                    normalized_parent,
                    normalized_branch_madi,
                    normalized_branch_hash,
                    now,
                ),
            )
            conn.commit()
        return self.get_run(project_id, rid)

    def add_run_checkpoint(
        self,
        project_id: str,
        server_run_id: str,
        checkpoint: dict[str, Any],
    ) -> dict[str, Any]:
        row = as_object(checkpoint)
        sequence = int(row.get("sequence"))
        committed_madi = int(row.get("committed_madi"))
        now = utc_now_iso()
        with self.connect() as conn:
            conn.execute("BEGIN IMMEDIATE")
            run = self._require_run(conn, project_id, server_run_id)
            if run["finalized_at"] is not None:
                raise ProjectConflictError("finalized run cannot be mutated")
            checkpoint_count = int(conn.execute(
                "SELECT COUNT(*) FROM run_checkpoints WHERE server_run_id = ?",
                (server_run_id,),
            ).fetchone()[0])
            if checkpoint_count >= MAX_RUN_CHECKPOINTS:
                raise ProjectLimitError(f"run checkpoint limit reached ({MAX_RUN_CHECKPOINTS})")
            if sequence <= int(run["latest_sequence"]):
                raise ProjectConflictError("checkpoint sequence must increase")
            if committed_madi <= int(run["latest_committed_madi"]):
                raise ProjectConflictError("committed madi must increase")
            latest = self._latest_run_checkpoint(conn, server_run_id)
            previous_sha = str(row.get("previous_checkpoint_sha256") or "")
            if latest and previous_sha != str(latest["checkpoint_sha256"]):
                raise ProjectConflictError("previous checkpoint sha mismatch")
            if not latest and previous_sha:
                raise ProjectConflictError("first checkpoint cannot have previous sha")
            artifact = stable_json({
                "schema": "seamgrim.run_checkpoint_artifact.v0",
                "server_run_id": server_run_id,
                "project_id": project_id,
                "sequence": sequence,
                "committed_madi": committed_madi,
                "previous_checkpoint_sha256": previous_sha,
                "input_snapshots": row.get("input_snapshots") if isinstance(row.get("input_snapshots"), list) else [],
                "state_hashes": row.get("state_hashes") if isinstance(row.get("state_hashes"), list) else [],
                "frame_summary": as_object(row.get("frame_summary")),
            })
            if len(artifact.encode("utf-8")) > MAX_CHECKPOINT_CANONICAL_BYTES:
                raise ProjectLimitError(f"run checkpoint too large ({MAX_CHECKPOINT_CANONICAL_BYTES} bytes)")
            checkpoint_sha = sha256_hex(artifact)
            object_path, byte_length = self._write_object_atomic(checkpoint_sha, artifact)
            conn.execute(
                "INSERT OR IGNORE INTO objects(content_sha256, byte_length, relative_path, created_at) VALUES(?, ?, ?, ?)",
                (checkpoint_sha, byte_length, object_path, now),
            )
            conn.execute(
                """
                INSERT INTO run_checkpoints(
                  server_run_id, sequence, project_id, committed_madi,
                  previous_checkpoint_sha256, checkpoint_sha256, object_path, created_at
                ) VALUES(?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (server_run_id, sequence, project_id, committed_madi, previous_sha, checkpoint_sha, object_path, now),
            )
            conn.execute(
                """
                UPDATE runs
                SET latest_sequence = ?, latest_committed_madi = ?
                WHERE server_run_id = ? AND project_id = ?
                """,
                (sequence, committed_madi, server_run_id, project_id),
            )
            conn.commit()
        return self.get_run(project_id, server_run_id)

    def finalize_run(self, project_id: str, server_run_id: str, *, final_status: str) -> dict[str, Any]:
        normalized = str(final_status or "").strip()
        if normalized not in {"stopped", "completed", "failed"}:
            raise ProjectValidationError("invalid final_status")
        now = utc_now_iso()
        with self.connect() as conn:
            conn.execute("BEGIN IMMEDIATE")
            run = self._require_run(conn, project_id, server_run_id)
            if run["finalized_at"] is not None:
                raise ProjectConflictError("run already finalized")
            conn.execute(
                "UPDATE runs SET final_status = ?, finalized_at = ? WHERE project_id = ? AND server_run_id = ?",
                (normalized, now, project_id, server_run_id),
            )
            conn.commit()
        return self.get_run(project_id, server_run_id)

    def list_runs(self, project_id: str) -> list[dict[str, Any]]:
        with self.connect() as conn:
            self._require_project(conn, project_id)
            rows = conn.execute(
                """
                SELECT * FROM runs
                WHERE project_id = ?
                ORDER BY created_at ASC, server_run_id ASC
                """,
                (project_id,),
            ).fetchall()
            return [self._run_row_to_dict(row, include_checkpoints=False) for row in rows]

    def _run_row_to_dict(self, row: sqlite3.Row, *, include_checkpoints: bool = True) -> dict[str, Any]:
        result = {
            "schema": "seamgrim.run_archive_record.v0",
            "server_run_id": row["server_run_id"],
            "project_id": row["project_id"],
            "source_revision_id": row["source_revision_id"],
            "source_content_sha256": row["source_content_sha256"],
            "recipe_hash": row["recipe_hash"],
            "seed": row["seed"],
            "parent_server_run_id": row["parent_server_run_id"],
            "branch_madi": row["branch_madi"],
            "branch_state_hash": row["branch_state_hash"],
            "final_status": row["final_status"],
            "created_at": row["created_at"],
            "finalized_at": row["finalized_at"],
            "latest_sequence": row["latest_sequence"],
            "latest_committed_madi": row["latest_committed_madi"],
        }
        if include_checkpoints:
            result["checkpoints"] = []
        return result

    def get_run(self, project_id: str, server_run_id: str) -> dict[str, Any]:
        with self.connect() as conn:
            run = self._require_run(conn, project_id, server_run_id)
            result = self._run_row_to_dict(run, include_checkpoints=True)
            rows = conn.execute(
                """
                SELECT * FROM run_checkpoints
                WHERE server_run_id = ?
                ORDER BY sequence ASC
                """,
                (server_run_id,),
            ).fetchall()
            checkpoints = []
            for row in rows:
                checkpoints.append({
                    "sequence": row["sequence"],
                    "committed_madi": row["committed_madi"],
                    "previous_checkpoint_sha256": row["previous_checkpoint_sha256"],
                    "checkpoint_sha256": row["checkpoint_sha256"],
                    "object_path": row["object_path"],
                    "created_at": row["created_at"],
                    "artifact": self.read_object_json(row["checkpoint_sha256"]),
                })
            result["checkpoints"] = checkpoints
            return result

    def restore_revision(
        self,
        project_id: str,
        source_revision_id: str,
        *,
        expected_parent_revision_id: str | None,
        revision_id: str | None = None,
    ) -> dict[str, Any]:
        with self.connect() as conn:
            self._require_project(conn, project_id)
            source = self._require_revision(conn, project_id, source_revision_id)
            snapshot = self.read_object_json(source["content_sha256"])
        artifact = {
            "schema": "seamgrim.project_snapshot_artifact.v0",
            "project_snapshot": snapshot,
            "canonical_json": stable_json(snapshot),
            "content_sha256": source["content_sha256"],
        }
        return self.add_revision(
            project_id,
            artifact,
            expected_parent_revision_id=expected_parent_revision_id,
            revision_id=revision_id,
            restore_of_revision_id=source_revision_id,
        )

    def clone_project(
        self,
        source_project_id: str,
        *,
        title: str = "",
        project_id: str | None = None,
        revision_id: str | None = None,
    ) -> dict[str, Any]:
        source = self.get_project(source_project_id)
        snapshot = dict(as_object(source["project_snapshot"]))
        snapshot["project_name"] = str(title or f"{source['title']} 복제본").strip()
        lineage = as_object(snapshot.get("lineage"))
        lineage.update({
            "source_project_id": source_project_id,
            "source_revision_id": source["head_revision_id"],
            "origin_content_sha256": source["content_sha256"],
            "origin_kind": "project_clone",
        })
        snapshot["lineage"] = lineage
        artifact = {
            "schema": "seamgrim.project_snapshot_artifact.v0",
            "project_snapshot": snapshot,
            "canonical_json": stable_json(snapshot),
            "content_sha256": sha256_hex(stable_json(snapshot)),
        }
        return self.create_project(
            artifact,
            title=snapshot["project_name"],
            project_id=project_id,
            revision_id=revision_id,
            origin_project_id=source_project_id,
            origin_revision_id=source["head_revision_id"],
        )

    def soft_delete_project(self, project_id: str) -> dict[str, Any]:
        now = utc_now_iso()
        with self.connect() as conn:
            conn.execute("BEGIN IMMEDIATE")
            self._require_project(conn, project_id)
            conn.execute(
                "UPDATE projects SET deleted_at = ?, updated_at = ? WHERE project_id = ? AND deleted_at IS NULL",
                (now, now, project_id),
            )
            conn.commit()
        return self.get_project(project_id, include_deleted=True)

    def store_capability(self, project_id: str, role: str, raw_capability: str) -> str:
        normalized_role = str(role or "").strip().lower()
        if normalized_role not in {ROLE_EDIT, ROLE_VIEW}:
            raise ProjectValidationError("invalid capability role")
        raw = str(raw_capability or "").strip()
        if len(raw) < 24:
            raise ProjectValidationError("capability too short")
        cap_hash = sha256_hex(raw)
        now = utc_now_iso()
        with self.connect() as conn:
            conn.execute("BEGIN IMMEDIATE")
            self._require_project(conn, project_id, include_deleted=True)
            conn.execute(
                """
                INSERT INTO capabilities(project_id, role, capability_sha256, created_at)
                VALUES(?, ?, ?, ?)
                ON CONFLICT(project_id, role) DO UPDATE SET
                  capability_sha256 = excluded.capability_sha256,
                  created_at = excluded.created_at
                """,
                (project_id, normalized_role, cap_hash, now),
            )
            conn.commit()
        return cap_hash

    def issue_capabilities(self, project_id: str) -> dict[str, str]:
        edit = new_capability()
        view = new_capability()
        self.store_capability(project_id, ROLE_EDIT, edit)
        self.store_capability(project_id, ROLE_VIEW, view)
        return {"edit": edit, "view": view}

    def authorize_capability(self, project_id: str, raw_capability: str, *, required_role: str = ROLE_VIEW) -> str:
        raw = str(raw_capability or "").strip()
        if not raw:
            raise ProjectPermissionError("capability required")
        required = str(required_role or ROLE_VIEW).strip().lower()
        if required not in {ROLE_EDIT, ROLE_VIEW}:
            raise ProjectValidationError("invalid required role")
        cap_hash = sha256_hex(raw)
        with self.connect() as conn:
            rows = conn.execute(
                """
                SELECT capabilities.role, projects.deleted_at
                FROM capabilities
                JOIN projects ON projects.project_id = capabilities.project_id
                WHERE capabilities.project_id = ? AND capabilities.capability_sha256 = ?
                """,
                (project_id, cap_hash),
            ).fetchall()
        if not rows:
            raise ProjectPermissionError("capability does not allow operation")
        if any(row["deleted_at"] for row in rows):
            raise ProjectDeletedError("project is deleted")
        roles = {str(row["role"]) for row in rows}
        if ROLE_EDIT in roles:
            return ROLE_EDIT
        if required == ROLE_VIEW and ROLE_VIEW in roles:
            return ROLE_VIEW
        raise ProjectPermissionError("capability does not allow operation")

    def audit_integrity(self) -> dict[str, Any]:
        diagnostics: list[dict[str, Any]] = []
        with self.connect() as conn:
            objects = conn.execute("SELECT * FROM objects ORDER BY content_sha256").fetchall()
            revisions = conn.execute("SELECT * FROM revisions ORDER BY project_id, created_at, revision_id").fetchall()
            projects = conn.execute("SELECT * FROM projects ORDER BY project_id").fetchall()
            capabilities = conn.execute("SELECT * FROM capabilities ORDER BY project_id, role").fetchall()
            runs = conn.execute("SELECT * FROM runs ORDER BY project_id, created_at, server_run_id").fetchall()
            checkpoints = conn.execute(
                "SELECT * FROM run_checkpoints ORDER BY server_run_id, sequence"
            ).fetchall()
            object_rows = {str(row["content_sha256"]): row for row in objects}
            revision_ids = {str(row["revision_id"]) for row in revisions}
            run_ids = {str(row["server_run_id"]) for row in runs}
            for row in objects:
                content_sha = str(row["content_sha256"])
                relative = str(row["relative_path"])
                target = self.data_dir / relative
                if not target.exists():
                    diagnostics.append({
                        "code": "E_PROJECT_OBJECT_MISSING",
                        "content_sha256": content_sha,
                        "relative_path": relative,
                    })
                    continue
                text = target.read_text(encoding="utf-8")
                actual_sha = sha256_hex(text)
                if actual_sha != content_sha:
                    diagnostics.append({
                        "code": "E_PROJECT_OBJECT_SHA_MISMATCH",
                        "content_sha256": content_sha,
                        "actual_sha256": actual_sha,
                        "relative_path": relative,
                    })
                if int(row["byte_length"] or 0) != len(text.encode("utf-8")):
                    diagnostics.append({
                        "code": "E_PROJECT_OBJECT_BYTE_LENGTH_MISMATCH",
                        "content_sha256": content_sha,
                        "relative_path": relative,
                    })
            for row in revisions:
                content_sha = str(row["content_sha256"])
                if content_sha not in object_rows:
                    diagnostics.append({
                        "code": "E_PROJECT_REVISION_OBJECT_REF_MISSING",
                        "project_id": row["project_id"],
                        "revision_id": row["revision_id"],
                        "content_sha256": content_sha,
                    })
                    continue
                expected_path = object_relative_path(content_sha)
                if Path(str(row["object_path"])).as_posix() != Path(expected_path).as_posix():
                    diagnostics.append({
                        "code": "E_PROJECT_REVISION_OBJECT_PATH_MISMATCH",
                        "project_id": row["project_id"],
                        "revision_id": row["revision_id"],
                        "object_path": row["object_path"],
                        "expected_object_path": expected_path,
                    })
                if row["parent_revision_id"] and str(row["parent_revision_id"]) not in revision_ids:
                    diagnostics.append({
                        "code": "E_PROJECT_REVISION_PARENT_REF_MISSING",
                        "project_id": row["project_id"],
                        "revision_id": row["revision_id"],
                        "parent_revision_id": row["parent_revision_id"],
                    })
                if row["restore_of_revision_id"] and str(row["restore_of_revision_id"]) not in revision_ids:
                    diagnostics.append({
                        "code": "E_PROJECT_REVISION_RESTORE_REF_MISSING",
                        "project_id": row["project_id"],
                        "revision_id": row["revision_id"],
                        "restore_of_revision_id": row["restore_of_revision_id"],
                    })
            for row in projects:
                if row["head_revision_id"] and str(row["head_revision_id"]) not in revision_ids:
                    diagnostics.append({
                        "code": "E_PROJECT_HEAD_REVISION_REF_MISSING",
                        "project_id": row["project_id"],
                        "head_revision_id": row["head_revision_id"],
                    })
            for row in runs:
                if str(row["source_revision_id"]) not in revision_ids:
                    diagnostics.append({
                        "code": "E_RUN_SOURCE_REVISION_REF_MISSING",
                        "project_id": row["project_id"],
                        "server_run_id": row["server_run_id"],
                        "source_revision_id": row["source_revision_id"],
                    })
                if str(row["source_content_sha256"]) not in object_rows:
                    diagnostics.append({
                        "code": "E_RUN_SOURCE_OBJECT_REF_MISSING",
                        "project_id": row["project_id"],
                        "server_run_id": row["server_run_id"],
                        "source_content_sha256": row["source_content_sha256"],
                    })
                if row["parent_server_run_id"] and str(row["parent_server_run_id"]) not in run_ids:
                    diagnostics.append({
                        "code": "E_RUN_PARENT_REF_MISSING",
                        "project_id": row["project_id"],
                        "server_run_id": row["server_run_id"],
                        "parent_server_run_id": row["parent_server_run_id"],
                    })
            for row in checkpoints:
                checkpoint_sha = str(row["checkpoint_sha256"])
                if checkpoint_sha not in object_rows:
                    diagnostics.append({
                        "code": "E_RUN_CHECKPOINT_OBJECT_REF_MISSING",
                        "project_id": row["project_id"],
                        "server_run_id": row["server_run_id"],
                        "checkpoint_sha256": checkpoint_sha,
                    })
                    continue
                expected_path = object_relative_path(checkpoint_sha)
                if Path(str(row["object_path"])).as_posix() != Path(expected_path).as_posix():
                    diagnostics.append({
                        "code": "E_RUN_CHECKPOINT_OBJECT_PATH_MISMATCH",
                        "project_id": row["project_id"],
                        "server_run_id": row["server_run_id"],
                        "object_path": row["object_path"],
                        "expected_object_path": expected_path,
                    })
            for row in capabilities:
                cap_hash = str(row["capability_sha256"] or "")
                if len(cap_hash) != 64 or any(ch not in "0123456789abcdef" for ch in cap_hash):
                    diagnostics.append({
                        "code": "E_PROJECT_CAPABILITY_HASH_INVALID",
                        "project_id": row["project_id"],
                        "role": row["role"],
                    })
        return {
            "schema": "seamgrim.project_store_integrity_audit.v0",
            "ok": not diagnostics,
            "diagnostics": diagnostics,
            "counts": {
                "projects": len(projects),
                "revisions": len(revisions),
                "runs": len(runs),
                "run_checkpoints": len(checkpoints),
                "objects": len(objects),
                "capability_hashes": len(capabilities),
            },
        }

    def backup_to(self, backup_dir: str | os.PathLike[str]) -> dict[str, Any]:
        audit = self.audit_integrity()
        if not audit["ok"]:
            raise ProjectValidationError(f"project store integrity audit failed: {audit['diagnostics']}")
        target = Path(backup_dir).resolve()
        target.mkdir(parents=True, exist_ok=True)
        db_target = target / self.db_path.name
        with self.connect() as conn:
            backup_conn = sqlite3.connect(db_target)
            try:
                conn.backup(backup_conn)
            finally:
                backup_conn.close()
        objects_target = target / "objects"
        if objects_target.exists():
            shutil.rmtree(objects_target)
        shutil.copytree(self.objects_dir, objects_target)
        return {
            "schema": "seamgrim.project_store_backup.v0",
            "db": str(db_target),
            "objects": str(objects_target),
            "audit": audit,
        }

    @staticmethod
    def restore_from_backup(
        backup_dir: str | os.PathLike[str],
        data_dir: str | os.PathLike[str],
        *,
        sqlite_synchronous: str | None = None,
    ) -> "SeamgrimProjectStore":
        source = Path(backup_dir).resolve()
        target = Path(data_dir).resolve()
        target.mkdir(parents=True, exist_ok=True)
        db_source = source / "seamgrim_projects.sqlite3"
        objects_source = source / "objects"
        if not db_source.exists() or not objects_source.exists():
            raise ProjectValidationError("backup missing db or objects")
        shutil.copy2(db_source, target / db_source.name)
        objects_target = target / "objects"
        if objects_target.exists():
            shutil.rmtree(objects_target)
        shutil.copytree(objects_source, objects_target)
        restored = SeamgrimProjectStore(target, sqlite_synchronous=sqlite_synchronous)
        audit = restored.audit_integrity()
        if not audit["ok"]:
            raise ProjectValidationError(f"restored project store integrity audit failed: {audit['diagnostics']}")
        return restored
