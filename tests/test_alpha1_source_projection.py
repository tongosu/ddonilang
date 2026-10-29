import importlib.util
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
import sys
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('alpha_projection', Path(__file__).parents[1] / 'tools/alpha1_source_projection.py')
projection = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(projection)
sys.path.insert(0, str(Path(__file__).parents[1] / 'tools'))
import alpha1_r20_archive as archive


class R23ProjectionTests(unittest.TestCase):
    def fixture(self, root):
        predecessor = root / 'r22/tree'
        predecessor.mkdir(parents=True)
        for name, data in {'Cargo.toml': b'public workspace', 'lang/src/runtime.rs': b'old'}.items():
            p = predecessor / name
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(data)
        manifest = predecessor.parent / 'SOURCE_MANIFEST.json'
        manifest.write_bytes(b'fixed manifest')
        wasm = projection.WASM_DIR.as_posix() + '/ddonirang_tool_bg.wasm'
        runtime_manifest = projection.WASM_DIR.as_posix() + '/ddonirang_tool.manifest.detjson'
        blobs = {'Cargo.toml': b'private full workspace', 'lang/src/runtime.rs': b'new',
                 wasm: b'new wasm', runtime_manifest: b'new manifest',
                 'work/private.md': b'never copy'}
        args = SimpleNamespace(source=root, predecessor=predecessor, output=root / 'r23',
            source_commit='a' * 40, source_tree='b' * 40,
            wasm_sha256=projection.digest(blobs[wasm]), runtime_manifest_sha256=projection.digest(blobs[runtime_manifest]))
        return args, blobs, (wasm, runtime_manifest)

    def run_fixture(self, args, blobs, required):
        predecessor_version = 'R23' if getattr(args, 'candidate', '') == 'R24' else 'R22'
        def git(_repo, *args):
            if args[0] == 'rev-parse': return 'b' * 40
            if '-r' in args: return '\n'.join(blobs)
            return '100644 blob ' + 'c' * 40 + '\t' + args[-1]
        with patch.object(projection, predecessor_version + '_SOURCE_IDENTITY', projection.r23_identity(projection.r23_inventory(args.predecessor))), \
             patch.object(projection, predecessor_version + '_MANIFEST_SHA256', projection.digest(b'fixed manifest')), \
             patch.object(projection, 'R23_ADDITIONS', ()), \
             patch.object(projection, 'R23_REQUIRED', required), \
             patch.object(projection, 'git', side_effect=git), \
             patch.object(projection.subprocess, 'check_output', side_effect=lambda command: blobs[command[-1].split(':', 1)[1]]):
            return projection.build_r23_projection(args)

    def test_r24_uses_r23_custody_and_requires_new_runtime_regression(self):
        with tempfile.TemporaryDirectory() as tmp:
            args, blobs, required = self.fixture(Path(tmp))
            args.candidate = 'R24'
            rel, = projection.R24_ADDITIONS
            with self.assertRaisesRegex(SystemExit, 'PRODUCT_BLOB_MISSING'):
                self.run_fixture(args, blobs, required)
            self.assertFalse(args.output.exists())
            blobs[rel] = b'new condition and binding regression'
            result = self.run_fixture(args, blobs, required)
            self.assertEqual((args.output / 'tree' / rel).read_bytes(), blobs[rel])
            self.assertEqual(result['delta_origins']['Cargo.toml']['origin'], 'R23_ALLOWLIST_DERIVED')
            self.assertEqual(result['predecessor_identity'], projection.r23_identity(projection.r23_inventory(args.predecessor)))

    def test_r24_guidance_keeps_untyped_rebinding_open(self):
        text = archive.r24_readme('a' * 40, 'b' * 40)
        self.assertIn('조건에는 참거짓만 허용', text)
        self.assertIn('씨를 명시하지 않은 이름', text)
        self.assertIn('처리가 아직 다릅니다', text)
        self.assertNotIn('알려진 제한: 숫자를 조건', text)

    def test_exact_git_bytes_and_preserved_public_workspace(self):
        with tempfile.TemporaryDirectory() as tmp:
            args, blobs, required = self.fixture(Path(tmp))
            result = self.run_fixture(args, blobs, required)
            self.assertEqual((args.output / 'tree/lang/src/runtime.rs').read_bytes(), b'new')
            self.assertEqual((args.output / 'tree/Cargo.toml').read_bytes(), b'public workspace')
            self.assertFalse((args.output / 'tree/work').exists())
            self.assertEqual(result['identity'], projection.r23_identity(projection.r23_inventory(args.output / 'tree')))
            self.assertEqual(result['delta_origins']['lang/src/runtime.rs']['commit'], args.source_commit)

    def test_artifact_mismatch_fails_before_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            args, blobs, required = self.fixture(Path(tmp))
            args.wasm_sha256 = '0' * 64
            with self.assertRaisesRegex(SystemExit, 'ARTIFACT_IDENTITY'):
                self.run_fixture(args, blobs, required)
            self.assertFalse(args.output.exists())

    def test_missing_required_blob_fails_before_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            args, blobs, required = self.fixture(Path(tmp))
            with self.assertRaisesRegex(SystemExit, 'PRODUCT_BLOB_MISSING'):
                self.run_fixture(args, blobs, (*required, 'lang/src/missing.rs'))
            self.assertFalse(args.output.exists())

    def test_exact_identity_and_safe_paths_required(self):
        with tempfile.TemporaryDirectory() as tmp:
            args, blobs, required = self.fixture(Path(tmp))
            args.source_commit = 'HEAD'
            with self.assertRaisesRegex(SystemExit, 'EXACT_IDENTITY_REQUIRED'):
                self.run_fixture(args, blobs, required)
        for path in ('../secret', '/secret', 'C:/secret', 'docs/ssot/file', 'work/internal', 'a\\b'):
            with self.assertRaises(SystemExit): projection.safe_public_path(path)

    def test_native_build_source_is_not_relabelled_as_assembly_commit(self):
        def outputs(command, **kwargs):
            if command[1] == 'rev-parse':
                return (('b' if command[2].startswith('a') else 'd') * 40).encode()
            return b'solutions/seamgrim_ui_mvp/ui/workbench/workbench.js\ntests/new_test.rs\n'
        with patch.object(archive.subprocess, 'check_output', side_effect=outputs), \
             patch.object(archive.subprocess, 'run'):
            archive.validate_native_source_lineage(Path('.'), 'a' * 40, 'b' * 40, 'c' * 40, 'd' * 40)
        def changed_inputs(command, **kwargs):
            return b'tool/src/ddn_runtime.rs\n' if command[1] == 'diff' else outputs(command, **kwargs)
        with patch.object(archive.subprocess, 'check_output', side_effect=changed_inputs), \
             patch.object(archive.subprocess, 'run'), self.assertRaisesRegex(ValueError, '빌드 입력'):
            archive.validate_native_source_lineage(Path('.'), 'a' * 40, 'b' * 40, 'c' * 40, 'd' * 40)


if __name__ == '__main__':
    unittest.main()
