//! Native main files include the latest live branch tabs and complete saved version histories.
use crate::{Session,branches};
use std::path::Path;
use std::io::Write;
pub fn save(session:&Session,directory:&Path)->Result<usize,String>{
 if !directory.is_dir(){return Err("Choose an existing recovery folder.".into())}
 let mut entries=Vec::new();
 for state in session.docs.iter().filter(|d|d.branch_of.is_none()){
  let (doc,versions,branches)=branches::recovery_copy(session,state);
  let bytes=figcraft_format::save_bytes_with_history(&doc,None,&versions,&branches).map_err(|e|e.to_string())?;
  let name=format!("document-{}.figcraft",state.uid);write_new(&directory.join(&name),&bytes)?;
  entries.push(serde_json::json!({"file":name,"original_path":state.path,"name":doc.name}));
 }
 write_new(&directory.join("RECOVERY.json"),&serde_json::to_vec_pretty(&entries).map_err(|e|e.to_string())?)?;Ok(entries.len())
}
fn write_new(path:&Path,bytes:&[u8])->Result<(),String>{
 let mut options=std::fs::OpenOptions::new();options.write(true).create_new(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
 let mut file=options.open(path).map_err(|e|e.to_string())?;file.write_all(bytes).and_then(|()|file.sync_all()).map_err(|e|e.to_string())
}
