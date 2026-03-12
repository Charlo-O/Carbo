import { basename, dirname } from './path'

export type RecentItem = {
  path: string
  name: string
  parent: string
  openedAt: number
}

export type FolderEntry = {
  path: string
  name: string
}

export type SaveStatus = 'idle' | 'saving' | 'saved' | 'error'

const RECENT_FILES_KEY = 'carbo-recent-files'
const RECENT_PROJECTS_KEY = 'carbo-recent-projects'
const DRAFT_KEY = 'carbo-markdown-content'
const MAX_RECENTS = 12

const readList = (key: string): RecentItem[] => {
  try {
    const raw = localStorage.getItem(key)
    if (!raw) return []
    const parsed = JSON.parse(raw) as unknown
    if (!Array.isArray(parsed)) return []
    return parsed.filter((item): item is RecentItem => Boolean(item && typeof item === 'object' && typeof (item as RecentItem).path === 'string'))
  } catch {
    return []
  }
}

const writeList = (key: string, items: RecentItem[]) => {
  localStorage.setItem(key, JSON.stringify(items.slice(0, MAX_RECENTS)))
}

export const loadRecentFiles = () => readList(RECENT_FILES_KEY)
export const loadRecentProjects = () => readList(RECENT_PROJECTS_KEY)

export const pushRecentFile = (path: string) => {
  const item: RecentItem = {
    path,
    name: basename(path),
    parent: dirname(path),
    openedAt: Date.now()
  }
  const next = [item, ...loadRecentFiles().filter((entry) => entry.path !== path)]
  writeList(RECENT_FILES_KEY, next)
  return next.slice(0, MAX_RECENTS)
}

export const pushRecentProject = (path: string) => {
  const cleanPath = path.replace(/\\/g, '/')
  const item: RecentItem = {
    path: cleanPath,
    name: basename(cleanPath),
    parent: dirname(cleanPath),
    openedAt: Date.now()
  }
  const next = [item, ...loadRecentProjects().filter((entry) => entry.path !== cleanPath)]
  writeList(RECENT_PROJECTS_KEY, next)
  return next.slice(0, MAX_RECENTS)
}

export const loadDraft = () => localStorage.getItem(DRAFT_KEY) || ''
export const saveDraft = (value: string) => localStorage.setItem(DRAFT_KEY, value)
export const clearDraft = () => localStorage.removeItem(DRAFT_KEY)
