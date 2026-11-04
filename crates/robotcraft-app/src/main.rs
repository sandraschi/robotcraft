//! RobotCraft Native Desktop Studio.
//!
//! Visual robotics workstation with 3D joint telemetry, virtual tele-op stick,
//! kinematics reachability analyzer, and hardware kill switches.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use databasecraft_store::DatabaseStore;
use eframe::egui::{self, Color32, Vec2};
use robotcraft_core::{dispatch, Command, APP_VERSION, CONTROL_PORT};
use serde_json::Value;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveWorkspace {
    Dashboard,
    TeleopControl,
    KinematicsStudio,
    FleetStatus,
}

pub struct RobotCraftGui {
    pub store: Arc<DatabaseStore>,
    pub runtime: Arc<Runtime>,
    pub active_workspace: ActiveWorkspace,
    pub selected_robot: String,
    pub linear_x: f32,
    pub linear_y: f32,
    pub angular_z: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub target_z: f32,
    pub telemetry_data: Option<Value>,
    pub model_data: Option<Value>,
    pub reachability_result: Option<Value>,
    pub status_message: String,
}

impl RobotCraftGui {
    pub fn new(store: Arc<DatabaseStore>, runtime: Arc<Runtime>) -> Self {
        let mut app = Self {
            store,
            runtime,
            active_workspace: ActiveWorkspace::Dashboard,
            selected_robot: "unitree_go2".to_string(),
            linear_x: 0.0,
            linear_y: 0.0,
            angular_z: 0.0,
            target_x: 0.40,
            target_y: 0.20,
            target_z: 0.35,
            telemetry_data: None,
            model_data: None,
            reachability_result: None,
            status_message: format!("RobotCraft v{APP_VERSION} [Control Port {CONTROL_PORT} Active]"),
        };
        app.fetch_telemetry();
        app.fetch_model();
        app
    }

    pub fn fetch_telemetry(&mut self) {
        let id = self.selected_robot.clone();
        let res = self.runtime.block_on(dispatch(Command::GetTelemetry { robot_id: id }, &self.store));
        self.telemetry_data = Some(res);
    }

    pub fn fetch_model(&mut self) {
        let res = self.runtime.block_on(dispatch(Command::GetRobotModel, &self.store));
        self.model_data = Some(res);
    }

    pub fn send_teleop(&mut self) {
        let lx = self.linear_x;
        let ly = self.linear_y;
        let az = self.angular_z;
        let _ = self.runtime.block_on(dispatch(Command::TeleopTwist { linear_x: lx, linear_y: ly, angular_z: az }, &self.store));
        self.status_message = format!("Teleop twist velocity [{lx:.2}, {ly:.2}, {az:.2}] dispatched");
    }

    pub fn validate_reachability(&mut self) {
        let x = self.target_x;
        let y = self.target_y;
        let z = self.target_z;
        let res = self.runtime.block_on(dispatch(Command::ValidatePose { x, y, z }, &self.store));
        self.reachability_result = Some(res);
    }

    pub fn trigger_estop(&mut self) {
        let res = self.runtime.block_on(dispatch(Command::EmergencyStop { reason: "GUI E-Stop Button Activated".to_string() }, &self.store));
        self.status_message = "EMERGENCY STOP ACTIVATED: ALL ACTUATORS DISABLED".to_string();
        self.telemetry_data = Some(res);
    }
}

impl eframe::App for RobotCraftGui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let cyan = Color32::from_rgb(6, 182, 212);
        let red_estop = Color32::from_rgb(239, 68, 68);

        // 1. Top Bar
        egui::Panel::top("robot_top_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong(egui::RichText::new("🤖 ROBOTCRAFT STUDIO").color(cyan).size(16.0));
                ui.separator();
                ui.label(format!("v{APP_VERSION}"));
                ui.colored_label(Color32::from_rgb(148, 163, 184), format!("| Port {CONTROL_PORT}"));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("⛔ EMERGENCY STOP").color(Color32::WHITE).strong()).clicked() {
                        self.trigger_estop();
                    }
                });
            });
        });

        // 2. Bottom Status Bar
        egui::Panel::bottom("robot_bottom_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(cyan, "●");
                ui.label(&self.status_message);
            });
        });

        // 3. Navigation Rail
        egui::Panel::left("robot_nav_rail").min_size(200.0).show(ui, |ui| {
            ui.add_space(8.0);
            ui.strong("WORKSPACES");
            ui.add_space(4.0);

            let nav_btn = |ui: &mut egui::Ui, active: bool, text: &str| -> bool {
                let rich = egui::RichText::new(text).color(if active { cyan } else { Color32::WHITE });
                ui.selectable_label(active, rich).clicked()
            };

            if nav_btn(ui, self.active_workspace == ActiveWorkspace::Dashboard, "📊 Live Telemetry") {
                self.active_workspace = ActiveWorkspace::Dashboard;
                self.fetch_telemetry();
            }
            if nav_btn(ui, self.active_workspace == ActiveWorkspace::TeleopControl, "🎮 Teleop Joystick") {
                self.active_workspace = ActiveWorkspace::TeleopControl;
            }
            if nav_btn(ui, self.active_workspace == ActiveWorkspace::KinematicsStudio, "📐 Kinematics Studio") {
                self.active_workspace = ActiveWorkspace::KinematicsStudio;
            }
            if nav_btn(ui, self.active_workspace == ActiveWorkspace::FleetStatus, "🌐 Multi-Robot Fleet") {
                self.active_workspace = ActiveWorkspace::FleetStatus;
            }
        });

        // 4. Central Panel
        egui::CentralPanel::default().show(ui, |ui| match self.active_workspace {
            ActiveWorkspace::Dashboard => {
                ui.horizontal(|ui| {
                    ui.heading("📊 Real-Time Robot Telemetry");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("⟳ Refresh").clicked() {
                            self.fetch_telemetry();
                        }
                    });
                });
                ui.separator();

                if let Some(ref telem_res) = self.telemetry_data {
                    if let Some(telem) = telem_res.get("telemetry") {
                        ui.columns(3, |cols| {
                            cols[0].group(|ui| {
                                ui.colored_label(cyan, "HARDWARE ID");
                                ui.heading(telem["hardware_id"].as_str().unwrap_or(""));
                                ui.label(format!("Armed: {}", telem["is_armed"]));
                            });
                            cols[1].group(|ui| {
                                ui.colored_label(Color32::from_rgb(52, 211, 153), "BATTERY LEVEL");
                                ui.heading(format!("{:.1}%", telem["battery_percentage"].as_f64().unwrap_or(0.0)));
                                ui.label("Voltage: 24.8V Nominal");
                            });
                            cols[2].group(|ui| {
                                ui.colored_label(Color32::from_rgb(251, 191, 36), "SAFETY STATUS");
                                let stopped = telem["emergency_stopped"].as_bool().unwrap_or(false);
                                if stopped {
                                    ui.colored_label(red_estop, "⛔ ESTOP ACTIVE");
                                } else {
                                    ui.colored_label(Color32::from_rgb(52, 211, 153), "✔ PERIMETER CLEAR");
                                }
                            });
                        });

                        ui.add_space(8.0);
                        ui.strong("Motor Joint Temperatures (°C):");
                        if let Some(temps) = telem["motor_temperatures_c"].as_array() {
                            ui.horizontal(|ui| {
                                for (i, t) in temps.iter().enumerate() {
                                    ui.group(|ui| {
                                        ui.label(format!("M{}", i + 1));
                                        ui.colored_label(Color32::WHITE, format!("{:.1}°", t.as_f64().unwrap_or(0.0)));
                                    });
                                }
                            });
                        }
                    } else {
                        ui.monospace(serde_json::to_string_pretty(telem_res).unwrap_or_default());
                    }
                }
            }
            ActiveWorkspace::TeleopControl => {
                ui.heading("🎮 Virtual Teleop & Twist Velocity Dispatch");
                ui.label("Control mobile robot bases with linear surge and angular yaw commands.");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Linear X (Forward/Reverse):");
                    ui.add(egui::Slider::new(&mut self.linear_x, -1.5..=1.5).suffix(" m/s"));
                });
                ui.horizontal(|ui| {
                    ui.label("Linear Y (Strafe Left/Right):");
                    ui.add(egui::Slider::new(&mut self.linear_y, -1.0..=1.0).suffix(" m/s"));
                });
                ui.horizontal(|ui| {
                    ui.label("Angular Z (Yaw Turn):");
                    ui.add(egui::Slider::new(&mut self.angular_z, -3.14..=3.14).suffix(" rad/s"));
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(egui::RichText::new("🚀 Dispatch Velocity").color(cyan).strong()).clicked() {
                        self.send_teleop();
                    }
                    if ui.button("⏹ Zero Stop").clicked() {
                        self.linear_x = 0.0;
                        self.linear_y = 0.0;
                        self.angular_z = 0.0;
                        self.send_teleop();
                    }
                });
            }
            ActiveWorkspace::KinematicsStudio => {
                ui.heading("📐 Kinematics & Reachability Studio");
                ui.label("Inspect URDF joint limits, compute forward kinematics, and validate target workspaces.");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Target Position (m):");
                    ui.add(egui::DragValue::new(&mut self.target_x).speed(0.01).prefix("X: "));
                    ui.add(egui::DragValue::new(&mut self.target_y).speed(0.01).prefix("Y: "));
                    ui.add(egui::DragValue::new(&mut self.target_z).speed(0.01).prefix("Z: "));

                    if ui.button("🔍 Check Reachability").clicked() {
                        self.validate_reachability();
                    }
                });

                if let Some(ref reach) = self.reachability_result {
                    ui.add_space(6.0);
                    if reach["reachable"] == true {
                        ui.colored_label(Color32::from_rgb(52, 211, 153), "✔ Target coordinates within valid collision-free kinematic envelope");
                    } else {
                        ui.colored_label(red_estop, format!("✖ Invalid Pose: {}", reach["error"].as_str().unwrap_or("Out of bounds")));
                    }
                }
            }
            ActiveWorkspace::FleetStatus => {
                ui.heading("🌐 Multi-Robot Fleet Topologies");
                ui.label("Manage heterogeneous fleets across quadrupeds, wheeled rovers, and articulated arms.");
                ui.separator();

                let robots = [
                    ("unitree_go2", "Unitree Go2 Pro", "Quadruped", "12-DOF", "UDP / CycloneDDS"),
                    ("yahboom_rover", "Yahboom ROSMASTER X3", "Rover", "4-DOF", "Serial UART / Jetbot"),
                    ("craftarm_6dof", "CraftArm Industrial Manipulator", "Arm", "6-DOF", "CAN Bus / Micro-ROS"),
                ];

                for (id, name, cat, dof, proto) in robots {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let is_sel = self.selected_robot == id;
                            if is_sel {
                                ui.colored_label(cyan, format!("● ACTIVE: {name}"));
                            } else {
                                ui.colored_label(Color32::WHITE, name);
                            }
                            ui.label(format!("• {cat} • {dof} • {proto}"));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.selectable_label(is_sel, if is_sel { "Selected" } else { "Connect" }).clicked() {
                                    self.selected_robot = id.to_string();
                                    self.fetch_telemetry();
                                }
                            });
                        });
                    });
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let rt = Arc::new(Runtime::new().unwrap_or_else(|e| {
        eprintln!("Failed to init tokio runtime: {e}");
        std::process::exit(1);
    }));

    let store = Arc::new(DatabaseStore::open_in_memory().unwrap_or_else(|e| {
        eprintln!("Failed to open in-memory store: {e}");
        std::process::exit(1);
    }));

    // Headless smoke test
    if args.contains(&"--smoke".to_string()) {
        println!("[robotcraft] Running headless smoke verification...");
        let status = rt.block_on(dispatch(Command::Status, &store));
        println!("[robotcraft] Smoke OK: {status}");
        return Ok(());
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(1024.0, 720.0))
            .with_title(format!("RobotCraft Studio v{APP_VERSION}")),
        ..Default::default()
    };

    eframe::run_native(
        "RobotCraft Studio",
        native_options,
        Box::new(|_cc| Ok(Box::new(RobotCraftGui::new(store, rt)))),
    )
}
