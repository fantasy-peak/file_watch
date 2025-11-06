mod config;
mod processor;
mod reader;
mod watcher;

use anyhow::Result;
use serde_yaml;
use std::fs;
use tokio::signal;
use watcher::FileWatcher;

#[tokio::main]
async fn main() -> Result<()> {
    let contents = fs::read_to_string("/root/github/file_watch/conf/cfg.yaml")?;
    let cfg: config::AppConfig = serde_yaml::from_str(&contents)?;

    let watcher = FileWatcher::new(&cfg);

    // 创建一个关闭信号通道
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // 监听 Ctrl+C 信号
    tokio::spawn(async move {
        if signal::ctrl_c().await.is_ok() {
            println!("\n🛑 Received Ctrl+C, stopping watcher...");
            let _ = shutdown_tx.send(());
        }
    });
    // let re = String::from(r"^out.*\.log$");
    // 启动监控（直到接收到关闭信号）
    watcher.run(shutdown_rx, &cfg.file_pattern).await?;

    println!("👋 Service stopped gracefully.");
    Ok(())
}
