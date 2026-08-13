# Upstream provenance

This repository was forked from the CachyOS CHWD repository at release `1.23.0`:

- Upstream project: <https://github.com/CachyOS/chwd>
- Upstream tag: `1.23.0`
- Upstream commit: `d63f827751a568b115e924ecea602df2abd4f111`
- Upstream author recorded by Cargo: Vladislav Nepogodin `<nepogodin.vlad@gmail.com>`
- License: GNU General Public License v3.0 only (`GPL-3.0-only`, retained from upstream)

The fork retains the upstream Git history and `LICENSE`. No upstream endorsement is implied.

## Linxira changes (2026-08-13)

1. **Naming**: upstream `chwd` ("C" = CachyOS) is renamed for Linxira:
   - command `lhwd` (Linxira Hardware Detection) — full CHWD CLI
   - package / binary `linxira-hwd-detector` — no-argument read-only JSON report
2. **Dual binary**: `lhwd` (full CLI: `list` / `autoconfigure` / `install` / `remove`)
   and `linxira-hwd-detector` (fixed no-argument contract consumed by
   `linxira-hardware-driver-manager`). Any argument routes to the full CLI.
3. **No-argument JSON contract** (Linxira addition): emits a versioned, deterministic
   JSON hardware report (PCI/DMI/CPU evidence + stable profile IDs) with zero side effects.
4. **Driver profile data** is shipped at `/var/lib/chwd/db/<subdir>/profiles.toml`
   (upstream layout), installed by the package.
5. Everything else (hardware-ID matching, profile parsing, transaction engine, conflict
   resolution, i18n) is ported from CHWD 1.23.0 unchanged in behavior.

Upstream CachyOS additionally builds custom-optimized kernels for new hardware; Linxira
does **not** do this — it ships the generic `linux` / `linux-lts` kernels, so no kernel
build workflow is present in this fork.
