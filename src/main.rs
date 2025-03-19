use axum::{
    extract::{Form, Path},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use mime_guess::mime;
use serde::Deserialize;
use todos::SavedTodos;
use tokio::fs;
use std::{
    net::{SocketAddr, TcpStream},
    os::fd::{IntoRawFd, FromRawFd},
    process::{Command, Stdio}, path::PathBuf,
};
use tower_http::{trace::TraceLayer, services::ServeDir};

use crate::terminal::terminal_handler;

mod terminal;
mod todos;

#[derive(Debug, Deserialize)]
struct ReverseShellQuery {
    ip: String,
    port: u16,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/terminal", get(terminal_handler))
        .route("/webterminal", get(web_term))
        .route("/sysinfo", get(sysinfo))
        .route("/jailbreak", get(jailbreak_info))
        .route("/reverse_shell", post(reverse_shell))
        .route("/files/{*path}", get(file_browser))
        .route("/todos", get(get_todos))
        .route("/todos/add", post(add_todo))
        .route("/todos/check/{index}", post(check_todo))
        .route("/todos/remove/{index}", post(remove_todo))
        .route("/todos/json", get(get_todos_json))
        .route("/todos/push_google", post(push_to_google))
        .route("/todos/pull_google", post(pull_from_google))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("couldn't bind to port 3000");
    println!("Listening on {}", addr);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("failed to serve");
}

async fn web_term() -> impl IntoResponse {
    Html(include_str!("../assets/terminal.html"))
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
            <li><a href="/webterminal">Web Terminal</a></li>
            <li><a href="/todos">Todo Manager</a></li>
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


async fn get_todos_json() -> impl IntoResponse {
    let saved_todos = load_todos().await;
    axum::Json(saved_todos.todos)
}

async fn get_todos() -> impl IntoResponse {
    let saved_todos = load_todos().await;
    
    let mut todos_html = String::new();
    for (i, todo) in saved_todos.todos.iter().enumerate() {
        todos_html.push_str(&format!(
            r#"<li style="margin: 10px 0;">
                <form method="post" action="/todos/check/{}" style="display: inline-block;">
                    <input type="checkbox" onchange="{onchange}" {checked} style="margin-right: 8px;">
                </form>
                <span style="{}">{text}</span>
                <form method="post" action="/todos/remove/{}" style="display: inline-block; margin-left: 10px;">
                    <button type="submit" style="color: red; border: none; background: none;">✕</button>
                </form>
            </li>"#,
            i,
            if todo.checked { "text-decoration: line-through;" } else { "" },
            i,
            onchange = "this.form.submit()",
            checked = if todo.checked { "checked" } else { "" },
            text = html_escape(&todo.text)
        ));
    }
    
    Html(format!(
        r#"
        <html>
            <head>
                <title>Todos</title>
                <style>
                    body {{ max-width: 600px; margin: 20px auto; padding: 0 20px; }}
                    form {{ margin: 20px 0; }}
                    ul {{ list-style: none; padding: 0; }}
                    input[type="text"] {{ padding: 8px; width: 300px; }}
                    button {{ padding: 8px 12px; }}
                    .sync-buttons {{
                        margin: 20px 0;
                        display: flex;
                        gap: 10px;
                    }}
                    .sync-buttons form {{
                        margin: 0;
                    }}
                </style>
            </head>
            <body>
                <h1>Todo List</h1>
                <div class="sync-buttons">
                    <form method="post" action="/todos/push_google">
                        <button type="submit">Push to Google</button>
                    </form>
                    <form method="post" action="/todos/pull_google">
                        <button type="submit">Pull from Google</button>
                    </form>
                </div>
                <form method="post" action="/todos/add">
                    <input type="text" name="text" placeholder="New todo" required>
                    <button type="submit">Add</button>
                </form>
                <ul>{}</ul>
                <p><a href="/">← Back to home</a></p>
            </body>
        </html>
        "#,
        todos_html
    ))
}

async fn add_todo(Form(form): Form<AddTodoForm>) -> impl IntoResponse {
    let mut saved_todos = load_todos().await;
    saved_todos.add_todo(form.text);
    saved_todos.save().unwrap();
    Redirect::to("/todos")
}

async fn check_todo(Path(index): Path<usize>) -> impl IntoResponse {
    let mut saved_todos = load_todos().await;
    let _ = saved_todos.check(index);
    saved_todos.save().unwrap();
    Redirect::to("/todos")
}

async fn remove_todo(Path(index): Path<usize>) -> impl IntoResponse {
    let mut saved_todos = load_todos().await;
    let _ = saved_todos.remove_todo(index);
    saved_todos.save().unwrap();
    Redirect::to("/todos")
}

// Helper functions
async fn load_todos() -> SavedTodos {
    let path;
    #[cfg(target_arch="x86_64")]
    {
        let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        path = format!("/home/{user}/.config/koreader/settings/todos.lua");
    }
    #[cfg(target_arch="arm")]
    {
        path = format!("/mnt/us/koreader/settings/todos.lua");
    }
    SavedTodos::load(&path).unwrap_or_else(|_| SavedTodos {
        path: "/mnt/us/koreader/settings/todos.lua".to_owned(),
        todos: Vec::new(),
    })
}

async fn push_to_google() -> impl IntoResponse {
    let saved_todos = load_todos().await;
    if let Err(e) = saved_todos.push_to_google().await {
        return Html(format!("Error pushing to Google: {}", e)).into_response();
    }
    Redirect::to("/todos").into_response()
}

async fn pull_from_google() -> impl IntoResponse {
    let mut saved_todos = match SavedTodos::load_from_google().await {
        Ok(todos) => todos,
        Err(e) => return Html(format!("Error pulling from Google: {}", e)).into_response(),
    };
    
    // Preserve the original path
    let original_path = load_todos().await.path;
    saved_todos.path = original_path;
    
    if let Err(e) = saved_todos.save() {
        return Html(format!("Error saving local todos: {}", e)).into_response();
    }
    
    Redirect::to("/todos").into_response()
}

fn html_escape(s: &str) -> String {
    ammonia::clean(s)
}

#[derive(Deserialize)]
struct AddTodoForm {
    text: String,
}

async fn file_browser(Path(path): Path<String>) -> impl IntoResponse {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Html("Error: File not found".to_string()).into_response();
    }

    if path.is_dir() {
        match handle_directory_listing(&path).await {
            Ok(html) => Html(html).into_response(),
            Err(e) => Html(format!("Error: {}", e)).into_response(),
        }
    } else {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
        let mime_type = mime_guess::from_path(&path).first_or_octet_stream();

        match fs::read(&path).await {
            Ok(content) => {
                if mime_type.type_() == mime::TEXT {
                    match String::from_utf8(content.clone()) {
                        Ok(text) => Html(format!("<pre>{}</pre>", text)).into_response(),
                        Err(_) => create_octet_response(content, filename),
                    }
                } else {
                    create_octet_response(content, filename)
                }
            }
            Err(e) => Html(format!("Error reading file: {}", e)).into_response(),
        }
    }
}

fn create_octet_response(content: Vec<u8>, filename: &str) -> axum::response::Response {
    let headers = [
        ("Content-Type", "application/octet-stream"),
        (
            "Content-Disposition",
            &format!("attachment; filename=\"{}\"", filename),
        ),
    ];

    (headers, content).into_response()
}

async fn handle_directory_listing(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut html = String::new();
    html.push_str("<h2>File Browser</h2><ul>");

    let mut entries = fs::read_dir(path).await?;
    let mut root = path.clone();
    root.pop();
    html.push_str(&format!(
        "<li>📁 <a href='/files/{}/'>{}/</a></li>",
        root.to_string_lossy(),
        ".."
    ));
    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();

        if entry_path.is_dir() {
            html.push_str(&format!(
                "<li>📁 <a href='/files/{}/'>{}/</a></li>",
                entry_path.display(),
                name
            ));
        } else {
            html.push_str(&format!(
                "<li>📄 <a href='/files/{}'>{}</a></li>",
                entry_path.display(),
                name
            ));
        }
    }

    html.push_str("</ul>");
    Ok(html)
}
