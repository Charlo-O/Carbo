<p align="center">
  <img width="80" src="src-tauri/icons/128x128.png" alt="Carbo Logo">
</p>

<h1 align="center">Carbo</h1>

<p align="center">
  一款轻量级桌面 Markdown 编辑器，采用极简黑白设计
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Vue-3.4-4FC08D?logo=vue.js" alt="Vue 3">
  <img src="https://img.shields.io/badge/Tauri-2.0-FFC131?logo=tauri" alt="Tauri 2">
  <img src="https://img.shields.io/badge/TypeScript-5.x-3178C6?logo=typescript" alt="TypeScript">
  <img src="https://img.shields.io/badge/Vite-5.x-646CFF?logo=vite" alt="Vite">
</p>

---

## 更新日志

### v1.1.0

- 简化顶部布局，移除独立 Header 栏
- 导出功能统一到 Vditor 工具栏「更多 ···」→「导出」
- 设置按钮移至底部状态栏，位于字数统计左侧
- 窗口标题简化为「Carbo」


## 功能特性

- **实时预览** — 编辑即预览，所见即所得
- **流程图支持** — 流程图、甘特图、时序图、Echarts 图表
- **多媒体支持** — 视频、音频、五线谱渲染
- **拖放打开** — 拖放 Markdown 文件直接编辑
- **本地图片** — 拖放/粘贴图片自动保存到本地
- **GitHub 图床** — 可选配置 GitHub 仓库作为图床上传
- **自动保存** — 编辑内容自动保存，防止丢失
- **多格式导出** — 导出为 PDF、PNG 等格式
- **PPT 预览** — 使用 `---` 分隔幻灯片

## 快速开始

### 环境要求

- Node.js >= 18
- Rust >= 1.70
- npm / pnpm / yarn

### 开发

```bash
# 安装依赖
npm install

# 启动开发模式（Tauri 桌面应用）
npm run tauri:dev

# 仅启动前端
npm run dev
```

### 构建

```bash
npm run tauri:build
```

## 使用说明

| 操作 | 方法 |
|------|------|
| 打开文件 | 拖放 `.md` / `.txt` 文件到窗口 |
| 插入图片 | 拖放或粘贴图片 |
| 导出文档 | 工具栏「更多 ···」→「导出」 |
| PPT 预览 | 底部设置菜单 ⚙ → PPT 预览 |
| 图床设置 | 底部设置菜单 ⚙ → 图床设置 |

### 快捷键

- `Ctrl/⌘ + Alt + 7` — 所见即所得模式
- `Ctrl/⌘ + Alt + 8` — 即时渲染模式
- `Ctrl/⌘ + Alt + 9` — 分屏渲染模式

## 技术栈

| 技术 | 用途 |
|------|------|
| Vue 3 | 前端框架 |
| Tauri 2 | 桌面应用框架 |
| Vditor | Markdown 编辑器 |
| Element Plus | UI 组件 |
| TypeScript | 类型安全 |
| Vite | 构建工具 |

## 项目结构

```
├── src/                  # 前端源码
│   ├── pages/           # 页面组件
│   ├── components/      # 公共组件
│   └── assets/          # 静态资源
├── src-tauri/            # Tauri 后端
│   ├── src/             # Rust 源码
│   ├── icons/           # 应用图标
│   └── tauri.conf.json  # Tauri 配置
└── package.json
```

## License

MIT
