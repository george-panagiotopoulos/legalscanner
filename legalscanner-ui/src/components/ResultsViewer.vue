<template>
  <div class="results-viewer">
    <h2>Scan Results</h2>

    <div v-if="loading" class="loading">Loading results...</div>

    <div v-else-if="error" class="error-message">{{ error }}</div>

    <div v-else-if="!results" class="loading">No results data received from API yet...</div>

    <div v-else class="results-container">
      <div class="tabs">
        <button
          :class="['tab', { active: activeTab === 'licenses' }]"
          @click="activeTab = 'licenses'"
        >
          Licenses ({{ results.results?.licenses?.length || 0 }})
        </button>
        <button
          :class="['tab', { active: activeTab === 'copyrights' }]"
          @click="activeTab = 'copyrights'"
        >
          Copyrights ({{ results.results?.copyrights?.length || 0 }})
        </button>
        <button
          :class="['tab', { active: activeTab === 'ecc' }]"
          @click="activeTab = 'ecc'"
        >
          ECC Findings ({{ results.results?.ecc_findings?.length || 0 }})
        </button>
        <button
          :class="['tab', { active: activeTab === 'json' }]"
          @click="activeTab = 'json'"
        >
          Raw JSON
        </button>
      </div>

      <div class="tab-content">
        <div v-if="activeTab === 'licenses'" class="licenses-view">
          <div v-if="!results.results?.licenses || results.results.licenses.length === 0" class="empty-state">
            No licenses found in this repository.
          </div>
          <div v-else class="table-container">
            <table>
              <thead>
                <tr>
                  <th>File Path</th>
                  <th>License</th>
                  <th>SPDX ID</th>
                  <th>Confidence</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(license, index) in results.results.licenses" :key="index">
                  <td class="file-path">{{ license.file_path }}</td>
                  <td>{{ license.license }}</td>
                  <td>
                    <span v-if="license.spdx_id" class="spdx-badge">
                      {{ license.spdx_id }}
                    </span>
                    <span v-else class="text-muted">-</span>
                  </td>
                  <td>
                    <span class="confidence-badge" :class="getConfidenceClass(license.confidence)">
                      {{ formatConfidence(license.confidence) }}
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <div v-else-if="activeTab === 'copyrights'" class="copyrights-view">
          <div v-if="!results.results?.copyrights || results.results.copyrights.length === 0" class="empty-state">
            No copyright statements found in this repository.
          </div>
          <div v-else class="table-container">
            <table>
              <thead>
                <tr>
                  <th>File Path</th>
                  <th>Statement</th>
                  <th>Holders</th>
                  <th>Years</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(copyright, index) in results.results.copyrights" :key="index">
                  <td class="file-path">{{ copyright.file_path }}</td>
                  <td>{{ copyright.statement }}</td>
                  <td>{{ copyright.holders.join(', ') || '-' }}</td>
                  <td>{{ copyright.years.join(', ') || '-' }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <div v-else-if="activeTab === 'ecc'" class="ecc-view">
          <div v-if="!results.results?.ecc_findings || results.results.ecc_findings.length === 0" class="empty-state">
            No export control findings detected in this repository.
          </div>
          <div v-else>
            <div class="ecc-summary">
              <p class="summary-text">
                <strong>Export Control Classification (ECC)</strong> findings indicate potential export-controlled technology such as cryptography, encryption algorithms, or other regulated content. Review these findings carefully before distributing internationally.
              </p>
            </div>
            <div class="table-container">
              <table>
                <thead>
                  <tr>
                    <th>Risk</th>
                    <th>Source</th>
                    <th>File Path</th>
                    <th>Line</th>
                    <th>Check ID</th>
                    <th>Finding</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(ecc, index) in results.results.ecc_findings" :key="index">
                    <td>
                      <span class="risk-badge" :class="getRiskClass(ecc.risk_severity)">
                        {{ ecc.risk_severity.toUpperCase() }}
                      </span>
                    </td>
                    <td>
                      <span v-if="ecc.source" class="source-badge">
                        {{ ecc.source }}
                      </span>
                      <span v-else class="text-muted">-</span>
                    </td>
                    <td class="file-path">{{ ecc.file_path }}</td>
                    <td>
                      <span v-if="ecc.line_number" class="line-number">
                        {{ ecc.line_number }}
                      </span>
                      <span v-else class="text-muted">-</span>
                    </td>
                    <td>
                      <span v-if="ecc.check_id" class="check-id" :title="ecc.check_id">
                        {{ ecc.check_id }}
                      </span>
                      <span v-else class="text-muted">-</span>
                    </td>
                    <td class="ecc-content">{{ ecc.content }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <div v-else-if="activeTab === 'json'" class="json-view">
          <button class="btn-copy" @click="copyToClipboard">Copy JSON</button>
          <pre><code>{{ JSON.stringify(results, null, 2) }}</code></pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue'
import { useScansStore } from '@/store/scans'

const props = defineProps({
  scanId: {
    type: String,
    required: true
  }
})

const scansStore = useScansStore()

const activeTab = ref('licenses')
const loading = computed(() => scansStore.loadingResults)
const error = computed(() => scansStore.error)
const results = computed(() => scansStore.currentResults)

onMounted(async () => {
  await scansStore.fetchResults(props.scanId)
})

const formatConfidence = (confidence) => {
  if (confidence === null || confidence === undefined) return 'N/A'
  return `${Math.round(confidence * 100)}%`
}

const getConfidenceClass = (confidence) => {
  if (confidence >= 0.9) return 'confidence-high'
  if (confidence >= 0.7) return 'confidence-medium'
  return 'confidence-low'
}

const getRiskClass = (severity) => {
  const severityLower = severity?.toLowerCase() || 'low'
  return `risk-${severityLower}`
}

const copyToClipboard = () => {
  const json = JSON.stringify(results.value, null, 2)
  navigator.clipboard.writeText(json).then(() => {
    alert('JSON copied to clipboard!')
  })
}
</script>

<style scoped>
.results-viewer {
  background: white;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

h2 {
  color: #2c3e50;
  margin-bottom: 1.5rem;
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
}

.tabs {
  display: flex;
  gap: 0.5rem;
  border-bottom: 2px solid #e0e0e0;
  margin-bottom: 1.5rem;
}

.tab {
  padding: 0.75rem 1.5rem;
  border: none;
  background: transparent;
  color: #7f8c8d;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -2px;
  transition: all 0.3s;
}

.tab:hover {
  color: #2c3e50;
}

.tab.active {
  color: #3498db;
  border-bottom-color: #3498db;
  font-weight: 600;
}

.tab-content {
  min-height: 300px;
}

.empty-state {
  text-align: center;
  padding: 3rem;
  color: #7f8c8d;
}

.table-container {
  overflow-x: auto;
}

table {
  width: 100%;
  border-collapse: collapse;
}

thead {
  background-color: #f8f9fa;
}

th {
  text-align: left;
  padding: 1rem;
  font-weight: 600;
  color: #2c3e50;
  border-bottom: 2px solid #e0e0e0;
}

td {
  padding: 1rem;
  border-bottom: 1px solid #f0f0f0;
}

tr:hover {
  background-color: #f8f9fa;
}

.file-path {
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  color: #555;
}

.spdx-badge {
  display: inline-block;
  padding: 0.25rem 0.5rem;
  background-color: #e3f2fd;
  color: #1976d2;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 500;
}

.confidence-badge {
  display: inline-block;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 500;
}

.confidence-high {
  background-color: #d1e7dd;
  color: #0f5132;
}

.confidence-medium {
  background-color: #fff3cd;
  color: #856404;
}

.confidence-low {
  background-color: #f8d7da;
  color: #842029;
}

.text-muted {
  color: #7f8c8d;
}

.json-view {
  position: relative;
}

.btn-copy {
  position: absolute;
  top: 1rem;
  right: 1rem;
  padding: 0.5rem 1rem;
  background-color: #3498db;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.btn-copy:hover {
  background-color: #2980b9;
}

pre {
  background-color: #f8f9fa;
  padding: 1.5rem;
  border-radius: 4px;
  overflow-x: auto;
  max-height: 600px;
}

code {
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  color: #2c3e50;
}

.ecc-summary {
  background-color: #fff3cd;
  border-left: 4px solid #ffc107;
  padding: 1rem;
  margin-bottom: 1.5rem;
  border-radius: 4px;
}

.summary-text {
  margin: 0;
  color: #856404;
  line-height: 1.6;
}

.ecc-content {
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  color: #555;
  max-width: 500px;
  word-wrap: break-word;
}

.risk-badge {
  display: inline-block;
  padding: 0.35rem 0.75rem;
  border-radius: 4px;
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.5px;
  min-width: 70px;
  text-align: center;
}

.risk-low {
  background-color: #d1e7dd;
  color: #0f5132;
}

.risk-medium {
  background-color: #fff3cd;
  color: #856404;
}

.risk-high {
  background-color: #f8d7da;
  color: #842029;
}

.risk-critical {
  background-color: #dc3545;
  color: white;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.8;
  }
}

.source-badge {
  display: inline-block;
  padding: 0.25rem 0.5rem;
  background-color: #e3f2fd;
  color: #1976d2;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 500;
  text-transform: lowercase;
}

.line-number {
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  color: #666;
  font-weight: 500;
}

.check-id {
  font-family: 'Courier New', monospace;
  font-size: 0.75rem;
  color: #555;
  max-width: 200px;
  display: inline-block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
