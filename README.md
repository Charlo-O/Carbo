<p align="center"><a href="https://www.niceshare.site/?ref=github.com" target="_blank"><img width="100"src="https://cdn.jsdelivr.net/gh/nicejade/markdown-online-editor/src/assets/images/logo.png"></a></p>

<h1 align="center">
  Carbo - Markdown 桌面编辑器
</h1>

<div align="center">
  <strong>
    📝 基于 <a href="https://vuejs.org/">Vue 3</a>、<a href="https://tauri.app/">Tauri</a>、<a href="https://github.com/Vanessa219/vditor">Vditor</a> 构建的桌面 Markdown 编辑器，采用极简 AiChat 风格设计。支持绘制流程图、甘特图、时序图、任务列表、Echarts 图表、五线谱，以及 PPT 预览、视频音频解析、本地图片插入等功能。
  </strong>
</div>

<br>

<div align="center">
  <a href="https://github.com/nicejade/markdown-online-editor">
    <img alt="GitHub package.json version" src="https://img.shields.io/github/package-json/v/nicejade/markdown-online-editor">
  </a>
  <a href="https://github.com/nicejade/markdown-online-editor">
    <img src="https://img.shields.io/github/license/nicejade/markdown-online-editor.svg" alt="LICENSE">
  </a>
  <img src="https://img.shields.io/badge/Vue-3.x-4FC08D?logo=vue.js" alt="Vue 3">
  <img src="https://img.shields.io/badge/Tauri-2.x-FFC131?logo=tauri" alt="Tauri 2">
  <img src="https://img.shields.io/badge/TypeScript-5.x-3178C6?logo=typescript" alt="TypeScript">
  <a href="https://niceshare.site/?ref=github.com">
    <img src="https://img.shields.io/badge/Author-nicejade-%23a696c8.svg" alt="Author nicejade">
  </a>
</div>

## ✨ 特性

- 🖥️ **桌面应用** - 基于 Tauri 2.x，跨平台支持 Windows/macOS/Linux
- 🎨 **AiChat 风格** - 白色主调、黑色强调的极简设计
- 📁 **拖放打开** - 直接拖放 Markdown 文件到窗口即可编辑
- 🖼️ **本地图片** - 支持拖放/粘贴图片，自动保存到本地
- 💾 **自动保存** - 编辑内容自动保存，防止意外丢失
- ⚡ **Vite 构建** - 快速的开发体验和构建速度

## 🚀 功能支持

- [x] 🎉 通常 `Markdown` 解析器自带的基本功能
- [x] 🍀 支持**流程图**、**甘特图**、**时序图**、**任务列表**
- [x] 🏁 支持粘贴 HTML 自动转换为 Markdown
- [x] 💃🏻 支持插入原生 Emoji、设置常用表情列表
- [x] 🚑 支持编辑内容保存**本地存储**，防止意外丢失
- [x] 📝 支持**实时预览**，主窗口大小拖拽，字符计数
- [x] 🛠 支持常用快捷键(**Tab**)，及代码块添加复制
- [x] ✨ 支持**导出** PDF、PNG、JPEG 等格式
- [x] ✨ 支持 `echarts` 图表渲染
- [x] 👏 注入 [RevealJs](https://revealjs.com/)，支持 `PPT` 预览
- [x] 🦑 支持五线谱、视频、音频解析
- [x] 🌟 支持**所见即所得**编辑模式
- [x] 📂 支持拖放本地 Markdown 文件直接打开
- [x] 🖼️ 支持拖放/粘贴本地图片自动保存

## 📦 技术栈

| 技术 | 版本 | 说明 |
|------|------|------|
| Vue | 3.4.x | 前端框架（Composition API） |
| TypeScript | 5.4.x | 类型安全 |
| Vite | 5.x | 构建工具 |
| Tauri | 2.x | 桌面应用框架 |
| Element Plus | 2.5.x | UI 组件库 |
| Vditor | 3.10.x | Markdown 编辑器 |

## 🔧 开发指南

### 先决条件

- [Node.js](https://nodejs.org/) >= 18.x
- [Rust](https://rustup.rs/) >= 1.70（用于 Tauri 编译）
- [pnpm](https://pnpm.io/) 或 [yarn](https://yarnpkg.com/)

### 快速开始

```bash
# 克隆项目
git clone https://github.com/nicejade/markdown-online-editor.git
cd markdown-online-editor

# 安装依赖
npm install

# 启动前端开发服务器（仅 Web）
npm run dev

# 启动 Tauri 桌面应用（需要 Rust 环境）
npm run tauri:dev

# 构建生产版本
npm run tauri:build
```

### 项目结构

```
├── src/                    # 前端源码
│   ├── main.ts            # 入口文件
│   ├── App.vue            # 根组件
│   ├── router/            # 路由配置
│   ├── pages/             # 页面组件
│   ├── components/        # 公共组件
│   └── assets/            # 静态资源
├── src-tauri/             # Tauri 后端
│   ├── src/               # Rust 源码
│   ├── tauri.conf.json    # Tauri 配置
│   └── Cargo.toml         # Rust 依赖
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## 📖 使用说明

### 基本操作

1. **打开文件** - 拖放 `.md` / `.txt` 文件到窗口
2. **插入图片** - 拖放或粘贴图片，自动保存到本地
3. **导出文档** - 点击顶部「导出」按钮选择格式
4. **PPT 预览** - 使用 `---` 分隔幻灯片

### 编辑模式切换

- 所见即所得：`Ctrl + Alt + 7`（`⌘ + ⌥ + 7`）
- 即时渲染：`Ctrl + Alt + 8`（`⌘ + ⌥ + 8`）
- 分屏渲染：`Ctrl + Alt + 9`（`⌘ + ⌥ + 9`）

## 🙏 特别鸣谢

- [Vditor](https://github.com/Vanessa219/vditor) - 强大的 Markdown 编辑器
- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Vue 3](https://vuejs.org/) - 渐进式前端框架
- [Element Plus](https://element-plus.org/) - Vue 3 UI 组件库


