# Marecchia: P2P Streaming on the Web

**Note:**
This project is still a work in progress and is not yet ready for production use. Please check back later for updates.

Welcome to the Marecchia project repository! Marecchia is a revolutionary solution designed to enable peer-to-peer (P2P) streaming directly in your web browser, leveraging the power of WebAssembly (Wasm) for efficient video content distribution among viewers. This project also features a backend "tracker" system, essential for facilitating the discovery and interconnection of peers for optimized P2P content sharing.

## Features

- **WebAssembly Library:** A cutting-edge Wasm library specifically crafted to work in tandem with your browser's video player element. It empowers the video player to engage in a P2P network, streaming video content from viewer to viewer, which significantly reduces the dependency on server bandwidth.

- **Tracker Binary:** A dedicated backend service that serves as a crucial meeting point for peers. This tracker enables viewers to find each other, creating a web of connections that ensure the most efficient paths for data transfer are utilized, making the streaming process smoother and faster.

## Getting Started

Begin your journey with Marecchia by following these easy steps:

### Prerequisites

- Node.js (version 14.0 or newer recommended)
- A modern web browser with WebAssembly support enabled
- Rust programming language environment (for compiling the tracker and Wasm library)

### Setting Up the Tracker

To set up the tracker, which helps peers find each other:

1. **Build the Tracker:**

    ```bash
    cd tracker
    cargo build --release
    ```

2. **Run the Tracker:**

    Start it by executing:

    ```bash
    ./target/release/tracker
    ```

    The tracker by default listens on port `8080`, but this can be adjusted as per your requirements in the configuration.

### Integrating the Wasm Library

To integrate the Wasm library with your video player:

1. **Compile the Wasm Library:**

    ```bash
    cd wasm-lib
    cargo build --target wasm32-unknown-unknown --release
    ```

    Then, use `wasm-bindgen` (or another tool of your choice) to generate the required JavaScript bindings.

2. **Integration Example:**

    An integration example can be found at `examples/integration.html`. Include the Wasm and JS bindings in your web application and initialize the Marecchia library within your video player setup to get started.

## Usage

With the Marecchia library integrated and the tracker running, your video player will automatically try to utilize P2P connections for streaming video content among viewers.

Ensure that your video player element is properly set up to use the Wasm library and initiates the P2P streaming functionality on initialization.

## Contributing

Contributions are highly appreciated! Whether it's feature enhancements, documentation improvements, or bug fixes, feel free to fork the project and submit a pull request.

## License

Marecchia is open-sourced under the [MIT License](LICENSE), providing a great degree of freedom for personal and commercial use.

---

For additional assistance, queries, or to report issues, please create an issue in the GitHub repository.
