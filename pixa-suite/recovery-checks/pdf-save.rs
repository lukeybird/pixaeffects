//! Uses the native PDF save pipeline, including encryption and signature-preserving behavior.
use crate::Session;
use std::io::Write;
use std::path::Path;
pub fn save(session:&Session,directory:&Path)->Result<usize,String>{
 if !directory.is_dir(){return Err("Choose an existing recovery folder.".into())}
 let mut entries=Vec::new();
 for (index,doc) in session.docs().iter().enumerate(){
  let name=format!("document-{index}.pdf");
  let bytes=session.save_bytes(doc.id).map_err(|e|e.to_string())?;
  session.check_save(doc.id,&bytes).map_err(|e|e.to_string())?;
  write_new(&directory.join(&name),&bytes)?;
  entries.push(format!("{name}: {:?} ({:?})",doc.name,doc.path));
 }
 write_new(&directory.join("RECOVERY.txt"),entries.join("\n").as_bytes())?;Ok(entries.len())
}
fn write_new(path:&Path,bytes:&[u8])->Result<(),String>{
 let mut options=std::fs::OpenOptions::new();options.write(true).create_new(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
 let mut file=options.open(path).map_err(|e|e.to_string())?;file.write_all(bytes).and_then(|()|file.sync_all()).map_err(|e|e.to_string())
}
