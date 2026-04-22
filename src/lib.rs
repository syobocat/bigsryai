use image::{GenericImage, Rgba, RgbaImage, imageops};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::OnceLock;

mod render;
mod util;

static CELL_SIZE: OnceLock<(u32, u32)> = OnceLock::new();

pub fn render_one(cell_index: u32) -> RgbaImage {
    let cell = render::render_cell(0, 0, cell_index);
    let _ = CELL_SIZE.set(cell.dimensions());

    cell
}

pub fn render_many(count: u32) -> RgbaImage {
    let cells: Vec<RgbaImage> = (0..count).into_par_iter().map(|i| render_one(i)).collect();

    let (cell_w, cell_h) = CELL_SIZE.get().unwrap_or(&(0, 0));

    let final_w = count * cell_w;
    let final_h = *cell_h;

    let mut canvas = RgbaImage::from_pixel(final_w, final_h, Rgba([255, 255, 255, 255]));
    for (i, cell) in cells.into_iter().enumerate() {
        let dest_x = i as u32 * cell_w;
        canvas.copy_from(&cell, dest_x, 0).unwrap();
    }

    canvas
}

pub fn render_result(count: u32, width: u32, height: u32) -> RgbaImage {
    let cells: Vec<RgbaImage> = (0..count).into_par_iter().map(|i| render_one(i)).collect();

    let (cell_w, cell_h) = CELL_SIZE.get().unwrap_or(&(0, 0));

    // 最もaspect_ratioに近い並べ方を探索
    // TODO: 最適化の余地いっぱいあり
    let mut best = u32::MAX;
    let mut columns = 1;
    for c in 1..=count {
        // aとbの値が最も近づくポイントを探す
        let a = c * cell_w * height;
        let b = count.div_ceil(c) * cell_h * width;
        let score = a.max(b) - a.min(b);
        if score < best {
            best = score;
            columns = c;
        }

        // aとbが逆転したらそれ以降は探索不要
        if a > b {
            break;
        }
    }

    // 求めた並べ方で並べる
    let columns = columns;
    let rows = count.div_ceil(columns);
    let mut canvas =
        RgbaImage::from_pixel(columns * cell_w, rows * cell_h, Rgba([255, 255, 255, 255]));
    for (i, row) in cells.chunks(columns as usize).enumerate() {
        for (j, cell) in row.iter().enumerate() {
            canvas
                .copy_from(cell, j as u32 * cell_w, i as u32 * cell_h)
                .unwrap();
        }
    }

    // 指定サイズにリサイズ
    let mut final_img = imageops::resize(&canvas, width, height, imageops::Lanczos3);
    render::render_overlay(&mut final_img, &format!("スコア: {count}"));

    final_img
}
