from abc import ABC, abstractmethod
from dataclasses import dataclass


@dataclass
class RankedDocument:
    id: str
    text: str
    score: float
    metadata: dict[str, str]


class Reranker(ABC):
    @abstractmethod
    def rerank(
        self, query: str, documents: list[RankedDocument], top_k: int
    ) -> list[RankedDocument]:
        ...
