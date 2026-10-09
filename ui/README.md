# Nvidia-Control Qt 6 / QML frontend

A minimal native GUI for the existing Rust/NVML backend. The Qt frontend **never** loads NVML or executes privileged commands: it reads the read-only session D-Bus API asynchronously.

## Prerequisites (Ubuntu/Kubuntu)

```bash
sudo apt install cmake ninja-build g++ qt6-base-dev qt6-declarative-dev \
  qml6-module-qtquick qml6-module-qtquick-window \
  qml6-module-qtquick-controls qml6-module-qtquick-layouts \
  qml6-module-qtqml-models qml6-module-qtqml-workerscript qml6-module-qtquick-templates
```

Rust and Cargo are required for the daemon. A functioning NVIDIA proprietary driver is needed for the real NVML backend.

## Build

Run these commands from the repository root:

```bash
cargo build -p nvidia-control-daemon
cmake -S ui -B build/ui -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build/ui --parallel
ctest --test-dir build/ui --output-on-failure
```

## Run

Recommended: from the repository root, run `bash scripts/install-user.sh`, then launch **Nvidia-Control** from KDE. The installer registers a D-Bus activated systemd user service so no terminal needs to remain open. See [ACTIVATION.md](../docs/ACTIVATION.md). Stop any manually running daemon first. No sudo, no login-time enabling.

For development without installing, use two terminals.

Terminal 1 (long-running, session D-Bus):

```bash
cargo run -p nvidia-control-daemon -- --session
```

Terminal 2 (GUI):

```bash
./build/ui/nvidia-control
```

For a GPU-less development machine use `NVIDIA_CONTROL_BACKEND=mock cargo run -p nvidia-control-daemon -- --session`. To test **two separate GPUs** without physical hardware, use `NVIDIA_CONTROL_BACKEND=mock-dual cargo run -p nvidia-control-daemon -- --session` (stop the existing systemd user service first; a single owner may claim the D-Bus name). In this mode the two devices have different UUIDs, limits and power/temperature values. The normal `nvml` backend is unaffected.

To print a one-time JSON diagnostic as before, run `cargo run -p nvidia-control-daemon` (or add `-- --once`).

The client requests only sensor telemetry once per second. Device enumeration, supported capabilities, power constraints and clocks are requested on startup/reconnection and once every 60 seconds, with another 60-second cache in the Rust daemon for multiple clients. Neither polling path blocks the Qt UI thread. A broken bus or incompatible JSON clears stale readings and retries discovery rather than presenting old data as live.

## D-Bus contract (v1, read only)

- bus: **session**
- name: `io.github.chmodmasx.NvidiaControl`
- object: `/io/github/chmodmasx/NvidiaControl`
- interface: `io.github.chmodmasx.NvidiaControl1`
- method `GetApiVersion() → u32`: returns `1`
- method `GetSnapshot() → string`: backward-compatible combined JSON array, including all device reports
- method `GetInventory() → string`: JSON array of static device details and operating limits
- method `GetTelemetry(index: u32, uuid: string) → string`: JSON object of live telemetry for one GPU

Example diagnostic:

```bash
busctl --user call io.github.chmodmasx.NvidiaControl \
  /io/github/chmodmasx/NvidiaControl \
  io.github.chmodmasx.NvidiaControl1 GetApiVersion
```

No privileged methods exist in this interface. Tuning operations require a separate Polkit-authorized service, **not** adding root privileges to this client.

## Current limitations

- The header shows a GPU selector only when inventory contains multiple cards. Selection uses the stable UUID and updates sensors, limits and capability metadata atomically. Changes clear the local history; reconnecting to the same UUID restores the preferred card. GPU reordering is handled by UUID, not index. The `GetInventory`/`GetTelemetry` wire format is unchanged.
- The **Historial** page plots GPU utilization, core temperature, power draw, VRAM usage, graphics clock and memory clock. Select 5, 15 or 60 minutes. Samples are retained only in RAM while the GUI runs, with a strict 60-minute/3601-sample cap. Missing measurements and gaps after disconnected periods are not joined. Moving to a different GPU resets the history; reconnecting to the same GPU keeps it.
- The optional per-user installer supports D-Bus on-demand startup via a systemd user unit; system-wide distribution packaging remains pending.
- Legacy application-clock table is read-only diagnostic information and does not imply supported overclock controls.
