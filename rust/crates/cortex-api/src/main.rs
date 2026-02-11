use cortex_common::{config::AppConfig, telemetry};

mod error;
mod routes;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init();

    let config = AppConfig::from_env().expect("Failed to load config");
    tracing::info!("Starting Cortex API server");

    let app_state = state::AppState::new(&config).await?;

    // Ensure Weaviate schema exists
    app_state.weaviate.ensure_schema().await.map_err(|e| {
        tracing::error!("Failed to create Weaviate schema: {e}");
        anyhow::anyhow!("Weaviate schema error: {e}")
    })?;

    let app = routes::create_router(app_state);

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
