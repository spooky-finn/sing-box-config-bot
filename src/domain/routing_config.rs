use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct RoutingConfig {
    pub dns_proxy_keywords: Vec<String>,
    pub dns_direct_keywords: Vec<String>,
    pub dns_direct_regex: Vec<String>,
    pub direct_route_keywords: Vec<String>,
}

impl RoutingConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: RoutingConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
}
