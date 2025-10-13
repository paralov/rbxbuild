# rbxbuild

A lightweight CLI that converts Rojo project JSON strings into Roblox place XML files.

## About

This project leverages the [Rojo](https://rojo.space/) ecosystem, specifically the `rbx_dom_weak` and related crates, to provide a minimal build tool. It uses Rojo's robust DOM implementation to parse project files and serialize them into Roblox XML format, without the full overhead of the complete Rojo toolchain.

## Usage

Pass a Rojo project JSON string to the tool:

```bash
rbxbuild '{"name": "MyProject", "tree": {...}}'
```

The tool will output the corresponding Roblox place XML to stdout.

## Building

Build the project using Cargo:

```bash
cargo build --release
```

The compiled binary will be available at `target/release/rojo-build-lite` (or `rbxbuild` depending on your Cargo configuration).
