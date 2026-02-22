use std::io;
use std::path::Path;

/// Load source file contents from disk.
pub fn load_source(path: &Path) -> io::Result<String> {
    // TODO(step-3): add path-specific error context for CLI reporting.
    std::fs::read_to_string(path)
}
