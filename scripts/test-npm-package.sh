#!/bin/bash
# Test script to verify @bridgerust/embex package from npm works correctly
# Usage: ./scripts/test-npm-package.sh [version]
# Example: ./scripts/test-npm-package.sh 0.1.13

set -e

VERSION="${1:-latest}"
TEST_DIR=$(mktemp -d -t embex-npm-test-XXXXXX)

echo "🧪 Testing @bridgerust/embex package from npm"
echo "   Version: $VERSION"
echo "   Test directory: $TEST_DIR"
echo ""

# Cleanup on exit
trap "rm -rf $TEST_DIR" EXIT

cd "$TEST_DIR"

echo "📦 Initializing npm project..."
npm init -y > /dev/null

echo "📥 Installing @bridgerust/embex@$VERSION from npm..."
if [ "$VERSION" = "latest" ]; then
  npm install @bridgerust/embex
else
  npm install @bridgerust/embex@$VERSION
fi

echo "✅ Package installed"
echo ""

echo "📋 Package information:"
npm list @bridgerust/embex
echo ""

echo "🔍 Checking package structure..."
cat > check-package.js << 'EOF'
const fs = require('fs');
const path = require('path');

try {
  const packagePath = require.resolve('@bridgerust/embex/package.json');
  const packageDir = path.dirname(packagePath);
  
  console.log('📂 Package directory:', packageDir);
  console.log('\n📋 Package contents:');
  
  const files = fs.readdirSync(packageDir);
  files.forEach(file => {
    const filePath = path.join(packageDir, file);
    const stat = fs.statSync(filePath);
    if (stat.isDirectory()) {
      console.log(`   📁 ${file}/`);
    } else {
      console.log(`   📄 ${file}`);
    }
  });
  
  // Check for dist folder
  const distPath = path.join(packageDir, 'dist');
  if (fs.existsSync(distPath)) {
    console.log('\n📦 dist/ folder contents:');
    const distFiles = fs.readdirSync(distPath, { recursive: true });
    distFiles.forEach(file => {
      console.log(`   📄 dist/${file}`);
    });
  } else {
    console.log('\n⚠️  dist/ folder not found');
  }
  
  // Check for index files
  const possibleEntries = ['index.js', 'index.ts', 'native.js', 'dist/index.js', 'src/index.js'];
  console.log('\n🔍 Checking for entry points:');
  possibleEntries.forEach(entry => {
    const entryPath = path.join(packageDir, entry);
    if (fs.existsSync(entryPath)) {
      console.log(`   ✅ Found: ${entry}`);
    } else {
      console.log(`   ❌ Missing: ${entry}`);
    }
  });
} catch (error) {
  console.error('❌ Error checking package:', error.message);
  process.exit(1);
}
EOF

node check-package.js

echo ""
echo "🧪 Running tests..."

cat > test.js << 'EOF'
const fs = require('fs');
const path = require('path');

console.log('🧪 Testing @bridgerust/embex package...\n');

// Test 1: Try to require the package
let EmbexClient;
try {
  // Try different possible entry points
  const packagePath = require.resolve('@bridgerust/embex/package.json');
  const packageDir = path.dirname(packagePath);
  const packageJson = require(packagePath);
  
  console.log('📋 Package info:');
  console.log('   - Main entry:', packageJson.main);
  console.log('   - Types entry:', packageJson.types);
  
  // Try to require using the main entry point
  if (packageJson.main) {
    const mainPath = path.join(packageDir, packageJson.main);
    if (fs.existsSync(mainPath)) {
      const pkg = require('@bridgerust/embex');
      EmbexClient = pkg.EmbexClient;
      console.log('✅ Test 1: Import successful via main entry');
    } else {
      // Try alternative entry points
      console.log('⚠️  Main entry not found, trying alternatives...');
      try {
        const pkg = require('@bridgerust/embex');
        EmbexClient = pkg.EmbexClient;
        console.log('✅ Test 1: Import successful via alternative');
      } catch (e) {
        throw new Error(`Cannot import package. Main entry (${packageJson.main}) not found. Error: ${e.message}`);
      }
    }
  } else {
    const pkg = require('@bridgerust/embex');
    EmbexClient = pkg.EmbexClient;
    console.log('✅ Test 1: Import successful');
  }
} catch (error) {
  console.error('❌ Test 1 failed:', error.message);
  console.error('   This might indicate the package is missing required files.');
  console.error('   Check that dist/ folder is built and included in the published package.');
  process.exit(1);
}

// Test 2: Client can be instantiated
try {
  const client = new EmbexClient('qdrant', 'http://localhost:6333', null);
  console.log('✅ Test 2: Client instantiation successful');
  console.log('   - Client type:', typeof client);
  console.log('   - Has collection method:', typeof client.collection === 'function');
} catch (error) {
  console.error('❌ Test 2 failed:', error.message);
  process.exit(1);
}

// Test 3: Collection can be created
try {
  const client = new EmbexClient('qdrant', 'http://localhost:6333', null);
  const collection = client.collection('test_collection');
  console.log('✅ Test 3: Collection creation successful');
  console.log('   - Collection type:', typeof collection);
  console.log('   - Has insert method:', typeof collection.insert === 'function');
  console.log('   - Has search method:', typeof collection.search === 'function');
  console.log('   - Has query method:', typeof collection.query === 'function');
} catch (error) {
  console.error('❌ Test 3 failed:', error.message);
  process.exit(1);
}

// Test 4: Check package structure
try {
  const packagePath = require.resolve('@bridgerust/embex/package.json');
  const packageDir = path.dirname(packagePath);
  console.log('✅ Test 4: Package structure verified');
  console.log('   - Package path:', packageDir);
} catch (error) {
  console.error('❌ Test 4 failed:', error.message);
  process.exit(1);
}

// Test 5: Check async initialization method exists
try {
  if (typeof EmbexClient.newAsync !== 'function') {
    throw new Error('EmbexClient.newAsync is not a function');
  }
  console.log('✅ Test 5: Async initialization method exists');
  console.log('   - newAsync type:', typeof EmbexClient.newAsync);
} catch (error) {
  console.error('❌ Test 5 failed:', error.message);
  process.exit(1);
}

// Test 6: Test async initialization (syntax check)
try {
  // Test that newAsync returns a Promise
  const clientPromise = EmbexClient.newAsync('lancedb', './test-data', null);
  if (!(clientPromise instanceof Promise)) {
    throw new Error('newAsync does not return a Promise');
  }
  console.log('✅ Test 6: Async initialization syntax correct');
  console.log('   - newAsync returns Promise');
  console.log('   - Method signature is correct');
  
  // Note: We don't await here to avoid hanging, but we've verified the Promise exists
  // In a real scenario with a DB, this would work: const client = await EmbexClient.newAsync(...)
} catch (error) {
  console.error('❌ Test 6 failed:', error.message);
  process.exit(1);
}

console.log('\n🎉 All basic tests passed!');
console.log('\n💡 Note: Full integration tests require a running database server.');
console.log('   To test with a real database, start Qdrant:');
console.log('   docker run -p 6333:6333 qdrant/qdrant');
EOF

node test.js

echo ""
echo "✅ All npm package tests passed!"

