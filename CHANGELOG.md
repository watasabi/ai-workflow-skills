# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- `ds-template-workflow` skill: navigate `ds-template-v2` Data Science projects (layout, UV workspaces, Ruff/commit conventions, changelog discipline).
- `nursing-health-reports` skill: clinical/nursing cohort analytics patterns.
- GitHub Actions `release.yml`: builds and publishes tarballs for Linux/macOS (x86_64/arm64) on `vX.Y.Z` tags.
- `docs/index.html` GitHub Pages landing page.
- `LICENSE-MIT` and `LICENSE-APACHE` files.
- Skills installed for the `claude-code` agent (`.claude/skills/`), in addition to Cursor.

### Changed
- `README.md` rewritten in English for a public audience.
- `install.sh` now downloads a prebuilt tarball for Linux and macOS (x86_64/arm64) instead of building from source on macOS.

## [0.3.0] - 2026-05-26

### Added
- Initial public groundwork: catalog-based skill installer, multi-agent support, interactive TUI, lockfile-tracked installs, audit log.
