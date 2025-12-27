import pytest
import kabod
from kabod import KabodClient, ConfigError, DatabaseError

def test_config_error():
    with pytest.raises(ConfigError) as excinfo:
        KabodClient(provider="invalid", url="http://localhost:6333")
    assert "Provider 'invalid' not available" in str(excinfo.value)

@pytest.mark.asyncio
async def test_database_error():
    client = KabodClient(provider="qdrant", url="http://localhost:6333")
    collection = client.collection("non_existent_collection")
    
    with pytest.raises(DatabaseError):
        await collection.search([0.1, 0.2, 0.3])

def test_error_inheritance():
    assert issubclass(ConfigError, kabod.KabodError)
    assert issubclass(DatabaseError, kabod.KabodError)
    assert issubclass(kabod.KabodError, Exception)
