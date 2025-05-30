use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid header name: {0}")]
    InvalidHeader(String),
    #[error("Invalid JSON in body: {0}")]
    InvalidJson(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestDefinition {
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub params: Option<HashMap<String, String>>,
    pub body: Option<RequestBody>,
    pub auth: Option<AuthConfig>,
    pub tests: Option<Vec<TestAssertion>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestBody {
    Json(serde_json::Value),
    Text(String),
    Form(HashMap<String, String>),
    File(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthConfig {
    Bearer { token: String },
    Basic { username: String, password: String },
    ApiKey { key: String, value: String, location: ApiKeyLocation },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiKeyLocation {
    Header,
    Query,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestAssertion {
    pub status_code: Option<u16>,
    pub response_time_less_than: Option<u64>,
    pub json_path: Option<String>,
    pub exists: Option<bool>,
    pub equals: Option<serde_json::Value>,
}

impl RequestDefinition {
   pub fn validate(&self) -> Result<(), ValidationError> {
        if self.name.trim().is_empty() {
            return Err(ValidationError::MissingField("name".to_string()));
        } 

        if self.url.trim().is_empty() {
            return Err(ValidationError::MissingField("url".to_string()));
        }

        if let Some(headers) = &self.headers {
            for key in headers.keys() {
                if !self.is_valid_header_name(key) {
                    return Err(ValidationError::InvalidHeader(key.to_string().clone()));
                }
            }
        }

        if let Some(RequestBody::Json(json)) = &self.body {
            if json.is_null() {
                return Err(ValidationError::InvalidJson("Body cannot be null".to_string()));
            }
        }

        if let Some(tests) = &self.tests {
            for test in tests {
                if let Some(status_code) = test.status_code {
                    if !(100..=599).contains(&status_code) {
                        return Err(ValidationError::InvalidJson(
                            format!("Invalid status code: {}", status_code)
                        ));
                    }
                }
            }
        }

        Ok(())
   }  

    fn is_valid_header_name(&self, name: &str) -> bool {
        // HTTP header names should not be empty and contain valid characters
        !name.trim().is_empty() && name.chars().all(|c| c.is_ascii() && !c.is_control())
    }
}
