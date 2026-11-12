"""Product presentation regressions; legal credits and stable IDs are retained."""
import unittest
import tempfile
from pathlib import Path
from branding import product_links,presentation_violations


class ProductLinks(unittest.TestCase):
 def product(self):
  return {'id':'pixaword','source':'wordcraft','name':'PixaWord'}

 def test_about_promotions_become_product_manual_and_account(self):
  source='''ui.hyperlink_to(tl!("aiwebdesignfirm.com/apps/wordcraft"), "https://aiwebdesignfirm.com/apps/wordcraft");
ui.hyperlink_to(tl!("Join us on Discord: aiwebdesignfirm.com/settings"), "https://aiwebdesignfirm.com/settings");
ui.hyperlink_to(tl!("Source code: github.com/storytold/wordcraft"), "https://aiwebdesignfirm.com");
'''
  result=product_links(source,self.product())
  self.assertNotIn('Discord',result)
  self.assertNotIn('github.com/storytold/',result)
  self.assertIn('"PixaWord User Manual"',result)
  self.assertIn('https://aiwebdesignfirm.com/pixa-showcase/manuals/pixaword',result)
  self.assertIn('https://aiwebdesignfirm.com/products/pixaword',result)
  self.assertIn('https://aiwebdesignfirm.com/account',result)

 def test_legal_contributors_comments_and_command_ids_are_unchanged(self):
  source='''ui.label("Copyright ArtCraft Team. Join us on Discord");
// Attribution: github.com/storytold/wordcraft
let command = "help.discord";
ui.hyperlink_to(contributor.name, format!("https://github.com/{}", contributor.login));
'''
  self.assertEqual(product_links(source,self.product()),source)

 def test_other_product_and_arbitrary_urls_are_not_rewritten(self):
  source='ui.hyperlink_to("https://aiwebdesignfirm.com/apps/deckcraft", "https://example.com");'
  self.assertEqual(product_links(source,self.product()),source)

 def test_deck_sound_and_material_promotional_labels(self):
  for label in ['Join our community on Discord','Join us on Discord','Community (Discord)']:
   self.assertEqual(product_links('ui.label("'+label+'");',self.product()),'ui.label("Manage your Pixa account");')

 def test_embedded_export_and_cli_urls_follow_product_routes(self):
  source='o.push_str("% Written by PixaWord (https://aiwebdesignfirm.com/apps/wordcraft).\\n");'
  result=product_links(source,self.product())
  self.assertIn('https://aiwebdesignfirm.com/products/pixaword',result)
  self.assertIn('\\n',result)

 def test_audit_rejects_production_promotion_without_flagging_credits(self):
  with tempfile.TemporaryDirectory() as directory:
   root=Path(directory);file=root/'crates/ui-egui/src/about.rs';file.parent.mkdir(parents=True)
   file.write_text('ui.label("Join us on Discord");\nui.label("Copyright ArtCraft Team");\n')
   self.assertEqual(len(presentation_violations(root,self.product())),1)
   file.write_text(product_links(file.read_text(),self.product()))
   self.assertEqual(presentation_violations(root,self.product()),[])

 def test_help_constants_open_matching_manual_not_generic_home(self):
  source='pub const GITHUB: &str = "https://aiwebdesignfirm.com";\npub const APPS: &str = "https://aiwebdesignfirm.com/apps";'
  result=product_links(source,self.product())
  self.assertIn('https://aiwebdesignfirm.com/pixa-showcase/manuals/pixaword',result)
  self.assertIn('"https://aiwebdesignfirm.com/products"',result)

if __name__=='__main__':unittest.main()
