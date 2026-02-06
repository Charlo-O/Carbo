<template>
  <div 
    class="main-page" 
    v-loading="isLoading"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
  >
    <!-- Header Navigation -->
    <header class="header-nav">
      <div class="header-content">
        <div class="header-logo">
          <router-link to="/" class="logo-link">
            <img src="/src-tauri/icons/32x32.png" class="logo-img" alt="Carbo logo" />
            <span class="logo-text">Carbo</span>
          </router-link>
        </div>
        <div class="header-actions">
          <el-dropdown trigger="click" @command="handleExport" popper-class="custom-dropdown">
            <button class="action-btn">
              <span>Export</span>
              <span class="dropdown-icon">▾</span>
            </button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="image">Image</el-dropdown-item>
                <el-dropdown-item command="pdf">PDF</el-dropdown-item>
                <el-dropdown-item command="ppt">PPT Preview</el-dropdown-item>
                <el-dropdown-item command="html">Copy HTML</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
          <el-dropdown trigger="click" @command="handleSetting" popper-class="custom-dropdown">
            <button class="action-btn icon-btn">
              <svg class="settings-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="3"></circle>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
              </svg>
            </button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="import">Import...</el-dropdown-item>
                <el-dropdown-item command="clear">Clear All</el-dropdown-item>
                <el-dropdown-item command="imagebed">Image Host</el-dropdown-item>
                <el-dropdown-item divided command="about">About</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </div>
    </header>

    <!-- Drop Zone Overlay -->
    <div v-if="isDragging" class="drop-overlay">
      <div class="drop-message">
        <span class="drop-icon">📄</span>
        <span>Drop to open</span>
      </div>
    </div>

    <!-- Editor -->
    <div class="editor-wrapper">
      <div id="vditor" class="vditor-container" />
    </div>

    <el-dialog v-model="isImageBedDialogOpen" title="Image Bed Settings" width="480px" class="custom-dialog">
      <el-form :model="imageBedForm" label-position="top">
        <el-form-item label="Enable GitHub Upload">
          <el-switch v-model="imageBedForm.enabled" />
        </el-form-item>
        <template v-if="imageBedForm.enabled">
          <el-form-item label="Repository (owner/repo)">
            <el-input v-model="imageBedForm.repo" placeholder="e.g. username/assets" />
          </el-form-item>
          <el-form-item label="Branch">
            <el-input v-model="imageBedForm.branch" placeholder="main" />
          </el-form-item>
          <el-form-item label="Path Prefix">
            <el-input v-model="imageBedForm.pathPrefix" placeholder="images" />
          </el-form-item>
          <el-form-item label="Personal Access Token">
            <el-input v-model="imageBedForm.token" placeholder="ghp_..." type="password" show-password />
          </el-form-item>
        </template>
      </el-form>

      <template #footer>
        <div class="dialog-footer">
          <el-button @click="validateImageBed" :loading="isValidatingImageBed" link>Test Connection</el-button>
          <div>
            <el-button @click="isImageBedDialogOpen = false">Cancel</el-button>
            <el-button type="primary" :loading="isSavingImageBed" @click="saveImageBed">Save</el-button>
          </div>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import Vditor from 'vditor'
import { defaultContent } from '@config/default'
import { convertFileSrc, invoke, isTauri } from '@tauri-apps/api/core'

const router = useRouter()

// Reactive state
const isLoading = ref(true)
const isDragging = ref(false)
let vditor: Vditor | null = null

// Local storage key
const STORAGE_KEY = 'carbo-markdown-content'

// Image bed (MVP): store config in localStorage.
// Note: Token in localStorage is NOT secure; MVP only.
const IMAGE_BED_KEY = 'carbo-imagebed-config'

type ImageBedConfig = {
  enabled: boolean
  repo: string
  branch: string
  pathPrefix: string
  token: string
}

type GitHubUploadJob = {
  localPath: string
  localUrl: string
}

const MAX_GITHUB_CONTENTS_BYTES = 1_000_000

const githubUploadQueue: GitHubUploadJob[] = []
let githubUploadRunning = false

const isImageBedDialogOpen = ref(false)
const isSavingImageBed = ref(false)
const isValidatingImageBed = ref(false)

const imageBedForm = reactive<ImageBedConfig>({
  enabled: false,
  repo: '',
  branch: 'main',
  pathPrefix: 'images',
  token: ''
})

const loadImageBed = () => {
  try {
    const raw = localStorage.getItem(IMAGE_BED_KEY)
    if (!raw) return
    const parsed: unknown = JSON.parse(raw)
    if (typeof parsed !== 'object' || parsed === null) return

    const r = parsed as Partial<ImageBedConfig>
    imageBedForm.enabled = Boolean(r.enabled)
    if (typeof r.repo === 'string') imageBedForm.repo = r.repo
    if (typeof r.branch === 'string') imageBedForm.branch = r.branch
    if (typeof r.pathPrefix === 'string') imageBedForm.pathPrefix = r.pathPrefix
    if (typeof r.token === 'string') imageBedForm.token = r.token
  } catch {
    // ignore
  }
}

const saveImageBed = async () => {
  const repo = imageBedForm.repo.trim()
  const branch = imageBedForm.branch.trim() || 'main'
  const pathPrefix = imageBedForm.pathPrefix.trim() || 'images'

  if (imageBedForm.enabled) {
    if (!repo || !/^[^/]+\/[^/]+$/.test(repo)) {
      ElMessage.error('Invalid repo format (owner/repo)')
      return
    }
    if (!imageBedForm.token.trim()) {
      ElMessage.error('Token is required')
      return
    }
  }

  isSavingImageBed.value = true
  try {
    const cfg: ImageBedConfig = {
      enabled: imageBedForm.enabled,
      repo,
      branch,
      pathPrefix,
      token: imageBedForm.token.trim()
    }
    localStorage.setItem(IMAGE_BED_KEY, JSON.stringify(cfg))
    ElMessage.success('Settings saved')
    isImageBedDialogOpen.value = false
  } finally {
    isSavingImageBed.value = false
  }
}

const validateImageBed = async () => {
  const repo = imageBedForm.repo.trim()
  if (!repo || !/^[^/]+\/[^/]+$/.test(repo)) {
    ElMessage.error('Invalid repo format')
    return
  }
  const token = imageBedForm.token.trim()
  if (!token) {
    ElMessage.error('Token is required')
    return
  }

  isValidatingImageBed.value = true
  try {
    const result = await invoke<{ push: boolean; admin: boolean }>('github_validate_repo', { repo, token })
    if (!result.push && !result.admin) {
      ElMessage.error('Validation failed: No write access')
      return
    }
    ElMessage.success('Validation successful')
  } catch (e) {
    ElMessage.error(`Validation error: ${String(e)}`)
  } finally {
    isValidatingImageBed.value = false
  }
}

const getActiveImageBedConfig = (): ImageBedConfig | null => {
  if (!imageBedForm.enabled) return null
  const repo = imageBedForm.repo.trim()
  const token = imageBedForm.token.trim()
  if (!repo || !token) return null
  return {
    enabled: true,
    repo,
    branch: imageBedForm.branch.trim() || 'main',
    pathPrefix: imageBedForm.pathPrefix.trim() || 'images',
    token
  }
}

const replaceUrlInEditor = (fromUrl: string, toUrl: string) => {
  if (!vditor) return
  const value = vditor.getValue()
  if (!value.includes(fromUrl)) return
  const updated = value.split(fromUrl).join(toUrl)
  if (updated === value) return
  vditor.setValue(updated)
  localStorage.setItem(STORAGE_KEY, updated)
}

const runGitHubUploadQueue = async () => {
  if (githubUploadRunning) return
  githubUploadRunning = true
  try {
    while (githubUploadQueue.length > 0) {
      const cfg = getActiveImageBedConfig()
      if (!cfg) break

      const job = githubUploadQueue.shift()
      if (!job) break

      try {
        const rawUrl = await invoke<string>('github_upload_image_from_path', {
          repo: cfg.repo,
          branch: cfg.branch,
          pathPrefix: cfg.pathPrefix,
          token: cfg.token,
          localPath: job.localPath,
          maxBytes: MAX_GITHUB_CONTENTS_BYTES
        })
        replaceUrlInEditor(job.localUrl, rawUrl)
      } catch (e) {
        ElMessage.error(`Upload failed: ${String(e)}`)
      }
    }
  } finally {
    githubUploadRunning = false
  }
}

const enqueueGitHubUpload = (job: GitHubUploadJob) => {
  const cfg = getActiveImageBedConfig()
  if (!cfg) return
  githubUploadQueue.push(job)
  void runGitHubUploadQueue()
}

// Initialize Vditor editor with local image support
const initVditor = () => {
  vditor = new Vditor('vditor', {
    width: '100%',
    height: '100%',
    tab: '\t',
    counter: { enable: true, max: 999999 },
    typewriterMode: true,
    mode: 'sv',
    toolbarConfig: {
      pin: true
    },
    hint: {
      delay: 200
    },
    preview: {
      delay: 100
    },
    outline: {
      enable: true,
      position: 'right'
    },
    cache: {
      enable: true,
      id: 'carbo-editor'
    },
    upload: {
      handler: async (files: File[]) => {
        for (const file of files) {
          if (!file.type.startsWith('image/')) continue

          const filePath = getDroppedFilePath(file)
          if (filePath) {
            await insertImageFromDiskPath(filePath)
            continue
          }

          const savedPath = await saveImageToAppData(file)
          if (savedPath) {
            const localUrl = insertImageFromPath(savedPath)
            if (localUrl) enqueueGitHubUpload({ localPath: savedPath, localUrl })
            continue
          }

          const reader = new FileReader()
          reader.onload = (e) => {
            const base64 = e.target?.result as string
            vditor?.insertValue(`![${file.name}](${base64})`)
          }
          reader.readAsDataURL(file)
        }
        return null
      },
      accept: 'image/*'
    },
    after: () => {
      const content = localStorage.getItem(STORAGE_KEY) || defaultContent
      vditor?.setValue(content)
      vditor?.focus()
      isLoading.value = false
    },
    input: (value: string) => {
      localStorage.setItem(STORAGE_KEY, value)
    }
  })
}

// Set default content if empty
const setDefaultContent = () => {
  const savedContent = localStorage.getItem(STORAGE_KEY) || ''
  if (!savedContent.trim()) {
    localStorage.setItem(STORAGE_KEY, defaultContent)
  }
}

// Drag and drop helpers
type UnlistenFn = () => void

const isRecord = (v: unknown): v is Record<string, unknown> => typeof v === 'object' && v !== null

const devLog = (...args: unknown[]) => {
  if (import.meta.env.DEV) console.log(...args)
}

const isTauriRuntime = () => isTauri()

const toFileUrl = (filePath: string) => {
  const path = filePath.replace(/\\/g, '/')
  if (/^[a-zA-Z]:\//.test(path)) return `file:///${encodeURI(path)}`
  if (path.startsWith('/')) return `file://${encodeURI(path)}`
  return filePath
}

const basename = (filePath: string) => {
  const normalized = filePath.replace(/\\/g, '/')
  const parts = normalized.split('/')
  return parts[parts.length - 1] || filePath
}

const isMarkdownPath = (filePath: string) => {
  const p = filePath.toLowerCase()
  return p.endsWith('.md') || p.endsWith('.markdown') || p.endsWith('.txt')
}

const isImagePath = (filePath: string) => /\.(png|jpe?g|gif|webp|svg|bmp|ico)$/i.test(filePath)

const isFileDrag = (e: DragEvent) => Array.from(e.dataTransfer?.types ?? []).includes('Files')

const readFileAsText = (file: File) =>
  new Promise<string>((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = (event) => resolve((event.target?.result ?? '') as string)
    reader.onerror = () => reject(new Error('FileReader error'))
    reader.readAsText(file)
  })

const readFileAsDataUrl = (file: File) =>
  new Promise<string>((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = (event) => resolve((event.target?.result ?? '') as string)
    reader.onerror = () => reject(new Error('FileReader error'))
    reader.readAsDataURL(file)
  })

const isMarkdownLike = (file: File) => {
  const name = file.name.toLowerCase()
  return name.endsWith('.md') || name.endsWith('.markdown') || name.endsWith('.txt')
}

const getDroppedFilePath = (file: File) => {
  const maybeRecord: unknown = file
  if (!isRecord(maybeRecord)) return null
  const maybePath = maybeRecord['path']
  return typeof maybePath === 'string' && maybePath.trim() ? maybePath : null
}

const openMarkdownFromPath = async (filePath: string) => {
  if (!vditor) return
  const content = isTauriRuntime()
    ? await invoke<string>('read_text_file', { path: filePath })
    : await (async () => {
        const url = toFileUrl(filePath)
        const res = await fetch(url)
        if (!res.ok) throw new Error('Failed to read file')
        return await res.text()
      })()

  vditor.setValue(content)
  localStorage.setItem(STORAGE_KEY, content)
  ElMessage.success(`Opened: ${basename(filePath)}`)
}

const insertImageFromPath = (filePath: string) => {
  if (!vditor) return null
  const url = isTauriRuntime() ? convertFileSrc(filePath) : toFileUrl(filePath)
  vditor.insertValue(`![${basename(filePath)}](${url})`)
  return url
}

const ensureImageInAppData = async (filePath: string) => {
  if (!isTauriRuntime()) return filePath
  const normalized = filePath.replace(/\\/g, '/')
  if (normalized.includes('/carbo-assets/images/')) return filePath
  return await invoke<string>('copy_image_to_app_data', { path: filePath })
}

const insertImageFromDiskPath = async (filePath: string) => {
  const resolvedPath = await ensureImageInAppData(filePath)
  const localUrl = insertImageFromPath(resolvedPath)
  if (localUrl) {
    enqueueGitHubUpload({ localPath: resolvedPath, localUrl })
  }
  return resolvedPath
}

const saveImageToAppData = async (file: File) => {
  if (!isTauriRuntime()) return null
  const buffer = await file.arrayBuffer()
  const bytes = Array.from(new Uint8Array(buffer))

  const result = await invoke<string>('save_image_bytes', {
    file_name: file.name,
    bytes
  })
  return result
}

const handleDroppedPaths = async (paths: string[]) => {
  if (!vditor) return
  if (paths.length === 0) return

  const openTarget = paths.find(isMarkdownPath) ?? paths[0]
  try {
    if (isMarkdownPath(openTarget)) {
      await openMarkdownFromPath(openTarget)
      return
    }

    const images = paths.filter(isImagePath)
    if (images.length > 0) {
      for (const imgPath of images) await insertImageFromDiskPath(imgPath)
      ElMessage.success(images.length === 1 ? `Inserted: ${basename(images[0])}` : `Inserted ${images.length} images`)
    }
  } catch {
    ElMessage.error('Failed to open file')
  }
}

const handleDroppedFiles = async (fileList: FileList) => {
  if (!vditor) return

  const files = Array.from(fileList)
  if (files.length === 0) return

  // Prefer opening a markdown-like file if present.
  const openTarget = files.find(isMarkdownLike) ?? files[0]

  try {
    if (isMarkdownLike(openTarget)) {
      const content = await readFileAsText(openTarget)
      vditor.setValue(content)
      localStorage.setItem(STORAGE_KEY, content)
      ElMessage.success(`Opened: ${openTarget.name}`)
      return
    }

    // For images: insert all dropped images.
    const images = files.filter((f) => f.type.startsWith('image/'))
    if (images.length > 0) {
      for (const img of images) {
        const filePath = getDroppedFilePath(img)
        if (filePath) {
          await insertImageFromDiskPath(filePath)
          continue
        }

        const savedPath = await saveImageToAppData(img)
        if (savedPath) {
          insertImageFromPath(savedPath)
          continue
        }

        const base64 = await readFileAsDataUrl(img)
        vditor.insertValue(`![${img.name}](${base64})`)
      }
      ElMessage.success(images.length === 1 ? `Inserted: ${images[0].name}` : `Inserted ${images.length} images`)
    }
  } catch {
    ElMessage.error('Failed to open file')
  }
}

// Drag and drop handlers
const onDragOver = (e: DragEvent) => {
  if (isTauriRuntime()) return
  if (!isFileDrag(e)) return
  e.preventDefault()
  isDragging.value = true
}

const onDragLeave = (e: DragEvent) => {
  if (isTauriRuntime()) return
  if (!isFileDrag(e)) return
  isDragging.value = false
}

const onDrop = (e: DragEvent) => {
  if (isTauriRuntime()) return
  if (!isFileDrag(e)) return
  e.preventDefault()
  isDragging.value = false
  const files = e.dataTransfer?.files
  if (files) void handleDroppedFiles(files)
}

const onWindowDragOverCapture = (e: DragEvent) => {
  if (isTauriRuntime()) return
  if (!isFileDrag(e)) return
  // Ensure dropping works even if inner components stop propagation.
  e.preventDefault()
}

const onWindowDropCapture = (e: DragEvent) => {
  if (isTauriRuntime()) return
  if (!isFileDrag(e)) return
  e.preventDefault()
  isDragging.value = false
  const files = e.dataTransfer?.files
  if (files) void handleDroppedFiles(files)
}

let tauriUnlistenFileDrop: UnlistenFn | null = null

const setupTauriFileDrop = async () => {
  // Tauri v2: use the dedicated drag/drop API to receive real filesystem paths.
  if (!isTauriRuntime()) return

  try {
    const { getCurrentWebview } = await import('@tauri-apps/api/webview')
    devLog('[drag-drop] register onDragDropEvent')

    tauriUnlistenFileDrop = await getCurrentWebview().onDragDropEvent((event) => {
      devLog('[drag-drop] event', event.payload.type)

      if (event.payload.type === 'over') {
        isDragging.value = true
        return
      }

      if (event.payload.type === 'drop') {
        isDragging.value = false
        void handleDroppedPaths(event.payload.paths)
        return
      }

      // cancelled
      isDragging.value = false
    })
  } catch (err) {
    console.error('[drag-drop] failed to register', err)
    if (import.meta.env.DEV) ElMessage.error('Drag drop init failed (check console)')
  }
}

// Export handlers
const handleExport = (command: string) => {
  switch (command) {
    case 'image':
      router.push('/export/image')
      break
    case 'pdf':
      router.push('/export/pdf')
      break
    case 'ppt':
      router.push('/export/ppt')
      break
    case 'html':
      const content = localStorage.getItem(STORAGE_KEY) || ''
      if (content) {
        navigator.clipboard.writeText(content)
        ElMessage.success('HTML copied to clipboard')
      }
      break
  }
}

// Settings handlers
const handleSetting = async (command: string) => {
  switch (command) {
    case 'import':
      importFile()
      break
    case 'imagebed':
      isImageBedDialogOpen.value = true
      break
    case 'clear':
      try {
        await ElMessageBox.confirm('Are you sure you want to clear all content?', 'Warning', {
          confirmButtonText: 'Yes, clear it',
          cancelButtonText: 'Cancel',
          type: 'warning'
        })
        localStorage.removeItem(STORAGE_KEY)
        vditor?.setValue('')
      } catch { }
      break
    case 'about':
      router.push('/about')
      break
  }
}

// Import file
const importFile = () => {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.md,.markdown,.txt,image/*'
  input.onchange = (e: Event) => {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (file) {
      if (file.type.startsWith('image/')) {
        const reader = new FileReader()
        reader.onload = (event) => {
          const base64 = event.target?.result as string
          vditor?.insertValue(`![${file.name}](${base64})`)
        }
        reader.readAsDataURL(file)
      } else {
        const reader = new FileReader()
        reader.onload = (event) => {
          const content = event.target?.result as string
          vditor?.setValue(content)
          localStorage.setItem(STORAGE_KEY, content)
        }
        reader.readAsText(file)
      }
    }
  }
  input.click()
}

// Lifecycle hooks
onMounted(() => {
  setDefaultContent()
  loadImageBed()
  initVditor()

  // Web fallback: use HTML5 drag/drop to read File objects.
  if (!isTauriRuntime()) {
    // Capture file drops globally so dropping works even if inner components stop propagation.
    window.addEventListener('dragover', onWindowDragOverCapture, true)
    window.addEventListener('drop', onWindowDropCapture, true)
  }

  // Tauri: native drag/drop yields real filesystem paths.
  void setupTauriFileDrop()
})

onBeforeUnmount(() => {
  window.removeEventListener('dragover', onWindowDragOverCapture, true)
  window.removeEventListener('drop', onWindowDropCapture, true)
  tauriUnlistenFileDrop?.()
  vditor?.destroy()
})
</script>

<style scoped>
.main-page {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: var(--color-bg-primary);
  position: relative;
}

/* Header Navigation */
.header-nav {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: var(--header-height);
  background-color: var(--color-bg-primary);
  border-bottom: 1px solid var(--color-border);
  z-index: 1000;
}

.header-content {
  max-width: var(--max-content-width);
  height: 100%;
  margin: 0 auto;
  padding: 0 var(--space-4);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-logo {
  flex-shrink: 0;
}

.logo-link {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--color-text-primary);
  font-weight: 600;
  font-size: 16px;
  text-decoration: none;
  letter-spacing: -0.01em;
}

.logo-img {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  object-fit: contain;
  flex: 0 0 auto;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 10px;
  background-color: transparent;
  color: var(--color-text-secondary);
  border: none;
  border-radius: var(--radius-md);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.action-btn:hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.action-btn.icon-btn {
  width: 32px;
  height: 32px;
  padding: 0;
  justify-content: center;
  border-radius: 999px;
}

.settings-icon {
  width: 18px;
  height: 18px;
}

.dropdown-icon {
  font-size: 10px;
  opacity: 0.5;
  margin-left: 2px;
}

/* Drop Zone */
.drop-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(255, 255, 255, 0.9);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 2000;
  border: 2px dashed var(--color-text-tertiary);
  margin: 16px;
  border-radius: var(--radius-lg);
  backdrop-filter: blur(4px);
}

.drop-message {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-4);
  color: var(--color-text-secondary);
  font-size: var(--text-lg);
  font-weight: 500;
}

.drop-icon {
  font-size: 48px;
}

/* Editor Wrapper */
.editor-wrapper {
  flex: 1;
  margin-top: var(--header-height);
  width: 100%;
  overflow: visible;
  background-color: var(--color-bg-primary);
}

.vditor-container {
  height: 100%;
}

:deep(.vditor) {
  border: none !important;
  overflow: visible !important;
}

:deep(.vditor-toolbar) {
  display: flow-root !important;
  padding: 0 var(--space-4) !important;
  border-bottom: 1px solid var(--color-border) !important;
  background-color: var(--color-bg-primary) !important;
  overflow: visible !important;
}

/* The pinned toolbar creates a stacking context (z-index: 1 in Vditor).
   Raise it above the fixed header so tooltips aren't hidden behind it. */
:deep(.vditor-toolbar--pin) {
  z-index: 1100 !important;
}

:deep(.vditor-toolbar__item) {
  overflow: visible !important;
}

:deep(.vditor-content) {
  font-family: var(--font-primary) !important;
}

/* Vditor Toolbar Tooltips - Force all tooltips to display ABOVE (north) */
/* Keep a consistent gap between buttons and tooltips */
:deep(.vditor-tooltipped) {
  position: relative;
  overflow: visible !important;
}

/* Force all toolbar tooltips to display ABOVE the button */
:deep(.vditor-toolbar__item .vditor-tooltipped__s::after),
:deep(.vditor-toolbar__item .vditor-tooltipped__se::after),
:deep(.vditor-toolbar__item .vditor-tooltipped__sw::after),
:deep(.vditor-toolbar__item .vditor-tooltipped__n::after),
:deep(.vditor-toolbar__item .vditor-tooltipped__ne::after),
:deep(.vditor-toolbar__item .vditor-tooltipped__nw::after) {
  top: auto !important;
  bottom: 100% !important;
  margin-top: 0 !important;
  margin-bottom: 5px !important;
  left: 50% !important;
  right: auto !important;
  transform: translateX(-50%) !important;
  margin-left: 0 !important;
  margin-right: 0 !important;
}

:deep(.vditor-toolbar__item .vditor-tooltipped__s::before),
:deep(.vditor-toolbar__item .vditor-tooltipped__se::before),
:deep(.vditor-toolbar__item .vditor-tooltipped__sw::before),
:deep(.vditor-toolbar__item .vditor-tooltipped__n::before),
:deep(.vditor-toolbar__item .vditor-tooltipped__ne::before),
:deep(.vditor-toolbar__item .vditor-tooltipped__nw::before) {
  top: -5px !important;
  bottom: auto !important;
  border-top-color: #3b3e43 !important;
  border-bottom-color: transparent !important;
  left: 50% !important;
  right: auto !important;
  margin-left: -5px !important;
  margin-right: 0 !important;
}

:deep(.vditor-tooltipped:hover::before),
:deep(.vditor-tooltipped:hover::after) {
  display: inline-block !important;
  opacity: 1 !important;
  visibility: visible !important;
  text-decoration: none !important;
  animation: tooltip-appear 0.15s ease-in forwards !important;
}

@keyframes tooltip-appear {
  from { opacity: 0; }
  to { opacity: 1; }
}

/* Dialog Footer */
.dialog-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
