export const normalizePath = (value: string) => value.replace(/\\/g, '/')

export const basename = (filePath: string) => {
  const normalized = normalizePath(filePath)
  const parts = normalized.split('/').filter(Boolean)
  return parts[parts.length - 1] || filePath
}

export const dirname = (filePath: string) => {
  const normalized = normalizePath(filePath)
  const index = normalized.lastIndexOf('/')
  if (index <= 0) return ''
  return normalized.slice(0, index)
}

export const extname = (filePath: string) => {
  const name = basename(filePath)
  const index = name.lastIndexOf('.')
  return index >= 0 ? name.slice(index).toLowerCase() : ''
}

export const isMarkdownPath = (filePath: string) => ['.md', '.markdown', '.txt'].includes(extname(filePath))

export const isImagePath = (filePath: string) => /\.(png|jpe?g|gif|webp|svg|bmp|ico)$/i.test(filePath)

export const toFileUrl = (filePath: string) => {
  const path = normalizePath(filePath)
  if (/^[a-zA-Z]:\//.test(path)) return `file:///${encodeURI(path)}`
  if (path.startsWith('/')) return `file://${encodeURI(path)}`
  return filePath
}

export const getRelativePath = (fromDir: string, targetPath: string) => {
  const fromParts = normalizePath(fromDir).split('/').filter(Boolean)
  const targetParts = normalizePath(targetPath).split('/').filter(Boolean)

  while (fromParts.length > 0 && targetParts.length > 0 && fromParts[0] === targetParts[0]) {
    fromParts.shift()
    targetParts.shift()
  }

  const upward = fromParts.map(() => '..')
  const combined = [...upward, ...targetParts]
  return combined.join('/') || '.'
}

export const truncateMiddle = (value: string, max = 48) => {
  if (value.length <= max) return value
  const left = Math.ceil((max - 1) / 2)
  const right = Math.floor((max - 1) / 2)
  return `${value.slice(0, left)}…${value.slice(value.length - right)}`
}
