-- Add risk scoring fields to scans table
ALTER TABLE scans ADD COLUMN risk_score INTEGER;
ALTER TABLE scans ADD COLUMN risk_level TEXT CHECK(risk_level IN ('low', 'medium', 'high', 'critical'));
ALTER TABLE scans ADD COLUMN risk_factors TEXT; -- JSON array

-- Create risk_config table for configurable license risk weights
CREATE TABLE IF NOT EXISTS risk_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    license_pattern TEXT NOT NULL UNIQUE, -- License name or pattern (e.g., 'GPL%', 'MIT', 'Apache-2.0')
    risk_weight INTEGER NOT NULL,         -- Points to add to risk score
    category TEXT NOT NULL CHECK(category IN ('copyleft', 'permissive', 'proprietary', 'unknown', 'other')),
    description TEXT,
    created_at DATETIME DEFAULT (datetime('now')),
    updated_at DATETIME DEFAULT (datetime('now'))
);

-- Seed risk_config with default license risk weights
-- High-risk copyleft licenses
INSERT INTO risk_config (license_pattern, risk_weight, category, description) VALUES
    ('GPL-3.0%', 10, 'copyleft', 'GPLv3 strong copyleft - may require releasing derivative works'),
    ('GPL-2.0%', 10, 'copyleft', 'GPLv2 strong copyleft - may require releasing derivative works'),
    ('GPL', 10, 'copyleft', 'GPL without version specification - ambiguous and risky'),
    ('AGPL%', 12, 'copyleft', 'Affero GPL - strongest copyleft, triggers on network use'),
    ('Sleepycat', 10, 'copyleft', 'Sleepycat license - copyleft requirements');

-- Medium-risk weak copyleft licenses
INSERT INTO risk_config (license_pattern, risk_weight, category, description) VALUES
    ('LGPL%', 5, 'copyleft', 'Lesser GPL - weak copyleft, library-friendly'),
    ('MPL%', 5, 'copyleft', 'Mozilla Public License - weak copyleft, file-level'),
    ('EPL%', 5, 'copyleft', 'Eclipse Public License - weak copyleft'),
    ('CDDL%', 5, 'copyleft', 'Common Development and Distribution License - weak copyleft'),
    ('CPL%', 5, 'copyleft', 'Common Public License - weak copyleft');

-- Proprietary/commercial licenses
INSERT INTO risk_config (license_pattern, risk_weight, category, description) VALUES
    ('%Proprietary%', 15, 'proprietary', 'Proprietary license - usage restrictions likely'),
    ('%Commercial%', 15, 'proprietary', 'Commercial license - may require payment or agreement');

-- Unknown/unrecognized
INSERT INTO risk_config (license_pattern, risk_weight, category, description) VALUES
    ('No_license_found', 8, 'unknown', 'No license detected - unclear usage rights'),
    ('See-file', 6, 'unknown', 'License in separate file - requires manual review'),
    ('Unknown%', 8, 'unknown', 'Unrecognized license - requires manual review'),
    ('%possibility', 6, 'unknown', 'Uncertain license detection - low confidence');

-- Low-risk permissive licenses (explicitly set to 0 for clarity)
INSERT INTO risk_config (license_pattern, risk_weight, category, description) VALUES
    ('MIT', 0, 'permissive', 'MIT License - very permissive'),
    ('Apache-2.0', 0, 'permissive', 'Apache License 2.0 - permissive with patent grant'),
    ('Apache', 0, 'permissive', 'Apache License - permissive'),
    ('BSD%', 0, 'permissive', 'BSD License family - permissive'),
    ('ISC', 0, 'permissive', 'ISC License - permissive, similar to MIT'),
    ('0BSD', 0, 'permissive', 'Zero-Clause BSD - public domain equivalent'),
    ('CC0%', 0, 'permissive', 'Creative Commons Zero - public domain dedication'),
    ('Unlicense', 0, 'permissive', 'Unlicense - public domain dedication'),
    ('WTFPL', 0, 'permissive', 'Do What The F*ck You Want - permissive'),
    ('Zlib', 0, 'permissive', 'Zlib License - permissive'),
    ('Curl', 0, 'permissive', 'Curl License - permissive'),
    ('Libpng', 0, 'permissive', 'Libpng License - permissive'),
    ('Python%', 0, 'permissive', 'Python Software Foundation License - permissive'),
    ('CC-BY%', 1, 'other', 'Creative Commons Attribution - requires attribution'),
    ('AFL%', 0, 'permissive', 'Academic Free License - permissive'),
    ('BlueOak%', 0, 'permissive', 'Blue Oak Model License - permissive'),
    ('Boost%', 0, 'permissive', 'Boost Software License - permissive'),
    ('PostgreSQL', 0, 'permissive', 'PostgreSQL License - permissive');

-- Create index for faster pattern matching
CREATE INDEX IF NOT EXISTS idx_risk_config_pattern ON risk_config(license_pattern);
