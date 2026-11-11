# RobotCraft — Tiered Execution Plan

**Parent Standard:** [`RUST_ARTCRAFT_STANDARD.md`](../../mcp-central-docs/standards/RUST_ARTCRAFT_STANDARD.md)  
**Menu Bar Standard:** [`CRAFT_MENU_BAR_STANDARD.md`](../../mcp-central-docs/standards/rust-artcraft/CRAFT_MENU_BAR_STANDARD.md)  
**Architecture Model:** 4-Tier ArtCraft Incremental Delivery  
**Status:** Active  

---

## Architecture Flow

```
Tier 1: Core Engine ──> Tier 2: MCP / CLI ──> Tier 3: Desktop UI ──> Tier 4: Menu Bar, Help & Packaging
(robotcraft-core)      (robotcraft-mcp)      (robotcraft-app)      (Hardened Workstation)
```

---

## Tier Specifications

### Tier 1 — Core Kinematics & Robotics Engine (`robotcraft-core`)
- [x] L1 `Command::dispatch` universal robotics execution bus.
- [x] URDF / SDF robot model tree parser.
- [x] Forward and Inverse Kinematics solvers.
- [x] Trajectory generator with joint velocity and acceleration limits.
- [x] Safety confirm-gates for physical hardware actuators.
- [x] `#![forbid(unsafe_code)]` and zero-panic clippy lints.

### Tier 2 — Headless Driver & FastMCP (`robotcraft-mcp`)
- [x] Unified CLI `robotcraft-cli` with `--json` output.
- [x] FastMCP stdio server communicating over JSON-RPC 2.0.
- [x] Simulator bridge interfaces (MuJoCo, Gazebo, Isaac Sim) with demo fallbacks.
- [x] Headless `--smoke` verification without window initialization.

### Tier 3 — Desktop Workstation UI (`robotcraft-app`)
- [x] Immediate-mode `egui` 0.37 / `wgpu` workstation.
- [x] 3D robot model interactive visualization with orbit camera controls.
- [x] Real-time joint angle sliders and end-effector pose indicators.

### Tier 4 — Craft Menu Bar Standard, Help Suite & Hardened Verification
- [ ] Universal 8-menu bar (`File`, `Edit`, `View`, `Insert`, `Modify`, `Commands`, `Window`, `Help`) with `switch_on_hover`.
- [ ] Mandatory 5-dialog Help suite (`Command Palette...`, `Keyboard Shortcuts...`, `Project Files & Docs...`, `Licenses & Notices...`, `About RobotCraft`).
- [ ] Fleet UI/UX Pattern 2 adherence: viewport-centric navigation, top button row mapped to View toggles, panel visibility managed via `View` / `Window`.
- [ ] Tofu Guard verification (100% font glyph coverage, zero tofu placeholder boxes).
- [ ] Headless `--smoke` parity validation.
- [ ] Standalone release binary packaging and NSIS installer.
