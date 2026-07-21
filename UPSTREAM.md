# Upstream provenance

This repository was cloned from the CachyOS CHWD repository and forked from release `1.23.0`:

- Upstream project: <https://github.com/CachyOS/chwd>
- Upstream tag: `1.23.0`
- Upstream commit: `d63f827751a568b115e924ecea602df2abd4f111`
- Upstream author recorded by Cargo: Vladislav Nepogodin `<nepogodin.vlad@gmail.com>`
- License: GNU General Public License v3.0 only (`GPL-3.0-only`)

The fork retains the upstream Git history and `LICENSE`. Its hardware-ID matching loop is adapted
from CHWD 1.23.0 `src/data.rs`. The fork replaces the original CLI, package/profile parser, USB and
native-library integrations, command execution, state inspection, and all mutation paths with a
fixed read-only Linux evidence collector and a versioned JSON report. No upstream endorsement is
implied.
