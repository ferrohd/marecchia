# Marecchia Project Root

Welcome to the Marecchia Project, a sophisticated ecosystem designed for enhancing video streaming applications with efficient Peer-to-Peer (P2P) capabilities.

The Marecchia ecosystem is crafted to address the challenges in video streaming by leveraging the power of P2P technology for scalable, cost-efficient content delivery.

This project is split into two core parts: Rust-based backend components housed under `crates` and a collection of npm packages tailored for integrating P2P streaming into various video players located under `marecchia`.

## Project Structure 📁

The repository is organized into the following main sections:

```plaintext
/
├── crates/
│   ├── marecchia-tracker/    # Rust backend for P2P peer discovery
│   └── marecchia-core/       # @marecchia/marecchia-core WASM library for P2P functionality (core dependency)
└── marecchia/
    └── packages/             # NPM packages for integration with various video players
```

### Crates (`/crates`)

- **marecchia-tracker**: A Rust-based tracker server, facilitating peer discovery within the P2P network.
- **marecchia-core**: The foundational library, compiled to WebAssembly, which powers the P2P functionality in the browser across video players.

**Consult individual READMEs within each subdirectory for more detailed information about developing and contributing to specific components of the Marecchia ecosystem.**

### Marecchia npm Workspace (`/marecchia`)

Under this directory, you will find npm packages specifically developed for integrating Marecchia's P2P streaming technology with prevalent video players

## Getting Started ➡️

To begin working with the Marecchia Project, ensure you have the following prerequisites installed on your system:

- [Rust](https://www.rust-lang.org/tools/install) (including Cargo), needed for developing and building the Rust-based components.
- [Node.js](https://nodejs.org/) and [npm](https://www.npmjs.com/), required for managing the npm packages and workspace.

### Setup Instructions

1. **Clone the Repo:**

    ```sh
    git clone https://github.com/username/marecchia.git
    cd marecchia
    ```

2. **Set Up the Rust Environment:**

    Navigate to each Rust crate under `/crates` to build or develop the Rust components:

    ```sh
    cd crates/marecchia-tracker
    cargo build
    ```

    Repeat for `marecchia-core`.

3. **Install npm Dependencies:**

    Within the `marecchia/` directory, install the npm dependencies for the workspace:

    ```sh
    cd marecchia
    npm install
    ```

## Contribution Guidelines 🤝

The Marecchia Project welcomes contributions from the community, whether you're fixing bugs, improving documentation, or proposing new features. Please see the [CONTRIBUTING.md](CONTRIBUTING.md) file for guidelines on how to contribute effectively to this project.

## License 📄

The Marecchia Project is licensed under the AGPL license. See the [LICENSE](LICENSE) file in the root of the repository for full license text.
