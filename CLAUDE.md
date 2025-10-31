# Legal Scanner - Architecture & AI Development Guidelines

This document provides the architectural blueprint for the Legal Scanner project and establishes strict guidelines for AI-assisted development.

---

## ⚠️ CRITICAL: AI Development Rules

### Architectural Integrity

**YOU MUST FOLLOW THESE RULES WITHOUT DEVIATION:**

1. **NO Architecture Changes Without Approval**
   - Do NOT add new frameworks or libraries without explicit user permission
   - Do NOT change the technology stack (Rust/Axum, Vue3/Vite, SQLite, Fossology)
   - Do NOT refactor the module structure without approval
   - Do NOT introduce new architectural patterns without discussion

2. **Always Ask Before Adding Components**
   - New dependencies → **ASK FIRST**
   - New database tables → **ASK FIRST**
   - New API endpoints → **ASK FIRST**
   - New scanner implementations → **ASK FIRST**
   - New Docker containers → **ASK FIRST**

3. **Stick to Established Patterns**
   - Use existing error handling (AppError enum)
   - Follow existing database patterns (SQLx with async)
   - Maintain RESTful API conventions
   - Use established Vue component structure
   - Follow Rust idioms (Result types, async/await, traits)

4. **When in Doubt**
   - Present a plan before implementing
   - Explain trade-offs and alternatives
   - Wait for user approval
   - Document any deviations in commit messages

---

## Technology Stack (IMMUTABLE)

### Backend
- **Language**: Rust (edition 2021)
- **Web Framework**: Axum 0.7
- **Database**: SQLite with SQLx 0.8 (async)
- **Git Operations**: git2 0.19 (libgit2)
- **Async Runtime**: Tokio 1.41
- **Error Handling**: thiserror 2.0 + custom AppError
- **Authentication**: Argon2 0.5 for API key hashing

### Frontend
- **Framework**: Vue 3 with Composition API
- **Build Tool**: Vite 6
- **State Management**: Pinia
- **HTTP Client**: Axios
- **Router**: Vue Router

### Scanner
- **Primary**: Fossology (containerized)
- **Integration**: REST API with polling

### Deployment
- **Orchestration**: Docker Compose
- **API Port**: 5301
- **UI Port**: 5300
- **Fossology Port**: 5302

---

## Project Structure

```
legalScanner/
├── legalscanner-api/          # Rust backend (Axum + SQLx)
│   ├── src/
│   │   ├── main.rs            # Entry point, server initialization
│   │   ├── config.rs          # Environment configuration
│   │   ├── error.rs           # Custom error types (AppError)
│   │   ├── api/               # REST API layer
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs      # Route registration
│   │   │   ├── handlers/      # Request handlers
│   │   │   │   ├── scans.rs   # Scan CRUD operations
│   │   │   │   ├── scan_job.rs # Background scan execution
│   │   │   │   ├── results.rs # Results retrieval
│   │   │   │   └── api_keys.rs # API key management
│   │   │   ├── models/        # Request/Response DTOs
│   │   │   └── middleware/    # Auth, CORS, etc.
│   │   ├── db/                # Database layer
│   │   │   ├── mod.rs         # Pool creation, migrations
│   │   │   └── models/        # Database models & queries
│   │   │       ├── scan.rs    # Scan entity
│   │   │       ├── scan_result.rs # Results entity
│   │   │       └── api_key.rs # API key entity
│   │   ├── scanner/           # Scanner abstraction
│   │   │   ├── mod.rs
│   │   │   ├── traits.rs      # Scanner trait definition
│   │   │   └── fossology/     # Fossology implementation
│   │   │       ├── mod.rs     # FossologyScanner
│   │   │       ├── client.rs  # HTTP client for Fossology API
│   │   │       ├── parser.rs  # Result parsing & SPDX mapping
│   │   │       └── models.rs  # Fossology-specific types
│   │   ├── git/               # Git operations
│   │   │   ├── mod.rs
│   │   │   ├── clone.rs       # Repository cloning with auth
│   │   │   └── workspace.rs   # Temporary directory management
│   │   └── utils/             # Shared utilities
│   │       └── crypto.rs      # API key hashing
│   ├── migrations/            # SQLx migration files
│   │   ├── 20250101000000_initial_schema.sql
│   │   └── 20250102000000_add_git_token.sql
│   └── Cargo.toml             # Dependencies
├── legalscanner-ui/           # Vue 3 frontend
│   ├── src/
│   │   ├── main.js            # App initialization
│   │   ├── App.vue            # Root component
│   │   ├── api/               # API client layer
│   │   │   ├── client.js      # Axios instance with interceptors
│   │   │   └── scans.js       # Scan API calls
│   │   ├── components/        # Reusable components
│   │   │   ├── ScanForm.vue   # Scan creation form
│   │   │   ├── ScanList.vue   # List of scans
│   │   │   ├── ResultsViewer.vue # Results display
│   │   │   └── ApiKeyManager.vue # API key management
│   │   ├── views/             # Page components
│   │   │   ├── Home.vue       # Main page
│   │   │   ├── ScanDetails.vue # Scan details page
│   │   │   └── Settings.vue   # Settings page
│   │   ├── store/             # Pinia stores
│   │   │   ├── scans.js       # Scan state management
│   │   │   └── auth.js        # API key state
│   │   └── router/            # Vue Router
│   │       └── index.js       # Route definitions
│   └── package.json           # Dependencies
├── docker/                    # Docker configurations
│   ├── api.Dockerfile         # Multi-stage Rust build
│   ├── ui.Dockerfile          # Multi-stage Vue build
│   └── nginx.conf             # Nginx config for UI
├── docker-compose.yml         # Service orchestration
├── .env.example               # Environment template
└── CLAUDE.md                  # THIS FILE
```

---

## Core Architecture Principles

### 1. Separation of Concerns

#### Backend Layers (Strict Separation)
```
User Request
    ↓
API Handlers (api/handlers/) - HTTP concerns only
    ↓
Business Logic (scan_job.rs, db/models/) - Core operations
    ↓
Database Layer (db/models/) - Data persistence
    ↓
External Services (scanner/, git/) - Third-party integrations
```

**Rules:**
- API handlers should NOT contain business logic
- Database models should NOT know about HTTP
- Scanner implementations should NOT touch the database directly
- Git operations should NOT know about scans

### 2. Error Handling Pattern

```rust
// ALWAYS use this pattern
pub enum AppError {
    Database(#[from] sqlx::Error),
    Git(#[from] git2::Error),
    NotFound(String),
    Validation(String),
    Internal,
}

// Implement IntoResponse for Axum
impl IntoResponse for AppError { ... }

// Use throughout codebase
async fn handler() -> Result<Json<T>, AppError> {
    // Business logic
}
```

**Never:**
- Use `.unwrap()` or `.expect()` in production code
- Return generic error types
- Swallow errors silently

### 3. Database Patterns

```rust
// ALWAYS use async SQLx with typed queries
impl Model {
    pub async fn create(pool: &SqlitePool, ...) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO table (...) VALUES (...) RETURNING *"
        )
        .bind(...)
        .fetch_one(pool)
        .await
    }
}
```

**Rules:**
- All queries use query_as! or query! macros for compile-time checking
- Use transactions for multi-step operations
- Always use parameterized queries (SQL injection protection)
- RETURNING * pattern for insert/update operations

### 4. Scanner Trait Pattern (EXTENSIBLE)

```rust
#[async_trait]
pub trait Scanner: Send + Sync {
    fn name(&self) -> &str;
    async fn scan(&self, repo_path: &Path) -> Result<Vec<ScanResult>, ScanError>;
    async fn health_check(&self) -> Result<(), ScanError>;
}
```

**To Add a New Scanner:**
1. Create module in `src/scanner/your_scanner/`
2. Implement `Scanner` trait
3. Register in `main.rs`
4. **ASK USER BEFORE IMPLEMENTING**

### 5. Background Job Pattern

```rust
// Spawn async task for long-running operations
tokio::spawn(async move {
    execute_scan_job(scan_id, state).await;
});

// Return immediately to user
Ok((StatusCode::CREATED, Json(response)))
```

**Rules:**
- Long operations (git clone, scanning) MUST be async
- Update database status at each step
- Handle failures gracefully (update to 'failed' status)
- Clean up resources (temporary directories)

---

## API Conventions

### Endpoint Naming
```
GET    /api/v1/resource         - List
GET    /api/v1/resource/:id     - Get single
POST   /api/v1/resource         - Create
PUT    /api/v1/resource/:id     - Update (full)
PATCH  /api/v1/resource/:id     - Update (partial)
DELETE /api/v1/resource/:id     - Delete
```

### Response Formats

**Success (200/201):**
```json
{
  "scan_id": "uuid",
  "status": "pending",
  "data": { ... }
}
```

**Error (4xx/5xx):**
```json
{
  "error": "Short error message",
  "details": "Longer explanation"
}
```

### Authentication
- Header: `X-API-Key: lgs_...`
- Middleware validates on every request
- Hashed with Argon2, never exposed after creation

---

## Frontend Patterns

### Component Structure
```vue
<template>
  <!-- Presentation -->
</template>

<script setup>
// Composition API only
import { ref, computed, onMounted } from 'vue'
import { useStore } from '@/store/name'

// State
const state = ref(initialValue)

// Computed
const derived = computed(() => ...)

// Methods
const handleAction = async () => { ... }

// Lifecycle
onMounted(async () => { ... })
</script>

<style scoped>
/* Component-specific styles */
</style>
```

### State Management (Pinia)
```javascript
// stores/scans.js
export const useScansStore = defineStore('scans', () => {
  // State
  const scans = ref([])

  // Actions
  const fetchScans = async () => {
    const data = await scansApi.getScans()
    scans.value = data
  }

  return { scans, fetchScans }
})
```

**Rules:**
- Use Composition API (setup syntax)
- Pinia for global state, ref/reactive for local state
- API calls go through `/src/api` layer
- Error handling in stores, display in components

---

## Security Patterns

### API Key Security
1. Generate: `lgs_` prefix + 32 random bytes (base64)
2. Hash: Argon2id with random salt
3. Store: Only hash in database
4. Verify: Hash incoming key, compare with stored hash
5. Never log or expose keys

### Git Token Security
1. Accept per-scan or via environment variable
2. Store in database but **NEVER return in API responses**
3. Use `#[serde(skip_serializing)]` on git_token field
4. Pass to git2 only during clone operation
5. Clean from memory after use

### Input Validation
```rust
// Validate all user input
if payload.git_url.is_empty() {
    return Err(AppError::Validation("URL required".into()));
}

git::validate_git_url(&payload.git_url)
    .map_err(|e| AppError::Validation(e))?;
```

---

## Docker & Deployment

### Multi-Stage Builds
- **API**: Builder (Rust) → Runtime (Debian slim)
- **UI**: Builder (Node) → Runtime (Nginx)
- Caching strategy: Copy Cargo.toml first, then source

### Health Checks
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
  interval: 30s
  timeout: 10s
  retries: 3
```

### Environment Variables
- Use `.env` file for Docker Compose
- Load with `dotenvy` crate in Rust
- Pass as build args or runtime env vars
- Never commit secrets (`.env` is gitignored)

---

## Database Schema

### Current Tables

**scans**
```sql
CREATE TABLE scans (
    id TEXT PRIMARY KEY,
    git_url TEXT NOT NULL,
    git_token TEXT,              -- Per-scan token (encrypted, never returned)
    status TEXT NOT NULL,        -- 'pending', 'in_progress', 'completed', 'failed'
    error_message TEXT,
    created_at DATETIME,
    started_at DATETIME,
    completed_at DATETIME,
    created_by_key_id TEXT
);
```

**scan_results**
```sql
CREATE TABLE scan_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    result_type TEXT NOT NULL,   -- 'license' or 'copyright'

    -- License fields
    license_name TEXT,
    license_spdx_id TEXT,
    confidence REAL,

    -- Copyright fields
    copyright_statement TEXT,
    copyright_holders TEXT,      -- JSON array
    copyright_years TEXT,        -- JSON array

    raw_data TEXT,               -- Original scanner output
    FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
);
```

**api_keys**
```sql
CREATE TABLE api_keys (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    key_hash TEXT NOT NULL UNIQUE,  -- Argon2 hash
    created_at DATETIME,
    last_used_at DATETIME,
    is_active BOOLEAN DEFAULT 1
);
```

### Migration Strategy
- Use SQLx migrations: `sqlx migrate add <name>`
- Migrations are run automatically on startup
- **NEVER** edit existing migrations (create new ones)
- Test migrations with `sqlx migrate run`

---

## Common Workflows

### Adding a New API Endpoint

1. **Define Request/Response Models** (`api/models/mod.rs`)
```rust
#[derive(Deserialize)]
pub struct CreateXRequest {
    pub field: String,
}

#[derive(Serialize)]
pub struct XResponse {
    pub id: String,
    pub field: String,
}
```

2. **Create Handler** (`api/handlers/x.rs`)
```rust
pub async fn create_x(
    State(state): State<AppState>,
    Json(payload): Json<CreateXRequest>,
) -> Result<Json<XResponse>, AppError> {
    // Validation
    // Business logic
    // Return response
}
```

3. **Register Route** (`api/routes.rs`)
```rust
.route("/api/v1/x", post(handlers::x::create_x))
```

4. **⚠️ ASK USER before implementing**

### Adding a Database Field

1. **Create Migration**
```bash
cd legalscanner-api
sqlx migrate add add_field_to_table
```

2. **Write SQL** (`migrations/YYYYMMDDHHMMSS_add_field_to_table.sql`)
```sql
ALTER TABLE table_name ADD COLUMN new_field TEXT;
```

3. **Update Model** (add field to struct)
4. **Update Queries** (include new field)
5. **Test Migration**
6. **⚠️ ASK USER before implementing**

### Handling Stuck Scans (Edge Case)

**Problem**: Container restart kills background jobs, leaving scans in "in_progress"

**Current Solution**: Manual database update

**Potential Enhancement** (requires approval):
```rust
// In main.rs after migrations
async fn cleanup_stuck_scans(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE scans
         SET status = 'failed',
             error_message = 'Interrupted by server restart',
             completed_at = datetime('now')
         WHERE status IN ('pending', 'in_progress')"
    )
    .execute(pool)
    .await?;
    Ok(())
}
```

**⚠️ ASK USER before implementing this**

---

## Testing Strategy

### Backend Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_scan() {
        // Arrange
        let pool = create_test_db().await;

        // Act
        let result = Scan::create(&pool, url, None, None).await;

        // Assert
        assert!(result.is_ok());
    }
}
```

**Run:**
```bash
cargo test
cargo test --package legalscanner-api
```

### Frontend Tests
```javascript
import { describe, it, expect } from 'vitest'

describe('ScanForm', () => {
  it('validates URL input', () => {
    // Test implementation
  })
})
```

**Run:**
```bash
npm run test
npm run test:unit
```

---

## Extending the System

### Adding a New Scanner (e.g., ScanCode)

**Steps:**
1. Create `src/scanner/scancode/` directory
2. Implement Scanner trait:
```rust
pub struct ScanCodeScanner {
    base_url: String,
}

#[async_trait]
impl Scanner for ScanCodeScanner {
    fn name(&self) -> &str { "scancode" }

    async fn scan(&self, repo_path: &Path) -> Result<Vec<ScanResult>, ScanError> {
        // Implementation
    }

    async fn health_check(&self) -> Result<(), ScanError> {
        // Check ScanCode availability
    }
}
```

3. **Register in main.rs** (OR make registry dynamic)
4. Update docker-compose.yml if new container needed

**⚠️ MANDATORY: Present plan to user first**

### Adding New Legal Scanning Capabilities

**Current Coverage:**
- Licenses (Nomos, Monk, Ojo)
- Copyrights (emails, authors, years)

**Planned Enhancements** (require approval):
- Export Control Classification (ECC agent)
- Keyword detection (patents, trademarks, confidential markers)
- License compatibility analysis
- SPDX/SBOM export

**Process:**
1. Research Fossology capability
2. Extend database schema if needed
3. Update FossologyClient to enable agent
4. Parse new result types
5. Update UI to display results
6. **PRESENT PLAN TO USER FIRST**

---

## Debugging & Troubleshooting

### Backend Logs
```bash
# All services
docker-compose logs -f

# API only
docker-compose logs -f api

# Last 50 lines
docker-compose logs --tail=50 api

# Follow with grep
docker-compose logs -f api | grep ERROR
```

### Database Inspection
```bash
# Access database
sqlite3 data/legalscanner.db

# Common queries
SELECT * FROM scans ORDER BY created_at DESC LIMIT 10;
SELECT COUNT(*) FROM scan_results WHERE scan_id = 'xxx';
SELECT DISTINCT license_name FROM scan_results WHERE result_type = 'license';
```

### Container Access
```bash
# Shell into API container
docker-compose exec api /bin/bash

# Shell into UI container
docker-compose exec ui /bin/sh

# Check workspace
docker-compose exec api ls -la /app/tmp
```

### Common Issues

**"Scan stuck in in_progress"**
→ Container restart killed background job
→ Manually update database or implement cleanup function

**"Authentication required but no callback set"**
→ Private repository needs git_token
→ Provide token via UI or environment variable

**"Database locked"**
→ SQLite doesn't handle high concurrency
→ Sequential scans only, or migrate to PostgreSQL

**"Fossology job fails"**
→ Check Fossology logs: `docker-compose logs fossology`
→ Verify Fossology API token is set correctly

---

## Code Style Guidelines

### Rust
- Use `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- Prefer `Result<T, E>` over `Option<T>` for fallible operations
- Use `thiserror` for custom error types
- Async all the way (no blocking operations)
- Document public APIs with `///` comments

### JavaScript/Vue
- Use `npm run lint` before committing
- Prefer Composition API over Options API
- Use `async/await` over Promise chains
- Destructure imports
- Use computed() for derived state
- Keep components under 200 lines

### SQL
- UPPERCASE keywords (SELECT, FROM, WHERE)
- Snake_case for table and column names
- Always use explicit column names (not SELECT *)
- Parameterize queries (use $1, $2, etc.)

---

## Performance Considerations

### Backend
- Connection pooling (SQLx pool size: 5)
- Background jobs for long operations
- Stream large responses
- Batch database operations where possible

### Frontend
- Lazy load routes
- Virtual scrolling for large lists
- Debounce user input
- Cache API responses (if stale is acceptable)

### Scanning
- One scan at a time (SQLite limitation)
- Cleanup workspace after scan
- Poll Fossology with exponential backoff
- Stream large repositories in chunks

---

## Deployment Checklist

Before deploying to production:

- [ ] Change `API_KEY_SALT` in `.env` to random secure value
- [ ] Set strong `FOSSOLOGY_API_TOKEN`
- [ ] Configure proper logging (RUST_LOG=info)
- [ ] Set up backup for `data/` directory (SQLite database)
- [ ] Configure firewall (only expose necessary ports)
- [ ] Enable HTTPS (add reverse proxy like Traefik/Caddy)
- [ ] Set up monitoring (health check endpoint)
- [ ] Configure log rotation
- [ ] Document restore procedure
- [ ] Test disaster recovery

---

## When to Deviate

You MAY deviate from this architecture ONLY if:

1. **Critical bug** requires immediate breaking change
2. **Security vulnerability** needs urgent fix
3. **User explicitly approves** architectural change

In ALL cases:
- Document the deviation in commit message
- Update this CLAUDE.md file
- Explain reasoning to user

---

## Summary: Golden Rules

1. ✅ **DO**: Follow established patterns
2. ✅ **DO**: Ask before adding dependencies
3. ✅ **DO**: Present plans for major changes
4. ✅ **DO**: Use type-safe error handling
5. ✅ **DO**: Write async code throughout

6. ❌ **DON'T**: Change tech stack without approval
7. ❌ **DON'T**: Add new architectural layers
8. ❌ **DON'T**: Skip error handling
9. ❌ **DON'T**: Block async operations
10. ❌ **DON'T**: Commit secrets or .env files

---

## Contact & Support

For questions about this architecture:
- Refer to existing code as examples
- Check README.md for user-facing docs
- Ask user for clarification on ambiguous requirements
- When proposing changes, present trade-offs

**This architecture has been carefully designed. Respect it.**
