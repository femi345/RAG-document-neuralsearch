use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}

async fn health_check(State(state): State<AppState>) -> Json<Value> {
    let weaviate_ok = state.weaviate.health_check().await.unwrap_or(false);

    let pg_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(state.postgres.pool())
        .await
        .is_ok();

    let status = if weaviate_ok && pg_ok { "ok" } else { "degraded" };

    Json(json!({
        "status": status,
        "services": {
            "postgres": if pg_ok { "ok" } else { "error" },
            "weaviate": if weaviate_ok { "ok" } else { "error" },
        }
    }))
}
