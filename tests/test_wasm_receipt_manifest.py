import importlib.util
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('freshness', Path(__file__).with_name('run_ddn_cli_wasm_freshness_check.py'))
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


class ReceiptManifestTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'scripts').mkdir()
        (self.root / 'scripts/build_wasm_tool.ps1').write_text('$cargoTargetRoot; Join-Path $cargoTargetRoot', encoding='utf-8')
        (self.root / 'scripts/copy_wasm_tool_to_ui.ps1').write_text('copy', encoding='utf-8')
        (self.root / 'Cargo.lock').write_text('lock', encoding='utf-8')
        self.js = self.root / 'ddonirang_tool.js'
        self.js.write_text('export function wasm_build_info() {}', encoding='utf-8')
        self.wasm = self.root / 'ddonirang_tool_bg.wasm'
        self.wasm.write_bytes(b'wasm-fixture')
        self.receipt_path = self.root / 'receipt.json'
        self.receipt = {
            'source_commit': 'a' * 40, 'source_tree': 'b' * 40,
            'profile': 'release', 'target': 'web',
            'toolchain': {'rustc': 'sealed rustc', 'cargo': 'sealed cargo', 'wasm_tool': 'wasm-bindgen 0.2.108'},
            'artifacts': {p.name: {'bytes': p.stat().st_size, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in (self.js, self.wasm)},
            'path_scan': {key: 0 for key in ('internal_absolute_path_count', 'private_include_count', 'official_store_path_count', 'unknown_path_count')},
        }
        self.write_receipt()
        self.stack = []
        for name, value in {'ROOT': self.root, 'WASM_DIR': self.root, 'JS_BINDING': self.js, 'WASM_BINARY': self.wasm, 'BUILD_RECEIPT': self.receipt_path}.items():
            context = patch.object(m, name, value)
            context.start()
            self.addCleanup(context.stop)
        for name, value in {'collect_source_files': lambda: [], 'extract_state_schema': lambda: 'fixture', 'run_text': lambda cmd: '' if 'status' in cmd else ('b' * 40 if cmd[-1] == 'HEAD^{tree}' else 'a' * 40)}.items():
            context = patch.object(m, name, value)
            context.start()
            self.addCleanup(context.stop)

    def write_receipt(self):
        payload = json.dumps(self.receipt, ensure_ascii=False, sort_keys=True, separators=(',', ':')) + '\n'
        sealed = {**self.receipt, 'semantic_identity': 'sha256:' + hashlib.sha256(payload.encode()).hexdigest()}
        self.receipt_path.write_text(json.dumps(sealed), encoding='utf-8')

    def test_sealed_bindgen_version_without_live_pack(self):
        result = m.build_manifest()
        self.assertEqual(result['wasm_bindgen_or_pack_version'], 'wasm-bindgen 0.2.108')
        self.assertEqual(result['schema'], 'ddn.wasm_artifact_manifest.v1')

    def test_tampered_receipt_rejected(self):
        data = json.loads(self.receipt_path.read_text())
        data['target'] = 'nodejs'
        self.receipt_path.write_text(json.dumps(data))
        with self.assertRaisesRegex(RuntimeError, 'identity mismatch'): m.build_manifest()

    def test_other_source_rejected(self):
        self.receipt['source_commit'] = 'c' * 40
        self.write_receipt()
        with self.assertRaisesRegex(RuntimeError, 'source mismatch'): m.build_manifest()

    def test_changed_binary_rejected(self):
        self.wasm.write_bytes(b'changed')
        with self.assertRaisesRegex(RuntimeError, 'artifact mismatch'): m.build_manifest()

    def test_private_paths_rejected(self):
        self.receipt['path_scan']['internal_absolute_path_count'] = 1
        self.write_receipt()
        with self.assertRaisesRegex(RuntimeError, 'private paths'): m.build_manifest()

    def test_nonrelease_rejected(self):
        self.receipt['profile'] = 'debug'
        self.write_receipt()
        with self.assertRaisesRegex(RuntimeError, 'target mismatch'): m.build_manifest()


if __name__ == '__main__': unittest.main()
