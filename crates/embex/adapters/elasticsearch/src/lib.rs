use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use bridge_embex_core::db::VectorDatabase;
use bridge_embex_core::error::{EmbexError, Result};
use bridge_embex_core::types::{
    CollectionSchema, Condition, DistanceMetric, Filter, MetadataUpdate, Point, ScrollResponse,
    SearchResponse, SearchResult, VectorQuery,
};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode, Url};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

const MAX_WINDOW: usize = 10_000;
const MAX_VECTOR_DIMENSION: usize = 4_096;
const BULK_BATCH_SIZE: usize = 500;
const PIT_KEEP_ALIVE: &str = "1m";

/// An Elasticsearch vector index adapter for Embex-managed collections.
#[derive(Debug, Clone)]
pub struct ElasticsearchAdapter {
    url: String,
    http: Client,
}

impl ElasticsearchAdapter {
    pub fn new(url: &str) -> Result<Self> {
        Self::with_api_key(url, None)
    }

    /// Connect using the encoded Elasticsearch API key if the cluster requires authentication.
    pub fn with_api_key(url: &str, api_key: Option<&str>) -> Result<Self> {
        let parsed_url = Url::parse(url.trim())
            .map_err(|_| EmbexError::Config("Elasticsearch requires an HTTP(S) URL".into()))?;
        if !matches!(parsed_url.scheme(), "http" | "https")
            || parsed_url.host().is_none()
            || parsed_url.query().is_some()
            || parsed_url.fragment().is_some()
            || !parsed_url.username().is_empty()
            || parsed_url.password().is_some()
        {
            return Err(EmbexError::Config(
                "Elasticsearch requires an HTTP(S) base URL without credentials, query or fragment"
                    .into(),
            ));
        }

        let mut headers = HeaderMap::new();
        if let Some(key) = api_key {
            let key = key.trim();
            if key.is_empty() {
                return Err(EmbexError::Config("Elasticsearch API key is empty".into()));
            }
            let mut header = HeaderValue::from_str(&format!("ApiKey {key}"))
                .map_err(|_| EmbexError::Config("Invalid Elasticsearch API key".into()))?;
            header.set_sensitive(true);
            headers.insert(AUTHORIZATION, header);
        }
        let http = Client::builder()
            .timeout(Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::none())
            .default_headers(headers)
            .build()
            .map_err(|error| EmbexError::Config(error.to_string()))?;

        Ok(Self {
            url: parsed_url.as_str().trim_end_matches('/').into(),
            http,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        self.http.request(method, format!("{}/{path}", self.url))
    }

    async fn bulk(&self, collection: &str, action: &str, body: String, count: usize) -> Result<()> {
        let response = send(
            self.request(Method::POST, &format!("{collection}/_bulk"))
                .query(&[("refresh", "wait_for")])
                .header(CONTENT_TYPE, "application/x-ndjson")
                .body(body),
            collection,
        )
        .await?;
        let response: BulkResponse = decode(response).await?;
        if response.items.len() != count {
            return Err(EmbexError::Database(
                "Elasticsearch returned an unexpected number of bulk results".into(),
            ));
        }
        for item in response.items {
            let result = item.get(action).ok_or_else(|| {
                EmbexError::Database("Elasticsearch returned a mismatched bulk operation".into())
            })?;
            if (200..300).contains(&result.status)
                || (action == "delete" && result.status == 404 && result.error.is_none())
            {
                continue;
            }
            let status = StatusCode::from_u16(result.status)
                .map_err(|error| EmbexError::Database(error.to_string()))?;
            let details = result.error.as_ref();
            return Err(server_error(
                status,
                collection,
                details.and_then(|error| error["type"].as_str()),
                &format!(
                    "bulk {action} failed for document {}: {}",
                    result.id.as_deref().unwrap_or("<unknown>"),
                    details.map_or_else(|| status.to_string(), Value::to_string)
                ),
            ));
        }
        Ok(())
    }

    async fn bulk_documents(
        &self,
        collection: &str,
        action: &str,
        documents: impl IntoIterator<Item = (String, Option<Value>)>,
    ) -> Result<()> {
        let mut body = String::new();
        let mut count = 0;
        for (id, document) in documents {
            let mut operation = json!({(action): {"_id": id}});
            if action == "update" {
                operation[action]["retry_on_conflict"] = json!(3);
            }
            body.push_str(&serde_json::to_string(&operation)?);
            body.push('\n');
            if let Some(document) = document {
                body.push_str(&serde_json::to_string(&document)?);
                body.push('\n');
            }
            count += 1;
            if count == BULK_BATCH_SIZE {
                self.bulk(collection, action, std::mem::take(&mut body), count)
                    .await?;
                count = 0;
            }
        }
        if count != 0 {
            self.bulk(collection, action, body, count).await?;
        }
        Ok(())
    }

    async fn close_pit(&self, collection: &str, id: &str) -> Result<()> {
        send(
            self.request(Method::DELETE, "_pit")
                .json(&json!({"id": id})),
            collection,
        )
        .await?;
        Ok(())
    }
}

#[async_trait]
impl VectorDatabase for ElasticsearchAdapter {
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<()> {
        validate_collection(&schema.name)?;
        if !(1..=MAX_VECTOR_DIMENSION).contains(&schema.dimension) {
            return Err(EmbexError::Validation(format!(
                "Elasticsearch dimensions must be between 1 and {MAX_VECTOR_DIMENSION}"
            )));
        }
        let similarity = match schema.metric {
            DistanceMetric::Cosine => "cosine",
            DistanceMetric::Euclidean => "l2_norm",
            DistanceMetric::Dot => "max_inner_product",
        };
        send(
            self.request(Method::PUT, &schema.name).json(&json!({
                "mappings": {
                    "dynamic": "strict",
                    "dynamic_templates": [
                        {"strings_as_keywords": {"match_mapping_type": "string", "mapping": {"type": "keyword"}}},
                        {"metadata_numbers": {"match_mapping_type": ["long", "double"], "mapping": {"type": "double"}}}
                    ],
                    "date_detection": false,
                    "properties": {
                        "vector": {
                            "type": "dense_vector", "dims": schema.dimension, "index": true,
                            "element_type": "float", "similarity": similarity
                        },
                        "metadata": {"type": "object", "dynamic": true}
                    }
                }
            })),
            &schema.name,
        )
        .await?;
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        validate_collection(name)?;
        send(self.request(Method::DELETE, name), name).await?;
        Ok(())
    }

    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<()> {
        validate_collection(collection)?;
        if points.is_empty() {
            return Ok(());
        }
        for point in &points {
            validate_id(&point.id)?;
            validate_vector(&point.vector)?;
        }
        // Bulk indexing can otherwise auto-create an index without the vector mapping.
        let response = self
            .request(Method::HEAD, collection)
            .send()
            .await
            .map_err(transport_error)?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(EmbexError::CollectionNotFound(collection.into()));
        }
        check_response(response, collection).await?;
        self.bulk_documents(
            collection,
            "index",
            points.into_iter().map(|point| {
                (
                    point.id,
                    Some(json!({"vector": point.vector, "metadata": point.metadata})),
                )
            }),
        )
        .await
    }

    async fn search(&self, query: &VectorQuery) -> Result<SearchResponse> {
        let collection = &query.collection;
        validate_collection(collection)?;
        let offset = query.offset.unwrap_or_default();
        let window = offset.saturating_add(query.top_k);
        if window > MAX_WINDOW {
            return Err(EmbexError::Query(format!(
                "Elasticsearch search offset plus limit cannot exceed {MAX_WINDOW}"
            )));
        }
        let filter = query
            .filter
            .as_ref()
            .map(filter_clause)
            .transpose()?
            .unwrap_or_else(|| json!({"match_all": {}}));
        if let Some(vector) = &query.vector {
            validate_vector(vector)?;
        }

        let mut body = json!({
            "size": query.top_k,
            "from": offset,
            "_source": source_fields(query.include_vector, query.include_metadata),
            "track_total_hits": false
        });
        if query.include_vector {
            // Elasticsearch 9 excludes vectors from _source by default.
            body["fields"] = json!(["vector"]);
        }

        let mut aggregations = HashMap::new();
        if !query.aggregations.is_empty() {
            let response = send(
                self.request(Method::POST, &format!("{collection}/_count"))
                    .json(&json!({"query": filter})),
                collection,
            )
            .await?;
            let count: CountResponse = decode(response).await?;
            if count.shards.failed != 0 {
                return Err(EmbexError::Database(
                    "Elasticsearch count returned partial results".into(),
                ));
            }
            aggregations.insert("count".into(), json!(count.count));
        }

        if query.top_k == 0 {
            return Ok(SearchResponse {
                results: Vec::new(),
                aggregations,
            });
        }
        if let Some(vector) = &query.vector {
            body["knn"] = json!({
                "field": "vector",
                "query_vector": vector,
                "k": window,
                "num_candidates": window.saturating_mul(2).clamp(100, MAX_WINDOW),
                "filter": filter
            });
        } else {
            body["query"] = filter;
        }

        let response = send(
            self.request(Method::POST, &format!("{collection}/_search"))
                .query(&[("allow_partial_search_results", "false")])
                .json(&body),
            collection,
        )
        .await?;
        let search: SearchHits = decode(response).await?;
        search.check_complete()?;
        let results = search
            .hits
            .hits
            .into_iter()
            .map(|hit| {
                let vector = query
                    .include_vector
                    .then(|| hit.vector())
                    .transpose()?
                    .flatten();
                Ok(SearchResult {
                    id: hit.id,
                    score: hit.score.unwrap_or(1.0),
                    vector,
                    metadata: if query.include_metadata {
                        hit.source.metadata
                    } else {
                        None
                    },
                })
            })
            .collect::<Result<_>>()?;
        Ok(SearchResponse {
            results,
            aggregations,
        })
    }

    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()> {
        validate_collection(collection)?;
        for id in &ids {
            validate_id(id)?;
        }
        self.bulk_documents(collection, "delete", ids.into_iter().map(|id| (id, None)))
            .await
    }

    async fn update_metadata(&self, collection: &str, updates: Vec<MetadataUpdate>) -> Result<()> {
        validate_collection(collection)?;
        for update in &updates {
            validate_id(&update.id)?;
        }
        self.bulk_documents(
            collection,
            "update",
            updates.into_iter().filter(|update| !update.updates.is_empty()).map(|update| {
                (
                    update.id,
                    Some(json!({
                        "script": {
                            "lang": "painless",
                            "source": "if (ctx._source.metadata == null) { ctx._source.metadata = [:]; } ctx._source.metadata.putAll(params.metadata);",
                            "params": {"metadata": update.updates}
                        }
                    })),
                )
            }),
        )
        .await
    }

    async fn scroll(
        &self,
        collection: &str,
        offset: Option<String>,
        limit: usize,
    ) -> Result<ScrollResponse> {
        validate_collection(collection)?;
        if !(1..MAX_WINDOW).contains(&limit) {
            return Err(EmbexError::Query(format!(
                "Elasticsearch scroll limit must be between 1 and {}",
                MAX_WINDOW - 1
            )));
        }

        let first_page = offset.is_none();
        let (mut pit_id, search_after) = if let Some(offset) = offset {
            let cursor: ScrollCursor = serde_json::from_str(&offset)
                .map_err(|_| EmbexError::Query("Invalid Elasticsearch scroll cursor".into()))?;
            if cursor.collection != collection || cursor.pit_id.is_empty() {
                return Err(EmbexError::Query(
                    "Invalid Elasticsearch scroll cursor".into(),
                ));
            }
            (cursor.pit_id, Some(cursor.search_after))
        } else {
            let response = send(
                self.request(Method::POST, &format!("{collection}/_pit"))
                    .query(&[("keep_alive", PIT_KEEP_ALIVE)]),
                collection,
            )
            .await?;
            let pit: PointInTime = decode(response).await?;
            (pit.id, None)
        };

        let mut body = json!({
            "pit": {"id": pit_id, "keep_alive": PIT_KEEP_ALIVE},
            "sort": ["_shard_doc"],
            "size": limit + 1,
            "track_total_hits": false,
            "_source": ["vector", "metadata"],
            "fields": ["vector"]
        });
        if let Some(search_after) = search_after {
            body["search_after"] = json!([search_after]);
        }

        let page = async {
            let response = send(
                self.request(Method::POST, "_search")
                    .query(&[("allow_partial_search_results", "false")])
                    .json(&body),
                collection,
            )
            .await?;
            let mut search: SearchHits = decode(response).await?;
            if let Some(id) = search.pit_id.take() {
                pit_id = id;
            }
            search.check_complete()?;
            let mut hits = search.hits.hits;
            let next_sort = if hits.len() > limit {
                hits.truncate(limit);
                let sort = hits[limit - 1].sort.first().copied().ok_or_else(|| {
                    EmbexError::Database("Elasticsearch scroll result has no sort key".into())
                })?;
                Some(sort)
            } else {
                None
            };

            let mut points = Vec::with_capacity(hits.len());
            for hit in hits {
                let vector = hit.vector()?.ok_or_else(|| {
                    EmbexError::Database(format!("Elasticsearch point {} has no vector", hit.id))
                })?;
                points.push(Point {
                    id: hit.id,
                    vector,
                    metadata: hit.source.metadata,
                });
            }
            Ok((points, next_sort))
        }
        .await;

        match page {
            Ok((points, Some(search_after))) => Ok(ScrollResponse {
                points,
                next_offset: Some(serde_json::to_string(&ScrollCursor {
                    collection: collection.into(),
                    pit_id,
                    search_after,
                })?),
            }),
            Ok((points, None)) => {
                let _ = self.close_pit(collection, &pit_id).await;
                Ok(ScrollResponse {
                    points,
                    next_offset: None,
                })
            }
            Err(error) => {
                if first_page {
                    let _ = self.close_pit(collection, &pit_id).await;
                }
                Err(error)
            }
        }
    }
}

fn validate_collection(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 255
        || !name.as_bytes()[0].is_ascii_alphanumeric()
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_.".contains(&b))
    {
        return Err(EmbexError::Validation(
            "Elasticsearch collection names must begin with a lowercase letter or digit and contain only lowercase letters, digits, hyphens, underscores or periods (up to 255 bytes)".into(),
        ));
    }
    Ok(())
}

fn validate_id(id: &str) -> Result<()> {
    if id.is_empty() || id.len() > 512 {
        return Err(EmbexError::Validation(
            "Elasticsearch IDs must be between 1 and 512 bytes".into(),
        ));
    }
    Ok(())
}

fn validate_vector(vector: &[f32]) -> Result<()> {
    if !(1..=MAX_VECTOR_DIMENSION).contains(&vector.len()) || !vector.iter().all(|v| v.is_finite())
    {
        return Err(EmbexError::InvalidVector(format!(
            "Elasticsearch vectors must have between 1 and {MAX_VECTOR_DIMENSION} finite elements"
        )));
    }
    Ok(())
}

fn source_fields(vector: bool, metadata: bool) -> Value {
    match (vector, metadata) {
        (true, true) => json!(["vector", "metadata"]),
        (true, false) => json!(["vector"]),
        (false, true) => json!(["metadata"]),
        (false, false) => json!(false),
    }
}

fn filter_clause(filter: &Filter) -> Result<Value> {
    Ok(match filter {
        Filter::Must(filters) => json!({"bool": {"filter": filter_clauses(filters)?}}),
        Filter::MustNot(filters) => json!({"bool": {"must_not": filter_clauses(filters)?}}),
        Filter::Should(filters) if filters.is_empty() => json!({"match_none": {}}),
        Filter::Should(filters) => json!({"bool": {
            "should": filter_clauses(filters)?, "minimum_should_match": 1
        }}),
        Filter::Key(key, condition) => {
            if key.split('.').any(str::is_empty) {
                return Err(EmbexError::Query(
                    "Elasticsearch metadata key cannot be empty".into(),
                ));
            }
            condition_clause(&format!("metadata.{key}"), condition)?
        }
    })
}

fn filter_clauses(filters: &[Filter]) -> Result<Vec<Value>> {
    filters.iter().map(filter_clause).collect()
}

fn condition_clause(field: &str, condition: &Condition) -> Result<Value> {
    let equal = |value: &Value| match value {
        Value::Null => Ok(json!({"bool": {"must_not": {"exists": {"field": field}}}})),
        Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            Ok(json!({"term": {(field): value}}))
        }
        _ => Err(EmbexError::Query(
            "Elasticsearch equality filters require a scalar".into(),
        )),
    };
    let terms = |values: &[Value]| -> Result<Value> {
        if values.is_empty() {
            return Ok(json!({"match_none": {}}));
        }
        // Decompose null comparisons into exists clauses. Plain terms works for scalars.
        if values.iter().any(Value::is_null) {
            let clauses = values.iter().map(equal).collect::<Result<Vec<_>>>()?;
            return Ok(json!({"bool": {"should": clauses, "minimum_should_match": 1}}));
        }
        if values.iter().any(|v| v.is_object() || v.is_array()) {
            return Err(EmbexError::Query(
                "Elasticsearch inclusion filters require scalars".into(),
            ));
        }
        Ok(json!({"terms": {(field): values}}))
    };
    let range = |op: &str, value: &Value| {
        if !value.is_number() && !value.is_string() {
            return Err(EmbexError::Query(
                "Elasticsearch range filters require a number or string".into(),
            ));
        }
        Ok(json!({"range": {(field): {(op): value}}}))
    };
    match condition {
        Condition::Eq(value) => equal(value),
        Condition::Ne(value) => Ok(json!({"bool": {"must_not": equal(value)?}})),
        Condition::In(values) => terms(values),
        Condition::NotIn(values) => Ok(json!({"bool": {"must_not": terms(values)?}})),
        Condition::Gt(value) => range("gt", value),
        Condition::Gte(value) => range("gte", value),
        Condition::Lt(value) => range("lt", value),
        Condition::Lte(value) => range("lte", value),
    }
}

async fn send(request: RequestBuilder, collection: &str) -> Result<Response> {
    let response = request.send().await.map_err(transport_error)?;
    check_response(response, collection).await
}

fn transport_error(error: reqwest::Error) -> EmbexError {
    if error.is_timeout() {
        EmbexError::Timeout(error.to_string())
    } else {
        EmbexError::Connection(error.to_string())
    }
}

async fn check_response(response: Response, collection: &str) -> Result<Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let body = response.text().await.unwrap_or_default();
    let error_type = serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|body| body["error"]["type"].as_str().map(str::to_owned));
    Err(server_error(
        status,
        collection,
        error_type.as_deref(),
        &body,
    ))
}

fn server_error(
    status: StatusCode,
    collection: &str,
    kind: Option<&str>,
    body: &str,
) -> EmbexError {
    let message = format!("Elasticsearch ({status}): {body}");
    match kind {
        Some("resource_already_exists_exception") => {
            EmbexError::CollectionExists(collection.into())
        }
        Some("index_not_found_exception") => EmbexError::CollectionNotFound(collection.into()),
        _ if status == StatusCode::TOO_MANY_REQUESTS => EmbexError::RateLimit(message),
        _ if status == StatusCode::BAD_REQUEST || status == StatusCode::NOT_FOUND => {
            EmbexError::Query(message)
        }
        _ => EmbexError::Database(message),
    }
}

async fn decode<T: DeserializeOwned>(response: Response) -> Result<T> {
    response
        .json()
        .await
        .map_err(|error| EmbexError::Database(format!("Invalid Elasticsearch response: {error}")))
}

#[derive(Deserialize)]
struct BulkResponse {
    items: Vec<HashMap<String, BulkResult>>,
}

#[derive(Deserialize)]
struct BulkResult {
    #[serde(rename = "_id")]
    id: Option<String>,
    status: u16,
    error: Option<Value>,
}

#[derive(Deserialize)]
struct CountResponse {
    count: u64,
    #[serde(rename = "_shards")]
    shards: ShardResults,
}

#[derive(Deserialize)]
struct ShardResults {
    failed: usize,
}

#[derive(Deserialize)]
struct PointInTime {
    id: String,
}

#[derive(Deserialize)]
struct SearchHits {
    pit_id: Option<String>,
    #[serde(default)]
    timed_out: bool,
    hits: HitList,
}

impl SearchHits {
    fn check_complete(&self) -> Result<()> {
        if self.timed_out {
            return Err(EmbexError::Timeout("Elasticsearch search timed out".into()));
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct HitList {
    hits: Vec<Hit>,
}

#[derive(Deserialize)]
struct Hit {
    #[serde(rename = "_id")]
    id: String,
    #[serde(rename = "_score")]
    score: Option<f32>,
    #[serde(default, rename = "_source")]
    source: Source,
    #[serde(default)]
    fields: HashMap<String, Value>,
    #[serde(default)]
    sort: Vec<u64>,
}

impl Hit {
    fn vector(&self) -> Result<Option<Vec<f32>>> {
        if let Some(vector) = &self.source.vector {
            return Ok(Some(vector.clone()));
        }
        let Some(value) = self.fields.get("vector") else {
            return Ok(None);
        };
        // Fields may wrap the vector in a one-element array.
        let vector = value.get(0).filter(|item| item.is_array()).unwrap_or(value);
        Ok(Some(serde_json::from_value(vector.clone())?))
    }
}

#[derive(Default, Deserialize)]
struct Source {
    vector: Option<Vec<f32>>,
    metadata: Option<HashMap<String, Value>>,
}

#[derive(Deserialize, Serialize)]
struct ScrollCursor {
    collection: String,
    pit_id: String,
    search_after: u64,
}
