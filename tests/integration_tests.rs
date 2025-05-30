#[cfg(test)]
mod tests {
    use rustman::request::parse_request_file;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parse_yaml_request() {
        let yaml_content = r#"
name: "Test Request"
method: GET
url: "https://api.example.com/test"
headers:
  Authorization: "Bearer token"
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();
        
        let result = parse_request_file(temp_file.path());
        assert!(result.is_ok());
        
        let request = result.unwrap();
        assert_eq!(request.name, "Test Request");
        assert_eq!(request.url, "https://api.example.com/test");
    }
}
