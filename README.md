# Nvidia-Control

> Unofficial NVIDIA control application for Linux. This project is not affiliated with or endorsed by NVIDIA Corporation.

Nvidia-Control aims to provide a modern, Wayland-first control application for NVIDIA GPUs on Linux, combining GPU telemetry and tuning, display integration, per-game profiles, Proton/DLSS controls, capture, and driver management behind a modular architecture.

## Project status

Early functional GUI prototype, with a Rust/NVML read-only daemon and a Qt 6/QML frontend.

The first real hardware backend is now present: NVML read-only telemetry. It dynamically loads NVIDIA's NVML library at runtime, so the project can still compile and be tested on CI machines without NVIDIA hardware.

## Design principles

- **Modular by default**: NVIDIA APIs, compositor integration, gaming integration, capture and package management live behind independent backends.
- **Wayland first**: display configuration is delegated to the compositor where appropriate instead of forcing legacy X11 mechanisms.
- **Least privilege**: the GUI never runs as root. Privileged operations belong in a narrowly scoped daemon and are authorized through Polkit.
- **Capability driven**: the UI exposes only features actually supported by the current GPU, driver, compositor and game.
- **No shell-command architecture**: stable APIs such as NVML, Vulkan, D-Bus and compositor protocols are preferred. Command-line fallbacks are isolated behind adapters when unavoidable.
- **Safe failure boundaries**: an experimental backend must be able to fail or be disabled without taking down the rest of the application.

## Architecture

```text
Qt 6 / QML application
        |
        | D-Bus
        v
nvidia-control-daemon
        |
        +-- control-core
        +-- backend-nvml        (read-only hardware path)
        +-- backend-nvapi       (experimental)
        +-- backend-nvkms       (experimental)
        +-- backend-display
        +-- backend-proton
        +-- backend-capture
        +-- backend-packages
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the module contracts and [docs/ROADMAP.md](docs/ROADMAP.md) for staged implementation.

## Current prototype

On a machine with the proprietary NVIDIA driver/NVML installed:

```bash
cargo run -p nvidia-control-daemon
```

The default backend is `nvml`. The command enumerates NVIDIA GPUs and prints a JSON report containing the stable device identity and the telemetry NVML exposes, including GPU temperature, GPU/memory utilization, VRAM usage, clocks, power draw and fan speed when available.

Metrics unsupported by a particular card/driver are represented as `null` instead of taking down the complete telemetry snapshot.

For development without NVIDIA hardware:

```bash
NVIDIA_CONTROL_BACKEND=mock cargo run -p nvidia-control-daemon
```

Read-only telemetry has been validated on physical hardware: RTX 3090 with NVIDIA driver 610.57.04. Cross-generation testing remains open. Unique hardware identifiers from the validation machine are not stored in this repository.

Capability semantics:

- `unknown` (default): the active backends have **not proven** feature availability. This does not imply incompatible hardware.
- `unsupported`: confirmed unsupported after appropriate checks.
- `read_only`: the corresponding getter succeeded; the application has **not** implemented or authorized changes.
- `read_write`: reserved for a future implemented and verified write operation.

NVML now checks read access to power-limit settings and graphics clocks. Fan control, V/F, compositor and gaming feature capabilities remain unknown until their adapters verify them. `memory_util_percent` is memory-controller activity, not percent of VRAM occupied.

The daemon also emits an `operating_limits` object separate from live telemetry. It reports power-limit current/default/enforced/min/max readings (W), maximum GPU/memory clocks (MHz), optional legacy supported application memory clocks and corresponding graphics clocks for the highest listed memory clock, and fan-setpoint min/max percentages where NVML supports them. These are diagnostic reads only. The legacy application-clock table is not a commitment to use the deprecated application-clock setters.

## Qt 6 / QML application

See [ui/README.md](ui/README.md) for build dependencies and instructions. The user-session D-Bus contract is documented in [docs/IPC.md](docs/IPC.md).

Build and run the daemon in **one terminal**:

```bash
cargo run -p nvidia-control-daemon -- --session
```

Build and launch the GUI in **another terminal**:

```bash
cmake -S ui -B build/ui -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build/ui --parallel
./build/ui/nvidia-control
```

The GUI shows GPU utilization, temperatures, power, clocks, memory and fan telemetry, plus a separate page for read-only hardware limits and capability states. It polls only live telemetry every second; the daemon caches GPU inventory and operating limits for 60 seconds, and the GUI refreshes those slowly changing values once per minute. The split D-Bus API preserves the existing `GetSnapshot` method. The GUI handles daemon disconnection. There is no write/control UI yet. The original `cargo run -p nvidia-control-daemon` one-off JSON mode remains supported.

The GUI has been validated on an RTX 3090 under KDE Plasma Wayland. It now includes a third **Historial** page with six time-series graphs and a 5 / 15 / 60 minute selector. History is limited to 3601 timestamped samples (up to 60 minutes), stored in memory only and not sent back to D-Bus. Missing readings and reconnection gaps are not drawn as continuous measurements.

Next steps: validate this history page on real hardware; add per-GPU selection and packaging before introducing authorized control operations.

## License

A license has not been selected yet. Do not copy third-party code into this repository until its license compatibility has been reviewed.
