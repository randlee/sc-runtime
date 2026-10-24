//! Tiny HTTP server for client integration tests.

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
#[cfg(unix)]
use tokio::net::UnixListener;

pub fn spawn_tcp(listener: TcpListener) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(handle(stream));
        }
    })
}

#[cfg(unix)]
pub fn spawn_unix(listener: UnixListener) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(handle(stream));
        }
    })
}

async fn handle<S>(mut stream: S)
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut buf = Vec::new();
    let mut tmp = [0_u8; 1024];
    let header_end = loop {
        let n = match stream.read(&mut tmp).await {
            Ok(0) | Err(_) => return,
            Ok(n) => n,
        };
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = buf.windows(4).position(|window| window == b"\r\n\r\n") {
            break pos;
        }
        if buf.len() > 65_536 {
            return;
        }
    };
    let header = String::from_utf8_lossy(&buf[..header_end]).into_owned();
    let length = content_length(&header);
    let mut body = buf[header_end + 4..].to_vec();
    while body.len() < length {
        let n = match stream.read(&mut tmp).await {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        body.extend_from_slice(&tmp[..n]);
    }
    body.truncate(length);
    let path = request_path(&header);
    if path == "/hang" {
        std::future::pending::<()>().await;
    }
    let (status, payload) = match path.as_str() {
        "/echo" => (200, body),
        "/doc" => (200, br#"{"name":"fixed"}"#.to_vec()),
        "/bad" => (200, b"not json".to_vec()),
        "/missing" => (404, br#"{"name":"missing"}"#.to_vec()),
        _ => (404, b"not json".to_vec()),
    };
    let reason = if status == 200 { "OK" } else { "Not Found" };
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    let _ = stream.write_all(head.as_bytes()).await;
    let _ = stream.write_all(&payload).await;
}

fn request_path(header: &str) -> String {
    header
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/")
        .to_owned()
}

fn content_length(header: &str) -> usize {
    header
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse().ok()
            } else {
                None
            }
        })
        .unwrap_or(0)
}
