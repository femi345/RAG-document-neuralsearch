import logging
from collections.abc import AsyncIterator

import openai

from .base import GenerationChunk, GenerationConfig, LLMProvider

logger = logging.getLogger(__name__)


class OpenAIProvider(LLMProvider):
    def __init__(self, api_key: str):
        self.client = openai.AsyncOpenAI(api_key=api_key)

    async def generate(self, prompt: str, config: GenerationConfig) -> str:
        messages = []
        if config.system_prompt:
            messages.append({"role": "system", "content": config.system_prompt})
        messages.append({"role": "user", "content": prompt})

        response = await self.client.chat.completions.create(
            model=config.model or "gpt-4o",
            messages=messages,
            temperature=config.temperature,
            max_tokens=config.max_tokens,
        )
        return response.choices[0].message.content or ""

    async def generate_stream(
        self, prompt: str, config: GenerationConfig
    ) -> AsyncIterator[GenerationChunk]:
        messages = []
        if config.system_prompt:
            messages.append({"role": "system", "content": config.system_prompt})
        messages.append({"role": "user", "content": prompt})

        stream = await self.client.chat.completions.create(
            model=config.model or "gpt-4o",
            messages=messages,
            temperature=config.temperature,
            max_tokens=config.max_tokens,
            stream=True,
        )

        async for chunk in stream:
            delta = chunk.choices[0].delta
            if delta.content:
                yield GenerationChunk(text=delta.content)
            if chunk.choices[0].finish_reason:
                yield GenerationChunk(text="", is_final=True)

    def supported_models(self) -> list[str]:
        return ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo"]
