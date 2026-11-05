use crate::analytics::AnalyticsTracker;
use crate::pipeline;
use crate::types::{BannerStyle, Config, Engine, EventTheme, MotionMode};
use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query},
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

    // Initialize royalty-free built-in music catalog across all eras
    let music_dir = PathBuf::from("reelforge_data/music");
    let _ = crate::audio_catalog::ensure_library(&music_dir);

    let app = Router::new()
        .route("/", get(|| async {
            let html = std::fs::read_to_string("web/index.html").unwrap_or_else(|_| include_str!("../web/index.html").to_string());
            Html(html)
        }))
        .route("/login", get(|| async {
            let html = std::fs::read_to_string("web/login.html").unwrap_or_else(|_| include_str!("../web/login.html").to_string());
            Html(html)
        }))
        .route("/signin", get(|| async {
            let html = std::fs::read_to_string("web/login.html").unwrap_or_else(|_| include_str!("../web/login.html").to_string());
            Html(html)
        }))
        .route("/admin", get(|| async {
            let html = std::fs::read_to_string("web/admin.html").unwrap_or_else(|_| include_str!("../web/admin.html").to_string());
            Html(html)
        }))
        .route("/analytics", get(|| async {
            let html = std::fs::read_to_string("web/admin.html").unwrap_or_else(|_| include_str!("../web/admin.html").to_string());
            Html(html)
        }))
        .route("/api/analytics", get({
            let t = tracker.clone();
            move || {
                let t = t.clone();
                async move { Json(t.get_summary()) }
            }
        }))
        .route("/api/music/list", get(get_music_list))
        .route("/api/music/suggest", get(get_music_suggest))
        .route("/api/music/stream/:filename", get(serve_music_stream))
        .route("/api/music/upload", post(upload_original_music))
        .route("/api/templates", get(get_templates))
        .route("/api/recommend-templates", post(recommend_templates))
        .route("/generate", post({
            let t = tracker.clone();
            move |mp| generate(mp, t)
        }))
        .route("/api/generate-variants", post({
            let t = tracker.clone();
            move |mp| generate_variants(mp, t)
        }))
        .route("/api/customize-reel", post(customize_reel))
        .route("/api/video/:job_id/:filename", get(serve_video))
        .route("/api/job/:job_id", get(get_job_variants))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    println!("ReelForge Sign-In Portal → http://localhost:{port}/login");
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

#[derive(Deserialize)]
pub struct MusicQuery {
    pub era: Option<String>,
    pub genre: Option<String>,
    pub artist: Option<String>,
    pub singer: Option<String>,
    pub event: Option<String>,
}

async fn get_music_list(Query(q): Query<MusicQuery>) -> Json<Vec<crate::audio_catalog::MusicTrack>> {
    let ev = q.event.as_deref().map(EventTheme::from_str);
    let artist_filter = q.artist.as_deref().or(q.singer.as_deref());
    let tracks = crate::audio_catalog::list_tracks(q.era.as_deref(), q.genre.as_deref(), artist_filter, ev);
    Json(tracks)
}

#[derive(Deserialize)]
pub struct MusicSuggestQuery {
    pub event: Option<String>,
    pub photos_count: Option<usize>,
    pub motion: Option<String>,
    pub era: Option<String>,
}

async fn get_music_suggest(Query(q): Query<MusicSuggestQuery>) -> Json<crate::audio_catalog::MusicTrack> {
    let ev = q.event.as_deref().map(EventTheme::from_str).unwrap_or(EventTheme::General);
    let motion = q.motion.as_deref().map(MotionMode::from_str).unwrap_or(MotionMode::DynamicMix);
    let count = q.photos_count.unwrap_or(6);
    let track = crate::audio_catalog::suggest_best_track(ev, count, motion, q.era.as_deref());
    Json(track)
}

async fn serve_music_stream(Path(filename): Path<String>) -> Result<Response, (StatusCode, String)> {
    let clean = sanitize(&filename);
    let path = PathBuf::from("reelforge_data/music").join(&clean);
    if !path.exists() {
        return Err((StatusCode::NOT_FOUND, "Music track file not found".into()));
    }
    let bytes = std::fs::read(&path).map_err(err)?;
    let content_type = if clean.ends_with(".mp3") { "audio/mpeg" } else { "audio/wav" };
    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_DISPOSITION, "inline"),
            (header::ACCEPT_RANGES, "bytes"),
        ],
        bytes,
    ).into_response())
}

#[derive(Serialize)]
pub struct UploadMusicResponse {
    pub success: bool,
    pub track_id: String,
    pub filename: String,
    pub stream_url: String,
    pub title: String,
    pub artist: String,
    pub duration: f32,
    pub bpm: u32,
}

async fn upload_original_music(mut mp: Multipart) -> Result<Json<UploadMusicResponse>, (StatusCode, String)> {
    let music_dir = PathBuf::from("reelforge_data/music");
    std::fs::create_dir_all(&music_dir).map_err(err)?;

    let mut saved_filename = String::new();
    let mut original_title = "Original Audio Track".to_string();

    while let Some(field) = mp.next_field().await.map_err(err)? {
        let name = field.name().unwrap_or("").to_string();
        if name == "audio" || name == "file" || name == "music" {
            let fname = field.file_name().unwrap_or("custom_song.mp3").to_string();
            let clean = sanitize(&fname);
            let ext = clean.rsplit('.').next().unwrap_or("mp3");
            let stem = clean.strip_suffix(&format!(".{ext}")).unwrap_or(&clean);
            original_title = stem.replace('_', " ").replace('-', " ");
            let id = format!("orig_{}_{}", &uuid::Uuid::new_v4().to_string()[..8], clean);
            let target = music_dir.join(&id);
            let data = field.bytes().await.map_err(err)?;
            if data.is_empty() {
                return Err((StatusCode::BAD_REQUEST, "Uploaded audio file was empty".into()));
            }
            std::fs::write(&target, &data).map_err(err)?;
            saved_filename = id;
            break;
        }
    }

    if saved_filename.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No audio file field received".into()));
    }

    let track_id = saved_filename.strip_suffix(".mp3").or_else(|| saved_filename.strip_suffix(".wav")).unwrap_or(&saved_filename).to_string();
    let stream_url = format!("/api/music/stream/{}", saved_filename);

    Ok(Json(UploadMusicResponse {
        success: true,
        track_id,
        filename: saved_filename,
        stream_url,
        title: original_title,
        artist: "Uploaded Original Audio".to_string(),
        duration: 30.0,
        bpm: 120,
    }))
}

async fn generate(mut mp: Multipart, tracker: Arc<AnalyticsTracker>) -> Result<Response, (StatusCode, String)> {
    let job = std::env::temp_dir().join(format!("reelforge-{}", uuid::Uuid::new_v4()));
    let upload = job.join("uploads");
    std::fs::create_dir_all(&upload).map_err(err)?;

    let mut user_id = "Guest Creator".to_string();
    let mut photos: Vec<PathBuf> = Vec::new();
    let mut music: Option<PathBuf> = None;
    let mut music_id: Option<String> = None;
    let (mut bpm, mut shot_secs, mut engine) = (None::<f32>, 2.4f32, Engine::Ffmpeg);
    let mut event = crate::types::EventTheme::General;
    let mut motion = crate::types::MotionMode::DynamicMix;
    let mut banner_style = BannerStyle::TopPill;
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
            "music_id" | "music_track" => {
                let txt = field.text().await.unwrap_or_default().trim().to_string();
                if !txt.is_empty() {
                    music_id = Some(txt);
                }
            }
            "banner" | "banner_style" => {
                let txt = field.text().await.unwrap_or_default();
                banner_style = BannerStyle::from_str(&txt);
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

    // Only resolve music if explicitly provided by user or uploaded
    if music.is_none() {
        if let Some(ref mid) = music_id {
            if !mid.trim().is_empty() {
                music = crate::audio_catalog::find_track_path(&PathBuf::from("reelforge_data/music"), mid);
            }
        }
    }

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
        banner_style,
        title: title.clone(),
        stickers,
        template_id,
    };
    tokio::task::spawn_blocking(move || pipeline::run(&cfg))
        .await
        .map_err(err)?
        .map_err(|e| err(format!("{e:#}")))?;

    tracker.record_reel(
        &user_id,
        event.as_str(),
        motion.as_str(),
        photos_count,
        approx_duration,
        title.as_deref(),
    );

    let bytes = std::fs::read(&out).map_err(err)?;
    let _ = std::fs::remove_dir_all(&job);
    Ok(([(header::CONTENT_TYPE, "video/mp4"), (header::CONTENT_DISPOSITION, "inline; filename=\"reel.mp4\"")], bytes).into_response())
}

#[derive(Serialize)]
pub struct VariantItem {
    pub id: String,
    pub filename: String,
    pub title: String,
    pub tag: String,
    pub description: String,
    pub duration: f32,
    pub video_url: String,
}

#[derive(Serialize)]
pub struct GenerateVariantsResponse {
    pub success: bool,
    pub job_id: String,
    pub variants: Vec<VariantItem>,
}

#[derive(Deserialize)]
pub struct CustomizeRequest {
    pub job_id: String,
    pub video_file: String,
    pub start_sec: f32,
    pub end_sec: f32,
    pub speed: Option<f32>,
    pub mute_audio: Option<bool>,
    pub volume: Option<f32>,
    pub filter: Option<String>,
}

#[derive(Serialize)]
pub struct CustomizeResponse {
    pub success: bool,
    pub video_url: String,
    pub filename: String,
    pub duration: f32,
    pub message: String,
}

async fn generate_variants(mut mp: Multipart, tracker: Arc<AnalyticsTracker>) -> Result<Json<GenerateVariantsResponse>, (StatusCode, String)> {
    let job_id = format!("job-{}", &uuid::Uuid::new_v4().to_string()[..12]);
    let job_dir = PathBuf::from("reelforge_data/jobs").join(&job_id);
    let upload = job_dir.join("uploads");
    std::fs::create_dir_all(&upload).map_err(err)?;

    let mut user_id = "Guest Creator".to_string();
    let mut photos: Vec<PathBuf> = Vec::new();
    let mut music: Option<PathBuf> = None;
    let mut music_id: Option<String> = None;
    let mut user_banner: Option<BannerStyle> = None;
    let mut event = EventTheme::General;
    let mut motion = MotionMode::DynamicMix;
    let mut title: Option<String> = None;
    let mut _batch = 1usize;

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
                let p = job_dir.join(sanitize(&fname));
                std::fs::write(&p, &data).map_err(err)?;
                music = Some(p);
            }
            "music_id" | "music_track" => {
                let txt = field.text().await.unwrap_or_default().trim().to_string();
                if !txt.is_empty() {
                    music_id = Some(txt);
                }
            }
            "banner" | "banner_style" => {
                let txt = field.text().await.unwrap_or_default().trim().to_string();
                if !txt.is_empty() {
                    user_banner = Some(BannerStyle::from_str(&txt));
                }
            }
            "event" => {
                let txt = field.text().await.unwrap_or_default();
                event = EventTheme::from_str(&txt);
            }
            "motion" => {
                let txt = field.text().await.unwrap_or_default();
                motion = MotionMode::from_str(&txt);
            }
            "title" => {
                let txt = field.text().await.unwrap_or_default().trim().to_string();
                if !txt.is_empty() {
                    title = Some(txt);
                }
            }
            "batch" => {
                if let Ok(v) = field.text().await.unwrap_or_default().trim().parse::<usize>() {
                    _batch = v.max(1);
                }
            }
            _ => {}
        }
    }

    if photos.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No photos uploaded".into()));
    }
    photos.sort();
    let photos_count = photos.len();

    // Only resolve music if explicitly provided by user or uploaded
    if music.is_none() {
        if let Some(ref mid) = music_id {
            if !mid.trim().is_empty() {
                music = crate::audio_catalog::find_track_path(&PathBuf::from("reelforge_data/music"), mid);
            }
        }
    }

    // Analyze photos to evaluate aspect ratios, focus sharpness, and composition for AI template matching
    let mut photo_analyses = Vec::new();
    for p in &photos {
        if let Ok(a) = crate::analyze::analyze(p) {
            photo_analyses.push(a);
        }
    }

    // Auto-match the top 3 best templates for this event, photos, and title
    let scored = crate::templates::score_templates(event, &photo_analyses, title.as_deref().unwrap_or(""));
    let t1 = scored.get(0).map(|s| &s.template).unwrap();
    let t2 = scored.get(1).map(|s| &s.template).unwrap_or(t1);
    let t3 = scored.get(2).map(|s| &s.template).unwrap_or(t2);

    println!("  ✨ Top 3 AI Auto-Matched Templates for {}:", event.as_str());
    println!("     Variant 1: '{}' [{:?}] ({})", t1.name, t1.motion, t1.description);
    println!("     Variant 2: '{}' [{:?}] ({})", t2.name, t2.motion, t2.description);
    println!("     Variant 3: '{}' [{:?}] ({})", t3.name, t3.motion, t3.description);

    let v1_title = format!("✨ {}", t1.name);
    let v1_tag = format!("{} • {:?}", t1.event.as_str(), t1.motion);
    let v1_desc = t1.description.to_string();
    let v1_motion = t1.motion;
    let v1_banner = user_banner.unwrap_or(BannerStyle::CenterHero);
    let v1_shot = t1.default_shot_secs;

    let v2_title = format!("🎬 {}", t2.name);
    let v2_tag = format!("{} • {:?}", t2.event.as_str(), t2.motion);
    let v2_desc = t2.description.to_string();
    let v2_motion = t2.motion;
    let v2_banner = user_banner.unwrap_or(BannerStyle::TopPill);
    let v2_shot = t2.default_shot_secs;

    let v3_title = format!("🌟 {}", t3.name);
    let v3_tag = format!("{} • {:?}", t3.event.as_str(), t3.motion);
    let v3_desc = t3.description.to_string();
    let v3_motion = t3.motion;
    let v3_banner = user_banner.unwrap_or(BannerStyle::CircularStamp);
    let v3_shot = t3.default_shot_secs;

    let out1 = job_dir.join("variant_1.mp4");
    let out2 = job_dir.join("variant_2.mp4");
    let out3 = job_dir.join("variant_3.mp4");

    let cfg1 = Config {
        inputs: photos.clone(),
        output: out1.clone(),
        work_dir: job_dir.join("work_v1"),
        width: 1080,
        height: 1920,
        fps: 30,
        shot_secs: v1_shot,
        trans_secs: (v1_shot * 0.2).min(0.5),
        music: music.clone(),
        engine: Engine::Ffmpeg,
        event,
        motion: v1_motion,
        banner_style: v1_banner,
        title: title.clone(),
        stickers: true,
        template_id: Some(t1.id.to_string()),
    };

    let cfg2 = Config {
        inputs: photos.clone(),
        output: out2.clone(),
        work_dir: job_dir.join("work_v2"),
        width: 1080,
        height: 1920,
        fps: 30,
        shot_secs: v2_shot,
        trans_secs: (v2_shot * 0.2).min(0.6),
        music: music.clone(),
        engine: Engine::Ffmpeg,
        event,
        motion: v2_motion,
        banner_style: v2_banner,
        title: title.clone(),
        stickers: true,
        template_id: Some(t2.id.to_string()),
    };

    let cfg3 = Config {
        inputs: photos.clone(),
        output: out3.clone(),
        work_dir: job_dir.join("work_v3"),
        width: 1080,
        height: 1920,
        fps: 30,
        shot_secs: v3_shot,
        trans_secs: (v3_shot * 0.2).min(0.5),
        music: music.clone(),
        engine: Engine::Ffmpeg,
        event,
        motion: v3_motion,
        banner_style: v3_banner,
        title: title.clone(),
        stickers: true,
        template_id: Some(t3.id.to_string()),
    };

    // Render all 3 variants in PARALLEL with pre-analyzed photos
    let analyses = photo_analyses.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        std::thread::scope(|s| {
            let h1 = s.spawn(|| {
                let res = pipeline::run_with_analyses(&cfg1, &analyses);
                let _ = std::fs::remove_dir_all(&cfg1.work_dir);
                res
            });
            let h2 = s.spawn(|| {
                let res = pipeline::run_with_analyses(&cfg2, &analyses);
                let _ = std::fs::remove_dir_all(&cfg2.work_dir);
                res
            });
            let h3 = s.spawn(|| {
                let res = pipeline::run_with_analyses(&cfg3, &analyses);
                let _ = std::fs::remove_dir_all(&cfg3.work_dir);
                res
            });

            h1.join().map_err(|_| anyhow::anyhow!("variant 1 thread panicked"))??;
            h2.join().map_err(|_| anyhow::anyhow!("variant 2 thread panicked"))??;
            h3.join().map_err(|_| anyhow::anyhow!("variant 3 thread panicked"))??;
            Ok(())
        })
    })
    .await
    .map_err(err)?
    .map_err(|e| err(format!("{e:#}")))?;

    let dur1 = (photos_count as f32 * v1_shot * 0.85).max(1.0);
    let dur2 = (photos_count as f32 * v2_shot * 0.85).max(1.0);
    let dur3 = (photos_count as f32 * v3_shot * 0.85).max(1.0);

    // Track analytics
    tracker.record_reel(
        &user_id,
        event.as_str(),
        motion.as_str(),
        photos_count,
        dur2,
        title.as_deref(),
    );

    let variants = vec![
        VariantItem {
            id: "variant_1".to_string(),
            filename: "variant_1.mp4".to_string(),
            title: v1_title,
            tag: v1_tag,
            description: v1_desc,
            duration: dur1,
            video_url: format!("/api/video/{}/variant_1.mp4", job_id),
        },
        VariantItem {
            id: "variant_2".to_string(),
            filename: "variant_2.mp4".to_string(),
            title: v2_title,
            tag: v2_tag,
            description: v2_desc,
            duration: dur2,
            video_url: format!("/api/video/{}/variant_2.mp4", job_id),
        },
        VariantItem {
            id: "variant_3".to_string(),
            filename: "variant_3.mp4".to_string(),
            title: v3_title,
            tag: v3_tag,
            description: v3_desc,
            duration: dur3,
            video_url: format!("/api/video/{}/variant_3.mp4", job_id),
        },
    ];

    Ok(Json(GenerateVariantsResponse {
        success: true,
        job_id,
        variants,
    }))
}

async fn customize_reel(Json(req): Json<CustomizeRequest>) -> Result<Json<CustomizeResponse>, (StatusCode, String)> {
    let clean_job = sanitize(&req.job_id);
    let clean_file = sanitize(&req.video_file);
    let input_path = PathBuf::from("reelforge_data/jobs").join(&clean_job).join(&clean_file);
    if !input_path.exists() {
        return Err((StatusCode::NOT_FOUND, "Original video variant not found".into()));
    }

    let custom_id = format!("custom_{}.mp4", &uuid::Uuid::new_v4().to_string()[..8]);
    let out_path = PathBuf::from("reelforge_data/jobs").join(&clean_job).join(&custom_id);

    let start = req.start_sec.max(0.0);
    let end = req.end_sec.max(start + 0.5);
    let dur = end - start;
    let speed = req.speed.unwrap_or(1.0).clamp(0.5, 2.5);

    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.arg("-y").arg("-hide_banner");

    // Fast seek & trim duration
    cmd.arg("-ss").arg(format!("{:.3}", start));
    cmd.arg("-t").arg(format!("{:.3}", dur));
    cmd.arg("-i").arg(&input_path);

    let mut vf_parts = Vec::new();
    if (speed - 1.0).abs() > 0.05 {
        vf_parts.push(format!("setpts={:.4}*PTS", 1.0 / speed));
    }
    match req.filter.as_deref().unwrap_or("none") {
        "warm" => vf_parts.push("eq=saturation=1.28:contrast=1.06:gamma_r=1.12:gamma_b=0.92".to_string()),
        "noir" => vf_parts.push("hue=s=0,eq=contrast=1.35:brightness=-0.04".to_string()),
        "vibrant" => vf_parts.push("eq=saturation=1.5:contrast=1.15".to_string()),
        "vintage" => vf_parts.push("curves=vintage,eq=saturation=0.85:contrast=1.08".to_string()),
        _ => {}
    }

    if !vf_parts.is_empty() {
        cmd.arg("-vf").arg(vf_parts.join(","));
    }

    if req.mute_audio.unwrap_or(false) {
        cmd.arg("-an");
    } else {
        let mut af_parts = Vec::new();
        if (speed - 1.0).abs() > 0.05 {
            af_parts.push(format!("atempo={:.3}", speed.clamp(0.5, 2.0)));
        }
        let vol = req.volume.unwrap_or(1.0).clamp(0.1, 3.0);
        if (vol - 1.0).abs() > 0.05 {
            af_parts.push(format!("volume={:.2}", vol));
        }
        if !af_parts.is_empty() {
            cmd.arg("-af").arg(af_parts.join(","));
        }
    }

    cmd.arg("-c:v").arg("libx264").arg("-preset").arg("fast").arg("-crf").arg("18");
    cmd.arg("-c:a").arg("aac").arg("-b:a").arg("192k");
    cmd.arg("-movflags").arg("+faststart");
    cmd.arg(&out_path);

    let status = cmd.status().map_err(err)?;
    if !status.success() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "FFmpeg customization failed".into()));
    }

    let final_dur = dur / speed;
    Ok(Json(CustomizeResponse {
        success: true,
        video_url: format!("/api/video/{}/{}", clean_job, custom_id),
        filename: custom_id,
        duration: final_dur,
        message: format!("Timeframe successfully trimmed ({:.1}s–{:.1}s) at {:.2}x speed!", start, end, speed),
    }))
}

async fn serve_video(Path((job_id, filename)): Path<(String, String)>) -> Result<Response, (StatusCode, String)> {
    let clean_job = sanitize(&job_id);
    let clean_file = sanitize(&filename);
    let path = PathBuf::from("reelforge_data/jobs").join(clean_job).join(clean_file);
    if !path.exists() {
        return Err((StatusCode::NOT_FOUND, "Video file not found".into()));
    }
    let bytes = std::fs::read(&path).map_err(err)?;
    Ok((
        [
            (header::CONTENT_TYPE, "video/mp4"),
            (header::CONTENT_DISPOSITION, "inline"),
            (header::ACCEPT_RANGES, "bytes"),
        ],
        bytes,
    ).into_response())
}

fn probe_video_duration(p: &std::path::Path) -> f32 {
    let out = std::process::Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1"])
        .arg(p)
        .output();
    if let Ok(o) = out {
        if let Ok(s) = std::str::from_utf8(&o.stdout) {
            if let Ok(d) = s.trim().parse::<f32>() {
                return d;
            }
        }
    }
    12.0
}

async fn get_job_variants(Path(job_id): Path<String>) -> Result<Json<GenerateVariantsResponse>, (StatusCode, String)> {
    let clean_job = sanitize(&job_id);
    let job_dir = PathBuf::from("reelforge_data/jobs").join(&clean_job);
    if !job_dir.exists() {
        return Err((StatusCode::NOT_FOUND, "Job not found".into()));
    }

    let mut variants = Vec::new();
    let v_files = [
        ("variant_1.mp4", "Variant 1 — Dynamic Auto-Match", "Top AI Match", "High-energy beat cuts with dynamic zoom & pan motion"),
        ("variant_2.mp4", "Variant 2 — Celebratory Elegance", "Stylistic Pick", "Smooth float & glide transitions with clean portrait focus"),
        ("variant_3.mp4", "Variant 3 — Heart & Star Montage", "Creative Cut", "Faceted gem framing with celebratory pacing and vivid motion"),
    ];

    for (fname, title, tag, desc) in v_files {
        let p = job_dir.join(fname);
        if p.exists() {
            let dur = probe_video_duration(&p);
            variants.push(VariantItem {
                id: fname.replace(".mp4", ""),
                filename: fname.to_string(),
                title: title.to_string(),
                tag: tag.to_string(),
                description: desc.to_string(),
                duration: dur,
                video_url: format!("/api/video/{}/{}", clean_job, fname),
            });
        }
    }

    if variants.is_empty() {
        let p = job_dir.join("reel.mp4");
        if p.exists() {
            let dur = probe_video_duration(&p);
            variants.push(VariantItem {
                id: "variant_1".to_string(),
                filename: "reel.mp4".to_string(),
                title: "Standard Reel".to_string(),
                tag: "Rendered".to_string(),
                description: "Clean vertical 9:16 reel".to_string(),
                duration: dur,
                video_url: format!("/api/video/{}/reel.mp4", clean_job),
            });
        }
    }

    Ok(Json(GenerateVariantsResponse {
        job_id: clean_job,
        success: true,
        variants,
    }))
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' }).collect()
}
