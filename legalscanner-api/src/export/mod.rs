pub mod spdx;

use serde::{Deserialize, Serialize};

/// SBOM export format
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SbomFormat {
    Json,
    Yaml,
}

impl SbomFormat {
    pub fn content_type(&self) -> &'static str {
        match self {
            SbomFormat::Json => "application/json",
            SbomFormat::Yaml => "application/x-yaml",
        }
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            SbomFormat::Json => "json",
            SbomFormat::Yaml => "yaml",
        }
    }
}

impl Default for SbomFormat {
    fn default() -> Self {
        SbomFormat::Json
    }
}
