"""Korean course URLs decode once without weakening filesystem containment."""
import importlib.util
from pathlib import Path
import unittest
import io
import tempfile
import threading
from http.server import ThreadingHTTPServer
from urllib.request import urlopen
from unittest.mock import Mock, patch
from urllib.parse import quote

SERVER = Path(__file__).resolve().parents[1] / "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py"
spec = importlib.util.spec_from_file_location("course_static_target", SERVER)
target = importlib.util.module_from_spec(spec)
spec.loader.exec_module(target)


class CourseStaticPaths(unittest.TestCase):
    def test_released_root_samples_have_explicit_routes(self):
        for name in ('alpha3_logical_reaction.ddn', 'alpha3_basic_tensor.ddn'):
            _, sent, _ = self.request('/samples/' + name)
            self.assertEqual(sent[0].args[1], target.ROOT / 'samples' / name)
        _, sent, _ = self.request('/samples/other.ddn')
        self.assertEqual(sent[0].args[1], (target.SAMPLES_DIR / 'other.ddn').resolve())

    def test_product_http_server_serves_course_manifests(self):
        server = ThreadingHTTPServer(("127.0.0.1", 0), target.DdnExecHandler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            for course in ("first_steps", "first_steps_alpha2", "first_steps_alpha3"):
                url = f"http://127.0.0.1:{server.server_port}/{course}/" + quote("검증목록.json")
                with urlopen(url, timeout=5) as response:
                    self.assertEqual(response.status, 200)
                    self.assertIn(b'"cases"', response.read())
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=5)

    def test_all_released_course_roots_are_served(self):
        for course in ("first_steps", "first_steps_alpha2", "first_steps_alpha3"):
            for prefix in ("/", "/solutions/seamgrim_ui_mvp/"):
                for name in ("검증목록.json", "예제/한글.ddn", "학생용.pdf"):
                    with self.subTest(course=course, prefix=prefix, name=name):
                        handler, sent, _ = self.request(prefix + course + "/" + quote(name))
                        self.assertEqual(len(sent), 1)
                        self.assertEqual(sent[0].args[1], (target.UI_DIR.parent / course / name).resolve())
                        handler.send_error.assert_not_called()

    def test_all_course_roots_reject_encoded_escape(self):
        for course in ("first_steps", "first_steps_alpha2", "first_steps_alpha3"):
            handler, sent, _ = self.request("/" + course + "/%2e%2e/tools/ddn_exec_server.py")
            self.assertEqual(sent, [])
            handler.send_error.assert_called_once_with(404)

    def request(self, path):
        handler = Mock(path=path)
        with patch.object(target, "request_boundary_error", return_value=None), \
             patch.object(target, "handle_project_get", return_value=False), \
             patch.object(target, "send_file") as send, \
             patch.object(target, "json_response") as response:
            target.DdnExecHandler.do_GET(handler)
            return handler, send.call_args_list, response.call_args_list

    def test_korean_source_and_guide_decode(self):
        for name in ("예제/01A_첫계산.ddn", "교사용_안내와_해답.md"):
            handler, sent, _ = self.request("/solutions/seamgrim_ui_mvp/first_steps/" + quote(name))
            self.assertEqual(len(sent), 1)
            self.assertEqual(sent[0].args[1], (target.UI_DIR.parent / "first_steps" / name).resolve())
            handler.send_error.assert_not_called()

    def test_encoded_escape_cannot_read_outside_course(self):
        handler, sent, _ = self.request("/first_steps/%2e%2e/tools/ddn_exec_server.py")
        self.assertEqual(sent, [])
        handler.send_error.assert_called_once_with(404)

    def test_invalid_utf8_is_explicitly_rejected(self):
        _, sent, response = self.request("/first_steps/%ff.ddn")
        self.assertEqual(sent, [])
        self.assertEqual(response[0].args[1], 400)
        self.assertEqual(response[0].args[2]["code"], "E_STATIC_PATH_UTF8")

    def test_double_encoding_is_not_decoded_twice(self):
        _, sent, _ = self.request("/first_steps/%252e%252e/secret.ddn")
        self.assertEqual(sent[0].args[1], (target.UI_DIR.parent / "first_steps/%2e%2e/secret.ddn").resolve())

    def test_pdf_and_markdown_have_correct_content_types(self):
        with tempfile.TemporaryDirectory() as directory:
            for name, data, content_type in (
                ("학생용.pdf", b"%PDF-test", "application/pdf"),
                ("안내.md", "# 한글 안내".encode("utf-8"), "text/plain; charset=utf-8"),
            ):
                file = Path(directory) / name
                file.write_bytes(data)
                handler = Mock(wfile=io.BytesIO())
                with patch.object(target, "_origin_allowed_for_response", return_value=None):
                    target.send_file(handler, file)
                handler.send_header.assert_any_call("Content-Type", content_type)
                self.assertEqual(handler.wfile.getvalue(), data)


if __name__ == "__main__":
    unittest.main()
