# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-11-25

### Added
- Initial release
- Automatic idle detection (Wayland and X11 support)
- Smart video detection via MPRIS
- Power state awareness (AC/Battery)
- Location-based profile switching (WiFi)
- Time-based brightness schedules
- Manual brightness override
- Multiple profile support
- IPC-based CLI for daemon control
- Systemd service integration
- Configuration validation
- Example profiles (home, office, mobile)

### Features
- Modular configuration system with separate profile files
- Optional AC always-on mode per profile
- Video detection with automatic backlight dimming
- WiFi-based automatic profile switching
- Time schedules for automatic brightness adjustment
- Manual override with highest priority
- Comprehensive error handling and logging

[Unreleased]: https://github.com/ayushjaipuriyar/kbd-backlight/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ayushjaipuriyar/kbd-backlight/releases/tag/v0.1.0
