# astro-proto-types

Generated Rust protobuf types for [CometBFT](https://github.com/cometbft/cometbft) protocol.

## Overview

This crate contains auto-generated Rust types from the CometBFT protobuf definitions. It provides:

- Serializable/deserializable structs for all CometBFT message types
- gRPC client and server code via Tonic
- Serde support for JSON serialization
- Well-typed Rust interfaces for CometBFT protocols

## Code Generation

The protobuf code is generated using [buf](https://buf.build/) with the neoeinstein plugin suite:

- `neoeinstein-prost` - Core protobuf generation
- `neoeinstein-prost-serde` - Serde serialization support
- `neoeinstein-prost-crate` - Crate-level configuration
- `neoeinstein-tonic` - gRPC client/server code

## Usage

To regenerate the protobuf code:

```bash
# Install buf if not already installed
make install-buf

# Generate protobuf code
make proto-gen

# Clean generated code
make clean-proto

# Full rebuild
make rebuild-proto
```

## Dependencies

- `prost` - Protocol buffer implementation
- `tonic` - gRPC implementation
- `serde` - Serialization framework
- `bytes` - Efficient byte buffer management
- `pbjson` - Protocol buffer JSON support

## Generated Structure

Generated code is placed in `src/codegen/` and organized by the CometBFT protobuf package structure. The main CometBFT types are re-exported from the crate root for convenient access:

```rust
use astro_proto_types::cometbft;

// Access types like:
// cometbft::abci::v1::Request
// cometbft::types::v1::Block
// etc.
```
