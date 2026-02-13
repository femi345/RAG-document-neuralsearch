"""Tests for the cross-encoder reranker."""
import pytest
from cortex_ml.reranker.base import RankedDocument


def test_reranker_reorders_by_relevance():
    """Test that the reranker correctly orders documents by query relevance."""
    from cortex_ml.reranker.cross_encoder_reranker import CrossEncoderReranker

    reranker = CrossEncoderReranker(device="cpu")

    query = "What is machine learning?"
    documents = [
        RankedDocument(id="1", text="The weather is sunny today", score=0.5, metadata={}),
        RankedDocument(
            id="2",
            text="Machine learning is a subset of AI that enables systems to learn from data",
            score=0.3,
            metadata={},
        ),
        RankedDocument(
            id="3", text="I enjoy cooking pasta dishes", score=0.8, metadata={}
        ),
    ]

    reranked = reranker.rerank(query, documents, top_k=2)

    assert len(reranked) == 2
    # The ML-related document should be ranked first
    assert reranked[0].id == "2"
