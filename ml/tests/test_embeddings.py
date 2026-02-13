"""Tests for the embedding provider."""
import pytest


def test_sentence_transformer_embed():
    """Test that sentence-transformers produces correct-dimensioned embeddings."""
    from cortex_ml.embeddings.sentence_transformers_provider import SentenceTransformerProvider

    provider = SentenceTransformerProvider("all-MiniLM-L6-v2", device="cpu")

    texts = ["Hello world", "This is a test document about machine learning"]
    embeddings = provider.embed(texts)

    assert len(embeddings) == 2
    assert len(embeddings[0]) == 384  # MiniLM-L6 produces 384-dim vectors
    assert len(embeddings[1]) == 384
    assert provider.dimensions() == 384

    # Embeddings should be normalized (L2 norm ≈ 1.0)
    import math
    norm = math.sqrt(sum(v**2 for v in embeddings[0]))
    assert abs(norm - 1.0) < 0.01


def test_similar_texts_have_high_similarity():
    """Test that semantically similar texts produce similar embeddings."""
    from cortex_ml.embeddings.sentence_transformers_provider import SentenceTransformerProvider

    provider = SentenceTransformerProvider("all-MiniLM-L6-v2", device="cpu")

    similar_1 = "The cat sat on the mat"
    similar_2 = "A feline was sitting on a rug"
    different = "Quantum computing uses qubits for parallel processing"

    embeddings = provider.embed([similar_1, similar_2, different])

    # Cosine similarity (vectors are normalized, so dot product = cosine sim)
    sim_similar = sum(a * b for a, b in zip(embeddings[0], embeddings[1]))
    sim_different = sum(a * b for a, b in zip(embeddings[0], embeddings[2]))

    assert sim_similar > sim_different
