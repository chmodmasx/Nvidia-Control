# Feature matrix

This is the architectural target, not a claim that every row is implemented in the current prototype.

| Feature | Linux feasibility | Planned adapter |
|---|---|---|
| GPU identity | Yes | NVML |
| GPU/VRAM utilization | Yes | NVML |
| Temperatures | Yes | NVML / optional NVIDIA API extension |
| Clocks | Yes | NVML / NVIDIA API |
| Power telemetry/limit | Yes | NVML |
| Fan telemetry/control | Yes, hardware dependent | NVML / NVIDIA API |
| V/F curve | Experimental | NVIDIA API |
| Per-application profiles | Yes | profile service |
| Digital Vibrance on Wayland | Experimental/driver-specific | NVKMS adapter |
| VRR / G-SYNC | Yes | compositor adapter |
| HDR | Yes | compositor adapter |
| Monitor configuration | Yes | compositor adapter |
| Overlay | Yes | separate gaming/overlay component |
| Recording | Yes | PipeWire + NVENC |
| Instant Replay | Yes | PipeWire + NVENC ring buffer |
| Screenshots | Yes | portal/PipeWire |
| Driver updates | Yes | distribution package adapter |
| Steam/Proton discovery | Yes | gaming adapters |
| Heroic/Lutris discovery | Yes | gaming adapters |
| DLSS SR/RR overrides under Proton | Yes | DXVK-NVAPI / NGX |
| DLSS FG override under Proton | Yes where hardware/game supports it | DXVK-NVAPI / NGX |
| DLAA/custom DLSS scaling | Yes under supported Proton paths | DXVK-NVAPI |
| NVIDIA Reflex | Yes where runtime/game supports it | DXVK-NVAPI / Vulkan |
| Smooth Motion | Yes on supported GPUs/drivers | NVIDIA present layer |
| RTX HDR/Freestyle parity | Not a core target | — |
| Reflex Analyzer | Optional hardware-specific tooling | future/optional |

Every runtime feature must still pass capability detection before it is shown as available.
