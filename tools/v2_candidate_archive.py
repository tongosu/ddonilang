"""고정 소스·검증된 빌드 부품으로 비공개 V2 후보를 한 번에 묶는다.

실행 시나리오·독립검토·Human 판정은 발급하지 않는다. Git 작업 파일,
공유 index, 원격 저장소, 실행물 bytes와 기존 묶음은 변경하지 않는다.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import zipfile

from alpha2_completion_archive import native_workbench_payload, read_notice_bundle


class PackageError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise PackageError(message)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def encode(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8")


def member_path(value):
    require(isinstance(value, str) and value and "\\" not in value and ":" not in value
            and not any(ord(char) < 32 or ord(char) == 127 for char in value),
            "묶음 경로는 상대 POSIX 경로여야 합니다")
    path = PurePosixPath(value)
    require(not path.is_absolute() and all(p not in ("", ".", "..") for p in value.split("/")),
            "묶음 경로가 안전하지 않습니다: " + value)
    return value


def pointer(value, path):
    require(isinstance(path, str) and path.startswith("/"), "빌드 기록의 JSON pointer가 필요합니다")
    try:
        for part in path[1:].split("/"):
            key = part.replace("~1", "/").replace("~0", "~")
            value = value[int(key)] if isinstance(value, list) else value[key]
    except (KeyError, IndexError, ValueError, TypeError) as error:
        raise PackageError("빌드 기록에 지정한 값이 없습니다: " + path) from error
    return value


def checked_file(entry, base):
    path = (base / entry["file"]).resolve()
    raw = path.read_bytes()
    require(digest(raw) == entry["sha256"], "입력 파일 해시 불일치: " + str(path))
    return raw


def build_source(record, requested_pointer):
    # 이전 판 이력의 소스 값으로 현재 실행물의 출처를 대신할 수 없다.
    candidates = ("/compiled_runtime_source_commit",
                  "/reused_runtime/compiled_runtime_source_commit", "/source_commit", "/source")
    for candidate in candidates:
        try:
            value = pointer(record, candidate)
        except PackageError:
            continue
        require(requested_pointer == candidate, "현재 컴파일 출처 대신 다른 이력 필드를 선택했습니다")
        require(isinstance(value, str) and re.fullmatch(r"[a-f0-9]{40}", value) is not None,
                "실제 컴파일 소스 commit을 확인할 수 없습니다")
        return value
    raise PackageError("지원하는 실제 컴파일 소스 기록이 없습니다")


def source_archive(repo, source, paths=()):
    command = ["git", "--no-optional-locks", "-C", str(repo), "archive", "--format=zip", source]
    if paths:
        command += ["--", *paths]
    return subprocess.check_output(command)


def source_payload(raw):
    payload = {}
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        for item in archive.infolist():
            if item.is_dir():
                continue
            name = member_path(item.filename)
            require((item.external_attr >> 16) & 0o170000 != 0o120000,
                    "실행 묶음의 심볼릭 링크는 지원하지 않습니다: " + name)
            require(name not in payload, "중복 묶음 경로: " + name)
            payload[name] = archive.read(item)
    require(len({path.casefold() for path in payload}) == len(payload),
            "Windows에서 같은 이름이 되는 경로가 있습니다")
    return payload


def personal_path_scan(payload):
    # 검사기 자신의 설명 문자열을 검사 자료에 섞지 않는다. 보고서는 ZIP 밖에 둔다.
    names = ("ddon" + "gosu", "uri" + "hanl")
    hits = []
    for path, raw in sorted(payload.items()):
        folded = raw.lower()
        for name in names:
            for encoding in ("utf-8", "utf-16-le", "utf-16-be"):
                needle = name.encode(encoding)
                offset, count = folded.find(needle), folded.count(needle)
                if count:
                    hits.append({"path": path, "pattern": name, "encoding": encoding,
                                 "count": count, "first_byte_offset": offset})
    return hits


def checksums(payload):
    return "".join(f"{digest(raw)}  {path}\n" for path, raw in sorted(payload.items())
                   if path != "SHA256SUMS.txt").encode("utf-8")


def zip_bytes(payload, name):
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, "w", zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
        for path, raw in sorted(payload.items()):
            info = zipfile.ZipInfo(name + "/" + member_path(path), (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, raw)
    raw = stream.getvalue()
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        require(archive.testzip() is None, "완성 ZIP CRC 검사 실패")
        require(len(archive.namelist()) == len(payload), "완성 ZIP 파일 수 불일치")
        for path, expected in payload.items():
            require(archive.read(name + "/" + path) == expected, "완성 ZIP bytes 불일치: " + path)
    return raw


def add_components(payload, recipe, base, source):
    bindings = []
    kinds = set()
    for component in recipe["components"]:
        kind = component["kind"]
        require(kind in ("native", "lsp", "wasm"), "알 수 없는 실행물 종류")
        require(kind not in kinds, "중복 실행물 종류")
        kinds.add(kind)
        record_bytes = checked_file(component["record"], base)
        record = json.loads(record_bytes)
        require(build_source(record, component["source_pointer"]) == source,
                kind + " 빌드와 묶음의 고정 소스가 다릅니다")
        bound = []
        for item in component["files"]:
            target = member_path(item["path"])
            require((kind == "native" and target in ("bin/ddonilang.exe", "bin/ddonirang.exe", "bin/또니랑.exe"))
                    or (kind == "lsp" and target == "bin/ddonirang-lsp.exe")
                    or (kind == "wasm" and target.startswith("solutions/seamgrim_ui_mvp/ui/wasm/")),
                    "배정된 실행물 위치가 아닙니다: " + target)
            raw = checked_file(item, base)
            if kind == "wasm" and target.endswith("/ddonirang_tool.manifest.detjson"):
                require(raw == record_bytes, "WASM manifest는 이미 검증한 원 빌드 기록이어야 합니다")
            else:
                require(pointer(record, item["record_sha256_pointer"]) == digest(raw),
                        "실제 빌드 기록과 실행물 해시가 다릅니다: " + target)
            require(target not in payload or (kind == "wasm" and target.endswith("/package.json")),
                    "실행물 중복 또는 오래된 WASM 혼입: " + target)
            payload[target] = raw
            bound.append({"path": target, "bytes": len(raw), "sha256": digest(raw)})
        bindings.append({"kind": kind, "source": source, "record_sha256": digest(record_bytes), "files": bound})
    required = {"bin/ddonilang.exe", "bin/ddonirang.exe", "bin/또니랑.exe", "bin/ddonirang-lsp.exe",
                "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.js",
                "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool_bg.wasm",
                "solutions/seamgrim_ui_mvp/ui/wasm/package.json",
                "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.manifest.detjson"}
    require(kinds == {"native", "lsp", "wasm"} and required <= payload.keys(),
            "CLI 세 이름·LSP·WASM의 같은 소스 빌드 부품이 모두 필요합니다")
    return bindings


def support_rows(recipe, track):
    rows = recipe["support"]
    required_rows = {f"V2-{n:02}" for n in range(1, 7)} if track == "V2" else {f"M{n}" for n in range(1, 8)}
    require(len(rows) == len(required_rows) and {row["id"] for row in rows} == required_rows,
            "해당 후보의 필수 지원·제한 행을 각각 제공해야 합니다")
    for row in rows:
        require(row.get("supported") and row.get("limits"), "지원과 제한을 모두 기록해야 합니다")
    return rows


def build(recipe_path, repo, out, require_clean_paths=False):
    recipe_raw = recipe_path.read_bytes()
    recipe = json.loads(recipe_raw)
    require(recipe.get("visibility") == "private", "이 도구는 비공개 후보만 만듭니다")
    track = recipe.get("track", "V2")
    require(track in ("V2", "V3"), "지원하는 후보 계열은 V2·V3입니다")
    source = recipe["source_commit"]
    require(re.fullmatch(r"[a-f0-9]{40}", source) is not None, "고정 commit 전체 값이 필요합니다")
    name = recipe["package_name"]
    require(re.fullmatch(r"DDN-[A-Za-z0-9._-]+", name) is not None, "안전한 묶음 이름이 필요합니다")
    require(not out.exists(), "기존 고정 묶음은 덮어쓰지 않습니다")
    paths = [member_path(path) for path in recipe["source_paths"]]
    require("solutions/seamgrim_ui_mvp" in paths, "공식 작업대·교재 원문이 필요합니다")
    require(not any(path == root or path.startswith(root + "/")
                    for path in paths for root in ("work", "docs", ".git")),
            "내부 작업 기록·정본은 실행 묶음에 넣지 않습니다")
    payload = source_payload(source_archive(repo, source, paths))
    # Git에 남은 과거 생성 WASM은 사용하지 않고 실제 같은 소스 빌드 기록으로만 공급한다.
    payload = {path: raw for path, raw in payload.items()
               if (not path.startswith("solutions/seamgrim_ui_mvp/ui/wasm/")
                   or path == "solutions/seamgrim_ui_mvp/ui/wasm/package.json")
               and not path.startswith("solutions/seamgrim_ui_mvp/tools/")}
    require("LICENSE" in payload, "기존 프로젝트 LICENSE를 포함해야 합니다")
    lock = subprocess.check_output(["git", "--no-optional-locks", "-C", str(repo), "show", source + ":Cargo.lock"])
    notice = recipe["notice_bundle"]
    notice_payload, notice_evidence = read_notice_bundle(
        (recipe_path.parent / notice["folder"]).resolve(), digest(lock), notice["manifest_sha256"])
    require(not payload.keys() & notice_payload.keys(), "제삼자 고지를 중복 또는 다른 출처로 공급했습니다")
    payload.update(notice_payload)
    bindings = add_components(payload, recipe, recipe_path.parent, source)
    rows = support_rows(recipe, track)
    support = [f"# {track} 비공개 후보 지원 범위", "", "구현 포함과 독립검토·또니 확인은 별도입니다.", ""]
    support += [f"- **{row['id']}**: {row['supported']} / 제한: {row['limits']}" for row in rows]
    support += ["", "- Linux — CLI·작업대 실행 지원, 보호 저장 미지원(다음 판)",
                "- V2-07: 부분 구현·후속 설계는 핵심 6개 완료 판정과 별도입니다.",
                "- V2-08: 다음 판입니다.", ""]
    if track == "V3":
        support = support[:-4] + ["", "- Linux 보호 저장: 다음 판. V2의 기존 제한·P5/P6 미완은 유지합니다.", "- M1~M7 포함 여부와 실제 실행·독립검토·Human은 별도입니다.", ""]
    payload[track + "_SUPPORT.md"] = "\n".join(support).encode("utf-8")
    require(recipe.get("known_limitations"), "기존 실패·제한·미실행 목록이 필요합니다")
    payload[track + "_KNOWN_LIMITATIONS.md"] = ("# 알려진 제한\n\n" + "\n".join(
        "- " + text for text in recipe["known_limitations"]) +
        "\n- P5/P6·학생·물리 OS IME·Human·공개 판정은 이 묶음으로 닫지 않습니다.\n").encode("utf-8")
    payload["START_SEAMGRIM.cmd"] = (b'@echo off\r\nsetlocal\r\ncd /d "%~dp0"\r\n'
        b'"%~dp0bin\\ddonilang.exe" workbench\r\nif errorlevel 1 (pause & exit /b 1)\r\nexit /b %errorlevel%\r\n')
    payload[track + "_START.md"] = (f"# {track} 비공개 후보 시작\n\n새 로컬 폴더에 모두 푼 뒤 START_SEAMGRIM.cmd를 엽니다. "
        "Python·Rust 설치는 필요 없습니다. 첫걸음·공식 모음의 학습 안내에서 교재와 예제를 엽니다.\n\n"
        "원문 저장과 세계 저장은 다릅니다. 완료 안내를 기다리고 종료하며, 재열기 후 필요한 검사를 명시적으로 실행합니다. "
        "종료는 서버 창에서 Enter입니다. Native 저장본이 USB에 자동으로 따라다니지는 않습니다.\n\n"
        "V2_SUPPORT.md와 V2_KNOWN_LIMITATIONS.md를 함께 읽으세요. "
        "핵심6 실제 시나리오 실행·리뷰 B/C·또니 확인은 별도입니다.\n").encode("utf-8")
    if track == "V3":
        payload["V3_START.md"] = ("# V3 비공개 탐구노트 후보\n\n새 폴더에 모두 풀고 START_SEAMGRIM.cmd를 엽니다. "
            "공급된 공식 탐구노트와 V3_SUPPORT.md·V3_KNOWN_LIMITATIONS.md를 읽습니다. "
            "원문 저장 후 정상 종료하고, 재열기 후 명시 재검사·실행합니다. "
            "과거 결과를 현재 증거로 사용하지 않습니다. Native 저장이 USB에 자동으로 따라다니지는 않습니다.\n"
            "M1~M7 실제 판정은 별도 RESULT.json과 두 단계 독립검토에 결속합니다. 이 도구는 판정을 발급하지 않습니다.\n").encode("utf-8")
    payload.update(native_workbench_payload(payload))
    source_zip = source_archive(repo, source)
    hits = personal_path_scan(payload)
    source_hits = personal_path_scan(source_payload(source_zip))
    require(not require_clean_paths or not (hits or source_hits), "개인 경로가 남아 있습니다. 검사 보고용 비공개 묶음과 공개 적합은 다릅니다")
    payload[track + "_CANDIDATE_MANIFEST.json"] = encode({
        "schema": "ddn." + track.lower() + ".private_candidate_package.v1", "source": source, "components": bindings,
        "source_zip_sha256": digest(source_zip), "visibility": "private",
        "third_party_notices": notice_evidence,
        "claim": "PACKAGE_INTEGRITY_CHECKED_NOT_SCENARIO_REVIEW_HUMAN_PASS",
        **({"core_six_scenarios": "NOT_RUN_BY_PACKAGER", "review_B_C": "NOT_ISSUED"} if track == "V2" else
           {"M1_M7_scenarios": "NOT_RUN_BY_PACKAGER", "stage1_final_reviews": "NOT_ISSUED"}), "Human": "NOT_RUN",
        "personal_path_findings": len(hits), "source_personal_path_findings": len(source_hits),
        "recipe_sha256": digest(recipe_raw)})
    payload["SHA256SUMS.txt"] = checksums(payload)
    artifact = zip_bytes(payload, name)
    out.mkdir(parents=True, exist_ok=False)
    zip_name, source_name = name + ".zip", name + "-source-exact.zip"
    for filename, raw in ((zip_name, artifact), (source_name, source_zip)):
        with (out / filename).open("xb") as stream:
            stream.write(raw)
    (out / "SHA256SUMS.txt").write_bytes(
        f"{digest(artifact)}  {zip_name}\n{digest(source_zip)}  {source_name}\n".encode("utf-8"))
    (out / "PERSONAL_PATH_SCAN.json").write_bytes(encode({"execution_members": hits, "source_members": source_hits}))
    report = {"source": source, "zip": str(out / zip_name), "bytes": len(artifact), "sha256": digest(artifact),
              "source_zip": str(out / source_name), "source_zip_sha256": digest(source_zip),
              "member_checksums": len(payload) - 1, "visibility": "private",
              "scenario_review_Human": "NOT_CLAIMED", "personal_paths_clean": not (hits or source_hits)}
    (out / "ARTIFACT.json").write_bytes(encode(report))
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--recipe", required=True, type=Path)
    parser.add_argument("--repo", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--require-clean-paths", action="store_true")
    args = parser.parse_args()
    try:
        print(json.dumps(build(args.recipe, args.repo, args.out, args.require_clean_paths), ensure_ascii=False))
    except (ValueError, TypeError, OSError, KeyError, zipfile.BadZipFile, subprocess.CalledProcessError) as error:
        print("묶기 거부: " + str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
