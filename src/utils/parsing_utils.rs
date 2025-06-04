use std::path::Path;
use anyhow::Context;
use serde::de::DeserializeOwned;

use crate::request::ValidationError;

enum FileFormat {
    Yaml,
    Json
}

fn determine_format_from_path(path: &Path, content: &str) -> Result<FileFormat, ValidationError> {
    match path.extension().and_then(|s| s.to_str()) {
        Some("yaml") | Some("yml") => Ok(FileFormat::Yaml),
        Some("json") => Ok(FileFormat::Json),
        Some(ext) => Err(ValidationError::UnsupportedFormat(ext.to_string())),
        _ =>  {
            if content.trim_start().starts_with('{') {
                Ok(FileFormat::Json)
            } else {
                Ok(FileFormat::Yaml)
            }
        }
    }
}

/// # Type Parameters
/// * `T`: The type to deserialize the file content into. Must implement `serde::de::DeserializeOwned`.
/// * `P`: A type that can be converted into a `Path` reference (e.g., `&str`, `PathBuf`).
pub fn load_and_parse_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, ValidationError> {
      let path_ref = path.as_ref(); 

    let content = std::fs::read_to_string(path_ref)
        .with_context(|| format!("Failed to read file {}", path_ref.display()))
        .map_err(|e: anyhow::Error| ValidationError::FileIo(e.to_string()))?;

    let format = determine_format_from_path(path_ref, &content)?;

    match format {
        FileFormat::Yaml => {
            serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse YAML content from {}", path_ref.display()))
                .map_err(|e| ValidationError::FileIo(e.to_string()))
        },
        FileFormat::Json => {
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse JSON content from {}", path_ref.display()))
                .map_err(|e| ValidationError::FileIo(e.to_string()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use crate::request::RequestDefinition; 

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[test]
    fn test_load_and_parse_yaml_file() {
        let yaml_content = "name: TestYAML\nvalue: 123";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", yaml_content).unwrap();

        let result: Result<TestData, ValidationError> = load_and_parse_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TestData { name: "TestYAML".to_string(), value: 123 });
    }

    #[test]
    fn test_load_and_parse_json_file() {
        let json_content = r#"{"name": "TestJSON", "value": 456}"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_content).unwrap();
        temp_file.flush().unwrap(); // Ensure content is written before parsing

        let result: Result<TestData, ValidationError> = load_and_parse_file(temp_file.path());
        assert!(result.is_ok(), "JSON parsing failed: {:?}", result.err());
        assert_eq!(result.unwrap(), TestData { name: "TestJSON".to_string(), value: 456 });
    }

    #[test]
    fn test_load_and_parse_file_no_extension_json_content() {
        let json_content = r#"{"name": "SniffJSON", "value": 789}"#;
        // Create a temp file without an extension for sniffing
        let mut temp_file = NamedTempFile::builder().suffix("").tempfile().unwrap();
        write!(temp_file, "{}", json_content).unwrap();
        temp_file.flush().unwrap();

        let result: Result<TestData, ValidationError> = load_and_parse_file(temp_file.path());
        assert!(result.is_ok(), "Sniffing JSON failed: {:?}", result.err());
        assert_eq!(result.unwrap(), TestData { name: "SniffJSON".to_string(), value: 789 });
    }

    #[test]
    fn test_load_and_parse_file_no_extension_yaml_content() {
        let yaml_content = "name: SniffYAML\nvalue: 101";
        let mut temp_file = NamedTempFile::builder().suffix("").tempfile().unwrap();
        write!(temp_file, "{}", yaml_content).unwrap();
        temp_file.flush().unwrap();

        let result: Result<TestData, ValidationError> = load_and_parse_file(temp_file.path());
        assert!(result.is_ok(), "Sniffing YAML failed: {:?}", result.err());
        assert_eq!(result.unwrap(), TestData { name: "SniffYAML".to_string(), value: 101 });
    }

    #[test]
    fn test_file_not_found() {
        let result: Result<TestData, ValidationError> = load_and_parse_file("non_existent_file.yaml");
        assert!(result.is_err());
        match result.err().unwrap() {
            ValidationError::FileIo(_) => {} // Expected
            e => panic!("Expected FileIo error, got {:?}", e),
        }
    }

    #[test]
    fn test_unsupported_format() {
        let mut temp_file = NamedTempFile::builder().suffix(".txt").tempfile().unwrap();
        write!(temp_file, "some text").unwrap();
        temp_file.flush().unwrap();

        let result: Result<TestData, ValidationError> = load_and_parse_file(temp_file.path());
        assert!(result.is_err());
        match result.err().unwrap() {
            ValidationError::UnsupportedFormat(ext) => assert_eq!(ext, "txt"),
            e => panic!("Expected UnsupportedFormat error, got {:?}", e),
        }
    }

    #[test]
    fn test_invalid_yaml_content() {
        let yaml_content = "name: TestYAML\nvalue: 123\ninvalid_indentation";
        let mut temp_file = NamedTempFile::new().unwrap(); // .yaml by default with tempfile
        write!(temp_file, "{}", yaml_content).unwrap();
        temp_file.flush().unwrap();

        let result: Result<TestData, ValidationError> = load_and_parse_file(temp_file.path());
        assert!(result.is_err());
        match result.err().unwrap() {
            ValidationError::Parse(_) => {} // Expected
            e => panic!("Expected Parse error for YAML, got {:?}", e),
        }
    }

     #[test]
    fn test_invalid_json_content() {
        let json_content = r#"{"name": "TestJSON", "value": 456,}"#; // Trailing comma
        let mut temp_file = NamedTempFile::builder().suffix(".json").tempfile().unwrap();
        write!(temp_file, "{}", json_content).unwrap();
        temp_file.flush().unwrap();

        let result: Result<TestData, ValidationError> = load_and_parse_file(temp_file.path());
        assert!(result.is_err());
        match result.err().unwrap() {
            ValidationError::Parse(_) => {} // Expected
            e => panic!("Expected Parse error for JSON, got {:?}", e),
        }
    }

    // Example using your actual RequestDefinition if it's accessible and simple enough for a test
    #[test]
    fn test_parse_actual_request_definition_yaml() {
        // Make sure RequestDefinition and HttpMethod are in scope
        // You might need: use crate::request::{RequestDefinition, HttpMethod};
        let yaml_content = r#"
name: "Sample Request"
method: GET
url: "http://example.com"
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", yaml_content).unwrap();
        temp_file.flush().unwrap();

        let result: Result<RequestDefinition, ValidationError> = load_and_parse_file(temp_file.path());
        assert!(result.is_ok(), "Parsing RequestDefinition from YAML failed: {:?}", result.err());
        let req_def = result.unwrap();
        assert_eq!(req_def.name, "Sample Request");
        // Add more assertions based on HttpMethod if it's PartialEq
    }
}
