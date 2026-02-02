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
            <span class="logo-icon">📝</span>
            <span class="logo-text">Carbo</span>
          </router-link>
        </div>
        <div class="header-actions">
          <el-dropdown trigger="click" @command="handleExport">
            <button class="action-btn">
              <span>导出</span>
              <span class="dropdown-icon">▾</span>
            </button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="image">导出图片</el-dropdown-item>
                <el-dropdown-item command="pdf">导出 PDF</el-dropdown-item>
                <el-dropdown-item command="ppt">PPT 预览</el-dropdown-item>
                <el-dropdown-item command="html">复制 HTML</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
          <el-dropdown trigger="click" @command="handleSetting">
            <button class="action-btn icon-btn">
              <span>⚙</span>
            </button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="import">导入文件</el-dropdown-item>
                <el-dropdown-item command="clear">清空内容</el-dropdown-item>
                <el-dropdown-item command="imagebed">图床设置</el-dropdown-item>
                <el-dropdown-item divided command="about">关于</el-dropdown-item>
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
        <span>释放以打开文件</span>
      </div>
    </div>

    <!-- Editor -->
    <div class="editor-wrapper">
      <div id="vditor" class="vditor-container" />
    </div>

    <el-dialog v-model="isImageBedDialogOpen" title="GitHub 图床设置" width="520px">
      <el-form :model="imageBedForm" label-width="120px">
        <el-form-item label="启用">
          <el-switch v-model="imageBedForm.enabled" />
        </el-form-item>
        <el-form-item label="仓库">
          <el-input v-model="imageBedForm.repo" placeholder="owner/repo" autocomplete="off" />
        </el-form-item>
        <el-form-item label="分支">
          <el-input v-model="imageBedForm.branch" placeholder="master" autocomplete="off" />
        </el-form-item>
        <el-form-item label="目录前缀">
          <el-input v-model="imageBedForm.pathPrefix" placeholder="images" autocomplete="off" />
        </el-form-item>
        <el-form-item label="Token">
          <el-input v-model="imageBedForm.token" placeholder="GitHub PAT" type="password" show-password autocomplete="off" />
        </el-form-item>
      </el-form>

      <template #footer>
        <div style="display: flex; justify-content: space-between; gap: 12px;">
          <el-button :loading="isValidatingImageBed" @click="validateImageBed">验证</el-button>
          <div style="display: flex; gap: 8px;">
            <el-button @click="isImageBedDialogOpen = false">取消</el-button>
            <el-button type="primary" :loading="isSavingImageBed" @click="saveImageBed">保存</el-button>
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
  branch: 'master',
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
  const branch = imageBedForm.branch.trim() || 'master'
  const pathPrefix = imageBedForm.pathPrefix.trim() || 'images'

  if (imageBedForm.enabled) {
    if (!repo || !/^[^/]+\/[^/]+$/.test(repo)) {
      ElMessage.error('仓库格式应为 owner/repo')
      return
    }
    if (!imageBedForm.token.trim()) {
      ElMessage.error('Token 不能为空')
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
    ElMessage.success('图床设置已保存')
    isImageBedDialogOpen.value = false
  } finally {
    isSavingImageBed.value = false
  }
}

const validateImageBed = async () => {
  const repo = imageBedForm.repo.trim()
  if (!repo || !/^[^/]+\/[^/]+$/.test(repo)) {
    ElMessage.error('仓库格式应为 owner/repo')
    return
  }
  const token = imageBedForm.token.trim()
  if (!token) {
    ElMessage.error('Token 不能为空')
    return
  }

  isValidatingImageBed.value = true
  try {
    const result = await invoke<{ push: boolean; admin: boolean }>('github_validate_repo', { repo, token })
    if (!result.push && !result.admin) {
      ElMessage.error('验证失败：该账号对该仓库没有写权限（需要 collaborator write 或 owner/admin）')
      return
    }
    ElMessage.success('验证通过（已确认账号对仓库有写权限）')
  } catch (e) {
    ElMessage.error(
      `验证失败: ${String(e)}\n\n需要检查：Repository access 包含该仓库；Permissions：Contents(Read and write) + Metadata(Read)。若为组织仓库还需授权 SSO。`
    )
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
    branch: imageBedForm.branch.trim() || 'master',
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
        const msg = String(e)
        if (msg.includes('Resource not accessible by personal access token')) {
          ElMessage.error(
            '图片上传失败：Token 无法写入该仓库。\n请检查：Repository access 包含该仓库；Permissions：Contents(Read and write) + Metadata(Read)；若为组织仓库需授权 SSO。'
          )
        } else {
          ElMessage.error(`图片上传失败: ${msg}`)
        }
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
      // Handle local image paste/upload
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
  ElMessage.success(`已打开: ${basename(filePath)}`)
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
      ElMessage.success(images.length === 1 ? `已插入图片: ${basename(images[0])}` : `已插入 ${images.length} 张图片`)
    }
  } catch {
    ElMessage.error('打开文件失败')
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
      ElMessage.success(`已打开: ${openTarget.name}`)
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
      ElMessage.success(images.length === 1 ? `已插入图片: ${images[0].name}` : `已插入 ${images.length} 张图片`)
    }
  } catch {
    ElMessage.error('打开文件失败')
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
    if (import.meta.env.DEV) ElMessage.error('拖拽监听初始化失败（请打开 DevTools 看 console）')
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
        ElMessage.success('内容已复制到剪贴板')
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
        await ElMessageBox.confirm('确定要清空所有内容吗？', '提示', {
          confirmButtonText: '确定',
          cancelButtonText: '取消',
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
  background-color: #FAFAFA;
  position: relative;
}

/* Header Navigation */
.header-nav {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 56px;
  background-color: #FFFFFF;
  border-bottom: 1px solid #E5E5E5;
  z-index: 1000;
}

.header-content {
  max-width: 1200px;
  height: 100%;
  margin: 0 auto;
  padding: 0 16px;
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
  gap: 8px;
  color: #1A1A1A;
  font-weight: 600;
  font-size: 18px;
  text-decoration: none;
}

.logo-icon {
  font-size: 20px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  background-color: transparent;
  color: #1A1A1A;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.action-btn:hover {
  background-color: #F0F0F0;
}

.action-btn.icon-btn {
  padding: 8px;
  font-size: 16px;
}

.dropdown-icon {
  font-size: 12px;
  opacity: 0.6;
}

/* Drop Overlay */
.drop-overlay {
  position: fixed;
  top: 56px;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
}

.drop-message {
  background: #FFFFFF;
  padding: 32px 48px;
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  font-size: 18px;
  font-weight: 500;
  color: #1A1A1A;
}

.drop-icon {
  font-size: 48px;
}

/* Editor */
.editor-wrapper {
  flex: 1;
  margin-top: 56px;
  padding: 16px;
  display: flex;
  justify-content: center;
}

.vditor-container {
  width: 100%;
  max-width: 1200px;
  height: calc(100vh - 56px - 32px);
  overflow: visible;
}

/* Vditor overrides */
:deep(.vditor) {
  border: 1px solid #E5E5E5;
  border-radius: 12px;
  background-color: #FFFFFF;
  height: 100% !important;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
  overflow: visible;
}

:deep(.vditor-toolbar) {
  background-color: #FFFFFF;
  border-bottom: 1px solid #E5E5E5;
  padding: 8px 12px;
}

:deep(.vditor-toolbar__item) {
  border-radius: 4px;
}

:deep(.vditor-toolbar__item:hover) {
  background-color: #F0F0F0;
}

:deep(.vditor-content) {
  background-color: #FFFFFF;
  height: calc(100% - 40px) !important;
  border-bottom-left-radius: 12px;
  border-bottom-right-radius: 12px;
}

:deep(.vditor-hint),
:deep(.vditor-panel) {
  z-index: 2000;
}

:deep(.vditor-sv) {
  height: 100% !important;
}

:deep(.vditor-reset) {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
}

/* Mobile */
@media (max-width: 768px) {
  .header-content {
    padding: 0 12px;
  }
  
  .editor-wrapper {
    padding: 8px;
  }
  
  .vditor-container {
    height: calc(100vh - 56px - 16px);
  }

  :deep(.vditor) {
    border-radius: 8px;
  }

  :deep(.vditor-content) {
    border-bottom-left-radius: 8px;
    border-bottom-right-radius: 8px;
  }
}
</style>
