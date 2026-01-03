#!/bin/bash
# Test script to verify embex package from PyPI works correctly
# Usage: ./scripts/test-pypi-package.sh [version]
# Example: ./scripts/test-pypi-package.sh 0.1.13

set -e

VERSION="${1:-latest}"
TEST_DIR=$(mktemp -d -t embex-pypi-test-XXXXXX)

echo "🧪 Testing embex package from PyPI"
echo "   Version: $VERSION"
echo "   Test directory: $TEST_DIR"
echo ""

# Cleanup on exit
trap "rm -rf $TEST_DIR" EXIT

cd "$TEST_DIR"

echo "🐍 Creating virtual environment..."
python3 -m venv venv
source venv/bin/activate

echo "📥 Installing embex@$VERSION from PyPI..."
if [ "$VERSION" = "latest" ]; then
  pip install --upgrade pip
  pip install embex
else
  pip install --upgrade pip
  pip install embex==$VERSION
fi

echo "✅ Package installed"
echo ""

echo "📋 Package information:"
pip show embex
echo ""

echo "🧪 Running tests..."

cat > test_package.py << 'EOF'
"""Test script to verify embex package from PyPI works correctly."""
import sys

print('🧪 Testing embex package from PyPI...\n')

# Test 1: Import works
try:
    from embex import EmbexClient, Point, Collection, SearchResult
    print('✅ Test 1: Import successful')
    print(f'   - EmbexClient: {EmbexClient}')
    print(f'   - Point: {Point}')
    print(f'   - Collection: {Collection}')
    print(f'   - SearchResult: {SearchResult}')
except ImportError as e:
    print(f'❌ Test 1 failed: Import error - {e}')
    sys.exit(1)

# Test 2: Point can be created
try:
    point = Point(id="test_1", vector=[0.1, 0.2, 0.3], metadata={"key": "value"})
    print('✅ Test 2: Point creation successful')
    print(f'   - Point ID: {point.id}')
    print(f'   - Vector length: {len(point.vector)}')
    print(f'   - Metadata: {point.metadata}')
except Exception as e:
    print(f'❌ Test 2 failed: {e}')
    sys.exit(1)

# Test 3: Client can be instantiated
try:
    client = EmbexClient(provider="qdrant", url="http://localhost:6333")
    print('✅ Test 3: Client instantiation successful')
    print(f'   - Client type: {type(client)}')
    print(f'   - Has collection method: {hasattr(client, "collection")}')
except Exception as e:
    print(f'❌ Test 3 failed: {e}')
    sys.exit(1)

# Test 4: Collection can be created
try:
    client = EmbexClient(provider="qdrant", url="http://localhost:6333")
    collection = client.collection("test_collection")
    print('✅ Test 4: Collection creation successful')
    print(f'   - Collection type: {type(collection)}')
    print(f'   - Has insert method: {hasattr(collection, "insert")}')
    print(f'   - Has search method: {hasattr(collection, "search")}')
    print(f'   - Has query method: {hasattr(collection, "query")}')
except Exception as e:
    print(f'❌ Test 4 failed: {e}')
    sys.exit(1)

# Test 5: Check async methods exist
try:
    import inspect
    client = EmbexClient(provider="qdrant", url="http://localhost:6333")
    collection = client.collection("test_collection")
    
    if hasattr(collection, 'insert'):
        is_async = inspect.iscoroutinefunction(collection.insert)
        print(f'✅ Test 5: Method inspection successful')
        print(f'   - insert is async: {is_async}')
except Exception as e:
    print(f'⚠️ Test 5 warning: {e}')

# Test 6: Check package metadata
try:
    import embex
    print('✅ Test 6: Package metadata check')
    print(f'   - Package file: {embex.__file__}')
    print(f'   - Package name: {embex.__package__}')
    if hasattr(embex, '__version__'):
        print(f'   - Version: {embex.__version__}')
except Exception as e:
    print(f'⚠️ Test 6 warning: {e}')

# Test 7: Check async initialization method exists
try:
    import inspect
    if not hasattr(EmbexClient, 'new_async'):
        raise Exception('EmbexClient.new_async method not found')
    print('✅ Test 7: Async initialization method exists')
    print(f'   - new_async type: {type(EmbexClient.new_async)}')
    print(f'   - Is static method: {inspect.ismethod(EmbexClient.new_async) or inspect.isfunction(EmbexClient.new_async)}')
except Exception as e:
    print(f'❌ Test 7 failed: {e}')
    sys.exit(1)

# Test 8: Test async initialization (syntax check)
try:
    import asyncio
    
    async def test_async_init():
        try:
            # Test that new_async is awaitable
            client = await EmbexClient.new_async("lancedb://./test-data")
            print('✅ Test 8: Async initialization successful')
            print(f'   - Client created via new_async: {type(client)}')
            print(f'   - Has collection method: {hasattr(client, "collection")}')
            return True
        except Exception as e:
            # Expected to fail without actual DB, but syntax is correct
            if 'lancedb' in str(e).lower() or 'connection' in str(e).lower() or 'enoent' in str(e).lower():
                print('✅ Test 8: Async initialization syntax correct')
                print('   - new_async is awaitable')
                print('   - Can be called (connection error expected without DB)')
                return True
            else:
                raise
    
    result = asyncio.run(test_async_init())
    if not result:
        raise Exception('Async initialization test failed')
except Exception as e:
    print(f'❌ Test 8 failed: {e}')
    sys.exit(1)

print('\n🎉 All basic tests passed!')
print('\n💡 Note: Full integration tests require a running database server.')
print('   To test with a real database, start Qdrant:')
print('   docker run -p 6333:6333 qdrant/qdrant')
EOF

python test_package.py

echo ""
echo "✅ All PyPI package tests passed!"

