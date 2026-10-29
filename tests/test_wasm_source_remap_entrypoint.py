from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "build_wasm_tool.ps1"


class WasmSourceRemapEntrypointTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="ddn-wasm-entrypoint-")
        self.base = Path(self.temp.name)
        self.bin = self.base / "bin"
        self.profile = self.base / "profile"
        self.cargo_home = self.profile / ".cargo"
        self.registry = self.cargo_home / "registry" / "src"
        self.sysroot = self.profile / ".rustup" / "toolchains" / "test-toolchain"
        for path in (self.bin, self.registry, self.sysroot):
            path.mkdir(parents=True, exist_ok=True)
        # Mocked build-entrypoint fixture only, not a fixed-source product seal.
        # Patch only the Git cleanliness response in the fake compiler's Python
        # process. Other Git reads are real and no index is changed. A .cmd Git
        # shim alone is not reliable with Windows CreateProcess executable search.
        python_fixture = self.bin / "python_fixture.py"
        python_fixture.write_text(
            'import os, runpy, subprocess, sys\n'
            'from unittest.mock import patch\n'
            'real_run = subprocess.run\n'
            'def fixture_run(args, *rest, **kw):\n'
            '    if args == ["git", "--no-optional-locks", "diff", "--quiet", "HEAD", "--"]:\n'
            '        return subprocess.CompletedProcess(args, int(os.environ["FIXTURE_GIT_DIFF_EXIT"]), b"", b"")\n'
            '    if args[:6] == ["git", "--no-optional-locks", "ls-files", "--others", "--exclude-standard", "-z"]:\n'
            '        return subprocess.CompletedProcess(args, 0, b"", b"")\n'
            '    return real_run(args, *rest, **kw)\n'
            'sys.argv = sys.argv[1:]\n'
            'with patch("subprocess.run", side_effect=fixture_run):\n'
            '    runpy.run_path(sys.argv[0], run_name="__main__")\n', encoding="utf-8",
        )
        (self.bin / "python.cmd").write_text(f'@"{sys.executable}" "{python_fixture}" %*\r\n', encoding="utf-8")
        (self.bin / "rustc.cmd").write_text(
            f'@echo off\r\nif "%1"=="--print" (echo {self.sysroot}& exit /b 0)\r\necho rustc fixture 1.0\r\n',
            encoding="utf-8",
        )
        (self.bin / "cargo.cmd").write_text(
            "@echo off\r\n"
            "if \"%1\"==\"--version\" (echo cargo fixture 1.0& exit /b 0)\r\n"
            "> \"%CAPTURE_CARGO_ARGS%\" echo %*\r\n"
            "if not exist \"%CARGO_TARGET_DIR%\\wasm32-unknown-unknown\\release\" mkdir \"%CARGO_TARGET_DIR%\\wasm32-unknown-unknown\\release\"\r\n"
            "> \"%CARGO_TARGET_DIR%\\wasm32-unknown-unknown\\release\\ddonirang_tool.wasm\" echo wasm fixture\r\n"
            "exit /b 0\r\n",
            encoding="utf-8",
        )
        (self.bin / "wasm-bindgen.cmd").write_text(
            "@echo off\r\n"
            "if \"%1\"==\"--version\" (echo wasm-bindgen fixture 1.0& exit /b 0)\r\n"
            "> \"%CAPTURE_BINDGEN_ARGS%\" echo %*\r\n"
            "set out=\r\n"
            ":loop\r\n"
            "if \"%~1\"==\"\" goto done\r\n"
            "if \"%~1\"==\"--out-dir\" (set out=%~2& shift& shift& goto loop)\r\n"
            "shift\r\n"
            "goto loop\r\n"
            ":done\r\n"
            "if not exist \"%out%\" mkdir \"%out%\"\r\n"
            "> \"%out%\\.gitignore\" echo.\r\n"
            "> \"%out%\\ddonirang_tool.d.ts\" echo fixture\r\n"
            "> \"%out%\\ddonirang_tool.js\" echo fixture\r\n"
            "> \"%out%\\ddonirang_tool_bg.wasm\" echo fixture\r\n"
            "> \"%out%\\ddonirang_tool_bg.wasm.d.ts\" echo fixture\r\n"
            "> \"%out%\\package.json\" echo {}\r\n"
            "exit /b 0\r\n",
            encoding="utf-8",
        )
        (self.bin / "wasm-pack.cmd").write_text(
            "@echo off\r\n"
            "if \"%1\"==\"--version\" (echo wasm-pack fixture 1.0& exit /b 0)\r\n"
            "> \"%CAPTURE_RUSTFLAGS%\" echo(%RUSTFLAGS%\r\n"
            "> \"%CAPTURE_ENCODED%\" echo(%CARGO_ENCODED_RUSTFLAGS%\r\n"
            "set out=\r\n:loop\r\n"
            "if \"%~1\"==\"\" goto done\r\n"
            "if \"%~1\"==\"--out-dir\" (set out=%~2& shift& shift& goto loop)\r\n"
            "shift\r\ngoto loop\r\n:done\r\n"
            "if not exist \"%out%\" mkdir \"%out%\"\r\n"
            "> \"%out%\\.gitignore\" echo.\r\n"
            "> \"%out%\\ddonirang_tool.d.ts\" echo fixture\r\n"
            "> \"%out%\\ddonirang_tool.js\" echo fixture\r\n"
            "> \"%out%\\ddonirang_tool_bg.wasm\" echo fixture\r\n"
            "> \"%out%\\ddonirang_tool_bg.wasm.d.ts\" echo fixture\r\n"
            "> \"%out%\\package.json\" echo {}\r\n"
            "exit /b 0\r\n",
            encoding="utf-8",
        )

    def tearDown(self) -> None:
        self.temp.cleanup()

    def environment(self) -> dict[str, str]:
        env = dict(os.environ)
        env["PATH"] = f"{self.bin}{os.pathsep}{env.get('PATH', '')}"
        env["USERPROFILE"] = str(self.profile)
        env["RUSTUP_HOME"] = str(self.profile / ".rustup")
        env["RUSTUP_TOOLCHAIN"] = "test-toolchain"
        env["CARGO_HOME"] = str(self.cargo_home)
        env["CARGO_NET_OFFLINE"] = "true"
        env["DDN_BUILD_DIR"] = str(self.base / "build")
        env["DDN_WASM_CANONICAL_SOURCE_MATERIALIZED"] = "1"
        env["FIXTURE_GIT_DIFF_EXIT"] = "0"
        env["CAPTURE_RUSTFLAGS"] = str(self.base / "rustflags.txt")
        env["CAPTURE_ENCODED"] = str(self.base / "encoded.txt")
        env["CAPTURE_CARGO_ARGS"] = str(self.base / "cargo-args.txt")
        env["CAPTURE_BINDGEN_ARGS"] = str(self.base / "bindgen-args.txt")
        for name in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
            env.pop(name, None)
        return env

    def run_entrypoint(self, env: dict[str, str], mode: str, copy: bool = False, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
        powershell = Path(os.environ["SystemRoot"]) / "System32" / "WindowsPowerShell" / "v1.0" / "powershell.exe"
        return subprocess.run(
            [
                str(powershell), "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command",
                f"& '{SCRIPT}' -Profile release -Target web -CopyToUi ${str(copy).lower()} -SourceRemapMode {mode}",
            ],
            cwd=cwd or ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
            text=True, encoding="utf-8", errors="replace",
        )

    def test_default_mode_preserves_repository_only_contract(self) -> None:
        env = self.environment()
        result = self.run_entrypoint(env, "default")
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertEqual(Path(env["CAPTURE_RUSTFLAGS"]).read_text().strip(), f"--remap-path-prefix={ROOT}=/ddn-src")
        self.assertEqual(Path(env["CAPTURE_ENCODED"]).read_text().strip(), "")
        self.assertFalse((self.base / "build" / "wasm" / "ddonirang_tool" / "R3N_WASM_REMAP_RECEIPT.detjson").exists())

    def test_candidate_mode_uses_exact_three_mappings_and_seals(self) -> None:
        env = self.environment()
        result = self.run_entrypoint(env, "candidate", cwd=self.base)
        self.assertEqual(result.returncode, 0, result.stdout)
        flags = Path(env["CAPTURE_ENCODED"]).read_bytes().strip().decode().split("\x1f")
        self.assertEqual(
            flags,
            [
                f"--remap-path-prefix={ROOT}=/ddn-src",
                f"--remap-path-prefix={self.registry}=/ddn-cargo-registry",
                f"--remap-path-prefix={self.sysroot}=/ddn-rust-toolchain",
            ],
        )
        receipt = json.loads(
            (self.base / "build" / "wasm" / "ddonirang_tool" / "R3N_WASM_REMAP_RECEIPT.detjson").read_text()
        )
        receipt_text = json.dumps(receipt, ensure_ascii=False)
        self.assertNotIn(str(self.base), receipt_text)
        self.assertEqual(receipt["path_scan"]["internal_absolute_path_count"], 0)

    def test_candidate_inherited_flags_fail_before_build(self) -> None:
        env = self.environment()
        env["RUSTFLAGS"] = "-Copt-level=0"
        result = self.run_entrypoint(env, "candidate")
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("E_WASM_BUILD_INHERITED_RUSTFLAGS_UNSUPPORTED", result.stdout)
        self.assertFalse(Path(env["CAPTURE_ENCODED"]).exists())

    def test_candidate_dirty_source_is_not_sealed_despite_materialization_flag(self) -> None:
        env = self.environment()
        env["FIXTURE_GIT_DIFF_EXIT"] = "1"
        result = self.run_entrypoint(env, "candidate")
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("E_WASM_CANDIDATE_SOURCE_DIRTY", result.stdout)
        self.assertFalse((self.base / "build" / "wasm" / "ddonirang_tool" / "R3N_WASM_REMAP_RECEIPT.detjson").exists())

    def test_candidate_copy_to_ui_is_forbidden_before_build(self) -> None:
        env = self.environment()
        result = self.run_entrypoint(env, "candidate", copy=True)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("E_WASM_CANDIDATE_TRACKED_COPY_FORBIDDEN", result.stdout)
        self.assertFalse(Path(env["CAPTURE_ENCODED"]).exists())

    def test_candidate_local_bindgen_fallback_builds_only_library_and_seals_full_family(self) -> None:
        env = self.environment()
        env["CARGO_TARGET_DIR"] = str(self.base / "cargo-target")
        (self.bin / "wasm-pack.cmd").unlink()
        git = shutil.which("git")
        self.assertIsNotNone(git)
        env["PATH"] = os.pathsep.join((str(self.bin), str(Path(git).parent)))
        result = self.run_entrypoint(env, "candidate")
        self.assertEqual(result.returncode, 0, result.stdout)
        cargo_args = Path(env["CAPTURE_CARGO_ARGS"]).read_text(encoding="utf-8")
        bindgen_args = Path(env["CAPTURE_BINDGEN_ARGS"]).read_text(encoding="utf-8")
        self.assertIn("--lib", cargo_args)
        self.assertNotIn("--no-typescript", bindgen_args)
        output = self.base / "build" / "wasm" / "ddonirang_tool"
        self.assertEqual((output / ".gitignore").read_bytes(), b"*\n")
        package = json.loads((output / "package.json").read_text(encoding="utf-8"))
        self.assertEqual(package["name"], "ddonirang-tool")
        self.assertEqual(package["version"], "0.1.0")
        self.assertEqual(package["types"], "ddonirang_tool.d.ts")
        self.assertTrue((output / "ddonirang_tool.d.ts").is_file())
        self.assertTrue((output / "R3N_WASM_REMAP_RECEIPT.detjson").is_file())

    def test_candidate_contract_owns_lf_fixed_head_materialization(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        self.assertIn("git clone --quiet --shared --no-checkout", source)
        self.assertIn("config core.autocrlf false", source)
        self.assertIn("config core.longpaths true", source)
        self.assertIn("checkout --quiet --detach $sourceHead", source)
        self.assertIn("E_WASM_CANDIDATE_MATERIALIZATION_INCOMPLETE", source)
        self.assertIn("E_WASM_CANDIDATE_MATERIALIZATION_TARGET_EXISTS", source)


if __name__ == "__main__":
    unittest.main()
