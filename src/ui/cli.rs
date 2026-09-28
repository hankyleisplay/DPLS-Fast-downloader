use crate::engine::downloader::{DownloadTask, TaskState};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

pub async fn run_cli_download(
    url: String,
    output_dir: PathBuf,
    filename: Option<String>,
    connections: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\x1b[1;36m===================================================\x1b[0m");
    println!("\x1b[1;32m  🚀 DPLS-Fast | 極速多線程下載器 (IDM-like Engine)  \x1b[0m");
    println!("\x1b[1;36m===================================================\x1b[0m");
    println!("  目標網址:   \x1b[1m{}\x1b[0m", url);
    println!("  儲存路徑:   {}", output_dir.display());
    println!("  併發連線:   {} 線程", connections);
    println!("  動態分段:   已啟用 (Dynamic Splitting)");
    println!("\x1b[1;36m---------------------------------------------------\x1b[0m");

    let task_id = uuid::Uuid::new_v4().to_string();
    let task = Arc::new(DownloadTask::new(
        task_id,
        url,
        output_dir,
        filename,
        connections,
        None,
        None,
        None,
    )?);

    // Setup Ctrl+C handler to pause and persist progress
    let task_ctrlc = task.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            eprintln!("\n\x1b[1;33m[!] 收到中斷訊號，正在儲存分段狀態以供後續斷點續傳...\x1b[0m");
            task_ctrlc.pause().await;
            std::process::exit(0);
        }
    });

    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({percent}%) | {msg}")
            .expect("Invalid progress style")
            .progress_chars("#>-"),
    );

    let mut rx = task.progress_tx.subscribe();
    let task_runner = task.clone();

    tokio::spawn(async move {
        if let Err(e) = task_runner.start().await {
            eprintln!("\x1b[1;31m下載錯誤: {}\x1b[0m", e);
        }
    });

    let start_time = Instant::now();
    let mut last_total = 0u64;

    while let Ok(snap) = rx.recv().await {
        match snap.state {
            TaskState::Probing => {
                pb.set_message("正在探測遠端伺服器 (Range & Headers)...");
            }
            TaskState::Downloading => {
                if let Some(total) = snap.total_size {
                    if last_total != total {
                        pb.set_length(total);
                        last_total = total;
                    }
                }
                pb.set_position(snap.downloaded_bytes);

                let speed_mb = (snap.speed_bps as f64) / (1024.0 * 1024.0);
                let eta_str = match snap.eta_seconds {
                    Some(eta) if eta > 3600 => format!("{}h {}m", eta / 3600, (eta % 3600) / 60),
                    Some(eta) if eta > 60 => format!("{}m {}s", eta / 60, eta % 60),
                    Some(eta) => format!("{}s", eta),
                    None => "--".to_string(),
                };

                let active_segs = snap
                    .segments
                    .iter()
                    .filter(|s| s.status == crate::engine::segment::SegmentStatus::Downloading)
                    .count();

                pb.set_message(format!(
                    "\x1b[1;32m{:.2} MB/s\x1b[0m | ETA: {} | 活躍分段: {}/{}",
                    speed_mb, eta_str, active_segs, snap.segments.len()
                ));
            }
            TaskState::Completed => {
                pb.finish_with_message("\x1b[1;32m下載完成！\x1b[0m");
                let elapsed = start_time.elapsed();
                let avg_speed = if elapsed.as_secs_f64() > 0.0 {
                    (snap.downloaded_bytes as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64()
                } else {
                    0.0
                };

                println!("\x1b[1;32m\n✔ 下載成功完成！\x1b[0m");
                println!("  檔案名稱: \x1b[1m{}\x1b[0m", snap.filename);
                println!("  完整路徑: {}", snap.output_path);
                println!("  總大小:   {:.2} MB", (snap.downloaded_bytes as f64) / (1024.0 * 1024.0));
                println!("  平均速度: {:.2} MB/s", avg_speed);
                println!("  總耗時:   {:.2} 秒", elapsed.as_secs_f64());
                break;
            }
            TaskState::Paused => {
                pb.abandon_with_message("\x1b[1;33m已暫停\x1b[0m");
                break;
            }
            TaskState::Error(err) => {
                pb.abandon_with_message(format!("\x1b[1;31m失敗: {}\x1b[0m", err));
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
