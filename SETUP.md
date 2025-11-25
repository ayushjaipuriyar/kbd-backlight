# Repository Setup Guide

This document explains the repository structure and setup for maintainers.

## Repository Structure

```
kbd-backlight/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml              # Continuous integration
│   │   ├── release.yml         # Automated releases
│   │   └── aur-publish.yml     # AUR package publishing
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── feature_request.md
│   └── pull_request_template.md
├── src/                        # Source code
├── profiles-example/           # Example configurations
├── Cargo.toml                  # Rust package manifest
├── PKGBUILD                    # Arch Linux package
├── README.md                   # User documentation
├── CONTRIBUTING.md             # Contributor guide
├── CHANGELOG.md                # Version history
└── LICENSE                     # MIT license
```

## GitHub Secrets Required

For automated workflows to work, configure these secrets in GitHub repository settings:

### For AUR Publishing

1. **AUR_USERNAME**: Your AUR username
2. **AUR_EMAIL**: Your AUR email
3. **AUR_SSH_PRIVATE_KEY**: SSH private key for AUR access

#### Generating AUR SSH Key

```bash
# Generate SSH key for AUR
ssh-keygen -t ed25519 -C "your.email@example.com" -f ~/.ssh/aur

# Add public key to AUR account
cat ~/.ssh/aur.pub
# Go to https://aur.archlinux.org/account/ and add the public key

# Add private key to GitHub secrets
cat ~/.ssh/aur
# Copy the entire private key including BEGIN and END lines
```

## Release Process

### Automated Release (Recommended)

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with changes
3. Commit changes:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: bump version to 0.2.0"
   ```
4. Create and push tag:
   ```bash
   git tag v0.2.0
   git push origin main
   git push origin v0.2.0
   ```
5. GitHub Actions will automatically:
   - Build binaries for x86_64 and aarch64
   - Create GitHub release
   - Upload release assets
   - Publish to AUR

### Manual AUR Publishing

If you need to publish to AUR manually:

1. Go to GitHub Actions
2. Select "Publish to AUR" workflow
3. Click "Run workflow"
4. Enter the version number (e.g., 0.2.0)
5. Click "Run workflow"

## Maintaining AUR Package

### Initial AUR Setup

```bash
# Clone AUR repository
git clone ssh://aur@aur.archlinux.org/kbd-backlight.git aur-kbd-backlight
cd aur-kbd-backlight

# Copy PKGBUILD
cp ../PKGBUILD .

# Update .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Commit and push
git add PKGBUILD .SRCINFO
git commit -m "Initial commit"
git push
```

### Updating AUR Package

The GitHub workflow handles this automatically, but for manual updates:

```bash
cd aur-kbd-backlight

# Update PKGBUILD version and sha256sums
vim PKGBUILD

# Update .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Commit and push
git add PKGBUILD .SRCINFO
git commit -m "Update to version X.Y.Z"
git push
```

## CI/CD Workflows

### CI Workflow (ci.yml)

Runs on every push and pull request:
- Runs tests
- Checks code formatting (rustfmt)
- Runs linter (clippy)
- Builds on stable and beta Rust

### Release Workflow (release.yml)

Triggers on version tags (v*):
- Builds for x86_64 and aarch64
- Creates GitHub release
- Uploads binary tarballs
- Triggers AUR publish

### AUR Publish Workflow (aur-publish.yml)

Can be triggered manually or by release workflow:
- Updates PKGBUILD with new version
- Calculates sha256sum
- Pushes to AUR repository

## Development Workflow

### Creating a New Feature

1. Create feature branch:
   ```bash
   git checkout -b feat/new-feature
   ```

2. Make changes and commit using conventional commits:
   ```bash
   git commit -m "feat(scope): add new feature"
   ```

3. Push and create PR:
   ```bash
   git push origin feat/new-feature
   ```

4. CI will automatically run tests

### Fixing a Bug

1. Create bugfix branch:
   ```bash
   git checkout -b fix/bug-description
   ```

2. Fix and commit:
   ```bash
   git commit -m "fix(scope): fix bug description"
   ```

3. Push and create PR

## Conventional Commits

We use [Conventional Commits](https://www.conventionalcommits.org/) for clear history:

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `perf`: Performance
- `test`: Tests
- `chore`: Maintenance
- `ci`: CI/CD

## Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

## Testing Locally

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Build release
cargo build --release

# Install locally
sudo cp target/release/kbd-backlight* /usr/local/bin/
```

## Troubleshooting

### CI Fails on Formatting

```bash
cargo fmt
git add -u
git commit --amend --no-edit
git push --force-with-lease
```

### CI Fails on Clippy

```bash
cargo clippy --fix --allow-dirty
git add -u
git commit -m "fix: address clippy warnings"
```

### Release Workflow Fails

Check:
1. Tag format is correct (v1.2.3)
2. Version in Cargo.toml matches tag
3. All tests pass locally

### AUR Publish Fails

Check:
1. GitHub secrets are configured
2. SSH key has access to AUR
3. PKGBUILD is valid (`makepkg --printsrcinfo`)

## Support

For questions or issues:
- Open an issue on GitHub
- Check existing issues and discussions
- Read CONTRIBUTING.md for development guidelines
