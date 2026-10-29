"""Offline build/CLI proof without private inputs; NOT a publication allowlist.

The projection is a fresh validation-only directory. Never copies ignored or
untracked inputs, docs, tool assets, or the legacy teul-cli source tree.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess


CRATES = {"core", "lang", "numeric", "proof", "symbolic", "tool"}
FIXTURES = {"pack/v1_core_supported_ingress_w1/fixtures/" + name for name in (
    "positive_definition_rebinding.ddn", "legacy_definition_equals.ddn",
    "direct_recursion.ddn", "mutual_recursion.ddn",
    "strict_selection_out_of_range.ddn", "value_type_mismatch.ddn",
    "language_core_collection.ddn",
)}


def selected(path):
    parts = Path(path).parts
    return (
        path in FIXTURES
        or path in {"Cargo.toml", "Cargo.lock", ".cargo/config.toml",
                 "tool/tests/p0_official_cli.rs"}
        or (len(parts) >= 2 and parts[0] in CRATES
            and (parts[1] == "src" or parts[1] == "Cargo.toml"))
    )


def run(args, cwd, env, log):
    with log.open("wb") as stream:
        result = subprocess.run(args, cwd=cwd, env=env, stdout=stream,
                                stderr=subprocess.STDOUT)
    if result.returncode:
        raise RuntimeError(f"실행 실패 ({result.returncode}): {args}; 기록: {log}")
    return log.read_bytes()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, required=True)
    args = parser.parse_args()
    root = args.source_root.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    source = output / "source"
    source.mkdir()
    paths = subprocess.check_output(["git", "ls-files", "-z"], cwd=root).decode().split("\0")
    manifest = {}
    for name in sorted(filter(selected, filter(None, paths))):
        target = source / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(root / name, target)
        manifest[name] = hashlib.sha256(target.read_bytes()).hexdigest()
    assert not (source / "docs").exists()
    assert not (source / "tool/assets").exists()
    assert not (source / "tools/teul-cli").exists()
    (output / "source-manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    env = dict(os.environ, CARGO_TARGET_DIR=str(args.target_dir.resolve()), CARGO_BUILD_JOBS="1")
    command = ["cargo", "test", "-p", "ddonirang-tool", "--offline", "--locked",
               "--no-default-features", "--test", "p0_official_cli"]
    run(command, source, env, output / "official-cli.log")
    # Cargo's built target list must omit the internal management binary.
    metadata = json.loads(run(["cargo", "metadata", "--offline", "--no-deps",
                              "--format-version", "1", "--no-default-features"],
                             source, env, output / "metadata.json"))
    tool = next(p for p in metadata["packages"] if p["name"] == "ddonirang-tool")
    internal = next(t for t in tool["targets"] if t["name"] == "ddonirang-tool")
    assert internal.get("required-features") == ["internal-tools"]
    assert tool["features"]["default"] == ["internal-tools"], "내부 빌드 호환성 변경"
    assert tool["features"]["internal-tools"] == []
    # Opting into internal tools without the private template must fail closed.
    negative = subprocess.run(["cargo", "check", "-p", "ddonirang-tool", "--offline",
                               "--locked", "--features", "internal-tools", "--lib"],
                              cwd=source, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (output / "missing-private-input.log").write_bytes(negative.stdout)
    assert negative.returncode != 0, "내부 자료 없이 내부 기능 성공 금지"
    assert b"ai_prompt_template_v20.0.3.txt" in negative.stdout
    print("P5_PUBLIC_BUILD_BOUNDARY: PASS; PUBLICATION_AUTHORIZATION: NONE")


if __name__ == "__main__":
    main()
