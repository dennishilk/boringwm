# Changelog

This project follows Keep a Changelog and semantic versioning. Historical entries describe repository history and are not claims about current integrated desktop features.

## [Unreleased]

### Added
- Explicit client/workspace state with per-workspace focus and deterministic ordering.
- Pure master/stack geometry with small-screen, gap, border, remainder, count, and ratio tests.
- Static `~/.config/boringwm/config.toml` configuration with strict validation and an example.
- Nine workspace actions, client reordering/promotion, ratio control, floating toggle, restart, and exit.
- Conservative EWMH root/client, desktop, active-window, and fullscreen support.
- CI, transparent Makefile installation, X session assets, manual page, and manual test plan.

### Changed
- Split command spawning, configuration, state, layout, keys, logging, and X11 orchestration.
- Hardened startup adoption, Map/Unmap/Destroy/Configure handling, duplicate suppression, focus fallback, client close, direct process spawning, and shutdown client restoration.
- Updated documentation to describe implemented behavior rather than desktop-environment helpers.

### Known limitations
- XRandR multi-monitor discovery/hotplug and mouse drag/resize are not implemented.
- Keyboard grabs use US X keycodes; fixed-size hints and a full TOML grammar are not supported.
- Real X11 applications and physical multi-monitor hardware require manual validation.

## [0.2.0] - 2026-01-03

Repository documentation and helper scripts expanded the suggested external desktop setup.

## [0.1.0] - 2026-01-01

Initial project and “boring by design” identity.
