"""Generate original product-specific starter guides. This is not an exhaustive tool reference."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
WORKFLOWS=json.loads((ROOT/'manuals/workflows.json').read_text(encoding="utf-8"))
def tool_chapters(app,source=None):
 if app['id']!='pixadesign':return []
 import re
 catalog=(source or ROOT/'sources/designcraft')/'crates/tools/src/catalog.rs'
 if not catalog.is_file():return []
 descriptions=json.loads((ROOT/'manuals/design-tools.json').read_text(encoding='utf-8'))
 rows=[]
 pattern=r't\("([^"]+)", "([^"]+)", (Some\("(?:[^"\\]|\\.)*"\)|None),'
 for match in re.finditer(pattern,catalog.read_text(encoding='utf-8')):
  identifier,label,shortcut=match.groups()
  if identifier not in descriptions:raise RuntimeError('Missing tool explanation: '+identifier)
  key=json.loads(shortcut[5:-1]) if shortcut.startswith('Some(') else None
  body=descriptions[identifier]+' Find this in the Tools panel; grouped tools share a flyout.'
  if key:body+=' Default shortcut: '+key+'. Custom keymaps may differ.'
  rows.append({'title':label,'body':body,'tool_id':identifier})
 if len(rows)!=len(descriptions):raise RuntimeError('Tool catalog and explanations differ')
 return rows
def guide(app,source=None):
 w=WORKFLOWS[app['id']]
 chapters=[{'title':'Start a first project','body':w['start']},{'title':'Make a first edit','body':w['work']},{'title':'Keep your work organized','body':w['organize']},{'title':'Save and export','body':w['finish']},{'title':'Your account','body':'Use Your account in the Pixa footer to open the account connection window. Passwords and payment details stay in your browser. Only approve an activation code displayed on your own device. This unpublished build tests account connectivity; editing enforcement and persistent sign-in are not yet release-ready.'},{'title':'Updates','body':'Use Updates in the Pixa footer to see update availability. Updates are deliberately disabled until the publisher configures a signed release feed and completes platform installation tests. You do not need to install an ArtCraft update for a Pixa product.'}]
 recovery=' If account access expires in a subscription preview, keep the app open and use Save recovery copy to choose a destination. Reconnect before closing if you have unfinished text fields or background operations. Open the recovered native files in the editor after reconnecting; retain the original files.'
 if app['id']=='pixalight':recovery+=' Library recovery includes catalogue, presets and preferences; original photos remain in their existing locations and must be kept.'
 if app['id']=='pixaanything':recovery+=' PixaAnything copies drafts and attachments into a session.pixasession file. Use Restore drafts in a fresh idle session after reconnecting. Keep existing project folders; jobs do not resume automatically. Provider settings, keys and account credentials are excluded.'
 chapters[3]['body']+=recovery
 return {'schema_version':1,'product':app['id'],'name':app['name'],'title':app['name']+' Starter Manual','scope':'Starter workflow and orientation; not a complete reference for every tool.','chapters':chapters,'tour':chapters[:4],'tools':tool_chapters(app,source)}
def markdown(app,source=None):
 data=guide(app,source)
 return '# '+data['title']+'\n\nby AIWebDesignFirm\n\n'+data['scope']+'\n\n'+ '\n\n'.join('## '+c['title']+'\n\n'+c['body'] for c in data['chapters']+data.get('tools',[]))+'\n'
