from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
from types import SimpleNamespace
import json
import io
import zipfile

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
import alpha1_r20_archive as archive


class WorkbenchAssets(unittest.TestCase):
    def test_startup_requirements_are_checked_from_zip_bytes(self):
        stream = io.BytesIO()
        prefix = archive.PACKAGE + '/'
        with zipfile.ZipFile(stream, 'w') as zipped:
            zipped.writestr(prefix + 'START_SEAMGRIM.cmd',
                           'sys.version_info >= (3,10) ddn_exec_server.py')
            zipped.writestr(prefix + 'README.ko.md',
                           'START_SEAMGRIM.cmd Python 3.10 http://127.0.0.1:8787/ Ctrl+C')
            zipped.writestr(prefix + 'solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py', '')
            zipped.writestr(prefix + archive.UI + 'index.html', '')
        stream.seek(0)
        with zipfile.ZipFile(stream) as zipped:
            result = archive.verify_first_launch_instructions(zipped)
            self.assertTrue(result['local_server_required'])
            self.assertFalse(result['python_bundled'])
            self.assertIn('PENDING', result['readme_launch_test'])
        missing = io.BytesIO()
        with zipfile.ZipFile(missing, 'w') as zipped:
            zipped.writestr(prefix + 'README.ko.md', '')
        missing.seek(0)
        with zipfile.ZipFile(missing) as zipped, self.assertRaisesRegex(ValueError, '필요한 파일'):
            archive.verify_first_launch_instructions(zipped)

    def test_zip_readback_rejects_retired_renamed_and_missing_dependencies(self):
        base = 'solutions/seamgrim_ui_mvp/ui/'
        entry = base + 'workbench/index.html'
        common = base + 'shared.js'
        def check(stage, payload):
            stream = io.BytesIO()
            with zipfile.ZipFile(stream, 'w') as zipped:
                for rel, data in payload.items():
                    zipped.writestr(archive.PACKAGE + '/' + rel, data)
            stream.seek(0)
            with zipfile.ZipFile(stream) as zipped:
                return archive.verify_successor_ui_archive(zipped, stage, payload)
        with tempfile.TemporaryDirectory() as folder:
            stage = Path(folder)
            old = stage / base / 'app.js'
            old.parent.mkdir(parents=True)
            old.write_bytes(b'RETIRED_APP_UNIQUE')
            valid = {entry: b'<script src="../shared.js"></script>', common: b'export const x=1;'}
            self.assertEqual(check(stage, valid)['dependency_files_checked'], 2)
            for name in ('app.js', 'renamed.js', 'other/copied.js'):
                with self.subTest(name=name), self.assertRaisesRegex(ValueError, '옛 화면'):
                    check(stage, {**valid, base + name: b'RETIRED_APP_UNIQUE'})
            with self.assertRaisesRegex(ValueError, '의존 파일'):
                check(stage, {entry: valid[entry]})
            with self.assertRaisesRegex(ValueError, '의존 파일'):
                check(stage, {entry: b'<script src="../app.js"></script>'})

    def test_retired_ui_zip_inventory_excludes_entrypoints_but_keeps_workbench(self):
        retired = ['index.html', 'play.html', 'app.js', 'playground.js', 'screens/run.js']
        with tempfile.TemporaryDirectory() as folder:
            stage = Path(folder)
            for rel in retired + ['workbench/index.html', 'wasm_page_common.js']:
                target = stage / 'solutions/seamgrim_ui_mvp/ui' / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text('fixture', encoding='utf-8')
            historic = archive.portable_inventory(stage)
            successor = archive.portable_inventory(stage, True)
            self.assertEqual(len(historic), 7)
            self.assertEqual(set(successor), {
                'solutions/seamgrim_ui_mvp/ui/workbench/index.html',
                'solutions/seamgrim_ui_mvp/ui/wasm_page_common.js'})
            self.assertFalse(any(archive.is_retired_ui_path(p) for p in successor))
            self.assertEqual(len(archive.inventory(stage)), 7)  # originals preserved

    def test_fixed_workbench_lists_only_required_grid_files(self):
        source = (ROOT / archive.UI / 'workbench.js').read_text(encoding='utf-8')
        assets = archive.workbench_project_assets(source)
        prefix = 'solutions/seamgrim_ui_mvp/lessons/rep_grid_game_state_drop_v1/'
        self.assertEqual(sorted(p[len(prefix):] for p in assets if p.startswith(prefix)),
                         ['action_map.json', 'lesson.ddn', 'meta.toml'])
        # Owner G1 B: sample drawings/rows are not execution inputs.
        self.assertNotIn(prefix + 'space2d.json', assets)
        self.assertNotIn(prefix + 'table.json', assets)

    def test_binding_uses_fixed_blobs_and_rejects_missing_or_changed_files(self):
        ui = b'const files = [{url: "../../lessons/grid/lesson.ddn"}];'
        rel = 'solutions/seamgrim_ui_mvp/lessons/grid/lesson.ddn'
        blobs = {archive.UI + 'workbench.js': ui, rel: '1 보여주기.'.encode()}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, stage = root / 'source', root / 'stage'
            for tree in (source, stage):
                target = tree / archive.UI / 'workbench.js'
                target.parent.mkdir(parents=True)
                target.write_bytes(ui)
            def read_blob(args, cwd):
                self.assertTrue(args[-1].startswith('fixed:'))
                return blobs[args[-1].split(':', 1)[1]]
            origins = {}
            with patch.object(archive.subprocess, 'check_output', side_effect=read_blob):
                self.assertEqual(archive.bind_workbench_project_assets(root, 'fixed', source, stage, origins), [rel])
            (stage / rel).unlink()
            with self.assertRaisesRegex(ValueError, '누락'):
                archive.verify_workbench_project_assets(stage, origins)
            (stage / rel).write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, '불일치'):
                archive.verify_workbench_project_assets(stage, origins)

    def test_path_escape_is_rejected(self):
        with self.assertRaises(ValueError):
            archive.workbench_project_assets('url: "../../lessons/../../secret"')

    def test_r23_requires_new_fixed_native_and_wasm_inputs(self):
        with self.assertRaisesRegex(ValueError, '입력 누락'):
            archive.validate_r23_inputs(SimpleNamespace(), ROOT)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, native_dir = root / 'source', root / 'bin'
            native_dir.mkdir()
            for rel in (archive.WASM, archive.MANIFEST):
                target = source / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(rel.encode())
            files = {}
            for name in ('ddonirang.exe', 'ddonilang.exe', '또니랑.exe'):
                data = name.encode()
                (native_dir / name).write_bytes(data)
                files[name] = archive.sha(data)
            manifest = root / 'native.json'
            official = native_dir / 'CANDIDATE_MANIFEST.detjson'
            official.write_text(json.dumps({'files': [
                {'path': name, 'sha256': checksum, 'bytes': (native_dir / name).stat().st_size}
                for name, checksum in files.items()]}), encoding='utf-8')
            native = {'source_commit': 'a' * 40, 'source_tree': 'b' * 40,
                'candidate_manifest_sha256': archive.sha(official.read_bytes()), 'files': files}
            manifest.write_text(json.dumps(native), encoding='utf-8')
            args = SimpleNamespace(product='c' * 40, product_tree='d' * 40, source_tree=str(source),
                source_identity=archive.identity(archive.inventory(source)), native_dir=str(native_dir),
                native_manifest=str(manifest), wasm_sha256=archive.sha(archive.WASM.encode()),
                runtime_manifest_sha256=archive.sha(archive.MANIFEST.encode()))
            drift = []
            def git_output(command, cwd):
                if command[1] == 'rev-parse':
                    return (('b' if command[2].startswith('a') else 'd') * 40).encode()
                self.assertEqual(command[1:3], ['diff', '--name-only'])
                return '\n'.join(drift).encode()
            with patch.object(archive.subprocess, 'check_output', side_effect=git_output), \
                 patch.object(archive.subprocess, 'run') as ancestry:
                _, validated = archive.validate_r23_inputs(args, root)
                self.assertEqual(validated['source_commit'], 'a' * 40)
                self.assertNotEqual(validated['source_commit'], args.product)
                ancestry.assert_called_once_with(
                    ['git', 'merge-base', '--is-ancestor', 'a' * 40, 'c' * 40], cwd=root, check=True)
                drift.append('tool/src/ddn_runtime.rs')
                with self.assertRaisesRegex(ValueError, '빌드 입력'):
                    archive.validate_r23_inputs(args, root)
                drift.clear()
                official.write_bytes(b'changed receipt')
                with self.assertRaisesRegex(ValueError, 'manifest 불일치'):
                    archive.validate_r23_inputs(args, root)
                official.write_text(json.dumps({'files': [
                    {'path': name, 'sha256': checksum, 'bytes': (native_dir / name).stat().st_size}
                    for name, checksum in files.items()]}), encoding='utf-8')
                (native_dir / 'ddonirang.exe').write_bytes(b'old')
                with self.assertRaisesRegex(ValueError, 'CLI 해시'):
                    archive.validate_r23_inputs(args, root)
                args.product = 'different'
                with self.assertRaisesRegex(ValueError, '출처'):
                    archive.validate_r23_inputs(args, root)


if __name__ == '__main__':
    unittest.main()
