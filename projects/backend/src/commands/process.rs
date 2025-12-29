//! Process-related Tauri commands

use crate::collectors::ProcessCollector;
use crate::models::{ProcessInfo, ProcessSummary};
use sysinfo::{Pid, System};

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

/// Kill a process by PID
/// Returns true if the process was successfully killed, false otherwise
#[tauri::command]
pub fn kill_process(pid: u32) -> Result<bool, String> {
    log::info!("Command: kill_process(pid={})", pid);

    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(pid)]));

    if let Some(process) = sys.process(Pid::from_u32(pid)) {
        let name = process.name().to_string_lossy().to_string();
        let killed = process.kill();

        if killed {
            log::info!("Successfully killed process: {} (PID: {})", name, pid);
            Ok(true)
        } else {
            log::warn!("Failed to kill process: {} (PID: {})", name, pid);
            Err(format!("Failed to kill process {} (PID: {}). Access may be denied.", name, pid))
        }
    } else {
        log::warn!("Process not found: PID {}", pid);
        Err(format!("Process with PID {} not found", pid))
    }
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
