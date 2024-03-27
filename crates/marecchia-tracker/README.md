# Marecchia Tracker 📍

[![CI](https://github.com/ferrohd/marecchia/actions/workflows/ci.yml/badge.svg)](https://github.com/ferrohd/marecchia/actions/workflows/ci.yml)
![License](https://img.shields.io/badge/license-AGPL--3.0-blue)
[![GitHub stars](https://img.shields.io/github/stars/ferrohd/marecchia?style=social)](https://github.com/ferrohd/marecchia)

Marecchia Tracker serves as the rendezvous server in the Marecchia P2P streaming ecosystem 🌐, enabling peers to discover each other and initiate P2P communications.

This critical component ensures efficient peer-to-peer streaming functionality, focusing on the seamless identification and connection of peers. Crafted in Rust for performance and reliability, Marecchia Tracker is designed for scalability and is available as a Docker 🐳 image for easy deployment.

## Overview 📜

The tracker operates alongside `@marecchia/marecchia-core`, a WebAssembly (Wasm) library enabling P2P streaming directly in web browsers. Moreover, the `@marecchia/hls` module encapsulates `@marecchia/marecchia-core`, ensuring seamless integration with HTTP Live Streaming (HLS) technologies.

## Key Features 🔑

- **Efficient Peer Discovery:** Facilitates quick and efficient finding and connecting of peers within the P2P network.
- **Written in Rust:** Built in Rust for performance, reliability, and security.
- **High Scalability:** Engineered to accommodate an expanding network of participants efficiently.
- **Cross-platform Compatibility:** Deployment flexibility across any system supporting Docker, expanding the ecosystem's reach.

## Getting Started 🚀

Deploying the Marecchia Tracker is possible through a Docker image.

### Prerequisites 📋

- [Docker](https://docs.docker.com/get-docker/) installed on your system.

### Deployment 🛠

1. **Pull the Marecchia Tracker Docker image:**

   ```bash
   docker pull ghcr.io/ferrohd/marecchia-tracker
   ```

2. **Run the Marecchia Tracker:**

   ```bash
   docker run -d -p 8000:8000 marecchia/marecchia-tracker:latest
   ```

   This command launches the Tracker and binds it to port 8000 on the host, adjustable to fit your networking needs.

## Contributing 💡

Contributions to Marecchia Tracker and the broader ecosystem are highly encouraged and appreciated.

Refer to  [CONTRIBUTING](https://github.com/ferrohd/marecchia/blob/master/CONTRIBUTING.md) for contribution guidelines and information on submitting pull requests.

## License 📄

Marecchia Tracker is distributed under the AGPL-3.0 license, ensuring open, collaborative development and distribution. For more comprehensive details, please review the LICENSE file in the source code.
