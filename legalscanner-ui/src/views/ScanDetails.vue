<template>
  <div class="scan-details">
    <div class="header">
      <button class="btn-back" @click="goBack">&larr; Back to Scans</button>
    </div>

    <div v-if="loading" class="loading">Loading...</div>

    <div v-else-if="error" class="error-message">{{ error }}</div>

    <div v-else-if="currentScan" class="scan-info">
      <h1>Scan Details</h1>

      <div class="info-card">
        <div class="info-row">
          <span class="info-label">Repository:</span>
          <span class="info-value">{{ currentScan.git_url }}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Status:</span>
          <span class="scan-status" :class="`status-${currentScan.status}`">
            {{ formatStatus(currentScan.status) }}
          </span>
        </div>
        <div class="info-row">
          <span class="info-label">Created:</span>
          <span class="info-value">{{ formatDate(currentScan.created_at) }}</span>
        </div>
        <div v-if="currentScan.completed_at" class="info-row">
          <span class="info-label">Completed:</span>
          <span class="info-value">{{ formatDate(currentScan.completed_at) }}</span>
        </div>
        <div v-if="currentScan.summary" class="info-row summary">
          <div class="summary-stats">
            <div class="stat">
              <span class="stat-value">{{ currentScan.summary.total_files }}</span>
              <span class="stat-label">Total Files</span>
            </div>
            <div class="stat">
              <span class="stat-value">{{ currentScan.summary.unique_licenses }}</span>
              <span class="stat-label">Licenses</span>
            </div>
            <div class="stat">
              <span class="stat-value">{{ currentScan.summary.unique_copyrights }}</span>
              <span class="stat-label">Copyrights</span>
            </div>
          </div>
        </div>
      </div>

      <div v-if="currentScan.status === 'completed'" class="results-section">
        <ResultsViewer :scan-id="scanId" />
      </div>

      <div v-else-if="currentScan.status === 'in_progress'" class="in-progress-message">
        <p>Scan is currently in progress. This page will update automatically...</p>
        <div class="spinner"></div>
      </div>

      <div v-else-if="currentScan.status === 'failed'" class="error-message">
        Scan failed: {{ currentScan.error_message || 'Unknown error' }}
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useScansStore } from '@/store/scans'
import ResultsViewer from '@/components/ResultsViewer.vue'

const route = useRoute()
const router = useRouter()
const scansStore = useScansStore()

const scanId = route.params.id
const loading = computed(() => scansStore.loading)
const error = computed(() => scansStore.error)
const currentScan = computed(() => scansStore.currentScan)

let refreshInterval = null

onMounted(async () => {
  await loadScanDetails()

  // Auto-refresh if scan is in progress
  if (currentScan.value?.status === 'in_progress' || currentScan.value?.status === 'pending') {
    refreshInterval = setInterval(loadScanDetails, 5000) // Refresh every 5 seconds
  }
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})

const loadScanDetails = async () => {
  try {
    await scansStore.fetchScan(scanId)

    // Stop refreshing if scan is complete or failed
    if (currentScan.value && ['completed', 'failed'].includes(currentScan.value.status)) {
      if (refreshInterval) {
        clearInterval(refreshInterval)
        refreshInterval = null
      }
    }
  } catch (e) {
    console.error('Failed to load scan details:', e)
  }
}

const goBack = () => {
  router.push('/')
}

const formatStatus = (status) => {
  const statusMap = {
    pending: 'Pending',
    in_progress: 'In Progress',
    completed: 'Completed',
    failed: 'Failed'
  }
  return statusMap[status] || status
}

const formatDate = (dateString) => {
  const date = new Date(dateString)
  return date.toLocaleString()
}
</script>

<style scoped>
.scan-details {
  max-width: 1200px;
  margin: 0 auto;
}

.header {
  margin-bottom: 2rem;
}

.btn-back {
  padding: 0.5rem 1rem;
  border: 1px solid #3498db;
  background: white;
  color: #3498db;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s;
}

.btn-back:hover {
  background: #3498db;
  color: white;
}

.loading {
  text-align: center;
  padding: 3rem;
  color: #7f8c8d;
}

.error-message {
  padding: 1rem;
  background-color: #ffe6e6;
  border: 1px solid #ff4d4d;
  border-radius: 4px;
  color: #c0392b;
  margin-bottom: 2rem;
}

.scan-info h1 {
  color: #2c3e50;
  margin-bottom: 2rem;
}

.info-card {
  background: white;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  margin-bottom: 2rem;
}

.info-row {
  display: flex;
  padding: 0.75rem 0;
  border-bottom: 1px solid #f0f0f0;
}

.info-row:last-child {
  border-bottom: none;
}

.info-label {
  font-weight: 600;
  color: #2c3e50;
  min-width: 120px;
}

.info-value {
  color: #555;
  word-break: break-all;
}

.scan-status {
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: 0.875rem;
  font-weight: 500;
}

.status-pending {
  background-color: #fff3cd;
  color: #856404;
}

.status-in_progress {
  background-color: #cfe2ff;
  color: #084298;
}

.status-completed {
  background-color: #d1e7dd;
  color: #0f5132;
}

.status-failed {
  background-color: #f8d7da;
  color: #842029;
}

.summary {
  flex-direction: column;
  padding-top: 1.5rem;
}

.summary-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 1.5rem;
  margin-top: 1rem;
}

.stat {
  text-align: center;
}

.stat-value {
  display: block;
  font-size: 2rem;
  font-weight: bold;
  color: #3498db;
}

.stat-label {
  display: block;
  font-size: 0.875rem;
  color: #7f8c8d;
  margin-top: 0.5rem;
}

.results-section {
  margin-top: 2rem;
}

.in-progress-message {
  text-align: center;
  padding: 3rem;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.spinner {
  margin: 2rem auto;
  width: 50px;
  height: 50px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #3498db;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>
