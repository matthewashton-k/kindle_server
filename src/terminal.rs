use std::{process::{ Stdio}, io::Read, net::SocketAddr};
use axum::{extract::{WebSocketUpgrade, ws::{WebSocket, Message  }, ConnectInfo}, response::IntoResponse};
use futures_util::{StreamExt, SinkExt};
use tokio::{sync::mpsc, process::Command, io::{AsyncReadExt, AsyncWriteExt}};

pub async fn terminal_handler(ws: WebSocketUpgrade, ConnectInfo(addr): ConnectInfo<SocketAddr>,) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut child = match Command::new("/bin/sh")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Failed to spawn shell: {}", e);
            return;
        }
    };

    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();
    let mut child_stderr = child.stderr.take().unwrap();

    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut buffer = [0u8; 1024];
        loop {
            match child_stderr.read(&mut buffer).await {
                Ok(0) => break, // EOF reached
                Ok(n) => {
                    if tx_clone.send(buffer[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error reading shell output: {}", e);
                    break;
                }
            }
        }
    });
    
    
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut buffer = [0u8; 1024];
        loop {
            match child_stdout.read(&mut buffer).await {
                Ok(0) => break, // EOF reached
                Ok(n) => {
                    if tx_clone.send(buffer[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error reading shell output: {}", e);
                    break;
                }
            }
        }
    });

    let (mut ws_sender, mut ws_rx) = socket.split();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender
                .send(Message::Text(String::from_utf8_lossy(&msg).to_string().into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = ws_rx.next().await {
        match msg {
            Message::Text(text) => {
                if let Err(e) = child_stdin.write_all(text.as_bytes()).await {
                    eprintln!("Failed to write to shell stdin: {}", e);
                    break;
                }
            }
            Message::Binary(bin) => {
                if let Err(e) = child_stdin.write_all(&bin).await {
                    eprintln!("Failed to write binary to shell stdin: {}", e);
                    break;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}
