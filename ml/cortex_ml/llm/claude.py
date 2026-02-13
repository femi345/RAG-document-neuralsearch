import logging
from collections.abc import AsyncIterator

import anthropic

from .base import GenerationChunk, GenerationConfig, LLMProvider

logger = logging.getLogger(__name__)


class ClaudeProvider(LLMProvider):
    def __init__(self, api_key: str):
        self.client = anthropic.AsyncAnthropic(api_key=api_key)

    async def generate(self, prompt: str, config: GenerationConfig) -> str:
        message = await self.client.messages.create(
            model=config.model or "claude-sonnet-4-5-20250929",
            max_tokens=config.max_tokens,
            temperature=config.temperature,
            system=config.system_prompt,
            messages=[{"role": "user", "content": prompt}],
        )
        return message.content[0].text

    async def generate_stream(
        self, prompt: str, config: GenerationConfig
    ) -> AsyncIterator[GenerationChunk]:
        async with self.client.messages.stream(
            model=config.model or "claude-sonnet-4-5-20250929",
            max_tokens=config.max_tokens,
            temperature=config.temperature,
            system=config.system_prompt,
            messages=[{"role": "user", "content": prompt}],
        ) as stream:
            async for text in stream.text_stream:
                yield GenerationChunk(text=text)
            yield GenerationChunk(text="", is_final=True)

    def supported_models(self) -> list[str]:
        return ["claude-sonnet-4-5-20250929", "claude-opus-4-6"]
