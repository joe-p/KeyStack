# WASM Guest Example

This example demonstrates how to create a Rust library that compiles to WASM and can be used as a guest module with the `keystack-core` host.

## Overview

The guest exports two functions:
- `alloc(len: i32) -> i32`: Allocates memory in the guest, returning a pointer
- `pre_action_hook(...) -> i32`: Processes context data and returns a pointer to result data

## Building

```bash
cargo build --package wasm-guest --target wasm32-unknown-unknown
```

The compiled WASM will be at:
`target/wasm32-unknown-unknown/debug/wasm_guest.wasm`

## Integration with Host

The host (`keystack-core`) loads this WASM and calls the exported functions. See `crates/keystack-core/src/context_provider/wasm_context_provider.rs` for the host implementation.

## API

### alloc
```rust
#[no_mangle]
pub extern "C" fn alloc(len: i32) -> i32
```

Allocates `len` bytes in the guest memory. Returns a pointer to the allocated memory, or 0 on failure.

Note: This leaks memory by design. The host is responsible for managing guest memory lifecycle.

### pre_action_hook
```rust
#[no_mangle]
pub extern "C" fn pre_action_hook(
    user_ptr: i32, user_len: i32,
    key_path_ptr: i32, key_path_len: i32,
    action_id_ptr: i32, action_id_len: i32,
    payload_ptr: i32, payload_len: i32,
) -> i32
```

Processes context data passed by the host and returns result data.

**Parameters:**
- `user_ptr`/`user_len`: Pointer/length to user ID string bytes
- `key_path_ptr`/`key_path_len`: Pointer/length to key path string bytes  
- `action_id_ptr`/`action_id_len`: Pointer/length to action ID string bytes
- `payload_ptr`/`payload_len`: Pointer/length to arbitrary payload bytes

**Returns:**
- Pointer to result buffer (format: `[length: i32][data...]`), or 0 on error
- The first 4 bytes contain the data length (little-endian i32)
- The remaining bytes contain the actual result data

## Testing

Run the integration test:
```bash
cargo test --package keystack-core test_wasm_context_provider_with_context
```

This test:
1. Compiles the guest to WASM
2. Loads the WASM into the host
3. Calls `pre_action_hook` with test context data
4. Verifies the returned result

## Implementation Notes

- Uses `#[unsafe(no_mangle)]` to prevent name mangling of exported functions
- Uses `std::mem::forget` to leak allocated memory (intentional for WASM guests)
- Result format uses a custom encoding with length prefix for easy reading by the host
- All pointer/length pairs are validated by the host before calling
