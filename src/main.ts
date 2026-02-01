import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import 'vditor/dist/index.css'

import App from './App.vue'
import { routes } from './router'
import './assets/styles/main.css'

// Create router
const router = createRouter({
    history: createWebHistory(),
    routes
})

// Update canonical URL on route change
router.beforeEach((to, _from, next) => {
    const canonicalUrl = `${window.location.origin}${to.path}`
    let link = document.querySelector("link[rel='canonical']") as HTMLLinkElement
    if (!link) {
        link = document.createElement('link')
        link.setAttribute('rel', 'canonical')
        document.head.appendChild(link)
    }
    link.setAttribute('href', canonicalUrl)
    next()
})

// Create and mount app
const app = createApp(App)
app.use(router)
app.use(ElementPlus)
app.mount('#app')
