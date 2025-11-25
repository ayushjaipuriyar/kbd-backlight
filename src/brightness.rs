// Brightness controller module
// This module will handle direct interface to sysfs brightness control

use std::fs;
use std::path::PathBuf;
use crate::{Error, Result};

#[derive(Debug)]
pub struct BrightnessController {
    path: PathBuf,
    max_brightness: u32,
}

impl BrightnessController {
    /// Create a new BrightnessController
    /// 
    /// # Arguments
    /// * `path` - Path to the brightness sysfs directory (e.g., /sys/class/leds/platform::kbd_backlight)
    /// 
    /// # Returns
    /// * `Result<Self>` - A new BrightnessController or an error if initialization fails
    pub fn new(path: PathBuf) -> Result<Self> {
        // Check if the path exists
        if !path.exists() {
            return Err(Error::PathNotFound { path });
        }

        // Read max_brightness
        let max_brightness_path = path.join("max_brightness");
        let max_brightness_str = fs::read_to_string(&max_brightness_path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Error::permission_denied(max_brightness_path.clone())
                } else {
                    Error::Io(e)
                }
            })?;
        
        let max_brightness = max_brightness_str.trim().parse::<u32>()
            .map_err(|e| Error::Parse(format!("Failed to parse max_brightness from '{}': {}", max_brightness_str.trim(), e)))?;

        let controller = Self {
            path,
            max_brightness,
        };

        // Validate write access on initialization
        controller.validate_access()?;

        Ok(controller)
    }

    /// Set the brightness to a specific value
    /// 
    /// # Arguments
    /// * `value` - Brightness value to set (must be between 0 and max_brightness)
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub fn set_brightness(&self, value: u32) -> Result<()> {
        // Validate brightness range
        if value > self.max_brightness {
            return Err(Error::InvalidBrightness(
                format!("Value {} exceeds maximum brightness {}. Valid range: 0-{}", 
                    value, self.max_brightness, self.max_brightness)
            ));
        }

        let brightness_path = self.path.join("brightness");
        
        // Try to write, with retry logic for transient failures
        let mut last_error = None;
        for attempt in 0..2 {
            match fs::write(&brightness_path, value.to_string()) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        return Err(Error::permission_denied(brightness_path.clone()));
                    }
                    last_error = Some(e);
                    if attempt == 0 {
                        // Brief pause before retry
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                }
            }
        }

        Err(Error::Io(last_error.unwrap()))
    }

    /// Get the current brightness value
    /// 
    /// # Returns
    /// * `Result<u32>` - Current brightness value or error
    pub fn get_brightness(&self) -> Result<u32> {
        let brightness_path = self.path.join("brightness");
        let brightness_str = fs::read_to_string(&brightness_path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Error::permission_denied(brightness_path.clone())
                } else {
                    Error::Io(e)
                }
            })?;
        
        brightness_str.trim().parse::<u32>()
            .map_err(|e| Error::Parse(format!("Failed to parse brightness from '{}': {}", brightness_str.trim(), e)))
    }

    /// Get the maximum brightness value supported by the hardware
    /// 
    /// # Returns
    /// * `Result<u32>` - Maximum brightness value
    pub fn get_max_brightness(&self) -> Result<u32> {
        Ok(self.max_brightness)
    }

    /// Validate that we have write access to the brightness control file
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if we have access, error otherwise
    pub fn validate_access(&self) -> Result<()> {
        let brightness_path = self.path.join("brightness");
        
        // Try to read the current brightness to validate access
        let current = self.get_brightness()?;
        
        // Try to write the same value back to validate write access
        fs::write(&brightness_path, current.to_string())
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Error::permission_denied(brightness_path.clone())
                } else {
                    Error::Io(e)
                }
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn setup_mock_sysfs(max_brightness: u32, initial_brightness: u32) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        
        fs::write(path.join("max_brightness"), max_brightness.to_string()).unwrap();
        fs::write(path.join("brightness"), initial_brightness.to_string()).unwrap();
        
        temp_dir
    }

    #[test]
    fn test_new_controller_success() {
        let temp_dir = setup_mock_sysfs(3, 2);
        let controller = BrightnessController::new(temp_dir.path().to_path_buf());
        
        assert!(controller.is_ok());
        let controller = controller.unwrap();
        assert_eq!(controller.get_max_brightness().unwrap(), 3);
    }

    #[test]
    fn test_new_controller_path_not_found() {
        let result = BrightnessController::new(PathBuf::from("/nonexistent/path"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PathNotFound { .. }));
    }

    #[test]
    fn test_set_and_get_brightness() {
        let temp_dir = setup_mock_sysfs(3, 0);
        let controller = BrightnessController::new(temp_dir.path().to_path_buf()).unwrap();
        
        controller.set_brightness(2).unwrap();
        assert_eq!(controller.get_brightness().unwrap(), 2);
        
        controller.set_brightness(0).unwrap();
        assert_eq!(controller.get_brightness().unwrap(), 0);
        
        controller.set_brightness(3).unwrap();
        assert_eq!(controller.get_brightness().unwrap(), 3);
    }

    #[test]
    fn test_set_brightness_exceeds_max() {
        let temp_dir = setup_mock_sysfs(3, 1);
        let controller = BrightnessController::new(temp_dir.path().to_path_buf()).unwrap();
        
        let result = controller.set_brightness(4);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidBrightness(_)));
    }

    #[test]
    fn test_brightness_range_validation() {
        let temp_dir = setup_mock_sysfs(100, 50);
        let controller = BrightnessController::new(temp_dir.path().to_path_buf()).unwrap();
        
        // Valid values should work
        assert!(controller.set_brightness(0).is_ok());
        assert!(controller.set_brightness(50).is_ok());
        assert!(controller.set_brightness(100).is_ok());
        
        // Invalid values should fail
        assert!(controller.set_brightness(101).is_err());
        assert!(controller.set_brightness(1000).is_err());
    }

    #[test]
    fn test_validate_access() {
        let temp_dir = setup_mock_sysfs(3, 2);
        let controller = BrightnessController::new(temp_dir.path().to_path_buf()).unwrap();
        
        // validate_access is called in new(), so if we got here it worked
        assert!(controller.validate_access().is_ok());
    }
}
