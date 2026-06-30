## Guide

Read [The Complete Guide to Bridging Rust with Swift Using UniFFI](Uniffi.md) for a detailed explanation of the integration process, exported APIs, custom types, error handling, and the zero-knowledge Sudoku example.

## SudokuCircuit

A 9×9 Sudoku zero-knowledge circuit written in Rust and exposed to Swift through UniFFI.

The circuit verifies a supplied solution without revealing it. It does not solve the puzzle.

The circuit implementation is based on the work from [this repository](https://github.com/tomasdelclaux/ZK-SNARKs).

### Requirements

- macOS with Xcode 26.2 and Swift 6.2.3
- Rust and Cargo 1.94.1
- `cargo-swift` 0.11.0

Install the required tools:

```sh
cargo install cargo-swift --version 0.11.0
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
```

### Generate the Swift bindings

From the project root, run:

```sh
make generate
```

This creates the local Swift package at `sdk/SudokuBindings`. Run the command again after changing the Rust API.

### Run the tests
Run the Swift tests:

```sh
open SudokuCircuit/SudokuCircuit.xcodeproj
```

Test location: `SudokuCircuit/SudokuCircuitTests/SudokuCircuitTests.swift`

In Xcode, select an iOS Simulator and press **Command-U**.

### Troubleshooting

- **Binding generation fails:** Xcode versions newer than 26.2 may cause binding-generation issues. Use Xcode 26.2 for the most reliable results.
- **Cargo reports `no such command: swift`:** install `cargo-swift` and ensure `~/.cargo/bin` is in `PATH`.
- **Xcode cannot find `SudokuBindings`:** run `make generate`, then resolve package dependencies in Xcode.
- **Swift does not reflect a Rust API change:** regenerate the bindings, then clean and rebuild the Xcode project.
