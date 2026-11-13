# RobotCraft — Product Requirements Document (PRD)

**Document Identifier:** `CRAFT-PRD-2026-ROBOT-001`  
**Version:** 1.2.0  
**Status:** Active  
**App version:** 0.3.0  
**Domain:** Embodied Intelligence, Kinematics, URDF/SDF Robotics Studio  
**Standard Port:** 14116  
**Parent Standard:** [`RUST_ARTCRAFT_STANDARD.md`](../../mcp-central-docs/standards/RUST_ARTCRAFT_STANDARD.md)  
**Menu Bar Standard:** [`CRAFT_MENU_BAR_STANDARD.md`](../../mcp-central-docs/standards/rust-artcraft/CRAFT_MENU_BAR_STANDARD.md)  

---

## 1. Executive Summary & Problem Statement

Robotics development currently suffers from extreme fragmentation: kinematics tools, ROS topic bridges, physics simulators (Gazebo, MuJoCo, Isaac Sim), and hardware drivers (Unitree quadrupeds, Yahboom rovers, LimX bipeds, 6-DOF arms) operate as detached silos.

**The RobotCraft Solution (shipped in 0.3.0):**
RobotCraft is the workstation in front of the fleet, not a second driver for every robot.
- Command bus for CLI, stdio MCP, and the egui studio.
- HTTP peers: `yahboom-mcp` (`:10892`), `vla-mcp` (`:11024`), `isaac-mcp` (`:11049`).
- Confirm gate before Yahboom motion, a VLA weight download, and an Isaac job start.
- Nori URDF loaded from disk (`nori.expanded.urdf`, or `ROBOTCRAFT_NORI_URDF`). The 3D view and joint sliders use that pose. Meshes are not loaded.
- VLA studio lists dataset episodes and marks rows whose source matches the selected robot.
- Studio fixtures for Saschibot, Bumi, Unitree, CraftArm, and for Nori when the URDF file is missing. Fixtures are labeled `studio-fixture`.

Numerical IK, RRT*, Gazebo, and MuJoCo are not in this build. Isaac is a probe plus a confirmed job start, not a rendered twin.

---

## 2. Core Functional Requirements

1. **Kinematics (`robotcraft-core`):**
   - URDF file loader in `urdf.rs`. Default file is the Nori expanded URDF. Link frames come from joint origins plus the slider angles.
   - Hardcoded joint tables remain for CraftArm, Saschibot, Bumi, and the Yahboom rover.
   - Planar forward kinematics and a reach sphere (`0.10 m` to `0.85 m`) remain for the kinematics studio.
   - Numerical IK and RRT* are not implemented.
2. **Peers (`robotcraft-hal::peers`):**
   - `yahboom-mcp` health, telemetry, tool, move, and stop.
   - `vla-mcp` status, tool list, fleet bridge, episode list, and the whitelist in the README.
   - `isaac-mcp` health, scene list, and `POST /api/jobs/start`.
   - `ROBOTCRAFT_YAHBOOM_URL`, `ROBOTCRAFT_VLA_URL`, `ROBOTCRAFT_ISAAC_URL`, `ROBOTCRAFT_NORI_URDF`. `off` disables a peer.
3. **Safety fences:**
   - Yahboom motion verbs, VLA `download`, and Isaac job start require `confirm`.
   - Stop, zero twist, and e-stop may post `POST /api/v1/stop_all` with no confirm flag.
   - Tests cover the hold paths. The Nori URDF test reads the file on this machine. Peer hold tests do not call the servers.
4. **Command bus:**
   - CLI, MCP `tools/call`, and the studio go through `robotcraft_core::dispatch`.

---

## 3. UI/UX Architecture & Menu Bar Standard

In accordance with [`CRAFT_MENU_BAR_STANDARD.md`](../../mcp-central-docs/standards/rust-artcraft/CRAFT_MENU_BAR_STANDARD.md):
- **Pattern 2 (Viewport-Centric Robotics Workstation):** Persistent 3D robot model viewport (wgpu / egui) with joint angle sliders, trajectory visualizers, and sensor telemetry monitors. Top quick button row acts as accelerators into the `View` menu. To add a new robot actuator or sensor view:
  1. Register a visibility toggle in the `View` or `Window` menu.
  2. Add an entry in the sidebar Robot / Kinematics toolbox.
  3. Wire the action directly into the universal `Command::dispatch` bus.
- **Target menu bar (not shipped).** The list below is the craft standard. The running app uses the shorter bar described at the end of this section.
- **Top Menu Bar Hierarchy:**
  - `File`: New Scene (`Ctrl+N`), Open URDF / SDF Robot (`Ctrl+O`), Open Recent ▸, Save Configuration (`Ctrl+S`), Export Trajectory (`Ctrl+E`), Close (`Ctrl+W`), Quit (`Ctrl+Q`).
  - `Edit`: Undo (`Ctrl+Z`), Redo (`Ctrl+Y`), Cut, Copy, Paste, Select All, Preferences (`Ctrl+,`).
  - `View`: 3D Robot Viewport, Joint Angle HUD, Kinematics Hierarchy, Sensor Telemetry, Simulator Bridge Status, Theme (`Dark`/`Light`).
  - `Insert`: Add Robot Link, Add Joint Constraint, Insert Obstacle / Payload, Add Goal Waypoint.
  - `Modify`: Adjust Joint Angles, Tune PID Controllers, Set Velocity Ceilings, Update Home Position.
  - `Commands`: Solve Inverse Kinematics, Plan Collision-Free Path, Step Simulation, Probe Hardware Adapters, Emergency Stop (`Space`).
  - `Window`: Reset Layout, 3D Viewport, Joint Control Drawer, Telemetry Terminal.
  - `Help` (Mandatory 5-Dialog Suite): `Command Palette...` (`Ctrl+K`), `Keyboard Shortcuts...` (`F1`), `Project Files & Docs...`, `Licenses & Notices...`, `About RobotCraft`.
- **Shipped chrome (0.3.0):** File, Robots, View, Security, Help (About only). Sidebar workspaces include the 3D viewport, telemetry, teleop, kinematics, fleet list, Yahboom deck, and VLA studio. Nori in the viewport is the URDF pose, with joint sliders under the view. The fleet page probes Isaac. The 8-menu bar and the five-dialog Help suite are not built.
- **Behavioral Mechanics:** Dark theme by default. `--smoke` on the desktop binary runs `Command::Status` and exits without a window. The 3D viewport is a local drawing.

---

## 4. Operator notes

See the README for CLI verbs, MCP tool names, and the confirm rules. A green telemetry card is live only when `source` is `yahboom-mcp` and `live` is true. `14116` is a status label. The process does not listen on it.
