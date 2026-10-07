# Nvidia-Control

> Unofficial NVIDIA control application for Linux. This project is not affiliated with or endorsed by NVIDIA Corporation.

Nvidia-Control aims to provide a modern, Wayland-first control application for NVIDIA GPUs on Linux, combining GPU telemetry and tuning, display integration, per-game profiles, Proton/DLSS controls, capture, and driver management behind a modular architecture.

## Project status

Early prototype / architecture bootstrap.

The first milestone is intentionally small: prove the module boundaries and hardware capability model before adding privileged writes or a large GUI.

## Design principles

- **Modular by default**: NVIDIA APIs, compositor integration, gaming integration, capture and package management live behind independent backends.
- **Wayland first**: display configuration is delegated to the compositor where appropriate instead of forcing legacy X11 mechanisms.
- **Least privilege**: the GUI never runs as root. Privileged operations belong in a narrowly scoped daemon and are authorized through Polkit.
- **Capability driven**: the UI exposes only features actually supported by the current GPU, driver, compositor and game.
- **No shell-command architecture**: stable APIs such as NVML, Vulkan, D-Bus and compositor protocols are preferred. Command-line fallbacks are isolated behind adapters when unavoidable.
- **Safe failure boundaries**: an experimental backend must be able to fail or be disabled without taking down the rest of the application.

## Planned architecture

```text
Qt 6 / QML application
        |
        | D-Bus
        v
nvidia-control-daemon
        |
        +-- control-core
        +-- control-ipc
        +-- backend-nvml
        +-- backend-nvapi        (experimental)
        +-- backend-nvkms        (experimental)
        +-- backend-display
        +-- backend-proton
        +-- backend-capture
        +-- backend-packages
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the module contracts and [docs/ROADMAP.md](docs/ROADMAP.md) for staged implementation.

## Initial target

The first functional target is read-only and safe:

1. Detect NVIDIA GPUs.
2. Read capabilities and telemetry through NVML.
3. Expose a stable internal model.
4. Make the daemon/UI boundary work.
5. Add write operations only after the read path is tested.

Later milestones add power limits, clocks, fan curves, V/F control, display features, Proton/DLSS overrides, Reflex, Smooth Motion where supported, capture/Instant Replay and distribution-native driver management.

## License

A license has not been selected yet. Do not copy third-party code into this repository until its license compatibility has been reviewed.
