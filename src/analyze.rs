use crate::types::Analysis;
use anyhow::{Context, Result};
use image::{imageops::FilterType, DynamicImage, ImageDecoder, ImageReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "tif", "tiff"];

pub fn collect_images(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|p| {
            p.extension()
                .and_then(|x| x.to_str())
                .map(|x| EXTS.contains(&x.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .collect();
    v.sort();
    v
}

/// Decode and honour EXIF orientation (phone photos!).
pub fn load(path: &Path) -> Result<DynamicImage> {
    let mut decoder = ImageReader::open(path)
        .with_context(|| format!("open {}", path.display()))?
        .with_guessed_format()?
        .into_decoder()?;
    let orientation = decoder.orientation()?;
    let mut img = DynamicImage::from_decoder(decoder)?;
    img.apply_orientation(orientation);
    Ok(img)
}

/// Sobel edge-energy map on a small grayscale thumbnail.
pub struct Energy {
    pub w: usize,
    pub h: usize,
    pub data: Vec<f32>,
}

pub fn energy_map(img: &DynamicImage) -> Energy {
    let thumb = img.resize(256, 256, FilterType::Triangle).to_luma8();
    let (w, h) = (thumb.width() as usize, thumb.height() as usize);
    let px = |x: usize, y: usize| thumb.get_pixel(x as u32, y as u32)[0] as f32;
    let mut data = vec![0.0f32; w * h];
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let gx = -px(x - 1, y - 1) - 2.0 * px(x - 1, y) - px(x - 1, y + 1)
                + px(x + 1, y - 1) + 2.0 * px(x + 1, y) + px(x + 1, y + 1);
            let gy = -px(x - 1, y - 1) - 2.0 * px(x, y - 1) - px(x + 1, y - 1)
                + px(x - 1, y + 1) + 2.0 * px(x, y + 1) + px(x + 1, y + 1);
            data[y * w + x] = (gx * gx + gy * gy).sqrt();
        }
    }
    Energy { w, h, data }
}

pub fn analyze(path: &Path) -> Result<Analysis> {
    let img = load(path)?;
    let e = energy_map(&img);
    let n = (e.w * e.h) as f32;
    let sharpness = e.data.iter().sum::<f32>() / n;

    // energy-weighted centroid with a mild centre prior (people centre their subject)
    let (mut sx, mut sy, mut sw) = (0.0f32, 0.0f32, 0.0f32);
    for y in 0..e.h {
        for x in 0..e.w {
            let nx = x as f32 / e.w as f32;
            let ny = y as f32 / e.h as f32;
            let prior = 1.0 - 0.6 * (((nx - 0.5).powi(2) + (ny - 0.5).powi(2)).sqrt() / 0.7071);
            let v = e.data[y * e.w + x] * prior;
            sx += v * nx;
            sy += v * ny;
            sw += v;
        }
    }
    let focus = if sw > 0.0 { (sx / sw, sy / sw) } else { (0.5, 0.5) };

    let gray = img.resize(64, 64, FilterType::Triangle).to_luma8();
    let brightness = gray.pixels().map(|p| p[0] as f32).sum::<f32>() / (gray.len() as f32 * 255.0);

    Ok(Analysis {
        path: path.to_path_buf(),
        width: img.width(),
        height: img.height(),
        sharpness,
        brightness,
        focus,
    })
}

/// Find the crop window with the given aspect (w/h) that keeps the most edge energy.
/// Returns (x, y, w, h) in source pixels.
pub fn smart_crop(img: &DynamicImage, aspect: f32) -> (u32, u32, u32, u32) {
    let (iw, ih) = (img.width() as f32, img.height() as f32);
    let (cw, ch) = if iw / ih > aspect {
        ((ih * aspect).round(), ih)
    } else {
        (iw, (iw / aspect).round())
    };
    let e = energy_map(img);
    let sx = e.w as f32 / iw;
    let sy = e.h as f32 / ih;
    let (tw, th) = (((cw * sx).round() as usize).clamp(1, e.w), ((ch * sy).round() as usize).clamp(1, e.h));

    // sliding window over the free axis; the other axis is full-size
    let mut best = (0usize, 0usize, f32::MIN);
    let steps_x = e.w - tw;
    let steps_y = e.h - th;
    let col_sum = |x: usize, y0: usize| -> f32 { (y0..y0 + th).map(|y| e.data[y * e.w + x]).sum() };
    if steps_x > 0 {
        let sums: Vec<f32> = (0..e.w).map(|x| col_sum(x, 0)).collect();
        for x0 in 0..=steps_x {
            let s: f32 = (x0..x0 + tw).map(|x| sums[x]).sum();
            // gentle centre bias breaks ties toward composed shots
            let c = 1.0 - 0.15 * ((x0 as f32 + tw as f32 / 2.0) / e.w as f32 - 0.5).abs();
            if s * c > best.2 {
                best = (x0, 0, s * c);
            }
        }
    } else if steps_y > 0 {
        let row_sums: Vec<f32> = (0..e.h)
            .map(|y| (0..tw).map(|x| e.data[y * e.w + x]).sum())
            .collect();
        for y0 in 0..=steps_y {
            let s: f32 = (y0..y0 + th).map(|y| row_sums[y]).sum();
            let c = 1.0 - 0.15 * ((y0 as f32 + th as f32 / 2.0) / e.h as f32 - 0.5).abs();
            if s * c > best.2 {
                best = (0, y0, s * c);
            }
        }
    } else {
        best = (0, 0, 0.0);
    }
    let x = ((best.0 as f32 / sx).round() as u32).min(img.width().saturating_sub(cw as u32));
    let y = ((best.1 as f32 / sy).round() as u32).min(img.height().saturating_sub(ch as u32));
    (x, y, cw as u32, ch as u32)
}
