/// OpenAI-compatible API proxy server using axum.
/// Runs on a separate port (default 8000), proxying requests to audiocpp_server.
use axum::{
    body::Bytes,
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};

struct ServerState {
    cpp_host: String, // e.g. "http://127.0.0.1:8080"
    api_key: Option<String>,
}

type SharedState = Arc<Mutex<ServerState>>;

/// Start the API server. Returns Ok when the server is running.
#[tauri::command]
pub async fn start_api_server(
    port: u16,
    cpp_port: u16,
    api_key: String,
) -> Result<(), String> {
    // Build a shared state
    let state = Arc::new(Mutex::new(ServerState {
        cpp_host: format!("http://127.0.0.1:{}", cpp_port),
        api_key: if api_key.is_empty() { None } else { Some(api_key) },
    }));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/v1/models", get(models_handler))
        .route("/v1/audio/transcriptions", post(transcribe_handler))
        .route("/v1/audio/speech", post(speech_handler))
        .layer(cors)
        .with_state(state.clone());

    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Bind failed: {}", e))?;

    // Spawn in background
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    Ok(())
}

// ── Handlers ──────────────────────────────────────────────

async fn check_auth(
    state: &SharedState,
    headers: &axum::http::HeaderMap,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let s = state.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"}))))?;
    if let Some(ref key) = s.api_key {
        let auth = headers
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if auth != key {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid API key"})),
            ));
        }
    }
    Ok(())
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok", "version": "0.1.0"}))
}

async fn models_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&state, &headers).await?;
    let models = crate::model_registry::get_models_with_status(None);
    let data: Vec<_> = models
        .iter()
        .map(|m| {
            serde_json::json!({
                "id": m.name,
                "object": "model",
                "owned_by": m.id.split('/').next().unwrap_or(""),
                "category": m.category,
                "languages": m.languages,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({"object": "list", "data": data})))
}

async fn transcribe_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&state, &headers).await?;

    let cpp_host = {
        let s = state.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"}))))?;
        s.cpp_host.clone()
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/audio/transcriptions", cpp_host))
        .header("Content-Type", headers.get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("multipart/form-data"))
        .body(body)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": format!("C++ server unreachable: {}", e)})),
            )
        })?;

    let status = resp.status();
    let resp_body = resp.bytes().await.unwrap_or_default();
    let ct = "application/json";

    Ok(axum::response::Response::builder()
        .status(status.as_u16())
        .header("Content-Type", ct)
        .body(axum::body::Body::from(resp_body))
        .unwrap())
}

async fn speech_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&state, &headers).await?;

    let cpp_host = {
        let s = state.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"}))))?;
        s.cpp_host.clone()
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/audio/speech", cpp_host))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": format!("TTS server unreachable: {}", e)})),
            )
        })?;

    let status = resp.status();
    let resp_body = resp.bytes().await.unwrap_or_default();

    Ok(axum::response::Response::builder()
        .status(status.as_u16())
        .header("Content-Type", "audio/wav")
        .body(axum::body::Body::from(resp_body))
        .unwrap())
}
