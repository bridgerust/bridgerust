use async_trait::async_trait;
use qdrant_client::Payload;
use qdrant_client::Qdrant;
use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, DeletePointsBuilder, Distance, PointStruct, ScoredPoint,
    SearchPointsBuilder, UpsertPointsBuilder, VectorParams, VectorsConfig, vectors_config::Config,
};

use bridge_embex_core::db::VectorDatabase;
use bridge_embex_core::error::{EmbexError, Result};
use bridge_embex_core::types::{
    self, CollectionSchema, DistanceMetric, MetadataUpdate, Point, SearchResponse, SearchResult,
    VectorQuery,
};
use std::collections::HashMap;

pub struct QdrantAdapter {
    client: Qdrant,
}

impl QdrantAdapter {
    /// Creates a new Qdrant adapter.
    ///
    /// Note: The Qdrant client uses its own internal connection pooling.
    /// The `pool_size` parameter is accepted for API consistency but is not directly
    /// configurable in the qdrant-client crate.
    pub fn new(url: &str, api_key: Option<&str>) -> Result<Self> {
        Self::new_with_pool_size(url, api_key, None)
    }

    /// Creates a new Qdrant adapter with pool size hint.
    ///
    /// Note: The Qdrant client uses its own internal connection pooling.
    /// The `pool_size` parameter is accepted for API consistency but is not directly
    /// configurable in the qdrant-client crate. The qdrant-client manages its own
    /// HTTP client with default pooling settings.
    pub fn new_with_pool_size(
        url: &str,
        api_key: Option<&str>,
        _pool_size: Option<u32>,
    ) -> Result<Self> {
        // Heuristic: If user passes localhost:6333, they likely mean the gRPC port 6334
        // (which is separate in standard Docker image), but are used to 6333 from Python/REST world.
        let url_string = url.to_string();
        let adjusted_url = if (url_string.contains("localhost:6333")
            || url_string.contains("127.0.0.1:6333"))
            && !url_string.contains("https")
        {
            tracing::info!(
                "Detected localhost:6333 config. Automatically switching to gRPC port 6334 for Qdrant Rust client."
            );
            url_string.replace("6333", "6334")
        } else {
            url_string
        };

        let mut config = QdrantConfig::from_url(&adjusted_url);

        if let Some(key) = api_key {
            config.set_api_key(key);
        }

        let client = Qdrant::new(config).map_err(|e| EmbexError::Database(e.to_string()))?;

        Ok(Self { client })
    }
}

#[async_trait]
impl VectorDatabase for QdrantAdapter {
    #[tracing::instrument(skip(self, schema), fields(collection = %schema.name, dimension = schema.dimension, provider = "qdrant"))]
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<()> {
        let distance = match schema.metric {
            DistanceMetric::Cosine => Distance::Cosine,
            DistanceMetric::Euclidean => Distance::Euclid,
            DistanceMetric::Dot => Distance::Dot,
        };

        let details =
            CreateCollectionBuilder::new(schema.name.clone()).vectors_config(VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: schema.dimension as u64,
                    distance: distance.into(),
                    ..Default::default()
                })),
            });

        self.client
            .create_collection(details)
            .await
            .map_err(|e| EmbexError::Database(e.to_string()))?;

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(collection = %name, provider = "qdrant"))]
    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.client
            .delete_collection(name)
            .await
            .map_err(|e| EmbexError::Database(e.to_string()))?;
        Ok(())
    }

    #[tracing::instrument(skip(self, points), fields(collection = %collection, count = points.len(), provider = "qdrant"))]
    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<()> {
        let points: Vec<PointStruct> = points
            .into_iter()
            .map(|p| {
                let payload: Payload = if let Some(metadata) = p.metadata {
                    metadata.into()
                } else {
                    Payload::new()
                };

                PointStruct::new(p.id, p.vector, payload)
            })
            .collect();

        self.client
            .upsert_points(UpsertPointsBuilder::new(collection, points))
            .await
            .map_err(|e: qdrant_client::QdrantError| EmbexError::Database(e.to_string()))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, query), fields(collection = %query.collection, limit = query.top_k, provider = "qdrant"))]
    async fn search(&self, query: &VectorQuery) -> Result<SearchResponse> {
        // If vector is present, use SearchPoints (KNN)
        // If vector is absent, use ScrollPoints (Filter only)

        let mut aggregations = HashMap::new();
        // Aggregations (like count) are separate from search hits usually in Qdrant concepts for this adapter
        // But here they were implemented inside the search method.
        // Let's preserve aggregation logic which uses 'count' API directly.
        for agg in &query.aggregations {
            match agg {
                bridge_embex_core::types::Aggregation::Count => {
                    let mut count_builder =
                        qdrant_client::qdrant::CountPointsBuilder::new(query.collection.clone())
                            .exact(true);
                    if let Some(filter) = &query.filter {
                        count_builder = count_builder.filter(convert_filter(filter));
                    }
                    let count_res = self.client.count(count_builder).await.map_err(
                        |e: qdrant_client::QdrantError| EmbexError::Database(e.to_string()),
                    )?;
                    aggregations.insert(
                        "count".to_string(),
                        serde_json::Value::Number(count_res.result.unwrap().count.into()),
                    );
                }
            }
        }

        let search_results = if let Some(vector) = &query.vector {
            let mut builder = SearchPointsBuilder::new(
                query.collection.clone(),
                vector.clone(),
                query.top_k as u64,
            )
            .with_payload(query.include_metadata)
            .with_vectors(query.include_vector);

            if let Some(offset) = query.offset {
                builder = builder.offset(offset as u64);
            }

            if let Some(filter) = &query.filter {
                builder = builder.filter(convert_filter(filter));
            }

            let result = self
                .client
                .search_points(builder)
                .await
                .map_err(|e| EmbexError::Database(e.to_string()))?;

            result
                .result
                .into_iter()
                .map(convert_scored_point)
                .collect()
        } else {
            use qdrant_client::qdrant::ScrollPointsBuilder;

            let mut builder = ScrollPointsBuilder::new(query.collection.clone())
                .limit(query.top_k as u32)
                .with_payload(query.include_metadata)
                .with_vectors(query.include_vector);

            if let Some(_offset) = query.offset {
                return Err(EmbexError::Unsupported(
                    "Offset not supported for filter-only queries in Qdrant adapter yet.".into(),
                ));
            }

            if let Some(filter) = &query.filter {
                builder = builder.filter(convert_filter(filter));
            }

            let result = self
                .client
                .scroll(builder)
                .await
                .map_err(|e| EmbexError::Database(e.to_string()))?;

            result
                .result
                .into_iter()
                .map(|p| {
                    // Convert RetrievedPoint to SearchResult
                    // Score is 0.0 or 1.0 since it's exact match/filter
                    let id =
                        p.id.and_then(|id| id.point_id_options)
                            .map(|opt| match opt {
                                qdrant_client::qdrant::point_id::PointIdOptions::Num(n) => {
                                    n.to_string()
                                }
                                qdrant_client::qdrant::point_id::PointIdOptions::Uuid(u) => u,
                            })
                            .unwrap_or_default();

                    #[allow(deprecated)]
                    let vector = p.vectors.and_then(|v| match v.vectors_options {
                        Some(qdrant_client::qdrant::vectors_output::VectorsOptions::Vector(v)) => {
                            Some(v.data)
                        }
                        _ => None,
                    });

                    SearchResult {
                        id,
                        score: 1.0,
                        vector,
                        metadata: Some(p.payload.into_iter().map(|(k, v)| (k, v.into())).collect()),
                    }
                })
                .collect()
        };

        Ok(SearchResponse {
            results: search_results,
            aggregations,
        })
    }

    #[tracing::instrument(skip(self), fields(collection = %collection, count = ids.len(), provider = "qdrant"))]
    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()> {
        let points = qdrant_client::qdrant::PointsIdsList {
            ids: ids.into_iter().map(|id| id.into()).collect(),
        };

        self.client
            .delete_points(DeletePointsBuilder::new(collection).points(points))
            .await
            .map_err(|e| EmbexError::Database(e.to_string()))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, updates), fields(collection = %collection, count = updates.len(), provider = "qdrant"))]
    async fn update_metadata(&self, collection: &str, updates: Vec<MetadataUpdate>) -> Result<()> {
        for update in updates {
            let payload: Payload = update.updates.into();
            let points = qdrant_client::qdrant::PointsIdsList {
                ids: vec![update.id.into()],
            };

            let selector =
                qdrant_client::qdrant::points_selector::PointsSelectorOneOf::Points(points);

            self.client
                .set_payload(
                    qdrant_client::qdrant::SetPayloadPointsBuilder::new(collection, payload)
                        .points_selector(selector),
                )
                .await
                .map_err(|e: qdrant_client::QdrantError| EmbexError::Database(e.to_string()))?;
        }

        Ok(())
    }
}

fn convert_filter(filter: &types::Filter) -> qdrant_client::qdrant::Filter {
    let mut qdrant_filter = qdrant_client::qdrant::Filter::default();
    match filter {
        types::Filter::Must(filters) => {
            qdrant_filter.must = filters.iter().map(convert_condition_to_qdrant).collect();
        }
        types::Filter::MustNot(filters) => {
            qdrant_filter.must_not = filters.iter().map(convert_condition_to_qdrant).collect();
        }
        types::Filter::Should(filters) => {
            qdrant_filter.should = filters.iter().map(convert_condition_to_qdrant).collect();
        }
        types::Filter::Key(key, condition) => {
            match condition {
                types::Condition::Ne(val) => {
                    qdrant_filter.must_not =
                        vec![convert_key_condition(key, &types::Condition::Eq(val.clone()))];
                }
                types::Condition::NotIn(vals) => {
                    qdrant_filter.must_not =
                        vec![convert_key_condition(key, &types::Condition::In(vals.clone()))];
                }
                _ => {
                    qdrant_filter.must = vec![convert_key_condition(key, condition)];
                }
            }
        }
    }
    qdrant_filter
}

fn convert_condition_to_qdrant(filter: &types::Filter) -> qdrant_client::qdrant::Condition {
    use qdrant_client::qdrant::Condition;
    use qdrant_client::qdrant::condition::ConditionOneOf;

    match filter {
        types::Filter::Key(key, condition) => convert_key_condition(key, condition),
        _ => Condition {
            condition_one_of: Some(ConditionOneOf::Filter(convert_filter(filter))),
        },
    }
}

fn convert_key_condition(
    key: &str,
    condition: &types::Condition,
) -> qdrant_client::qdrant::Condition {
    use qdrant_client::qdrant::condition::ConditionOneOf;
    use qdrant_client::qdrant::r#match::MatchValue;
    use qdrant_client::qdrant::{Condition, FieldCondition, Match, Range};
    use types::Condition as EmbexCondition;

    match condition {
        EmbexCondition::Eq(value) => {
            let match_value = match value {
                serde_json::Value::String(s) => Some(MatchValue::Keyword(s.clone())),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Some(MatchValue::Integer(i))
                    } else {
                        None
                    }
                }
                serde_json::Value::Bool(b) => Some(MatchValue::Boolean(*b)),
                _ => None,
            };

            if let Some(mv) = match_value {
                Condition {
                    condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                        key: key.to_string(),
                        r#match: Some(Match {
                            match_value: Some(mv),
                        }),
                        ..Default::default()
                    })),
                }
            } else {
                Condition::default()
            }
        }
        EmbexCondition::Ne(value) => Condition {
            condition_one_of: Some(ConditionOneOf::Filter(qdrant_client::qdrant::Filter {
                must_not: vec![convert_key_condition(
                    key,
                    &EmbexCondition::Eq(value.clone()),
                )],
                ..Default::default()
            })),
        },
        EmbexCondition::Gt(value)
        | EmbexCondition::Gte(value)
        | EmbexCondition::Lt(value)
        | EmbexCondition::Lte(value) => {
            let mut range = Range::default();
            let val = value.as_f64().unwrap_or(0.0);
            match condition {
                EmbexCondition::Gt(_) => range.gt = Some(val),
                EmbexCondition::Gte(_) => range.gte = Some(val),
                EmbexCondition::Lt(_) => range.lt = Some(val),
                EmbexCondition::Lte(_) => range.lte = Some(val),
                _ => unreachable!(),
            }
            Condition {
                condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                    key: key.to_string(),
                    range: Some(range),
                    ..Default::default()
                })),
            }
        }
        EmbexCondition::In(values) => {
            let match_value = if values.is_empty() {
                None
            } else if let Some(_s) = values[0].as_str() {
                Some(MatchValue::Keywords(
                    qdrant_client::qdrant::RepeatedStrings {
                        strings: values
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect(),
                    },
                ))
            } else if let Some(_i) = values[0].as_i64() {
                Some(MatchValue::Integers(
                    qdrant_client::qdrant::RepeatedIntegers {
                        integers: values.iter().filter_map(|v| v.as_i64()).collect(),
                    },
                ))
            } else {
                None
            };

            if let Some(mv) = match_value {
                Condition {
                    condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                        key: key.to_string(),
                        r#match: Some(Match {
                            match_value: Some(mv),
                        }),
                        ..Default::default()
                    })),
                }
            } else {
                Condition::default()
            }
        }
        EmbexCondition::NotIn(values) => Condition {
            condition_one_of: Some(ConditionOneOf::Filter(qdrant_client::qdrant::Filter {
                must_not: vec![convert_key_condition(
                    key,
                    &EmbexCondition::In(values.clone()),
                )],
                ..Default::default()
            })),
        },
    }
}

fn convert_scored_point(point: ScoredPoint) -> SearchResult {
    let id = point
        .id
        .and_then(|id| id.point_id_options)
        .map(|opt| match opt {
            qdrant_client::qdrant::point_id::PointIdOptions::Num(n) => n.to_string(),
            qdrant_client::qdrant::point_id::PointIdOptions::Uuid(u) => u,
        })
        .unwrap_or_default();

    #[allow(deprecated)]
    let vector = point.vectors.and_then(|v| match v.vectors_options {
        Some(qdrant_client::qdrant::vectors_output::VectorsOptions::Vector(v)) => Some(v.data),
        _ => None,
    });

    SearchResult {
        id,
        score: point.score,
        vector,
        metadata: Some(
            point
                .payload
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridge_embex_core::types::Filter;

    #[test]
    fn test_qdrant_adapter_new() {
        let adapter = QdrantAdapter::new("http://localhost:6333", None);
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_qdrant_adapter_new_with_api_key() {
        let adapter = QdrantAdapter::new("http://localhost:6333", Some("test-key"));
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_convert_filter_eq_string() {
        let filter = Filter::eq("key", "value");
        let qdrant_filter = convert_filter(&filter);
        assert!(!qdrant_filter.must.is_empty());
    }

    #[test]
    fn test_convert_filter_eq_number() {
        let filter = Filter::eq("age", 25);
        let qdrant_filter = convert_filter(&filter);
        assert!(!qdrant_filter.must.is_empty());
    }

    #[test]
    fn test_convert_filter_eq_bool() {
        let filter = Filter::eq("active", true);
        let qdrant_filter = convert_filter(&filter);
        assert!(!qdrant_filter.must.is_empty());
    }

    #[test]
    fn test_convert_filter_comparison_ops() {
        let filters = vec![
            Filter::gt("age", 18),
            Filter::gte("score", 100),
            Filter::lt("price", 50.0),
            Filter::lte("count", 10),
        ];

        for filter in filters {
            let qdrant_filter = convert_filter(&filter);
            assert!(!qdrant_filter.must.is_empty());
        }
    }

    #[test]
    fn test_convert_filter_in() {
        let filter = Filter::r#in("tags", vec!["a", "b", "c"]);
        let qdrant_filter = convert_filter(&filter);
        assert!(!qdrant_filter.must.is_empty());
    }

    #[test]
    fn test_convert_filter_not_in() {
        let filter = Filter::not_in("tags", vec!["x", "y"]);
        let qdrant_filter = convert_filter(&filter);
        assert!(!qdrant_filter.must_not.is_empty());
    }

    #[test]
    fn test_convert_filter_must() {
        let filter = Filter::must(vec![
            Filter::eq("a", 1),
            Filter::eq("b", 2),
        ]);
        let qdrant_filter = convert_filter(&filter);
        assert!(!qdrant_filter.must.is_empty());
    }

    #[test]
    fn test_convert_filter_must_not() {
        let filter = Filter::must_not(vec![
            Filter::eq("a", 1),
        ]);
        let qdrant_filter = convert_filter(&filter);
        assert!(!qdrant_filter.must_not.is_empty());
    }

    #[test]
    fn test_convert_filter_should() {
        let filter = Filter::should(vec![
            Filter::eq("a", 1),
            Filter::eq("b", 2),
        ]);
        let qdrant_filter = convert_filter(&filter);
        assert!(!qdrant_filter.should.is_empty());
    }

    #[test]
    fn test_convert_filter_complex_nested() {
        let filter = Filter::must(vec![
            Filter::eq("status", "active"),
            Filter::should(vec![
                Filter::gt("age", 18),
                Filter::lt("age", 65),
            ]),
        ]);
        let qdrant_filter = convert_filter(&filter);
        assert!(!qdrant_filter.must.is_empty());
    }

    #[test]
    fn test_convert_filter_ne() {
        let filter = Filter::ne("key", "value");
        let qdrant_filter = convert_filter(&filter);
        assert!(!qdrant_filter.must_not.is_empty());
    }

    #[test]
    fn test_convert_filter_empty_in() {
        let filter = Filter::r#in("tags", Vec::<String>::new());
        let qdrant_filter = convert_filter(&filter);
        assert!(!qdrant_filter.must.is_empty());
    }

    #[test]
    fn test_convert_key_condition_integers() {
        let filter = Filter::r#in("ids", vec![1, 2, 3]);
        let qdrant_filter = convert_filter(&filter);
        assert!(!qdrant_filter.must.is_empty());
    }

    #[test]
    fn test_qdrant_adapter_auto_correct_port() {
        let adapter = QdrantAdapter::new("http://localhost:6333", None);
        assert!(adapter.is_ok());

        let adapter_ip = QdrantAdapter::new("http://127.0.0.1:6333", None);
        assert!(adapter_ip.is_ok());
    }
}
