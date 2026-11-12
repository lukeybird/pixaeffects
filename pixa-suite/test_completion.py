"""Regression checks for candidate packaging and source-bound guide generation."""
import importlib.util,json,subprocess,sys,tempfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT))
import manuals
class CompletionTests(unittest.TestCase):
 def test_control_guides_fail_when_control_disappears(self):
  app={'id':'pixalight','source':'lightcraft'}
  with tempfile.TemporaryDirectory() as td:
   source=Path(td);file=source/'crates/ui-egui/src/state/view.rs';file.parent.mkdir(parents=True);file.write_text('enum RightPanel { Edit }')
   with self.assertRaisesRegex(RuntimeError,'Manual control reference changed'):manuals.control_chapters(app,source)
 def test_anything_embeds_actual_beginner_and_expert_help(self):
  with tempfile.TemporaryDirectory() as td:
   source=Path(td);file=source/'crates/engine/src/tools/example.rs';file.parent.mkdir(parents=True)
   file.write_text('ToolSpec { id: "test_tool", title: "A tool", summary: "Exact source detail.", beginner: "Simple explanation.", toolset: Toolset::Core }')
   rows=manuals.anything_tools({'source':'craft-anything'},source)
   self.assertEqual(rows,[{'title':'A tool','body':'Simple explanation. Exact source detail.','tool_id':'test_tool'}])
 def test_candidate_packaging_rejects_development_and_wrong_host(self):
  spec=importlib.util.spec_from_file_location('candidate_package',ROOT/'package.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
  with tempfile.TemporaryDirectory() as td:
   root=Path(td);m.ROOT=root
   (root/'catalog.json').write_text(json.dumps({'apps':[{'id':'pixafixture','source':'fixture','package':'fixture','revision':'a'*40}]}))
   binary=root/'target/release'/('fixture.exe' if sys.platform=='win32' else 'fixture');binary.parent.mkdir(parents=True);binary.write_bytes(b'fixture')
   receipt=root/'build-logs/pixafixture-release.json';receipt.parent.mkdir();receipt.write_text(json.dumps({'platform':sys.platform,'access_mode':'development'}))
   with self.assertRaisesRegex(RuntimeError,'account-gated'):m.package('pixafixture',True)
   receipt.write_text(json.dumps({'platform':'different-host','access_mode':'subscription'}))
   with self.assertRaisesRegex(RuntimeError,'platform'):m.package('pixafixture',True)
if __name__=='__main__':unittest.main()
