use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn find_request_files<P: AsRef<Path>>(dir: P) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "yaml" || ext == "yml" || ext == "json" {
                    files.push(entry.path().to_path_buf());
                }
            }
        }
    }
    
    Ok(files)
}

pub fn is_request_file<P: AsRef<Path>>(path: P) -> bool {
    if let Some(ext) = path.as_ref().extension() {
        ext == "yaml" || ext == "yml" || ext == "json"
    } else {
        false
    }
}
