use std::path::PathBuf;

pub struct Source {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub position: usize,
}
