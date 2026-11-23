//! 2D Apartment Floorplan Digital Twin, 360-degree LIDAR Raycaster,
//! Occupancy-Grid A* Patrol Path Planner, and Doorway Clearance UI.

use crate::RobotCraftGui;
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use robotcraft_core::dojo2d::{
    check_doorway_clearances, plan_patrol_path, robot_footprint_width_m, simulate_lidar_scan,
    FloorplanTwin,
};

pub fn render_apartment_dojo_2d(app: &mut RobotCraftGui, ui: &mut egui::Ui) {
    let cyan = Color32::from_rgb(6, 182, 212);
    let green = Color32::from_rgb(34, 197, 94);
    let amber = Color32::from_rgb(245, 158, 11);
    let red = Color32::from_rgb(239, 68, 68);

    ui.heading("🗺 2D Apartment LIDAR & Floorplan Patrol Dojo");
    ui.label(
        "Vienna Altbau Digital Twin (12m x 8m): 360-degree LIDAR raycasting, occupancy-grid A* patrol planner, and doorway clearance checker.",
    );
    ui.separator();

    let twin = FloorplanTwin::vienna_altbau_seed();

    ui.horizontal_wrapped(|ui| {
        ui.label("Robot:");
        for (id, label) in [
            ("saschibot", "Saschibot P6 (52cm)"),
            ("nori_a3", "Nori A3 (56cm)"),
            ("yahboom_rover", "Yahboom X3 (24cm)"),
            ("unitree_go2", "Unitree Go2 (38cm)"),
            ("noetix_bumi", "Noetix Bumi (48cm)"),
            ("loona_petbot", "Loona KY004 (21cm)"),
        ] {
            if ui
                .selectable_label(app.selected_robot == id, label)
                .clicked()
            {
                app.selected_robot = id.to_string();
            }
        }
    });

    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        ui.label("Pose X (m):");
        ui.add(egui::Slider::new(&mut app.dojo_robot_x, 0.4..=11.6).step_by(0.1));
        ui.label("Y (m):");
        ui.add(egui::Slider::new(&mut app.dojo_robot_y, 0.4..=7.6).step_by(0.1));
        ui.label("Yaw (deg):");
        ui.add(egui::Slider::new(&mut app.dojo_robot_yaw, 0.0..=360.0).step_by(5.0));
        ui.label("Rays:");
        ui.add(egui::Slider::new(&mut app.dojo_lidar_rays, 24..=180));
        if ui.button("Step Patrol Forward").clicked() {
            let plan = plan_patrol_path(
                &twin,
                &app.selected_robot,
                app.dojo_robot_x,
                app.dojo_robot_y,
                app.dojo_goal_x,
                app.dojo_goal_y,
            );
            if let Some(&(nx, ny)) = plan.waypoints.get(1) {
                let dx = nx - app.dojo_robot_x;
                let dy = ny - app.dojo_robot_y;
                if dx.abs() > 1e-3 || dy.abs() > 1e-3 {
                    app.dojo_robot_yaw = dy.atan2(dx).to_degrees().rem_euclid(360.0);
                }
                app.dojo_robot_x = nx;
                app.dojo_robot_y = ny;
                app.status_message = format!(
                    "Patrol step -> ({nx:.2}m, {ny:.2}m), yaw {:.0} deg",
                    app.dojo_robot_yaw
                );
            }
        }
        if ui.button("Preset: Salon -> Goliath Lab").clicked() {
            app.dojo_robot_x = 1.5;
            app.dojo_robot_y = 3.7;
            app.dojo_goal_x = 8.0;
            app.dojo_goal_y = 3.2;
        }
        if ui.button("Preset: Lab -> Kitchen").clicked() {
            app.dojo_robot_x = 7.5;
            app.dojo_robot_y = 3.2;
            app.dojo_goal_x = 9.0;
            app.dojo_goal_y = 5.6;
        }
    });

    let scan = simulate_lidar_scan(
        &twin,
        app.dojo_robot_x,
        app.dojo_robot_y,
        app.dojo_robot_yaw,
        app.dojo_lidar_rays,
        10.0,
    );
    let plan = plan_patrol_path(
        &twin,
        &app.selected_robot,
        app.dojo_robot_x,
        app.dojo_robot_y,
        app.dojo_goal_x,
        app.dojo_goal_y,
    );
    let clearances = check_doorway_clearances(&twin, &app.selected_robot);

    ui.add_space(6.0);
    ui.columns(2, |cols| {
        // Left column: 2D Floorplan & LIDAR Canvas
        let avail = cols[0].available_width().max(380.0);
        let scale = (avail / twin.width_m).min(46.0);
        let canvas_size = Vec2::new(twin.width_m * scale, twin.height_m * scale);
        let (rect, response) = cols[0].allocate_exact_size(canvas_size, Sense::click_and_drag());
        let painter = cols[0].painter_at(rect);

        let bg = if app.is_dark_theme {
            Color32::from_rgb(15, 23, 42)
        } else {
            Color32::from_rgb(241, 245, 249)
        };
        painter.rect_filled(rect, 6.0, bg);

        let to_screen = |wx: f32, wy: f32| -> Pos2 {
            Pos2::new(rect.left() + wx * scale, rect.top() + wy * scale)
        };

        // Left-click moves robot, right-click moves patrol goal
        if response.clicked() || response.dragged_by(egui::PointerButton::Primary) {
            if let Some(pos) = response.interact_pointer_pos() {
                app.dojo_robot_x = ((pos.x - rect.left()) / scale).clamp(0.4, twin.width_m - 0.4);
                app.dojo_robot_y = ((pos.y - rect.top()) / scale).clamp(0.4, twin.height_m - 0.4);
            }
        }
        if response.secondary_clicked() || response.dragged_by(egui::PointerButton::Secondary) {
            if let Some(pos) = response.interact_pointer_pos() {
                app.dojo_goal_x = ((pos.x - rect.left()) / scale).clamp(0.4, twin.width_m - 0.4);
                app.dojo_goal_y = ((pos.y - rect.top()) / scale).clamp(0.4, twin.height_m - 0.4);
            }
        }

        // Draw 1m grid lines
        let grid_col = if app.is_dark_theme {
            Color32::from_rgba_unmultiplied(148, 163, 184, 28)
        } else {
            Color32::from_rgba_unmultiplied(100, 116, 139, 35)
        };
        for gx in 1..(twin.width_m as i32) {
            let x = gx as f32;
            painter.line_segment(
                [to_screen(x, 0.0), to_screen(x, twin.height_m)],
                Stroke::new(1.0, grid_col),
            );
        }
        for gy in 1..(twin.height_m as i32) {
            let y = gy as f32;
            painter.line_segment(
                [to_screen(0.0, y), to_screen(twin.width_m, y)],
                Stroke::new(1.0, grid_col),
            );
        }

        // Draw LIDAR rays
        let robot_pos = to_screen(app.dojo_robot_x, app.dojo_robot_y);
        for hit in &scan.hits {
            let hit_pos = to_screen(hit.hit_x, hit.hit_y);
            let ray_col = if hit.distance_m < 0.65 {
                Color32::from_rgba_unmultiplied(239, 68, 68, 140)
            } else {
                Color32::from_rgba_unmultiplied(34, 197, 94, 85)
            };
            painter.line_segment([robot_pos, hit_pos], Stroke::new(1.0, ray_col));
            painter.circle_filled(hit_pos, 2.0, green);
        }

        // Draw obstacles
        for obs in &twin.obstacles {
            let r = Rect::from_min_max(
                to_screen(obs.x_min, obs.y_min),
                to_screen(obs.x_max, obs.y_max),
            );
            painter.rect_filled(r, 3.0, Color32::from_rgba_unmultiplied(100, 116, 139, 170));
            painter.rect_stroke(
                r,
                3.0,
                Stroke::new(1.5, Color32::from_rgb(148, 163, 184)),
                egui::StrokeKind::Outside,
            );
            painter.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                &obs.label,
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );
        }

        // Draw walls
        let wall_col = if app.is_dark_theme {
            Color32::from_rgb(226, 232, 240)
        } else {
            Color32::from_rgb(30, 41, 59)
        };
        for w in &twin.walls {
            painter.line_segment(
                [to_screen(w.x1, w.y1), to_screen(w.x2, w.y2)],
                Stroke::new(3.0, wall_col),
            );
        }

        // Draw A* patrol path
        if plan.reachable && plan.waypoints.len() >= 2 {
            for pair in plan.waypoints.windows(2) {
                painter.line_segment(
                    [
                        to_screen(pair[0].0, pair[0].1),
                        to_screen(pair[1].0, pair[1].1),
                    ],
                    Stroke::new(2.5, cyan),
                );
            }
        }

        // Draw goal marker
        let goal_pos = to_screen(app.dojo_goal_x, app.dojo_goal_y);
        painter.circle_stroke(goal_pos, 7.0, Stroke::new(2.0, amber));
        painter.circle_filled(goal_pos, 3.5, amber);

        // Draw robot footprint & heading vector
        let r_radius_px = (robot_footprint_width_m(&app.selected_robot) * 0.5 * scale).max(6.0);
        painter.circle_filled(robot_pos, r_radius_px, cyan);
        let head_rad = app.dojo_robot_yaw.to_radians();
        let nose = Pos2::new(
            robot_pos.x + head_rad.cos() * (r_radius_px + 7.0),
            robot_pos.y + head_rad.sin() * (r_radius_px + 7.0),
        );
        painter.line_segment([robot_pos, nose], Stroke::new(2.5, Color32::WHITE));

        cols[0].small("Left-click/drag on floorplan: move robot | Right-click: move patrol goal");

        // Right column: LIDAR Telemetry, A* Patrol Metrics & Doorway Clearances
        cols[1].group(|ui| {
            ui.strong("📡 360° LIDAR Scan Telemetry");
            ui.label(format!(
                "Origin: ({:.2}m, {:.2}m) @ {:.0}° | Rays: {}",
                scan.origin_x, scan.origin_y, scan.yaw_deg, scan.ray_count
            ));
            let clr_color = if scan.min_clearance_m < 0.45 {
                red
            } else if scan.min_clearance_m < 0.90 {
                amber
            } else {
                green
            };
            ui.colored_label(
                clr_color,
                format!(
                    "Min Obstacle Clearance: {:.3} m (nearest: {})",
                    scan.min_clearance_m, scan.nearest_surface
                ),
            );
        });

        cols[1].add_space(6.0);
        cols[1].group(|ui| {
            ui.strong("🧭 Occupancy-Grid A* Patrol Plan");
            ui.horizontal(|ui| {
                ui.label("Goal X (m):");
                ui.add(egui::Slider::new(&mut app.dojo_goal_x, 0.5..=11.5).step_by(0.1));
                ui.label("Goal Y (m):");
                ui.add(egui::Slider::new(&mut app.dojo_goal_y, 0.5..=7.5).step_by(0.1));
            });
            if plan.reachable {
                ui.colored_label(
                    green,
                    format!(
                        "Route Found: {} waypoints | {:.2} m | ~{:.1} s | {:.2}% battery",
                        plan.waypoints.len(),
                        plan.total_distance_m,
                        plan.estimated_seconds,
                        plan.battery_cost_pct
                    ),
                );
            } else {
                ui.colored_label(
                    red,
                    "Goal unreachable with current robot footprint clearance.",
                );
            }
        });

        cols[1].add_space(6.0);
        cols[1].group(|ui| {
            ui.strong(format!(
                "🚪 Doorway Clearance Matrix ({} = {:.0} cm wide)",
                app.selected_robot,
                robot_footprint_width_m(&app.selected_robot) * 100.0
            ));
            for c in &clearances {
                let col = match c.status.as_str() {
                    "passable" => green,
                    "tight" => amber,
                    _ => red,
                };
                ui.horizontal(|ui| {
                    ui.colored_label(col, format!("[{}]", c.status.to_uppercase()));
                    ui.label(format!(
                        "{}: door {:.0} cm, margin {:+.0} cm",
                        c.label,
                        c.width_m * 100.0,
                        c.margin_m * 100.0
                    ));
                });
            }
        });
    });
}
