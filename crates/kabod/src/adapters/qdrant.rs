use async_trait::async_trait;
use qdrant_client::Payload;
use qdrant_client::Qdrant;
use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, DeletePointsBuilder, Distance, Filter, PointStruct, ScoredPoint,
    SearchPointsBuilder, UpsertPointsBuilder, VectorParams, VectorsConfig, vectors_config::Config,
};

use crate::db::VectorDatabase;
use crate::error::{KabodError, Result};
use crate::types::{
    self, CollectionSchema, DistanceMetric, MetadataUpdate, Point, SearchResult, VectorQuery,
};

pub struct QdrantAdapter {
    client: Qdrant,
}

impl QdrantAdapter {
    pub fn new(url: &str, api_key: Option<&str>) -> Result<Self> {
        let mut config = QdrantConfig::from_url(url);

        if let Some(key) = api_key {
            config.set_api_key(key);
        }

        let client = Qdrant::new(config).map_err(|e| KabodError::Database(e.to_string()))?;

        Ok(Self { client })
    }
}

#[async_trait]
impl VectorDatabase for QdrantAdapter {
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
            .map_err(|e| KabodError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.client
            .delete_collection(name)
            .await
            .map_err(|e| KabodError::Database(e.to_string()))?;
        Ok(())
    }

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
            .map_err(|e: qdrant_client::QdrantError| KabodError::Database(e.to_string()))?;

        Ok(())
    }

    async fn search(&self, query: &VectorQuery) -> Result<Vec<SearchResult>> {
        let mut builder = SearchPointsBuilder::new(
            query.collection.clone(),
            query.vector.clone(),
            query.top_k as u64,
        )
        .with_payload(query.include_metadata)
        .with_vectors(query.include_vector);

        if let Some(filter) = &query.filter {
            builder = builder.filter(convert_filter(filter));
        }

        let result = self
            .client
            .search_points(builder)
            .await
            .map_err(|e| KabodError::Database(e.to_string()))?;

        Ok(result
            .result
            .into_iter()
            .map(convert_scored_point)
            .collect())
    }

    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()> {
        let points = qdrant_client::qdrant::PointsIdsList {
            ids: ids.into_iter().map(|id| id.into()).collect(),
        };

        self.client
            .delete_points(DeletePointsBuilder::new(collection).points(points))
            .await
            .map_err(|e| KabodError::Database(e.to_string()))?;

        Ok(())
    }

    async fn update_metadata(
        &self,
        _collection: &str,
        _updates: Vec<MetadataUpdate>,
    ) -> Result<()> {
        // TODO: Implement using set_payload
        Err(KabodError::NotImplemented("update_metadata".to_string()))
    }
}

fn convert_filter(_filter: &types::Filter) -> Filter {
    // TODO: Implement full filter conversion
    Filter::default()
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
