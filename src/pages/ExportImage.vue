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

const previewRef = ref<HTMLElement | null>(null)
const htmlContent = ref('')

const STORAGE_KEY = 'carbo-markdown-content'

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
      scale: 2
    })
    
    const link = document.createElement('a')
    link.download = `carbo-export.${format}`
    link.href = canvas.toDataURL(`image/${format}`, 0.9)
    link.click()
    
    ElMessage.success('图片导出成功')
  } catch (error) {
    ElMessage.error('导出失败，请重试')
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
