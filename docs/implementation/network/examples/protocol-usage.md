# Protocol Usage Examples

This guide demonstrates protocol setup, interaction, debugging, and optimization in the P2P network module.

## Protocol Setup
```rust
// Example: Define a protocol state and initialize it
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum ProtocolState {
    Init,
    Handshake,
    Active,
    Closed,
}

let mut state = ProtocolState::Init;
```

## Protocol Interaction
```rust
// Simulate a handshake and activation
state = ProtocolState::Handshake;
// ... perform handshake logic ...
state = ProtocolState::Active;
```

## Protocol Debugging
```rust
// Print protocol state for debugging
println!("Current protocol state: {:?}", state);
```

## Protocol Optimization
```rust
// Example: Use enums and pattern matching for efficient state transitions
match state {
    ProtocolState::Init => println!("Initializing..."),
    ProtocolState::Handshake => println!("Handshaking..."),
    ProtocolState::Active => println!("Active!"),
    ProtocolState::Closed => println!("Closed."),
}
```
