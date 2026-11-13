# RobotCraft — Tiered Execution Plan

**Parent Standard:** [`RUST_ARTCRAFT_STANDARD.md`](../../mcp-central-docs/standards/RUST_ARTCRAFT_STANDARD.md)  
**Menu Bar Standard:** [`CRAFT_MENU_BAR_STANDARD.md`](../../mcp-central-docs/standards/rust-artcraft/CRAFT_MENU_BAR_STANDARD.md)  
**Architecture Model:** 4-Tier ArtCraft Incremental Delivery  
**Status:** Active (reconciled to app 0.3.0 on 2026-11-13)  

---

## Architecture Flow

```
Tier 1: Core Engine ──> Tier 2: MCP / CLI ──> Tier 3: Desktop UI ──> Tier 4: Menu Bar, Help & Packaging
(robotcraft-core)      (robotcraft-mcp)      (robotcraft-app)      (Hardened Workstation)
```

---

## Tier Specifications

### Tier 1 — Core (`robotcraft-core`)
- [x] `Command::dispatch` for CLI, MCP, and the studio.
- [x] Hardcoded joint tables (CraftArm, Saschibot, Nori A3, Bumi, Yahboom rover).
- [x] Planar forward kinematics and a reachability check. Tests in `crates/robotcraft-core/tests/dispatch_tests.rs`.
- [x] URDF file parser for the Nori expanded URDF (`urdf.rs`). SDF is not parsed. Meshes are not loaded.
- [ ] Numerical IK, RRT*, and a trajectory generator with velocity limits. AI workflow keyframes are canned JSON.
- [x] Confirm gate for Yahboom motion and VLA weight download.
- [ ] `#![forbid(unsafe_code)]`. Clippy denies `unwrap`, `expect`, `panic`, `todo`, and `unimplemented` on the crates. That is not a `forbid(unsafe_code)` attribute.

### Tier 2 — CLI, MCP, peers
- [x] `robotcraft-cli`. Output is JSON. There is no `--json` flag.
- [x] Stdio MCP `2024-11-05` (`--mcp`). Twelve tools, including `robot_peers`, `yahboom_call`, `vla_snapshot`, `vla_call`, `urdf_pose`, `isaac_snapshot`, `isaac_start`.
- [x] HTTP peers in `robotcraft-hal`: `yahboom-mcp` `:10892`, `vla-mcp` `:11024`, `isaac-mcp` `:11049`.
- [ ] MuJoCo and Gazebo clients. Isaac is health, scenes, and a confirmed job start.
- [x] Desktop `--smoke` runs status and exits. It does not open a window.

### Tier 3 — Desktop studio (`robotcraft-app`)
- [x] egui 0.37 workstation and an orbitable local viewport, including a Yahboom rover drawing.
- [x] Yahboom deck and VLA studio. VLA studio lists dataset episodes.
- [x] Teleop velocity sliders and a confirm checkbox.
- [x] Nori joint sliders drive URDF link frames (boxes, not STL meshes).

### Tier 4 — Craft Menu Bar Standard, Help Suite & Hardened Verification
- [ ] Universal 8-menu bar (`File`, `Edit`, `View`, `Insert`, `Modify`, `Commands`, `Window`, `Help`) with `switch_on_hover`.
- [ ] Mandatory 5-dialog Help suite (`Command Palette...`, `Keyboard Shortcuts...`, `Project Files & Docs...`, `Licenses & Notices...`, `About RobotCraft`).
- [ ] Fleet UI/UX Pattern 2 adherence: viewport-centric navigation, top button row mapped to View toggles, panel visibility managed via `View` / `Window`.
- [ ] Tofu Guard verification (100% font glyph coverage, zero tofu placeholder boxes).
- [ ] Headless `--smoke` parity validation.
- [ ] Standalone release binary packaging and NSIS installer.
