# RobotCraft

![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)
![Rust 1.90+](https://img.shields.io/badge/rust-1.90%2B-orange?logo=rust)
![MCP stdio](https://img.shields.io/badge/MCP-stdio-purple)
![ArtCraft harness](https://img.shields.io/badge/ArtCraft-harness-green)

**RobotCraft** is the embodied intelligence studio and universal robotics operating workstation for the sandraschi fleet. It consolidates hardware telemetry, forward/inverse kinematics solvers, URDF models, Gazebo simulation bridges, and physical actuator tele-op.

## Architecture

Version **0.3.0**. Four crates. The studio does not bind port 14116; that number is a control id in status JSON. Live hardware and policies stay in the peer servers.

| Crate | Role |
|---|---|
| `robotcraft-hal` | HTTP peers. Yahboom on `:10892`, VLA on `:11024`, Isaac on `:11049`. Fixture telemetry for robots with no peer. |
| `robotcraft-core` | `Command::dispatch`. URDF loader (Nori file), planar forward kinematics, reachability sphere. |
| `robotcraft-mcp` | Stdio MCP `2024-11-05` and `robotcraft-cli`. |
| `robotcraft-app` | egui studio: local 3D viewport, Yahboom deck, VLA studio. |

`web/three_viewer.html` is a separate browser viewport. It does not call the peers.

## Fleet peers

| Peer | Default | What RobotCraft calls |
|---|---|---|
| `yahboom-mcp` | `http://127.0.0.1:10892` | `GET /api/v1/health`, `GET /api/v1/telemetry`, `POST /api/v1/control/tool`, `POST /api/v1/control/move`, `POST /api/v1/stop_all` |
| `vla-mcp` | `http://127.0.0.1:11024` | `GET /api/health`, `GET /api/v1/status`, `GET /api/v1/tools`, `GET /api/v1/fleet`, `GET /api/v1/dataset/episodes`, `POST /api/v1/control/{tool}` |
| `isaac-mcp` | `http://127.0.0.1:11049` | `GET /api/health`, `GET /api/scenes`, `POST /api/jobs/start` (confirm) |

Disable a peer with `ROBOTCRAFT_YAHBOOM_URL=off`, `ROBOTCRAFT_VLA_URL=off`, or `ROBOTCRAFT_ISAAC_URL=off`. Any other value replaces the base URL (no trailing slash required).

## Nori URDF

`urdf` loads `D:\Dev\repos\norirobotics-mcp\models\nori_description\urdf\nori.expanded.urdf`, or `ROBOTCRAFT_NORI_URDF` when that is set. The studio draws those link frames when Nori is selected. Joint sliders re-pose the same file. Mesh STLs are not loaded. A link with only a mesh uses its collision box, or a 4 cm cube.

**Confirm.** `forward`, `backward`, `turn_left`, `turn_right`, `strafe_left`, and `strafe_right` return `confirm_required` and do not post until `confirm` is true. A zero twist and `stop` / `stop_all` / e-stop post `stop_all` without that flag. VLA `download` is the only weight call that sends `X-VLA-Confirm: 1`, and only when `confirm` is true. `vla_training` and `vla_agentic_workflow` are rejected. Isaac `POST /api/jobs/start` also requires `confirm`.

**Fixtures.** Saschibot, Nori A3, Bumi, Unitree, and CraftArm telemetry is `source: studio-fixture`. Yahboom telemetry is live only when `yahboom-mcp` returns `status: live`. If the server is up and the robot bridge is down, the response keeps the fixture body, sets `live: false`, and copies the server's note (for example "Robot bridge disconnected").

Allowed VLA calls: `vla_wall` `health` / `infer_prepare` / `list_tasks`; `vla_xvla` `health` / `list_targets` / `infer_prepare` / `peft_config_template`; `vla_weights` `list_models` / `local_status` / `download`; `vla_world_model` `health`.

## CLI

From `start.ps1 -Cli`, or `cargo run -p robotcraft-mcp --`:

```text
status
robots
telemetry yahboom_rover
teleop --robot yahboom_rover --x 0.2 --confirm
peers
yahboom read_all
yahboom forward --confirm
vla
vla-call vla_wall infer_prepare --task-hint "drive to the doorway and stop" --target raspbot
urdf
isaac
isaac-start --scene lab --confirm
estop --reason "operator halt"
--mcp
```

## MCP tools

`robot_status`, `robot_list`, `robot_telemetry`, `robot_teleop` (`robot_id`, `confirm`), `robot_estop`, `robot_peers`, `yahboom_call`, `vla_snapshot`, `vla_call`, `urdf_pose`, `isaac_snapshot`, `isaac_start`.

## Studio

`start.ps1` builds and opens the egui window. Sidebar: **Yahboom deck** (probe, read sensors, motion with the confirm checkbox, stop) and **VLA studio** (snapshot, Wall-OSS / X-VLA health, infer prepare). The 3D view is a local drawing. It is not a live twin.

## Status

**Does today:** Yahboom, VLA, and Isaac peers, confirm gates, labeled fixtures, Nori URDF posing in the viewport, FK plus reachability tests, `--smoke` on the desktop binary.

**Still open:** one URDF (Nori). No Gazebo or MuJoCo client. Isaac is a probe and a confirmed job start, not a rendered twin. No Unitree or LimX transport. Menu bar is File / Robots / View / Security / Help, not the 8-menu craft standard. Help is About only. `14116` is not a listening port. `web/three_viewer.html` does not load the URDF.

**Non-goals:** motion without confirm. A fixture must not be reported as live hardware. Voice wake stays on the fleet voice bus, not in this studio.

## Lineage — the Storytold craft-app pattern

Storytold ArtCraft ([storytold/artcraft](https://github.com/storytold/artcraft) — "the IDE for artists") was the starting point. The 16 craft apps in the Storytold stable each ship one binary that is both the human CLI and the agent MCP server (stdio), headless-first, with `run.json` + report artifacts. That modus operandi and harness is what this repo implements; the code here is independent, the pattern is shared.
