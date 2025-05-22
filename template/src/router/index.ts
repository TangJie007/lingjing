import { createRouter,createWebHashHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes:RouteRecordRaw[] = [
    {
        path:'/',
        component:()=>import('../views/main.vue')
    },
    {
        path:'/modal',
        component:()=>import('../views/modal.vue')
    }
]

const router = createRouter({
    history:createWebHashHistory(),
    routes
})

export default router