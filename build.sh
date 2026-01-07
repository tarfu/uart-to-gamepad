#!/bin/bash

# Build script for silencer-dryer embedded project
# This script provides convenient commands for building with different panic handlers

set -e

# Default target for rp2040
TARGET="thumbv6m-none-eabi"

case "$1" in
    dev)
        echo "Building development version with panic-probe (for debugging)..."
        cargo build --target $TARGET
        echo "Development build complete!"
        ;;

    release)
        echo "Building release version with panic-probe (optimized but with debugging)..."
        cargo build --target $TARGET --release
        echo "Release build complete!"
        ;;

    production)
        echo "Building production version with panic-reset (optimized for size, no debugging)..."
        cargo build --target $TARGET --profile production --no-default-features --features prod-panic
        echo "Production build complete!"
        ;;

    check-dev)
        echo "Checking development configuration..."
        cargo check --target $TARGET
        ;;

    check-production)
        echo "Checking production configuration..."
        cargo check --target $TARGET --no-default-features --features prod-panic
        ;;

    size)
        echo "Analyzing binary sizes..."
        echo ""
        echo "Development build:"
        cargo size --target $TARGET -- -A 2>/dev/null || echo "Dev build not found. Run './build.sh dev' first."
        echo ""
        echo "Release build:"
        cargo size --target $TARGET --release -- -A 2>/dev/null || echo "Release build not found. Run './build.sh release' first."
        echo ""
        echo "Production build:"
        cargo size --target $TARGET --profile production --no-default-features --features prod-panic -- -A 2>/dev/null || echo "Production build not found. Run './build.sh production' first."
        ;;

    clean)
        echo "Cleaning build artifacts..."
        cargo clean
        echo "Clean complete!"
        ;;

    *)
        echo "Usage: $0 {dev|release|production|check-dev|check-production|size|clean}"
        echo ""
        echo "Build modes:"
        echo "  dev         - Development build with panic-probe for debugging"
        echo "  release     - Release build with optimizations but keeping panic-probe"
        echo "  production  - Production build with panic-reset, maximum optimization"
        echo ""
        echo "Other commands:"
        echo "  check-dev        - Check development configuration without building"
        echo "  check-production - Check production configuration without building"
        echo "  size            - Show binary size comparison between builds"
        echo "  clean           - Remove all build artifacts"
        echo ""
        echo "Panic handlers:"
        echo "  panic-probe - Halts on panic, prints message via defmt (dev/release)"
        echo "  panic-reset - Silently resets the device on panic (production)"
        exit 1
        ;;
esac
