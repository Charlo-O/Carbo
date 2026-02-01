<template>
  <div class="about-page">
    <div class="about-content">
      <div class="about-header">
        <span class="about-icon">📝</span>
        <h1>Carbo Markdown Editor</h1>
        <p class="version">v1.0.0</p>
      </div>

      <div class="about-section">
        <h2>关于</h2>
        <p>Carbo 是一款基于 Vue 3 和 Vditor 构建的桌面 Markdown 编辑器，采用极简的 AiChat 风格设计。</p>
      </div>

      <div class="about-section">
        <h2>功能特性</h2>
        <ul>
          <li>✨ 实时预览 Markdown 内容</li>
          <li>📊 支持流程图、甘特图、时序图</li>
          <li>🎵 支持视频、音频、五线谱</li>
          <li>💾 自动保存到本地存储</li>
          <li>📤 导出 PDF、PNG 等格式</li>
        </ul>
      </div>

      <div class="about-section">
        <h2>技术栈</h2>
        <div class="tech-tags">
          <span class="tag">Vue 3</span>
          <span class="tag">TypeScript</span>
          <span class="tag">Vite</span>
          <span class="tag">Tauri</span>
          <span class="tag">Vditor</span>
          <span class="tag">Element Plus</span>
        </div>
      </div>

      <div class="about-footer">
        <router-link to="/" class="back-btn">← 返回编辑器</router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// About page component
</script>

<style scoped>
.about-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-bg-secondary);
  padding: var(--space-6);
}

.about-content {
  max-width: 480px;
  width: 100%;
  background-color: var(--color-bg-primary);
  border-radius: var(--radius-xl);
  padding: var(--space-8);
  box-shadow: var(--shadow-lg);
}

.about-header {
  text-align: center;
  margin-bottom: var(--space-8);
}

.about-icon {
  font-size: 48px;
  display: block;
  margin-bottom: var(--space-3);
}

.about-header h1 {
  font-size: var(--text-2xl);
  font-weight: var(--font-weight-bold);
  color: var(--color-text-primary);
  margin-bottom: var(--space-2);
}

.version {
  color: var(--color-text-tertiary);
  font-size: var(--text-sm);
}

.about-section {
  margin-bottom: var(--space-6);
}

.about-section h2 {
  font-size: var(--text-base);
  font-weight: var(--font-weight-semibold);
  color: var(--color-text-primary);
  margin-bottom: var(--space-3);
}

.about-section p,
.about-section li {
  color: var(--color-text-secondary);
  font-size: var(--text-sm);
  line-height: var(--line-height-relaxed);
}

.about-section ul {
  list-style: none;
}

.about-section li {
  padding: var(--space-1) 0;
}

.tech-tags {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.tag {
  padding: var(--space-1) var(--space-3);
  background-color: var(--color-accent);
  color: var(--color-text-inverse);
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  font-weight: var(--font-weight-medium);
}

.about-footer {
  text-align: center;
  padding-top: var(--space-6);
  border-top: 1px solid var(--color-border);
}

.back-btn {
  color: var(--color-text-secondary);
  font-size: var(--text-sm);
  transition: color var(--transition-fast);
}

.back-btn:hover {
  color: var(--color-text-primary);
}
</style>
