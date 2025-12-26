#!/bin/bash
# Kabod Integration Test Runner
# This script spins up databases and runs integration tests

set -e

echo "🚀 Kabod Integration Test Runner"
echo "================================"

if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

echo ""
echo "📦 Starting databases..."
docker-compose up -d

echo ""
echo "⏳ Waiting for databases to be ready (30s)..."
sleep 30

echo ""
echo "🔍 Checking database connectivity..."

check_service() {
    local name=$1
    local url=$2
    if curl -s "$url" > /dev/null 2>&1; then
        echo "  ✅ $name is ready"
        return 0
    else
        echo "  ❌ $name is not responding"
        return 1
    fi
}

check_service "Qdrant" "http://localhost:6333/collections"
check_service "Chroma" "http://localhost:8000/api/v1/heartbeat"
check_service "Weaviate" "http://localhost:8080/v1/.well-known/ready"
check_service "Milvus" "http://localhost:9091/healthz"
    
if pg_isready -h localhost -p 5432 -U kabod > /dev/null 2>&1; then
    echo "  ✅ pgvector is ready"
else
    echo "  ⚠️  pgvector might not be ready (pg_isready not available or failed)"
fi

echo ""
echo "🧪 Running Python integration tests..."
cd bindings/python/kabod
if command -v uv > /dev/null 2>&1; then
    uv pip install -e . pytest pytest-asyncio
    uv run pytest tests/integration/ -v --integration
else
    pip install -e . pytest pytest-asyncio
    pytest tests/integration/ -v --integration
fi

echo ""
echo "🧪 Running Node.js integration tests..."
cd ../../../bindings/node/@bridgerust/kabod
if command -v bun > /dev/null 2>&1; then
    bun run build
    bun test tests/integration/
else
    npm run build
    npm test -- tests/integration/
fi

echo ""
echo "✅ All integration tests completed!"
echo ""
echo "💡 To stop databases: docker-compose down"
echo "💡 To stop and remove data: docker-compose down -v"
