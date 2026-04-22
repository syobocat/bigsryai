use ab_glyph::{Font, FontRef, Point, Rect, ScaleFont};
use image::{Rgba, RgbaImage};

use crate::util;

const FONT_DATA_NYASHI: &[u8; 4754620] = include_bytes!("../assets/Nyashi.ttf");

const FONT_SCALE_STAMP: f32 = 32.0;
const FONT_SCALE_OVERLAY: f32 = 64.0;
const MARGIN_STAMP: u32 = 2;
const MARGIN_OVERLAY: u32 = 10;

const TEXT_STAMP: &str = "nexryai";

fn calculate_bounding_box(font: &impl Font, scale: f32, text: &str) -> Rect {
    let scaled_font = font.as_scaled(scale);

    let mut caret = ab_glyph::point(0.0, scaled_font.ascent());
    let mut previous = None;
    let mut min_x = f32::NAN;
    let mut min_y = f32::NAN;
    let mut max_x = f32::NAN;
    let mut max_y = f32::NAN;
    for c in text.chars() {
        let glyph_id = font.glyph_id(c);
        if let Some(prev_id) = previous {
            caret.x += scaled_font.kern(prev_id, glyph_id);
        }
        let glyph = glyph_id.with_scale_and_position(scale, caret);
        caret.x += scaled_font.h_advance(glyph_id);
        previous = Some(glyph_id);

        if let Some(og) = font.outline_glyph(glyph) {
            let bb = og.px_bounds();
            min_x = min_x.min(bb.min.x);
            min_y = min_y.min(bb.min.y);
            max_x = max_x.max(bb.max.x);
            max_y = max_y.max(bb.max.y);
        }
    }

    Rect {
        min: ab_glyph::point(min_x, min_y),
        max: ab_glyph::point(max_x, max_y),
    }
}

fn draw_text(
    font: &impl Font,
    scale: f32,
    text: &str,
    offset: Point,
    mut f: impl FnMut(u32, u32, f32, Rect),
) {
    let scaled_font = font.as_scaled(scale);

    let mut caret = offset;
    let mut previous = None;
    for c in text.chars() {
        let glyph_id = font.glyph_id(c);
        if let Some(prev_id) = previous {
            caret.x += scaled_font.kern(prev_id, glyph_id);
        }
        let glyph = glyph_id.with_scale_and_position(scale, caret);
        caret.x += scaled_font.h_advance(glyph_id);
        previous = Some(glyph_id);

        if let Some(og) = font.outline_glyph(glyph) {
            let bb = og.px_bounds();
            og.draw(|gx, gy, v| f(gx, gy, v, bb));
        }
    }
}

// 文字スタンプ生成
pub fn generate_stamp() -> RgbaImage {
    let font = FontRef::try_from_slice(FONT_DATA_NYASHI).unwrap();
    let bb = calculate_bounding_box(&font, FONT_SCALE_STAMP, TEXT_STAMP);

    let text_width = (bb.max.x - bb.min.x) as u32;
    let text_height = (bb.max.y - bb.min.y) as u32;

    let stamp_width = text_width + 2 * MARGIN_STAMP;
    let stamp_height = text_height + 2 * MARGIN_STAMP;

    let mut text_stamp =
        RgbaImage::from_pixel(stamp_width, stamp_height, Rgba([255, 255, 255, 255]));

    // Draw
    let scaled_font = font.as_scaled(FONT_SCALE_STAMP);
    let offset = ab_glyph::point(
        MARGIN_STAMP as f32 - bb.min.x,
        MARGIN_STAMP as f32 - bb.min.y + scaled_font.ascent(),
    );
    draw_text(&font, 32.0, TEXT_STAMP, offset, |gx, gy, v, bb| {
        let x = bb.min.x + gx as f32;
        let y = bb.min.y + gy as f32;
        if x >= 0.0 && y >= 0.0 && (x as u32) < stamp_width && (y as u32) < stamp_height {
            let hue = x / (stamp_width as f32);
            let (r, g, b) = util::hsv_to_rgb(hue, 1.0, 1.0);
            let blended_r = f32::from(r).mul_add(v, 255.0 * (1.0 - v)).round() as u8;
            let blended_g = f32::from(g).mul_add(v, 255.0 * (1.0 - v)).round() as u8;
            let blended_b = f32::from(b).mul_add(v, 255.0 * (1.0 - v)).round() as u8;
            text_stamp.put_pixel(
                x as u32,
                y as u32,
                Rgba([blended_r, blended_g, blended_b, 255]),
            );
        }
    });

    text_stamp
}

pub fn generate_overlay(text: &str) -> RgbaImage {
    let font = FontRef::try_from_slice(FONT_DATA_NYASHI).unwrap();
    let bb = calculate_bounding_box(&font, FONT_SCALE_OVERLAY, text);

    let text_width = (bb.max.x - bb.min.x) as u32;
    let text_height = (bb.max.y - bb.min.y) as u32;

    let stamp_width = text_width + 2 * MARGIN_OVERLAY;
    let stamp_height = text_height + 2 * MARGIN_OVERLAY;

    let mut text_stamp = RgbaImage::from_pixel(stamp_width, stamp_height, Rgba([0, 0, 0, 0]));

    // Draw
    let scaled_font = font.as_scaled(FONT_SCALE_OVERLAY);
    let offset = ab_glyph::point(
        MARGIN_OVERLAY as f32 - bb.min.x,
        MARGIN_OVERLAY as f32 - bb.min.y + scaled_font.ascent(),
    );
    draw_text(&font, 64.0, text, offset, |gx, gy, v, bb| {
        let x = bb.min.x + gx as f32;
        let y = bb.min.y + gy as f32;
        if x >= 0.0 && y >= 0.0 && (x as u32) < stamp_width && (y as u32) < stamp_height {
            let alpha = (v * 255.0).round() as u8;
            text_stamp.put_pixel(x as u32, y as u32, Rgba([255, 255, 255, alpha]));
        }
    });

    text_stamp
}
