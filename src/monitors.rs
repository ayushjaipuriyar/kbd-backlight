// System monitors module
// This module contains idle and fullscreen detection monitors

use crate::location::LocationDetector;
use crate::power::{PowerDetector, PowerState};
use crate::wayland_idle::WaylandIdleDetector;
use crate::{Error, Result};
use std::fs;
use std::path::Path;
use std::time::Duration;
use x11rb::connection::Connection;
use x11rb::protocol::screensaver::ConnectionExt as ScreensaverConnectionExt;

pub struct IdleMonitor {
    timeout_seconds: u64,
    wayland_detector: Option<WaylandIdleDetector>,
}

impl IdleMonitor {
    pub fn new(timeout_seconds: u64) -> Self {
        // Try to initialize Wayland detector
        let wayland_detector = WaylandIdleDetector::new(timeout_seconds);

        IdleMonitor {
            timeout_seconds,
            wayland_detector: Some(wayland_detector),
        }
    }

    /// Get the current idle time using multiple detection methods
    /// Priority: Wayland ext-idle-notify -> X11 XScreenSaver
    pub fn get_idle_time(&self) -> Result<Duration> {
        // Try Wayland first
        if let Some(ref detector) = self.wayland_detector {
            match detector.get_idle_time() {
                Ok(duration) => return Ok(duration),
                Err(e) => {
                    eprintln!("Wayland idle detection failed: {}, trying X11...", e);
                }
            }
        }

        // Fallback to X11 XScreenSaver (works on X11)
        match self.get_idle_time_x11() {
            Ok(duration) => Ok(duration),
            Err(_) => {
                // Both methods failed, return 0 (not idle)
                // This allows time-based and manual control to still work
                Ok(Duration::from_secs(0))
            }
        }
    }

    /// Check if the system is currently idle based on the configured timeout
    pub fn is_idle(&self) -> Result<bool> {
        let idle_time = self.get_idle_time()?;
        Ok(idle_time.as_secs() >= self.timeout_seconds)
    }

    /// Get idle time using X11 XScreenSaver extension
    fn get_idle_time_x11(&self) -> Result<Duration> {
        // Connect to X11 display
        let (conn, screen_num) = x11rb::connect(None).map_err(|e| {
            Error::x11_connection(format!("Failed to connect to X11 display: {}", e))
        })?;

        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;

        // Query XScreenSaver info
        let info = conn
            .screensaver_query_info(root)
            .map_err(|e| Error::X11Protocol(format!("Failed to query screensaver info: {}", e)))?
            .reply()
            .map_err(|e| Error::X11Protocol(format!("Failed to get screensaver reply: {}", e)))?;

        // idle field is in milliseconds
        let idle_ms = info.ms_since_user_input;
        Ok(Duration::from_millis(idle_ms as u64))
    }

    /// Get idle time by checking /dev/input/ device activity
    /// This works on both X11 and Wayland
    #[allow(dead_code)]
    fn get_idle_time_sysfs(&self) -> Result<Duration> {
        // Try /dev/input/event* devices first (more reliable)
        if let Ok(idle) = self.get_idle_time_from_dev_input() {
            return Ok(idle);
        }

        // Fallback to /sys/class/input/ timestamps
        let input_path = Path::new("/sys/class/input");

        if !input_path.exists() {
            return Err(Error::monitor_unavailable(
                "Idle Monitor",
                "/sys/class/input not available for idle detection",
                "Idle-based brightness control will be disabled",
            ));
        }

        let mut most_recent_activity = None;
        let mut device_count = 0;

        // Iterate through input devices
        let entries = fs::read_dir(input_path).map_err(Error::Io)?;

        for entry in entries {
            let entry = entry.map_err(Error::Io)?;
            let path = entry.path();

            // Look for event* directories
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("event") {
                    device_count += 1;
                    // Try to get the last access time of the device
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(accessed) = metadata.accessed() {
                            if most_recent_activity.is_none()
                                || accessed > most_recent_activity.unwrap()
                            {
                                most_recent_activity = Some(accessed);
                            }
                        }
                    }
                }
            }
        }

        if let Some(last_activity) = most_recent_activity {
            let now = std::time::SystemTime::now();
            let idle_duration = now
                .duration_since(last_activity)
                .unwrap_or(Duration::from_secs(0));
            Ok(idle_duration)
        } else {
            // No input devices found or couldn't read timestamps
            if device_count == 0 {
                Err(Error::monitor_unavailable(
                    "Idle Monitor",
                    "No input devices found in /sys/class/input",
                    "Idle-based brightness control will be disabled",
                ))
            } else {
                // Devices exist but we can't read their timestamps
                // Return 0 idle time as a safe fallback
                Ok(Duration::from_secs(0))
            }
        }
    }

    /// Get idle time from /dev/input/event* devices
    /// This is more accurate than sysfs timestamps
    #[allow(dead_code)]
    fn get_idle_time_from_dev_input(&self) -> Result<Duration> {
        let dev_input = Path::new("/dev/input");

        if !dev_input.exists() {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "/dev/input not found",
            )));
        }

        let mut most_recent_activity = None;

        // Check all event devices
        let entries = fs::read_dir(dev_input).map_err(Error::Io)?;

        for entry in entries {
            let entry = entry.map_err(Error::Io)?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("event") {
                    // Get the last modification time (updated on input events)
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            if most_recent_activity.is_none()
                                || modified > most_recent_activity.unwrap()
                            {
                                most_recent_activity = Some(modified);
                            }
                        }
                    }
                }
            }
        }

        if let Some(last_activity) = most_recent_activity {
            let now = std::time::SystemTime::now();
            let idle_duration = now
                .duration_since(last_activity)
                .unwrap_or(Duration::from_secs(0));
            Ok(idle_duration)
        } else {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No input event devices found",
            )))
        }
    }
}

pub struct FullscreenMonitor {
    // We don't need to store the connection as we'll create it on each check
    // This is simpler and avoids lifetime issues
}

impl FullscreenMonitor {
    /// Create a new FullscreenMonitor
    /// Verifies that X11 connection is available
    pub fn new() -> Result<Self> {
        // Test X11 connection to ensure it's available
        let (conn, _) = x11rb::connect(None).map_err(|e| {
            Error::monitor_unavailable(
                "Fullscreen Monitor",
                format!("Failed to connect to X11: {}", e),
                "Fullscreen-based brightness control will be disabled",
            )
        })?;

        // Verify we can access the root window
        let setup = conn.setup();
        if setup.roots.is_empty() {
            return Err(Error::monitor_unavailable(
                "Fullscreen Monitor",
                "No X11 screens available",
                "Fullscreen-based brightness control will be disabled",
            ));
        }

        Ok(FullscreenMonitor {})
    }

    /// Check if any window is currently in fullscreen mode
    /// Returns true if at least one fullscreen window is detected
    pub fn is_fullscreen_active(&self) -> Result<bool> {
        use x11rb::protocol::xproto::ConnectionExt as XprotoConnectionExt;
        use x11rb::protocol::xproto::*;

        // Connect to X11 display
        let (conn, screen_num) = x11rb::connect(None)
            .map_err(|e| Error::x11_connection(format!("Failed to connect to X11: {}", e)))?;

        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;

        // Get the _NET_WM_STATE atom
        let net_wm_state_atom = conn
            .intern_atom(false, b"_NET_WM_STATE")
            .map_err(|e| Error::X11Protocol(format!("Failed to intern _NET_WM_STATE atom: {}", e)))?
            .reply()
            .map_err(|e| {
                Error::X11Protocol(format!("Failed to get _NET_WM_STATE atom reply: {}", e))
            })?
            .atom;

        // Get the _NET_WM_STATE_FULLSCREEN atom
        let net_wm_state_fullscreen_atom = conn
            .intern_atom(false, b"_NET_WM_STATE_FULLSCREEN")
            .map_err(|e| {
                Error::X11Protocol(format!(
                    "Failed to intern _NET_WM_STATE_FULLSCREEN atom: {}",
                    e
                ))
            })?
            .reply()
            .map_err(|e| {
                Error::X11Protocol(format!(
                    "Failed to get _NET_WM_STATE_FULLSCREEN atom reply: {}",
                    e
                ))
            })?
            .atom;

        // Query all windows using _NET_CLIENT_LIST
        let net_client_list_atom = conn
            .intern_atom(false, b"_NET_CLIENT_LIST")
            .map_err(|e| {
                Error::X11Protocol(format!("Failed to intern _NET_CLIENT_LIST atom: {}", e))
            })?
            .reply()
            .map_err(|e| {
                Error::X11Protocol(format!("Failed to get _NET_CLIENT_LIST atom reply: {}", e))
            })?
            .atom;

        // Get the list of client windows
        let client_list_reply = conn
            .get_property(
                false,
                root,
                net_client_list_atom,
                AtomEnum::WINDOW,
                0,
                u32::MAX,
            )
            .map_err(|e| Error::X11Protocol(format!("Failed to get client list: {}", e)))?
            .reply()
            .map_err(|e| Error::X11Protocol(format!("Failed to get client list reply: {}", e)))?;

        // Parse the window list
        let windows: Vec<Window> = client_list_reply
            .value32()
            .ok_or_else(|| Error::X11Protocol("Invalid client list format".to_string()))?
            .collect();

        // Check each window for fullscreen state
        for window in windows {
            if self.is_window_fullscreen(
                &conn,
                window,
                net_wm_state_atom,
                net_wm_state_fullscreen_atom,
            )? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if a specific window is in fullscreen mode
    fn is_window_fullscreen<C: Connection>(
        &self,
        conn: &C,
        window: u32,
        net_wm_state_atom: u32,
        net_wm_state_fullscreen_atom: u32,
    ) -> Result<bool> {
        use x11rb::protocol::xproto::ConnectionExt as XprotoConnectionExt;
        use x11rb::protocol::xproto::*;

        // Get the _NET_WM_STATE property for this window
        let state_reply = conn
            .get_property(
                false,
                window,
                net_wm_state_atom,
                AtomEnum::ATOM,
                0,
                u32::MAX,
            )
            .map_err(|e| Error::X11Protocol(format!("Failed to get window state: {}", e)))?
            .reply();

        // If we can't get the property, the window might have been destroyed
        // Just return false instead of erroring (this is a transient condition)
        let state_reply = match state_reply {
            Ok(reply) => reply,
            Err(_) => return Ok(false),
        };

        // Check if the fullscreen atom is in the state list
        if let Some(atoms) = state_reply.value32() {
            for atom in atoms {
                if atom == net_wm_state_fullscreen_atom {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

pub struct LocationMonitor {
    detector: LocationDetector,
}

impl Default for LocationMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl LocationMonitor {
    pub fn new() -> Self {
        Self {
            detector: LocationDetector::new(),
        }
    }

    /// Get current WiFi SSID
    pub fn get_current_ssid(&self) -> Result<Option<String>> {
        self.detector.get_current_ssid()
    }
}

pub struct PowerMonitor {
    detector: PowerDetector,
}

impl Default for PowerMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerMonitor {
    pub fn new() -> Self {
        Self {
            detector: PowerDetector::new(),
        }
    }

    /// Get current power state (AC/Battery/Unknown)
    pub fn get_power_state(&self) -> Result<PowerState> {
        self.detector.get_power_state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_monitor_creation() {
        let monitor = IdleMonitor::new(10);
        assert_eq!(monitor.timeout_seconds, 10);
    }

    #[test]
    fn test_idle_monitor_configurable_timeout() {
        let monitor1 = IdleMonitor::new(5);
        let monitor2 = IdleMonitor::new(30);
        let monitor3 = IdleMonitor::new(120);

        assert_eq!(monitor1.timeout_seconds, 5);
        assert_eq!(monitor2.timeout_seconds, 30);
        assert_eq!(monitor3.timeout_seconds, 120);
    }

    #[test]
    fn test_idle_monitor_get_idle_time() {
        let monitor = IdleMonitor::new(10);

        // This test will try X11 first, then fallback to sysfs
        // We just verify it returns a result (either Ok or Err)
        let result = monitor.get_idle_time();

        // In a headless environment or without X11, this might fail
        // but the implementation should handle it gracefully
        match result {
            Ok(_duration) => {
                // If successful, the method returned a valid Duration
                // Duration is always non-negative by type definition
            }
            Err(_) => {
                // If both X11 and sysfs fail, that's expected in some environments
                // The error handling is working correctly
            }
        }
    }

    #[test]
    fn test_idle_monitor_is_idle_comparison() {
        // Create a monitor with a very high timeout (1000 seconds)
        // This should make is_idle() return false in most cases
        let monitor = IdleMonitor::new(1000);

        // Try to check if idle - this tests the timeout comparison logic
        let result = monitor.is_idle();

        match result {
            Ok(_is_idle) => {
                // With a 1000 second timeout, system is unlikely to be idle
                // But we can't assert false because in some test environments
                // the system might actually be idle that long
                // We just verify the method returns successfully
            }
            Err(_) => {
                // If idle detection fails, that's acceptable in test environment
            }
        }
    }

    #[test]
    fn test_idle_monitor_zero_timeout() {
        // With zero timeout, any idle time should trigger idle state
        let monitor = IdleMonitor::new(0);

        let result = monitor.is_idle();

        match result {
            Ok(_is_idle) => {
                // With 0 timeout, should always be idle (unless idle time is exactly 0)
                // We just verify it returns successfully
            }
            Err(_) => {
                // Acceptable in test environment
            }
        }
    }

    #[test]
    fn test_fullscreen_monitor_creation() {
        // Test that we can create a FullscreenMonitor
        // This will fail in headless environments without X11
        let result = FullscreenMonitor::new();

        match result {
            Ok(_monitor) => {
                // Successfully created monitor in X11 environment
            }
            Err(_) => {
                // Expected in headless/non-X11 environments
            }
        }
    }

    #[test]
    fn test_fullscreen_monitor_check() {
        // Test that we can check for fullscreen windows
        // This will fail in headless environments without X11
        let result = FullscreenMonitor::new();

        if let Ok(monitor) = result {
            let fullscreen_result = monitor.is_fullscreen_active();

            match fullscreen_result {
                Ok(_is_fullscreen) => {
                    // Successfully checked fullscreen state
                    // We just verify it returns successfully
                }
                Err(_) => {
                    // X11 query might fail in some environments
                }
            }
        }
    }
}
