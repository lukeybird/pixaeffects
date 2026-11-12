//! Synthetic PixaVector core-workflow acceptance, independent of account/network/Mac signing.
use serde_json::json;
use vectorcraft_engine::Session;
use std::{error::Error, path::Path};

fn temporary_directory() -> Result<std::path::PathBuf, Box<dyn Error>> {
    for n in 0..64 {
        let path = std::env::temp_dir().join(format!("pixa-vector-workflow-{}-{n}", std::process::id()));
        match std::fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {},
            Err(error) => return Err(error.into()),
        }
    }
    Err("Could not allocate a private synthetic-workflow directory".into())
}

fn verify(directory: &Path) -> Result<(), Box<dyn Error>> {
    let mut session = Session::new();
    session.execute("file.new", &json!({"width":128,"height":96}))?;
    let empty = serde_json::to_value(&session.doc()?.doc)?;
    session.execute("shape.rectangle", &json!({"x":10,"y":12,"width":40,"height":24}))?;
    let edited = serde_json::to_value(&session.doc()?.doc)?;
    if edited == empty { return Err("Drawing did not change the document".into()); }
    session.execute("edit.undo", &json!({}))?;
    if serde_json::to_value(&session.doc()?.doc)? != empty { return Err("Undo did not restore the document".into()); }
    session.execute("edit.redo", &json!({}))?;
    if serde_json::to_value(&session.doc()?.doc)? != edited { return Err("Redo did not restore the edit".into()); }
    let project = directory.join("synthetic.vectorcraft");
    session.execute("document.save", &json!({"path":project}))?;
    let mut reopened = Session::new();
    reopened.execute("document.open", &json!({"path":project}))?;
    // Saving intentionally records a modification timestamp. Compare the complete
    // reopened document to the post-save state, and require all other content unchanged.
    let saved = serde_json::to_value(&session.doc()?.doc)?;
    let mut expected = edited.clone();
    expected["metadata"]["modified"] = saved["metadata"]["modified"].clone();
    if saved != expected { return Err("Save changed document content beyond its timestamp".into()); }
    if !saved["metadata"]["modified"].is_number() { return Err("Save did not record its timestamp".into()); }
    if serde_json::to_value(&reopened.doc()?.doc)? != saved { return Err("Saved and reopened document differs".into()); }
    let svg = directory.join("synthetic.svg");
    reopened.execute("document.export", &json!({"format":"svg","path":svg}))?;
    if !std::fs::read_to_string(&svg)?.contains("<svg") { return Err("SVG export is invalid".into()); }
    let pdf = directory.join("synthetic.pdf");
    reopened.execute("document.export", &json!({"format":"pdf","path":pdf}))?;
    if !std::fs::read(pdf)?.starts_with(b"%PDF-") { return Err("PDF export is invalid".into()); }
    let png = directory.join("synthetic.png");
    reopened.execute("document.export", &json!({"format":"png","path":png}))?;
    let bytes = std::fs::read(png)?;
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") || bytes.len() < 32 { return Err("PNG export is invalid".into()); }
    let import = directory.join("original-fixture.svg");
    std::fs::write(&import, r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect x="8" y="8" width="32" height="32" fill="#ff6600"/></svg>"##)?;
    let mut imported = Session::new();
    imported.execute("document.open", &json!({"path":import}))?;
    imported.execute("document.save", &json!({"path":directory.join("imported.vectorcraft")}))?;
    let returned = directory.join("imported.svg");
    imported.execute("document.export", &json!({"format":"svg","path":returned}))?;
    let returned = std::fs::read_to_string(returned)?;
    if !returned.contains("<svg") || returned.len() < 100 { return Err("Imported SVG could not be exported".into()); }
    for command in ["help.github", "help.appPage"] {
        let answer = session.execute(command, &json!({}))?;
        if answer.get("url").and_then(serde_json::Value::as_str) != Some("https://aiwebdesignfirm.com/products/pixavector") {
            return Err("Legacy help command did not resolve to the Pixa product".into());
        }
    }
    println!("PixaVector: drawing, undo/redo, native save/reopen, SVG import and SVG/PDF/PNG export passed");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let directory = temporary_directory()?;
    let result = verify(&directory);
    // Only this process's synthetic files are removed; no user files are read or modified.
    let cleanup = std::fs::remove_dir_all(&directory);
    result?;
    cleanup?;
    Ok(())
}
