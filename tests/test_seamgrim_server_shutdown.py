"""Windows main-loop interrupt regression; no foreign listener is touched."""
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import Mock, patch

SERVER = Path(__file__).resolve().parents[1] / 'solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py'
spec = importlib.util.spec_from_file_location('shutdown_target', SERVER)
target = importlib.util.module_from_spec(spec)
spec.loader.exec_module(target)


class ShutdownTests(unittest.TestCase):
    def run_main(self, interrupt):
        server = Mock(server_address=('127.0.0.1', 18939))
        thread = Mock()
        thread.is_alive.side_effect = [True, False]
        if interrupt:
            thread.join.side_effect = [KeyboardInterrupt(), None]
        with patch.object(target, 'SeamgrimHTTPServer', return_value=server), \
             patch.object(target.threading, 'Thread', return_value=thread), \
             patch.object(target, 'assert_runtime_config_startable', return_value={}), \
             patch('builtins.print'):
            target.main(['--port', '18939'])
        return server, thread

    def test_interruptible_wait_and_shutdown(self):
        server, thread = self.run_main(True)
        self.assertEqual(thread.join.call_args_list[0].kwargs, {'timeout': 0.25})
        server.shutdown.assert_called_once_with()
        server.server_close.assert_called_once_with()
        self.assertEqual(thread.join.call_args_list[1].kwargs, {'timeout': 5})

    def test_no_unbounded_join_on_regular_exit(self):
        server, thread = self.run_main(False)
        self.assertEqual(thread.join.call_args.kwargs, {'timeout': 0.25})
        server.server_close.assert_called_once_with()

    def test_second_server_is_rejected(self):
        first = target.SeamgrimHTTPServer(('127.0.0.1', 0), target.DdnExecHandler)
        try:
            with self.assertRaises(OSError):
                target.SeamgrimHTTPServer(first.server_address, target.DdnExecHandler)
        finally:
            first.server_close()

    def test_bind_failure_does_not_print_ready(self):
        with patch.object(target, 'SeamgrimHTTPServer', side_effect=OSError('in use')), \
             patch.object(target, 'assert_runtime_config_startable', return_value={}), \
             patch('builtins.print') as output:
            with self.assertRaisesRegex(SystemExit, 'E_SEAMGRIM_SERVER_BIND'):
                target.main(['--port', '18939'])
            output.assert_not_called()


if __name__ == '__main__':
    unittest.main()
