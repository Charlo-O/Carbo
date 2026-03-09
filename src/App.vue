<template>
  <div id="app">
    <router-view />
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

import { setPendingOpenPaths } from '@utils/openPaths'

type OpenPathsPayload = {
  paths: string[]
}

type UnlistenFn = () => void

const router = useRouter()
let unlistenOpenPaths: UnlistenFn | null = null

const queueOpenPaths = async (paths: string[]) => {
  setPendingOpenPaths(paths)
  if (router.currentRoute.value.path !== '/') {
    await router.push('/')
  }
}

const isNonEmptyStringArray = (v: unknown): v is string[] =>
  Array.isArray(v) && v.every((x) => typeof x === 'string' && x.trim().length > 0)

onMounted(() => {
  if (!isTauri()) return

  // Startup: open file passed by OS (e.g. double-click in Explorer).
  void (async () => {
    try {
      const paths = await invoke<string[]>('consume_startup_open_paths')
      if (isNonEmptyStringArray(paths)) await queueOpenPaths(paths)
    } catch {
      // ignore
    }
  })()

  // Runtime: if the app is already running, a second launch should forward args.
  void (async () => {
    try {
      unlistenOpenPaths = await listen<OpenPathsPayload>('carbo-open-paths', async (event) => {
        const paths: unknown = event.payload?.paths
        if (isNonEmptyStringArray(paths)) await queueOpenPaths(paths)
      })
    } catch {
      // ignore
    }
  })()
})

onBeforeUnmount(() => {
  unlistenOpenPaths?.()
})
</script>

<style>
@import './assets/styles/main.css';

#app {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  width: 100%;
  height: 100vh;
}
</style>
