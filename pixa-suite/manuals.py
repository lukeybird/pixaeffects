"""Generate original product-specific starter guides. This is not an exhaustive tool reference."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
WORKFLOWS=json.loads((ROOT/'manuals/workflows.json').read_text(encoding="utf-8"))
def design_tool_chapters(app,source=None):
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
REGISTRIES={
 'pixavector':('vectorcraft','crates/tools/src/catalog.rs','catalog'),
 'pixafilm':('filmcraft','crates/ui-egui/src/state.rs','Tool'),
 'pixasound':('soundcraft','crates/model/src/session.rs','Tool'),
 'pixatexture':('texturecraft','crates/engine/src/session.rs','Tool'),
 'pixainterface':('figcraft','crates/engine/src/canvas/tools.rs','ToolId'),
}
def control_chapters(app,source=None):
 data=json.loads((ROOT/'manuals/control-guides.json').read_text(encoding='utf-8')).get(app['id'])
 if not data:return []
 file=(source or ROOT/'sources'/app['source'])/data['source_path']
 if not file.is_file():return []
 text=file.read_text(encoding='utf-8');rows=[]
 for item in data['sections']:
  if item['anchor'] not in text:raise RuntimeError('Manual control reference changed: '+app['id']+': '+item['title'])
  rows.append({'title':item['title'],'body':item['body']})
 return rows

def anything_tools(app,source=None):
 import re
 root=(source or ROOT/'sources'/app['source'])/'crates/engine/src/tools'
 if not root.exists():return []
 literal=r'("(?:[^"\\]|\\.)*")'
 pattern=r'ToolSpec\s*\{\s*id:\s*'+literal+r',\s*title:\s*'+literal+r',\s*summary:\s*'+literal+r',\s*beginner:\s*'+literal+r','
 rows={}
 for file in sorted(root.rglob('*.rs')):
  for match in re.finditer(pattern,file.read_text(encoding='utf-8')):
   identifier,title,summary,beginner=[json.loads(v) for v in match.groups()]
   rows[identifier]={'title':title,'body':beginner+' '+summary,'tool_id':identifier}
 return list(rows.values())

def tool_chapters(app,source=None):
 if app['id']=='pixadesign':return design_tool_chapters(app,source)
 if app['id']=='pixaanything':return anything_tools(app,source)
 if app['id'] not in REGISTRIES:return control_chapters(app,source)
 import re
 folder,path,kind=REGISTRIES[app['id']];file=(source or ROOT/'sources'/folder)/path
 if not file.is_file():return []
 text=file.read_text(encoding='utf-8');notes=json.loads((ROOT/'manuals/tool-notes.json').read_text())[app['id']];rows=[]
 if kind=='catalog':
  pattern=r't\("([^"\\]+)", "((?:[^"\\]|\\.)*)", (Some\("(?:[^"\\]|\\.)*"\)|None),'
  for m in re.finditer(pattern,text):
   identifier,label,shortcut=m.groups();body=notes.get(identifier)
   if body is None:raise RuntimeError('Missing tool explanation: '+identifier)
   if shortcut.startswith('Some('):body+=' Default shortcut: '+json.loads(shortcut[5:-1])+'. Custom keymaps may differ.'
   rows.append({'title':json.loads('"'+label+'"'),'body':body,'tool_id':identifier})
 else:
  block=text.split('impl '+kind+' {',1)[1];all_block=block.split('pub const ALL:',1)[1].split('];',1)[0]
  identifiers=re.findall(re.escape(kind)+r'::(\w+)',all_block)
  labels=dict(re.findall(re.escape(kind)+r'::(\w+) => "([^"]+)"',block.split('pub fn label',1)[1].split('\n    }',1)[0]))
  for identifier in identifiers:
   if identifier not in notes:raise RuntimeError('Missing tool explanation: '+identifier)
   rows.append({'title':labels.get(identifier,identifier),'body':notes[identifier],'tool_id':identifier})
 if {r['tool_id'] for r in rows}!=set(notes):raise RuntimeError('Tool catalog and explanations differ: '+app['id'])
 return rows
def guide(app,source=None):
 w=WORKFLOWS[app['id']]
 chapters=[{'title':'Start a first project','body':w['start']},{'title':'Make a first edit','body':w['work']},{'title':'Keep your work organized','body':w['organize']},{'title':'Save and export','body':w['finish']},{'title':'Your account','body':'Use Your account in the Pixa footer to connect this device. Pixa is free early access with no card requirement and no scheduled account expiration. Only approve the code displayed on your own device. Device access refreshes online; a disconnected, revoked or signed-out device must reconnect. Your files remain local.'},{'title':'Updates','body':'Use Updates in the Pixa footer to check the Pixa publisher feed. On Mac, download the candidate, save your work and choose Install and restart. The app checks the installer digest, Apple publisher signature, bundle identity and Gatekeeper before replacement. A canceled editor-close prompt cancels the pending restart. Windows and Linux open the Pixa download page. No update is offered until a matching publisher release exists.'}]
 recovery=' If the device access refresh fails, keep the app open and use Save recovery copy to choose a destination. Reconnect before closing if you have unfinished text fields or background operations. Open the recovered native files in the editor after reconnecting; retain the original files.'
 if app['id']=='pixalight':recovery+=' Library recovery includes catalogue, presets and preferences; original photos remain in their existing locations and must be kept.'
 if app['id']=='pixaanything':recovery+=' PixaAnything copies drafts and attachments into a session.pixasession file. Use Restore drafts in a fresh idle session after reconnecting. Keep existing project folders; jobs do not resume automatically. Provider settings, keys and account credentials are excluded.'
 chapters[3]['body']+=recovery
 chapters.extend([{'title':'Find your way around','body':'The menu bar and editor panels contain the main controls. Use the existing workspace controls to show a hidden panel. Manual and Quick tour in the Pixa footer reopen this help without changing the project.'},{'title':'Undo and safe experiments','body':'Use Undo to reverse a normal document edit, and Redo to restore it. Keep an original project or duplicate before destructive operations. Undo does not reverse an external file deletion, a published upload or every background operation.'},{'title':'Linked files and portability','body':'An editable project may refer to media, fonts, textures or other external files. Keep those resources alongside your project in a stable folder. Copy the complete folder when moving computers and reopen it before removing the original.'},{'title':'Working with another application','body':'Save a native editable copy before exporting an interchange format. Features such as effects, fonts and linked resources may differ in another application. Reopen exported files in the destination application before relying on them.'},{'title':'Recovery and backups','body':'Save regularly and keep a separate backup of important projects and original media. Recovery copies contain committed project state; unfinished text fields, external files and background tasks may need separate attention. Restore into a new project or folder rather than overwriting the original.'},{'title':'Troubleshooting','body':'For missing content, check the active document, selected object, visible panels, layers and linked-file paths. For account problems, reconnect in Your account. For failed updates, keep the existing application and use the verified Pixa download page; do not disable operating-system security checks.'}])
 if app['id']=='pixavector':
  chapters[0]['body']+=' Open Help → PixaVector User Manual or the Manual footer button for offline instructions. Help → Quick Tool Tour walks through all 92 registered tools; Next, Back, Finish and Escape control the read-only bubbles. Visible toolbar groups are highlighted without changing your active tool or document.'
  chapters[5]['body']+=' PixaVector checks its own publisher feed in the background once at startup. An available matching update opens the Updates window; downloading and installation still require your choice.'
 return {'schema_version':1,'product':app['id'],'name':app['name'],'title':app['name']+' User Manual','scope':'Product workflows, account access, recovery and updates. Source-backed tool tours are included where a registered tool catalog is connected; this is not a complete reference for every editor command.','chapters':chapters,'tour':chapters[:4],'tools':tool_chapters(app,source)}
def markdown(app,source=None):
 data=guide(app,source)
 return '# '+data['title']+'\n\nby AIWebDesignFirm\n\n'+data['scope']+'\n\n'+ '\n\n'.join('## '+c['title']+'\n\n'+c['body'] for c in data['chapters']+data.get('tools',[]))+'\n'
