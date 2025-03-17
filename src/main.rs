use axum::{
    extract::{Form, Path},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use tokio::fs;
use std::{
    net::{SocketAddr, TcpStream},
    os::fd::{IntoRawFd, FromRawFd},
    process::{Command, Stdio}, path::PathBuf,
};
use tower_http::{trace::TraceLayer, services::ServeDir};

#[derive(Debug, Deserialize)]
struct ReverseShellQuery {
    ip: String,
    port: u16,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/sysinfo", get(sysinfo))
        .route("/jailbreak", get(jailbreak_info))
        .route("/reverse_shell", post(reverse_shell))
        .route("/files/{*path}", get(file_browser))  // New file browser route
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("couldn't bind to port 3000");
    println!("Listening on {}", addr);
    axum::serve(listener, app)
        .await
        .expect("failed to serve");
}

async fn reverse_shell(Form(query): Form<ReverseShellQuery>) -> impl IntoResponse {
    // Combine ip and port into a single address string.
    let addr = format!("{}:{}", query.ip, query.port);
    let addr_clone = addr.clone();
    let result = tokio::task::spawn_blocking(move || {
        match TcpStream::connect(&addr) {
            Ok(stream) => {
                // Clone the stream for separate ownership of stdin and stdout.
                let stream_stdin = stream.try_clone().map_err(|e| e.to_string())?;
                let stream_stdout = stream.try_clone().map_err(|e| e.to_string())?;
                
                // Convert each cloned stream (and the original for stderr) to raw file descriptors.
                let stdin_fd = stream_stdin.into_raw_fd();
                let stdout_fd = stream_stdout.into_raw_fd();
                let stderr_fd = stream.into_raw_fd();

                match Command::new("/bin/sh")
                    .arg("-i")
                    .stdin(unsafe { Stdio::from_raw_fd(stdin_fd) })
                    .stdout(unsafe { Stdio::from_raw_fd(stdout_fd) })
                    .stderr(unsafe { Stdio::from_raw_fd(stderr_fd) })
                    .spawn()
                {
                    Ok(child) => {
                        // Prevent dropping of the child process handle.
                        std::mem::forget(child);
                        Ok(())
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(e.to_string()),
        }
    })
    .await;

    match result {
        Ok(Ok(())) => Html(format!("Reverse shell spawned to {}", addr_clone)),
        Ok(Err(e)) => Html(format!("Error: {}", e)),
        Err(join_err) => Html(format!("Join error: {}", join_err)),
    }
}

async fn sysinfo() -> impl IntoResponse {
    let output = Command::new("uname")
        .arg("-a")
        .output()
        .expect("failed to execute process");
    let sysinfo = String::from_utf8_lossy(&output.stdout).to_string();
    Html(format!(
        "<h2>System Information</h2><pre>{}</pre>",
        sysinfo
    ))
}

async fn jailbreak_info() -> impl IntoResponse {
    let info = r#"
        <h2>Jailbreak Info</h2>
        <ul>
            <li>Serial Number Prefix: B024</li>
            <li>Firmware version: 5.6.1.1</li>
            <li>Model Name: Kindle PaperWhite WiFi</li>
        </ul>
    "#;
    Html(info)
}



// Updated root route with file browser link
async fn root() -> impl IntoResponse {
    Html(
        r#"
        <h1>Hello from your Jailbroken Kindle!</h1>
        <ul>
            <li><a href="/sysinfo">System Information</a></li>
            <li><a href="/jailbreak">Jailbreak Details</a></li>
            <li><a href="/files//">File Browser</a></li>
        </ul>
        <h2>Reverse Shell</h2>
        <form method="post" action="/reverse_shell">
            <input type="text" name="ip" placeholder="IP Address" required>
            <input type="number" name="port" placeholder="Port" required>
            <input type="submit" value="Spawn Reverse Shell">
        </form>
        "#,
    )
}

// New file browser handler
async fn file_browser(Path(path): Path<String>) -> impl IntoResponse {
    let path = PathBuf::from(path);
    match handle_directory_listing(&path).await {
        Ok(html) => Html(html),
        Err(e) => Html(format!("Error: {}", e)),
    }
}

async fn handle_directory_listing(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut html = String::new();
    
    html.push_str("<h2>File Browser</h2>");
    html.push_str("<ul>");  
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            
            if entry_path.is_dir() {
                html.push_str(&format!(
                    "<li>📁 <a href='/files/{}/'>{}/</a></li>",
                    entry_path.to_string_lossy(), name
                ));
            } else {
                html.push_str(&format!(
                    "<li>📄 <a href='/files/{}'>{}</a></li>",
                    entry_path.to_string_lossy(), name
                ));
            }
        }
    } else {
        return Ok(format!("File content:<pre>{}</pre>", std::fs::read_to_string(path)?));
    }

    html.push_str("</ul>");
    Ok(html)
}
