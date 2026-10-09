#!/usr/bin/env bash
set -euo pipefail

bin_dir="$HOME/.local/bin"
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
bus_name="io.github.chmodmasx.NvidiaControl"
unit_name="nvidia-control-daemon.service"
unit_file="$config_home/systemd/user/$unit_name"
dbus_file="$data_home/dbus-1/services/$bus_name.service"
desktop_file="$data_home/applications/$bus_name.desktop"
marker="$data_home/nvidia-control/user-install.marker"

if [[ ! -f "$marker" ]]; then
    echo "No Nvidia-Control user installation marker found; nothing removed." >&2
    exit 1
fi

if command -v systemctl >/dev/null 2>&1; then
    systemctl --user stop "$unit_name" >/dev/null 2>&1 || true
fi

# Only remove activation files we previously created, never unrelated files.
for file in "$unit_file" "$dbus_file"; do
    if [[ -f "$file" ]] && grep -q '^# Managed by Nvidia-Control' "$file"; then
        rm -f -- "$file"
    fi
done
if [[ -f "$desktop_file" ]] &&
    grep -q '^X-Nvidia-Control-Managed=true$' "$desktop_file"; then
    rm -f -- "$desktop_file"
fi

rm -f -- "$bin_dir/nvidia-control" "$bin_dir/nvidia-control-daemon" "$marker"

if command -v systemctl >/dev/null 2>&1; then
    systemctl --user daemon-reload ||
        echo "Warning: user systemd manager unavailable" >&2
fi
if command -v dbus-send >/dev/null 2>&1; then
    dbus-send --session --dest=org.freedesktop.DBus --type=method_call \
        / org.freedesktop.DBus.ReloadConfig >/dev/null 2>&1 || true
fi

echo "Removed Nvidia-Control's per-user executables and activation files."
echo "The repository, source files and build directories were left untouched."
