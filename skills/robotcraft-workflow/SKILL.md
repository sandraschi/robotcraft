---
name: robotcraft-workflow
description: End-to-end RobotCraft operator workflow for probing fleet peers, inspecting live vs fixture telemetry, posing URDFs, and dispatching confirm-gated teleoperation.
---

# RobotCraft Operator Workflow

Use this workflow when operating RobotCraft via MCP (`robotcraft-cli --mcp`), CLI (`robotcraft-cli`), or the desktop studio (`robotcraft.exe`).

## 1. Check Studio Status & Fleet Peers

1. Call `robot_status` to verify version (`0.3.0`), control port label (`14116`), and configured peer URLs (`yahboom_mcp`, `vla_mcp`, `isaac_mcp`).
2. Call `robot_peers` to probe `yahboom-mcp` (`:10892`), `vla-mcp` (`:11024`), and `isaac-mcp` (`:11049`) without moving any hardware.
3. If a peer is intentionally offline, set `ROBOTCRAFT_YAHBOOM_URL=off`, `ROBOTCRAFT_VLA_URL=off`, or `ROBOTCRAFT_ISAAC_URL=off` to skip the 400 ms connect timeout.

## 2. Inspect Supported Robots & Telemetry

1. Call `robot_list` to enumerate the 6 supported platforms (`saschibot`, `nori_a3`, `noetix_bumi`, `unitree_go2`, `yahboom_rover`, `craftarm_6dof`).
2. Call `robot_telemetry` with `robot_id`:
   - `yahboom_rover` queries `yahboom-mcp` (`GET /api/v1/telemetry`). Telemetry is live **only** when `"live": true` and `"source": "yahboom-mcp"`.
   - All other `robot_id` values—and `yahboom_rover` when the bridge is down—return `"live": false` and `"source": "studio-fixture"`.

## 3. Pose URDF & Inspect VLA / Isaac

- Call `urdf_pose` (empty `path` loads Nori's `nori.expanded.urdf` or `ROBOTCRAFT_NORI_URDF`) to compute link frames without touching hardware.
- Call `vla_snapshot` to inspect `vla-mcp` status, weight catalog, tools, fleet bridge, and recent dataset episodes.
- Call `isaac_snapshot` to inspect `isaac-mcp` health and available simulation scenes.

## 4. Confirmed Actuation & Emergency Stop

- Dry-run first: call `robot_teleop` or `yahboom_call` with `confirm: false` to verify the command payload (`confirm_required: true`).
- Execute: set `confirm: true` only after verifying workspace clearance.
- Halt immediately: call `robot_estop` with a `reason` (or send zero twist `0, 0, 0`). Both post `POST /api/v1/stop_all` to `yahboom-mcp` without requiring `confirm`.
