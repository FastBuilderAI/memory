use axum::{routing::get, Router, extract::{Query, State}};
use std::sync::Arc;

#[derive(serde::Deserialize)]
struct SearchQuery {
    q: String,
}

pub async fn start_server(memory_json: String, port: u16) {
    let app = Router::new()
        .route("/query", get(query_handler))
        .route("/components", get(components_handler))
        .with_state(Arc::new(memory_json));
    
    let addr = format!("0.0.0.0:{}", port);
    eprintln!("Starting FastMemory REST API on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn query_handler(State(memory_json): State<Arc<String>>, Query(params): Query<SearchQuery>) -> axum::response::Json<serde_json::Value> {
    let result = crate::query::search_memory(&memory_json, &params.q);
    let val: serde_json::Value = serde_json::from_str(&result).unwrap_or(serde_json::Value::Null);
    axum::response::Json(val)
}

async fn components_handler(State(memory_json): State<Arc<String>>) -> axum::response::Json<serde_json::Value> {
    let val: serde_json::Value = serde_json::from_str(&memory_json).unwrap_or(serde_json::Value::Null);
    axum::response::Json(val)
}
