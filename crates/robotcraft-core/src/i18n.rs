//! RobotCraft Internationalization (i18n) Engine.
//!
//! Follows the fleet standard: typed locale enums, zero-panic string lookups,
//! multi-language translations (EN, DE, FR, ES, JA, KO, ZH),
//! and robotics kinematics / ROS telemetry terminology localization.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AppLocale {
    #[default]
    En, // English (Global Default)
    De, // German (Industrial Robotics / KUKA native)
    Fr, // French
    Es, // Spanish
    Ja, // Japanese (Fanuc / Yaskawa robotics native)
    Ko, // Korean (Hyundai Robotics native)
    Zh, // Chinese
}

impl AppLocale {
    pub fn all() -> &'static [AppLocale] {
        &[
            AppLocale::En,
            AppLocale::De,
            AppLocale::Fr,
            AppLocale::Es,
            AppLocale::Ja,
            AppLocale::Ko,
            AppLocale::Zh,
        ]
    }

    pub fn code(&self) -> &'static str {
        match self {
            AppLocale::En => "en",
            AppLocale::De => "de",
            AppLocale::Fr => "fr",
            AppLocale::Es => "es",
            AppLocale::Ja => "ja",
            AppLocale::Ko => "ko",
            AppLocale::Zh => "zh",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AppLocale::En => "English (US/UK)",
            AppLocale::De => "Deutsch (DE/AT/CH)",
            AppLocale::Fr => "Français (FR)",
            AppLocale::Es => "Español (ES/LATAM)",
            AppLocale::Ja => "日本語 (JA)",
            AppLocale::Ko => "한국어 (KO)",
            AppLocale::Zh => "中文 (ZH)",
        }
    }

    pub fn flag(&self) -> &'static str {
        match self {
            AppLocale::En => "🇬🇧",
            AppLocale::De => "🇩🇪",
            AppLocale::Fr => "🇫🇷",
            AppLocale::Es => "🇪🇸",
            AppLocale::Ja => "🇯🇵",
            AppLocale::Ko => "🇰🇷",
            AppLocale::Zh => "🇨🇳",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum I18nKey {
    // Navigation Tabs
    TabJointControl,
    TabForwardKinematics,
    TabInverseKinematics,
    TabRosBridge,
    TabTrajectoryPlanner,

    // Controls & Status
    JointAngles,
    EndEffectorPos,
    EmergencyStop,
    HomingAllJoints,
    StatusReady,
}

pub fn t(key: I18nKey, locale: AppLocale) -> &'static str {
    match (key, locale) {
        (I18nKey::TabJointControl, AppLocale::En) => "🦾 Joint Controller",
        (I18nKey::TabJointControl, AppLocale::De) => "🦾 Gelenksteuerung",
        (I18nKey::TabJointControl, AppLocale::Fr) => "🦾 Contrôle des articulations",
        (I18nKey::TabJointControl, AppLocale::Es) => "🦾 Control de articulaciones",
        (I18nKey::TabJointControl, AppLocale::Ja) => "🦾 関節角度コントローラ",
        (I18nKey::TabJointControl, AppLocale::Ko) => "🦾 관절 제어기",
        (I18nKey::TabJointControl, AppLocale::Zh) => "🦾 关节伺服控制器",

        (I18nKey::TabForwardKinematics, AppLocale::En) => "📐 Forward Kinematics (FK)",
        (I18nKey::TabForwardKinematics, AppLocale::De) => "📐 Vorwärtskinematik (FK)",
        (I18nKey::TabForwardKinematics, AppLocale::Fr) => "📐 Cinématique directe (FK)",
        (I18nKey::TabForwardKinematics, AppLocale::Es) => "📐 Cinemática directa (FK)",
        (I18nKey::TabForwardKinematics, AppLocale::Ja) => "📐 順運動学 (FK計算)",
        (I18nKey::TabForwardKinematics, AppLocale::Ko) => "📐 순기구학 (FK)",
        (I18nKey::TabForwardKinematics, AppLocale::Zh) => "📐 正向运动学 (FK)",

        (I18nKey::TabInverseKinematics, AppLocale::En) => "🎯 Inverse Kinematics (IK)",
        (I18nKey::TabInverseKinematics, AppLocale::De) => "🎯 Inverse Kinematik (IK)",
        (I18nKey::TabInverseKinematics, AppLocale::Fr) => "🎯 Cinématique inverse (IK)",
        (I18nKey::TabInverseKinematics, AppLocale::Es) => "🎯 Cinemática inversa (IK)",
        (I18nKey::TabInverseKinematics, AppLocale::Ja) => "🎯 逆運動学 (IKソルバー)",
        (I18nKey::TabInverseKinematics, AppLocale::Ko) => "🎯 역기구학 (IK)",
        (I18nKey::TabInverseKinematics, AppLocale::Zh) => "🎯 逆向运动学 (IK)",

        (I18nKey::TabRosBridge, AppLocale::En) => "🌐 ROS2 DDS Bridge",
        (I18nKey::TabRosBridge, AppLocale::De) => "🌐 ROS2 DDS Brücke",
        (I18nKey::TabRosBridge, AppLocale::Fr) => "🌐 Pont ROS2 DDS",
        (I18nKey::TabRosBridge, AppLocale::Es) => "🌐 Puente ROS2 DDS",
        (I18nKey::TabRosBridge, AppLocale::Ja) => "🌐 ROS2 通信ブリッジ",
        (I18nKey::TabRosBridge, AppLocale::Ko) => "🌐 ROS2 DDS 브릿지",
        (I18nKey::TabRosBridge, AppLocale::Zh) => "🌐 ROS2 DDS 节点桥接",

        (I18nKey::TabTrajectoryPlanner, AppLocale::En) => "⏱️ Trajectory Planner",
        (I18nKey::TabTrajectoryPlanner, AppLocale::De) => "⏱️ Trajektorienplaner",
        (I18nKey::TabTrajectoryPlanner, AppLocale::Fr) => "⏱️ Planificateur de trajectoire",
        (I18nKey::TabTrajectoryPlanner, AppLocale::Es) => "⏱️ Planificador de trayectoria",
        (I18nKey::TabTrajectoryPlanner, AppLocale::Ja) => "⏱️ 軌道生成・補間計画",
        (I18nKey::TabTrajectoryPlanner, AppLocale::Ko) => "⏱️ 궤적 계획기",
        (I18nKey::TabTrajectoryPlanner, AppLocale::Zh) => "⏱️ 机械臂运动轨迹规划",

        (I18nKey::JointAngles, AppLocale::En) => "Joint Coordinates (rad / deg)",
        (I18nKey::JointAngles, AppLocale::De) => "Gelenkkoordinaten (rad / deg)",
        (I18nKey::JointAngles, AppLocale::Fr) => "Coordonnées articulaires (rad / deg)",
        (I18nKey::JointAngles, AppLocale::Es) => "Coordenadas articuladas (rad / deg)",
        (I18nKey::JointAngles, AppLocale::Ja) => "各軸角度座標 (rad / deg)",
        (I18nKey::JointAngles, AppLocale::Ko) => "관절 축 좌표 (rad / deg)",
        (I18nKey::JointAngles, AppLocale::Zh) => "关节轴角坐标 (rad / deg)",

        (I18nKey::EndEffectorPos, AppLocale::En) => "End-Effector Pose (X, Y, Z, Roll, Pitch, Yaw)",
        (I18nKey::EndEffectorPos, AppLocale::De) => "Endeffektor-Pose (X, Y, Z, Roll, Pitch, Yaw)",
        (I18nKey::EndEffectorPos, AppLocale::Fr) => "Pose de l'effecteur (X, Y, Z, R, P, Y)",
        (I18nKey::EndEffectorPos, AppLocale::Es) => "Pose del efector (X, Y, Z, R, P, Y)",
        (I18nKey::EndEffectorPos, AppLocale::Ja) => "手先位置姿勢 (X, Y, Z, Roll, Pitch, Yaw)",
        (I18nKey::EndEffectorPos, AppLocale::Ko) => "엔드이펙터 자세 (X, Y, Z, R, P, Y)",
        (I18nKey::EndEffectorPos, AppLocale::Zh) => "末端执行器空间位姿 (X, Y, Z, 姿态角)",

        (I18nKey::EmergencyStop, AppLocale::En) => "🛑 EMERGENCY STOP (E-STOP)",
        (I18nKey::EmergencyStop, AppLocale::De) => "🛑 NOT-AUS (E-STOP)",
        (I18nKey::EmergencyStop, AppLocale::Fr) => "🛑 ARRÊT D'URGENCE (E-STOP)",
        (I18nKey::EmergencyStop, AppLocale::Es) => "🛑 PARADA DE EMERGENCIA",
        (I18nKey::EmergencyStop, AppLocale::Ja) => "🛑 非常停止 (E-STOP)",
        (I18nKey::EmergencyStop, AppLocale::Ko) => "🛑 비상 정지 (E-STOP)",
        (I18nKey::EmergencyStop, AppLocale::Zh) => "🛑 紧急停止 (急停断电)",

        (I18nKey::HomingAllJoints, AppLocale::En) => "Home All Joints",
        (I18nKey::HomingAllJoints, AppLocale::De) => "Alle Gelenke referenzieren",
        (I18nKey::HomingAllJoints, AppLocale::Fr) => "Retour origine tous axes",
        (I18nKey::HomingAllJoints, AppLocale::Es) => "Posición inicial de ejes",
        (I18nKey::HomingAllJoints, AppLocale::Ja) => "全軸原点復帰 (ホーミング)",
        (I18nKey::HomingAllJoints, AppLocale::Ko) => "전축 원점 복귀",
        (I18nKey::HomingAllJoints, AppLocale::Zh) => "全关节回原点",

        (I18nKey::StatusReady, AppLocale::En) => "RobotCraft Hardware Abstraction Layer Ready",
        (I18nKey::StatusReady, AppLocale::De) => "RobotCraft Hardware-Abstraktionsschicht bereit",
        (I18nKey::StatusReady, AppLocale::Fr) => "Couche d'abstraction matérielle RobotCraft prête",
        (I18nKey::StatusReady, AppLocale::Es) => "Capa de abstracción de hardware RobotCraft lista",
        (I18nKey::StatusReady, AppLocale::Ja) => "RobotCraft HAL 制御レイヤー準備完了",
        (I18nKey::StatusReady, AppLocale::Ko) => "RobotCraft HAL 제어 계층 준비완료",
        (I18nKey::StatusReady, AppLocale::Zh) => "RobotCraft 机器人硬件抽象层就绪",
    }
}
