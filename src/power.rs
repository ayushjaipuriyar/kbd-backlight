// Power state detection (AC vs Battery)
use std::fs;
use std::path::Path;
use crate::{Result, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    AC,
    Battery,
    Unknown,
}

pub struct PowerDetector;

impl PowerDetector {
    pub fn new() -> Self {
        Self
    }

    /// Get current power state
    pub fn get_power_state(&self) -> Result<PowerState> {
        // Check /sys/class/power_supply/
        let power_supply_path = Path::new("/sys/class/power_supply");
        
        if !power_supply_path.exists() {
            return Ok(PowerState::Unknown);
        }

        // Look for AC adapter
        let entries = fs::read_dir(power_supply_path)
            .map_err(Error::Io)?;

        for entry in entries {
            let entry = entry.map_err(Error::Io)?;
            let path = entry.path();
            
            // Check if this is an AC adapter
            let type_file = path.join("type");
            if let Ok(device_type) = fs::read_to_string(&type_file) {
                if device_type.trim() == "Mains" {
                    // This is the AC adapter, check if it's online
                    let online_file = path.join("online");
                    if let Ok(online) = fs::read_to_string(&online_file) {
                        let is_online = online.trim() == "1";
                        return Ok(if is_online {
                            PowerState::AC
                        } else {
                            PowerState::Battery
                        });
                    }
                }
            }
        }

        // Fallback: check battery status
        let entries = fs::read_dir(power_supply_path)
            .map_err(Error::Io)?;

        for entry in entries {
            let entry = entry.map_err(Error::Io)?;
            let path = entry.path();
            
            let type_file = path.join("type");
            if let Ok(device_type) = fs::read_to_string(&type_file) {
                if device_type.trim() == "Battery" {
                    let status_file = path.join("status");
                    if let Ok(status) = fs::read_to_string(&status_file) {
                        let status = status.trim();
                        return Ok(match status {
                            "Charging" | "Full" => PowerState::AC,
                            "Discharging" => PowerState::Battery,
                            _ => PowerState::Unknown,
                        });
                    }
                }
            }
        }

        Ok(PowerState::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_detector_creation() {
        let detector = PowerDetector::new();
        let state = detector.get_power_state();
        assert!(state.is_ok());
    }
}
