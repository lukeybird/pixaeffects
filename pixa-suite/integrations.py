"""Install shared desktop modules; only verified native copy commands become recovery pilots."""
from pathlib import Path
import json
from manuals import guide
ROOT=Path(__file__).resolve().parent
PILOTS={'pixafilm','pixatexture'}

def install(root,app):
 main=root/'apps'/app['source']/'src/main.rs'
 editor={'pixafilm':'filmcraft_ui_egui::FilmcraftApp','pixatexture':'crate::Shell'}.get(app['id'],'dyn eframe::App')
 on_exit='fn on_exit(&mut self){eframe::App::on_exit(self.inner.as_mut());}'
 if app['source'] in {'lightcraft','pdfcraft'}:on_exit='fn on_exit(&mut self,gl:Option<&eframe::glow::Context>){eframe::App::on_exit(self.inner.as_mut(),gl);}'
 text=(ROOT/'templates/shell.rs').read_text(encoding='utf-8')
 for key,value in {'NAME':app['name'],'ID':app['id'],'EDITOR_TYPE':editor,'ON_EXIT':on_exit}.items():text=text.replace('@'+key+'@',value)
 (main.parent/'pixa_shell.rs').write_text(text,encoding='utf-8')
 for template,destination in [('account_ui.rs','pixa_account.rs'),('help_ui.rs','pixa_help.rs')]:
  text=(ROOT/'templates'/template).read_text(encoding='utf-8').replace('@NAME@',app['name']).replace('@ID@',app['id'])
  (main.parent/destination).write_text(text,encoding='utf-8')
 (main.parent/'pixa-guide.json').write_text(json.dumps(guide(app),indent=2)+'\n',encoding='utf-8')
 if app['id']=='pixafilm':
  body='''let file=directory.join("recovery.fcproj");let path=file.to_str().ok_or("Choose a folder with a Unicode path.")?;
 editor.session.execute("file.saveCopy",serde_json::json!({"path":path})).map_err(|e|e.to_string())?;Ok(())'''
 elif app['id']=='pixatexture':
  body='''editor.app.session.end_stroke().map_err(|e|e.to_string())?;
 let file=directory.join("recovery.tcraft");let path=file.to_str().ok_or("Choose a folder with a Unicode path.")?;
 texturecraft_engine::execute(&mut editor.app.session,"file.saveCopy",&serde_json::json!({"path":path})).map_err(|e|e.to_string())?;Ok(())'''
 else:body='let _=(editor,directory);Err("Recovery adapter is not configured for this editor.".into())'
 ready=app['id'] in PILOTS
 recovery='use super::Editor;\nuse std::path::{Path,PathBuf};\npub fn ready()->bool{'+str(ready).lower()+'}\n'
 recovery+='pub fn choose_folder()->Option<PathBuf>{'+('rfd::FileDialog::new().pick_folder()' if ready else 'None')+'}\n'
 recovery+='pub fn save(editor:&mut Editor,directory:&Path)->Result<(),String>{'+body+'}\n'
 (main.parent/'pixa_recovery.rs').write_text(recovery,encoding='utf-8')
 manifest=main.parent.parent/'Cargo.toml';text=manifest.read_text(encoding='utf-8')
 import os
 runtime_path=Path(os.path.relpath(ROOT/'runtime',manifest.parent)).as_posix()
 if 'pixa-runtime =' not in text:text=text.replace('[dependencies]','[dependencies]\npixa-runtime = { path = '+json.dumps(runtime_path)+' }',1)
 manifest.write_text(text,encoding='utf-8')
 text=main.read_text(encoding='utf-8')
 if app['id']=='pixafilm' and 'if pixa_shell::subscription_build()' not in text:
  old='let control = control_port.map(';new='let control = (if pixa_shell::subscription_build() { None } else { control_port }).map('
  if old not in text:raise RuntimeError('Film automation integration point changed')
  text=text.replace(old,new,1)
 if app['id']=='pixatexture' and 'if pixa_shell::subscription_build()' not in text:
  old='let control = args.control;';new='let control = if pixa_shell::subscription_build() { None } else { args.control };'
  if old not in text:raise RuntimeError('Texture automation integration point changed')
  text=text.replace(old,new,1)
 main.write_text(text,encoding='utf-8')

 if app['id'] in PILOTS:
  example=root/'crates/engine/examples/pixa_recovery_check.rs';example.parent.mkdir(parents=True,exist_ok=True)
  source='film.rs' if app['id']=='pixafilm' else 'texture.rs'
  example.write_text((ROOT/'recovery-checks'/source).read_text(encoding='utf-8'),encoding='utf-8')
