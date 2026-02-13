"""
Cortex ML gRPC Server

Provides embedding, reranking, and LLM generation services over gRPC.
"""

import asyncio
import logging
import sys
from concurrent import futures
from pathlib import Path

import grpc
from grpc import aio

# Add proto directory to path for imports
sys.path.insert(0, str(Path(__file__).parent))

from .config import settings
from .embeddings.registry import get_embedding_provider
from .llm.registry import get_llm_provider
from .llm.base import GenerationConfig
from .reranker.base import RankedDocument
from .reranker.cross_encoder_reranker import CrossEncoderReranker

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)s %(name)s: %(message)s",
)
logger = logging.getLogger(__name__)

# Import generated protobuf stubs
from .proto import ml_service_pb2
from .proto import ml_service_pb2_grpc


class MlServiceServicer(ml_service_pb2_grpc.MlServiceServicer):
    def __init__(self):
        logger.info("Initializing ML service...")
        self.embedder = get_embedding_provider(settings.embedding_model, settings.device)
        self.reranker = CrossEncoderReranker(settings.reranker_model, settings.device)
        logger.info("ML service initialized")

    async def EmbedBatch(self, request, context):
        texts = list(request.texts)
        model = request.model or settings.embedding_model

        logger.info(f"EmbedBatch: {len(texts)} texts, model={model}")

        embedder = get_embedding_provider(model, settings.device)
        vectors = embedder.embed(texts)

        embeddings = [ml_service_pb2.Embedding(values=v) for v in vectors]
        return ml_service_pb2.EmbedResponse(
            embeddings=embeddings,
            dimensions=embedder.dimensions(),
        )

    async def Rerank(self, request, context):
        query = request.query
        top_k = request.top_k or 10

        documents = [
            RankedDocument(
                id=doc.id,
                text=doc.text,
                score=doc.score,
                metadata=dict(doc.metadata),
            )
            for doc in request.documents
        ]

        logger.info(f"Rerank: query='{query[:50]}...', {len(documents)} docs, top_k={top_k}")
        reranked = self.reranker.rerank(query, documents, top_k)

        response_docs = [
            ml_service_pb2.RerankDocument(
                id=doc.id,
                text=doc.text,
                score=doc.score,
                metadata=doc.metadata,
            )
            for doc in reranked
        ]
        return ml_service_pb2.RerankResponse(documents=response_docs)

    async def Generate(self, request, context):
        provider_name = request.provider or "claude"
        logger.info(f"Generate: provider={provider_name}, model={request.model}")

        try:
            provider = get_llm_provider(provider_name)
        except ValueError as e:
            context.abort(grpc.StatusCode.INVALID_ARGUMENT, str(e))
            return

        config = GenerationConfig(
            model=request.model,
            temperature=request.temperature or 0.7,
            max_tokens=request.max_tokens or 2048,
            system_prompt=request.system_prompt,
        )

        try:
            async for chunk in provider.generate_stream(request.prompt, config):
                yield ml_service_pb2.GenerateChunk(
                    text=chunk.text,
                    is_final=chunk.is_final,
                )
        except Exception as e:
            logger.error(f"Generation error: {e}")
            context.abort(grpc.StatusCode.INTERNAL, str(e))


async def serve():
    server = aio.server(
        futures.ThreadPoolExecutor(max_workers=4),
        options=[
            ("grpc.max_send_message_length", 100 * 1024 * 1024),  # 100MB
            ("grpc.max_receive_message_length", 100 * 1024 * 1024),
        ],
    )

    servicer = MlServiceServicer()
    ml_service_pb2_grpc.add_MlServiceServicer_to_server(servicer, server)

    listen_addr = f"[::]:{settings.grpc_port}"
    server.add_insecure_port(listen_addr)

    logger.info(f"Starting ML gRPC server on {listen_addr}")
    await server.start()
    logger.info("ML service ready")
    await server.wait_for_termination()


def main():
    asyncio.run(serve())


if __name__ == "__main__":
    main()
