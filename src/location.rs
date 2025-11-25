// Location detection using WiFi SSID
use std::process::Command;
use crate::{Result, Error};

pub struct LocationDetector;

impl LocationDetector {
    pub fn new() -> Self {
        Self
    }

    /// Get current WiFi SSID
    pub fn get_current_ssid(&self) -> Result<Option<String>> {
        // Try nmcli first (NetworkManager)
        if let Ok(ssid) = self.get_ssid_nmcli() {
            return Ok(Some(ssid));
        }

        // Try iw (wireless tools)
        if let Ok(ssid) = self.get_ssid_iw() {
            return Ok(Some(ssid));
        }

        // No WiFi connection or unable to detect
        Ok(None)
    }

    fn get_ssid_nmcli(&self) -> Result<String> {
        let output = Command::new("nmcli")
            .args(&["-t", "-f", "active,ssid", "dev", "wifi"])
            .output()
            .map_err(|e| Error::Io(e))?;

        if !output.status.success() {
            return Err(Error::Parse("nmcli command failed".to_string()));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Find the active connection
        for line in output_str.lines() {
            if line.starts_with("yes:") {
                let ssid = line.strip_prefix("yes:").unwrap_or("").trim();
                if !ssid.is_empty() {
                    return Ok(ssid.to_string());
                }
            }
        }

        Err(Error::Parse("No active WiFi connection found".to_string()))
    }

    fn get_ssid_iw(&self) -> Result<String> {
        // Get the wireless interface name
        let output = Command::new("sh")
            .arg("-c")
            .arg("iw dev | grep Interface | awk '{print $2}'")
            .output()
            .map_err(|e| Error::Io(e))?;

        let interface = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        if interface.is_empty() {
            return Err(Error::Parse("No wireless interface found".to_string()));
        }

        // Get SSID for the interface
        let output = Command::new("iw")
            .args(&["dev", &interface, "link"])
            .output()
            .map_err(|e| Error::Io(e))?;

        if !output.status.success() {
            return Err(Error::Parse("iw command failed".to_string()));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines() {
            if line.trim().starts_with("SSID:") {
                let ssid = line.split(':').nth(1).unwrap_or("").trim();
                if !ssid.is_empty() {
                    return Ok(ssid.to_string());
                }
            }
        }

        Err(Error::Parse("No SSID found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_detector_creation() {
        let detector = LocationDetector::new();
        // Just verify it can be created
        let _ = detector.get_current_ssid();
    }
}
