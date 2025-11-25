// Rule engine module
// This module evaluates rules and determines appropriate brightness levels

use std::sync::{Arc, RwLock};
use chrono::{DateTime, Local, Timelike};
use crate::config::{Config, TimeSchedule};

pub struct RuleEngine {
    config: Arc<RwLock<Config>>,
    pub manual_override: Option<u32>,
}

#[derive(Debug, PartialEq)]
pub enum BrightnessDecision {
    SetBrightness(u32),
    NoChange,
}

pub struct SystemContext {
    pub is_idle: bool,
    pub is_fullscreen: bool,
    pub current_time: DateTime<Local>,
    pub previous_brightness: u32,
}

impl RuleEngine {
    /// Create a new RuleEngine with the given configuration
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        Self {
            config,
            manual_override: None,
        }
    }

    /// Evaluate all rules and determine the appropriate brightness level
    /// 
    /// Rule Priority (highest to lowest):
    /// 1. Manual override
    /// 2. Fullscreen detection (brightness = 0)
    /// 3. Idle timeout (brightness = 0)
    /// 4. Time-based schedule
    /// 5. Default brightness (0 if no rules apply)
    pub fn evaluate(&self, context: &SystemContext) -> BrightnessDecision {
        // Priority 1: Manual override
        if let Some(brightness) = self.manual_override {
            return BrightnessDecision::SetBrightness(brightness);
        }

        // Priority 2: Fullscreen detection
        if context.is_fullscreen {
            return BrightnessDecision::SetBrightness(0);
        }

        // Priority 3: Idle timeout
        if context.is_idle {
            return BrightnessDecision::SetBrightness(0);
        }

        // Priority 4: Time-based schedule
        if let Some(brightness) = self.get_time_based_brightness(context) {
            return BrightnessDecision::SetBrightness(brightness);
        }

        // Priority 5: Default brightness (0 if no rules apply)
        BrightnessDecision::SetBrightness(0)
    }

    /// Set or clear manual override
    pub fn set_manual_override(&mut self, brightness: Option<u32>) {
        self.manual_override = brightness;
    }

    /// Get the brightness based on time schedule rules
    /// Returns the brightness from the most recent time rule
    fn get_time_based_brightness(&self, context: &SystemContext) -> Option<u32> {
        let config = self.config.read().ok()?;
        let profile = config.profiles.get(&config.active_profile)?;

        // Find the most recent time schedule rule
        let current_minutes = context.current_time.hour() * 60 + context.current_time.minute();
        
        let mut applicable_schedule: Option<&TimeSchedule> = None;
        let mut best_minutes: Option<u32> = None;

        for schedule in &profile.time_schedules {
            let schedule_minutes = schedule.hour as u32 * 60 + schedule.minute as u32;
            
            // Only consider schedules that have already occurred today
            if schedule_minutes <= current_minutes {
                // If this is the first applicable schedule or it's more recent than the current best
                if best_minutes.is_none() || schedule_minutes > best_minutes.unwrap() {
                    applicable_schedule = Some(schedule);
                    best_minutes = Some(schedule_minutes);
                }
            }
        }

        applicable_schedule.map(|s| s.brightness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, LocationProfile, TimeSchedule};
    use std::collections::HashMap;
    use chrono::Local;

    fn create_test_config() -> Arc<RwLock<Config>> {
        let mut profiles = HashMap::new();
        profiles.insert(
            "test".to_string(),
            LocationProfile {
                name: "test".to_string(),
                idle_timeout: 10,
                time_schedules: vec![
                    TimeSchedule {
                        hour: 9,
                        minute: 0,
                        brightness: 2,
                    },
                    TimeSchedule {
                        hour: 14,
                        minute: 30,
                        brightness: 3,
                    },
                    TimeSchedule {
                        hour: 22,
                        minute: 0,
                        brightness: 1,
                    },
                ],
                video_detection_enabled: true,
                wifi_networks: vec![],
                ac_always_on: false,
            },
        );

        Arc::new(RwLock::new(Config {
            profiles,
            active_profile: "test".to_string(),
            auto_switch_location: false,
        }))
    }

    fn create_context(is_idle: bool, is_fullscreen: bool, hour: u32, minute: u32) -> SystemContext {
        let now = Local::now();
        let time = now
            .with_hour(hour)
            .unwrap()
            .with_minute(minute)
            .unwrap()
            .with_second(0)
            .unwrap();

        SystemContext {
            is_idle,
            is_fullscreen,
            current_time: time,
            previous_brightness: 2,
        }
    }

    #[test]
    fn test_manual_override_highest_priority() {
        let config = create_test_config();
        let mut engine = RuleEngine::new(config);
        engine.set_manual_override(Some(3));

        // Manual override should take precedence over everything
        let context = create_context(true, true, 10, 0);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(3));
    }

    #[test]
    fn test_fullscreen_priority() {
        let config = create_test_config();
        let engine = RuleEngine::new(config);

        // Fullscreen should set brightness to 0
        let context = create_context(false, true, 10, 0);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(0));
    }

    #[test]
    fn test_idle_priority() {
        let config = create_test_config();
        let engine = RuleEngine::new(config);

        // Idle should set brightness to 0
        let context = create_context(true, false, 10, 0);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(0));
    }

    #[test]
    fn test_time_schedule_rule() {
        let config = create_test_config();
        let engine = RuleEngine::new(config);

        // At 10:00, should use the 9:00 rule (brightness 2)
        let context = create_context(false, false, 10, 0);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(2));

        // At 15:00, should use the 14:30 rule (brightness 3)
        let context = create_context(false, false, 15, 0);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(3));

        // At 23:00, should use the 22:00 rule (brightness 1)
        let context = create_context(false, false, 23, 0);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(1));
    }

    #[test]
    fn test_most_recent_time_rule() {
        let config = create_test_config();
        let engine = RuleEngine::new(config);

        // At 14:30 exactly, should use the 14:30 rule (brightness 3)
        let context = create_context(false, false, 14, 30);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(3));

        // At 14:29, should use the 9:00 rule (brightness 2)
        let context = create_context(false, false, 14, 29);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(2));
    }

    #[test]
    fn test_no_applicable_time_rule() {
        let config = create_test_config();
        let engine = RuleEngine::new(config);

        // At 8:00 (before any schedule), should use default (0)
        let context = create_context(false, false, 8, 0);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(0));
    }

    #[test]
    fn test_rule_priority_order() {
        let config = create_test_config();
        let mut engine = RuleEngine::new(config);

        // Test that manual override beats fullscreen
        engine.set_manual_override(Some(2));
        let context = create_context(false, true, 10, 0);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(2));

        // Clear manual override, fullscreen should now apply
        engine.set_manual_override(None);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(0));

        // Test that fullscreen beats idle
        let context = create_context(true, true, 10, 0);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(0));

        // Test that idle beats time schedule
        let context = create_context(true, false, 10, 0);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(0));
    }

    #[test]
    fn test_set_manual_override() {
        let config = create_test_config();
        let mut engine = RuleEngine::new(config);

        // Set manual override
        engine.set_manual_override(Some(3));
        let context = create_context(false, false, 10, 0);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(3));

        // Clear manual override
        engine.set_manual_override(None);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(2));
    }

    #[test]
    fn test_empty_time_schedules() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "empty".to_string(),
            LocationProfile {
                name: "empty".to_string(),
                idle_timeout: 10,
                time_schedules: vec![],
                video_detection_enabled: true,
                wifi_networks: vec![],
                ac_always_on: false,
            },
        );

        let config = Arc::new(RwLock::new(Config {
            profiles,
            active_profile: "empty".to_string(),
            auto_switch_location: false,
        }));

        let engine = RuleEngine::new(config);
        let context = create_context(false, false, 10, 0);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(0));
    }

    #[test]
    fn test_profile_specific_rules() {
        let mut profiles = HashMap::new();
        
        // Home profile with one schedule
        profiles.insert(
            "home".to_string(),
            LocationProfile {
                name: "home".to_string(),
                idle_timeout: 10,
                time_schedules: vec![
                    TimeSchedule {
                        hour: 9,
                        minute: 0,
                        brightness: 2,
                    },
                ],
                video_detection_enabled: true,
                wifi_networks: vec![],
                ac_always_on: false,
            },
        );
        
        // Office profile with different schedule
        profiles.insert(
            "office".to_string(),
            LocationProfile {
                name: "office".to_string(),
                idle_timeout: 5,
                time_schedules: vec![
                    TimeSchedule {
                        hour: 9,
                        minute: 0,
                        brightness: 3,
                    },
                ],
                video_detection_enabled: true,
                wifi_networks: vec![],
                ac_always_on: false,
            },
        );

        let config = Arc::new(RwLock::new(Config {
            profiles,
            active_profile: "home".to_string(),
            auto_switch_location: false,
        }));

        let engine = RuleEngine::new(Arc::clone(&config));
        
        // At 10:00 with home profile, should get brightness 2
        let context = create_context(false, false, 10, 0);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(2));
        
        // Switch to office profile
        config.write().unwrap().active_profile = "office".to_string();
        
        // At 10:00 with office profile, should get brightness 3
        let context = create_context(false, false, 10, 0);
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(3));
    }

    #[test]
    fn test_multiple_independent_profiles() {
        let mut profiles = HashMap::new();
        
        // Create three different profiles with distinct settings
        profiles.insert(
            "home".to_string(),
            LocationProfile {
                name: "home".to_string(),
                idle_timeout: 10,
                time_schedules: vec![
                    TimeSchedule { hour: 9, minute: 0, brightness: 1 },
                ],
                video_detection_enabled: true,
                wifi_networks: vec![],
                ac_always_on: false,
            },
        );
        
        profiles.insert(
            "office".to_string(),
            LocationProfile {
                name: "office".to_string(),
                idle_timeout: 5,
                time_schedules: vec![
                    TimeSchedule { hour: 9, minute: 0, brightness: 2 },
                ],
                video_detection_enabled: true,
                wifi_networks: vec![],
                ac_always_on: false,
            },
        );
        
        profiles.insert(
            "travel".to_string(),
            LocationProfile {
                name: "travel".to_string(),
                idle_timeout: 15,
                time_schedules: vec![
                    TimeSchedule { hour: 9, minute: 0, brightness: 3 },
                ],
                video_detection_enabled: true,
                wifi_networks: vec![],
                ac_always_on: false,
            },
        );

        let config = Arc::new(RwLock::new(Config {
            profiles,
            active_profile: "home".to_string(),
            auto_switch_location: false,
        }));

        let engine = RuleEngine::new(Arc::clone(&config));
        let context = create_context(false, false, 10, 0);
        
        // Test home profile
        config.write().unwrap().active_profile = "home".to_string();
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(1));
        
        // Test office profile
        config.write().unwrap().active_profile = "office".to_string();
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(2));
        
        // Test travel profile
        config.write().unwrap().active_profile = "travel".to_string();
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(3));
        
        // Verify switching back to home still works
        config.write().unwrap().active_profile = "home".to_string();
        let decision = engine.evaluate(&context);
        assert_eq!(decision, BrightnessDecision::SetBrightness(1));
    }
}
