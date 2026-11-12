//! Tiny original procedural video exercises the actual editor engine, without accounts or Mac tools.
use std::{error::Error, path::{Path, PathBuf}, sync::Arc};
use filmcraft_engine::Session;
use filmcraft_media::{DemoScene, Generator, MediaSource, SharedSource, generators::GeneratorSource};
use filmcraft_project::{ItemId, ItemKind, Label, MediaClip, MediaRef, Project, SequenceSettings, TrackKind};
use filmcraft_time::{FrameRate, Tick, TimeRange};
use serde_json::json;

fn directory() -> Result<PathBuf, Box<dyn Error>> {
    for n in 0..64 {
        let p = std::env::temp_dir().join(format!("pixa-film-workflow-{}-{n}", std::process::id()));
        match std::fs::create_dir(&p) {
            Ok(()) => return Ok(p),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {},
            Err(e) => return Err(e.into()),
        }
    }
    Err("Could not allocate synthetic-workflow directory".into())
}

fn fixture(path: &Path) -> Result<(), Box<dyn Error>> {
    let rate = FrameRate::FPS_24;
    let duration = rate.tick_of(4);
    let generator = Generator::Demo(DemoScene::Dunes);
    let source = GeneratorSource::new(generator.clone(), 64, 48, rate, duration);
    let mut project = Project::new("Pixa synthetic fixture");
    let media = MediaClip { media: MediaRef::Generator(generator), info: source.info().clone(), interpret: Default::default(), mark_in: None, mark_out: None, markers: vec![], offline: false, proxy: None, identity: None };
    let item = project.add_item("Original procedural scene", Label::Iris, ItemKind::Media(Box::new(media)), None);
    let sequence = project.new_sequence("Fixture", SequenceSettings { width:64, height:48, frame_rate:rate, ..Default::default() }, 1, 1, None);
    let range = TimeRange::new(Tick::ZERO, duration);
    let video = project.make_track_item(item, TrackKind::Video, Tick::ZERO, range, rate).ok_or("Could not place video")?;
    let audio = project.make_track_item(item, TrackKind::Audio, Tick::ZERO, range, rate).ok_or("Could not place audio")?;
    let track = project.sequence_mut(sequence).ok_or("No fixture sequence")?;
    track.video_tracks[0].items.push(video);
    track.audio_tracks[0].items.push(audio);
    let shared: SharedSource = Arc::new(source);
    let provider = move |id: ItemId| (id == item).then(|| shared.clone());
    let settings = filmcraft_export::ExportSettings { format:filmcraft_export::Format::H264, path:path.to_string_lossy().into_owned(), bitrate_kbps:500, ..Default::default() };
    filmcraft_export::export(&Arc::new(project), sequence, &settings, &provider, &Default::default())?;
    Ok(())
}

fn verify(dir: &Path) -> Result<(), Box<dyn Error>> {
    let movie = dir.join("original-procedural.mp4");
    fixture(&movie)?;
    let mut session = Session::default();
    let imported = session.execute("file.import", json!({"paths":[movie]}))?;
    let items = imported["items"].as_array().ok_or("No imported items")?;
    if items.len() != 1 { return Err("Expected one original movie".into()); }
    session.execute("file.newSequenceFromClip", json!({"items":items}))?;
    let before = session.project.clone();
    session.execute("sequence.addEdit", json!({"seconds":1.0/24.0}))?;
    let edited = session.project.clone();
    if *edited == *before { return Err("Timeline edit did not change project".into()); }
    session.execute("edit.undo", json!({}))?;
    if *session.project != *before { return Err("Undo did not restore project".into()); }
    session.execute("edit.redo", json!({}))?;
    if *session.project != *edited { return Err("Redo did not restore timeline edit".into()); }
    let project = dir.join("edited.fcproj");
    session.execute("file.saveAs", json!({"path":project}))?;
    let saved = session.project.clone();
    let mut reopened = Session::default();
    reopened.execute("file.open", json!({"path":project}))?;
    if *reopened.project != *saved { return Err("Saved project did not reopen unchanged".into()); }
    for (format, extension) in [("h264", "mp4"), ("wav", "wav")] {
        let output = dir.join(format!("edited-export.{extension}"));
        reopened.execute("file.exportMedia", json!({"path":output,"format":format,"hardwareEncoding":"off","gpuRendering":"off","wait":true}))?;
        let bytes = std::fs::read(&output)?;
        if bytes.len()<32 { return Err("Export is empty".into()); }
        if extension=="wav" && !bytes.starts_with(b"RIFF") { return Err("WAV signature missing".into()); }
        if extension=="mp4" {
            if !bytes.iter().take(64).copied().collect::<Vec<_>>().windows(4).any(|w|w==b"ftyp") { return Err("MP4 signature missing".into()); }
            let mut roundtrip = Session::default();
            let result = roundtrip.execute("file.import", json!({"paths":[output]}))?;
            if result["items"].as_array().map(Vec::len)!=Some(1) { return Err("Exported movie cannot be imported".into()); }
        }
    }
    println!("PixaFilm: original video import, timeline edit, undo/redo, native save/reopen, CPU H.264 and WAV export and movie re-import passed");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let dir = directory()?;
    let result = verify(&dir);
    let cleanup = std::fs::remove_dir_all(&dir);
    result?;
    cleanup?;
    Ok(())
}
