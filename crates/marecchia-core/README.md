# @marecchia/marecchia-core 🪐

[![CI](https://github.com/ferrohd/marecchia/actions/workflows/ci.yml/badge.svg)](https://github.com/ferrohd/marecchia/actions/workflows/ci.yml)
![npm version](https://img.shields.io/npm/v/@marecchia/marecchia-core.svg)
![License](https://img.shields.io/badge/license-AGPL--3.0-blue)
[![GitHub stars](https://img.shields.io/github/stars/ferrohd/marecchia?style=social)](https://github.com/ferrohd/marecchia)

`@marecchia/marecchia-core` is the central dependency of the Marecchia ecosystem 🛠.

## Important Note 🚨

**Please note that `@marecchia/marecchia-core` is not intended to be used directly**. Instead, it acts as the core dependency for Marecchia modules designed for specific video players.

**To utilize the Marecchia ecosystem, install the Marecchia modules appropriate for your video player**.

## Marecchia Modules 🧩

Each module is designed to seamlessly bridge `@marecchia/marecchia-core` with its respective video player.

- **[@marecchia/hlsjs](https://www.npmjs.com/package/@marecchia/hlsjs)**: Provides integration with HLS.js, enabling P2P streaming for HLS content.

## Building and Development 👷‍♂️

### Working with wasm-pack

`@marecchia/marecchia-core` is a Rust crate compiled to WebAssembly using `wasm-pack`. To work with the project, you'll need to have both Rust and `wasm-pack` installed.

#### Pre-requisites

- Install Rust: Follow the instructions on the [official Rust site](https://rustup.rs/).
- Install wasm-pack: Instructions can be found at [wasm-pack installation](https://rustwasm.github.io/wasm-pack/installer/).

#### Building the Project

1. **Clone the repository**:

   ```shell
   git clone https://github.com/ferrohd/marecchia.git
   ```

2. **Navigate to the project directory**:

   ```shell
   cd crates/marecchia-core
   ```

3. **Build with wasm-pack**:

   ```shell
   wasm-pack build --target web --scope marecchia
   ```

   This command compiles the Rust code into WebAssembly and prepares it for use in a web project. By default, the output is placed in the `pkg` directory.

## Contribution 🤝

The Marecchia project is open to contributions. Whether you're interested in improving the core package 📦, extending support to additional video players, or helping with documentation 📝, your input is highly valued. Visit our [GitHub](https://github.com/ferrohd/marecchia) repository to learn how you can contribute.

## License 📄

`@marecchia/marecchia-core` is released under the AGPL-3.0 license. For more information, refer to the LICENSE file.

We hope to see the Marecchia ecosystem grow 🌱, with more modules supporting an ever-increasing range of video players and streaming technologies.
