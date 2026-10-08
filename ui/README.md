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
```

## Run

Terminal 1 (long-running, session D-Bus):

```bash
cargo run -p nvidia-control-daemon -- --session
```

Terminal 2 (GUI):

```bash
./build/ui/nvidia-control
```

For a GPU-less development machine use `NVIDIA_CONTROL_BACKEND=mock cargo run -p nvidia-control-daemon -- --session`.

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

- Dashboard initially displays the first GPU. Inventory includes all enumerated devices; GPU selection can be added without changing the wire format.
- No telemetry history/graphs yet. The frontend updates current values at one-second intervals.
- No D-Bus activation/systemd user unit packaging yet; start the daemon manually.
- Legacy application-clock table is read-only diagnostic information and does not imply supported overclock controls.
