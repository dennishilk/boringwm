#!/bin/sh
set -e

echo "==> BoringWM testing setup (Debian 13 / minimal)"

# --------------------------------------------------
# Base system
# --------------------------------------------------
echo "==> Installing base system packages"
sudo apt update
sudo apt install -y \
  xorg \
  xinit \
  dbus-x11 \
  git \
  curl \
  build-essential

# --------------------------------------------------
# Rust (user-local)
# --------------------------------------------------
if ! command -v cargo >/dev/null 2>&1; then
  echo "==> Installing Rust (user-local)"
  curl https://sh.rustup.rs -sSf | sh -s -- -y
fi

# shellcheck source=/dev/null
. "$HOME/.cargo/env"

# --------------------------------------------------
# Build and install BoringWM
# --------------------------------------------------
echo "==> Building BoringWM"
if [ ! -d boringwm ]; then
  git clone https://github.com/dennishilk/boringwm.git
fi

cd boringwm
cargo build --release

echo "==> Installing BoringWM to /usr/local/bin"
sudo install -Dm755 target/release/boringwm /usr/local/bin/boringwm

# --------------------------------------------------
# Recommended desktop tools
# --------------------------------------------------
echo "==> Installing recommended desktop tools"
sudo apt install -y kitty picom feh

# --------------------------------------------------
# Autostart configuration
# --------------------------------------------------
echo "==> Setting up autostart"
mkdir -p "$HOME/.config/boringwm"

cat > "$HOME/.config/boringwm/autostart.sh" << 'EOF'
#!/bin/sh
feh --bg-fill "$HOME/.wallpaper" &
picom &
EOF

chmod +x "$HOME/.config/boringwm/autostart.sh"

# --------------------------------------------------
# X11 start configuration
# --------------------------------------------------
echo "==> Writing ~/.xinitrc"
cat > "$HOME/.xinitrc" << 'EOF'
exec boringwm
EOF

# --------------------------------------------------
# Done
# --------------------------------------------------
echo
echo "==> Setup complete."
echo "==> Place a wallpaper at: ~/.wallpaper"
echo "==> Start BoringWM with: startx"
