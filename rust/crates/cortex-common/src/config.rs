use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default = "default_database_url")]
    pub database_url: String,
    #[serde(default = "default_weaviate_url")]
    pub weaviate_url: String,
    #[serde(default = "default_ml_service_url")]
    pub ml_service_url: String,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_database_url() -> String {
    "postgres://cortex:cortex@localhost:5432/cortex".to_string()
}

fn default_weaviate_url() -> String {
    "http://localhost:8081".to_string()
}

fn default_ml_service_url() -> String {
    "http://localhost:50051".to_string()
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

impl AppConfig {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}
