// Daemon main entry point
// This will orchestrate all components and run the main event loop

use chrono::Local;
use kbd_backlight::{
    brightness::BrightnessController,
    config::Config,
    ipc::{IpcMessage, IpcResponse, IpcServer, StatusInfo, DEFAULT_SOCKET_PATH},
    location::LocationDetector,
    monitors::{FullscreenMonitor, IdleMonitor},
    power::{PowerDetector, PowerState},
    rules::{RuleEngine, SystemContext},
    video_detector::VideoDetector,
    Result,
};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::signal;
use tokio::time::{interval, Duration};

/// Main daemon struct that orchestrates all components
struct Daemon {
    brightness_controller: BrightnessController,
    idle_monitor: Arc<RwLock<IdleMonitor>>,
    current_idle_timeout: Arc<RwLock<u64>>,
    fullscreen_monitor: Option<Arc<FullscreenMonitor>>,
    video_detector: Option<VideoDetector>,
    location_detector: LocationDetector,
    power_detector: PowerDetector,
    rule_engine: Arc<RwLock<RuleEngine>>,
    config: Arc<RwLock<Config>>,
    ipc_server: IpcServer,
    current_brightness: Arc<RwLock<u32>>,
    last_ssid: Arc<RwLock<Option<String>>>,
}

impl Daemon {
    /// Create a new Daemon instance with all components initialized
    async fn new() -> Result<Self> {
        // Load configuration
        let config = Config::load().map_err(|e| {
            if e.is_config_error() {
                eprintln!("Configuration error: {}", e);
                eprintln!("\nPlease fix the configuration file and try again.");
            }
            e
        })?;
        let config = Arc::new(RwLock::new(config));

        // Initialize brightness controller
        let brightness_path = PathBuf::from("/sys/class/leds/platform::kbd_backlight");
        let brightness_controller = BrightnessController::new(brightness_path).map_err(|e| {
            eprintln!("Failed to initialize brightness controller: {}", e);
            e
        })?;

        // Get initial brightness
        let current_brightness = brightness_controller.get_brightness()?;
        let current_brightness = Arc::new(RwLock::new(current_brightness));

        // Initialize idle monitor with timeout from active profile
        let idle_timeout = {
            let cfg = config.read().unwrap();
            let profile = cfg.profiles.get(&cfg.active_profile).unwrap();
            profile.idle_timeout
        };
        let idle_monitor = Arc::new(RwLock::new(IdleMonitor::new(idle_timeout)));
        let current_idle_timeout = Arc::new(RwLock::new(idle_timeout));

        // Initialize fullscreen monitor with graceful degradation
        let fullscreen_monitor = match FullscreenMonitor::new() {
            Ok(monitor) => {
                println!("Fullscreen monitor initialized successfully");
                Some(Arc::new(monitor))
            }
            Err(e) => {
                if e.is_recoverable() {
                    eprintln!("Warning: {}", e);
                    eprintln!("Continuing without fullscreen detection...");
                    None
                } else {
                    return Err(e);
                }
            }
        };

        // Initialize video detector
        let video_detector = match VideoDetector::new().await {
            Ok(detector) => {
                println!("Video detector initialized successfully");
                Some(detector)
            }
            Err(e) => {
                eprintln!("Warning: Failed to initialize video detector: {}", e);
                eprintln!("Video detector: Using fullscreen detection instead");
                None
            }
        };

        // Initialize location detector
        let location_detector = LocationDetector::new();
        println!("Location detector initialized");

        // Initialize power detector
        let power_detector = PowerDetector::new();
        println!("Power detector initialized");

        // Initialize rule engine
        let rule_engine = Arc::new(RwLock::new(RuleEngine::new(Arc::clone(&config))));

        // Initialize IPC server
        let ipc_server = IpcServer::new(DEFAULT_SOCKET_PATH).await.map_err(|e| {
            eprintln!("Failed to create IPC server: {}", e);
            e
        })?;

        Ok(Self {
            brightness_controller,
            idle_monitor,
            current_idle_timeout,
            fullscreen_monitor,
            video_detector,
            location_detector,
            power_detector,
            rule_engine,
            config,
            ipc_server,
            current_brightness,
            last_ssid: Arc::new(RwLock::new(None)),
        })
    }

    /// Run the main daemon event loop
    async fn run(&mut self) -> Result<()> {
        println!("Daemon started successfully");
        println!("IPC socket: {}", self.ipc_server.socket_path().display());

        // Create periodic timers
        let mut monitor_interval = interval(Duration::from_secs(1)); // Poll monitors every second
        let mut schedule_interval = interval(Duration::from_secs(60)); // Evaluate time schedules every minute

        loop {
            tokio::select! {
                // Handle shutdown signals
                _ = signal::ctrl_c() => {
                    println!("Received SIGINT, shutting down...");
                    break;
                }
                _ = Self::wait_for_sigterm() => {
                    println!("Received SIGTERM, shutting down...");
                    break;
                }

                // Handle IPC connections
                Ok(mut stream) = self.ipc_server.accept() => {
                    // Receive message
                    match IpcMessage::receive(&mut stream).await {
                        Ok(message) => {
                            let response = self.handle_ipc_message(message);
                            if let Err(e) = response.send(&mut stream).await {
                                eprintln!("Error sending IPC response: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error receiving IPC message: {}", e);
                        }
                    }
                }

                // Periodic monitor polling (every second)
                _ = monitor_interval.tick() => {
                    if let Err(e) = self.evaluate_and_apply_rules().await {
                        eprintln!("Error evaluating rules: {}", e);
                    }
                }

                // Periodic time schedule evaluation (every minute)
                _ = schedule_interval.tick() => {
                    if let Err(e) = self.evaluate_and_apply_rules().await {
                        eprintln!("Error evaluating time schedules: {}", e);
                    }
                }
            }
        }

        println!("Daemon shutdown complete");
        Ok(())
    }

    /// Evaluate rules and apply brightness changes
    async fn evaluate_and_apply_rules(&mut self) -> Result<()> {
        // Check for location-based profile switching
        self.check_location_profile_switch();

        // Get current power state
        let power_state = self
            .power_detector
            .get_power_state()
            .unwrap_or(PowerState::Unknown);

        // Get profile idle timeout and video detection settings
        let (idle_timeout, video_detection_enabled, ac_always_on) = {
            let config = self.config.read().unwrap();
            let profile = config.profiles.get(&config.active_profile).unwrap();
            (profile.idle_timeout, profile.video_detection_enabled, profile.ac_always_on)
        };

        // Update idle monitor timeout only if changed (to avoid file descriptor leak)
        let current_timeout = *self.current_idle_timeout.read().unwrap();
        if current_timeout != idle_timeout {
            *self.idle_monitor.write().unwrap() = IdleMonitor::new(idle_timeout);
            *self.current_idle_timeout.write().unwrap() = idle_timeout;
        }

        // Check idle state with error handling
        let is_idle = match self.idle_monitor.read().unwrap().is_idle() {
            Ok(idle) => idle,
            Err(e) => {
                if e.is_recoverable() {
                    eprintln!("Warning: Idle detection failed: {}. Assuming not idle.", e);
                    false
                } else {
                    return Err(e);
                }
            }
        };

        // Check video playback state (replaces fullscreen for video detection)
        let is_video_playing = if video_detection_enabled {
            if let Some(ref detector) = self.video_detector {
                detector.is_video_playing().await.unwrap_or(false)
            } else {
                false
            }
        } else {
            // Fall back to fullscreen detection if video detection is disabled
            if let Some(ref monitor) = self.fullscreen_monitor {
                monitor.is_fullscreen_active().unwrap_or(false)
            } else {
                false
            }
        };

        // Get current time
        let current_time = Local::now();

        // Get previous brightness
        let previous_brightness = *self.current_brightness.read().unwrap();

        // Create system context
        let context = SystemContext {
            is_idle,
            is_fullscreen: is_video_playing, // Use video playing as "fullscreen" for now
            current_time,
            previous_brightness,
        };

        // Evaluate rules
        let decision = self.rule_engine.read().unwrap().evaluate(&context);
        let has_manual_override = self.rule_engine.read().unwrap().manual_override.is_some();

        // Apply brightness decision with optional AC power handling
        if let kbd_backlight::rules::BrightnessDecision::SetBrightness(mut brightness) = decision {
            // Optional AC power handling (disabled by default):
            // - If ac_always_on is enabled: Keep brightness at 1 when on AC (except during video or manual override)
            // - If disabled: Respect all rules regardless of power state
            if ac_always_on
                && power_state == PowerState::AC
                && !is_video_playing
                && !has_manual_override
            {
                brightness = 1;
            }

            if brightness != previous_brightness {
                self.brightness_controller.set_brightness(brightness)?;
                *self.current_brightness.write().unwrap() = brightness;
                println!(
                    "Brightness changed: {} -> {} (idle: {}, video: {}, power: {:?})",
                    previous_brightness, brightness, is_idle, is_video_playing, power_state
                );
            }
        }

        Ok(())
    }

    /// Check if we should switch profiles based on WiFi location
    fn check_location_profile_switch(&mut self) {
        let config = self.config.read().unwrap();
        if !config.auto_switch_location {
            return;
        }

        // Get current SSID
        if let Ok(Some(ssid)) = self.location_detector.get_current_ssid() {
            let mut last_ssid = self.last_ssid.write().unwrap();

            // Only switch if SSID changed
            if last_ssid.as_ref() != Some(&ssid) {
                // Build location mappings from profiles
                let location_mappings = config.build_location_mappings();

                // Check if we have a mapping for this SSID
                if let Some(profile_name) = location_mappings.get(&ssid).cloned() {
                    if profile_name != config.active_profile {
                        let old_profile = config.active_profile.clone();
                        drop(config);
                        drop(last_ssid);

                        // Switch profile
                        let mut config = self.config.write().unwrap();
                        config.active_profile = profile_name.clone();

                        println!(
                            "Location changed: {} -> Switching profile: {} -> {}",
                            ssid, old_profile, profile_name
                        );

                        // Save active profile state
                        let _ = config.save_active_profile();
                        drop(config);

                        // Update last SSID
                        *self.last_ssid.write().unwrap() = Some(ssid);
                        return;
                    }
                }

                *last_ssid = Some(ssid);
            }
        }
    }

    /// Force immediate rule evaluation and brightness application
    fn force_rule_evaluation(&mut self) -> Result<()> {
        // Check idle state with error handling
        let is_idle = self
            .idle_monitor
            .read()
            .unwrap()
            .is_idle()
            .unwrap_or_else(|e| {
                eprintln!("Warning: Idle detection failed: {}. Assuming not idle.", e);
                false
            });

        // Check fullscreen state with graceful degradation
        let is_fullscreen = if let Some(ref monitor) = self.fullscreen_monitor {
            monitor.is_fullscreen_active().unwrap_or_else(|e| {
                eprintln!(
                    "Warning: Fullscreen detection failed: {}. Assuming not fullscreen.",
                    e
                );
                false
            })
        } else {
            false
        };

        // Get current time
        let current_time = Local::now();

        // Get previous brightness
        let previous_brightness = *self.current_brightness.read().unwrap();

        // Create system context
        let context = SystemContext {
            is_idle,
            is_fullscreen,
            current_time,
            previous_brightness,
        };

        // Evaluate rules
        let decision = self.rule_engine.read().unwrap().evaluate(&context);

        // Apply brightness decision
        if let kbd_backlight::rules::BrightnessDecision::SetBrightness(brightness) = decision {
            self.brightness_controller.set_brightness(brightness)?;
            *self.current_brightness.write().unwrap() = brightness;
            println!("Brightness applied: {}", brightness);
        }

        Ok(())
    }

    /// Handle a single IPC message
    fn handle_ipc_message(&mut self, message: IpcMessage) -> IpcResponse {
        match message {
            IpcMessage::GetStatus => {
                let config = self.config.read().unwrap();
                let current_brightness = *self.current_brightness.read().unwrap();
                let is_idle = self.idle_monitor.read().unwrap().is_idle().unwrap_or(false);
                let is_fullscreen = if let Some(ref monitor) = self.fullscreen_monitor {
                    monitor.is_fullscreen_active().unwrap_or(false)
                } else {
                    false
                };
                let manual_override = self.rule_engine.read().unwrap().manual_override;

                IpcResponse::Status(StatusInfo {
                    active_profile: config.active_profile.clone(),
                    current_brightness,
                    is_idle,
                    is_fullscreen,
                    manual_override,
                })
            }

            IpcMessage::SetProfile(profile_name) => {
                let mut config = self.config.write().unwrap();

                // Check if profile exists
                if !config.profiles.contains_key(&profile_name) {
                    let available: Vec<_> = config.profiles.keys().cloned().collect();
                    return IpcResponse::Error(format!(
                        "Profile '{}' not found. Available profiles: {}",
                        profile_name,
                        available.join(", ")
                    ));
                }

                // Store old profile for logging
                let old_profile = config.active_profile.clone();

                // Update active profile
                config.active_profile = profile_name.clone();

                // Update idle monitor timeout from new profile
                if let Some(profile) = config.profiles.get(&profile_name) {
                    *self.idle_monitor.write().unwrap() = IdleMonitor::new(profile.idle_timeout);
                }

                // Save active profile state to persist the profile change
                if let Err(e) = config.save_active_profile() {
                    // Rollback on save failure
                    config.active_profile = old_profile;
                    return IpcResponse::Error(format!("Failed to save active profile: {}", e));
                }

                // Release the config lock before forcing rule evaluation
                drop(config);

                println!("Profile switched: {} -> {}", old_profile, profile_name);

                // Immediately apply rules from the new profile
                if let Err(e) = self.force_rule_evaluation() {
                    eprintln!("Warning: Failed to apply new profile rules: {}", e);
                }

                IpcResponse::ProfileChanged
            }

            IpcMessage::SetManualBrightness(brightness) => {
                // Validate brightness range
                let max_brightness = match self.brightness_controller.get_max_brightness() {
                    Ok(max) => max,
                    Err(e) => {
                        return IpcResponse::Error(format!("Failed to get max brightness: {}", e))
                    }
                };

                if brightness > max_brightness {
                    return IpcResponse::Error(format!(
                        "Brightness {} exceeds maximum {}",
                        brightness, max_brightness
                    ));
                }

                // Set manual override
                self.rule_engine
                    .write()
                    .unwrap()
                    .set_manual_override(Some(brightness));

                // Apply immediately
                if let Err(e) = self.brightness_controller.set_brightness(brightness) {
                    return IpcResponse::Error(format!("Failed to set brightness: {}", e));
                }

                *self.current_brightness.write().unwrap() = brightness;
                println!("Manual brightness override set to: {}", brightness);
                IpcResponse::BrightnessSet
            }

            IpcMessage::ClearManualOverride => {
                self.rule_engine.write().unwrap().set_manual_override(None);
                println!("Manual brightness override cleared");
                IpcResponse::Ok
            }

            IpcMessage::ListProfiles => {
                let config = self.config.read().unwrap();
                let profiles: Vec<String> = config.profiles.keys().cloned().collect();
                IpcResponse::ProfileList(profiles)
            }

            IpcMessage::AddTimeSchedule {
                profile,
                hour,
                minute,
                brightness,
            } => {
                // Validate inputs
                if hour > 23 {
                    return IpcResponse::Error(format!("Invalid hour {} (must be 0-23)", hour));
                }
                if minute > 59 {
                    return IpcResponse::Error(format!("Invalid minute {} (must be 0-59)", minute));
                }

                let mut config = self.config.write().unwrap();

                // Check if profile exists
                if let Some(prof) = config.profiles.get_mut(&profile) {
                    prof.time_schedules
                        .push(kbd_backlight::config::TimeSchedule {
                            hour,
                            minute,
                            brightness,
                        });

                    // Save profile to its file
                    if let Err(e) = config.save_profile(&profile) {
                        return IpcResponse::Error(format!("Failed to save profile: {}", e));
                    }

                    println!(
                        "Added time schedule to profile '{}': {:02}:{:02} -> brightness {}",
                        profile, hour, minute, brightness
                    );
                    IpcResponse::ScheduleAdded
                } else {
                    IpcResponse::Error(format!("Profile '{}' not found", profile))
                }
            }

            IpcMessage::Shutdown => {
                println!("Shutdown requested via IPC");
                std::process::exit(0);
            }
        }
    }

    /// Wait for SIGTERM signal
    async fn wait_for_sigterm() {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm =
                signal(SignalKind::terminate()).expect("Failed to setup SIGTERM handler");
            sigterm.recv().await;
        }

        #[cfg(not(unix))]
        {
            // On non-Unix systems, just wait forever
            std::future::pending::<()>().await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create and run daemon
    let mut daemon = Daemon::new().await?;
    daemon.run().await
}
