// Core library modules
pub mod brightness;
pub mod config;
pub mod error;
pub mod ipc;
pub mod location;
pub mod monitors;
pub mod power;
pub mod rules;
pub mod video_detector;
pub mod wayland_idle;

pub use error::{Error, Result};
