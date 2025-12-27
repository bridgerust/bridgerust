use bridge_embex_infrastructure::pooling::{get_pooling_status, PoolConfig, PoolingStatus};

#[test]
fn test_pool_config_default() {
    let config = PoolConfig::default();
    assert_eq!(config.max_connections, 10);
    assert_eq!(config.max_idle, 10);
    assert_eq!(config.idle_timeout.as_secs(), 90);
    assert_eq!(config.connect_timeout.as_secs(), 30);
}

#[test]
fn test_pooling_status_all_providers() {
    // Test HTTP-based adapters with configurable pooling
    let http_providers = vec!["pinecone", "milvus", "weaviate"];
    for provider in http_providers {
        let status = get_pooling_status(provider);
        match status {
            PoolingStatus::Configurable { .. } => {
                // Expected
            }
            _ => panic!("Provider {} should have configurable pooling", provider),
        }
    }

    // Test PgVector with SQL connection pooling
    let status = get_pooling_status("pgvector");
    match status {
        PoolingStatus::Configurable { .. } => {
            // Expected
        }
        _ => panic!("PgVector should have configurable pooling"),
    }

    // Test adapters with internal pooling
    assert_eq!(get_pooling_status("qdrant"), PoolingStatus::Default);
    assert_eq!(get_pooling_status("chroma"), PoolingStatus::Default);

    // Test embedded adapter
    assert_eq!(get_pooling_status("lancedb"), PoolingStatus::NotApplicable);
}

#[test]
fn test_pooling_status_configurable_values() {
    let status = get_pooling_status("pgvector");
    if let PoolingStatus::Configurable {
        max_connections,
        idle_timeout_secs,
    } = status
    {
        assert!(max_connections > 0);
        assert!(idle_timeout_secs > 0);
    } else {
        panic!("PgVector should return Configurable status");
    }
}

