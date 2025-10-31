-- Create scans table
CREATE TABLE IF NOT EXISTS scans (
    id TEXT PRIMARY KEY,
    git_url TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending', 'in_progress', 'completed', 'failed')),
    error_message TEXT,
    created_at DATETIME DEFAULT (datetime('now')),
    started_at DATETIME,
    completed_at DATETIME,
    created_by_key_id TEXT,
    FOREIGN KEY (created_by_key_id) REFERENCES api_keys(id) ON DELETE SET NULL
);

-- Create scan_results table
CREATE TABLE IF NOT EXISTS scan_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    result_type TEXT NOT NULL CHECK(result_type IN ('license', 'copyright')),
    license_name TEXT,
    license_spdx_id TEXT,
    copyright_statement TEXT,
    copyright_holders TEXT, -- JSON array
    copyright_years TEXT,   -- JSON array
    confidence REAL,
    raw_data TEXT,          -- Original scanner output (JSON)
    FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
);

-- Create licenses summary table
CREATE TABLE IF NOT EXISTS licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id TEXT NOT NULL,
    name TEXT NOT NULL,
    spdx_id TEXT,
    file_count INTEGER DEFAULT 1,
    FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
);

-- Create copyrights summary table
CREATE TABLE IF NOT EXISTS copyrights (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id TEXT NOT NULL,
    statement TEXT NOT NULL,
    file_count INTEGER DEFAULT 1,
    FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
);

-- Create api_keys table
CREATE TABLE IF NOT EXISTS api_keys (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    key_hash TEXT NOT NULL UNIQUE,
    created_at DATETIME DEFAULT (datetime('now')),
    last_used_at DATETIME,
    is_active BOOLEAN DEFAULT 1
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_scans_status ON scans(status);
CREATE INDEX IF NOT EXISTS idx_scans_created_at ON scans(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_scan_results_scan_id ON scan_results(scan_id);
CREATE INDEX IF NOT EXISTS idx_scan_results_type ON scan_results(result_type);
CREATE INDEX IF NOT EXISTS idx_scan_results_file_path ON scan_results(file_path);
CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_api_keys_active ON api_keys(is_active);
