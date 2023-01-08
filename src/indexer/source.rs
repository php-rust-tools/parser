use std::path::PathBuf;

#[derive(Debug)]
pub struct Source {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub position: usize,
}
