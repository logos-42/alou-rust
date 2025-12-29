import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
    // 配置代理解决CORS问题
    proxy: {
      '/api': {
        target: 'https://alou-edge.yuanjieliu65.workers.dev',
        changeOrigin: true,
        secure: false,
        rewrite: (path) => path.replace(/^\/api/, '/api'),
        configure: (proxy, options) => {
          // 添加CORS头
          proxy.on('proxyRes', (proxyRes, req, res) => {
            // 添加CORS头
            const origin = req.headers.origin
            if (origin && (origin.includes('localhost') || origin.includes('127.0.0.1'))) {
              res.setHeader('Access-Control-Allow-Origin', origin)
              res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS')
              res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization, X-Requested-With, Accept, Origin')
              res.setHeader('Access-Control-Allow-Credentials', 'true')
              res.setHeader('Access-Control-Expose-Headers', 'Content-Length, Content-Range')
            }
          })
        },
      },
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    // BigInt literals are required by blockchain libs (viem/ox). Safari 13 lacks support,
    // so we target modern runtimes (Tauri desktop Chromium/WebKit >= 14).
    target: 'esnext',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    outDir: './dist',
    emptyOutDir: true,
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
})
