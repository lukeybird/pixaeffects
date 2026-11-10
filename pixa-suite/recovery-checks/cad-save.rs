//! Native drawing copies include unsaved Block Editor changes without changing the live document.
use crate::Session;
use std::path::Path;
use std::io::Write;
pub fn save(session:&Session,directory:&Path)->Result<usize,String>{
 if !directory.is_dir(){return Err("Choose an existing recovery folder.".into())}
 let hooks=crate::cmd::file::io().ok_or("Native drawing codecs are not installed.")?;
 let mut entries=Vec::new();
 for (index,state) in session.docs.iter().enumerate(){
  let drawing=state.recovery_drawing().map_err(|e|e.to_string())?;
  if !crate::cmd::file::not_saved(&drawing).is_empty(){return Err("This drawing contains data the recovery writer cannot preserve. Reconnect and save it with the editor.".into())}
  let name=format!("drawing-{index}.dwg");let bytes=(hooks.write)(&drawing,&name)?;
  // Refuse a corrupt copy before it is presented as a completed recovery file.
  (hooks.read)(&bytes,&name)?;write_new(&directory.join(&name),&bytes)?;
  entries.push(serde_json::json!({"file":name,"original_path":state.path,"title":state.title,"block_editor":state.block_editor()}));
 }
 write_new(&directory.join("RECOVERY.json"),&serde_json::to_vec_pretty(&entries).map_err(|e|e.to_string())?)?;Ok(entries.len())
}
fn write_new(path:&Path,bytes:&[u8])->Result<(),String>{
 let mut options=std::fs::OpenOptions::new();options.write(true).create_new(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
 let mut file=options.open(path).map_err(|e|e.to_string())?;file.write_all(bytes).and_then(|()|file.sync_all()).map_err(|e|e.to_string())
}
