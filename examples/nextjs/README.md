# Marecchia Next.js HLS Example 🎥

This example project demonstrates how to integrate `@marecchia/hls` with Next.js to create a video streaming application utilizing the power of P2P networking for HLS content.

## Prerequisites 📋

Before diving into this example, make sure you have the following installed:

- Node.js (LTS version)
- npm or yarn

## Getting Started 🚀

To get started with this example, follow these simple steps:

### Clone the Repository

First, clone this repository to your local machine:

```bash
git clone https://github.com/your-repository/marecchia-nextjs-example.git
cd marecchia-nextjs-example
```

### Install Dependencies

Next, install the project dependencies using npm or yarn:

```bash
npm install
# or
yarn install
```

### Run the Development Server

Once the dependencies are installed, you can run the development server:

```bash
npm run dev
# or
yarn dev
```

This will start the Next.js development server, typically available at `http://localhost:3000` on your web browser.

## How It Works ✨

The core of this example is a React component that utilizes `@marecchia/hls` to create a P2P-enabled video player.

You can find this component in the [`components/video.tsx`](https://github.com/ferrohd/marecchia/blob/master/examples/nextjs/src/components/video.tsx) file.
