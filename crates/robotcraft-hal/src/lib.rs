//! RobotCraft Hardware Abstraction Layer (HAL).
//!
//! Provides protocol framing, telemetry parsing, and connection abstractions
//! for quadrupeds (Unitree Go1/Go2), differential rovers (Yahboom), serial microcontrollers,
//! and native ROS 2 topic streams.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HardwareKind {
    UnitreeQuadruped,
    YahboomRover,
    Ros2Bridge,
    GazeboSimulation,
    SerialMicrocontroller,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwistVelocity {
    pub linear_x: f32,
    pub linear_y: f32,
    pub angular_z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub hardware_id: String,
    pub kind: HardwareKind,
    pub battery_percentage: f32,
    pub is_armed: bool,
    pub emergency_stopped: bool,
    pub motor_temperatures_c: Vec<f32>,
    pub current_velocity: TwistVelocity,
    pub timestamp_ms: u64,
}

pub struct HardwareAdapter;

impl HardwareAdapter {
    pub fn mock_telemetry(kind: HardwareKind) -> TelemetrySnapshot {
        match kind {
            HardwareKind::UnitreeQuadruped => TelemetrySnapshot {
                hardware_id: "UNITREE-GO2-PRO-881".to_string(),
                kind: HardwareKind::UnitreeQuadruped,
                battery_percentage: 84.5,
                is_armed: true,
                emergency_stopped: false,
                motor_temperatures_c: vec![38.2, 39.1, 41.0, 40.5, 37.8, 38.6, 42.1, 41.8, 39.0, 39.5, 40.2, 41.1],
                current_velocity: TwistVelocity { linear_x: 0.0, linear_y: 0.0, angular_z: 0.0 },
                timestamp_ms: 1762234000000,
            },
            HardwareKind::YahboomRover => TelemetrySnapshot {
                hardware_id: "YAHBOOM-ROBOT-02".to_string(),
                kind: HardwareKind::YahboomRover,
                battery_percentage: 91.0,
                is_armed: true,
                emergency_stopped: false,
                motor_temperatures_c: vec![32.0, 32.5, 33.1, 32.8],
                current_velocity: TwistVelocity { linear_x: 0.0, linear_y: 0.0, angular_z: 0.0 },
                timestamp_ms: 1762234000000,
            },
            _ => TelemetrySnapshot {
                hardware_id: "GENERIC-ROBOT-VIRTUAL".to_string(),
                kind,
                battery_percentage: 100.0,
                is_armed: true,
                emergency_stopped: false,
                motor_temperatures_c: vec![25.0, 25.0],
                current_velocity: TwistVelocity { linear_x: 0.0, linear_y: 0.0, angular_z: 0.0 },
                timestamp_ms: 1762234000000,
            },
        }
    }
}
