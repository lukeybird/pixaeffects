//! Native audio-session copy, including memory-only audio; does not retarget or clean the engine.
use crate::{Engine,io};
use std::path::Path;
pub fn save(engine:&Engine,directory:&Path)->Result<usize,String>{
 if !directory.is_dir(){return Err("Choose an existing recovery folder.".into())}
 if std::fs::read_dir(directory).map_err(|e|e.to_string())?.next().is_some(){return Err("Recovery requires a new empty folder.".into())}
 let file=directory.join("recovery.scraft");let path=file.to_str().ok_or("Choose a Unicode recovery path.")?;
 io::save_session_copy(engine,path).map_err(|e|e.to_string())
}
