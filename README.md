# RobotCraft

**RobotCraft** is the embodied intelligence studio and universal robotics operating workstation for the sandraschi fleet. It consolidates hardware telemetry, forward/inverse kinematics solvers, URDF models, Gazebo simulation bridges, and physical actuator tele-op.

## Architecture
- **Consolidates Fleet Repos**: `robotics-mcp`, `ros-mcp`, `gazebo-mcp`, `unitree-mcp`, `yahboom-mcp`, `limx`, `norirobotics`, `vbot`.
- **Universal 3-Tier ArtCraft Standard**:
  - `crates/robotcraft-hal`: Hardware Abstraction Layer (Unitree UDP, Yahboom serial, ROS 2 DDS, Gazebo).
  - `crates/robotcraft-core`: Kinematics solvers, URDF link trees, and Command Bus (`Command::dispatch`).
  - `crates/robotcraft-mcp`: Stdio JSON-RPC server (MCP 2024-11-05) + CLI (`robotcraft-cli`).
  - `crates/robotcraft-app`: Immediate-mode Native Studio GUI (`robotcraft`).
- **Standard Control Port**: `14116`.
