use axum::{
    Form, Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use serde::Deserialize;
use std::{env, fs, path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

const MESSAGE_FILE: &str = "message.txt";
const MAX_MESSAGE_LENGTH: usize = 144;

#[derive(Clone)]
struct AppState {
    username: String,
    password: String,
    message_path: PathBuf,
    message: Arc<RwLock<String>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let username = env::var("AUTH_USERNAME").unwrap_or_else(|_| {
        eprintln!("WARNING: AUTH_USERNAME not set, using default 'admin'");
        "admin".to_string()
    });

    let password = env::var("AUTH_PASSWORD").unwrap_or_else(|_| {
        eprintln!("WARNING: AUTH_PASSWORD not set, using default 'password'");
        "password".to_string()
    });

    let message_path = PathBuf::from(MESSAGE_FILE);

    // Load existing message or create default
    let initial_message = if message_path.exists() {
        fs::read_to_string(&message_path).unwrap_or_else(|_| "No message yet".to_string())
    } else {
        "No message yet".to_string()
    };

    let state = AppState {
        username,
        password,
        message_path,
        message: Arc::new(RwLock::new(initial_message)),
    };

    let app = Router::new()
        .route("/", get(get_message))
        .route("/form", get(show_form).post(update_message))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn auth_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(auth_value) = auth_header {
        if auth_value.starts_with("Basic ") {
            let encoded = &auth_value[6..];
            if let Ok(decoded) = BASE64.decode(encoded) {
                if let Ok(credentials) = String::from_utf8(decoded) {
                    let parts: Vec<&str> = credentials.splitn(2, ':').collect();
                    if parts.len() == 2 && parts[0] == state.username && parts[1] == state.password
                    {
                        return next.run(req).await;
                    }
                }
            }
        }
    }

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::WWW_AUTHENTICATE, "Basic realm=\"Message Server\"")
        .body(Body::from("Unauthorized"))
        .unwrap()
}

async fn get_message(State(state): State<AppState>) -> String {
    let message = state.message.read().await;
    message.clone()
}

async fn show_form(State(state): State<AppState>) -> Html<String> {
    let current_message = state.message.read().await;
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Update Message</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        .container {{
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{
            margin-top: 0;
            color: #333;
        }}
        form {{
            display: flex;
            flex-direction: column;
            gap: 15px;
        }}
        label {{
            font-weight: 600;
            color: #555;
        }}
        input, textarea {{
            padding: 10px;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-size: 14px;
            font-family: inherit;
        }}
        textarea {{
            resize: vertical;
            min-height: 100px;
        }}
        button {{
            padding: 12px 20px;
            background: #007bff;
            color: white;
            border: none;
            border-radius: 4px;
            font-size: 16px;
            font-weight: 600;
            cursor: pointer;
        }}
        button:hover {{
            background: #0056b3;
        }}
        .char-count {{
            text-align: right;
            font-size: 12px;
            color: #666;
            margin-top: -10px;
        }}
        .current {{
            background: #f8f9fa;
            padding: 15px;
            border-radius: 4px;
            margin-bottom: 20px;
            border-left: 4px solid #007bff;
        }}
        .current-label {{
            font-weight: 600;
            color: #555;
            margin-bottom: 8px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Update Message</h1>

        <div class="current">
            <div class="current-label">Current Message:</div>
            <div>{}</div>
        </div>

        <form method="POST" action="/form">
            <div>
                <label for="username">Username:</label>
                <input type="text" id="username" name="username" required>
            </div>

            <div>
                <label for="password">Password:</label>
                <input type="password" id="password" name="password" required>
            </div>

            <div>
                <label for="message">Message (max 144 characters):</label>
                <textarea
                    id="message"
                    name="message"
                    maxlength="{}"
                    required
                    oninput="updateCharCount()"
                ></textarea>
                <div class="char-count">
                    <span id="charCount">0</span> / {} characters
                </div>
            </div>

            <button type="submit">Update Message</button>
        </form>
    </div>

    <script>
        function updateCharCount() {{
            const textarea = document.getElementById('message');
            const charCount = document.getElementById('charCount');
            charCount.textContent = textarea.value.length;
        }}
    </script>
</body>
</html>"#,
        html_escape(&current_message),
        MAX_MESSAGE_LENGTH,
        MAX_MESSAGE_LENGTH
    );
    Html(html)
}

#[derive(Deserialize)]
struct MessageForm {
    username: String,
    password: String,
    message: String,
}

async fn update_message(State(state): State<AppState>, Form(form): Form<MessageForm>) -> Response {
    // Verify credentials
    if form.username != state.username || form.password != state.password {
        return (
            StatusCode::UNAUTHORIZED,
            Html("<h1>Invalid credentials</h1><a href=\"/form\">Try again</a>"),
        )
            .into_response();
    }

    // Validate message length
    if form.message.len() > MAX_MESSAGE_LENGTH {
        return (
            StatusCode::BAD_REQUEST,
            Html(format!(
                "<h1>Message too long</h1><p>Maximum {} characters allowed</p><a href=\"/form\">Try again</a>",
                MAX_MESSAGE_LENGTH
            )),
        )
            .into_response();
    }

    let trimmed_message = form.message.trim().to_string();

    // Save to disk
    if let Err(e) = fs::write(&state.message_path, &trimmed_message) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!(
                "<h1>Error saving message</h1><p>{}</p><a href=\"/form\">Try again</a>",
                e
            )),
        )
            .into_response();
    }

    // Update in-memory state
    let mut message = state.message.write().await;
    *message = trimmed_message;

    Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Message Updated</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
            text-align: center;
        }
        .success {
            background: #d4edda;
            color: #155724;
            padding: 20px;
            border-radius: 8px;
            border: 1px solid #c3e6cb;
        }
        a {
            display: inline-block;
            margin-top: 20px;
            padding: 10px 20px;
            background: #007bff;
            color: white;
            text-decoration: none;
            border-radius: 4px;
        }
        a:hover {
            background: #0056b3;
        }
    </style>
</head>
<body>
    <div class="success">
        <h1>✓ Message Updated Successfully!</h1>
    </div>
    <a href="/form">Update Another Message</a>
</body>
</html>"#,
    )
    .into_response()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
