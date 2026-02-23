# Changelog

All notable changes to BridgeRust projects will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.20] - 2026-02-23

### Fixed

- **Release reliability**:
  - Fixed Node.js publish pipeline cache initialization failures in GitHub Actions by disabling auto package-manager cache for release jobs.
  - Added explicit tag/version validation in Node.js publish jobs to prevent publishing mismatched package versions.
  - Added clear manifest existence checks in Python publish jobs for actionable failures when release metadata files are missing.

## [0.2.2] - 2026-02-23

### Changed

- **BridgeTime API surface**:
  - Removed uppercase public binding exports (`BridgeTime`, `BridgeDuration`) from Python and Node.js package entrypoints.
  - Standardized public constructor/class exports to lowercase names:
    - Python: `bridge_time`, `bridge_duration`
    - Node.js: `bridgeTime`, `bridgeDuration`
  - Updated binding tests, docs, and package smoke-test script to the lowercase API.

## [0.2.1] - 2026-02-22

### Changed

- **BridgeTime release maintenance**:
  - Bumped BridgeTime package versions for Python (`bridgetime`), Node.js (`@bridgerust/bridgetime`), and Rust bridge crate (`bridgetime-bridge`) to `0.2.1`.
  - No API changes from `0.2.0`; this is a packaging/maintenance patch release.

## [0.2.0] - 2026-02-22

### Added

- **BridgeTime parity expansion**:
  - Added Day.js-style calendar, comparison, and range helpers.
  - Added explicit component getters/setters and week/year helpers.
  - Added relative-day and relative-time helpers.
  - Added custom parse helper (`parse_format` / `parseFormat`).
  - Added timezone helpers (`utc_offset`, `is_utc`, `is_dst`).
  - Added array and ordering helpers (`from_array`, `to_array`, `min`, `max`, `clamp`).
  - Added duration API (`BridgeDuration`, duration factories/converters/humanize, `add_duration`, `subtract_duration`).

### Fixed

- **BridgeTime correctness**:
  - Fixed `end_of(unit)` boundary computation and added regression coverage for day/week/month boundaries.

### Added

- **BridgeTime (preview)**:
  - Added new `bridgetime-bridge` Rust crate with BridgeRust exports for Python and Node.js.
  - Implemented Day.js/Moment-style datetime API: parsing, formatting, timezone conversion, immutable arithmetic, `start_of`, `end_of`, and `diff`.
  - Added Day.js-style calendar helpers: `get`, `set`, `days_in_month`, `is_leap_year`, `is_valid`.
  - Added comparison/range helpers: `is_before_unit`, `is_after_unit`, `is_same_unit`, `is_same_or_before`, `is_same_or_after`, `is_between`.
  - Added explicit Day.js-style component methods (`year`, `month`, `date`, `day`, `hour`, `minute`, `second`, `millisecond`) plus setter variants.
  - Added day/week parity helpers: `day_of_year`, `set_day_of_year`, `week`, `week_of_year`, `set_week`, `iso_week`, `set_iso_week`.
  - Added calendar/week parity helpers: `quarter`, `set_quarter`, `iso_weekday`, `set_iso_weekday`.
  - Added relative-day helpers: `is_today`, `is_yesterday`, `is_tomorrow`.
  - Added relative-time helpers: `from_time`, `to_time`, `from_now`, `to_now`.
  - Added unit-aware comparison helpers: `is_same_or_before_unit`, `is_same_or_after_unit`.
  - Added custom Day.js-style parse helper: `parse_format` / `parseFormat`.
  - Added timezone/year parity helpers: `utc_offset`, `is_utc`, `iso_week_year`, `days_in_year`, `weeks_in_year`, `iso_weeks_in_year`.
  - Added array and ordering parity helpers: `from_array`, `to_array`, `BridgeTime.min`, `BridgeTime.max`, `clamp`, `is_dst`.
  - Added duration parity helpers: `BridgeDuration`, `BridgeTime.duration`, `add_duration`, `subtract_duration`, duration conversion/humanize APIs.
  - Added package scaffolding:
    - Python: `bindings/python/bridgetime`
    - Node.js: `bindings/node/@bridgerust/bridgetime`
  - Added CI checks for `bridgetime-bridge` (base + python/nodejs features).

### Fixed

- **BridgeTime correctness**:
  - Fixed `end_of(unit)` boundary calculation to use `start_of(unit) + 1 unit - 1ms`, ensuring correct results for units like `day` and `week`.

### Changed

- **Release automation**:
  - Added dedicated BridgeTime tag workflows:
    - `.github/workflows/release-python-bridgetime.yml`
    - `.github/workflows/release-node-bridgetime.yml`
    - `.github/workflows/release-bridgetime.yml`
  - Generalized `.github/scripts/publish-npm-platforms.sh` to support reusable package/tag settings (used by both Embex and BridgeTime lanes).
  - Fixed BridgeTime Node release build command to use `npx napi build` in CI.
  - Added BridgeTime PyPI auth fallback: `PYPI_API_TOKEN` if provided, otherwise Trusted Publisher mode.
  - Fixed BridgeTime Node release artifact generation by emitting `.node` binaries directly from cross-target `cargo build` outputs.
  - Added versioned BridgeTime `native.js` / `native.d.ts` loader files for stable package publish and TypeScript compilation.
  - Removed unnecessary `protoc` installation from BridgeTime release workflows to avoid flaky Windows package-feed failures.
  - Narrowed BridgeTime Node release matrix to supported dynamic targets (darwin x64/arm64, linux x64 gnu, win32 x64 msvc).
  - Fixed npm platform publish tag resolution by using explicit `NPM_TAG_NAME` input instead of relying on immutable GitHub ref variables.

## [0.1.19] - 2026-02-22

### Fixed

- **Embex release CI/CD**:
  - Fixed Python publish job checkout and tag/version validation for tag/manual runs.
  - Replaced Bun-based Node release steps with Node/npm tooling to avoid macOS Intel Bun download failures.
  - Added manual tag input handling in Embex publish workflows.

## [0.1.18] - 2026-02-18

### Fixed

- **Adapter correctness**:
  - Weaviate now preserves caller IDs through `embex_id` and uses deterministic UUID mapping.
  - Chroma now persists metadata on insert and correctly translates `must_not` filters.
  - Qdrant count aggregation now fails with a clear error when response payload is missing.
- **Migration reliability**:
  - Migration state reads now paginate via `scroll` instead of a fixed-size fetch.
  - Migration execution now validates duplicate versions before running.
- **Streaming and batching validation**:
  - Added validation for `batch_size > 0` and `parallel > 0` in batch/stream insertion paths.
  - Data migrator now rejects zero-sized batches explicitly.

### Changed

- **Performance**:
  - Pinecone adapter now caches index hosts to reduce control-plane requests.
  - PgVector adapter now performs transactional chunked upserts for higher insert throughput.
- **Core SIMD**:
  - Added runtime SIMD backend detection (Scalar/AVX2/SSE4.1/NEON).
  - Added fallible SIMD APIs (`try_*`) for safer embedding/vector operations.
- **Capability introspection**:
  - Added provider capability metadata via `get_provider_capabilities()` and `EmbexClient::capabilities()`.

### Added

- **Bridge CLI wiring**:
  - `embex-bridge` now routes Python/Node CLI entrypoints to `embex-cli`.
- **Python DX**:
  - Python binding now accepts plain dict points in addition to `Point` objects.
  - `insert_stream` now validates `batch_size` and returns explicit errors for invalid items.
- **Release operations**:
  - Added `docs/RELEASE.md` runbook.
  - Standardized release tagging around `v*` across GitHub workflows.

## [0.1.17] - 2026-01-13

### Added

- **Data Migration**: Added `DataMigrator` utility for moving data between providers.
- **Pagination**: Added `scroll()` method to `Collection` and `VectorDatabase` trait.
- **Adapters**: Implemented `scroll` support for all adapters (Chroma, Pinecone, Milvus, Weaviate, PgVector, LanceDB, Qdrant).

## [0.1.16] - 2026-01-05

### Fixed

- **CI/CD**: Fixed multiple issues in the publish workflow:
  - Resolved `cargo metadata` failure by preserving `path` dependencies during version bump.
  - Added `protobuf-compiler` installation for `lance-encoding` dependency.
  - Corrected `bridge` CLI test command (removed `--lib` flag).
  - Fixed crate verification URLs to use correct `api/v1` endpoints.
- **Lints**: Resolved `clippy::collapsible_if` warnings across crates (`bridgerust-macros`, `bridge` CLI).
- **Metadata**: Fixed output filename collision between `embex-bridge` crate and `embex` python binding.

### Changed

- **Metadata**: Refactored `Cargo.toml` files to inherit `authors`, `license`, `repository`, and `homepage` from workspace.
- **Documentation**: Updated `README.md` with new status tables, badges, and "BridgeRust (Framework)" section.

### Added - Embex

#### Performance Optimizations

- **SIMD Optimizations**: Added SIMD-accelerated vector operations for x86_64 (AVX2, SSE4.1) and aarch64 (NEON)
  - Dot product, L2 distance, cosine similarity, normalization, and L2 norm
  - Feature-gated with `simd` Cargo feature
  - Automatic scalar fallback for unsupported platforms
  - Integrated into `Point` struct with helper methods
  - Comprehensive benchmarks showing 2-4x performance improvements

#### Connection Pooling

- **Connection Pooling**: Implemented connection pooling for all adapters
  - HTTP-based adapters (Pinecone, Milvus, Weaviate): `reqwest` client pooling
  - PgVector: `sqlx` connection pool
  - Qdrant and Chroma: Leverage internal client pooling
  - Configurable via `pool_size` and `idle_timeout_secs` in `EmbexConfig`
  - Documentation and verification utilities added

#### Observability

- **Metrics Collection**: Comprehensive metrics for all operations
  - Operation counts (inserts, searches, deletes, creates)
  - Latency tracking (insert, search, delete latencies)
  - Error tracking (errors, retries, timeouts)
  - Helper methods for aggregated statistics (error rates, average latencies)
- **Structured Logging**: Integrated `tracing` for structured logging
  - Instrumentation on all major operations
  - Collection-level context in spans
  - Configurable via `init_tracing()`

#### Migration System

- **Enhanced Migration System**: Robust migration management
  - Automatic rollback on migration failure
  - `rollback_migrations()` and `rollback_last()` for flexible rollback
  - `validate_migrations()` to check for duplicate versions
  - `validate_migration_state()` for consistency checks
  - `get_latest_migration()` to retrieve most recent applied migration
  - Improved `ensure_migration_table()` with robust collection existence checks
  - Comprehensive test coverage

#### Node.js API Parity

- **New Methods**: Complete API parity with Python bindings
  - `updateMetadata()`: Update metadata for existing points
  - `buildQuery()`: Filter-only query builder (no vector search)
  - `aggregation()`: Aggregations on SearchBuilder and QueryBuilder
  - Full TypeScript support with generated type definitions

#### Query Builder Enhancements

- **QueryBuilder**: Enhanced query building capabilities
  - Filter-only queries without vector search
  - Aggregation support (count)
  - Method chaining for fluent API
  - Available in Rust, Python, and Node.js

### Changed - Embex

#### Architecture Refactoring

- **Clean Architecture**: Improved separation of concerns
  - Created `bridge-embex-infrastructure` crate for cross-cutting concerns
  - Refactored `bridge-embex-core` to pure domain layer
  - Moved query builder from core to client (application layer)
  - Better dependency management and modularity

#### Error Handling

- **Enhanced Error Types**: More specific error handling
  - `CollectionNotFound`, `CollectionExists`, `DimensionMismatch`
  - `InvalidVector`, `Timeout`, `RateLimit`
  - Helper methods: `is_retryable()`, `is_collection_error()`

### Documentation

#### API Documentation

- **Complete API Reference**: Comprehensive documentation for all languages
  - Rust API documentation with examples
  - Python API documentation with examples
  - Node.js/TypeScript API documentation with examples
  - All new methods documented (updateMetadata, buildQuery, aggregation)

#### Guides

- **Migration System Guide**: Complete guide for database migrations
  - Creating migrations
  - Running and rolling back migrations
  - Validation and best practices
- **Connection Pooling Guide**: Detailed connection pooling documentation
  - Configuration for each adapter
  - Best practices and recommendations
- **Observability Guide**: Metrics and tracing documentation
  - Setting up observability
  - Using metrics and tracing
  - Production recommendations

#### Examples

- **Enhanced Examples**: Updated examples to showcase new features
  - RAG system example with aggregations and metadata updates
  - Semantic search example with filter-only queries
  - Real-world usage patterns

### Testing

#### Test Coverage

- **Comprehensive Test Suite**: Extensive test coverage
  - Property-based testing with `proptest`
  - Unit tests for all adapters
  - Integration tests for Node.js bindings
  - Migration system tests
  - Observability tests

#### Test Organization

- **Reorganized Test Structure**: Better test organization
  - Python: `tests/unit/` and `tests/integration/`
  - Node.js: `tests/unit/` and `tests/integration/`
  - Test configuration files and READMEs

### Performance

#### Benchmarks

- **Performance Benchmarking**: Comprehensive benchmarks
  - SIMD vs scalar comparisons
  - Native client comparisons
  - Overhead measurements
  - Documentation in `docs/PERFORMANCE.md`

## [0.1.0] - Initial Release

### Added - Embex

#### Core Features

- Unified API for multiple vector database providers
- Support for Qdrant, Pinecone, Chroma, Weaviate, Milvus, PgVector, LanceDB
- Rust, Python, and Node.js bindings
- Query builder with filters and aggregations
- Batch operations with parallel execution
- Streaming support for large datasets

#### Adapters

- Qdrant adapter
- Pinecone adapter
- Chroma adapter
- Weaviate adapter
- Milvus adapter
- PgVector adapter
- LanceDB adapter

#### Features

- Vector similarity search
- Metadata filtering
- Collection management
- Point insertion and deletion
- Distance metrics (cosine, euclidean, dot product)

---

## Upgrade Guide

### From 0.1.0 to Unreleased

#### SIMD Optimizations

- Enable SIMD feature: `bridge-embex = { version = "0.1", features = ["simd"] }`
- Use new `Point` helper methods: `point.cosine_similarity()`, `point.l2_distance()`, etc.

#### Connection Pooling

- Configure pooling via `EmbexConfig`:
  ```rust
  let config = EmbexConfig {
      pool_size: 10,
      idle_timeout_secs: 90,
      ..Default::default()
  };
  ```

#### Migration System

- Use new migration methods:
  ```rust
  manager.rollback_last(2, &migrations).await?;
  manager.validate_migrations(&migrations)?;
  ```

#### Node.js API

- New methods available:
  ```typescript
  await collection.updateMetadata([...]);
  const builder = collection.buildQuery();
  await builder.aggregation("count").execute();
  ```

---

## Notes

- All changes are backward compatible unless otherwise noted
- Performance improvements are automatic (no code changes required)
- New features are opt-in via feature flags or explicit usage
