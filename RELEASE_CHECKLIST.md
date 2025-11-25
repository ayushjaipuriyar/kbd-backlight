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
- [ ] Update repository URL in all files:
  - [ ] README.md
  - [ ] Cargo.toml
  - [ ] PKGBUILD
  - [ ] CONTRIBUTING.md
  - [ ] GitHub workflows
- [ ] Update maintainer information in PKGBUILD
- [ ] Verify LICENSE file is correct

### GitHub Configuration
- [ ] Repository created on GitHub
- [ ] Repository description set
- [ ] Topics/tags added (rust, linux, keyboard, backlight, daemon)
- [ ] Default branch set to `main`
- [ ] Branch protection rules configured (optional)

### GitHub Secrets
- [ ] `AUR_USERNAME` - Your AUR username
- [ ] `AUR_EMAIL` - Your AUR email  
- [ ] `AUR_SSH_PRIVATE_KEY` - SSH key for AUR access

### AUR Setup
- [ ] AUR account created
- [ ] SSH key added to AUR account
- [ ] Test SSH connection: `ssh aur@aur.archlinux.org`

## Release Process

### Version 0.1.0

1. **Update Version**
   ```bash
   # Update Cargo.toml
   sed -i 's/version = "0.1.0"/version = "0.1.0"/' Cargo.toml
   
   # Verify
   grep version Cargo.toml
   ```

2. **Update Changelog**
   ```bash
   # Edit CHANGELOG.md
   # Change [Unreleased] to [0.1.0] - 2025-11-25
   # Add release notes
   ```

3. **Commit Changes**
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: prepare release 0.1.0"
   ```

4. **Create Tag**
   ```bash
   git tag -a v0.1.0 -m "Release version 0.1.0"
   ```

5. **Push to GitHub**
   ```bash
   git push origin main
   git push origin v0.1.0
   ```

6. **Verify GitHub Actions**
   - Go to Actions tab
   - Verify CI workflow passes
   - Verify Release workflow runs
   - Check that release is created
   - Download and test release binaries

7. **Verify AUR Package**
   - Check AUR repository updated
   - Test installation: `yay -S kbd-backlight`
   - Verify service starts: `systemctl --user status kbd-backlight-daemon`

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

For subsequent releases (0.2.0, 0.3.0, etc.):

1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Commit: `git commit -m "chore: bump version to X.Y.Z"`
4. Tag: `git tag vX.Y.Z`
5. Push: `git push origin main && git push origin vX.Y.Z`
6. GitHub Actions handles the rest

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
