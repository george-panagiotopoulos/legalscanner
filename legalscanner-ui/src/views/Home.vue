<template>
  <div class="home">
    <h1>Repository Scans</h1>

    <div class="scan-form-section">
      <ScanForm @scan-created="handleScanCreated" />
    </div>

    <div class="scans-list-section">
      <ScanList />
    </div>
  </div>
</template>

<script setup>
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/store/auth'
import { useScansStore } from '@/store/scans'
import ScanForm from '@/components/ScanForm.vue'
import ScanList from '@/components/ScanList.vue'

const router = useRouter()
const authStore = useAuthStore()
const scansStore = useScansStore()

onMounted(() => {
  // Check if user has API key
  if (!authStore.isAuthenticated) {
    alert('Please configure your API key in Settings first')
    router.push('/settings')
    return
  }

  // Load scans
  scansStore.fetchScans().catch(error => {
    console.error('Failed to load scans:', error)
  })
})

const handleScanCreated = (scan) => {
  console.log('Scan created:', scan)
  // Optionally navigate to scan details
  // router.push(`/scans/${scan.scan_id}`)
}
</script>

<style scoped>
.home {
  max-width: 1000px;
  margin: 0 auto;
}

h1 {
  color: #2c3e50;
  margin-bottom: 2rem;
}

.scan-form-section {
  margin-bottom: 3rem;
}

.scans-list-section {
  margin-top: 2rem;
}
</style>
