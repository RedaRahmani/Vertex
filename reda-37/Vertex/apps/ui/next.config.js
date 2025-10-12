/** @type {import('next').NextConfig} */

const nextConfig = {
  transpilePackages: [
    '@keystone/sdk',
    '@solana/web3.js',
    '@coral-xyz/anchor',
    '@solana/wallet-adapter-react',
    '@solana/wallet-adapter-react-ui',
    '@solana/wallet-adapter-wallets'
  ],
  webpack: (config) => {
    config.resolve = config.resolve || {};
    config.resolve.fallback = {
      ...(config.resolve.fallback || {}),
      fs: false,
      net: false,
      tls: false,
      crypto: false
    };
    return config;
  }
};
module.exports = nextConfig;
