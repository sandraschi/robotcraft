//! Help Center (10-item Craft standard), Command Palette, Keyboard Shortcuts,
//! `switch_on_hover`, and local-first `Ctrl+J` AI Chat Dock.

use crate::{ActiveWorkspace, RobotCraftGui, BUILD_UTC, GIT_SHA, RELEASE_NOTES};
use eframe::egui::{self, Color32};
use robotcraft_core::{
    dispatch,
    skills::{bundled_skills, get_skill},
    Command, APP_VERSION, CONTROL_PORT,
};

const HELP_MD: &str = include_str!("../../../HELP.md");
const PRD_MD: &str = include_str!("../../../docs/PRD.md");
const TIERED_PLAN_MD: &str = include_str!("../../../docs/TIERED_PLAN.md");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTab {
    GetStarted,
    Functions,
    Usage,
    TipsAndTricks,
    PitfallsAndSafety,
    PlannedExtensions,
    BundledSkills,
    ProjectDocs,
    Licenses,
    About,
}

impl HelpTab {
    pub fn label(self) -> &'static str {
        match self {
            Self::GetStarted => "Get Started",
            Self::Functions => "Functions",
            Self::Usage => "Usage (GUI/CLI/MCP)",
            Self::TipsAndTricks => "Tips & Tricks",
            Self::PitfallsAndSafety => "Pitfalls & Safety",
            Self::PlannedExtensions => "Planned Extensions",
            Self::BundledSkills => "Bundled Skills",
            Self::ProjectDocs => "Project Docs",
            Self::Licenses => "Licenses",
            Self::About => "About",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::GetStarted,
            Self::Functions,
            Self::Usage,
            Self::TipsAndTricks,
            Self::PitfallsAndSafety,
            Self::PlannedExtensions,
            Self::BundledSkills,
            Self::ProjectDocs,
            Self::Licenses,
            Self::About,
        ]
    }
}

/// SOTA `switch_on_hover` implementation for egui top menu bars.
pub fn switch_on_hover(ctx: &egui::Context, titles: &[egui::Response]) {
    let is_open =
        |t: &egui::Response| egui::Popup::is_id_open(ctx, egui::Popup::default_response_id(t));
    if !titles.iter().any(is_open) {
        return;
    }
    if let Some(title) = titles
        .iter()
        .find(|t| t.hovered() && !t.clicked() && !is_open(t))
    {
        egui::Popup::open_id(ctx, egui::Popup::default_response_id(title));
    }
}

pub fn handle_global_shortcuts(app: &mut RobotCraftGui, ctx: &egui::Context) {
    let mut open_palette = false;
    let mut open_shortcuts = false;
    let mut toggle_chat = false;

    ctx.input(|i| {
        let cmd = i.modifiers.command || i.modifiers.ctrl;
        if cmd
            && ((i.modifiers.shift && i.key_pressed(egui::Key::P)) || i.key_pressed(egui::Key::K))
        {
            open_palette = true;
        }
        if i.key_pressed(egui::Key::F1) || (cmd && i.key_pressed(egui::Key::Slash)) {
            open_shortcuts = true;
        }
        if cmd && i.key_pressed(egui::Key::J) {
            toggle_chat = true;
        }
    });

    if open_palette {
        app.show_command_palette = !app.show_command_palette;
        if app.show_command_palette {
            app.command_palette_query.clear();
        }
    }
    if open_shortcuts {
        app.show_shortcuts_dialog = !app.show_shortcuts_dialog;
    }
    if toggle_chat {
        app.show_ai_chat_dock = !app.show_ai_chat_dock;
    }
}

pub fn render_help_center(app: &mut RobotCraftGui, ctx: &egui::Context) {
    if !app.show_help_center {
        return;
    }

    let mut open = app.show_help_center;
    egui::Window::new("RobotCraft Help & Documentation")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(760.0)
        .default_height(540.0)
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for tab in HelpTab::all() {
                    if ui
                        .selectable_label(app.help_tab == *tab, tab.label())
                        .clicked()
                    {
                        app.help_tab = *tab;
                    }
                }
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(430.0)
                .show(ui, |ui| match app.help_tab {
                    HelpTab::GetStarted => render_get_started(ui),
                    HelpTab::Functions => render_functions(ui),
                    HelpTab::Usage => render_usage(ui),
                    HelpTab::TipsAndTricks => render_tips(ui),
                    HelpTab::PitfallsAndSafety => render_safety(ui),
                    HelpTab::PlannedExtensions => render_planned(ui),
                    HelpTab::BundledSkills => render_skills(app, ui),
                    HelpTab::ProjectDocs => render_project_docs(app, ui),
                    HelpTab::Licenses => render_licenses(ui),
                    HelpTab::About => render_about(ui),
                });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    app.show_help_center = false;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Shortcuts: Ctrl+K Palette | F1 Keys | Ctrl+J AI Dock");
                });
            });
        });

    if !open {
        app.show_help_center = false;
    }
}

fn render_get_started(ui: &mut egui::Ui) {
    ui.heading("What RobotCraft Is & 5-Step First-Run Tour");
    ui.add_space(4.0);
    ui.label("RobotCraft is the embodied intelligence studio and universal robotics workstation for the sandraschi fleet. Live hardware drivers and policies stay in sibling peer servers (yahboom-mcp, vla-mcp, isaac-mcp); RobotCraft unifies telemetry, URDF posing, planar kinematics, and confirm-gated control.");
    ui.add_space(8.0);
    for (step, detail) in [
        ("1. Launch & Smoke Check", "Run `start.ps1` for the studio, or `robotcraft --smoke` to verify `Command::Status` headlessly."),
        ("2. Probe Fleet Peers", "Open Yahboom Deck or Fleet Status (or run `robotcraft-cli peers`) to check yahboom-mcp (:10892), vla-mcp (:11024), and isaac-mcp (:11049) without moving hardware."),
        ("3. Select Robot & Pose URDF", "Pick NoriRobotics A3 or Yahboom ROSMASTER from the Robots menu. Drag joint sliders in the 3D View to re-pose Nori's URDF link frames."),
        ("4. Validate Kinematics & Dry-Run Teleop", "Check reachability (0.10 m to 0.85 m) in Kinematics Studio. Leave Confirm unchecked in Teleop/Yahboom Deck to inspect the `confirm_required` safety gate first."),
        ("5. Automate via CLI, MCP & AI Dock", "Use `robotcraft-cli --mcp` for stdio JSON-RPC 2026-07-28, or press Ctrl+J for the local-first AI Chat Dock."),
    ] {
        ui.strong(step);
        ui.label(detail);
        ui.add_space(4.0);
    }
}

fn render_functions(ui: &mut egui::Ui) {
    ui.heading("Core Capabilities & Architecture");
    ui.add_space(4.0);
    for (title, body) in [
        ("URDF Link Posing (robotcraft-core::urdf)", "Loads `nori.expanded.urdf` (or `ROBOTCRAFT_NORI_URDF`). Keeps revolute, continuous, and prismatic joints; clamps angles to [lower, upper]. Mesh STLs are not loaded—mesh-only links fall back to collision boxes or a 4 cm cube."),
        ("Planar Forward Kinematics & Reachability Sphere", "4-link planar FK starting at z = 0.20 m with link lengths [0.25, 0.25, 0.15, 0.10] m. `validate_target_pose` enforces 0.10 m (self-collision floor) <= d <= 0.85 m (max reach)."),
        ("Fleet HTTP Peers (robotcraft-hal::peers)", "yahboom-mcp (:10892), vla-mcp (:11024), isaac-mcp (:11049). Override URLs via `ROBOTCRAFT_YAHBOOM_URL`, `ROBOTCRAFT_VLA_URL`, `ROBOTCRAFT_ISAAC_URL` (set `off` to disable)."),
        ("Live vs. Studio Fixtures", "Only `yahboom_rover` queries live telemetry (`live: true`, `source: yahboom-mcp`). Saschibot, Nori A3, Bumi, Unitree Go2, CraftArm, and offline Yahboom return labeled `source: studio-fixture` snapshots."),
        ("Other Crafts Ecosystem", "Connects to yahboom-mcp, vla-mcp, isaac-mcp, norirobotics-mcp (URDF/STEP), rust-saschirob-mcp (DXF/STL), databasecraft (store), and sibling Storytold Craft studios (cadcraft, celcraft)."),
    ] {
        ui.strong(title);
        ui.label(body);
        ui.add_space(4.0);
    }
}

fn render_usage(ui: &mut egui::Ui) {
    ui.heading("Usage: Desktop GUI, CLI & MCP");
    ui.add_space(4.0);
    ui.strong("Desktop GUI");
    ui.label("Switch workspaces from the left rail or Command Palette (Ctrl+K). Use Ctrl+J to open the local-first AI Chat Dock, and F1 / Ctrl+/ for Keyboard Shortcuts.");
    ui.add_space(6.0);
    ui.strong("CLI (`robotcraft-cli`, JSON stdout)");
    ui.monospace(
        "robotcraft-cli status\n\
         robotcraft-cli peers\n\
         robotcraft-cli telemetry yahboom_rover\n\
         robotcraft-cli teleop --robot yahboom_rover --x 0.2 --confirm\n\
         robotcraft-cli fk 0.0 0.35 1.20 -0.45\n\
         robotcraft-cli urdf\n\
         robotcraft-cli vla-call vla_wall infer_prepare --task-hint \"drive to doorway\" --target raspbot\n\
         robotcraft-cli isaac-start --scene lab --confirm\n\
         robotcraft-cli estop --reason \"operator halt\"",
    );
    ui.add_space(6.0);
    ui.strong("MCP 2026-07-28 Stdio (`robotcraft-cli --mcp`)");
    ui.label("Tools: robot_status, robot_list, robot_telemetry, robot_teleop, robot_estop, robot_peers, yahboom_call, vla_snapshot, vla_call, urdf_pose, isaac_snapshot, isaac_start, robotcraft_skill_read.");
    ui.label("Resources: `resources/list` and `resources/read` expose `robotcraft://skills/<id>`.");
}

fn render_tips(ui: &mut egui::Ui) {
    ui.heading("Tips & Tricks");
    ui.add_space(4.0);
    for tip in [
        "Non-destructive dry runs: omit `--confirm` (or pass `confirm: false`) on Yahboom motion, VLA weight download, or Isaac job start to inspect `confirm_required` payloads before anything executes.",
        "Skip offline peer delays: HTTP peers use a 400 ms connect timeout. Set `ROBOTCRAFT_YAHBOOM_URL=off`, `ROBOTCRAFT_VLA_URL=off`, or `ROBOTCRAFT_ISAAC_URL=off` when working offline.",
        "Custom URDF inspection: set `ROBOTCRAFT_NORI_URDF=/path/to/robot.urdf` to pose any URDF in both `urdf_pose` and the 3D Viewport.",
        "Instant navigation: press Ctrl+K (or Ctrl+Shift+P) to jump to any workspace, probe peers, or open any Help tab.",
    ] {
        ui.label(format!("• {tip}"));
        ui.add_space(3.0);
    }
}

fn render_safety(ui: &mut egui::Ui) {
    ui.heading("Common Pitfalls & Safety Gates");
    ui.add_space(4.0);
    for item in [
        "Actuator Tele-Op Confirm Gates: `forward`, `backward`, `turn_left`, `turn_right`, `strafe_left`, `strafe_right`, and non-zero Yahboom twists stay local (`confirm_required: true`) until `confirm: true`.",
        "Emergency Stop Bypasses Confirm: `robot_estop`, `robotcraft-cli estop`, the red EMERGENCY STOP button, `stop`/`stop_all`, and zero twist (0, 0, 0) immediately post `POST /api/v1/stop_all`.",
        "VLA Whitelist & Weight Downloads: only whitelisted `(tool, operation)` pairs are forwarded. `vla_training` and `vla_agentic_workflow` are rejected. `vla_weights` `download` requires `confirm: true` (`X-VLA-Confirm: 1`).",
        "Joint Limits & Reachability: URDF sliders clamp to `[lower, upper]` (or `[-pi, pi]`). `validate_target_pose` rejects targets < 0.10 m (self-collision) or > 0.85 m (max reach).",
        "Peer Timeouts & Fixture Labeling: peers time out at 400 ms connect / 1500 ms read. Always verify `live: true` and `source: yahboom-mcp` before treating telemetry as physical hardware.",
        "Port 14116 Notice: `CONTROL_PORT = 14116` is a status identifier; RobotCraft does not listen on TCP port 14116.",
    ] {
        ui.label(format!("• {item}"));
        ui.add_space(3.0);
    }
}

fn render_planned(ui: &mut egui::Ui) {
    ui.heading("Planned Extensions (from docs/TIERED_PLAN.md)");
    ui.add_space(4.0);
    for item in [
        "Numerical Inverse Kinematics (IK) solver and RRT* collision-free path planning with velocity limits (replacing canned AI workflow keyframes).",
        "Native Gazebo and MuJoCo simulation bridges (beyond `isaac-mcp` health/scene/start).",
        "STL/GLB mesh rendering in the 3D viewport and URDF loading in `web/three_viewer.html`.",
        "Full 8-menu Storytold Craft bar (`File`, `Edit`, `View`, `Insert`, `Modify`, `Commands`, `Window`, `Help`) across all workspaces.",
        "Workspace-wide `#![forbid(unsafe_code)]` and Tofu Guard font-coverage audit.",
    ] {
        ui.label(format!("• {item}"));
        ui.add_space(3.0);
    }
}

fn render_skills(app: &mut RobotCraftGui, ui: &mut egui::Ui) {
    ui.heading("Bundled Skills (Embedded in robotcraft-core)");
    ui.label("Also exposed over MCP via `resources/list`, `resources/read`, and `robotcraft_skill_read`.");
    ui.add_space(6.0);
    let skills = bundled_skills();
    ui.horizontal(|ui| {
        for (idx, skill) in skills.iter().enumerate() {
            if ui
                .selectable_label(app.selected_skill_idx == idx, skill.id)
                .clicked()
            {
                app.selected_skill_idx = idx;
            }
        }
    });
    ui.separator();
    if let Some(skill) = skills.get(app.selected_skill_idx) {
        ui.strong(skill.title);
        ui.label(format!("URI: {}", skill.uri));
        ui.add_space(4.0);
        ui.monospace(skill.content);
    }
}

fn render_project_docs(app: &mut RobotCraftGui, ui: &mut egui::Ui) {
    let docs: [(&str, &str); 4] = [
        ("HELP.md", HELP_MD),
        ("docs/PRD.md", PRD_MD),
        ("docs/TIERED_PLAN.md", TIERED_PLAN_MD),
        ("CHANGELOG_LATEST.md", RELEASE_NOTES),
    ];
    ui.horizontal(|ui| {
        for (idx, (name, _)) in docs.iter().enumerate() {
            if ui
                .selectable_label(app.selected_doc_idx == idx, *name)
                .clicked()
            {
                app.selected_doc_idx = idx;
            }
        }
    });
    ui.separator();
    if let Some((_, content)) = docs.get(app.selected_doc_idx) {
        ui.label(*content);
    }
}

fn render_licenses(ui: &mut egui::Ui) {
    ui.heading("Licenses & Legal Notices");
    ui.add_space(4.0);
    ui.label("RobotCraft is dual-licensed under MIT OR Apache-2.0.");
    ui.label("Safety lint compliance: workspace crates enforce `#![deny(clippy::unwrap_used)]`, `#![deny(clippy::expect_used)]`, `#![deny(clippy::panic)]`, `#![deny(clippy::unimplemented)]`, and `#![deny(clippy::todo)]`.");
    ui.add_space(6.0);
    ui.strong("Key Third-Party Crates");
    for dep in [
        "eframe / egui (MIT OR Apache-2.0) — Immediate-mode native GUI",
        "tokio (MIT) — Async runtime for Command::dispatch",
        "serde / serde_json (MIT OR Apache-2.0) — Serialization & MCP JSON-RPC",
        "ureq (MIT OR Apache-2.0) — Synchronous HTTP client for fleet peers",
        "roxmltree (MIT OR Apache-2.0) — Zero-copy URDF XML parser",
        "clap (MIT OR Apache-2.0) — CLI argument parser",
        "rusqlite (MIT) — Bundled SQLite backing databasecraft-store",
        "chrono / thiserror (MIT OR Apache-2.0) — Timestamps & error types",
    ] {
        ui.label(format!("• {dep}"));
    }
}

fn render_about(ui: &mut egui::Ui) {
    ui.heading("About RobotCraft Studio");
    ui.label(format!("Version: {APP_VERSION}"));
    ui.label(format!("Profile: {}", crate::build_profile()));
    ui.label(format!("Built: {BUILD_UTC}"));
    ui.label(format!("Commit: {GIT_SHA}"));
    ui.label(format!("Binary: {}", crate::binary_path()));
    ui.label(format!("Control port label: {CONTROL_PORT}"));
    ui.hyperlink(format!(
        "https://github.com/sandraschi/robotcraft/releases/tag/v{APP_VERSION}"
    ));
    ui.separator();
    ui.strong("Release Notes");
    ui.label(RELEASE_NOTES);
}

pub fn render_command_palette(app: &mut RobotCraftGui, ctx: &egui::Context) {
    if !app.show_command_palette {
        return;
    }

    #[derive(Clone, Copy)]
    enum PaletteAction {
        Workspace(ActiveWorkspace),
        Help(HelpTab),
        ProbePeers,
        SyncTelemetry,
        EmergencyStop,
        ToggleTheme,
        ToggleAiChat,
        OpenShortcuts,
    }

    let entries: &[(&str, &str, PaletteAction)] = &[
        (
            "Workspace: 3D Viewport",
            "View",
            PaletteAction::Workspace(ActiveWorkspace::ThreeJsViewer),
        ),
        (
            "Workspace: AI Motion Workflows",
            "View",
            PaletteAction::Workspace(ActiveWorkspace::AiWorkflows),
        ),
        (
            "Workspace: CAD Plans & BOM",
            "View",
            PaletteAction::Workspace(ActiveWorkspace::CadPlans),
        ),
        (
            "Workspace: Live Telemetry",
            "View",
            PaletteAction::Workspace(ActiveWorkspace::Dashboard),
        ),
        (
            "Workspace: Teleop Joystick",
            "View",
            PaletteAction::Workspace(ActiveWorkspace::TeleopControl),
        ),
        (
            "Workspace: Kinematics Studio",
            "View",
            PaletteAction::Workspace(ActiveWorkspace::KinematicsStudio),
        ),
        (
            "Workspace: Fleet Status",
            "View",
            PaletteAction::Workspace(ActiveWorkspace::FleetStatus),
        ),
        (
            "Workspace: Yahboom Deck",
            "View",
            PaletteAction::Workspace(ActiveWorkspace::YahboomDeck),
        ),
        (
            "Workspace: VLA Studio",
            "View",
            PaletteAction::Workspace(ActiveWorkspace::VlaStudio),
        ),
        (
            "Action: Probe Fleet Peers",
            "Peers",
            PaletteAction::ProbePeers,
        ),
        (
            "Action: Sync Robot Telemetry",
            "Telemetry",
            PaletteAction::SyncTelemetry,
        ),
        (
            "Action: Emergency Stop (stop_all)",
            "Safety",
            PaletteAction::EmergencyStop,
        ),
        (
            "Action: Toggle Dark / Light Theme",
            "View",
            PaletteAction::ToggleTheme,
        ),
        (
            "Action: Toggle AI Chat Dock (Ctrl+J)",
            "AI",
            PaletteAction::ToggleAiChat,
        ),
        (
            "Help: Getting Started",
            "Help",
            PaletteAction::Help(HelpTab::GetStarted),
        ),
        (
            "Help: Functions & Architecture",
            "Help",
            PaletteAction::Help(HelpTab::Functions),
        ),
        (
            "Help: Usage (GUI, CLI & MCP)",
            "Help",
            PaletteAction::Help(HelpTab::Usage),
        ),
        (
            "Help: Tips & Tricks",
            "Help",
            PaletteAction::Help(HelpTab::TipsAndTricks),
        ),
        (
            "Help: Common Pitfalls & Safety",
            "Help",
            PaletteAction::Help(HelpTab::PitfallsAndSafety),
        ),
        (
            "Help: Planned Extensions",
            "Help",
            PaletteAction::Help(HelpTab::PlannedExtensions),
        ),
        (
            "Help: Bundled Skills",
            "Help",
            PaletteAction::Help(HelpTab::BundledSkills),
        ),
        (
            "Help: Project Files & Docs",
            "Help",
            PaletteAction::Help(HelpTab::ProjectDocs),
        ),
        (
            "Help: Licenses & Notices",
            "Help",
            PaletteAction::Help(HelpTab::Licenses),
        ),
        (
            "Help: About RobotCraft",
            "Help",
            PaletteAction::Help(HelpTab::About),
        ),
        (
            "Help: Keyboard Shortcuts (F1)",
            "Help",
            PaletteAction::OpenShortcuts,
        ),
    ];

    let query = app.command_palette_query.trim().to_ascii_lowercase();
    let filtered: Vec<_> = entries
        .iter()
        .filter(|(label, cat, _)| {
            query.is_empty()
                || label.to_ascii_lowercase().contains(&query)
                || cat.to_ascii_lowercase().contains(&query)
        })
        .copied()
        .collect();

    let mut chosen: Option<PaletteAction> = None;
    let mut open = app.show_command_palette;
    egui::Window::new("Command Palette (Ctrl+Shift+P / Ctrl+K)")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(480.0)
        .show(ctx, |ui| {
            let resp = ui.add(
                egui::TextEdit::singleline(&mut app.command_palette_query)
                    .hint_text("Type a command, workspace, or help topic...")
                    .desired_width(f32::INFINITY),
            );
            resp.request_focus();
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Some((_, _, act)) = filtered.first() {
                    chosen = Some(*act);
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                app.show_command_palette = false;
            }
            ui.separator();
            egui::ScrollArea::vertical()
                .max_height(280.0)
                .show(ui, |ui| {
                    for (label, cat, act) in &filtered {
                        if ui
                            .selectable_label(false, format!("[{cat}] {label}"))
                            .clicked()
                        {
                            chosen = Some(*act);
                        }
                    }
                    if filtered.is_empty() {
                        ui.label("No matching commands.");
                    }
                });
        });

    if !open {
        app.show_command_palette = false;
    }

    if let Some(act) = chosen {
        app.show_command_palette = false;
        match act {
            PaletteAction::Workspace(ws) => app.active_workspace = ws,
            PaletteAction::Help(tab) => {
                app.help_tab = tab;
                app.show_help_center = true;
            }
            PaletteAction::ProbePeers => app.fetch_peers(),
            PaletteAction::SyncTelemetry => app.fetch_telemetry(),
            PaletteAction::EmergencyStop => app.trigger_estop(),
            PaletteAction::ToggleTheme => app.is_dark_theme = !app.is_dark_theme,
            PaletteAction::ToggleAiChat => app.show_ai_chat_dock = !app.show_ai_chat_dock,
            PaletteAction::OpenShortcuts => app.show_shortcuts_dialog = true,
        }
    }
}

pub fn render_shortcuts_dialog(app: &mut RobotCraftGui, ctx: &egui::Context) {
    if !app.show_shortcuts_dialog {
        return;
    }
    let mut open = app.show_shortcuts_dialog;
    egui::Window::new("Keyboard Shortcuts")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(460.0)
        .show(ctx, |ui| {
            egui::Grid::new("shortcuts_grid")
                .num_columns(3)
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Shortcut");
                    ui.strong("Action");
                    ui.strong("Scope");
                    ui.end_row();
                    for (key, desc, scope) in [
                        ("Ctrl+Shift+P / Ctrl+K", "Open Command Palette", "Global"),
                        ("Ctrl+/ / F1", "Open Keyboard Shortcuts", "Global"),
                        ("Ctrl+J", "Toggle AI Chat Dock", "Global"),
                        ("Escape", "Close Command Palette", "Overlay"),
                        ("Left-Drag", "Orbit 3D Viewport", "3D View"),
                        ("Scroll", "Zoom 3D Viewport", "3D View"),
                    ] {
                        ui.monospace(key);
                        ui.label(desc);
                        ui.label(scope);
                        ui.end_row();
                    }
                });
            ui.add_space(8.0);
            if ui.button("Close").clicked() {
                app.show_shortcuts_dialog = false;
            }
        });
    if !open {
        app.show_shortcuts_dialog = false;
    }
}

pub fn render_ai_chat_dock(app: &mut RobotCraftGui, ui: &mut egui::Ui) {
    if !app.show_ai_chat_dock {
        return;
    }
    let cyan = Color32::from_rgb(6, 182, 212);
    egui::Panel::right("ai_chat_dock")
        .min_size(300.0)
        .default_size(340.0)
        .show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.strong(egui::RichText::new("AI Chat Dock (Local-First)").color(cyan));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("X").clicked() {
                        app.show_ai_chat_dock = false;
                    }
                });
            });
            ui.label(
                "Runs locally via Command::dispatch & bundled skills. Press Ctrl+J to toggle.",
            );
            ui.separator();

            let mut quick_prompt: Option<&str> = None;
            ui.horizontal_wrapped(|ui| {
                for (label, prompt) in [
                    ("Status", "status"),
                    ("Probe Peers", "peers"),
                    ("Robots", "robots"),
                    ("Safety", "safety"),
                    ("URDF & FK", "urdf"),
                    ("Skills", "skills"),
                ] {
                    if ui.small_button(label).clicked() {
                        quick_prompt = Some(prompt);
                    }
                }
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(460.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for (role, msg) in &app.ai_chat_history {
                        let color = if role == "You" {
                            cyan
                        } else {
                            ui.visuals().text_color()
                        };
                        ui.strong(egui::RichText::new(role).color(color));
                        ui.label(msg);
                        ui.add_space(6.0);
                    }
                });

            ui.separator();
            let mut submit = false;
            ui.horizontal(|ui| {
                let edit = ui.add(
                    egui::TextEdit::singleline(&mut app.ai_chat_input)
                        .hint_text("Ask about status, peers, URDF, safety, skills...")
                        .desired_width(ui.available_width() - 56.0),
                );
                if edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    submit = true;
                }
                if ui.button("Send").clicked() {
                    submit = true;
                }
            });

            if let Some(prompt) = quick_prompt {
                submit_ai_chat(app, prompt.to_string());
            } else if submit && !app.ai_chat_input.trim().is_empty() {
                let text = std::mem::take(&mut app.ai_chat_input);
                submit_ai_chat(app, text);
            }
        });
}

fn submit_ai_chat(app: &mut RobotCraftGui, prompt: String) {
    let q = prompt.trim().to_ascii_lowercase();
    app.ai_chat_history.push(("You".to_string(), prompt));

    let reply = if q.contains("status") {
        let res = app.runtime.block_on(dispatch(Command::Status, &app.store));
        serde_json::to_string_pretty(&res).unwrap_or_else(|_| res.to_string())
    } else if q.contains("peer") || q.contains("probe") {
        let res = app
            .runtime
            .block_on(dispatch(Command::ProbePeers, &app.store));
        app.peer_snapshot = Some(res.clone());
        serde_json::to_string_pretty(&res).unwrap_or_else(|_| res.to_string())
    } else if q.contains("robot") || q.contains("list") {
        let res = app
            .runtime
            .block_on(dispatch(Command::ListRobots, &app.store));
        serde_json::to_string_pretty(&res).unwrap_or_else(|_| res.to_string())
    } else if q.contains("safety") || q.contains("confirm") || q.contains("estop") {
        get_skill("robotcraft-safety")
            .map(|s| s.content.to_string())
            .unwrap_or_else(|| "Safety skill unavailable.".to_string())
    } else if q.contains("urdf")
        || q.contains("fk")
        || q.contains("kinematic")
        || q.contains("reach")
    {
        get_skill("robotcraft-urdf-kinematics")
            .map(|s| s.content.to_string())
            .unwrap_or_else(|| "URDF skill unavailable.".to_string())
    } else if q.contains("skill") {
        let list: Vec<String> = bundled_skills()
            .iter()
            .map(|s| format!("• {} ({}) — {}", s.id, s.uri, s.summary))
            .collect();
        format!(
            "Bundled skills (open Help -> Bundled Skills... or call MCP `robotcraft_skill_read`):\n{}",
            list.join("\n")
        )
    } else {
        "Local-first assistant ready. Try: `status`, `peers`, `robots`, `urdf`, `safety`, or `skills`. For full docs open Help -> Project Files & Docs...".to_string()
    };

    app.ai_chat_history.push(("RobotCraft".to_string(), reply));
}
