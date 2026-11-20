---
name: robotcraft-safety
description: Safety gates, confirm flags, VLA whitelists, peer timeouts, and emergency stop rules in RobotCraft.
---

# RobotCraft Safety & Confirm Gates

RobotCraft enforces explicit confirmation gates before any physical motion, large weight download, or remote simulation job start.

## 1. Yahboom Motion Confirm Gate

- **Affected tools:** `robot_teleop` (when `robot_id` resolves to Yahboom: `yahboom`, `yahboom_rover`, `raspbot`, `boomy`, `rosmaster`) and `yahboom_call`.
- **Gated verbs:** `forward`, `backward`, `turn_left`, `turn_right`, `strafe_left`, `strafe_right`, and any non-zero `robot_teleop` twist.
- **Behavior when `confirm: false`:** Returns `success: false`, `dispatched: false`, `confirm_required: true`. Nothing is posted to `yahboom-mcp`.
- **Exception (Always Allowed):** Zero twist (`linear_x == 0 && linear_y == 0 && angular_z == 0`), `yahboom_call` with `stop` / `stop_all`, and `robot_estop` immediately post `POST /api/v1/stop_all` without requiring `confirm`.

## 2. VLA Whitelist & Download Gate

- **Tool:** `vla_call`
- **Allowed `(tool, operation)` pairs:**
  - `vla_wall`: `health`, `infer_prepare`, `list_tasks`
  - `vla_xvla`: `health`, `list_targets`, `infer_prepare`, `peft_config_template`
  - `vla_weights`: `list_models`, `local_status`, `download`
  - `vla_world_model`: `health`
- **Rejected:** Any other tool or operation (including `vla_training` and `vla_agentic_workflow`) is rejected locally before any HTTP call.
- **Download confirm:** `vla_weights` `download` requires `confirm: true`, which sends `X-VLA-Confirm: 1` to `vla-mcp`.

## 3. Isaac Sim Job Gate

- **Tool:** `isaac_start` (CLI: `robotcraft-cli isaac-start --scene <name> --confirm`)
- **Behavior:** Requires `confirm: true` and a non-empty `scene_name` before posting `POST /api/jobs/start` to `isaac-mcp` (`:11049`).

## 4. Peer Timeouts & Fixture Honesty

- HTTP calls in `robotcraft-hal::peers` use a `400 ms` connect timeout and `1500 ms` read timeout.
- Never treat `source: "studio-fixture"` (`live: false`) as live hardware telemetry.
