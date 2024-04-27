// Helper functions without a better home.

use std::io::Read;

use crate::structs::ZakopaneError;

// Ingests the contents of a file.
pub fn ingest_file(path: std::path::PathBuf) -> Result<String, ZakopaneError> {
    let mut file = std::fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
