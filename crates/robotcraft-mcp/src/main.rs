//! RobotCraft MCP Server & Universal CLI.
//!
//! Follows the MCP 2024-11-05 standard: stdout is 100% JSON-RPC, diagnostics go to stderr.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use clap::{Parser, Subcommand};
use databasecraft_store::DatabaseStore;
use robotcraft_core::{dispatch, Command, APP_VERSION, CONTROL_PORT};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use tokio::runtime::Runtime;

#[derive(Parser)]
#[command(name = "robotcraft-cli")]
#[command(about = "RobotCraft: Embodied Intelligence Studio & Universal Robotics CLI", version = APP_VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Run as MCP 2024-11-05 stdio server
    #[arg(long)]
    mcp: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Check RobotCraft status and active hardware drivers
    Status,
    /// List supported robot hardware platforms
    Robots,
    /// Query real-time robot telemetry (battery, motors, velocity)
    Telemetry {
        #[arg(default_value = "unitree_go2")]
        robot_id: String,
    },
    /// Send velocity twist command to robot base
    Teleop {
        #[arg(long, default_value_t = 0.5)]
        x: f32,
        #[arg(long, default_value_t = 0.0)]
        y: f32,
        #[arg(long, default_value_t = 0.0)]
        z: f32,
    },
    /// Inspect robot kinematic model and joint bounds
    Model,
    /// Compute forward kinematics from joint angles
    Fk {
        #[arg(num_args = 1..)]
        angles: Vec<f32>,
    },
    /// Trigger immediate hardware-enforced emergency stop
    Estop {
        #[arg(long, default_value = "Manual CLI trigger")]
        reason: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let rt = Runtime::new().unwrap_or_else(|e| {
        eprintln!("Failed to create tokio runtime: {e}");
        std::process::exit(1);
    });

    let store = DatabaseStore::open_in_memory().unwrap_or_else(|e| {
        eprintln!("Failed to open in-memory store: {e}");
        std::process::exit(1);
    });

    if cli.mcp {
        run_mcp_stdio(&rt, &store);
        return;
    }

    match cli.command {
        Some(Commands::Status) => {
            let res = rt.block_on(dispatch(Command::Status, &store));
            println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
        }
        Some(Commands::Robots) => {
            let res = rt.block_on(dispatch(Command::ListRobots, &store));
            println!("{}", serde_json::to_string_pretty(&res["available_robots"]).unwrap_or_default());
        }
        Some(Commands::Telemetry { robot_id }) => {
            let res = rt.block_on(dispatch(Command::GetTelemetry { robot_id }, &store));
            println!("{}", serde_json::to_string_pretty(&res["telemetry"]).unwrap_or_default());
        }
        Some(Commands::Teleop { x, y, z }) => {
            let res = rt.block_on(dispatch(Command::TeleopTwist { linear_x: x, linear_y: y, angular_z: z }, &store));
            println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
        }
        Some(Commands::Model) => {
            let res = rt.block_on(dispatch(Command::GetRobotModel, &store));
            println!("{}", serde_json::to_string_pretty(&res["model"]).unwrap_or_default());
        }
        Some(Commands::Fk { angles }) => {
            let res = rt.block_on(dispatch(Command::CalculateFk { joint_angles: angles }, &store));
            println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
        }
        Some(Commands::Estop { reason }) => {
            let res = rt.block_on(dispatch(Command::EmergencyStop { reason }, &store));
            println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
        }
        None => {
            println!("RobotCraft CLI v{APP_VERSION} on Port {CONTROL_PORT}");
            println!("Use --help to view available commands or --mcp to start JSON-RPC server.");
        }
    }
}

fn run_mcp_stdio(rt: &Runtime, store: &DatabaseStore) {
    eprintln!("[robotcraft-mcp] Starting MCP stdio loop on port {CONTROL_PORT}");
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

        let resp = match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "robotcraft-mcp", "version": APP_VERSION }
                }
            }),
            "tools/list" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": [
                        { "name": "robot_status", "description": "Check RobotCraft status and active drivers", "inputSchema": { "type": "object", "properties": {} } },
                        { "name": "robot_list", "description": "List supported robot hardware platforms", "inputSchema": { "type": "object", "properties": {} } },
                        {
                            "name": "robot_telemetry",
                            "description": "Fetch real-time robot battery, motors, and velocity metrics",
                            "inputSchema": {
                                "type": "object",
                                "properties": { "robot_id": { "type": "string" } },
                                "required": ["robot_id"]
                            }
                        },
                        {
                            "name": "robot_teleop",
                            "description": "Dispatch linear and angular twist velocities to robot base",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "linear_x": { "type": "number" },
                                    "linear_y": { "type": "number" },
                                    "angular_z": { "type": "number" }
                                }
                            }
                        },
                        {
                            "name": "robot_estop",
                            "description": "Activate immediate hardware emergency stop",
                            "inputSchema": {
                                "type": "object",
                                "properties": { "reason": { "type": "string" } },
                                "required": ["reason"]
                            }
                        }
                    ]
                }
            }),
            "tools/call" => {
                let params = req.get("params").cloned().unwrap_or(Value::Null);
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                let tool_res = match tool_name {
                    "robot_status" => rt.block_on(dispatch(Command::Status, store)),
                    "robot_list" => rt.block_on(dispatch(Command::ListRobots, store)),
                    "robot_telemetry" => {
                        let id = args["robot_id"].as_str().unwrap_or("unitree_go2").to_string();
                        rt.block_on(dispatch(Command::GetTelemetry { robot_id: id }, store))
                    }
                    "robot_teleop" => {
                        let lx = args["linear_x"].as_f64().unwrap_or(0.0) as f32;
                        let ly = args["linear_y"].as_f64().unwrap_or(0.0) as f32;
                        let az = args["angular_z"].as_f64().unwrap_or(0.0) as f32;
                        rt.block_on(dispatch(Command::TeleopTwist { linear_x: lx, linear_y: ly, angular_z: az }, store))
                    }
                    "robot_estop" => {
                        let r = args["reason"].as_str().unwrap_or("MCP emergency stop").to_string();
                        rt.block_on(dispatch(Command::EmergencyStop { reason: r }, store))
                    }
                    unknown => json!({ "success": false, "error": format!("Unknown tool '{unknown}'") })
                };

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "content": [{ "type": "text", "text": tool_res.to_string() }] }
                })
            }
            _ => json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": "Method not found" } })
        };

        let _ = writeln!(stdout, "{}", resp.to_string());
        let _ = stdout.flush();
    }
}
