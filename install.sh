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
# Zsh + plugins
# --------------------------------------------------
echo "==> Installing zsh and shell enhancements"
sudo apt install -y zsh

mkdir -p "$HOME/.zsh"
if [ ! -d "$HOME/.zsh/zsh-autosuggestions" ]; then
  git clone --depth=1 https://github.com/zsh-users/zsh-autosuggestions.git "$HOME/.zsh/zsh-autosuggestions"
fi

if [ ! -d "$HOME/.zsh/zsh-syntax-highlighting" ]; then
  git clone --depth=1 https://github.com/zsh-users/zsh-syntax-highlighting.git "$HOME/.zsh/zsh-syntax-highlighting"
fi

echo "==> Writing ~/.zshrc"
cat > "$HOME/.zshrc" << 'EOF'
# Plugins
source "$HOME/.zsh/zsh-autosuggestions/zsh-autosuggestions.zsh"
source "$HOME/.zsh/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh"

# Basic usability
autoload -Uz compinit && compinit
setopt HIST_IGNORE_ALL_DUPS
HISTFILE="$HOME/.zsh_history"
HISTSIZE=10000
SAVEHIST=10000
EOF

if command -v chsh >/dev/null 2>&1 && command -v zsh >/dev/null 2>&1; then
  echo "==> Setting zsh as default shell"
  chsh -s "$(command -v zsh)" "$USER"
else
  echo "==> chsh or zsh not found, skipping default shell change"
fi

# NetworkManager (nmcli) + audio + notifications
echo "==> Installing network, audio, and notification tools"
sudo apt install -y \
  network-manager \
  network-manager-gnome \
  pulseaudio-utils \
  pipewire-pulse \
  libnotify-bin \
  dunst \
  volumeicon-alsa

# Guard: Only enable NetworkManager if systemctl exists
if command -v systemctl &>/dev/null 2>&1; then
  sudo systemctl enable --now NetworkManager || true
else
  echo "==> systemctl not found, skipping NetworkManager enable"
fi

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
  firefox-esr \
  rofi

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
if command -v nm-applet >/dev/null 2>&1; then
  nm-applet &
fi

if command -v volumeicon >/dev/null 2>&1; then
  volumeicon &
fi
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
