<template>
  <div class="export-page ppt-page">
    <div class="export-header">
      <router-link to="/" class="back-btn">← 返回</router-link>
      <h1>PPT 预览</h1>
      <p class="hint">使用 --- 分隔幻灯片</p>
    </div>
    <div class="ppt-container" ref="revealRef">
      <div class="reveal">
        <div class="slides" v-html="slidesHtml"></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import Vditor from 'vditor'

const revealRef = ref<HTMLElement | null>(null)
const slidesHtml = ref('')

const STORAGE_KEY = 'carbo-markdown-content'

onMounted(async () => {
  const markdown = localStorage.getItem(STORAGE_KEY) || ''
  
  // Split by --- for horizontal slides
  const slides = markdown.split(/^---$/m)
  
  let html = ''
  for (const slide of slides) {
    if (slide.trim()) {
      const slideHtml = await Vditor.md2html(slide.trim())
      html += `<section>${slideHtml}</section>`
    }
  }
  
  slidesHtml.value = html
  
  // Initialize Reveal.js if available
  try {
    const Reveal = (await import('reveal.js')).default
    const deck = new Reveal(revealRef.value?.querySelector('.reveal') as HTMLElement, {
      hash: true,
      transition: 'slide'
    })
    deck.initialize()
  } catch (e) {
    console.log('Reveal.js not available, showing static slides')
  }
})
</script>

<style scoped>
.ppt-page {
  background-color: var(--color-bg-primary);
}

.export-header {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  padding: var(--space-4);
  background: var(--color-bg-primary);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  gap: var(--space-4);
  z-index: 100;
}

.export-header h1 {
  font-size: var(--text-lg);
  font-weight: var(--font-weight-semibold);
}

.hint {
  color: var(--color-text-tertiary);
  font-size: var(--text-sm);
  margin-left: auto;
}

.back-btn {
  color: var(--color-text-secondary);
}

.ppt-container {
  padding-top: 60px;
  height: 100vh;
}

.reveal {
  height: 100%;
}

:deep(.slides section) {
  background: var(--color-bg-primary);
  padding: var(--space-8);
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  text-align: center;
}
</style>
