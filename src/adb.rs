use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;

use crate::parser::LogEntry;

pub enum AdbMessage {
    Entry(LogEntry),
    UnparsedLine,
    Disconnected(String),
}

pub struct AdbHandle {
    child: Option<Child>,
}

impl AdbHandle {
    pub fn kill(&mut self) {
        if let Some(ref mut child) = self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.child = None;
    }
}

impl Drop for AdbHandle {
    fn drop(&mut self) {
        self.kill();
    }
}

#[cfg(windows)]
fn hide_window(cmd: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x08000000) // CREATE_NO_WINDOW
}

#[cfg(not(windows))]
fn hide_window(cmd: &mut Command) -> &mut Command {
    cmd
}

pub fn spawn_logcat(tx: mpsc::Sender<AdbMessage>) -> Result<AdbHandle, String> {
    let mut cmd = Command::new("adb");
    cmd.args(["logcat", "-v", "threadtime"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_window(&mut cmd);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn adb: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture adb stdout")?;

    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    // Strip trailing \r (Windows ADB outputs \r\n)
                    let line = line.trim_end_matches('\r').to_string();
                    let msg = if let Some(entry) = LogEntry::parse(&line) {
                        AdbMessage::Entry(entry)
                    } else {
                        AdbMessage::UnparsedLine
                    };
                    if tx.send(msg).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let _ = tx.send(AdbMessage::Disconnected(format!("ADB read error: {}", e)));
                    break;
                }
            }
        }
        let _ = tx.send(AdbMessage::Disconnected("ADB process ended".to_string()));
    });

    Ok(AdbHandle { child: Some(child) })
}

pub fn list_devices() -> Vec<String> {
    let mut cmd = Command::new("adb");
    cmd.args(["devices", "-l"]);
    hide_window(&mut cmd);

    match cmd.output() {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            text.lines()
                .skip(1)
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.to_string())
                .collect()
        }
        Err(_) => vec!["Failed to run adb devices".to_string()],
    }
}

pub fn get_package_pid(package: &str) -> Option<u32> {
    let mut cmd = Command::new("adb");
    cmd.args(["shell", &format!("pidof {}", package)]);
    hide_window(&mut cmd);

    let output = cmd.output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.trim().split_whitespace().next()?.parse().ok()
}

pub fn clear_buffer() -> Result<(), String> {
    let mut cmd = Command::new("adb");
    cmd.args(["logcat", "-c"]);
    hide_window(&mut cmd);

    cmd.output()
        .map_err(|e| format!("Failed to clear logcat: {}", e))?;
    Ok(())
}
