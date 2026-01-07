# UART-to-Gamepad

A USB HID gamepad bridge for the Raspberry Pi Pico (RP2040) that receives gamepad state over UART and presents it as a standard USB gamepad to the host computer.

## Features

- **USB HID Gamepad**: Appears as a standard gamepad to any OS (Windows, macOS, Linux)
- **UART Input**: Receives gamepad state via serial protocol at 115200 baud
- **16 Buttons**: Full button support with bitfield encoding
- **Dual Analog Sticks**: Left and right sticks with 16-bit precision (scaled to 8-bit for HID)
- **Analog Triggers**: Left and right triggers with 8-bit precision
- **Incremental Updates**: Efficient protocol supports both full state and delta updates
- **XOR Checksum**: Error detection on all messages

## Hardware Requirements

- Raspberry Pi Pico (RP2040) or compatible board
- USB connection to host computer
- UART connection to gamepad state source (e.g., another microcontroller, computer)

### Pinout

| Function | GPIO Pin | Description |
|----------|----------|-------------|
| UART1 TX | GPIO 8   | Transmit (directly directly to source) |
| UART1 RX | GPIO 9   | Receive gamepad data |
| LED      | GPIO 25  | Error indicator (on-board LED) |

Optional (with `uart-flow-control` feature):
| Function | GPIO Pin | Description |
|----------|----------|-------------|
| UART1 CTS | GPIO 10 | Clear to Send |
| UART1 RTS | GPIO 11 | Request to Send |

## Building

### Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the thumbv6m target for RP2040
rustup target add thumbv6m-none-eabi

# Install elf2uf2-rs for creating UF2 files (optional)
cargo install elf2uf2-rs

# Install probe-rs for debugging (optional)
cargo install probe-rs-tools
```

### Build Commands

```bash
# Development build
cargo build -p uart-to-gamepad

# Release build (optimized for size)
cargo build -p uart-to-gamepad --release

# Production build (maximum optimization, no debug info)
cargo build -p uart-to-gamepad --profile production
```

### Flashing

**Using UF2 (no debugger required):**

1. Hold BOOTSEL button on the Pico while connecting USB
2. Copy the UF2 file to the mounted drive:

```bash
# Convert ELF to UF2
elf2uf2-rs target/thumbv6m-none-eabi/release/uart-to-gamepad uart-to-gamepad.uf2

# Copy to Pico (path may vary)
cp uart-to-gamepad.uf2 /Volumes/RPI-RP2/
```

**Using probe-rs (with debug probe):**

```bash
probe-rs run --chip RP2040 target/thumbv6m-none-eabi/release/uart-to-gamepad
```

## Testing

The project is structured as a Cargo workspace to enable fast host-based testing:

```bash
# Run all unit tests on host (~0.5s)
cargo test -p gamepad-core

# Run with verbose output
cargo test -p gamepad-core -- --nocapture

# Run specific test
cargo test -p gamepad-core parser::tests::test_parse_neutral
```

## Protocol Specification

The UART protocol uses ASCII-based messages terminated with newline (`\n`). All messages include an XOR checksum for error detection.

### Full State Message

Reports complete gamepad state. Use when initializing or when multiple fields change.

```
G<buttons>:<lx>:<ly>:<rx>:<ry>:<lt>:<rt>*<checksum>\n
```

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `buttons` | u16 (hex) | 0000-FFFF | Button bitfield |
| `lx` | i16 | -32768 to 32767 | Left stick X |
| `ly` | i16 | -32768 to 32767 | Left stick Y |
| `rx` | i16 | -32768 to 32767 | Right stick X |
| `ry` | i16 | -32768 to 32767 | Right stick Y |
| `lt` | u8 | 0-255 | Left trigger |
| `rt` | u8 | 0-255 | Right trigger |
| `checksum` | u8 (hex) | 00-FF | XOR of all bytes before `*` |

**Example:**
```
G0001:0:0:0:0:0:0*47\n
```
(Button 1 pressed, all else neutral)

### Incremental Update Message

Reports a single field change. More efficient for real-time updates.

```
U<field>:<value>*<checksum>\n
```

| Field Code | Value Type | Description |
|------------|------------|-------------|
| `B` | u16 (hex) | Buttons bitfield |
| `LX` | i16 | Left stick X |
| `LY` | i16 | Left stick Y |
| `RX` | i16 | Right stick X |
| `RY` | i16 | Right stick Y |
| `LT` | u8 | Left trigger |
| `RT` | u8 | Right trigger |

**Examples:**
```
UB:0003*checksum\n     # Buttons 1 and 2 pressed
ULX:16000*checksum\n   # Left stick X = 16000
ULT:255*checksum\n     # Left trigger fully pressed
```

### Button Mapping

| Bit | Button | Common Mapping |
|-----|--------|----------------|
| 0   | Button 1 | A / Cross |
| 1   | Button 2 | B / Circle |
| 2   | Button 3 | X / Square |
| 3   | Button 4 | Y / Triangle |
| 4   | Button 5 | Left Bumper |
| 5   | Button 6 | Right Bumper |
| 6   | Button 7 | Back / Select |
| 7   | Button 8 | Start |
| 8   | Button 9 | Left Stick Click |
| 9   | Button 10 | Right Stick Click |
| 10  | Button 11 | Guide / Home |
| 11  | Button 12 | D-Pad Up |
| 12  | Button 13 | D-Pad Down |
| 13  | Button 14 | D-Pad Left |
| 14  | Button 15 | D-Pad Right |
| 15  | Button 16 | (Reserved) |

### Checksum Calculation

The checksum is the XOR of all ASCII bytes before the `*` character:

```rust
fn calculate_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc ^ b)
}
```

**Example:**
```
Message: "G0000:0:0:0:0:0:0"
Checksum: 'G' ^ '0' ^ '0' ^ '0' ^ '0' ^ ':' ^ '0' ^ ':' ^ '0' ^ ':' ^ '0' ^ ':' ^ '0' ^ ':' ^ '0' ^ ':' ^ '0'
        = 0x47 ^ 0x30 ^ 0x30 ^ 0x30 ^ 0x30 ^ 0x3A ^ 0x30 ^ 0x3A ^ 0x30 ^ 0x3A ^ 0x30 ^ 0x3A ^ 0x30 ^ 0x3A ^ 0x30
        = 0x00
Full message: "G0000:0:0:0:0:0:0*00\n"
```

## Project Structure

```
uart-to-gamepad/
├── Cargo.toml              # Workspace configuration
├── LICENSE                 # MIT License
├── README.md               # This file
│
├── gamepad-core/           # Platform-agnostic library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Crate root, re-exports
│       ├── types.rs        # Buttons, AnalogStick, GamepadState
│       ├── parser.rs       # Protocol parser with tests
│       ├── input.rs        # InputSource trait
│       ├── output.rs       # OutputSink trait
│       └── bridge.rs       # GamepadBridge orchestrator
│
└── firmware/               # RP2040 embedded application
    ├── Cargo.toml
    ├── build.rs            # Linker script setup
    ├── memory.x            # RP2040 memory layout
    └── src/
        ├── lib.rs          # Re-exports for convenience
        ├── bin/main.rs     # Application entry point
        ├── uart_input.rs   # UART receiver implementation
        └── usb_output.rs   # USB HID implementation
```

## Features Flags

### firmware (uart-to-gamepad)

| Feature | Default | Description |
|---------|---------|-------------|
| `dev-panic` | Yes | Use `panic-probe` for debugging (prints panic info) |
| `prod-panic` | No | Use `panic-reset` for production (silent reset) |
| `standard-hid` | Yes | Standard HID gamepad descriptor (cross-platform) |
| `xinput-compat` | No | Xbox-style descriptor (better Windows game support) |
| `uart-flow-control` | No | Enable CTS/RTS flow control on GPIO 10/11 |

### gamepad-core

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | No | Enable standard library (for host testing) |
| `defmt` | No | Enable defmt formatting (for embedded logging) |

## Architecture

```
┌─────────────────┐     UART      ┌─────────────────┐     USB HID     ┌──────────────┐
│  Input Source   │──────────────▶│   RP2040 Pico   │────────────────▶│  Host PC     │
│  (MCU, PC, etc) │   115200 8N1  │  uart-to-gamepad│   Gamepad HID   │  (Games, OS) │
└─────────────────┘               └─────────────────┘                 └──────────────┘
```

**Internal Flow:**
```
UART RX ──▶ UartInputSource ──▶ Signal ──▶ UsbHidOutput ──▶ USB HID
              (parse protocol)   (latest    (format report)
                                  value)
```

The firmware uses Embassy async runtime with three concurrent tasks:
1. **USB Task**: Runs the USB stack
2. **Input Task**: Reads UART, parses messages, signals latest state
3. **Output Task**: Waits for state signals, sends USB HID reports

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

When contributing code:
1. Run `cargo test -p gamepad-core` to ensure tests pass
2. Run `cargo clippy -p uart-to-gamepad` for lint checks
3. Run `cargo build -p uart-to-gamepad --release` to verify embedded build
