<template>
  <div class="export-page">
    <div class="export-header">
      <router-link to="/" class="back-btn">← 返回</router-link>
      <h1>导出 PDF</h1>
    </div>
    <div class="export-content">
      <div class="preview-area" ref="previewRef">
        <div v-html="htmlContent" class="markdown-body"></div>
      </div>
      <div class="export-actions">
        <el-button type="primary" @click="exportPdf">导出 PDF</el-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import Vditor from 'vditor'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { useRoute } from 'vue-router'
import type { DialogFilter, SaveDialogOptions } from '@tauri-apps/plugin-dialog'

const previewRef = ref<HTMLElement | null>(null)
const htmlContent = ref('')

const STORAGE_KEY = 'carbo-markdown-content'

const route = useRoute()

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

  if (route.query.auto === '1') {
    await nextTick()
    // Allow DOM + images to settle a bit for canvas capture.
    setTimeout(() => {
      void exportPdf()
    }, 60)
  }
})

const exportPdf = async () => {
  if (!previewRef.value) return

  try {
    const html2canvas = (await import('html2canvas')).default
    const { jsPDF } = await import('jspdf')

    const canvas = await html2canvas(previewRef.value, {
      backgroundColor: '#ffffff',
      scale: 2,
      useCORS: true,
      allowTaint: true
    })

    const width = canvas.width
    const height = canvas.height
    const orientation: 'portrait' | 'landscape' = width > height ? 'landscape' : 'portrait'

    const pdf = new jsPDF({
      orientation,
      unit: 'px',
      format: [width, height]
    })
    const imgData = canvas.toDataURL('image/jpeg', 0.92)
    if (!imgData.startsWith('data:image/jpeg')) {
      throw new Error('Failed to build JPEG data URL')
    }
    pdf.addImage(imgData, 'JPEG', 0, 0, width, height)

    const fileName = 'carbo-export.pdf'

    if (isTauri()) {
      const bytes = Array.from(new Uint8Array(pdf.output('arraybuffer')))
      const savedPath = await saveExportBytes({
        title: '导出 PDF',
        fileName,
        filters: [{ name: 'PDF', extensions: ['pdf'] }],
        bytes
      })
      if (savedPath) ElMessage.success(`已导出: ${savedPath}`)
      return
    }

    pdf.save(fileName)
    ElMessage.success('PDF 导出成功')
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

@media print {
  .export-header,
  .export-actions {
    display: none;
  }
  
  .export-page {
    padding: 0;
    background: white;
  }
  
  .preview-area {
    box-shadow: none;
    border-radius: 0;
  }
}
</style>
