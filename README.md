# RobotCraft

Embodied intelligence studio and robotics operating workstation uniting URDF kinematics, safety-gated teleoperation, VLA/Isaac peer bridges, and a stdio MCP server in pure Rust.

![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)
![Rust 1.90+](https://img.shields.io/badge/rust-1.90%2B-orange?logo=rust)
![MCP stdio](https://img.shields.io/badge/MCP-stdio-purple)
![ArtCraft harness](https://img.shields.io/badge/ArtCraft-harness-green)

## What You Can Do

**How it runs:** Native `egui` studio (`robotcraft-app`), CLI (`robotcraft-cli`), and pure-stdio MCP `2024-11-05` server (`robotcraft-cli --mcp`) sharing `robotcraft-core`. Live hardware and simulation policies stay in peer servers; no listening port is opened by the studio.

- **Nori URDF 3D Viewport & Kinematics:** Load `nori.expanded.urdf` (or `ROBOTCRAFT_NORI_URDF`), pose joint sliders in the local 3D viewport, and test planar forward kinematics and target reachability (`0.10 m` to `0.85 m`).
- **Safety-Gated Teleop & E-Stop:** Motion commands (`forward`, `backward`, `turn_*`, `strafe_*`) and Isaac job starts require explicit `confirm: true`; zero twists and `stop_all` / e-stop execute immediately without a confirm gate.
- **Fleet Peer Bridges:** Connect to `yahboom-mcp` (`:10892`), `loona-mcp` (`:11069`), `vla-mcp` (`:11024`), and `isaac-mcp` (`:11049`), or disable any peer via `ROBOTCRAFT_*_URL=off`.
- **Honest Labeled Fixtures:** Saschibot, Nori A3, Bumi, Unitree, and CraftArm telemetry is explicitly marked `source: studio-fixture`; Yahboom telemetry is `live` only when `yahboom-mcp` reports live hardware.
- **12 MCP Tools & Bundled Skills:** Drive everything from Claude or Cursor via 12 MCP tools (`robot_status`, `robot_teleop`, `urdf_pose`, `vla_call`, `isaac_start`, etc.) and read bundled skills via `resources/list` / `resources/read` (`robotcraft://skills/<id>`).
- **AI Chat Dock & Help Center:** Press `Ctrl+J` for the local-first AI Chat Dock, `Ctrl+Shift+P` or `Ctrl+K` for the **Command Palette**, and use the Help Center for operator guides and shortcuts.

## Install

1. Download **`robotcraft-0.3.0-setup.exe`** from the [latest release](https://github.com/sandraschi/robotcraft/releases/latest).
2. Run the installer (if Windows SmartScreen appears on an unsigned build, choose **More info > Run anyway**).
3. Launch **RobotCraft** from the Start menu.

**Only want the MCP tools in Claude Desktop?** Download `robotcraft-0.3.0.mcpb` from the same release and open it with Claude Desktop (this registers the MCP server only and does **not** install the desktop app):

> Install the robotcraft MCP extension: download `robotcraft-0.3.0.mcpb` from https://github.com/sandraschi/robotcraft/releases/latest and open it with Claude Desktop.

**From a local checkout (Windows app, CLI, and MCP bundle):**

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install-craft.ps1
```

For source builds, peer URL overrides, and CLI examples, see [HELP.md](HELP.md) and [docs/](docs/).

## Example Prompts

> "List all configured robots and check which fleet peers (`yahboom-mcp`, `vla-mcp`, `isaac-mcp`) are online."

> "Pose the Nori URDF joints in `urdf_pose` and verify target reachability before sending a confirmed teleop move."

> "Prepare a `vla_wall` inference plan for `raspbot` with task hint 'drive to the doorway and stop'."

## Workspace Layout

```text
robotcraft/
|-- crates/robotcraft-hal/    # HTTP peer clients (Yahboom, Loona, VLA, Isaac) + labeled fixtures
|-- crates/robotcraft-core/   # Command::dispatch, URDF loader, planar FK, reachability, skills
|-- crates/robotcraft-mcp/    # CLI (robotcraft-cli) + MCP 2024-11-05 stdio server (12 tools)
|-- crates/robotcraft-app/    # egui studio: local 3D viewport, Yahboom deck, VLA studio, Help UI
`-- docs/                     # Specifications and architecture notes
```

## Documentation

| Doc | Contents |
|-----|----------|
| [Operator & Developer Help](HELP.md) | 5-step tour, peer URLs, confirm gates, CLI/MCP usage, shortcuts |
| [Docs Folder](docs/) | Architecture notes, CAD/BOM plans, and peer integration docs |

## Requirements

- Windows 10/11 64-bit
- Optional: running peer servers (`yahboom-mcp`, `loona-mcp`, `vla-mcp`, `isaac-mcp`) for live hardware or simulation

## License

MIT OR Apache-2.0. Adapted from the Storytold ArtCraft pattern ([storytold/artcraft](https://github.com/storytold/artcraft)).
