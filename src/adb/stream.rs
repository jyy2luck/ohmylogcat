use crate::parser::{is_continuation_line, parse_threadtime_line, LogEntry};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt};
use tokio::runtime::Handle;

const MAX_LINE_LEN: usize = 4096;
const MAX_MESSAGE_LEN: usize = 4096;

/// Spawn an async task that runs `adb -s <serial> logcat -v threadtime`,
/// parses lines, and calls `on_entry` for each parsed entry.
///
/// Handles multi-line continuations (stack traces, etc.) by appending
/// continuation lines to the previous entry's message.
///
/// Reads raw bytes and lossy-decodes UTF-8 so binary / invalid sequences in
/// log messages cannot kill the stream.
pub fn spawn_logcat_stream(
    handle: Handle,
    adb_path: String,
    serial: String,
    stop_flag: Arc<AtomicBool>,
    on_entry: impl Fn(LogEntry) + Send + 'static,
    on_error: impl Fn(String) + Send + 'static,
) {
    handle.spawn(async move {
        let mut child = match tokio::process::Command::new(&adb_path)
            .args(["-s", &serial, "logcat", "-v", "threadtime"])
            .stdout(std::process::Stdio::piped())
            // Must drain or null stderr: an unread stderr pipe eventually fills
            // (~64KB) and adb blocks forever — count freezes with no EOF/error.
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                on_error(format!("Failed to spawn adb logcat: {}", e));
                return;
            }
        };

        // Continuously drain stderr so the pipe never blocks adb.
        if let Some(mut stderr) = child.stderr.take() {
            let stop = stop_flag.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                while !stop.load(Ordering::Relaxed) {
                    match stderr.read(&mut buf).await {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {}
                    }
                }
            });
        }

        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => {
                on_error("Failed to capture adb logcat stdout".into());
                return;
            }
        };
        let mut reader = tokio::io::BufReader::new(stdout);
        let mut raw_line: Vec<u8> = Vec::with_capacity(512);

        // Buffer for continuation lines
        let mut pending_continuation: Option<String> = None;

        while !stop_flag.load(Ordering::Relaxed) {
            raw_line.clear();
            // Do not wrap this in select/timeout: cancelling mid-line would drop
            // already-consumed bytes from the BufReader.
            match reader.read_until(b'\n', &mut raw_line).await {
                Ok(0) => {
                    if !stop_flag.load(Ordering::Relaxed) {
                        on_error(
                            "Logcat stream ended unexpectedly (adb disconnected?)".into(),
                        );
                    }
                    break;
                }
                Ok(_) => {
                    // Strip trailing \n / \r\n then lossy-decode so invalid
                    // UTF-8 becomes � instead of aborting the stream.
                    while raw_line.last().copied() == Some(b'\n')
                        || raw_line.last().copied() == Some(b'\r')
                    {
                        raw_line.pop();
                    }
                    let line = truncate_line(&String::from_utf8_lossy(&raw_line));

                    if !is_continuation_line(&line) {
                        if let Some(buf) = pending_continuation.take() {
                            if let Some(entry) = parse_threadtime_line(&buf) {
                                on_entry(entry);
                            }
                        }
                        pending_continuation = Some(line);
                    } else if let Some(ref mut buf) = pending_continuation {
                        if buf.len() + line.len() + 1 < MAX_MESSAGE_LEN {
                            buf.push('\n');
                            buf.push_str(&line);
                        }
                    }
                }
                Err(e) => {
                    on_error(format!("Logcat read error: {}", e));
                    break;
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
        let _ = child.wait().await;
    });
}

/// Truncate a line if it exceeds MAX_LINE_LEN bytes, without splitting UTF-8.
fn truncate_line(line: &str) -> String {
    if line.len() <= MAX_LINE_LEN {
        return line.to_string();
    }
    let mut end = MAX_LINE_LEN;
    while end > 0 && !line.is_char_boundary(end) {
        end -= 1;
    }
    let mut truncated = line[..end].to_string();
    truncated.push_str("... [truncated]");
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_ascii() {
        let long = "a".repeat(5000);
        let out = truncate_line(&long);
        assert!(out.ends_with("... [truncated]"));
        assert!(out.len() < 5000);
    }

    #[test]
    fn truncate_utf8_multibyte_safe() {
        let mut s = String::new();
        while s.len() < MAX_LINE_LEN - 1 {
            s.push('你');
        }
        s.push('好');
        s.push_str(&"x".repeat(100));
        let out = truncate_line(&s);
        assert!(out.ends_with("... [truncated]"));
    }

    #[test]
    fn lossy_decode_invalid_utf8() {
        let bytes = b"hello \xff\xfe world";
        let line = String::from_utf8_lossy(bytes);
        assert!(line.contains("hello"));
        assert!(line.contains("world"));
        // Replacement char for invalid bytes — must not panic.
        assert!(line.contains('\u{FFFD}') || line.len() >= 5);
    }
}
