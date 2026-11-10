//! Recover native drafts without copying AI settings, key inputs or credential stores.
use crate::{CraftApp,Prefs,PREFS_KEY,state};
use serde::{Serialize,Deserialize};
use std::path::{Path,PathBuf};
use std::collections::BTreeMap;
use std::io::{Read,Write};
const MAX_JSON:u64=16*1024*1024;
const MAX_ASSET:u64=64*1024*1024;
#[derive(Serialize,Deserialize)]
struct Attachments{root:PathBuf,files:Vec<PathBuf>}
#[derive(Serialize,Deserialize)]
struct Note{root:PathBuf,text:String,files:Vec<PathBuf>}
#[derive(Serialize,Deserialize)]
struct Snapshot{
 version:u32,product:String,prefs:Prefs,chat:Vec<state::ChatItem>,input:String,url_input:String,
 project_input:String,project_attachments:Vec<PathBuf>,unsent:Vec<Attachments>,notes:Vec<Note>,
}
pub fn save(app:&CraftApp,directory:&Path)->Result<usize,String>{
 if !directory.is_dir() || std::fs::read_dir(directory).map_err(|e|e.to_string())?.next().is_some(){return Err("Recovery requires a new empty folder.".into())}
 let directory=std::fs::canonicalize(directory).map_err(|e|e.to_string())?;
 let mut data=Snapshot{version:1,product:"pixaanything".into(),prefs:app.prefs(),chat:app.state.chat.clone(),input:app.state.input.clone(),url_input:app.state.url_input.clone(),project_input:app.state.project_input.clone(),project_attachments:app.state.project_attachments.clone(),unsent:app.state.unsent.iter().map(|(root,(_,files))|Attachments{root:root.clone(),files:files.clone()}).collect(),notes:app.drafts.values().map(|n|Note{root:n.root.clone(),text:n.text.clone(),files:n.attachments.clone()}).collect()};
 let mut copied=BTreeMap::new();let mut total=0;
 if let Some(make)=data.prefs.make.as_mut(){copy_assets(&mut make.pictures,&directory,&mut copied,&mut total)?;}
 copy_assets(&mut data.project_attachments,&directory,&mut copied,&mut total)?;
 for attachments in &mut data.unsent{copy_assets(&mut attachments.files,&directory,&mut copied,&mut total)?;}
 for note in &mut data.notes{copy_assets(&mut note.files,&directory,&mut copied,&mut total)?;}
 let bytes=serde_json::to_vec_pretty(&data).map_err(|e|e.to_string())?;
 if bytes.len() as u64>MAX_JSON{return Err("The draft recovery file is too large. Keep the app open and save individual work first.".into())}
 write_new(&directory.join("session.pixasession"),&bytes)?;
 write_new(&directory.join("READ-ME.txt"),b"Use Restore drafts in PixaAnything in a fresh idle session. Drafts, chat and attached pictures are copied. Project files remain in their existing folders; keep those folders. Settings, provider keys and account credentials are not included. Running jobs may still need to finish before the app closes.")?;
 Ok(1)
}
fn copy_assets(paths:&mut [PathBuf],directory:&Path,copied:&mut BTreeMap<PathBuf,PathBuf>,total:&mut u64)->Result<(),String>{
 for path in paths{
  if let Some(saved)=copied.get(path){*path=saved.clone();continue}
  let file=std::fs::File::open(&*path).map_err(|e|format!("An attached file could not be opened: {e}"))?;
  let metadata=file.metadata().map_err(|e|e.to_string())?;
  if !metadata.is_file() || metadata.len()>MAX_ASSET{return Err("An attachment is not a regular file or exceeds the recovery limit.".into())}
  let mut bytes=Vec::new();file.take(MAX_ASSET+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
  *total=total.saturating_add(bytes.len() as u64);
  if bytes.len() as u64>MAX_ASSET || *total>512*1024*1024{return Err("Attachments exceed the recovery size limit.".into())}
  let assets=directory.join("assets");if !assets.exists(){std::fs::create_dir(&assets).map_err(|e|e.to_string())?;}
  let ext=path.extension().and_then(|e|e.to_str()).filter(|e|e.len()<=12 && e.bytes().all(|b|b.is_ascii_alphanumeric())).unwrap_or("bin");
  let destination=assets.join(format!("attachment-{}.{}",copied.len(),ext));write_new(&destination,&bytes)?;
  copied.insert(path.clone(),destination.clone());*path=destination;
 }
 Ok(())
}
pub fn restore(app:&mut CraftApp,file:&Path)->Result<(),String>{
 if app.jobs.running().next().is_some() || app.runs.any_busy() || app.conversation.is_some() || !app.pending_attach_checks.is_empty() || app.pending_make_checks!=0 || !app.send_after_checks.is_empty() || app.start_after_pictures.is_some(){return Err("Recovery needs a fresh idle session. Keep current work open, or reopen the app first.".into())}
 if !app.state.chat.is_empty() || !app.state.url_input.trim().is_empty() || !app.state.project_attachments.is_empty() || !app.state.input.trim().is_empty() || !app.state.project_input.trim().is_empty() || !app.state.make.request.trim().is_empty() || !app.state.make.url.trim().is_empty() || !app.state.make.pictures.is_empty() || !app.state.unsent.is_empty() || !app.drafts.is_empty(){return Err("Existing drafts were left untouched. Restore into a fresh session.".into())}
 let mut bytes=Vec::new();let reader=std::fs::File::open(file).map_err(|e|e.to_string())?;
 if !reader.metadata().map_err(|e|e.to_string())?.is_file(){return Err("Choose a regular Pixa session file.".into())}
 reader.take(MAX_JSON+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
 if bytes.len() as u64>MAX_JSON{return Err("The recovery file is too large.".into())}
 let data:Snapshot=serde_json::from_slice(&bytes).map_err(|_|"This is not a readable Pixa draft recovery file.")?;
 if data.version!=1 || data.product!="pixaanything"{return Err("This recovery file belongs to a different product or version.".into())}
 // Use native preference restoration, with no writeable OS/settings storage.
 struct MemoryPrefs(String);
 impl eframe::Storage for MemoryPrefs{
  fn get_string(&self,key:&str)->Option<String>{(key==PREFS_KEY).then(||self.0.clone())}
  fn set_string(&mut self,_key:&str,_value:String){}
  fn remove_string(&mut self,_key:&str){}
  fn flush(&mut self){}
 }
 let prefs=serde_json::to_string(&data.prefs).map_err(|e|e.to_string())?;
 app.load_prefs(Some(&MemoryPrefs(prefs)));
 app.state.chat=data.chat;for item in &mut app.state.chat{if let state::ChatItem::Assistant{streaming,..}=item{*streaming=false;}}app.state.input=data.input;app.state.url_input=data.url_input;app.state.project_input=data.project_input;app.state.project_attachments=data.project_attachments;
 let active=app.workspace.as_ref().map(|w|w.root.clone());
 for attachments in data.unsent{
  if active.as_ref()==Some(&attachments.root){for file in attachments.files{if !app.state.project_attachments.contains(&file){app.state.project_attachments.push(file);}}}
  else{app.state.unsent.entry(attachments.root).or_default().1=attachments.files;}
 }
 for note in data.notes{
  let (text,files)=if active.as_ref()==Some(&note.root){(&mut app.state.project_input,&mut app.state.project_attachments)}else{let pair=app.state.unsent.entry(note.root).or_default();(&mut pair.0,&mut pair.1)};
  if !text.is_empty(){text.push_str("\n\n");}text.push_str(&note.text);
  for file in note.files{if !files.contains(&file){files.push(file);}}
 }
 Ok(())
}
fn write_new(path:&Path,bytes:&[u8])->Result<(),String>{
 let mut options=std::fs::OpenOptions::new();options.write(true).create_new(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
 let mut file=options.open(path).map_err(|e|e.to_string())?;file.write_all(bytes).and_then(|()|file.sync_all()).map_err(|e|e.to_string())
}
