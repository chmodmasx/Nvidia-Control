# Roadmap

The order is intentionally conservative. We first stabilize contracts and read-only paths, then introduce writes and experimental APIs.

## M0 — Architecture bootstrap

Status: **complete**

- [x] repository initialized
- [x] Rust workspace
- [x] core GPU/capability model
- [x] backend abstraction
- [x] deterministic mock backend
- [x] runnable prototype output
- [x] CI for format, lint and tests
- [x] initial Qt/QML shell (validated on RTX 3090 / KDE Plasma Wayland)
- [x] versioned read-only session D-Bus contract
- [x] mock-backed D-Bus integration smoke test in CI
- [x] split D-Bus inventory/telemetry API (backward compatible)
- [x] daemon-side 60 s inventory cache

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
- [x] live history graphs (GPU utilization, temperature, power, VRAM, core/memory clocks)
- [x] select 5 / 15 / 60 minute range
- [x] bounded in-memory retention with gap-aware plotting
- [x] frontend history-store unit tests in CI
- [ ] persistent historical data (optional future feature)
- [x] basic backend status and capability view
- [x] D-Bus connection/reconnection
- [x] separate 1 s telemetry and 60 s inventory polling
- [x] preserve selected GPU by UUID on inventory refresh
- [x] stale-value clearing on disconnect
- [x] UUID-based GPU selector in Qt (shown for inventories with 2+ GPUs)
- [x] multi-GPU selection and reordering tests (mock-dual + Qt catalogue)
- [x] dual mock inventory visually verified in KDE Plasma (two GPUs shown in selector; 2026-10-08)
- [ ] multi-GPU visual validation on physical hardware
- [x] GUI validated on physical RTX 3090 / KDE Plasma Wayland
- [x] optional per-user D-Bus activation and systemd user service registration
- [x] per-user KDE desktop launcher and uninstall script
- [x] on-demand D-Bus/systemd user activation validated on physical KDE Plasma Wayland (RTX 3090, driver 610.57.04; 2026-10-08)

## M3 — Safe tuning

**Next priority:** detect actual write support separately from NVML read capabilities, then implement a narrowly scoped, authorized power-limit backend with limits validation, readback and safe restoration. Never infer write permission from readable limits.

- [x] independent pure Rust power-change validation and mock-tested transaction engine
- [x] NVML one-shot privileged power helper (root-only write command, read-only inspect)
- [x] separate Polkit action and opt-in root-owned installation workflow
- [x] stale request, bound, factory ceiling, readback and best-effort rollback tests
- [ ] validate read-only inspect on physical NVIDIA GPU
- [ ] validate privileged apply and restoration on test hardware (explicit approval)
- [ ] Qt authenticated apply/revert flow and daemon inventory refresh
- [ ] power-limit UI write integration (not enabled yet)
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
- [x] development-stage per-user desktop entry and D-Bus activation
- [ ] distribution-ready desktop integration
- [ ] update safety checks

## Non-goals

- running the GUI as root;
- depending on X11 as the primary architecture;
- parsing command output when a stable API is available;
- coupling game logic to one launcher;
- allowing undocumented NVIDIA APIs to become mandatory dependencies;
- using NVIDIA's generic `.run` installer as the normal driver update mechanism.
