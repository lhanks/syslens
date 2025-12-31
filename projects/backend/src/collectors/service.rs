//! Windows service information collector

use crate::models::{ServiceInfo, ServiceSummary};
use std::process::Command;

/// Collector for Windows service information using sc.exe and PowerShell
pub struct ServiceCollector;

impl ServiceCollector {
    /// Get all Windows services
    pub fn get_services() -> Vec<ServiceInfo> {
        // Use PowerShell with a single WMI query (much faster than per-service queries)
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                r#"Get-WmiObject Win32_Service | Select-Object Name, DisplayName, State, StartMode, Description, PathName, StartName, ProcessId | ForEach-Object {
                    [PSCustomObject]@{
                        Name = $_.Name
                        DisplayName = $_.DisplayName
                        Status = $_.State
                        StartType = $_.StartMode
                        Description = $_.Description
                        PathName = $_.PathName
                        StartName = $_.StartName
                        ProcessId = if($_.ProcessId -gt 0) { $_.ProcessId } else { $null }
                    }
                } | ConvertTo-Json -Compress"#,
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let json = String::from_utf8_lossy(&output.stdout);
                Self::parse_services_json(&json)
            }
            _ => {
                log::warn!("Failed to get services via PowerShell, falling back to sc.exe");
                Self::get_services_fallback()
            }
        }
    }

    /// Parse JSON output from PowerShell
    fn parse_services_json(json: &str) -> Vec<ServiceInfo> {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct PsService {
            name: String,
            display_name: String,
            status: String,
            start_type: String,
            description: Option<String>,
            path_name: Option<String>,
            start_name: Option<String>,
            process_id: Option<u32>,
        }

        // Handle both array and single object (PowerShell returns object if only one item)
        let services: Vec<PsService> = if json.trim().starts_with('[') {
            serde_json::from_str(json).unwrap_or_default()
        } else {
            serde_json::from_str::<PsService>(json)
                .map(|s| vec![s])
                .unwrap_or_default()
        };

        services
            .into_iter()
            .map(|s| ServiceInfo {
                name: s.name,
                display_name: s.display_name,
                status: s.status,
                startup_type: s.start_type,
                description: s.description,
                binary_path: s.path_name,
                service_account: s.start_name,
                pid: s.process_id,
            })
            .collect()
    }

    /// Fallback method using sc.exe (less comprehensive but faster)
    fn get_services_fallback() -> Vec<ServiceInfo> {
        let output = Command::new("sc")
            .args(["query", "type=", "service", "state=", "all"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let text = String::from_utf8_lossy(&output.stdout);
                Self::parse_sc_output(&text)
            }
            _ => Vec::new(),
        }
    }

    /// Parse sc.exe output
    fn parse_sc_output(text: &str) -> Vec<ServiceInfo> {
        let mut services = Vec::new();
        let mut current_service: Option<ServiceInfo> = None;

        for line in text.lines() {
            let line = line.trim();

            if line.starts_with("SERVICE_NAME:") {
                if let Some(svc) = current_service.take() {
                    services.push(svc);
                }
                let name = line.trim_start_matches("SERVICE_NAME:").trim().to_string();
                current_service = Some(ServiceInfo {
                    name,
                    display_name: String::new(),
                    status: "Unknown".to_string(),
                    startup_type: "Unknown".to_string(),
                    description: None,
                    binary_path: None,
                    service_account: None,
                    pid: None,
                });
            } else if let Some(ref mut svc) = current_service {
                if line.starts_with("DISPLAY_NAME:") {
                    svc.display_name = line.trim_start_matches("DISPLAY_NAME:").trim().to_string();
                } else if line.starts_with("STATE") {
                    // STATE              : 4  RUNNING
                    if line.contains("RUNNING") {
                        svc.status = "Running".to_string();
                    } else if line.contains("STOPPED") {
                        svc.status = "Stopped".to_string();
                    } else if line.contains("START_PENDING") {
                        svc.status = "StartPending".to_string();
                    } else if line.contains("STOP_PENDING") {
                        svc.status = "StopPending".to_string();
                    }
                } else if line.starts_with("PID") {
                    // PID                : 1234
                    if let Some(pid_str) = line.split(':').nth(1) {
                        if let Ok(pid) = pid_str.trim().parse::<u32>() {
                            if pid > 0 {
                                svc.pid = Some(pid);
                            }
                        }
                    }
                }
            }
        }

        if let Some(svc) = current_service {
            services.push(svc);
        }

        services
    }

    /// Get service summary statistics
    pub fn get_service_summary() -> ServiceSummary {
        let services = Self::get_services();

        let mut summary = ServiceSummary {
            total: services.len() as u32,
            running: 0,
            stopped: 0,
            start_pending: 0,
            stop_pending: 0,
        };

        for service in &services {
            match service.status.as_str() {
                "Running" => summary.running += 1,
                "Stopped" => summary.stopped += 1,
                "StartPending" => summary.start_pending += 1,
                "StopPending" => summary.stop_pending += 1,
                _ => {}
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_services() {
        let services = ServiceCollector::get_services();
        // Should have at least some services on Windows
        assert!(!services.is_empty(), "Should find at least one service");

        // Check that we got real service data
        let first = &services[0];
        assert!(!first.name.is_empty(), "Service name should not be empty");
    }

    #[test]
    fn test_get_service_summary() {
        let summary = ServiceCollector::get_service_summary();
        assert!(summary.total > 0, "Should have at least one service");
        assert!(
            summary.running > 0,
            "Should have at least one running service"
        );
    }
}
