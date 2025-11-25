use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Permission denied accessing brightness control at {path}.\n\nTo fix this issue, you can:\n1. Add a udev rule: Create /etc/udev/rules.d/90-kbd-backlight.rules with:\n   SUBSYSTEM==\"leds\", ACTION==\"add\", KERNEL==\"*kbd_backlight\", RUN+=\"/bin/chmod g+w /sys/class/leds/%k/brightness\", GROUP=\"{user}\"\n2. Add your user to the 'input' group: sudo usermod -a -G input $USER\n3. Reload udev rules: sudo udevadm control --reload-rules && sudo udevadm trigger\n4. Log out and back in for group changes to take effect")]
    PermissionDenied { path: PathBuf, user: String },

    #[error("Brightness control path not found: {path}\n\nThis usually means:\n1. Your hardware doesn't have a keyboard backlight\n2. The kernel module for keyboard backlight is not loaded\n3. The sysfs path is different on your system\n\nTry checking: ls -la /sys/class/leds/")]
    PathNotFound { path: PathBuf },

    #[error("Invalid brightness value: {0}")]
    InvalidBrightness(String),

    #[error("Configuration error at {location}: {message}")]
    Config { location: String, message: String },

    #[error("Configuration file syntax error: {0}")]
    ConfigSyntax(String),

    #[error("Configuration validation failed: {0}")]
    ConfigValidation(String),

    #[error("IPC communication error: {0}\n\nPossible causes:\n1. Daemon is not running (try: kbd-backlight daemon start)\n2. Socket file is stale (try: rm /tmp/kbd-backlight-daemon.sock)\n3. Permission issue with socket file")]
    IpcConnection(String),

    #[error("IPC protocol error: {0}")]
    IpcProtocol(String),

    #[error("IPC socket error: {0}")]
    IpcSocket(String),

    #[error("X11 connection error: {0}\n\nNote: X11 features will be disabled. The daemon will continue with reduced functionality.")]
    X11Connection(String),

    #[error("X11 protocol error: {0}")]
    X11Protocol(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Monitor unavailable: {monitor_type}\nReason: {reason}\nImpact: {impact}")]
    MonitorUnavailable {
        monitor_type: String,
        reason: String,
        impact: String,
    },
}

impl Error {
    /// Create a permission denied error with helpful context
    pub fn permission_denied(path: PathBuf) -> Self {
        let user = std::env::var("USER").unwrap_or_else(|_| "your_username".to_string());
        Error::PermissionDenied { path, user }
    }

    /// Create a configuration error with location context
    pub fn config_error(location: impl Into<String>, message: impl Into<String>) -> Self {
        Error::Config {
            location: location.into(),
            message: message.into(),
        }
    }

    /// Create an IPC connection error
    pub fn ipc_connection(message: impl Into<String>) -> Self {
        Error::IpcConnection(message.into())
    }

    /// Create an IPC protocol error
    pub fn ipc_protocol(message: impl Into<String>) -> Self {
        Error::IpcProtocol(message.into())
    }

    /// Create an X11 connection error (non-fatal)
    pub fn x11_connection(message: impl Into<String>) -> Self {
        Error::X11Connection(message.into())
    }

    /// Create a monitor unavailable error
    pub fn monitor_unavailable(
        monitor_type: impl Into<String>,
        reason: impl Into<String>,
        impact: impl Into<String>,
    ) -> Self {
        Error::MonitorUnavailable {
            monitor_type: monitor_type.into(),
            reason: reason.into(),
            impact: impact.into(),
        }
    }

    /// Check if this error is recoverable (daemon can continue with degraded functionality)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Error::X11Connection(_) | Error::X11Protocol(_) | Error::MonitorUnavailable { .. }
        )
    }

    /// Check if this error is a configuration error
    pub fn is_config_error(&self) -> bool {
        matches!(
            self,
            Error::Config { .. } | Error::ConfigSyntax(_) | Error::ConfigValidation(_)
        )
    }

    /// Check if this error is an IPC error
    pub fn is_ipc_error(&self) -> bool {
        matches!(
            self,
            Error::IpcConnection(_) | Error::IpcProtocol(_) | Error::IpcSocket(_)
        )
    }
}

pub type Result<T> = std::result::Result<T, Error>;
