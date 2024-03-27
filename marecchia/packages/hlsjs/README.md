# @marecchia/hlsjs 📦

[![CI](https://github.com/ferrohd/marecchia/actions/workflows/ci.yml/badge.svg)](https://github.com/ferrohd/marecchia/actions/workflows/ci.yml)
![npm version](https://img.shields.io/npm/v/@marecchia/hlsjs.svg)
![License](https://img.shields.io/badge/license-AGPL--3.0-blue)
[![GitHub stars](https://img.shields.io/github/stars/ferrohd/marecchia?style=social)](https://github.com/ferrohd/marecchia)

Enhance your HLS streaming experience with Marecchia, a TypeScript library that brings peer-to-peer (P2P) capabilities to HLS streams.

Through the power of a WebAssembly (WASM) module written in Rust, Marecchia efficiently manages P2P networking, optimizing bandwidth and improving streaming quality.

> [!IMPORTANT]
> This README covers the setup and usage of the Marecchia NPM package.
> For details on setting up the `Marecchia Tracker`, please refer to the separate [README in the`crates/marecchia-tracker`](<https://github.com/ferrohd/marecchia/blob/master/crates/marecchia-tracker/README.md>) directory.

## Features 🚀

- **P2P-Enabled HLS Streaming**: Offload your server's load by distributing streaming data across viewers.
- **WASM-Powered Performance**: Benefit from a high-performance WASM module that handles all P2P logic.
- **Seamless Integration**: Easily integrates with existing HLS.js projects through the `P2PFragmentLoader` class.
- **Automatic Fallback**: Ensures uninterrupted streaming by defaulting to HTTP loading if P2P connectivity fails.

## Installation 💾

Add Marecchia to your project using npm:

```bash
npm install @marecchia/hlsjs
```

## Getting Started 🌱

To get started with Marecchia, ensure your project is already utilizing HLS.js. Marecchia serves as an extension to HLS.js, adding P2P functionality.

### Usage 🔧

Below is a basic guide on how to integrate Marecchia into your HLS.js setup:

1. **Import in Your Project**:

```typescript
import Hls from 'hls.js';
import init, { p2pFragmentLoader } from "@marecchia/hlsjs";
```

2. **Configure HLS.js to Use Marecchia**:

```typescript
// Ensure HLS.js is supported in the user's browser
if (Hls.isSupported()) {
    const video = document.getElementById('video');
    // Init the Marecchia WASM module
    await init();
    const fLoader = p2pFragmentLoader(props.src);
    const hls = new Hls({
        // Set the custom fragment loader in the Hls config
        fLoader
    });

    // Load your .m3u8 source
    hls.loadSource('https://example.com/your_stream.m3u8');
    hls.attachMedia(video);

    // Play the video after the manifest is loaded
    hls.on(Hls.Events.MANIFEST_PARSED, function () {
        video.play();
    });

    // Enjoy P2P streaming!
}
```

For more advanced usage and configuration options, refer to the [examples](https://github.com/ferrohd/marecchia/tree/master/examples) folder

## Contribution 🤝

Contributions to Marecchia are welcome! If you have improvements or fixes, please fork the repository, make your changes, and submit a pull request. See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## License 📜

Marecchia and its NPM package are under the AGPL-3.0 license. For more details, refer to the [LICENSE](LICENSE) file.
