import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [vue()],

    resolve: {
        alias: {
            '@': resolve(__dirname, 'src'),
            '@components': resolve(__dirname, 'src/components'),
            '@pages': resolve(__dirname, 'src/pages'),
            '@assets': resolve(__dirname, 'src/assets'),
            '@config': resolve(__dirname, 'src/config'),
            '@utils': resolve(__dirname, 'src/utils')
        }
    },

    // Vite options tailored for Tauri development
    clearScreen: false,
    server: {
        port: 3000,
        strictPort: true,
        watch: {
            ignored: ['**/src-tauri/**']
        }
    },

    build: {
        // Tauri uses Chromium on Windows and WebKit on macOS and Linux
        target: process.env.TAURI_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
        // don't minify for debug builds
        minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
        // produce sourcemaps for debug builds
        sourcemap: !!process.env.TAURI_DEBUG
    }
})
