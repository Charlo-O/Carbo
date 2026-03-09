import { shallowRef } from 'vue'

const normalizePaths = (paths: string[]) =>
  paths
    .map((p) => p.trim())
    .filter((p) => p.length > 0)

export const pendingOpenPaths = shallowRef<string[] | null>(null)

export const setPendingOpenPaths = (paths: string[]) => {
  const normalized = normalizePaths(paths)
  pendingOpenPaths.value = normalized.length > 0 ? normalized : null
}

export const consumePendingOpenPaths = () => {
  const paths = pendingOpenPaths.value
  pendingOpenPaths.value = null
  return paths
}
