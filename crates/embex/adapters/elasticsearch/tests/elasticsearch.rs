use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use bridge_embex_core::db::VectorDatabase;
use bridge_embex_core::error::{EmbexError, Result};
use bridge_embex_core::types::{
    Aggregation, CollectionSchema, DistanceMetric, Filter, MetadataUpdate, Point, VectorQuery,
};
use bridge_embex_elasticsearch::ElasticsearchAdapter;
use mockito::{Matcher, Server};
use serde_json::{Value, json};

fn schema(name: &str) -> CollectionSchema {
    CollectionSchema {
        name: name.into(),
        dimension: 2,
        metric: DistanceMetric::Dot,
    }
}

fn points() -> Vec<Point> {
    vec![
        Point::new("first", vec![2.0, 0.0]).with_metadata(HashMap::from([
            ("category".into(), json!("wanted")),
            ("rating".into(), json!(10)),
        ])),
        Point::new("other", vec![3.0, 0.0])
            .with_metadata(HashMap::from([("category".into(), json!("excluded"))])),
        Point::new("last", vec![1.0, 0.0])
            .with_metadata(HashMap::from([("category".into(), json!("wanted"))])),
    ]
}

fn filtered_query(collection: &str) -> VectorQuery {
    VectorQuery {
        collection: collection.into(),
        vector: Some(vec![2.0, 0.0]),
        filter: Some(Filter::must(vec![
            Filter::eq("category", "wanted"),
            Filter::gte("rating", 2),
        ])),
        top_k: 1,
        offset: None,
        include_vector: true,
        include_metadata: true,
        aggregations: vec![Aggregation::Count],
    }
}

#[tokio::test]
async fn collection_lifecycle_over_http() {
    let mut server = Server::new_async().await;
    let create = server
        .mock("PUT", "/proxy/embex")
        .match_header("authorization", "ApiKey encoded-test-key")
        .match_body(Matcher::PartialJson(json!({
            "mappings": {"properties": {
                "vector": {"type": "dense_vector", "dims": 2, "similarity": "max_inner_product"},
                "metadata": {"type": "object"}
            }}
        })))
        .with_body(r#"{"acknowledged":true}"#)
        .create_async()
        .await;
    let exists = server
        .mock("HEAD", "/proxy/embex")
        .match_header("authorization", "ApiKey encoded-test-key")
        .create_async()
        .await;
    let insert = server
        .mock("POST", "/proxy/embex/_bulk?refresh=wait_for")
        .match_header("content-type", "application/x-ndjson")
        .match_body(concat!(
            "{\"index\":{\"_id\":\"first\"}}\n",
            "{\"metadata\":{\"category\":\"wanted\",\"rating\":10},\"vector\":[2.0,0.0]}\n",
            "{\"index\":{\"_id\":\"other\"}}\n",
            "{\"metadata\":{\"category\":\"excluded\"},\"vector\":[3.0,0.0]}\n",
            "{\"index\":{\"_id\":\"last\"}}\n",
            "{\"metadata\":{\"category\":\"wanted\"},\"vector\":[1.0,0.0]}\n",
        ))
        .with_body(r#"{"items":[{"index":{"status":201}},{"index":{"status":201}},{"index":{"status":201}}]}"#)
        .create_async()
        .await;

    let filter = json!({"bool": {"filter": [
        {"term": {"metadata.category": "wanted"}},
        {"range": {"metadata.rating": {"gte": 2}}}
    ]}});
    let count = server
        .mock("POST", "/proxy/embex/_count")
        .match_body(Matcher::Json(json!({"query": filter})))
        .with_body(r#"{"count":1,"_shards":{"failed":0}}"#)
        .create_async()
        .await;
    let search = server
        .mock("POST", "/proxy/embex/_search?allow_partial_search_results=false")
        .match_body(Matcher::PartialJson(json!({
            "knn": {"field": "vector", "query_vector": [2.0, 0.0], "k": 1, "filter": filter},
            "_source": ["vector", "metadata"],
            "fields": ["vector"]
        })))
        .with_body(r#"{"hits":{"hits":[{"_id":"first","_score":5.0,"fields":{"vector":[[2.0,0.0]]},"_source":{"metadata":{"category":"wanted","rating":10}}}]}}"#)
        .create_async()
        .await;
    let update = server
        .mock("POST", "/proxy/embex/_bulk?refresh=wait_for")
        .match_body(Matcher::Regex(
            r#"(?s)^\{"update":\{"_id":"first","retry_on_conflict":3\}\}\n\{.*"params":\{"metadata":\{"rating":20\}\}.*\}\n$"#.into(),
        ))
        .with_body(r#"{"items":[{"update":{"status":200}}]}"#)
        .create_async()
        .await;
    let pit = server
        .mock("POST", "/proxy/embex/_pit?keep_alive=1m")
        .with_body(r#"{"id":"start"}"#)
        .create_async()
        .await;
    let first = server
        .mock("POST", "/proxy/_search?allow_partial_search_results=false")
        .match_body(Matcher::PartialJson(json!({
            "pit": {"id":"start", "keep_alive":"1m"},
            "size": 3,
            "sort": ["_shard_doc"],
        })))
        .with_body(json!({"pit_id":"page-1", "hits":{"hits":[
            {"_id":"first", "_source":{"vector":[2.0,0.0], "metadata":{"rating":20}}, "sort":[5]},
            {"_id":"other", "fields":{"vector":[3.0,0.0]}, "sort":[9]},
            {"_id":"last", "_source":{"vector":[1.0,0.0]}, "sort":[16]}
        ]}}).to_string())
        .create_async()
        .await;
    let second = server
        .mock("POST", "/proxy/_search?allow_partial_search_results=false")
        .match_body(Matcher::PartialJson(json!({
            "pit": {"id":"page-1", "keep_alive":"1m"},
            "search_after": [9],
            "size": 3
        })))
        .with_body(r#"{"pit_id":"page-2","hits":{"hits":[{"_id":"last","_source":{"vector":[1.0,0.0]},"sort":[16]}]}}"#)
        .create_async()
        .await;
    let close = server
        .mock("DELETE", "/proxy/_pit")
        .match_body(Matcher::Json(json!({"id":"page-2"})))
        .with_body(r#"{"succeeded":true}"#)
        .create_async()
        .await;
    let delete = server
        .mock("POST", "/proxy/embex/_bulk?refresh=wait_for")
        .match_body("{\"delete\":{\"_id\":\"first\"}}\n{\"delete\":{\"_id\":\"missing\"}}\n")
        .with_body(r#"{"items":[{"delete":{"status":200}},{"delete":{"status":404}}]}"#)
        .create_async()
        .await;
    let drop_index = server
        .mock("DELETE", "/proxy/embex")
        .with_body(r#"{"acknowledged":true}"#)
        .create_async()
        .await;

    let db = ElasticsearchAdapter::with_api_key(
        &format!("{}/proxy/", server.url()),
        Some("encoded-test-key"),
    )
    .unwrap();
    db.create_collection(&schema("embex")).await.unwrap();
    db.insert("embex", points()).await.unwrap();
    let response = db.search(&filtered_query("embex")).await.unwrap();
    assert_eq!(response.aggregations["count"], 1);
    let hit = &response.results[0];
    assert_eq!(hit.id, "first");
    assert_eq!(hit.score, 5.0);
    assert_eq!(hit.vector.as_deref(), Some([2.0, 0.0].as_slice()));
    assert_eq!(hit.metadata.as_ref().unwrap()["rating"], 10);

    db.update_metadata(
        "embex",
        vec![MetadataUpdate {
            id: "first".into(),
            updates: HashMap::from([("rating".into(), json!(20))]),
        }],
    )
    .await
    .unwrap();

    let page = db.scroll("embex", None, 2).await.unwrap();
    assert_eq!(
        page.points
            .iter()
            .map(|p| p.id.as_str())
            .collect::<Vec<_>>(),
        ["first", "other"]
    );
    let page = db.scroll("embex", page.next_offset, 2).await.unwrap();
    assert_eq!(page.points[0].id, "last");
    assert!(page.next_offset.is_none());

    db.delete("embex", vec!["first".into(), "missing".into()])
        .await
        .unwrap();
    db.delete_collection("embex").await.unwrap();

    for mock in [
        create, exists, insert, count, search, update, pit, first, second, close, delete,
        drop_index,
    ] {
        mock.assert_async().await;
    }
}

#[tokio::test]
async fn detects_item_errors_and_does_not_create_missing_indexes() {
    let mut server = Server::new_async().await;
    let exists = server.mock("HEAD", "/embex").create_async().await;
    let failed = server
        .mock("POST", "/embex/_bulk?refresh=wait_for")
        .with_body(r#"{"items":[
            {"index":{"status":201}},
            {"index":{"_id":"other","status":400,"error":{"type":"mapper_parsing_exception","reason":"wrong dimension"}}},
            {"index":{"status":201}}
        ]}"#)
        .create_async()
        .await;
    let missing = server
        .mock("HEAD", "/missing")
        .with_status(404)
        .create_async()
        .await;
    let db = ElasticsearchAdapter::new(&server.url()).unwrap();

    let error = db.insert("embex", points()).await.unwrap_err();
    assert!(
        matches!(error, EmbexError::Query(message) if message.contains("other") && message.contains("wrong dimension"))
    );
    assert!(
        matches!(db.insert("missing", points()).await, Err(EmbexError::CollectionNotFound(name)) if name == "missing")
    );
    for invalid in ["_all", "../embex", "items,*", "x/y", "X", "foo%2fbar"] {
        assert!(matches!(
            db.delete_collection(invalid).await,
            Err(EmbexError::Validation(_))
        ));
    }
    for mock in [exists, failed, missing] {
        mock.assert_async().await;
    }
}

#[tokio::test]
async fn filter_only_queries_honor_offset_and_projection() {
    let mut server = Server::new_async().await;
    let request = server
        .mock("POST", "/embex/_search?allow_partial_search_results=false")
        .match_body(Matcher::Json(json!({
            "size": 1, "from": 2, "_source": false, "track_total_hits": false,
            "query": {"bool": {
                "should": [
                    {"terms": {"metadata.category": ["first", "second"]}},
                    {"bool": {"must_not": {"term": {"metadata.retired": true}}}}
                ],
                "minimum_should_match": 1
            }}
        })))
        .with_body(r#"{"hits":{"hits":[{"_id":"item","_score":1}]}}"#)
        .create_async()
        .await;
    let db = ElasticsearchAdapter::new(&server.url()).unwrap();
    let result = db
        .search(&VectorQuery {
            collection: "embex".into(),
            vector: None,
            filter: Some(Filter::should(vec![
                Filter::r#in("category", vec!["first", "second"]),
                Filter::ne("retired", true),
            ])),
            top_k: 1,
            offset: Some(2),
            include_vector: false,
            include_metadata: false,
            aggregations: Vec::new(),
        })
        .await
        .unwrap();
    assert_eq!(result.results[0].id, "item");
    assert!(result.results[0].vector.is_none());
    assert!(result.results[0].metadata.is_none());
    request.assert_async().await;
}

/// Run with ELASTICSEARCH_URL and, if necessary, ELASTICSEARCH_API_KEY.
#[tokio::test]
#[ignore = "requires an Elasticsearch instance"]
async fn real_elasticsearch_lifecycle() -> Result<()> {
    let url = std::env::var("ELASTICSEARCH_URL")
        .expect("set ELASTICSEARCH_URL to a running Elasticsearch instance");
    let key = std::env::var("ELASTICSEARCH_API_KEY").ok();
    let db = ElasticsearchAdapter::with_api_key(&url, key.as_deref())?;
    let run_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let name = format!("embex-integration-{run_id}");
    db.create_collection(&schema(&name)).await?;

    let result = async {
        db.insert(&name, points()).await?;
        let response = db.search(&filtered_query(&name)).await?;
        assert_eq!(response.aggregations["count"], 1);
        assert_eq!(response.results[0].id, "first");
        assert_eq!(response.results[0].vector, Some(vec![2.0, 0.0]));

        let page = db.scroll(&name, None, 2).await?;
        assert_eq!(page.points.len(), 2);
        assert!(page.next_offset.is_some());
        let last = db.scroll(&name, page.next_offset, 2).await?;
        assert_eq!(last.points.len(), 1);
        assert!(last.next_offset.is_none());

        db.update_metadata(
            &name,
            vec![MetadataUpdate {
                id: "first".into(),
                updates: HashMap::from([("rating".into(), Value::from(0))]),
            }],
        )
        .await?;
        let response = db.search(&filtered_query(&name)).await?;
        assert_eq!(response.aggregations["count"], 0);
        assert!(response.results.is_empty());

        db.delete(&name, vec!["first".into(), "other".into(), "last".into()])
            .await?;
        assert!(db.scroll(&name, None, 2).await?.points.is_empty());
        Ok(())
    }
    .await;
    let cleanup = db.delete_collection(&name).await;
    result.and(cleanup)
}
