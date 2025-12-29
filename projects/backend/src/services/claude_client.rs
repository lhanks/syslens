//! Claude API client for AI-powered device information lookup.
//!
//! Uses the Anthropic Claude API to intelligently search and parse
//! device specifications when web scraping fails.

use crate::models::{
    DataMetadata, DataSource, DeviceDeepInfo, DeviceIdentifier, DeviceSpecifications,
    DeviceType, DocumentationLinks, DriverInfo, SpecCategory, SpecItem,
};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// Claude API endpoint
const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Claude model to use
const CLAUDE_MODEL: &str = "claude-sonnet-4-20250514";

/// API version header
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Request timeout in seconds
const REQUEST_TIMEOUT: u64 = 30;

/// Maximum tokens for response
const MAX_TOKENS: u32 = 4096;

/// Claude API client for device information lookup.
pub struct ClaudeClient {
    client: Client,
    api_key: Option<String>,
}

/// Message in Claude API request
#[derive(Debug, Clone, Serialize)]
struct Message {
    role: String,
    content: String,
}

/// Claude API request body
#[derive(Debug, Clone, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
}

/// Content block in Claude API response
#[derive(Debug, Clone, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

/// Claude API response body
#[derive(Debug, Clone, Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
    #[allow(dead_code)]
    stop_reason: Option<String>,
}

/// Parsed device specification from Claude response
#[derive(Debug, Clone, Deserialize)]
struct ParsedDeviceInfo {
    manufacturer: Option<String>,
    model: Option<String>,
    specifications: Option<HashMap<String, String>>,
    release_date: Option<String>,
    description: Option<String>,
    driver_url: Option<String>,
    product_url: Option<String>,
    confidence: Option<f32>,
}

impl ClaudeClient {
    /// Create a new Claude API client.
    ///
    /// Reads API key from ANTHROPIC_API_KEY environment variable.
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT))
            .build()
            .context("Failed to create HTTP client")?;

        let api_key = env::var("ANTHROPIC_API_KEY").ok();

        if api_key.is_none() {
            log::warn!("ANTHROPIC_API_KEY not set - Claude AI features will be disabled");
        }

        Ok(Self { client, api_key })
    }

    /// Check if Claude API is available (API key is set).
    pub fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    /// Look up device information using Claude AI.
    pub async fn lookup_device(
        &self,
        identifier: &DeviceIdentifier,
        device_type: &DeviceType,
    ) -> Result<DeviceDeepInfo> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Claude API key not configured"))?;

        log::info!(
            "Claude AI looking up: {} {} ({:?})",
            identifier.manufacturer,
            identifier.model,
            device_type
        );

        // Build the prompt
        let system_prompt = self.get_system_prompt();
        let user_prompt = self.get_user_prompt(identifier, device_type);

        // Make API request
        let request = ClaudeRequest {
            model: CLAUDE_MODEL.to_string(),
            max_tokens: MAX_TOKENS,
            system: system_prompt,
            messages: vec![Message {
                role: "user".to_string(),
                content: user_prompt,
            }],
        };

        let response = self
            .client
            .post(CLAUDE_API_URL)
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send Claude API request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            log::error!("Claude API error: {} - {}", status, error_text);
            return Err(anyhow::anyhow!("Claude API error: {}", status));
        }

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .context("Failed to parse Claude API response")?;

        // Extract text from response
        let response_text = claude_response
            .content
            .iter()
            .filter_map(|block| {
                if block.content_type == "text" {
                    block.text.clone()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Parse the response
        self.parse_response(&response_text, identifier, device_type)
    }

    /// Get system prompt for Claude.
    fn get_system_prompt(&self) -> String {
        r#"You are a hardware specification expert. Your task is to provide accurate, structured information about computer hardware components.

When given a device identifier (manufacturer and model), provide detailed specifications in a structured JSON format.

Always respond with valid JSON containing these fields:
{
  "manufacturer": "string",
  "model": "string",
  "specifications": {
    "key": "value"
  },
  "release_date": "string or null",
  "description": "brief product description",
  "driver_url": "URL to driver download page or null",
  "product_url": "URL to product page or null",
  "confidence": 0.0-1.0
}

Guidelines:
- Only include verified, accurate information
- Set confidence based on how certain you are (1.0 = certain, 0.5 = somewhat sure, 0.0 = guessing)
- Use standard units (GHz, GB, MHz, nm, W, etc.)
- Include relevant specifications for the device type
- If you're unsure about specific details, omit them rather than guess
- For CPUs: include cores, threads, base/boost clock, cache, TDP, socket, architecture
- For GPUs: include VRAM, memory type, base/boost clock, CUDA cores/stream processors, TDP, bus width
- For Motherboards: include chipset, socket, form factor, memory slots, PCIe slots
- For Memory: include capacity, speed, timings, type (DDR4/DDR5), voltage
- For Storage: include capacity, interface, read/write speeds, form factor"#.to_string()
    }

    /// Get user prompt for a specific device lookup.
    fn get_user_prompt(&self, identifier: &DeviceIdentifier, device_type: &DeviceType) -> String {
        let type_str = match device_type {
            DeviceType::Cpu => "CPU/Processor",
            DeviceType::Gpu => "GPU/Graphics Card",
            DeviceType::Motherboard => "Motherboard",
            DeviceType::Memory => "RAM/Memory Module",
            DeviceType::Storage => "Storage Device (SSD/HDD)",
        };

        format!(
            r#"Please provide specifications for the following {} device:

Manufacturer: {}
Model: {}
{}

Respond with JSON only, no additional text."#,
            type_str,
            identifier.manufacturer,
            identifier.model,
            identifier
                .part_number
                .as_ref()
                .map(|p| format!("Part Number: {}", p))
                .unwrap_or_default()
        )
    }

    /// Parse Claude's response into DeviceDeepInfo.
    fn parse_response(
        &self,
        response_text: &str,
        identifier: &DeviceIdentifier,
        device_type: &DeviceType,
    ) -> Result<DeviceDeepInfo> {
        // Try to extract JSON from the response
        let json_text = self.extract_json(response_text)?;

        // Parse the JSON
        let parsed: ParsedDeviceInfo =
            serde_json::from_str(&json_text).context("Failed to parse Claude response as JSON")?;

        let confidence = parsed.confidence.unwrap_or(0.5);

        // Build specifications
        let specifications = parsed.specifications.map(|specs| {
            let spec_items: Vec<SpecItem> = specs
                .iter()
                .map(|(k, v)| SpecItem {
                    label: k.clone(),
                    value: v.clone(),
                    unit: None,
                })
                .collect();

            let category_name = match device_type {
                DeviceType::Cpu => "CPU Specifications",
                DeviceType::Gpu => "GPU Specifications",
                DeviceType::Motherboard => "Motherboard Specifications",
                DeviceType::Memory => "Memory Specifications",
                DeviceType::Storage => "Storage Specifications",
            };

            DeviceSpecifications {
                specs,
                categories: vec![SpecCategory {
                    name: category_name.to_string(),
                    specs: spec_items,
                }],
                description: parsed.description.clone(),
                release_date: parsed.release_date.clone(),
                eol_date: None,
            }
        });

        // Build driver info
        let drivers = parsed.driver_url.map(|url| DriverInfo {
            installed_version: None,
            latest_version: None,
            download_url: None,
            release_date: None,
            release_notes_url: None,
            driver_page_url: Some(url),
            update_available: false,
        });

        // Build documentation links
        let documentation = parsed.product_url.map(|url| DocumentationLinks {
            product_page: Some(url),
            support_page: None,
            manuals: vec![],
            datasheets: vec![],
            firmware_updates: vec![],
        });

        // Generate device ID
        let device_id = format!(
            "{}-{}-{}",
            match device_type {
                DeviceType::Cpu => "cpu",
                DeviceType::Gpu => "gpu",
                DeviceType::Motherboard => "mb",
                DeviceType::Memory => "mem",
                DeviceType::Storage => "stor",
            },
            identifier.manufacturer.to_lowercase().replace(' ', "-"),
            identifier.model.to_lowercase().replace(' ', "-")
        );

        Ok(DeviceDeepInfo {
            device_id,
            device_type: device_type.clone(),
            identifier: DeviceIdentifier {
                manufacturer: parsed
                    .manufacturer
                    .unwrap_or_else(|| identifier.manufacturer.clone()),
                model: parsed.model.unwrap_or_else(|| identifier.model.clone()),
                part_number: identifier.part_number.clone(),
                serial_number: identifier.serial_number.clone(),
                hardware_ids: identifier.hardware_ids.clone(),
            },
            specifications,
            drivers,
            documentation,
            images: None,
            metadata: DataMetadata {
                source: DataSource::AiAgent,
                last_updated: Utc::now(),
                expires_at: Utc::now() + Duration::days(3),
                source_url: None,
                ai_confidence: Some(confidence),
            },
        })
    }

    /// Extract JSON from Claude's response text.
    fn extract_json(&self, text: &str) -> Result<String> {
        // Try to find JSON in the response
        let trimmed = text.trim();

        // If the whole response is JSON, use it directly
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            return Ok(trimmed.to_string());
        }

        // Try to find JSON block in markdown code fence
        if let Some(start) = trimmed.find("```json") {
            let json_start = start + 7;
            if let Some(end) = trimmed[json_start..].find("```") {
                return Ok(trimmed[json_start..json_start + end].trim().to_string());
            }
        }

        // Try to find JSON block in generic code fence
        if let Some(start) = trimmed.find("```") {
            let json_start = start + 3;
            // Skip to newline
            if let Some(newline) = trimmed[json_start..].find('\n') {
                let content_start = json_start + newline + 1;
                if let Some(end) = trimmed[content_start..].find("```") {
                    let content = trimmed[content_start..content_start + end].trim();
                    if content.starts_with('{') {
                        return Ok(content.to_string());
                    }
                }
            }
        }

        // Try to find any JSON object in the text
        if let Some(start) = trimmed.find('{') {
            // Find matching closing brace
            let mut depth = 0;
            let chars: Vec<char> = trimmed[start..].chars().collect();
            for (i, c) in chars.iter().enumerate() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(trimmed[start..start + i + 1].to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        Err(anyhow::anyhow!("Could not find valid JSON in response"))
    }
}

impl Default for ClaudeClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default ClaudeClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_direct() {
        let client = ClaudeClient::new().unwrap();
        let json = r#"{"manufacturer": "Intel", "model": "Core i9-14900K"}"#;
        let result = client.extract_json(json);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Intel"));
    }

    #[test]
    fn test_extract_json_markdown() {
        let client = ClaudeClient::new().unwrap();
        let text = r#"Here is the information:

```json
{"manufacturer": "AMD", "model": "Ryzen 9 7950X"}
```

Hope this helps!"#;
        let result = client.extract_json(text);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("AMD"));
    }

    #[test]
    fn test_extract_json_embedded() {
        let client = ClaudeClient::new().unwrap();
        let text = r#"Based on my knowledge, here is the device info: {"manufacturer": "NVIDIA", "model": "RTX 4090"} as requested."#;
        let result = client.extract_json(text);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("NVIDIA"));
    }

    #[test]
    fn test_is_available_without_key() {
        // When ANTHROPIC_API_KEY is not set
        let client = ClaudeClient::new().unwrap();
        // This will be false unless the env var is actually set
        // Just test that it doesn't panic
        let _ = client.is_available();
    }

    #[test]
    fn test_system_prompt() {
        let client = ClaudeClient::new().unwrap();
        let prompt = client.get_system_prompt();
        assert!(prompt.contains("hardware specification expert"));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_user_prompt() {
        let client = ClaudeClient::new().unwrap();
        let identifier = DeviceIdentifier {
            manufacturer: "Intel".to_string(),
            model: "Core i9-14900K".to_string(),
            part_number: Some("BX8071514900K".to_string()),
            serial_number: None,
            hardware_ids: vec![],
        };
        let prompt = client.get_user_prompt(&identifier, &DeviceType::Cpu);
        assert!(prompt.contains("Intel"));
        assert!(prompt.contains("Core i9-14900K"));
        assert!(prompt.contains("CPU/Processor"));
        assert!(prompt.contains("BX8071514900K"));
    }
}
