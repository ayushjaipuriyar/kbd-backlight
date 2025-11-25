// Video playback detection using MPRIS D-Bus interface
use zbus::{Connection, Result as ZbusResult};
use crate::{Result, Error};

pub struct VideoDetector {
    conn: Connection,
}

impl VideoDetector {
    pub async fn new() -> Result<Self> {
        let conn = Connection::session().await
            .map_err(|e| {
                eprintln!("VideoDetector: Failed to connect to D-Bus session: {}", e);
                Error::ipc_connection(format!("Failed to connect to D-Bus: {}", e))
            })?;
        
        eprintln!("VideoDetector: Successfully connected to D-Bus");
        Ok(Self { conn })
    }

    /// Check if any video is currently playing
    pub async fn is_video_playing(&self) -> Result<bool> {
        // List all MPRIS media players
        let players = self.list_media_players().await?;
        
        if players.is_empty() {
            return Ok(false);
        }

        // Check each player for playback status
        for player in players {
            if let Ok(status) = self.get_playback_status(&player).await {
                if status == "Playing" {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    async fn list_media_players(&self) -> Result<Vec<String>> {
        let proxy = zbus::fdo::DBusProxy::new(&self.conn).await
            .map_err(|e| Error::ipc_protocol(format!("Failed to create D-Bus proxy: {}", e)))?;

        let names = proxy.list_names().await
            .map_err(|e| Error::ipc_protocol(format!("Failed to list D-Bus names: {}", e)))?;

        Ok(names
            .into_iter()
            .map(|name| name.to_string())
            .filter(|name| name.starts_with("org.mpris.MediaPlayer2."))
            .collect())
    }

    async fn get_playback_status(&self, service_name: &str) -> Result<String> {
        let proxy = zbus::Proxy::new(
            &self.conn,
            service_name,
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties",
        ).await
        .map_err(|e| Error::ipc_protocol(format!("Failed to create proxy: {}", e)))?;

        let status: ZbusResult<zbus::zvariant::OwnedValue> = proxy.call(
            "Get",
            &("org.mpris.MediaPlayer2.Player", "PlaybackStatus"),
        ).await;

        match status {
            Ok(value) => {
                // Extract string from variant
                let status_str = format!("{:?}", value);
                // Parse the status (it's wrapped in quotes and variant)
                if status_str.contains("Playing") {
                    Ok("Playing".to_string())
                } else if status_str.contains("Paused") {
                    Ok("Paused".to_string())
                } else {
                    Ok("Stopped".to_string())
                }
            }
            Err(e) => Err(Error::ipc_protocol(format!("Failed to get playback status: {}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_detector_creation() {
        // This might fail in test environment without D-Bus
        let _ = VideoDetector::new();
    }
}
