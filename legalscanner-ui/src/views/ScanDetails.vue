<template>
  <div class="scan-details">
    <div class="header">
      <button class="btn-back" @click="goBack">&larr; Back to Scans</button>
      <button v-if="currentScan" class="btn-delete" @click="handleDelete">Delete Scan</button>
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
          <span class="info-label">Overall Status:</span>
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
      </div>

      <div v-if="currentScan.fossology_status || currentScan.semgrep_status" class="scanner-progress-section">
        <h2>Scanner Progress</h2>
        <div class="scanner-cards">
          <div class="scanner-card">
            <div class="scanner-card-header">
              <h3>Fossology</h3>
              <span class="scan-status" :class="`status-${currentScan.fossology_status}`">
                {{ formatStatus(currentScan.fossology_status) }}
              </span>
            </div>
            <p class="scanner-description">License and Copyright Detection</p>
          </div>
          <div class="scanner-card">
            <div class="scanner-card-header">
              <h3>Semgrep</h3>
              <span class="scan-status" :class="`status-${currentScan.semgrep_status}`">
                {{ formatStatus(currentScan.semgrep_status) }}
              </span>
            </div>
            <p class="scanner-description">ECC and Cryptographic Analysis</p>
          </div>
        </div>
      </div>

      <div v-if="currentScan.risk_assessment" class="risk-assessment-section">
        <h2>Risk Assessment</h2>
        <div class="risk-overview">
          <div class="risk-score-card" :class="`risk-${currentScan.risk_assessment.level}`">
            <div class="risk-score-value">{{ currentScan.risk_assessment.score }}</div>
            <div class="risk-score-label">Risk Score</div>
            <div class="risk-level-badge" :class="`risk-${currentScan.risk_assessment.level}`">
              {{ formatRiskLevel(currentScan.risk_assessment.level) }}
            </div>
          </div>
          <div class="risk-factors">
            <h3>Risk Factors</h3>
            <div v-for="factor in currentScan.risk_assessment.factors" :key="factor.category" class="risk-factor-card">
              <div class="risk-factor-header">
                <span class="risk-factor-category">{{ formatCategory(factor.category) }}</span>
                <span class="risk-factor-severity" :class="`severity-${factor.severity}`">
                  {{ formatSeverity(factor.severity) }}
                </span>
              </div>
              <p class="risk-factor-description">{{ factor.description }}</p>
              <div class="risk-factor-details">
                <span class="affected-count">Affected: {{ factor.affected_count }}</span>
                <div v-if="factor.details && factor.details.length > 0" class="details-list">
                  <details>
                    <summary>View details ({{ factor.details.length }})</summary>
                    <ul>
                      <li v-for="(detail, idx) in factor.details" :key="idx">{{ detail }}</li>
                    </ul>
                  </details>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-if="currentScan.summary" class="summary-section">
        <h2>Summary</h2>
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

      <div v-if="currentScan.status === 'completed'" class="results-section">
        <ResultsViewer :scan-id="scanId" />
      </div>

      <div v-else-if="currentScan.status === 'in_progress' || currentScan.status === 'pending'" class="in-progress-message">
        <div class="spinner"></div>
        <p>Scan is currently in progress. This page will update automatically...</p>
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
import { deleteScan } from '@/api/scans'

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

const formatRiskLevel = (level) => {
  const levelMap = {
    low: 'Low Risk',
    medium: 'Medium Risk',
    high: 'High Risk',
    critical: 'Critical Risk'
  }
  return levelMap[level] || level
}

const formatSeverity = (severity) => {
  const severityMap = {
    low: 'Low',
    medium: 'Medium',
    high: 'High',
    critical: 'Critical'
  }
  return severityMap[severity] || severity
}

const formatCategory = (category) => {
  const categoryMap = {
    copyleft_license: 'Copyleft License',
    missing_spdx: 'Missing SPDX',
    high_risk_license: 'High Risk License',
    patent_concern: 'Patent Concern',
    export_control: 'Export Control (ECC)',
    cryptography: 'Cryptographic Content',
    no_license: 'No License Information'
  }
  return categoryMap[category] || category.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())
}

const handleDelete = async () => {
  if (!confirm('Are you sure you want to delete this scan? This action cannot be undone.')) {
    return
  }

  try {
    await deleteScan(scanId)
    router.push('/')
  } catch (e) {
    console.error('Failed to delete scan:', e)
    alert('Failed to delete scan. Please try again.')
  }
}
</script>

<style scoped>
.scan-details {
  max-width: 1200px;
  margin: 0 auto;
}

.header {
  margin-bottom: 2rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
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

.btn-delete {
  padding: 0.5rem 1rem;
  border: 1px solid #e74c3c;
  background: white;
  color: #e74c3c;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s;
  font-weight: 500;
}

.btn-delete:hover {
  background: #e74c3c;
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

.scanner-progress-section {
  margin-bottom: 2rem;
}

.scanner-progress-section h2 {
  color: #2c3e50;
  margin-bottom: 1rem;
  font-size: 1.25rem;
}

.scanner-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1rem;
}

.scanner-card {
  background: white;
  padding: 1.5rem;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  border: 2px solid #e0e0e0;
  transition: all 0.3s;
}

.scanner-card:hover {
  border-color: #3498db;
  box-shadow: 0 4px 12px rgba(52, 152, 219, 0.15);
}

.scanner-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
}

.scanner-card-header h3 {
  margin: 0;
  color: #2c3e50;
  font-size: 1.1rem;
  font-weight: 600;
}

.scanner-description {
  color: #7f8c8d;
  font-size: 0.9rem;
  margin: 0;
}

.summary-section {
  margin-bottom: 2rem;
}

.summary-section h2 {
  color: #2c3e50;
  margin-bottom: 1rem;
  font-size: 1.25rem;
}

.summary-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 1.5rem;
  background: white;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
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

.in-progress-message p {
  margin-top: 1.5rem;
  color: #555;
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

/* Risk Assessment Styles */
.risk-assessment-section {
  margin-bottom: 2rem;
}

.risk-assessment-section h2 {
  color: #2c3e50;
  margin-bottom: 1rem;
  font-size: 1.25rem;
}

.risk-overview {
  display: grid;
  grid-template-columns: 300px 1fr;
  gap: 1.5rem;
  background: white;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.risk-score-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  border-radius: 12px;
  border: 3px solid;
  background: linear-gradient(135deg, rgba(255,255,255,0.9) 0%, rgba(255,255,255,0.7) 100%);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  transition: all 0.3s;
}

.risk-score-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
}

.risk-score-card.risk-low {
  border-color: #27ae60;
  background: linear-gradient(135deg, #d1e7dd 0%, #a3cfbb 100%);
}

.risk-score-card.risk-medium {
  border-color: #f39c12;
  background: linear-gradient(135deg, #fff3cd 0%, #ffe69c 100%);
}

.risk-score-card.risk-high {
  border-color: #e67e22;
  background: linear-gradient(135deg, #ffd6a5 0%, #ffb347 100%);
}

.risk-score-card.risk-critical {
  border-color: #c0392b;
  background: linear-gradient(135deg, #f8d7da 0%, #f5c2c7 100%);
  animation: pulse-risk 2s ease-in-out infinite;
}

@keyframes pulse-risk {
  0%, 100% {
    box-shadow: 0 4px 12px rgba(192, 57, 43, 0.3);
  }
  50% {
    box-shadow: 0 4px 20px rgba(192, 57, 43, 0.5);
  }
}

.risk-score-value {
  font-size: 4rem;
  font-weight: bold;
  line-height: 1;
  margin-bottom: 0.5rem;
}

.risk-score-card.risk-low .risk-score-value {
  color: #27ae60;
}

.risk-score-card.risk-medium .risk-score-value {
  color: #f39c12;
}

.risk-score-card.risk-high .risk-score-value {
  color: #e67e22;
}

.risk-score-card.risk-critical .risk-score-value {
  color: #c0392b;
}

.risk-score-label {
  font-size: 0.875rem;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: #555;
  font-weight: 600;
  margin-bottom: 1rem;
}

.risk-level-badge {
  padding: 0.5rem 1rem;
  border-radius: 20px;
  font-size: 0.875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.risk-level-badge.risk-low {
  background-color: #27ae60;
  color: white;
}

.risk-level-badge.risk-medium {
  background-color: #f39c12;
  color: white;
}

.risk-level-badge.risk-high {
  background-color: #e67e22;
  color: white;
}

.risk-level-badge.risk-critical {
  background-color: #c0392b;
  color: white;
}

.risk-factors {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.risk-factors h3 {
  color: #2c3e50;
  font-size: 1.1rem;
  margin-bottom: 0.5rem;
}

.risk-factor-card {
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 1rem;
  background: #f8f9fa;
  transition: all 0.3s;
}

.risk-factor-card:hover {
  border-color: #3498db;
  background: white;
  box-shadow: 0 2px 8px rgba(52, 152, 219, 0.1);
}

.risk-factor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
}

.risk-factor-category {
  font-weight: 600;
  color: #2c3e50;
  font-size: 0.95rem;
}

.risk-factor-severity {
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.risk-factor-severity.severity-low {
  background-color: #d1e7dd;
  color: #0f5132;
}

.risk-factor-severity.severity-medium {
  background-color: #fff3cd;
  color: #856404;
}

.risk-factor-severity.severity-high {
  background-color: #ffd6a5;
  color: #8b4513;
}

.risk-factor-severity.severity-critical {
  background-color: #f8d7da;
  color: #842029;
}

.risk-factor-description {
  color: #555;
  font-size: 0.9rem;
  margin-bottom: 0.75rem;
  line-height: 1.5;
}

.risk-factor-details {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.affected-count {
  font-size: 0.85rem;
  color: #7f8c8d;
  font-weight: 500;
}

.details-list details {
  margin-top: 0.5rem;
}

.details-list summary {
  cursor: pointer;
  color: #3498db;
  font-size: 0.85rem;
  font-weight: 500;
  user-select: none;
  padding: 0.25rem 0;
}

.details-list summary:hover {
  color: #2980b9;
  text-decoration: underline;
}

.details-list ul {
  margin: 0.5rem 0 0 1.5rem;
  padding: 0;
  list-style: disc;
}

.details-list li {
  color: #555;
  font-size: 0.85rem;
  padding: 0.25rem 0;
  word-break: break-all;
}

@media (max-width: 768px) {
  .risk-overview {
    grid-template-columns: 1fr;
  }
}
</style>
