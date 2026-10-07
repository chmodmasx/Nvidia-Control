# Architecture

## Goal

Nvidia-Control must remain maintainable even when NVIDIA changes an undocumented API, a compositor changes its integration surface, or a distribution changes its package-management workflow.

The application therefore treats every external integration as a replaceable backend rather than allowing hardware- or desktop-specific code to spread through the UI and daemon.

## Process model

```text
+----------------------------+
| Qt 6 / QML GUI             |
| unprivileged               |
+-------------+--------------+
              |
              | stable D-Bus contract
              v
+-------------+--------------+
| nvidia-control-daemon      |
| policy + orchestration     |
+-------------+--------------+
              |
       +------+-------------------------------+
       |                                      |
       v                                      v
+------+-------+                      +-------+--------+
| safe/read    |                      | privileged     |
| backends     |                      | write adapters |
+--------------+                      +----------------+
```

The GUI is a client. It must not contain NVML, NVKMS, NVAPI, package-manager or Proton implementation details.

## Core contracts

`control-core` owns the domain model:

- GPU identity
- capability discovery
- telemetry
- feature availability
- errors
- backend traits

It does **not** know how those values are obtained.

A backend translates one external technology into the core model. This lets us replace or disable a backend without changing consumers.

## Backend boundaries

### backend-nvml

Primary supported NVIDIA hardware backend.

Responsibilities:

- enumerate GPUs
- stable identity/UUID
- temperature
- utilization
- VRAM usage
- clocks
- power telemetry
- supported stable tuning operations

Writes must be added one operation at a time and guarded by explicit capabilities.

### backend-nvapi

Experimental Linux NvAPI adapter.

Intended for functionality exposed by `libnvidia-api.so` that is not available through stable NVML, including V/F functionality where verified.

Rules:

- never make it a hard dependency
- runtime capability detection
- driver-version guards
- failures cannot disable NVML functionality
- no undocumented call is exposed directly to the UI

### backend-nvkms

Experimental display/NVIDIA-specific adapter.

Potential responsibilities include Digital Vibrance and NVIDIA-specific display controls that genuinely belong below the compositor.

It must not duplicate controls owned by the Wayland compositor.

### backend-display

Desktop/compositor adapters:

```text
backend-display
  kde/
  gnome/
  wlroots/
  cosmic/
```

Responsibilities may include resolution, refresh rate, HDR and VRR when those operations are compositor-owned.

### backend-proton

Gaming integration layer.

Responsibilities:

- discover Steam/Proton titles
- manage per-game launch environment
- DXVK-NVAPI settings
- NGX updater integration
- DLSS SR/RR/FG overrides
- DLAA
- custom scaling
- Reflex
- Smooth Motion capability/configuration where supported

The game-profile data model should not depend on Steam so Heroic, Lutris and other launchers can reuse it.

### backend-capture

Capture pipeline behind a common interface:

- PipeWire screen/game capture
- NVENC encode
- recording
- screenshots
- ring-buffer Instant Replay

### backend-packages

Distribution-specific driver/package management.

Adapters are isolated by distribution:

```text
apt
dnf
pacman
rpm-ostree
nix
```

The project must not install NVIDIA's `.run` package as a generic update mechanism.

## Privilege model

Read-only telemetry should remain unprivileged whenever the kernel/driver allows it.

Operations such as power limits, clocks, fan policies or package changes are routed through narrowly scoped daemon methods and authorized with Polkit.

The GUI itself never runs as root.

## Capability model

The UI must never infer support from a GPU marketing name alone.

Each feature reports an access level:

- `unsupported`
- `read_only`
- `read_write`
- `experimental_read_write`

The final capability is computed from the complete environment where relevant:

```text
GPU
+ driver
+ backend availability
+ compositor
+ game/runtime
= exposed capability
```

Example: Smooth Motion may exist in the installed driver but remain unavailable on unsupported GPU generations.

## Failure isolation

A failed experimental NvAPI probe must not prevent:

- GPU telemetry
- game discovery
- display management
- package management

Each backend should eventually expose its own health state so the GUI can explain a degraded feature instead of failing globally.

## Prototype contract

The current prototype intentionally implements only:

1. the core domain model;
2. a backend trait;
3. a deterministic mock backend;
4. a small executable that enumerates a device and reads telemetry.

The next implementation step is a read-only NVML backend. Privileged writes come later.
