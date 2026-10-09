#!/usr/bin/env bash
set -euo pipefail
# Explicit system-wide installation. Run this script as your desktop user;
# sudo is used only for the two controlled root-owned install destinations.
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
helper="/usr/libexec/nvidia-control-power-helper"
policy="/usr/share/polkit-1/actions/io.github.chmodmasx.nvidiacontrol.policy"
src="$repo_root/target/release/nvidia-control-power-helper"
src_policy="$repo_root/packaging/polkit/io.github.chmodmasx.nvidiacontrol.policy"

if (( EUID == 0 )); then
    echo "Run this installer as a regular user (not sudo bash)." >&2
    exit 1
fi
if [[ ! -x "$src" ]]; then
    echo "Build first: cargo build --release -p nvidia-control-power-helper" >&2
    exit 1
fi
if ! command -v pkexec >/dev/null 2>&1; then
    echo "Missing pkexec (polkit); no files changed" >&2
    exit 1
fi

echo "WARNING: this will install a root-owned power-limit helper and a Polkit action."
echo "This is separate from the unprivileged monitoring daemon."
echo "Review crates/power-helper, crates/control-power and the policy before installing."
read -r -p "Type INSTALL to proceed: " confirmation
if [[ "$confirmation" != "INSTALL" ]]; then
    echo "Cancelled"
    exit 1
fi

sudo install -o root -g root -m 0755 "$src" "$helper"
sudo install -o root -g root -m 0644 "$src_policy" "$policy"
echo "Installed root-owned helper: $helper"
echo "Installed Polkit action: $policy"
echo "No power setting has been changed. Use inspect before any explicit apply."
