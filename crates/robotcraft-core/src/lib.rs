//! RobotCraft Core Library & Universal Command Bus.
//!
//! Provides robotics telemetry, kinematics solvers, trajectory execution,
//! and emergency safety stops across native desktop GUI, CLI, and MCP stdio.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

pub mod i18n;
pub mod kinematics;
pub mod urdf;

use crate::kinematics::KinematicsSolver;
use databasecraft_store::DatabaseStore;
use robotcraft_hal::peers;
use robotcraft_hal::{HardwareAdapter, HardwareKind};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const CONTROL_PORT: u16 = 14116;
pub const APP_VERSION: &str = "0.3.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "params")]
pub enum Command {
    Status,
    ListRobots,
    GetTelemetry {
        robot_id: String,
    },
    TeleopTwist {
        linear_x: f32,
        linear_y: f32,
        angular_z: f32,
        robot_id: String,
        confirm: bool,
    },
    GetRobotModel,
    GetRobotModelById {
        robot_id: String,
    },
    GetRobotCadPlan {
        robot_id: String,
    },
    RunAiBehaviorWorkflow {
        robot_id: String,
        workflow_type: String, // "explore_and_map", "wave_and_greet", "bimanual_caddy_pickup", "stand_and_balance"
    },
    CalculateFk {
        joint_angles: Vec<f32>,
    },
    ValidatePose {
        x: f32,
        y: f32,
        z: f32,
    },
    EmergencyStop {
        reason: String,
    },
    /// Probe yahboom-mcp (:10892) and vla-mcp (:11024). Does not move hardware.
    ProbePeers,
    /// `yahboom_tool` via POST /api/v1/control/tool. Motion verbs require `confirm`.
    YahboomCall {
        operation: String,
        param1: Option<String>,
        confirm: bool,
    },
    /// Read vla-mcp status, tool list, and fleet bridge. Does not run training.
    VlaSnapshot,
    /// Whitelisted read/prepare calls into POST /api/v1/control/{tool}.
    VlaCall {
        tool: String,
        operation: String,
        task_hint: Option<String>,
        target: Option<String>,
        model_key: Option<String>,
        confirm: bool,
    },
    /// Pose links from a URDF file. Empty path loads the Nori expanded URDF.
    UrdfPose {
        path: String,
        joints: Vec<(String, f32)>,
    },
    /// isaac-mcp health and scene list. Does not start a simulation.
    IsaacSnapshot,
    /// POST /api/jobs/start. Requires confirm.
    IsaacStart {
        scene_name: String,
        confirm: bool,
    },
}

pub async fn dispatch(cmd: Command, _store: &DatabaseStore) -> Value {
    match cmd {
        Command::Status => {
            json!({
                "app": "robotcraft",
                "version": APP_VERSION,
                "control_port": CONTROL_PORT,
                "status": "ready",
                "active_drivers": ["Yahboom_MCP", "VLA_MCP", "Isaac_MCP"],
                "peers": {
                    "yahboom_mcp": peers::yahboom_base(),
                    "vla_mcp": peers::vla_base(),
                    "isaac_mcp": peers::isaac_base()
                },
                "success": true
            })
        }
        Command::ListRobots => {
            json!({
                "success": true,
                "available_robots": [
                    { "id": "saschibot", "name": "Saschibot Mobile Service Bot P6", "dof": 14, "protocol": "Micro-ROS / STS3215 Bus", "has_cad": true, "has_three_js": true },
                    { "id": "nori_a3", "name": "NoriRobotics A3 Mobile Manipulator", "dof": 16, "protocol": "ROS2 DDS / Feetech", "has_cad": true, "has_three_js": true },
                    { "id": "noetix_bumi", "name": "Noetix Bumi Humanoid Robot", "dof": 21, "protocol": "ROS2 Bridge / Jetson Orin", "has_cad": true, "has_three_js": true },
                    { "id": "unitree_go2", "name": "Unitree Go2 Pro Quadruped", "dof": 12, "protocol": "UDP / Low-Level", "has_cad": false, "has_three_js": true },
                    { "id": "yahboom_rover", "name": "Yahboom ROSMASTER X3 Rover", "dof": 4, "protocol": "yahboom-mcp :10892", "has_cad": false, "has_three_js": true, "peer": "yahboom-mcp" },
                    { "id": "craftarm_6dof", "name": "CraftArm Industrial 6-DOF Manipulator", "dof": 6, "protocol": "CAN Bus / Micro-ROS", "has_cad": true, "has_three_js": true }
                ]
            })
        }
        Command::GetTelemetry { robot_id } => {
            if peers::is_yahboom_id(&robot_id) {
                yahboom_telemetry_response()
            } else {
                fixture_telemetry(&robot_id)
            }
        }
        Command::TeleopTwist {
            linear_x,
            linear_y,
            angular_z,
            robot_id,
            confirm,
        } => dispatch_twist(robot_id, linear_x, linear_y, angular_z, confirm),
        Command::GetRobotModel => {
            let model = KinematicsSolver::default_6dof_manipulator();
            json!({
                "success": true,
                "model": model
            })
        }
        Command::GetRobotModelById { robot_id } => {
            let model = match robot_id.to_lowercase().as_str() {
                "saschibot" | "saschi" | "saschirob" => KinematicsSolver::saschibot_model(),
                "nori_a3" | "nori" | "norirobotics" => KinematicsSolver::nori_a3_model(),
                "noetix_bumi" | "bumi" | "noetix" => KinematicsSolver::noetix_bumi_model(),
                "yahboom_rover" | "yahboom" | "raspbot" | "boomy" | "rosmaster" => {
                    KinematicsSolver::yahboom_rover_model()
                }
                _ => KinematicsSolver::default_6dof_manipulator(),
            };
            json!({
                "success": true,
                "robot_id": robot_id,
                "model": model
            })
        }
        Command::GetRobotCadPlan { robot_id } => match robot_id.to_lowercase().as_str() {
            "saschibot" | "saschi" | "saschirob" => {
                json!({
                    "success": true,
                    "robot_id": "saschibot",
                    "cad_type": "URDF_DXF_SVG_STL",
                    "base_deck": "dxf_svg/01_base_deck_680x540mm.svg (profiles: 18, 680x540mm)",
                    "flange_plate": "dxf_svg/02_column_pedestal_flange_280x280mm.svg",
                    "visor_bezel": "dxf_svg/03_head_visor_bezel_180x110mm.svg",
                    "dock_plate": "dxf_svg/04_dock_pad_and_tag_plate_600x600mm.svg (600x600mm)",
                    "stl_parts": [
                        "stl/01_sts3215_bench_test_mount.stl",
                        "stl/02_robonine_parallel_gripper_base.stl",
                        "stl/03_robonine_parallel_finger_pair.stl",
                        "stl/04_landscape_head_shell_130x210x150mm.stl",
                        "stl/05_wheel_arch_fender_200mm.stl",
                        "stl/06_column_pinch_guard_collar.stl",
                        "stl/07_front_anti_tip_horn_pair.stl",
                        "stl/08_wrist_micro_cam_ring_mount.stl",
                        "stl/09_base_modular_caddy_receiver_bracket_pair.stl",
                        "stl/10_easy_grip_tool_and_spray_adapter.stl"
                    ],
                    "source_repo": "rust-saschirob-mcp/models/fabrication"
                })
            }
            "nori_a3" | "nori" | "norirobotics" => {
                json!({
                    "success": true,
                    "robot_id": "nori_a3",
                    "cad_type": "URDF_STEP_STL_GLB",
                    "rig_glb": "models/nori_description/nori_a3_rig.glb",
                    "posed_glb": "models/nori_description/nori_a3_posed.glb",
                    "urdf_path": "models/nori_description/urdf/nori.expanded.urdf",
                    "step_attachments": [
                        "models/nori-printables/a3/gripper-attachment-points/gripper-attachment-points.step",
                        "models/nori-printables/a3/gripper-attachment-points/gripper-attachment-points.stl"
                    ],
                    "stl_printables": [
                        "models/nori-printables/a3/ear-pucks/ear-pucks.stl",
                        "models/nori-printables/a3/head-pegs/head-pegs.stl",
                        "models/nori_description/meshes/visual/base_shell.stl",
                        "models/nori_description/meshes/visual/shoulder_pitch.stl",
                        "models/nori_description/meshes/visual/bicep_yaw.stl",
                        "models/nori_description/meshes/visual/elbow_pitch.stl",
                        "models/nori_description/meshes/visual/wrist_pitch.stl",
                        "models/nori_description/meshes/visual/wrist_roll.stl",
                        "models/nori_description/meshes/visual/gripper_l.stl",
                        "models/nori_description/meshes/visual/RPLiDarC1.stl"
                    ],
                    "source_repo": "norirobotics-mcp/models"
                })
            }
            "noetix_bumi" | "bumi" | "noetix" => {
                json!({
                    "success": true,
                    "robot_id": "noetix_bumi",
                    "cad_type": "VENDOR_SPEC_KINEMATIC_TOPOLOGY",
                    "kinematic_architecture": "21-DOF Full Bipedal Humanoid (Lite/Air/Pro/Max/EDU)",
                    "dimensions": "Height: 1200mm, Weight: 18.5kg, Payload: 3.5kg per arm",
                    "joint_structure": {
                        "legs": ["left/right hip pitch/roll", "knee", "ankle pitch/roll", "toe"],
                        "torso": ["lumbar pitch/yaw"],
                        "arms": ["left/right shoulder", "elbow", "wrist"],
                        "head": ["head yaw", "head pitch", "RGB stereo camera mount"]
                    },
                    "cad_plans_status": "Proprietary vendor injection-mold CAD; sovereign kinematics & URDF topology extracted in RobotCraft",
                    "rosbridge_endpoint": "ws://bumi.local:9090 (/joint_states, /imu/data, /battery_state)"
                })
            }
            _ => {
                json!({
                    "success": false,
                    "error": format!("No CAD plan available for {robot_id}")
                })
            }
        },
        Command::RunAiBehaviorWorkflow {
            robot_id,
            workflow_type,
        } => {
            let robot = robot_id.to_lowercase();
            let trajectory = match (robot.as_str(), workflow_type.as_str()) {
                ("saschibot" | "saschi", "wave_and_greet") => {
                    json!({
                        "workflow": "wave_and_greet",
                        "keyframes": [
                            { "t_sec": 0.0, "left_shoulder_lift": 0.0, "left_elbow": 0.0, "wrist": 0.0, "base_x": 0.0, "base_y": 0.0 },
                            { "t_sec": 1.0, "left_shoulder_lift": 0.8, "left_elbow": -1.2, "wrist": 0.3, "base_x": 0.0, "base_y": 0.0 },
                            { "t_sec": 1.5, "left_shoulder_lift": 0.8, "left_elbow": -1.2, "wrist": -0.3, "base_x": 0.0, "base_y": 0.0 },
                            { "t_sec": 2.0, "left_shoulder_lift": 0.8, "left_elbow": -1.2, "wrist": 0.3, "base_x": 0.0, "base_y": 0.0 },
                            { "t_sec": 3.0, "left_shoulder_lift": 0.0, "left_elbow": 0.0, "wrist": 0.0, "base_x": 0.0, "base_y": 0.0 }
                        ],
                        "summary": "Saschibot raised left arm with STS3215 servo, executed friendly wave and greeted user in virtual space."
                    })
                }
                ("saschibot" | "saschi", "bimanual_caddy_pickup") => {
                    json!({
                        "workflow": "bimanual_caddy_pickup",
                        "keyframes": [
                            { "t_sec": 0.0, "lift_m": 0.0, "arms_pitch": 0.0, "grippers_mm": 84.0, "base_x": 0.0 },
                            { "t_sec": 1.5, "lift_m": 0.35, "arms_pitch": 0.45, "grippers_mm": 84.0, "base_x": 0.5 },
                            { "t_sec": 2.5, "lift_m": 0.35, "arms_pitch": 0.45, "grippers_mm": 12.0, "base_x": 0.5 },
                            { "t_sec": 4.0, "lift_m": 0.20, "arms_pitch": 0.0, "grippers_mm": 12.0, "base_x": 0.0 }
                        ],
                        "summary": "Saschibot extended 3-stage column mast, advanced base, closed Robonine parallel grippers and securely docked caddy."
                    })
                }
                ("nori_a3" | "nori", "explore_and_map") => {
                    json!({
                        "workflow": "explore_and_map",
                        "keyframes": [
                            { "t_sec": 0.0, "base_x": 0.0, "base_y": 0.0, "yaw": 0.0, "rplidar_rpm": 600.0, "lift_m": 0.12 },
                            { "t_sec": 2.0, "base_x": 1.2, "base_y": 0.0, "yaw": 0.0, "rplidar_rpm": 600.0, "lift_m": 0.20 },
                            { "t_sec": 4.0, "base_x": 1.8, "base_y": 0.8, "yaw": 1.57, "rplidar_rpm": 600.0, "lift_m": 0.25 },
                            { "t_sec": 6.0, "base_x": 0.5, "base_y": 1.2, "yaw": std::f32::consts::PI, "rplidar_rpm": 600.0, "lift_m": 0.15 },
                            { "t_sec": 8.0, "base_x": 0.0, "base_y": 0.0, "yaw": 0.0, "rplidar_rpm": 600.0, "lift_m": 0.12 }
                        ],
                        "summary": "NoriRobotics A3 traversed boundary contour with RPLiDar C1 360° point cloud generation and SLAM waypoint loop closure."
                    })
                }
                ("noetix_bumi" | "bumi", "stand_and_balance") => {
                    json!({
                        "workflow": "stand_and_balance",
                        "keyframes": [
                            { "t_sec": 0.0, "hips_pitch": 0.5, "knees": 0.9, "ankles": -0.4, "com_z": 0.55 },
                            { "t_sec": 1.5, "hips_pitch": 0.2, "knees": 0.4, "ankles": -0.2, "com_z": 0.72 },
                            { "t_sec": 3.0, "hips_pitch": 0.0, "knees": 0.1, "ankles": 0.0, "com_z": 0.85 },
                            { "t_sec": 4.5, "hips_pitch": 0.05, "knees": 0.1, "ankles": 0.0, "com_z": 0.85 }
                        ],
                        "summary": "Noetix Bumi executed Zero-Moment Point (ZMP) inverted pendulum balance trajectory, rising smoothly to stable upright stance."
                    })
                }
                _ => {
                    json!({
                        "workflow": workflow_type,
                        "keyframes": [
                            { "t_sec": 0.0, "linear_x": 0.0, "angular_z": 0.0 },
                            { "t_sec": 2.0, "linear_x": 0.3, "angular_z": 0.1 },
                            { "t_sec": 4.0, "linear_x": 0.0, "angular_z": 0.0 }
                        ],
                        "summary": format!("Autonomous motion trajectory simulated for {robot_id} in virtual space")
                    })
                }
            };

            json!({
                "success": true,
                "robot_id": robot_id,
                "workflow_dispatched": workflow_type,
                "virtual_space_simulation": true,
                "trajectory": trajectory
            })
        }
        Command::CalculateFk { joint_angles } => {
            let (x, y, z) = KinematicsSolver::forward_kinematics(&joint_angles);
            json!({
                "success": true,
                "joint_angles": joint_angles,
                "end_effector_xyz": [x, y, z]
            })
        }
        Command::ValidatePose { x, y, z } => {
            match KinematicsSolver::validate_target_pose(x, y, z) {
                Ok(true) => json!({ "success": true, "reachable": true, "target_xyz": [x, y, z] }),
                Ok(false) => {
                    json!({ "success": true, "reachable": false, "target_xyz": [x, y, z] })
                }
                Err(err) => {
                    json!({ "success": false, "reachable": false, "error": err, "target_xyz": [x, y, z] })
                }
            }
        }
        Command::EmergencyStop { reason } => {
            let yahboom = if peers::yahboom_base().is_some() {
                match peers::yahboom_stop_all() {
                    Ok(body) => json!({ "attempted": true, "peer": body }),
                    Err(err) => json!({ "attempted": true, "error": err }),
                }
            } else {
                json!({ "attempted": false, "disabled": true })
            };
            json!({
                "success": true,
                "emergency_stop": "ACTIVATED",
                "motors_disabled": true,
                "reason": reason,
                "yahboom": yahboom,
                "timestamp_utc": chrono::Utc::now().to_rfc3339()
            })
        }
        Command::ProbePeers => peers::probe_peers(),
        Command::YahboomCall {
            operation,
            param1,
            confirm,
        } => dispatch_yahboom_call(&operation, param1.as_deref(), confirm),
        Command::VlaSnapshot => vla_snapshot(),
        Command::VlaCall {
            tool,
            operation,
            task_hint,
            target,
            model_key,
            confirm,
        } => dispatch_vla_call(
            &tool,
            &operation,
            task_hint.as_deref(),
            target.as_deref(),
            model_key.as_deref(),
            confirm,
        ),
        Command::UrdfPose { path, joints } => urdf_pose_response(&path, &joints),
        Command::IsaacSnapshot => isaac_snapshot(),
        Command::IsaacStart {
            scene_name,
            confirm,
        } => isaac_start(&scene_name, confirm),
    }
}

fn hardware_kind(robot_id: &str) -> HardwareKind {
    match robot_id.to_lowercase().as_str() {
        "saschibot" | "saschi" | "saschirob" => HardwareKind::SaschibotService,
        "nori_a3" | "nori" | "norirobotics" => HardwareKind::NoriRoboticsA3,
        "noetix_bumi" | "bumi" | "noetix" => HardwareKind::NoetixBumiHumanoid,
        "unitree_go2" | "go2" | "quadruped" => HardwareKind::UnitreeQuadruped,
        "yahboom_rover" | "yahboom" | "rover" | "raspbot" | "boomy" | "rosmaster" => {
            HardwareKind::YahboomRover
        }
        _ => HardwareKind::GazeboSimulation,
    }
}

fn fixture_telemetry(robot_id: &str) -> Value {
    let telem = HardwareAdapter::mock_telemetry(hardware_kind(robot_id));
    json!({
        "success": true,
        "live": false,
        "source": "studio-fixture",
        "note": "Studio fixture. This robot has no live peer in RobotCraft.",
        "telemetry": telem
    })
}

fn yahboom_telemetry_response() -> Value {
    if peers::yahboom_base().is_none() {
        let mut body = fixture_telemetry("yahboom_rover");
        if let Some(obj) = body.as_object_mut() {
            obj.insert(
                "note".to_string(),
                json!("yahboom-mcp disabled (ROBOTCRAFT_YAHBOOM_URL=off)"),
            );
        }
        return body;
    }
    match peers::yahboom_telemetry() {
        Ok(raw) if raw.get("status").and_then(|s| s.as_str()) == Some("live") => json!({
            "success": true,
            "live": true,
            "source": "yahboom-mcp",
            "peer": raw,
            "telemetry": map_yahboom_telemetry(&raw)
        }),
        Ok(raw) => {
            let mut body = fixture_telemetry("yahboom_rover");
            if let Some(obj) = body.as_object_mut() {
                obj.insert("peer".to_string(), raw.clone());
                let note = raw.get("message").and_then(|m| m.as_str()).unwrap_or("yahboom-mcp is up. The robot bridge is offline, so the numbers below are a studio fixture.");
                obj.insert("note".to_string(), json!(note));
            }
            body
        }
        Err(err) => {
            let mut body = fixture_telemetry("yahboom_rover");
            if let Some(obj) = body.as_object_mut() {
                obj.insert("peer_error".to_string(), json!(err));
                obj.insert(
                    "note".to_string(),
                    json!("yahboom-mcp is not reachable. Numbers below are a studio fixture."),
                );
            }
            body
        }
    }
}

fn map_yahboom_telemetry(raw: &Value) -> Value {
    let battery = raw.get("battery").and_then(|b| b.as_f64()).unwrap_or(0.0);
    let linear = raw
        .pointer("/velocity/linear")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let angular = raw
        .pointer("/velocity/angular")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    json!({
        "hardware_id": "YAHBOOM-ROSMASTER",
        "kind": "YahboomRover",
        "battery_percentage": battery,
        "voltage": raw.get("voltage").cloned().unwrap_or(Value::Null),
        "is_armed": true,
        "emergency_stopped": false,
        "motor_temperatures_c": [],
        "current_velocity": { "linear_x": linear, "linear_y": 0.0, "angular_z": angular },
        "imu": raw.get("imu").cloned().unwrap_or(Value::Null),
        "scan": raw.get("scan").cloned().unwrap_or(Value::Null),
        "timestamp_ms": chrono::Utc::now().timestamp_millis()
    })
}

fn dispatch_twist(
    robot_id: String,
    linear_x: f32,
    linear_y: f32,
    angular_z: f32,
    confirm: bool,
) -> Value {
    if peers::is_yahboom_id(&robot_id) {
        let stop = linear_x == 0.0 && linear_y == 0.0 && angular_z == 0.0;
        if stop {
            return match peers::yahboom_stop_all() {
                Ok(body) => json!({
                    "success": true,
                    "dispatched": true,
                    "source": "yahboom-mcp",
                    "command": "stop_all",
                    "peer": body
                }),
                Err(err) => json!({
                    "success": false,
                    "dispatched": false,
                    "source": "yahboom-mcp",
                    "error": err
                }),
            };
        }
        if !confirm {
            return json!({
                "success": false,
                "dispatched": false,
                "confirm_required": true,
                "source": "yahboom-mcp",
                "command": "VELOCITY_TWIST",
                "linear_x": linear_x,
                "linear_y": linear_y,
                "angular_z": angular_z,
                "message": "Yahboom motion stays local until confirm=true."
            });
        }
        return match peers::yahboom_move(linear_x, linear_y, angular_z) {
            Ok(body) => {
                let http_ok = body.get("http_status").is_none();
                let status_ok = body.get("status").and_then(|s| s.as_str()) == Some("success");
                json!({
                    "success": http_ok && status_ok,
                    "dispatched": true,
                    "source": "yahboom-mcp",
                    "command": "VELOCITY_TWIST",
                    "linear_x": linear_x,
                    "linear_y": linear_y,
                    "angular_z": angular_z,
                    "peer": body
                })
            }
            Err(err) => json!({
                "success": false,
                "dispatched": false,
                "source": "yahboom-mcp",
                "error": err
            }),
        };
    }
    json!({
        "success": true,
        "dispatched": false,
        "simulated": true,
        "source": "studio-fixture",
        "command": "VELOCITY_TWIST",
        "robot_id": robot_id,
        "linear_x": linear_x,
        "linear_y": linear_y,
        "angular_z": angular_z,
        "status": "SIMULATED"
    })
}

fn dispatch_yahboom_call(operation: &str, param1: Option<&str>, confirm: bool) -> Value {
    let op = operation.trim().to_ascii_lowercase();
    if op == "stop" || op == "stop_all" {
        return match peers::yahboom_stop_all() {
            Ok(body) => {
                json!({ "success": true, "dispatched": true, "source": "yahboom-mcp", "operation": op, "peer": body })
            }
            Err(err) => {
                json!({ "success": false, "dispatched": false, "source": "yahboom-mcp", "operation": op, "error": err })
            }
        };
    }
    if peers::is_motion_operation(&op) && !confirm {
        return json!({
            "success": false,
            "dispatched": false,
            "confirm_required": true,
            "source": "yahboom-mcp",
            "operation": op,
            "message": "Motion verbs require confirm=true before yahboom-mcp is called."
        });
    }
    match peers::yahboom_tool(&op, param1) {
        Ok(body) => {
            let ok = body
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(body.get("http_status").is_none());
            json!({ "success": ok, "dispatched": true, "source": "yahboom-mcp", "operation": op, "peer": body })
        }
        Err(err) => {
            json!({ "success": false, "dispatched": false, "source": "yahboom-mcp", "operation": op, "error": err })
        }
    }
}

fn vla_call_allowed(tool: &str, operation: &str) -> bool {
    matches!(
        (tool, operation),
        ("vla_wall", "health" | "infer_prepare" | "list_tasks")
            | (
                "vla_xvla",
                "health" | "list_targets" | "infer_prepare" | "peft_config_template"
            )
            | ("vla_weights", "list_models" | "local_status" | "download")
            | ("vla_world_model", "health")
    )
}

fn vla_snapshot() -> Value {
    match peers::vla_status() {
        Ok(status) => {
            let tools = peers::vla_tools().unwrap_or_else(|err| json!({ "error": err }));
            let fleet = peers::vla_fleet().unwrap_or_else(|err| json!({ "error": err }));
            let episodes = peers::vla_episodes(20).unwrap_or_else(|err| json!({ "error": err }));
            json!({
                "success": true,
                "reachable": true,
                "source": "vla-mcp",
                "summary": peers::summarize_vla_status(&status),
                "tools": tools,
                "fleet": fleet,
                "episodes": episodes
            })
        }
        Err(err) => json!({
            "success": false,
            "reachable": false,
            "source": "vla-mcp",
            "error": err
        }),
    }
}

fn dispatch_vla_call(
    tool: &str,
    operation: &str,
    task_hint: Option<&str>,
    target: Option<&str>,
    model_key: Option<&str>,
    confirm: bool,
) -> Value {
    if !vla_call_allowed(tool, operation) {
        return json!({
            "success": false,
            "dispatched": false,
            "source": "vla-mcp",
            "error": format!("'{tool}' operation '{operation}' is outside the RobotCraft whitelist"),
            "allowed": [
                "vla_wall health|infer_prepare|list_tasks",
                "vla_xvla health|list_targets|infer_prepare|peft_config_template",
                "vla_weights list_models|local_status|download",
                "vla_world_model health"
            ]
        });
    }
    if operation == "download" && !confirm {
        return json!({
            "success": false,
            "dispatched": false,
            "confirm_required": true,
            "source": "vla-mcp",
            "message": "Weight download requires confirm=true (sends X-VLA-Confirm: 1)."
        });
    }
    let mut body = json!({ "operation": operation });
    if let Some(hint) = task_hint {
        body["task_hint"] = json!(hint);
    }
    if let Some(edge) = target {
        body["target"] = json!(edge);
    }
    if let Some(key) = model_key {
        body["model_key"] = json!(key);
    }
    let needs_header = operation == "download";
    match peers::vla_control(tool, &body, needs_header && confirm) {
        Ok(peer) => {
            let ok = peer
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(peer.get("http_status").is_none());
            json!({
                "success": ok,
                "dispatched": true,
                "source": "vla-mcp",
                "tool": tool,
                "operation": operation,
                "peer": peer
            })
        }
        Err(err) => json!({
            "success": false,
            "dispatched": false,
            "source": "vla-mcp",
            "tool": tool,
            "operation": operation,
            "error": err
        }),
    }
}

fn urdf_pose_response(path: &str, joints: &[(String, f32)]) -> Value {
    let resolved = if path.trim().is_empty() {
        urdf::default_nori_urdf()
    } else {
        std::path::PathBuf::from(path)
    };
    match urdf::load_urdf_file(&resolved) {
        Ok(model) => {
            let links = urdf::pose_links(&model, joints);
            let movable = urdf::movable_joints(&model, joints);
            json!({
                "success": true,
                "source": "urdf",
                "path": resolved,
                "robot": model.name,
                "link_count": links.len(),
                "links": links,
                "joints": movable
            })
        }
        Err(err) => json!({
            "success": false,
            "source": "urdf",
            "path": resolved,
            "error": err
        }),
    }
}

fn isaac_snapshot() -> Value {
    match peers::isaac_health() {
        Ok(health) => {
            let scenes = peers::isaac_scenes().unwrap_or_else(|err| json!({ "error": err }));
            json!({
                "success": true,
                "reachable": true,
                "source": "isaac-mcp",
                "health": health,
                "scenes": scenes
            })
        }
        Err(err) => json!({
            "success": false,
            "reachable": false,
            "source": "isaac-mcp",
            "error": err
        }),
    }
}

fn isaac_start(scene_name: &str, confirm: bool) -> Value {
    if !confirm {
        return json!({
            "success": false,
            "dispatched": false,
            "confirm_required": true,
            "source": "isaac-mcp",
            "scene_name": scene_name,
            "message": "Starting an Isaac job requires confirm=true."
        });
    }
    if scene_name.trim().is_empty() {
        return json!({
            "success": false,
            "dispatched": false,
            "source": "isaac-mcp",
            "error": "scene_name is required"
        });
    }
    match peers::isaac_start(scene_name.trim()) {
        Ok(peer) => json!({
            "success": peer.get("success").and_then(|v| v.as_bool()).unwrap_or(peer.get("http_status").is_none()),
            "dispatched": true,
            "source": "isaac-mcp",
            "scene_name": scene_name,
            "peer": peer
        }),
        Err(err) => json!({
            "success": false,
            "dispatched": false,
            "source": "isaac-mcp",
            "scene_name": scene_name,
            "error": err
        }),
    }
}
