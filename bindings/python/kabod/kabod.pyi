from typing import List, Optional, Dict, Any, Iterable

class Point:
    id: str
    vector: List[float]
    metadata: Optional[Dict[str, Any]]
    
    def __init__(
        self, 
        id: str, 
        vector: List[float], 
        metadata: Optional[Dict[str, Any]] = None
    ) -> None: ...

class SearchResult:
    id: str
    score: float
    vector: Optional[List[float]]
    metadata: Optional[Dict[str, Any]]

class Collection:
    async def insert(self, points: List[Point]) -> None: ...
    async def insert_batch(
        self, 
        points: List[Point], 
        batch_size: int = 1000
    ) -> None: ...
    async def insert_stream(
        self, 
        points: Iterable[Point], 
        batch_size: int = 1000
    ) -> None: ...
    async def search(
        self, 
        vector: List[float], 
        top_k: Optional[int] = 10, 
        include_metadata: Optional[bool] = True,
        include_vector: Optional[bool] = False
    ) -> List[SearchResult]: ...
    async def delete(self, ids: List[str]) -> None: ...
    async def delete_collection(self) -> None: ...
    async def create(
        self, 
        dimension: int, 
        distance: str
    ) -> None: ...

class KabodClient:
    def __init__(
        self, 
        provider: str, 
        url: str, 
        api_key: Optional[str] = None
    ) -> None: ...
    
    def collection(self, name: str) -> Collection: ...
