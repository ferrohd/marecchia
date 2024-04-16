# Marecchia Project 🌊

> [!IMPORTANT]
> 🚧 This project is still under active development and not production-ready

[![CI](https://github.com/ferrohd/marecchia/actions/workflows/ci.yml/badge.svg)](https://github.com/ferrohd/marecchia/actions/workflows/ci.yml)
![License](https://img.shields.io/badge/license-AGPL--3.0-blue)
[![GitHub stars](https://img.shields.io/github/stars/ferrohd/marecchia?style=social)](https://github.com/ferrohd/marecchia)

Welcome to the Marecchia Project, a sophisticated ecosystem designed for enhancing video streaming applications with Peer-to-Peer capabilities for scalable, cost-efficient content delivery.

## Project Structure 📁

The repository is a monorepo organized into the following main sections:

```plaintext
/
├── crates/
│   ├── marecchia-tracker/    # Tracker server for P2P peer discovery
│   └── marecchia-core/       # @marecchia/marecchia-core WASM library for P2P functionality
└── marecchia/
    └── packages/             # NPM packages for integration with various video players
```

### Crates (`/crates`) 🏗️

- **marecchia-tracker**: A Rust-based tracker server, facilitating peer discovery within the P2P network.
- **marecchia-core**: The foundational library, compiled to WebAssembly, which powers the P2P functionality in the browser across video players.

**Consult individual READMEs within each subdirectory for more detailed information about developing and contributing to specific components of the Marecchia ecosystem.**

### Npm Packages (`/marecchia`) 📦

Under this directory, you will find npm packages specifically developed for integrating Marecchia's P2P streaming technology with prevalent video players

## Getting Started ➡️

Jump into the Marecchia Project with these quick steps, whether you're aiming to contribute or implement our P2P tech in your video streaming solutions.

1. **Choose Your Interest**: Decide where you'd like to make an impact—within the Rust-based tracker (`/crates/marecchia-tracker/`), the core WASM library (`/crates/marecchia-core/`), or the npm packages (`/marecchia/packages/`) for video players.

2. **Read the README**: Navigate to the relevant directory and open the `README.md`. It contains essential information on setup, usage, and contribution guidelines.

3. **Get in Touch**: If you have questions or need help, don't hesitate to reach out to the maintainers.

### Need Help? 🆘

Stuck or got questions? Open an issue or see the readme for contact info. We're here to help!

## Contribution Guidelines 🤝

The Marecchia Project welcomes contributions from the community, whether you're fixing bugs, improving documentation, or proposing new features. Please see the [CONTRIBUTING.md](CONTRIBUTING.md) file for guidelines on how to contribute effectively to this project.

## License 📄

The Marecchia Project is licensed under the AGPL license. See the [LICENSE](LICENSE.md) file in the root of the repository for full license text.
