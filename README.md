# kbd-backlight

[![CI](https://github.com/ayushjaipuriyar/kbd-backlight/workflows/CI/badge.svg)](https://github.com/ayushjaipuriyar/kbd-backlight/actions/workflows/ci.yml)
[![Release](https://github.com/ayushjaipuriyar/kbd-backlight/workflows/Release/badge.svg)](https://github.com/ayushjaipuriyar/kbd-backlight/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/kbd-backlight.svg)](https://crates.io/crates/kbd-backlight)

Intelligent keyboard backlight daemon for Linux laptops with automatic brightness control based on idle time, video playback, power state, WiFi location, and time schedules.

## Features

- 🌙 **Automatic idle detection** - Turns off backlight after configurable inactivity
- 🎬 **Smart video detection** - Automatically dims during video playback (MPRIS support)
- 🔌 **Power-aware** - Optional AC power behavior (always-on mode)
- 📍 **Location-based profiles** - Auto-switch profiles based on WiFi network
- ⏰ **Time schedules** - Set brightness based on time of day
- 🎯 **Manual override** - Temporary manual control when needed
- 🔄 **Multiple profiles** - Home, office, mobile, or custom profiles
- 🖥️ **Wayland & X11** - Works on both display servers

## Installation

### Arch Linux (AUR)

```bash
yay -S kbd-backlight
# or
paru -S kbd-backlight
```

### From Source

**Requirements:**
- Rust 1.70+
- D-Bus development files
- Wayland development files (optional, for Wayland support)
- X11 development files (optional, for X11 support)

**Debian/Ubuntu:**
```bash
sudo apt install build-essential libdbus-1-dev libwayland-dev libx11-dev libxss-dev
```

**Arch Linux:**
```bash
sudo pacman -S base-devel dbus wayland libx11 libxss
```

**Build and install:**
```bash
git clone https://github.com/ayushjaipuriyar/kbd-backlight.git
cd kbd-backlight
cargo build --release
sudo cp target/release/kbd-backlight /usr/local/bin/
sudo cp target/release/kbd-backlight-daemon /usr/local/bin/
sudo cp kbd-backlight-daemon.service /usr/lib/systemd/user/
```

**Enable the service:**
```bash
systemctl --user enable --now kbd-backlight-daemon
```

## Quick Start

### Basic Usage

```bash
# Check status
kbd-backlight status

# List available profiles
kbd-backlight list

# Switch profile
kbd-backlight profile office

# Set manual brightness (0-3, depending on your hardware)
kbd-backlight set 2

# Resume automatic control
kbd-backlight auto

# Add time schedule
kbd-backlight schedule add home 22:00 0
```

### Configuration

Configuration files are stored in `~/.config/kbd-backlight/`:

```
~/.config/kbd-backlight/
├── config.toml          # Global settings
├── state.toml           # Active profile (auto-managed)
└── profiles/            # Profile definitions
    ├── home.toml
    ├── office.toml
    └── mobile.toml
```

**Example profile (`~/.config/kbd-backlight/profiles/home.toml`):**

```toml
name = "home"
idle_timeout = 30
video_detection_enabled = true
ac_always_on = false

wifi_networks = [
    "HomeWiFi",
    "HomeWiFi_5GHz"
]

[[time_schedules]]
hour = 9
minute = 0
brightness = 1

[[time_schedules]]
hour = 22
minute = 0
brightness = 0
```

## Configuration Options

### Global Settings (`config.toml`)

- `auto_switch_location` - Enable automatic profile switching based on WiFi

### Profile Settings

- `name` - Profile identifier (must match filename)
- `idle_timeout` - Seconds of inactivity before turning off backlight
- `video_detection_enabled` - Use MPRIS to detect video playback
- `ac_always_on` - Keep backlight on when connected to AC power
- `wifi_networks` - WiFi SSIDs that trigger this profile
- `time_schedules` - Time-based brightness rules

## Use Cases

### Home Profile
- Comfortable idle timeout (30s)
- Respects time schedules
- Auto-switches when connected to home WiFi

### Office Profile
- Shorter idle timeout (15s)
- AC always-on mode for desktop use
- Auto-switches when connected to office WiFi

### Mobile Profile
- Aggressive battery saving (5s idle timeout)
- Never keeps backlight on unnecessarily
- For coffee shops and travel

## How It Works

The daemon monitors multiple inputs and applies rules in priority order:

1. **Manual override** (highest priority)
2. **Video playback detection** (via MPRIS)
3. **AC always-on setting** (if enabled)
4. **Time schedules**
5. **Idle timeout**

### Idle Detection

- **Wayland**: Uses `ext-idle-notify-v1` protocol
- **X11**: Uses `XScreenSaver` extension
- Monitors keyboard and mouse activity

### Video Detection

- Monitors MPRIS D-Bus interface
- Detects playing media from browsers, media players
- Automatically turns off backlight during playback

### Power State

- Detects AC/Battery state via sysfs
- Optional AC always-on mode per profile
- Respects manual override even on AC

## Troubleshooting

### Backlight not turning off

1. Check idle timeout: `grep idle_timeout ~/.config/kbd-backlight/profiles/*.toml`
2. Verify daemon is running: `systemctl --user status kbd-backlight-daemon`
3. Check logs: `journalctl --user -u kbd-backlight-daemon -f`

### Profile not switching automatically

1. Verify `auto_switch_location = true` in `config.toml`
2. Check WiFi SSID spelling (case-sensitive)
3. Ensure SSID isn't assigned to multiple profiles

### Video detection not working

1. Check if media player supports MPRIS: `busctl --user list | grep mpris`
2. Try disabling: Set `video_detection_enabled = false` in profile

### Permission denied errors

Ensure your user has access to `/sys/class/leds/platform::kbd_backlight/`:

```bash
sudo usermod -aG input $USER
# Log out and back in
```

## Contributing

See [CONTRIBUTING.md](.github/CONTRIBUTING.md) for development setup and guidelines.

This project uses [Conventional Commits](https://www.conventionalcommits.org/) and automated semantic versioning. Commit messages must follow the format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

Examples:
- `feat: add WiFi-based profile switching`
- `fix: resolve file descriptor leak in idle detection`
- `docs: update installation instructions`
- `feat!: change config file format` (breaking change)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Uses [wayrs](https://github.com/MaxVerevkin/wayrs) for Wayland protocol support
- Inspired by various backlight control tools in the Linux ecosystem
