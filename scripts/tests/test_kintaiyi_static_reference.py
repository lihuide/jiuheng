import importlib.util
from pathlib import Path
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "check-kintaiyi-static-reference.py"
SPEC = importlib.util.spec_from_file_location("static_reference_checker", SCRIPT)
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


class LiteralExtractionTests(unittest.TestCase):
    def source(self, expression=None, suffix=""):
        rows = repr([[7, 13, 13]] * 72) if expression is None else expression
        return ("raise RuntimeError('must never execute module')\n"
                "import pickle\n"
                "pickle.load(open('must-never-open', 'rb'))\n"
                "def find_cal(yingyang, num):\n"
                f"    yangcal = {rows}\n    yingcal = {rows}\n"
                "    raise RuntimeError('must never execute function')\n"
                + suffix).encode()

    def test_only_literals_are_read_without_executing_module_or_function(self):
        tables = CHECKER.extract_tables(self.source())
        self.assertEqual(tables, {"yang": [[7, 13]] * 72, "yin": [[7, 13]] * 72})

    def test_call_is_rejected_without_executing_it(self):
        with self.assertRaises(ValueError):
            CHECKER.extract_tables(self.source("__import__('os').system('must-never-run')"))

    def test_boolean_and_out_of_range_and_wrong_shapes_are_rejected(self):
        for rows in ([[True, 13, 13]] * 72, [[0, 13, 13]] * 72,
                     [[41, 13, 13]] * 72, [[7, 13]] * 72, [[7, 13, 13]] * 71):
            with self.subTest(rows=rows[0]):
                with self.assertRaises(ValueError):
                    CHECKER.extract_tables(self.source(repr(rows)))

    def test_duplicate_function_is_rejected(self):
        with self.assertRaises(ValueError):
            CHECKER.extract_tables(self.source(suffix="def find_cal(a, b):\n    pass\n"))

    def test_duplicate_assignment_is_rejected(self):
        source = self.source().replace(b"    raise RuntimeError", b"    yangcal = []\n    raise RuntimeError")
        with self.assertRaises(ValueError):
            CHECKER.extract_tables(source)


if __name__ == "__main__":
    unittest.main()
