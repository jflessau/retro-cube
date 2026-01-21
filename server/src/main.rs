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
use tracing::{debug, info, warn};
use tracing_subscriber::prelude::*;

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
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info,server=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let username = env::var("AUTH_USERNAME").expect("AUTH_USERNAME not set");
    let password = env::var("AUTH_PASSWORD").expect("AUTH_PASSWORD not set");

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

    info!("Server running on http://0.0.0.0:3000");
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

    if let Some(auth_value) = auth_header
        && let Some(encoded) = auth_value.strip_prefix("Basic ")
        && let Ok(decoded) = BASE64.decode(encoded)
        && let Ok(credentials) = String::from_utf8(decoded)
    {
        let parts: Vec<&str> = credentials.splitn(2, ':').collect();
        if parts.len() == 2 {
            if parts[0] == state.username && parts[1] == state.password {
                return next.run(req).await;
            } else {
                warn!(
                    "Failed authentication attempt - username: '{}', password: '{}'",
                    parts[0], parts[1]
                );
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
    debug!("Message read: {}", message);
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
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: monospace;
            background: #000;
            color: #fff;
            padding: 20px;
            min-height: 100vh;
        }}
        .container {{
            max-width: 600px;
            margin: 0 auto;
        }}
        h1 {{
            font-size: 18px;
            font-weight: normal;
            margin-bottom: 20px;
            border-bottom: 1px solid #333;
            padding-bottom: 10px;
        }}
        form {{
            display: flex;
            flex-direction: column;
            gap: 20px;
        }}
        label {{
            display: block;
            margin-bottom: 5px;
            color: #999;
        }}
        input, textarea {{
            width: 100%;
            padding: 10px;
            background: #000;
            color: #fff;
            border: 1px solid #333;
            font-family: monospace;
            font-size: 14px;
        }}
        input:focus, textarea:focus {{
            outline: none;
            border-color: #666;
        }}
        textarea {{
            resize: vertical;
            min-height: 100px;
        }}
        button {{
            padding: 10px 20px;
            background: #fff;
            color: #000;
            border: none;
            font-family: monospace;
            cursor: pointer;
            width: fit-content;
        }}
        button:hover {{
            background: #ccc;
        }}
        .char-count {{
            text-align: right;
            font-size: 12px;
            color: #666;
            margin-top: 5px;
        }}
        .current {{
            background: #000;
            padding: 15px;
            border: 1px solid #333;
            margin-bottom: 30px;
        }}
        .current-label {{
            color: #999;
            margin-bottom: 10px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>update message</h1>

        <div class="current">
            <div class="current-label">current:</div>
            <div>{}</div>
        </div>

        <form method="POST" action="/form">
            <div>
                <label for="message">message (max 144 characters)</label>
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

            <button type="submit">update</button>
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
    message: String,
}

async fn update_message(State(state): State<AppState>, Form(form): Form<MessageForm>) -> Response {
    // Validate message length
    if form.message.len() > MAX_MESSAGE_LENGTH {
        return (
            StatusCode::BAD_REQUEST,
            Html(format!(
                "<style>body{{font-family:monospace;background:#000;color:#fff;padding:20px;}}a{{color:#fff;}}</style><h1>message too long</h1><p>maximum {} characters allowed</p><a href=\"/form\">try again</a>",
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
                "<style>body{{font-family:monospace;background:#000;color:#fff;padding:20px;}}a{{color:#fff;}}</style><h1>error saving message</h1><p>{}</p><a href=\"/form\">try again</a>",
                e
            )),
        )
            .into_response();
    }

    // Update in-memory state
    let mut message = state.message.write().await;
    let old_message = message.clone();
    *message = trimmed_message.clone();
    info!(
        "Message changed from '{}' to '{}'",
        old_message, trimmed_message
    );

    Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Message Updated</title>
    <style>
        body {
            font-family: monospace;
            background: #000;
            color: #fff;
            padding: 20px;
            max-width: 600px;
            margin: 0 auto;
        }
        .success {
            border: 1px solid #333;
            padding: 20px;
            margin-bottom: 20px;
        }
        a {
            color: #fff;
            text-decoration: none;
            padding: 10px 20px;
            background: #fff;
            color: #000;
            display: inline-block;
        }
        a:hover {
            background: #ccc;
        }
    </style>
</head>
<body>
    <div class="success">
        <h1>message updated</h1>
    </div>
    <a href="/form">update again</a>
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
