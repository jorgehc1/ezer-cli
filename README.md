# Ezer CLI 💠

The official command-line interface for the **Ezerdesk** plugin ecosystem. It allows developers to quickly scaffold, build, and package WebAssembly plugins.

## Installation

```bash
cargo install ezer-cli
```

## Commands

### `new`
Scaffolds a new plugin project with the recommended directory structure and dependencies.

```bash
ezer-cli new my-plugin
```

### `build`
Compiles the plugin to the `wasm32-unknown-unknown` target.

```bash
ezer-cli build
```

### `package` (Coming Soon)
Creates a `.ezer` package bundle ready for upload to the Ezerdesk Marketplace.

## Getting Started

1. Create a new plugin: `ezer-cli new my-awesome-plugin`
2. Enter the directory: `cd my-awesome-plugin`
3. Build the Wasm module: `ezer-cli build`
4. The output will be in `target/wasm32-unknown-unknown/debug/my_awesome_plugin.wasm`

## License

Apache-2.0