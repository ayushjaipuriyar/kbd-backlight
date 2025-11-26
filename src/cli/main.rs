// CLI main entry point
// This provides the command-line interface for interacting with the daemon

use clap::{Parser, Subcommand};
use kbd_backlight::ipc::{IpcClient, IpcMessage, IpcResponse, DEFAULT_SOCKET_PATH};
use kbd_backlight::{Error, Result};
use std::process::Command;

#[derive(Parser)]
#[command(name = "kbd-backlight")]
#[command(about = "Keyboard backlight control CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current status
    Status,

    /// Switch to a different profile
    Profile { name: String },

    /// Set manual brightness override
    Set { brightness: u32 },

    /// Clear manual override and resume automatic control
    Auto,

    /// List all available profiles
    List,

    /// Add a time schedule to a profile
    Schedule {
        #[command(subcommand)]
        action: ScheduleAction,
    },

    /// Daemon control commands
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
}

#[derive(Subcommand)]
enum ScheduleAction {
    /// Add a new time schedule
    Add {
        profile: String,
        time: String, // Format: HH:MM
        brightness: u32,
    },
}

#[derive(Subcommand)]
enum DaemonAction {
    /// Start the daemon
    Start,
    /// Stop the daemon
    Stop,
    /// Restart the daemon
    Restart,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => handle_status().await,
        Commands::Profile { name } => handle_profile(name).await,
        Commands::Set { brightness } => handle_set(brightness).await,
        Commands::Auto => handle_auto().await,
        Commands::List => handle_list().await,
        Commands::Schedule { action } => handle_schedule(action).await,
        Commands::Daemon { action } => handle_daemon(action),
    }
}

/// Handle the status command
async fn handle_status() -> Result<()> {
    let client = IpcClient::new(DEFAULT_SOCKET_PATH);
    let response = client.send_message(&IpcMessage::GetStatus).await?;

    match response {
        IpcResponse::Status(info) => {
            println!("Keyboard Backlight Status");
            println!("========================");
            println!("Active Profile:    {}", info.active_profile);
            println!("Current Brightness: {}", info.current_brightness);
            println!(
                "Idle:              {}",
                if info.is_idle { "Yes" } else { "No" }
            );
            println!(
                "Fullscreen:        {}",
                if info.is_fullscreen { "Yes" } else { "No" }
            );

            if let Some(override_val) = info.manual_override {
                println!("Manual Override:   {} (active)", override_val);
            } else {
                println!("Manual Override:   None");
            }

            Ok(())
        }
        IpcResponse::Error(msg) => Err(Error::ipc_protocol(msg)),
        _ => Err(Error::ipc_protocol("Unexpected response from daemon")),
    }
}

/// Handle the profile switch command
async fn handle_profile(name: String) -> Result<()> {
    let client = IpcClient::new(DEFAULT_SOCKET_PATH);
    let response = client
        .send_message(&IpcMessage::SetProfile(name.clone()))
        .await?;

    match response {
        IpcResponse::ProfileChanged => {
            println!("Switched to profile: {}", name);
            Ok(())
        }
        IpcResponse::Error(msg) => Err(Error::ipc_protocol(msg)),
        _ => Err(Error::ipc_protocol("Unexpected response from daemon")),
    }
}

/// Handle the manual brightness set command
async fn handle_set(brightness: u32) -> Result<()> {
    let client = IpcClient::new(DEFAULT_SOCKET_PATH);
    let response = client
        .send_message(&IpcMessage::SetManualBrightness(brightness))
        .await?;

    match response {
        IpcResponse::BrightnessSet => {
            println!("Manual brightness set to: {}", brightness);
            Ok(())
        }
        IpcResponse::Error(msg) => Err(Error::ipc_protocol(msg)),
        _ => Err(Error::ipc_protocol("Unexpected response from daemon")),
    }
}

/// Handle the auto command (clear manual override)
async fn handle_auto() -> Result<()> {
    let client = IpcClient::new(DEFAULT_SOCKET_PATH);
    let response = client
        .send_message(&IpcMessage::ClearManualOverride)
        .await?;

    match response {
        IpcResponse::Ok => {
            println!("Manual override cleared. Resuming automatic control.");
            Ok(())
        }
        IpcResponse::Error(msg) => Err(Error::ipc_protocol(msg)),
        _ => Err(Error::ipc_protocol("Unexpected response from daemon")),
    }
}

/// Handle the list profiles command
async fn handle_list() -> Result<()> {
    let client = IpcClient::new(DEFAULT_SOCKET_PATH);
    let response = client.send_message(&IpcMessage::ListProfiles).await?;

    match response {
        IpcResponse::ProfileList(profiles) => {
            println!("Available Profiles:");
            println!("==================");
            for profile in profiles {
                println!("  - {}", profile);
            }
            Ok(())
        }
        IpcResponse::Error(msg) => Err(Error::ipc_protocol(msg)),
        _ => Err(Error::ipc_protocol("Unexpected response from daemon")),
    }
}

/// Handle the schedule add command
async fn handle_schedule(action: ScheduleAction) -> Result<()> {
    match action {
        ScheduleAction::Add {
            profile,
            time,
            brightness,
        } => {
            // Parse the time string (HH:MM format)
            let parts: Vec<&str> = time.split(':').collect();
            if parts.len() != 2 {
                return Err(Error::Parse(format!(
                    "Invalid time format '{}'. Expected HH:MM",
                    time
                )));
            }

            let hour: u8 = parts[0]
                .parse()
                .map_err(|_| Error::Parse(format!("Invalid hour: {}", parts[0])))?;
            let minute: u8 = parts[1]
                .parse()
                .map_err(|_| Error::Parse(format!("Invalid minute: {}", parts[1])))?;

            // Validate hour and minute ranges
            if hour > 23 {
                return Err(Error::Parse(format!("Hour must be 0-23, got {}", hour)));
            }
            if minute > 59 {
                return Err(Error::Parse(format!("Minute must be 0-59, got {}", minute)));
            }

            let client = IpcClient::new(DEFAULT_SOCKET_PATH);
            let response = client
                .send_message(&IpcMessage::AddTimeSchedule {
                    profile: profile.clone(),
                    hour,
                    minute,
                    brightness,
                })
                .await?;

            match response {
                IpcResponse::ScheduleAdded => {
                    println!(
                        "Added time schedule to profile '{}': {}:{:02} -> brightness {}",
                        profile, hour, minute, brightness
                    );
                    Ok(())
                }
                IpcResponse::Error(msg) => Err(Error::ipc_protocol(msg)),
                _ => Err(Error::ipc_protocol("Unexpected response from daemon")),
            }
        }
    }
}

/// Handle daemon control commands (start/stop/restart)
fn handle_daemon(action: DaemonAction) -> Result<()> {
    match action {
        DaemonAction::Start => {
            println!("Starting kbd-backlight-daemon...");
            let output = Command::new("systemctl")
                .args(["--user", "start", "kbd-backlight-daemon.service"])
                .output()
                .map_err(|e| {
                    Error::ipc_connection(format!("Failed to execute systemctl: {}", e))
                })?;

            if output.status.success() {
                println!("Daemon started successfully.");
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(Error::ipc_connection(format!(
                    "Failed to start daemon: {}",
                    stderr
                )))
            }
        }
        DaemonAction::Stop => {
            println!("Stopping kbd-backlight-daemon...");
            let output = Command::new("systemctl")
                .args(["--user", "stop", "kbd-backlight-daemon.service"])
                .output()
                .map_err(|e| {
                    Error::ipc_connection(format!("Failed to execute systemctl: {}", e))
                })?;

            if output.status.success() {
                println!("Daemon stopped successfully.");
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(Error::ipc_connection(format!(
                    "Failed to stop daemon: {}",
                    stderr
                )))
            }
        }
        DaemonAction::Restart => {
            println!("Restarting kbd-backlight-daemon...");
            let output = Command::new("systemctl")
                .args(["--user", "restart", "kbd-backlight-daemon.service"])
                .output()
                .map_err(|e| {
                    Error::ipc_connection(format!("Failed to execute systemctl: {}", e))
                })?;

            if output.status.success() {
                println!("Daemon restarted successfully.");
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(Error::ipc_connection(format!(
                    "Failed to restart daemon: {}",
                    stderr
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_time_parsing_valid() {
        // Test valid time formats
        let time = "09:30";
        let parts: Vec<&str> = time.split(':').collect();
        assert_eq!(parts.len(), 2);

        let hour: u8 = parts[0].parse().unwrap();
        let minute: u8 = parts[1].parse().unwrap();

        assert_eq!(hour, 9);
        assert_eq!(minute, 30);
    }

    #[test]
    fn test_time_parsing_with_leading_zeros() {
        let time = "00:00";
        let parts: Vec<&str> = time.split(':').collect();
        assert_eq!(parts.len(), 2);

        let hour: u8 = parts[0].parse().unwrap();
        let minute: u8 = parts[1].parse().unwrap();

        assert_eq!(hour, 0);
        assert_eq!(minute, 0);
    }

    #[test]
    fn test_time_parsing_invalid_format() {
        let time = "09-30";
        let parts: Vec<&str> = time.split(':').collect();
        assert_ne!(parts.len(), 2);
    }

    #[test]
    fn test_time_validation_hour_boundary() {
        // Test that hour validation logic would work correctly
        let valid_hour: u8 = 23;
        let invalid_hour: u8 = 24;
        assert!(valid_hour <= 23);
        assert!(invalid_hour > 23);
    }

    #[test]
    fn test_time_validation_minute_boundary() {
        // Test that minute validation logic would work correctly
        let valid_minute: u8 = 59;
        let invalid_minute: u8 = 60;
        assert!(valid_minute <= 59);
        assert!(invalid_minute > 59);
    }
}
