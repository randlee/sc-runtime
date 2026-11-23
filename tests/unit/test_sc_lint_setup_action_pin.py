"""Keep the vendored sc-publish fallback distinct from active sc-lint pins."""
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class SetupScLintActionPinTests(unittest.TestCase):
    def test_vendored_fallback_is_unused_by_repository_workflows(self):
        action = (ROOT / '.github/actions/setup-sc-lint/action.yml').read_text()
        self.assertIn('default: "0.8.4"', action)
        workflows = list((ROOT / '.github/workflows').glob('*.yml'))
        workflows.extend((ROOT / '.github/workflows').glob('*.yaml'))
        callers = [path for path in workflows
                   if './.github/actions/setup-sc-lint' in path.read_text()]
        self.assertEqual(callers, [],
                         'pin an explicit current sc-lint version or refresh the vendored kit before use')


if __name__ == '__main__':
    unittest.main()
