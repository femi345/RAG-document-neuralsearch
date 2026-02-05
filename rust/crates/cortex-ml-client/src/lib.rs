pub mod proto {
    tonic::include_proto!("cortex.ml");
}

use proto::ml_service_client::MlServiceClient as GrpcClient;
use proto::{EmbedRequest, GenerateRequest, RerankDocument, RerankRequest};
use tonic::transport::Channel;

#[derive(Clone)]
pub struct MlClient {
    client: GrpcClient<Channel>,
}

impl MlClient {
    pub async fn connect(url: &str) -> Result<Self, MlClientError> {
        let client = GrpcClient::connect(url.to_string()).await?;
        Ok(Self { client })
    }

    /// Generate embeddings for a batch of texts.
    pub async fn embed_batch(
        &self,
        texts: Vec<String>,
        model: Option<&str>,
    ) -> Result<Vec<Vec<f32>>, MlClientError> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(EmbedRequest {
            texts,
            model: model.unwrap_or("all-MiniLM-L6-v2").to_string(),
        });

        let response = client.embed_batch(request).await?.into_inner();
        let embeddings = response
            .embeddings
            .into_iter()
            .map(|e| e.values)
            .collect();

        Ok(embeddings)
    }

    /// Rerank documents against a query using a cross-encoder.
    pub async fn rerank(
        &self,
        query: &str,
        documents: Vec<(String, String, f32)>, // (id, text, score)
        top_k: i32,
    ) -> Result<Vec<(String, String, f32)>, MlClientError> {
        let mut client = self.client.clone();
        let docs = documents
            .into_iter()
            .map(|(id, text, score)| RerankDocument {
                id,
                text,
                score,
                metadata: Default::default(),
            })
            .collect();

        let request = tonic::Request::new(RerankRequest {
            query: query.to_string(),
            documents: docs,
            top_k,
            model: String::new(),
        });

        let response = client.rerank(request).await?.into_inner();
        let results = response
            .documents
            .into_iter()
            .map(|d| (d.id, d.text, d.score))
            .collect();

        Ok(results)
    }

    /// Stream a generation response from the LLM.
    pub async fn generate_stream(
        &self,
        prompt: &str,
        system_prompt: &str,
        provider: &str,
        model: &str,
    ) -> Result<tonic::Streaming<proto::GenerateChunk>, MlClientError> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(GenerateRequest {
            prompt: prompt.to_string(),
            system_prompt: system_prompt.to_string(),
            provider: provider.to_string(),
            model: model.to_string(),
            temperature: 0.7,
            max_tokens: 2048,
        });

        let response = client.generate(request).await?;
        Ok(response.into_inner())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MlClientError {
    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    #[error("gRPC status error: {0}")]
    Status(#[from] tonic::Status),
}
