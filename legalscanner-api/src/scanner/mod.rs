pub mod fossology;
pub mod semgrep;
pub mod traits;

pub use traits::{CopyrightFinding, EccFinding, LicenseFinding, ScanError, ScanResult, Scanner};
