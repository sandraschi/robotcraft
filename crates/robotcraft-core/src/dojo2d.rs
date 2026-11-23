//! 2D Apartment Floorplan Digital Twin, 360-degree LIDAR Raycaster,
//! Occupancy-Grid A* Path Planner, and Doorway Clearance Analyzer.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallSegment {
    pub id: String,
    pub label: String,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doorway {
    pub id: String,
    pub label: String,
    pub room_a: String,
    pub room_b: String,
    pub x: f32,
    pub y: f32,
    pub width_m: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObstacleBox {
    pub id: String,
    pub label: String,
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorplanTwin {
    pub name: String,
    pub width_m: f32,
    pub height_m: f32,
    pub walls: Vec<WallSegment>,
    pub doorways: Vec<Doorway>,
    pub obstacles: Vec<ObstacleBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LidarHit {
    pub angle_deg: f32,
    pub distance_m: f32,
    pub hit_x: f32,
    pub hit_y: f32,
    pub surface: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LidarScanResult {
    pub origin_x: f32,
    pub origin_y: f32,
    pub yaw_deg: f32,
    pub ray_count: usize,
    pub max_range_m: f32,
    pub min_clearance_m: f32,
    pub nearest_surface: String,
    pub hits: Vec<LidarHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoorwayClearance {
    pub doorway_id: String,
    pub label: String,
    pub width_m: f32,
    pub robot_width_m: f32,
    pub margin_m: f32,
    pub status: String, // "passable", "tight", "blocked"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatrolPlanResult {
    pub robot_id: String,
    pub reachable: bool,
    pub start: (f32, f32),
    pub goal: (f32, f32),
    pub waypoints: Vec<(f32, f32)>,
    pub total_distance_m: f32,
    pub estimated_seconds: f32,
    pub battery_cost_pct: f32,
}

impl FloorplanTwin {
    /// Default Vienna Altbau Dojo floorplan (12m x 8m apartment with Living Room,
    /// Hallway, Goliath AI/Robotics Study, and Kitchen).
    pub fn vienna_altbau_seed() -> Self {
        let walls = vec![
            // Outer perimeter (12m x 8m)
            WallSegment {
                id: "w_north".to_string(),
                label: "North Outer Wall".to_string(),
                x1: 0.0,
                y1: 0.0,
                x2: 12.0,
                y2: 0.0,
            },
            WallSegment {
                id: "w_south".to_string(),
                label: "South Outer Wall".to_string(),
                x1: 0.0,
                y1: 8.0,
                x2: 12.0,
                y2: 8.0,
            },
            WallSegment {
                id: "w_west".to_string(),
                label: "West Outer Wall".to_string(),
                x1: 0.0,
                y1: 0.0,
                x2: 0.0,
                y2: 8.0,
            },
            WallSegment {
                id: "w_east".to_string(),
                label: "East Outer Wall".to_string(),
                x1: 12.0,
                y1: 0.0,
                x2: 12.0,
                y2: 8.0,
            },
            // Interior divider at x = 6.0 with doorway gap y in [3.2, 4.2] (1.00m Altbau Fluegeltuer)
            WallSegment {
                id: "w_mid_top".to_string(),
                label: "Salon/Study Partition N".to_string(),
                x1: 6.0,
                y1: 0.0,
                x2: 6.0,
                y2: 3.2,
            },
            WallSegment {
                id: "w_mid_bot".to_string(),
                label: "Salon/Study Partition S".to_string(),
                x1: 6.0,
                y1: 4.2,
                x2: 6.0,
                y2: 8.0,
            },
            // Horizontal divider on right side at y = 4.8 with doorway gap x in [8.4, 9.25] (0.85m door)
            WallSegment {
                id: "w_right_w".to_string(),
                label: "Study/Kitchen Wall W".to_string(),
                x1: 6.0,
                y1: 4.8,
                x2: 8.4,
                y2: 4.8,
            },
            WallSegment {
                id: "w_right_e".to_string(),
                label: "Study/Kitchen Wall E".to_string(),
                x1: 9.25,
                y1: 4.8,
                x2: 12.0,
                y2: 4.8,
            },
        ];

        let doorways = vec![
            Doorway {
                id: "door_fluegel".to_string(),
                label: "Altbau Fluegeltuer (Salon <-> Goliath Lab)".to_string(),
                room_a: "Salon".to_string(),
                room_b: "Goliath AI & Robotics Lab".to_string(),
                x: 6.0,
                y: 3.7,
                width_m: 1.00,
            },
            Doorway {
                id: "door_kitchen".to_string(),
                label: "Kitchen Door (Goliath Lab <-> Kitchen)".to_string(),
                room_a: "Goliath AI & Robotics Lab".to_string(),
                room_b: "Kitchen".to_string(),
                x: 8.82,
                y: 4.8,
                width_m: 0.85,
            },
            Doorway {
                id: "door_server_rack".to_string(),
                label: "Goliath eGPU / Rack Niche Passage".to_string(),
                room_a: "Goliath AI & Robotics Lab".to_string(),
                room_b: "Rack Corner".to_string(),
                x: 10.2,
                y: 1.8,
                width_m: 0.58,
            },
        ];

        let obstacles = vec![
            ObstacleBox {
                id: "obs_sofa".to_string(),
                label: "Salon Sofa".to_string(),
                x_min: 1.2,
                y_min: 1.2,
                x_max: 3.6,
                y_max: 2.3,
            },
            ObstacleBox {
                id: "obs_table".to_string(),
                label: "Coffee Table".to_string(),
                x_min: 2.0,
                y_min: 5.2,
                x_max: 3.8,
                y_max: 6.4,
            },
            ObstacleBox {
                id: "obs_goliath_desk".to_string(),
                label: "Goliath 4090 Workbench".to_string(),
                x_min: 8.8,
                y_min: 0.8,
                x_max: 11.2,
                y_max: 2.0,
            },
            ObstacleBox {
                id: "obs_kitchen_island".to_string(),
                label: "Kitchen Counter".to_string(),
                x_min: 7.4,
                y_min: 6.2,
                x_max: 10.4,
                y_max: 7.2,
            },
        ];

        Self {
            name: "Vienna Altbau Dojo (12m x 8m)".to_string(),
            width_m: 12.0,
            height_m: 8.0,
            walls,
            doorways,
            obstacles,
        }
    }

    pub fn all_segments(&self) -> Vec<WallSegment> {
        let mut segs = self.walls.clone();
        for obs in &self.obstacles {
            segs.push(WallSegment {
                id: format!("{}_n", obs.id),
                label: obs.label.clone(),
                x1: obs.x_min,
                y1: obs.y_min,
                x2: obs.x_max,
                y2: obs.y_min,
            });
            segs.push(WallSegment {
                id: format!("{}_s", obs.id),
                label: obs.label.clone(),
                x1: obs.x_min,
                y1: obs.y_max,
                x2: obs.x_max,
                y2: obs.y_max,
            });
            segs.push(WallSegment {
                id: format!("{}_w", obs.id),
                label: obs.label.clone(),
                x1: obs.x_min,
                y1: obs.y_min,
                x2: obs.x_min,
                y2: obs.y_max,
            });
            segs.push(WallSegment {
                id: format!("{}_e", obs.id),
                label: obs.label.clone(),
                x1: obs.x_max,
                y1: obs.y_min,
                x2: obs.x_max,
                y2: obs.y_max,
            });
        }
        segs
    }
}

pub fn robot_footprint_width_m(robot_id: &str) -> f32 {
    match robot_id {
        "yahboom_rover" => 0.24,
        "loona_petbot" => 0.21,
        "unitree_go2" => 0.38,
        "saschibot" => 0.52,
        "nori_a3" => 0.56,
        "noetix_bumi" => 0.48,
        "craftarm_6dof" => 0.65,
        _ => 0.45,
    }
}

/// Cast `ray_count` 2D LIDAR rays from `(origin_x, origin_y)` across 360 degrees.
pub fn simulate_lidar_scan(
    twin: &FloorplanTwin,
    origin_x: f32,
    origin_y: f32,
    yaw_deg: f32,
    ray_count: usize,
    max_range_m: f32,
) -> LidarScanResult {
    let count = ray_count.clamp(8, 360);
    let max_r = max_range_m.clamp(0.5, 25.0);
    let segments = twin.all_segments();

    let mut hits = Vec::with_capacity(count);
    let mut min_clearance = max_r;
    let mut nearest_surface = "open_space".to_string();

    for i in 0..count {
        let rel_deg = (i as f32) * 360.0 / (count as f32);
        let world_deg = (yaw_deg + rel_deg).rem_euclid(360.0);
        let rad = world_deg.to_radians();
        let dx = rad.cos();
        let dy = rad.sin();

        let mut best_t = max_r;
        let mut best_surface = "max_range".to_string();

        for seg in &segments {
            if let Some(t) =
                ray_segment_intersection(origin_x, origin_y, dx, dy, seg.x1, seg.y1, seg.x2, seg.y2)
            {
                if t > 0.001 && t < best_t {
                    best_t = t;
                    best_surface = seg.label.clone();
                }
            }
        }

        if best_t < min_clearance {
            min_clearance = best_t;
            nearest_surface = best_surface.clone();
        }

        hits.push(LidarHit {
            angle_deg: (rel_deg * 10.0).round() / 10.0,
            distance_m: (best_t * 1000.0).round() / 1000.0,
            hit_x: ((origin_x + dx * best_t) * 1000.0).round() / 1000.0,
            hit_y: ((origin_y + dy * best_t) * 1000.0).round() / 1000.0,
            surface: best_surface,
        });
    }

    LidarScanResult {
        origin_x,
        origin_y,
        yaw_deg,
        ray_count: count,
        max_range_m: max_r,
        min_clearance_m: (min_clearance * 1000.0).round() / 1000.0,
        nearest_surface,
        hits,
    }
}

#[allow(clippy::too_many_arguments)]
fn ray_segment_intersection(
    ox: f32,
    oy: f32,
    dx: f32,
    dy: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) -> Option<f32> {
    let sx = x2 - x1;
    let sy = y2 - y1;
    let denom = dx * sy - dy * sx;
    if denom.abs() < 1e-6 {
        return None;
    }
    let qpx = x1 - ox;
    let qpy = y1 - oy;
    let t = (qpx * sy - qpy * sx) / denom;
    let u = (qpx * dy - qpy * dx) / denom;
    if t >= 0.0 && (0.0..=1.0).contains(&u) {
        Some(t)
    } else {
        None
    }
}

pub fn check_doorway_clearances(twin: &FloorplanTwin, robot_id: &str) -> Vec<DoorwayClearance> {
    let rw = robot_footprint_width_m(robot_id);
    twin.doorways
        .iter()
        .map(|d| {
            let margin = ((d.width_m - rw) * 1000.0).round() / 1000.0;
            let status = if margin < 0.04 {
                "blocked"
            } else if margin < 0.16 {
                "tight"
            } else {
                "passable"
            };
            DoorwayClearance {
                doorway_id: d.id.clone(),
                label: d.label.clone(),
                width_m: d.width_m,
                robot_width_m: rw,
                margin_m: margin,
                status: status.to_string(),
            }
        })
        .collect()
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct GridNode {
    f_cost: i32,
    g_cost: i32,
    gx: i32,
    gy: i32,
}

impl Ord for GridNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_cost.cmp(&self.f_cost)
    }
}

impl PartialOrd for GridNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Occupancy-grid A* path planner (20 cm grid resolution) with obstacle & wall clearance.
pub fn plan_patrol_path(
    twin: &FloorplanTwin,
    robot_id: &str,
    start_x: f32,
    start_y: f32,
    goal_x: f32,
    goal_y: f32,
) -> PatrolPlanResult {
    let cell = 0.25_f32;
    let cols = (twin.width_m / cell).ceil() as i32;
    let rows = (twin.height_m / cell).ceil() as i32;
    let clearance = (robot_footprint_width_m(robot_id) * 0.45).clamp(0.12, 0.30);

    let sx = ((start_x / cell).round() as i32).clamp(1, cols - 2);
    let sy = ((start_y / cell).round() as i32).clamp(1, rows - 2);
    let gx = ((goal_x / cell).round() as i32).clamp(1, cols - 2);
    let gy = ((goal_y / cell).round() as i32).clamp(1, rows - 2);

    let segments = twin.all_segments();
    let is_blocked = |cx: i32, cy: i32| -> bool {
        if cx <= 0 || cy <= 0 || cx >= cols - 1 || cy >= rows - 1 {
            return true;
        }
        let wx = (cx as f32) * cell;
        let wy = (cy as f32) * cell;
        for obs in &twin.obstacles {
            if wx >= obs.x_min - clearance
                && wx <= obs.x_max + clearance
                && wy >= obs.y_min - clearance
                && wy <= obs.y_max + clearance
            {
                return true;
            }
        }
        for seg in &segments {
            if point_segment_dist(wx, wy, seg.x1, seg.y1, seg.x2, seg.y2) < clearance {
                return true;
            }
        }
        false
    };

    let mut open = BinaryHeap::new();
    let mut g_scores: HashMap<(i32, i32), i32> = HashMap::new();
    let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();

    g_scores.insert((sx, sy), 0);
    open.push(GridNode {
        f_cost: heuristic(sx, sy, gx, gy),
        g_cost: 0,
        gx: sx,
        gy: sy,
    });

    let dirs = [
        (1, 0, 10),
        (-1, 0, 10),
        (0, 1, 10),
        (0, -1, 10),
        (1, 1, 14),
        (1, -1, 14),
        (-1, 1, 14),
        (-1, -1, 14),
    ];

    let mut found = false;
    while let Some(curr) = open.pop() {
        if (curr.gx, curr.gy) == (gx, gy) {
            found = true;
            break;
        }
        for (dx, dy, step_cost) in dirs {
            let nx = curr.gx + dx;
            let ny = curr.gy + dy;
            if (nx, ny) != (gx, gy) && is_blocked(nx, ny) {
                continue;
            }
            let tentative_g = curr.g_cost + step_cost;
            if tentative_g < *g_scores.get(&(nx, ny)).unwrap_or(&i32::MAX) {
                came_from.insert((nx, ny), (curr.gx, curr.gy));
                g_scores.insert((nx, ny), tentative_g);
                open.push(GridNode {
                    f_cost: tentative_g + heuristic(nx, ny, gx, gy),
                    g_cost: tentative_g,
                    gx: nx,
                    gy: ny,
                });
            }
        }
    }

    if !found {
        return PatrolPlanResult {
            robot_id: robot_id.to_string(),
            reachable: false,
            start: (start_x, start_y),
            goal: (goal_x, goal_y),
            waypoints: vec![(start_x, start_y)],
            total_distance_m: 0.0,
            estimated_seconds: 0.0,
            battery_cost_pct: 0.0,
        };
    }

    let mut grid_path = vec![(gx, gy)];
    let mut cur = (gx, gy);
    while let Some(&prev) = came_from.get(&cur) {
        grid_path.push(prev);
        if prev == (sx, sy) {
            break;
        }
        cur = prev;
    }
    grid_path.reverse();

    let waypoints: Vec<(f32, f32)> = grid_path
        .into_iter()
        .map(|(x, y)| {
            (
                ((x as f32) * cell * 100.0).round() / 100.0,
                ((y as f32) * cell * 100.0).round() / 100.0,
            )
        })
        .collect();

    let mut dist = 0.0_f32;
    for pair in waypoints.windows(2) {
        let dx = pair[1].0 - pair[0].0;
        let dy = pair[1].1 - pair[0].1;
        dist += (dx * dx + dy * dy).sqrt();
    }
    let total_distance_m = (dist * 100.0).round() / 100.0;
    let estimated_seconds = ((total_distance_m / 0.45) * 10.0).round() / 10.0;
    let battery_cost_pct = ((total_distance_m * 0.35) * 100.0).round() / 100.0;

    PatrolPlanResult {
        robot_id: robot_id.to_string(),
        reachable: true,
        start: (start_x, start_y),
        goal: (goal_x, goal_y),
        waypoints,
        total_distance_m,
        estimated_seconds,
        battery_cost_pct,
    }
}

fn heuristic(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    let dx = (x1 - x2).abs();
    let dy = (y1 - y2).abs();
    10 * (dx + dy) - 6 * dx.min(dy)
}

fn point_segment_dist(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let vx = x2 - x1;
    let vy = y2 - y1;
    let len_sq = vx * vx + vy * vy;
    if len_sq < 1e-6 {
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }
    let t = (((px - x1) * vx + (py - y1) * vy) / len_sq).clamp(0.0, 1.0);
    let proj_x = x1 + t * vx;
    let proj_y = y1 + t * vy;
    ((px - proj_x).powi(2) + (py - proj_y).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lidar_scan_detects_walls_and_obstacles_in_vienna_altbau() {
        let twin = FloorplanTwin::vienna_altbau_seed();
        let scan = simulate_lidar_scan(&twin, 2.5, 3.7, 0.0, 72, 10.0);
        assert_eq!(scan.ray_count, 72);
        assert_eq!(scan.hits.len(), 72);
        assert!(scan.min_clearance_m > 0.2 && scan.min_clearance_m < 4.0);
    }

    #[test]
    fn astar_patrol_routes_through_altbau_fluegeltuer() {
        let twin = FloorplanTwin::vienna_altbau_seed();
        let plan = plan_patrol_path(&twin, "saschibot", 1.5, 3.7, 8.0, 3.2);
        assert!(plan.reachable);
        assert!(plan.waypoints.len() >= 10);
        assert!(plan.total_distance_m > 6.0);
    }

    #[test]
    fn doorway_clearance_distinguishes_rover_from_industrial_arm() {
        let twin = FloorplanTwin::vienna_altbau_seed();
        let rover = check_doorway_clearances(&twin, "yahboom_rover");
        assert!(rover.iter().all(|c| c.status == "passable"));

        let arm = check_doorway_clearances(&twin, "craftarm_6dof");
        assert!(arm.iter().any(|c| c.status == "blocked"));
    }
}
