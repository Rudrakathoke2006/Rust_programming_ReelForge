use crate::pipeline;
use crate::types::{Config, Engine};
use axum::{
    extract::{DefaultBodyLimit, Multipart},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::path::PathBuf;

pub async fn serve(port: u16) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("../web/index.html")) }))
        .route("/generate", post(generate))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024));
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    println!("ReelForge UI → http://localhost:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

async fn generate(mut mp: Multipart) -> Result<Response, (StatusCode, String)> {
    let job = std::env::temp_dir().join(format!("reelforge-{}", uuid::Uuid::new_v4()));
    let upload = job.join("uploads");
    std::fs::create_dir_all(&upload).map_err(err)?;

    let mut photos: Vec<PathBuf> = Vec::new();
    let mut music: Option<PathBuf> = None;
    let (mut bpm, mut shot_secs, mut engine) = (None::<f32>, 2.4f32, Engine::Ffmpeg);
    let mut event = crate::types::EventTheme::General;
    let mut motion = crate::types::MotionMode::DynamicMix;
    let mut title: Option<String> = None;
    let mut stickers = true;

    while let Some(field) = mp.next_field().await.map_err(err)? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "photos" => {
                let fname = field.file_name().unwrap_or("photo.jpg").to_string();
                let data = field.bytes().await.map_err(err)?;
                if data.is_empty() { continue; }
                let p = upload.join(format!("{:04}_{}", photos.len(), sanitize(&fname)));
                std::fs::write(&p, &data).map_err(err)?;
                photos.push(p);
            }
            "music" => {
                let fname = field.file_name().unwrap_or("music.mp3").to_string();
                let data = field.bytes().await.map_err(err)?;
                if data.is_empty() { continue; }
                let p = job.join(sanitize(&fname));
                std::fs::write(&p, &data).map_err(err)?;
                music = Some(p);
            }
            "bpm" => bpm = field.text().await.ok().and_then(|t| t.trim().parse().ok()),
            "shot_secs" => {
                if let Some(v) = field.text().await.ok().and_then(|t| t.trim().parse::<f32>().ok()) {
                    shot_secs = v.clamp(0.8, 8.0);
                }
            }
            "engine" => {
                engine = if field.text().await.unwrap_or_default() == "melt" { Engine::Melt } else { Engine::Ffmpeg };
            }
            "event" => {
                let txt = field.text().await.unwrap_or_default();
                event = crate::types::EventTheme::from_str(&txt);
            }
            "motion" => {
                let txt = field.text().await.unwrap_or_default();
                motion = crate::types::MotionMode::from_str(&txt);
            }
            "title" => {
                let txt = field.text().await.unwrap_or_default().trim().to_string();
                if !txt.is_empty() {
                    title = Some(txt);
                }
            }
            "stickers" => {
                let txt = field.text().await.unwrap_or_default().to_lowercase();
                stickers = txt != "false" && txt != "0";
            }
            _ => {}
        }
    }
    if photos.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "no photos uploaded".into()));
    }
    photos.sort();
    let shot = bpm.map(pipeline::shot_secs_from_bpm).unwrap_or(shot_secs);
    let out = job.join("reel.mp4");
    let cfg = Config {
        inputs: photos,
        output: out.clone(),
        work_dir: job.join("work"),
        width: 1080,
        height: 1920,
        fps: 30,
        shot_secs: shot,
        trans_secs: (shot * 0.2).min(0.6),
        music,
        engine,
        event,
        motion,
        title,
        stickers,
    };
    tokio::task::spawn_blocking(move || pipeline::run(&cfg))
        .await
        .map_err(err)?
        .map_err(|e| err(format!("{e:#}")))?;

    let bytes = std::fs::read(&out).map_err(err)?;
    let _ = std::fs::remove_dir_all(&job);
    Ok(([(header::CONTENT_TYPE, "video/mp4"), (header::CONTENT_DISPOSITION, "inline; filename=\"reel.mp4\"")], bytes).into_response())
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' }).collect()
}
