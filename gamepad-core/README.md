# gamepad-core

Core types and traits for the gamepad bridge. Provides platform-agnostic abstractions for input sources and output sinks.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | No | Enable standard library (for host testing) |
| `defmt` | No | Enable defmt formatting (for embedded logging) |
| `heapless` | No | Enable heapless Vec serialization (passes to gamepad-proto) |
| `embedded-io` | No | Enable embedded-io Write serialization (passes to gamepad-proto) |

## Core Types

### GamepadState

Complete gamepad state including buttons, analog sticks, and triggers. Re-exported from `gamepad-proto`.

### Traits

#### InputSource

Async trait for receiving gamepad state from any source:

```rust
pub trait InputSource {
    async fn receive(&mut self) -> Result<GamepadState, InputError>;
}
```

#### OutputSink

Async trait for sending gamepad state to any destination:

```rust
pub trait OutputSink {
    async fn send(&mut self, state: &GamepadState) -> Result<(), OutputError>;
    async fn wait_ready(&mut self);
}
```

### GamepadBridge

Orchestrates data flow between an input source and output sink:

```rust
use gamepad_core::{GamepadBridge, InputSource, OutputSink};

let bridge = GamepadBridge::new(input, output);
bridge.run().await; // Runs forever, forwarding state
```

### Telemetry

Battery and signal telemetry types for bidirectional communication:

```rust
use gamepad_core::{BatteryInfo, SignalInfo, TelemetryData};

let battery = BatteryInfo {
    voltage_mv: 3700,
    percentage: 75,
    charging: false,
};
```

## Usage

```rust
use gamepad_core::{GamepadState, InputSource, OutputSink, InputError, OutputError};

struct MyInput;

impl InputSource for MyInput {
    async fn receive(&mut self) -> Result<GamepadState, InputError> {
        // Read from your hardware
        Ok(GamepadState::neutral())
    }
}

struct MyOutput;

impl OutputSink for MyOutput {
    async fn send(&mut self, state: &GamepadState) -> Result<(), OutputError> {
        // Send to your hardware
        Ok(())
    }

    async fn wait_ready(&mut self) {
        // Wait for output to be ready
    }
}
```

## License

MIT
