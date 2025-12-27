from pydantic import BaseModel, ConfigDict
from typing import List, Optional, Dict, Any
import pytest
from embex import Point, SearchResult, SearchResponse

class PyPoint(BaseModel):
    id: str
    vector: List[float]
    metadata: Optional[Dict[str, Any]] = None

class PySearchResult(BaseModel):
    id: str
    score: float
    vector: Optional[List[float]] = None
    metadata: Optional[Dict[str, Any]] = None

def test_point_methods():
    kp = Point(id="1", vector=[1.0, 2.0], metadata={"a": 1})
    
    assert kp.id == "1"
    
    d = kp.dict()
    assert isinstance(d, dict)
    assert d == {"id": "1", "vector": [1.0, 2.0], "metadata": {"a": 1}}
    
    md = kp.model_dump()
    assert md == d
    
def test_pydantic_validation_via_dump():
    kp = Point(id="1", vector=[1.0, 2.0], metadata={"a": 1})
    
    pp = PyPoint.model_validate(kp.model_dump())
    assert pp.id == "1"
    assert pp.vector == [1.0, 2.0]
    assert pp.metadata == {"a": 1}
