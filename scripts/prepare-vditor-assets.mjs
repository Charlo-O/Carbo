import { cp, mkdir, stat } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const __dirname = dirname(fileURLToPath(import.meta.url))
const rootDir = join(__dirname, '..')

const vditorDistDir = join(rootDir, 'node_modules', 'vditor', 'dist')
const publicVditorDistDir = join(rootDir, 'public', 'vditor', 'dist')
const tauriFaviconPng = join(rootDir, 'src-tauri', 'icons', '32x32.png')
const publicFaviconPng = join(rootDir, 'public', 'favicon.png')

await stat(vditorDistDir)
await mkdir(publicVditorDistDir, { recursive: true })

const copyDir = async (name) =>
  cp(join(vditorDistDir, name), join(publicVditorDistDir, name), {
    recursive: true,
    force: true
  })

await Promise.all([copyDir('js'), copyDir('css'), copyDir('images')])
await cp(join(vditorDistDir, 'index.css'), join(publicVditorDistDir, 'index.css'), { force: true })

// Keep the web favicon in sync with the Tauri app icon.
await cp(tauriFaviconPng, publicFaviconPng, { force: true })
