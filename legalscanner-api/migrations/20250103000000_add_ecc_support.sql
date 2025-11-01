-- Add ECC (Export Control Classification) support to scan results
-- This enables detection of export-controlled technologies (cryptography, etc.)

-- SQLite doesn't support ALTER TABLE DROP/ADD CONSTRAINT, so we need to:
-- 1. Create a new table with the updated schema
-- 2. Copy data from old table
-- 3. Drop old table
-- 4. Rename new table

-- Create new table with updated schema
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
    FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
);

-- Copy existing data
INSERT INTO scan_results_new (
    id, scan_id, file_path, result_type, license_name, license_spdx_id,
    copyright_statement, copyright_holders, copyright_years, confidence, raw_data
)
SELECT
    id, scan_id, file_path, result_type, license_name, license_spdx_id,
    copyright_statement, copyright_holders, copyright_years, confidence, raw_data
FROM scan_results;

-- Drop old table
DROP TABLE scan_results;

-- Rename new table
ALTER TABLE scan_results_new RENAME TO scan_results;
