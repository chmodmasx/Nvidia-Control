# UI module

The production UI is planned as Qt 6 + QML.

This directory is intentionally a placeholder during M0. We first stabilize the core model and daemon/IPC boundary so the UI cannot accidentally become the place where NVIDIA, Proton or compositor logic lives.

## Rules

The UI may:

- render state;
- request domain operations through IPC;
- keep presentation-only state;
- show backend health and feature availability.

The UI may not:

- load NVML/NvAPI/NVKMS directly;
- run package managers;
- invoke privileged tuning operations directly;
- parse `nvidia-smi`;
- edit Proton configuration files behind the daemon's back.

This separation lets us redesign or replace the UI without touching the hardware and gaming backends.
