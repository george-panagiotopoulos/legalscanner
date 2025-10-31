-- Add git_token column to scans table for per-scan authentication
ALTER TABLE scans ADD COLUMN git_token TEXT;
