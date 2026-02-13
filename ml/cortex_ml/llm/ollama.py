import logging
from collections.abc import AsyncIterator

import httpx

from .base import GenerationChunk, GenerationConfig, LLMProvider

logger = logging.getLogger(__name__)


class OllamaProvider(LLMProvider):
    def __init__(self, host: str = "http://localhost:11434"):
        self.host = host.rstrip("/")
        self.client = httpx.AsyncClient(timeout=120.0)

    async def generate(self, prompt: str, config: GenerationConfig) -> str:
        response = await self.client.post(
            f"{self.host}/api/generate",
            json={
                "model": config.model or "llama3",
                "prompt": prompt,
                "system": config.system_prompt,
                "options": {
                    "temperature": config.temperature,
                    "num_predict": config.max_tokens,
                },
                "stream": False,
            },
        )
        response.raise_for_status()
        return response.json()["response"]

    async def generate_stream(
        self, prompt: str, config: GenerationConfig
    ) -> AsyncIterator[GenerationChunk]:
        async with self.client.stream(
            "POST",
            f"{self.host}/api/generate",
            json={
                "model": config.model or "llama3",
                "prompt": prompt,
                "system": config.system_prompt,
                "options": {
                    "temperature": config.temperature,
                    "num_predict": config.max_tokens,
                },
                "stream": True,
            },
        ) as response:
            import json

            async for line in response.aiter_lines():
                if not line:
                    continue
                data = json.loads(line)
                if data.get("done"):
                    yield GenerationChunk(text="", is_final=True)
                    break
                yield GenerationChunk(text=data.get("response", ""))

    def supported_models(self) -> list[str]:
        return ["llama3", "mistral", "codellama", "phi3"]
