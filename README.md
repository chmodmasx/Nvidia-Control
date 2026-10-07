# Nvidia-Control

> Unofficial NVIDIA control application for Linux. This project is not affiliated with or endorsed by NVIDIA Corporation.

Nvidia-Control aims to provide a modern, Wayland-first control application for NVIDIA GPUs on Linux, combining GPU telemetry and tuning, display integration, per-game profiles, Proton/DLSS controls, capture, and driver management behind a modular architecture.

## Project status

Early prototype / architecture bootstrap.

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

The next steps are physical-hardware validation, richer capability probing, stable daemon/UI IPC and the Qt/QML shell.

## License

A license has not been selected yet. Do not copy third-party code into this repository until its license compatibility has been reviewed.
