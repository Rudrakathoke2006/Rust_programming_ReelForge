use crate::analyze::{load, smart_crop};
use crate::types::{Analysis, BannerStyle, Config, EventTheme, Layout, MotionMode, Shot, ShotMotion};
use anyhow::Result;
use image::{imageops, imageops::FilterType, DynamicImage, Rgba, RgbaImage};
use std::path::PathBuf;

/// Pick a layout per shot for visual variety, and pair up photos for collages / split-screens.
pub fn plan(cfg: &Config, photos: &[Analysis]) -> Vec<(Layout, Vec<usize>, ShotMotion)> {
    let mut order: Vec<usize> = (0..photos.len()).collect();
    if let Some((best, _)) = photos
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.sharpness.partial_cmp(&b.1.sharpness).unwrap())
    {
        order.retain(|&i| i != best);
        order.insert(0, best);
    }

    let active_template = if let Some(ref tid) = cfg.template_id {
        crate::templates::TEMPLATES
            .iter()
            .find(|t| t.id == tid)
            .cloned()
            .unwrap_or_else(|| {
                let scored = crate::templates::score_templates(cfg.event, photos, cfg.title.as_deref().unwrap_or(""));
                scored[0].template.clone()
            })
    } else {
        let scored = crate::templates::score_templates(cfg.event, photos, cfg.title.as_deref().unwrap_or(""));
        scored[0].template.clone()
    };

    let cycle = active_template.layouts;

    let mut shots = Vec::new();
    let mut k = 0usize;
    let mut i = 0usize;

    while i < order.len() {
        let idx = order[i];
        let p = &photos[idx];

        if i == 0 {
            // Opener shot
            let opener_layout = if cycle.contains(&Layout::Heart) && photos.len() >= 2 {
                Layout::Heart
            } else if cycle.contains(&Layout::Star) && photos.len() >= 2 {
                Layout::Star
            } else {
                cycle[0]
            };
            shots.push((opener_layout, vec![idx]));
            i += 1;
            continue;
        }

        // 3-photo celebration collage when photos available
        let wants_collage3 = cycle.contains(&Layout::Collage3) && (k % 3 == 1);
        if wants_collage3 && i + 2 < order.len() {
            shots.push((Layout::Collage3, vec![idx, order[i + 1], order[i + 2]]));
            i += 3;
            k += 1;
            continue;
        }

        // Two landscapes in a row -> split screen
        if p.is_landscape() && i + 1 < order.len() && photos[order[i + 1]].is_landscape() && shots.len() % 3 == 2 {
            shots.push((Layout::Split, vec![idx, order[i + 1]]));
            i += 2;
            k += 1;
            continue;
        }

        let layout = if !p.is_landscape() && p.aspect() < 0.8 && k % 4 == 0 {
            Layout::Fill
        } else {
            cycle[k % cycle.len()]
        };
        k += 1;
        shots.push((layout, vec![idx]));
        i += 1;
    }

    // Assign motion mode per shot (combining template motion and user preference)
    let base_motion = if cfg.motion == MotionMode::DynamicMix {
        active_template.motion
    } else {
        cfg.motion
    };

    shots
        .into_iter()
        .enumerate()
        .map(|(shot_idx, (layout, idxs))| {
            let motion = match base_motion {
                MotionMode::Boomerang => ShotMotion::Boomerang,
                MotionMode::FastForward => ShotMotion::FastForward,
                MotionMode::KenBurns => {
                    if shot_idx % 2 == 0 {
                        ShotMotion::ZoomIn
                    } else {
                        ShotMotion::ZoomOut
                    }
                }
                MotionMode::BeatDrop => {
                    if shot_idx % 2 == 0 {
                        ShotMotion::FastForward
                    } else {
                        ShotMotion::Boomerang
                    }
                }
                MotionMode::WaveFlow => {
                    if shot_idx % 2 == 0 {
                        ShotMotion::ZoomIn
                    } else {
                        ShotMotion::ZoomOut
                    }
                }
                MotionMode::TikTokSnap => {
                    match shot_idx % 3 {
                        0 => ShotMotion::Boomerang,
                        1 => ShotMotion::FastForward,
                        _ => ShotMotion::ZoomIn,
                    }
                }
                MotionMode::VintageRoll => {
                    if shot_idx % 2 == 0 {
                        ShotMotion::ZoomOut
                    } else {
                        ShotMotion::ZoomIn
                    }
                }
                MotionMode::DynamicMix => match shot_idx % 5 {
                    0 => ShotMotion::FastForward,
                    1 => ShotMotion::ZoomIn,
                    2 => ShotMotion::Boomerang,
                    3 => ShotMotion::ZoomOut,
                    _ => ShotMotion::Boomerang,
                },
            };
            (layout, idxs, motion)
        })
        .collect()
}

pub fn render_all(cfg: &Config, photos: &[Analysis]) -> Result<Vec<Shot>> {
    std::fs::create_dir_all(cfg.work_dir.join("frames"))?;
    let mut out = Vec::new();

    let active_template = if let Some(ref tid) = cfg.template_id {
        crate::templates::TEMPLATES
            .iter()
            .find(|t| t.id == tid)
            .cloned()
            .unwrap_or_else(|| {
                let scored = crate::templates::score_templates(cfg.event, photos, cfg.title.as_deref().unwrap_or(""));
                scored[0].template.clone()
            })
    } else {
        let scored = crate::templates::score_templates(cfg.event, photos, cfg.title.as_deref().unwrap_or(""));
        scored[0].template.clone()
    };

    println!("  🎨 Active Template: '{}' [{:?}] ({})", active_template.name, active_template.motion, active_template.description);
    let border_color = active_template.palette.border_rgb();

    let planned = plan(cfg, photos);
    let total = planned.len();

    // Preload photos once into RAM so we avoid repeated disk decode per shot
    let loaded_images: Vec<DynamicImage> = photos.iter().map(|p| load(&p.path)).collect::<Result<_>>()?;

    for (n, (layout, idxs, motion)) in planned.into_iter().enumerate() {
        let imgs: Vec<DynamicImage> = idxs.iter().map(|&i| loaded_images[i].clone()).collect();
        let mut canvas = compose(cfg.width, cfg.height, layout, &imgs, border_color);
        if cfg.stickers {
            apply_event_decorations(&mut canvas, cfg.event, active_template.sticker_theme, cfg.banner_style, n, total);
        }
        let frame: PathBuf = cfg.work_dir.join("frames").join(format!("shot_{n:03}.jpg"));
        image::DynamicImage::ImageRgba8(canvas).to_rgb8().save_with_format(&frame, image::ImageFormat::Jpeg)?;
        println!("  shot {n:02}: {:?} [{:?}] <- {} photo(s)", layout, motion, idxs.len());
        out.push(Shot { layout, photos: idxs, frame, motion });
    }
    Ok(out)
}

pub fn compose(w: u32, h: u32, layout: Layout, imgs: &[DynamicImage], border_color: [u8; 3]) -> RgbaImage {
    let first = &imgs[0];
    match layout {
        Layout::Fill => fill(first, w, h).to_rgba8(),
        Layout::BlurFit => {
            let mut c = blurred_bg(first, w, h);
            let fitted = first.resize(w, (h as f32 * 0.55) as u32, FilterType::Lanczos3).to_rgba8();
            let (x, y) = ((w - fitted.width()) / 2, (h - fitted.height()) / 2);
            imageops::overlay(&mut c, &fitted, x as i64, y as i64);
            c
        }
        Layout::Circle => shaped(first, w, h, 0.86, 1.0, Shape::Circle, border_color),
        Layout::Diamond => shaped(first, w, h, 0.90, 1.0, Shape::Diamond, border_color),
        Layout::Card => shaped(first, w, h, 0.90, 5.0 / 4.0, Shape::Rounded(56.0), border_color),
        Layout::Heart => shaped(first, w, h, 0.92, 1.0, Shape::Heart, border_color),
        Layout::Star => shaped(first, w, h, 0.92, 1.0, Shape::Star, border_color),
        Layout::Split => {
            let top = fill(&imgs[0], w, h / 2).to_rgba8();
            let bot = fill(imgs.get(1).unwrap_or(first), w, h - h / 2).to_rgba8();
            let mut c = RgbaImage::from_pixel(w, h, Rgba([12, 12, 12, 255]));
            imageops::overlay(&mut c, &top, 0, 0);
            imageops::overlay(&mut c, &bot, 0, (h / 2) as i64);
            for y in (h / 2 - 3)..(h / 2 + 3) {
                for x in 0..w {
                    c.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                }
            }
            c
        }
        Layout::Collage3 => {
            // 3-photo celebratory collage
            let top_h = (h as f32 * 0.52) as u32;
            let bot_h = h - top_h - 10;
            let half_w = (w - 10) / 2;

            let top = fill(&imgs[0], w, top_h).to_rgba8();
            let b1 = fill(imgs.get(1).unwrap_or(first), half_w, bot_h).to_rgba8();
            let b2 = fill(imgs.get(2).unwrap_or(imgs.get(1).unwrap_or(first)), w - half_w - 10, bot_h).to_rgba8();

            let mut c = blurred_bg(first, w, h);
            imageops::overlay(&mut c, &top, 0, 0);
            imageops::overlay(&mut c, &b1, 0, (top_h + 10) as i64);
            imageops::overlay(&mut c, &b2, (half_w + 10) as i64, (top_h + 10) as i64);

            // Dividing white borders
            for y in (top_h)..(top_h + 10) {
                for x in 0..w {
                    c.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                }
            }
            for y in (top_h + 10)..h {
                for x in (half_w)..(half_w + 10) {
                    c.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                }
            }
            c
        }
        Layout::Polaroid => {
            // Retro Polaroid photo card
            let mut c = blurred_bg(first, w, h);
            let cw = (w as f32 * 0.88) as u32;
            let ch = (cw as f32 * 1.22) as u32;
            let pad_side = 40u32;
            let pad_top = 40u32;
            let pad_bottom = 140u32;

            let iw = cw.saturating_sub(pad_side * 2);
            let ih = ch.saturating_sub(pad_top + pad_bottom);
            let inner = fill(first, iw, ih).to_rgba8();

            let ox = (w - cw) / 2;
            let oy = (h - ch) / 2;

            // Draw white polaroid card with subtle rounded corners
            for ty in 0..ch {
                for tx in 0..cw {
                    let cx = ox + tx;
                    let cy = oy + ty;
                    if cx < w && cy < h {
                        c.put_pixel(cx, cy, Rgba([255, 255, 255, 255]));
                    }
                }
            }

            // Overlay inner photo
            imageops::overlay(&mut c, &inner, (ox + pad_side) as i64, (oy + pad_top) as i64);

            // Decorative washi tape at the top
            let tape_w = 140u32;
            let tape_h = 36u32;
            let tape_x = (w - tape_w) / 2;
            let tape_y = oy.saturating_sub(16);
            for ty in 0..tape_h {
                for tx in 0..tape_w {
                    let cx = tape_x + tx;
                    let cy = tape_y + ty;
                    if cx < w && cy < h {
                        c.put_pixel(cx, cy, Rgba([245, 215, 150, 230]));
                    }
                }
            }
            c
        }
    }
}

fn fill(img: &DynamicImage, w: u32, h: u32) -> DynamicImage {
    let (x, y, cw, ch) = smart_crop(img, w as f32 / h as f32);
    img.crop_imm(x, y, cw, ch).resize_exact(w, h, FilterType::Triangle)
}

/// Cheap strong blur: shrink hard, enlarge smoothly, then darken a bit.
fn blurred_bg(img: &DynamicImage, w: u32, h: u32) -> RgbaImage {
    let (x, y, cw, ch) = smart_crop(img, w as f32 / h as f32);
    let small = img.crop_imm(x, y, cw, ch).resize_exact(w / 24, h / 24, FilterType::Triangle);
    let mut big = small.resize_exact(w, h, FilterType::CatmullRom).to_rgba8();
    for p in big.pixels_mut() {
        for c in 0..3 {
            p[c] = (p[c] as f32 * 0.55) as u8;
        }
    }
    big
}

#[derive(Clone, Copy)]
enum Shape {
    Circle,
    Diamond,
    Rounded(f32),
    Heart,
    Star,
}

/// Coverage 0..1 (anti-aliased) of `shape` at pixel (px,py) inside a size x size_h tile.
/// Designed specifically to preserve human faces, portrait headroom, and hairstyles.
fn coverage(shape: Shape, px: f32, py: f32, tw: f32, th: f32, inset: f32) -> f32 {
    let (cx, cy) = (tw / 2.0, th / 2.0);
    let (dx, dy) = (px - cx, py - cy);
    let sd = match shape {
        Shape::Circle => (dx * dx + dy * dy).sqrt() - tw.min(th) / 2.0,
        Shape::Diamond => {
            // Face-Safe Crowned Faceted Gem Diamond:
            // Slicing portraits into standard 45° rhombuses chops off foreheads, hair, and cheeks.
            // Upper crown (dy < 0): maintains wide headroom (nx <= 1.0) with soft 45° chamfers.
            // Lower pavilion (dy >= 0): tapers gracefully down to a brilliant diamond point.
            let rx = (tw / 2.0) * 0.94;
            let ry = (th / 2.0) * 0.94;
            let nx = dx.abs() / rx;
            let ny = dy / ry; // -1.0 at top (head), +1.0 at bottom (torso)

            let d = if ny < 0.0 {
                let up_y = -ny;
                let chamfer = (nx + up_y * 0.75 - 1.22).max(0.0);
                let box_d = nx.max(up_y) - 1.0;
                box_d.max(chamfer * 1.4)
            } else {
                (nx + ny * 0.92 - 1.0) * 0.7071
            };
            d * rx.min(ry)
        }
        Shape::Rounded(r) => {
            let qx = dx.abs() - (cx - r);
            let qy = dy.abs() - (cy - r);
            (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - r
        }
        Shape::Heart => {
            let r = tw.min(th) / 2.0;
            // Face-Safe zone: guaranteed circle preserving central face & forehead
            let face_dx = dx;
            let face_dy = dy + r * 0.15;
            let face_d = (face_dx * face_dx + face_dy * face_dy).sqrt() - (r * 0.58);

            // Elevated, soft romantic cleft notch
            let x = dx.abs() / (r * 0.96);
            let y = -dy / (r * 0.96) + 0.28;
            let d_heart = if y + x > 1.05 {
                ((x - 0.28).powi(2) + (y - 0.72).powi(2)).sqrt() - 0.42
            } else {
                let d1 = x.powi(2) + (y - 0.92).powi(2);
                let k = 0.5 * (x + y).max(0.0);
                let d2 = (x - k).powi(2) + (y - k).powi(2);
                let sign = if x < y { -1.0 } else { 1.0 };
                d1.min(d2).sqrt() * sign
            };
            let heart_d = d_heart * (r * 0.96);
            heart_d.min(face_d)
        }
        Shape::Star => {
            let r = tw.min(th) / 2.0;
            let dist_sq = dx * dx + dy * dy;
            let core_r = r * 0.72;
            if dist_sq < core_r * core_r {
                dist_sq.sqrt() - core_r
            } else {
                let core_d = dist_sq.sqrt() - core_r;
                let ax = dx.abs();
                let ay = dy.abs();
                let star_hyp = (ax + ay) * 0.7071 + (ax.max(ay)) * 0.45 - r * 0.95;
                star_hyp.min(core_d)
            }
        }
    };
    (0.5 - (sd + inset)).clamp(0.0, 1.0)
}

fn shaped(img: &DynamicImage, w: u32, h: u32, width_frac: f32, tile_aspect_h_over_w: f32, shape: Shape, border_color: [u8; 3]) -> RgbaImage {
    let mut canvas = blurred_bg(img, w, h);
    let tw = (w as f32 * width_frac) as u32;
    let th = (tw as f32 * tile_aspect_h_over_w) as u32;
    let (x, y, cw, ch) = smart_crop(img, tw as f32 / th as f32);
    let tile = img.crop_imm(x, y, cw, ch).resize_exact(tw, th, FilterType::Triangle).to_rgba8();
    let (ox, oy) = ((w - tw) as i64 / 2, (h - th) as i64 / 2);

    let border_f = [border_color[0] as f32, border_color[1] as f32, border_color[2] as f32];

    // 10px themed border: outer coverage minus inner (inset) coverage
    for ty in 0..th {
        for tx in 0..tw {
            let a = coverage(shape, tx as f32 + 0.5, ty as f32 + 0.5, tw as f32, th as f32, 0.0);
            if a <= 0.0 {
                continue;
            }
            let cx = (ox + tx as i64) as u32;
            let cy = (oy + ty as i64) as u32;
            let src = tile.get_pixel(tx, ty);
            let inner = coverage(shape, tx as f32 + 0.5, ty as f32 + 0.5, tw as f32, th as f32, 10.0);
            let mut rgb = [src[0] as f32, src[1] as f32, src[2] as f32];
            let b = 1.0 - inner;
            for c in 0..3 {
                rgb[c] = rgb[c] * (1.0 - b) + border_f[c] * b;
            }
            let dst = canvas.get_pixel_mut(cx, cy);
            for c in 0..3 {
                dst[c] = (dst[c] as f32 * (1.0 - a) + rgb[c] * a) as u8;
            }
        }
    }
    canvas
}

/// Apply decorative event elements and banner badge backgrounds
pub fn apply_event_decorations(canvas: &mut RgbaImage, event: EventTheme, sticker_theme: &str, banner_style: BannerStyle, shot_idx: usize, total_shots: usize) {
    let (w, h) = (canvas.width(), canvas.height());

    // Draw styled greeting banner base on opener (and closing) shot
    if shot_idx == 0 || shot_idx == total_shots.saturating_sub(1) {
        let badge_border = match event {
            EventTheme::Anniversary | EventTheme::Romantic => Rgba([255, 140, 180, 230]),
            EventTheme::Birthday | EventTheme::Party => Rgba([255, 215, 60, 230]),
            EventTheme::Travel => Rgba([80, 210, 255, 230]),
            EventTheme::Fitness => Rgba([255, 90, 50, 230]),
            EventTheme::Food => Rgba([255, 165, 0, 230]),
            EventTheme::Fashion => Rgba([230, 80, 180, 230]),
            EventTheme::Business => Rgba([60, 140, 255, 230]),
            EventTheme::Pets => Rgba([250, 200, 80, 230]),
            EventTheme::Graduation => Rgba([255, 215, 0, 230]),
            EventTheme::General => Rgba([200, 210, 230, 220]),
        };

        match banner_style {
            BannerStyle::CircularStamp => {
                // Circular emblem / seal badge at upper center
                let cx = (w / 2) as f32;
                let cy = 290.0f32;
                let r_outer = 152.0f32;
                let r_inner = 136.0f32;

                let min_x = (cx - r_outer - 90.0).max(0.0) as u32;
                let max_x = (cx + r_outer + 90.0).min(w as f32) as u32;
                let min_y = (cy - r_outer - 10.0).max(0.0) as u32;
                let max_y = (cy + r_outer + 10.0).min(h as f32) as u32;

                for py in min_y..max_y {
                    for px in min_x..max_x {
                        let dx = px as f32 - cx;
                        let dy = py as f32 - cy;
                        let dist = (dx * dx + dy * dy).sqrt();

                        // 1. Center frosted glass circle
                        if dist <= r_inner {
                            let old = canvas.get_pixel(px, py);
                            canvas.put_pixel(
                                px,
                                py,
                                Rgba([
                                    (old[0] as f32 * 0.16) as u8,
                                    (old[1] as f32 * 0.16) as u8,
                                    (old[2] as f32 * 0.22) as u8,
                                    250,
                                ]),
                            );
                        } else if dist <= r_outer && dist >= r_inner {
                            // 2. Double border ring with circular accent notches
                            let is_ring = dist >= r_outer - 3.5 || dist <= r_inner + 3.0;
                            let angle = dy.atan2(dx);
                            let is_notch = (angle * 12.0).sin().abs() > 0.85;

                            if is_ring || is_notch {
                                canvas.put_pixel(px, py, badge_border);
                            } else {
                                let old = canvas.get_pixel(px, py);
                                canvas.put_pixel(
                                    px,
                                    py,
                                    Rgba([
                                        (old[0] as f32 * 0.28) as u8,
                                        (old[1] as f32 * 0.28) as u8,
                                        (old[2] as f32 * 0.35) as u8,
                                        240,
                                    ]),
                                );
                            }
                        }

                        // 3. Side decorative ribbons extending left and right
                        let ribbon_y_dist = (py as f32 - cy).abs();
                        if ribbon_y_dist <= 26.0 {
                            let ribbon_x_dist = dx.abs();
                            if ribbon_x_dist > r_outer - 8.0 && ribbon_x_dist <= r_outer + 80.0 {
                                if ribbon_y_dist >= 23.5 || ribbon_x_dist >= r_outer + 77.0 {
                                    canvas.put_pixel(px, py, badge_border);
                                } else {
                                    let old = canvas.get_pixel(px, py);
                                    canvas.put_pixel(
                                        px,
                                        py,
                                        Rgba([
                                            (old[0] as f32 * 0.22) as u8,
                                            (old[1] as f32 * 0.22) as u8,
                                            (old[2] as f32 * 0.28) as u8,
                                            230,
                                        ]),
                                    );
                                }
                            }
                        }
                    }
                }
            }

            BannerStyle::LowerThird => {
                // Editorial cinematic lower-third card
                let card_w = w - 120;
                let card_h = 120u32;
                let bx = 60u32;
                let by = h - 360;
                let r = 16.0f32;

                for py in 0..card_h {
                    for px in 0..card_w {
                        let qx = (px as f32 - card_w as f32 / 2.0).abs() - (card_w as f32 / 2.0 - r);
                        let qy = (py as f32 - card_h as f32 / 2.0).abs() - (card_h as f32 / 2.0 - r);
                        let sd = if qx > 0.0 && qy > 0.0 {
                            (qx * qx + qy * qy).sqrt() - r
                        } else {
                            qx.max(qy)
                        };

                        if sd <= 0.0 {
                            let cx = bx + px;
                            let cy = by + py;
                            if cx < w && cy < h {
                                // Left accent stripe
                                if px >= 16 && px <= 24 && py >= 20 && py <= card_h - 20 {
                                    canvas.put_pixel(cx, cy, badge_border);
                                } else if sd > -2.0 {
                                    canvas.put_pixel(cx, cy, Rgba([badge_border[0], badge_border[1], badge_border[2], 180]));
                                } else {
                                    let old = canvas.get_pixel(cx, cy);
                                    canvas.put_pixel(
                                        cx,
                                        cy,
                                        Rgba([
                                            (old[0] as f32 * 0.18) as u8,
                                            (old[1] as f32 * 0.18) as u8,
                                            (old[2] as f32 * 0.24) as u8,
                                            245,
                                        ]),
                                    );
                                }
                            }
                        }
                    }
                }
            }

            BannerStyle::CenterHero => {
                // Bold magazine headline card across center
                let card_w = w - 80;
                let card_h = 180u32;
                let bx = 40u32;
                let by = (h / 2).saturating_sub(90);

                for py in 0..card_h {
                    for px in 0..card_w {
                        let cx = bx + px;
                        let cy = by + py;
                        if cx < w && cy < h {
                            let is_edge = py < 3 || py >= card_h - 3 || px < 2 || px >= card_w - 2;
                            if is_edge {
                                canvas.put_pixel(cx, cy, badge_border);
                            } else {
                                let old = canvas.get_pixel(cx, cy);
                                canvas.put_pixel(
                                    cx,
                                    cy,
                                    Rgba([
                                        (old[0] as f32 * 0.15) as u8,
                                        (old[1] as f32 * 0.15) as u8,
                                        (old[2] as f32 * 0.20) as u8,
                                        245,
                                    ]),
                                );
                            }
                        }
                    }
                }
            }

            BannerStyle::BottomFloat => {
                // Minimalist floating pill at bottom
                let badge_w = 520u32;
                let badge_h = 76u32;
                let bx = (w.saturating_sub(badge_w)) / 2;
                let by = h - 240;
                let r = badge_h as f32 / 2.0;

                for py in 0..badge_h {
                    for px in 0..badge_w {
                        let dx = px as f32 - (badge_w as f32 / 2.0);
                        let dy = py as f32 - r;
                        let qx = dx.abs() - (badge_w as f32 / 2.0 - r);
                        let sd = if qx > 0.0 { (qx * qx + dy * dy).sqrt() - r } else { dy.abs() - r };

                        if sd <= 0.0 {
                            let cx = bx + px;
                            let cy = by + py;
                            if cx < w && cy < h {
                                if sd > -2.5 {
                                    canvas.put_pixel(cx, cy, badge_border);
                                } else {
                                    let old = canvas.get_pixel(cx, cy);
                                    canvas.put_pixel(
                                        cx,
                                        cy,
                                        Rgba([
                                            (old[0] as f32 * 0.20) as u8,
                                            (old[1] as f32 * 0.20) as u8,
                                            (old[2] as f32 * 0.25) as u8,
                                            240,
                                        ]),
                                    );
                                }
                            }
                        }
                    }
                }
            }

            BannerStyle::TopPill => {
                // Standard floating frosted pill at top
                let badge_w = 560u32;
                let badge_h = 84u32;
                let bx = (w.saturating_sub(badge_w)) / 2;
                let by = 120u32;
                let r = badge_h as f32 / 2.0;

                for py in 0..badge_h {
                    for px in 0..badge_w {
                        let dx = px as f32 - (badge_w as f32 / 2.0);
                        let dy = py as f32 - r;
                        let qx = dx.abs() - (badge_w as f32 / 2.0 - r);
                        let sd = if qx > 0.0 { (qx * qx + dy * dy).sqrt() - r } else { dy.abs() - r };

                        if sd <= 0.0 {
                            let cx = bx + px;
                            let cy = by + py;
                            if cx < w && cy < h {
                                if sd > -2.5 {
                                    canvas.put_pixel(cx, cy, badge_border);
                                } else {
                                    let old = canvas.get_pixel(cx, cy);
                                    canvas.put_pixel(
                                        cx,
                                        cy,
                                        Rgba([
                                            (old[0] as f32 * 0.22) as u8,
                                            (old[1] as f32 * 0.22) as u8,
                                            (old[2] as f32 * 0.25) as u8,
                                            255,
                                        ]),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Floating Stickers & Emojis matching template theme
    if sticker_theme == "hearts" || matches!(event, EventTheme::Anniversary | EventTheme::Romantic) {
        let hearts = [
            (120.0f32, 280.0f32, 38.0f32),
            (960.0, 320.0, 30.0),
            (100.0, 1640.0, 34.0),
            (950.0, 1680.0, 44.0),
        ];
        for &(hx, hy, hr) in &hearts {
            draw_mini_heart(canvas, hx, hy, hr, [255, 80, 130]);
        }
    } else if sticker_theme == "confetti" || matches!(event, EventTheme::Birthday | EventTheme::Party) {
        let confetti = [
            (100u32, 260u32, 10u32, [255, 215, 0]),
            (160, 320, 7, [255, 90, 160]),
            (940, 260, 9, [0, 220, 255]),
            (980, 330, 12, [140, 255, 100]),
            (90, 1620, 11, [255, 100, 180]),
            (140, 1690, 8, [255, 225, 50]),
            (940, 1640, 12, [255, 130, 40]),
            (980, 1710, 8, [0, 230, 255]),
        ];
        for &(cx, cy, cr, col) in &confetti {
            draw_circle_particle(canvas, cx, cy, cr, col);
        }
        draw_sparkle_cross(canvas, 130, 240, 24, [255, 240, 180]);
        draw_sparkle_cross(canvas, 950, 290, 20, [255, 240, 180]);
    } else {
        draw_sparkle_cross(canvas, 120, 260, 22, [255, 255, 255]);
        draw_sparkle_cross(canvas, 960, 260, 22, [255, 255, 255]);
    }
}

fn draw_mini_heart(canvas: &mut RgbaImage, cx: f32, cy: f32, r: f32, rgb: [u8; 3]) {
    let (w, h) = (canvas.width() as f32, canvas.height() as f32);
    let size = (r * 2.2) as i32;
    for dy in -size..=size {
        for dx in -size..=size {
            let px = cx + dx as f32;
            let py = cy + dy as f32;
            if px < 0.0 || py < 0.0 || px >= w || py >= h {
                continue;
            }
            let x = dx.abs() as f32 / r;
            let y = -dy as f32 / r + 0.38;
            let inside = if y + x > 1.0 {
                ((x - 0.25).powi(2) + (y - 0.75).powi(2)).sqrt() - std::f32::consts::SQRT_2 / 4.0 <= 0.0
            } else {
                let d1 = (x - 0.0).powi(2) + (y - 1.0).powi(2);
                let k = 0.5 * (x + y).max(0.0);
                let d2 = (x - k).powi(2) + (y - k).powi(2);
                let sign = if x < y { -1.0 } else { 1.0 };
                d1.min(d2).sqrt() * sign <= 0.0
            };
            if inside {
                canvas.put_pixel(px as u32, py as u32, Rgba([rgb[0], rgb[1], rgb[2], 240]));
            }
        }
    }
}

fn draw_circle_particle(canvas: &mut RgbaImage, cx: u32, cy: u32, r: u32, rgb: [u8; 3]) {
    let (w, h) = (canvas.width(), canvas.height());
    let ir = r as i32;
    for dy in -ir..=ir {
        for dx in -ir..=ir {
            if dx * dx + dy * dy <= ir * ir {
                let px = cx as i32 + dx;
                let py = cy as i32 + dy;
                if px >= 0 && py >= 0 && (px as u32) < w && (py as u32) < h {
                    canvas.put_pixel(px as u32, py as u32, Rgba([rgb[0], rgb[1], rgb[2], 240]));
                }
            }
        }
    }
}

fn draw_sparkle_cross(canvas: &mut RgbaImage, cx: u32, cy: u32, size: u32, rgb: [u8; 3]) {
    let (w, h) = (canvas.width(), canvas.height());
    let s = size as i32;
    for d in -s..=s {
        let px1 = cx as i32 + d;
        let py1 = cy as i32;
        if px1 >= 0 && (px1 as u32) < w && py1 >= 0 && (py1 as u32) < h {
            canvas.put_pixel(px1 as u32, py1 as u32, Rgba([rgb[0], rgb[1], rgb[2], 255]));
        }
        let px2 = cx as i32;
        let py2 = cy as i32 + d;
        if px2 >= 0 && (px2 as u32) < w && py2 >= 0 && (py2 as u32) < h {
            canvas.put_pixel(px2 as u32, py2 as u32, Rgba([rgb[0], rgb[1], rgb[2], 255]));
        }
    }
}

