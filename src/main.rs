mod watcher;
mod reader;
mod processor;

use std::process::exit;

use anyhow::Result;
use watcher::FileWatcher;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    let dir = "./";
    // std::fs::create_dir_all(dir)?;

    let watcher = FileWatcher::new(dir);

    // 创建一个关闭信号通道
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // 监听 Ctrl+C 信号
    tokio::spawn(async move {
        if signal::ctrl_c().await.is_ok() {
            println!("\n🛑 Received Ctrl+C, stopping watcher...");
            let _ = shutdown_tx.send(());
        }
    });
    let re = String::from(r"^out.*\.log$");
    // 启动监控（直到接收到关闭信号）
    watcher.run(shutdown_rx, &re).await?;

    println!("👋 Service stopped gracefully.");
    Ok(())
}
