use anyhow::Result;
use reqwest::Client;
use crate::request::RequestDefinition;

pub struct RequestExecutor {
    client: Client,
}

impl RequestExecutor {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn execute(&self, request: &RequestDefinition) -> Result<reqwest::Response> {
        // TODO: Implement request execution
        todo!("Implement request execution")
    }
}
