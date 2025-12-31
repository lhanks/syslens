//! Wikipedia API source for device information.
//!
//! Uses the MediaWiki API to fetch and parse hardware specifications
//! from Wikipedia articles. This is a reliable, free source with no
//! bot blocking issues.

use crate::models::{DeviceIdentifier, DeviceType, SpecCategory, SpecItem};
use crate::services::device_sources::DeviceSource;
use crate::services::PartialDeviceInfo;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

const WIKIPEDIA_API_URL: &str = "https://en.wikipedia.org/w/api.php";

/// Wikipedia-based device information source.
pub struct WikipediaSource {
    client: Client,
}

/// Image data extracted from Wikipedia.
#[derive(Debug, Clone)]
pub struct WikipediaImage {
    pub url: String,
    pub title: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl WikipediaSource {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Syslens/1.0 (Desktop System Monitor)")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Convert device identifier to Wikipedia search query.
    fn make_search_query(device_type: &DeviceType, identifier: &DeviceIdentifier) -> String {
        let model = &identifier.model;

        match device_type {
            DeviceType::Cpu => {
                // AMD Ryzen 9 9900X -> "AMD Ryzen 9"
                // Intel Core i9-13900K -> "Intel Core (13th generation)"
                if model.to_lowercase().contains("ryzen") {
                    // Extract Ryzen generation (e.g., "Ryzen 9" or "Ryzen 5")
                    let parts: Vec<&str> = model.split_whitespace().collect();
                    if let Some(pos) = parts.iter().position(|p| p.to_lowercase() == "ryzen") {
                        if pos + 1 < parts.len() {
                            return format!("AMD Ryzen {}", parts[pos + 1]);
                        }
                    }
                    "AMD Ryzen".to_string()
                } else if model.to_lowercase().contains("core i") {
                    // Try to extract generation from model number
                    // e.g., "Core i9-13900K" -> 13th gen
                    if let Some(gen) = Self::extract_intel_generation(model) {
                        format!("Intel Core ({}th generation)", gen)
                    } else {
                        format!("Intel {}", model)
                    }
                } else {
                    format!("{} {}", identifier.manufacturer, model)
                }
            }
            DeviceType::Gpu => {
                // GeForce RTX 5070 -> "GeForce RTX 50 series"
                // Radeon RX 7900 XTX -> "Radeon RX 7000 series"
                if model.to_lowercase().contains("geforce") {
                    if let Some(series) = Self::extract_nvidia_series(model) {
                        format!("GeForce RTX {} series", series)
                    } else {
                        format!("Nvidia {}", model)
                    }
                } else if model.to_lowercase().contains("radeon") {
                    if let Some(series) = Self::extract_amd_gpu_series(model) {
                        format!("Radeon RX {} series", series)
                    } else {
                        format!("AMD {}", model)
                    }
                } else {
                    format!("{} {}", identifier.manufacturer, model)
                }
            }
            _ => format!("{} {}", identifier.manufacturer, model),
        }
    }

    /// Extract Intel CPU generation from model string.
    fn extract_intel_generation(model: &str) -> Option<u32> {
        // Look for pattern like "i9-13900K" -> 13
        let re = regex::Regex::new(r"i[3579]-(\d{2,})").ok()?;
        if let Some(caps) = re.captures(model) {
            let num_str = caps.get(1)?.as_str();
            // First 1-2 digits are the generation
            if num_str.len() >= 5 {
                num_str[..2].parse().ok()
            } else if num_str.len() >= 4 {
                num_str[..1].parse().ok()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Extract NVIDIA GPU series from model string.
    fn extract_nvidia_series(model: &str) -> Option<&str> {
        let model_lower = model.to_lowercase();
        if model_lower.contains("5090")
            || model_lower.contains("5080")
            || model_lower.contains("5070")
        {
            Some("50")
        } else if model_lower.contains("4090")
            || model_lower.contains("4080")
            || model_lower.contains("4070")
            || model_lower.contains("4060")
        {
            Some("40")
        } else if model_lower.contains("3090")
            || model_lower.contains("3080")
            || model_lower.contains("3070")
            || model_lower.contains("3060")
        {
            Some("30")
        } else {
            None
        }
    }

    /// Extract AMD GPU series from model string.
    fn extract_amd_gpu_series(model: &str) -> Option<&str> {
        let model_lower = model.to_lowercase();
        if model_lower.contains("7900")
            || model_lower.contains("7800")
            || model_lower.contains("7700")
            || model_lower.contains("7600")
        {
            Some("7000")
        } else if model_lower.contains("6900")
            || model_lower.contains("6800")
            || model_lower.contains("6700")
            || model_lower.contains("6600")
        {
            Some("6000")
        } else {
            None
        }
    }

    /// Search Wikipedia for a page title.
    async fn search_page(&self, query: &str) -> Result<Option<String>> {
        #[derive(Deserialize)]
        struct SearchResponse {
            query: Option<QueryResult>,
        }

        #[derive(Deserialize)]
        struct QueryResult {
            search: Vec<SearchResult>,
        }

        #[derive(Deserialize)]
        struct SearchResult {
            title: String,
        }

        let response = self
            .client
            .get(WIKIPEDIA_API_URL)
            .query(&[
                ("action", "query"),
                ("list", "search"),
                ("srsearch", query),
                ("srlimit", "5"),
                ("format", "json"),
            ])
            .send()
            .await
            .context("Failed to search Wikipedia")?;

        let search: SearchResponse = response
            .json()
            .await
            .context("Failed to parse search response")?;

        Ok(search
            .query
            .and_then(|q| q.search.into_iter().next().map(|r| r.title)))
    }

    /// Fetch page content from Wikipedia.
    async fn fetch_page_content(&self, title: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct PageResponse {
            query: Option<PageQuery>,
        }

        #[derive(Deserialize)]
        struct PageQuery {
            pages: HashMap<String, PageContent>,
        }

        #[derive(Deserialize)]
        struct PageContent {
            revisions: Option<Vec<Revision>>,
        }

        #[derive(Deserialize)]
        struct Revision {
            #[serde(rename = "*")]
            content: Option<String>,
            slots: Option<Slots>,
        }

        #[derive(Deserialize)]
        struct Slots {
            main: Option<MainSlot>,
        }

        #[derive(Deserialize)]
        struct MainSlot {
            #[serde(rename = "*")]
            content: Option<String>,
        }

        let response = self
            .client
            .get(WIKIPEDIA_API_URL)
            .query(&[
                ("action", "query"),
                ("titles", title),
                ("prop", "revisions"),
                ("rvprop", "content"),
                ("rvslots", "main"),
                ("format", "json"),
            ])
            .send()
            .await
            .context("Failed to fetch Wikipedia page")?;

        let page_resp: PageResponse = response
            .json()
            .await
            .context("Failed to parse page response")?;

        let pages = page_resp.query.context("No query result")?.pages;
        for (_, page) in pages {
            if let Some(revisions) = page.revisions {
                if let Some(rev) = revisions.into_iter().next() {
                    // Try slots.main.content first (newer API format)
                    if let Some(slots) = rev.slots {
                        if let Some(main) = slots.main {
                            if let Some(content) = main.content {
                                return Ok(content);
                            }
                        }
                    }
                    // Fall back to direct content (older API format)
                    if let Some(content) = rev.content {
                        return Ok(content);
                    }
                }
            }
        }

        Err(anyhow::anyhow!("No content found for page"))
    }

    /// Parse infobox from Wikipedia wikitext.
    fn parse_infobox(&self, content: &str) -> HashMap<String, String> {
        let mut specs = HashMap::new();

        // Find infobox - look for patterns like {{Infobox GPU}} or {{Infobox processor}}
        let infobox_start = content.find("{{Infobox");
        if infobox_start.is_none() {
            // Try alternative infobox names
            if let Some(start) = content.find("{{infobox") {
                return self.parse_infobox_content(&content[start..]);
            }
            return specs;
        }

        let start = infobox_start.unwrap();
        specs = self.parse_infobox_content(&content[start..]);

        specs
    }

    /// Parse the content of an infobox template.
    fn parse_infobox_content(&self, content: &str) -> HashMap<String, String> {
        let mut specs = HashMap::new();
        let mut brace_count = 0;
        let mut in_infobox = false;
        let mut end_pos = 0;

        // Find the end of the infobox by counting braces
        for (i, c) in content.chars().enumerate() {
            match c {
                '{' => {
                    brace_count += 1;
                    if brace_count >= 2 {
                        in_infobox = true;
                    }
                }
                '}' => {
                    brace_count -= 1;
                    if in_infobox && brace_count < 2 {
                        end_pos = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        if end_pos == 0 {
            return specs;
        }

        let infobox_content = &content[..end_pos];

        // Parse key-value pairs (| key = value)
        for line in infobox_content.lines() {
            let trimmed = line.trim();
            if let Some(without_pipe) = trimmed.strip_prefix('|') {
                if let Some(eq_pos) = without_pipe.find('=') {
                    let key = without_pipe[..eq_pos].trim().to_lowercase();
                    let value = Self::clean_wiki_value(without_pipe[eq_pos + 1..].trim());

                    if !value.is_empty() && !key.is_empty() {
                        specs.insert(key, value);
                    }
                }
            }
        }

        specs
    }

    /// Clean Wikipedia markup from a value.
    fn clean_wiki_value(value: &str) -> String {
        let mut cleaned = value.to_string();

        // Remove wiki links [[text|display]] -> display, [[text]] -> text
        while let Some(start) = cleaned.find("[[") {
            if let Some(end) = cleaned[start..].find("]]") {
                let link_content = &cleaned[start + 2..start + end];
                let display = if let Some(pipe) = link_content.find('|') {
                    &link_content[pipe + 1..]
                } else {
                    link_content
                };
                cleaned = format!(
                    "{}{}{}",
                    &cleaned[..start],
                    display,
                    &cleaned[start + end + 2..]
                );
            } else {
                break;
            }
        }

        // Remove templates {{...}} - simple version
        while let Some(start) = cleaned.find("{{") {
            if let Some(end) = cleaned[start..].find("}}") {
                // Check if it's a unit template like {{nowrap|123 MHz}}
                let template_content = &cleaned[start + 2..start + end];
                if let Some(inner) = template_content.strip_prefix("nowrap|") {
                    cleaned = format!(
                        "{}{}{}",
                        &cleaned[..start],
                        inner,
                        &cleaned[start + end + 2..]
                    );
                } else {
                    cleaned = format!("{}{}", &cleaned[..start], &cleaned[start + end + 2..]);
                }
            } else {
                break;
            }
        }

        // Remove HTML tags
        let tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
        cleaned = tag_re.replace_all(&cleaned, "").to_string();

        // Remove references
        let ref_re = regex::Regex::new(r"\{\{[Rr]ef[^}]*\}\}").unwrap();
        cleaned = ref_re.replace_all(&cleaned, "").to_string();

        // Clean up extra whitespace
        cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

        cleaned.trim().to_string()
    }

    /// Extract specific specs for a device type from parsed infobox.
    fn extract_device_specs(
        &self,
        raw_specs: &HashMap<String, String>,
        device_type: &DeviceType,
        model: &str,
    ) -> (HashMap<String, String>, Vec<SpecCategory>) {
        let mut specs = HashMap::new();
        let mut categories = Vec::new();

        match device_type {
            DeviceType::Cpu => {
                self.extract_cpu_specs(raw_specs, &mut specs, &mut categories, model);
            }
            DeviceType::Gpu => {
                self.extract_gpu_specs(raw_specs, &mut specs, &mut categories, model);
            }
            _ => {
                // Generic extraction
                for (key, value) in raw_specs {
                    specs.insert(key.clone(), value.clone());
                }
            }
        }

        (specs, categories)
    }

    /// Extract CPU-specific specifications.
    fn extract_cpu_specs(
        &self,
        raw: &HashMap<String, String>,
        specs: &mut HashMap<String, String>,
        categories: &mut Vec<SpecCategory>,
        model: &str,
    ) {
        let mut processor_specs = Vec::new();
        let mut cache_specs = Vec::new();
        let mut power_specs = Vec::new();

        // Common CPU spec mappings
        let mappings = [
            ("cores", "Cores", "cores"),
            ("threads", "Threads", "threads"),
            ("l1cache", "L1 Cache", "l1cache"),
            ("l2cache", "L2 Cache", "l2cache"),
            ("l3cache", "L3 Cache", "l3cache"),
            ("frequency", "Base Clock", "baseclock"),
            ("turbo", "Boost Clock", "boostclock"),
            ("tdp", "TDP", "tdp"),
            ("socket", "Socket", "socket"),
            ("process", "Process", "process"),
            ("microarchitecture", "Architecture", "architecture"),
        ];

        for (wiki_key, label, spec_key) in mappings {
            if let Some(value) = raw.get(wiki_key) {
                specs.insert(spec_key.to_string(), value.clone());

                let item = SpecItem {
                    label: label.to_string(),
                    value: value.clone(),
                    unit: Self::extract_unit_from_value(value),
                };

                if wiki_key.contains("cache")
                    || wiki_key.contains("l1")
                    || wiki_key.contains("l2")
                    || wiki_key.contains("l3")
                {
                    cache_specs.push(item);
                } else if wiki_key == "tdp" {
                    power_specs.push(item);
                } else {
                    processor_specs.push(item);
                }
            }
        }

        // Look for model-specific specs in tables (will be parsed separately)
        // For now, add any we found
        if !processor_specs.is_empty() {
            categories.push(SpecCategory {
                name: "Processor".to_string(),
                specs: processor_specs,
            });
        }
        if !cache_specs.is_empty() {
            categories.push(SpecCategory {
                name: "Cache".to_string(),
                specs: cache_specs,
            });
        }
        if !power_specs.is_empty() {
            categories.push(SpecCategory {
                name: "Power".to_string(),
                specs: power_specs,
            });
        }

        // Add model to specs for context
        specs.insert("model".to_string(), model.to_string());
    }

    /// Extract GPU-specific specifications.
    fn extract_gpu_specs(
        &self,
        raw: &HashMap<String, String>,
        specs: &mut HashMap<String, String>,
        categories: &mut Vec<SpecCategory>,
        model: &str,
    ) {
        let mut engine_specs = Vec::new();
        let mut memory_specs = Vec::new();
        let mut power_specs = Vec::new();

        // Common GPU spec mappings
        let mappings = [
            ("shaders", "CUDA Cores", "cudacores"),
            ("cuda cores", "CUDA Cores", "cudacores"),
            ("stream processors", "Stream Processors", "streamprocessors"),
            ("tmus", "TMUs", "tmus"),
            ("rops", "ROPs", "rops"),
            ("memory", "Memory", "memory"),
            ("memory_size", "Memory Size", "memorysize"),
            ("memory_type", "Memory Type", "memorytype"),
            ("memory_bus", "Memory Bus", "memorybus"),
            ("bandwidth", "Memory Bandwidth", "bandwidth"),
            ("core_clock", "Base Clock", "baseclock"),
            ("boost_clock", "Boost Clock", "boostclock"),
            ("tdp", "TDP", "tdp"),
            ("process", "Process", "process"),
            ("architecture", "Architecture", "architecture"),
        ];

        for (wiki_key, label, spec_key) in mappings {
            if let Some(value) = raw.get(wiki_key) {
                specs.insert(spec_key.to_string(), value.clone());

                let item = SpecItem {
                    label: label.to_string(),
                    value: value.clone(),
                    unit: Self::extract_unit_from_value(value),
                };

                if wiki_key.contains("memory")
                    || wiki_key.contains("bandwidth")
                    || wiki_key.contains("bus")
                {
                    memory_specs.push(item);
                } else if wiki_key == "tdp" {
                    power_specs.push(item);
                } else {
                    engine_specs.push(item);
                }
            }
        }

        if !engine_specs.is_empty() {
            categories.push(SpecCategory {
                name: "GPU Engine".to_string(),
                specs: engine_specs,
            });
        }
        if !memory_specs.is_empty() {
            categories.push(SpecCategory {
                name: "Memory".to_string(),
                specs: memory_specs,
            });
        }
        if !power_specs.is_empty() {
            categories.push(SpecCategory {
                name: "Power".to_string(),
                specs: power_specs,
            });
        }

        specs.insert("model".to_string(), model.to_string());
    }

    /// Extract unit from value string.
    fn extract_unit_from_value(value: &str) -> Option<String> {
        let units = ["MHz", "GHz", "GB", "MB", "W", "nm", "bit", "GB/s", "MT/s"];
        for unit in units {
            if value.contains(unit) {
                return Some(unit.to_string());
            }
        }
        None
    }

    /// Extract release date from raw specs.
    fn extract_release_date(raw: &HashMap<String, String>) -> Option<String> {
        for key in [
            "released",
            "release_date",
            "launch_date",
            "date",
            "first_released",
        ] {
            if let Some(value) = raw.get(key) {
                return Some(value.clone());
            }
        }
        None
    }

    /// Fetch images from a Wikipedia page.
    pub async fn fetch_page_images(&self, title: &str) -> Result<Vec<WikipediaImage>> {
        #[derive(Deserialize)]
        struct ImageResponse {
            query: Option<ImageQuery>,
        }

        #[derive(Deserialize)]
        struct ImageQuery {
            pages: HashMap<String, ImagePage>,
        }

        #[derive(Deserialize)]
        struct ImagePage {
            images: Option<Vec<ImageInfo>>,
        }

        #[derive(Deserialize)]
        struct ImageInfo {
            title: String,
        }

        // First, get the list of images on the page
        let response = self
            .client
            .get(WIKIPEDIA_API_URL)
            .query(&[
                ("action", "query"),
                ("titles", title),
                ("prop", "images"),
                ("imlimit", "20"),
                ("format", "json"),
            ])
            .send()
            .await
            .context("Failed to fetch Wikipedia images")?;

        let image_resp: ImageResponse = response.json().await?;

        let mut image_titles = Vec::new();
        if let Some(query) = image_resp.query {
            for (_, page) in query.pages {
                if let Some(images) = page.images {
                    for img in images {
                        // Filter out common non-product images
                        if self.is_relevant_image(&img.title) {
                            image_titles.push(img.title);
                        }
                    }
                }
            }
        }

        if image_titles.is_empty() {
            return Ok(Vec::new());
        }

        // Now get the actual image URLs
        self.fetch_image_urls(&image_titles).await
    }

    /// Check if an image title is likely to be a product image.
    fn is_relevant_image(&self, title: &str) -> bool {
        let title_lower = title.to_lowercase();

        // Skip common Wikipedia template images
        let skip_patterns = [
            "commons-logo",
            "wiki",
            "icon",
            "flag",
            "symbol",
            "logo.svg",
            "ambox",
            "question book",
            "disambig",
            "edit-clear",
            "folder",
            "crystal",
            "nuvola",
            "gnome",
            "padlock",
            "red_x",
            "check",
            "yes_check",
            "x_mark",
        ];

        for pattern in skip_patterns {
            if title_lower.contains(pattern) {
                return false;
            }
        }

        // Prefer product-related images
        let prefer_patterns = [
            ".jpg", ".jpeg", ".png", "photo", "product", "chip", "die", "gpu", "cpu", "geforce",
            "radeon", "ryzen", "core i", "nvidia", "amd", "intel",
        ];

        prefer_patterns.iter().any(|p| title_lower.contains(p))
    }

    /// Fetch actual image URLs for a list of image titles.
    async fn fetch_image_urls(&self, titles: &[String]) -> Result<Vec<WikipediaImage>> {
        #[derive(Deserialize)]
        struct ImageUrlResponse {
            query: Option<ImageUrlQuery>,
        }

        #[derive(Deserialize)]
        struct ImageUrlQuery {
            pages: HashMap<String, ImageUrlPage>,
        }

        #[derive(Deserialize)]
        struct ImageUrlPage {
            title: Option<String>,
            imageinfo: Option<Vec<ImageUrlInfo>>,
        }

        #[derive(Deserialize)]
        struct ImageUrlInfo {
            url: String,
            width: Option<u32>,
            height: Option<u32>,
        }

        let titles_str = titles.join("|");

        let response = self
            .client
            .get(WIKIPEDIA_API_URL)
            .query(&[
                ("action", "query"),
                ("titles", &titles_str),
                ("prop", "imageinfo"),
                ("iiprop", "url|size"),
                ("format", "json"),
            ])
            .send()
            .await
            .context("Failed to fetch image URLs")?;

        let url_resp: ImageUrlResponse = response.json().await?;

        let mut images = Vec::new();
        if let Some(query) = url_resp.query {
            for (_, page) in query.pages {
                if let (Some(title), Some(imageinfo)) = (page.title, page.imageinfo) {
                    if let Some(info) = imageinfo.into_iter().next() {
                        images.push(WikipediaImage {
                            url: info.url,
                            title,
                            width: info.width,
                            height: info.height,
                        });
                    }
                }
            }
        }

        // Sort by size (larger images first)
        images.sort_by(|a, b| {
            let size_a = a.width.unwrap_or(0) * a.height.unwrap_or(0);
            let size_b = b.width.unwrap_or(0) * b.height.unwrap_or(0);
            size_b.cmp(&size_a)
        });

        Ok(images)
    }

    /// Extract the primary image from an infobox.
    fn extract_infobox_image(&self, content: &str) -> Option<String> {
        // Look for image parameter in infobox
        let image_patterns = [
            r"\|\s*image\s*=\s*([^\|\}\n]+)",
            r"\|\s*logo\s*=\s*([^\|\}\n]+)",
            r"\|\s*photo\s*=\s*([^\|\}\n]+)",
        ];

        for pattern in image_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(content) {
                    if let Some(m) = caps.get(1) {
                        let image_name = m.as_str().trim();
                        if !image_name.is_empty() && !image_name.contains("{{") {
                            return Some(image_name.to_string());
                        }
                    }
                }
            }
        }

        None
    }
}

#[async_trait]
impl DeviceSource for WikipediaSource {
    fn name(&self) -> &str {
        "Wikipedia"
    }

    fn priority(&self) -> u8 {
        50 // Medium priority - good for general info
    }

    fn supports(&self, _device_type: &DeviceType, _identifier: &DeviceIdentifier) -> bool {
        // Wikipedia has articles for most hardware
        true
    }

    async fn fetch(
        &self,
        device_type: &DeviceType,
        identifier: &DeviceIdentifier,
    ) -> Result<PartialDeviceInfo> {
        // Build search query
        let query = Self::make_search_query(device_type, identifier);
        log::debug!("Wikipedia search query: {}", query);

        // Search for the page
        let title = self
            .search_page(&query)
            .await?
            .context("No Wikipedia article found")?;

        log::debug!("Found Wikipedia article: {}", title);

        // Fetch page content
        let content = self.fetch_page_content(&title).await?;

        // Parse infobox
        let raw_specs = self.parse_infobox(&content);

        if raw_specs.is_empty() {
            return Err(anyhow::anyhow!(
                "No specifications found in Wikipedia article"
            ));
        }

        // Extract device-specific specs
        let (specs, categories) =
            self.extract_device_specs(&raw_specs, device_type, &identifier.model);

        // Get release date
        let release_date = Self::extract_release_date(&raw_specs);

        // Build Wikipedia URL
        let wiki_url = format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_"));

        // Try to extract infobox image first
        let infobox_image = self.extract_infobox_image(&content);

        // Fetch page images
        let images = self.fetch_page_images(&title).await.unwrap_or_default();

        // Use infobox image if found, otherwise use first page image
        let primary_image = if let Some(img_name) = infobox_image {
            // Need to resolve the image name to a URL
            let img_titles = vec![format!("File:{}", img_name)];
            if let Ok(resolved) = self.fetch_image_urls(&img_titles).await {
                resolved.into_iter().next().map(|i| i.url)
            } else {
                None
            }
        } else {
            images.first().map(|i| i.url.clone())
        };

        // Build gallery from remaining images
        let image_gallery: Vec<(String, Option<String>)> = images
            .iter()
            .skip(if primary_image.is_some() { 1 } else { 0 })
            .take(5) // Limit to 5 gallery images
            .map(|img| (img.url.clone(), None))
            .collect();

        Ok(PartialDeviceInfo {
            specs,
            categories,
            description: raw_specs.get("name").cloned(),
            release_date,
            product_page: Some(wiki_url.clone()),
            support_page: None,
            image_url: primary_image,
            source_name: "Wikipedia".to_string(),
            source_url: Some(wiki_url),
            confidence: 0.7, // Wikipedia is generally reliable
            image_cached_path: None,
            thumbnail_url: None,
            thumbnail_cached_path: None,
            image_gallery,
            documentation: None,
            driver_info: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_wiki_value() {
        assert_eq!(
            WikipediaSource::clean_wiki_value("[[GDDR6X|GDDR6X]]"),
            "GDDR6X"
        );
        assert_eq!(WikipediaSource::clean_wiki_value("[[GDDR6]]"), "GDDR6");
        assert_eq!(
            WikipediaSource::clean_wiki_value("{{nowrap|2520 MHz}}"),
            "2520 MHz"
        );
    }

    #[test]
    fn test_extract_intel_generation() {
        assert_eq!(
            WikipediaSource::extract_intel_generation("Core i9-13900K"),
            Some(13)
        );
        assert_eq!(
            WikipediaSource::extract_intel_generation("Core i7-12700K"),
            Some(12)
        );
        assert_eq!(
            WikipediaSource::extract_intel_generation("Core i5-9600K"),
            Some(9)
        );
    }

    #[test]
    fn test_extract_nvidia_series() {
        assert_eq!(
            WikipediaSource::extract_nvidia_series("GeForce RTX 5070"),
            Some("50")
        );
        assert_eq!(
            WikipediaSource::extract_nvidia_series("GeForce RTX 4090"),
            Some("40")
        );
        assert_eq!(
            WikipediaSource::extract_nvidia_series("GeForce RTX 3080"),
            Some("30")
        );
    }

    #[test]
    fn test_make_search_query() {
        let gpu_id = DeviceIdentifier {
            manufacturer: "NVIDIA".to_string(),
            model: "GeForce RTX 5070".to_string(),
            part_number: None,
            serial_number: None,
            hardware_ids: vec![],
        };
        let query = WikipediaSource::make_search_query(&DeviceType::Gpu, &gpu_id);
        assert_eq!(query, "GeForce RTX 50 series");
    }
}
