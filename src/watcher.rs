use crate::reader::FileReader;
use anyhow::Result;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{mpsc, oneshot};
use tokio::task;

pub struct FileWatcher {
    dir: String,
}

impl FileWatcher {
    pub fn new(dir: &str) -> Self {
        Self {
            dir: dir.to_string(),
        }
    }

    pub async fn run(&self, mut shutdown_rx: oneshot::Receiver<()>, re:&str) -> Result<()> {
        let states = Arc::new(FileReader::new(re));
        let (tx, mut rx) = mpsc::channel(100);

        let dir_clone = self.dir.clone();
        let tx_clone = tx.clone();
        let exit_flag = Arc::new(AtomicBool::new(false));
        let exit_flag_clone = exit_flag.clone();
        task::spawn_blocking(move || -> Result<()> {
            let mut watcher = RecommendedWatcher::new(
                move |res| {
                    if let Ok(event) = res {
                        let _ = tx_clone.blocking_send(event);
                    }
                },
                Config::default(),
            )?;

            watcher.watch(Path::new(&dir_clone), RecursiveMode::Recursive)?;

            while !exit_flag_clone.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_secs(4));
            }
            Ok(())
        });

        println!("📡 Watching directory: {}", self.dir);

        // 启动时读取已有文件
        if let Err(e) = states.read_existing(&self.dir) {
            eprintln!("⚠️ Failed to read existing files: {}", e);
        }

        loop {
            tokio::select! {
                maybe_event = rx.recv() => {
                    match maybe_event {
                        Some(event) => {
                            for path in event.paths {
                                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                                    let re = Regex::new(r"^out.*\.log$").unwrap();
                                    if re.is_match(filename) {
                                        match event.kind {
                                            EventKind::Modify(_) => {
                                                if let Err(e) = states.read_incremental(&path) {
                                                    eprintln!("⚠️ Failed to read {:?}: {}", path, e);
                                                }
                                            }
                                            EventKind::Create(_) => {
                                                println!("➕ File created: {:?}", path);
                                                // 启动时读取新文件的已有内容
                                                // if let Err(e) = states.read_incremental(&path) {
                                                //     eprintln!("⚠️ Failed to read new file {:?}: {}", path, e);
                                                // }
                                            }
                                            EventKind::Remove(_) => {
                                                println!("➖ File deleted: {:?}", path);
                                                // 可选：清理缓存状态
                                                // let mut map = states.states.lock().unwrap();
                                                // map.remove(&path);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        None => break,
                    }
                }
                _ = &mut shutdown_rx => {
                    println!("🧹 Shutting down watcher...");
                    exit_flag.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }

        Ok(())
    }
}
