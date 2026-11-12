"""Exercise replacement/rollback with fake platform tools; not Apple acceptance."""
from pathlib import Path
import os
import sys
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
HANDLER = '''#!/usr/bin/env python3
import os, pathlib, shutil, sys
cmd=pathlib.Path(sys.argv[0]).name
args=sys.argv[1:]
fail=os.environ.get('FAIL','')
if cmd=='codesign':
 if '--verify' in args: sys.exit(1 if fail=='signature' else 0)
 print('TeamIdentifier=' + ('WRONG' if fail=='publisher' else '945M5X4DY3'))
 print('Identifier=com.aiwebdesignfirm.pixaeffects')
elif cmd=='spctl': sys.exit(1 if fail=='notary' else 0)
elif cmd=='mv':
 if fail=='replace' and args[0].endswith('/PixaEffects.app') and args[1]==os.environ['TARGET']: sys.exit(1)
 shutil.move(*args)
elif cmd=='open':
 marker=pathlib.Path(args[0])/'identity'
 if fail=='launch' and marker.read_text()=='new': sys.exit(1)
 with open(os.environ['LAUNCHES'],'a') as f: f.write(marker.read_text()+'\\n')
elif cmd=='rm': shutil.rmtree(args[-1])
'''

@unittest.skipIf(sys.platform == 'win32', 'Mac POSIX restart helper requires /bin/sh and executable Unix fixtures; not Windows updater acceptance')
class RestartTests(unittest.TestCase):
 def run_case(self, fail):
  with tempfile.TemporaryDirectory() as temporary:
   folder=Path(temporary); stage=folder/'stage'; stage.mkdir()
   target=folder/'installed.app'; target.mkdir(); (target/'identity').write_text('old')
   new=stage/'PixaEffects.app'; new.mkdir(); (new/'identity').write_text('new')
   tools=folder/'tools'; tools.mkdir()
   script=(ROOT/'pixaeffects/overlay/crates/ui-egui/src/pixaeffects_restart.sh').read_text()
   for name, original in [('codesign','/usr/bin/codesign'),('spctl','/usr/sbin/spctl'),('mv','/bin/mv'),('open','/usr/bin/open'),('rm','/bin/rm')]:
    tool=tools/name; tool.write_text(HANDLER); tool.chmod(0o755)
    script=script.replace(original, str(tool))
   helper=folder/'restart.sh'; helper.write_text(script)
   launches=folder/'launches'
   result=subprocess.run(['/bin/sh',str(helper),'99999999',str(stage),str(target)],env={**os.environ,'FAIL':fail,'TARGET':str(target),'LAUNCHES':str(launches)},capture_output=True,text=True,timeout=10)
   return result.returncode,(target/'identity').read_text(),launches.read_text() if launches.exists() else '',stage.exists()
 def test_success_installs_and_launches_new(self):
  self.assertEqual(self.run_case(''),(0,'new','new\n',False))
 def test_signature_failure_preserves_old_without_launch(self):
  status,identity,launches,_=self.run_case('signature'); self.assertNotEqual(status,0); self.assertEqual((identity,launches),('old',''))
 def test_wrong_publisher_preserves_old(self):
  status,identity,launches,_=self.run_case('publisher'); self.assertNotEqual(status,0); self.assertEqual((identity,launches),('old',''))
 def test_notary_rejection_preserves_old(self):
  status,identity,launches,_=self.run_case('notary'); self.assertNotEqual(status,0); self.assertEqual((identity,launches),('old',''))
 def test_failed_replacement_restores_and_launches_old(self):
  status,identity,launches,_=self.run_case('replace'); self.assertNotEqual(status,0); self.assertEqual((identity,launches),('old','old\n'))
 def test_failed_launch_restores_and_launches_old(self):
  status,identity,launches,_=self.run_case('launch'); self.assertNotEqual(status,0); self.assertEqual((identity,launches),('old','old\n'))

if __name__=='__main__': unittest.main()
