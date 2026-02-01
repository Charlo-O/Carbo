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
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import Vditor from 'vditor'

const previewRef = ref<HTMLElement | null>(null)
const htmlContent = ref('')

const STORAGE_KEY = 'carbo-markdown-content'

onMounted(async () => {
  const markdown = localStorage.getItem(STORAGE_KEY) || ''
  htmlContent.value = await Vditor.md2html(markdown)
})

const exportPdf = () => {
  try {
    window.print()
    ElMessage.success('请在打印对话框中选择"另存为 PDF"')
  } catch (error) {
    ElMessage.error('导出失败')
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
