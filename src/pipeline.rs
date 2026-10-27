use crate::types::{Config, Engine};
use crate::{analyze, compose, render_ffmpeg, render_mlt};
use anyhow::{bail, Result};

pub fn run(cfg: &Config) -> Result<()> {
    if cfg.inputs.is_empty() {
        bail!("no images found");
    }
    std::fs::create_dir_all(&cfg.work_dir)?;

    println!("① analysing {} photos", cfg.inputs.len());
    let mut photos = Vec::new();
    for p in &cfg.inputs {
        match analyze::analyze(p) {
            Ok(a) => {
                println!(
                    "  {} {}x{} sharp={:.1} bright={:.2} focus=({:.2},{:.2})",
                    p.file_name().unwrap().to_string_lossy(), a.width, a.height, a.sharpness, a.brightness, a.focus.0, a.focus.1
                );
                photos.push(a);
            }
            Err(e) => eprintln!("  skipping {}: {e:#}", p.display()),
        }
    }
    if photos.is_empty() {
        bail!("none of the images could be decoded");
    }

    println!("② framing & shaping");
    let shots = compose::render_all(cfg, &photos)?;

    println!("③ rendering ({:?})", cfg.engine);
    match cfg.engine {
        Engine::Ffmpeg => render_ffmpeg::render(cfg, &shots)?,
        Engine::Melt => {
            let proj = render_mlt::write_project(cfg, &shots)?;
            render_mlt::render(cfg, &proj)?;
        }
        Engine::MltOnly => {
            let proj = render_mlt::write_project(cfg, &shots)?;
            println!("MLT project written: {}", proj.display());
            println!("Open it in Kdenlive, or: melt {}", proj.display());
        }
    }
    Ok(())
}

/// Turn a BPM into a shot length: 4 beats per shot, doubled/halved into a 1.6–3.6 s sweet spot.
pub fn shot_secs_from_bpm(bpm: f32) -> f32 {
    let mut s = 4.0 * 60.0 / bpm.max(1.0);
    while s > 3.6 {
        s /= 2.0;
    }
    while s < 1.6 {
        s *= 2.0;
    }
    s
}
