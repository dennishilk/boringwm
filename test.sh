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
# Wallpaper (automatic)
# --------------------------------------------------
echo "==> Installing default wallpaper"
if [ -f assets/wallpaper/boringwm-wallpaper.png ]; then
  cp assets/wallpaper/boringwm-wallpaper.png "$HOME/.wallpaper"
else
  echo "!! Wallpaper not found, skipping"
fi

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
# Auto startx on tty1 (boring & optional)
# --------------------------------------------------
echo
echo "==> Enable automatic startx on tty1? (y/N)"
read -r answer

if [ "$answer" = "y" ] || [ "$answer" = "Y" ]; then
  echo "==> Enabling auto-start X on tty1"

  cat >> "$HOME/.bash_profile" << 'EOF'

# Auto-start X on tty1 (BoringWM)
if [ -z "$DISPLAY" ] && [ "$(tty)" = "/dev/tty1" ]; then
  exec startx
fi
EOF

else
  echo "==> Skipping auto-start X"
fi

# --------------------------------------------------
# Done
# --------------------------------------------------
echo
echo "==> Setup complete."
echo "==> Log in on tty1 to start BoringWM automatically"
echo "==> Or start manually with: startx"
