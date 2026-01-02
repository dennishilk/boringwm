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
# Desktop tools
# --------------------------------------------------
echo "==> Installing desktop tools"
sudo apt install -y \
  kitty \
  picom \
  feh \
  thunar \
  firefox-esr

# --------------------------------------------------
# Wallpaper
# --------------------------------------------------
echo "==> Installing default wallpaper"
if [ -f assets/wallpaper/boringwm-wallpaper.png ]; then
  cp assets/wallpaper/boringwm-wallpaper.png "$HOME/.wallpaper"
else
  echo "!! Wallpaper not found, skipping"
fi

# --------------------------------------------------
# Picom configuration (safe, no GL)
# --------------------------------------------------
echo "==> Writing picom config"
mkdir -p "$HOME/.config/picom"

cat > "$HOME/.config/picom/picom.conf" << 'EOF'
backend = "xrender";
vsync = true;

active-opacity   = 0.95;
inactive-opacity = 0.95;
frame-opacity    = 0.95;

fading = true;
fade-in-step  = 0.03;
fade-out-step = 0.03;

blur-method = "none";
EOF

# --------------------------------------------------
# Kitty configuration (transparency)
# --------------------------------------------------
echo "==> Writing kitty config"
mkdir -p "$HOME/.config/kitty"

cat > "$HOME/.config/kitty/kitty.conf" << 'EOF'
background_opacity 0.90
enable_audio_bell no
EOF

# --------------------------------------------------
# Autostart (safe)
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
# X11 start configuration (loop-safe)
# --------------------------------------------------
echo "==> Writing ~/.xinitrc"
cat > "$HOME/.xinitrc" << 'EOF'
exec boringwm || xterm
EOF

# --------------------------------------------------
# Auto startx on tty1 (OPTIONAL, DEV UNSAFE)
# --------------------------------------------------
echo
echo "==> Enable automatic startx on tty1? (y/N)"
read -r answer

if [ "$answer" = "y" ] || [ "$answer" = "Y" ]; then
  echo "==> Enabling auto-start X on tty1 (DEV MODE WARNING)"

  cat >> "$HOME/.bash_profile" << 'EOF'

# Auto-start X on tty1 (BoringWM)
if [ -z "$DISPLAY" ] && [ "$(tty)" = "/dev/tty1" ]; then
  exec startx
fi
EOF

else
  echo "==> Skipping auto-start X (recommended for dev)"
fi

# --------------------------------------------------
# Done
# --------------------------------------------------
echo
echo "==> Setup complete."
echo "==> Start with: startx"
