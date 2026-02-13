import logging
from sentence_transformers import CrossEncoder

from .base import RankedDocument, Reranker

logger = logging.getLogger(__name__)


class CrossEncoderReranker(Reranker):
    def __init__(
        self, model_name: str = "cross-encoder/ms-marco-MiniLM-L-12-v2", device: str = "cpu"
    ):
        logger.info(f"Loading reranker model: {model_name} on {device}")
        self.model = CrossEncoder(model_name, device=device)
        logger.info("Reranker model loaded")

    def rerank(
        self, query: str, documents: list[RankedDocument], top_k: int
    ) -> list[RankedDocument]:
        if not documents:
            return []

        pairs = [(query, doc.text) for doc in documents]
        scores = self.model.predict(pairs)

        for doc, score in zip(documents, scores):
            doc.score = float(score)

        documents.sort(key=lambda d: d.score, reverse=True)
        return documents[:top_k]
