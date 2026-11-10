//! Synthetic native recovery round trip. Does not read user documents or use the account service.
use texturecraft_engine::{Session,execute};
use serde_json::json;
use std::path::PathBuf;
fn directory()->Result<PathBuf,Box<dyn std::error::Error>> {
    for n in 0..64 {
        let path=std::env::temp_dir().join(format!("pixa-texture-recovery-check-{}-{n}",std::process::id()));
        match std::fs::create_dir(&path) { Ok(())=>return Ok(path),Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(e.into()) }
    }
    Err("No free fixture directory".into())
}
fn session()->Result<Session,Box<dyn std::error::Error>> {
    let mesh=texturecraft_geom::primitives::builtin("cube").ok_or("No cube fixture")?;
    Ok(Session::new_project("Recovery fixture",mesh,"builtin:cube",None,128)?)
}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let dir=directory()?;let original=dir.join("original.tcraft");let copy=dir.join("recovery.tcraft");
    let mut project=session()?;execute(&mut project,"file.saveAs",&json!({"path":original}))?;
    let original_bytes=std::fs::read(&original)?;let path_before=project.path.clone();
    project.doc.project.name="Unsaved recovery fixture".into();project.modified=true;
    project.end_stroke()?;let before=serde_json::to_value(&project.doc.project)?;
    execute(&mut project,"file.saveCopy",&json!({"path":copy}))?;
    if !project.modified || project.path!=path_before{return Err("Recovery copy changed the original path or unsaved state".into());}
    if std::fs::read(&original)?!=original_bytes{return Err("Recovery copy overwrote the original".into());}
    let mut restored=session()?;execute(&mut restored,"file.open",&json!({"path":copy}))?;
    if serde_json::to_value(&restored.doc.project)?!=before{return Err("Recovery project did not round trip".into());}
    std::fs::remove_dir_all(&dir)?;
    println!("PixaTexture recovery round-trip passed; original and unsaved state preserved.");Ok(())
}
