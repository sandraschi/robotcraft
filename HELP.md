# RobotCraft — Operator & Developer Help Guide

**Version:** `0.3.0`  
**Control Port Label:** `14116` (status identifier; RobotCraft does not bind a listening TCP socket on `14116`)  
**Architecture:** 4-crate Rust workspace (`robotcraft-core`, `robotcraft-hal`, `robotcraft-mcp`, `robotcraft-app`)  
**License:** MIT OR Apache-2.0  

---

## 1. What RobotCraft Is & Does + Get Started

**RobotCraft** is the embodied intelligence studio and universal robotics operating workstation for the `sandraschi` fleet. Rather than reimplementing hardware drivers inside the studio, RobotCraft sits in front of sibling fleet servers and unifies:

- **Live & Fixture Telemetry** across 6 robot platforms (`saschibot`, `nori_a3`, `noetix_bumi`, `unitree_go2`, `yahboom_rover`, `craftarm_6dof`).
- **URDF Link Posing** (`robotcraft-core::urdf`) for Nori's expanded URDF or any custom URDF via `ROBOTCRAFT_NORI_URDF`.
- **Planar Forward Kinematics & Reachability Sphere** (`robotcraft-core::kinematics`).
- **Confirm-Gated Actuation & Simulation Bridges** (`robotcraft-hal::peers`) to `yahboom-mcp` (`:10892`), `vla-mcp` (`:11024`), and `isaac-mcp` (`:11049`).

### 5-Step First-Run Tour

1. **Launch & Smoke Check:** Start the desktop studio with `.\start.ps1` (or `cargo run -p robotcraft-app -- --smoke` for a headless `Command::Status` check).
2. **Probe Fleet Peers:** Open the **Yahboom Deck** or **Fleet Status** workspace (or run `robotcraft-cli peers` / MCP `robot_peers`) to check reachability of `yahboom-mcp`, `vla-mcp`, and `isaac-mcp` without moving hardware.
3. **Select a Robot & Pose URDF:** Use the **Robots** menu to pick `NoriRobotics A3` or `Yahboom ROSMASTER`. In the 3D Viewport, drag joint sliders to re-pose Nori's URDF link frames, or inspect CAD/BOM plans in **CAD Plans & BOM**.
4. **Validate Kinematics & Dry-Run Teleop:** In **Kinematics Studio**, test planar forward kinematics and target reachability (`0.10 m` to `0.85 m`). In **Teleop Joystick** or **Yahboom Deck**, leave **Confirm** unchecked first to verify the `confirm_required` gate.
5. **Automate via CLI or MCP:** Run `robotcraft-cli --mcp` for stdio JSON-RPC `2024-11-05` integration with Cursor, Claude, Antigravity, or OpenCode, or press `Ctrl+J` in the studio to open the local-first AI Chat Dock.

---

## 2. Functions

### Core Kinematics & URDF (`robotcraft-core`)

| Module / Function | Behavior & Exact Bounds |
|---|---|
| `urdf::load_urdf_file` / `pose_links` | Parses `<robot>` XML via `roxmltree`, keeps `revolute`, `continuous`, and `prismatic` joints, walks from `base_footprint` or `base_link` (512-node cycle guard), and clamps joint angles to `[lower, upper]` (or `[-pi, pi]` for `continuous`/unbounded joints). Default path: `D:\Dev\repos\norirobotics-mcp\models\nori_description\urdf\nori.expanded.urdf` (override with `ROBOTCRAFT_NORI_URDF`). Mesh STLs/GLBs are **not** loaded; mesh-only links fall back to `<collision>` boxes or a `4 cm` (`0.04 m`) cube. |
| `KinematicsSolver::forward_kinematics` | Planar 4-link forward kinematics starting at base height `z = 0.20 m` with link lengths `[0.25, 0.25, 0.15, 0.10]` m. Joint 0 applies base yaw (`0.05 m` offset); joints 1..3 accumulate pitch in `(x, z)`. |
| `KinematicsSolver::validate_target_pose` | Evaluates Euclidean distance `d = sqrt(x^2 + y^2 + z^2)`. Rejects `d < 0.10 m` (self-collision zone) and `d > 0.85 m` (maximum arm reach). |
| Hardcoded Kinematic Models | `Yahboom-ROSMASTER-X3` (4-DOF), `CraftArm-6DOF` (6-DOF), `Saschibot-Mobile-P6` (14-DOF), `NoriRobotics A3` (16-DOF), and `Noetix Bumi` (21-DOF). |
| `Command::dispatch` | Single async command bus shared by `robotcraft-app`, `robotcraft-cli`, and `robotcraft-mcp`. |

### Fleet HTTP Peers (`robotcraft-hal::peers`)

| Peer | Default URL | Env Override | Endpoints Called |
|---|---|---|---|
| `yahboom-mcp` | `http://127.0.0.1:10892` | `ROBOTCRAFT_YAHBOOM_URL` | `GET /api/v1/health`, `GET /api/v1/telemetry`, `POST /api/v1/control/tool`, `POST /api/v1/control/move`, `POST /api/v1/stop_all` |
| `vla-mcp` | `http://127.0.0.1:11024` | `ROBOTCRAFT_VLA_URL` | `GET /api/health`, `GET /api/v1/status`, `GET /api/v1/tools`, `GET /api/v1/fleet`, `GET /api/v1/dataset/episodes`, `POST /api/v1/control/{tool}` |
| `isaac-mcp` | `http://127.0.0.1:11049` | `ROBOTCRAFT_ISAAC_URL` | `GET /api/health`, `GET /api/scenes`, `POST /api/jobs/start` |

Set any `ROBOTCRAFT_*_URL=off` (or empty string) to disable that peer immediately.

### Live Telemetry vs. Studio Fixtures

- **Live Telemetry:** Only `yahboom_rover` (aliases: `yahboom`, `raspbot`, `boomy`, `rosmaster`) queries `yahboom-mcp`. Responses are live **only** when `yahboom-mcp` returns `"status": "live"` (`"live": true`, `"source": "yahboom-mcp"`).
- **Studio Fixtures:** `saschibot`, `nori_a3`, `noetix_bumi`, `unitree_go2`, `craftarm_6dof`, and `yahboom_rover` when the bridge or server is offline return `"live": false` and `"source": "studio-fixture"`.

---

## 3. Usage (GUI, CLI & MCP)

### Desktop GUI (`robotcraft-app`)

- **Workspaces:** 3D View, AI Motion Workflows, CAD Plans & BOM, Live Telemetry, Teleop Joystick, Kinematics Studio, Fleet Status, Yahboom Deck, and VLA Studio.
- **Keyboard Accelerators:**
  - `Ctrl+Shift+P` or `Ctrl+K` — Open **Command Palette**
  - `Ctrl+/` or `F1` — Open **Keyboard Shortcuts**
  - `Ctrl+J` — Toggle local-first **AI Chat Dock**
- **Headless Smoke Test:** `robotcraft.exe --smoke` runs `Command::Status` and exits without creating a window.

### CLI (`robotcraft-cli`)

Every CLI subcommand prints JSON to stdout:

```powershell
robotcraft-cli status
robotcraft-cli robots
robotcraft-cli telemetry yahboom_rover
robotcraft-cli peers
robotcraft-cli teleop --robot yahboom_rover --x 0.2 --confirm
robotcraft-cli yahboom read_all
robotcraft-cli yahboom forward --confirm
robotcraft-cli fk 0.0 0.35 1.20 -0.45
robotcraft-cli urdf
robotcraft-cli vla
robotcraft-cli vla-call vla_wall infer_prepare --task-hint "drive to the doorway and stop" --target raspbot
robotcraft-cli isaac
robotcraft-cli isaac-start --scene lab --confirm
robotcraft-cli estop --reason "operator halt"
```

### MCP `2024-11-05` Stdio Server (`robotcraft-cli --mcp`)

Stdout is 100% JSON-RPC; diagnostics go to stderr.

- **Tools (13):** `robot_status`, `robot_list`, `robot_telemetry`, `robot_teleop`, `robot_estop`, `robot_peers`, `yahboom_call`, `vla_snapshot`, `vla_call`, `urdf_pose`, `isaac_snapshot`, `isaac_start`, `robotcraft_skill_read`.
- **Resources:** `resources/list` and `resources/read` expose the bundled skills under `robotcraft://skills/<id>` (`text/markdown`).

---

## 4. Tips & Tricks

- **Non-Destructive Dry Runs:** Omit `--confirm` (CLI) or pass `confirm: false` (MCP) on `robot_teleop`, `yahboom_call`, `vla_call` (`download`), or `isaac_start`. RobotCraft returns `confirm_required: true` with the exact parsed payload without contacting hardware or starting jobs.
- **Skip Offline Peer Delays:** Each HTTP peer call has a `400 ms` connect timeout. When working offline or only testing URDF/kinematics, set `ROBOTCRAFT_YAHBOOM_URL=off`, `ROBOTCRAFT_VLA_URL=off`, and `ROBOTCRAFT_ISAAC_URL=off`.
- **Custom URDF Testing:** Point `ROBOTCRAFT_NORI_URDF` to any valid URDF file to inspect its link frames and joint limits in both `urdf_pose` and the 3D Viewport.
- **Fast Navigation:** Press `Ctrl+K` to jump to any workspace, probe peers, or open any Help Center tab without touching the mouse.

---

## 5. Common Pitfalls & Safety

1. **Actuator Tele-Op Confirm Gate:** `forward`, `backward`, `turn_left`, `turn_right`, `strafe_left`, `strafe_right`, and non-zero `TeleopTwist` commands for Yahboom **never** move hardware unless `confirm` is `true`.
2. **Emergency Stop Always Bypasses Confirm:** `robot_estop`, `robotcraft-cli estop`, the red **EMERGENCY STOP** button, `yahboom_call` with `stop`/`stop_all`, and a zero twist `(0, 0, 0)` immediately post `POST /api/v1/stop_all` to `yahboom-mcp` with no confirmation gate.
3. **VLA Whitelist & Weight Downloads:** Only whitelisted `(tool, operation)` pairs are forwarded (`vla_wall`, `vla_xvla`, `vla_weights`, `vla_world_model`). `vla_training` and `vla_agentic_workflow` are rejected locally. `vla_weights` `download` requires `confirm: true` (sends `X-VLA-Confirm: 1`).
4. **Joint Limits & Reachability Sphere:** URDF sliders clamp values to `[lower, upper]` (or `[-pi, pi]`). Target coordinates outside `0.10 m <= d <= 0.85 m` fail `validate_target_pose` to prevent self-collision or over-extension.
5. **Peer Timeouts & Fixture Confusion:** `ureq` uses `400 ms` connect and `1500 ms` read timeouts. Always check `"live": true` and `"source": "yahboom-mcp"` before assuming telemetry comes from physical hardware; `"source": "studio-fixture"` is simulated fallback data.
6. **Port `14116`:** `CONTROL_PORT = 14116` is a status label only—do not expect a REST server listening on `localhost:14116`.

---

## 6. Planned Extensions

Tracked in [TIERED_PLAN.md](docs/TIERED_PLAN.md):

- Numerical Inverse Kinematics (IK) solver and RRT* collision-free path planning with velocity limits (replacing canned AI workflow keyframes).
- Native Gazebo and MuJoCo simulation bridges (beyond `isaac-mcp` health/scene/start).
- STL/GLB mesh rendering in the 3D viewport (currently box/frame fallbacks) and `web/three_viewer.html` URDF loading.
- Full 8-menu Storytold Craft bar (`File`, `Edit`, `View`, `Insert`, `Modify`, `Commands`, `Window`, `Help`) across all workspaces.
- Workspace-wide `#![forbid(unsafe_code)]` attribute and Tofu Guard font-coverage verification.

---

## 7. Other Crafts & Fleet Ecosystem

| Sibling App / Repo | Role & Handoff | Boundary |
|---|---|---|
| `yahboom-mcp` (`:10892`) | Owns physical Raspbot / ROSMASTER X3 serial/ROS bridge, telemetry, and `stop_all`. | RobotCraft sends HTTP control/stop; never talks to serial directly. |
| `vla-mcp` (`:11024`) | Owns Wall-OSS, X-VLA, weight catalog, and dataset episodes. | RobotCraft reads snapshots & whitelisted prepare/download calls only. |
| `isaac-mcp` (`:11049`) | Owns Isaac Sim health, scene catalog, and headless job execution. | RobotCraft probes scenes and starts confirmed jobs; not a live 3D twin. |
| `norirobotics-mcp` | Provides Nori A3 `nori.expanded.urdf`, STEP/STL printables, and GLB assets. | Read-only URDF consumption via `ROBOTCRAFT_NORI_URDF`. |
| `rust-saschirob-mcp` | Provides Saschibot P6 DXF/SVG/STL fabrication plans. | Referenced in `GetRobotCadPlan` metadata. |
| `databasecraft` | Provides `databasecraft-store` and `databasecraft-core` workspace crates. | Embedded in-memory store for `Command::dispatch`. |
| `cadcraft` / `celcraft` | Sibling Storytold Craft workstations (CAD fabrication / 2D cel animation). | Share the Craft Menu Bar, Help Center, and CLI/MCP `--smoke` standard. |
