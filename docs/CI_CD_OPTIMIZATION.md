# CI/CD Optimization Guide

## Changes Made to Save GitHub Actions Minutes

### 1. Dependabot Configuration

**Issues Fixed:**

- ✅ Fixed wrong paths (`kabod` → `embex`)
- ✅ Changed frequency from **weekly** to **monthly** (saves ~75% of PRs)
- ✅ Reduced PR limits (10→5 for Rust, 5→3 for others)
- ✅ Kept GitHub Actions updates weekly (security-critical)

**Impact:** Dependabot will create ~4x fewer PRs, each triggering CI workflows.

### 2. Integration Tests

**Before:** Ran on every push/PR to main/develop
**After:** Only runs on:

- Pushes to `main` branch
- Manual dispatch (`workflow_dispatch`)

**Why:** Integration tests are expensive (Docker, multiple databases, multiple languages)

### 3. Benchmarks

**Before:** Ran on every push/PR + weekly schedule
**After:** Only runs on:

- Monthly schedule (first Sunday of month)
- Manual dispatch

**Why:** Benchmarks are expensive and don't need to run on every change

### 4. CI Matrix Reduction

**Python:** 4 versions → 2 versions (3.10, 3.12)
**Node.js:** 3 versions → 2 versions (20, 22)
**Multi-platform:** Only runs on `main` branch

**Why:** At early phase, testing 2 versions per language is sufficient

### 5. Workflow File Cleanup

Removed `.github/workflows/dependabot.yml` (it was a workflow file, not config)

- Dependabot config is in `.github/dependabot.yml` (correct location)

## Estimated Minutes Savings

| Workflow          | Before                       | After                         | Savings             |
| ----------------- | ---------------------------- | ----------------------------- | ------------------- |
| Dependabot PRs    | ~4/week × 15min = 60min/week | ~1/month × 15min = 4min/month | **~240min/month**   |
| Integration Tests | Every push/PR                | Main only                     | **~50-100min/week** |
| Benchmarks        | Weekly + PRs                 | Monthly only                  | **~30min/week**     |
| CI Matrix         | 4 Python + 3 Node            | 2 Python + 2 Node             | **~40min/run**      |

**Total estimated savings: ~400-500 minutes/month**

## When to Re-enable

As the project grows, you can:

1. **Increase Dependabot frequency** when you have more minutes
2. **Add more Python/Node versions** when you have active users on those versions
3. **Re-enable integration tests on PRs** when you have more contributors
4. **Run benchmarks on PRs** when performance regressions become a concern

## Manual Testing

For now, you can manually trigger expensive workflows:

- Integration tests: Use "Run workflow" button in GitHub Actions
- Benchmarks: Use "Run workflow" button or wait for monthly schedule

## Dependabot PRs and CI

Dependabot PRs will still trigger CI workflows. To further reduce minutes:

1. **Add path filters** to skip CI for dependency-only changes:

```yaml
on:
  pull_request:
    paths-ignore:
      - "Cargo.lock"
      - "package-lock.json"
      - "**/Cargo.toml"
```

2. **Or use commit message patterns** to skip CI:

   - Configure Dependabot to add `[skip ci]` to commit messages
   - Add workflow condition: `if: "!contains(github.event.head_commit.message, '[skip ci]')"`

3. **Or disable CI for Dependabot PRs entirely** (not recommended for security)

## Current Workflow Summary

| Workflow              | Triggers         | Frequency       |
| --------------------- | ---------------- | --------------- |
| **CI**                | Every push/PR    | Always          |
| **Integration Tests** | Main branch only | On push to main |
| **Benchmarks**        | Monthly schedule | Once per month  |
| **Release**           | Tags only        | On release      |
| **Dependabot**        | Monthly check    | Once per month  |
