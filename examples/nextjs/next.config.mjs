/** @type {import('next').NextConfig} */
const nextConfig = {
    // React Strict Mode is disabled in development mode to avoid double rendering
    reactStrictMode: process.env.NODE_ENV === 'production',
    output: 'export',
    webpack: (config, _options) => {
        config.experiments = {
            layers: true,
            asyncWebAssembly: true
        };
        return config;
    },
};

export default nextConfig;
