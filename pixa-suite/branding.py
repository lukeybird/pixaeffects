"""Display branding only; retain internal IDs, file formats, legal and contributor attribution."""
from pathlib import Path
import json,re
ROOT=Path(__file__).resolve().parent

def display_strings(text,names):
 if not any(name in text for name in names) and ' on GitHub' not in text and 'discord.gg/artcraft' not in text:return text
 def replace(match):
  value=match.group(0)
  for old,new in names.items():value=re.sub(r'\b'+re.escape(old)+r'\b',new,value)
  value=value.replace(' on GitHub',' help').replace('discord.gg/artcraft','aiwebdesignfirm.com/settings')
  return value
 return re.sub(r'"(?:\\.|[^"\\])*"',replace,text)

def install(root,app):
 catalog=json.loads((ROOT/'catalog.json').read_text(encoding='utf-8'))
 names={a['upstream_name']:a['name'] for a in catalog['apps']}
 names.update({'PhotoCraft':'PixaShop','EffectCraft':'PixaEffects'})
 for base in ['apps','crates']:
  for file in (root/base).rglob('*.rs'):
   if any(x in {'credits','contributors'} for x in file.parts) or any(x in file.name.lower() for x in ['license','notice','attribution','contributors','credits']):continue
   text=file.read_text(encoding='utf-8')
   # A line containing a copyright notice is never relabeled.
   updated=''.join(line if 'copyright' in line.lower() else display_strings(line,names) for line in text.splitlines(keepends=True))
   if updated!=text:file.write_text(updated,encoding='utf-8')
 # This editor draws its publisher mark as an inline SVG path rather than an image asset.
 if app['id']=='pixadesign':
  file=root/'crates/ui-egui/src/about.rs';text=file.read_text(encoding='utf-8')
  from icon_art import inline_path
  path=inline_path(app['icon_mark'])
  text=re.sub(r'const MARK_PATH: &str = "[^"]*";',f'const MARK_PATH: &str = "{path}";',text)
  rgb=tuple(bytes.fromhex(app['color'].lstrip('#')))
  text=re.sub(r'pub const BRAND: Color32 = Color32::from_rgb\([^;]+;',f'pub const BRAND: Color32 = Color32::from_rgb({rgb[0]}, {rgb[1]}, {rgb[2]});',text)
  text=re.sub(r'pub const SIBLINGS: &\[\(&str, &str, &str\)\] = &\[[\s\S]*?\n\];','pub const SIBLINGS: &[(&str, &str, &str)] = &[("pixashop", "PixaShop", "image editing"), ("pixaeffects", "PixaEffects", "motion graphics and visual effects")];',text)
  text=text.replace('format!("{}/apps/{slug}", links::WEBSITE)','format!("{}/{slug}", links::WEBSITE)')
  text=text.replace('"Open source. No telemetry."','"by AIWebDesignFirm. Account access checks require an internet connection."')
  file.write_text(text,encoding='utf-8')

 # The existing PixaShop package defines these exact bundle/executable names.
 if app['id']=='pixalight':
  file=root/'apps/lightcraft/src/editors.rs';text=file.read_text(encoding='utf-8')
  text=text.replace('"ai.storyteller.photocraft"','"com.aiwebdesignfirm.pixashop"').replace('aiwebdesignfirm.com/apps/photocraft','aiwebdesignfirm.com/pixashop')
  # Audited literal replacements: do not run a multiline string regex over Rust comments.
  text=text.replace('"photocraft"','"pixashop"').replace('photocraft.exe','PixaShop.exe').replace('photocraft-','pixashop-').replace('/opt/photocraft/','/opt/pixashop/').replace('/bin/photocraft','/bin/pixashop')
  file.write_text(text,encoding='utf-8')

