#!/bin/bash

# Initialize test environment for Vault

set -e

echo "🔧 Initializing Vault test environment..."

# Build vault CLI
echo "📦 Building vault CLI..."
cd ..
cargo build --release
cd test.code

# Copy binary to test directories
cp ../target/release/vault dev-1/
cp ../target/release/vault dev-2/

# Initialize dev-1
echo "🏠 Setting up dev-1..."
cd dev-1
export VAULT_CONFIG_DIR="./config"
mkdir -p config data
./vault init --tenant acme-corp --admin alice@acme.com
echo "✅ dev-1 initialized"

# Initialize dev-2  
echo "🏠 Setting up dev-2..."
cd ../dev-2
export VAULT_CONFIG_DIR="./config"
mkdir -p config data
./vault init --tenant acme-corp --admin alice@acme.com
echo "✅ dev-2 initialized"

cd ..
echo "🎉 Test environment ready!"
echo ""
echo "Next steps:"
echo "  1. Run './scripts/simulate-sync.sh' to test synchronization"
echo "  2. Run './scripts/run-tests.sh' to execute integration tests"