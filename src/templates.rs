use crate::types::{Analysis, EventTheme, Layout, MotionMode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaletteVibe {
    GoldLuxury,      // [255, 215, 0], [255, 240, 180]
    RoseElegance,    // [255, 105, 180], [255, 220, 235]
    RubyRomantic,    // [235, 45, 85], [255, 160, 180]
    CyberParty,      // [168, 85, 247], [59, 130, 246]
    NeonSunset,      // [249, 115, 22], [236, 72, 153]
    TropicalAzure,   // [14, 165, 233], [20, 184, 166]
    VintagePolaroid, // [245, 245, 240], [210, 180, 140]
    EmeraldPrestige, // [16, 185, 129], [52, 211, 153]
    VelvetMidnight,  // [139, 92, 246], [192, 132, 252]
    WarmSunlight,    // [251, 191, 36], [245, 158, 11]
}

impl PaletteVibe {
    pub fn border_rgb(&self) -> [u8; 3] {
        match self {
            Self::GoldLuxury => [255, 225, 130],
            Self::RoseElegance => [255, 190, 220],
            Self::RubyRomantic => [255, 150, 180],
            Self::CyberParty => [180, 130, 255],
            Self::NeonSunset => [255, 160, 110],
            Self::TropicalAzure => [130, 225, 255],
            Self::VintagePolaroid => [255, 255, 255],
            Self::EmeraldPrestige => [140, 240, 190],
            Self::VelvetMidnight => [200, 160, 255],
            Self::WarmSunlight => [255, 220, 110],
        }
    }

    pub fn title_hex(&self) -> &'static str {
        match self {
            Self::GoldLuxury => "0xffd700",
            Self::RoseElegance => "0xff69b4",
            Self::RubyRomantic => "0xff3366",
            Self::CyberParty => "0xa855f7",
            Self::NeonSunset => "0xff7733",
            Self::TropicalAzure => "0x00e5ff",
            Self::VintagePolaroid => "0xffffff",
            Self::EmeraldPrestige => "0x10b981",
            Self::VelvetMidnight => "0xc084fc",
            Self::WarmSunlight => "0xfbbf24",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ReelTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub event: EventTheme,
    pub description: &'static str,
    pub palette: PaletteVibe,
    pub layouts: &'static [Layout],
    pub motion: MotionMode,
    pub default_shot_secs: f32,
    pub sticker_theme: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScoredTemplate {
    pub template: ReelTemplate,
    pub match_score: usize, // 0 - 100%
    pub match_reason: String,
}

/// Registry of 48 Extraordinary Predefined Templates across all occasions
pub const TEMPLATES: &[ReelTemplate] = &[
    // --- BIRTHDAY (8 templates) ---
    ReelTemplate {
        id: "bday_golden_confetti",
        name: "Golden Confetti Bash",
        event: EventTheme::Birthday,
        description: "3-photo collages, festive star frames & celebratory gold confetti",
        palette: PaletteVibe::GoldLuxury,
        layouts: &[Layout::Collage3, Layout::Star, Layout::Circle, Layout::Card],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 2.0,
        sticker_theme: "confetti",
    },
    ReelTemplate {
        id: "bday_neon_bash",
        name: "Neon Night Glow",
        event: EventTheme::Birthday,
        description: "High-voltage multi-cuts, diamond masks and rapid celebratory transitions",
        palette: PaletteVibe::CyberParty,
        layouts: &[Layout::Collage3, Layout::Star, Layout::Diamond, Layout::Split],
        motion: MotionMode::FastForward,
        default_shot_secs: 1.6,
        sticker_theme: "sparkles",
    },
    ReelTemplate {
        id: "bday_cupcake_dream",
        name: "Pastel Cupcake Dream",
        event: EventTheme::Birthday,
        description: "Soft romantic heart & star frames with gentle celebratory bounce",
        palette: PaletteVibe::RoseElegance,
        layouts: &[Layout::Heart, Layout::Star, Layout::Card, Layout::BlurFit],
        motion: MotionMode::Boomerang,
        default_shot_secs: 2.2,
        sticker_theme: "hearts",
    },
    ReelTemplate {
        id: "bday_superstar_hero",
        name: "Superstar Spotlight",
        event: EventTheme::Birthday,
        description: "Full-bleed opening hook with star collages and warm golden glow",
        palette: PaletteVibe::WarmSunlight,
        layouts: &[Layout::Star, Layout::Fill, Layout::Collage3, Layout::Circle],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 1.8,
        sticker_theme: "sparkles",
    },
    ReelTemplate {
        id: "bday_balloon_festival",
        name: "Balloon Pop Carnival",
        event: EventTheme::Birthday,
        description: "Playful circles, 3-photo celebration collages and snappy cuts",
        palette: PaletteVibe::NeonSunset,
        layouts: &[Layout::Circle, Layout::Collage3, Layout::Diamond, Layout::Card],
        motion: MotionMode::FastForward,
        default_shot_secs: 1.5,
        sticker_theme: "confetti",
    },
    ReelTemplate {
        id: "bday_scrapbook_diary",
        name: "Birthday Scrapbook",
        event: EventTheme::Birthday,
        description: "Retro polaroid cards, washi tape borders and nostalgic memory cuts",
        palette: PaletteVibe::VintagePolaroid,
        layouts: &[Layout::Polaroid, Layout::Collage3, Layout::Split, Layout::Card],
        motion: MotionMode::KenBurns,
        default_shot_secs: 2.4,
        sticker_theme: "polaroid",
    },
    ReelTemplate {
        id: "bday_champagne_cheers",
        name: "Champagne Toast",
        event: EventTheme::Birthday,
        description: "Elegant gold stars, card framing and rhythmic beat bounce",
        palette: PaletteVibe::GoldLuxury,
        layouts: &[Layout::Star, Layout::Card, Layout::Star, Layout::BlurFit],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 2.1,
        sticker_theme: "sparkles",
    },
    ReelTemplate {
        id: "bday_firework_rush",
        name: "Fireworks Finale",
        event: EventTheme::Birthday,
        description: "Snappy multi-cut montage for high-energy celebrations",
        palette: PaletteVibe::CyberParty,
        layouts: &[Layout::Collage3, Layout::Star, Layout::Split, Layout::Circle],
        motion: MotionMode::FastForward,
        default_shot_secs: 1.4,
        sticker_theme: "confetti",
    },

    // --- ANNIVERSARY (8 templates) ---
    ReelTemplate {
        id: "anniv_ruby_heartbeat",
        name: "Ruby Heartbeat Symphony",
        event: EventTheme::Anniversary,
        description: "Anti-aliased Heart frames with romantic Boomerang pulse and rose glow",
        palette: PaletteVibe::RubyRomantic,
        layouts: &[Layout::Heart, Layout::Card, Layout::Heart, Layout::BlurFit],
        motion: MotionMode::Boomerang,
        default_shot_secs: 2.3,
        sticker_theme: "hearts",
    },
    ReelTemplate {
        id: "anniv_golden_milestone",
        name: "Golden Milestone Story",
        event: EventTheme::Anniversary,
        description: "Luxury gold borders, romantic cards, and elegant slow-zoom glides",
        palette: PaletteVibe::GoldLuxury,
        layouts: &[Layout::Heart, Layout::Card, Layout::Split, Layout::BlurFit],
        motion: MotionMode::KenBurns,
        default_shot_secs: 2.5,
        sticker_theme: "hearts",
    },
    ReelTemplate {
        id: "anniv_velvet_rose",
        name: "Velvet Rose Memories",
        event: EventTheme::Anniversary,
        description: "Deep rose heart frames, floating love hearts and gentle bounce zoom",
        palette: PaletteVibe::RoseElegance,
        layouts: &[Layout::Heart, Layout::Heart, Layout::Card, Layout::BlurFit],
        motion: MotionMode::Boomerang,
        default_shot_secs: 2.2,
        sticker_theme: "hearts",
    },
    ReelTemplate {
        id: "anniv_vintage_fairytale",
        name: "Fairytale Scrapbook",
        event: EventTheme::Anniversary,
        description: "Polaroid cards and heart collages capturing years of love",
        palette: PaletteVibe::VintagePolaroid,
        layouts: &[Layout::Card, Layout::Heart, Layout::Polaroid, Layout::Split],
        motion: MotionMode::KenBurns,
        default_shot_secs: 2.6,
        sticker_theme: "polaroid",
    },
    ReelTemplate {
        id: "anniv_modern_chic",
        name: "Modern Chic Duet",
        event: EventTheme::Anniversary,
        description: "Split panoramas, heart hero shot and contemporary rhythm",
        palette: PaletteVibe::EmeraldPrestige,
        layouts: &[Layout::Split, Layout::Heart, Layout::Card, Layout::Fill],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 2.0,
        sticker_theme: "sparkles",
    },
    ReelTemplate {
        id: "anniv_sunset_vows",
        name: "Sunset Promise",
        event: EventTheme::Anniversary,
        description: "Atmospheric blur-fit and heart pulse bathed in sunset amber",
        palette: PaletteVibe::NeonSunset,
        layouts: &[Layout::BlurFit, Layout::Heart, Layout::Card, Layout::Heart],
        motion: MotionMode::Boomerang,
        default_shot_secs: 2.2,
        sticker_theme: "hearts",
    },
    ReelTemplate {
        id: "anniv_diamond_forever",
        name: "Diamond Eternity",
        event: EventTheme::Anniversary,
        description: "Diamond and heart geometry with soft purple-gold accents",
        palette: PaletteVibe::VelvetMidnight,
        layouts: &[Layout::Diamond, Layout::Heart, Layout::Card, Layout::BlurFit],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 2.1,
        sticker_theme: "hearts",
    },
    ReelTemplate {
        id: "anniv_candlelight_glow",
        name: "Candlelight Serenade",
        event: EventTheme::Anniversary,
        description: "Slow cinematic zooms with soft glowing heart borders",
        palette: PaletteVibe::WarmSunlight,
        layouts: &[Layout::Heart, Layout::Card, Layout::BlurFit, Layout::Split],
        motion: MotionMode::KenBurns,
        default_shot_secs: 2.8,
        sticker_theme: "hearts",
    },

    // --- ROMANTIC (8 templates) ---
    ReelTemplate {
        id: "romance_endless_pulse",
        name: "Endless Love Pulse",
        event: EventTheme::Romantic,
        description: "Signature forward-and-back Boomerang zoom on romantic heart frames",
        palette: PaletteVibe::RubyRomantic,
        layouts: &[Layout::Heart, Layout::BlurFit, Layout::Heart, Layout::Card],
        motion: MotionMode::Boomerang,
        default_shot_secs: 2.0,
        sticker_theme: "hearts",
    },
    ReelTemplate {
        id: "romance_cupid_whisper",
        name: "Cupid's Sweet Whisper",
        event: EventTheme::Romantic,
        description: "Soft circle vignettes, heart cutouts and floating love particles",
        palette: PaletteVibe::RoseElegance,
        layouts: &[Layout::Heart, Layout::Circle, Layout::Card, Layout::BlurFit],
        motion: MotionMode::Boomerang,
        default_shot_secs: 2.2,
        sticker_theme: "hearts",
    },
    ReelTemplate {
        id: "romance_cinematic_story",
        name: "Cinematic Love Story",
        event: EventTheme::Romantic,
        description: "Slow-paced poetic glides and widescreen split-frame memories",
        palette: PaletteVibe::VelvetMidnight,
        layouts: &[Layout::Card, Layout::Heart, Layout::Split, Layout::Fill],
        motion: MotionMode::KenBurns,
        default_shot_secs: 2.5,
        sticker_theme: "sparkles",
    },
    ReelTemplate {
        id: "romance_polaroid_date",
        name: "Coffee Date Polaroid",
        event: EventTheme::Romantic,
        description: "Scrapbook aesthetic with washi tape accents and heart badges",
        palette: PaletteVibe::VintagePolaroid,
        layouts: &[Layout::Polaroid, Layout::Heart, Layout::Card, Layout::Split],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 2.1,
        sticker_theme: "polaroid",
    },
    ReelTemplate {
        id: "romance_golden_hour",
        name: "Golden Hour Romance",
        event: EventTheme::Romantic,
        description: "Warm amber sunburst framing and rhythmic heart pulses",
        palette: PaletteVibe::WarmSunlight,
        layouts: &[Layout::Heart, Layout::Card, Layout::BlurFit, Layout::Heart],
        motion: MotionMode::Boomerang,
        default_shot_secs: 2.2,
        sticker_theme: "hearts",
    },
    ReelTemplate {
        id: "romance_midnight_stars",
        name: "Midnight Constellation",
        event: EventTheme::Romantic,
        description: "Star and heart masks under deep violet mood lighting",
        palette: PaletteVibe::VelvetMidnight,
        layouts: &[Layout::Star, Layout::Heart, Layout::Diamond, Layout::BlurFit],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 2.0,
        sticker_theme: "sparkles",
    },
    ReelTemplate {
        id: "romance_rose_garden",
        name: "English Rose Garden",
        event: EventTheme::Romantic,
        description: "Delicate pastel rose borders and smooth cinematic glides",
        palette: PaletteVibe::RoseElegance,
        layouts: &[Layout::Heart, Layout::Card, Layout::Split, Layout::BlurFit],
        motion: MotionMode::KenBurns,
        default_shot_secs: 2.4,
        sticker_theme: "hearts",
    },
    ReelTemplate {
        id: "romance_butterflies_rush",
        name: "Butterflies Rush",
        event: EventTheme::Romantic,
        description: "Snappy fast-cut montage of sweet romantic moments",
        palette: PaletteVibe::RubyRomantic,
        layouts: &[Layout::Heart, Layout::Fill, Layout::Star, Layout::Card],
        motion: MotionMode::FastForward,
        default_shot_secs: 1.6,
        sticker_theme: "hearts",
    },

    // --- PARTY / CELEBRATION (8 templates) ---
    ReelTemplate {
        id: "party_electro_rave",
        name: "Electro Hype Rave",
        event: EventTheme::Party,
        description: "Rapid-fire 1.2s cuts, multi-photo collages and diamond lasers",
        palette: PaletteVibe::CyberParty,
        layouts: &[Layout::Collage3, Layout::Diamond, Layout::Star, Layout::Split],
        motion: MotionMode::FastForward,
        default_shot_secs: 1.3,
        sticker_theme: "sparkles",
    },
    ReelTemplate {
        id: "party_vip_red_carpet",
        name: "VIP Red Carpet",
        event: EventTheme::Party,
        description: "Luxury gold diamonds, 3-photo collages and punchy bass bounce",
        palette: PaletteVibe::GoldLuxury,
        layouts: &[Layout::Diamond, Layout::Star, Layout::Collage3, Layout::Card],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 1.8,
        sticker_theme: "confetti",
    },
    ReelTemplate {
        id: "party_bass_drop",
        name: "Bass Drop Boomerang",
        event: EventTheme::Party,
        description: "Bouncing forward-and-back zooms synced to dance rhythm",
        palette: PaletteVibe::CyberParty,
        layouts: &[Layout::Collage3, Layout::Split, Layout::Diamond, Layout::Fill],
        motion: MotionMode::Boomerang,
        default_shot_secs: 1.5,
        sticker_theme: "sparkles",
    },
    ReelTemplate {
        id: "party_rooftop_sunset",
        name: "Rooftop Cocktails",
        event: EventTheme::Party,
        description: "Sunset orange gradients, split panoramas and chic cards",
        palette: PaletteVibe::NeonSunset,
        layouts: &[Layout::Split, Layout::Card, Layout::Diamond, Layout::BlurFit],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 2.0,
        sticker_theme: "confetti",
    },
    ReelTemplate {
        id: "party_laser_blast",
        name: "Laser Beam Festival",
        event: EventTheme::Party,
        description: "Maximum tempo montage cuts for festival and club footage",
        palette: PaletteVibe::CyberParty,
        layouts: &[Layout::Diamond, Layout::Star, Layout::Fill, Layout::Split],
        motion: MotionMode::FastForward,
        default_shot_secs: 1.2,
        sticker_theme: "sparkles",
    },
    ReelTemplate {
        id: "party_sparkler_countdown",
        name: "Midnight Countdown",
        event: EventTheme::Party,
        description: "Gold sparkle stars, multi-photo layouts and celebration rush",
        palette: PaletteVibe::GoldLuxury,
        layouts: &[Layout::Star, Layout::Collage3, Layout::Star, Layout::Card],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 1.7,
        sticker_theme: "sparkles",
    },
    ReelTemplate {
        id: "party_wild_carnival",
        name: "Wild Carnival Heat",
        event: EventTheme::Party,
        description: "Accelerated whip zooms and split screens with amber energy",
        palette: PaletteVibe::WarmSunlight,
        layouts: &[Layout::Fill, Layout::Split, Layout::Collage3, Layout::Diamond],
        motion: MotionMode::FastForward,
        default_shot_secs: 1.4,
        sticker_theme: "confetti",
    },
    ReelTemplate {
        id: "party_studio_disco",
        name: "Studio 54 Disco",
        event: EventTheme::Party,
        description: "Funky circle frames, diamond cards and bouncy transitions",
        palette: PaletteVibe::VelvetMidnight,
        layouts: &[Layout::Circle, Layout::Diamond, Layout::Collage3, Layout::Card],
        motion: MotionMode::Boomerang,
        default_shot_secs: 1.8,
        sticker_theme: "confetti",
    },

    // --- TRAVEL / ADVENTURE (8 templates) ---
    ReelTemplate {
        id: "travel_polaroid_wander",
        name: "Polaroid Wanderlust",
        event: EventTheme::Travel,
        description: "Classic retro polaroid cards with washi tape and postcard borders",
        palette: PaletteVibe::VintagePolaroid,
        layouts: &[Layout::Polaroid, Layout::Split, Layout::Card, Layout::BlurFit],
        motion: MotionMode::KenBurns,
        default_shot_secs: 2.4,
        sticker_theme: "polaroid",
    },
    ReelTemplate {
        id: "travel_turquoise_coast",
        name: "Turquoise Coast",
        event: EventTheme::Travel,
        description: "Azure blue accents, beach split-panoramas and polaroid snapshots",
        palette: PaletteVibe::TropicalAzure,
        layouts: &[Layout::Split, Layout::Polaroid, Layout::Card, Layout::Fill],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 2.0,
        sticker_theme: "polaroid",
    },
    ReelTemplate {
        id: "travel_highway_rush",
        name: "Highway Horizon Rush",
        event: EventTheme::Travel,
        description: "High-speed roadtrip montage with accelerating push zooms",
        palette: PaletteVibe::NeonSunset,
        layouts: &[Layout::Fill, Layout::Split, Layout::Card, Layout::Polaroid],
        motion: MotionMode::FastForward,
        default_shot_secs: 1.7,
        sticker_theme: "sparkles",
    },
    ReelTemplate {
        id: "travel_summit_peaks",
        name: "Summit Peaks Panorama",
        event: EventTheme::Travel,
        description: "Wide split-screen views and slow majestic camera glides",
        palette: PaletteVibe::EmeraldPrestige,
        layouts: &[Layout::Split, Layout::Card, Layout::Fill, Layout::BlurFit],
        motion: MotionMode::KenBurns,
        default_shot_secs: 2.6,
        sticker_theme: "minimal",
    },
    ReelTemplate {
        id: "travel_metropolis_vibe",
        name: "Metropolis Beat",
        event: EventTheme::Travel,
        description: "Urban architecture cards, polaroids and dynamic motion cuts",
        palette: PaletteVibe::VelvetMidnight,
        layouts: &[Layout::Polaroid, Layout::Split, Layout::Diamond, Layout::Card],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 1.9,
        sticker_theme: "polaroid",
    },
    ReelTemplate {
        id: "travel_sahara_sunset",
        name: "Sahara Golden Dunes",
        event: EventTheme::Travel,
        description: "Golden hour desert tones with gentle Boomerang zoom bounce",
        palette: PaletteVibe::WarmSunlight,
        layouts: &[Layout::Card, Layout::Polaroid, Layout::Split, Layout::BlurFit],
        motion: MotionMode::Boomerang,
        default_shot_secs: 2.2,
        sticker_theme: "polaroid",
    },
    ReelTemplate {
        id: "travel_island_hopping",
        name: "Island Hopping Diary",
        event: EventTheme::Travel,
        description: "Tropical card frames, circle water views and sunny pacing",
        palette: PaletteVibe::TropicalAzure,
        layouts: &[Layout::Polaroid, Layout::Card, Layout::Split, Layout::Circle],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 2.0,
        sticker_theme: "polaroid",
    },
    ReelTemplate {
        id: "travel_backpacker_post",
        name: "Backpacker Postcards",
        event: EventTheme::Travel,
        description: "Nostalgic double-polaroid montage with documentary pacing",
        palette: PaletteVibe::VintagePolaroid,
        layouts: &[Layout::Polaroid, Layout::Polaroid, Layout::Split, Layout::Card],
        motion: MotionMode::KenBurns,
        default_shot_secs: 2.5,
        sticker_theme: "polaroid",
    },

    // --- CINEMATIC / GENERAL (8 templates) ---
    ReelTemplate {
        id: "cine_minimal_luxe",
        name: "Minimalist Luxury",
        event: EventTheme::General,
        description: "Clean full-bleed cuts, rounded editorial cards and smooth flow",
        palette: PaletteVibe::EmeraldPrestige,
        layouts: &[Layout::Fill, Layout::Card, Layout::Circle, Layout::BlurFit],
        motion: MotionMode::KenBurns,
        default_shot_secs: 2.6,
        sticker_theme: "minimal",
    },
    ReelTemplate {
        id: "cine_vogue_editorial",
        name: "Vogue Editorial Cut",
        event: EventTheme::General,
        description: "High-fashion diamond masks, split screens and velvet tone",
        palette: PaletteVibe::VelvetMidnight,
        layouts: &[Layout::Card, Layout::Split, Layout::Diamond, Layout::Fill],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 2.0,
        sticker_theme: "minimal",
    },
    ReelTemplate {
        id: "cine_velocity_rhythm",
        name: "Velocity Creator Rhythm",
        event: EventTheme::General,
        description: "Snappy fast-forward zooms alternating with focus circles",
        palette: PaletteVibe::GoldLuxury,
        layouts: &[Layout::Fill, Layout::Circle, Layout::Split, Layout::Card],
        motion: MotionMode::FastForward,
        default_shot_secs: 1.6,
        sticker_theme: "sparkles",
    },
    ReelTemplate {
        id: "cine_golden_monochrome",
        name: "Warm Noir Cinema",
        event: EventTheme::General,
        description: "Atmospheric depth blur, smart-crop portraits and slow glide",
        palette: PaletteVibe::WarmSunlight,
        layouts: &[Layout::BlurFit, Layout::Card, Layout::Fill, Layout::Split],
        motion: MotionMode::KenBurns,
        default_shot_secs: 2.7,
        sticker_theme: "minimal",
    },
    ReelTemplate {
        id: "cine_ethereal_pulse",
        name: "Ethereal Pulse",
        event: EventTheme::General,
        description: "Hypnotic forward-and-back Boomerang zoom on focus points",
        palette: PaletteVibe::RoseElegance,
        layouts: &[Layout::Circle, Layout::Card, Layout::BlurFit, Layout::Heart],
        motion: MotionMode::Boomerang,
        default_shot_secs: 2.1,
        sticker_theme: "sparkles",
    },
    ReelTemplate {
        id: "cine_art_gallery",
        name: "Art Gallery Frames",
        event: EventTheme::General,
        description: "Curated white margins, card framing and slow museum pan",
        palette: PaletteVibe::VintagePolaroid,
        layouts: &[Layout::Card, Layout::Split, Layout::Card, Layout::BlurFit],
        motion: MotionMode::KenBurns,
        default_shot_secs: 2.4,
        sticker_theme: "minimal",
    },
    ReelTemplate {
        id: "cine_hyper_snap",
        name: "Hyperlapse Montage",
        event: EventTheme::General,
        description: "Accelerated pace with diamond and split geometric cuts",
        palette: PaletteVibe::CyberParty,
        layouts: &[Layout::Fill, Layout::Split, Layout::Diamond, Layout::Card],
        motion: MotionMode::FastForward,
        default_shot_secs: 1.4,
        sticker_theme: "sparkles",
    },
    ReelTemplate {
        id: "cine_zenith_flow",
        name: "Zenith Clean Cut",
        event: EventTheme::General,
        description: "Perfect balance of full-bleed portraits, circles and modern motion",
        palette: PaletteVibe::TropicalAzure,
        layouts: &[Layout::Circle, Layout::BlurFit, Layout::Card, Layout::Fill],
        motion: MotionMode::DynamicMix,
        default_shot_secs: 2.2,
        sticker_theme: "sparkles",
    },
];

/// Smart analysis & scoring engine: Ranks templates by image attributes, event, and title
pub fn score_templates(
    event: EventTheme,
    photos: &[Analysis],
    title: &str,
) -> Vec<ScoredTemplate> {
    let lower_title = title.to_lowercase();
    let num_photos = photos.len();

    // 1. Aspect ratio statistics
    let landscape_count = photos.iter().filter(|p| p.is_landscape()).count();
    let portrait_count = num_photos.saturating_sub(landscape_count);
    let mostly_landscape = landscape_count > portrait_count;

    // 2. Average brightness & sharpness
    let avg_brightness = if num_photos > 0 {
        photos.iter().map(|p| p.brightness).sum::<f32>() / num_photos as f32
    } else {
        0.5
    };

    let avg_sharpness = if num_photos > 0 {
        photos.iter().map(|p| p.sharpness).sum::<f32>() / num_photos as f32
    } else {
        50.0
    };

    let mut scored: Vec<ScoredTemplate> = TEMPLATES
        .iter()
        .map(|t| {
            let mut score: i32 = 50;
            let mut reasons = Vec::new();

            // Event affinity (strongest match)
            if t.event == event {
                score += 35;
                reasons.push("Perfect occasion match");
            } else if (event == EventTheme::Anniversary && t.event == EventTheme::Romantic)
                || (event == EventTheme::Romantic && t.event == EventTheme::Anniversary)
                || (event == EventTheme::Birthday && t.event == EventTheme::Party)
                || (event == EventTheme::Party && t.event == EventTheme::Birthday)
            {
                score += 18;
                reasons.push("Complementary event vibe");
            }

            // Aspect ratio bonus
            if mostly_landscape && t.layouts.contains(&Layout::Split) {
                score += 12;
                reasons.push("Optimized for your landscape shots");
            } else if !mostly_landscape && (t.layouts.contains(&Layout::Heart) || t.layouts.contains(&Layout::Card)) {
                score += 10;
                reasons.push("Frames your portrait photos naturally");
            }

            // Brightness / Color harmony match
            if avg_brightness > 0.58 && matches!(t.palette, PaletteVibe::WarmSunlight | PaletteVibe::GoldLuxury | PaletteVibe::RoseElegance) {
                score += 8;
                reasons.push("Harmonious with your bright & warm images");
            } else if avg_brightness < 0.46 && matches!(t.palette, PaletteVibe::CyberParty | PaletteVibe::VelvetMidnight) {
                score += 8;
                reasons.push("Complements your rich moody contrast");
            }

            // Photo count & pacing bonus
            if num_photos >= 8 && t.motion == MotionMode::FastForward {
                score += 7;
                reasons.push("Snappy pace for large photo batch");
            } else if num_photos <= 5 && (t.motion == MotionMode::Boomerang || t.motion == MotionMode::KenBurns) {
                score += 7;
                reasons.push("Deep visual retention for concise story");
            }

            // Title keyword sentiment
            if lower_title.contains("love") || lower_title.contains("forever") || lower_title.contains("heart") {
                if t.layouts.contains(&Layout::Heart) {
                    score += 9;
                    reasons.push("Heart theme matches your title");
                }
            }
            if lower_title.contains("party") || lower_title.contains("night") || lower_title.contains("rave") {
                if t.event == EventTheme::Party || t.motion == MotionMode::FastForward {
                    score += 9;
                    reasons.push("High energy aligns with title");
                }
            }
            if lower_title.contains("trip") || lower_title.contains("travel") || lower_title.contains("vacation") {
                if t.layouts.contains(&Layout::Polaroid) {
                    score += 9;
                    reasons.push("Polaroid matches your travel title");
                }
            }

            // High sharpness opener bonus
            if avg_sharpness > 70.0 && t.layouts.first() == Some(&Layout::Fill) {
                score += 5;
                reasons.push("Hero sharp hook opener");
            }

            let normalized = score.clamp(60, 99) as usize;
            let reason_str = if reasons.is_empty() {
                "Balanced creative flow".to_string()
            } else {
                reasons.join(" • ")
            };

            ScoredTemplate {
                template: t.clone(),
                match_score: normalized,
                match_reason: reason_str,
            }
        })
        .collect();

    // Sort descending by match score
    scored.sort_by(|a, b| b.match_score.cmp(&a.match_score));
    scored
}

pub fn find_template(id: &str) -> Option<&'static ReelTemplate> {
    TEMPLATES.iter().find(|t| t.id == id)
}

/// Split title into two lines: Line 1 (Event / Prefix) and Line 2 (Name / Subject in accent color)
pub fn split_title(raw: &str, event: EventTheme) -> (String, String) {
    let clean = raw.trim();
    if clean.is_empty() {
        return match event {
            EventTheme::Birthday => ("HAPPY BIRTHDAY".to_string(), "CELEBRATION 🎉".to_string()),
            EventTheme::Anniversary => ("HAPPY ANNIVERSARY".to_string(), "TOGETHER FOREVER ❤️".to_string()),
            EventTheme::Romantic => ("FOREVER & ALWAYS".to_string(), "MY SWEET LOVE 💖".to_string()),
            EventTheme::Party => ("WEEKEND VIBES".to_string(), "PARTY TIME 🍾".to_string()),
            EventTheme::Travel => ("WANDERLUST".to_string(), "TRAVEL MEMORIES ✈️".to_string()),
            EventTheme::General => ("SPECIAL MOMENTS".to_string(), "MEMORIES ✨".to_string()),
        };
    }

    let lower = clean.to_lowercase();
    for prefix in &[
        "happy birthday",
        "happy anniversary",
        "happy bday",
        "forever and always",
        "forever & always",
        "celebrating",
        "trip to",
        "vacation in",
        "memories of",
    ] {
        if lower.starts_with(prefix) {
            let p_len = prefix.len();
            let rem = clean[p_len..]
                .trim()
                .trim_start_matches(|c: char| c == ':' || c == '-' || c == ',' || c == '!');
            if !rem.is_empty() {
                return (prefix.to_uppercase(), rem.trim().to_uppercase());
            }
        }
    }

    // Split by delimiter
    if let Some(pos) = clean.find(|c| c == '-' || c == ':' || c == '|' || c == ',') {
        let left = clean[..pos].trim();
        let right = clean[pos + 1..].trim();
        if !left.is_empty() && !right.is_empty() {
            return (left.to_uppercase(), right.to_uppercase());
        }
    }

    // Split words
    let words: Vec<&str> = clean.split_whitespace().collect();
    if words.len() >= 3 {
        let line1 = words[0..2].join(" ");
        let line2 = words[2..].join(" ");
        (line1.to_uppercase(), line2.to_uppercase())
    } else if words.len() == 2 {
        (words[0].to_uppercase(), words[1].to_uppercase())
    } else {
        ("MOMENTS".to_string(), clean.to_uppercase())
    }
}
