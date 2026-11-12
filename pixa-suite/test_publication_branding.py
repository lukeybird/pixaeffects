import json,sys,tempfile,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
from branding import publication_metadata
class PublicationBrandingTests(unittest.TestCase):
 def test_metadata_and_locales_preserve_legal_and_document_identifiers(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d);package=root/'packaging/linux';package.mkdir(parents=True)
   flat=package/'app.yml';flat.write_text('id: ai.storyteller.vectorcraft\n  - --filesystem=xdg-data/ArtCraft/versions:ro\nformat: vectorcraft\n')
   meta=package/'app.metainfo.xml.in';meta.write_text('<component><developer id="ai.storyteller"><name>Learning Machines LLC</name></developer><url type="homepage">https://old.invalid</url><screenshots><screenshot>old</screenshot></screenshots><project_license>MIT OR Apache-2.0</project_license></component>')
   locale=root/'crates/ui-egui/locales/de.json';locale.parent.mkdir(parents=True);locale.write_text(json.dumps({'VectorCraft':'VectorCraft','Join the ArtCraft Discord':'old','Copyright ArtCraft':'Copyright ArtCraft','format':'vectorcraft'}))
   app={'source':'vectorcraft','bundle_id':'com.aiwebdesignfirm.pixavector','id':'pixavector'}
   publication_metadata(root,app,{'VectorCraft':'PixaVector'})
   self.assertIn('com.aiwebdesignfirm.pixavector',flat.read_text());self.assertNotIn('ArtCraft/versions',flat.read_text());self.assertIn('format: vectorcraft',flat.read_text())
   self.assertIn('AIWebDesignFirm',meta.read_text());self.assertNotIn('<screenshots>',meta.read_text());self.assertIn('MIT OR Apache-2.0',meta.read_text())
   values=json.loads(locale.read_text());self.assertEqual(values,{'PixaVector':'PixaVector','Copyright ArtCraft':'Copyright ArtCraft','format':'vectorcraft'})
if __name__=='__main__':unittest.main()
