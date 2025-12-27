# Python Tests

## Structure

```
tests/
├── unit/              # Unit tests (no database required)
│   ├── test_basic.py      # Basic functionality tests
│   ├── test_errors.py     # Error handling tests
│   └── test_pydantic.py   # Pydantic integration tests
│
└── integration/       # Integration tests (require database)
    ├── test_adapters.py   # Adapter-specific tests
    ├── test_features.py   # Feature tests (batch, etc.)
    └── test_streaming.py  # Streaming operation tests
```

## Running Tests

```bash
# Run all tests
pytest

# Run only unit tests
pytest tests/unit/

# Run only integration tests
pytest tests/integration/

# Run with markers
pytest -m unit
pytest -m integration
```

## Test Markers

- `@pytest.mark.unit` - Unit tests
- `@pytest.mark.integration` - Integration tests requiring database
- `@pytest.mark.slow` - Slow running tests
