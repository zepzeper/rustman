use anyhow::{Context, Result as AnyhowResult};
use walkdir::WalkDir;
use std::path::{Path, PathBuf};

use super::{RequestDefinition, ValidationError};

pub struct RequestParser;

impl RequestParser {
   pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<RequestDefinition, ValidationError> {
      let path_ref = path.as_ref(); 

        let content = std::fs::read_to_string(path_ref)
            .with_context(|| format!("Failed to read file {}", path_ref.display()))
            .map_err(|e: anyhow::Error| ValidationError::FileIo(e.to_string()))?;

        let request = match path_ref.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => Self::parse_yaml(&content)
                .map_err(|e: anyhow::Error| ValidationError::Parse(e.to_string()))?,
            Some("json") => Self::parse_json(&content)
                .map_err(|e: anyhow::Error| ValidationError::Parse(e.to_string()))?,
            Some(ext) => return Err(ValidationError::UnsupportedFormat(ext.to_string())),
            None => {
                // Try to detect format from content
                if content.trim_start().starts_with('{') {
                    Self::parse_json(&content)
                        .map_err(|e: anyhow::Error| ValidationError::Parse(e.to_string()))?
                } else {
                    Self::parse_yaml(&content)
                        .map_err(|e: anyhow::Error| ValidationError::Parse(e.to_string()))?
                }
            }
        };

        Ok(request)
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

  fn parse_yaml(content: &str) -> AnyhowResult<RequestDefinition> {
    serde_yaml::from_str(content) 
        .with_context(|| "Failed to parse YAML content")
  }

    fn parse_json(content: &str)-> AnyhowResult<RequestDefinition> {
        serde_json::from_str(content) 
            .with_context(|| "Failed to parse YAML content")
    }
}
