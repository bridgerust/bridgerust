"""
Integration test configuration for pytest.
"""
import pytest


def pytest_addoption(parser):
    parser.addoption(
        "--integration",
        action="store_true",
        default=False,
        help="Run integration tests (requires Docker databases)"
    )


def pytest_configure(config):
    config.addinivalue_line("markers", "integration: mark test as integration test")


def pytest_collection_modifyitems(config, items):
    if config.getoption("--integration"):
        return
    
    skip_integration = pytest.mark.skip(reason="Use --integration to run")
    for item in items:
        if "integration" in item.keywords:
            item.add_marker(skip_integration)
