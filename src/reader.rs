use crate::processor::process_line;
use anyhow::Result;
use regex::Regex;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Mutex,
};

#[derive(Default, Debug)]
pub struct FileState {
    pub offset: u64,
    pub buffer: String,
}

pub struct FileReader {
    states: Mutex<HashMap<PathBuf, FileState>>,
    re: Regex,
}

impl FileReader {
    pub fn new(str: &str) -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
            re: Regex::new(str).unwrap(),
        }
    }

    /// 启动时读取已有文件
    pub fn read_existing(&self, dir: &str) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if self.re.is_match(filename) {
                    self.read_incremental(&path)?;
                }
            }
        }
        Ok(())
    }

    /// 增量读取文件
    pub fn read_incremental(&self, path: &Path) -> Result<()> {
        let mut file = File::open(path)?;

        let mut map = match self.states.lock() {
            Ok(m) => m,
            Err(poisoned) => {
                eprintln!("⚠️ Mutex poisoned while reading {:?}", path);
                poisoned.into_inner()
            }
        };

        let state = map.entry(path.to_path_buf()).or_default();
        file.seek(SeekFrom::Start(state.offset))?;

        let mut new_data = String::new();
        let bytes_read = file.read_to_string(&mut new_data)?;
        if bytes_read == 0 {
            return Ok(());
        }

        state.offset += bytes_read as u64;
        state.buffer.push_str(&new_data);

        // 按行分割处理
        while let Some(pos) = state.buffer.find('\n') {
            let line = state.buffer[..pos].trim_end().to_string();
            if let Err(e) = process_line(path, &line) {
                eprintln!("⚠️ process_line failed in {:?}: {}", path, e);
            }
            state.buffer.drain(..=pos);
        }

        Ok(())
    }
}
