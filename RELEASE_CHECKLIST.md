# Release Checklist

Use this checklist before creating the first release.

## Pre-Release

### Code Quality
- [x] All features implemented and tested
- [x] No compiler warnings
- [x] Tests passing (`cargo test`)
- [x] Code formatted (`cargo fmt`)
- [x] Linter passing (`cargo clippy`)
- [x] No security vulnerabilities (`cargo audit`)

### Documentation
- [x] README.md complete with examples
- [x] CONTRIBUTING.md with development guide
- [x] CHANGELOG.md with version history
- [x] Inline code documentation
- [x] Example configurations provided

### Repository Setup
- [x] Update repository URL in all files:
  - [x] README.md
  - [x] Cargo.toml
  - [x] PKGBUILD
  - [x] CONTRIBUTING.md
  - [x] GitHub workflows
- [x] Update maintainer information in PKGBUILD
- [x] Verify LICENSE file is correct
- [x] Add Dependabot configuration
- [x] Add CI/CD badges to README
- [x] Configure semantic-release for automated versioning
- [x] Add commitlint for conventional commit validation
- [x] Create SEMANTIC_RELEASE.md documentation

### GitHub Configuration
- [x] Repository created on GitHub
- [x] Repository description set
- [x] Topics/tags added (rust, linux, keyboard, backlight, daemon)
- [x] Default branch set to `main`
- [ ] Branch protection rules configured (optional)

### GitHub Secrets
- [x] `AUR_USERNAME` - Your AUR username
- [x] `AUR_EMAIL` - Your AUR email  
- [x] `AUR_SSH_PRIVATE_KEY` - SSH key for AUR access

### AUR Setup
- [ ] AUR account created
- [ ] SSH key added to AUR account
- [ ] Test SSH connection: `ssh aur@aur.archlinux.org`

## Release Process

### Automated Semantic Release (Recommended)

This project uses automated semantic versioning. See [SEMANTIC_RELEASE.md](SEMANTIC_RELEASE.md) for details.

1. **Make Changes with Conventional Commits**
   ```bash
   # Feature (minor bump)
   git commit -m "feat: add new feature"
   
   # Bug fix (patch bump)
   git commit -m "fix: resolve issue"
   
   # Breaking change (major bump)
   git commit -m "feat!: breaking change"
   ```

2. **Push to Main**
   ```bash
   git push origin main
   ```

3. **Automatic Process**
   - Semantic Release analyzes commits
   - Determines next version automatically
   - Updates CHANGELOG.md
   - Updates Cargo.toml and PKGBUILD
   - Creates git tag
   - Creates GitHub release
   - Triggers build workflow
   - Publishes to AUR

4. **Verify Release**
   - Go to Actions tab
   - Verify Semantic Release workflow passes
   - Verify Release workflow runs
   - Check GitHub release created
   - Download and test release binaries
   - Verify AUR package updated

### Manual Release (Emergency Only)

Only use if semantic-release fails:

1. **Update Version**
   ```bash
   sed -i 's/version = "0.1.0"/version = "0.2.0"/' Cargo.toml
   sed -i 's/pkgver=0.1.0/pkgver=0.2.0/' PKGBUILD
   ```

2. **Update Changelog**
   ```bash
   # Manually edit CHANGELOG.md
   ```

3. **Commit and Tag**
   ```bash
   git add Cargo.toml PKGBUILD CHANGELOG.md
   git commit -m "chore(release): 0.2.0 [skip ci]"
   git tag v0.2.0
   git push origin main
   git push origin v0.2.0
   ```

## Post-Release

### Announcement
- [ ] Create GitHub release notes
- [ ] Post on relevant forums/communities:
  - [ ] Reddit r/linux
  - [ ] Reddit r/rust
  - [ ] Hacker News (optional)
  - [ ] Linux subreddit for your distro

### Monitoring
- [ ] Watch for issues
- [ ] Respond to questions
- [ ] Monitor AUR comments
- [ ] Check CI/CD status

### Documentation
- [ ] Update README if needed
- [ ] Add troubleshooting entries based on feedback
- [ ] Update FAQ if questions arise

## Testing Checklist

Before tagging, manually test:

### Basic Functionality
- [ ] Daemon starts without errors
- [ ] CLI commands work:
  - [ ] `kbd-backlight status`
  - [ ] `kbd-backlight list`
  - [ ] `kbd-backlight profile <name>`
  - [ ] `kbd-backlight set <brightness>`
  - [ ] `kbd-backlight auto`

### Features
- [ ] Idle detection works (wait for timeout)
- [ ] Video detection works (play a video)
- [ ] Profile switching works
- [ ] Manual override works
- [ ] Time schedules work (if testable)
- [ ] Configuration validation catches errors

### Edge Cases
- [ ] Daemon handles missing config gracefully
- [ ] Invalid profile names rejected
- [ ] Duplicate WiFi networks detected
- [ ] File descriptor leak fixed (run for extended period)

## Rollback Plan

If issues are discovered after release:

1. **Immediate**
   ```bash
   # Delete tag locally and remotely
   git tag -d v0.1.0
   git push origin :refs/tags/v0.1.0
   
   # Delete GitHub release
   # Go to Releases → Delete release
   ```

2. **Fix Issues**
   ```bash
   # Fix the issues
   git commit -m "fix: critical issue"
   ```

3. **Re-release**
   ```bash
   # Create new tag
   git tag -a v0.1.1 -m "Release version 0.1.1"
   git push origin main
   git push origin v0.1.1
   ```

## Future Releases

With semantic-release, future releases are automatic:

1. Make changes with conventional commits
2. Push to main
3. Semantic Release handles everything

**Commit Types:**
- `feat:` → Minor version bump (0.1.0 → 0.2.0)
- `fix:` → Patch version bump (0.1.0 → 0.1.1)
- `feat!:` or `BREAKING CHANGE:` → Major version bump (0.1.0 → 1.0.0)
- `docs:`, `chore:`, `style:`, etc. → No release

See [SEMANTIC_RELEASE.md](SEMANTIC_RELEASE.md) for complete guide.

## Notes

- Always test locally before pushing tags
- Keep CHANGELOG.md updated with each change
- Follow semantic versioning strictly
- Respond to issues promptly
- Be patient with first-time contributors

## Success Criteria

Release is successful when:
- [x] Code compiles without warnings
- [x] All tests pass
- [x] Documentation is complete
- [ ] GitHub release created automatically
- [ ] Binaries available for download
- [ ] AUR package updated
- [ ] No critical bugs reported in first 24 hours
