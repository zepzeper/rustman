use walkdir::WalkDir;
use std::path::{Path, PathBuf};

use crate::utils::load_and_parse_file;

use super::{RequestDefinition, ValidationError};

pub struct RequestParser;

impl RequestParser {
   pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<RequestDefinition, ValidationError> {
        load_and_parse_file(path)
   } 

   pub fn parse_directory<P: AsRef<Path>>(
        dir: P,
    ) -> Vec<(PathBuf, Result<RequestDefinition, ValidationError>)> {
        let mut results = Vec::new();

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                    if matches!(ext, "yaml" | "yml" | "json") {
                        let path_buf = entry.path().to_path_buf();
                        let result = Self::parse_file(&path_buf);
                        results.push((path_buf, result));
                    }
                }
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::HttpMethod; // For RequestDefinition fields
    use std::io::Write;
    use tempfile::{Builder as TempFileBuilder, NamedTempFile};

    // Test for parse_file (which now uses the utility)
    #[test]
    fn test_parse_file_yaml_delegation() {
        let yaml_content = r#"
name: "Test Request via Parser"
method: GET
url: "http://example.com/yaml"
"#;
        let mut temp_file = TempFileBuilder::new().suffix(".yaml").NamedTempFile().unwrap();
        write!(temp_file, "{}", yaml_content).unwrap();
        temp_file.flush().unwrap();

        let result = RequestParser::parse_file(temp_file.path());
        assert!(result.is_ok(), "parse_file failed for YAML: {:?}", result.err());
        let req_def = result.unwrap();
        assert_eq!(req_def.name, "Test Request via Parser");
        assert_eq!(req_def.method, HttpMethod::GET);
    }

    #[test]
    fn test_parse_file_json_delegation() {
        let json_content = r#"
{
    "name": "Test Request via Parser JSON",
    "method": "POST",
    "url": "http://example.com/json"
}
"#;
        let mut temp_file = TempFileBuilder::new().suffix(".json").NamedTempFile().unwrap();
        write!(temp_file, "{}", json_content).unwrap();
        temp_file.flush().unwrap();

        let result = RequestParser::parse_file(temp_file.path());
        assert!(result.is_ok(), "parse_file failed for JSON: {:?}", result.err());
        let req_def = result.unwrap();
        assert_eq!(req_def.name, "Test Request via Parser JSON");
        assert_eq!(req_def.method, HttpMethod::POST);
    }

    // Test for parse_directory
    #[test]
    fn test_parse_directory_multiple_files() {
        let temp_dir = TempFileBuilder::new().prefix("test_requests_").tempdir().unwrap();
        let dir_path = temp_dir.path();

        // Create a YAML file
        let yaml_content = "name: YAML In Dir\nmethod: GET\nurl: http://dir.com/yaml";
        let mut yaml_file = TempFileBuilder::new().suffix(".yaml").tempfile_in(dir_path).unwrap();
        write!(yaml_file, "{}", yaml_content).unwrap();
        yaml_file.flush().unwrap();

        // Create a JSON file
        let json_content = r#"{"name": "JSON In Dir", "method": "PUT", "url": "http://dir.com/json"}"#;
        let mut json_file = TempFileBuilder::new().suffix(".json").tempfile_in(dir_path).unwrap();
        write!(json_file, "{}", json_content).unwrap();
        json_file.flush().unwrap();

        // Create an unsupported file
        let mut txt_file = TempFileBuilder::new().suffix(".txt").tempfile_in(dir_path).unwrap();
        write!(txt_file, "some text").unwrap();
        txt_file.flush().unwrap();

        let results = RequestParser::parse_directory(dir_path);
        assert_eq!(results.len(), 2, "Should only parse YAML and JSON files");

        let mut found_yaml = false;
        let mut found_json = false;

        for (_path, result) in results {
            assert!(result.is_ok());
            let req_def = result.unwrap();
            if req_def.name == "YAML In Dir" {
                assert_eq!(req_def.method, HttpMethod::GET);
                found_yaml = true;
            } else if req_def.name == "JSON In Dir" {
                assert_eq!(req_def.method, HttpMethod::PUT);
                found_json = true;
            }
        }
        assert!(found_yaml, "YAML file was not parsed correctly from directory");
        assert!(found_json, "JSON file was not parsed correctly from directory");
    }
}
