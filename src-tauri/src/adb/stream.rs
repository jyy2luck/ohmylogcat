use crate::parser::{is_continuation_line, parse_threadtime_line, LogEntry};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::async_runtime;
use tokio::io::AsyncBufReadExt;

const MAX_LINE_LEN: usize = 4096;
const MAX_MESSAGE_LEN: usize = 4096;

/// Spawn an async task that runs `adb -s <serial> logcat -v threadtime`,
/// parses lines, and calls `on_entry` for each parsed entry.
///
/// Handles multi-line continuations (stack traces, etc.) by appending
/// continuation lines to the previous entry's message.
pub fn spawn_logcat_stream(
    adb_path: String,
    serial: String,
    stop_flag: Arc<AtomicBool>,
    on_entry: impl Fn(LogEntry) + Send + 'static,
    on_error: impl Fn(String) + Send + 'static,
) {
    async_runtime::spawn(async move {
        let mut child = match tokio::process::Command::new(&adb_path)
            .args(["-s", &serial, "logcat", "-v", "threadtime"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                on_error(format!("Failed to spawn adb logcat: {}", e));
                return;
            }
        };

        let stdout = child.stdout.take().unwrap();
        let reader = tokio::io::BufReader::new(stdout);
        let mut lines = reader.lines();

        // Buffer for continuation lines
        let mut pending_continuation: Option<String> = None;

        while !stop_flag.load(Ordering::Relaxed) {
            tokio::select! {
                line_result = lines.next_line() => {
                    match line_result {
                        Ok(Some(line)) => {
                            let line = truncate_line(&line);

                            if !is_continuation_line(&line) {
                                // New log line — flush any pending continuation first
                                if let Some(buf) = pending_continuation.take() {
                                    if let Some(entry) = parse_threadtime_line(&buf) {
                                        on_entry(entry);
                                    }
                                }

                                // Store this line; it will be emitted after we see the next
                                // new log line or when the stream ends (or immediately if
                                // the line is malformed — handled below)
                                pending_continuation = Some(line);
                            } else {
                                // Continuation line — append to pending message
                                if let Some(ref mut buf) = pending_continuation {
                                    if buf.len() + line.len() + 1 < MAX_MESSAGE_LEN {
                                        buf.push('\n');
                                        buf.push_str(&line);
                                    }
                                    // If buffer exceeds MAX_MESSAGE_LEN, discard extra
                                }
                                // If no pending entry, this is leading garbage — skip
                            }
                        }
                        Ok(None) => break, // EOF
                        Err(e) => {
                            on_error(format!("Logcat read error: {}", e));
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
                    // Periodic wake-up to check stop flag
                }
            }
        }

        // Flush any remaining pending entry
        if let Some(buf) = pending_continuation.take() {
            if let Some(entry) = parse_threadtime_line(&buf) {
                on_entry(entry);
            }
        }

        let _ = child.kill().await;
    });
}

/// Truncate a line if it exceeds MAX_LINE_LEN bytes.
fn truncate_line(line: &str) -> String {
    if line.len() > MAX_LINE_LEN {
        let mut truncated = line[..MAX_LINE_LEN].to_string();
        truncated.push_str("... [truncated]");
        truncated
    } else {
        line.to_string()
    }
}
