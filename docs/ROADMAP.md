# Roadmap

The order is intentionally conservative. We first stabilize contracts and read-only paths, then introduce writes and experimental APIs.

## M0 — Architecture bootstrap

Status: **in progress**

- [x] repository initialized
- [x] Rust workspace
- [x] core GPU/capability model
- [x] backend abstraction
- [x] deterministic mock backend
- [x] runnable prototype output
- [x] CI for format, lint and tests
- [x] initial Qt/QML shell (hardware GUI validation pending)
- [x] versioned read-only session D-Bus contract
- [x] mock-backed D-Bus integration smoke test in CI

Exit condition: the mock backend can feed both daemon and UI without either depending on NVIDIA implementation details.

## M1 — Read-only NVIDIA hardware

Status: **in progress**

- [x] NVML runtime detection
- [x] enumerate real NVIDIA GPUs
- [x] GPU UUID/name
- [x] driver version
- [x] VBIOS version when exposed by NVML
- [x] temperature
- [x] GPU/memory utilization
- [x] VRAM usage
- [x] graphics/memory clocks
- [x] power telemetry
- [x] fan telemetry where available
- [x] unsupported metrics degrade independently
- [ ] hotspot temperature through extended NVIDIA APIs
- [ ] VRAM junction temperature through extended NVIDIA APIs
- [x] initial dynamic NVML read-access probes (power limit and clocks)
- [x] read current/default/enforced/min/max power limits
- [x] read maximum graphics/memory clocks
- [x] read available legacy application memory clocks and graphics clocks for highest memory clock where supported
- [x] read advertised fan-setpoint limits where supported
- [ ] full multi-backend capability probing
- [ ] test on multiple driver generations
- [x] validate NVML read-only telemetry on physical NVIDIA hardware (RTX 3090, driver 610.57.04)
- [x] validate NVML power/clock/fan limit getters on that RTX 3090 (see [HARDWARE_VALIDATION.md](HARDWARE_VALIDATION.md))
- [ ] validate other GPU families

Exit condition: normal monitoring works on NVIDIA hardware without root and without `nvidia-smi` parsing.

## M2 — Qt 6 / QML application

- [x] initial navigation shell (Overview / Details)
- [x] GPU overview for first GPU
- [x] live telemetry cards (1 s refresh)
- [ ] historical graphs
- [x] basic backend status and capability view
- [x] D-Bus connection/reconnection
- [x] stale-value clearing on disconnect
- [ ] GPU selection and physical-device UI validation

## M3 — Safe tuning

- [ ] power-limit read/write
- [ ] clocks
- [ ] fan controls
- [ ] persistent profiles
- [ ] Polkit authorization
- [ ] validation and safe ranges
- [ ] automatic rollback on rejected settings

## M4 — Experimental NVIDIA controls

- [ ] isolate `libnvidia-api.so` adapter
- [ ] V/F curve discovery
- [ ] V/F editor
- [ ] Digital Vibrance/NVKMS research
- [ ] driver compatibility matrix
- [ ] explicit experimental warnings

Experimental features must never be prerequisites for stable features.

## M5 — Wayland display integration

- [ ] KDE/KWin adapter
- [ ] GNOME/Mutter adapter
- [ ] wlroots adapter strategy
- [ ] COSMIC adapter strategy
- [ ] VRR
- [ ] HDR
- [ ] resolution/refresh configuration
- [ ] multi-monitor capability model

## M6 — Gaming profiles

- [ ] shared game/profile model
- [ ] Steam discovery
- [ ] Proton runtime detection
- [ ] DXVK-NVAPI configuration
- [ ] NGX updater
- [ ] DLSS SR override
- [ ] Ray Reconstruction override
- [ ] Frame Generation override
- [ ] DLAA
- [ ] custom scaling
- [ ] Reflex
- [ ] Smooth Motion when supported
- [ ] Heroic
- [ ] Lutris
- [ ] GOG

## M7 — Capture

- [ ] PipeWire capture
- [ ] NVENC encoder backend
- [ ] video recording
- [ ] screenshots
- [ ] Instant Replay ring buffer
- [ ] configurable audio sources

## M8 — Drivers and packaging

- [ ] APT adapter
- [ ] DNF adapter
- [ ] pacman adapter
- [ ] rpm-ostree/image-based strategy
- [ ] Nix/NixOS strategy
- [ ] package builds
- [ ] desktop integration
- [ ] update safety checks

## Non-goals

- running the GUI as root;
- depending on X11 as the primary architecture;
- parsing command output when a stable API is available;
- coupling game logic to one launcher;
- allowing undocumented NVIDIA APIs to become mandatory dependencies;
- using NVIDIA's generic `.run` installer as the normal driver update mechanism.
