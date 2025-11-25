# Contributing to kbd-backlight

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.70 or later
- D-Bus development files
- Wayland development files (optional)
- X11 development files (optional)

### Building from Source

```bash
git clone https://github.com/yourusername/kbd-backlight.git
cd kbd-backlight
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Running the Daemon Locally

```bash
# Stop system daemon if running
systemctl --user stop kbd-backlight-daemon

# Run from source
cargo run --bin kbd-backlight-daemon

# In another terminal, test CLI
cargo run --bin kbd-backlight -- status
```

## Code Style

We follow standard Rust conventions:

- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Write documentation for public APIs
- Add tests for new features

## Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `ci`: CI/CD changes

### Examples

```
feat(config): add support for multiple WiFi networks per profile

- Profiles can now specify multiple WiFi SSIDs
- Auto-switching works with any matching network
- Added validation to prevent duplicate SSIDs

Closes #42
```

```
fix(idle): prevent file descriptor leak in Wayland idle detection

The IdleMonitor was being recreated every second, spawning new
threads and opening new Wayland connections. Now only recreates
when timeout changes.

Fixes #38
```

## Pull Request Process

1. **Fork the repository** and create a feature branch
2. **Make your changes** following the code style guidelines
3. **Add tests** for new functionality
4. **Update documentation** if needed
5. **Run tests** and ensure they pass
6. **Commit** using conventional commit format
7. **Push** to your fork and create a pull request

### PR Checklist

- [ ] Code follows project style guidelines
- [ ] Tests added/updated and passing
- [ ] Documentation updated
- [ ] Commit messages follow conventional commits
- [ ] No merge conflicts
- [ ] PR description explains the changes

## Project Structure

```
kbd-backlight/
├── src/
│   ├── brightness.rs      # Brightness control via sysfs
│   ├── config.rs          # Configuration management
│   ├── error.rs           # Error types
│   ├── ipc.rs             # IPC client/server
│   ├── location.rs        # WiFi location detection
│   ├── monitors.rs        # Idle and fullscreen detection
│   ├── power.rs           # Power state detection
│   ├── rules.rs           # Rule engine
│   ├── video_detector.rs  # MPRIS video detection
│   ├── wayland_idle.rs    # Wayland idle detection
│   ├── lib.rs             # Library entry point
│   ├── cli/
│   │   └── main.rs        # CLI application
│   └── daemon/
│       └── main.rs        # Daemon application
├── profiles-example/      # Example profile configurations
├── tests/                 # Integration tests
└── docs/                  # Additional documentation
```

## Adding New Features

### 1. Configuration Options

If adding a new configuration option:

1. Update `src/config.rs` with the new field
2. Add validation in `Config::validate()`
3. Update example profiles in `profiles-example/`
4. Document in README.md

### 2. Detection Methods

If adding a new detection method (e.g., Bluetooth, ambient light):

1. Create a new module in `src/`
2. Implement the detector with error handling
3. Integrate into `src/daemon/main.rs`
4. Add tests
5. Document the feature

### 3. CLI Commands

If adding a new CLI command:

1. Update `src/cli/main.rs` with the new command
2. Add corresponding IPC message in `src/ipc.rs`
3. Handle the message in daemon's `handle_ipc_message()`
4. Update README.md with usage examples

## Testing Guidelines

### Unit Tests

- Test individual functions and modules
- Mock external dependencies when possible
- Use `#[cfg(test)]` modules

### Integration Tests

- Test end-to-end functionality
- Use temporary directories for config files
- Clean up resources after tests

### Manual Testing

Before submitting a PR, manually test:

1. Profile switching
2. Manual brightness override
3. Idle detection
4. Video detection (if applicable)
5. Configuration validation

## Documentation

### Code Documentation

- Document all public APIs with `///` comments
- Include examples in doc comments
- Explain complex algorithms

### User Documentation

- Update README.md for user-facing changes
- Add examples for new features
- Update troubleshooting section if needed

## Release Process

Releases are automated via GitHub Actions:

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Create a git tag: `git tag v1.2.3`
4. Push tag: `git push origin v1.2.3`
5. GitHub Actions will build and create release

## Getting Help

- Open an issue for bugs or feature requests
- Join discussions for questions
- Check existing issues before creating new ones

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on the code, not the person
- Help others learn and grow

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
