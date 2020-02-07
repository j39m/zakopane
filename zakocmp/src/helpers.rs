// Helper functions without a better home.

use std::io::Read;

use crate::structs::ZakocmpError;

// Ingests the contents of a file.
pub fn ingest_file(path: &str) -> Result<String, ZakocmpError> {
    let mut file = std::fs::File::open(std::path::Path::new(path)).map_err(ZakocmpError::Io)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(ZakocmpError::Io)?;
    Ok(contents)
}
