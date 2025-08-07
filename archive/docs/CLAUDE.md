# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Melda is a Delta-State JSON CRDT (Conflict-free Replicated Data Type) library written in Rust. It provides a way to synchronize changes made to arbitrary JSON documents across multiple replicas without requiring central coordination.

## Key Architecture

### Core Components

- **CRDT Core** (`src/melda.rs`): Main CRDT implementation providing update, commit, meld, and conflict resolution functionality
- **Adapters** (`src/*adapter.rs`): Pluggable storage backends for persisting delta states
  - `FilesystemAdapter`: Stores data as files in a directory
  - `MemoryAdapter`: In-memory storage
  - `SQLiteAdapter`: SQLite database storage (feature-gated)
  - `SolidAdapter`: Solid Pod storage (feature-gated)
  - `Flate2Adapter`: Compression wrapper using Deflate
  - `BrotliAdapter`: Compression wrapper using Brotli (feature-gated)
- **Revision Management** (`src/revision.rs`, `src/revisiontree.rs`): Handles versioning and revision trees for tracking object changes
- **Data Storage** (`src/datastorage.rs`): Manages delta blocks and data packs

### Data Model

- **Delta Blocks** (`.delta` files): Contains versioning information for each object
- **Data Packs** (`.pack` files): Stores actual JSON content
- **Object Flattening**: Arrays marked with `♭` suffix are flattened to track individual objects
- **Unique Identifiers**: Each object has an `_id` field, root object uses `√`

## Development Commands

### Building
```bash
cargo build
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run doc tests
cargo test --doc

# Run specific test
cargo test test_name
```

### Code Quality
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run clippy linter
cargo clippy -- -D warnings

# Type checking
cargo check
```

### Examples
```bash
# Run the simple example
cargo run --example simple

# Run the concurrent array insertions example
cargo run --example concurrent_array_insertions

# Run the concurrent array deletions example (idempotence test)
cargo run --example concurrent_array_deletions

# Run the complex interleaved operations example
cargo run --example complex_interleaved_operations

# Run ordering behavior analysis tests
cargo run --example simple_index_ordering_test
cargo run --example replica_grouping_test
```

### Feature Flags
```bash
# Build with specific features
cargo build --features "solid sqlitedb brotliadapter"

# Build without default features
cargo build --no-default-features
```

## Testing Considerations

- Tests use `#[test]` attribute and are found in `mod tests` blocks within source files
- Some tests use `serial_test` for sequential execution to avoid concurrent filesystem access
- Solid adapter tests require environment variables: `MELDA_SOLID_USERNAME`, `MELDA_SOLID_PASSWORD`, `MELDA_SOLID_URL`, `MELDA_SOLID_FOLDER`

## Common Development Tasks

When modifying the CRDT implementation:
1. Changes to core logic should be made in `src/melda.rs`
2. New storage adapters should implement the `Adapter` trait from `src/adapter.rs`
3. Test adapters thoroughly as they handle critical data persistence

When working with flattened arrays:
- Objects in arrays marked with `♭` are extracted and tracked individually
- Each extracted object needs a unique `_id` field
- The parent object references extracted objects by their IDs