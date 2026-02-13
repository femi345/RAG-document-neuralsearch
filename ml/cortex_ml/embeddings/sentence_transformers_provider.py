import logging
from sentence_transformers import SentenceTransformer

from .base import EmbeddingProvider

logger = logging.getLogger(__name__)


class SentenceTransformerProvider(EmbeddingProvider):
    def __init__(self, model_name: str = "all-MiniLM-L6-v2", device: str = "cpu"):
        logger.info(f"Loading embedding model: {model_name} on {device}")
        self.model = SentenceTransformer(model_name, device=device)
        self._dimensions = self.model.get_sentence_embedding_dimension()
        logger.info(f"Model loaded. Dimensions: {self._dimensions}")

    def embed(self, texts: list[str]) -> list[list[float]]:
        embeddings = self.model.encode(texts, convert_to_numpy=True, normalize_embeddings=True)
        return embeddings.tolist()

    def dimensions(self) -> int:
        return self._dimensions
