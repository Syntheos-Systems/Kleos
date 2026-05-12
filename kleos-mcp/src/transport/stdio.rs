use crate::{handle_jsonrpc, App};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};

/// Reads one JSON-RPC framed message from `reader` (Content-Length prefix).
fn read_message<R: BufRead>(reader: &mut R) -> Result<Option<Value>, String> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            let parsed = value
                .trim()
                .parse::<usize>()
                .map_err(|e| e.to_string())?;
            content_length = Some(parsed);
        }
    }

    let len = content_length.ok_or_else(|| "missing Content-Length header".to_string())?;

    // SECURITY (SEC-C4): cap allocation at 10 MiB to prevent OOM via a
    // malicious Content-Length value.
    const MAX_MCP_MSG_SIZE: usize = 10 * 1024 * 1024;
    if len > MAX_MCP_MSG_SIZE {
        return Err(format!(
            "Content-Length {} exceeds max {}",
            len, MAX_MCP_MSG_SIZE
        ));
    }

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
    let value = serde_json::from_slice(&buf).map_err(|e| e.to_string())?;
    Ok(Some(value))
}

/// Writes one JSON-RPC framed message (Content-Length prefix + body).
fn write_message<W: Write>(writer: &mut W, value: &Value) -> Result<(), String> {
    let body = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len()).map_err(|e| e.to_string())?;
    writer.write_all(&body).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// Runs the stdio JSON-RPC loop against the given `App` until EOF.
pub async fn serve(app: App) -> Result<(), String> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();

    while let Some(message) = read_message(&mut reader)? {
        if let Some(response) = handle_jsonrpc(&app, message).await {
            write_message(&mut writer, &response)?;
        }
    }

    Ok(())
}
