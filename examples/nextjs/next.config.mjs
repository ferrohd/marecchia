/** @type {import('next').NextConfig} */
const nextConfig = {
    reactStrictMode: true,
    output: 'export',
    webpack: (config, _options) => {
        config.experiments = {
            layers: true,
            asyncWebAssembly: true
        };
        return config;
    }
};

export default nextConfig;
