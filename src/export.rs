use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::parser::LogEntry;

pub fn export_logs(entries: &[&LogEntry], dir: Option<&str>) -> Result<PathBuf, String> {
    let dir = dir.unwrap_or(".");
    let _ = fs::create_dir_all(dir);

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("logcat_{}.txt", timestamp);
    let path = PathBuf::from(dir).join(&filename);

    let mut file = fs::File::create(&path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    for entry in entries {
        writeln!(file, "{}", entry.raw)
            .map_err(|e| format!("Failed to write: {}", e))?;
    }

    Ok(path)
}
