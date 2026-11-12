"""Synthetic staging fixtures, not evidence of a native app or Mac acceptance."""
import hashlib
import json
from pathlib import Path
import plistlib
import tempfile
import unittest
from unittest.mock import patch
from prepare_macos_handoff import stage
from macos_candidate import signing_command, validate_bundle


class MacHandoff(unittest.TestCase):
    def fixture(self, root, app_id='pixasound'):
        product = {'id': app_id, 'name': 'PixaSound' if app_id == 'pixasound' else 'PixaFilm', 'version': '0.1.0', 'bundle_id': 'com.aiwebdesignfirm.' + app_id, 'revision': 'a' * 40}
        app = root / (product['name'] + '.app')
        (app / 'Contents/MacOS').mkdir(parents=True)
        info = {'CFBundleIdentifier': product['bundle_id'], 'CFBundleShortVersionString': product['version'], 'CFBundleExecutable': app_id}
        (app / 'Contents/Info.plist').write_bytes(plistlib.dumps(info))
        (app / 'Contents/MacOS' / app_id).write_bytes(b'fixture, not a Mach-O app')
        source = root / 'source'
        (source / 'packaging/macos').mkdir(parents=True)
        (source / '.pixa-prepared.json').write_text(json.dumps({'app': product}))
        (source / 'packaging/macos/Info.plist.in').write_bytes(plistlib.dumps({'NSMicrophoneUsageDescription': 'Fixture purpose'}))
        (source / 'LICENSE-MIT').write_text('Fixture license')
        font = source / 'assets/fonts/OFL-Fixture.txt'
        font.parent.mkdir(parents=True)
        font.write_text('Fixture font notice')
        receipt = root / 'candidate.json'
        data = {'product': app_id, 'version': product['version'], 'bundle_id': product['bundle_id'], 'revision': product['revision'], 'access_mode': 'subscription', 'sha256': hashlib.sha256(b'fixture, not a Mach-O app').hexdigest(), 'architectures': ['arm64', 'x86_64'], 'signed': False, 'published': False}
        receipt.write_text(json.dumps(data))
        return product, app, source, receipt

    def test_stage_supplies_sound_privacy_and_recursive_licenses_on_copy(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            product, app, source, receipt = self.fixture(root)
            with patch('manuals.markdown', return_value='Fixture manual'):
                report = stage(product, app, source, receipt, root / 'out')
            target = root / 'out/PixaSound.app'
            self.assertEqual(plistlib.loads((target / 'Contents/Info.plist').read_bytes())['NSMicrophoneUsageDescription'], 'Fixture purpose')
            self.assertNotIn('NSMicrophoneUsageDescription', plistlib.loads((app / 'Contents/Info.plist').read_bytes()))
            self.assertTrue((root / 'out/Licenses/assets/fonts/OFL-Fixture.txt').is_file())
            self.assertFalse(report['ready_for_notarization'])
            self.assertFalse(report['signed'])
            validate_bundle(target, product)
            command = signing_command('Fixture', target, product, target, root)
            entitlement = command[command.index('--entitlements') + 1]
            self.assertEqual(plistlib.loads(entitlement.read_bytes()), {'com.apple.security.device.audio-input': True})

    def test_rejects_old_version_tampered_binary_and_existing_output(self):
        for failure in ['version', 'binary', 'existing']:
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                product, app, source, receipt = self.fixture(root)
                if failure == 'version':
                    data = json.loads(receipt.read_text()); data['version'] = '0.0.1'; receipt.write_text(json.dumps(data))
                if failure == 'binary':
                    (app / 'Contents/MacOS/pixasound').write_bytes(b'changed')
                if failure == 'existing':
                    (root / 'out').mkdir()
                with self.assertRaises(ValueError):
                    stage(product, app, source, receipt, root / 'out')


if __name__ == '__main__':
    unittest.main()
