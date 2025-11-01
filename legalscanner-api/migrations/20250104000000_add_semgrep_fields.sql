-- Add Semgrep-specific fields for ECC findings
-- This enables storing source, line numbers, and check IDs from Semgrep

-- SQLite doesn't support ALTER TABLE ADD COLUMN with constraints easily,
-- so we need to recreate the table

-- Create new table with Semgrep fields
CREATE TABLE scan_results_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    result_type TEXT NOT NULL CHECK(result_type IN ('license', 'copyright', 'ecc')),
    license_name TEXT,
    license_spdx_id TEXT,
    copyright_statement TEXT,
    copyright_holders TEXT,
    copyright_years TEXT,
    confidence REAL,
    raw_data TEXT,
    risk_severity TEXT CHECK(risk_severity IN ('low', 'medium', 'high', 'critical')),
    ecc_source TEXT,          -- Source scanner (e.g., 'semgrep', 'scancode')
    ecc_line_number INTEGER,  -- Line number where finding was detected
    ecc_check_id TEXT,        -- Rule/check ID from scanner
    FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
);

-- Copy existing data
INSERT INTO scan_results_new (
    id, scan_id, file_path, result_type, license_name, license_spdx_id,
    copyright_statement, copyright_holders, copyright_years, confidence, raw_data, risk_severity
)
SELECT
    id, scan_id, file_path, result_type, license_name, license_spdx_id,
    copyright_statement, copyright_holders, copyright_years, confidence, raw_data, risk_severity
FROM scan_results;

-- Drop old table
DROP TABLE scan_results;

-- Rename new table
ALTER TABLE scan_results_new RENAME TO scan_results;
