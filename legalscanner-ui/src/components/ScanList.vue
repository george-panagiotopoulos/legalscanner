<template>
  <div class="scan-list">
    <h2>Recent Scans</h2>

    <div v-if="loading" class="loading">
      Loading scans...
    </div>

    <div v-else-if="error" class="error-message">
      {{ error }}
    </div>

    <div v-else-if="scans.length === 0" class="empty-state">
      No scans yet. Create your first scan above!
    </div>

    <div v-else class="scans-grid">
      <div
        v-for="scan in scans"
        :key="scan.scan_id"
        class="scan-card"
        @click="viewScan(scan.scan_id)"
      >
        <div class="scan-header">
          <span class="scan-status" :class="`status-${scan.status}`">
            {{ formatStatus(scan.status) }}
          </span>
          <span class="scan-date">{{ formatDate(scan.created_at) }}</span>
        </div>
        <div class="scan-url">{{ scan.git_url }}</div>
        <div class="scan-actions">
          <button class="btn-small btn-view" @click.stop="viewScan(scan.scan_id)">
            View Details
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useScansStore } from '@/store/scans'

const router = useRouter()
const scansStore = useScansStore()

const scans = computed(() => scansStore.scans)
const loading = computed(() => scansStore.loading)
const error = computed(() => scansStore.error)

const viewScan = (scanId) => {
  router.push(`/scans/${scanId}`)
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
.scan-list {
  background: white;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

h2 {
  color: #2c3e50;
  margin-bottom: 1.5rem;
  font-size: 1.5rem;
}

.loading,
.empty-state {
  text-align: center;
  padding: 2rem;
  color: #7f8c8d;
}

.error-message {
  padding: 1rem;
  background-color: #ffe6e6;
  border: 1px solid #ff4d4d;
  border-radius: 4px;
  color: #c0392b;
}

.scans-grid {
  display: grid;
  gap: 1rem;
}

.scan-card {
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 1.5rem;
  cursor: pointer;
  transition: all 0.3s;
}

.scan-card:hover {
  border-color: #3498db;
  box-shadow: 0 4px 12px rgba(52, 152, 219, 0.15);
  transform: translateY(-2px);
}

.scan-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
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

.scan-date {
  color: #7f8c8d;
  font-size: 0.875rem;
}

.scan-url {
  color: #2c3e50;
  font-weight: 500;
  margin-bottom: 1rem;
  word-break: break-all;
}

.scan-actions {
  display: flex;
  gap: 0.5rem;
}

.btn-small {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.3s;
}

.btn-view {
  background-color: #3498db;
  color: white;
}

.btn-view:hover {
  background-color: #2980b9;
}
</style>
