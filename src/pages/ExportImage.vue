<template>
  <div class="export-page">
    <div class="export-header">
      <router-link to="/" class="back-btn">← 返回</router-link>
      <h1>导出图片</h1>
    </div>
    <div class="export-content">
      <div class="preview-area" ref="previewRef">
        <div v-html="htmlContent" class="markdown-body"></div>
      </div>
      <div class="export-actions">
        <el-button type="primary" @click="exportImage('png')">导出 PNG</el-button>
        <el-button @click="exportImage('jpeg')">导出 JPEG</el-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import Vditor from 'vditor'
import { invoke, isTauri } from '@tauri-apps/api/core'
import type { DialogFilter, SaveDialogOptions } from '@tauri-apps/plugin-dialog'

const previewRef = ref<HTMLElement | null>(null)
const htmlContent = ref('')

const STORAGE_KEY = 'carbo-markdown-content'

const canvasToBlob = (canvas: HTMLCanvasElement, mimeType: string, quality?: number) =>
  new Promise<Blob>((resolve, reject) => {
    canvas.toBlob(
      (blob) => {
        if (!blob) {
          reject(new Error('Failed to build image blob'))
          return
        }
        resolve(blob)
      },
      mimeType,
      quality
    )
  })

const pickSavePath = async (options: SaveDialogOptions) => {
  const { save } = await import('@tauri-apps/plugin-dialog')
  return await save(options)
}

const saveExportBytes = async (params: {
  title: string
  fileName: string
  filters: DialogFilter[]
  bytes: number[]
}) => {
  const filePath = await pickSavePath({
    title: params.title,
    defaultPath: params.fileName,
    filters: params.filters
  })
  if (!filePath) return null
  return await invoke<string>('save_export_bytes', {
    fileName: params.fileName,
    filePath,
    bytes: params.bytes
  })
}

onMounted(async () => {
  const markdown = localStorage.getItem(STORAGE_KEY) || ''
  htmlContent.value = await Vditor.md2html(markdown)
})

const exportImage = async (format: 'png' | 'jpeg') => {
  if (!previewRef.value) return
  
  try {
    const html2canvas = (await import('html2canvas')).default
    const canvas = await html2canvas(previewRef.value, {
      backgroundColor: '#ffffff',
      scale: 2,
      useCORS: true,
      allowTaint: true
    })

    const fileName = `carbo-export.${format === 'jpeg' ? 'jpg' : format}`

    const blob = await canvasToBlob(canvas, format === 'png' ? 'image/png' : 'image/jpeg', 0.9)
    const bytes = Array.from(new Uint8Array(await blob.arrayBuffer()))

    if (isTauri()) {
      const savedPath = await saveExportBytes({
        title: format === 'png' ? '导出 PNG' : '导出 JPEG',
        fileName,
        filters: [{ name: format === 'png' ? 'PNG' : 'JPEG', extensions: format === 'png' ? ['png'] : ['jpg', 'jpeg'] }],
        bytes
      })
      if (savedPath) ElMessage.success(`已导出: ${savedPath}`)
      return
    }
    
    const link = document.createElement('a')
    link.download = fileName
    link.href = canvas.toDataURL(`image/${format}`, 0.9)
    link.click()
    
    ElMessage.success('图片导出成功')
  } catch (error) {
    ElMessage.error(`导出失败: ${String(error)}`)
    console.error(error)
  }
}
</script>

<style scoped>
.export-page {
  min-height: 100vh;
  background-color: var(--color-bg-secondary);
  padding: var(--space-6);
}

.export-header {
  max-width: 800px;
  margin: 0 auto var(--space-6);
  display: flex;
  align-items: center;
  gap: var(--space-4);
}

.export-header h1 {
  font-size: var(--text-xl);
  font-weight: var(--font-weight-semibold);
}

.back-btn {
  color: var(--color-text-secondary);
}

.export-content {
  max-width: 800px;
  margin: 0 auto;
}

.preview-area {
  background-color: var(--color-bg-primary);
  border-radius: var(--radius-lg);
  padding: var(--space-8);
  margin-bottom: var(--space-4);
  box-shadow: var(--shadow-md);
}

.export-actions {
  display: flex;
  gap: var(--space-3);
  justify-content: center;
}
</style>
