"""Audit generated product source; do not infer installed release acceptance."""
from pathlib import Path
import sys
import unittest
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'pixaeffects'))
from branding import clean_metadata


class BrandingTests(unittest.TestCase):
    def test_metadata_cleanup_preserves_license(self):
        fixture = '<component><name>Learning Machines LLC</name><project_license>MIT OR Apache-2.0</project_license><screenshots><image>upstream</image></screenshots><url>https://getartcraft.com/apps/effectcraft</url></component>'
        cleaned = clean_metadata(fixture, 'app.metainfo.xml.in')
        self.assertNotIn('Learning Machines', cleaned)
        self.assertNotIn('<screenshots>', cleaned)
        self.assertIn('<project_license>MIT OR Apache-2.0</project_license>', cleaned)
        self.assertIn('https://aiwebdesignfirm.com/pixaeffects', cleaned)

    def require_prepared(self):
        if not (ROOT / 'pixaeffects/.prepared').exists():
            self.skipTest('Run prepare.py on a clean checkout first')

    def test_menu_and_palette_have_no_community_promotion(self):
        self.require_prepared()
        for relative in ['crates/engine/src/cmd/help.rs', 'crates/engine/src/menus.rs', 'crates/ui-egui/src/panels/dialogs.rs', 'crates/ui-egui/src/panels/home.rs']:
            text = (ROOT / relative).read_text()
            for forbidden in ['"help.discord"', '"help.github"', '| help.discord', '| help.github', 'Join the AIWebDesignFirm Discord']:
                self.assertNotIn(forbidden, text, relative)
        self.assertIn('"help.account"', (ROOT / 'crates/ui-egui/src/panels/dialogs.rs').read_text())

    def test_metadata_and_demo_use_product_brand(self):
        self.require_prepared()
        for file in (ROOT / 'packaging').rglob('*'):
            if file.is_file() and ('.metainfo.xml' in file.name or file.name == 'nfpm.yaml'):
                text = file.read_text()
                for forbidden in ['Learning Machines LLC', 'getartcraft.com', 'discord.gg/artcraft', '<screenshots>']:
                    self.assertNotIn(forbidden, text, str(file))
        self.assertNotIn('"EFFECTCRAFT"', (ROOT / 'crates/engine/src/home/demo.rs').read_text())

    def test_product_ui_has_no_upstream_promotional_destinations(self):
        self.require_prepared()
        for folder in ['crates/ui-egui/src', 'apps/effectcraft/src']:
            for file in (ROOT / folder).rglob('*.rs'):
                if 'credits' in file.name or 'credits' in file.parts:
                    continue
                text = file.read_text()
                for forbidden in ['https://getartcraft.com', 'https://discord.gg/artcraft', 'https://github.com/storytold/effectcraft', 'Join the AIWebDesignFirm Discord']:
                    self.assertNotIn(forbidden, text, str(file))
        for relative in ['LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'ATTRIBUTION.md', 'contributors/contributors.json']:
            self.assertTrue((ROOT / relative).is_file())

    def test_existing_e_icon_and_own_release_feed(self):
        self.assertIn("render('E',", (ROOT / 'pixaeffects/icons.py').read_text())
        for file in (ROOT / 'pixaeffects/overlay').rglob('*.rs'):
            text = file.read_text()
            self.assertNotIn('api.github.com/repos/storytold/', text)


if __name__ == '__main__':
    unittest.main()
