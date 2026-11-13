//! RobotCraft Kinematics Engine.
//!
//! Forward and Inverse Kinematics, URDF model trees, and trajectory validation.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use std::f32::consts::PI;

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
    pub fn yahboom_rover_model() -> UrdfRobotModel {
        UrdfRobotModel {
            name: "Yahboom-ROSMASTER-X3".to_string(),
            dof: 4,
            joints: vec![
                JointDefinition {
                    name: "wheel_fl".to_string(),
                    joint_type: "continuous".to_string(),
                    min_limit_rad: -100.0,
                    max_limit_rad: 100.0,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "wheel_fr".to_string(),
                    joint_type: "continuous".to_string(),
                    min_limit_rad: -100.0,
                    max_limit_rad: 100.0,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "wheel_rl".to_string(),
                    joint_type: "continuous".to_string(),
                    min_limit_rad: -100.0,
                    max_limit_rad: 100.0,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "wheel_rr".to_string(),
                    joint_type: "continuous".to_string(),
                    min_limit_rad: -100.0,
                    max_limit_rad: 100.0,
                    current_angle_rad: 0.0,
                },
            ],
            base_link: "base_link".to_string(),
            end_effector_link: "camera_link".to_string(),
        }
    }

    pub fn default_6dof_manipulator() -> UrdfRobotModel {
        UrdfRobotModel {
            name: "CraftArm-6DOF".to_string(),
            dof: 6,
            joints: vec![
                JointDefinition {
                    name: "base_waist".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -PI,
                    max_limit_rad: PI,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "shoulder".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.35,
                },
                JointDefinition {
                    name: "elbow".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -2.09,
                    max_limit_rad: 2.09,
                    current_angle_rad: 1.20,
                },
                JointDefinition {
                    name: "wrist_pitch".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: -0.45,
                },
                JointDefinition {
                    name: "wrist_roll".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -PI,
                    max_limit_rad: PI,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "gripper_yaw".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.10,
                },
            ],
            base_link: "base_stand".to_string(),
            end_effector_link: "tool_gripper_tip".to_string(),
        }
    }

    pub fn saschibot_model() -> UrdfRobotModel {
        UrdfRobotModel {
            name: "Saschibot-Mobile-P6".to_string(),
            dof: 14,
            joints: vec![
                JointDefinition {
                    name: "wheel_L_spin".to_string(),
                    joint_type: "continuous".to_string(),
                    min_limit_rad: -100.0,
                    max_limit_rad: 100.0,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "wheel_R_spin".to_string(),
                    joint_type: "continuous".to_string(),
                    min_limit_rad: -100.0,
                    max_limit_rad: 100.0,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "lift_stage2".to_string(),
                    joint_type: "prismatic".to_string(),
                    min_limit_rad: 0.0,
                    max_limit_rad: 0.40,
                    current_angle_rad: 0.15,
                },
                JointDefinition {
                    name: "lift_stage3".to_string(),
                    joint_type: "prismatic".to_string(),
                    min_limit_rad: 0.0,
                    max_limit_rad: 0.40,
                    current_angle_rad: 0.15,
                },
                JointDefinition {
                    name: "left_shoulder_pan".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "left_shoulder_lift".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.2,
                },
                JointDefinition {
                    name: "left_elbow".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -2.0,
                    max_limit_rad: 2.0,
                    current_angle_rad: -0.5,
                },
                JointDefinition {
                    name: "left_wrist_pitch".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "left_flange".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -PI,
                    max_limit_rad: PI,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "right_shoulder_pan".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "right_shoulder_lift".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.2,
                },
                JointDefinition {
                    name: "right_elbow".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -2.0,
                    max_limit_rad: 2.0,
                    current_angle_rad: -0.5,
                },
                JointDefinition {
                    name: "right_wrist_pitch".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "right_flange".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -PI,
                    max_limit_rad: PI,
                    current_angle_rad: 0.0,
                },
            ],
            base_link: "base_deck_680x540mm".to_string(),
            end_effector_link: "robonine_parallel_gripper_l".to_string(),
        }
    }

    pub fn nori_a3_model() -> UrdfRobotModel {
        UrdfRobotModel {
            name: "NoriRobotics-A3".to_string(),
            dof: 16,
            joints: vec![
                JointDefinition {
                    name: "left_wheel_joint".to_string(),
                    joint_type: "continuous".to_string(),
                    min_limit_rad: -100.0,
                    max_limit_rad: 100.0,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "right_wheel_joint".to_string(),
                    joint_type: "continuous".to_string(),
                    min_limit_rad: -100.0,
                    max_limit_rad: 100.0,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "lift_extension_joint".to_string(),
                    joint_type: "prismatic".to_string(),
                    min_limit_rad: 0.0,
                    max_limit_rad: 0.35,
                    current_angle_rad: 0.12,
                },
                JointDefinition {
                    name: "left_shoulder_pitch_joint".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -2.0,
                    max_limit_rad: 2.0,
                    current_angle_rad: -0.6,
                },
                JointDefinition {
                    name: "left_shoulder_roll_joint".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.3,
                },
                JointDefinition {
                    name: "left_elbow_pitch_joint".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -2.2,
                    max_limit_rad: 2.2,
                    current_angle_rad: -0.8,
                },
                JointDefinition {
                    name: "left_wrist_roll_joint".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -PI,
                    max_limit_rad: PI,
                    current_angle_rad: 0.2,
                },
                JointDefinition {
                    name: "left_gripper_joint".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: 0.0,
                    max_limit_rad: 1.2,
                    current_angle_rad: 0.4,
                },
                JointDefinition {
                    name: "right_shoulder_pitch_joint".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -2.0,
                    max_limit_rad: 2.0,
                    current_angle_rad: -0.6,
                },
                JointDefinition {
                    name: "right_shoulder_roll_joint".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: -0.3,
                },
                JointDefinition {
                    name: "right_elbow_pitch_joint".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -2.2,
                    max_limit_rad: 2.2,
                    current_angle_rad: -0.8,
                },
                JointDefinition {
                    name: "right_wrist_roll_joint".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -PI,
                    max_limit_rad: PI,
                    current_angle_rad: -0.2,
                },
                JointDefinition {
                    name: "right_gripper_joint".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: 0.0,
                    max_limit_rad: 1.2,
                    current_angle_rad: 0.4,
                },
                JointDefinition {
                    name: "head_pan".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "head_tilt".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -0.7,
                    max_limit_rad: 0.7,
                    current_angle_rad: 0.1,
                },
                JointDefinition {
                    name: "rplidar_c1_spin".to_string(),
                    joint_type: "continuous".to_string(),
                    min_limit_rad: -1000.0,
                    max_limit_rad: 1000.0,
                    current_angle_rad: 0.0,
                },
            ],
            base_link: "base_shell".to_string(),
            end_effector_link: "left_gripper_tcp".to_string(),
        }
    }

    pub fn noetix_bumi_model() -> UrdfRobotModel {
        UrdfRobotModel {
            name: "Noetix-Bumi-Humanoid".to_string(),
            dof: 21,
            joints: vec![
                JointDefinition {
                    name: "left_hip_pitch".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.1,
                },
                JointDefinition {
                    name: "left_hip_roll".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -0.78,
                    max_limit_rad: 0.78,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "left_knee".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: 0.0,
                    max_limit_rad: 2.35,
                    current_angle_rad: 0.2,
                },
                JointDefinition {
                    name: "left_ankle_pitch".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -0.78,
                    max_limit_rad: 0.78,
                    current_angle_rad: -0.1,
                },
                JointDefinition {
                    name: "left_ankle_roll".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -0.52,
                    max_limit_rad: 0.52,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "left_toe".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -0.35,
                    max_limit_rad: 0.35,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "right_hip_pitch".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.1,
                },
                JointDefinition {
                    name: "right_hip_roll".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -0.78,
                    max_limit_rad: 0.78,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "right_knee".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: 0.0,
                    max_limit_rad: 2.35,
                    current_angle_rad: 0.2,
                },
                JointDefinition {
                    name: "right_ankle_pitch".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -0.78,
                    max_limit_rad: 0.78,
                    current_angle_rad: -0.1,
                },
                JointDefinition {
                    name: "right_ankle_roll".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -0.52,
                    max_limit_rad: 0.52,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "right_toe".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -0.35,
                    max_limit_rad: 0.35,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "lumbar".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -0.78,
                    max_limit_rad: 0.78,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "left_shoulder".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -2.0,
                    max_limit_rad: 2.0,
                    current_angle_rad: 0.3,
                },
                JointDefinition {
                    name: "left_elbow".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -2.0,
                    max_limit_rad: 0.0,
                    current_angle_rad: -0.6,
                },
                JointDefinition {
                    name: "left_wrist".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "right_shoulder".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -2.0,
                    max_limit_rad: 2.0,
                    current_angle_rad: 0.3,
                },
                JointDefinition {
                    name: "right_elbow".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -2.0,
                    max_limit_rad: 0.0,
                    current_angle_rad: -0.6,
                },
                JointDefinition {
                    name: "right_wrist".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "head_yaw".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -1.57,
                    max_limit_rad: 1.57,
                    current_angle_rad: 0.0,
                },
                JointDefinition {
                    name: "head_pitch".to_string(),
                    joint_type: "revolute".to_string(),
                    min_limit_rad: -0.78,
                    max_limit_rad: 0.78,
                    current_angle_rad: 0.05,
                },
            ],
            base_link: "pelvis".to_string(),
            end_effector_link: "head_rgb_cam".to_string(),
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
    pub fn validate_target_pose(
        target_x: f32,
        target_y: f32,
        target_z: f32,
    ) -> Result<bool, &'static str> {
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
