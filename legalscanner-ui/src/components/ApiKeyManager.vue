<template>
  <div class="api-key-manager">
    <div class="current-key-section">
      <h3>Current API Key</h3>
      <div v-if="authStore.isAuthenticated" class="key-status">
        <span class="status-icon">✓</span>
        <span>API key is configured</span>
        <button class="btn-small btn-danger" @click="clearKey">Remove Key</button>
      </div>
      <div v-else class="key-status warning">
        <span class="status-icon">⚠</span>
        <span>No API key configured. Create or enter one below.</span>
      </div>
    </div>

    <div class="create-key-section">
      <h3>Create New API Key</h3>
      <form @submit.prevent="createNewKey">
        <div class="form-group">
          <label for="keyName">Key Name</label>
          <input
            id="keyName"
            v-model="newKeyName"
            type="text"
            placeholder="e.g., My Development Key"
            required
            :disabled="isCreating"
          />
        </div>
        <button type="submit" class="btn btn-primary" :disabled="isCreating || !newKeyName">
          {{ isCreating ? 'Creating...' : 'Create API Key' }}
        </button>
      </form>

      <div v-if="newlyCreatedKey" class="new-key-display">
        <p class="warning-text">
          ⚠️ Save this key securely. It will only be shown once!
        </p>
        <div class="key-display-box">
          <code>{{ newlyCreatedKey }}</code>
          <button class="btn-copy-small" @click="copyKey(newlyCreatedKey)">Copy</button>
        </div>
        <button class="btn btn-primary" @click="useThisKey">Use This Key</button>
      </div>
    </div>

    <div class="manual-key-section">
      <h3>Or Enter Existing API Key</h3>
      <form @submit.prevent="setManualKey">
        <div class="form-group">
          <label for="manualKey">API Key</label>
          <input
            id="manualKey"
            v-model="manualKey"
            type="text"
            placeholder="lgs_..."
            :disabled="isSettingKey"
          />
        </div>
        <button type="submit" class="btn btn-secondary" :disabled="isSettingKey || !manualKey">
          {{ isSettingKey ? 'Setting...' : 'Set API Key' }}
        </button>
      </form>
    </div>

    <div v-if="error" class="error-message">
      {{ error }}
    </div>

    <div v-if="success" class="success-message">
      {{ success }}
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useAuthStore } from '@/store/auth'
import * as apiKeysApi from '@/api/apiKeys'

const authStore = useAuthStore()

const newKeyName = ref('')
const newlyCreatedKey = ref(null)
const manualKey = ref('')
const isCreating = ref(false)
const isSettingKey = ref(false)
const error = ref(null)
const success = ref(null)

const createNewKey = async () => {
  isCreating.value = true
  error.value = null
  success.value = null
  newlyCreatedKey.value = null

  try {
    const response = await apiKeysApi.createApiKey(newKeyName.value)
    newlyCreatedKey.value = response.key
    newKeyName.value = ''
    success.value = 'API key created successfully!'

    // Clear success message after 5 seconds
    setTimeout(() => {
      success.value = null
    }, 5000)
  } catch (e) {
    error.value = e.response?.data?.details || e.message || 'Failed to create API key'
  } finally {
    isCreating.value = false
  }
}

const useThisKey = () => {
  if (newlyCreatedKey.value) {
    authStore.setApiKey(newlyCreatedKey.value)
    success.value = 'API key set successfully!'
    newlyCreatedKey.value = null

    setTimeout(() => {
      success.value = null
    }, 3000)
  }
}

const setManualKey = () => {
  isSettingKey.value = true
  error.value = null
  success.value = null

  try {
    authStore.setApiKey(manualKey.value)
    success.value = 'API key set successfully!'
    manualKey.value = ''

    setTimeout(() => {
      success.value = null
    }, 3000)
  } catch (e) {
    error.value = 'Failed to set API key'
  } finally {
    isSettingKey.value = false
  }
}

const clearKey = () => {
  if (confirm('Are you sure you want to remove the current API key?')) {
    authStore.clearApiKey()
    success.value = 'API key removed'

    setTimeout(() => {
      success.value = null
    }, 3000)
  }
}

const copyKey = (key) => {
  navigator.clipboard.writeText(key).then(() => {
    alert('API key copied to clipboard!')
  })
}
</script>

<style scoped>
.api-key-manager {
  background: white;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

h3 {
  color: #2c3e50;
  margin-bottom: 1rem;
  font-size: 1.125rem;
}

.current-key-section,
.create-key-section,
.manual-key-section {
  margin-bottom: 2rem;
  padding-bottom: 2rem;
  border-bottom: 1px solid #e0e0e0;
}

.manual-key-section {
  border-bottom: none;
}

.key-status {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem;
  background-color: #d1e7dd;
  border: 1px solid #0f5132;
  border-radius: 4px;
  color: #0f5132;
}

.key-status.warning {
  background-color: #fff3cd;
  border-color: #856404;
  color: #856404;
}

.status-icon {
  font-size: 1.25rem;
}

.form-group {
  margin-bottom: 1rem;
}

label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: 500;
  color: #2c3e50;
}

input[type="text"] {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 1rem;
}

input[type="text"]:focus {
  outline: none;
  border-color: #3498db;
}

input[type="text"]:disabled {
  background-color: #f5f5f5;
  cursor: not-allowed;
}

.btn {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 4px;
  font-size: 1rem;
  cursor: pointer;
  transition: all 0.3s;
}

.btn-primary {
  background-color: #3498db;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #2980b9;
}

.btn-secondary {
  background-color: #7f8c8d;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background-color: #6c7a89;
}

.btn:disabled {
  background-color: #bdc3c7;
  cursor: not-allowed;
}

.btn-small {
  padding: 0.375rem 0.75rem;
  font-size: 0.875rem;
}

.btn-danger {
  background-color: #e74c3c;
  color: white;
}

.btn-danger:hover {
  background-color: #c0392b;
}

.new-key-display {
  margin-top: 1.5rem;
  padding: 1.5rem;
  background-color: #f8f9fa;
  border-radius: 4px;
  border: 2px solid #3498db;
}

.warning-text {
  color: #e67e22;
  font-weight: 600;
  margin-bottom: 1rem;
}

.key-display-box {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: white;
  padding: 1rem;
  border-radius: 4px;
  margin-bottom: 1rem;
}

.key-display-box code {
  flex: 1;
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  word-break: break-all;
  color: #2c3e50;
}

.btn-copy-small {
  padding: 0.375rem 0.75rem;
  background-color: #3498db;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.875rem;
  white-space: nowrap;
}

.btn-copy-small:hover {
  background-color: #2980b9;
}

.error-message {
  margin-top: 1rem;
  padding: 1rem;
  background-color: #ffe6e6;
  border: 1px solid #ff4d4d;
  border-radius: 4px;
  color: #c0392b;
}

.success-message {
  margin-top: 1rem;
  padding: 1rem;
  background-color: #d1e7dd;
  border: 1px solid #0f5132;
  border-radius: 4px;
  color: #0f5132;
}
</style>
