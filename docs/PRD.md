# RobotCraft — Product Requirements Document (PRD)

**Document Identifier:** `CRAFT-PRD-2026-ROBOT-001`  
**Version:** 1.0.0  
**Status:** Active  
**Domain:** Embodied Intelligence, Kinematics, URDF/SDF Robotics Studio  
**Standard Port:** 14116  
**Parent Standard:** [`RUST_ARTCRAFT_STANDARD.md`](../../mcp-central-docs/standards/RUST_ARTCRAFT_STANDARD.md)  
**Menu Bar Standard:** [`CRAFT_MENU_BAR_STANDARD.md`](../../mcp-central-docs/standards/rust-artcraft/CRAFT_MENU_BAR_STANDARD.md)  

---

## 1. Executive Summary & Problem Statement

Robotics development currently suffers from extreme fragmentation: kinematics tools, ROS topic bridges, physics simulators (Gazebo, MuJoCo, Isaac Sim), and hardware drivers (Unitree quadrupeds, Yahboom rovers, LimX bipeds, 6-DOF arms) operate as detached silos.

**The RobotCraft Solution:**
RobotCraft consolidates physical hardware drivers, simulators, and kinematic mathematics into a single unified robotics workstation, CLI, and FastMCP stdio server:
- Parses URDF and SDF robot kinematic models into memory.
- Forward and inverse kinematics (IK) solvers for 6-DOF industrial arms and multi-legged robots.
- Hardware simulation and digital twin bridges (Gazebo, MuJoCo, Isaac Sim) with demo oracle fallbacks.
- Hardware adapters (Unitree, Yahboom, LimX, ROS 2 DDS).
- Safety confirm-gates preventing unintended real-world actuator motion.

---

## 2. Core Functional Requirements

1. **Kinematics & Planning Engine (`robotcraft-core`):**
   - URDF / SDF robot tree parser with link transforms and joint limits.
   - Analytical and numerical Inverse Kinematics (IK) solvers.
   - Obstacle collision avoidance and path trajectory planning (RRT*).
2. **Simulation & Digital Twins:**
   - Unified simulator bridge interface (MuJoCo, Gazebo, Isaac Sim).
   - Headless test harnesses with deterministic kinematic fixtures.
3. **Hardware Adapters & Safety Fences:**
   - Safety confirm-gates for physical actuators: simulated-only in CI, explicit confirmation required before hardware motion.
4. **Universal Command Bus (`Command::dispatch`):**
   - 100% of CLI, MCP, and UI actions route through `robotcraft-core::dispatch`.

---

## 3. UI/UX Architecture & Menu Bar Standard

In accordance with [`CRAFT_MENU_BAR_STANDARD.md`](../../mcp-central-docs/standards/rust-artcraft/CRAFT_MENU_BAR_STANDARD.md):
- **Pattern 2 (Viewport-Centric Robotics Workstation):** Persistent 3D robot model viewport (wgpu / egui) with joint angle sliders, trajectory visualizers, and sensor telemetry monitors. Top quick button row acts as accelerators into the `View` menu. To add a new robot actuator or sensor view:
  1. Register a visibility toggle in the `View` or `Window` menu.
  2. Add an entry in the sidebar Robot / Kinematics toolbox.
  3. Wire the action directly into the universal `Command::dispatch` bus.
- **Top Menu Bar Hierarchy:**
  - `File`: New Scene (`Ctrl+N`), Open URDF / SDF Robot (`Ctrl+O`), Open Recent ▸, Save Configuration (`Ctrl+S`), Export Trajectory (`Ctrl+E`), Close (`Ctrl+W`), Quit (`Ctrl+Q`).
  - `Edit`: Undo (`Ctrl+Z`), Redo (`Ctrl+Y`), Cut, Copy, Paste, Select All, Preferences (`Ctrl+,`).
  - `View`: 3D Robot Viewport, Joint Angle HUD, Kinematics Hierarchy, Sensor Telemetry, Simulator Bridge Status, Theme (`Dark`/`Light`).
  - `Insert`: Add Robot Link, Add Joint Constraint, Insert Obstacle / Payload, Add Goal Waypoint.
  - `Modify`: Adjust Joint Angles, Tune PID Controllers, Set Velocity Ceilings, Update Home Position.
  - `Commands`: Solve Inverse Kinematics, Plan Collision-Free Path, Step Simulation, Probe Hardware Adapters, Emergency Stop (`Space`).
  - `Window`: Reset Layout, 3D Viewport, Joint Control Drawer, Telemetry Terminal.
  - `Help` (Mandatory 5-Dialog Suite): `Command Palette...` (`Ctrl+K`), `Keyboard Shortcuts...` (`F1`), `Project Files & Docs...`, `Licenses & Notices...`, `About RobotCraft`.
- **Behavioral Mechanics:** Smooth `switch_on_hover` between top-level menus, zero-unsafe immediate-mode rendering, and headless `--smoke` parity.
