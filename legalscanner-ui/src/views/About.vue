<template>
  <div class="about">
    <h1>About Legal Scanner</h1>

    <div class="about-section">
      <h2>Overview</h2>
      <div class="content-card">
        <p><strong>Legal Scanner</strong> is a comprehensive compliance and security analysis tool designed to help development teams identify potential legal and regulatory issues in their codebases. By scanning Git repositories, the system automatically detects open source licenses, copyright statements, and export control compliance concerns.</p>

        <p>The scanner leverages industry-standard tools to provide thorough coverage of both legal compliance and security-related export control requirements. Whether you're auditing a new open source dependency, preparing for a compliance review, or ensuring your project meets export control regulations, Legal Scanner provides the automated analysis you need to maintain a compliant and secure codebase.</p>

        <p class="version"><strong>Version:</strong> 0.1.0</p>
      </div>
    </div>

    <div class="about-section">
      <h2>Fossology Scanner</h2>
      <div class="content-card scanner-card">
        <div class="scanner-header">
          <h3>License & Copyright Detection</h3>
          <p class="scanner-url"><strong>Website:</strong> <a href="https://www.fossology.org" target="_blank">https://www.fossology.org</a></p>
        </div>
        <p class="scanner-description">
          Fossology is an open source license compliance software system and toolkit. As a comprehensive scanning engine, Fossology employs multiple specialized agents including Nomos, Monk, and Ojo to identify open source licenses with high accuracy. It extracts copyright statements, holders, and years from source code, providing detailed attribution information. The scanner automatically maps detected licenses to SPDX identifiers, ensuring standardized compliance reporting. Fossology serves as the primary engine for legal compliance analysis, offering deep inspection of license obligations and copyright notices across your entire codebase.
        </p>
      </div>
    </div>

    <div class="about-section">
      <h2>Semgrep Scanner</h2>
      <div class="content-card scanner-card">
        <div class="scanner-header">
          <h3>Export Control & Security Analysis</h3>
          <p class="scanner-url"><strong>Website:</strong> <a href="https://semgrep.dev" target="_blank">https://semgrep.dev</a></p>
        </div>
        <p class="scanner-description">
          Semgrep is a fast, open source static analysis tool that uses pattern matching to detect code patterns of interest. In Legal Scanner, Semgrep identifies cryptographic implementations and security-sensitive code patterns that may be subject to export control regulations. The tool scans for symmetric and asymmetric encryption algorithms, key generation functions, digital signatures, and secure communication protocols. By detecting these patterns early in the development cycle, Semgrep helps organizations understand potential export control obligations and maintain compliance with international trade regulations such as EAR and ITAR.
        </p>
      </div>
    </div>

    <div class="about-section">
      <h2>Technology Stack</h2>
      <div class="content-card">
        <ul class="tech-list">
          <li><strong>Backend:</strong> Rust + Axum web framework</li>
          <li><strong>Frontend:</strong> Vue 3 + Vite + Pinia</li>
          <li><strong>Database:</strong> SQLite with async SQLx</li>
          <li><strong>Scanners:</strong> Fossology + Semgrep</li>
          <li><strong>Deployment:</strong> Docker Compose</li>
        </ul>
      </div>
    </div>
  </div>
</template>

<script setup>
// About page - static content only
</script>

<style scoped>
.about {
  max-width: 900px;
  margin: 0 auto;
}

h1 {
  color: #2c3e50;
  margin-bottom: 2rem;
}

.about-section {
  margin-bottom: 3rem;
}

.about-section h2 {
  color: #2c3e50;
  margin-bottom: 1rem;
  font-size: 1.5rem;
}

.about-section h3 {
  color: #34495e;
  margin: 0 0 0.5rem 0;
  font-size: 1.25rem;
}

.content-card {
  background: white;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.content-card p {
  margin-bottom: 1rem;
  color: #555;
  line-height: 1.8;
  text-align: justify;
}

.content-card p:last-child {
  margin-bottom: 0;
}

.version {
  margin-top: 1.5rem;
  padding-top: 1rem;
  border-top: 1px solid #e0e0e0;
  text-align: left;
}

.scanner-card {
  border-left: 4px solid #3498db;
}

.scanner-header {
  margin-bottom: 1rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid #e0e0e0;
}

.scanner-url {
  margin: 0.5rem 0 0 0;
  font-size: 0.9rem;
  color: #666;
}

.scanner-url a {
  color: #3498db;
  text-decoration: none;
}

.scanner-url a:hover {
  text-decoration: underline;
}

.scanner-description {
  font-size: 0.95rem;
  color: #555;
  line-height: 1.8;
  text-align: justify;
}

.tech-list {
  list-style: none;
  padding: 0;
}

.tech-list li {
  padding: 0.75rem 0;
  color: #555;
  border-bottom: 1px solid #f0f0f0;
}

.tech-list li:last-child {
  border-bottom: none;
}

.tech-list strong {
  color: #2c3e50;
  display: inline-block;
  min-width: 120px;
}
</style>
