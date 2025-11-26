// IPC protocol module
// This module handles communication between CLI and daemon

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

/// Default socket path for IPC communication
pub const DEFAULT_SOCKET_PATH: &str = "/tmp/kbd-backlight-daemon.sock";

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessage {
    GetStatus,
    SetProfile(String),
    SetManualBrightness(u32),
    ClearManualOverride,
    ListProfiles,
    AddTimeSchedule {
        profile: String,
        hour: u8,
        minute: u8,
        brightness: u32,
    },
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcResponse {
    Status(StatusInfo),
    ProfileChanged,
    BrightnessSet,
    ProfileList(Vec<String>),
    ScheduleAdded,
    Error(String),
    Ok,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusInfo {
    pub active_profile: String,
    pub current_brightness: u32,
    pub is_idle: bool,
    pub is_fullscreen: bool,
    pub manual_override: Option<u32>,
}

impl IpcMessage {
    /// Serialize the message to JSON bytes
    pub fn serialize(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| Error::ipc_protocol(format!("Failed to serialize message: {}", e)))
    }

    /// Deserialize a message from JSON bytes
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        serde_json::from_slice(data).map_err(|e| {
            let preview = if data.len() > 100 {
                format!(
                    "{}... ({} bytes)",
                    String::from_utf8_lossy(&data[..100]),
                    data.len()
                )
            } else {
                String::from_utf8_lossy(data).to_string()
            };
            Error::ipc_protocol(format!(
                "Failed to deserialize message: {}. Data: {}",
                e, preview
            ))
        })
    }

    /// Send this message over a Unix stream
    pub async fn send(&self, stream: &mut UnixStream) -> Result<()> {
        let data = self.serialize()?;
        let len = data.len() as u32;

        // Write length prefix (4 bytes)
        stream
            .write_all(&len.to_be_bytes())
            .await
            .map_err(|e| Error::ipc_protocol(format!("Failed to write message length: {}", e)))?;

        // Write message data
        stream
            .write_all(&data)
            .await
            .map_err(|e| Error::ipc_protocol(format!("Failed to write message data: {}", e)))?;

        stream
            .flush()
            .await
            .map_err(|e| Error::ipc_protocol(format!("Failed to flush stream: {}", e)))?;

        Ok(())
    }

    /// Receive a message from a Unix stream
    pub async fn receive(stream: &mut UnixStream) -> Result<Self> {
        // Read length prefix (4 bytes)
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                Error::ipc_protocol(
                    "Connection closed by peer while reading message length".to_string(),
                )
            } else {
                Error::ipc_protocol(format!("Failed to read message length: {}", e))
            }
        })?;

        let len = u32::from_be_bytes(len_bytes) as usize;

        // Sanity check: reject messages larger than 1MB
        if len > 1_000_000 {
            return Err(Error::ipc_protocol(format!(
                "Message too large: {} bytes (max: 1MB). Possible protocol mismatch.",
                len
            )));
        }

        if len == 0 {
            return Err(Error::ipc_protocol(
                "Received zero-length message".to_string(),
            ));
        }

        // Read message data
        let mut data = vec![0u8; len];
        stream.read_exact(&mut data).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                Error::ipc_protocol(format!(
                    "Connection closed while reading message data (expected {} bytes)",
                    len
                ))
            } else {
                Error::ipc_protocol(format!("Failed to read message data: {}", e))
            }
        })?;

        Self::deserialize(&data)
    }
}

impl IpcResponse {
    /// Serialize the response to JSON bytes
    pub fn serialize(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| Error::ipc_protocol(format!("Failed to serialize response: {}", e)))
    }

    /// Deserialize a response from JSON bytes
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        serde_json::from_slice(data).map_err(|e| {
            let preview = if data.len() > 100 {
                format!(
                    "{}... ({} bytes)",
                    String::from_utf8_lossy(&data[..100]),
                    data.len()
                )
            } else {
                String::from_utf8_lossy(data).to_string()
            };
            Error::ipc_protocol(format!(
                "Failed to deserialize response: {}. Data: {}",
                e, preview
            ))
        })
    }

    /// Send this response over a Unix stream
    pub async fn send(&self, stream: &mut UnixStream) -> Result<()> {
        let data = self.serialize()?;
        let len = data.len() as u32;

        // Write length prefix (4 bytes)
        stream
            .write_all(&len.to_be_bytes())
            .await
            .map_err(|e| Error::ipc_protocol(format!("Failed to write response length: {}", e)))?;

        // Write response data
        stream
            .write_all(&data)
            .await
            .map_err(|e| Error::ipc_protocol(format!("Failed to write response data: {}", e)))?;

        stream
            .flush()
            .await
            .map_err(|e| Error::ipc_protocol(format!("Failed to flush stream: {}", e)))?;

        Ok(())
    }

    /// Receive a response from a Unix stream
    pub async fn receive(stream: &mut UnixStream) -> Result<Self> {
        // Read length prefix (4 bytes)
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                Error::ipc_protocol(
                    "Connection closed by daemon while reading response length".to_string(),
                )
            } else {
                Error::ipc_protocol(format!("Failed to read response length: {}", e))
            }
        })?;

        let len = u32::from_be_bytes(len_bytes) as usize;

        // Sanity check: reject responses larger than 1MB
        if len > 1_000_000 {
            return Err(Error::ipc_protocol(format!(
                "Response too large: {} bytes (max: 1MB). Possible protocol mismatch.",
                len
            )));
        }

        if len == 0 {
            return Err(Error::ipc_protocol(
                "Received zero-length response".to_string(),
            ));
        }

        // Read response data
        let mut data = vec![0u8; len];
        stream.read_exact(&mut data).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                Error::ipc_protocol(format!(
                    "Connection closed while reading response data (expected {} bytes)",
                    len
                ))
            } else {
                Error::ipc_protocol(format!("Failed to read response data: {}", e))
            }
        })?;

        Self::deserialize(&data)
    }
}

/// IPC Server for handling daemon-side communication
pub struct IpcServer {
    listener: UnixListener,
    socket_path: PathBuf,
}

impl IpcServer {
    /// Create a new IPC server at the specified socket path
    pub async fn new<P: AsRef<Path>>(socket_path: P) -> Result<Self> {
        let socket_path = socket_path.as_ref().to_path_buf();

        // Remove existing socket file if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)
                .map_err(|e| {
                    Error::IpcSocket(format!(
                        "Failed to remove existing socket at {:?}: {}. Try manually removing it with: rm {:?}",
                        socket_path, e, socket_path
                    ))
                })?;
        }

        // Ensure parent directory exists
        if let Some(parent) = socket_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    Error::IpcSocket(format!(
                        "Failed to create socket directory {:?}: {}",
                        parent, e
                    ))
                })?;
            }
        }

        // Create Unix domain socket listener
        let listener = UnixListener::bind(&socket_path)
            .map_err(|e| {
                Error::IpcSocket(format!(
                    "Failed to bind socket at {:?}: {}. Check permissions and ensure no other daemon is running.",
                    socket_path, e
                ))
            })?;

        Ok(Self {
            listener,
            socket_path,
        })
    }

    /// Accept a new connection and return the stream
    pub async fn accept(&self) -> Result<UnixStream> {
        let (stream, _addr) =
            self.listener.accept().await.map_err(|e| {
                Error::ipc_connection(format!("Failed to accept connection: {}", e))
            })?;

        Ok(stream)
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        // Clean up socket file on drop
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

/// IPC Client for CLI-side communication
pub struct IpcClient {
    socket_path: PathBuf,
}

impl IpcClient {
    /// Create a new IPC client that will connect to the specified socket path
    pub fn new<P: AsRef<Path>>(socket_path: P) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
        }
    }

    /// Connect to the daemon and send a message, returning the response
    pub async fn send_message(&self, message: &IpcMessage) -> Result<IpcResponse> {
        // Connect to the Unix domain socket
        let mut stream = UnixStream::connect(&self.socket_path).await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Error::ipc_connection(format!(
                        "Socket not found at {:?}. The daemon is not running. Start it with: kbd-backlight daemon start",
                        self.socket_path
                    ))
                } else if e.kind() == std::io::ErrorKind::ConnectionRefused {
                    Error::ipc_connection(format!(
                        "Connection refused at {:?}. The socket file exists but daemon is not responding. Try: rm {:?} && kbd-backlight daemon start",
                        self.socket_path, self.socket_path
                    ))
                } else {
                    Error::ipc_connection(format!(
                        "Failed to connect to daemon at {:?}: {}",
                        self.socket_path, e
                    ))
                }
            })?;

        // Send the message
        message.send(&mut stream).await?;

        // Receive the response
        IpcResponse::receive(&mut stream).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_message_serialization() {
        let messages = vec![
            IpcMessage::GetStatus,
            IpcMessage::SetProfile("home".to_string()),
            IpcMessage::SetManualBrightness(2),
            IpcMessage::ClearManualOverride,
            IpcMessage::ListProfiles,
            IpcMessage::AddTimeSchedule {
                profile: "office".to_string(),
                hour: 9,
                minute: 30,
                brightness: 3,
            },
            IpcMessage::Shutdown,
        ];

        for msg in messages {
            let serialized = msg.serialize().expect("Failed to serialize");
            let deserialized = IpcMessage::deserialize(&serialized).expect("Failed to deserialize");

            // Verify round-trip works by serializing again and comparing
            let reserialized = deserialized.serialize().expect("Failed to reserialize");
            assert_eq!(serialized, reserialized, "Round-trip serialization failed");
        }
    }

    #[test]
    fn test_ipc_response_serialization() {
        let responses = vec![
            IpcResponse::Status(StatusInfo {
                active_profile: "home".to_string(),
                current_brightness: 2,
                is_idle: false,
                is_fullscreen: false,
                manual_override: None,
            }),
            IpcResponse::ProfileChanged,
            IpcResponse::BrightnessSet,
            IpcResponse::ProfileList(vec!["home".to_string(), "office".to_string()]),
            IpcResponse::ScheduleAdded,
            IpcResponse::Error("Test error".to_string()),
            IpcResponse::Ok,
        ];

        for resp in responses {
            let serialized = resp.serialize().expect("Failed to serialize");
            let deserialized =
                IpcResponse::deserialize(&serialized).expect("Failed to deserialize");

            // Verify round-trip works by serializing again and comparing
            let reserialized = deserialized.serialize().expect("Failed to reserialize");
            assert_eq!(serialized, reserialized, "Round-trip serialization failed");
        }
    }

    #[tokio::test]
    async fn test_ipc_server_creation() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let socket_path = temp_dir.path().join("test.sock");

        let server = IpcServer::new(&socket_path)
            .await
            .expect("Failed to create server");
        assert_eq!(server.socket_path(), socket_path);
        assert!(socket_path.exists(), "Socket file should exist");
    }

    #[tokio::test]
    async fn test_ipc_client_server_communication() {
        use tempfile::TempDir;
        use tokio::time::{timeout, Duration};

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let socket_path = temp_dir.path().join("test.sock");

        let server = IpcServer::new(&socket_path)
            .await
            .expect("Failed to create server");
        let client = IpcClient::new(&socket_path);

        // Spawn server task
        let server_task = tokio::spawn(async move {
            let mut stream = server.accept().await.expect("Failed to accept connection");
            let message = IpcMessage::receive(&mut stream)
                .await
                .expect("Failed to receive message");

            // Echo back a response based on the message
            let response = match message {
                IpcMessage::GetStatus => IpcResponse::Status(StatusInfo {
                    active_profile: "test".to_string(),
                    current_brightness: 1,
                    is_idle: false,
                    is_fullscreen: false,
                    manual_override: None,
                }),
                _ => IpcResponse::Ok,
            };

            response
                .send(&mut stream)
                .await
                .expect("Failed to send response");
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Send message from client
        let message = IpcMessage::GetStatus;
        let response = timeout(Duration::from_secs(1), client.send_message(&message))
            .await
            .expect("Timeout waiting for response")
            .expect("Failed to send message");

        // Verify response
        match response {
            IpcResponse::Status(info) => {
                assert_eq!(info.active_profile, "test");
                assert_eq!(info.current_brightness, 1);
            }
            _ => panic!("Expected Status response"),
        }

        server_task.await.expect("Server task failed");
    }
}
