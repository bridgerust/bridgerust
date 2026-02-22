# Changelog

All notable changes to the Embex Node.js bindings will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.19] - 2026-02-22

### Fixed

- Replaced Bun-based release build/publish pipeline with Node/npm tooling for improved runner compatibility.
- Added manual release tag input support in the Node publish workflow.

## [0.1.18] - 2026-02-18

### Fixed

- Release pipeline now keeps platform `optionalDependencies` versions aligned with the main package version during publish.

### Changed

- GitHub release flow is now tag-driven (`v*`) with a dedicated release creation workflow.

## [0.1.16] - 2026-01-05

### Added

#### Performance

- **SIMD Optimizations**: Added SIMD-accelerated vector operations with scalar fallback.
- **Connection Pooling**: Implemented internal pooling for Qdrant and Chroma adapters.

#### Features

- **API Parity**: Added new methods to match Python SDK (`updateMetadata`, `buildQuery`, `aggregation`).
- **Query Builder**: Added support for filter-only queries and aggregations.
- **Observability**: Added metrics collection and structured logging instrumentation.

### Changed

- **Architecture**: Refactored internal architecture for better separation of concerns.
- **Error Handling**: Introduced more specific error types and better exception messages.

### Fixed

- **Output Filename**: Resolved filename collision with `embex-bridge` crate.
