import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useAuthStore = defineStore('auth', () => {
  const apiKey = ref(null)

  const isAuthenticated = computed(() => !!apiKey.value)

  const loadApiKey = () => {
    const stored = localStorage.getItem('api_key')
    if (stored) {
      apiKey.value = stored
    }
  }

  const setApiKey = (key) => {
    apiKey.value = key
    localStorage.setItem('api_key', key)
  }

  const clearApiKey = () => {
    apiKey.value = null
    localStorage.removeItem('api_key')
  }

  return {
    apiKey,
    isAuthenticated,
    loadApiKey,
    setApiKey,
    clearApiKey
  }
})
