mod analyze;
mod analytics;
mod compose;
mod pipeline;
mod render_ffmpeg;
mod render_mlt;
mod server;
mod types;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use types::{Config, Engine};

#[derive(Parser)]
#[command(name = "reelforge", about = "Photos → vertical reel (analyse, smart-crop, shape, cut, render)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Build a reel from a folder of photos
    Make {
        /// Folder containing photos (searched recursively)
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long, default_value = "reel.mp4")]
        output: PathBuf,
        /// Optional background music
        #[arg(short, long)]
        music: Option<PathBuf>,
        /// Beats per minute of the music; sets cut length to 4 beats
        #[arg(long)]
        bpm: Option<f32>,
        /// Seconds per photo (ignored if --bpm is set)
        #[arg(long, default_value_t = 2.4)]
        shot_secs: f32,
        #[arg(long, value_enum, default_value_t = Engine::Ffmpeg)]
        engine: Engine,
        /// Working folder for composed frames and the .mlt project
        #[arg(long, default_value = "reel_work")]
        work: PathBuf,
        #[arg(long, default_value_t = 1080)]
        width: u32,
        #[arg(long, default_value_t = 1920)]
        height: u32,
        #[arg(long, default_value_t = 30)]
        fps: u32,
        /// Occasion theme (birthday, anniversary, romantic, party, travel, general)
        #[arg(long, value_enum, default_value_t = types::EventTheme::General)]
        event: types::EventTheme,
        /// Motion pacing style (dynamic-mix, boomerang, fast-forward, ken-burns)
        #[arg(long, value_enum, default_value_t = types::MotionMode::DynamicMix)]
        motion: types::MotionMode,
        /// Optional title / banner greeting
        #[arg(long)]
        title: Option<String>,
        /// Include event decorative stickers & emojis
        #[arg(long, default_value_t = true)]
        stickers: bool,
    },
    /// Start the web UI (upload photos in the browser)
    Serve {
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match Cli::parse().cmd {
        Cmd::Make {
            input,
            output,
            music,
            bpm,
            shot_secs,
            engine,
            work,
            width,
            height,
            fps,
            event,
            motion,
            title,
            stickers,
        } => {
            let shot = bpm.map(pipeline::shot_secs_from_bpm).unwrap_or(shot_secs);
            let cfg = Config {
                inputs: analyze::collect_images(&input),
                output,
                work_dir: work,
                width,
                height,
                fps,
                shot_secs: shot,
                trans_secs: (shot * 0.2).min(0.6),
                music,
                engine,
                event,
                motion,
                title,
                stickers,
            };
            pipeline::run(&cfg)
        }
        Cmd::Serve { port } => server::serve(port).await,
    }
}
