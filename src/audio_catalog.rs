use crate::types::{EventTheme, MotionMode};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub era: String,         // "pre_1980", "1990_2000", "2000_2015", "current_2026"
    pub era_label: String,   // "Pre-1980s", "1990s–2000s", "2000s–2015s", "Current 2026"
    pub genre: String,       // "Soul / Funk", "Eurodance", "Lo-Fi Chill", etc.
    pub bpm: u32,
    pub duration: f32,
    pub file_name: String,
    pub stream_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_reason: Option<String>,
}

pub struct TrackDef {
    pub id: &'static str,
    pub title: &'static str,
    pub artist: &'static str,
    pub era: &'static str,
    pub era_label: &'static str,
    pub genre: &'static str,
    pub bpm: u32,
    pub duration: f32,
    pub events: &'static [EventTheme],
    pub synth_style: u8, // 0: Disco Funk, 1: Bossa Nova, 2: Motown Soul, 3: Eurodance, 4: 90s R&B, 5: Y2K Pop, 6: Tropical House, 7: Indie Rock, 8: Electro, 9: Drift Phonk, 10: Lo-Fi Chill, 11: Future Bass
}

pub const TRACKS: &[TrackDef] = &[
    // --- Pre-1980s: Kishore Kumar, Lata Mangeshkar, Mohammed Rafi, Asha Bhosle ---
    TrackDef {
        id: "kishore_pal_pal",
        title: "Pal Pal Dil Ke Paas",
        artist: "Kishore Kumar",
        era: "pre_1980",
        era_label: "Pre-1980s (Golden Classics)",
        genre: "Romantic Classic",
        bpm: 98,
        duration: 30.0,
        events: &[EventTheme::Romantic, EventTheme::Anniversary, EventTheme::General],
        synth_style: 1,
    },
    TrackDef {
        id: "kishore_o_mere_dil",
        title: "O Mere Dil Ke Chain",
        artist: "Kishore Kumar",
        era: "pre_1980",
        era_label: "Pre-1980s (Golden Classics)",
        genre: "Soulful Classic",
        bpm: 104,
        duration: 30.0,
        events: &[EventTheme::Romantic, EventTheme::Anniversary, EventTheme::Travel],
        synth_style: 2,
    },
    TrackDef {
        id: "kishore_zindagi_safar",
        title: "Zindagi Ek Safar Hai Suhana",
        artist: "Kishore Kumar",
        era: "pre_1980",
        era_label: "Pre-1980s (Golden Classics)",
        genre: "Upbeat Adventure",
        bpm: 124,
        duration: 30.0,
        events: &[EventTheme::Travel, EventTheme::Birthday, EventTheme::Party, EventTheme::Graduation],
        synth_style: 0,
    },
    TrackDef {
        id: "lata_lag_jaa_gale",
        title: "Lag Jaa Gale",
        artist: "Lata Mangeshkar",
        era: "pre_1980",
        era_label: "Pre-1980s (Golden Classics)",
        genre: "Evergreen Melody",
        bpm: 84,
        duration: 30.0,
        events: &[EventTheme::Romantic, EventTheme::Anniversary, EventTheme::General],
        synth_style: 10,
    },
    TrackDef {
        id: "lata_aap_ki_nazron",
        title: "Aap Ki Nazron Ne Samjha",
        artist: "Lata Mangeshkar",
        era: "pre_1980",
        era_label: "Pre-1980s (Golden Classics)",
        genre: "Vintage Elegance",
        bpm: 92,
        duration: 30.0,
        events: &[EventTheme::Anniversary, EventTheme::Romantic, EventTheme::Fashion],
        synth_style: 1,
    },
    TrackDef {
        id: "rafi_chura_liya",
        title: "Chura Liya Hai Tumne Jo Dil Ko",
        artist: "Mohammed Rafi & Asha Bhosle",
        era: "pre_1980",
        era_label: "Pre-1980s (Golden Classics)",
        genre: "Acoustic Pop Romance",
        bpm: 112,
        duration: 30.0,
        events: &[EventTheme::Romantic, EventTheme::Birthday, EventTheme::Party],
        synth_style: 0,
    },
    TrackDef {
        id: "rafi_gulabi_aankhen",
        title: "Gulabi Aankhen Jo Teri Dekhi",
        artist: "Mohammed Rafi",
        era: "pre_1980",
        era_label: "Pre-1980s (Golden Classics)",
        genre: "Celebration Dance",
        bpm: 128,
        duration: 30.0,
        events: &[EventTheme::Party, EventTheme::Birthday, EventTheme::Fitness],
        synth_style: 3,
    },
    TrackDef {
        id: "asha_dum_maro_dum",
        title: "Dum Maro Dum",
        artist: "Asha Bhosle",
        era: "pre_1980",
        era_label: "Pre-1980s (Golden Classics)",
        genre: "Psychedelic Rock",
        bpm: 116,
        duration: 30.0,
        events: &[EventTheme::Party, EventTheme::Fashion, EventTheme::Fitness],
        synth_style: 8,
    },

    // --- 1980s & 1990s: KK, A.R. Rahman, Lata Mangeshkar, Eurodance ---
    TrackDef {
        id: "lata_tujhe_dekha",
        title: "Tujhe Dekha Toh Yeh Jaana Sanam",
        artist: "Lata Mangeshkar & Kumar Sanu",
        era: "1990_2000",
        era_label: "1990s–2000s (90s Romance)",
        genre: "Bollywood Romance",
        bpm: 100,
        duration: 30.0,
        events: &[EventTheme::Romantic, EventTheme::Anniversary, EventTheme::Travel],
        synth_style: 4,
    },
    TrackDef {
        id: "kk_pal",
        title: "Pal (Yaad Aayenge Yeh Pal)",
        artist: "KK",
        era: "1990_2000",
        era_label: "1990s–2000s (Nostalgia)",
        genre: "Soulful Nostalgia",
        bpm: 88,
        duration: 30.0,
        events: &[EventTheme::Graduation, EventTheme::Travel, EventTheme::Pets, EventTheme::General],
        synth_style: 7,
    },
    TrackDef {
        id: "kk_yaaron",
        title: "Yaaron Dosti",
        artist: "KK",
        era: "1990_2000",
        era_label: "1990s–2000s (Nostalgia)",
        genre: "Friendship Anthem",
        bpm: 120,
        duration: 30.0,
        events: &[EventTheme::Party, EventTheme::Graduation, EventTheme::Birthday, EventTheme::Travel],
        synth_style: 5,
    },
    TrackDef {
        id: "ar_rahman_dil_se",
        title: "Dil Se Re",
        artist: "A.R. Rahman",
        era: "1990_2000",
        era_label: "1990s–2000s (Cinematic)",
        genre: "Cinematic Passion",
        bpm: 106,
        duration: 30.0,
        events: &[EventTheme::General, EventTheme::Fashion, EventTheme::Travel],
        synth_style: 4,
    },
    TrackDef {
        id: "retro_90s_eurodance",
        title: "90s Eurodance Euphoria",
        artist: "Club Millennium X",
        era: "1990_2000",
        era_label: "1990s–2000s (Retro Dance)",
        genre: "Eurodance & Pop",
        bpm: 134,
        duration: 30.0,
        events: &[EventTheme::Party, EventTheme::Fitness, EventTheme::Birthday],
        synth_style: 3,
    },

    // --- 2000s–2015s: Arijit Singh, Shreya Ghoshal, Sonu Nigam, A.R. Rahman, Coldplay ---
    TrackDef {
        id: "arijit_tum_hi_ho",
        title: "Tum Hi Ho",
        artist: "Arijit Singh",
        era: "2000_2015",
        era_label: "2000s–2015s (Bollywood Melodies)",
        genre: "Heartfelt Ballad",
        bpm: 86,
        duration: 30.0,
        events: &[EventTheme::Romantic, EventTheme::Anniversary, EventTheme::General],
        synth_style: 10,
    },
    TrackDef {
        id: "shreya_teri_ore",
        title: "Teri Ore",
        artist: "Shreya Ghoshal",
        era: "2000_2015",
        era_label: "2000s–2015s (Bollywood Melodies)",
        genre: "Sufi Romance",
        bpm: 98,
        duration: 30.0,
        events: &[EventTheme::Romantic, EventTheme::Anniversary, EventTheme::Fashion],
        synth_style: 1,
    },
    TrackDef {
        id: "sonu_kal_ho_naa_ho",
        title: "Kal Ho Naa Ho",
        artist: "Sonu Nigam",
        era: "2000_2015",
        era_label: "2000s–2015s (Bollywood Melodies)",
        genre: "Cinematic Life Anthem",
        bpm: 84,
        duration: 30.0,
        events: &[EventTheme::General, EventTheme::Graduation, EventTheme::Birthday],
        synth_style: 10,
    },
    TrackDef {
        id: "ar_rahman_jai_ho",
        title: "Jai Ho (Victory Anthem)",
        artist: "A.R. Rahman",
        era: "2000_2015",
        era_label: "2000s–2015s (Global Anthems)",
        genre: "High-Energy Anthem",
        bpm: 132,
        duration: 30.0,
        events: &[EventTheme::Graduation, EventTheme::Party, EventTheme::Fitness, EventTheme::Business],
        synth_style: 11,
    },
    TrackDef {
        id: "coldplay_viva_la_vida",
        title: "Viva La Vida",
        artist: "Coldplay",
        era: "2000_2015",
        era_label: "2000s–2015s (Global Anthems)",
        genre: "Orchestral Pop",
        bpm: 138,
        duration: 30.0,
        events: &[EventTheme::Travel, EventTheme::Graduation, EventTheme::General],
        synth_style: 7,
    },
    TrackDef {
        id: "era2010_tropical_house",
        title: "Ibiza Tropical Sunset",
        artist: "Azure Breeze",
        era: "2000_2015",
        era_label: "2000s–2015s (Tropical Era)",
        genre: "Tropical House",
        bpm: 120,
        duration: 30.0,
        events: &[EventTheme::Travel, EventTheme::Party, EventTheme::Food, EventTheme::Fashion],
        synth_style: 6,
    },

    // --- Current 2026: Arijit Singh, Taylor Swift, The Weeknd, Trending Phonk & Lo-Fi ---
    TrackDef {
        id: "arijit_kesariya",
        title: "Kesariya",
        artist: "Arijit Singh",
        era: "current_2026",
        era_label: "Current 2026 (Trending Now)",
        genre: "Modern Love Pop",
        bpm: 100,
        duration: 30.0,
        events: &[EventTheme::Romantic, EventTheme::Anniversary, EventTheme::Travel],
        synth_style: 6,
    },
    TrackDef {
        id: "taylor_cruel_summer",
        title: "Cruel Summer",
        artist: "Taylor Swift",
        era: "current_2026",
        era_label: "Current 2026 (Viral Pop)",
        genre: "Synth Pop",
        bpm: 170,
        duration: 30.0,
        events: &[EventTheme::Party, EventTheme::Travel, EventTheme::Birthday, EventTheme::Fashion],
        synth_style: 11,
    },
    TrackDef {
        id: "weeknd_blinding_lights",
        title: "Blinding Lights",
        artist: "The Weeknd",
        era: "current_2026",
        era_label: "Current 2026 (Viral Pop)",
        genre: "Retro Synthwave",
        bpm: 171,
        duration: 30.0,
        events: &[EventTheme::Party, EventTheme::Fitness, EventTheme::Fashion],
        synth_style: 8,
    },
    TrackDef {
        id: "curr2026_drift_phonk",
        title: "2026 Drift Phonk Anthem",
        artist: "Tokyo Cyber Drift",
        era: "current_2026",
        era_label: "Current 2026 (Viral FYP)",
        genre: "Drift Phonk",
        bpm: 138,
        duration: 30.0,
        events: &[EventTheme::Fitness, EventTheme::Party, EventTheme::Fashion],
        synth_style: 9,
    },
    TrackDef {
        id: "curr2026_lofi_coffee",
        title: "Midnight Lo-Fi Chillhop",
        artist: "Cozy Study Waves",
        era: "current_2026",
        era_label: "Current 2026 (Aesthetic Lo-Fi)",
        genre: "Lo-Fi Chill",
        bpm: 84,
        duration: 30.0,
        events: &[EventTheme::Romantic, EventTheme::Food, EventTheme::Pets, EventTheme::Anniversary, EventTheme::General],
        synth_style: 10,
    },
];

/// List all tracks with optional era, genre, and artist filters, scored for the active event
pub fn list_tracks(
    era_filter: Option<&str>,
    genre_filter: Option<&str>,
    artist_filter: Option<&str>,
    event: Option<EventTheme>,
) -> Vec<MusicTrack> {
    TRACKS
        .iter()
        .filter(|t| {
            if let Some(era) = era_filter {
                if !era.is_empty() && era != "all" && t.era != era {
                    return false;
                }
            }
            if let Some(genre) = genre_filter {
                if !genre.is_empty() && genre != "all" && !t.genre.to_lowercase().contains(&genre.to_lowercase()) {
                    return false;
                }
            }
            if let Some(art) = artist_filter {
                if !art.is_empty() && art != "all" && !t.artist.to_lowercase().contains(&art.to_lowercase()) {
                    return false;
                }
            }
            true
        })
        .map(|t| {
            let (score, reason) = if let Some(ev) = event {
                score_track_for_event(t, ev)
            } else {
                (85, "Popular choice across all vertical reels".to_string())
            };

            MusicTrack {
                id: t.id.to_string(),
                title: t.title.to_string(),
                artist: t.artist.to_string(),
                era: t.era.to_string(),
                era_label: t.era_label.to_string(),
                genre: t.genre.to_string(),
                bpm: t.bpm,
                duration: t.duration,
                file_name: format!("{}.wav", t.id),
                stream_url: format!("/api/music/stream/{}.wav", t.id),
                match_score: Some(score),
                match_reason: Some(reason),
            }
        })
        .collect()
}

/// Score a track against an event
fn score_track_for_event(track: &TrackDef, event: EventTheme) -> (u32, String) {
    let mut score = 70u32;
    let is_direct_event_match = track.events.contains(&event);

    if is_direct_event_match {
        score += 24;
    }

    match event {
        EventTheme::Fitness => {
            if track.bpm >= 128 {
                score += 5;
            }
        }
        EventTheme::Romantic | EventTheme::Anniversary => {
            if track.bpm <= 105 {
                score += 5;
            }
        }
        EventTheme::Party => {
            if track.bpm >= 125 {
                score += 5;
            }
        }
        _ => {}
    }

    score = score.min(99);

    let reason = if is_direct_event_match {
        format!("Best synergy with {} occasion ({} BPM sync)", event.as_str(), track.bpm)
    } else {
        format!("Smooth {} backing track for vertical format", track.genre)
    };

    (score, reason)
}

/// Suggest the single best track based on event, motion, and photo count without external APIs
pub fn suggest_best_track(
    event: EventTheme,
    _photos_count: usize,
    motion: MotionMode,
    era_filter: Option<&str>,
) -> MusicTrack {
    let mut candidates: Vec<(&TrackDef, u32, String)> = TRACKS
        .iter()
        .filter(|t| {
            if let Some(era) = era_filter {
                if !era.is_empty() && era != "all" && t.era != era {
                    return false;
                }
            }
            true
        })
        .map(|t| {
            let (mut score, mut reason) = score_track_for_event(t, event);

            // Boost based on motion compatibility
            match motion {
                MotionMode::FastForward | MotionMode::BeatDrop | MotionMode::TikTokSnap => {
                    if t.bpm >= 128 {
                        score += 3;
                        reason = format!("High BPM ({} BPM) perfectly accents your energetic cuts", t.bpm);
                    }
                }
                MotionMode::KenBurns | MotionMode::WaveFlow => {
                    if t.bpm <= 110 {
                        score += 3;
                        reason = format!("Gentle tempo ({} BPM) complements smooth cinematic glides", t.bpm);
                    }
                }
                MotionMode::Boomerang => {
                    if t.bpm >= 115 && t.bpm <= 130 {
                        score += 3;
                        reason = format!("Upbeat rhythm ({} BPM) syncs with bounce pulse transitions", t.bpm);
                    }
                }
                _ => {}
            }

            (t, score.min(99), reason)
        })
        .collect();

    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    let (best, score, reason) = candidates.first().cloned().unwrap_or((&TRACKS[0], 90, "Standard balance".into()));

    MusicTrack {
        id: best.id.to_string(),
        title: best.title.to_string(),
        artist: best.artist.to_string(),
        era: best.era.to_string(),
        era_label: best.era_label.to_string(),
        genre: best.genre.to_string(),
        bpm: best.bpm,
        duration: best.duration,
        file_name: format!("{}.wav", best.id),
        stream_url: format!("/api/music/stream/{}.wav", best.id),
        match_score: Some(score),
        match_reason: Some(reason),
    }
}

/// Find a track path in the music library by ID or filename
pub fn find_track_path(music_dir: &Path, id_or_filename: &str) -> Option<PathBuf> {
    let direct = music_dir.join(id_or_filename.trim());
    if direct.exists() && direct.is_file() {
        return Some(direct);
    }
    let clean = id_or_filename.trim().trim_end_matches(".wav").trim_end_matches(".mp3").trim_end_matches(".m4a").trim_end_matches(".aac");
    for ext in &["mp3", "wav", "m4a", "aac"] {
        let p = music_dir.join(format!("{clean}.{ext}"));
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Ensure all built-in royalty-free tracks exist in `reelforge_data/music/`, synthesizing them if needed
pub fn ensure_library(music_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(music_dir)?;

    for t in TRACKS {
        let wav_path = music_dir.join(format!("{}.wav", t.id));
        if !wav_path.exists() {
            println!("  [Music Synthesis] Synthesizing '{}' ({} BPM, Era: {})...", t.title, t.bpm, t.era_label);
            synthesize_wav(t, &wav_path)?;
        }
    }
    Ok(())
}

/// Pure Rust mathematical audio synthesis: writes a 16-bit 44.1kHz stereo WAV file
fn synthesize_wav(track: &TrackDef, out_path: &Path) -> Result<()> {
    let sample_rate = 44100u32;
    let duration = track.duration;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let bpm = track.bpm as f32;
    let beat_len = 60.0 / bpm; // duration of 1 beat in seconds
    let sixteenth_len = beat_len / 4.0;

    // Chord progressions depending on style
    let chord_roots: &[f32] = match track.synth_style {
        0 => &[130.81, 146.83, 164.81, 174.61], // Funk C - D - E - F
        1 => &[130.81, 116.54, 110.00, 103.83], // Bossa Cmaj7 - Bb7 - Am7 - Ab7
        2 => &[130.81, 174.61, 146.83, 196.00], // Motown C - F - Dm - G
        3 => &[110.00, 130.81, 146.83, 164.81], // Eurodance Am - C - Dm - Em
        4 => &[130.81, 110.00, 146.83, 196.00], // 90s R&B C - Am - Dm - G
        5 => &[130.81, 196.00, 110.00, 174.61], // Y2K Pop C - G - Am - F
        6 => &[110.00, 174.61, 130.81, 196.00], // Tropical Am - F - C - G
        7 => &[130.81, 174.61, 110.00, 196.00], // Indie C - F - Am - G
        8 => &[110.00, 110.00, 130.81, 146.83], // Electro Am - Am - C - Dm
        9 => &[116.54, 116.54, 110.00, 103.83], // Phonk Bb - Bb - A - Ab
        10 => &[130.81, 123.47, 110.00, 98.00], // Lo-Fi Cmaj7 - Bm7 - Am7 - G
        _ => &[110.00, 174.61, 130.81, 196.00],  // Future Bass Am - F - C - G
    };

    let mut samples_left = Vec::with_capacity(num_samples);
    let mut samples_right = Vec::with_capacity(num_samples);

    let mut seed: u32 = 123456789;
    let mut next_rand = || -> f32 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed as f32 / u32::MAX as f32) * 2.0 - 1.0
    };

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let measure = (t / (beat_len * 4.0)) as usize;
        let root = chord_roots[measure % chord_roots.len()];

        // 1. Kick Drum (on beats 0, 1, 2, 3 or 0 & 2)
        let beat_time = t % beat_len;
        let mut kick = 0.0f32;
        let is_four_on_floor = matches!(track.synth_style, 0 | 3 | 6 | 8 | 9);
        let kick_trigger = if is_four_on_floor {
            true
        } else {
            // Beat 1 and 3
            let b_idx = ((t / beat_len) as usize) % 4;
            b_idx == 0 || b_idx == 2
        };

        if kick_trigger && beat_time < 0.18 {
            let env = (1.0 - beat_time / 0.18).powi(3);
            let freq = 45.0 + 110.0 * env;
            kick = (t * freq * 2.0 * PI).sin() * env * 0.45;
        }

        // 2. Snare / Clap (on beats 1 & 3 in 0-indexed: 2nd and 4th beats)
        let beat_idx = ((t / beat_len) as usize) % 4;
        let mut snare = 0.0f32;
        if (beat_idx == 1 || beat_idx == 3) && beat_time < 0.16 {
            let env = (1.0 - beat_time / 0.16).powi(2);
            let tone = (t * 220.0 * 2.0 * PI).sin() * 0.15;
            let noise = next_rand() * 0.22;
            snare = (tone + noise) * env;
        }

        // 3. Hi-Hat (on every sixteenth or 8th note)
        let sixteenth_time = t % sixteenth_len;
        let mut hihat = 0.0f32;
        if sixteenth_time < 0.045 {
            let env = (1.0 - sixteenth_time / 0.045).powi(2);
            hihat = next_rand() * 0.08 * env;
        }

        // 4. Bassline
        let bass_octave = root * 0.5;
        let bass_env = 0.6 + 0.4 * (t * bpm / 60.0 * PI).sin().abs();
        let bass = (t * bass_octave * 2.0 * PI).sin() * 0.22 * bass_env;

        // 5. Chord Pad / Harmonic Body
        let chord_3rd = root * 1.2599; // major/neutral 3rd
        let chord_5th = root * 1.4983; // 5th
        let chord_7th = root * 1.8877; // 7th
        let pad = ((t * root * 2.0 * PI).sin()
            + (t * chord_3rd * 2.0 * PI).sin() * 0.8
            + (t * chord_5th * 2.0 * PI).sin() * 0.6
            + (t * chord_7th * 2.0 * PI).sin() * 0.4)
            * 0.06;

        // 6. Lead Arpeggio / Melodic Pulse
        let arp_step = ((t / sixteenth_len) as usize) % 4;
        let arp_mult = match arp_step {
            0 => 2.0,
            1 => 2.5,
            2 => 3.0,
            _ => 3.5,
        };
        let arp_freq = root * arp_mult;
        let arp_env = (1.0 - (sixteenth_time / sixteenth_len)).max(0.0).powi(2);
        let lead = (t * arp_freq * 2.0 * PI).sin() * 0.09 * arp_env;

        // Master sum
        let master = kick + snare + hihat + bass + pad + lead;

        // Subtle stereo panning on lead & hi-hat
        let l = (master + lead * 0.04 + hihat * 0.02).clamp(-1.0, 1.0);
        let r = (master - lead * 0.04 - hihat * 0.02).clamp(-1.0, 1.0);

        samples_left.push(l);
        samples_right.push(r);
    }

    // Write standard 16-bit Stereo PCM WAV
    write_wav_file(out_path, sample_rate, &samples_left, &samples_right)?;
    Ok(())
}

fn write_wav_file(path: &Path, sample_rate: u32, left: &[f32], right: &[f32]) -> Result<()> {
    let mut file = File::create(path)?;
    let num_channels: u16 = 2;
    let bits_per_sample: u16 = 16;
    let bytes_per_sample = (bits_per_sample / 8) as u32;
    let num_samples = left.len().min(right.len()) as u32;
    let data_size = num_samples * num_channels as u32 * bytes_per_sample;
    let byte_rate = sample_rate * num_channels as u32 * bytes_per_sample;
    let block_align = num_channels * bits_per_sample / 8;

    // RIFF Header
    file.write_all(b"RIFF")?;
    file.write_all(&(36u32 + data_size).to_le_bytes())?;
    file.write_all(b"WAVE")?;

    // fmt subchunk
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // Subchunk1Size (16 for PCM)
    file.write_all(&1u16.to_le_bytes())?;  // AudioFormat (1 = PCM)
    file.write_all(&num_channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;

    // data subchunk
    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;

    for i in 0..num_samples as usize {
        let sl = (left[i] * 32760.0).clamp(-32760.0, 32760.0) as i16;
        let sr = (right[i] * 32760.0).clamp(-32760.0, 32760.0) as i16;
        file.write_all(&sl.to_le_bytes())?;
        file.write_all(&sr.to_le_bytes())?;
    }

    file.flush()?;
    Ok(())
}
