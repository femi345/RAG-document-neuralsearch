from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    grpc_port: int = 50051
    embedding_model: str = "all-MiniLM-L6-v2"
    reranker_model: str = "cross-encoder/ms-marco-MiniLM-L-12-v2"

    # LLM provider API keys
    anthropic_api_key: str = ""
    openai_api_key: str = ""
    ollama_host: str = "http://localhost:11434"

    # Device selection
    device: str = "cpu"  # "cpu", "cuda", "mps"

    model_config = {"env_prefix": "", "case_sensitive": False}


settings = Settings()
