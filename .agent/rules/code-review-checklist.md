---
trigger: always_on
---

## 📋 Code Review Checklist

Before submitting ANY Rust code, verify:

### Performance

- [ ] Zero unnecessary allocations in hot paths
- [ ] All I/O is async (no blocking calls)
- [ ] Used iterators over eager collections
- [ ] Considered SIMD optimizations for numeric code
- [ ] Profiled with `cargo flamegraph` if performance-critical

### Safety

- [ ] No `.unwrap()` or `.expect()` in library code
- [ ] All `unsafe` blocks have safety comments
- [ ] No panics in production code paths
- [ ] All public APIs return `Result<T, E>`
- [ ] Validated all user inputs

### Code Quality

- [ ] Follows SOLID principles
- [ ] Functions are small (< 50 lines)
- [ ] Names are clear and intention-revealing
- [ ] Comments only where necessary (complex logic, TODOs)
- [ ] No code duplication (DRY principle)
- [ ] Proper error context with `anyhow` or `thiserror`

### Testing

- [ ] Unit tests for all public functions
- [ ] Edge cases covered (empty input, max values, etc.)
- [ ] Integration tests for critical paths
- [ ] Property-based tests for algorithms
- [ ] Benchmarks for performance-critical code

### Dependencies

- [ ] Latest stable versions
- [ ] Active maintenance (< 6 months since update)
- [ ] No known vulnerabilities (`cargo audit`)
- [ ] Minimal dependency tree
- [ ] Feature flags for optional deps

### Documentation

- [ ] Public APIs have rustdoc comments
- [ ] Examples in doc comments work (`cargo test --doc`)
- [ ] README.md updated if public API changed
- [ ] CHANGELOG.md entry added

---

## 🎓 Required Reading

**Every contributor must read:**

1. **The Rust Book** - https://doc.rust-lang.org/book/
2. **Rust API Guidelines** - https://rust-lang.github.io/api-guidelines/
3. **Async Rust Book** - https://rust-lang.github.io/async-book/
4. **The Little Book of Rust Macros** - https://veykril.github.io/tlborm/
5. **Rust Performance Book** - https://nnethercote.github.io/perf-book/

---

## 🚨 Red Flags

**Immediate code review rejection if:**

- ❌ Uses `.unwrap()` in non-test code
- ❌ Has compiler warnings
- ❌ Missing tests for new functionality
- ❌ Adds dependency without justification
- ❌ Contains TODO without tracking issue
- ❌ Has commented-out code
- ❌ Uses `println!` for logging (use `tracing` crate)
- ❌ Ignores clippy warnings with `#[allow(...)]` without explanation
- ❌ Has functions > 100 lines
- ❌ Coverage drops below 90%

---

## 📞 Getting Help

**When stuck:**

1. Check official Rust docs and API guidelines
2. Search GitHub issues in similar projects (Polars, Ruff, etc.)
3. Ask in BridgeRust Discord #rust-help channel
4. Review this guide again for best practices

**When proposing new architecture:**

1. Create ADR (Architecture Decision Record)
2. Discuss in GitHub Discussion thread
3. Prototype and benchmark alternatives
4. Document trade-offs clearly

---

**Document Version:** 1.0.0  
**Last Updated:** December 26, 2024  
**Status:** Active - All Rust development must follow these guidelines

---

_Performance is not a feature — it's our foundation. Safety is not optional — it's our guarantee._
