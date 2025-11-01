-- Add individual scanner status columns to track parallel scanning progress
ALTER TABLE scans ADD COLUMN fossology_status TEXT DEFAULT 'pending' CHECK(fossology_status IN ('pending', 'in_progress', 'completed', 'failed'));
ALTER TABLE scans ADD COLUMN semgrep_status TEXT DEFAULT 'pending' CHECK(semgrep_status IN ('pending', 'in_progress', 'completed', 'failed'));

-- Add timestamps for individual scanner operations
ALTER TABLE scans ADD COLUMN fossology_started_at DATETIME;
ALTER TABLE scans ADD COLUMN fossology_completed_at DATETIME;
ALTER TABLE scans ADD COLUMN semgrep_started_at DATETIME;
ALTER TABLE scans ADD COLUMN semgrep_completed_at DATETIME;

-- Add error messages for individual scanners
ALTER TABLE scans ADD COLUMN fossology_error TEXT;
ALTER TABLE scans ADD COLUMN semgrep_error TEXT;

-- Update existing scans to have completed status for both scanners if overall status is completed
UPDATE scans SET
    fossology_status = CASE
        WHEN status = 'completed' THEN 'completed'
        WHEN status = 'failed' THEN 'failed'
        ELSE 'pending'
    END,
    semgrep_status = CASE
        WHEN status = 'completed' THEN 'completed'
        WHEN status = 'failed' THEN 'failed'
        ELSE 'pending'
    END
WHERE 1=1;
