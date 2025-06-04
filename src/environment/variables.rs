use anyhow::{Context, Result as AnyhowResult};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

use crate::request::ValidationError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    pub variables: Option<HashMap<String, String>>
}

#[derive(Default, Debug, Deserialize)]
pub struct EnvironmentResolver {
    pub environments: Vec<Environment>
}

impl EnvironmentResolver {
    pub fn load_environment_file<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(), ValidationError> {
        let path_ref = path.as_ref();
        let content = std::fs::read_to_string(path_ref)
            .with_context(|| format!("Failed to read file {}", path_ref.display()))
            .map_err(|e: anyhow::Error| ValidationError::FileIo(e.to_string()))?;

        let loaded_data: Environment = match path_ref.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => Self::parse_yaml(&content)
                .map_err(|e: anyhow::Error| ValidationError::Parse(e.to_string()))?,
            Some("json") => Self::parse_json(&content)
                .map_err(|e: anyhow::Error| ValidationError::Parse(e.to_string()))?,
            Some(ext) => return Err(ValidationError::UnsupportedFormat(ext.to_string())),
            None => {
                if content.trim_start().starts_with('{') {
                    Self::parse_json(&content)
                        .map_err(|e: anyhow::Error| ValidationError::Parse(e.to_string()))?
                } else {
                    Self::parse_yaml(&content)
                        .map_err(|e: anyhow::Error| ValidationError::Parse(e.to_string()))?
                }
            }
        };

        self.environments.push(loaded_data);

        Ok(())
    }

    pub fn resolve_template(&self, template: &str) -> AnyhowResult<String> {
        Ok(template.to_string())
    }

    fn parse_yaml(content: &str) -> AnyhowResult<Environment> {
        serde_yaml::from_str(content)
            .with_context(|| "Failed to parse YAML content")
    }

    fn parse_json(content: &str) -> AnyhowResult<Environment> {
        serde_json::from_str(content)
            .with_context(|| "Failed to parse JSON content") // Corrected message
    }
}
