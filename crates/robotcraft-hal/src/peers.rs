//! HTTP bridges to sibling fleet servers.
//!
//! `yahboom-mcp` (port 10892) owns the Raspbot / ROSMASTER base.
//! `vla-mcp` (port 11024) owns Wall-OSS, X-VLA, and the weight catalog.
//! RobotCraft does not invent telemetry when those servers answer.

use serde_json::{json, Value};
use std::time::Duration;

pub const DEFAULT_YAHBOOM_URL: &str = "http://127.0.0.1:10892";
pub const DEFAULT_VLA_URL: &str = "http://127.0.0.1:11024";
pub const DEFAULT_ISAAC_URL: &str = "http://127.0.0.1:11049";

const MOTION_OPS: &[&str] = &[
    "forward",
    "backward",
    "turn_left",
    "turn_right",
    "strafe_left",
    "strafe_right",
];

pub fn is_yahboom_id(robot_id: &str) -> bool {
    matches!(
        robot_id.trim().to_ascii_lowercase().as_str(),
        "yahboom" | "yahboom_rover" | "raspbot" | "boomy" | "rosmaster"
    )
}

pub fn is_motion_operation(operation: &str) -> bool {
    MOTION_OPS.contains(&operation.trim().to_ascii_lowercase().as_str())
}

/// `off` or empty disables the peer. Missing env uses the fleet default port.
pub fn normalize_base(raw: Option<&str>, default_url: &str) -> Option<String> {
    match raw.map(str::trim) {
        Some("") | Some("off") | Some("OFF") => None,
        Some(value) => Some(value.trim_end_matches('/').to_string()),
        None => Some(default_url.to_string()),
    }
}

pub fn yahboom_base() -> Option<String> {
    normalize_base(
        std::env::var("ROBOTCRAFT_YAHBOOM_URL").ok().as_deref(),
        DEFAULT_YAHBOOM_URL,
    )
}

pub fn vla_base() -> Option<String> {
    normalize_base(
        std::env::var("ROBOTCRAFT_VLA_URL").ok().as_deref(),
        DEFAULT_VLA_URL,
    )
}

pub fn isaac_base() -> Option<String> {
    normalize_base(
        std::env::var("ROBOTCRAFT_ISAAC_URL").ok().as_deref(),
        DEFAULT_ISAAC_URL,
    )
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(400))
        .timeout_read(Duration::from_millis(1500))
        .build()
}

fn response_json(result: Result<ureq::Response, ureq::Error>) -> Result<Value, String> {
    match result {
        Ok(resp) => resp.into_json::<Value>().map_err(|err| err.to_string()),
        Err(ureq::Error::Status(code, resp)) => {
            let text = resp.into_string().unwrap_or_else(|err| err.to_string());
            match serde_json::from_str::<Value>(&text) {
                Ok(mut body) => {
                    if let Some(obj) = body.as_object_mut() {
                        obj.insert("http_status".to_string(), json!(code));
                    }
                    Ok(body)
                }
                Err(_) => Ok(json!({
                    "success": false,
                    "http_status": code,
                    "body": text
                })),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

pub fn get_json(url: &str) -> Result<Value, String> {
    response_json(agent().get(url).call())
}

pub fn post_json(url: &str, body: &Value, confirm: bool) -> Result<Value, String> {
    let mut req = agent().post(url);
    if confirm {
        req = req.set("X-VLA-Confirm", "1");
    }
    response_json(req.send_json(body))
}

pub fn post_empty(url: &str) -> Result<Value, String> {
    response_json(agent().post(url).send_string(""))
}

fn disabled(peer: &str) -> Value {
    json!({
        "peer": peer,
        "reachable": false,
        "disabled": true,
        "error": "peer disabled via ROBOTCRAFT_*_URL=off"
    })
}

pub fn probe_peers() -> Value {
    let yahboom = match yahboom_base() {
        Some(base) => probe_get("yahboom-mcp", &base, "/api/v1/health"),
        None => disabled("yahboom-mcp"),
    };
    let vla = match vla_base() {
        Some(base) => probe_get("vla-mcp", &base, "/api/health"),
        None => disabled("vla-mcp"),
    };
    let isaac = match isaac_base() {
        Some(base) => probe_get("isaac-mcp", &base, "/api/health"),
        None => disabled("isaac-mcp"),
    };
    json!({
        "success": true,
        "yahboom": yahboom,
        "vla": vla,
        "isaac": isaac
    })
}

fn probe_get(peer: &str, base: &str, path: &str) -> Value {
    let url = format!("{base}{path}");
    match get_json(&url) {
        Ok(body) => json!({
            "peer": peer,
            "reachable": true,
            "base_url": base,
            "body": body
        }),
        Err(err) => json!({
            "peer": peer,
            "reachable": false,
            "base_url": base,
            "error": err
        }),
    }
}

pub fn yahboom_health() -> Result<Value, String> {
    let base = yahboom_base().ok_or_else(|| "yahboom-mcp disabled".to_string())?;
    get_json(&format!("{base}/api/v1/health"))
}

pub fn yahboom_telemetry() -> Result<Value, String> {
    let base = yahboom_base().ok_or_else(|| "yahboom-mcp disabled".to_string())?;
    get_json(&format!("{base}/api/v1/telemetry"))
}

pub fn yahboom_stop_all() -> Result<Value, String> {
    let base = yahboom_base().ok_or_else(|| "yahboom-mcp disabled".to_string())?;
    post_empty(&format!("{base}/api/v1/stop_all"))
}

pub fn yahboom_move(linear: f32, linear_y: f32, angular: f32) -> Result<Value, String> {
    let base = yahboom_base().ok_or_else(|| "yahboom-mcp disabled".to_string())?;
    let url = format!(
        "{base}/api/v1/control/move?linear={linear:.4}&angular={angular:.4}&linear_y={linear_y:.4}"
    );
    post_empty(&url)
}

pub fn yahboom_tool(operation: &str, param1: Option<&str>) -> Result<Value, String> {
    let base = yahboom_base().ok_or_else(|| "yahboom-mcp disabled".to_string())?;
    let body = json!({
        "operation": operation,
        "param1": param1,
    });
    post_json(&format!("{base}/api/v1/control/tool"), &body, false)
}

pub fn vla_status() -> Result<Value, String> {
    let base = vla_base().ok_or_else(|| "vla-mcp disabled".to_string())?;
    get_json(&format!("{base}/api/v1/status"))
}

pub fn vla_tools() -> Result<Value, String> {
    let base = vla_base().ok_or_else(|| "vla-mcp disabled".to_string())?;
    get_json(&format!("{base}/api/v1/tools"))
}

pub fn vla_fleet() -> Result<Value, String> {
    let base = vla_base().ok_or_else(|| "vla-mcp disabled".to_string())?;
    get_json(&format!("{base}/api/v1/fleet"))
}

pub fn vla_episodes(limit: u32) -> Result<Value, String> {
    let base = vla_base().ok_or_else(|| "vla-mcp disabled".to_string())?;
    let limit = limit.clamp(1, 50);
    get_json(&format!("{base}/api/v1/dataset/episodes?limit={limit}"))
}

pub fn isaac_health() -> Result<Value, String> {
    let base = isaac_base().ok_or_else(|| "isaac-mcp disabled".to_string())?;
    get_json(&format!("{base}/api/health"))
}

pub fn isaac_scenes() -> Result<Value, String> {
    let base = isaac_base().ok_or_else(|| "isaac-mcp disabled".to_string())?;
    get_json(&format!("{base}/api/scenes"))
}

/// POST /api/jobs/start. Caller must already have checked confirm.
pub fn isaac_start(scene_name: &str) -> Result<Value, String> {
    let base = isaac_base().ok_or_else(|| "isaac-mcp disabled".to_string())?;
    post_json(
        &format!("{base}/api/jobs/start"),
        &json!({ "scene_name": scene_name, "headless": true }),
        false,
    )
}

/// Read-mostly calls into `POST /api/v1/control/{tool}`.
/// `download` is the only op that sends `X-VLA-Confirm: 1`, and only when `confirm` is set.
pub fn vla_control(tool: &str, body: &Value, confirm: bool) -> Result<Value, String> {
    let base = vla_base().ok_or_else(|| "vla-mcp disabled".to_string())?;
    post_json(&format!("{base}/api/v1/control/{tool}"), body, confirm)
}

/// Compact model rows from `GET /api/v1/status` (`weights.models`).
pub fn summarize_vla_status(status: &Value) -> Value {
    let models = status
        .get("weights")
        .and_then(|w| w.get("models"))
        .and_then(|m| m.as_array())
        .map(|rows| {
            rows.iter()
                .filter_map(|row| {
                    let key = row.get("key")?.as_str()?;
                    Some(json!({
                        "key": key,
                        "repo_id": row.get("repo_id").and_then(|v| v.as_str()).unwrap_or(""),
                        "downloaded": row.get("downloaded").and_then(|v| v.as_bool()).unwrap_or(false),
                        "description": row.get("description").and_then(|v| v.as_str()).unwrap_or("")
                    }))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    json!({
        "server": status.get("server").cloned().unwrap_or(json!("vla-mcp")),
        "version": status.get("version").cloned().unwrap_or(Value::Null),
        "dataset_episodes": status.get("dataset_episodes").cloned().unwrap_or(json!(0)),
        "dataset_root": status.get("dataset_root").cloned().unwrap_or(Value::Null),
        "wall": status.get("wall").cloned().unwrap_or(Value::Null),
        "xvla": status.get("xvla").cloned().unwrap_or(Value::Null),
        "models": models
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_disables_peer_and_default_keeps_port() {
        assert_eq!(normalize_base(Some("off"), DEFAULT_VLA_URL), None);
        assert_eq!(normalize_base(Some(""), DEFAULT_VLA_URL), None);
        assert_eq!(
            normalize_base(None, DEFAULT_YAHBOOM_URL).as_deref(),
            Some(DEFAULT_YAHBOOM_URL)
        );
        assert_eq!(
            normalize_base(Some("http://127.0.0.1:10892/"), DEFAULT_YAHBOOM_URL).as_deref(),
            Some("http://127.0.0.1:10892")
        );
    }

    #[test]
    fn motion_ops_are_the_base_verbs_only() {
        assert!(is_motion_operation("forward"));
        assert!(is_motion_operation("Turn_Left"));
        assert!(!is_motion_operation("read_battery"));
        assert!(!is_motion_operation("stop"));
        assert!(is_yahboom_id("Boomy"));
        assert!(!is_yahboom_id("nori_a3"));
    }

    #[test]
    fn vla_summary_keeps_model_rows() {
        let status = json!({
            "server": "vla-mcp",
            "version": "0.3.0",
            "dataset_episodes": 4,
            "weights": {
                "models": [
                    {"key": "smolvla", "repo_id": "lerobot/smolvla_base", "downloaded": false, "description": "edge"}
                ]
            }
        });
        let summary = summarize_vla_status(&status);
        assert_eq!(summary["dataset_episodes"], 4);
        assert_eq!(summary["models"][0]["key"], "smolvla");
        assert_eq!(summary["models"][0]["downloaded"], false);
    }
}
