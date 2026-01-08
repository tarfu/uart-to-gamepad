# mavlink-proto

MAVLink protocol parsing and gamepad mapping. Chip-agnostic implementation supporting MAVLink v1 and v2.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | No | Enable standard library (for host testing) |
| `defmt` | No | Enable defmt formatting (for embedded logging) |

## MAVLink Protocol

MAVLink is a lightweight messaging protocol for communicating with drones and other robotic systems.

### UART Configuration

| Use Case | Baud Rate |
|----------|-----------|
| Telemetry Radio | 57600 |
| Direct Serial | 115200 |

Data format: 8N1 (8 data bits, no parity, 1 stop bit)

### Supported Messages

| Message | ID | Description |
|---------|-----|-------------|
| HEARTBEAT | 0 | Connection presence indicator |
| MANUAL_CONTROL | 69 | Joystick/gamepad control input |

### MANUAL_CONTROL Message

Primary message for gamepad/joystick input:

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| x | i16 | -1000..1000 | Pitch / Left Stick X |
| y | i16 | -1000..1000 | Roll / Left Stick Y |
| z | u16 | 0..1000 | Throttle (split to triggers) |
| r | i16 | -1000..1000 | Yaw / Right Stick X |
| buttons | u16 | Bitfield | Primary buttons (0-15) |
| buttons2 | u16 | Bitfield | Extended buttons (16-31) |
| target | u8 | System ID | Target system |

### Default Axis Mapping

| MAVLink Field | Gamepad Function |
|---------------|------------------|
| x | Left Stick X |
| y | Left Stick Y |
| z | Triggers (split at 500) |
| r | Right Stick X |
| buttons | Button bitfield |

The `z` axis (0-1000) is split into two triggers:
- z < 500: Left trigger (0-255)
- z > 500: Right trigger (0-255)

## Usage

### Parsing MAVLink Messages

```rust
use mavlink_proto::{MavlinkParser, MavMessage, manual_control_to_gamepad, DEFAULT_AXIS_MAPPING};

let mut parser = MavlinkParser::new();

// Feed bytes from UART
for byte in uart_bytes {
    if let Ok(Some(msg)) = parser.push_byte(byte) {
        match msg {
            MavMessage::ManualControl(mc) => {
                let state = manual_control_to_gamepad(
                    mc.x, mc.y, mc.z, mc.r,
                    mc.buttons, mc.buttons2,
                    &DEFAULT_AXIS_MAPPING,
                );
                // Use state.buttons, state.left_stick, etc.
            }
            MavMessage::Heartbeat => {
                // Connection alive
            }
        }
    }
}
```

### Custom Axis Mapping

```rust
use mavlink_proto::{manual_control_to_gamepad, AxisMapping};

let mapping = AxisMapping {
    x_axis: mavlink_proto::mapping::GamepadAxis::LeftX,
    y_axis: mavlink_proto::mapping::GamepadAxis::LeftY,
    z_axis: mavlink_proto::mapping::GamepadAxis::Triggers,
    r_axis: mavlink_proto::mapping::GamepadAxis::RightX,
    invert_x: false,
    invert_y: false,
    invert_r: false,
};

let state = manual_control_to_gamepad(x, y, z, r, buttons, buttons2, &mapping);
```

### Filtering by Target System

```rust
use mavlink_proto::MavlinkParser;

let mut parser = MavlinkParser::new();

// Only accept messages for system ID 1
if let Ok(Some(msg)) = parser.push_byte(byte) {
    if let MavMessage::ManualControl(mc) = msg {
        if mc.target == 1 {
            // Process message
        }
    }
}
```

## Conversion Functions

| Function | Description |
|----------|-------------|
| `manual_control_to_gamepad` | Convert MANUAL_CONTROL to GamepadState |
| `mavlink_to_stick` | Convert axis (-1000..1000) to stick (-32768..32767) |
| `mavlink_z_to_trigger` | Convert z (0..1000) to trigger pair |
| `mavlink_to_buttons` | Convert button bitfields to Buttons |

## Protocol Details

### Frame Format (v1)

```
| STX | LEN | SEQ | SYS | CMP | MSG | PAYLOAD | CRC |
| FE  | 1B  | 1B  | 1B  | 1B  | 1B  | 0-255B  | 2B  |
```

### Frame Format (v2)

```
| STX | LEN | INC | CMP | SEQ | SYS | CMP | MSG(3B) | PAYLOAD | CRC |
| FD  | 1B  | 1B  | 1B  | 1B  | 1B  | 1B  | 3B      | 0-255B  | 2B  |
```

CRC: CRC-16/MCRF4XX with message-specific seed byte.

## License

MIT
