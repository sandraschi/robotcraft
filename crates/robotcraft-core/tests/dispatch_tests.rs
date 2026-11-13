//! Integration and dispatch verification tests for RobotCraft.
use databasecraft_store::DatabaseStore;
use robotcraft_core::{dispatch, Command};

#[tokio::test]
async fn test_robotcraft_status_and_control_port() -> Result<(), Box<dyn std::error::Error>> {
    let store = DatabaseStore::open_in_memory()?;
    let res = dispatch(Command::Status, &store).await;
    assert_eq!(res["success"], true);
    assert_eq!(res["app"], "robotcraft");
    assert_eq!(res["control_port"], 14116);
    Ok(())
}

#[tokio::test]
async fn test_robotcraft_telemetry_and_kinematics() -> Result<(), Box<dyn std::error::Error>> {
    let store = DatabaseStore::open_in_memory()?;

    // 1. Query telemetry for Unitree quadruped
    let telem_res = dispatch(
        Command::GetTelemetry {
            robot_id: "unitree_go2".to_string(),
        },
        &store,
    )
    .await;
    assert_eq!(telem_res["success"], true);
    assert_eq!(telem_res["telemetry"]["hardware_id"], "UNITREE-GO2-PRO-881");
    assert!(
        telem_res["telemetry"]["battery_percentage"]
            .as_f64()
            .unwrap_or(0.0)
            > 80.0
    );

    // 2. Query 6-DOF Robot Model
    let model_res = dispatch(Command::GetRobotModel, &store).await;
    assert_eq!(model_res["success"], true);
    assert_eq!(model_res["model"]["dof"], 6);

    // 3. Forward Kinematics calculation
    let fk_res = dispatch(
        Command::CalculateFk {
            joint_angles: vec![0.0, 0.35, 1.20, -0.45, 0.0, 0.10],
        },
        &store,
    )
    .await;
    assert_eq!(fk_res["success"], true);
    let xyz = fk_res["end_effector_xyz"].as_array().ok_or("missing xyz")?;
    assert_eq!(xyz.len(), 3);

    // 4. Reachability boundary test
    let reach_ok = dispatch(
        Command::ValidatePose {
            x: 0.3,
            y: 0.2,
            z: 0.4,
        },
        &store,
    )
    .await;
    assert_eq!(reach_ok["reachable"], true);

    let reach_exceed = dispatch(
        Command::ValidatePose {
            x: 2.5,
            y: 3.0,
            z: 1.0,
        },
        &store,
    )
    .await;
    assert_eq!(reach_exceed["reachable"], false);

    // 5. Emergency Stop
    let estop = dispatch(
        Command::EmergencyStop {
            reason: "Safety perimeter breached".to_string(),
        },
        &store,
    )
    .await;
    assert_eq!(estop["emergency_stop"], "ACTIVATED");
    assert_eq!(estop["motors_disabled"], true);

    // 6. Test Saschibot, Nori A3, and Noetix Bumi CAD plans and models
    let saschi_cad = dispatch(
        Command::GetRobotCadPlan {
            robot_id: "saschibot".to_string(),
        },
        &store,
    )
    .await;
    assert_eq!(saschi_cad["success"], true);
    assert_eq!(saschi_cad["cad_type"], "URDF_DXF_SVG_STL");

    let nori_cad = dispatch(
        Command::GetRobotCadPlan {
            robot_id: "nori_a3".to_string(),
        },
        &store,
    )
    .await;
    assert_eq!(nori_cad["success"], true);
    assert_eq!(nori_cad["cad_type"], "URDF_STEP_STL_GLB");

    let bumi_cad = dispatch(
        Command::GetRobotCadPlan {
            robot_id: "noetix_bumi".to_string(),
        },
        &store,
    )
    .await;
    assert_eq!(bumi_cad["success"], true);
    assert_eq!(bumi_cad["cad_type"], "VENDOR_SPEC_KINEMATIC_TOPOLOGY");

    // 7. Test AI Behavior Workflows
    let wf_wave = dispatch(
        Command::RunAiBehaviorWorkflow {
            robot_id: "saschibot".to_string(),
            workflow_type: "wave_and_greet".to_string(),
        },
        &store,
    )
    .await;
    assert_eq!(wf_wave["success"], true);
    assert_eq!(wf_wave["virtual_space_simulation"], true);

    let wf_explore = dispatch(
        Command::RunAiBehaviorWorkflow {
            robot_id: "nori_a3".to_string(),
            workflow_type: "explore_and_map".to_string(),
        },
        &store,
    )
    .await;
    assert_eq!(wf_explore["success"], true);

    let wf_stand = dispatch(
        Command::RunAiBehaviorWorkflow {
            robot_id: "noetix_bumi".to_string(),
            workflow_type: "stand_and_balance".to_string(),
        },
        &store,
    )
    .await;
    assert_eq!(wf_stand["success"], true);

    Ok(())
}

#[tokio::test]
async fn yahboom_and_vla_gates_do_not_call_peers() -> Result<(), Box<dyn std::error::Error>> {
    let store = DatabaseStore::open_in_memory()?;

    let held = dispatch(
        Command::TeleopTwist {
            linear_x: 0.2,
            linear_y: 0.0,
            angular_z: 0.0,
            robot_id: "yahboom_rover".to_string(),
            confirm: false,
        },
        &store,
    )
    .await;
    assert_eq!(held["dispatched"], false);
    assert_eq!(held["confirm_required"], true);
    assert_eq!(held["source"], "yahboom-mcp");

    let blocked = dispatch(
        Command::YahboomCall {
            operation: "forward".to_string(),
            param1: None,
            confirm: false,
        },
        &store,
    )
    .await;
    assert_eq!(blocked["confirm_required"], true);
    assert_eq!(blocked["dispatched"], false);

    let sim = dispatch(
        Command::TeleopTwist {
            linear_x: 0.2,
            linear_y: 0.0,
            angular_z: 0.0,
            robot_id: "saschibot".to_string(),
            confirm: true,
        },
        &store,
    )
    .await;
    assert_eq!(sim["simulated"], true);
    assert_eq!(sim["dispatched"], false);

    let rejected = dispatch(
        Command::VlaCall {
            tool: "vla_training".to_string(),
            operation: "launch_co_train".to_string(),
            task_hint: None,
            target: None,
            model_key: None,
            confirm: true,
        },
        &store,
    )
    .await;
    assert_eq!(rejected["success"], false);
    assert_eq!(rejected["dispatched"], false);

    let download = dispatch(
        Command::VlaCall {
            tool: "vla_weights".to_string(),
            operation: "download".to_string(),
            task_hint: None,
            target: None,
            model_key: Some("smolvla".to_string()),
            confirm: false,
        },
        &store,
    )
    .await;
    assert_eq!(download["confirm_required"], true);

    let model = dispatch(
        Command::GetRobotModelById {
            robot_id: "yahboom_rover".to_string(),
        },
        &store,
    )
    .await;
    assert_eq!(model["model"]["dof"], 4);
    assert_eq!(model["model"]["name"], "Yahboom-ROSMASTER-X3");

    let held_isaac = dispatch(
        Command::IsaacStart {
            scene_name: "do_not_start".to_string(),
            confirm: false,
        },
        &store,
    )
    .await;
    assert_eq!(held_isaac["confirm_required"], true);
    assert_eq!(held_isaac["dispatched"], false);

    let posed = dispatch(
        Command::UrdfPose {
            path: String::new(),
            joints: vec![("left_shoulder_pitch_joint".to_string(), 0.4)],
        },
        &store,
    )
    .await;
    assert_eq!(posed["success"], true);
    assert_eq!(posed["robot"], "nori");
    assert!(posed["link_count"].as_u64().unwrap_or(0) > 10);

    Ok(())
}
