# Node.js Tests

## Structure

```
tests/
├── unit/              # Unit tests (no database required)
│   ├── basic.test.ts      # Basic functionality tests
│   └── errors.test.ts     # Error handling tests
│
└── integration/       # Integration tests (require database)
    ├── adapters.test.js   # Adapter-specific tests
    ├── aggregations.test.ts # Aggregation tests (count, etc.)
    ├── batch.test.ts      # Batch operation tests
    ├── errors.test.ts     # Error handling tests
    ├── features.test.ts   # Feature tests
    ├── filters.test.ts    # Filter operation tests
    ├── metadata.test.ts   # Metadata update tests
    ├── pooling.test.ts    # Connection pooling tests
    ├── search.test.ts     # Search functionality tests
    └── streaming.test.ts  # Streaming operation tests
```

## Running Tests

```bash
# Run all tests
npm test

# Run only unit tests
npm run test:unit

# Run only integration tests
npm run test:integration

# Run in watch mode
npm run test:watch
```

## Test Framework

Tests use [Vitest](https://vitest.dev/) for running. Configuration is in `vitest.config.ts`.
