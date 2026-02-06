<template>
  <header class="header-nav">
    <div class="header-content">
      <!-- Logo -->
      <div class="header-logo">
        <router-link to="/" class="logo-link">
          <span class="logo-icon">📝</span>
          <span class="logo-text">Arya</span>
        </router-link>
      </div>

      <!-- Actions -->
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
              <el-dropdown-item divided command="about">关于</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'

const router = useRouter()

// Handle export actions
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
      copyHtmlToClipboard()
      break
  }
}

// Handle settings actions
const handleSetting = async (command: string) => {
  switch (command) {
    case 'import':
      importFile()
      break
    case 'clear':
      await clearContent()
      break
    case 'about':
      router.push('/about')
      break
  }
}

// Copy HTML to clipboard (for WeChat)
const copyHtmlToClipboard = () => {
  const content = localStorage.getItem('carbo-markdown-content') || ''
  if (content) {
    navigator.clipboard.writeText(content)
    ElMessage.success('内容已复制到剪贴板')
  }
}

// Import markdown file
const importFile = () => {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.md,.markdown,.txt'
  input.onchange = (e: Event) => {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (file) {
      const reader = new FileReader()
      reader.onload = (event) => {
        const content = event.target?.result as string
        localStorage.setItem('carbo-markdown-content', content)
        window.location.reload()
      }
      reader.readAsText(file)
    }
  }
  input.click()
}

// Clear content with confirmation
const clearContent = async () => {
  try {
    await ElMessageBox.confirm('确定要清空所有内容吗？此操作不可撤销。', '提示', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    localStorage.removeItem('carbo-markdown-content')
    window.location.reload()
  } catch {
    // User cancelled
  }
}
</script>

<style scoped>
.header-nav {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: var(--header-height);
  background-color: var(--color-bg-primary);
  border-bottom: 1px solid var(--color-border);
  z-index: 100;
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
  letter-spacing: -0.01em;
}

.logo-link:hover {
  color: var(--color-text-primary);
}

.logo-icon {
  font-size: 20px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.action-btn {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  padding: 8px 12px;
  background-color: transparent;
  color: var(--color-text-secondary);
  border-radius: var(--radius-md);
  font-size: 14px;
  font-weight: 500;
  transition: all var(--transition-fast);
}

.action-btn:hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.action-btn.icon-btn {
  padding: 8px;
  font-size: 16px;
}

.dropdown-icon {
  font-size: 10px;
  opacity: 0.5;
  margin-left: 2px;
}
</style>
