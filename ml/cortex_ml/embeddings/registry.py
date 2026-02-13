from .base import EmbeddingProvider
from .sentence_transformers_provider import SentenceTransformerProvider

_providers: dict[str, EmbeddingProvider] = {}


def get_embedding_provider(model_name: str, device: str = "cpu") -> EmbeddingProvider:
    if model_name not in _providers:
        _providers[model_name] = SentenceTransformerProvider(model_name, device)
    return _providers[model_name]
