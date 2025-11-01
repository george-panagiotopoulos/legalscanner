<template>
  <div class="settings">
    <h1>Settings</h1>

    <div class="settings-section">
      <h2>Database Management</h2>
      <div class="management-card">
        <p>Remove all scan records from the database. This action cannot be undone.</p>
        <button class="btn-cleanup" @click="handleCleanup">
          Clean Up Database
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { deleteAllScans } from '@/api/scans'
import { useScansStore } from '@/store/scans'

const scansStore = useScansStore()

const handleCleanup = async () => {
  const confirmation = confirm(
    'Are you sure you want to delete ALL scans from the database? This action cannot be undone.'
  )

  if (!confirmation) return

  const doubleConfirmation = confirm(
    'This will permanently delete all scan data. Are you absolutely sure?'
  )

  if (!doubleConfirmation) return

  try {
    const result = await deleteAllScans()
    alert(`Successfully deleted ${result.deleted} scan(s)`)

    // Refresh the scans list
    await scansStore.fetchScans()
  } catch (error) {
    console.error('Failed to clean up database:', error)
    alert('Failed to clean up database. Please try again.')
  }
}
</script>

<style scoped>
.settings {
  max-width: 800px;
  margin: 0 auto;
}

h1 {
  color: #2c3e50;
  margin-bottom: 2rem;
}

.settings-section {
  margin-bottom: 3rem;
}

.settings-section h2 {
  color: #2c3e50;
  margin-bottom: 1rem;
  font-size: 1.25rem;
}

.management-card {
  background: white;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.management-card p {
  margin-bottom: 1rem;
  color: #555;
  line-height: 1.6;
}

.btn-cleanup {
  padding: 0.75rem 1.5rem;
  border: 2px solid #e74c3c;
  background: white;
  color: #e74c3c;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s;
  font-weight: 600;
  font-size: 1rem;
}

.btn-cleanup:hover {
  background: #e74c3c;
  color: white;
}
</style>
