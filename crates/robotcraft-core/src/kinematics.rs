//! RobotCraft Kinematics Engine.
//!
//! Forward and Inverse Kinematics, URDF model trees, and trajectory validation.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointDefinition {
    pub name: String,
    pub joint_type: String,
    pub min_limit_rad: f32,
    pub max_limit_rad: f32,
    pub current_angle_rad: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrdfRobotModel {
    pub name: String,
    pub dof: usize,
    pub joints: Vec<JointDefinition>,
    pub base_link: String,
    pub end_effector_link: String,
}

pub struct KinematicsSolver;

impl KinematicsSolver {
    pub fn default_6dof_manipulator() -> UrdfRobotModel {
        UrdfRobotModel {
            name: "CraftArm-6DOF".to_string(),
            dof: 6,
            joints: vec![
                JointDefinition { name: "base_waist".to_string(), joint_type: "revolute".to_string(), min_limit_rad: -3.14, max_limit_rad: 3.14, current_angle_rad: 0.0 },
                JointDefinition { name: "shoulder".to_string(), joint_type: "revolute".to_string(), min_limit_rad: -1.57, max_limit_rad: 1.57, current_angle_rad: 0.35 },
                JointDefinition { name: "elbow".to_string(), joint_type: "revolute".to_string(), min_limit_rad: -2.09, max_limit_rad: 2.09, current_angle_rad: 1.20 },
                JointDefinition { name: "wrist_pitch".to_string(), joint_type: "revolute".to_string(), min_limit_rad: -1.57, max_limit_rad: 1.57, current_angle_rad: -0.45 },
                JointDefinition { name: "wrist_roll".to_string(), joint_type: "revolute".to_string(), min_limit_rad: -3.14, max_limit_rad: 3.14, current_angle_rad: 0.0 },
                JointDefinition { name: "gripper_yaw".to_string(), joint_type: "revolute".to_string(), min_limit_rad: -1.57, max_limit_rad: 1.57, current_angle_rad: 0.10 },
            ],
            base_link: "base_stand".to_string(),
            end_effector_link: "tool_gripper_tip".to_string(),
        }
    }

    /// Forward Kinematics simulation: computes end-effector XYZ from joint angles.
    pub fn forward_kinematics(joints: &[f32]) -> (f32, f32, f32) {
        let mut x = 0.0f32;
        let mut y = 0.0f32;
        let mut z = 0.20f32; // base height

        let link_lengths = [0.25f32, 0.25f32, 0.15f32, 0.10f32];
        let mut accum_pitch = 0.0f32;

        for (i, &angle) in joints.iter().take(4).enumerate() {
            if i == 0 {
                // Base yaw
                x += angle.cos() * 0.05;
                y += angle.sin() * 0.05;
            } else {
                accum_pitch += angle;
                let l = link_lengths.get(i).copied().unwrap_or(0.1);
                x += accum_pitch.cos() * l;
                z += accum_pitch.sin() * l;
            }
        }

        (x, y, z)
    }

    /// Inverse Kinematics solver stub: validates reachability bounds.
    pub fn validate_target_pose(target_x: f32, target_y: f32, target_z: f32) -> Result<bool, &'static str> {
        let dist = (target_x.powi(2) + target_y.powi(2) + target_z.powi(2)).sqrt();
        if dist > 0.85 {
            Err("Target coordinate exceeds maximum arm reach (0.85m)")
        } else if dist < 0.10 {
            Err("Target coordinate within robot self-collision zone (<0.10m)")
        } else {
            Ok(true)
        }
    }
}
