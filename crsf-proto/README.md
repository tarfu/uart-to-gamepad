# crsf-proto

CRSF (Crossfire) protocol parsing and gamepad mapping. Chip-agnostic implementation for ExpressLRS and TBS Crossfire receivers.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | No | Enable standard library (for host testing) |
| `defmt` | No | Enable defmt formatting (for embedded logging) |

## CRSF Protocol

CRSF is a bidirectional serial protocol used by ExpressLRS and TBS Crossfire RC systems.

### UART Configuration

| Parameter | ExpressLRS | TBS Crossfire |
|-----------|------------|---------------|
| Baud Rate | 420000 | 416666 |
| Data Bits | 8 | 8 |
| Parity | None | None |
| Stop Bits | 1 | 1 |

### Channel Values

| Value | Meaning |
|-------|---------|
| 172 | Minimum (CRSF_MIN) |
| 992 | Center (CRSF_CENTER) |
| 1811 | Maximum (CRSF_MAX) |

### Default Channel Mapping

| Channel | Gamepad Function |
|---------|------------------|
| 1 | Right Stick X (Roll) |
| 2 | Right Stick Y (Pitch) |
| 3 | Left Stick Y (Throttle) |
| 4 | Left Stick X (Yaw) |
| 5 | Button 1 (3-pos switch) |
| 6 | Button 2 (3-pos switch) |
| 7 | Button 3 (3-pos switch) |
| 8 | Button 4 (3-pos switch) |
| 9 | Left Trigger |
| 10 | Right Trigger |

3-position switches map to buttons as:
- Low position: No button
- Mid position: Button pressed
- High position: Button pressed

## Usage

### Parsing RC Channels

```rust
use crsf_proto::{channels_to_gamepad, CrsfParser, Packet, DEFAULT_MAPPING};

let mut parser = CrsfParser::new();

// Feed bytes from UART
for byte in uart_bytes {
    if let Some(packet) = parser.push(byte) {
        if let Packet::RcChannels(channels) = packet {
            let state = channels_to_gamepad(&channels.0, &DEFAULT_MAPPING);
            // Use state.buttons, state.left_stick, etc.
        }
    }
}
```

### Custom Channel Mapping

```rust
use crsf_proto::{channels_to_gamepad, ChannelMapping};

let mapping = ChannelMapping {
    left_stick_x: 3,   // Channel 4 (0-indexed)
    left_stick_y: 2,   // Channel 3
    right_stick_x: 0,  // Channel 1
    right_stick_y: 1,  // Channel 2
    left_trigger: 8,   // Channel 9
    right_trigger: 9,  // Channel 10
    button_channels: [4, 5, 6, 7], // Channels 5-8
};

let state = channels_to_gamepad(&channels, &mapping);
```

### Telemetry Encoding

```rust
use crsf_proto::encode_telemetry;
use gamepad_core::TelemetryData;

let telemetry = TelemetryData {
    battery_voltage_mv: Some(3700),
    battery_percentage: Some(75),
    rssi_dbm: Some(-80),
    ..Default::default()
};

let mut buf = [0u8; 64];
if let Some(len) = encode_telemetry(&telemetry, &mut buf) {
    // Send buf[..len] back over UART
}
```

## Conversion Functions

| Function | Description |
|----------|-------------|
| `channels_to_gamepad` | Convert 16 CRSF channels to GamepadState |
| `crsf_to_stick` | Convert channel value to stick axis (-32768..32767) |
| `crsf_to_trigger` | Convert channel value to trigger (0..255) |
| `crsf_to_button` | Convert channel value to button state |
| `encode_telemetry` | Encode telemetry data to CRSF frame |

## License

MIT
