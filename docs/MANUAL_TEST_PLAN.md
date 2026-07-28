# BoringWM manual validation checklist

This checklist requires a real X11 session and was **not performed in the automated development container**.

- [ ] Start an empty X session using `startx`; repeat through a display-manager session.
- [ ] Open 1, 2, 3, and 10 xterm/kitty windows; close them in different orders and crash one client.
- [ ] Cycle focus, reorder both ways, promote a client, and adjust the master ratio.
- [ ] Switch all workspaces and move focused tiled, floating, and fullscreen clients between them.
- [ ] Open Firefox and Thunar file choosers; verify dialogs float and remain focusable.
- [ ] Toggle floating and fullscreen repeatedly with xterm, Firefox video, mpv, and an SDL game/test.
- [ ] Run rofi and verify its override-redirect or transient windows are never tiled.
- [ ] Restart and exit; verify clients survive and are visible.
- [ ] Test missing and malformed config, missing autostart, and missing command executables.
- [ ] Run picom and feh externally and verify BoringWM does not interfere.
- [ ] On two monitors, including a monitor with a non-zero origin, assess placement. Multi-monitor discovery is currently not implemented, so record this expected limitation.
