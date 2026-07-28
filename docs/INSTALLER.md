# Debian 13 guided installer

> **The BoringWM installer never creates users and never asks for or changes passwords. Use the normal Debian installer to create your user account first.**

The guided installer is deliberately small: Bash menus, Debian packages, Cargo, and the existing Makefile. It supports **Debian 13 stable (Trixie), amd64, X11, and a local Git clone**. It does not claim support for Ubuntu, testing/unstable, other distributions or architectures, Wayland, or LXC. It may work over a physical TTY, SSH, or a Proxmox console; no mouse is required.

## Install

Start from a fresh minimal installation with an existing non-root user:

```sh
sudo apt update
sudo apt install git
git clone https://github.com/dennishilk/boringwm.git
cd boringwm
sudo ./install.sh
```

The script must run as root but never invokes `sudo` itself. It resolves its own directory, so `sudo /home/dennis/boringwm/install.sh` also works. Before the final confirmation it performs detection and planning only; it does not install packages or edit files. The simple terminal menu lets you revisit the user, login method, profile, optional helpers, and review. On terminals without `dialog`/`whiptail`, its readable numbered Bash interface is used without bootstrapping packages.

## Existing-user rules

Accounts come exclusively from `getent passwd`. A candidate has UID 1000–59999, a usable interactive shell, and an existing absolute home directory. Root and system/service accounts are rejected. A valid `SUDO_USER` is preferred, one unambiguous account is preselected, and multiple candidates produce a selection. The home is read from the account database, not assumed to be `/home/$USER`. No candidate causes a safe stop.

## Login methods

### LightDM graphical login

The installer adds `lightdm` and `lightdm-gtk-greeter`, enables LightDM, chooses `graphical.target`, and uses the Makefile-installed `/usr/share/xsessions/boringwm.desktop` and `/usr/local/bin/boringwm-session`. It does not enable autologin or force a session for all users. If another display-manager service is configured, interactive installation stops rather than silently replacing it; review that conflict and explicitly use `--yes` only when replacement is intended. No other display manager is removed.

### Console and `startx`

This path installs Xorg and xinit, selects `multi-user.target`, and writes an executable `~/.xinitrc` that executes `/usr/local/bin/boringwm-session`. It neither installs/enables LightDM nor starts X during login. Log in normally and run `startx`.

## Profiles and packages

Both profiles install `build-essential`, Debian's `cargo`/`rustc`, `libxcb1-dev`, `pkg-config`, `xorg`, `xinit`, `dbus-x11`, `x11-xserver-utils`, and `kitty`. Debian 13's packaged toolchain is used; no Rustup, third-party repository, signing key, binary download, upgrade, or source-list edit occurs.

**Complete desktop** (default) adds `rofi`, `thunar`, `firefox-esr`, `gvfs`, and `polkitd`. (`policykit-1` was only a transitional package and is not used on Debian 13.) Its defaults match the canonical example configuration and key bindings. Its restrained optional checklist defaults to picom, feh, dunst, DejaVu/Liberation fonts, with xterm available as an off-by-default fallback. Helpers remain external to the Rust core.

**Minimal** adds no browser, file manager, launcher, compositor, wallpaper tool, notifications, or extra fonts. Its generated config maps unavailable application bindings to kitty so every launch binding remains honest and functional.

LightDM packages are added only for the LightDM path. `--no-install-recommends` keeps the plan focused. The review shows every package group before apt runs.

## Build, files, and safety

The phases are detection, existing-user selection, choices, review, apt index, grouped package installation, Rust checks/build, Makefile installation, user configuration, login configuration, and verification. Cargo output and commands go to the mode-0600 `/var/log/boringwm-installer.log`. A failed phase is named and false success is never shown. The current implementation builds in the clone as the invoking root; this is a known tradeoff for clones that a selected account cannot access, and can leave root-owned `target/` artifacts. It never modifies Rust source.

The Makefile owns these repeatable system paths:

* `/usr/local/bin/boringwm` and `/usr/local/bin/boringwm-session`
* `/usr/share/xsessions/boringwm.desktop`
* `/usr/local/share/man/man1/boringwm.1`
* `/usr/local/share/doc/boringwm/boringwm.example.toml`

User paths are `~/.config/boringwm/config.toml`, executable `autostart.sh`, optional repository-owned `wallpaper.svg`, and (for startx) `~/.xinitrc`. Directories/files are explicitly mode 0755/0644/0755 and owned by the selected user's primary group. Existing files are kept by default. `--replace-existing` makes timestamped `.boringwm-backup-YYYYMMDD-HHMMSS` copies before replacement. Re-running is a safe reinstall/repair: system files are replaced in place, sessions and autostart commands are not duplicated, and user files remain intact by default.

The generated POSIX autostart checks availability and existing process names before starting selected feh, picom, and dunst helpers. It starts no panel and downloads nothing.

## Dry run and automation

Dry run detects the real system/user and prints the complete file, package, login, and build plan but performs no apt, Cargo build, copy, service, target, or home-directory change:

```sh
sudo ./install.sh --dry-run
```

Automation requires an unambiguous existing user and explicit login method. Existing files stay untouched unless replacement is explicit:

```sh
sudo ./install.sh --non-interactive --yes --user dennis \
  --login lightdm --profile complete --without-picom
sudo ./install.sh --non-interactive --yes --user dennis \
  --login startx --profile minimal
```

Run `./install.sh --help` for toggles for picom, feh, dunst, fonts, xterm, dry-run, and replacement. Invalid or missing required choices return nonzero.

## Uninstall

Run `sudo ./uninstall.sh`. The numbered menu can remove Makefile-owned system files only (the safe default), additionally back up and remove only known generated files inside `~/.config/boringwm`, restore the newest `.xinitrc` backup, or explicitly disable LightDM and return to console boot. Complete removal combines those operations. It does not remove accounts, passwords, applications, X11, LightDM packages, or an entire `.config` directory. Disabling LightDM has a prominent shared-desktop warning.

## Troubleshooting and Proxmox

After updating apt, the installer now checks that every planned package is available before making an installation attempt. Package groups are installed separately, so a failure names the affected group, prints the failed command and the last 30 log lines, and retains the complete output in `/var/log/boringwm-installer.log`. Confirm `/etc/os-release` says Debian 13/Trixie, `dpkg --print-architecture` says `amd64`, the selected home exists, and the Debian 13 `main` repository is enabled. Fixing the repository or package-manager error and rerunning is safe. For config errors run `RUST_LOG=boringwm=debug boringwm` inside X.

For Proxmox, use a Debian 13 amd64 VM (not an LXC), install the standard Debian user during setup, allocate a practical video display and memory, and test keyboard capture in the noVNC/SPICE console. The installer does not alter the hypervisor, VM networking, SSH, firewall, hostname, kernel, or guest tools.

Known limitations: X11 and one screen only; no automated graphical launch; no VM hardware validation; another display manager needs an explicit administrator decision; existing-file inspection is done outside the installer; and a real Debian 13 LightDM/startx reboot was not exercised by CI.

## Manual installer test plan

These scenarios require disposable Debian 13 amd64 VMs and remain manual:

* **A — LightDM:** minimal VM, one user, complete profile; reboot, select BoringWM, and verify login, terminal, rofi, Thunar, Firefox ESR, wallpaper, picom, dunst, restart, and exit bindings.
* **B — startx:** minimal VM, minimal profile; reboot/return to console, log in, run `startx`, verify BoringWM and kitty, exit X, and verify console boot remains.
* **C — existing `.xinitrc`:** create it, confirm default preservation, repeat with backup/replacement, verify timestamped backup, then use uninstall restore.
* **D — existing config:** customize `config.toml`, rerun and verify preservation; repeat with explicit backup/replacement.
* **E — multiple users:** create two valid users, remove `SUDO_USER` ambiguity, select one, and verify only that home changes.
* **F — reinstall:** rerun after success; verify in-place system repair and no duplicate session/autostart entries.
* **G — recovery:** simulate apt/build failure; verify phase, protected log, nonzero status, no success screen, and limited partial configuration.
* **H — dry run:** record package/service/filesystem state, execute `--dry-run`, and prove it is unchanged.
