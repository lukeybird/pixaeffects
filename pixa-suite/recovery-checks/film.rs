//! Synthetic native recovery round trip. Does not read user documents or use the account service.
use filmcraft_engine::Session;
use serde_json::json;
use std::path::PathBuf;
fn directory()->Result<PathBuf,Box<dyn std::error::Error>> {
    for n in 0..64 {
        let path=std::env::temp_dir().join(format!("pixa-film-recovery-check-{}-{n}",std::process::id()));
        match std::fs::create_dir(&path) { Ok(())=>return Ok(path),Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(e.into()) }
    }
    Err("No free fixture directory".into())
}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let dir=directory()?;let original=dir.join("original.fcproj");let copy=dir.join("recovery.fcproj");
    let mut session=Session::default();session.execute("file.openDemoProject",json!({}))?;
    session.execute("file.saveAs",json!({"path":original}))?;let original_bytes=std::fs::read(&original)?;
    session.execute("sequence.addEdit",json!({"seconds":1.0}))?;
    let before=session.project.clone();let path_before=session.path.clone();
    if !session.is_dirty(){return Err("Fixture edit did not mark the project unsaved".into());}
    session.execute("file.saveCopy",json!({"path":copy}))?;
    if !session.is_dirty() || session.path!=path_before{return Err("Recovery copy changed the original path or unsaved state".into());}
    if std::fs::read(&original)?!=original_bytes{return Err("Recovery copy overwrote the original".into());}
    let mut restored=Session::default();restored.execute("file.open",json!({"path":copy}))?;
    if *restored.project!=*before{return Err("Recovery project did not round trip".into());}
    std::fs::remove_dir_all(&dir)?;
    println!("PixaFilm recovery round-trip passed; original and unsaved state preserved.");Ok(())
}
