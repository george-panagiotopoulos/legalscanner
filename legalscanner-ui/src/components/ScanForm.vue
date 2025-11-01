<template>
  <div class="scan-form">
    <h2>Create New Scan</h2>
    <form @submit.prevent="handleSubmit">
      <div class="form-group">
        <label for="gitUrl">Git Repository URL</label>
        <input
          id="gitUrl"
          v-model="gitUrl"
          type="text"
          placeholder="https://github.com/user/repo.git"
          required
          :disabled="isSubmitting"
        />
        <small>Enter the full URL of the Git repository to scan</small>
      </div>

      <div class="form-group">
        <label for="gitToken">
          Git Access Token (Optional)
          <span class="optional-badge">Optional</span>
        </label>
        <input
          id="gitToken"
          v-model="gitToken"
          type="password"
          placeholder="ghp_xxxxxxxxxxxxxxxxxxxx"
          :disabled="isSubmitting"
        />
        <small>
          Required for private repositories.
          <a href="https://github.com/settings/tokens" target="_blank" rel="noopener">
            Create a GitHub token
          </a>
        </small>
      </div>

      <button type="submit" :disabled="isSubmitting || !gitUrl" class="btn btn-primary">
        {{ isSubmitting ? 'Starting Scan...' : 'Start Scan' }}
      </button>

      <div v-if="error" class="error-message">
        {{ error }}
      </div>

      <div v-if="success" class="success-message">
        Scan created successfully! It will begin processing shortly.
      </div>
    </form>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useScansStore } from '@/store/scans'

const emit = defineEmits(['scan-created'])

const scansStore = useScansStore()

const gitUrl = ref('')
const gitToken = ref('')
const isSubmitting = ref(false)
const error = ref(null)
const success = ref(false)

const handleSubmit = async () => {
  isSubmitting.value = true
  error.value = null
  success.value = false

  // Trim whitespace from inputs
  const trimmedUrl = gitUrl.value.trim()
  const trimmedToken = gitToken.value.trim()

  try {
    const scan = await scansStore.createScan(trimmedUrl, trimmedToken || null)
    success.value = true
    gitUrl.value = ''
    gitToken.value = ''
    emit('scan-created', scan)

    // Clear success message after 3 seconds
    setTimeout(() => {
      success.value = false
    }, 3000)
  } catch (e) {
    error.value = e.response?.data?.details || e.message || 'Failed to create scan'
  } finally {
    isSubmitting.value = false
  }
}
</script>

<style scoped>
.scan-form {
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

.form-group {
  margin-bottom: 1.5rem;
}

label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: 500;
  color: #2c3e50;
}

input[type="text"],
input[type="password"] {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 1rem;
  transition: border-color 0.3s;
}

input[type="text"]:focus,
input[type="password"]:focus {
  outline: none;
  border-color: #3498db;
}

input[type="text"]:disabled,
input[type="password"]:disabled {
  background-color: #f5f5f5;
  cursor: not-allowed;
}

input[type="password"] {
  font-family: monospace;
}

.optional-badge {
  display: inline-block;
  margin-left: 0.5rem;
  padding: 0.125rem 0.5rem;
  background-color: #e8f4f8;
  color: #3498db;
  font-size: 0.75rem;
  font-weight: normal;
  border-radius: 3px;
}

small {
  display: block;
  margin-top: 0.5rem;
  color: #7f8c8d;
  font-size: 0.875rem;
}

small a {
  color: #3498db;
  text-decoration: none;
}

small a:hover {
  text-decoration: underline;
}

.btn {
  padding: 0.75rem 2rem;
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

.btn:disabled {
  background-color: #bdc3c7;
  cursor: not-allowed;
}

.error-message {
  margin-top: 1rem;
  padding: 0.75rem;
  background-color: #ffe6e6;
  border: 1px solid #ff4d4d;
  border-radius: 4px;
  color: #c0392b;
}

.success-message {
  margin-top: 1rem;
  padding: 0.75rem;
  background-color: #e6f7e6;
  border: 1px solid #4dff4d;
  border-radius: 4px;
  color: #27ae60;
}
</style>
