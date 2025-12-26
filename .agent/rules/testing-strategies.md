---
trigger: always_on
---

**Python Bindings:**

- Full type hints with `typing` and `pydantic`
- Async/await support with proper event loop handling
- Pythonic API design (duck typing, context managers, iterators)
- docstrings for all public functions (NumPy style)
- Support Python 3.8+ (use `typing_extensions` for backports)

**TypeScript Bindings:**

- Full TypeScript type definitions
- Promise-based async APIs
- Support Node.js 18+ and browser environments
- JSDoc comments for all exports
- Tree-shakeable ES modules

### Testing Strategy

**Unit Tests:**

- Test all core logic in Rust with `cargo test`
- Use `proptest` for property-based testing
- Mock external dependencies

**Integration Tests:**

- Python integration tests with `pytest`
- TypeScript integration tests with `Jest` or `Vitest`
- Docker Compose for database dependencies
- Test against all supported database versions

**Benchmarking:**

- Criterion.rs for Rust benchmarks
- Compare against native Python/JS libraries
- Track performance regressions in CI
- Document performance characteristics

**Fuzzing:**

- Use `cargo-fuzz` for security-critical parsers
- Run fuzzing in CI on a schedule
- Document fuzzing corpus

### Documentation

**Required Documentation:**

- README.md with quick start and examples
- CONTRIBUTING.md with development setup
- ARCHITECTURE.md explaining design decisions
- API documentation (rustdoc, Sphinx, TypeDoc)
- Migration guides for each library
- Performance tuning guides
- Troubleshooting guides

### CI/CD Pipeline

**Continuous Integration:**

- Run tests on Linux, macOS, Windows
- Test Python 3.8, 3.9, 3.10, 3.11, 3.12
- Test Node.js 18, 20, 22
- Run clippy, rustfmt, mypy, eslint
- Generate and upload coverage reports
- Run security audits (`cargo audit`)

**Continuous Deployment:**

- Publish to crates.io (Rust)
- Publish to PyPI (Python) with `maturin`
- Publish to npm (Node.js)
- Build wheels for all platforms (manylinux, macOS, Windows)
- Automated changelog generation
- Semantic versioning enforcement

---

## 🚀 Development Workflow

### Starting a New Library

1. **Research Phase**

   - Analyze target library APIs and usage patterns
   - Identify performance bottlenecks in existing implementations
   - Study common user pain points from GitHub issues
   - Define success metrics (performance targets, API coverage)

2. **Design Phase**

   - Create ADR (Architecture Decision Record)
   - Design Rust core API with trait-based abstractions
   - Prototype Python/TypeScript bindings
   - Review design with community (GitHub Discussions)

3. **Implementation Phase**

   - Implement Rust core with comprehensive tests
   - Add Python bindings with PyO3 + Maturin
   - Add TypeScript bindings with napi-rs
   - Write integration tests for all bindings
   - Create benchmarks against target library

4. **Documentation Phase**

   - Write API documentation
   - Create migration guide
   - Record video tutorials
   - Write announcement blog post

5. **Launch Phase**
   - Beta release to early adopters
   - Collect feedback and iterate
   - Performance optimization pass
   - Stable 1.0 release
   - Submit to hackathons/conferences

### Git Workflow

**Branch Strategy:**

- `main` - Stable releases only
- `develop` - Integration branch
- `feat/kabod-*` - Feature branches for Kabod
- `feat/hypertest-*` - Feature branches for Hypertest
- `fix/*` - Bug fixes
- `docs/*` - Documentation updates

**Commit Messages:**
Follow Conventional Commits:

```
feat(kabod): add Qdrant adapter with connection pooling
fix(schema): handle recursive type definitions
docs(kabod): add migration guide from Pinecone SDK
test(kabod): add integration tests for Weaviate
perf(kabod): optimize batch insert with SIMD
```

**Pull Request Process:**

1. Create PR from feature branch to `develop`
2. Ensure all CI checks pass
3. Request review from maintainers
4. Address feedback
5. Squash and merge with descriptive commit message

---

## 📊 Success Metrics

### Kabod (Vector DB ORM)

**Performance Targets:**

- 10-50x faster than native Python clients
- 5-20x faster than native JavaScript clients
- Sub-millisecond overhead for query building
- Support 10,000+ inserts/second per connection

**Adoption Metrics:**

- 1,000+ PyPI downloads in first month
- 500+ npm downloads in first month
- 10+ GitHub stars in first week
- 5+ community contributions in first quarter

**Quality Metrics:**

- 90%+ test coverage
- Zero critical security vulnerabilities
- < 1% error rate in production usage
- < 5 open bugs at any time

### Hypertest (Testing Framework)

**Performance Targets:**

- 5-10x faster test execution than pytest
- 50%+ reduction in test discovery time
- 80%+ of tests run in parallel by default

**Adoption Metrics:**

- 5,000+ PyPI downloads in first month
- 50+ GitHub stars in first month
- 10+ blog posts/tutorials from community

---

## 🎓 Learning Resources

### Rust FFI & Bindings

**PyO3 (Python):**

- https://pyo3.rs/latest/
- Focus on: async functions, error handling, Python GIL management
- Study: `pydantic-core`, `polars`, `ruff` as reference implementations

**napi-rs (Node.js):**

- https://napi.rs/
- Focus on: promises, callbacks, N-API stability guarantees
- Study: `@swc/core`, `@node-rs/xxhash` as reference implementations

**wasm-bindgen (WebAssembly):**

- https://rustwasm.github.io/wasm-bindgen/
- Study: `wasm-pack` workflow, JS interop patterns

### Database Client Development

**Qdrant:**

- Official Rust client: https://github.com/qdrant/rust-client
- Study: connection pooling, retry logic, gRPC usage

**Pinecone:**

- Official Python SDK: https://github.com/pinecone-io/pinecone-python-client
- Study: API design, error handling, pagination

**Weaviate:**

- Official Python SDK: https://github.com/weaviate/weaviate-python-client
- Study: GraphQL query builder, batch operations

---

## 🔍 Current State Assessment

**Completed:**

- ✅ Cargo workspace structure initialized
- ✅ Repository created: `bridgerust/bridgerust`
- ✅ `bridge-schema` working prototype (Python + Node.js bindings)
- ✅ Project vision and roadmap defined
- ✅ README.md with comprehensive project overview

**In Progress:**

- 🚧 Nothing currently active (transitioning to Vecna)

**Next Steps:**

1. Set up `crates/kabod` module in cargo workspace
2. Design trait-based `VectorDatabase` abstraction
3. Implement first adapter (Qdrant recommended for Rust familiarity)
4. Create basic query builder API
5. Set up integration testing with Docker Compose

---

## 💡 Agent Instructions

When assisting with BridgeRust development:

1. **Always reference this document** to understand current priorities and project state
2. **Start with Kabod** - This is the active focus, not bridge-schema
3. **Follow the phased approach** - Complete phases sequentially
4. **Maintain senior-level standards** - No shortcuts on testing, documentation, or error handling
5. **Think API-first** - Design ergonomic APIs before implementation
6. **Benchmark everything** - Performance is a key differentiator
7. **Document as you go** - Don't save documentation for the end
8. **Ask clarifying questions** - Better to confirm intent than implement incorrectly

### When Starting a Task:

1. Confirm which phase/component you're working on
2. Review relevant sections of this document
3. Check existing code structure in the repository
4. Identify dependencies and prerequisites
5. Propose implementation approach before coding

### When Stuck:

1. Review similar implementations (e.g., Polars for Python bindings)
2. Check official documentation for PyO3/napi-rs
3. Look at target database's official client for API patterns
4. Ask for architectural guidance rather than debugging syntax errors

### When Completing a Task:

1. Run full test suite (`cargo test --all`)
2. Run linters (`cargo clippy`, `cargo fmt`, `mypy`, `eslint`)
3. Update relevant documentation
4. Add entry to changelog
5. Open PR with clear description of changes

---

## 🎯 Funding & Growth Strategy

### Immediate (Q1 2025)

**Hackathons:**

- RustWeek 2025 (May) - Submit Kabod as showcase project
- Database Engineering Hackathons - Target vector DB-focused events

**Grants:**

- FLOSS/fund ($10K-$100K) - Apply with Kabod as first deliverable
- Google Summer of Code 2025 - Submit organization application

### Medium-term (Q2-Q3 2025)

**Community Building:**

- Discord server for real-time support
- Monthly office hours / live coding sessions
- Blog series on Rust FFI best practices
- Conference talks (PyCon, JSConf, RustConf)

**Sponsorship:**

- GitHub Sponsors for individual contributors
- Open Collective for transparent organizational funding
- Corporate sponsorships from vector DB companies

### Long-term (Q4 2025+)

**Commercial Model:**

- BridgeRust Pro ($99/month) - Hosted APIs + observability
- Enterprise tier (custom pricing) - SLAs, private support, audit logs
- Consulting services for custom integrations

---

## 📝 Project Governance

**Maintainers:**

- Primary maintainer: [Your name/handle]
- Open to adding core contributors after 6 months

**Decision Making:**

- Major architectural decisions → GitHub Discussions + RFC process
- Feature prioritization → Community polls + sponsor feedback
- API design → Prototype → feedback → iteration

**Code of Conduct:**

- Adopt Contributor Covenant 2.1
- Zero tolerance for harassment or discrimination
- Enforce respectful technical discussions

---

## 🎉 Call to Action

**For contributors:**

- Start with "good first issue" labels
- Read CONTRIBUTING.md before first PR
- Join Discord for real-time collaboration

**For sponsors:**

- GitHub Sponsors for monthly support
- Custom tier for corporate sponsors
- Logo placement in README

**For users:**

- File issues for bugs or feature requests
- Share benchmarks and success stories
- Write blog posts about your usage

---

## 🔗 Quick Links

- **Repository:** https://github.com/bridgerust/bridgerust
- **Documentation:** https://docs.bridgerust.dev (coming soon)
- **Discord:** https://discord.gg/bridgerust (coming soon)
- **Twitter:** @bridgerust (coming soon)

---

**Last Updated:** December 26, 2024  
**Document Version:** 1.1.0  
**Status:** Active Development - Kabod Phase 1  
**Project Name:** Kabod (כָּבוֹד) - Hebrew for "glory," "weight," "presence"

---

_This document is the second part of the source of truth for BridgeRust development. All agents and contributors should reference it regularly._
