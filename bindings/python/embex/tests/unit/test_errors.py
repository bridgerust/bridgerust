import pytest
import embex
from embex import EmbexClient, ConfigError, DatabaseError

def test_config_error():
    with pytest.raises(ConfigError) as excinfo:
        EmbexClient(provider="invalid", url="http://localhost:6333")
    assert "Provider 'invalid' not available" in str(excinfo.value)

@pytest.mark.asyncio
async def test_database_error():
    client = EmbexClient(provider="qdrant", url="http://localhost:6333")
    collection = client.collection("non_existent_collection")
    
    with pytest.raises(DatabaseError):
        await collection.search([0.1, 0.2, 0.3])

def test_error_inheritance():
    assert issubclass(ConfigError, embex.EmbexError)
    assert issubclass(DatabaseError, embex.EmbexError)
    assert issubclass(embex.EmbexError, Exception)
