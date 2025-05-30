use anyhow::{Context, Result};
use walkdir::WalkDir;
use std::path::{Path, PathBuf};

use super::{RequestDefinition, ValidationError};

pub struct RequestParser;

impl RequestParser {
   pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<RequestDefinition, ValidationError> {
      let path = path.as_ref(); 

        // if path.exists() {
        //     return Err(ValidationError::FileNotFound(path.display().to_string()));
        // }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file {}", path.display())).unwrap();

        let request = match path.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => Self::parse_yaml(&content),
            Some("json") => Self::parse_json(&content),
            Some(ext) => return Err(ValidationError::UnsupportedFormat(ext.to_string())),
            None => {
                // Try to detect format from content
                if content.trim_start().starts_with('{') {
                    Self::parse_json(&content)
                } else {
                    Self::parse_yaml(&content)
                }
            }
        };

        Ok(request.unwrap())
   } 

    pub fn parse_directory<P: AsRef<Path>>(dir: P) -> Vec<(PathBuf, Result<RequestDefinition, ValidationError>)> {
        let mut results = Vec::new();

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "yaml" || ext == "yml" || ext == "json" {
                        let path = entry.path().to_path_buf();
                        let result = Self::parse_file(&path);
                        results.push((path, result));
                    }
                }
            }
        }

        results
    }

  fn parse_yaml(content: &str) -> Result<RequestDefinition> {
    serde_yaml::from_str(content) 
        .with_context(|| "Failed to parse YAML content")
  }

    fn parse_json(content: &str)-> Result<RequestDefinition> {
        serde_json::from_str(content) 
            .with_context(|| "Failed to parse YAML content")
    }
}

pub fn parse_request_file<P: AsRef<Path>>(path: P) -> Result<RequestDefinition> {
    let content = std::fs::read_to_string(path)?;
    let request: RequestDefinition = serde_yaml::from_str(&content)?;
    Ok(request)
}

pub fn parse_request_files_in_directory<P: AsRef<Path>>(dir: P) -> Result<Vec<RequestDefinition>> {
    // TODO: Implement directory parsing
    Ok(vec![])
}
