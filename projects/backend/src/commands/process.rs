//! Process-related Tauri commands

use crate::collectors::ProcessCollector;
use crate::models::{ProcessInfo, ProcessSummary};

/// Get list of all running processes
#[tauri::command]
pub fn get_processes() -> Vec<ProcessInfo> {
    log::debug!("Command: get_processes");
    ProcessCollector::get_processes()
}

/// Get process summary statistics
#[tauri::command]
pub fn get_process_summary() -> ProcessSummary {
    log::debug!("Command: get_process_summary");
    ProcessCollector::get_process_summary()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_processes() {
        let processes = get_processes();
        assert!(!processes.is_empty());
    }

    #[test]
    fn test_get_process_summary() {
        let summary = get_process_summary();
        assert!(summary.total_count > 0);
    }
}
