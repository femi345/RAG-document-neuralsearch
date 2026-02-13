import logging

from ..config import settings
from .base import LLMProvider
from .claude import ClaudeProvider
from .ollama import OllamaProvider
from .openai_gpt import OpenAIProvider

logger = logging.getLogger(__name__)

_providers: dict[str, LLMProvider] = {}


def get_llm_provider(provider_name: str) -> LLMProvider:
    if provider_name in _providers:
        return _providers[provider_name]

    match provider_name:
        case "claude":
            if not settings.anthropic_api_key:
                raise ValueError("ANTHROPIC_API_KEY not set")
            provider = ClaudeProvider(settings.anthropic_api_key)
        case "openai":
            if not settings.openai_api_key:
                raise ValueError("OPENAI_API_KEY not set")
            provider = OpenAIProvider(settings.openai_api_key)
        case "ollama":
            provider = OllamaProvider(settings.ollama_host)
        case _:
            raise ValueError(f"Unknown LLM provider: {provider_name}")

    _providers[provider_name] = provider
    logger.info(f"Initialized LLM provider: {provider_name}")
    return provider
