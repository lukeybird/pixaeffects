//! An independent catalogue snapshot must include in-memory edits absent from the source log.
use lightcraft_engine::{Session,pixa_recovery};
use lightcraft_catalog::Op;
use serde_json::json;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let mut folder=None;for n in 0..64{let d=std::env::temp_dir().join(format!("pixa-light-recovery-{}-{n}",std::process::id()));match std::fs::create_dir(&d){Ok(())=>{folder=Some(d);break},Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(e.into())}}
 let dir=folder.ok_or("No free fixture folder")?;let original=dir.join("original");let copies=dir.join("copies");std::fs::create_dir(&copies)?;
 let mut session=Session::new();session.open_library(&original,true)?;session.flush_log()?;
 let before_log=std::fs::read(original.join("catalog.log")).unwrap_or_default();let before_snapshot=std::fs::read(original.join("catalog.snap")).unwrap_or_default();
 let id=session.catalog.photos().next().ok_or("Demo library has no photos")?.id;
 // Deliberately change the live catalogue without writing the original journal.
 session.execute("library.select",&json!({"ids":[id.0]}))?;session.flush_log()?;
 // Establish the source bytes after any explicitly requested selection persistence.
 let source_log=std::fs::read(original.join("catalog.log")).unwrap_or_default();let source_snapshot=std::fs::read(original.join("catalog.snap")).unwrap_or_default();
 session.catalog.apply(Op::SetRating{id,rating:5})?;session.cache_mb=1234;
 let expected=serde_json::to_value(&session.catalog)?;
 pixa_recovery::save(&session,&copies)?;
 if pixa_recovery::save(&session,&copies).is_ok(){return Err("Recovery overwrote an existing library".into())}
 if std::fs::read(original.join("catalog.log")).unwrap_or_default()!=source_log || std::fs::read(original.join("catalog.snap")).unwrap_or_default()!=source_snapshot{return Err("Recovery changed original journal".into())}
 if serde_json::to_value(&session.catalog)?!=expected{return Err("Recovery changed live catalogue".into())}
 let mut reopened=Session::new();reopened.open_library(copies.join("library"),false)?;
 if serde_json::to_value(&reopened.catalog)?!=expected || reopened.cache_mb!=1234{return Err("Recovery lost catalogue edits or preferences".into())}
 let _=(before_log,before_snapshot);drop(reopened);drop(session);let _=std::fs::remove_dir_all(dir);
 println!("PixaLight live catalogue and preferences recovery passed");Ok(())
}
