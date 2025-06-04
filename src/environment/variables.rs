use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use regex::Regex;

use crate::utils::load_and_parse_file;
use crate::request::ValidationError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Environment {
    pub name: String,
    pub variables: Option<HashMap<String, String>>,
}

#[derive(Default, Debug)]
pub struct EnvironmentResolver {
    active_variables: Option<HashMap<String, String>>,
    active_environment_name: Option<String>, 
}

impl EnvironmentResolver {
    pub fn load_environment_file<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(), ValidationError> {
        let loaded_environment: Environment = load_and_parse_file(path)?;
        
        self.active_environment_name = Some(loaded_environment.name);
        self.active_variables = loaded_environment.variables;
        
        Ok(())
    }

    pub fn resolve_template(&self, template: &str) -> String { 
        if let Some(vars) = &self.active_variables {
            let re = Regex::new(r"\{\{([a-zA-Z0-9_]+)\}\}").unwrap(); // Infallible regex
            
            let resolved_string = re.replace_all(template, |caps: &regex::Captures| {
                let var_name = &caps[1];
                vars.get(var_name) 
                    .cloned()
                    .unwrap_or(caps[0].to_string()) 
            });
            return resolved_string.into_owned(); 
        }

        // If no active variables, return the template as is.
        template.to_string()
    }

    pub fn active_environment_name(&self) -> Option<&str> {
        self.active_environment_name.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::Builder as TempFileBuilder;

    #[test]
    fn test_load_environment_file_sets_active() {
        let yaml_content = "name: TestEnvActive\nvariables:\n  key1: val1\n  api_key: secret";
        let mut temp_file = TempFileBuilder::new().suffix(".yaml").NamedTempFile().unwrap();
        write!(temp_file, "{}", yaml_content).unwrap();
        temp_file.flush().unwrap();

        let mut resolver = EnvironmentResolver::default();
        let result = resolver.load_environment_file(temp_file.path());

        assert!(result.is_ok(), "load_environment_file failed: {:?}", result.err());
        assert_eq!(resolver.active_environment_name.as_deref(), Some("TestEnvActive"));
        assert!(resolver.active_variables.is_some());
        let vars = resolver.active_variables.as_ref().unwrap();
        assert_eq!(vars.get("key1"), Some(&"val1".to_string()));
        assert_eq!(vars.get("api_key"), Some(&"secret".to_string()));
    }

    #[test]
    fn test_load_environment_overwrites_previous() {
        let mut resolver = EnvironmentResolver::default();
        
        // Load first environment
        let yaml_content1 = "name: Env1\nvariables:\n  url: site1.com";
        let mut temp_file1 = TempFileBuilder::new().suffix(".yaml").NamedTempFile().unwrap();
        write!(temp_file1, "{}", yaml_content1).unwrap();
        temp_file1.flush().unwrap();
        resolver.load_environment_file(temp_file1.path()).unwrap();

        assert_eq!(resolver.active_environment_name.as_deref(), Some("Env1"));
        assert_eq!(resolver.active_variables.as_ref().unwrap().get("url"), Some(&"site1.com".to_string()));

        // Load second environment
        let yaml_content2 = "name: Env2\nvariables:\n  url: site2.com\n  token: tok123";
        let mut temp_file2 = TempFileBuilder::new().suffix(".yaml").NamedTempFile().unwrap();
        write!(temp_file2, "{}", yaml_content2).unwrap();
        temp_file2.flush().unwrap();
        resolver.load_environment_file(temp_file2.path()).unwrap();
        
        assert_eq!(resolver.active_environment_name.as_deref(), Some("Env2"));
        let vars = resolver.active_variables.as_ref().unwrap();
        assert_eq!(vars.get("url"), Some(&"site2.com".to_string()));
        assert_eq!(vars.get("token"), Some(&"tok123".to_string()));
        assert_eq!(vars.len(), 2); // Ensure old vars are gone
    }

    #[test]
    fn test_resolve_template_with_active_variables() {
        let mut resolver = EnvironmentResolver::default();
        resolver.active_variables = Some({
            let mut vars = HashMap::new();
            vars.insert("base_url".to_string(), "http://api.example.com".to_string());
            vars.insert("user_id".to_string(), "123".to_string());
            vars.insert("API_VERSION".to_string(), "v2".to_string());
            vars
        });

        let template1 = "{{base_url}}/users/{{user_id}}?version={{API_VERSION}}";
        assert_eq!(resolver.resolve_template(template1), "http://api.example.com/users/123?version=v2");

        let template2 = "No variables here.";
        assert_eq!(resolver.resolve_template(template2), "No variables here.");

        let template3 = "{{base_url}}/products/{{undefined_var}}"; // undefined_var should remain
        assert_eq!(resolver.resolve_template(template3), "http://api.example.com/products/{{undefined_var}}");
        
        let template4 = "Path: {{base_url}} - User: {{user_id}} - Again: {{base_url}}";
        assert_eq!(resolver.resolve_template(template4), "Path: http://api.example.com - User: 123 - Again: http://api.example.com");
    }

    #[test]
    fn test_resolve_template_no_active_variables() {
        let resolver = EnvironmentResolver::default(); // No environment loaded
        let template = "{{base_url}}/items";
        assert_eq!(resolver.resolve_template(template), "{{base_url}}/items");
    }

    #[test]
    fn test_resolve_template_empty_variables_map() {
        let mut resolver = EnvironmentResolver::default();
        resolver.active_variables = Some(HashMap::new()); // Empty map
        let template = "{{base_url}}/items";
        assert_eq!(resolver.resolve_template(template), "{{base_url}}/items");
    }

    #[test]
    fn test_active_environment_name_accessor() {
        let mut resolver = EnvironmentResolver::default();
        assert_eq!(resolver.active_environment_name(), None);

        let yaml_content = "name: MyNamedEnv\nvariables:\n  var: val";
        let mut temp_file = TempFileBuilder::new().suffix(".yaml").NamedTempFile().unwrap();
        write!(temp_file, "{}", yaml_content).unwrap();
        temp_file.flush().unwrap();
        resolver.load_environment_file(temp_file.path()).unwrap();

        assert_eq!(resolver.active_environment_name(), Some("MyNamedEnv"));
    }
}
