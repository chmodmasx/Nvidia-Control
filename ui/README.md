# Nvidia-Control Qt 6 / QML frontend

A minimal native GUI for the existing Rust/NVML backend. The Qt frontend **never** loads NVML or executes privileged commands: it reads the read-only session D-Bus API asynchronously.

## Prerequisites (Ubuntu/Kubuntu)

```bash
sudo apt install cmake ninja-build g++ qt6-base-dev qt6-declarative-dev \
  qml6-module-qtquick qml6-module-qtquick-window \
  qml6-module-qtquick-controls qml6-module-qtquick-layouts
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

The client reconnects automatically on its next one-second poll if the daemon restarts. A broken bus or incompatible JSON clears stale readings and displays an error rather than pretending telemetry remains live.

## D-Bus contract (v1, read only)

- bus: **session**
- name: `io.github.chmodmasx.NvidiaControl`
- object: `/io/github/chmodmasx/NvidiaControl`
- interface: `io.github.chmodmasx.NvidiaControl1`
- method `GetApiVersion() → u32`: returns `1`
- method `GetSnapshot() → string`: JSON array of device reports, each with `backend`, `device`, `telemetry`, `operating_limits`

Example diagnostic:

```bash
busctl --user call io.github.chmodmasx.NvidiaControl \
  /io/github/chmodmasx/NvidiaControl \
  io.github.chmodmasx.NvidiaControl1 GetApiVersion
```

No privileged methods exist in this interface. Tuning operations require a separate Polkit-authorized service, **not** adding root privileges to this client.

## Current limitations

- Dashboard initially displays the first GPU. The D-Bus payload already includes all enumerated devices; GPU selection can be added without changing the wire format.
- No telemetry history/graphs yet. The frontend updates current values at one-second intervals.
- No D-Bus activation/systemd user unit packaging yet; start the daemon manually.
- Legacy application-clock table is read-only diagnostic information and does not imply supported overclock controls.
