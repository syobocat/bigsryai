use std::{
    env,
    time::{Duration, Instant},
};
use sysinfo::System;

fn main() {
    // ----- ベンチマーク部（横一列レンダリング） -----
    let args: Vec<String> = env::args().collect();
    let threshold_secs: f64 = if args.len() > 1 {
        args[1].parse().unwrap_or(2.0)
    } else {
        2.0
    };
    let threshold = Duration::from_secs_f64(threshold_secs);
    println!("使用する閾値:\t{threshold_secs:.2} 秒");

    let mut sys = System::new_all();
    sys.refresh_all();
    println!("■ システムスペック");
    println!(" Total Memory:\t{} MB", sys.total_memory() / 1024);
    println!("---------------------------------------");

    // 前回の消費時間から柔軟にジャンプ倍率を算出して letter_count を増加
    let mut count = 1;
    let mut lower;
    loop {
        lower = count;

        let start = Instant::now();
        let canvas = bigsryai::render_many(count);
        let duration = start.elapsed();

        let (canvas_width, canvas_height) = canvas.dimensions();
        print!(
            "ベンチマーク中:\tねく数 = {count}\tキャンバスサイズ = {canvas_width}x{canvas_height}\t経過時間 = {duration:.2?}"
        );
        let ratio = threshold.as_secs_f64() / duration.as_secs_f64();
        let factor = if ratio < 1.1 { 1.1 } else { ratio };
        count = (f64::from(count) * factor).ceil() as u32;
        if duration <= threshold {
            println!();
        } else {
            println!("<-- 閾値超過");
            break;
        }
    }
    let mut upper = count;

    while upper - lower > 1 {
        let mid = (lower + upper) / 2;

        let start = Instant::now();
        let canvas = bigsryai::render_many(mid);
        let duration = start.elapsed();

        let (canvas_width, canvas_height) = canvas.dimensions();
        println!(
            "二分探索中:\tねく数 = {mid}\tキャンバスサイズ = {canvas_width}x{canvas_height}\t経過時間 = {duration:.2?}"
        );
        if duration > threshold {
            upper = mid;
        } else {
            lower = mid;
        }
    }
    count = lower;

    println!("■ ベンチマーク結果");
    println!("スコア:\t{count}");

    // ----- 最終結果画像生成 -----
    println!("結果画像出力中……");
    let final_img = bigsryai::render_result(count, 1920, 1080);
    final_img
        .save("output.png")
        .expect("結果の書き込みに失敗しました");
    println!("FHDリザルト画像（1920×1080）として output.png に保存しました。");
}
