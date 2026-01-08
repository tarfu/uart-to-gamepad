# UART-to-Gamepad build recipes

target := "thumbv6m-none-eabi"
package := "uart-to-gamepad-rp2040"

# List available recipes
default:
    @just --list

# Build firmware with specified protocol and profile
# proto: gamepad, crsf, mavlink
# profile: dev, release (default), production
build proto profile="release":
    #!/usr/bin/env bash
    set -euo pipefail

    # Determine features based on protocol
    case "{{proto}}" in
        gamepad)
            features="dev-panic,standard-hid,proto-gamepad"
            ;;
        crsf)
            features="dev-panic,standard-hid,proto-crsf"
            ;;
        mavlink)
            features="dev-panic,standard-hid,proto-mavlink"
            ;;
        *)
            echo "Unknown protocol: {{proto}}"
            echo "Valid options: gamepad, crsf, mavlink"
            exit 1
            ;;
    esac

    # Determine cargo profile args
    case "{{profile}}" in
        dev)
            profile_args=""
            echo "Building {{proto}} (dev)..."
            ;;
        release)
            profile_args="--release"
            echo "Building {{proto}} (release)..."
            ;;
        production)
            features="prod-panic,standard-hid,proto-{{proto}}"
            profile_args="--profile production"
            echo "Building {{proto}} (production)..."
            ;;
        *)
            echo "Unknown profile: {{profile}}"
            echo "Valid options: dev, release, production"
            exit 1
            ;;
    esac

    cargo build -p {{package}} --target {{target}} $profile_args --no-default-features --features "$features"
    echo "Build complete!"

# Build all protocol variants
build-all profile="release":
    just build gamepad {{profile}}
    just build crsf {{profile}}
    just build mavlink {{profile}}

# Run all host tests (auto-detects host target)
test:
    #!/usr/bin/env bash
    set -euo pipefail
    host_target=$(rustc -vV | grep host | cut -d' ' -f2)
    cargo test -p gamepad-proto -p gamepad-core -p crsf-proto -p mavlink-proto --target "$host_target"

# Check all variants compile
check:
    cargo check -p {{package}} --target {{target}}
    cargo check -p {{package}} --target {{target}} --no-default-features --features "dev-panic,standard-hid,proto-crsf"
    cargo check -p {{package}} --target {{target}} --no-default-features --features "dev-panic,standard-hid,proto-mavlink"

# Run clippy lints
clippy:
    cargo clippy -p {{package}} --target {{target}}
    cargo clippy -p gamepad-proto -p gamepad-core -p crsf-proto -p mavlink-proto

# Show binary size for specified protocol
size proto="gamepad":
    #!/usr/bin/env bash
    set -euo pipefail

    case "{{proto}}" in
        gamepad)
            features="dev-panic,standard-hid,proto-gamepad"
            ;;
        crsf)
            features="dev-panic,standard-hid,proto-crsf"
            ;;
        mavlink)
            features="dev-panic,standard-hid,proto-mavlink"
            ;;
        *)
            echo "Unknown protocol: {{proto}}"
            exit 1
            ;;
    esac

    cargo size -p {{package}} --target {{target}} --release --no-default-features --features "$features" -- -A

# Run firmware via probe-rs
run proto="gamepad" profile="release":
    #!/usr/bin/env bash
    set -euo pipefail

    case "{{profile}}" in
        dev)
            binary="target/{{target}}/debug/{{package}}"
            ;;
        release)
            binary="target/{{target}}/release/{{package}}"
            ;;
        production)
            binary="target/{{target}}/production/{{package}}"
            ;;
        *)
            echo "Unknown profile: {{profile}}"
            exit 1
            ;;
    esac

    echo "Running {{proto}} ({{profile}})..."
    probe-rs run --chip RP2040 "$binary"

# Generate UF2 file
uf2 proto="gamepad" profile="release":
    #!/usr/bin/env bash
    set -euo pipefail

    case "{{profile}}" in
        dev)
            binary="target/{{target}}/debug/{{package}}"
            ;;
        release)
            binary="target/{{target}}/release/{{package}}"
            ;;
        production)
            binary="target/{{target}}/production/{{package}}"
            ;;
        *)
            echo "Unknown profile: {{profile}}"
            exit 1
            ;;
    esac

    output="{{package}}-{{proto}}.uf2"
    echo "Generating $output..."
    elf2uf2-rs "$binary" "$output"
    echo "Created $output"

# Clean build artifacts
clean:
    cargo clean

# Generate documentation
docs:
    cargo doc --workspace --no-deps --target {{target}}
