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
