use anyhow::Result;
use bridge_embex::client::EmbexClient;
use bridge_embex::EmbexConfig;
use bridge_embex_core::types::{CollectionSchema, DistanceMetric, Point};

#[tokio::main]
async fn main() -> Result<()> {
    let config = EmbexConfig {
        provider: "qdrant".to_string(),
        url: "http://localhost:6333".to_string(),
        api_key: None,
        ..Default::default()
    };

    let client = EmbexClient::new(config)?;

    let collection_name = "rust_example";
    let schema = CollectionSchema {
        name: collection_name.to_string(),
        dimension: 4,
        metric: DistanceMetric::Cosine,
    };

    println!("Creating collection '{}'...", collection_name);
    let _ = client.collection(collection_name).create(schema).await;
    println!("Inserting data...");
    let points = vec![
        Point {
            id: "vec1".to_string(),
            vector: vec![0.1, 0.2, 0.3, 0.4],
            metadata: None,
        },
        Point {
            id: "vec2".to_string(),
            vector: vec![0.9, 0.8, 0.7, 0.6],
            metadata: None,
        },
    ];

    let collection = client.collection(collection_name);
    collection.insert(points).await?;

    println!("Searching...");
    let query_vector = vec![0.1, 0.2, 0.3, 0.4];
    let results = collection.search(query_vector).limit(1).execute().await?;

    for res in results.results {
        println!("Found ID: {}, Score: {}", res.id, res.score);
    }

    Ok(())
}
