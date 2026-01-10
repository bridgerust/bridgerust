
from qdrant_client import QdrantClient
client = QdrantClient(url="http://localhost:6333")
print(dir(client))
try:
    print(client.search)
    print("Search exists")
except AttributeError:
    print("Search DOES NOT EXIST")
