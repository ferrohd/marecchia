# Marecchia npm Workspace

This directory serves as the heart of our efforts to bring peer-to-peer (P2P) streaming capabilities to various video players.

## Available Packages 📦

Below is a list of the current npm packages available within the Marecchia project, each tailored to a specific video player technology.

- **@marecchia/hlsjs**: Designed for [`HLS.js`](https://github.com/video-dev/hls.js), this package integrates P2P streaming to HLS (HTTP Live Streaming) content, optimizing video delivery while ensuring seamless playback.

_Aside from the listed package, the Marecchia team is continuously working on expanding support to other popular video players. Stay tuned for more updates._

## Getting Started 🚀

To integrate Marecchia's P2P streaming capabilities with your video player using one of our packages, you'll need Node.js and npm installed on your machine. Follow these steps to get started:

1. **Install the desired package** using npm or yarn. For example, to install the package for HLS.js, run:

    ```bash
    npm install @marecchia/hlsjs
    # or, if you're using yarn
    yarn add @marecchia/hlsjs
    ```

2. **Follow the package-specific documentation** for implementation guidelines and configuration options. Each package comes with its README.md, providing detailed instructions on using Marecchia's P2P capabilities with your chosen video player.

## Development and Contribution 🛠️

We welcome contributions to the Marecchia npm workspace! Whether you're interested in improving an existing package or developing a new integration for another video player, your contributions are invaluable.

### Setting Up For Development

1. **Clone the Marecchia repository** and navigate to the `marecchia` folder:

```bash
git clone https://github.com/your-repository/marecchia.git
cd marecchia/marecchia
```

2. **Install dependencies**:

```bash
npm install
```

3. **Start working on a package** by navigating to its respective directory under `packages/`.

For more detailed information on how to contribute, please refer to the project's main [CONTRIBUTING.md](../CONTRIBUTING.md) document.

## License 📄

All Marecchia npm packages are released under the AGPL License. For more information, please see the [LICENSE](../LICENSE) file located in the root directory of this repository.
