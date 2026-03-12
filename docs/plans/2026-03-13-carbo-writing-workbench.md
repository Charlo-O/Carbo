# Carbo Writing Workbench Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade Carbo from a localStorage-first Markdown MVP into a file-oriented writing workbench with a reliable save model, recent items, lightweight workspace sidebar, and better PDF export.

**Architecture:** Add missing filesystem commands in the Tauri backend, then centralize document/workspace state in the main editor flow so the UI can expose file identity, autosave, recents, and sidebar navigation without changing the app’s lightweight feel. Keep P0 in the main route, and upgrade PDF export to a print-style document path instead of screenshot capture.

**Tech Stack:** Vue 3, Vditor, Element Plus, Tauri 2, Rust backend commands, TypeScript utilities.

---

### Task 1: Backend filesystem primitives

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Steps:**
1. Add path helper utilities for basename, dirname, extension checks, sibling asset paths, and directory listing.
2. Add Tauri commands for opening/saving text files, save-as, listing markdown files in a folder, opening a folder context, and copying/saving images beside the current document.
3. Add a PDF export command that writes HTML to a temp file and invokes a document-style print/export flow where feasible, with a clean fallback.
4. Register all new commands in `invoke_handler`.
5. Keep existing commands intact for backward compatibility.

### Task 2: Frontend document/workspace state

**Files:**
- Create: `src/utils/path.ts`
- Create: `src/utils/workbench.ts`
- Modify: `src/utils/openPaths.ts`
- Modify: `src/pages/Main.vue`

**Steps:**
1. Add path normalization helpers for display names, parent folders, relative paths, and recent item serialization.
2. Add a small workbench state layer for `currentFilePath`, `isDirty`, `saveStatus`, `recentFiles`, `recentProjects`, `openedFolderPath`, and folder file entries.
3. Migrate editor initialization from localStorage-first to file-first with local draft fallback only when no current file exists.
4. Wire pending open paths from Tauri into the new document opening flow.
5. Ensure autosave writes back to the active file after debounce and updates save status.

### Task 3: Main workbench UI

**Files:**
- Modify: `src/pages/Main.vue`
- Optionally create: `src/components/*` if the single file becomes too large

**Steps:**
1. Add a lightweight, collapsible sidebar for recent files, recent projects, and folder markdown entries.
2. Add a top/status area that shows current document name/path, dirty state, save status, and quick actions.
3. Add commands and shortcuts for New/Open/Open Folder/Save/Save As.
4. Keep drag-and-drop open working and route dropped files through the new document workflow.
5. Preserve existing image bed and export affordances while fitting them into the new workbench structure.

### Task 4: PDF + image workflow upgrades

**Files:**
- Modify: `src/pages/ExportPdf.vue`
- Modify: `src/pages/Main.vue`
- Modify: `src-tauri/src/lib.rs`

**Steps:**
1. Replace screenshot PDF export with a print/document-oriented HTML export path.
2. Keep manual save dialogs in Tauri and a browser fallback for web mode.
3. Update image insertion so local images can be stored beside the current file in an `assets/`-style directory and inserted as relative paths when possible.
4. Preserve GitHub image bed support as an optional post-processing path.

### Task 5: P1 ergonomics and verification

**Files:**
- Modify: `src/pages/Main.vue`
- Modify: any touched utility files

**Steps:**
1. Add the lowest-cost writing improvements: focus mode, width presets, quick search hook, clickable outline access if Vditor already exposes it.
2. Run `npm install` if needed, then `npm run build`.
3. Fix compile/type issues from the refactor.
4. Summarize completed work, partial work, and next steps.
