# ReelForge — Photos In, Vertical Reel Out (Rust) 🎬✨

A high-performance Rust pipeline and Creator Studio for transforming photos into high-engagement **9:16 vertical reels** for Instagram, YouTube Shorts, and TikTok.

Featuring smart-crop focus detection, anti-aliased event shape masks (Hearts, Stars, Collages, Polaroids), dynamic Boomerang pulse motion, Fast-Forward whip zooms, and FFmpeg/MLT rendering.

---

## 🌟 Key Features

- **Smart Analysis & Hook Optimization**:
  - Sobel edge-energy focus point detection and brightness analysis.
  - Automatically identifies the sharpest photo and uses it as the opening hook.
  - EXIF orientation-aware decoding for mobile photos.

- **Event Themes & Custom Occasions**:
  - 🎂 **Birthday**: 3-photo celebratory collages, sparkle star shapes, party confetti particles, and birthday banner badges (`🎂 Happy Birthday! 🎉`).
  - 💍 **Anniversary & Romance**: Smooth anti-aliased Heart masks (`Shape::Heart`), soft glowing rose borders, floating love hearts stickers (`❤️`), and romantic badges (`❤️ Happy Anniversary ✨`).
  - 🎉 **Party / Club**: Multi-cut collage montages, rapid star & diamond masks, and celebration vibes.
  - ✈️ **Travel / Adventure**: Retro Polaroid cards with paper borders and washi tape accents.
  - ✨ **Cinematic**: Full-bleed smart crops, circles, and minimalist modern motion.

- **Dynamic Motion Effects**:
  - 🔄 **Boomerang Pulse**: Playful bounce zoom-in and zoom-out on every photo (`1.0 + 0.22 · sin(π · frame / d)`).
  - ⏩ **Fast-Forward Montage**: Snappy, accelerating rush zooms with snappy transitions.
  - ⚡ **Dynamic Mix**: Smartly combines Boomerang, Fast-Forward, and Ken Burns cuts for maximum viewer retention.
  - 🎬 **Smooth Cinematic**: Slow classic Ken Burns camera glides.

- **Creator Studio Web UI**:
  - Non-AI-ish, modern creator studio interface with dark mode aesthetics.
  - Drag-and-drop upload zone with real-time thumbnail gallery preview and photo count.
  - Interactive occasion cards, custom title greetings, stickers toggle, and music beat sync.
  - Embedded vertical video player with one-click MP4 download.

- **Rendering Engines**:
  - **FFmpeg**: Direct x264 hardware/software encoding with `filter_complex` (zoompan + xfade transitions + audio sync + emoji banners).
  - **MLT / Melt**: Export MLT XML projects for further editing in **Kdenlive** or Shotcut.

---

## 🚀 Quick Start

### Prerequisites
- **Rust 1.80+** (`rustup`)
- **FFmpeg** installed and on `PATH`

### Run the Web Creator Studio
```bash
cargo run -- serve --port 8080
```
Open **[http://localhost:8080](http://localhost:8080)** in your browser!

### CLI Commands

```bash
# Generate a reel from a folder of photos
cargo run --release -- make -i ./photos -o reel.mp4 --shot-secs 2.2

# Generate a Birthday Reel with music and beat sync
cargo run --release -- make -i ./photos -o birthday_reel.mp4 --event birthday --motion dynamic-mix --music song.mp3 --bpm 120

# Generate an Anniversary Reel with custom greeting
cargo run --release -- make -i ./photos -o anniversary_reel.mp4 --event anniversary --motion boomerang --title "Happy 5th Anniversary! ❤️"

# Export MLT project for Kdenlive
cargo run --release -- make -i ./photos --engine mlt-only
```

---

## 📂 Architecture

```
src/
├── analyze.rs        # EXIF decoding, Sobel energy, sharpness & focus point detection
├── compose.rs        # Layout planner, heart/star SDF masks, collages, stickers & badges
├── render_ffmpeg.rs  # FFmpeg filter_complex (zoompan, boomerang, xfade & emoji text)
├── render_mlt.rs     # MLT XML writer + melt runner for Kdenlive timeline
├── pipeline.rs       # Orchestration & BPM-to-shot-length mapping
├── server.rs         # Axum web server & multipart upload endpoint
├── types.rs          # Core data models, event themes, motion modes & layouts
└── main.rs           # Clap CLI entry point (make / serve)
web/
└── index.html        # Modern Creator Studio web UI with live preview & video player
```

---

## 📜 License
MIT License
