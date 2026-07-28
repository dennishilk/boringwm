#!/usr/bin/env bash
set -Eeuo pipefail

readonly REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly PREFIX=/usr/local LOG_FILE=/var/log/boringwm-installer.log
# shellcheck source=scripts/installer/lib.sh
source "$REPO_ROOT/scripts/installer/lib.sh"

DRY_RUN=no NON_INTERACTIVE=no ASSUME_YES=no TARGET_USER= LOGIN_METHOD= PROFILE=complete
WITH_PICOM=yes WITH_FEH=yes WITH_DUNST=yes WITH_FONTS=yes WITH_XTERM=no REPLACE_EXISTING=no
TARGET_HOME= TARGET_GROUP= CURRENT_PHASE="System detection" TEMP_DIR= UI=text

usage() { cat <<'EOF'
Usage: sudo ./install.sh [OPTIONS]
  --user USER                 existing normal user (never created)
  --login lightdm|startx      graphical login or console/startx
  --profile complete|minimal  installation profile
  --with-picom|--without-picom (also feh, dunst, fonts, xterm)
  --replace-existing          back up and replace managed user files
  --non-interactive --yes     accept the reviewed plan
  --dry-run                   show the complete plan; change nothing
  --help
EOF
}

die() { printf '\nError: %s\n' "$*" >&2; exit 1; }
on_error() { local status=$?; printf '\nInstallation failed during:\n\n  %s\n\nDetails were written to:\n\n  %s\n' "$CURRENT_PHASE" "$LOG_FILE" >&2; exit "$status"; }
cleanup() { [[ -z ${TEMP_DIR:-} ]] || rm -rf -- "$TEMP_DIR"; }
trap cleanup EXIT
trap on_error ERR

parse_args() {
    while (($#)); do
        case "$1" in
            --user) (($# >= 2)) || die '--user needs a value'; TARGET_USER=$2; shift ;;
            --login) (($# >= 2)) || die '--login needs a value'; LOGIN_METHOD=$2; shift ;;
            --profile) (($# >= 2)) || die '--profile needs a value'; PROFILE=$2; shift ;;
            --with-picom) WITH_PICOM=yes ;; --without-picom) WITH_PICOM=no ;;
            --with-feh) WITH_FEH=yes ;; --without-feh) WITH_FEH=no ;;
            --with-dunst) WITH_DUNST=yes ;; --without-dunst) WITH_DUNST=no ;;
            --with-fonts) WITH_FONTS=yes ;; --without-fonts) WITH_FONTS=no ;;
            --with-xterm) WITH_XTERM=yes ;; --without-xterm) WITH_XTERM=no ;;
            --replace-existing) REPLACE_EXISTING=yes ;;
            --non-interactive) NON_INTERACTIVE=yes ;; --yes) ASSUME_YES=yes ;;
            --dry-run) DRY_RUN=yes ;; --help|-h) usage; exit 0 ;;
            *) die "Unknown option: $1" ;;
        esac; shift
    done
    [[ $LOGIN_METHOD == '' || $LOGIN_METHOD == lightdm || $LOGIN_METHOD == startx ]] || die 'Login must be lightdm or startx.'
    [[ $PROFILE == complete || $PROFILE == minimal ]] || die 'Profile must be complete or minimal.'
    if [[ $PROFILE == minimal ]]; then WITH_PICOM=no; WITH_FEH=no; WITH_DUNST=no; WITH_FONTS=no; fi
}

require_root() {
    if ((EUID != 0)); then cat >&2 <<'EOF'
BoringWM Installer requires administrative privileges.

Run:

  sudo ./install.sh
EOF
        exit 1
    fi
}

detect_system() {
    detect_debian_13 "${BORINGWM_OS_RELEASE:-/etc/os-release}" || exit 1
    ARCHITECTURE=$(dpkg --print-architecture)
    [[ $ARCHITECTURE == amd64 ]] || die "Unsupported architecture: $ARCHITECTURE. The supported target is Debian 13 amd64."
}

find_existing_users() {
    local entry candidates=()
    mapfile -t candidates < <(getent passwd | find_existing_users_from_file /dev/stdin)
    ((${#candidates[@]})) || die $'No existing normal user was found.\n\nCreate a user through Debian first, then run the installer again.\nThe BoringWM installer does not create user accounts or passwords.'
    if [[ -z $TARGET_USER && ${SUDO_USER:-root} != root ]] && resolve_user "$SUDO_USER" >/dev/null; then TARGET_USER=$SUDO_USER; fi
    if [[ -z $TARGET_USER && ${#candidates[@]} -eq 1 ]]; then TARGET_USER=${candidates[0]%%:*}; fi
    if [[ -z $TARGET_USER ]]; then
        [[ $NON_INTERACTIVE == no ]] || die 'Multiple users exist; supply --user USER.'
        printf 'Select the existing user who will use BoringWM:\n'
        select entry in "${candidates[@]}"; do [[ -n $entry ]] && { TARGET_USER=${entry%%:*}; break; }; done
    fi
    TARGET_HOME=$(resolve_user "$TARGET_USER") || die "'$TARGET_USER' is not a suitable existing normal user."
    TARGET_GROUP=$(id -gn "$TARGET_USER")
}

choose() { local prompt=$1 default=$2 answer; read -r -p "$prompt [$default]: " answer; printf '%s' "${answer:-$default}"; }
select_choices() {
    [[ -n $LOGIN_METHOD ]] || LOGIN_METHOD=$(choose 'Login method (lightdm/startx)' lightdm)
    [[ $LOGIN_METHOD == lightdm || $LOGIN_METHOD == startx ]] || die 'Invalid login selection.'
    if [[ $NON_INTERACTIVE == no ]]; then PROFILE=$(choose 'Profile (complete/minimal)' "$PROFILE"); fi
    [[ $PROFILE == complete || $PROFILE == minimal ]] || die 'Invalid profile selection.'
    if [[ $PROFILE == minimal ]]; then WITH_PICOM=no; WITH_FEH=no; WITH_DUNST=no; WITH_FONTS=no; fi
}

welcome() { command -v clear >/dev/null && clear || true; cat <<'EOF'
┌──────────────────────────────────────────────┐
│                 BoringWM                     │
│              Debian Installer                │
│                                              │
│          boring is not a bug.                │
│              it's a feature.                 │
└──────────────────────────────────────────────┘
EOF
    printf '\nDebian GNU/Linux 13 detected\nArchitecture: %s\n\n' "$ARCHITECTURE"
}

menu_loop() {
    [[ $NON_INTERACTIVE == no ]] || return
    while :; do
        cat <<EOF
BoringWM Installation
  1. Target user          $TARGET_USER
  2. Login method        ${LOGIN_METHOD:-not selected}
  3. Installation profile $PROFILE
  4. Optional components picom=$WITH_PICOM, feh=$WITH_FEH, dunst=$WITH_DUNST, fonts=$WITH_FONTS, xterm=$WITH_XTERM
  5. Review installation
  6. Begin installation
  7. Exit
Keyboard: enter a number, then press Enter.
EOF
        read -r -p '> ' choice
        case $choice in
            1) TARGET_USER=$(choose 'Existing username' "$TARGET_USER"); TARGET_HOME=$(resolve_user "$TARGET_USER") || die 'Invalid user'; TARGET_GROUP=$(id -gn "$TARGET_USER") ;;
            2) LOGIN_METHOD=$(choose 'lightdm or startx' "${LOGIN_METHOD:-lightdm}") ;;
            3) PROFILE=$(choose 'complete or minimal' "$PROFILE"); [[ $PROFILE == minimal ]] && WITH_PICOM=no && WITH_FEH=no && WITH_DUNST=no && WITH_FONTS=no ;;
            4) [[ $PROFILE == complete ]] || { printf 'Optional helpers are disabled for the minimal profile.\n'; continue; }; WITH_PICOM=$(choose 'picom (yes/no)' "$WITH_PICOM"); WITH_FEH=$(choose 'feh (yes/no)' "$WITH_FEH"); WITH_DUNST=$(choose 'dunst (yes/no)' "$WITH_DUNST"); WITH_FONTS=$(choose 'fonts (yes/no)' "$WITH_FONTS"); WITH_XTERM=$(choose 'xterm fallback (yes/no)' "$WITH_XTERM") ;;
            5) show_installation_summary; read -r -p 'Press Enter to return.' _ ;;
            6) break ;; 7) exit 0 ;; *) printf 'Choose 1-7.\n' ;;
        esac
    done
}

show_installation_summary() {
    build_package_plan "$LOGIN_METHOD" "$PROFILE" "$WITH_PICOM" "$WITH_FEH" "$WITH_DUNST" "$WITH_FONTS" "$WITH_XTERM"
    local action=keep; [[ $REPLACE_EXISTING == yes ]] && action=replace
    cat <<EOF

Review BoringWM installation
System: Debian GNU/Linux 13 ($ARCHITECTURE), host $(hostname)
User: $TARGET_USER ($TARGET_HOME)
Source: $REPO_ROOT    Prefix: $PREFIX    Build: release
Login: $LOGIN_METHOD ($( [[ $LOGIN_METHOD == lightdm ]] && echo graphical.target || echo multi-user.target))
Profile: $PROFILE
Build packages: ${BUILD_PACKAGES[*]}
X11 packages: ${X11_PACKAGES[*]}
Applications: ${APPLICATION_PACKAGES[*]:-none}
Desktop helpers: ${HELPER_PACKAGES[*]:-none}
Login packages: ${LOGIN_PACKAGES[*]:-none}
Existing files:
  config.toml: $(file_decision "$TARGET_HOME/.config/boringwm/config.toml" "$action")
  autostart.sh: $(file_decision "$TARGET_HOME/.config/boringwm/autostart.sh" "$action")
  .xinitrc: $( [[ $LOGIN_METHOD == startx ]] && file_decision "$TARGET_HOME/.xinitrc" "$action" || echo 'not used')
No user account or password will be created or changed.
EOF
}

run_logged() { printf '+ %q ' "$@" >>"$LOG_FILE"; printf '\n' >>"$LOG_FILE"; "$@" >>"$LOG_FILE" 2>&1; }
phase() { CURRENT_PHASE=$2; printf '[%s/10] %s\n' "$1" "$2"; }
install_packages() {
    phase 1 'Updating Debian package index'; run_logged apt-get update
    phase 2 'Installing build dependencies'; DEBIAN_FRONTEND=noninteractive run_logged apt-get install -y --no-install-recommends "${BUILD_PACKAGES[@]}"
    phase 3 'Installing X11 packages'; DEBIAN_FRONTEND=noninteractive run_logged apt-get install -y --no-install-recommends "${X11_PACKAGES[@]}"
    phase 4 'Installing desktop applications'; local p=("${APPLICATION_PACKAGES[@]}" "${HELPER_PACKAGES[@]}" "${LOGIN_PACKAGES[@]}"); ((${#p[@]} == 0)) || DEBIAN_FRONTEND=noninteractive run_logged apt-get install -y --no-install-recommends "${p[@]}"
}

build_boringwm() {
    phase 5 'Running BoringWM tests'; run_logged cargo fmt --all -- --check; run_logged cargo test --all
    command -v cargo-clippy >/dev/null && run_logged cargo clippy --all-targets --all-features -- -D warnings
    phase 6 'Building BoringWM release binary'; run_logged cargo build --release
}
install_boringwm() { phase 7 'Installing BoringWM system files'; run_logged make install PREFIX="$PREFIX"; }

backup_if_needed() {
    local path=$1
    [[ -e $path ]] || return 0
    [[ $REPLACE_EXISTING == yes ]] || return 1
    cp -a -- "$path" "$path.boringwm-backup-$(date +%Y%m%d-%H%M%S)"
}

write_managed() { local source=$1 destination=$2 mode=$3; if [[ -e $destination ]] && ! backup_if_needed "$destination"; then printf 'Keeping existing %s.\n' "$destination"; return; fi; install -m "$mode" "$source" "$destination"; chown "$TARGET_USER:$TARGET_GROUP" "$destination"; }
configure_user() {
    phase 8 'Writing user configuration'; local dir="$TARGET_HOME/.config/boringwm" autostart="$TEMP_DIR/autostart.sh"
    install -d -m 0755 -o "$TARGET_USER" -g "$TARGET_GROUP" "$dir"
    cp "$REPO_ROOT/config/boringwm.example.toml" "$TEMP_DIR/config.toml"
    if [[ $PROFILE == minimal ]]; then sed -i 's/^file_manager =.*/file_manager = ["kitty"]\n/; s/^browser =.*/browser = ["kitty"]\n/; s/^launcher =.*/launcher = ["kitty"]\n/' "$TEMP_DIR/config.toml"; fi
    cat >"$autostart" <<'EOF'
#!/bin/sh
# Generated by the BoringWM installer. Optional helpers are guarded and de-duplicated.
EOF
    [[ $WITH_FEH == yes ]] && printf 'command -v feh >/dev/null 2>&1 && ! pgrep -x feh >/dev/null 2>&1 && feh --bg-fill "$HOME/.config/boringwm/wallpaper.svg" &\n' >>"$autostart"
    [[ $WITH_PICOM == yes ]] && printf 'command -v picom >/dev/null 2>&1 && ! pgrep -x picom >/dev/null 2>&1 && picom &\n' >>"$autostart"
    [[ $WITH_DUNST == yes ]] && printf 'command -v dunst >/dev/null 2>&1 && ! pgrep -x dunst >/dev/null 2>&1 && dunst &\n' >>"$autostart"
    write_managed "$TEMP_DIR/config.toml" "$dir/config.toml" 0644
    write_managed "$autostart" "$dir/autostart.sh" 0755
    [[ $WITH_FEH == yes ]] && write_managed "$REPO_ROOT/assets/wallpaper/boringwm-wallpaper.svg" "$dir/wallpaper.svg" 0644
}

configure_login() {
    phase 9 "Configuring $LOGIN_METHOD"
    if [[ $LOGIN_METHOD == startx ]]; then
        printf '#!/bin/sh\nexec %s/bin/boringwm-session\n' "$PREFIX" >"$TEMP_DIR/xinitrc"
        write_managed "$TEMP_DIR/xinitrc" "$TARGET_HOME/.xinitrc" 0755
        run_logged systemctl set-default multi-user.target
    else
        local dm
        dm=$(basename "$(readlink -f /etc/systemd/system/display-manager.service 2>/dev/null || true)")
        if [[ -n $dm && $dm != lightdm.service && $ASSUME_YES != yes ]]; then die "Another display manager is configured: $dm. Re-run with --yes only after deciding to replace it, or choose startx."; fi
        run_logged systemctl enable lightdm.service
        run_logged systemctl set-default graphical.target
    fi
}

verify_installation() {
    phase 10 'Verifying installation'
    [[ -x $PREFIX/bin/boringwm && -x $PREFIX/bin/boringwm-session && -f $PREFIX/share/man/man1/boringwm.1 && -f /usr/share/xsessions/boringwm.desktop ]]
    [[ -d $TARGET_HOME/.config/boringwm && -x $TARGET_HOME/.config/boringwm/autostart.sh ]]
    ! find "$TARGET_HOME/.config/boringwm" -user root -print -quit | grep -q .
    if [[ $LOGIN_METHOD == startx ]]; then [[ -x $TARGET_HOME/.xinitrc ]] && grep -Fq "$PREFIX/bin/boringwm-session" "$TARGET_HOME/.xinitrc"; else dpkg-query -W lightdm lightdm-gtk-greeter >/dev/null; systemctl is-enabled --quiet lightdm.service; [[ $(systemctl get-default) == graphical.target ]]; ! rg -i 'autologin-user\s*=' /etc/lightdm >/dev/null 2>&1; fi
}

complete() { printf '\nBoringWM installation completed successfully.\n\nUser\n  %s\n\nLogin method\n  %s\n\nConfiguration\n  %s/.config/boringwm/config.toml\n\nAutostart\n  %s/.config/boringwm/autostart.sh\n' "$TARGET_USER" "$LOGIN_METHOD" "$TARGET_HOME" "$TARGET_HOME"; [[ $LOGIN_METHOD == startx ]] && printf '\nLog in as %s and run:\n\n  startx\n' "$TARGET_USER" || printf '\nThe system is ready. Reboot with: sudo reboot\nAt the login screen, select “BoringWM” if required.\n'; }

main() {
    parse_args "$@"; require_root; detect_system; TEMP_DIR=$(mktemp -d -t boringwm-installer.XXXXXX); find_existing_users; welcome
    if [[ $NON_INTERACTIVE == yes ]]; then [[ -n $LOGIN_METHOD ]] || die '--login is required in non-interactive mode.'; else select_choices; menu_loop; fi
    show_installation_summary
    if [[ $DRY_RUN == yes ]]; then printf '\nDRY RUN: no packages, files, builds, services, or systemd targets were changed.\n'; exit 0; fi
    if [[ $ASSUME_YES != yes ]]; then read -r -p 'Begin installation? [y/N] ' answer; [[ $answer == y || $answer == Y ]] || exit 0; fi
    : >"$LOG_FILE"; chmod 0600 "$LOG_FILE"; build_package_plan "$LOGIN_METHOD" "$PROFILE" "$WITH_PICOM" "$WITH_FEH" "$WITH_DUNST" "$WITH_FONTS" "$WITH_XTERM"
    install_packages; cd "$REPO_ROOT"; build_boringwm; install_boringwm; configure_user; configure_login; verify_installation; complete
}

[[ ${BORINGWM_INSTALLER_SOURCE_ONLY:-0} == 1 ]] || main "$@"
