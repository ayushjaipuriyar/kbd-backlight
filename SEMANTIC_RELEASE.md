# Semantic Release Guide

This project uses automated semantic versioning and release management based on [Conventional Commits](https://www.conventionalcommits.org/).

## How It Works

1. **Commit with conventional format** - All commits must follow the conventional commit format
2. **Push to main** - When commits are pushed to the `main` branch
3. **Automated analysis** - Semantic Release analyzes commit messages
4. **Version bump** - Automatically determines the next version number
5. **Changelog generation** - Updates CHANGELOG.md with release notes
6. **File updates** - Updates version in Cargo.toml and PKGBUILD
7. **Git tag creation** - Creates a new git tag (e.g., v0.2.0)
8. **GitHub release** - Creates a GitHub release with notes
9. **Build & publish** - Triggers the release workflow to build binaries and publish to AUR

## Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type

Determines the version bump:

- `feat`: New feature → **MINOR** version bump (0.1.0 → 0.2.0)
- `fix`: Bug fix → **PATCH** version bump (0.1.0 → 0.1.1)
- `perf`: Performance improvement → **PATCH** version bump
- `refactor`: Code refactoring → **PATCH** version bump
- `revert`: Revert previous commit → **PATCH** version bump
- `docs`: Documentation only → No release
- `style`: Code style changes → No release
- `test`: Test changes → No release
- `build`: Build system changes → No release
- `ci`: CI configuration changes → No release
- `chore`: Other changes → No release

### Breaking Changes

Add `!` after type or `BREAKING CHANGE:` in footer → **MAJOR** version bump (0.1.0 → 1.0.0)

```
feat!: change configuration file format

BREAKING CHANGE: Config files now use YAML instead of TOML
```

### Scope (Optional)

The scope specifies what part of the codebase is affected:

- `daemon`: Daemon-related changes
- `cli`: CLI-related changes
- `config`: Configuration handling
- `idle`: Idle detection
- `video`: Video detection
- `power`: Power management
- `profiles`: Profile management
- `wayland`: Wayland support
- `x11`: X11 support

### Examples

#### Feature Addition (Minor Bump)
```
feat(profiles): add support for Bluetooth-based profile switching

Profiles can now automatically switch based on connected Bluetooth devices
in addition to WiFi networks.
```

#### Bug Fix (Patch Bump)
```
fix(daemon): resolve file descriptor leak in idle detection

The Wayland idle monitor was not properly cleaning up file descriptors,
causing the daemon to eventually run out of available FDs.

Fixes #42
```

#### Breaking Change (Major Bump)
```
feat(config)!: migrate configuration format from TOML to YAML

BREAKING CHANGE: Configuration files must be converted from TOML to YAML format.
Use the provided migration tool: `kbd-backlight migrate-config`
```

#### Documentation (No Release)
```
docs: add troubleshooting section for Wayland issues
```

#### Chore (No Release)
```
chore: update dependencies
```

## Release Process

### Automatic (Recommended)

1. Make changes and commit with conventional format:
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

2. Push to main:
   ```bash
   git push origin main
   ```

3. Semantic Release automatically:
   - Analyzes commits since last release
   - Determines next version
   - Updates CHANGELOG.md
   - Updates Cargo.toml and PKGBUILD
   - Creates git tag
   - Creates GitHub release
   - Triggers build and AUR publish

### Manual Override (Emergency)

If you need to manually create a release:

```bash
# Update version manually
sed -i 's/version = "0.1.0"/version = "0.2.0"/' Cargo.toml
sed -i 's/pkgver=0.1.0/pkgver=0.2.0/' PKGBUILD

# Update CHANGELOG.md manually

# Commit and tag
git add Cargo.toml PKGBUILD CHANGELOG.md
git commit -m "chore(release): 0.2.0 [skip ci]"
git tag v0.2.0
git push origin main
git push origin v0.2.0
```

Note: Use `[skip ci]` to prevent semantic-release from running again.

## Version Numbering

Following [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.1.0): New features (backward compatible)
- **PATCH** (0.0.1): Bug fixes (backward compatible)

### Pre-1.0.0 Releases

Before reaching 1.0.0, the API is considered unstable:
- Breaking changes → MINOR bump (0.1.0 → 0.2.0)
- New features → MINOR bump (0.1.0 → 0.2.0)
- Bug fixes → PATCH bump (0.1.0 → 0.1.1)

### Post-1.0.0 Releases

After 1.0.0, strict semantic versioning applies:
- Breaking changes → MAJOR bump (1.0.0 → 2.0.0)
- New features → MINOR bump (1.0.0 → 1.1.0)
- Bug fixes → PATCH bump (1.0.0 → 1.0.1)

## Changelog

The CHANGELOG.md is automatically generated and organized by:

- **Features**: New functionality
- **Bug Fixes**: Bug fixes
- **Performance Improvements**: Performance enhancements
- **Code Refactoring**: Code improvements
- **Documentation**: Documentation updates
- **Reverts**: Reverted changes

## Commit Validation

Pull requests are automatically validated to ensure:
- PR title follows conventional commit format
- All commits follow conventional commit format
- Commit messages are properly formatted

If validation fails, the PR cannot be merged until fixed.

## Best Practices

1. **Write clear commit messages**: Explain what and why, not how
2. **One logical change per commit**: Makes it easier to review and revert
3. **Use imperative mood**: "add feature" not "added feature"
4. **Reference issues**: Include "Fixes #123" in commit body
5. **Keep subject line short**: Under 72 characters
6. **Use body for details**: Explain context and reasoning
7. **Mark breaking changes clearly**: Use `!` or `BREAKING CHANGE:`

## Troubleshooting

### No Release Created

Check if:
- Commits follow conventional format
- Commits include release-worthy types (feat, fix, perf, refactor)
- You're pushing to the `main` branch
- GitHub Actions has proper permissions

### Wrong Version Bump

Verify:
- Commit types are correct
- Breaking changes are marked with `!` or `BREAKING CHANGE:`
- No `[skip ci]` in commit messages

### Changelog Not Updated

Ensure:
- CHANGELOG.md exists in repository
- Semantic Release has write permissions
- No merge conflicts in CHANGELOG.md

## Tools

### Commitizen (Optional)

Install commitizen for interactive commit message creation:

```bash
npm install -g commitizen cz-conventional-changelog
echo '{ "path": "cz-conventional-changelog" }' > ~/.czrc
```

Use `git cz` instead of `git commit` for guided commit creation.

### Commitlint (Integrated)

Commitlint is already configured and runs on:
- Pull requests (validates all commits)
- Pre-commit hook (optional, can be added locally)

## References

- [Conventional Commits](https://www.conventionalcommits.org/)
- [Semantic Versioning](https://semver.org/)
- [Semantic Release](https://semantic-release.gitbook.io/)
- [Commitlint](https://commitlint.js.org/)
