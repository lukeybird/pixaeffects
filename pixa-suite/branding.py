"""Display branding only; retain internal IDs, file formats, legal and contributor attribution."""
from pathlib import Path
import json,re
ROOT=Path(__file__).resolve().parent

def product_links(text,app):
 """Replace exact promotional string literals, not command IDs or legal credits."""
 replacements={
  'Join us on Discord':'Manage your Pixa account',
  'Join our community on Discord':'Manage your Pixa account',
  'Join us on Discord: aiwebdesignfirm.com/settings':'Manage your Pixa account',
  'Community (Discord)':'Manage your Pixa account',
  'https://aiwebdesignfirm.com/settings':'https://aiwebdesignfirm.com/account',
  'aiwebdesignfirm.com/settings':'aiwebdesignfirm.com/account',
  'https://aiwebdesignfirm.com/apps/'+app['source']:'https://aiwebdesignfirm.com/products/'+app['id'],
  'aiwebdesignfirm.com/apps/'+app['source']:'aiwebdesignfirm.com/products/'+app['id'],
  'Source code: github.com/storytold/'+app['source']:app['name']+' User Manual',
  'https://aiwebdesignfirm.com#readme':'https://aiwebdesignfirm.com/pixa-showcase/manuals/'+app['id'],
 }
 def change(match):
  value=match.group(0)[1:-1]
  value=replacements.get(value,value)
  # Also correct URLs embedded in export provenance and CLI/MCP help strings.
  value=value.replace('aiwebdesignfirm.com/apps/'+app['source'],'aiwebdesignfirm.com/products/'+app['id'])
  value=value.replace('aiwebdesignfirm.com/settings','aiwebdesignfirm.com/account')
  return '"'+value+'"'
 lines=[]
 for line in text.splitlines(keepends=True):
  if 'copyright' in line.lower() or line.lstrip().startswith(('//','/*','*')):lines.append(line);continue
  # The old source-promotion label must open the manual, not the generic home.
  if 'ui.hyperlink_to' in line and 'Source code: github.com/storytold/'+app['source'] in line:
   line=line.replace('"https://aiwebdesignfirm.com"','"https://aiwebdesignfirm.com/pixa-showcase/manuals/'+app['id']+'"')
  if re.search(r'\bconst GITHUB\s*:',line):
   line=line.replace('"https://aiwebdesignfirm.com"','"https://aiwebdesignfirm.com/pixa-showcase/manuals/'+app['id']+'"')
  if re.search(r'\bconst ISSUES\s*:',line):
   line=line.replace('"https://aiwebdesignfirm.com"','"https://aiwebdesignfirm.com/products/'+app['id']+'"')
  if re.search(r'\bconst APPS\s*:',line):
   line=line.replace('"https://aiwebdesignfirm.com/apps"','"https://aiwebdesignfirm.com/products"')
  lines.append(re.sub(r'"(?:\\.|[^"\\])*"',change,line))
 return ''.join(lines)

def presentation_violations(root,app):
 """Find residual known promotions in production Rust, excluding legal text."""
 forbidden=['Join us on Discord','Join our community on Discord','Community (Discord)',
            'Source code: github.com/storytold/'+app['source'],
            'aiwebdesignfirm.com/apps/'+app['source']]
 failures=[]
 for base in ['apps','crates']:
  for file in (root/base).rglob('*.rs'):
   relative=file.relative_to(root)
   if any(part in {'tests','credits','contributors','locales','i18n'} for part in relative.parts) or 'i18n' in file.name:continue
   if any(token in file.name.lower() for token in ['test','credits','contributors','license','attribution']):continue
   for number,line in enumerate(file.read_text(encoding='utf-8').splitlines(),1):
    if 'copyright' in line.lower() or line.lstrip().startswith(('//','/*','*')):continue
    for literal in re.findall(r'"(?:\\.|[^"\\])*"',line):
     if any(token in literal for token in forbidden):
      failures.append(str(relative)+':'+str(number)+': remaining product promotion')
 return failures

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
   # Translation catalogs retain their original lookup keys; the changed UI
   # label falls back to English rather than reusing a false Discord translation.
   if app['id'] not in {'pixavector','pixafilm'} and 'i18n' not in str(file.relative_to(root)) and 'locales' not in file.parts:
    updated=product_links(updated,app)
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
