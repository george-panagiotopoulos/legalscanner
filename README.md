# Legal Scanner

A comprehensive tool to scan Git repositories for legal and compliance issues, identifying open source licenses, copyright statements, and supporting private repository authentication.

## Features

- **License Detection**: Identify all open source licenses using multiple scanners (Nomos, Monk, Ojo)
- **Copyright Detection**: Extract all copyright statements, holders, and years
- **Export Control & Security Analysis**: Detect cryptographic implementations and export control-relevant code patterns using Semgrep
- **Private Repository Support**: Scan private repositories with GitHub Personal Access Tokens
- **REST API**: Programmatic access for automation and CI/CD integration
- **Web UI**: User-friendly interface with real-time scan monitoring
- **Extensible Architecture**: Plugin system for adding new scanners
- **Docker-based**: One-command deployment with Docker Compose
- **SPDX Mapping**: Automatic mapping to SPDX license identifiers

## Technology Stack

- **Backend**: Rust + Axum web framework
- **Frontend**: Vue 3 + Vite + Pinia
- **Scanner Engines**:
  - Fossology (license and copyright detection)
  - Semgrep (export control and security pattern analysis)
- **Database**: SQLite with async SQLx
- **Git Operations**: libgit2 with authentication support
- **Deployment**: Docker Compose with health checks

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Git
- (Optional) GitHub Personal Access Token for private repositories

### Installation

1. Clone the repository:
```bash
git clone https://github.com/george-panagiotopoulos/legalscanner
cd legalscanner
```

2. Copy the environment file:
```bash
cp .env.example .env
```

3. Edit `.env` and configure:
   - `API_KEY_SALT`: Change to a random string for production
   - `FOSSOLOGY_API_TOKEN`: Automatically generated on first Fossology start
   - `GIT_TOKEN`: (Optional) GitHub token for private repos, or provide per-scan via UI

4. Start all services:
```bash
docker-compose up -d
```

5. Wait for services to be healthy (Fossology takes 2-3 minutes on first start):
```bash
docker-compose ps
```

### Access

- **Web UI**: http://localhost:5300
- **API**: http://localhost:5301
- **Fossology**: http://localhost:5302
- **API Health Check**: http://localhost:5301/health

## Usage

### Via Web UI (Recommended)

1. Navigate to http://localhost:5300
2. Go to **Settings** and create an API key
3. Return to Home and enter a Git repository URL
4. **For private repositories**:
   - Click the GitHub token link to create a Personal Access Token
   - Paste the token in the "Git Access Token (Optional)" field
5. Click "Start Scan"
6. Monitor scan progress in real-time
7. View detailed results with licenses and copyrights per file

### Via REST API

#### 1. Create an API Key

```bash
curl -X POST http://localhost:5301/api/v1/api-keys \
  -H "Content-Type: application/json" \
  -d '{"name": "My API Key"}'
```

Response:
```json
{
  "id": "...",
  "name": "My API Key",
  "key": "lgs_...",
  "created_at": "...",
  "message": "Save this key securely. It will not be shown again."
}
```

#### 2. Create a Scan (Public Repository)

```bash
curl -X POST http://localhost:5301/api/v1/scans \
  -H "Content-Type: application/json" \
  -H "X-API-Key: lgs_..." \
  -d '{"git_url": "https://github.com/user/repo.git"}'
```

#### 3. Create a Scan (Private Repository)

```bash
curl -X POST http://localhost:5301/api/v1/scans \
  -H "Content-Type: application/json" \
  -H "X-API-Key: lgs_..." \
  -d '{
    "git_url": "https://github.com/user/private-repo.git",
    "git_token": "ghp_your_github_token_here"
  }'
```

Response:
```json
{
  "scan_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "pending",
  "created_at": "2025-10-30T10:30:00Z",
  "git_url": "https://github.com/user/repo.git"
}
```

#### 4. Check Scan Status

```bash
curl http://localhost:5301/api/v1/scans/550e8400-e29b-41d4-a716-446655440000 \
  -H "X-API-Key: lgs_..."
```

#### 5. Get Results

```bash
curl http://localhost:5301/api/v1/scans/550e8400-e29b-41d4-a716-446655440000/results \
  -H "X-API-Key: lgs_..."
```

Response:
```json
{
  "scan_id": "550e8400-e29b-41d4-a716-446655440000",
  "repository_url": "https://github.com/user/repo.git",
  "scan_date": "2025-10-30T10:30:00Z",
  "status": "completed",
  "results": {
    "licenses": [
      {
        "file_path": "src/main.rs",
        "license": "MIT License",
        "spdx_id": "MIT",
        "confidence": 0.98
      }
    ],
    "copyrights": [
      {
        "file_path": "src/main.rs",
        "statement": "Copyright (c) 2025 John Doe",
        "holders": ["John Doe"],
        "years": ["2025"]
      }
    ]
  }
}
```

## Private Repository Authentication

### Option 1: Per-Scan Token (Recommended)

Provide a GitHub Personal Access Token directly when creating a scan via the UI or API. This is the most secure approach as tokens are stored encrypted and never exposed in API responses.

**Create a GitHub Token:**
1. Go to https://github.com/settings/tokens
2. Click "Generate new token (classic)"
3. Select scope: `repo` (Full control of private repositories)
4. Generate and copy the token

### Option 2: Global Environment Variable

Set `GIT_TOKEN` in your `.env` file to apply to all scans without a specific token.

## API Documentation

### Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check (no auth required) |
| POST | `/api/v1/scans` | Create new scan |
| GET | `/api/v1/scans` | List all scans |
| GET | `/api/v1/scans/:id` | Get scan details with summary |
| GET | `/api/v1/scans/:id/results` | Get detailed scan results |
| DELETE | `/api/v1/scans/:id` | Delete scan and results |
| POST | `/api/v1/api-keys` | Create API key |
| GET | `/api/v1/api-keys` | List API keys |
| DELETE | `/api/v1/api-keys/:id` | Delete API key |

### Authentication

All API endpoints (except `/health`) require authentication via API key header:

```
X-API-Key: lgs_your_api_key_here
```

API keys are hashed with Argon2 before storage and never exposed after creation.

## Development

### Project Structure

```
legalScanner/
├── legalscanner-api/          # Rust backend
│   ├── src/
│   │   ├── main.rs            # Application entry point
│   │   ├── api/               # REST API handlers and routes
│   │   ├── db/                # Database models and migrations
│   │   ├── scanner/           # Scanner abstraction and Fossology client
│   │   ├── git/               # Git operations with authentication
│   │   ├── error.rs           # Error types
│   │   ├── config.rs          # Configuration management
│   │   └── utils/             # Crypto and utilities
│   └── migrations/            # SQLx migrations
├── legalscanner-ui/           # Vue 3 frontend
│   ├── src/
│   │   ├── components/        # Vue components
│   │   ├── views/             # Page views
│   │   ├── store/             # Pinia state management
│   │   ├── api/               # API client
│   │   └── router/            # Vue Router
├── docker/                    # Dockerfiles
│   ├── api.Dockerfile         # Backend container
│   ├── ui.Dockerfile          # Frontend container
│   └── nginx.conf             # Nginx configuration for UI
├── docker-compose.yml         # Service orchestration
├── CLAUDE.md                  # Architecture documentation for AI
└── README.md                  # This file
```

### Local Development

#### Backend (Rust)

```bash
cd legalscanner-api

# Install dependencies (first time only)
cargo build

# Run locally (requires Fossology running)
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

#### Frontend (Vue)

```bash
cd legalscanner-ui

# Install dependencies
npm install

# Development server with hot reload
npm run dev

# Build for production
npm run build

# Lint
npm run lint
```

### Environment Variables

See `.env.example` for all configuration options:

- `API_PORT`: API server port (default: 5301)
- `UI_PORT`: Web UI port (default: 5300)
- `FOSSOLOGY_PORT`: Fossology port (default: 5302)
- `DATABASE_URL`: SQLite database path
- `FOSSOLOGY_API_TOKEN`: Fossology JWT token
- `GIT_TOKEN`: Optional global GitHub token
- `API_KEY_SALT`: Salt for API key hashing (change in production!)
- `RUST_LOG`: Logging level (info, debug, trace)

## Architecture

### System Components

1. **Rust API** (Port 5301)
   - Axum web framework for REST API
   - SQLx for async database operations
   - Background job processing for scans
   - Git operations with libgit2

2. **Vue UI** (Port 5300)
   - Vue 3 with Composition API
   - Pinia for state management
   - Vite for build tooling
   - Axios for API communication

3. **Fossology** (Port 5302)
   - License scanning (Nomos, Monk, Ojo agents)
   - Copyright detection
   - SPDX identifier extraction
   - REST API for job submission

4. **Semgrep**
   - Static analysis for security patterns
   - Cryptographic implementation detection
   - Export control compliance checking
   - Custom ruleset for legal compliance

5. **SQLite Database**
   - Stores scans, results, API keys
   - Async operations via SQLx
   - Automatic migrations

### Scan Workflow

1. **Initiation**: User submits Git URL (and optional token) via UI or API
2. **Git Clone**: Repository cloned to temporary workspace with authentication if needed
3. **Parallel Scanning**:
   - **Fossology**: Files uploaded to Fossology for license and copyright analysis (nomos, monk, ojo, copyright agents)
   - **Semgrep**: Repository scanned for cryptographic implementations and export control patterns
4. **Result Retrieval**: API polls both scanners for job completion
5. **Parsing**: Results normalized to standard format with SPDX mapping and security classifications
6. **Storage**: Licenses, copyrights, and security findings stored per-file in database
7. **Cleanup**: Temporary workspace deleted
8. **Display**: Comprehensive results available via API and UI with filtering capabilities

### Security Features

- API keys hashed with Argon2
- Git tokens encrypted in database, never exposed in responses
- Private token field in UI (password input)
- CORS protection
- Input validation
- SQL injection protection (SQLx parameterized queries)

## Extending the Scanner

The system uses a trait-based architecture for extensibility. To add a new scanner:

1. Implement the `Scanner` trait in `legalscanner-api/src/scanner/traits.rs`
2. Add your scanner module to `src/scanner/`
3. Register it in `main.rs`

Example:

```rust
use async_trait::async_trait;
use crate::scanner::traits::{Scanner, ScanResult, ScanError};

pub struct CustomScanner {
    config: String,
}

#[async_trait]
impl Scanner for CustomScanner {
    fn name(&self) -> &str {
        "custom-scanner"
    }

    async fn scan(&self, repo_path: &Path) -> Result<Vec<ScanResult>, ScanError> {
        // Your scanning logic here
        Ok(vec![])
    }

    async fn health_check(&self) -> Result<(), ScanError> {
        // Health check implementation
        Ok(())
    }
}
```

## Troubleshooting

### Fossology takes long to start

Fossology requires 2-5 minutes to initialize on first start. Monitor progress:

```bash
docker-compose logs -f fossology
```

Wait for: `fossology_1 | Apache started successfully`

### Scans fail with authentication errors

For private repositories, ensure you:
1. Created a GitHub Personal Access Token with `repo` scope
2. Provided the token in the UI or API request
3. Token hasn't expired (check on GitHub)

### Scans stuck in "in_progress" after restart

When containers restart, background jobs are lost. Stuck scans are automatically marked as failed on next API start.

### API container won't start

Check logs for database errors:

```bash
docker-compose logs api
```

Common issues:
- Database file permissions (ensure `data/` directory is writable)
- Missing migrations (check `legalscanner-api/migrations/`)

### Permission issues with workspace

Ensure workspace directory exists and is writable:

```bash
docker-compose exec api ls -la /app/tmp
```

## Performance Considerations

- **Scan Duration**: 2-10 minutes depending on repository size
- **Concurrency**: Sequential scans recommended (SQLite limitation)
- **Repository Size**: Tested up to 10,000 files
- **Memory**: 2GB minimum, 4GB recommended
- **Fossology**: Most resource-intensive component

For high-load production environments:
- Consider migrating to PostgreSQL
- Implement job queue (RabbitMQ, Redis)
- Add horizontal scaling for API workers

## Roadmap

### Phase 1 (Current)
- ✅ License detection (Nomos, Monk, Ojo)
- ✅ Copyright detection
- ✅ Private repository support
- ✅ REST API
- ✅ Web UI
- ✅ SPDX mapping

### Phase 2 (Planned)
- [ ] Export Control Classification (ECC) scanning
- [ ] Keyword detection (patents, trademarks, confidential markers)
- [ ] License compatibility analysis
- [ ] SPDX/SBOM export
- [ ] Enhanced reporting with risk severity

### Phase 3 (Future)
- [ ] Additional scanners (ScanCode, Black Duck)
- [ ] CVE/security vulnerability scanning
- [ ] Webhook notifications
- [ ] Scheduled scans
- [ ] CI/CD integration plugins
- [ ] Multi-tenancy and user management

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Follow Rust and Vue style guides
4. Add tests for new features
5. Submit a pull request

For architecture decisions, consult `CLAUDE.md`.

## License

MIT License - See LICENSE file for details.

## Support

- **Issues**: Create an issue on GitHub
- **Architecture**: See CLAUDE.md for detailed code documentation
- **Questions**: Open a discussion on GitHub

---

**Built with ❤️ using Rust, Vue 3, and Fossology**
