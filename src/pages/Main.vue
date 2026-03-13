<template>
  <div class="workbench" :class="[`theme-${theme}`, widthClass, { 'sidebar-collapsed': !sidebarOpen, 'focus-mode': focusMode }]">
    <aside class="sidebar">
      <div class="sidebar-top">
        <button class="ghost-btn" @click="sidebarOpen = !sidebarOpen">{{ sidebarOpen ? '收起' : '展开' }}</button>
        <button class="ghost-btn" @click="openFolder">打开文件夹</button>
      </div>

      <template v-if="sidebarOpen">
        <section class="sidebar-section">
          <div class="section-title">最近文件</div>
          <button v-for="item in recentFiles" :key="item.path" class="sidebar-item" @click="openFilePath(item.path)">
            <strong>{{ item.name }}</strong>
            <span>{{ truncateMiddle(item.parent, 32) }}</span>
          </button>
          <div v-if="recentFiles.length === 0" class="empty-text">还没有最近文件</div>
        </section>

        <section class="sidebar-section">
          <div class="section-title">最近项目</div>
          <button v-for="item in recentProjects" :key="item.path" class="sidebar-item" @click="openFolderPath(item.path)">
            <strong>{{ item.name }}</strong>
            <span>{{ truncateMiddle(item.path, 32) }}</span>
          </button>
          <div v-if="recentProjects.length === 0" class="empty-text">还没有最近项目</div>
        </section>

        <section class="sidebar-section">
          <div class="section-title">当前文件夹</div>
          <div v-if="openedFolderPath" class="folder-path">{{ truncateMiddle(openedFolderPath, 40) }}</div>
          <button v-for="entry in folderEntries" :key="entry.path" class="sidebar-item" :class="{ active: entry.path === currentFilePath }" @click="openFilePath(entry.path)">
            <strong>{{ entry.name }}</strong>
            <span>{{ entry.path === currentFilePath ? '当前文档' : '点击打开' }}</span>
          </button>
          <div v-if="folderEntries.length === 0" class="empty-text">打开一个文件夹后，这里会显示 .md / .txt 文件。</div>
        </section>
      </template>
    </aside>

    <main class="editor-shell" v-loading="isLoading" @dragover="onDragOver" @dragleave="onDragLeave" @drop="onDrop">
      <div v-if="isDragging" class="drop-overlay">
        <div class="drop-card">拖拽文件到这里打开或插入</div>
      </div>

      <header class="topbar">
        <div>
          <div class="doc-title-row">
            <h1>{{ currentDocumentName }}</h1>
            <span class="dirty-dot" :class="{ visible: isDirty }"></span>
          </div>
          <p class="doc-meta">{{ currentDocumentMeta }}</p>
        </div>

        <div class="toolbar-actions">
          <button class="toolbar-btn" @click="createNewDocument">新建</button>
          <button class="toolbar-btn" @click="openFile">打开</button>
          <button class="toolbar-btn" @click="saveDocument" :disabled="saveStatus === 'saving'">保存</button>
          <button class="toolbar-btn" @click="saveAsDocument">另存为</button>
          <el-dropdown trigger="click" @command="handleCommand">
            <button class="toolbar-btn primary">更多 ▾</button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="export-pdf">导出 PDF</el-dropdown-item>
                <el-dropdown-item command="export-html">导出 HTML</el-dropdown-item>
                <el-dropdown-item command="export-md">导出 Markdown</el-dropdown-item>
                <el-dropdown-item divided command="toggle-theme">切换主题</el-dropdown-item>
                <el-dropdown-item command="toggle-focus">专注模式</el-dropdown-item>
                <el-dropdown-item command="toggle-width">阅读宽度</el-dropdown-item>
                <el-dropdown-item command="about">关于</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </header>

      <div class="statusbar">
        <span>{{ saveStatusLabel }}</span>
        <span>字数 {{ wordCount }}</span>
        <span>{{ currentFilePath ? '自动保存到当前文件' : '未命名文档：内容先保存为草稿' }}</span>
        <span>快捷键：⌘/Ctrl+N O S Shift+S / F / H / P</span>
      </div>

      <div v-if="searchPanelOpen" class="search-panel">
        <div class="search-row">
          <input v-model="searchQuery" class="search-input" placeholder="搜索文本"
            @keydown.enter.exact.prevent="findNext"
            @keydown.shift.enter.prevent="findPrev" />
          <span class="search-count" v-if="searchMatches.length > 0">{{ currentMatchIdx + 1 }}/{{ searchMatches.length }}</span>
          <button class="toolbar-btn icon-btn" title="上一个 (Shift+Enter)" @click="findPrev">↑</button>
          <button class="toolbar-btn icon-btn" title="下一个 (Enter)" @click="findNext">↓</button>
        </div>
        <div class="search-row">
          <input v-model="replaceQuery" class="search-input" placeholder="替换为（可选）" @keydown.enter.prevent="replaceNext" />
          <button class="toolbar-btn" @click="replaceNext">替换</button>
          <button class="toolbar-btn" @click="replaceAll">全部</button>
          <button class="toolbar-btn" @click="searchPanelOpen = false">关闭</button>
        </div>
      </div>

      <div class="editor-wrapper">
        <div id="vditor" class="vditor-container" />
      </div>
    </main>
  </div>

  <Teleport to="body">
    <div v-if="commandPaletteOpen" class="cmd-backdrop" @click="commandPaletteOpen = false">
      <div class="cmd-palette" @click.stop @keydown.escape.prevent="commandPaletteOpen = false">
        <input
          ref="cmdInputRef"
          v-model="commandPaletteQuery"
          class="cmd-input"
          placeholder="搜索文件…"
          @keydown.escape.prevent="commandPaletteOpen = false"
          @keydown.enter.prevent="cmdPaletteConfirm"
          @keydown.up.prevent="commandPaletteIdx = Math.max(0, commandPaletteIdx - 1)"
          @keydown.down.prevent="commandPaletteIdx = Math.min(commandPaletteFiles.length - 1, commandPaletteIdx + 1)"
        />
        <div class="cmd-list">
          <div v-if="commandPaletteFiles.length === 0" class="cmd-empty">
            {{ openedFolderPath ? '没有匹配的文件' : '请先打开一个文件夹' }}
          </div>
          <button
            v-for="(file, i) in commandPaletteFiles"
            :key="file.path"
            class="cmd-item"
            :class="{ active: i === commandPaletteIdx }"
            @click="cmdPaletteSelect(file.path)"
            @mouseenter="commandPaletteIdx = i"
          >
            <span class="cmd-item-name">{{ file.name }}</span>
            <span class="cmd-item-path">{{ truncateMiddle(file.path, 64) }}</span>
          </button>
        </div>
        <div class="cmd-footer">
          <span>↑↓ 导航</span>
          <span>↵ 打开</span>
          <span>Esc 关闭</span>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import type Vditor from 'vditor'
import { convertFileSrc, invoke, isTauri } from '@tauri-apps/api/core'
import { defaultContent } from '@config/default'
import { consumePendingOpenPaths, pendingOpenPaths } from '@utils/openPaths'
import { basename, dirname, getRelativePath, isImagePath, isMarkdownPath, toFileUrl, truncateMiddle } from '@utils/path'
import { clearDraft, loadDraft, loadRecentFiles, loadRecentProjects, pushRecentFile, pushRecentProject, saveDraft, type FolderEntry, type RecentItem, type SaveStatus } from '@utils/workbench'

const compressImage = (file: File, maxSize = 1920, quality = 0.82): Promise<Blob> =>
  new Promise((resolve) => {
    const img = new Image()
    const url = URL.createObjectURL(file)
    img.onload = () => {
      URL.revokeObjectURL(url)
      let { width, height } = img
      if (width > maxSize || height > maxSize) {
        if (width > height) { height = Math.round(height * maxSize / width); width = maxSize }
        else { width = Math.round(width * maxSize / height); height = maxSize }
      }
      const canvas = document.createElement('canvas')
      canvas.width = width; canvas.height = height
      canvas.getContext('2d')!.drawImage(img, 0, 0, width, height)
      canvas.toBlob((blob) => resolve(blob ?? file), file.type === 'image/png' ? 'image/png' : 'image/jpeg', quality)
    }
    img.onerror = () => { URL.revokeObjectURL(url); resolve(file) }
    img.src = url
  })

const router = useRouter()

const isLoading = ref(true)
const isDragging = ref(false)
const sidebarOpen = ref(true)
const commandPaletteOpen = ref(false)
const commandPaletteQuery = ref('')
const commandPaletteIdx = ref(0)
const cmdInputRef = ref<HTMLInputElement | null>(null)
const focusMode = ref(false)
const theme = ref<'light' | 'dark'>('light')
const widthMode = ref<'narrow' | 'medium' | 'wide'>('medium')
const wordCount = ref(0)
const saveStatus = ref<SaveStatus>('idle')
const currentFilePath = ref('')
const openedFolderPath = ref('')
const recentFiles = ref<RecentItem[]>(loadRecentFiles())
const recentProjects = ref<RecentItem[]>(loadRecentProjects())
const folderEntries = ref<FolderEntry[]>([])
const searchPanelOpen = ref(false)
const searchQuery = ref('')
const replaceQuery = ref('')
const searchMatches = ref<number[]>([])
const currentMatchIdx = ref(-1)
let vditor: Vditor | null = null
let autosaveTimer: number | null = null
let suspendInput = false
let lastSavedContent = ''
let outlineScrollCleanup: (() => void) | null = null

const commandPaletteFiles = computed(() => {
  const source = folderEntries.value.length > 0
    ? folderEntries.value
    : recentFiles.value.map(f => ({ path: f.path, name: f.name }))
  if (!commandPaletteQuery.value.trim()) return source
  const q = commandPaletteQuery.value.toLowerCase()
  return source.filter(f => f.name.toLowerCase().includes(q) || f.path.toLowerCase().includes(q))
})

const currentDocumentName = computed(() => currentFilePath.value ? basename(currentFilePath.value) : '未命名文档')
const currentDocumentMeta = computed(() => {
  if (currentFilePath.value) return truncateMiddle(currentFilePath.value, 88)
  return '草稿模式 · 第一次保存后会绑定真实文件'
})
const isDirty = computed(() => !!vditor && vditor.getValue() !== lastSavedContent)
const widthClass = computed(() => `width-${widthMode.value}`)
const saveStatusLabel = computed(() => ({ idle: '未保存', saving: '正在保存…', saved: '已保存', error: '保存失败' }[saveStatus.value]))

const isTauriRuntime = () => isTauri()

const pickOpen = async (options: Record<string, unknown>) => {
  const { open } = await import('@tauri-apps/plugin-dialog')
  return await open(options)
}

const pickSavePath = async (options: Record<string, unknown>) => {
  const { save } = await import('@tauri-apps/plugin-dialog')
  return await save(options)
}

const loadFolderEntries = async (folderPath: string) => {
  if (!folderPath || !isTauriRuntime()) {
    folderEntries.value = []
    return
  }
  try {
    folderEntries.value = await invoke<FolderEntry[]>('list_text_files_in_dir', { path: folderPath })
    openedFolderPath.value = folderPath
    recentProjects.value = pushRecentProject(folderPath)
  } catch (error) {
    folderEntries.value = []
    ElMessage.error(`读取文件夹失败: ${String(error)}`)
  }
}

const applyEditorContent = (content: string) => {
  if (!vditor) return
  suspendInput = true
  vditor.setValue(content)
  suspendInput = false
  wordCount.value = content.length
  saveDraft(content)
  lastSavedContent = content
  saveStatus.value = currentFilePath.value ? 'saved' : 'idle'
}

const loadDocumentState = async (path: string, content: string) => {
  currentFilePath.value = path
  applyEditorContent(content)
  recentFiles.value = pushRecentFile(path)
  const folderPath = dirname(path)
  if (folderPath) await loadFolderEntries(folderPath)
}

const openFilePath = async (path: string) => {
  if (!vditor) return
  try {
    const content = isTauriRuntime()
      ? await invoke<string>('read_text_file', { path })
      : await fetch(toFileUrl(path)).then((res) => res.text())
    await loadDocumentState(path, content)
    ElMessage.success(`已打开 ${basename(path)}`)
  } catch (error) {
    ElMessage.error(`打开失败: ${String(error)}`)
  }
}

const createNewDocument = () => {
  currentFilePath.value = ''
  saveStatus.value = 'idle'
  openedFolderPath.value = ''
  folderEntries.value = []
  applyEditorContent(defaultContent)
}

const persistCurrentContent = async (targetPath: string) => {
  if (!vditor) return ''
  const content = vditor.getValue()
  if (!isTauriRuntime()) {
    const blob = new Blob([content], { type: 'text/markdown;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = basename(targetPath)
    link.click()
    URL.revokeObjectURL(url)
    lastSavedContent = content
    saveStatus.value = 'saved'
    return targetPath
  }

  saveStatus.value = 'saving'
  const savedPath = await invoke<string>('write_text_file', { path: targetPath, content })
  currentFilePath.value = savedPath
  lastSavedContent = content
  saveStatus.value = 'saved'
  saveDraft(content)
  clearDraft()
  recentFiles.value = pushRecentFile(savedPath)
  const folderPath = dirname(savedPath)
  if (folderPath) await loadFolderEntries(folderPath)
  return savedPath
}

const saveAsDocument = async () => {
  try {
    const suggested = currentFilePath.value || 'untitled.md'
    const filePath = await pickSavePath({
      title: '另存为 Markdown',
      defaultPath: suggested,
      filters: [{ name: 'Markdown', extensions: ['md', 'markdown', 'txt'] }]
    })
    if (!filePath || Array.isArray(filePath)) return
    const savedPath = await persistCurrentContent(filePath)
    if (savedPath) ElMessage.success(`已保存到 ${savedPath}`)
  } catch (error) {
    saveStatus.value = 'error'
    ElMessage.error(`保存失败: ${String(error)}`)
  }
}

const saveDocument = async () => {
  try {
    if (!currentFilePath.value) {
      await saveAsDocument()
      return
    }
    const savedPath = await persistCurrentContent(currentFilePath.value)
    if (savedPath) ElMessage.success('已保存')
  } catch (error) {
    saveStatus.value = 'error'
    ElMessage.error(`保存失败: ${String(error)}`)
  }
}

const scheduleAutosave = () => {
  if (!currentFilePath.value || !vditor) return
  if (autosaveTimer) window.clearTimeout(autosaveTimer)
  autosaveTimer = window.setTimeout(async () => {
    try {
      await persistCurrentContent(currentFilePath.value)
    } catch {
      saveStatus.value = 'error'
    }
  }, 1200)
}

const openFile = async () => {
  try {
    const result = await pickOpen({
      title: '打开 Markdown 文件',
      multiple: false,
      filters: [{ name: 'Markdown', extensions: ['md', 'markdown', 'txt'] }]
    })
    if (!result || Array.isArray(result)) return
    await openFilePath(result)
  } catch (error) {
    ElMessage.error(`打开失败: ${String(error)}`)
  }
}

const openFolderPath = async (folderPath: string) => {
  await loadFolderEntries(folderPath)
}

const openFolder = async () => {
  try {
    const result = await pickOpen({ title: '打开文件夹', directory: true, multiple: false })
    if (!result || Array.isArray(result)) return
    await openFolderPath(result)
  } catch (error) {
    ElMessage.error(`打开文件夹失败: ${String(error)}`)
  }
}

const saveExportBytesWithDialog = async (params: { title: string; fileName: string; filters: { name: string; extensions: string[] }[]; bytes: number[] }) => {
  if (!isTauriRuntime()) return null
  const filePath = await pickSavePath({
    title: params.title,
    defaultPath: params.fileName,
    filters: params.filters
  })
  if (!filePath || Array.isArray(filePath)) return null
  return await invoke<string>('save_export_bytes', { fileName: params.fileName, filePath, bytes: params.bytes })
}

const buildExportHtmlDocument = (title: string, bodyHtml: string) => `<!DOCTYPE html>
<html lang="zh-CN"><head><meta charset="UTF-8" /><meta name="viewport" content="width=device-width, initial-scale=1.0" /><title>${title}</title><style>
:root{color-scheme:light}@page{size:A4;margin:18mm 16mm}body{margin:0;color:#111827;font:16px/1.75 -apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif}main{max-width:860px;margin:0 auto;padding:24px 20px 48px}h1,h2,h3,h4{line-height:1.28;break-after:avoid-page}pre,blockquote,table,img{break-inside:avoid;page-break-inside:avoid}img{max-width:100%;height:auto}pre{padding:14px 16px;border-radius:12px;background:#f3f4f6;overflow:auto}code{font-family:ui-monospace,SFMono-Regular,Menlo,monospace}table{width:100%;border-collapse:collapse}th,td{border:1px solid #d1d5db;padding:8px 10px}blockquote{margin:0;padding-left:16px;border-left:4px solid #d1d5db;color:#4b5563}hr{border:0;border-top:1px solid #e5e7eb;margin:24px 0}a{color:#2563eb}
</style></head><body><main class="markdown-body">${bodyHtml}</main></body></html>`

const exportMarkdown = async () => {
  if (!vditor) return
  const fileName = `${currentDocumentName.value.replace(/\.(md|markdown|txt)$/i, '') || 'carbo-export'}.md`
  const bytes = Array.from(new TextEncoder().encode(vditor.getValue()))
  const savedPath = await saveExportBytesWithDialog({ title: '导出 Markdown', fileName, filters: [{ name: 'Markdown', extensions: ['md'] }], bytes })
  if (savedPath) ElMessage.success(`已导出: ${savedPath}`)
}

const exportHtml = async () => {
  if (!vditor) return
  const fileName = `${currentDocumentName.value.replace(/\.(md|markdown|txt)$/i, '') || 'carbo-export'}.html`
  const html = buildExportHtmlDocument(currentDocumentName.value, vditor.getHTML())
  const bytes = Array.from(new TextEncoder().encode(html))
  const savedPath = await saveExportBytesWithDialog({ title: '导出 HTML', fileName, filters: [{ name: 'HTML', extensions: ['html'] }], bytes })
  if (savedPath) ElMessage.success(`已导出: ${savedPath}`)
}

const getAllMatchPositions = (text: string, query: string): number[] => {
  const positions: number[] = []
  let idx = 0
  while (true) {
    const found = text.indexOf(query, idx)
    if (found < 0) break
    positions.push(found)
    idx = found + 1
  }
  return positions
}

const navigateToMatchInEditor = (query: string, matchIdx: number) => {
  const editor = document.querySelector('.vditor-sv__editor') as HTMLElement | null
  if (!editor) return

  // Walk text nodes to find the Nth occurrence
  const walker = document.createTreeWalker(editor, NodeFilter.SHOW_TEXT)
  let currentPos = 0
  let occurrenceCount = 0
  const range = document.createRange()
  let startSet = false
  let endSet = false

  let node: Node | null
  while ((node = walker.nextNode())) {
    const textNode = node as Text
    const nodeText = textNode.data
    const nodeLen = nodeText.length

    let searchFrom = 0
    while (true) {
      const found = nodeText.indexOf(query, searchFrom)
      if (found < 0) break
      if (occurrenceCount === matchIdx) {
        range.setStart(textNode, found)
        startSet = true
        const endInNode = found + query.length
        if (endInNode <= nodeLen) {
          range.setEnd(textNode, endInNode)
          endSet = true
        }
      }
      occurrenceCount++
      searchFrom = found + 1
    }

    // Handle match spanning nodes (rare in sv mode, but handle gracefully)
    if (startSet && !endSet) {
      const remaining = query.length - (nodeLen - (range.startOffset))
      if (remaining <= 0) {
        range.setEnd(textNode, nodeLen)
        endSet = true
      }
    }

    if (startSet && endSet) break
    currentPos += nodeLen
  }

  if (!startSet) return

  const selection = window.getSelection()
  selection?.removeAllRanges()
  selection?.addRange(range)
  editor.focus()
  try {
    ;(range.startContainer as Element).parentElement?.scrollIntoView({ behavior: 'smooth', block: 'center' })
  } catch {
    // ignore scroll errors
  }
}

const runSearch = () => {
  if (!vditor || !searchQuery.value) return
  const value = vditor.getValue()
  const matches = getAllMatchPositions(value, searchQuery.value)
  if (matches.length === 0) {
    ElMessage.warning('没有找到匹配内容')
    searchMatches.value = []
    currentMatchIdx.value = -1
    return
  }
  searchMatches.value = matches
  currentMatchIdx.value = 0
  navigateToMatchInEditor(searchQuery.value, 0)
}

const findNext = () => {
  if (!searchQuery.value) return
  if (searchMatches.value.length === 0) { runSearch(); return }
  currentMatchIdx.value = (currentMatchIdx.value + 1) % searchMatches.value.length
  navigateToMatchInEditor(searchQuery.value, currentMatchIdx.value)
}

const findPrev = () => {
  if (!searchQuery.value) return
  if (searchMatches.value.length === 0) { runSearch(); return }
  currentMatchIdx.value = (currentMatchIdx.value - 1 + searchMatches.value.length) % searchMatches.value.length
  navigateToMatchInEditor(searchQuery.value, currentMatchIdx.value)
}

const replaceNext = () => {
  if (!vditor || !searchQuery.value) return
  const value = vditor.getValue()
  // Use current match position if available, otherwise find first
  const idx = searchMatches.value.length > 0 && currentMatchIdx.value >= 0
    ? searchMatches.value[currentMatchIdx.value]
    : value.indexOf(searchQuery.value)
  if (idx < 0) {
    ElMessage.warning('没有找到可替换内容')
    return
  }
  const updated = `${value.slice(0, idx)}${replaceQuery.value}${value.slice(idx + searchQuery.value.length)}`
  applyEditorContent(updated)
  saveDraft(updated)
  scheduleAutosave()
  // Refresh matches after replacement
  const newMatches = getAllMatchPositions(updated, searchQuery.value)
  searchMatches.value = newMatches
  currentMatchIdx.value = Math.min(currentMatchIdx.value, newMatches.length - 1)
  if (newMatches.length > 0) navigateToMatchInEditor(searchQuery.value, currentMatchIdx.value)
}

const replaceAll = () => {
  if (!vditor || !searchQuery.value) return
  const value = vditor.getValue()
  if (!value.includes(searchQuery.value)) {
    ElMessage.warning('没有找到可替换内容')
    return
  }
  const updated = value.split(searchQuery.value).join(replaceQuery.value)
  applyEditorContent(updated)
  saveDraft(updated)
  scheduleAutosave()
  ElMessage.success('已全部替换')
}

const setupOutlineHighlight = () => {
  const preview = (document.querySelector('.vditor-sv__preview') ?? document.querySelector('.vditor-preview')) as HTMLElement | null
  if (!preview) return

  const getHeadings = () => Array.from(preview.querySelectorAll('h1,h2,h3,h4,h5,h6')) as HTMLElement[]
  const getItems = () => Array.from(document.querySelectorAll('.vditor-outline__item')) as HTMLElement[]

  const highlight = () => {
    const headings = getHeadings()
    const items = getItems()
    if (!headings.length || !items.length) return
    const scrollTop = preview.scrollTop + 80
    let activeIdx = 0
    for (let i = 0; i < headings.length; i++) {
      if (headings[i].offsetTop <= scrollTop) activeIdx = i
    }
    items.forEach((item, i) => item.classList.toggle('vditor-outline__item--current', i === activeIdx))
  }

  const clickHandler = (e: MouseEvent) => {
    const item = (e.target as HTMLElement).closest('.vditor-outline__item') as HTMLElement | null
    if (!item) return
    const idx = getItems().indexOf(item)
    const headings = getHeadings()
    if (idx >= 0 && headings[idx]) {
      e.stopPropagation()
      e.preventDefault()
      preview.scrollTo({ top: headings[idx].offsetTop - 20, behavior: 'smooth' })
      getItems().forEach((el, i) => el.classList.toggle('vditor-outline__item--current', i === idx))
    }
  }

  const outlineEl = document.querySelector('.vditor-outline') as HTMLElement | null
  outlineEl?.addEventListener('click', clickHandler, true)
  preview.addEventListener('scroll', highlight, { passive: true })

  outlineScrollCleanup = () => {
    preview.removeEventListener('scroll', highlight)
    outlineEl?.removeEventListener('click', clickHandler, true)
  }
}

const handleCommand = async (command: string) => {
  switch (command) {
    case 'export-pdf':
      router.push({ path: '/export/pdf', query: { auto: '1', name: currentDocumentName.value } })
      break
    case 'export-html':
      await exportHtml()
      break
    case 'export-md':
      await exportMarkdown()
      break
    case 'toggle-theme':
      theme.value = theme.value === 'light' ? 'dark' : 'light'
      break
    case 'toggle-focus':
      focusMode.value = !focusMode.value
      break
    case 'toggle-width':
      widthMode.value = widthMode.value === 'narrow' ? 'medium' : widthMode.value === 'medium' ? 'wide' : 'narrow'
      break
    case 'about':
      router.push('/about')
      break
  }
}

const handleDroppedPaths = async (paths: string[]) => {
  const markdownTarget = paths.find(isMarkdownPath)
  if (markdownTarget) {
    await openFilePath(markdownTarget)
    return
  }
  const imageTargets = paths.filter(isImagePath)
  if (imageTargets.length === 0 || !vditor) return
  for (const path of imageTargets) {
    if (currentFilePath.value && isTauriRuntime()) {
      const savedPath = await invoke<string>('copy_image_for_document', { sourcePath: path, documentPath: currentFilePath.value })
      const relativePath = getRelativePath(dirname(currentFilePath.value), savedPath)
      vditor.insertValue(`![${basename(savedPath)}](${relativePath})`)
    } else {
      const url = isTauriRuntime() ? convertFileSrc(path) : toFileUrl(path)
      vditor.insertValue(`![${basename(path)}](${url})`)
    }
  }
}

const onDragOver = (event: DragEvent) => {
  event.preventDefault()
  isDragging.value = true
}

const onDragLeave = () => {
  isDragging.value = false
}

const onDrop = async (event: DragEvent) => {
  event.preventDefault()
  isDragging.value = false
  const files = Array.from(event.dataTransfer?.files || [])
  const pathList = files.map((file) => (file as File & { path?: string }).path).filter((value): value is string => Boolean(value))
  if (pathList.length > 0) {
    await handleDroppedPaths(pathList)
    return
  }
  for (const file of files) {
    if (!file.type.startsWith('image/') || !vditor) continue
    const compressed = await compressImage(file)
    if (currentFilePath.value && isTauriRuntime()) {
      const buffer = await compressed.arrayBuffer()
      const savedPath = await invoke<string>('save_image_for_document', {
        fileName: file.name,
        bytes: Array.from(new Uint8Array(buffer)),
        documentPath: currentFilePath.value
      })
      const relativePath = getRelativePath(dirname(currentFilePath.value), savedPath)
      vditor.insertValue(`![${basename(savedPath)}](${relativePath})`)
      continue
    }
    const reader = new FileReader()
    reader.onload = (e) => vditor?.insertValue(`![${file.name}](${e.target?.result as string})`)
    reader.readAsDataURL(compressed)
  }
}

const cmdPaletteSelect = async (path: string) => {
  commandPaletteOpen.value = false
  await openFilePath(path)
}

const cmdPaletteConfirm = async () => {
  const file = commandPaletteFiles.value[commandPaletteIdx.value]
  if (file) await cmdPaletteSelect(file.path)
}

const handleKeydown = (event: KeyboardEvent) => {
  const mod = event.metaKey || event.ctrlKey
  if (!mod) return
  if (event.key.toLowerCase() === 's') {
    event.preventDefault()
    if (event.shiftKey) void saveAsDocument()
    else void saveDocument()
  }
  if (event.key.toLowerCase() === 'o') {
    event.preventDefault()
    void openFile()
  }
  if (event.key.toLowerCase() === 'n') {
    event.preventDefault()
    createNewDocument()
  }
  if (event.key.toLowerCase() === 'f') {
    event.preventDefault()
    searchPanelOpen.value = true
  }
  if (event.key.toLowerCase() === 'h' && event.shiftKey) {
    event.preventDefault()
    searchPanelOpen.value = true
  }
  if (event.key.toLowerCase() === 'p') {
    event.preventDefault()
    commandPaletteOpen.value = true
  }
}

const initVditor = async () => {
  const { default: VditorClass } = await import('vditor')
  vditor = new VditorClass('vditor', {
    cdn: `${import.meta.env.BASE_URL}vditor`,
    lang: 'zh_CN',
    width: '100%',
    height: '100%',
    tab: '\t',
    mode: 'sv',
    typewriterMode: true,
    counter: { enable: true, max: 999999 },
    cache: { enable: false, id: 'carbo-editor' },
    outline: { enable: true, position: 'right' },
    preview: { delay: 120 },
    upload: {
      handler: async (files: File[]) => {
        for (const file of files) {
          if (!file.type.startsWith('image/')) continue
          const compressed = await compressImage(file)
          if (currentFilePath.value && isTauriRuntime()) {
            const buffer = await compressed.arrayBuffer()
            const savedPath = await invoke<string>('save_image_for_document', {
              fileName: file.name,
              bytes: Array.from(new Uint8Array(buffer)),
              documentPath: currentFilePath.value
            })
            const relativePath = getRelativePath(dirname(currentFilePath.value), savedPath)
            vditor?.insertValue(`![${basename(savedPath)}](${relativePath})`)
          } else {
            const reader = new FileReader()
            reader.onload = (event) => vditor?.insertValue(`![${file.name}](${event.target?.result as string})`)
            reader.readAsDataURL(compressed)
          }
        }
        return null
      }
    },
    after: async () => {
      const draft = loadDraft() || defaultContent
      applyEditorContent(draft)
      isLoading.value = false
      const pending = consumePendingOpenPaths()
      if (pending && pending.length > 0) await handleDroppedPaths(pending)
      setTimeout(setupOutlineHighlight, 600)
    },
    input: (value: string) => {
      if (suspendInput) return
      wordCount.value = value.length
      saveDraft(value)
      saveStatus.value = currentFilePath.value ? 'idle' : 'idle'
      scheduleAutosave()
    }
  })
}

watch(commandPaletteOpen, async (open) => {
  if (open) {
    commandPaletteQuery.value = ''
    commandPaletteIdx.value = 0
    await nextTick()
    cmdInputRef.value?.focus()
  }
})

watch(commandPaletteQuery, () => {
  commandPaletteIdx.value = 0
})

watch(searchQuery, () => {
  searchMatches.value = []
  currentMatchIdx.value = -1
})

watch(searchPanelOpen, async (open) => {
  if (open) {
    await nextTick()
    document.querySelector<HTMLInputElement>('.search-input')?.focus()
  } else {
    searchMatches.value = []
    currentMatchIdx.value = -1
  }
})

watch(pendingOpenPaths, async (paths) => {
  if (!paths || paths.length === 0) return
  await handleDroppedPaths(paths)
  consumePendingOpenPaths()
})

onMounted(() => {
  initVditor()
  window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
  if (autosaveTimer) window.clearTimeout(autosaveTimer)
  outlineScrollCleanup?.()
  vditor?.destroy()
})
</script>

<style scoped>
.workbench {
  min-height: 100vh;
  display: grid;
  grid-template-columns: 280px 1fr;
  background: #f5f7fb;
  color: #111827;
}

.workbench.theme-dark {
  background: #0f172a;
  color: #e5e7eb;
}

.sidebar {
  border-right: 1px solid rgba(148, 163, 184, 0.18);
  padding: 18px 14px;
  background: rgba(255, 255, 255, 0.78);
  backdrop-filter: blur(14px);
  overflow: auto;
}

.theme-dark .sidebar,
.theme-dark .topbar,
.theme-dark .statusbar {
  background: rgba(15, 23, 42, 0.82);
}

.sidebar-top,
.topbar,
.statusbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.sidebar-section {
  margin-top: 20px;
}

.section-title {
  font-size: 12px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #64748b;
  margin-bottom: 10px;
}

.sidebar-item {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  padding: 10px 12px;
  margin-bottom: 8px;
  border: 0;
  border-radius: 14px;
  background: transparent;
  cursor: pointer;
  text-align: left;
}

.sidebar-item.active,
.sidebar-item:hover,
.toolbar-btn:hover,
.ghost-btn:hover {
  background: rgba(37, 99, 235, 0.08);
}

.sidebar-item span,
.doc-meta,
.empty-text,
.folder-path,
.statusbar {
  color: #64748b;
  font-size: 13px;
}

.editor-shell {
  position: relative;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.topbar {
  padding: 22px 28px 10px;
}

.statusbar {
  padding: 0 28px 14px;
  justify-content: flex-start;
  flex-wrap: wrap;
}

.editor-wrapper {
  flex: 1;
  padding: 0 20px 20px;
  min-height: 0;
}

.vditor-container {
  height: calc(100vh - 120px);
  border-radius: 24px;
  overflow: hidden;
  box-shadow: 0 20px 48px rgba(15, 23, 42, 0.08);
}

.doc-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.doc-title-row h1 {
  margin: 0;
  font-size: 26px;
}

.dirty-dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  background: #f59e0b;
  opacity: 0;
}

.dirty-dot.visible {
  opacity: 1;
}

.toolbar-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.toolbar-btn,
.ghost-btn {
  border: 0;
  border-radius: 999px;
  padding: 10px 14px;
  background: transparent;
  cursor: pointer;
  color: inherit;
}

.toolbar-btn.primary {
  background: #111827;
  color: #fff;
}

.theme-dark .toolbar-btn.primary {
  background: #e5e7eb;
  color: #111827;
}

.drop-overlay {
  position: absolute;
  inset: 0;
  z-index: 20;
  background: rgba(15, 23, 42, 0.12);
  display: grid;
  place-items: center;
}

.drop-card {
  padding: 22px 28px;
  border-radius: 20px;
  background: #fff;
  box-shadow: 0 20px 48px rgba(15, 23, 42, 0.12);
}

.width-narrow .editor-wrapper,
.width-narrow .topbar,
.width-narrow .statusbar {
  max-width: 980px;
}

.width-medium .editor-wrapper,
.width-medium .topbar,
.width-medium .statusbar {
  max-width: 1240px;
}

.width-wide .editor-wrapper,
.width-wide .topbar,
.width-wide .statusbar {
  max-width: 100%;
}

.sidebar-collapsed {
  grid-template-columns: 88px 1fr;
}

.focus-mode :deep(.vditor-preview > *:not(:hover)) {
  opacity: 0.72;
}

.theme-dark :deep(.vditor),
.theme-dark :deep(.vditor-toolbar),
.theme-dark :deep(.vditor-content),
.theme-dark :deep(.vditor-preview) {
  background: #111827 !important;
  color: #e5e7eb !important;
}

.search-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 0 28px 14px;
}

.search-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.search-count {
  font-size: 12px;
  color: #64748b;
  white-space: nowrap;
  min-width: 40px;
  text-align: center;
}

.icon-btn {
  padding: 8px 10px;
  font-size: 14px;
}

.search-input {
  width: 100%;
  border: 1px solid rgba(148, 163, 184, 0.28);
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.72);
  padding: 10px 12px;
  color: inherit;
}

@media (max-width: 960px) {
  .workbench {
    grid-template-columns: 1fr;
  }

  .sidebar {
    display: none;
  }

  .search-row {
    flex-wrap: wrap;
  }
}

/* Command Palette */
.cmd-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(15, 23, 42, 0.4);
  backdrop-filter: blur(4px);
  display: flex;
  justify-content: center;
  padding-top: 15vh;
}

.cmd-palette {
  width: 560px;
  max-width: calc(100vw - 40px);
  background: rgba(255, 255, 255, 0.96);
  backdrop-filter: blur(20px);
  border-radius: 20px;
  box-shadow: 0 24px 64px rgba(15, 23, 42, 0.2), 0 0 0 1px rgba(148, 163, 184, 0.2);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  max-height: 60vh;
}

.theme-dark .cmd-palette {
  background: rgba(15, 23, 42, 0.96);
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(148, 163, 184, 0.1);
}

.cmd-input {
  width: 100%;
  border: 0;
  border-bottom: 1px solid rgba(148, 163, 184, 0.2);
  background: transparent;
  padding: 16px 20px;
  font-size: 15px;
  color: inherit;
  outline: none;
  box-sizing: border-box;
}

.cmd-input::placeholder {
  color: #94a3b8;
}

.cmd-list {
  overflow-y: auto;
  flex: 1;
}

.cmd-empty {
  padding: 24px 20px;
  color: #94a3b8;
  font-size: 13px;
  text-align: center;
}

.cmd-item {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 20px;
  border: 0;
  background: transparent;
  cursor: pointer;
  text-align: left;
  color: inherit;
}

.cmd-item:hover,
.cmd-item.active {
  background: rgba(37, 99, 235, 0.1);
}

.theme-dark .cmd-item:hover,
.theme-dark .cmd-item.active {
  background: rgba(37, 99, 235, 0.18);
}

.cmd-item-name {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex-shrink: 0;
  max-width: 200px;
}

.cmd-item-path {
  font-size: 12px;
  color: #94a3b8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  direction: rtl;
  text-align: left;
}

:deep(.vditor-outline__item--current) {
  color: #2563eb !important;
  font-weight: 600;
  background: rgba(37, 99, 235, 0.08);
  border-radius: 6px;
}

.theme-dark :deep(.vditor-outline__item--current) {
  color: #60a5fa !important;
  background: rgba(96, 165, 250, 0.12);
}

.cmd-footer {
  display: flex;
  gap: 16px;
  padding: 10px 20px;
  border-top: 1px solid rgba(148, 163, 184, 0.2);
  font-size: 12px;
  color: #94a3b8;
}
</style>
