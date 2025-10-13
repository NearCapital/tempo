#!/bin/bash

# Script to easily switch between different reth versions for testing

set -e

CURRENT_COMMIT="c02a68dc78eb4e080288ed6779439fd1d3169667"
TEST_COMMIT="1619408"

print_usage() {
    echo "Usage: ./switch_reth.sh [main|commit|current]"
    echo ""
    echo "Options:"
    echo "  main     - Switch to reth main branch"
    echo "  commit   - Switch to reth commit 1619408"
    echo "  current  - Switch back to current commit $CURRENT_COMMIT"
    echo ""
    echo "After switching, run: cargo update -p reth && cargo build"
}

if [ $# -eq 0 ]; then
    print_usage
    exit 1
fi

case "$1" in
    main)
        echo "Switching to reth main branch..."
        # Replace rev with branch = "main"
        sed -i '' 's/git = "https:\/\/github.com\/paradigmxyz\/reth", rev = "[^"]*"/git = "https:\/\/github.com\/paradigmxyz\/reth", branch = "main"/g' Cargo.toml
        echo "✓ Switched to main branch"
        ;;
    commit)
        echo "Switching to reth commit $TEST_COMMIT..."
        # Replace with the test commit
        sed -i '' 's/git = "https:\/\/github.com\/paradigmxyz\/reth", branch = "main"/git = "https:\/\/github.com\/paradigmxyz\/reth", rev = "'$TEST_COMMIT'"/g' Cargo.toml
        sed -i '' 's/git = "https:\/\/github.com\/paradigmxyz\/reth", rev = "[^"]*"/git = "https:\/\/github.com\/paradigmxyz\/reth", rev = "'$TEST_COMMIT'"/g' Cargo.toml
        echo "✓ Switched to commit $TEST_COMMIT"
        ;;
    current)
        echo "Switching back to current commit $CURRENT_COMMIT..."
        # Replace with the current commit
        sed -i '' 's/git = "https:\/\/github.com\/paradigmxyz\/reth", branch = "main"/git = "https:\/\/github.com\/paradigmxyz\/reth", rev = "'$CURRENT_COMMIT'"/g' Cargo.toml
        sed -i '' 's/git = "https:\/\/github.com\/paradigmxyz\/reth", rev = "[^"]*"/git = "https:\/\/github.com\/paradigmxyz\/reth", rev = "'$CURRENT_COMMIT'"/g' Cargo.toml
        echo "✓ Switched back to current commit"
        ;;
    *)
        echo "Error: Unknown option '$1'"
        print_usage
        exit 1
        ;;
esac

echo ""
echo "Next steps:"
echo "  1. cargo update -p reth"
echo "  2. cargo build"
echo ""
echo "Or run both together:"
echo "  cargo update -p reth && cargo build"
