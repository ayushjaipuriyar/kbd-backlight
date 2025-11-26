// Configuration management module
// This module will handle loading, validation, and persistence of configuration

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub profiles: HashMap<String, LocationProfile>,
    #[serde(skip)]
    pub active_profile: String,
    #[serde(default)]
    pub auto_switch_location: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationProfile {
    pub name: String,
    pub idle_timeout: u64,
    pub time_schedules: Vec<TimeSchedule>,
    #[serde(default = "default_true")]
    pub video_detection_enabled: bool,
    #[serde(default)]
    pub wifi_networks: Vec<String>, // WiFi SSIDs for this profile
    #[serde(default)]
    pub ac_always_on: bool, // Keep backlight on when on AC power (except during video)
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSchedule {
    pub hour: u8,
    pub minute: u8,
    pub brightness: u32,
}

impl Config {
    /// Load configuration from the XDG config directory
    /// If the file doesn't exist, create a default configuration
    pub fn load() -> Result<Self> {
        let config_dir = Self::get_config_dir();
        let config_path = config_dir.join("config.toml");

        // Create default configuration if it doesn't exist
        if !config_path.exists() {
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        // Load main config.toml
        let content = fs::read_to_string(&config_path).map_err(|e| {
            Error::config_error(
                config_path.display().to_string(),
                format!("Failed to read config file: {}", e),
            )
        })?;

        let mut config: Config = toml::from_str(&content).map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("line") {
                Error::ConfigSyntax(format!(
                    "TOML syntax error in {}: {}",
                    config_path.display(),
                    error_msg
                ))
            } else {
                Error::ConfigSyntax(format!(
                    "Failed to parse config file {}: {}",
                    config_path.display(),
                    error_msg
                ))
            }
        })?;

        // Load all profile files from profiles/ directory
        config.profiles = Self::load_profiles(&config_dir)?;

        // Load active profile from state file
        config.active_profile = Self::load_active_profile(&config_dir)?;

        config.validate()?;
        Ok(config)
    }

    /// Load all profile files from the profiles directory
    fn load_profiles(config_dir: &Path) -> Result<HashMap<String, LocationProfile>> {
        let profiles_dir = config_dir.join("profiles");

        // Create profiles directory if it doesn't exist
        if !profiles_dir.exists() {
            fs::create_dir_all(&profiles_dir).map_err(|e| {
                Error::config_error(
                    profiles_dir.display().to_string(),
                    format!("Failed to create profiles directory: {}", e),
                )
            })?;
        }

        let mut profiles = HashMap::new();
        let mut seen_names = HashSet::new();

        // Read all .toml files in profiles directory
        let entries = fs::read_dir(&profiles_dir).map_err(|e| {
            Error::config_error(
                profiles_dir.display().to_string(),
                format!("Failed to read profiles directory: {}", e),
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                Error::config_error(
                    profiles_dir.display().to_string(),
                    format!("Failed to read directory entry: {}", e),
                )
            })?;

            let path = entry.path();

            // Skip non-files and non-.toml files
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }

            // Load profile
            let content = fs::read_to_string(&path).map_err(|e| {
                Error::config_error(
                    path.display().to_string(),
                    format!("Failed to read profile file: {}", e),
                )
            })?;

            let profile: LocationProfile = toml::from_str(&content).map_err(|e| {
                let error_msg = e.to_string();
                if error_msg.contains("line") {
                    Error::ConfigSyntax(format!(
                        "TOML syntax error in {}: {}",
                        path.display(),
                        error_msg
                    ))
                } else {
                    Error::ConfigSyntax(format!(
                        "Failed to parse profile file {}: {}",
                        path.display(),
                        error_msg
                    ))
                }
            })?;

            // Check for duplicate profile names
            if !seen_names.insert(profile.name.clone()) {
                return Err(Error::ConfigValidation(format!(
                    "Duplicate profile name '{}' found in {}",
                    profile.name,
                    path.display()
                )));
            }

            profiles.insert(profile.name.clone(), profile);
        }

        // Ensure at least one profile exists
        if profiles.is_empty() {
            return Err(Error::ConfigValidation(
                "No profiles found. At least one profile is required.".to_string(),
            ));
        }

        Ok(profiles)
    }

    /// Load active profile from state file
    fn load_active_profile(config_dir: &Path) -> Result<String> {
        let state_path = config_dir.join("state.toml");

        if !state_path.exists() {
            // Default to first available profile
            return Ok("home".to_string());
        }

        let content = fs::read_to_string(&state_path).map_err(|e| {
            Error::config_error(
                state_path.display().to_string(),
                format!("Failed to read state file: {}", e),
            )
        })?;

        #[derive(Deserialize)]
        struct State {
            active_profile: String,
        }

        let state: State = toml::from_str(&content)
            .map_err(|e| Error::ConfigSyntax(format!("Failed to parse state file: {}", e)))?;

        Ok(state.active_profile)
    }

    /// Save active profile to state file
    pub fn save_active_profile(&self) -> Result<()> {
        let config_dir = Self::get_config_dir();
        let state_path = config_dir.join("state.toml");

        #[derive(Serialize)]
        struct State {
            active_profile: String,
        }

        let state = State {
            active_profile: self.active_profile.clone(),
        };

        let content = toml::to_string_pretty(&state).map_err(|e| {
            Error::config_error(
                state_path.display().to_string(),
                format!("Failed to serialize state: {}", e),
            )
        })?;

        fs::write(&state_path, content).map_err(|e| {
            Error::config_error(
                state_path.display().to_string(),
                format!("Failed to write state file: {}", e),
            )
        })?;

        Ok(())
    }

    /// Save configuration to the XDG config directory
    pub fn save(&self) -> Result<()> {
        self.validate()?;

        let config_dir = Self::get_config_dir();
        let config_path = config_dir.join("config.toml");
        let profiles_dir = config_dir.join("profiles");

        // Ensure the config and profiles directories exist
        fs::create_dir_all(&config_dir).map_err(|e| {
            Error::config_error(
                config_dir.display().to_string(),
                format!("Failed to create config directory: {}", e),
            )
        })?;

        fs::create_dir_all(&profiles_dir).map_err(|e| {
            Error::config_error(
                profiles_dir.display().to_string(),
                format!("Failed to create profiles directory: {}", e),
            )
        })?;

        let content = toml::to_string_pretty(self).map_err(|e| {
            Error::config_error(
                config_path.display().to_string(),
                format!("Failed to serialize config: {}", e),
            )
        })?;

        // Write to temporary file first, then rename for atomic operation
        let temp_path = config_dir.join(".config.toml.tmp");

        fs::write(&temp_path, &content).map_err(|e| {
            Error::config_error(
                temp_path.display().to_string(),
                format!("Failed to write temporary config file: {}", e),
            )
        })?;

        fs::rename(&temp_path, &config_path).map_err(|e| {
            // Clean up temp file on error
            let _ = fs::remove_file(&temp_path);
            Error::config_error(
                config_path.display().to_string(),
                format!("Failed to save config file: {}", e),
            )
        })?;

        // Save all profiles
        for profile_name in self.profiles.keys() {
            self.save_profile(profile_name)?;
        }

        // Save active profile state
        self.save_active_profile()?;

        Ok(())
    }

    /// Save a single profile to its own file
    pub fn save_profile(&self, profile_name: &str) -> Result<()> {
        let profile = self.profiles.get(profile_name).ok_or_else(|| {
            Error::ConfigValidation(format!("Profile '{}' not found", profile_name))
        })?;

        let config_dir = Self::get_config_dir();
        let profiles_dir = config_dir.join("profiles");

        // Ensure profiles directory exists
        fs::create_dir_all(&profiles_dir).map_err(|e| {
            Error::config_error(
                profiles_dir.display().to_string(),
                format!("Failed to create profiles directory: {}", e),
            )
        })?;

        let profile_path = profiles_dir.join(format!("{}.toml", profile_name));

        let content = toml::to_string_pretty(profile).map_err(|e| {
            Error::config_error(
                profile_path.display().to_string(),
                format!("Failed to serialize profile: {}", e),
            )
        })?;

        // Write to temporary file first
        let temp_path = profiles_dir.join(format!(".{}.toml.tmp", profile_name));

        fs::write(&temp_path, &content).map_err(|e| {
            Error::config_error(
                temp_path.display().to_string(),
                format!("Failed to write temporary profile file: {}", e),
            )
        })?;

        fs::rename(&temp_path, &profile_path).map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            Error::config_error(
                profile_path.display().to_string(),
                format!("Failed to save profile file: {}", e),
            )
        })?;

        Ok(())
    }

    /// Validate configuration for correctness
    pub fn validate(&self) -> Result<()> {
        // Check that profiles is not empty
        if self.profiles.is_empty() {
            return Err(Error::ConfigValidation(
                "Configuration must have at least one profile".to_string(),
            ));
        }

        // Check that active_profile exists in profiles
        if !self.profiles.contains_key(&self.active_profile) {
            let available: Vec<_> = self.profiles.keys().collect();
            return Err(Error::ConfigValidation(format!(
                "Active profile '{}' not found in profiles. Available profiles: {:?}",
                self.active_profile, available
            )));
        }

        // Check for duplicate WiFi networks across profiles
        let mut wifi_to_profile: HashMap<String, String> = HashMap::new();
        for (profile_name, profile) in &self.profiles {
            for wifi in &profile.wifi_networks {
                if let Some(existing_profile) = wifi_to_profile.get(wifi) {
                    return Err(Error::ConfigValidation(format!(
                        "WiFi network '{}' is assigned to multiple profiles: '{}' and '{}'",
                        wifi, existing_profile, profile_name
                    )));
                }
                wifi_to_profile.insert(wifi.clone(), profile_name.clone());
            }
        }

        // Validate each profile
        for (name, profile) in &self.profiles {
            // Validate profile name matches key
            if profile.name != *name {
                return Err(Error::ConfigValidation(format!(
                    "Profile name '{}' does not match filename (expected '{}.toml')",
                    profile.name, name
                )));
            }

            // Validate idle timeout is reasonable (not too small)
            if profile.idle_timeout == 0 {
                return Err(Error::ConfigValidation(format!(
                    "Profile '{}' has idle_timeout of 0. Use a positive value (recommended: 5-60 seconds)",
                    name
                )));
            }

            // Validate time schedules
            for (idx, schedule) in profile.time_schedules.iter().enumerate() {
                if schedule.hour > 23 {
                    return Err(Error::ConfigValidation(format!(
                        "Profile '{}', schedule #{}: Invalid hour {} (must be 0-23)",
                        name,
                        idx + 1,
                        schedule.hour
                    )));
                }
                if schedule.minute > 59 {
                    return Err(Error::ConfigValidation(format!(
                        "Profile '{}', schedule #{}: Invalid minute {} (must be 0-59)",
                        name,
                        idx + 1,
                        schedule.minute
                    )));
                }
                // Note: brightness validation is hardware-specific, so we don't validate it here
            }
        }

        Ok(())
    }

    /// Get the configuration directory using XDG config directory
    pub fn get_config_dir() -> PathBuf {
        // Use XDG_CONFIG_HOME if set, otherwise use ~/.config
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").expect("HOME environment variable not set");
                PathBuf::from(home).join(".config")
            });

        config_dir.join("kbd-backlight")
    }

    /// Build WiFi SSID to profile name mapping from all profiles
    pub fn build_location_mappings(&self) -> HashMap<String, String> {
        let mut mappings = HashMap::new();
        for (profile_name, profile) in &self.profiles {
            for wifi in &profile.wifi_networks {
                mappings.insert(wifi.clone(), profile_name.clone());
            }
        }
        mappings
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut profiles = HashMap::new();

        // Create a default "home" profile
        profiles.insert(
            "home".to_string(),
            LocationProfile {
                name: "home".to_string(),
                idle_timeout: 30,
                video_detection_enabled: true,
                wifi_networks: vec![],
                ac_always_on: false,
                time_schedules: vec![
                    TimeSchedule {
                        hour: 9,
                        minute: 0,
                        brightness: 1,
                    },
                    TimeSchedule {
                        hour: 22,
                        minute: 0,
                        brightness: 0,
                    },
                ],
            },
        );

        Config {
            profiles,
            active_profile: "home".to_string(),
            auto_switch_location: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Mutex to serialize tests that modify environment variables
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_test_env() -> (TempDir, std::sync::MutexGuard<'static, ()>) {
        let guard = ENV_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        (temp_dir, guard)
    }

    #[test]
    fn test_default_config_creation() {
        let config = Config::default();
        assert_eq!(config.active_profile, "home");
        assert!(config.profiles.contains_key("home"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_save_and_load() {
        let (_temp_dir, _guard) = setup_test_env();

        let config = Config::default();
        assert!(config.save().is_ok());

        let loaded_config = Config::load().unwrap();
        assert_eq!(loaded_config.active_profile, config.active_profile);
        assert_eq!(loaded_config.profiles.len(), config.profiles.len());
    }

    #[test]
    fn test_config_validation_invalid_profile() {
        let config = Config {
            active_profile: "nonexistent".to_string(),
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not found in profiles"));
    }

    #[test]
    fn test_config_validation_invalid_hour() {
        let mut config = Config::default();
        if let Some(profile) = config.profiles.get_mut("home") {
            profile.time_schedules.push(TimeSchedule {
                hour: 25,
                minute: 0,
                brightness: 2,
            });
        }

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid hour"));
    }

    #[test]
    fn test_config_validation_invalid_minute() {
        let mut config = Config::default();
        if let Some(profile) = config.profiles.get_mut("home") {
            profile.time_schedules.push(TimeSchedule {
                hour: 12,
                minute: 60,
                brightness: 2,
            });
        }

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid minute"));
    }

    #[test]
    fn test_toml_serialization() {
        let (_temp_dir, _guard) = setup_test_env();
        let config = Config::default();

        // Save and reload to test serialization
        assert!(config.save().is_ok());
        let loaded = Config::load().unwrap();

        assert_eq!(config.active_profile, loaded.active_profile);
        assert_eq!(config.auto_switch_location, loaded.auto_switch_location);
    }

    #[test]
    fn test_multiple_profiles() {
        let mut config = Config::default();

        config.profiles.insert(
            "office".to_string(),
            LocationProfile {
                name: "office".to_string(),
                idle_timeout: 5,
                video_detection_enabled: true,
                wifi_networks: vec![],
                ac_always_on: false,
                time_schedules: vec![TimeSchedule {
                    hour: 8,
                    minute: 0,
                    brightness: 3,
                }],
            },
        );

        assert!(config.validate().is_ok());
        assert_eq!(config.profiles.len(), 2);
    }

    #[test]
    fn test_profile_independence() {
        let mut config = Config::default();

        // Add a second profile with different settings
        config.profiles.insert(
            "office".to_string(),
            LocationProfile {
                name: "office".to_string(),
                idle_timeout: 5,
                video_detection_enabled: true,
                wifi_networks: vec![],
                ac_always_on: false,
                time_schedules: vec![TimeSchedule {
                    hour: 8,
                    minute: 0,
                    brightness: 3,
                }],
            },
        );

        // Verify home profile is unchanged
        let home = config.profiles.get("home").unwrap();
        assert_eq!(home.idle_timeout, 30);
        assert_eq!(home.time_schedules.len(), 2);

        // Verify office profile has its own settings
        let office = config.profiles.get("office").unwrap();
        assert_eq!(office.idle_timeout, 5);
        assert_eq!(office.time_schedules.len(), 1);
        assert_eq!(office.time_schedules[0].brightness, 3);
    }

    #[test]
    fn test_profile_switching() {
        let mut config = Config::default();

        // Add office profile
        config.profiles.insert(
            "office".to_string(),
            LocationProfile {
                name: "office".to_string(),
                idle_timeout: 5,
                video_detection_enabled: true,
                wifi_networks: vec![],
                ac_always_on: false,
                time_schedules: vec![],
            },
        );

        // Initially on home profile
        assert_eq!(config.active_profile, "home");

        // Switch to office profile
        config.active_profile = "office".to_string();
        assert_eq!(config.active_profile, "office");
        assert!(config.validate().is_ok());

        // Verify we can access the active profile's settings
        let active = config.profiles.get(&config.active_profile).unwrap();
        assert_eq!(active.idle_timeout, 5);
    }

    #[test]
    fn test_profile_persistence() {
        let (_temp_dir, _guard) = setup_test_env();

        let mut config = Config::default();

        // Add office profile
        config.profiles.insert(
            "office".to_string(),
            LocationProfile {
                name: "office".to_string(),
                idle_timeout: 5,
                video_detection_enabled: true,
                wifi_networks: vec![],
                ac_always_on: false,
                time_schedules: vec![],
            },
        );

        // Switch to office profile
        config.active_profile = "office".to_string();

        // Save everything (profiles and state)
        assert!(config.save().is_ok());

        // Load config and verify active profile persisted
        let loaded = Config::load().unwrap();
        assert_eq!(loaded.active_profile, "office");
        assert_eq!(loaded.profiles.len(), 2);
    }
}
