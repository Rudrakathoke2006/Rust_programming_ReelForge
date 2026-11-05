use crate::types::{Config, Shot};
use anyhow::{bail, Context, Result};
use std::process::Command;

const XFADES: &[&str] = &["fade", "slideleft", "circleopen", "wipeup", "smoothleft", "zoomin", "dissolve", "slideright"];

/// Build the full ffmpeg command: per-shot Ken Burns (zoompan) + xfade chain + optional music.
pub fn build_args(cfg: &Config, shots: &[Shot]) -> Vec<String> {
    let d = cfg.shot_frames();
    let t = cfg.trans_frames();
    let fps = cfg.fps;
    let (w, h) = (cfg.width, cfg.height);
    let dsec = d as f32 / fps as f32;
    let tsec = t as f32 / fps as f32;

    let mut args: Vec<String> = vec!["-y".into(), "-hide_banner".into()];
    for s in shots {
        args.push("-i".into());
        args.push(s.frame.to_string_lossy().into_owned());
    }
    let music_idx = shots.len();
    if let Some(m) = &cfg.music {
        args.push("-stream_loop".into());
        args.push("-1".into());
        args.push("-i".into());
        args.push(m.to_string_lossy().into_owned());
    }

    let mut f = String::new();
    for (i, s) in shots.iter().enumerate() {
        let z = match s.motion {
            crate::types::ShotMotion::Boomerang => format!("1.0+0.22*sin(3.14159265*on/{d})"),
            crate::types::ShotMotion::FastForward => format!("1.0+0.32*(on/{d})*(on/{d})"),
            crate::types::ShotMotion::ZoomIn => format!("1.0+0.14*on/{d}"),
            crate::types::ShotMotion::ZoomOut => format!("1.14-0.14*on/{d}"),
        };
        let zw = (w as f32 * 1.15) as u32;
        let zh = (h as f32 * 1.15) as u32;
        f.push_str(&format!(
            "[{i}:v]scale={zw}:{zh},zoompan=z='{z}':x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':d={d}:s={w}x{h}:fps={fps},setsar=1,format=yuv420p[v{i}];"
        ));
    }

    let n = shots.len();
    let last_label;
    if n == 1 {
        last_label = "v0".to_string();
    } else {
        let mut prev = "v0".to_string();
        for i in 1..n {
            let offset = i as f32 * (dsec - tsec);
            let name = format!("x{i}");
            f.push_str(&format!(
                "[{prev}][v{i}]xfade=transition={}:duration={tsec:.3}:offset={offset:.3}[{name}];",
                XFADES[(i - 1) % XFADES.len()]
            ));
            prev = name;
        }
        last_label = prev;
    }
    let total = n as f32 * dsec - (n.saturating_sub(1)) as f32 * tsec;

    let mut text_filter = String::new();
    if cfg.stickers || cfg.title.is_some() {
        let font_path = if std::path::Path::new("C:/Windows/Fonts/seguiemj.ttf").exists() {
            Some("C\\:/Windows/Fonts/seguiemj.ttf")
        } else if std::path::Path::new("C:/Windows/Fonts/arial.ttf").exists() {
            Some("C\\:/Windows/Fonts/arial.ttf")
        } else {
            None
        };

        if let Some(font) = font_path {
            let (line1, line2) = crate::templates::split_title(
                cfg.title.as_deref().unwrap_or(""),
                cfg.event,
            );
            let line1_clean = line1.replace('\'', "").replace(':', " ").replace('\\', "");
            let line2_clean = line2.replace('\'', "").replace(':', " ").replace('\\', "");

            // Get title accent color from template
            let accent_hex = if let Some(tid) = &cfg.template_id {
                crate::templates::find_template(tid)
                    .map(|t| t.palette.title_hex())
                    .unwrap_or("0xffd700")
            } else {
                "0xffd700"
            };

            let badge_duration = (total.min(4.2)).max(2.2);
            let h = cfg.height as i32;

            let (y_line1, y_line2, font_size1, font_size2, x_expr1, x_expr2) = match cfg.banner_style {
                crate::types::BannerStyle::CircularStamp => {
                    // Aligned directly in the circular emblem seal
                    (240, 290, 20, 38, "(w-text_w)/2".to_string(), "(w-text_w)/2".to_string())
                }
                crate::types::BannerStyle::LowerThird => {
                    // Left-aligned in the cinematic lower-third card
                    (h - 330, h - 290, 22, 38, "100".to_string(), "100".to_string())
                }
                crate::types::BannerStyle::CenterHero => {
                    // Bold magazine headline across the center
                    ((h / 2) - 65, (h / 2) - 15, 26, 50, "(w-text_w)/2".to_string(), "(w-text_w)/2".to_string())
                }
                crate::types::BannerStyle::BottomFloat => {
                    (h - 225, h - 192, 19, 34, "(w-text_w)/2".to_string(), "(w-text_w)/2".to_string())
                }
                crate::types::BannerStyle::TopPill => {
                    (120, 160, 24, 40, "(w-text_w)/2".to_string(), "(w-text_w)/2".to_string())
                }
            };

            if !line1_clean.is_empty() || !line2_clean.is_empty() {
                let mut parts = Vec::new();
                if !line1_clean.is_empty() {
                    parts.push(format!(
                        "drawtext=fontfile='{font}':text='{line1_clean}':fontsize={font_size1}:fontcolor=white@0.9:x={x_expr1}:y={y_line1}:enable='between(t,0.2,{badge_duration:.2})'"
                    ));
                }
                if !line2_clean.is_empty() {
                    parts.push(format!(
                        "drawtext=fontfile='{font}':text='{line2_clean}':fontsize={font_size2}:fontcolor={accent_hex}:x={x_expr2}:y={y_line2}:enable='between(t,0.2,{badge_duration:.2})'"
                    ));
                }
                if !parts.is_empty() {
                    text_filter = format!(",{}", parts.join(","));
                }
            }
        }
    }

    f.push_str(&format!(
        "[{last_label}]fade=t=in:st=0:d=0.3,fade=t=out:st={:.3}:d=0.5{text_filter}[vout]",
        (total - 0.5).max(0.0)
    ));

    args.push("-filter_complex".into());
    args.push(f);
    args.push("-map".into());
    args.push("[vout]".into());
    if cfg.music.is_some() {
        args.push("-map".into());
        args.push(format!("{music_idx}:a"));
        args.push("-af".into());
        args.push(format!("afade=t=out:st={:.3}:d=1.5", (total - 1.5).max(0.0)));
        args.push("-c:a".into());
        args.push("aac".into());
        args.push("-b:a".into());
        args.push("192k".into());
    }
    let tail: Vec<String> = vec![
        "-t".into(), format!("{total:.3}"),
        "-c:v".into(), "libx264".into(),
        "-preset".into(), "ultrafast".into(),
        "-crf".into(), "22".into(),
        "-tune".into(), "fastdecode".into(),
        "-threads".into(), "0".into(),
        "-pix_fmt".into(), "yuv420p".into(),
        "-r".into(), fps.to_string(),
        "-movflags".into(), "+faststart".into(),
    ];
    args.extend(tail);
    args.push(cfg.output.to_string_lossy().into_owned());
    args
}

pub fn render(cfg: &Config, shots: &[Shot]) -> Result<()> {
    if shots.is_empty() {
        bail!("no shots to render");
    }
    let args = build_args(cfg, shots);
    println!("▶ ffmpeg {} inputs → {}", shots.len(), cfg.output.display());
    let status = Command::new("ffmpeg").args(&args).status().context("failed to run ffmpeg (is it installed and on PATH?)")?;
    if !status.success() {
        bail!("ffmpeg exited with {status}");
    }
    Ok(())
}
