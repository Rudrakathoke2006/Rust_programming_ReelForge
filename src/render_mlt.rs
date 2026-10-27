use crate::types::{Config, Shot};
use anyhow::{bail, Context, Result};
use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::Command;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Write an MLT XML project: two alternating video tracks with luma dissolves
/// between overlapping clips (the classic Kdenlive/Shotcut layout), plus optional music.
pub fn write_project(cfg: &Config, shots: &[Shot]) -> Result<PathBuf> {
    let d = cfg.shot_frames() as i64;
    let t = cfg.trans_frames() as i64;
    let n = shots.len() as i64;
    let total = n * d - (n - 1) * t;
    let step = d - t; // start-to-start distance between clips

    let mut x = String::new();
    writeln!(x, r#"<?xml version="1.0" encoding="utf-8"?>"#)?;
    writeln!(x, r#"<mlt LC_NUMERIC="C" version="7.0.0" title="ReelForge" producer="main">"#)?;
    writeln!(
        x,
        r#"  <profile description="reel" width="{}" height="{}" progressive="1" sample_aspect_num="1" sample_aspect_den="1" display_aspect_num="{}" display_aspect_den="{}" frame_rate_num="{}" frame_rate_den="1" colorspace="709"/>"#,
        cfg.width, cfg.height, cfg.width, cfg.height, cfg.fps
    )?;

    for (i, s) in shots.iter().enumerate() {
        let abs = std::fs::canonicalize(&s.frame).unwrap_or(s.frame.clone());
        writeln!(x, r#"  <producer id="clip{i}" in="0" out="{}">"#, d - 1)?;
        writeln!(x, r#"    <property name="resource">{}</property>"#, esc(&abs.to_string_lossy()))?;
        writeln!(x, r#"    <property name="length">{d}</property>"#)?;
        writeln!(x, r#"    <property name="eof">pause</property>"#)?;
        writeln!(x, r#"    <property name="aspect_ratio">1</property>"#)?;
        writeln!(x, r#"    <property name="ttl">1</property>"#)?;
        writeln!(x, r#"    <property name="kdenlive:clipname">shot_{i:03}</property>"#)?;
        writeln!(x, r#"  </producer>"#)?;
    }

    // Track A holds even clips, track B odd clips. Blanks position them on the timeline.
    let mut a = String::new();
    let mut b = String::new();
    // B starts after one `step`
    writeln!(b, r#"    <blank length="{step}"/>"#)?;
    for i in 0..n {
        let tgt = if i % 2 == 0 { &mut a } else { &mut b };
        writeln!(tgt, r#"    <entry producer="clip{i}" in="0" out="{}"/>"#, d - 1)?;
        // gap until this track's next clip: 2*step - d
        if i + 2 < n {
            let gap = 2 * step - d;
            if gap > 0 {
                writeln!(tgt, r#"    <blank length="{gap}"/>"#)?;
            }
        }
    }
    writeln!(x, "  <playlist id=\"trackA\">\n{a}  </playlist>")?;
    writeln!(x, "  <playlist id=\"trackB\">\n{b}  </playlist>")?;

    let has_music = cfg.music.is_some();
    if let Some(m) = &cfg.music {
        let abs = std::fs::canonicalize(m).unwrap_or(m.clone());
        writeln!(x, r#"  <producer id="music" in="0" out="{}">"#, total - 1)?;
        writeln!(x, r#"    <property name="resource">{}</property>"#, esc(&abs.to_string_lossy()))?;
        writeln!(x, r#"  </producer>"#)?;
        writeln!(x, r#"  <playlist id="trackM"><entry producer="music" in="0" out="{}"/></playlist>"#, total - 1)?;
    }

    writeln!(x, r#"  <tractor id="main" in="0" out="{}">"#, total - 1)?;
    writeln!(x, r#"    <track producer="trackA"/>"#)?;
    writeln!(x, r#"    <track producer="trackB"/>"#)?;
    if has_music {
        writeln!(x, r#"    <track producer="trackM" hide="video"/>"#)?;
    }
    for i in 1..n {
        let start = i * step;
        let (from, to) = if i % 2 == 1 { (0, 1) } else { (1, 0) };
        writeln!(
            x,
            r#"    <transition mlt_service="luma" in="{start}" out="{}" a_track="{from}" b_track="{to}"><property name="softness">0.15</property></transition>"#,
            start + t - 1
        )?;
    }
    if has_music {
        writeln!(x, r#"    <transition mlt_service="mix" in="0" out="{}" a_track="0" b_track="2" always_active="1" sum="1"/>"#, total - 1)?;
        // 1.5 s audio fade-out on the music
        let fo = (cfg.fps as f32 * 1.5) as i64;
        writeln!(x, r#"    <filter mlt_service="volume" track="2" in="{}" out="{}"><property name="gain">1</property><property name="end">0</property></filter>"#, (total - fo).max(0), total - 1)?;
    }
    writeln!(x, "  </tractor>")?;
    writeln!(x, "</mlt>")?;

    let path = cfg.work_dir.join("reel.mlt");
    std::fs::write(&path, x)?;
    Ok(path)
}

pub fn render(cfg: &Config, project: &PathBuf) -> Result<()> {
    println!("▶ melt {} → {}", project.display(), cfg.output.display());
    let status = Command::new("melt")
        .arg(project)
        .arg("-consumer")
        .arg(format!("avformat:{}", cfg.output.display()))
        .args(["vcodec=libx264", "acodec=aac", "crf=18", "preset=medium", "pix_fmt=yuv420p", "movflags=+faststart"])
        .status()
        .context("failed to run melt (install MLT: `apt install melt` / `brew install mlt`)")?;
    if !status.success() {
        bail!("melt exited with {status}");
    }
    Ok(())
}
