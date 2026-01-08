# gamepad-proto

Text-based gamepad protocol for UART communication. Provides parsing, serialization, and a fluent builder API.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | No | Enable standard library support |
| `defmt` | No | Enable defmt formatting (for embedded logging) |
| `heapless` | No | Enable `serialize_to_vec()` for `heapless::Vec` output |
| `embedded-io` | No | Enable `serialize_io()` for `embedded_io::Write` targets |

## Protocol Specification

The protocol uses ASCII-based messages terminated with newline (`\n`). All messages include an XOR checksum for error detection.

### Full State Message

Reports complete gamepad state:

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

**Example:** `G0001:0:0:0:0:0:0*47\n` (Button 1 pressed)

### Incremental Update Message

Reports a single field change:

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

### Button Mapping

| Bit | Button | Common Mapping |
|-----|--------|----------------|
| 0 | Button 1 | A / Cross |
| 1 | Button 2 | B / Circle |
| 2 | Button 3 | X / Square |
| 3 | Button 4 | Y / Triangle |
| 4 | Button 5 | Left Bumper |
| 5 | Button 6 | Right Bumper |
| 6 | Button 7 | Back / Select |
| 7 | Button 8 | Start |
| 8 | Button 9 | Left Stick Click |
| 9 | Button 10 | Right Stick Click |
| 10 | Button 11 | Guide / Home |
| 11 | Button 12 | D-Pad Up |
| 12 | Button 13 | D-Pad Down |
| 13 | Button 14 | D-Pad Left |
| 14 | Button 15 | D-Pad Right |
| 15 | Button 16 | Reserved |

### Checksum

XOR of all ASCII bytes before the `*` character:

```rust
fn calculate_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc ^ b)
}
```

## Usage

### Serializing Full State

```rust
use gamepad_proto::{GamepadState, Buttons, Serialize};

let state = GamepadState {
    buttons: Buttons::A | Buttons::B,
    left_stick: gamepad_proto::AnalogStick { x: 1000, y: -500 },
    ..GamepadState::neutral()
};

let mut buf = [0u8; 64];
let len = state.serialize(&mut buf).unwrap();
// Send buf[..len] over UART
```

### Using the Builder API

```rust
use gamepad_proto::{MessageBuilder, Buttons};

let mut buf = [0u8; 64];

// Full state message
let len = MessageBuilder::full_state()
    .buttons(Buttons::X | Buttons::Y)
    .left_stick(1000, -500)
    .triggers(128, 64)
    .serialize(&mut buf)
    .unwrap();

// Incremental update
let len = MessageBuilder::update()
    .buttons(Buttons::A)
    .serialize(&mut buf)
    .unwrap();
```

### Parsing Messages

```rust
use gamepad_proto::{parse_message, ParsedMessage};

let input = b"G0001:0:0:0:0:0:0*47\n";
match parse_message(input) {
    Ok(ParsedMessage::FullState(state)) => {
        println!("Buttons: {:?}", state.buttons);
    }
    Ok(ParsedMessage::Update(update)) => {
        println!("Update: {:?}", update);
    }
    Err(e) => eprintln!("Parse error: {:?}", e),
}
```

## License

MIT
