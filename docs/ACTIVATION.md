# Per-user D-Bus / systemd activation

The service can now start automatically when Nvidia-Control sends its first D-Bus request. This feature is **on-demand activation**, not launching the daemon at every login.

## Install (Kubuntu / KDE Plasma)

From the repository root:

```bash
# Stop an old manually running "cargo run ... -- --session" first.
bash scripts/install-user.sh
```

This builds release binaries (Rust and Qt), then installs **only in the current user's home**. No sudo is required. Prerequisites are the Rust toolchain, CMake, Ninja/G++, Qt 6 development packages and the Qt Quick runtime packages documented in [ui/README.md](../ui/README.md).

| Item | Default destination |
| --- | --- |
| GUI | `~/.local/bin/nvidia-control` |
| Daemon | `~/.local/bin/nvidia-control-daemon` |
| systemd user unit | `~/.config/systemd/user/nvidia-control-daemon.service` |
| D-Bus activation | `~/.local/share/dbus-1/services/io.github.chmodmasx.NvidiaControl.service` |
| Desktop launcher | `~/.local/share/applications/io.github.chmodmasx.NvidiaControl.desktop` |
| Installer marker | `~/.local/share/nvidia-control/user-install.marker` |

Paths beneath `~/.local/share` and `~/.config` honor `XDG_DATA_HOME` and `XDG_CONFIG_HOME`, respectively. Use the **same values** when uninstalling.

The systemd unit uses `Type=dbus` and `BusName=io.github.chmodmasx.NvidiaControl`. The D-Bus service declares `SystemdService=nvidia-control-daemon.service`, and also supplies `Exec=` for session bus implementations that activate directly. The installer reloads the user systemd manager and asks D-Bus to reload its service registry.

**It does not call `systemctl --user enable` or add a desktop autostart entry.** The service starts when requested and generally remains available until stopped or the user session ends.

## Test without a daemon terminal

1. Stop an existing manually started `cargo run -p nvidia-control-daemon -- --session` (Ctrl+C), and close any previously launched development copies of the GUI.
2. Check that the unit is currently inactive with `systemctl --user is-active nvidia-control-daemon.service` (expected `inactive`).
3. Launch `Nvidia-Control` from KDE's application menu, or run `~/.local/bin/nvidia-control`.
4. The GUI sends `GetInventory`, which triggers the service.
5. Confirm `systemctl --user status nvidia-control-daemon.service` now shows `active (running)`.

To request activation without opening the GUI:

```bash
systemctl --user stop nvidia-control-daemon.service

busctl --user call io.github.chmodmasx.NvidiaControl \
  /io/github/chmodmasx/NvidiaControl \
  io.github.chmodmasx.NvidiaControl1 GetApiVersion
# Expected: u 1
```

To examine service logs:

```bash
journalctl --user -u nvidia-control-daemon.service -n 50 --no-pager
```

If you see a `NameHasNoOwner`, `ServiceUnknown`, or an activation timeout, verify that the two service files exist at the destinations above and that `systemctl --user daemon-reload` completed. Restart the old manually launched process only for debugging, not alongside the activated daemon.

## Uninstall

```bash
bash scripts/uninstall-user.sh
```

This stops the user unit and removes the executables, managed service descriptions and launcher. It leaves the source repository and build files untouched. The removal script requires the installed marker and only removes activation/launcher definitions marked as owned by this project.

For development/CI, the installer also accepts `--prebuilt` (uses `target/debug/nvidia-control-daemon` and `build/ui/nvidia-control`), `--daemon-only`, and `--no-reload`. These flags are not needed for ordinary installation.

## Compatibility notes

- The service is read-only, on the user's **session bus**. No Polkit or root privilege is used.
- The backend is `nvml` by default, with `NVIDIA_CONTROL_BACKEND=mock` reserved for development. An installed systemd user unit does not inherit an environment variable set only in an interactive terminal.
- The integration relies on standard D-Bus service activation and a functioning systemd user manager (as provided by a typical KDE login). A standalone D-Bus bus without systemd may instead use the `Exec` fallback.
- An existing conflicting D-Bus service must be stopped before requesting activation. The bus name allows only one owner.
- Changes to `GetInventory` / `GetTelemetry` were **not** needed; no daemon or UI code changes were made for activation.
