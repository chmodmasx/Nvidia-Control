#!/usr/bin/env bash
set -euo pipefail
helper="/usr/libexec/nvidia-control-power-helper"
policy="/usr/share/polkit-1/actions/io.github.chmodmasx.nvidiacontrol.policy"

if (( EUID == 0 )); then
    echo "Run this script as your desktop user (not sudo bash)." >&2
    exit 1
fi

echo "Remove Nvidia-Control's root-owned power helper and Polkit action?"
read -r -p "Type REMOVE to proceed: " confirmation
if [[ "$confirmation" != "REMOVE" ]]; then
    echo "Cancelled"
    exit 1
fi
# Only remove files identified as Nvidia-Control components.
if [[ -f "$policy" ]] && grep -q 'io.github.chmodmasx.nvidiacontrol.power-limit' "$policy"; then
    sudo rm -f -- "$policy"
fi
if [[ -f "$helper" ]] && [[ "$(stat -c %u "$helper")" == "0" ]]; then
    sudo rm -f -- "$helper"
fi
echo "Power-helper uninstallation completed. Monitoring components were not changed."
