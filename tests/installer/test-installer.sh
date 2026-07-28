#!/usr/bin/env bash
set -Eeuo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
# shellcheck source=scripts/installer/lib.sh
source "$ROOT/scripts/installer/lib.sh"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
pass=0
ok() { "$@" || { printf 'FAIL: %s\n' "$*" >&2; exit 1; }; pass=$((pass+1)); }
fail() { if "$@" >/dev/null 2>&1; then printf 'UNEXPECTED PASS: %s\n' "$*" >&2; exit 1; fi; pass=$((pass+1)); }

cat >"$tmp/debian" <<EOF
ID=debian
VERSION_ID="13"
VERSION_CODENAME=trixie
EOF
cat >"$tmp/ubuntu" <<EOF
ID=ubuntu
VERSION_ID="24.04"
EOF
cat >"$tmp/old" <<EOF
ID=debian
VERSION_ID="12"
VERSION_CODENAME=bookworm
EOF
ok detect_debian_13 "$tmp/debian"; fail detect_debian_13 "$tmp/ubuntu"; fail detect_debian_13 "$tmp/old"
mkdir "$tmp/alice" "$tmp/bob"
cat >"$tmp/passwd" <<EOF
root:x:0:0:root:/root:/bin/bash
daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin
alice:x:1000:1000::${tmp}/alice:/bin/bash
bob:x:1001:1001::${tmp}/bob:/bin/zsh
service:x:999:999::${tmp}:/bin/bash
EOF
mapfile -t users < <(find_existing_users_from_file "$tmp/passwd")
ok test "${#users[@]}" -eq 2; ok test "${users[0]}" = "alice:$tmp/alice"; fail valid_desktop_account root 0 /root /bin/bash
ok valid_desktop_account alice 1000 "$tmp/alice" /bin/bash
build_package_plan lightdm complete yes yes yes yes no
ok test " ${LOGIN_PACKAGES[*]} " = ' lightdm lightdm-gtk-greeter '; ok test " ${APPLICATION_PACKAGES[*]} " = ' rofi thunar firefox-esr gvfs polkitd '
mapfile -t packages < <(package_plan)
ok test "${packages[*]}" = 'build-essential cargo dbus-x11 dunst feh firefox-esr fonts-dejavu fonts-liberation gvfs kitty libxcb1-dev lightdm lightdm-gtk-greeter picom pkg-config polkitd rofi rustc thunar x11-xserver-utils xinit xorg'
build_package_plan startx minimal no no no no no
ok test "${#LOGIN_PACKAGES[@]}" -eq 0; ok test "${#APPLICATION_PACKAGES[@]}" -eq 0; ok test " ${X11_PACKAGES[*]} " = ' xorg xinit dbus-x11 x11-xserver-utils kitty '
touch "$tmp/existing"; ok test "$(file_decision "$tmp/existing" keep)" = 'keep existing'; ok test "$(file_decision "$tmp/existing" replace)" = 'backup and replace'; ok test "$(file_decision "$tmp/new" keep)" = new
dry_line=$(grep -n 'if \[\[ \$DRY_RUN == yes \]\]' "$ROOT/install.sh" | cut -d: -f1)
apt_line=$(grep -n 'install_packages; cd' "$ROOT/install.sh" | cut -d: -f1)
ok test "$dry_line" -lt "$apt_line"
printf 'PASS: %d installer logic assertions\n' "$pass"
