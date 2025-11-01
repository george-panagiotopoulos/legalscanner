import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import ScanDetails from '../views/ScanDetails.vue'
import Settings from '../views/Settings.vue'
import About from '../views/About.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: Home
    },
    {
      path: '/scans/:id',
      name: 'scanDetails',
      component: ScanDetails
    },
    {
      path: '/settings',
      name: 'settings',
      component: Settings
    },
    {
      path: '/about',
      name: 'about',
      component: About
    }
  ]
})

export default router
