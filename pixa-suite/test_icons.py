"""Packaging-art checks; do not imply a native editor/release build."""
import importlib.util, tempfile, unittest, xml.etree.ElementTree as ET
from pathlib import Path
from PIL import Image
import prepare
from icon_art import rectangles, svg

class ProductIcons(unittest.TestCase):
 def test_requested_product_marks(self):
  expected={'pixavector':'V','pixafilm':'F','pixalight':'L','pixapdf':'PDF','pixadesign':'D','pixainterface':'I','pixatexture':'T','pixamaterial':'M','pixaword':'W','pixasound':'S','pixadeck':'D','pixacad':'CAD','pixagrid':'G','pixaanything':'A'}
  self.assertEqual({a['id']:a['icon_mark'] for a in prepare.CATALOG},expected)
 def test_all_sizes_and_svg_share_geometry(self):
  for app in prepare.CATALOG:
   for size in (16,32,128,1024):
    image=prepare.icon(app,size)
    self.assertEqual(image.size,(size,size))
    self.assertEqual(image.getpixel((0,0))[3],0)
   doc=ET.fromstring(svg(app['icon_mark'],app['color'],app['icon_background']))
   self.assertEqual(len(doc)-1,len(rectangles(app['icon_mark'])))
   self.assertTrue(all(float(r.attrib['x'])>=0 and float(r.attrib['y'])>=0 for r in list(doc)[1:]))
 def test_suite_packaging_assets_and_sidecars(self):
  for app in prepare.CATALOG:
   with tempfile.TemporaryDirectory() as td:
    root=Path(td);folder=root/'assets/app-icon';folder.mkdir(parents=True)
    Image.new('RGBA',(128,128)).save(folder/'icon.png')
    (folder/'icon.svg').write_text('<svg/>')
    prepare.brand_assets(root,app)
    self.assertEqual(Image.open(folder/'icon.png').tobytes(),prepare.icon(app,128).tobytes())
    self.assertEqual((folder/'icon.svg').read_text(),svg(app['icon_mark'],app['color'],app['icon_background']))
    self.assertTrue((folder/'icon.png.attribution').is_file())
 def test_standalone_shop_and_effects_packaging(self):
  base=Path(__file__).resolve().parents[2]
  for branch,product,mark,color,bg,asset in [('pixashop-product','pixashop','S','#54c8ff','#101e32','photocraft'),('pixaeffects-product','pixaeffects','E','#b89aff','#211536','effectcraft')]:
   path=base/branch/product/'icons.py'
   # Standalone overlay checks are optional outside this multi-repository worktree.
   if not path.is_file():self.skipTest('Standalone product overlays are in separate branches')
   spec=importlib.util.spec_from_file_location(product+'_icons',path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
   with tempfile.TemporaryDirectory() as td:
    root=Path(td);folder=root/'assets/app-icon';folder.mkdir(parents=True)
    Image.new('RGBA',(128,128)).save(folder/'icon.png');(folder/'icon.svg').write_text('<svg/>')
    module.generate(root)
    self.assertEqual(Image.open(folder/'icon.png').tobytes(),module.render(mark,color,bg,128).tobytes())
    self.assertEqual((folder/'icon.svg').read_text(),module.svg(mark,color,bg))
    for ext in ('ico','icns'):self.assertTrue((folder/(asset+'.'+ext)).is_file())
if __name__=='__main__':unittest.main()
