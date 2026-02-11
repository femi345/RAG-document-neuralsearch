use cortex_common::config::AppConfig;
use cortex_ingestion::pipeline::IngestionPipeline;
use cortex_ml_client::MlClient;
use cortex_scheduler::WorkerPool;
use cortex_store::postgres::PostgresStore;
use cortex_store::weaviate::WeaviateStore;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub postgres: PostgresStore,
    pub weaviate: WeaviateStore,
    pub ml_client: MlClient,
    pub worker_pool: Arc<WorkerPool>,
}

impl AppState {
    pub async fn new(config: &AppConfig) -> anyhow::Result<Self> {
        tracing::info!("Connecting to PostgreSQL...");
        let postgres = PostgresStore::connect(&config.database_url).await?;
        postgres.run_migrations().await?;
        tracing::info!("PostgreSQL connected and migrations applied");

        let weaviate = WeaviateStore::new(&config.weaviate_url);
        tracing::info!("Weaviate client configured at {}", config.weaviate_url);

        tracing::info!("Connecting to ML service at {}...", config.ml_service_url);
        let ml_client = MlClient::connect(&config.ml_service_url).await?;
        tracing::info!("ML service connected");

        let pipeline = Arc::new(IngestionPipeline::new(
            postgres.clone(),
            weaviate.clone(),
            ml_client.clone(),
        ));

        let worker_pool = Arc::new(WorkerPool::spawn(4, pipeline, postgres.clone()));
        tracing::info!("Worker pool started with 4 workers");

        Ok(Self {
            postgres,
            weaviate,
            ml_client,
            worker_pool,
        })
    }
}
