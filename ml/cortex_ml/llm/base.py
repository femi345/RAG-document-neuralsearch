from abc import ABC, abstractmethod
from collections.abc import AsyncIterator
from dataclasses import dataclass, field


@dataclass
class GenerationConfig:
    model: str = ""
    temperature: float = 0.7
    max_tokens: int = 2048
    system_prompt: str = ""


@dataclass
class GenerationChunk:
    text: str
    is_final: bool = False


class LLMProvider(ABC):
    @abstractmethod
    async def generate(self, prompt: str, config: GenerationConfig) -> str:
        ...

    @abstractmethod
    async def generate_stream(
        self, prompt: str, config: GenerationConfig
    ) -> AsyncIterator[GenerationChunk]:
        ...

    @abstractmethod
    def supported_models(self) -> list[str]:
        ...
