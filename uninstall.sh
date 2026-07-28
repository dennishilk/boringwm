#!/usr/bin/env bash
set -Eeuo pipefail
readonly REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
PREFIX=/usr/local TARGET_USER= TARGET_HOME=

die() { printf 'Error: %s\n' "$*" >&2; exit 1; }
((EUID == 0)) || die $'BoringWM Uninstaller requires administrative privileges.\nRun: sudo ./uninstall.sh'

read -r -p "Existing user whose generated files should be considered (leave blank for system files only): " TARGET_USER
if [[ -n $TARGET_USER ]]; then
    [[ $TARGET_USER != root ]] || die 'Root cannot be selected.'
    record=$(getent passwd "$TARGET_USER") || die 'User does not exist.'
    IFS=: read -r _ _ _ _ _ TARGET_HOME _ <<<"$record"
fi

cat <<'EOF'
What should be removed?
  1. BoringWM system files only (configuration is preserved)
  2. System files and installer-generated user configuration
  3. Restore newest startx backup
  4. Disable LightDM configured for BoringWM (packages remain installed)
  5. Complete removal (2 and 4; shared packages remain installed)
  6. Exit
EOF
read -r -p '> ' choice

remove_system() { make -C "$REPO_ROOT" uninstall PREFIX="$PREFIX"; }
remove_config() {
    [[ -n $TARGET_HOME ]] || die 'Select an existing user first.'
    local dir="$TARGET_HOME/.config/boringwm" backup
    [[ -d $dir ]] || return 0
    backup="$TARGET_HOME/.config/boringwm.uninstall-backup-$(date +%Y%m%d-%H%M%S)"
    cp -a -- "$dir" "$backup"
    rm -f -- "$dir/config.toml" "$dir/autostart.sh" "$dir/wallpaper.svg"
    rmdir -- "$dir" 2>/dev/null || true
    printf 'Configuration backup: %s\n' "$backup"
}
restore_xinit() {
    [[ -n $TARGET_HOME ]] || die 'Select an existing user first.'
    local backups=() latest
    shopt -s nullglob; backups=("$TARGET_HOME"/.xinitrc.boringwm-backup-*); shopt -u nullglob
    ((${#backups[@]})) || die 'No BoringWM .xinitrc backup was found.'
    latest=${backups[${#backups[@]}-1]}
    [[ ! -e $TARGET_HOME/.xinitrc ]] || cp -a -- "$TARGET_HOME/.xinitrc" "$TARGET_HOME/.xinitrc.before-restore-$(date +%Y%m%d-%H%M%S)"
    cp -a -- "$latest" "$TARGET_HOME/.xinitrc"; chown "$TARGET_USER:$(id -gn "$TARGET_USER")" "$TARGET_HOME/.xinitrc"
}
disable_lightdm() { printf 'Warning: this affects every graphical desktop using LightDM.\n'; read -r -p 'Disable LightDM and select console boot? [y/N] ' a; [[ $a == y || $a == Y ]] || return; systemctl disable lightdm.service; systemctl set-default multi-user.target; }

case $choice in 1) remove_system ;; 2) remove_system; remove_config ;; 3) restore_xinit ;; 4) disable_lightdm ;; 5) remove_system; remove_config; disable_lightdm ;; 6) exit 0 ;; *) die 'Choose 1-6.' ;; esac
printf '\nRemoval completed. User accounts, passwords, and shared packages were not changed.\n'
