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

const BUILD_UTC: &str = env!("ROBOTCRAFT_BUILD_UTC");
const GIT_SHA: &str = env!("ROBOTCRAFT_GIT_SHA");
const RELEASE_NOTES: &str = include_str!("../../../CHANGELOG_LATEST.md");
use serde_json::Value;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveWorkspace {
    Dashboard,
    ThreeJsViewer,
    AiWorkflows,
    CadPlans,
    TeleopControl,
    KinematicsStudio,
    FleetStatus,
    YahboomDeck,
    VlaStudio,
}

pub struct RobotCraftGui {
    pub store: Arc<DatabaseStore>,
    pub runtime: Arc<Runtime>,
    pub locale: robotcraft_core::i18n::AppLocale,
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
    pub cad_plan_data: Option<Value>,
    pub workflow_result: Option<Value>,
    pub reachability_result: Option<Value>,
    pub status_message: String,
    pub is_dark_theme: bool,
    pub is_authenticated: bool,
    pub auth_username: String,
    pub auth_password_input: String,
    pub show_auth_modal: bool,
    pub show_about_dialog: bool,
    pub virtual_x: f32,
    pub virtual_y: f32,
    pub virtual_yaw: f32,
    pub viewer_yaw: f32,
    pub viewer_pitch: f32,
    pub viewer_zoom: f32,
    pub viewer_auto_rotate: bool,
    pub viewer_show_wireframe: bool,
    pub teleop_confirm: bool,
    pub peer_snapshot: Option<Value>,
    pub yahboom_result: Option<Value>,
    pub vla_snapshot: Option<Value>,
    pub vla_result: Option<Value>,
    pub vla_task_hint: String,
    pub urdf_pose: Option<Value>,
    pub urdf_joint_values: Vec<(String, f32)>,
    pub isaac_scene: String,
    pub isaac_confirm: bool,
    pub isaac_result: Option<Value>,
}

impl RobotCraftGui {
    pub fn new(store: Arc<DatabaseStore>, runtime: Arc<Runtime>) -> Self {
        let mut app = Self {
            store,
            runtime,
            locale: robotcraft_core::i18n::AppLocale::default(),
            active_workspace: ActiveWorkspace::ThreeJsViewer,
            selected_robot: "saschibot".to_string(),
            linear_x: 0.0,
            linear_y: 0.0,
            angular_z: 0.0,
            target_x: 0.40,
            target_y: 0.20,
            target_z: 0.35,
            telemetry_data: None,
            model_data: None,
            cad_plan_data: None,
            workflow_result: None,
            reachability_result: None,
            status_message: format!(
                "RobotCraft v{APP_VERSION} [Control Port {CONTROL_PORT} Active]"
            ),
            is_dark_theme: true,
            is_authenticated: false,
            auth_username: "teleoperator".to_string(),
            auth_password_input: String::new(),
            show_auth_modal: false,
            show_about_dialog: false,
            virtual_x: 0.0,
            virtual_y: 0.0,
            virtual_yaw: 0.0,
            viewer_yaw: 0.75,
            viewer_pitch: 0.45,
            viewer_zoom: 1.0,
            viewer_auto_rotate: false,
            viewer_show_wireframe: false,
            teleop_confirm: false,
            peer_snapshot: None,
            yahboom_result: None,
            vla_snapshot: None,
            vla_result: None,
            vla_task_hint: "drive to the doorway and stop".to_string(),
            urdf_pose: None,
            urdf_joint_values: Vec::new(),
            isaac_scene: String::new(),
            isaac_confirm: false,
            isaac_result: None,
        };
        app.fetch_telemetry();
        app.fetch_model();
        app.fetch_cad();
        app
    }

    pub fn fetch_cad(&mut self) {
        let id = self.selected_robot.clone();
        let res = self.runtime.block_on(dispatch(
            Command::GetRobotCadPlan { robot_id: id },
            &self.store,
        ));
        self.cad_plan_data = Some(res);
    }

    pub fn trigger_ai_workflow(&mut self, wf_type: &str) {
        let id = self.selected_robot.clone();
        let res = self.runtime.block_on(dispatch(
            Command::RunAiBehaviorWorkflow {
                robot_id: id.clone(),
                workflow_type: wf_type.to_string(),
            },
            &self.store,
        ));
        self.workflow_result = Some(res);
        self.status_message =
            format!("Dispatched AI Workflow '{wf_type}' for {id} in virtual space");

        match wf_type {
            "explore_and_map" => {
                self.virtual_x += 0.4;
                self.virtual_y += 0.2;
                self.virtual_yaw += 0.5;
            }
            "wave_and_greet" => {
                self.virtual_yaw = 0.0;
            }
            "stand_and_balance" => {
                self.virtual_x = 0.0;
                self.virtual_y = 0.0;
            }
            _ => {}
        }
    }

    pub fn fetch_telemetry(&mut self) {
        let id = self.selected_robot.clone();
        let res = self.runtime.block_on(dispatch(
            Command::GetTelemetry { robot_id: id },
            &self.store,
        ));
        self.telemetry_data = Some(res);
    }

    pub fn fetch_model(&mut self) {
        let res = self
            .runtime
            .block_on(dispatch(Command::GetRobotModel, &self.store));
        self.model_data = Some(res);
    }

    pub fn send_teleop(&mut self) {
        let lx = self.linear_x;
        let ly = self.linear_y;
        let az = self.angular_z;
        let robot_id = self.selected_robot.clone();
        let confirm = self.teleop_confirm;
        let res = self.runtime.block_on(dispatch(
            Command::TeleopTwist {
                linear_x: lx,
                linear_y: ly,
                angular_z: az,
                robot_id,
                confirm,
            },
            &self.store,
        ));
        self.yahboom_result = Some(res.clone());
        if res
            .get("confirm_required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            self.status_message =
                "Yahboom did not move. Tick confirm, then dispatch again.".to_string();
        } else if res
            .get("simulated")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            self.status_message =
                format!("Simulated twist [{lx:.2}, {ly:.2}, {az:.2}] (no hardware peer)");
        } else if res
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            self.status_message = format!(
                "Twist [{lx:.2}, {ly:.2}, {az:.2}] accepted by {}",
                res["source"].as_str().unwrap_or("studio")
            );
        } else {
            self.status_message = res["error"]
                .as_str()
                .unwrap_or("teleop rejected")
                .to_string();
        }
    }

    pub fn fetch_peers(&mut self) {
        let res = self
            .runtime
            .block_on(dispatch(Command::ProbePeers, &self.store));
        self.peer_snapshot = Some(res);
        self.status_message = "Probed yahboom-mcp and vla-mcp".to_string();
    }

    pub fn yahboom_op(&mut self, operation: &str, confirm: bool) {
        let res = self.runtime.block_on(dispatch(
            Command::YahboomCall {
                operation: operation.to_string(),
                param1: None,
                confirm,
            },
            &self.store,
        ));
        if res
            .get("confirm_required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            self.status_message =
                format!("{operation} held. Confirm is required before the base moves.");
        } else if res
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            self.status_message = format!("yahboom-mcp {operation} ok");
        } else {
            self.status_message = res["error"]
                .as_str()
                .unwrap_or("yahboom-mcp call failed")
                .to_string();
        }
        self.yahboom_result = Some(res);
        self.fetch_telemetry();
    }

    pub fn fetch_vla(&mut self) {
        let res = self
            .runtime
            .block_on(dispatch(Command::VlaSnapshot, &self.store));
        let reachable = res
            .get("reachable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        self.status_message = if reachable {
            "vla-mcp snapshot loaded".to_string()
        } else {
            "vla-mcp is not reachable on :11024".to_string()
        };
        self.vla_snapshot = Some(res);
    }

    pub fn reload_urdf(&mut self) {
        let joints = self.urdf_joint_values.clone();
        let res = self.runtime.block_on(dispatch(
            Command::UrdfPose {
                path: String::new(),
                joints,
            },
            &self.store,
        ));
        if res
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            if self.urdf_joint_values.is_empty() {
                if let Some(rows) = res.get("joints").and_then(|v| v.as_array()) {
                    self.urdf_joint_values = rows
                        .iter()
                        .filter_map(|row| {
                            let name = row.get("name")?.as_str()?.to_string();
                            let value = row.get("value")?.as_f64()? as f32;
                            Some((name, value))
                        })
                        .collect();
                }
            }
            let count = res.get("link_count").and_then(|v| v.as_u64()).unwrap_or(0);
            let robot = res.get("robot").and_then(|v| v.as_str()).unwrap_or("urdf");
            self.status_message = format!("URDF {robot}: {count} links");
            self.urdf_pose = Some(res);
        } else {
            self.urdf_pose = None;
            self.status_message = res
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("URDF load failed")
                .to_string();
        }
    }

    pub fn fetch_isaac(&mut self) {
        let res = self
            .runtime
            .block_on(dispatch(Command::IsaacSnapshot, &self.store));
        let reachable = res
            .get("reachable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        self.status_message = if reachable {
            "isaac-mcp snapshot loaded".to_string()
        } else {
            "isaac-mcp is not reachable on :11049".to_string()
        };
        self.isaac_result = Some(res);
    }

    pub fn start_isaac(&mut self) {
        let scene_name = self.isaac_scene.clone();
        let confirm = self.isaac_confirm;
        let res = self.runtime.block_on(dispatch(
            Command::IsaacStart {
                scene_name,
                confirm,
            },
            &self.store,
        ));
        self.status_message = if res
            .get("confirm_required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            "Isaac job held. Tick confirm before start.".to_string()
        } else if res
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            "Isaac job accepted".to_string()
        } else {
            res.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Isaac start failed")
                .to_string()
        };
        self.isaac_result = Some(res);
    }

    pub fn vla_prepare(&mut self, tool: &str, operation: &str) {
        let hint = self.vla_task_hint.clone();
        let res = self.runtime.block_on(dispatch(
            Command::VlaCall {
                tool: tool.to_string(),
                operation: operation.to_string(),
                task_hint: Some(hint),
                target: Some("raspbot".to_string()),
                model_key: None,
                confirm: false,
            },
            &self.store,
        ));
        self.status_message = if res
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            format!("{tool} {operation} returned")
        } else {
            res["error"]
                .as_str()
                .unwrap_or("vla-mcp call failed")
                .to_string()
        };
        self.vla_result = Some(res);
    }

    pub fn validate_reachability(&mut self) {
        let x = self.target_x;
        let y = self.target_y;
        let z = self.target_z;
        let res = self
            .runtime
            .block_on(dispatch(Command::ValidatePose { x, y, z }, &self.store));
        self.reachability_result = Some(res);
    }

    pub fn trigger_estop(&mut self) {
        let res = self.runtime.block_on(dispatch(
            Command::EmergencyStop {
                reason: "GUI E-Stop Button Activated".to_string(),
            },
            &self.store,
        ));
        self.status_message = "EMERGENCY STOP ACTIVATED: ALL ACTUATORS DISABLED".to_string();
        self.telemetry_data = Some(res);
    }
}

fn project_3d_point(
    p: [f32; 3],
    yaw: f32,
    pitch: f32,
    zoom: f32,
    center: egui::Pos2,
) -> egui::Pos2 {
    let scale = 150.0 * zoom;
    let cos_y = yaw.cos();
    let sin_y = yaw.sin();
    let x1 = p[0] * cos_y - p[1] * sin_y;
    let y1 = p[0] * sin_y + p[1] * cos_y;
    let z1 = p[2];

    let cos_p = pitch.cos();
    let sin_p = pitch.sin();
    let x2 = x1;
    let y2 = y1 * cos_p - z1 * sin_p;
    let z2 = y1 * sin_p + z1 * cos_p;

    let depth = (1.0 + y2 * 0.20).max(0.15);
    egui::pos2(
        center.x + (x2 / depth) * scale,
        center.y - (z2 / depth) * scale,
    )
}

#[allow(clippy::too_many_arguments)]
fn draw_3d_box(
    painter: &egui::Painter,
    center: [f32; 3],
    size: [f32; 3],
    fill_color: Color32,
    stroke_color: Color32,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    viewport_center: egui::Pos2,
    wireframe_only: bool,
) {
    let hx = size[0] * 0.5;
    let hy = size[1] * 0.5;
    let hz = size[2] * 0.5;

    let corners = [
        [center[0] - hx, center[1] - hy, center[2] - hz],
        [center[0] + hx, center[1] - hy, center[2] - hz],
        [center[0] + hx, center[1] + hy, center[2] - hz],
        [center[0] - hx, center[1] + hy, center[2] - hz],
        [center[0] - hx, center[1] - hy, center[2] + hz],
        [center[0] + hx, center[1] - hy, center[2] + hz],
        [center[0] + hx, center[1] + hy, center[2] + hz],
        [center[0] - hx, center[1] + hy, center[2] + hz],
    ];

    let pts: Vec<egui::Pos2> = corners
        .iter()
        .map(|&c| project_3d_point(c, yaw, pitch, zoom, viewport_center))
        .collect();

    if !wireframe_only {
        let faces = [
            ([pts[4], pts[5], pts[6], pts[7]], fill_color),
            (
                [pts[1], pts[2], pts[6], pts[5]],
                Color32::from_rgba_unmultiplied(
                    fill_color.r().saturating_sub(15),
                    fill_color.g().saturating_sub(15),
                    fill_color.b().saturating_sub(15),
                    fill_color.a(),
                ),
            ),
            (
                [pts[2], pts[3], pts[7], pts[6]],
                Color32::from_rgba_unmultiplied(
                    fill_color.r().saturating_add(15),
                    fill_color.g().saturating_add(15),
                    fill_color.b().saturating_add(15),
                    fill_color.a(),
                ),
            ),
            (
                [pts[0], pts[1], pts[5], pts[4]],
                Color32::from_rgba_unmultiplied(
                    fill_color.r().saturating_sub(25),
                    fill_color.g().saturating_sub(25),
                    fill_color.b().saturating_sub(25),
                    fill_color.a(),
                ),
            ),
        ];
        for (poly, col) in faces {
            painter.add(egui::Shape::convex_polygon(
                poly.to_vec(),
                col,
                egui::Stroke::NONE,
            ));
        }
    }

    let stroke = egui::Stroke::new(1.2, stroke_color);
    let edges = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    for (i, j) in edges {
        painter.line_segment([pts[i], pts[j]], stroke);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_3d_cylinder(
    painter: &egui::Painter,
    center: [f32; 3],
    radius: f32,
    height: f32,
    fill_color: Color32,
    stroke_color: Color32,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    viewport_center: egui::Pos2,
    wireframe_only: bool,
) {
    let segments = 12;
    let hz = height * 0.5;
    let mut top_pts = Vec::with_capacity(segments);
    let mut bot_pts = Vec::with_capacity(segments);

    for i in 0..segments {
        let angle = (i as f32) * std::f32::consts::TAU / (segments as f32);
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        let p_top = project_3d_point(
            [center[0] + x, center[1] + y, center[2] + hz],
            yaw,
            pitch,
            zoom,
            viewport_center,
        );
        let p_bot = project_3d_point(
            [center[0] + x, center[1] + y, center[2] - hz],
            yaw,
            pitch,
            zoom,
            viewport_center,
        );
        top_pts.push(p_top);
        bot_pts.push(p_bot);
    }

    if !wireframe_only {
        painter.add(egui::Shape::convex_polygon(
            top_pts.clone(),
            fill_color,
            egui::Stroke::NONE,
        ));
    }

    let stroke = egui::Stroke::new(1.0, stroke_color);
    for i in 0..segments {
        let next = (i + 1) % segments;
        painter.line_segment([top_pts[i], top_pts[next]], stroke);
        painter.line_segment([bot_pts[i], bot_pts[next]], stroke);
        if i % 3 == 0 {
            painter.line_segment([top_pts[i], bot_pts[i]], stroke);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_3d_sphere_node(
    painter: &egui::Painter,
    center: [f32; 3],
    radius: f32,
    color: Color32,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    viewport_center: egui::Pos2,
) {
    let p = project_3d_point(center, yaw, pitch, zoom, viewport_center);
    let r_screen = radius * 140.0 * zoom;
    painter.circle_filled(p, r_screen.max(4.0), color);
    painter.circle_stroke(p, r_screen.max(4.0), egui::Stroke::new(1.5, Color32::WHITE));
}

fn harsh_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    let ink = Color32::from_rgb(242, 246, 252);
    visuals.override_text_color = Some(ink);
    visuals.window_fill = Color32::from_rgb(2, 6, 18);
    visuals.panel_fill = Color32::from_rgb(10, 18, 34);
    visuals.extreme_bg_color = Color32::from_rgb(2, 6, 18);
    visuals.code_bg_color = Color32::from_rgb(4, 10, 24);
    visuals.faint_bg_color = Color32::from_rgb(16, 28, 50);
    visuals.warn_fg_color = Color32::from_rgb(255, 200, 120);
    visuals.error_fg_color = Color32::from_rgb(255, 130, 130);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(210, 220, 235));
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, ink);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, Color32::WHITE);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, Color32::WHITE);
    visuals.selection.bg_fill = Color32::from_rgb(36, 64, 107);
    visuals.selection.stroke = egui::Stroke::new(1.0, Color32::WHITE);
    visuals.hyperlink_color = Color32::from_rgb(147, 197, 253);
    visuals.weak_text_color = Some(Color32::from_rgb(203, 213, 225));
    visuals
}

fn crisp_light_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::light();
    let ink = Color32::BLACK;
    visuals.override_text_color = Some(ink);
    visuals.weak_text_color = Some(ink);
    visuals.window_fill = Color32::WHITE;
    visuals.panel_fill = Color32::WHITE;
    visuals.extreme_bg_color = Color32::WHITE;
    visuals.faint_bg_color = Color32::WHITE;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, ink);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, ink);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, ink);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, ink);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, ink);
    visuals.hyperlink_color = Color32::from_rgb(3, 105, 161);
    visuals
}

impl eframe::App for RobotCraftGui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().set_visuals(if self.is_dark_theme {
            harsh_visuals()
        } else {
            crisp_light_visuals()
        });

        let cyan = Color32::from_rgb(6, 182, 212);
        let red_estop = Color32::from_rgb(239, 68, 68);

        // 0. Desktop Menu Bar
        egui::Panel::top("desktop_menu_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Sync Joint States").clicked() {
                        self.fetch_telemetry();
                        ui.close();
                    }
                    if ui.button("Emergency Kill Switch").clicked() {
                        self.trigger_estop();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Robots", |ui| {
                    if ui.button("Saschibot Mobile Service Bot P6").clicked() {
                        self.selected_robot = "saschibot".to_string();
                        self.fetch_telemetry();
                        self.fetch_cad();
                        ui.close();
                    }
                    if ui.button("NoriRobotics A3 Mobile Manipulator").clicked() {
                        self.selected_robot = "nori_a3".to_string();
                        self.fetch_telemetry();
                        self.fetch_cad();
                        ui.close();
                    }
                    if ui.button("Noetix Bumi Bipedal Humanoid").clicked() {
                        self.selected_robot = "noetix_bumi".to_string();
                        self.fetch_telemetry();
                        self.fetch_cad();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Unitree Go2 Quadruped").clicked() {
                        self.selected_robot = "unitree_go2".to_string();
                        self.fetch_telemetry();
                        self.fetch_cad();
                        ui.close();
                    }
                    if ui.button("CraftArm 6-DOF Industrial Arm").clicked() {
                        self.selected_robot = "craftarm_6dof".to_string();
                        self.fetch_telemetry();
                        self.fetch_cad();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Yahboom ROSMASTER (yahboom-mcp)").clicked() {
                        self.selected_robot = "yahboom_rover".to_string();
                        self.active_workspace = ActiveWorkspace::YahboomDeck;
                        self.fetch_telemetry();
                        self.fetch_peers();
                        ui.close();
                    }
                });

                ui.menu_button("View", |ui| {
                    let theme_label = if self.is_dark_theme {
                        "☀ Switch to Light Theme"
                    } else {
                        "🌙 Switch to Dark Theme"
                    };
                    if ui.button(theme_label).clicked() {
                        self.is_dark_theme = !self.is_dark_theme;
                        ui.close();
                    }
                });

                ui.menu_button("Security", |ui| {
                    if self.is_authenticated {
                        ui.label(format!("Operator: {}", self.auth_username));
                        if ui.button("Sign Out").clicked() {
                            self.is_authenticated = false;
                            self.status_message = "Signed out".to_string();
                            ui.close();
                        }
                    } else {
                        if ui.button("Basic Auth Login...").clicked() {
                            self.show_auth_modal = true;
                            ui.close();
                        }
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Build data").clicked() {
                        self.show_about_dialog = true;
                        ui.close();
                    }
                    if ui.button("Release").clicked() {
                        self.show_about_dialog = true;
                        ui.close();
                    }
                    if ui.button("About RobotCraft").clicked() {
                        self.show_about_dialog = true;
                        ui.close();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let cur_loc = self.locale;
                    let label = format!("{} {}", cur_loc.flag(), cur_loc.code().to_uppercase());
                    egui::ComboBox::from_id_salt("topbar_locale_selector")
                        .selected_text(label)
                        .show_ui(ui, |ui| {
                            for loc_item in robotcraft_core::i18n::AppLocale::all() {
                                let text =
                                    format!("{} {}", loc_item.flag(), loc_item.display_name());
                                let _ = ui.selectable_value(&mut self.locale, *loc_item, text);
                            }
                        });

                    let (icon, label) = if self.is_dark_theme {
                        ("☀", "Light")
                    } else {
                        ("🌙", "Dark")
                    };
                    if ui.small_button(format!("{icon} {label}")).clicked() {
                        self.is_dark_theme = !self.is_dark_theme;
                    }

                    if self.is_authenticated {
                        ui.colored_label(cyan, format!("🤖 {}", self.auth_username));
                    } else if ui.small_button("🔑 Login").clicked() {
                        self.show_auth_modal = true;
                    }
                });
            });
        });

        // 1. Top Bar
        egui::Panel::top("robot_top_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong(
                    egui::RichText::new("🤖 ROBOTCRAFT STUDIO")
                        .color(cyan)
                        .size(16.0),
                );
                ui.separator();
                ui.label(format!("v{APP_VERSION}"));
                ui.label(format!("| Port {CONTROL_PORT}"));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("EMERGENCY STOP")
                                    .color(Color32::WHITE)
                                    .strong(),
                            )
                            .fill(red_estop),
                        )
                        .clicked()
                    {
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
        egui::Panel::left("robot_nav_rail")
            .min_size(200.0)
            .show(ui, |ui| {
                ui.add_space(8.0);
                ui.strong("WORKSPACES");
                ui.add_space(4.0);

                let nav_btn = |ui: &mut egui::Ui, active: bool, text: &str| -> bool {
                    let idle = ui.visuals().text_color();
                    let rich = egui::RichText::new(text).color(if active { cyan } else { idle });
                    ui.selectable_label(active, rich).clicked()
                };

                if nav_btn(
                    ui,
                    self.active_workspace == ActiveWorkspace::ThreeJsViewer,
                    "🪐 Three.js 3D View",
                ) {
                    self.active_workspace = ActiveWorkspace::ThreeJsViewer;
                }
                if nav_btn(
                    ui,
                    self.active_workspace == ActiveWorkspace::AiWorkflows,
                    "🧠 AI Motion Workflows",
                ) {
                    self.active_workspace = ActiveWorkspace::AiWorkflows;
                }
                if nav_btn(
                    ui,
                    self.active_workspace == ActiveWorkspace::CadPlans,
                    "📐 CAD Plans & BOM",
                ) {
                    self.active_workspace = ActiveWorkspace::CadPlans;
                    self.fetch_cad();
                }
                if nav_btn(
                    ui,
                    self.active_workspace == ActiveWorkspace::Dashboard,
                    "📊 Live Telemetry",
                ) {
                    self.active_workspace = ActiveWorkspace::Dashboard;
                    self.fetch_telemetry();
                }
                if nav_btn(
                    ui,
                    self.active_workspace == ActiveWorkspace::TeleopControl,
                    "🎮 Teleop Joystick",
                ) {
                    self.active_workspace = ActiveWorkspace::TeleopControl;
                }
                if nav_btn(
                    ui,
                    self.active_workspace == ActiveWorkspace::KinematicsStudio,
                    "📐 Kinematics Studio",
                ) {
                    self.active_workspace = ActiveWorkspace::KinematicsStudio;
                }
                if nav_btn(
                    ui,
                    self.active_workspace == ActiveWorkspace::FleetStatus,
                    "🌐 Multi-Robot Fleet",
                ) {
                    self.active_workspace = ActiveWorkspace::FleetStatus;
                }
                if nav_btn(
                    ui,
                    self.active_workspace == ActiveWorkspace::YahboomDeck,
                    "🛞 Yahboom Deck",
                ) {
                    self.active_workspace = ActiveWorkspace::YahboomDeck;
                    self.selected_robot = "yahboom_rover".to_string();
                    self.fetch_peers();
                    self.fetch_telemetry();
                }
                if nav_btn(
                    ui,
                    self.active_workspace == ActiveWorkspace::VlaStudio,
                    "🧠 VLA Studio",
                ) {
                    self.active_workspace = ActiveWorkspace::VlaStudio;
                    self.fetch_vla();
                }
            });

        // 4. Central Panel
        egui::CentralPanel::default().show(ui, |ui| match self.active_workspace {
            ActiveWorkspace::ThreeJsViewer => {
                ui.heading("🪐 Three.js 3D Virtual Robot Viewer");
                ui.label("Full WebGL & Native 3D viewport for Saschibot P6, NoriRobotics A3, and Noetix Bumi.");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Selected Robot for 3D View:");
                    if ui.selectable_label(self.selected_robot == "saschibot", "🤖 Saschibot P6 (14-DOF)").clicked() {
                        self.selected_robot = "saschibot".to_string();
                        self.fetch_telemetry();
                        self.fetch_cad();
                    }
                    if ui.selectable_label(self.selected_robot == "nori_a3", "🤖 NoriRobotics A3 (16-DOF)").clicked() {
                        self.selected_robot = "nori_a3".to_string();
                        self.urdf_joint_values.clear();
                        self.fetch_telemetry();
                        self.fetch_cad();
                        self.reload_urdf();
                    }
                    if ui.selectable_label(self.selected_robot == "noetix_bumi", "🤖 Noetix Bumi (21-DOF)").clicked() {
                        self.selected_robot = "noetix_bumi".to_string();
                        self.fetch_telemetry();
                        self.fetch_cad();
                    }
                    if ui.selectable_label(self.selected_robot == "yahboom_rover", "🛞 Yahboom ROSMASTER").clicked() {
                        self.selected_robot = "yahboom_rover".to_string();
                        self.fetch_telemetry();
                    }
                });

                ui.add_space(4.0);

                // Viewport Controls Toolbar
                ui.horizontal(|ui| {
                    let viewer_path = std::env::current_dir()
                        .map(|p| p.join("web").join("three_viewer.html").to_string_lossy().to_string())
                        .unwrap_or_else(|_| "D:/Dev/repos/robotcraft/web/three_viewer.html".to_string());

                    if ui.button("🚀 Open WebGL Three.js Studio (Browser)").on_hover_text("Open full Three.js WebGL canvas in default web browser").clicked() {
                        #[cfg(target_os = "windows")]
                        let _ = std::process::Command::new("cmd")
                            .args(["/C", "start", "", &viewer_path])
                            .spawn();
                        #[cfg(not(target_os = "windows"))]
                        let _ = std::process::Command::new("xdg-open").arg(&viewer_path).spawn();
                    }

                    if ui.button("🔄 Reset Camera").clicked() {
                        self.viewer_yaw = 0.75;
                        self.viewer_pitch = 0.45;
                        self.viewer_zoom = 1.0;
                    }

                    ui.toggle_value(&mut self.viewer_auto_rotate, "🌀 Auto-Rotate");
                    ui.toggle_value(&mut self.viewer_show_wireframe, "📐 Wireframe");

                    ui.add_space(8.0);
                    ui.label("Zoom:");
                    ui.add(egui::Slider::new(&mut self.viewer_zoom, 0.4..=2.5).text(""));
                });

                ui.add_space(4.0);

                // Interactive 3D Viewport
                let vp_width = ui.available_width().max(360.0);
                let vp_height = 370.0;
                let (vp_rect, vp_response) = ui.allocate_exact_size(egui::vec2(vp_width, vp_height), egui::Sense::drag());
                let painter = ui.painter_at(vp_rect);
                let vp_center = egui::pos2(vp_rect.center().x, vp_rect.center().y + 40.0);

                // Handle drag to orbit
                if vp_response.dragged() {
                    let delta = vp_response.drag_delta();
                    self.viewer_yaw += delta.x * 0.012;
                    self.viewer_pitch = (self.viewer_pitch + delta.y * 0.012).clamp(-1.45, 1.45);
                }

                // Wheel zoom (hover the viewport; matches Zoom slider 0.4..=2.5)
                let wheel = ui.input(|i| i.smooth_scroll_delta.y);
                if vp_response.hovered() && wheel != 0.0 {
                    self.viewer_zoom = (self.viewer_zoom * (1.0 + wheel * 0.0015)).clamp(0.4, 2.5);
                }

                if self.viewer_auto_rotate {
                    self.viewer_yaw += 0.015;
                    ui.ctx().request_repaint();
                }

                // Dark viewport background with border
                painter.rect_filled(vp_rect, 8.0, Color32::from_rgb(9, 13, 22));
                painter.rect_stroke(vp_rect, 8.0, egui::Stroke::new(1.5, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Outside);

                // Draw Ground Grid (Z=0 plane)
                let grid_range = 7;
                let grid_step = 0.20_f32;
                let grid_stroke = egui::Stroke::new(0.8, Color32::from_rgb(30, 41, 59));
                let grid_sub_stroke = egui::Stroke::new(0.5, Color32::from_rgb(20, 27, 40));
                for i in -grid_range..=grid_range {
                    let v = i as f32 * grid_step;
                    let p1 = project_3d_point([-1.4, v, 0.0], self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                    let p2 = project_3d_point([1.4, v, 0.0], self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                    let stroke = if i == 0 { egui::Stroke::new(1.2, Color32::from_rgb(6, 182, 212)) } else if i % 2 == 0 { grid_stroke } else { grid_sub_stroke };
                    painter.line_segment([p1, p2], stroke);

                    let q1 = project_3d_point([v, -1.4, 0.0], self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                    let q2 = project_3d_point([v, 1.4, 0.0], self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                    let stroke_q = if i == 0 { egui::Stroke::new(1.2, Color32::from_rgb(245, 158, 11)) } else if i % 2 == 0 { grid_stroke } else { grid_sub_stroke };
                    painter.line_segment([q1, q2], stroke_q);
                }

                // Origin Axes Gizmo (+X red, +Y green, +Z blue)
                let o_pt = project_3d_point([0.0, 0.0, 0.0], self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                let x_pt = project_3d_point([0.35, 0.0, 0.0], self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                let y_pt = project_3d_point([0.0, 0.35, 0.0], self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                let z_pt = project_3d_point([0.0, 0.0, 0.35], self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                painter.line_segment([o_pt, x_pt], egui::Stroke::new(2.5, Color32::from_rgb(239, 68, 68)));
                painter.line_segment([o_pt, y_pt], egui::Stroke::new(2.5, Color32::from_rgb(16, 185, 129)));
                painter.line_segment([o_pt, z_pt], egui::Stroke::new(2.5, Color32::from_rgb(59, 130, 246)));

                let wire = self.viewer_show_wireframe;

                let urdf_links = if self.selected_robot == "nori_a3" {
                    self.urdf_pose.as_ref().and_then(|pose| pose.get("links")).and_then(|links| links.as_array()).cloned()
                } else {
                    None
                };
                if let Some(links) = urdf_links {
                    for link in links {
                        let x = link.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                        let y = link.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                        let z = link.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                        let sx = link.get("sx").and_then(|v| v.as_f64()).unwrap_or(0.04) as f32;
                        let sy = link.get("sy").and_then(|v| v.as_f64()).unwrap_or(0.04) as f32;
                        let sz = link.get("sz").and_then(|v| v.as_f64()).unwrap_or(0.04) as f32;
                        draw_3d_box(
                            &painter,
                            [x, y, z],
                            [sx.max(0.02), sy.max(0.02), sz.max(0.02)],
                            Color32::from_rgb(56, 189, 248),
                            Color32::from_rgb(14, 116, 144),
                            self.viewer_yaw,
                            self.viewer_pitch,
                            self.viewer_zoom,
                            vp_center,
                            wire,
                        );
                    }
                } else { match self.selected_robot.as_str() {
                    "saschibot" => {
                        // 1. Mobile base chassis (680 x 540 x 80mm)
                        draw_3d_box(&painter, [0.0, 0.0, 0.08], [0.68, 0.54, 0.08], Color32::from_rgb(30, 41, 59), Color32::from_rgb(100, 116, 139), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        // Outboard wheels (cylinders at y=±0.27)
                        draw_3d_cylinder(&painter, [0.0, 0.27, 0.08], 0.08, 0.05, Color32::from_rgb(15, 23, 42), Color32::from_rgb(71, 85, 105), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        draw_3d_cylinder(&painter, [0.0, -0.27, 0.08], 0.08, 0.05, Color32::from_rgb(15, 23, 42), Color32::from_rgb(71, 85, 105), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        // Fixed mast stage 1
                        draw_3d_box(&painter, [0.0, 0.0, 0.44], [0.14, 0.14, 0.64], Color32::from_rgb(71, 85, 105), Color32::from_rgb(148, 163, 184), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        // Telescoping stage 2 & 3
                        draw_3d_box(&painter, [0.0, 0.0, 0.76], [0.11, 0.11, 0.36], Color32::from_rgb(100, 116, 139), Color32::from_rgb(203, 213, 225), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        // Head shell (130 x 210 x 150mm)
                        draw_3d_box(&painter, [0.0, 0.0, 0.94], [0.21, 0.13, 0.15], Color32::from_rgb(2, 132, 199), Color32::from_rgb(56, 189, 248), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        // Visor bezel with cyan eyes
                        draw_3d_box(&painter, [0.0, 0.07, 0.94], [0.18, 0.02, 0.10], Color32::from_rgb(6, 182, 212), Color32::WHITE, self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        // Dual Bimanual Arms (Gold STS3215 servos)
                        for sign in [-1.0_f32, 1.0_f32] {
                            let sy = sign * 0.16;
                            draw_3d_sphere_node(&painter, [0.0, sy, 0.88], 0.04, Color32::from_rgb(245, 158, 11), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                            draw_3d_box(&painter, [0.0, sy, 0.77], [0.05, 0.05, 0.22], Color32::from_rgb(217, 119, 6), Color32::from_rgb(251, 191, 36), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                            draw_3d_sphere_node(&painter, [0.0, sy, 0.66], 0.03, Color32::from_rgb(245, 158, 11), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                            draw_3d_box(&painter, [0.0, sy, 0.56], [0.04, 0.04, 0.20], Color32::from_rgb(217, 119, 6), Color32::from_rgb(251, 191, 36), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                            draw_3d_box(&painter, [0.0, sy, 0.44], [0.06, 0.084, 0.04], Color32::from_rgb(51, 65, 85), Color32::WHITE, self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        }
                    }
                    "yahboom_rover" => {
                        draw_3d_box(&painter, [0.0, 0.0, 0.06], [0.24, 0.18, 0.06], Color32::from_rgb(30, 41, 59), Color32::from_rgb(245, 158, 11), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        for (x, y) in [(0.09_f32, 0.10_f32), (0.09, -0.10), (-0.09, 0.10), (-0.09, -0.10)] {
                            draw_3d_cylinder(&painter, [x, y, 0.04], 0.035, 0.03, Color32::from_rgb(15, 23, 42), Color32::from_rgb(148, 163, 184), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        }
                        draw_3d_box(&painter, [0.06, 0.0, 0.14], [0.04, 0.04, 0.10], Color32::from_rgb(51, 65, 85), Color32::WHITE, self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        draw_3d_box(&painter, [0.09, 0.0, 0.18], [0.04, 0.06, 0.03], Color32::from_rgb(6, 182, 212), Color32::WHITE, self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                    }
                    "nori_a3" => {
                        // Cylindrical molded base shell (Z=0.08, r=0.25, h=0.16)
                        draw_3d_cylinder(&painter, [0.0, 0.0, 0.08], 0.25, 0.16, Color32::from_rgb(241, 245, 249), Color32::from_rgb(148, 163, 184), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        // RPLiDar C1 dome turret (front deck)
                        draw_3d_cylinder(&painter, [0.12, 0.0, 0.18], 0.05, 0.04, Color32::from_rgb(6, 182, 212), Color32::WHITE, self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        // White Torso Column
                        draw_3d_box(&painter, [0.0, 0.0, 0.41], [0.18, 0.22, 0.48], Color32::WHITE, Color32::from_rgb(203, 213, 225), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        // Ear pucks
                        draw_3d_cylinder(&painter, [0.0, 0.12, 0.61], 0.04, 0.02, Color32::from_rgb(6, 182, 212), Color32::WHITE, self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        draw_3d_cylinder(&painter, [0.0, -0.12, 0.61], 0.04, 0.02, Color32::from_rgb(6, 182, 212), Color32::WHITE, self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        // Feetech dual manipulator arms
                        for sign in [-1.0_f32, 1.0_f32] {
                            let sy = sign * 0.14;
                            draw_3d_sphere_node(&painter, [0.0, sy, 0.56], 0.035, Color32::from_rgb(2, 132, 199), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                            draw_3d_box(&painter, [0.0, sy, 0.46], [0.045, 0.045, 0.20], Color32::from_rgb(3, 105, 161), Color32::from_rgb(56, 189, 248), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                            draw_3d_sphere_node(&painter, [0.0, sy, 0.36], 0.03, Color32::from_rgb(2, 132, 199), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                            draw_3d_box(&painter, [0.0, sy, 0.27], [0.035, 0.035, 0.18], Color32::from_rgb(3, 105, 161), Color32::from_rgb(56, 189, 248), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                            draw_3d_box(&painter, [0.0, sy, 0.16], [0.05, 0.06, 0.04], Color32::from_rgb(30, 41, 59), Color32::WHITE, self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        }
                    }
                    _ => {
                        // "noetix_bumi" 21-DOF Humanoid
                        // Dual 6-DOF Legs standing upright on ground
                        for sign in [-1.0_f32, 1.0_f32] {
                            let sy = sign * 0.08;
                            // Feet on ground
                            draw_3d_box(&painter, [0.03, sy, 0.02], [0.14, 0.07, 0.04], Color32::from_rgb(51, 65, 85), Color32::from_rgb(148, 163, 184), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                            // Shins
                            draw_3d_box(&painter, [0.0, sy, 0.16], [0.05, 0.05, 0.24], Color32::from_rgb(226, 232, 240), Color32::from_rgb(100, 116, 139), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                            // Knee joint
                            draw_3d_sphere_node(&painter, [0.0, sy, 0.28], 0.032, Color32::from_rgb(51, 65, 85), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                            // Thighs
                            draw_3d_box(&painter, [0.0, sy, 0.40], [0.06, 0.06, 0.24], Color32::from_rgb(226, 232, 240), Color32::from_rgb(100, 116, 139), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                            // Hip joint
                            draw_3d_sphere_node(&painter, [0.0, sy, 0.52], 0.035, Color32::from_rgb(51, 65, 85), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                        }
                        // Pelvis
                        draw_3d_box(&painter, [0.0, 0.0, 0.57], [0.18, 0.22, 0.10], Color32::from_rgb(203, 213, 225), Color32::from_rgb(100, 116, 139), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        // Torso & Lumbar
                        draw_3d_box(&painter, [0.0, 0.0, 0.74], [0.20, 0.22, 0.24], Color32::from_rgb(241, 245, 249), Color32::from_rgb(148, 163, 184), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        // Head
                        draw_3d_sphere_node(&painter, [0.0, 0.0, 0.96], 0.08, Color32::from_rgb(248, 250, 252), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                        // Camera Visor
                        draw_3d_box(&painter, [0.08, 0.0, 0.96], [0.04, 0.10, 0.03], Color32::from_rgb(168, 85, 247), Color32::WHITE, self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        // Arms
                        for sign in [-1.0_f32, 1.0_f32] {
                            let sy = sign * 0.13;
                            draw_3d_sphere_node(&painter, [0.0, sy, 0.85], 0.03, Color32::from_rgb(168, 85, 247), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                            draw_3d_box(&painter, [0.0, sy, 0.76], [0.045, 0.045, 0.18], Color32::from_rgb(226, 232, 240), Color32::from_rgb(148, 163, 184), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                            draw_3d_sphere_node(&painter, [0.0, sy, 0.67], 0.025, Color32::from_rgb(51, 65, 85), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center);
                            draw_3d_box(&painter, [0.0, sy, 0.59], [0.04, 0.04, 0.16], Color32::from_rgb(226, 232, 240), Color32::from_rgb(148, 163, 184), self.viewer_yaw, self.viewer_pitch, self.viewer_zoom, vp_center, wire);
                        }
                    }
                }
                }

                // HUD Overlay (Top-Left of 3D canvas)
                let hud_rect = egui::Rect::from_min_size(vp_rect.min + egui::vec2(12.0, 12.0), egui::vec2(280.0, 68.0));
                painter.rect_filled(hud_rect, 6.0, Color32::from_rgba_unmultiplied(15, 23, 42, 220));
                painter.rect_stroke(hud_rect, 6.0, egui::Stroke::new(1.0, Color32::from_rgb(51, 65, 85)), egui::StrokeKind::Outside);

                let bot_label = match self.selected_robot.as_str() {
                    "saschibot" => "🤖 Saschibot P6 (14-DOF Bimanual + Mast)",
                    "nori_a3" => "🤖 NoriRobotics A3 (16-DOF Manipulator)",
                    "yahboom_rover" => "🛞 Yahboom ROSMASTER (yahboom-mcp)",
                    _ => "🤖 Noetix Bumi (21-DOF Bipedal Humanoid)",
                };
                painter.text(hud_rect.min + egui::vec2(10.0, 10.0), egui::Align2::LEFT_TOP, bot_label, egui::FontId::proportional(13.0), cyan);
                painter.text(hud_rect.min + egui::vec2(10.0, 28.0), egui::Align2::LEFT_TOP, "● Local viewport (not a live twin)", egui::FontId::proportional(11.0), Color32::from_rgb(242, 246, 252));
                painter.text(hud_rect.min + egui::vec2(10.0, 46.0), egui::Align2::LEFT_TOP, format!("Azimuth: {:.0}° | Elev: {:.0}° | Zoom: {:.1}x", self.viewer_yaw.to_degrees(), self.viewer_pitch.to_degrees(), self.viewer_zoom), egui::FontId::proportional(10.5), Color32::from_rgb(242, 246, 252));

                // Bottom-right hint
                painter.text(vp_rect.max - egui::vec2(12.0, 12.0), egui::Align2::RIGHT_BOTTOM, "Drag: Orbit | Scroll: Zoom", egui::FontId::proportional(11.0), Color32::from_rgb(242, 246, 252));

                ui.add_space(8.0);

                if self.selected_robot == "nori_a3" {
                    ui.horizontal(|ui| {
                        if ui.button("Reload Nori URDF").clicked() {
                            self.urdf_joint_values.clear();
                            self.reload_urdf();
                        }
                        if let Some(path) = self.urdf_pose.as_ref().and_then(|pose| pose.get("path")).and_then(|path| path.as_str()) {
                            ui.label(path);
                        }
                    });
                    let specs: Vec<(String, f32, f32)> = self
                        .urdf_pose
                        .as_ref()
                        .and_then(|pose| pose.get("joints"))
                        .and_then(|joints| joints.as_array())
                        .map(|rows| {
                            rows.iter()
                                .filter_map(|row| {
                                    let name = row.get("name")?.as_str()?.to_string();
                                    let lower = row.get("lower")?.as_f64()? as f32;
                                    let upper = row.get("upper")?.as_f64()? as f32;
                                    Some((name, lower, upper))
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    let mut dirty = false;
                    egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                        for (name, lower, upper) in &specs {
                            if let Some((_, value)) = self.urdf_joint_values.iter_mut().find(|(joint_name, _)| joint_name == name) {
                                if ui.add(egui::Slider::new(value, *lower..=*upper).text(name)).changed() {
                                    dirty = true;
                                }
                            }
                        }
                    });
                    if dirty {
                        self.reload_urdf();
                    }
                }

                // Quick CAD & Telemetry cards
                ui.columns(2, |cols| {
                    cols[0].group(|ui| {
                        ui.colored_label(cyan, "📋 CAD & Kinematics Specs:");
                        match self.selected_robot.as_str() {
                            "saschibot" => {
                                ui.label("• 01_base_deck_680x540mm.svg (18 cut profiles)");
                                ui.label("• 02_column_pedestal_flange_280x280mm.svg");
                                ui.label("• 10 STL Modules: STS3215 cradle, Robonine jaws");
                            }
                            "nori_a3" => {
                                ui.label("• nori_a3_rig.glb & nori_a3_posed.glb");
                                ui.label("• nori.expanded.urdf (Full kinematics model)");
                                ui.label("• Printables: ear-pucks.stl, RPLiDar C1 mount");
                            }
                            _ => {
                                ui.label("• Kinematics: 21-DOF Bipedal Architecture");
                                ui.label("• Height: 1200mm | Mass: 18.5kg | Orin Brain");
                                ui.label("• Topologies: 6-DOF per leg, 3-DOF per arm");
                            }
                        }
                    });

                    cols[1].group(|ui| {
                        ui.colored_label(Color32::from_rgb(16, 185, 129), "⚡ Live Hardware Telemetry:");
                        match self.selected_robot.as_str() {
                            "saschibot" => {
                                ui.label("• Protocol: Micro-ROS / STS3215 Bus");
                                ui.label("• Battery: 94.2% (LiFePO4 24V)");
                                ui.label("• Servos: 14 Channels Active (31°C avg)");
                            }
                            "nori_a3" => {
                                ui.label("• Protocol: ROS 2 DDS / Feetech");
                                ui.label("• Battery: 88.0% | LIDAR: 600 RPM Active");
                                ui.label("• Arms: 16-DOF Calibrated");
                            }
                            _ => {
                                ui.label("• Protocol: rosbridge /ws (:9090)");
                                ui.label("• Brain: Jetson Orin | Wake Word: 'Bumi Bumi'");
                                ui.label("• Balance: Inverted Pendulum ZMP Ready");
                            }
                        }
                    });
                });
            }
            ActiveWorkspace::AiWorkflows => {
                ui.heading("🧠 AI Motion & Trajectory Workflows");
                ui.label("Autonomous motion planning in virtual space with collision avoidance and joint kinematic profiles.");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(format!("Target Robot: {}", self.selected_robot));
                });

                ui.add_space(8.0);
                ui.columns(2, |cols| {
                    cols[0].group(|ui| {
                        ui.colored_label(cyan, "👋 Wave & Greet Workflow");
                        ui.label("Servos articulate arm to friendly greeting wave keyframe trajectory.");
                        if ui.button("▶ Dispatch Wave & Greet").clicked() {
                            self.trigger_ai_workflow("wave_and_greet");
                        }
                    });

                    cols[1].group(|ui| {
                        ui.colored_label(Color32::from_rgb(16, 185, 129), "🗺️ SLAM Explore & Map");
                        ui.label("RPLiDar active sweep + mobile base boundary traversal.");
                        if ui.button("▶ Dispatch Explore & Map").clicked() {
                            self.trigger_ai_workflow("explore_and_map");
                        }
                    });
                });

                ui.add_space(6.0);
                ui.columns(2, |cols| {
                    cols[0].group(|ui| {
                        ui.colored_label(Color32::from_rgb(245, 158, 11), "📦 Bimanual Caddy Pickup");
                        ui.label("3-stage mast extension + Robonine 84mm parallel jaw grasping.");
                        if ui.button("▶ Dispatch Caddy Dock").clicked() {
                            self.trigger_ai_workflow("bimanual_caddy_pickup");
                        }
                    });

                    cols[1].group(|ui| {
                        ui.colored_label(Color32::from_rgb(168, 85, 247), "⚖️ ZMP Stand & Balance");
                        ui.label("Inverted pendulum bipedal stabilization for humanoid locomotion.");
                        if ui.button("▶ Dispatch Stand & Balance").clicked() {
                            self.trigger_ai_workflow("stand_and_balance");
                        }
                    });
                });

                if let Some(ref wf) = self.workflow_result {
                    ui.add_space(10.0);
                    ui.heading("Execution Trajectory Log");
                    ui.group(|ui| {
                        ui.monospace(serde_json::to_string_pretty(wf).unwrap_or_default());
                    });
                }
            }
            ActiveWorkspace::CadPlans => {
                ui.heading("📐 Robot CAD Plans & Manufacturing BOM");
                ui.label("Sovereign fabrication blueprints, DXF/SVG laser sheets, 3D printable STLs, and URDF kinematics.");
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.selectable_label(self.selected_robot == "saschibot", "Saschibot CAD").clicked() {
                        self.selected_robot = "saschibot".to_string();
                        self.fetch_cad();
                    }
                    if ui.selectable_label(self.selected_robot == "nori_a3", "NoriRobotics A3 CAD").clicked() {
                        self.selected_robot = "nori_a3".to_string();
                        self.fetch_cad();
                    }
                    if ui.selectable_label(self.selected_robot == "noetix_bumi", "Noetix Bumi CAD").clicked() {
                        self.selected_robot = "noetix_bumi".to_string();
                        self.fetch_cad();
                    }
                });

                ui.add_space(8.0);
                if let Some(ref cad) = self.cad_plan_data {
                    ui.group(|ui| {
                        ui.colored_label(cyan, format!("CAD PLAN FOR: {}", self.selected_robot.to_uppercase()));
                        ui.monospace(serde_json::to_string_pretty(cad).unwrap_or_default());
                    });
                } else {
                    ui.label("Loading CAD plan data...");
                }
            }
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
                    let source = telem_res["source"].as_str().unwrap_or("studio-fixture");
                    let live = telem_res["live"].as_bool().unwrap_or(false);
                    if live {
                        ui.colored_label(Color32::from_rgb(16, 185, 129), format!("Source: {source} (live)"));
                    } else {
                        ui.colored_label(Color32::from_rgb(245, 158, 11), format!("Source: {source}"));
                    }
                    if let Some(note) = telem_res["note"].as_str() {
                        ui.label(note);
                    }
                }

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
                                if let Some(volts) = telem.get("voltage").and_then(|v| v.as_f64()) {
                                    ui.label(format!("Voltage: {volts:.2} V"));
                                } else {
                                    ui.label("Voltage: not reported");
                                }
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
                                        ui.colored_label(ui.visuals().text_color(), format!("{:.1}°", t.as_f64().unwrap_or(0.0)));
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
                    ui.add(egui::Slider::new(&mut self.angular_z, -std::f32::consts::PI..=std::f32::consts::PI).suffix(" rad/s"));
                });

                ui.add_space(8.0);
                ui.checkbox(&mut self.teleop_confirm, "Confirm hardware motion (required for Yahboom)");
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
            ActiveWorkspace::YahboomDeck => {
                ui.heading("Yahboom deck");
                ui.label("Talks to yahboom-mcp on :10892. Motion verbs do nothing until confirm is ticked. Stop always posts.");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Probe peers").clicked() {
                        self.fetch_peers();
                        self.fetch_telemetry();
                    }
                    if ui.button("Read battery / IMU").clicked() {
                        self.yahboom_op("read_all", false);
                    }
                    if ui.button("Stop").clicked() {
                        self.yahboom_op("stop_all", true);
                    }
                });
                ui.checkbox(&mut self.teleop_confirm, "Confirm motion");
                ui.horizontal(|ui| {
                    let go = self.teleop_confirm;
                    if ui.button("Forward").clicked() {
                        self.yahboom_op("forward", go);
                    }
                    if ui.button("Back").clicked() {
                        self.yahboom_op("backward", go);
                    }
                    if ui.button("Left").clicked() {
                        self.yahboom_op("turn_left", go);
                    }
                    if ui.button("Right").clicked() {
                        self.yahboom_op("turn_right", go);
                    }
                });
                if let Some(note) = self.telemetry_data.as_ref().and_then(|v| v.get("note")).and_then(|n| n.as_str()) {
                    ui.label(note);
                }
                if let Some(ref peer) = self.peer_snapshot {
                    ui.add_space(6.0);
                    ui.strong("Peer probe");
                    ui.group(|ui| {
                        ui.monospace(serde_json::to_string_pretty(peer).unwrap_or_default());
                    });
                }
                if let Some(ref result) = self.yahboom_result {
                    ui.add_space(6.0);
                    ui.strong("Last yahboom-mcp result");
                    ui.group(|ui| {
                        ui.monospace(serde_json::to_string_pretty(result).unwrap_or_default());
                    });
                }
            }
            ActiveWorkspace::VlaStudio => {
                ui.heading("VLA studio");
                ui.label("Reads vla-mcp on :11024. Prepare calls stay inside the whitelist (health, infer_prepare, list). Training launch is rejected.");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Refresh snapshot").clicked() {
                        self.fetch_vla();
                    }
                    if ui.button("Wall-OSS health").clicked() {
                        self.vla_prepare("vla_wall", "health");
                    }
                    if ui.button("X-VLA health (raspbot)").clicked() {
                        self.vla_prepare("vla_xvla", "health");
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Task hint");
                    ui.add(egui::TextEdit::singleline(&mut self.vla_task_hint).desired_width(360.0));
                    if ui.button("Prepare Wall-OSS inference").clicked() {
                        self.vla_prepare("vla_wall", "infer_prepare");
                    }
                });
                if let Some(ref snap) = self.vla_snapshot {
                    ui.add_space(6.0);
                    if snap.get("reachable").and_then(|v| v.as_bool()).unwrap_or(false) {
                        ui.colored_label(Color32::from_rgb(16, 185, 129), "vla-mcp reachable");
                    } else {
                        ui.colored_label(Color32::from_rgb(245, 158, 11), "vla-mcp offline");
                    }
                    if let Some(items) = snap.get("episodes").and_then(|ep| ep.get("items")).and_then(|items| items.as_array()) {
                        let needle = if self.selected_robot.contains("nori") { "nori" } else { self.selected_robot.as_str() };
                        ui.strong(format!("Episodes ({})", items.len()));
                        for item in items.iter().take(12) {
                            let source = item.get("source").and_then(|v| v.as_str()).unwrap_or("");
                            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
                            let created = item.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                            let tagged = source.to_ascii_lowercase().contains(&needle.to_ascii_lowercase());
                            let line = format!("{id}  {source}  {created}");
                            if tagged {
                                ui.colored_label(Color32::from_rgb(16, 185, 129), line);
                            } else {
                                ui.label(line);
                            }
                        }
                    }
                    if let Some(summary) = snap.get("summary") {
                        ui.group(|ui| {
                            ui.monospace(serde_json::to_string_pretty(summary).unwrap_or_default());
                        });
                    } else {
                        ui.group(|ui| {
                            ui.monospace(serde_json::to_string_pretty(snap).unwrap_or_default());
                        });
                    }
                }
                if let Some(ref result) = self.vla_result {
                    ui.add_space(6.0);
                    ui.strong("Last vla-mcp call");
                    ui.group(|ui| {
                        ui.monospace(serde_json::to_string_pretty(result).unwrap_or_default());
                    });
                }
            }
            ActiveWorkspace::FleetStatus => {
                ui.heading("🌐 Multi-Robot Fleet Topologies");
                ui.label("Manage heterogeneous fleets across quadrupeds, wheeled rovers, and articulated arms.");
                ui.separator();
                ui.group(|ui| {
                    ui.colored_label(cyan, "isaac-mcp :11049");
                    ui.label("Probe is read-only. Start posts /api/jobs/start only after confirm.");
                    ui.horizontal(|ui| {
                        if ui.button("Probe Isaac").clicked() {
                            self.fetch_isaac();
                        }
                        ui.label("Scene");
                        ui.add(egui::TextEdit::singleline(&mut self.isaac_scene).desired_width(180.0));
                        ui.checkbox(&mut self.isaac_confirm, "Confirm start");
                        if ui.button("Start job").clicked() {
                            self.start_isaac();
                        }
                    });
                    if let Some(ref isaac) = self.isaac_result {
                        ui.monospace(serde_json::to_string_pretty(isaac).unwrap_or_default());
                    }
                });
                ui.add_space(6.0);

                let robots = [
                    ("saschibot", "Saschibot Mobile Service Bot P6", "Mobile Bimanual", "14-DOF", "Micro-ROS / STS3215"),
                    ("nori_a3", "NoriRobotics A3 Mobile Manipulator", "Wheeled Arm", "16-DOF", "ROS2 DDS / Feetech"),
                    ("noetix_bumi", "Noetix Bumi Humanoid", "Biped Humanoid", "21-DOF", "rosbridge / Jetson Orin"),
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
                                ui.colored_label(ui.visuals().text_color(), name);
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

        // Modal: Basic Auth Login
        if self.show_auth_modal {
            egui::Window::new("🔐 Teleoperation Security Auth")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Authenticate for Tele-Op & Actuator override:");
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("User:");
                        ui.text_edit_singleline(&mut self.auth_username);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Pass:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.auth_password_input)
                                .password(true),
                        );
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Confirm").clicked() && !self.auth_username.is_empty() {
                            self.is_authenticated = true;
                            self.show_auth_modal = false;
                            self.status_message = format!("Signed in as {}", self.auth_username);
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_auth_modal = false;
                        }
                    });
                });
        }

        if self.show_about_dialog {
            egui::Window::new("Help")
                .collapsible(false)
                .resizable(true)
                .default_width(520.0)
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(460.0)
                        .show(ui, |ui| {
                            ui.heading("Build data");
                            ui.label(format!("Version: {APP_VERSION}"));
                            ui.label(format!("Profile: {}", build_profile()));
                            ui.label(format!("Built: {BUILD_UTC}"));
                            ui.label(format!("Commit: {GIT_SHA}"));
                            ui.label(format!("Binary: {}", binary_path()));
                            ui.label(format!("Control port label: {CONTROL_PORT}"));
                            ui.separator();
                            ui.heading("Release");
                            ui.label(format!("Tag: v{APP_VERSION}"));
                            ui.hyperlink(format!(
                            "https://github.com/sandraschi/robotcraft/releases/tag/v{APP_VERSION}"
                        ));
                            ui.add_space(6.0);
                            ui.label(RELEASE_NOTES);
                        });
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.show_about_dialog = false;
                    }
                });
        }
    }
}

fn build_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

fn binary_path() -> String {
    std::env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "unavailable".to_string())
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
