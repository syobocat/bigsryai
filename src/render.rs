use image::{Rgba, RgbaImage, imageops};

use crate::util;

mod text;

/// 各セル内で各種効果を適用して文字を描画する（描画位置は `base_x`, `base_y` から）
pub fn render_cell(base_x: u32, base_y: u32, cell_index: u32) -> RgbaImage {
    let text_stamp = text::generate_stamp();
    let stamp_w = text_stamp.width();
    let stamp_h = text_stamp.height();

    let mut canvas = RgbaImage::from_pixel(stamp_w, stamp_h, Rgba([255, 255, 255, 255]));

    // (1) Glow 効果
    let glow_range: i32 = 3;
    for dx in -glow_range..=glow_range {
        for dy in -glow_range..=glow_range {
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            let alpha_factor = ((glow_range as f32 - dist) / glow_range as f32).max(0.0) * 0.3;
            for (px, py, &p) in text_stamp.enumerate_pixels() {
                let dest_x = base_x as i32 + dx + px as i32;
                let dest_y = base_y as i32 + dy + py as i32;
                if dest_x >= 0
                    && dest_y >= 0
                    && dest_x < canvas.width() as i32
                    && dest_y < canvas.height() as i32
                {
                    let Rgba([r, g, b, a]) = p;
                    let white = 255u8;
                    let new_r = f32::from(r)
                        .mul_add(1.0 - alpha_factor, f32::from(white) * alpha_factor)
                        .min(255.0) as u8;
                    let new_g = f32::from(g)
                        .mul_add(1.0 - alpha_factor, f32::from(white) * alpha_factor)
                        .min(255.0) as u8;
                    let new_b = f32::from(b)
                        .mul_add(1.0 - alpha_factor, f32::from(white) * alpha_factor)
                        .min(255.0) as u8;
                    canvas.put_pixel(dest_x as u32, dest_y as u32, Rgba([new_r, new_g, new_b, a]));
                }
            }
        }
    }

    // (2) Extrusion 効果（縦方向の伸びを抑えるため base_dy を 2 に）
    let extrude_steps = 8;
    let base_dx = 5;
    let base_dy = 2;
    for extrude in 0..extrude_steps {
        let off_x = base_x + extrude * base_dx;
        let off_y = base_y + extrude * base_dy;
        let dark_factor = 1.0 - (extrude as f32) / ((extrude_steps + 1) as f32);
        for (px, py, &p) in text_stamp.enumerate_pixels() {
            let distortion_x = 5.0 * (((px as f32) + cell_index as f32) * 0.17).sin();
            let distortion_y = 5.0 * (((py as f32) + cell_index as f32) * 0.17).cos();
            let dest_x = off_x + px + distortion_x.round() as u32;
            let dest_y = off_y + py + distortion_y.round() as u32;
            if dest_x < canvas.width() && dest_y < canvas.height() {
                let Rgba([r, g, b, a]) = p;
                let new_r = (f32::from(r) * dark_factor).min(255.0) as u8;
                let new_g = (f32::from(g) * dark_factor).min(255.0) as u8;
                let new_b = (f32::from(b) * dark_factor).min(255.0) as u8;
                canvas.put_pixel(dest_x, dest_y, Rgba([new_r, new_g, new_b, a]));
            }
        }
    }

    // (3) 回転・変形効果付き前面描画（y軸は 0.7 倍で圧縮）
    let center_x = stamp_w as f32 / 2.0;
    let center_y = stamp_h as f32 / 2.0;
    let angle = 0.3 * ((cell_index as f32 * 0.7).sin());
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    for (px, py, &p) in text_stamp.enumerate_pixels() {
        let dx = px as f32 - center_x;
        let dy = py as f32 - center_y;
        let rdx = dx.mul_add(cos_a, -(dy * sin_a));
        let rdy = dx.mul_add(sin_a, dy * cos_a);
        let new_x = center_x + rdx;
        let new_y = rdy.mul_add(0.7, center_y);
        let dest_x = base_x + new_x.round() as u32;
        let dest_y = base_y + new_y.round() as u32;
        if dest_x < canvas.width() && dest_y < canvas.height() {
            let Rgba([r, g, b, a]) = p;
            let new_r = r.saturating_add(30);
            canvas.put_pixel(dest_x, dest_y, Rgba([new_r, g, b, a]));
        }
    }

    // (4) Sparkle 効果
    for (px, py, &p) in text_stamp.enumerate_pixels() {
        let Rgba([r, g, b, a]) = p;
        let lum = (u32::from(r) + u32::from(g) + u32::from(b)) / 3;
        if lum > 200 && ((px + py + cell_index) % 97 == 0) {
            let dest_x = base_x + px;
            let dest_y = base_y + py;
            if dest_x < canvas.width() && dest_y < canvas.height() {
                canvas.put_pixel(dest_x, dest_y, Rgba([255, 255, 255, a]));
            }
        }
    }

    // (5) 既存のシュール効果：特定条件で若干右へずらし、色味を微調整
    for (px, py, &p) in text_stamp.enumerate_pixels() {
        if (px + py + cell_index) % 101 == 0 {
            let dest_x = base_x + px + 3; // 横方向に 3 ピクセルずらす
            let dest_y = base_y + py;
            if dest_x < canvas.width() && dest_y < canvas.height() {
                let Rgba([r, g, b, a]) = p;
                let new_r = r.saturating_sub(10);
                let new_g = g.saturating_sub(10);
                let new_b = ((u16::from(b) + 10).min(255)) as u8;
                canvas.put_pixel(dest_x, dest_y, Rgba([new_r, new_g, new_b, a]));
            }
        }
    }

    // (6) 新たなカラフル乱れ効果：1/7 程度のピクセルをにゃぐにゃぐ動かし、HSV で大幅な色変調
    for (px, py, &p) in text_stamp.enumerate_pixels() {
        if (px + py + cell_index) % 7 == 0 {
            let offset_x = (5.0 * ((px as f32 + cell_index as f32) * 0.27).sin()).round() as i32;
            let offset_y = (5.0 * ((py as f32 + cell_index as f32) * 0.27).cos()).round() as i32;
            let dest_x = base_x as i32 + px as i32 + offset_x;
            let dest_y = base_y as i32 + py as i32 + offset_y;
            if dest_x >= 0
                && dest_y >= 0
                && dest_x < canvas.width() as i32
                && dest_y < canvas.height() as i32
            {
                let hue = (cell_index as f32).mul_add(
                    0.05,
                    (px as f32 / stamp_w as f32) + (py as f32 / stamp_h as f32),
                ) % 1.0;
                let (r2, g2, b2) = util::hsv_to_rgb(hue, 0.9, 1.0);
                let Rgba([r, g, b, a]) = p;
                let new_r = ((u16::from(r) + u16::from(r2)) / 2) as u8;
                let new_g = ((u16::from(g) + u16::from(g2)) / 2) as u8;
                let new_b = ((u16::from(b) + u16::from(b2)) / 2) as u8;
                canvas.put_pixel(dest_x as u32, dest_y as u32, Rgba([new_r, new_g, new_b, a]));
            }
        }
    }

    canvas
}

pub fn render_overlay(canvas: &mut RgbaImage, text: &str) {
    let overlay = text::generate_overlay(text);
    let pos_x = canvas.width().saturating_sub(overlay.width() + 10);
    let pos_y = canvas.height().saturating_sub(overlay.height() + 10);

    imageops::overlay(canvas, &overlay, pos_x as i64, pos_y as i64);
}
