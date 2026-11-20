---
name: robotcraft-urdf-kinematics
description: URDF link posing, planar forward kinematics, and reachability sphere validation in RobotCraft.
---

# RobotCraft URDF & Kinematics

RobotCraft provides two complementary kinematics subsystems in `robotcraft-core`:

## 1. URDF Link Posing (`urdf_pose`)

- **Tool:** `urdf_pose` (CLI: `robotcraft-cli urdf [--path <file>]`)
- **Default model:** `D:\Dev\repos\norirobotics-mcp\models\nori_description\urdf\nori.expanded.urdf`, overridable via `ROBOTCRAFT_NORI_URDF`.
- **Joint handling:**
  - Keeps `revolute`, `continuous`, and `prismatic` joints; walks the kinematic tree from `base_footprint` / `base_link` (guard limit: 512 nodes).
  - Joint values passed in `joints: [{ "name": "...", "angle": <rad> }]` are clamped to `[lower, upper]` (or `[-pi, pi]` for `continuous` / unbounded joints).
- **Geometry fallback:** Mesh STL/GLB files are **not** loaded. A link with only a mesh visual falls back to its `<collision>` box, or a `0.04 m` (4 cm) cube at the link origin.

## 2. Planar Forward Kinematics & Reachability Sphere

- **Forward Kinematics (`KinematicsSolver::forward_kinematics`):**
  - Base height `z = 0.20 m`, link lengths `[0.25, 0.25, 0.15, 0.10]` m.
  - Joint 0 applies base yaw (`cos/sin * 0.05 m`); joints 1..3 accumulate pitch in the `(x, z)` plane.
  - CLI: `robotcraft-cli fk <j0> <j1> <j2> <j3>`
- **Reachability Validation (`KinematicsSolver::validate_target_pose`):**
  - Computes Euclidean distance `d = sqrt(x^2 + y^2 + z^2)`.
  - `d < 0.10 m`: rejected as inside the robot self-collision zone.
  - `d > 0.85 m`: rejected as exceeding maximum arm reach (`0.85 m`).
  - `0.10 m <= d <= 0.85 m`: `reachable: true`.
- **Note:** Numerical IK and RRT* collision-free path planning are not yet implemented in `0.3.0`.
