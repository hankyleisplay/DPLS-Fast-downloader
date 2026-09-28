use dpls_fast::engine;
use dpls_fast::ui;

use clap::{Parser, Subcommand};
use engine::manager::TaskManager;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "dpls")]
#[command(about = "🚀 DPLS-Fast: 極速多線程下載器 (IDM-like Multi-segment Downloader)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// 下載目標網址 (直接輸入網址進行終端極速下載)
    #[arg(value_name = "URL")]
    url: Option<String>,

    /// 動態併發連線數 (預設: 16)
    #[arg(short = 'c', long, default_value_t = 16)]
    connections: usize,

    /// 輸出檔案名稱 (預設: 自動由伺服器探測)
    #[arg(short = 'o', long)]
    output: Option<String>,

    /// 儲存目錄 (預設: 當前目錄或 ~/Downloads)
    #[arg(short = 'd', long)]
    dir: Option<PathBuf>,

    /// 啟動 Web UI 視覺化儀表板
    #[arg(long)]
    ui: bool,

    /// Web 伺服器連接埠 (預設: 6800)
    #[arg(short = 'p', long, default_value_t = 6800)]
    port: u16,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 啟動 Web 視覺化控制台並自動開啟瀏覽器
    Ui {
        #[arg(short = 'p', long, default_value_t = 6800)]
        port: u16,
    },
    /// 以無頭背景伺服器模式運行 (相容 REST API 與 WebSockets)
    Server {
        #[arg(short = 'p', long, default_value_t = 6800)]
        port: u16,
    },
    /// 下載指定檔案
    Download {
        /// 目標網址
        url: String,

        /// 併發連線數
        #[arg(short = 'c', long, default_value_t = 16)]
        connections: usize,

        /// 自訂檔名
        #[arg(short = 'o', long)]
        output: Option<String>,

        /// 儲存目錄
        #[arg(short = 'd', long)]
        dir: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    let default_dir = dirs::download_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // 1. Check direct URL CLI argument
    if let Some(url) = cli.url {
        let target_dir = cli.dir.unwrap_or(default_dir);
        return ui::cli::run_cli_download(url, target_dir, cli.output, cli.connections).await;
    }

    // 2. Check subcommand
    match cli.command {
        Some(Commands::Download {
            url,
            connections,
            output,
            dir,
        }) => {
            let target_dir = dir.unwrap_or(default_dir);
            ui::cli::run_cli_download(url, target_dir, output, connections).await?;
        }
        Some(Commands::Ui { port }) => {
            let manager = Arc::new(TaskManager::new(default_dir));
            ui::web_server::start_web_server(manager, port, true).await?;
        }
        Some(Commands::Server { port }) => {
            let manager = Arc::new(TaskManager::new(default_dir));
            ui::web_server::start_web_server(manager, port, false).await?;
        }
        None => {
            // Default behavior when no URL or subcommand: start Web UI
            let port = cli.port;
            println!("\x1b[1;36m===================================================\x1b[0m");
            println!("\x1b[1;32m  🚀 DPLS-Fast | 極速多線程下載加速器 Web 儀表板  \x1b[0m");
            println!("\x1b[1;36m===================================================\x1b[0m");
            println!("  預設下載目錄: {}", default_dir.display());
            println!("  訪問網址:     http://localhost:{}", port);
            println!("\x1b[1;36m---------------------------------------------------\x1b[0m");

            let manager = Arc::new(TaskManager::new(default_dir));
            ui::web_server::start_web_server(manager, port, true).await?;
        }
    }

    Ok(())
}
