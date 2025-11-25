---
name: Bug report
about: Create a report to help us improve
title: '[BUG] '
labels: bug
assignees: ''
---

**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Run command '...'
2. Check status '...'
3. See error

**Expected behavior**
A clear and concise description of what you expected to happen.

**Actual behavior**
What actually happened.

**Logs**
```
# Paste relevant logs from:
journalctl --user -u kbd-backlight-daemon -n 50
```

**Configuration**
```toml
# Paste your config.toml and relevant profile files
```

**Environment:**
- OS: [e.g., Arch Linux]
- Kernel version: [e.g., 6.6.1]
- Display server: [e.g., Wayland/X11]
- Desktop environment: [e.g., KDE Plasma, GNOME]
- kbd-backlight version: [e.g., 0.1.0]

**Hardware:**
- Laptop model: [e.g., ThinkPad X1 Carbon Gen 11]
- Keyboard backlight path: [e.g., /sys/class/leds/platform::kbd_backlight]

**Additional context**
Add any other context about the problem here.
