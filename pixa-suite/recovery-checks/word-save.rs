//! Immutable recovery of parked and active documents. Password-protected documents stay encrypted.
use super::Session;
use std::io::Write;
use std::path::Path;
pub fn save(session:&Session,directory:&Path)->Result<usize,String>{
 if !directory.is_dir(){return Err("Choose an existing recovery folder.".into())}
 let active=std::iter::once((session.document_id,&session.doc,&session.path,&session.password));
 let parked=session.others.iter().map(|d|(d.document_id,&d.doc,&d.path,&d.password));
 let mut entries=Vec::new();
 for (id,doc,path,password) in active.chain(parked){
  let ext=if password.is_some(){"docx"}else{"json"};let name=format!("document-{id}.{ext}");
  let bytes=crate::io::save_bytes_with(&name,doc,password.as_ref().map(crate::io::Password::as_str),&crate::io::ExportOptions::default())?;
  write_new(&directory.join(&name),&bytes)?;
  entries.push(serde_json::json!({"file":name,"original_path":path,"encrypted":password.is_some()}));
 }
 write_new(&directory.join("RECOVERY.json"),&serde_json::to_vec_pretty(&entries).map_err(|e|e.to_string())?)?;Ok(entries.len())
}
fn write_new(path:&Path,bytes:&[u8])->Result<(),String>{
 let mut options=std::fs::OpenOptions::new();options.write(true).create_new(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
 let mut file=options.open(path).map_err(|e|e.to_string())?;file.write_all(bytes).and_then(|()|file.sync_all()).map_err(|e|e.to_string())
}
