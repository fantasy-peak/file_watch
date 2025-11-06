use anyhow::Result;
use std::path::Path;

pub fn process_line(path: &Path, line: &str) -> Result<()> {
    println!("🪶 [{}] {}", path.display(), line);

    if line.contains("ERROR") {
        eprintln!("🚨 Error detected in {}: {}", path.display(), line);
    }

    // 这里可以扩展更多业务逻辑，比如：
    // - 发送到 Kafka
    // - 写入数据库
    // - 触发异步任务

    Ok(())
}
