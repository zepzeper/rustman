use anyhow::Result;
use reqwest::Client;
use crate::{environment::EnvironmentResolver, request::RequestDefinition};

#[derive(Default)]
pub struct RequestExecutor {
    client: Client,
}

impl RequestExecutor {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn execute(&self, request: &RequestDefinition, environment: &EnvironmentResolver) -> Result<reqwest::Response> {
        todo!("Implement request execution")
    }
}
