"""Control-flow tests only: never execute codesign or contact Apple."""
import json
import plistlib
import shutil
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import macos_candidate as candidate


class BeforeNotarization(unittest.TestCase):
    def test_stop_mode_preserves_outputs_and_never_submits(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            catalog = root / 'catalog'
            catalog.mkdir()
            product = {'id': 'pixavector', 'name': 'PixaVector', 'version': '0.1.1', 'bundle_id': 'com.aiwebdesignfirm.pixavector'}
            (catalog / 'catalog.json').write_text(json.dumps({'apps': [product]}))
            app = root / 'PixaVector.app'
            contents = app / 'Contents'
            (contents / 'MacOS').mkdir(parents=True)
            info = {'CFBundleIdentifier': product['bundle_id'], 'CFBundleShortVersionString': product['version'], 'CFBundleExecutable': 'pixavector'}
            (contents / 'Info.plist').write_bytes(plistlib.dumps(info))
            (contents / 'MacOS/pixavector').write_bytes(b'fixture only')
            notices = root / 'Licenses'
            notices.mkdir()
            (notices / 'LICENSE-MIT').write_text('Fixture license')
            commands = []

            def fake_run(args, capture=False):
                args = [str(item) for item in args]
                commands.append(args)
                if 'notarytool' in args or 'stapler' in args or args[0].endswith('/spctl'):
                    self.fail('Apple submission/acceptance command reached in stop mode')
                if args[0].endswith('/ditto'):
                    if '-c' in args:
                        Path(args[-1]).write_bytes(b'fixture archive')
                    else:
                        shutil.copytree(args[-2], args[-1])
                return SimpleNamespace(stdout='arm64 x86_64' if args[0].endswith('/lipo') else '')

            arguments = ['macos_candidate.py', 'pixavector', str(app), '--identity', 'Developer ID Application: Fixture', '--output', str(root / 'out'), '--notices', str(notices), '--stop-before-notarization']
            with patch.object(candidate, 'ROOT', catalog), patch.object(candidate.sys, 'platform', 'darwin'), patch.object(candidate.sys, 'argv', arguments), patch.object(candidate, 'run', side_effect=fake_run):
                candidate.main()
            result = next((root / 'out').glob('*-signed-unnotarized'))
            report = json.loads((result / 'pre-notarization.json').read_text())
            self.assertTrue(report['signed'])
            self.assertFalse(report['notarized'])
            self.assertFalse(report['release_ready'])
            self.assertTrue((result / 'PixaVector.app/Contents/MacOS/pixavector').is_file())
            self.assertTrue((result / 'notarize.zip').is_file())
            self.assertTrue((result / 'Licenses/LICENSE-MIT').is_file())
            self.assertEqual((app / 'Contents/MacOS/pixavector').read_bytes(), b'fixture only')
            self.assertTrue(any('--verify' in command for command in commands))


if __name__ == '__main__':
    unittest.main()
