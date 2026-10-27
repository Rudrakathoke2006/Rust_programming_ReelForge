use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Engine {
    /// Render directly with FFmpeg (Ken Burns zoom + xfade transitions)
    Ffmpeg,
    /// Write an MLT project and render it with `melt`
    Melt,
    /// Only write the MLT project (open it in Kdenlive / melt later)
    MltOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Default, Serialize, Deserialize)]
pub enum EventTheme {
    #[default]
    General,
    Birthday,
    Anniversary,
    Romantic,
    Party,
    Travel,
}

impl EventTheme {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "birthday" => Self::Birthday,
            "anniversary" => Self::Anniversary,
            "romantic" | "love" => Self::Romantic,
            "party" | "celebration" => Self::Party,
            "travel" | "trip" => Self::Travel,
            _ => Self::General,
        }
    }

    pub fn default_badge_text(&self) -> &'static str {
        match self {
            Self::Birthday => "🎂 Happy Birthday! 🎉",
            Self::Anniversary => "❤️ Happy Anniversary ✨",
            Self::Romantic => "💖 Forever & Always 💕",
            Self::Party => "🎉 Celebration Time 🍾",
            Self::Travel => "✈️ Wanderlust Memories 📸",
            Self::General => "✨ Special Moments ✨",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Default, Serialize, Deserialize)]
pub enum MotionMode {
    #[default]
    DynamicMix,
    Boomerang,
    FastForward,
    KenBurns,
}

impl MotionMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "boomerang" => Self::Boomerang,
            "fast-forward" | "fastforward" | "fast" => Self::FastForward,
            "kenburns" | "ken-burns" | "cinematic" => Self::KenBurns,
            _ => Self::DynamicMix,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotMotion {
    Boomerang,
    FastForward,
    ZoomIn,
    ZoomOut,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub inputs: Vec<PathBuf>,
    pub output: PathBuf,
    pub work_dir: PathBuf,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub shot_secs: f32,
    pub trans_secs: f32,
    pub music: Option<PathBuf>,
    pub engine: Engine,
    pub event: EventTheme,
    pub motion: MotionMode,
    pub title: Option<String>,
    pub stickers: bool,
    pub template_id: Option<String>,
}

impl Config {
    pub fn shot_frames(&self) -> u32 {
        (self.shot_secs * self.fps as f32).round().max(2.0) as u32
    }
    pub fn trans_frames(&self) -> u32 {
        let t = (self.trans_secs * self.fps as f32).round() as u32;
        // MLT two-track layout needs shot >= 2 * transition
        t.min(self.shot_frames() / 2).max(1)
    }
}

/// Result of analysing one photo.
#[derive(Clone, Debug)]
pub struct Analysis {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub sharpness: f32,
    pub brightness: f32,
    /// Normalised (0..1) centre of visual interest.
    pub focus: (f32, f32),
}

impl Analysis {
    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
    pub fn is_landscape(&self) -> bool {
        self.aspect() > 1.15
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layout {
    /// Smart-cropped to fill the whole 9:16 frame
    Fill,
    /// Whole photo fitted over a blurred copy of itself
    BlurFit,
    Circle,
    Diamond,
    /// Rounded 4:5 card
    Card,
    /// Two photos stacked
    Split,
    /// Romantic Heart shape with soft glowing border
    Heart,
    /// Festive sparkle star shape
    Star,
    /// 3-photo celebration collage (1 hero top, 2 side-by-side bottom)
    Collage3,
    /// Classic retro polaroid card with white border
    Polaroid,
}

#[derive(Clone, Debug)]
pub struct Shot {
    pub layout: Layout,
    pub photos: Vec<usize>, // indices into the analysis list
    pub frame: PathBuf,     // composed 1080x1920 PNG
    pub motion: ShotMotion, // Boomerang, FastForward, ZoomIn, ZoomOut
}
