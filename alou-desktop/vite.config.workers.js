import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import path from 'path'
import { fileURLToPath } from 'url'

const _filename = fileURLToPath(import.meta.url)
const _dirname = path.dirname(_filename)

// Workers 生产环境配置
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
    // 配置代理到 Workers 生产环境
    proxy: {
      '/api': {
        target: 'https://alou-edge.yuanjieliu65.workers.dev',
        changeOrigin: true,
        secure: false,
        rewrite: (path) => path.replace(/^\/api/, '/api'),
        configure: (_proxy, _options) => {
          // 添加CORS头
          _proxy.on('proxyRes', (proxyRes, req, res) => {
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
    target: 'esnext',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    outDir: './dist',
    emptyOutDir: true,
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@modules': path.resolve(__dirname, './src/modules'),
      '@ui': path.resolve(__dirname, './src/ui'),
      '@shared': path.resolve(__dirname, './src/shared'),
      '@bridges': path.resolve(__dirname, './src/bridges'),
    },
  },
})
