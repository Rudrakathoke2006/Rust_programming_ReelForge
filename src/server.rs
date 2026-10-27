use crate::analytics::AnalyticsTracker;
use crate::pipeline;
use crate::types::{Config, Engine, EventTheme};
use axum::{
    extract::{DefaultBodyLimit, Multipart, Query},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

pub async fn serve(port: u16) -> anyhow::Result<()> {
    let analytics_path = PathBuf::from("reelforge_data/analytics.json");
    let tracker = AnalyticsTracker::new(analytics_path);

    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("../web/index.html")) }))
        .route("/admin", get(|| async { Html(include_str!("../web/admin.html")) }))
        .route("/analytics", get(|| async { Html(include_str!("../web/admin.html")) }))
        .route("/api/analytics", get({
            let t = tracker.clone();
            move || {
                let t = t.clone();
                async move { Json(t.get_summary()) }
            }
        }))
        .route("/api/templates", get(get_templates))
        .route("/api/recommend-templates", post(recommend_templates))
        .route("/generate", post({
            let t = tracker.clone();
            move |mp| generate(mp, t)
        }))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    println!("ReelForge Studio UI → http://localhost:{port}");
    println!("ReelForge Analytics & Admin Hub → http://localhost:{port}/admin");
    axum::serve(listener, app).await?;
    Ok(())
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

#[derive(Deserialize)]
pub struct TemplateQuery {
    pub event: Option<String>,
    pub title: Option<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct TemplatesResponse {
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub templates: Vec<crate::templates::ScoredTemplate>,
}

async fn get_templates(Query(q): Query<TemplateQuery>) -> Json<TemplatesResponse> {
    let event = q.event.as_deref().map(EventTheme::from_str).unwrap_or(EventTheme::General);
    let title = q.title.as_deref().unwrap_or("");
    let offset = q.offset.unwrap_or(0);
    let limit = q.limit.unwrap_or(3).clamp(1, 48);

    let scored = crate::templates::score_templates(event, &[], title);
    let total = scored.len();
    let offset_clamped = if total == 0 { 0 } else { offset % total };
    let paged = scored.into_iter().skip(offset_clamped).take(limit).collect();

    Json(TemplatesResponse {
        total,
        offset: offset_clamped,
        limit,
        templates: paged,
    })
}

async fn recommend_templates(mut mp: Multipart) -> Result<Json<TemplatesResponse>, (StatusCode, String)> {
    let job = std::env::temp_dir().join(format!("reelforge-rec-{}", uuid::Uuid::new_v4()));
    let upload = job.join("uploads");
    std::fs::create_dir_all(&upload).map_err(err)?;

    let mut event = EventTheme::General;
    let mut title = String::new();
    let mut offset = 0usize;
    let mut limit = 3usize;
    let mut photos: Vec<PathBuf> = Vec::new();

    while let Some(field) = mp.next_field().await.map_err(err)? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "event" => {
                let txt = field.text().await.unwrap_or_default();
                event = EventTheme::from_str(&txt);
            }
            "title" => {
                title = field.text().await.unwrap_or_default();
            }
            "offset" => {
                if let Ok(v) = field.text().await.unwrap_or_default().trim().parse::<usize>() {
                    offset = v;
                }
            }
            "limit" => {
                if let Ok(v) = field.text().await.unwrap_or_default().trim().parse::<usize>() {
                    limit = v.clamp(1, 48);
                }
            }
            "photos" => {
                let fname = field.file_name().unwrap_or("photo.jpg").to_string();
                let data = field.bytes().await.map_err(err)?;
                if !data.is_empty() && photos.len() < 12 {
                    let p = upload.join(format!("{:04}_{}", photos.len(), sanitize(&fname)));
                    if std::fs::write(&p, &data).is_ok() {
                        photos.push(p);
                    }
                }
            }
            _ => {}
        }
    }

    let mut analyses = Vec::new();
    for p in &photos {
        if let Ok(a) = crate::analyze::analyze(p) {
            analyses.push(a);
        }
    }

    let _ = std::fs::remove_dir_all(&job);

    let scored = crate::templates::score_templates(event, &analyses, &title);
    let total = scored.len();
    let offset_clamped = if total == 0 { 0 } else { offset % total };
    let paged = scored.into_iter().skip(offset_clamped).take(limit).collect();

    Ok(Json(TemplatesResponse {
        total,
        offset: offset_clamped,
        limit,
        templates: paged,
    }))
}

async fn generate(mut mp: Multipart, tracker: Arc<AnalyticsTracker>) -> Result<Response, (StatusCode, String)> {
    let job = std::env::temp_dir().join(format!("reelforge-{}", uuid::Uuid::new_v4()));
    let upload = job.join("uploads");
    std::fs::create_dir_all(&upload).map_err(err)?;

    let mut user_id = "Guest Creator".to_string();
    let mut photos: Vec<PathBuf> = Vec::new();
    let mut music: Option<PathBuf> = None;
    let (mut bpm, mut shot_secs, mut engine) = (None::<f32>, 2.4f32, Engine::Ffmpeg);
    let mut event = crate::types::EventTheme::General;
    let mut motion = crate::types::MotionMode::DynamicMix;
    let mut title: Option<String> = None;
    let mut stickers = true;
    let mut template_id: Option<String> = None;

    while let Some(field) = mp.next_field().await.map_err(err)? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "user" | "user_id" | "email" => {
                let txt = field.text().await.unwrap_or_default().trim().to_string();
                if !txt.is_empty() {
                    user_id = txt;
                }
            }
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
            "template" | "template_id" => {
                let txt = field.text().await.unwrap_or_default().trim().to_string();
                if !txt.is_empty() {
                    template_id = Some(txt);
                }
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
    let photos_count = photos.len();
    let approx_duration = (photos_count as f32 * shot * 0.85).max(1.0);

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
        title: title.clone(),
        stickers,
        template_id,
    };
    tokio::task::spawn_blocking(move || pipeline::run(&cfg))
        .await
        .map_err(err)?
        .map_err(|e| err(format!("{e:#}")))?;

    // Record user activity in persistent analytics store
    let event_str = match event {
        crate::types::EventTheme::Birthday => "birthday",
        crate::types::EventTheme::Anniversary => "anniversary",
        crate::types::EventTheme::Romantic => "romantic",
        crate::types::EventTheme::Party => "party",
        crate::types::EventTheme::Travel => "travel",
        crate::types::EventTheme::General => "general",
    };
    let motion_str = match motion {
        crate::types::MotionMode::DynamicMix => "dynamic-mix",
        crate::types::MotionMode::Boomerang => "boomerang",
        crate::types::MotionMode::FastForward => "fast-forward",
        crate::types::MotionMode::KenBurns => "ken-burns",
    };
    tracker.record_reel(
        &user_id,
        event_str,
        motion_str,
        photos_count,
        approx_duration,
        title.as_deref(),
    );

    let bytes = std::fs::read(&out).map_err(err)?;
    let _ = std::fs::remove_dir_all(&job);
    Ok(([(header::CONTENT_TYPE, "video/mp4"), (header::CONTENT_DISPOSITION, "inline; filename=\"reel.mp4\"")], bytes).into_response())
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' }).collect()
}
