#!/usr/bin/env bash
# Pure installer decisions. This file may be sourced by tests; it changes nothing.

detect_debian_13() {
    local file=${1:-/etc/os-release} id version codename
    [[ -r "$file" ]] || { printf 'Cannot read %s.\n' "$file" >&2; return 1; }
    id=$( ( . "$file"; printf '%s' "${ID:-}" ) )
    version=$( ( . "$file"; printf '%s' "${VERSION_ID:-}" ) )
    codename=$( ( . "$file"; printf '%s' "${VERSION_CODENAME:-}" ) )
    [[ $id == debian ]] || { printf 'Unsupported distribution: %s. Only Debian 13 stable is supported.\n' "${id:-unknown}" >&2; return 1; }
    [[ $version == 13 && $codename == trixie ]] || { printf 'Unsupported Debian release: %s (%s). Only Debian 13 stable is supported.\n' "${version:-unknown}" "${codename:-unknown}" >&2; return 1; }
}

valid_desktop_account() {
    local name=$1 uid=$2 home=$3 shell=$4
    [[ $name != root && $uid =~ ^[0-9]+$ && $uid -ge ${UID_MIN:-1000} && $uid -le ${UID_MAX:-59999} ]] || return 1
    [[ $home == /* && -d $home ]] || return 1
    case "$shell" in ''|*/nologin|*/false) return 1 ;; esac
}

find_existing_users_from_file() {
    local file=$1 name _ uid _ home shell
    while IFS=: read -r name _ uid _ _ home shell; do
        valid_desktop_account "$name" "$uid" "$home" "$shell" && printf '%s:%s\n' "$name" "$home"
    done < "$file"
}

resolve_user() {
    local user=$1 record uid home shell
    [[ $user != root ]] || return 1
    record=$(getent passwd "$user") || return 1
    IFS=: read -r _ _ uid _ _ home shell <<< "$record"
    valid_desktop_account "$user" "$uid" "$home" "$shell" || return 1
    printf '%s\n' "$home"
}

build_package_plan() {
    local login=$1 profile=$2 picom=$3 feh=$4 dunst=$5 fonts=$6 xterm=${7:-no}
    BUILD_PACKAGES=(build-essential cargo rustc libxcb1-dev pkg-config)
    X11_PACKAGES=(xorg xinit dbus-x11 x11-xserver-utils kitty)
    APPLICATION_PACKAGES=()
    HELPER_PACKAGES=()
    LOGIN_PACKAGES=()
    if [[ $profile == complete ]]; then
        # policykit-1 was a transitional package and is no longer part of
        # Debian 13.  polkitd is the actual Trixie package.
        APPLICATION_PACKAGES=(rofi thunar firefox-esr gvfs polkitd)
    fi
    [[ $picom == yes ]] && HELPER_PACKAGES+=(picom)
    [[ $feh == yes ]] && HELPER_PACKAGES+=(feh)
    [[ $dunst == yes ]] && HELPER_PACKAGES+=(dunst)
    [[ $fonts == yes ]] && HELPER_PACKAGES+=(fonts-dejavu fonts-liberation)
    [[ $xterm == yes ]] && HELPER_PACKAGES+=(xterm)
    [[ $login == lightdm ]] && LOGIN_PACKAGES=(lightdm lightdm-gtk-greeter)
    return 0
}

package_plan() {
    printf '%s\n' "${BUILD_PACKAGES[@]}" "${X11_PACKAGES[@]}" \
        "${APPLICATION_PACKAGES[@]}" "${HELPER_PACKAGES[@]}" \
        "${LOGIN_PACKAGES[@]}" | sed '/^$/d' | sort -u
}

file_decision() {
    local path=$1 replacement=${2:-keep}
    [[ -e $path ]] || { printf 'new\n'; return; }
    case "$replacement" in replace) printf 'backup and replace\n' ;; keep) printf 'keep existing\n' ;; *) return 1 ;; esac
}
