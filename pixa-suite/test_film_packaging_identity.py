"""The rebranded installer cannot upgrade the independent upstream product."""
import json, tempfile, unittest
from pathlib import Path
from film_integration import rebrand_packaging

class FilmPackagingIdentityTests(unittest.TestCase):
 def test_independent_upgrade_identity_and_legal_notices(self):
  with tempfile.TemporaryDirectory() as td:
   root=Path(td);packaging=root/'packaging/windows';packaging.mkdir(parents=True)
   policy=packaging/'install-policy.json'
   policy.write_text(json.dumps({'name':'FilmCraft','upgradeCode':'31E6FB14-9DBC-41DD-B24B-26B71FB31B62','payloads':['filmcraft.exe'],'registry':[r'Software\Microsoft\Windows\CurrentVersion\App Paths\filmcraft.exe']}))
   source=packaging/'filmcraft.wxs'
   source.write_text('<Package Name="FilmCraft" Manufacturer="Learning Machines LLC" UpgradeCode="31E6FB14-9DBC-41DD-B24B-26B71FB31B62" />\n<!-- Copyright the FilmCraft contributors -->\n')
   rebrand_packaging(root)
   data=json.loads(policy.read_text());self.assertEqual(data['name'],'PixaFilm')
   self.assertEqual(data['upgradeCode'],'2029AEB3-50B9-5F36-A7A8-A6B59605C416')
   self.assertEqual(data['payloads'],['filmcraft.exe'])
   self.assertTrue(data['registry'][0].endswith(r'App Paths\PixaFilm.exe'))
   self.assertIn('Manufacturer="AIWebDesignFirm"',source.read_text())
   self.assertNotIn('31E6FB14-',source.read_text())
   self.assertIn('Copyright the FilmCraft contributors',source.read_text())

if __name__=='__main__':unittest.main()
