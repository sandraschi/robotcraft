# RobotCraft

**RobotCraft** is the embodied intelligence studio and universal robotics operating workstation for the sandraschi fleet. It consolidates hardware telemetry, forward/inverse kinematics solvers, URDF models, Gazebo simulation bridges, and physical actuator tele-op.

## Architecture
- **Consolidates Fleet Repos**: `robotics-mcp`, `ros-mcp`, `gazebo-mcp`, `unitree-mcp`, `yahboom-mcp`, `limx`, `nori` (`norirobotics-mcp`: Nori A3 home robot, LeRobot recording, VR spawn), `vbot`, `rust-saschirob-mcp` (personal robotics cookbook).
- **Universal 3-Tier ArtCraft Standard**:
  - `crates/robotcraft-hal`: Hardware Abstraction Layer (Unitree UDP, Yahboom serial, ROS 2 DDS, Gazebo).
  - `crates/robotcraft-core`: Kinematics solvers, URDF link trees, and Command Bus (`Command::dispatch`).
  - `crates/robotcraft-mcp`: Stdio JSON-RPC server (MCP 2024-11-05) + CLI (`robotcraft-cli`).
  - `crates/robotcraft-app`: Immediate-mode Native Studio GUI (`robotcraft`).
- **Standard Control Port**: `14116`.

## Lineage — the Storytold craft-app pattern

Storytold ArtCraft ([storytold/artcraft](https://github.com/storytold/artcraft) — "the IDE for artists") was the starting point. The 16 craft apps in the Storytold stable each ship one binary that is both the human CLI and the agent MCP server (stdio), headless-first, with `run.json` + report artifacts. That modus operandi and harness is what this repo implements; the code here is independent, the pattern is shared.
