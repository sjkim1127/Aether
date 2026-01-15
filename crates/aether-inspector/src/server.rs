use rust_embed::RustEmbed;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use std::sync::Arc;
use crate::model::{Inspector, InspectorEvent};

#[derive(RustEmbed)]
#[folder = "ui/dist/"]
struct Assets;

pub struct InspectorServer {
    inspector: Arc<Inspector>,
}

impl InspectorServer {
    pub fn new(inspector: Arc<Inspector>) -> Self {
        Self { inspector }
    }

    pub async fn start(self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let app = Router::new()
            .route("/api/events", get(list_events))
            .route("/api/events/:id", get(get_event))
            .fallback(static_handler)
            .with_state(self.inspector);

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        tracing::info!("Aether Inspector UI available at http://localhost:{}", port);
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn list_events(
    State(inspector): State<Arc<Inspector>>,
) -> Json<Vec<InspectorEvent>> {
    let mut events: Vec<_> = inspector.events.iter().map(|e| e.value().clone()).collect();
    events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Json(events)
}

async fn get_event(
    Path(id): Path<String>,
    State(inspector): State<Arc<Inspector>>,
) -> Result<Json<InspectorEvent>, StatusCode> {
    inspector.events.get(&id)
        .map(|e| Json(e.value().clone()))
        .ok_or(StatusCode::NOT_FOUND)
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    
    if path.is_empty() || path == "index.html" {
        return index_html().await;
    }

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => index_html().await,
    }
}

async fn index_html() -> Response {
    match Assets::get("index.html") {
        Some(content) => Html(content.data).into_response(),
        None => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}
