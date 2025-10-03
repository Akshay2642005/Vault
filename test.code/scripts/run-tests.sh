#!/bin/bash

# Run integration tests

set -e

echo "🧪 Running Vault integration tests..."

# Build and test
cd ..
cargo test

echo "🔍 Running CLI integration tests..."
cd test.code

# Test basic operations
echo "Testing basic vault operations..."

cd dev-1
export VAULT_CONFIG_DIR="./config"

# Test secret storage and retrieval
echo "Testing secret put/get..."
./vault put test-secret --namespace test --value "test-value-123"
RETRIEVED=$(./vault get test-secret --namespace test)

if [ "$RETRIEVED" = "test-value-123" ]; then
    echo "✅ Secret storage/retrieval test passed"
else
    echo "❌ Secret storage/retrieval test failed"
    exit 1
fi

# Test listing
echo "Testing secret listing..."
./vault list --namespace test | grep -q "test-secret"
if [ $? -eq 0 ]; then
    echo "✅ Secret listing test passed"
else
    echo "❌ Secret listing test failed"
    exit 1
fi

cd ..
echo "🎉 All tests passed!"