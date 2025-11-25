// Core library modules
pub mod brightness;
pub mod config;
pub mod monitors;
pub mod rules;
pub mod ipc;
pub mod error;
pub mod wayland_idle;
pub mod location;
pub mod power;
pub mod video_detector;

pub use error::{Error, Result};
