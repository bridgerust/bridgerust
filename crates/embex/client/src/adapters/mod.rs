#[cfg(feature = "qdrant")]
pub use bridge_embex_qdrant::QdrantAdapter;

#[cfg(feature = "pinecone")]
pub use bridge_embex_pinecone::PineconeAdapter;

#[cfg(feature = "chroma")]
pub use bridge_embex_chroma::ChromaAdapter;

#[cfg(feature = "lancedb")]
pub use bridge_embex_lancedb::LanceDBAdapter;

#[cfg(feature = "pgvector")]
pub use bridge_embex_pgvector::PgVectorAdapter;

#[cfg(feature = "weaviate")]
pub use bridge_embex_weaviate::WeaviateAdapter;

#[cfg(feature = "milvus")]
pub use bridge_embex_milvus::MilvusAdapter;

#[cfg(feature = "elasticsearch")]
pub use bridge_embex_elasticsearch::ElasticsearchAdapter;

#[cfg(feature = "opensearch")]
pub use bridge_embex_opensearch::OpenSearchAdapter;

#[cfg(feature = "redis")]
pub use bridge_embex_redis::RedisAdapter;
