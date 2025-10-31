<template>
  <div class="settings">
    <h1>Settings</h1>

    <div class="settings-section">
      <h2>API Configuration</h2>
      <ApiKeyManager />
    </div>

    <div class="settings-section">
      <h2>About</h2>
      <div class="about-card">
        <p><strong>Legal Scanner</strong> - A tool to scan Git repositories for legal and compliance issues.</p>
        <p>Version: 0.1.0</p>
        <p>Identifies open source licenses and copyright statements in your codebase.</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import ApiKeyManager from '@/components/ApiKeyManager.vue'
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

.about-card {
  background: white;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.about-card p {
  margin-bottom: 0.5rem;
  color: #555;
  line-height: 1.6;
}

.about-card p:last-child {
  margin-bottom: 0;
}
</style>
