import type { RouteRecordRaw } from 'vue-router'

export const routes: RouteRecordRaw[] = [
    {
        path: '/',
        name: 'Home',
        component: () => import('@pages/Main.vue'),
        meta: { title: 'Carbo' }
    },
    {
        path: '/about',
        name: 'About',
        component: () => import('@pages/About.vue'),
        meta: { title: '关于 - Arya' }
    },
    {
        path: '/export/image',
        name: 'ExportImage',
        component: () => import('@pages/ExportImage.vue'),
        meta: { title: '导出图片 - Arya' }
    },
    {
        path: '/export/pdf',
        name: 'ExportPdf',
        component: () => import('@pages/ExportPdf.vue'),
        meta: { title: '导出 PDF - Arya' }
    },
    {
        path: '/export/ppt',
        name: 'ExportPPT',
        component: () => import('@pages/ExportPPT.vue'),
        meta: { title: 'PPT 预览 - Arya' }
    }
]
