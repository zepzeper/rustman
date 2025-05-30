use anyhow::Result;
use colored::*;

pub struct ResponseFormatter;

impl ResponseFormatter {
    pub fn format_response(response: &reqwest::Response, body: &str) -> Result<String> {
        // TODO: Implement response formatting
        Ok(format!("Status: {}\nBody: {}", response.status(), body))
    }

    pub fn format_json(json: &str) -> Result<String> {
        let parsed: serde_json::Value = serde_json::from_str(json)?;
        Ok(serde_json::to_string_pretty(&parsed)?)
    }
}
