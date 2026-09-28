/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "standalone",
  reactStrictMode: true,
  async rewrites() {
    // Tarayıcıyı her zaman aynı origin'de tutar: hangi IP/host'tan açılırsa açılsın
    // /api/* istekleri sunucu tarafından API container'ına proxy'lenir (CORS sorunları biter).
    const api =
      process.env.API_PROXY_TARGET ??
      process.env.NEXT_PUBLIC_API_URL ??
      "http://127.0.0.1:18081";
    return [
      {
        source: "/api/:path*",
        destination: `${api}/api/:path*`,
      },
    ];
  },
};

export default nextConfig;
