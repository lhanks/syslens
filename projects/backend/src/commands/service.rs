//! Service-related Tauri commands

use crate::collectors::ServiceCollector;
use crate::models::{ServiceInfo, ServiceSummary};

/// Get all Windows services
#[tauri::command]
pub fn get_services() -> Vec<ServiceInfo> {
    log::debug!("Command: get_services");
    ServiceCollector::get_services()
}

/// Get service summary statistics
#[tauri::command]
pub fn get_service_summary() -> ServiceSummary {
    log::debug!("Command: get_service_summary");
    ServiceCollector::get_service_summary()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_services() {
        let services = get_services();
        // Should have at least some services on Windows
        assert!(!services.is_empty());
    }

    #[test]
    fn test_get_service_summary() {
        let summary = get_service_summary();
        assert!(summary.total > 0);
    }
}
