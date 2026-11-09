from pathlib import Path
import re,json,unittest
R=Path(__file__).resolve().parents[2]
class ProductTests(unittest.TestCase):
 def test_every_tool_documented(self):
  source=(R/'crates/ui-egui/src/state.rs').read_text();block=source.split('pub const ALL:')[1].split('];',1)[0]
  names=re.findall(r'Tool::(\w+)',block);data=json.loads((R/'pixaeffects/manual/tools.json').read_text());self.assertEqual(set(names),set(data));self.assertEqual(len(names),31)
 def test_update_identity(self):
  s=(R/'pixaeffects/overlay/crates/ui-egui/src/pixaeffects_updater.rs').read_text();self.assertIn('com.aiwebdesignfirm.pixaeffects',s);self.assertIn('lukeybird/pixaeffects',s);self.assertNotIn('lukeybird/pixashop',s);self.assertIn('spctl',s);self.assertIn('sha256:',s)
 def test_activation_uses_existing_entitlement(self):
  s=(R/'pixaeffects/overlay/apps/effectcraft/src/access.rs').read_text();self.assertIn('/api/pixaeffects/device',s);self.assertRegex(s,r'if self.allowed\(\) \{\s*self.inner.logic');self.assertIn('seconds.min(300)',s)
 def test_manual_complete(self):
  data=json.loads((R/'pixaeffects/manual/guide.json').read_text());self.assertEqual(len(data),12)
if __name__=='__main__':unittest.main()
