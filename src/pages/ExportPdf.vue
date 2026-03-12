<template>
  <div class="export-page">
    <div class="export-header no-print">
      <router-link to="/" class="back-btn">← 返回</router-link>
      <div>
        <h1>导出 PDF</h1>
        <p>使用打印式排版导出，文字可选中，分页更稳定。</p>
      </div>
    </div>

    <div class="export-actions no-print">
      <el-button @click="saveHtml">保存 HTML</el-button>
      <el-button type="primary" @click="printPdf">打印 / 导出 PDF</el-button>
    </div>

    <article class="preview-area markdown-body" ref="previewRef" v-html="htmlContent"></article>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { useRoute } from 'vue-router'
import Vditor from 'vditor'
import { invoke, isTauri } from '@tauri-apps/api/core'
import type { DialogFilter, SaveDialogOptions } from '@tauri-apps/plugin-dialog'

const STORAGE_KEY = 'carbo-markdown-content'
const route = useRoute()
const previewRef = ref<HTMLElement | null>(null)
const htmlContent = ref('')

const title = computed(() => (route.query.name as string) || 'carbo-export')

const buildHtmlDocument = (bodyHtml: string) => `<!DOCTYPE html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>${title.value}</title>
    <style>
      :root { color-scheme: light; }
      @page { size: A4; margin: 18mm 16mm; }
      * { box-sizing: border-box; }
      body { margin: 0; color: #111827; font: 16px/1.75 -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; }
      article { max-width: 860px; margin: 0 auto; padding: 24px 20px 48px; }
      h1, h2, h3, h4 { line-height: 1.28; break-after: avoid-page; }
      p, li, blockquote { orphans: 3; widows: 3; }
      pre, blockquote, table, img { break-inside: avoid; page-break-inside: avoid; }
      img { max-width: 100%; height: auto; }
      pre { padding: 14px 16px; border-radius: 12px; background: #f3f4f6; overflow: auto; }
      code { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
      table { width: 100%; border-collapse: collapse; }
      th, td { border: 1px solid #d1d5db; padding: 8px 10px; }
      blockquote { margin: 0; padding-left: 16px; border-left: 4px solid #d1d5db; color: #4b5563; }
      hr { border: 0; border-top: 1px solid #e5e7eb; margin: 24px 0; }
      a { color: #2563eb; }
    </style>
  </head>
  <body>
    <article class="markdown-body">${bodyHtml}</article>
  </body>
</html>`

const pickSavePath = async (options: SaveDialogOptions) => {
  const { save } = await import('@tauri-apps/plugin-dialog')
  return await save(options)
}

const saveExportBytes = async (params: { title: string; fileName: string; filters: DialogFilter[]; bytes: number[] }) => {
  if (!isTauri()) return null
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

const saveHtml = async () => {
  try {
    const html = buildHtmlDocument(htmlContent.value)
    const fileName = `${title.value}.html`
    const bytes = Array.from(new TextEncoder().encode(html))

    if (isTauri()) {
      const savedPath = await saveExportBytes({
        title: '导出 HTML',
        fileName,
        filters: [{ name: 'HTML', extensions: ['html'] }],
        bytes
      })
      if (savedPath) ElMessage.success(`已保存: ${savedPath}`)
      return
    }

    const blob = new Blob([html], { type: 'text/html;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = fileName
    link.click()
    URL.revokeObjectURL(url)
  } catch (error) {
    ElMessage.error(`保存失败: ${String(error)}`)
  }
}

const printPdf = async () => {
  await nextTick()
  window.print()
}

onMounted(async () => {
  const markdown = localStorage.getItem(STORAGE_KEY) || ''
  htmlContent.value = await Vditor.md2html(markdown)

  if (route.query.auto === '1') {
    setTimeout(() => {
      void printPdf()
    }, 80)
  }
})
</script>

<style scoped>
.export-page {
  min-height: 100vh;
  background: #f5f7fb;
  padding: 24px;
}

.export-header,
.export-actions,
.preview-area {
  max-width: 920px;
  margin: 0 auto;
}

.export-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
}

.export-header h1 {
  margin: 0;
  font-size: 28px;
}

.export-header p {
  margin: 4px 0 0;
  color: #6b7280;
}

.export-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-bottom: 16px;
}

.preview-area {
  background: #fff;
  border-radius: 20px;
  box-shadow: 0 20px 48px rgba(15, 23, 42, 0.08);
  padding: 40px 44px;
}

.back-btn {
  color: #4b5563;
}

@media print {
  .no-print {
    display: none !important;
  }

  .export-page {
    padding: 0;
    background: #fff;
  }

  .preview-area {
    max-width: none;
    padding: 0;
    box-shadow: none;
    border-radius: 0;
  }
}
</style>
