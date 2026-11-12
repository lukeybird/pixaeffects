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
 publication_metadata(root,app,names)
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


def publication_metadata(root,app,names):
 """Rebrand distribution metadata and translated display strings, never legal notices."""
 for file in (root/'packaging').rglob('*'):
  if not file.is_file():continue
  text=file.read_text(encoding='utf-8') if file.suffix in {'.yml','.yaml','.in'} else None
  if text is None:continue
  if file.suffix in {'.yml','.yaml'}:
   text=text.replace('ai.storyteller.'+app['source'],app['bundle_id'])
   # The Pixa updater must never read the upstream version cache.
   text=''.join(line for line in text.splitlines(keepends=True) if '--filesystem=xdg-data/ArtCraft/versions:ro' not in line)
  if file.name.endswith('.metainfo.xml.in'):
   text=re.sub(r'<developer id="ai.storyteller">[\s\S]*?</developer>','<developer id="com.aiwebdesignfirm"><name>AIWebDesignFirm</name></developer>',text)
   # Upstream screenshots are not evidence of the Pixa release UI.
   text=re.sub(r'\s*<screenshots>[\s\S]*?</screenshots>','',text)
   text=re.sub(r'<url type="(?:homepage|bugtracker|help)">[^<]*</url>',lambda m:'<url type="'+re.search(r'type="([^"]+)"',m[0])[1]+'">https://aiwebdesignfirm.com/products/'+app['id']+'</url>',text)
  file.write_text(text,encoding='utf-8')
 for file in (root/'crates').rglob('locales/*.json'):
  values=json.loads(file.read_text(encoding='utf-8'))
  if not isinstance(values,dict):continue
  updated={}
  def display(value):
   for old,new in names.items():value=value.replace(old,new)
   return value.replace('ArtCraft','AIWebDesignFirm')
  for key,value in values.items():
   if not isinstance(value,str):updated[key]=value;continue
   # Retire promotional entries; an untranslated account/help label is preferable to a false community link.
   if 'Discord' in key or 'GitHub' in key:continue
   if 'copyright' in key.lower():updated[key]=value;continue
   updated[display(key)]=display(value)
  file.write_text(json.dumps(updated,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
 # Internal command IDs remain stable; only this previously missed visible label changes.
 file=root/'crates/ui-egui/src/menus.rs'
 if file.is_file():
  text=file.read_text(encoding='utf-8').replace('"Join the community (Discord)..."','"Your account…"')
  file.write_text(text,encoding='utf-8')
