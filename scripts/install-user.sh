#!/usr/bin/env bash
set -euo pipefail

# Install Nvidia-Control only for the current desktop user.
# D-Bus triggers systemd --user on demand; no boot or login autostart.
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
bin_dir="$HOME/.local/bin"
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
dbus_dir="$data_home/dbus-1/services"
unit_dir="$config_home/systemd/user"
desktop_dir="$data_home/applications"
manifest_dir="$data_home/nvidia-control"
bus_name="io.github.chmodmasx.NvidiaControl"
unit_name="nvidia-control-daemon.service"
prebuilt=0
daemon_only=0
no_reload=0

usage() {
    cat <<'HELP'
Usage: scripts/install-user.sh [--prebuilt] [--daemon-only] [--no-reload]
  (default)     Build optimized Rust daemon and Qt GUI before installing.
  --prebuilt    Install from target/debug/ and build/ui/ (CI/development).
  --daemon-only Install only the service (CI/headless testing).
  --no-reload   Skip systemd and D-Bus reload (isolated CI session).
No sudo needed; installs into current user's home only.
HELP
}

while (($#)); do
    case "$1" in
        --prebuilt) prebuilt=1 ;;
        --daemon-only) daemon_only=1 ;;
        --no-reload) no_reload=1 ;;
        -h|--help) usage; exit 0 ;;
        *) printf 'Unknown option: %s\n' "$1" >&2; usage >&2; exit 2 ;;
    esac
    shift
done

# Files using the XDG paths are standard user-level activation locations.
# Refuse control characters in paths; they cannot be represented safely in
# freedesktop .desktop and D-Bus service fields.
for dir in "$bin_dir" "$dbus_dir" "$unit_dir" "$desktop_dir" "$manifest_dir"; do
    if [[ "$dir" == *$'\n'* || "$dir" == *$'\r'* ]]; then
        echo "Unsupported newline/carriage-return in install path" >&2
        exit 1
    fi
done

daemon_src="$repo_root/target/release/nvidia-control-daemon"
gui_src="$repo_root/build/ui/nvidia-control"
if ((prebuilt)); then
    daemon_src="$repo_root/target/debug/nvidia-control-daemon"
else
    cargo build --manifest-path "$repo_root/Cargo.toml" --release -p nvidia-control-daemon
fi
if (( ! daemon_only && ! prebuilt )); then
    cmake -S "$repo_root/ui" -B "$repo_root/build/ui" -DCMAKE_BUILD_TYPE=Release
    cmake --build "$repo_root/build/ui" --parallel 2
fi

if [[ ! -x "$daemon_src" ]]; then
    printf 'Missing daemon executable: %s\n' "$daemon_src" >&2
    exit 1
fi
if (( ! daemon_only )) && [[ ! -x "$gui_src" ]]; then
    printf 'Missing GUI executable: %s\n' "$gui_src" >&2
    exit 1
fi

dbus_file="$dbus_dir/$bus_name.service"
unit_file="$unit_dir/$unit_name"
desktop_file="$desktop_dir/$bus_name.desktop"
marker="$manifest_dir/user-install.marker"

# Never overwrite somebody else's activation definitions.
for file in "$unit_file" "$dbus_file"; do
    if [[ -e "$file" ]] && ! grep -q '^# Managed by Nvidia-Control' "$file"; then
        printf 'Refusing to overwrite unmanaged file: %s\n' "$file" >&2
        exit 1
    fi
done
if (( ! daemon_only )) && [[ -e "$desktop_file" ]] &&
    ! grep -q '^X-Nvidia-Control-Managed=true$' "$desktop_file"; then
    printf 'Refusing to overwrite unmanaged desktop entry: %s\n' "$desktop_file" >&2
    exit 1
fi

# Use quotes in Exec fields; double escape % only for systemd specifiers.
quoted_exec() {
    local value="$1"
    value="${value//\\/\\\\}"
    value="${value//\"/\\\"}"
    value="${value//\`/\\\`}"
    value="${value//\$/\\$}"
    printf '"%s"' "$value"
}

mkdir -p "$bin_dir" "$dbus_dir" "$unit_dir" "$manifest_dir"
if (( ! daemon_only )); then
    mkdir -p "$desktop_dir"
fi

# Stop an older systemd-managed instance before replacing its binary.
# A manually launched cargo run -- --session must be stopped by the user.
if (( ! no_reload )) && command -v systemctl >/dev/null 2>&1; then
    systemctl --user stop "$unit_name" >/dev/null 2>&1 || true
fi

install -m 755 "$daemon_src" "$bin_dir/nvidia-control-daemon"
if (( ! daemon_only )); then
    install -m 755 "$gui_src" "$bin_dir/nvidia-control"
fi

dbus_exec="$(quoted_exec "$bin_dir/nvidia-control-daemon")"
unit_exec="${dbus_exec//%/%%}"
cat > "$unit_file" <<EOF
# Managed by Nvidia-Control
[Unit]
Description=Nvidia-Control read-only D-Bus service

[Service]
Type=dbus
BusName=$bus_name
ExecStart=$unit_exec --session
Restart=on-failure
RestartSec=2
NoNewPrivileges=true
EOF

cat > "$dbus_file" <<EOF
# Managed by Nvidia-Control
[D-BUS Service]
Name=$bus_name
Exec=$dbus_exec --session
SystemdService=$unit_name
EOF

if (( ! daemon_only )); then
    gui_exec="$(quoted_exec "$bin_dir/nvidia-control")"
    # The desktop-entry spec reserves % for field codes.
    gui_exec="${gui_exec//%/%%}"
    cat > "$desktop_file" <<EOF
[Desktop Entry]
Type=Application
Name=Nvidia-Control
Comment=NVIDIA GPU monitoring for Linux
Exec=$gui_exec
TryExec=$bin_dir/nvidia-control
Terminal=false
Categories=System;Settings;
Keywords=NVIDIA;GPU;telemetry;monitor;
X-Nvidia-Control-Managed=true
EOF
fi

printf '%s\n' 'Nvidia-Control user installation' > "$marker"

if (( ! no_reload )); then
    if command -v systemctl >/dev/null 2>&1; then
        systemctl --user daemon-reload ||
            echo "Warning: user systemd manager unavailable (D-Bus Exec fallback may still work)" >&2
    fi
    if command -v dbus-send >/dev/null 2>&1; then
        dbus-send --session --dest=org.freedesktop.DBus --type=method_call \
            / org.freedesktop.DBus.ReloadConfig >/dev/null 2>&1 || true
    fi
fi

echo "Installed Nvidia-Control for this user."
echo "D-Bus activation: $dbus_file"
echo "systemd user unit: $unit_file"
if (( ! daemon_only )); then
    echo "GUI launcher: $desktop_file"
fi
echo "No login autostart was enabled; D-Bus activates the daemon on first request."
