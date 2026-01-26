#!/bin/bash

# Test script for consumer frontend feature flags
# Tests compilation of different feature combinations

set -e

CONSUMER_DIR="frontend/consumer-dioxus"
cd "$(dirname "$0")/$CONSUMER_DIR"

echo "=================================================="
echo "Consumer Frontend Feature Flag Compilation Tests"
echo "=================================================="
echo ""

# Test 1: Basic mode (no auth)
echo "📦 Test 1: Basic Mode (no auth)"
echo "Running: cargo check --features basic"
if cargo check --features basic 2>&1 | grep -q "Finished"; then
    echo "✅ Basic mode compiles successfully"
else
    echo "❌ Basic mode compilation failed"
    exit 1
fi
echo ""

# Test 2: Full mode (all features)
echo "📦 Test 2: Full Mode (all features)"
echo "Running: cargo check --features full"
if cargo check --features full 2>&1 | grep -q "Finished"; then
    echo "✅ Full mode compiles successfully"
else
    echo "❌ Full mode compilation failed"
    exit 1
fi
echo ""

# Test 3: Consumer-auth only
echo "📦 Test 3: Consumer-Auth Only"
echo "Running: cargo check --features consumer-auth"
if cargo check --features consumer-auth 2>&1 | grep -q "Finished"; then
    echo "✅ Consumer-auth mode compiles successfully"
else
    echo "❌ Consumer-auth compilation failed"
    exit 1
fi
echo ""

# Test 4: Profile management (should pull in consumer-auth)
echo "📦 Test 4: Profile Management (with consumer-auth dependency)"
echo "Running: cargo check --features profile-management"
if cargo check --features profile-management 2>&1 | grep -q "Finished"; then
    echo "✅ Profile management mode compiles successfully"
else
    echo "❌ Profile management compilation failed"
    exit 1
fi
echo ""

# Test 5: Comments only
echo "📦 Test 5: Comments Only"
echo "Running: cargo check --features comments"
if cargo check --features comments 2>&1 | grep -q "Finished"; then
    echo "✅ Comments mode compiles successfully"
else
    echo "❌ Comments compilation failed"
    exit 1
fi
echo ""

# Summary
echo "=================================================="
echo "✅ All feature combinations compile successfully!"
echo "=================================================="
echo ""
echo "Feature Flag Summary:"
echo "  • basic              - Pure public blog (no auth)"
echo "  • consumer-auth      - Login/register functionality"
echo "  • profile-management - User profiles (requires consumer-auth)"
echo "  • comments          - Comment functionality"
echo "  • full              - All features enabled"
echo ""
echo "Default build uses: basic mode"
echo ""
echo "To test runtime behavior:"
echo "  Basic:  dx serve --features basic"
echo "  Full:   dx serve --features full"
echo ""
