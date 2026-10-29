from __future__ import annotations

import hashlib
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUNTIME = ROOT / "tool" / "src" / "ddn_runtime.rs"
TEST = ["cargo", "test", "-p", "ddonirang-tool", "p1a_", "--", "--nocapture"]

MUTATIONS = [
    (
        "PROFILE_IDENTITY_DISABLED",
        b'Some(V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY) => Ok(SupportedSourceProfile::V1CoreV25),',
        b'Some("v1-core-v25-disabled") => Ok(SupportedSourceProfile::V1CoreV25),',
        (
            "p1a_v1_core_profile_accepts_definition_and_rebinding",
            "p1a_supported_profile_run_executes_v25_definition_and_rebinding",
        ),
    ),
    (
        "PREV25_PATH_SUBSTITUTION",
        b'let program = Self::from_v25_source_with_mode(source, file_path, mode)?;',
        b'let program = Self::from_source_with_mode(source, file_path, mode)?;',
        (
            "p1a_v1_core_profile_accepts_definition_and_rebinding",
            "p1a_supported_profile_run_executes_v25_definition_and_rebinding",
        ),
    ),
    (
        "HIDDEN_PREV25_RETRY",
        b'let program = Self::from_v25_source_with_mode(source, file_path, mode)?;',
        (
            b'let program = Self::from_v25_source_with_mode(source, file_path, mode)'
            b'.or_else(|_| Self::from_source_with_mode(source, file_path, mode))?;'
        ),
        (
            "p1a_v1_core_profile_reports_exact_legacy_definition_migration",
            "p1a_supported_profile_run_fails_closed_for_profile_and_v25_boundaries",
        ),
    ),
    (
        "RECURSION_GUARD_BYPASS",
        b'validate_v1_core_no_named_callable_recursion(&program.program)?;',
        b'let _recursion_guard_bypassed = &program.program;',
        ("p1a_v1_core_profile_rejects_direct_and_mutual_recursion_deterministically",),
    ),
]


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run_focused() -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(TEST, cwd=ROOT, capture_output=True)


def fail_with_output(message: str, completed: subprocess.CompletedProcess[bytes]) -> None:
    output = completed.stdout + completed.stderr
    raise SystemExit(f"{message}\n{output.decode('utf-8', errors='replace')}")


def main() -> None:
    original = RUNTIME.read_bytes()
    original_hash = sha256(original)
    runner_hash = sha256(Path(__file__).read_bytes())
    print(f"RUNNER_SOURCE_SHA256={runner_hash}")
    print(f"RUNTIME_RESTORE_SHA256={original_hash}")
    try:
        for mutation_id, needle, replacement, expected_red_markers in MUTATIONS:
            if original.count(needle) != 1:
                raise SystemExit(f"{mutation_id}: expected one mutation target")
            RUNTIME.write_bytes(original.replace(needle, replacement, 1))
            completed = run_focused()
            if completed.returncode == 0:
                fail_with_output(f"{mutation_id}: mutation unexpectedly stayed GREEN", completed)
            output = (completed.stdout + completed.stderr).decode("utf-8", errors="replace")
            missing_markers = [marker for marker in expected_red_markers if marker not in output]
            if missing_markers:
                fail_with_output(
                    f"{mutation_id}: RED output missing markers {missing_markers}", completed
                )
            RUNTIME.write_bytes(original)
            if sha256(RUNTIME.read_bytes()) != original_hash:
                raise SystemExit(f"{mutation_id}: exact restore failed")
            restored = run_focused()
            if restored.returncode != 0:
                fail_with_output(f"{mutation_id}: restored source is not GREEN", restored)
            print(
                f"{mutation_id}: RED exit={completed.returncode} "
                f"signature={','.join(expected_red_markers)} -> exact restore -> GREEN exit=0"
            )
    finally:
        RUNTIME.write_bytes(original)

    completed = run_focused()
    if completed.returncode != 0:
        fail_with_output("final restored source is not GREEN", completed)
    if sha256(RUNTIME.read_bytes()) != original_hash:
        raise SystemExit("final source hash differs after restore")
    print(f"P1A_MUTATION_PASS mutations={len(MUTATIONS)} restore_sha256={original_hash}")


if __name__ == "__main__":
    main()
