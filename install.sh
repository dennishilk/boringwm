#!/bin/sh
set -eu
cat <<'MSG'
BoringWM's installer is deliberately non-interactive.
Build first with: cargo build --release
Install with:    sudo make install PREFIX=/usr/local
See README.md for Debian dependencies, startx, display-manager, config, and external-tool setup.
MSG
